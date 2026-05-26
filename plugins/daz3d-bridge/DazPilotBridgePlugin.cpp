#include "DazPilotBridgePlugin.h"
#include "dzplugin.h"
#include "dzundostack.h"
#include "dzexportmgr.h"
#include "dzfloatproperty.h"
#include "dzscript.h"
#include "dzbox3.h"
#include "dzvec3.h"
#include "dzfileiosettings.h"
#include "dzexporter.h"
#include "DazPilotPhyModifier.h"
#include "Log.h"
#include "JsonUtil.h"
#include "DazPilotExportOptions.h"
#include <QtCore/QBuffer>
#include <QtCore/QByteArray>
#include <cstdlib>

#include <atomic>
#include <sstream>
#include <string>
#include <thread>
#include <mutex>
#include <condition_variable>
#include <cstring>
#include <QtCore/QCoreApplication>
#include <QtCore/QEvent>

#include "dzpane.h"
#include <QtGui/QLabel>
#include <QtGui/QVBoxLayout>
#include <QtGui/QTextEdit>

static DazPilotBridgeState g_state = {nullptr, QList<QTcpSocket*>(), "127.0.0.1", 8765, ""};
static std::atomic<bool> g_serverRunning(false);

class DazPilotPane : public DzPane {
    Q_OBJECT
public:
    DazPilotPane() : DzPane("DazPilot Bridge") {
        QVBoxLayout* layout = new QVBoxLayout(this);
        
        QLabel* titleLabel = new QLabel("DazPilot AI Bridge", this);
        QFont titleFont = titleLabel->font();
        titleFont.setBold(true);
        titleFont.setPointSize(12);
        titleLabel->setFont(titleFont);
        layout->addWidget(titleLabel);

        m_statusLabel = new QLabel("Status: Disconnected", this);
        layout->addWidget(m_statusLabel);

        layout->addWidget(new QLabel("Last Commands:", this));
        m_logArea = new QTextEdit(this);
        m_logArea->setReadOnly(true);
        layout->addWidget(m_logArea);

        QTimer* timer = new QTimer(this);
        connect(timer, SIGNAL(timeout()), this, SLOT(updateStatus()));
        timer->start(1000);
    }

public slots:
    void updateStatus() {
        if (g_serverRunning.load()) {
            m_statusLabel->setText(QString("Status: Listening on %1:%2")
                .arg(g_state.host).arg(g_state.port));
            m_statusLabel->setStyleSheet("color: #00ff00;");
        } else {
            m_statusLabel->setText("Status: Stopped");
            m_statusLabel->setStyleSheet("color: #ff0000;");
        }
    }

private:
    QLabel* m_statusLabel;
    QTextEdit* m_logArea;
};

#include "DazPilotBridgePlugin.moc"

#ifdef _WIN32
using BridgeSocket = SOCKET;
static constexpr BridgeSocket INVALID_BRIDGE_SOCKET = INVALID_SOCKET;
static void CloseBridgeSocket(BridgeSocket socket) { closesocket(socket); }
#else
#include <arpa/inet.h>
#include <netinet/in.h>
#include <sys/socket.h>
#include <unistd.h>
using BridgeSocket = int;
static constexpr BridgeSocket INVALID_BRIDGE_SOCKET = -1;
static void CloseBridgeSocket(BridgeSocket socket) { close(socket); }
#endif

static std::thread g_serverThread;
static BridgeSocket g_listenSocket = INVALID_BRIDGE_SOCKET;

static std::string JsonEscape(const QString& value);

class RunScriptEvent : public QEvent {
public:
    static const QEvent::Type EventType = static_cast<QEvent::Type>(QEvent::User + 100);

    QString script;
    QString argsJson;
    QString* resultOut;
    std::mutex* mtx;
    std::condition_variable* cv;
    bool* done;

    RunScriptEvent(const QString& s, const QString& a, QString* r, std::mutex* m, std::condition_variable* c, bool* d)
        : QEvent(EventType), script(s), argsJson(a), resultOut(r), mtx(m), cv(c), done(d) {}
};

class ExportSceneEvent : public QEvent {
public:
    static const QEvent::Type EventType = static_cast<QEvent::Type>(QEvent::User + 101);

    QString path;
    QString settingsJson;
    QString* resultOut;
    std::mutex* mtx;
    std::condition_variable* cv;
    bool* done;

    ExportSceneEvent(const QString& p, const QString& s, QString* r, std::mutex* m, std::condition_variable* c, bool* d)
        : QEvent(EventType), path(p), settingsJson(s), resultOut(r), mtx(m), cv(c), done(d) {}
};

class ScriptExecutor : public QObject {
public:
    ScriptExecutor(QObject* parent = nullptr) : QObject(parent) {}

protected:
    void customEvent(QEvent* e) override {
        if (e->type() == RunScriptEvent::EventType) {
            RunScriptEvent* rse = static_cast<RunScriptEvent*>(e);
            
            std::string escapedArgs = JsonEscape(rse->argsJson.isEmpty() ? "{}" : rse->argsJson);
            QString fullScript = QString(
                "var __args = JSON.parse('%1');\n"
                "(function(){\n%2\n}).call(null, __args);\n"
            ).arg(escapedArgs.c_str()).arg(rse->script);

            DzScript dzScript;
            dzScript.addLine(fullScript);
            
            if (dzScript.execute()) {
                *rse->resultOut = "{\"success\":true}";
            } else {
                *rse->resultOut = "{\"success\":false}";
            }

            {
                std::lock_guard<std::mutex> lock(*rse->mtx);
                *rse->done = true;
            }
            rse->cv->notify_one();
        } else if (e->type() == ExportSceneEvent::EventType) {
            ExportSceneEvent* ese = static_cast<ExportSceneEvent*>(e);
            
            bool success = false;
            if (dzApp && dzApp->getExportMgr()) {
                DzExportMgr* exportMgr = dzApp->getExportMgr();
                int exporterIndex = exportMgr->findExporterIndex(ese->path);
                
                if (exporterIndex >= 0) {
                    DzExporter* exporter = exportMgr->getExporter(exporterIndex);
                    if (exporter) {
                        DazPilotExportOptions opts = DazPilotExportOptions::fromJson(ese->settingsJson);
                        DzFileIOSettings ioSettings;
                        opts.applyToSettings(&ioSettings);

                        DzError err = exporter->writeFile(ese->path, &ioSettings);
                        if (err == DZ_NO_ERROR) {
                            *ese->resultOut = "{\"success\":true}";
                            success = true;
                        }
                    }
                }
            }
            
            if (!success) {
                *ese->resultOut = "{\"success\":false}";
            }

            {
                std::lock_guard<std::mutex> lock(*ese->mtx);
                *ese->done = true;
            }
            ese->cv->notify_one();
        }
    }
};

static ScriptExecutor* g_scriptExecutor = nullptr;

static std::string JsonEscape(const QString& value) {
    std::string input = value.toUtf8().constData();
    std::ostringstream out;
    for (char ch : input) {
        switch (ch) {
        case '\\': out << "\\\\"; break;
        case '"': out << "\\\""; break;
        case '\n': out << "\\n"; break;
        case '\r': out << "\\r"; break;
        case '\t': out << "\\t"; break;
        default: out << ch; break;
        }
    }
    return out.str();
}

static std::string JsonEscape(const char* value) {
    return JsonEscape(QString(value ? value : ""));
}

static QString ExtractJsonValue(const std::string& json, const std::string& key) {
    std::string needle = "\"" + key + "\"";
    size_t keyPos = json.find(needle);
    if (keyPos == std::string::npos) return "";
    size_t colon = json.find(':', keyPos + needle.size());
    if (colon == std::string::npos) return "";
    
    size_t valStart = colon + 1;
    while (valStart < json.size() && (json[valStart] == ' ' || json[valStart] == '\t' || json[valStart] == '\r' || json[valStart] == '\n')) {
        valStart++;
    }
    if (valStart >= json.size()) return "";
    
    char firstChar = json[valStart];
    if (firstChar == '"') {
        std::string raw;
        bool escaped = false;
        size_t pos = valStart + 1;
        for (; pos < json.size(); ++pos) {
            char ch = json[pos];
            if (escaped) {
                switch (ch) {
                    case '"': raw += '"'; break;
                    case '\\': raw += '\\'; break;
                    case '/': raw += '/'; break;
                    case 'b': raw += '\b'; break;
                    case 'f': raw += '\f'; break;
                    case 'n': raw += '\n'; break;
                    case 'r': raw += '\r'; break;
                    case 't': raw += '\t'; break;
                    case 'u': {
                        if (pos + 4 < json.size()) {
                            std::string hex = json.substr(pos + 1, 4);
                            unsigned int codepoint;
                            std::istringstream(hex) >> std::hex >> codepoint;
                            if (codepoint <= 0x7F) {
                                raw += static_cast<char>(codepoint);
                            } else if (codepoint <= 0x7FF) {
                                raw += static_cast<char>(0xC0 | (codepoint >> 6));
                                raw += static_cast<char>(0x80 | (codepoint & 0x3F));
                            } else {
                                raw += static_cast<char>(0xE0 | (codepoint >> 12));
                                raw += static_cast<char>(0x80 | ((codepoint >> 6) & 0x3F));
                                raw += static_cast<char>(0x80 | (codepoint & 0x3F));
                            }
                            pos += 4;
                        }
                        break;
                    }
                    default: raw += ch; break;
                }
                escaped = false;
                continue;
            }
            if (ch == '\\') {
                escaped = true;
                continue;
            }
            if (ch == '"') break;
            raw += ch;
        }
        return QString::fromUtf8(raw.c_str());
    } else if (firstChar == '{' || firstChar == '[') {
        char closeChar = (firstChar == '{') ? '}' : ']';
        int depth = 1;
        size_t idx = valStart + 1;
        bool inString = false;
        bool escaped = false;
        for (; idx < json.size(); ++idx) {
            char ch = json[idx];
            if (escaped) {
                escaped = false;
                continue;
            }
            if (ch == '\\' && inString) {
                escaped = true;
                continue;
            }
            if (ch == '"') {
                inString = !inString;
                continue;
            }
            if (!inString) {
                if (ch == firstChar) {
                    depth++;
                } else if (ch == closeChar) {
                    depth--;
                    if (depth == 0) {
                        idx++;
                        break;
                    }
                }
            }
        }
        if (depth != 0 || idx > json.size()) return "";
        return QString::fromUtf8(json.substr(valStart, idx - valStart).c_str());
    } else {
        size_t idx = valStart;
        for (; idx < json.size(); ++idx) {
            char ch = json[idx];
            if (ch == ',' || ch == '}' || ch == ']' || ch == ' ' || ch == '\t' || ch == '\r' || ch == '\n') {
                break;
            }
        }
        return QString::fromUtf8(json.substr(valStart, idx - valStart).c_str());
    }
}

static QString ExtractJsonString(const std::string& json, const std::string& key) {
    return ExtractJsonValue(json, key);
}

static QString ExtractArgString(const std::string& json, const std::string& key) {
    return ExtractJsonValue(json, key);
}

static std::string OkResponse(const QString& id, const std::string& data) {
    std::ostringstream oss;
    oss << "{\"id\":\"" << JsonEscape(id) << "\",\"status\":\"ok\",\"data\":" << data << "}\n";
    return oss.str();
}

static std::string ErrorResponse(const QString& id, const QString& error) {
    std::ostringstream oss;
    oss << "{\"id\":\"" << JsonEscape(id) << "\",\"status\":\"error\",\"error\":\"" << JsonEscape(error) << "\"}\n";
    return oss.str();
}

static QString NodeType(DzNode* node) {
    if (qobject_cast<DzFigure*>(node)) return "Figure";
    if (qobject_cast<DzLight*>(node)) return "Light";
    if (qobject_cast<DzCamera*>(node)) return "Camera";
    return "Node";
}

static std::string SceneInfoData() {
    if (!dzScene) return "{\"available\":false}";

    QString filename = dzScene->getFilename();
    if (filename.isEmpty()) filename = "Untitled Scene";

    std::ostringstream oss;
    oss << "{";
    oss << "\"scene\":\"" << JsonEscape(filename) << "\",";
    oss << "\"nodes\":" << dzScene->getNumNodes() << ",";
    oss << "\"lights\":" << dzScene->getNumLights() << ",";
    oss << "\"cameras\":" << dzScene->getNumCameras();
    DzNode* selected = dzScene->getPrimarySelection();
    if (selected) {
        oss << ",\"selected\":\"" << JsonEscape(selected->getName()) << "\"";
    }
    oss << "}";
    return oss.str();
}

static std::string NodeListData(bool selectedOnly) {
    std::ostringstream oss;
    oss << "{\"nodes\":[";
    if (dzScene) {
        bool first = true;
        DzNodeListIterator iter = selectedOnly ? dzScene->selectedNodeListIterator() : dzScene->nodeListIterator();
        while (iter.hasNext()) {
            DzNode* node = iter.next();
            if (!node) continue;
            if (!first) oss << ",";
            first = false;
            oss << "{";
            oss << "\"id\":\"" << JsonEscape(node->getName()) << "\",";
            oss << "\"name\":\"" << JsonEscape(node->getName()) << "\",";
            oss << "\"type\":\"" << JsonEscape(NodeType(node)) << "\",";
            oss << "\"selected\":" << (node->isSelected() ? "true" : "false");
            oss << "}";
        }
    }
    oss << "]}";
    return oss.str();
}

static std::string CamerasData() {
    std::ostringstream oss;
    oss << "{\"cameras\":[";
    if (dzScene) {
        bool first = true;
        for (int i = 0; i < dzScene->getNumCameras(); i++) {
            DzCamera* camera = dzScene->getCamera(i);
            if (!camera) continue;
            if (!first) oss << ",";
            first = false;
            oss << "{\"id\":\"" << JsonEscape(camera->getName()) << "\",";
            oss << "\"name\":\"" << JsonEscape(camera->getName()) << "\"}";
        }
    }
    oss << "]}";
    return oss.str();
}

static bool SelectNodeInDaz(const QString& nodeId) {
    if (!dzScene || nodeId.isEmpty()) return false;
    DzNode* node = dzScene->findNode(nodeId);
    if (!node) return false;
    dzScene->setPrimarySelection(node);
    return true;
}

static bool OpenContentFile(const QString& path, bool merge) {
    if (!dzApp || path.isEmpty()) return false;
    DzContentMgr* contentMgr = dzApp->getContentMgr();
    if (!contentMgr) return false;
    return contentMgr->openFile(path, merge);
}

static bool ImportContentFile(const QString& path) {
    if (!dzApp || path.isEmpty()) return false;
    DzContentMgr* contentMgr = dzApp->getContentMgr();
    if (!contentMgr) return false;
    return contentMgr->importFile(path);
}

static std::string CaptureActiveViewport(const QString& path) {
    if (!dzApp) return "";
    DzMainWindow* mainWindow = dzApp->getInterface();
    if (!mainWindow) return "";
    DzViewportMgr* viewportMgr = mainWindow->getViewportMgr();
    if (!viewportMgr) return "";
    DzViewport* viewport = viewportMgr->getActiveViewport();
    if (!viewport) return "";
    Dz3DViewport* viewport3d = viewport->get3DViewport();
    if (!viewport3d) return "";
    QImage image = viewport3d->captureImage();
    if (image.isNull()) return "";

    if (path == "stream") {
        QByteArray ba;
        QBuffer buffer(&ba);
        buffer.open(QIODevice::WriteOnly);
        image.save(&buffer, "PNG");
        return ba.toBase64().constData();
    } else if (!path.isEmpty()) {
        if (image.save(path)) return path.toStdString();
    }
    return "";
}

static bool BeginUndoBatch() {
    if (!dzUndoStack) return false;
    dzUndoStack->beginHold();
    return true;
}

static bool AcceptUndoBatch(const QString& caption) {
    if (!dzUndoStack) return false;
    dzUndoStack->accept(caption);
    return true;
}

static bool CancelUndoBatch() {
    if (!dzUndoStack) return false;
    dzUndoStack->cancel();
    return true;
}

static bool AddNode(const QString& type, const QString& name) {
    if (!dzScene) return false;
    DzNode* newNode = nullptr;
    if (type == "point_light") newNode = new DzPointLight();
    else if (type == "spot_light") newNode = new DzSpotLight();
    else if (type == "distant_light") newNode = new DzDistantLight();
    else if (type == "camera") newNode = new DzBasicCamera();
    else if (type == "null") newNode = new DzNode();
    
    if (!newNode) return false;
    
    if (!name.isEmpty()) newNode->setName(name);
    dzScene->addNode(newNode);
    return true;
}

static bool SetProperty(const QString& nodeId, const QString& propName, const QString& valueStr) {
    if (!dzScene) return false;
    DzNode* node = dzScene->findNode(nodeId);
    if (!node) node = dzScene->getPrimarySelection();
    if (!node) return false;

    DzProperty* prop = node->findProperty(propName);
    if (!prop) return false;

    if (DzFloatProperty* fProp = qobject_cast<DzFloatProperty*>(prop)) {
        fProp->setValue(valueStr.toFloat());
        return true;
    }
    if (DzBoolProperty* bProp = qobject_cast<DzBoolProperty*>(prop)) {
        bProp->setBoolValue(valueStr.toLower() == "true" || valueStr == "1");
        return true;
    }
    if (DzColorProperty* cProp = qobject_cast<DzColorProperty*>(prop)) {
        QStringList parts = valueStr.split(",");
        if (parts.size() >= 3) {
            cProp->setColorValue(QColor(parts[0].toInt(), parts[1].toInt(), parts[2].toInt()));
            return true;
        }
    }
    if (DzStringProperty* sProp = qobject_cast<DzStringProperty*>(prop)) {
        sProp->setValue(valueStr);
        return true;
    }
    return false;
}

static bool SetMaterialProperty(const QString& nodeId, const QString& propName, const QString& valueStr) {
    if (!dzScene) return false;
    DzNode* node = dzScene->findNode(nodeId);
    if (!node) node = dzScene->getPrimarySelection();
    if (!node) return false;

    DzObject* obj = node->getObject();
    if (!obj) return false;
    DzShape* shape = obj->getCurrentShape();
    if (!shape) return false;

    bool setAny = false;
    for (int i = 0; i < shape->getNumMaterials(); ++i) {
        DzMaterial* mat = shape->getMaterial(i);
        if (mat) {
            DzProperty* prop = mat->findProperty(propName);
            if (prop) {
                if (DzColorProperty* cProp = qobject_cast<DzColorProperty*>(prop)) {
                    QStringList parts = valueStr.split(",");
                    if (parts.size() >= 3) {
                        cProp->setColorValue(QColor(parts[0].toInt(), parts[1].toInt(), parts[2].toInt()));
                        setAny = true;
                    }
                } else if (DzFloatProperty* fProp = qobject_cast<DzFloatProperty*>(prop)) {
                    fProp->setValue(valueStr.toFloat());
                    setAny = true;
                } else if (DzBoolProperty* bProp = qobject_cast<DzBoolProperty*>(prop)) {
                    bProp->setBoolValue(valueStr.toLower() == "true" || valueStr == "1");
                    setAny = true;
                } else if (DzStringProperty* sProp = qobject_cast<DzStringProperty*>(prop)) {
                    sProp->setValue(valueStr);
                    setAny = true;
                }
            }
        }
    }
    return setAny;
}

static DzNode* ResolveNodeOrSelection(const QString& nodeId) {
    if (!dzScene) return nullptr;
    if (!nodeId.isEmpty() && nodeId.toLower() != "selected") {
        DzNode* node = dzScene->findNode(nodeId);
        if (node) return node;
    }
    return dzScene->getPrimarySelection();
}

static float ClampOpacity(float value) {
    if (value < 0.0f) return 0.0f;
    if (value > 1.0f) return 1.0f;
    return value;
}

static bool IsInternalSurfaceName(const QString& text) {
    QString lower = text.toLower();
    const char* keywords[] = {
        "skull", "bone", "rib", "spine", "pelvis", "clavicle", "scapula",
        "skeleton", "sternum", "vertebra", "femur", "humerus", "anatomy"
    };
    for (const char* keyword : keywords) {
        if (lower.contains(keyword)) return true;
    }
    return false;
}

static int SetOpacityOnMaterials(DzNode* node, const QString& surfacePattern, float value, QStringList* affectedSurfaces = nullptr) {
    if (!node) return 0;
    DzObject* obj = node->getObject();
    if (!obj) return 0;
    DzShape* shape = obj->getCurrentShape();
    if (!shape) return 0;

    QString pattern = surfacePattern.toLower();
    bool matchAll = pattern.isEmpty();
    int count = 0;
    for (int i = 0; i < shape->getNumMaterials(); ++i) {
        DzMaterial* mat = shape->getMaterial(i);
        if (!mat) continue;

        QString name = mat->getName();
        QString label = mat->getLabel();
        QString nameLower = name.toLower();
        QString labelLower = label.toLower();
        bool matches = matchAll ||
            nameLower == pattern ||
            labelLower == pattern ||
            nameLower.contains(pattern) ||
            labelLower.contains(pattern);
        if (!matches) continue;

        DzProperty* prop = mat->findProperty("Opacity");
        if (DzFloatProperty* fProp = qobject_cast<DzFloatProperty*>(prop)) {
            fProp->setValue(value);
            count++;
            if (affectedSurfaces) {
                affectedSurfaces->append(!label.isEmpty() ? label : name);
            }
        }
    }
    return count;
}

static std::string JsonStringArray(const QStringList& values) {
    std::ostringstream oss;
    oss << "[";
    for (int i = 0; i < values.size(); ++i) {
        if (i > 0) oss << ",";
        oss << "\"" << JsonEscape(values[i]) << "\"";
    }
    oss << "]";
    return oss.str();
}

static QStringList GetInternalSurfaceNames(DzNode* node) {
    QStringList surfaces;
    if (!node) return surfaces;
    DzObject* obj = node->getObject();
    if (!obj) return surfaces;
    DzShape* shape = obj->getCurrentShape();
    if (!shape) return surfaces;

    for (int i = 0; i < shape->getNumMaterials(); ++i) {
        DzMaterial* mat = shape->getMaterial(i);
        if (!mat) continue;
        QString name = mat->getName();
        QString label = mat->getLabel();
        if (IsInternalSurfaceName(name) || IsInternalSurfaceName(label)) {
            surfaces.append(!label.isEmpty() ? label : name);
        }
    }
    return surfaces;
}

static DzNode* FindLoadedNode(DzNode* beforeSelection, int beforeNodeCount) {
    DzNode* selected = dzScene ? dzScene->getPrimarySelection() : nullptr;
    if (selected && selected != beforeSelection) return selected;
    if (!dzScene) return selected;
    for (int i = dzScene->getNumNodes() - 1; i >= beforeNodeCount; --i) {
        DzNode* node = dzScene->getNode(i);
        if (node) return node;
    }
    return selected;
}

static std::string PlaceAssetInsideFigure(const QString& figureId, const QString& assetPath) {
    if (!dzScene) return "{\"placed\":false,\"error\":\"No scene\"}";
    DzNode* figure = ResolveNodeOrSelection(figureId);
    if (!figure) return "{\"placed\":false,\"error\":\"Figure not found\"}";

    DzNode* beforeSelection = dzScene->getPrimarySelection();
    int beforeNodeCount = dzScene->getNumNodes();
    if (!OpenContentFile(assetPath, true)) {
        return "{\"placed\":false,\"error\":\"Asset load failed\"}";
    }

    DzNode* asset = FindLoadedNode(beforeSelection, beforeNodeCount);
    if (!asset) return "{\"placed\":false,\"error\":\"Loaded asset node not found\"}";

    DzBox3 box = figure->getWSBoundingBox();
    DzVec3 minVec = box.getMin();
    DzVec3 maxVec = box.getMax();
    DzVec3 center(
        (minVec.m_x + maxVec.m_x) * 0.5f,
        minVec.m_y + ((maxVec.m_y - minVec.m_y) * 0.55f),
        (minVec.m_z + maxVec.m_z) * 0.5f
    );
    asset->setWSPos(center);
    figure->addNodeChild(asset, true);

    std::ostringstream oss;
    oss << "{\"placed\":true,\"figure_id\":\"" << JsonEscape(figure->getName()) << "\",";
    oss << "\"node_id\":\"" << JsonEscape(asset->getName()) << "\",";
    oss << "\"position\":[" << center.m_x << "," << center.m_y << "," << center.m_z << "]}";
    return oss.str();
}

static std::string GetMaterialProperties(const QString& nodeId) {
    if (!dzScene) return "{\"materials\":[]}";
    DzNode* node = dzScene->findNode(nodeId);
    if (!node) node = dzScene->getPrimarySelection();
    if (!node) return "{\"materials\":[]}";

    DzObject* obj = node->getObject();
    if (!obj) return "{\"materials\":[]}";

    std::ostringstream oss;
    oss << "{\"materials\":[";
    bool firstMat = true;

    for (int i = 0; i < obj->getNumShapes(); ++i) {
        DzShape* shape = obj->getShape(i);
        if (!shape) continue;

        for (int j = 0; j < shape->getNumMaterials(); ++j) {
            DzMaterial* mat = shape->getMaterial(j);
            if (!mat) continue;

            if (!firstMat) oss << ",";
            firstMat = false;

            oss << "{";
            oss << "\"name\":\"" << JsonEscape(mat->getName()) << "\",";
            oss << "\"label\":\"" << JsonEscape(mat->getLabel()) << "\",";
            oss << "\"properties\":[";
            
            bool firstProp = true;
            for (int k = 0; k < mat->getNumProperties(); ++k) {
                DzProperty* prop = mat->getProperty(k);
                if (!prop) continue;
                
                DzFloatProperty* fProp = qobject_cast<DzFloatProperty*>(prop);
                if (!fProp) continue;

                if (!firstProp) oss << ",";
                firstProp = false;

                oss << "{";
                oss << "\"name\":\"" << JsonEscape(prop->getName()) << "\",";
                oss << "\"label\":\"" << JsonEscape(prop->getLabel()) << "\",";
                oss << "\"value\":" << fProp->getValue() << ",";
                oss << "\"min\":" << fProp->getMin() << ",";
                oss << "\"max\":" << fProp->getMax();
                oss << "}";
            }
            oss << "]";
            oss << "}";
        }
    }
    oss << "]}";
    return oss.str();
}

static std::string GetNodeProperties(const QString& nodeId) {
    if (!dzScene) return "{\"properties\":[]}";
    DzNode* node = dzScene->findNode(nodeId);
    if (!node) node = dzScene->getPrimarySelection();
    if (!node) return "{\"properties\":[]}";

    std::ostringstream oss;
    oss << "{\"properties\":[";
    bool first = true;
    
    // We want to collect numeric properties, especially morphs and transform properties
    for (int i = 0; i < node->getNumProperties(); ++i) {
        DzProperty* prop = node->getProperty(i);
        if (!prop) continue;
        
        DzFloatProperty* fProp = qobject_cast<DzFloatProperty*>(prop);
        if (!fProp) continue;

        if (!first) oss << ",";
        first = false;
        
        oss << "{";
        oss << "\"name\":\"" << JsonEscape(prop->getName()) << "\",";
        oss << "\"label\":\"" << JsonEscape(prop->getLabel()) << "\",";
        oss << "\"value\":" << fProp->getValue() << ",";
        oss << "\"min\":" << fProp->getMin() << ",";
        oss << "\"max\":" << fProp->getMax() << ",";
        oss << "\"path\":\"" << JsonEscape(prop->getPath()) << "\",";
        oss << "\"is_morph\":" << (prop->getPath().contains("Morphs") ? "true" : "false");
        oss << "}";
    }
    oss << "]}";
    return oss.str();
}

static bool DeleteNode(const QString& nodeId) {
    if (!dzScene) return false;
    DzNode* node = dzScene->findNode(nodeId);
    if (!node) return false;

    dzScene->removeNode(node);
    return true;
}

static std::string GetGeoshellsData() {
    if (!dzScene) return "{\"shells\":[]}";
    std::ostringstream oss;
    oss << "{\"shells\":[";
    bool first = true;
    DzNodeListIterator it = dzScene->nodeListIterator();
    while (it.hasNext()) {
        DzNode* node = it.next();
        if (node && node->inherits("DzGeometryShellNode")) {
            if (!first) oss << ",";
            first = false;
            oss << "{\"id\":\"" << JsonEscape(node->getName()) << "\",";
            oss << "\"name\":\"" << JsonEscape(node->getName()) << "\",";
            oss << "\"label\":\"" << JsonEscape(node->getLabel()) << "\"}";
        }
    }
    oss << "]}";
    return oss.str();
}

static std::string GetSceneAssetsData() {
    if (!dzScene) return "{\"assets\":[]}";
    std::ostringstream oss;
    oss << "{\"assets\":[";
    bool first = true;
    DzNodeListIterator it = dzScene->nodeListIterator();
    while (it.hasNext()) {
        DzNode* node = it.next();
        if (!node) continue;
        QString label = node->getLabel();
        if (label.isEmpty()) label = node->getName();
        if (label.isEmpty()) continue;
        if (!first) oss << ",";
        first = false;
        oss << "\"" << JsonEscape(label) << "\"";
    }
    oss << "]}";
    return oss.str();
}

static bool SetMorphValue(const QString& nodeId, const QString& morphName, const QString& valueStr) {
    if (!dzScene) return false;
    DzNode* node = dzScene->findNode(nodeId);
    if (!node) node = dzScene->getPrimarySelection();
    if (!node) return false;

    DzProperty* prop = node->findProperty(morphName, false);
    if (!prop) {
        for (int i = 0; i < node->getNumProperties(); ++i) {
            DzProperty* candidate = node->getProperty(i);
            if (!candidate) continue;
            if (candidate->getName().compare(morphName, Qt::CaseInsensitive) == 0 ||
                candidate->getLabel().compare(morphName, Qt::CaseInsensitive) == 0) {
                prop = candidate;
                break;
            }
        }
    }
    if (!prop) return false;

    if (DzFloatProperty* fProp = qobject_cast<DzFloatProperty*>(prop)) {
        fProp->setValue(valueStr.toFloat());
        return true;
    }
    return SetProperty(nodeId, morphName, valueStr);
}

static bool ApplyRenderSettings(const QString& widthStr, const QString& heightStr) {
    if (!dzApp) return false;
    int width = widthStr.toInt();
    int height = heightStr.toInt();
    if (width <= 0) width = 1920;
    if (height <= 0) height = 1080;

    QString script = QString(
        "var rm = App.getRenderMgr();\n"
        "if (rm) {\n"
        "  rm.setRenderImgSize(%1, %2);\n"
        "  true;\n"
        "} else { false; }\n"
    ).arg(width).arg(height);

    DzScript dzScript;
    dzScript.addLine(script);
    return dzScript.execute();
}

static std::string GetFigureMorphsData(const QString& nodeId) {
    if (!dzScene) return "{\"morphs\":[]}";
    DzNode* node = dzScene->findNode(nodeId);
    if (!node) return "{\"morphs\":[]}";

    std::ostringstream oss;
    oss << "{\"morphs\":[";
    bool first = true;

    for (int i = 0; i < node->getNumProperties(); ++i) {
        DzProperty* prop = node->getProperty(i);
        if (!prop) continue;
        QString path = prop->getPath();
        if (!path.contains("Morphs")) continue;

        DzFloatProperty* fProp = qobject_cast<DzFloatProperty*>(prop);
        if (!fProp) continue;

        if (!first) oss << ",";
        first = false;

        oss << "{";
        oss << "\"id\":\"" << JsonEscape(prop->getName()) << "\",";
        oss << "\"label\":\"" << JsonEscape(prop->getLabel()) << "\",";
        oss << "\"value\":" << fProp->getValue() << ",";
        oss << "\"min\":" << fProp->getMin() << ",";
        oss << "\"max\":" << fProp->getMax() << ",";
        oss << "\"type\":\"morph\"";
        oss << "}";
    }
    oss << "]}";
    return oss.str();
}

static std::string GetFittedItemsData(const QString& nodeId) {
    if (!dzScene) return "{\"items\":[]}";
    DzNode* figure = dzScene->findNode(nodeId);
    if (!figure) return "{\"items\":[]}";

    std::ostringstream oss;
    oss << "{\"items\":[";
    bool first = true;

    // Iterate all nodes and check if they are children of the figure or wearables
    DzNodeListIterator it = dzScene->nodeListIterator();
    while (it.hasNext()) {
        DzNode* node = it.next();
        if (!node || node == figure) continue;

        // Check if this node's label suggests it's fitted clothing
        QString label = node->getLabel();
        if (label.isEmpty()) continue;

        // A fitted item is typically a child of the figure or has a fitting relationship
        bool isFitted = false;
        if (node->getNodeParent() == figure) {
            isFitted = true;
        }
        // Also check if it's a wearable/fitted item via label heuristics
        if (!isFitted) {
            QString lower = label.toLower();
            if (lower.contains("wearable") || lower.contains("outfit") ||
                lower.contains("clothing") || lower.contains("fit")) {
                isFitted = true;
            }
        }
        if (!isFitted) continue;

        if (!first) oss << ",";
        first = false;
        oss << "{\"node_id\":\"" << JsonEscape(node->getName()) << "\",";
        oss << "\"label\":\"" << JsonEscape(label) << "\"}";
    }
    oss << "]}";
    return oss.str();
}

static std::string GetActiveExpressionsData(const QString& nodeId) {
    if (!dzScene) return "{\"expressions\":[]}";
    DzNode* node = dzScene->findNode(nodeId);
    if (!node) return "{\"expressions\":[]}";

    std::ostringstream oss;
    oss << "{\"expressions\":[";
    bool first = true;

    for (int i = 0; i < node->getNumProperties(); ++i) {
        DzProperty* prop = node->getProperty(i);
        if (!prop) continue;
        QString path = prop->getPath();
        if (!path.contains("Expression")) continue;

        DzFloatProperty* fProp = qobject_cast<DzFloatProperty*>(prop);
        if (!fProp) continue;

        if (!first) oss << ",";
        first = false;

        oss << "{";
        oss << "\"id\":\"" << JsonEscape(prop->getName()) << "\",";
        oss << "\"label\":\"" << JsonEscape(prop->getLabel()) << "\",";
        oss << "\"value\":" << fProp->getValue();
        oss << "}";
    }
    oss << "]}";
    return oss.str();
}

static std::string GetTimelineStateData() {
    if (!dzScene) return "{\"available\":false}";
    std::ostringstream oss;

    int curFrame = dzScene->getFrame();
    DzTimeRange playRange = dzScene->getPlayRange();
    DzTime timeStep = dzScene->getTimeStep();
    float fps = 30.0f;
    if (timeStep > 0) {
        fps = 1.0f / (static_cast<float>(timeStep) / 1000.0f);
    }

    oss << "{";
    oss << "\"current_frame\":" << curFrame << ",";
    oss << "\"start_frame\":" << (playRange.getStart() / timeStep) << ",";
    oss << "\"end_frame\":" << (playRange.getEnd() / timeStep) << ",";
    oss << "\"fps\":" << fps << ",";
    oss << "\"is_playing\":false";
    oss << "}";
    return oss.str();
}

static std::string GetBoundingBoxesData() {
    if (!dzScene) return "{\"bounds\":[]}";
    
    std::ostringstream oss;
    oss << "{\"bounds\":[";
    bool first = true;
    
    DzNodeListIterator it = dzScene->nodeListIterator();
    while (it.hasNext()) {
        DzNode* node = it.next();
        if (!node) continue;
        
        DzBox3 box = node->getWSBoundingBox();
        DzVec3 minVec = box.getMin();
        DzVec3 maxVec = box.getMax();
        DzVec3 centerVec = box.getCenter();
        
        if (!first) oss << ",";
        first = false;
        
        oss << "{";
        oss << "\"node_id\":\"" << JsonEscape(node->getName()) << "\",";
        oss << "\"min\":[" << minVec.m_x << "," << minVec.m_y << "," << minVec.m_z << "],";
        oss << "\"max\":[" << maxVec.m_x << "," << maxVec.m_y << "," << maxVec.m_z << "],";
        oss << "\"center\":[" << centerVec.m_x << "," << centerVec.m_y << "," << centerVec.m_z << "]";
        oss << "}";
    }
    oss << "]}";
    return oss.str();
}

static std::string CommandsData() {
    return "{\"commands\":["
        "{\"name\":\"get_scene_info\",\"description\":\"Get current scene info\",\"category\":\"Scene\",\"parameters\":[]},"
        "{\"name\":\"list_nodes\",\"description\":\"List scene nodes\",\"category\":\"Scene\",\"parameters\":[]},"
        "{\"name\":\"get_selected_nodes\",\"description\":\"List selected nodes\",\"category\":\"Selection\",\"parameters\":[]},"
        "{\"name\":\"select_node\",\"description\":\"Select node by name/id\",\"category\":\"Selection\",\"parameters\":[\"node_id\"]},"
        "{\"name\":\"get_cameras\",\"description\":\"List cameras\",\"category\":\"Camera\",\"parameters\":[]},"
        "{\"name\":\"add_node\",\"description\":\"Add a primitive node\",\"category\":\"Scene\",\"parameters\":[\"type\",\"name\"]},"
        "{\"name\":\"set_property\",\"description\":\"Set a node property\",\"category\":\"Properties\",\"parameters\":[\"node_id\",\"property\",\"value\"]},"
        "{\"name\":\"set_material_property\",\"description\":\"Set a material property\",\"category\":\"Materials\",\"parameters\":[\"node_id\",\"property\",\"value\"]},"
        "{\"name\":\"set_body_opacity\",\"description\":\"Set opacity across all body surfaces\",\"category\":\"Materials\",\"parameters\":[\"node_id\",\"value\"]},"
        "{\"name\":\"set_surface_opacity\",\"description\":\"Set opacity on matching material surfaces\",\"category\":\"Materials\",\"parameters\":[\"node_id\",\"surface_pattern\",\"value\"]},"
        "{\"name\":\"get_internal_surfaces\",\"description\":\"List likely internal anatomy material surfaces\",\"category\":\"Materials\",\"parameters\":[\"node_id\"]},"
        "{\"name\":\"show_anatomy\",\"description\":\"Make internal anatomy surfaces fully opaque\",\"category\":\"Materials\",\"parameters\":[\"node_id\"]},"
        "{\"name\":\"place_asset_inside\",\"description\":\"Load and place an asset inside a figure\",\"category\":\"Assets\",\"parameters\":[\"figure_id\",\"asset_path\"]},"
        "{\"name\":\"get_node_properties\",\"description\":\"Get animatable properties of a node\",\"category\":\"Properties\",\"parameters\":[\"node_id\"]},"
        "{\"name\":\"load_asset\",\"description\":\"Load Daz asset\",\"category\":\"Assets\",\"parameters\":[\"path\"]},"
        "{\"name\":\"apply_pose\",\"description\":\"Apply pose file\",\"category\":\"Pose\",\"parameters\":[\"pose_path\",\"figure_id\"]},"
        "{\"name\":\"render_preview\",\"description\":\"Trigger preview render\",\"category\":\"Render\",\"parameters\":[]},"
        "{\"name\":\"capture_viewport\",\"description\":\"Capture viewport\",\"category\":\"Viewport\",\"parameters\":[\"path\"]},"
        "{\"name\":\"import_model\",\"description\":\"Import model if Daz import support is available\",\"category\":\"Assets\",\"parameters\":[\"path\",\"settings\"]},"
        "{\"name\":\"export_scene\",\"description\":\"Export scene via Daz export pipeline. Settings (JSON): selected_only, include_materials, include_animations, bake_textures, generate_normal_maps, export_all_textures, combine_diffuse_and_alpha_maps, resize_textures, target_texture_width, target_texture_height, bake_makeup_overlay, bake_translucency, bake_specular_to_metallic, bake_refraction_weight\",\"category\":\"Assets\",\"parameters\":[\"node_id\",\"path\",\"settings\"]},"
        "{\"name\":\"begin_undo_batch\",\"description\":\"Start a new undo batch\",\"category\":\"Scene\",\"parameters\":[]},"
        "{\"name\":\"accept_undo_batch\",\"description\":\"Accept the current undo batch\",\"category\":\"Scene\",\"parameters\":[\"caption\"]},"
        "{\"name\":\"cancel_undo_batch\",\"description\":\"Cancel the current undo batch\",\"category\":\"Scene\",\"parameters\":[]},"
        "{\"name\":\"get_bounding_boxes\",\"description\":\"Get world-space 3D bounding boxes of all scene nodes\",\"category\":\"Vision\",\"parameters\":[]},"
        "{\"name\":\"run_script\",\"description\":\"Evaluate arbitrary DazScript\",\"category\":\"Scripting\",\"parameters\":[\"script\",\"args\"]},"
        "{\"name\":\"get_scene_assets\",\"description\":\"List loaded scene node labels\",\"category\":\"Scene\",\"parameters\":[]},"
        "{\"name\":\"add_figure\",\"description\":\"Add Genesis figure (requires path or content)\",\"category\":\"Scene\",\"parameters\":[\"figure_type\",\"path\"]},"
        "{\"name\":\"set_morph\",\"description\":\"Set morph dial value\",\"category\":\"Properties\",\"parameters\":[\"node_id\",\"morph\",\"value\"]},"
        "{\"name\":\"set_light\",\"description\":\"Set light property\",\"category\":\"Lighting\",\"parameters\":[\"node_id\",\"property\",\"value\"]},"
        "{\"name\":\"set_render_settings\",\"description\":\"Set render image size\",\"category\":\"Render\",\"parameters\":[\"width\",\"height\"]},"
        "{\"name\":\"play_timeline\",\"description\":\"Start timeline playback\",\"category\":\"Animation\",\"parameters\":[]},"
        "{\"name\":\"pause_timeline\",\"description\":\"Pause timeline playback\",\"category\":\"Animation\",\"parameters\":[]},"
        "{\"name\":\"stop_timeline\",\"description\":\"Stop playback and reset to frame 0\",\"category\":\"Animation\",\"parameters\":[]},"
        "{\"name\":\"get_timeline_state\",\"description\":\"Get current timeline state\",\"category\":\"Animation\",\"parameters\":[]},"
        "{\"name\":\"get_figure_morphs\",\"description\":\"Get morphs for a figure\",\"category\":\"Properties\",\"parameters\":[\"figure_id\"]},"
        "{\"name\":\"get_fitted_items\",\"description\":\"Get fitted items on a figure\",\"category\":\"Scene\",\"parameters\":[\"figure_id\"]},"
        "{\"name\":\"get_active_expressions\",\"description\":\"Get active expressions on a figure\",\"category\":\"Properties\",\"parameters\":[\"figure_id\"]},"
        "{\"name\":\"get_material_zones\",\"description\":\"Get material zones on a figure\",\"category\":\"Materials\",\"parameters\":[\"figure_id\"]},"
        "{\"name\":\"apply_morph\",\"description\":\"Set a morph dial on a figure\",\"category\":\"Properties\",\"parameters\":[\"figure_id\",\"morph_id\",\"value\"]},"
        "{\"name\":\"apply_expression\",\"description\":\"Set an expression dial on a figure\",\"category\":\"Properties\",\"parameters\":[\"figure_id\",\"expression_id\",\"value\"]},"
        "{\"name\":\"save_scene\",\"description\":\"Save the current scene to a file\",\"category\":\"Scene\",\"parameters\":[\"path\"]},"
        "{\"name\":\"load_scene\",\"description\":\"Load a scene file (method: default/new/merge)\",\"category\":\"Scene\",\"parameters\":[\"path\",\"method\"]},"
        "{\"name\":\"clear_scene\",\"description\":\"Clear the current scene\",\"category\":\"Scene\",\"parameters\":[]},"
        "{\"name\":\"set_camera\",\"description\":\"Set active camera or adjust camera properties\",\"category\":\"Camera\",\"parameters\":[\"camera\",\"focal_length\",\"focal_distance\"]},"
        "{\"name\":\"get_node_transform\",\"description\":\"Get node world-space transform (pos/rot/scale)\",\"category\":\"Scene\",\"parameters\":[\"node_id\"]},"
        "{\"name\":\"set_node_transform\",\"description\":\"Set node world-space position, rotation, or scale\",\"category\":\"Scene\",\"parameters\":[\"node_id\",\"position\",\"rotation\",\"scale\"]},"
        "{\"name\":\"set_render_options\",\"description\":\"Set render quality and output options\",\"category\":\"Render\",\"parameters\":[\"width\",\"height\",\"pixel_samples\",\"ray_trace_depth\",\"shading_rate\",\"gamma\"]},"
        "{\"name\":\"search_content\",\"description\":\"Search Daz content library for assets by name/type\",\"category\":\"Assets\",\"parameters\":[\"query\",\"type\",\"max_results\"]},"
        "{\"name\":\"set_material_texture\",\"description\":\"Assign a texture map file to a material surface channel\",\"category\":\"Materials\",\"parameters\":[\"node_id\",\"channel\",\"file_path\"]},"
        "{\"name\":\"get_material_channels\",\"description\":\"Get all surface channels with texture paths and values\",\"category\":\"Materials\",\"parameters\":[\"node_id\"]},"
        "{\"name\":\"list_bones\",\"description\":\"List all bones in a figure's skeleton\",\"category\":\"Animation\",\"parameters\":[\"figure_id\"]},"
        "{\"name\":\"set_bone_transform\",\"description\":\"Set a bone's world-space position or rotation\",\"category\":\"Animation\",\"parameters\":[\"figure_id\",\"bone_name\",\"position\",\"rotation\"]},"
        "{\"name\":\"list_keyframes\",\"description\":\"List all keyframes on a node property\",\"category\":\"Animation\",\"parameters\":[\"node_id\",\"property\"]},"
        "{\"name\":\"delete_keyframes\",\"description\":\"Delete keyframes from a node property (range or all)\",\"category\":\"Animation\",\"parameters\":[\"node_id\",\"property\",\"start\",\"end\"]},"
        "{\"name\":\"list_modifiers\",\"description\":\"List all modifiers on a node's geometry object\",\"category\":\"Scene\",\"parameters\":[\"node_id\"]},"
        "{\"name\":\"set_viewport_mode\",\"description\":\"Set viewport display mode (texture, shaded, wireframe, lit_wireframe, hidden_line, smooth_lit)\",\"category\":\"Viewport\",\"parameters\":[\"mode\"]}"
    "]}";
}

static std::string DispatchRequest(const std::string& line) {
    QString id = ExtractJsonString(line, "id");
    if (id.isEmpty()) id = "unknown";
    QString command = ExtractJsonString(line, "command");

    if (command == "get_commands") return OkResponse(id, CommandsData());
    if (command == "get_scene_info") return OkResponse(id, SceneInfoData());
    if (command == "list_nodes") return OkResponse(id, NodeListData(false));
    if (command == "get_selected_nodes") return OkResponse(id, NodeListData(true));
    if (command == "get_cameras") return OkResponse(id, CamerasData());
    if (command == "get_bounding_boxes") return OkResponse(id, GetBoundingBoxesData());
    if (command == "run_script") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString script = ExtractArgString(line, "script");
        QString argsJson = ExtractArgString(line, "args");
        
        QString result;
        std::mutex mtx;
        std::condition_variable cv;
        bool done = false;

        RunScriptEvent* event = new RunScriptEvent(script, argsJson, &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, event);

        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        
        return OkResponse(id, result.toStdString());
    }
    if (command == "select_node") {
        QString nodeId = ExtractArgString(line, "node_id");
        if (SelectNodeInDaz(nodeId)) {
            return OkResponse(id, "{\"selected\":true}");
        }
        return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
    }
    if (command == "render_preview") {
        if (!dzApp) return ErrorResponse(id, "No app");
        DzRenderMgr* renderMgr = dzApp->getRenderMgr();
        if (renderMgr) {
            renderMgr->doRender();
            return OkResponse(id, "{\"rendering\":true}");
        }
        return ErrorResponse(id, "No render manager available");
    }
    if (command == "capture_viewport") {
        QString path = ExtractArgString(line, "path");
        std::string result = CaptureActiveViewport(path);
        if (!result.empty()) {
            return OkResponse(id, std::string("{\"result\":\"") + (path == "stream" ? std::string("base64") : JsonEscape(QString::fromStdString(result))) + "\", \"data\":\"" + (path == "stream" ? result : "") + "\"}");
        }
        return ErrorResponse(id, QString("Viewport capture failed: %1").arg(path));
    }
    if (command == "begin_undo_batch") {
        if (BeginUndoBatch()) return OkResponse(id, "{\"started\":true}");
        return ErrorResponse(id, "Failed to start undo batch");
    }
    if (command == "accept_undo_batch") {
        QString caption = ExtractArgString(line, "caption");
        if (AcceptUndoBatch(caption)) return OkResponse(id, "{\"accepted\":true}");
        return ErrorResponse(id, "Failed to accept undo batch");
    }
    if (command == "cancel_undo_batch") {
        if (CancelUndoBatch()) return OkResponse(id, "{\"cancelled\":true}");
        return ErrorResponse(id, "Failed to cancel undo batch");
    }
    if (command == "load_asset") {
        QString path = ExtractArgString(line, "path");
        if (OpenContentFile(path, true)) {
            return OkResponse(id, std::string("{\"path\":\"") + JsonEscape(path) + "\"}");
        }
        return ErrorResponse(id, QString("Asset load failed: %1").arg(path));
    }
    if (command == "apply_pose") {
        QString poseFile = ExtractArgString(line, "pose_path");
        QString figureId = ExtractArgString(line, "figure_id");
        if (!figureId.isEmpty() && dzScene) {
            DzNode* target = dzScene->findNode(figureId);
            if (target) {
                dzScene->setPrimarySelection(target);
            }
        }
        if (OpenContentFile(poseFile, true)) {
            return OkResponse(id, std::string("{\"applied\":\"") + JsonEscape(poseFile) + "\"}");
        }
        return ErrorResponse(id, QString("Failed to apply pose: %1").arg(poseFile));
    }
    if (command == "import_model") {
        QString path = ExtractArgString(line, "path");
        if (ImportContentFile(path)) {
            return OkResponse(id, std::string("{\"path\":\"") + JsonEscape(path) + "\"}");
        }
        return ErrorResponse(id, QString("Model import failed: %1").arg(path));
    }
    if (command == "export_scene") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString path = ExtractArgString(line, "path");
        QString settingsStr = ExtractArgString(line, "settings");
        
        QString result;
        std::mutex mtx;
        std::condition_variable cv;
        bool done = false;

        ExportSceneEvent* event = new ExportSceneEvent(path, settingsStr, &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, event);

        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        
        if (result.contains("\"success\":true")) {
            return OkResponse(id, std::string("{\"path\":\"") + JsonEscape(path) + "\"}");
        }
        return ErrorResponse(id, "Scene export failed or format not supported");
    }
    if (command == "viewport_click") {
        QString xStr = ExtractArgString(line, "x");
        QString yStr = ExtractArgString(line, "y");
        if (dzApp) {
            DzMainWindow* mainWindow = dzApp->getInterface();
            if (mainWindow) {
                DzViewportMgr* viewportMgr = mainWindow->getViewportMgr();
                if (viewportMgr) {
                    DzViewport* viewport = viewportMgr->getActiveViewport();
                    if (viewport) {
                        Dz3DViewport* viewport3d = viewport->get3DViewport();
                        if (viewport3d) {
                            DzNode* pickedNode = viewport3d->pickOnNode(QPoint(xStr.toInt(), yStr.toInt()));
                            if (pickedNode) {
                                if (dzScene) dzScene->setPrimarySelection(pickedNode);
                                return OkResponse(id, std::string("{\"node_id\":\"") + JsonEscape(pickedNode->getName()) + "\"}");
                            }
                        }
                    }
                }
            }
        }
        return OkResponse(id, "{\"node_id\":null}");
    }
    if (command == "add_node") {
        QString type = ExtractArgString(line, "type");
        QString name = ExtractArgString(line, "name");
        if (AddNode(type, name)) {
            return OkResponse(id, "{\"added\":true}");
        }
        return ErrorResponse(id, QString("Failed to add node: %1").arg(type));
    }
    if (command == "set_property") {
        QString nodeId = ExtractArgString(line, "node_id");
        QString prop = ExtractArgString(line, "property");
        QString val = ExtractArgString(line, "value");
        if (SetProperty(nodeId, prop, val)) {
            return OkResponse(id, "{\"set\":true}");
        }
        return ErrorResponse(id, QString("Failed to set property %1 on node %2").arg(prop, nodeId));
    }
    if (command == "set_material_property") {
        QString nodeId = ExtractArgString(line, "node_id");
        QString prop = ExtractArgString(line, "property");
        QString val = ExtractArgString(line, "value");
        if (SetMaterialProperty(nodeId, prop, val)) {
            return OkResponse(id, "{\"set\":true}");
        }
        return ErrorResponse(id, QString("Failed to set material property %1 on node %2").arg(prop, nodeId));
    }
    if (command == "set_body_opacity") {
        QString nodeId = ExtractArgString(line, "node_id");
        float value = ClampOpacity(ExtractArgString(line, "value").toFloat());
        DzNode* node = ResolveNodeOrSelection(nodeId);
        QStringList affected;
        int count = SetOpacityOnMaterials(node, "", value, &affected);
        if (count > 0) {
            std::ostringstream oss;
            oss << "{\"set\":true,\"matched_count\":" << count << ",\"surfaces\":" << JsonStringArray(affected) << "}";
            return OkResponse(id, oss.str());
        }
        return ErrorResponse(id, QString("No opacity material properties found on node %1").arg(nodeId));
    }
    if (command == "set_surface_opacity") {
        QString nodeId = ExtractArgString(line, "node_id");
        QString pattern = ExtractArgString(line, "surface_pattern");
        float value = ClampOpacity(ExtractArgString(line, "value").toFloat());
        DzNode* node = ResolveNodeOrSelection(nodeId);
        QStringList affected;
        int count = SetOpacityOnMaterials(node, pattern, value, &affected);
        if (count > 0) {
            std::ostringstream oss;
            oss << "{\"set\":true,\"matched_count\":" << count << ",\"surfaces\":" << JsonStringArray(affected) << "}";
            return OkResponse(id, oss.str());
        }
        return ErrorResponse(id, QString("No matching opacity surfaces for pattern %1").arg(pattern));
    }
    if (command == "get_internal_surfaces") {
        QString nodeId = ExtractArgString(line, "node_id");
        DzNode* node = ResolveNodeOrSelection(nodeId);
        QStringList surfaces = GetInternalSurfaceNames(node);
        std::ostringstream oss;
        oss << "{\"surfaces\":" << JsonStringArray(surfaces) << ",\"count\":" << surfaces.size() << "}";
        return OkResponse(id, oss.str());
    }
    if (command == "show_anatomy") {
        QString nodeId = ExtractArgString(line, "node_id");
        DzNode* node = ResolveNodeOrSelection(nodeId);
        QStringList surfaces = GetInternalSurfaceNames(node);
        QStringList affected;
        for (int i = 0; i < surfaces.size(); ++i) {
            SetOpacityOnMaterials(node, surfaces[i], 1.0f, &affected);
        }
        std::ostringstream oss;
        oss << "{\"shown\":true,\"matched_count\":" << affected.size() << ",\"surfaces\":" << JsonStringArray(affected) << "}";
        return OkResponse(id, oss.str());
    }
    if (command == "place_asset_inside") {
        QString figureId = ExtractArgString(line, "figure_id");
        QString assetPath = ExtractArgString(line, "asset_path");
        std::string result = PlaceAssetInsideFigure(figureId, assetPath);
        if (result.find("\"placed\":true") != std::string::npos) {
            return OkResponse(id, result);
        }
        return ErrorResponse(id, QString::fromStdString(result));
    }
    if (command == "get_node_properties") {
        QString nodeId = ExtractArgString(line, "node_id");
        return OkResponse(id, GetNodeProperties(nodeId));
    }
    if (command == "get_material_properties") {
        QString nodeId = ExtractArgString(line, "node_id");
        return OkResponse(id, GetMaterialProperties(nodeId));
    }
    if (command == "delete_node") {
        QString nodeId = ExtractArgString(line, "node_id");
        if (DeleteNode(nodeId)) {
            return OkResponse(id, "{\"deleted\":true}");
        }
        return ErrorResponse(id, "Node not found");
    }
    if (command == "get_geoshells") {
        return OkResponse(id, GetGeoshellsData());
    }
    if (command == "get_scene_assets") {
        return OkResponse(id, GetSceneAssetsData());
    }
    if (command == "add_figure") {
        QString path = ExtractArgString(line, "path");
        if (path.isEmpty()) {
            QString figureType = ExtractArgString(line, "figure_type").toLower();
            if (figureType == "genesis9" || figureType == "genesis 9") {
                path = "/People/Genesis 9/Characters/Genesis 9.duf";
            } else if (figureType == "genesis8" || figureType == "genesis 8") {
                path = "/People/Genesis 8 Female/Characters/Genesis 8 Female.duf";
            } else if (figureType == "genesis8.1" || figureType == "genesis 8.1") {
                path = "/People/Genesis 8.1 Female/Characters/Genesis 8.1 Female.duf";
            } else {
                return ErrorResponse(id, QString("Unknown figure_type '%1'. Provide a content path or use: genesis9, genesis8, genesis8.1").arg(figureType));
            }
        }
        if (OpenContentFile(path, true)) {
            return OkResponse(id, std::string("{\"path\":\"") + JsonEscape(path) + "\",\"figure_type\":\"" + JsonEscape(ExtractArgString(line, "figure_type")) + "\"}");
        }
        return ErrorResponse(id, QString("Figure load failed: %1").arg(path));
    }
    if (command == "set_morph") {
        QString nodeId = ExtractArgString(line, "node_id");
        QString morph = ExtractArgString(line, "morph");
        QString val = ExtractArgString(line, "value");
        if (SetMorphValue(nodeId, morph, val)) {
            return OkResponse(id, "{\"set\":true}");
        }
        return ErrorResponse(id, QString("Failed to set morph %1").arg(morph));
    }
    if (command == "set_light") {
        QString nodeId = ExtractArgString(line, "node_id");
        QString prop = ExtractArgString(line, "property");
        QString val = ExtractArgString(line, "value");
        if (SetProperty(nodeId, prop, val)) {
            return OkResponse(id, "{\"set\":true}");
        }
        return ErrorResponse(id, QString("Failed to set light property %1").arg(prop));
    }
    if (command == "set_render_settings") {
        QString width = ExtractArgString(line, "width");
        QString height = ExtractArgString(line, "height");
        if (ApplyRenderSettings(width, height)) {
            return OkResponse(id, std::string("{\"width\":\"") + JsonEscape(width) + "\",\"height\":\"" + JsonEscape(height) + "\"}");
        }
        return ErrorResponse(id, "Failed to apply render settings");
    }

    if (command == "apply_phy_modifier") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        float stiffness = ExtractArgString(line, "stiffness").toFloat();
        float damping   = ExtractArgString(line, "damping").toFloat();
        float mass      = ExtractArgString(line, "mass").toFloat();
        if (stiffness <= 0.0f) stiffness = 12.0f;
        if (damping   <= 0.0f) damping   = 4.0f;
        if (mass      <= 0.0f) mass      = 0.5f;

        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
        DzObject* obj = node->getObject();
        if (!obj)  return ErrorResponse(id, "Node has no geometry object");

        // Remove any existing DazPilotPhy modifier to avoid stacking
        DzModifier* existing = obj->findModifier("DazPilotPhy");
        if (existing) obj->removeModifier(existing);

        DazPilotPhyModifier* mod = new DazPilotPhyModifier();
        mod->setStiffness(stiffness);
        mod->setDamping(damping);
        mod->setMass(mass);
        obj->addModifier(mod);

        return OkResponse(id, std::string("{\"applied\":true,\"node\":\"") + JsonEscape(node->getName()) + "\"}");
    }
    if (command == "remove_phy_modifier") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
        DzObject* obj = node->getObject();
        if (!obj)  return ErrorResponse(id, "Node has no geometry object");
        DzModifier* mod = obj->findModifier("DazPilotPhy");
        if (!mod)  return ErrorResponse(id, "DazPilotPhy modifier not found on node");
        obj->removeModifier(mod);
        return OkResponse(id, "{\"removed\":true}");
    }
    if (command == "set_phy_modifier_params") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
        DzObject* obj = node->getObject();
        if (!obj)  return ErrorResponse(id, "Node has no geometry object");
        DzModifier* baseMod = obj->findModifier("DazPilotPhy");
        DazPilotPhyModifier* mod = qobject_cast<DazPilotPhyModifier*>(baseMod);
        if (!mod)  return ErrorResponse(id, "DazPilotPhy modifier not found on node");
        QString sStr = ExtractArgString(line, "stiffness");
        QString dStr = ExtractArgString(line, "damping");
        QString mStr = ExtractArgString(line, "mass");
        if (!sStr.isEmpty()) mod->setStiffness(sStr.toFloat());
        if (!dStr.isEmpty()) mod->setDamping(dStr.toFloat());
        if (!mStr.isEmpty()) mod->setMass(mStr.toFloat());
        mod->resetSimulation();
        return OkResponse(id, std::string("{\"updated\":true,\"stiffness\":") +
            std::to_string(mod->getStiffness()) + ",\"damping\":" +
            std::to_string(mod->getDamping()) + ",\"mass\":" +
            std::to_string(mod->getMass()) + "}");
    }

    // ─── Animation Commands ───────────────────────────────────────────────────

    if (command == "set_keyframe") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId   = ExtractArgString(line, "node_id");
        QString propName = ExtractArgString(line, "property");
        float   frame    = ExtractArgString(line, "frame").toFloat();
        float   value    = ExtractArgString(line, "value").toFloat();
        QString interpStr = ExtractArgString(line, "interpolation").toLower();

        DzNode* node = dzScene->findNode(nodeId);
        if (!node) node = dzScene->getPrimarySelection();
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));

        DzProperty* prop = node->findProperty(propName);
        if (!prop) return ErrorResponse(id, QString("Property not found: %1").arg(propName));

        DzFloatProperty* fProp = qobject_cast<DzFloatProperty*>(prop);
        if (!fProp) return ErrorResponse(id, QString("Property %1 is not animatable (float)").arg(propName));

        // Convert frame number to DzTime (ticks = frame * timeStep)
        // DzScene::getTimeStep() returns ticks per frame
        DzTime ticksPerFrame = dzScene->getTimeStep();
        DzTime atTime = static_cast<DzTime>(frame) * ticksPerFrame;

        DzFloatProperty::InterpolationType interp = DzFloatProperty::LINEAR_INTERP;
        if (interpStr == "tcb")      interp = DzFloatProperty::TCB_INTERP;
        else if (interpStr == "hermite") interp = DzFloatProperty::HERMITE_INTERP;
        else if (interpStr == "constant") interp = DzFloatProperty::CONSTANT_INTERP;

        BeginUndoBatch();
        fProp->setValue(atTime, value, interp);
        AcceptUndoBatch(QString("Keyframe: %1/%2 @ %3").arg(nodeId, propName).arg(frame));

        return OkResponse(id, std::string("{\"node\":\"") + JsonEscape(node->getName()) +
                          "\",\"property\":\"" + JsonEscape(propName) +
                          "\",\"frame\":" + std::to_string((int)frame) +
                          ",\"value\":" + std::to_string(value) + "}");
    }

    if (command == "set_timeline_range") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        int startFrame = ExtractArgString(line, "start_frame").toInt();
        int endFrame   = ExtractArgString(line, "end_frame").toInt();
        if (endFrame <= startFrame) return ErrorResponse(id, "end_frame must be > start_frame");

        DzTime ticksPerFrame = dzScene->getTimeStep();
        DzTime startTime = static_cast<DzTime>(startFrame) * ticksPerFrame;
        DzTime endTime   = static_cast<DzTime>(endFrame)   * ticksPerFrame;

        dzScene->setPlayRange(DzTimeRange(startTime, endTime));
        dzScene->setAnimRange(DzTimeRange(startTime, endTime));

        return OkResponse(id, std::string("{\"start_frame\":") + std::to_string(startFrame) +
                          ",\"end_frame\":" + std::to_string(endFrame) + "}");
    }

    if (command == "seek_to_frame") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        int frame = ExtractArgString(line, "frame").toInt();
        dzScene->setFrame(frame);
        return OkResponse(id, std::string("{\"frame\":") + std::to_string(frame) + "}");
    }

    if (command == "run_dforce_simulation") {
        if (!dzApp) return ErrorResponse(id, "No app");
        int startFrame = ExtractArgString(line, "start_frame").toInt();
        int endFrame   = ExtractArgString(line, "end_frame").toInt();
        QString nodeId = ExtractArgString(line, "node_id");

        // dForce has no public C++ SDK surface, so we trigger it via inline DAZ Script.
        // This is the officially supported method per DAZ documentation.
        QString script;
        if (!nodeId.isEmpty()) {
            script = QString(
                "var node = Scene.findNode('%1');\n"
                "if (node) {\n"
                "  App.getSimulator().simulate(node, %2, %3);\n"
                "}\n"
            ).arg(nodeId).arg(startFrame).arg(endFrame);
        } else {
            script = QString(
                "App.getSimulator().simulate(null, %1, %2);\n"
            ).arg(startFrame).arg(endFrame);
        }

        DzScript dzScript;
        dzScript.addLine(script);
        bool ok = dzScript.execute();
        if (ok) {
            return OkResponse(id, std::string("{\"simulated\":true,\"start_frame\":") +
                              std::to_string(startFrame) + ",\"end_frame\":" + std::to_string(endFrame) + "}");
        }
        return ErrorResponse(id, "dForce simulation failed — ensure dForce modifier is applied to the node.");
    }

    // ─── Animation Playback Commands ──────────────────────────────────────────

    if (command == "play_timeline") {
        if (!dzApp) return ErrorResponse(id, "No app");
        QString script = "App.play();";
        DzScript dzScript;
        dzScript.addLine(script);
        if (dzScript.execute()) {
            return OkResponse(id, "{\"playing\":true}");
        }
        return ErrorResponse(id, "Failed to start playback");
    }

    if (command == "pause_timeline") {
        if (!dzApp) return ErrorResponse(id, "No app");
        QString script = "App.pause();";
        DzScript dzScript;
        dzScript.addLine(script);
        if (dzScript.execute()) {
            return OkResponse(id, "{\"playing\":false}");
        }
        return ErrorResponse(id, "Failed to pause playback");
    }

    if (command == "stop_timeline") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString script = "App.stop();\nScene.setCurFrame(0);";
        DzScript dzScript;
        dzScript.addLine(script);
        if (dzScript.execute()) {
            return OkResponse(id, "{\"frame\":0}");
        }
        return ErrorResponse(id, "Failed to stop playback");
    }

    if (command == "get_timeline_state") {
        return OkResponse(id, GetTimelineStateData());
    }

    // ─── Scene Property Mirror Commands ───────────────────────────────────────

    if (command == "get_figure_morphs") {
        QString nodeId = ExtractArgString(line, "figure_id");
        return OkResponse(id, GetFigureMorphsData(nodeId));
    }

    if (command == "get_fitted_items") {
        QString nodeId = ExtractArgString(line, "figure_id");
        return OkResponse(id, GetFittedItemsData(nodeId));
    }

    if (command == "get_active_expressions") {
        QString nodeId = ExtractArgString(line, "figure_id");
        return OkResponse(id, GetActiveExpressionsData(nodeId));
    }

    if (command == "get_material_zones") {
        QString nodeId = ExtractArgString(line, "figure_id");
        return OkResponse(id, GetMaterialProperties(nodeId));
    }

    if (command == "apply_morph") {
        QString nodeId = ExtractArgString(line, "figure_id");
        QString morphId = ExtractArgString(line, "morph_id");
        QString val = ExtractArgString(line, "value");
        if (SetMorphValue(nodeId, morphId, val)) {
            return OkResponse(id, "{\"set\":true}");
        }
        return ErrorResponse(id, QString("Failed to set morph %1 on %2").arg(morphId, nodeId));
    }

    if (command == "apply_expression") {
        QString nodeId = ExtractArgString(line, "figure_id");
        QString exprId = ExtractArgString(line, "expression_id");
        QString val = ExtractArgString(line, "value");
        if (SetMorphValue(nodeId, exprId, val)) {
            return OkResponse(id, "{\"set\":true}");
        }
        return ErrorResponse(id, QString("Failed to set expression %1 on %2").arg(exprId, nodeId));
    }

    // ─── Scene Lifecycle Commands ─────────────────────────────────────────────

    if (command == "save_scene") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString path = ExtractArgString(line, "path");
        DzError err = dzScene->saveScene(path);
        if (err == DZ_NO_ERROR) {
            return OkResponse(id, std::string("{\"path\":\"") + JsonEscape(path) + "\"}");
        }
        return ErrorResponse(id, QString("Save failed: error code %1").arg((int)err));
    }

    if (command == "load_scene") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString path = ExtractArgString(line, "path");
        QString methodStr = ExtractArgString(line, "method").toLower();
        DzScene::DzOpenMethod method = DzScene::DefaultMethod;
        if (methodStr == "new") method = DzScene::OpenNew;
        else if (methodStr == "merge") method = DzScene::MergeFile;
        DzError err = dzScene->loadScene(path, method);
        if (err == DZ_NO_ERROR) {
            return OkResponse(id, std::string("{\"path\":\"") + JsonEscape(path) + "\",\"method\":\"" + JsonEscape(methodStr) + "\"}");
        }
        return ErrorResponse(id, QString("Load failed: error code %1").arg((int)err));
    }

    if (command == "clear_scene") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        dzScene->clear();
        return OkResponse(id, "{\"cleared\":true}");
    }

    // ─── Camera Commands ───────────────────────────────────────────────────────

    if (command == "set_camera") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        DzNode* activeCam = nullptr;
        QString cameraName = ExtractArgString(line, "camera");
        if (!cameraName.isEmpty()) {
            for (int i = 0; i < dzScene->getNumCameras(); i++) {
                DzCamera* cam = dzScene->getCamera(i);
                if (cam && cam->getName() == cameraName) {
                    activeCam = cam;
                    break;
                }
            }
            if (!activeCam) return ErrorResponse(id, QString("Camera not found: %1").arg(cameraName));
        }
        if (activeCam) {
            // Set as active camera in viewport
            if (dzApp) {
                DzMainWindow* mainWindow = dzApp->getInterface();
                if (mainWindow) {
                    DzViewportMgr* viewportMgr = mainWindow->getViewportMgr();
                    if (viewportMgr) {
                        DzViewport* viewport = viewportMgr->getActiveViewport();
                        if (viewport) {
                            Dz3DViewport* viewport3d = viewport->get3DViewport();
                            if (viewport3d) {
                                viewport3d->setCamera(qobject_cast<DzCamera*>(activeCam));
                                viewport3d->frameCamera();
                            }
                        }
                    }
                }
            }
            QString focalLen = ExtractArgString(line, "focal_length");
            if (!focalLen.isEmpty()) {
                qobject_cast<DzCamera*>(activeCam)->setFocalLength(focalLen.toDouble());
            }
            QString focalDist = ExtractArgString(line, "focal_distance");
            if (!focalDist.isEmpty()) {
                qobject_cast<DzCamera*>(activeCam)->setFocalDistance(focalDist.toDouble());
            }
            return OkResponse(id, std::string("{\"camera\":\"") + JsonEscape(activeCam->getName()) + "\",\"updated\":true}");
        }
        // No camera specified — return current camera info
        if (dzApp) {
            DzMainWindow* mainWindow = dzApp->getInterface();
            if (mainWindow) {
                DzViewportMgr* viewportMgr = mainWindow->getViewportMgr();
                if (viewportMgr) {
                    DzViewport* viewport = viewportMgr->getActiveViewport();
                    if (viewport) {
                        Dz3DViewport* viewport3d = viewport->get3DViewport();
                        if (viewport3d) {
                            DzCamera* cam = viewport3d->getCamera();
                            if (cam) {
                                std::ostringstream oss;
                                oss << "{\"camera\":\"" << JsonEscape(cam->getName()) << "\",";
                                oss << "\"focal_length\":" << cam->getFocalLength() << ",";
                                oss << "\"focal_distance\":" << cam->getFocalDistance() << ",";
                                oss << "\"aspect_ratio\":" << cam->getAspectRatio();
                                oss << "}";
                                return OkResponse(id, oss.str());
                            }
                        }
                    }
                }
            }
        }
        return OkResponse(id, "{\"camera\":null}");
    }

    // ─── Render Options ────────────────────────────────────────────────────────

    if (command == "set_render_options") {
        if (!dzApp) return ErrorResponse(id, "No app");
        DzRenderMgr* renderMgr = dzApp->getRenderMgr();
        if (!renderMgr) return ErrorResponse(id, "No render manager");
        DzRenderOptions* options = renderMgr->getRenderOptions();
        if (!options) return ErrorResponse(id, "No render options");

        QString wStr = ExtractArgString(line, "width");
        QString hStr = ExtractArgString(line, "height");
        if (!wStr.isEmpty() && !hStr.isEmpty()) {
            options->setImageSize(QSize(wStr.toInt(), hStr.toInt()));
        }
        QString samplesStr = ExtractArgString(line, "pixel_samples");
        if (!samplesStr.isEmpty()) {
            int s = samplesStr.toInt();
            options->setPixelSamples(s, s);
        }
        QString rayDepthStr = ExtractArgString(line, "ray_trace_depth");
        if (!rayDepthStr.isEmpty()) {
            options->setRayTraceDepth(rayDepthStr.toInt());
        }
        QString shadingStr = ExtractArgString(line, "shading_rate");
        if (!shadingStr.isEmpty()) {
            options->setShadingRate(shadingStr.toDouble());
        }
        QString gammaStr = ExtractArgString(line, "gamma");
        if (!gammaStr.isEmpty()) {
            options->setGamma(gammaStr.toDouble());
        }
        return OkResponse(id, "{\"updated\":true}");
    }

    // ─── Content Search ────────────────────────────────────────────────────────

    if (command == "search_content") {
        if (!dzApp) return ErrorResponse(id, "No app");
        DzContentMgr* contentMgr = dzApp->getContentMgr();
        if (!contentMgr) return ErrorResponse(id, "No content manager");

        QString query = ExtractArgString(line, "query");
        QString typeFilter = ExtractArgString(line, "type").toLower();
        int maxResults = ExtractArgString(line, "max_results").toInt();
        if (maxResults <= 0) maxResults = 50;

        QStringList filters;
        if (typeFilter == "figure") filters << "*.duf";
        else if (typeFilter == "pose") filters << "*.duf" << "*.pz2";
        else if (typeFilter == "morph") filters << "*.duf";
        else if (typeFilter == "material") filters << "*.duf";
        else if (typeFilter == "light") filters << "*.duf" << "*.lw";
        else if (typeFilter == "animation") filters << "*.duf";
        else filters << "*.duf" << "*.pz2" << "*.lw" << "*.obj" << "*.fbx";

        // Recursively search content directories using QDirIterator
        std::ostringstream oss;
        oss << "{\"results\":[";
        bool first = true;
        int count = 0;

        for (int i = 0; i < contentMgr->getNumContentDirectories(); i++) {
            QString dir = contentMgr->getContentDirectoryPath(i);
            QDirIterator it(dir, filters, QDir::Files, QDirIterator::Subdirectories);
            while (it.hasNext() && count < maxResults) {
                QString filePath = it.next();
                QString fileName = it.fileName();
                if (!query.isEmpty() && !fileName.contains(query, Qt::CaseInsensitive)) continue;

                if (!first) oss << ",";
                first = false;
                oss << "{\"name\":\"" << JsonEscape(fileName) << "\",";
                oss << "\"path\":\"" << JsonEscape(filePath) << "\",";
                oss << "\"type\":\"" << JsonEscape(typeFilter.isEmpty() ? "unknown" : typeFilter) << "\"}";
                count++;
            }
        }
        oss << "],\"count\":" << count << "}";
        return OkResponse(id, oss.str());
    }

    // ─── Material Texture Commands ─────────────────────────────────────────────

    if (command == "set_material_texture") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        QString channel = ExtractArgString(line, "channel");
        QString filePath = ExtractArgString(line, "file_path");

        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));

        DzObject* obj = node->getObject();
        if (!obj) return ErrorResponse(id, "Node has no geometry object");
        DzShape* shape = obj->getCurrentShape();
        if (!shape) return ErrorResponse(id, "Node has no shape");

        // Build candidate property names: exact match + common aliases
        QStringList candidates;
        QString chanLower = channel.toLower();
        if (chanLower == "diffuse" || chanLower == "diffuse_map") {
            candidates << "Diffuse Value Map" << "Diffuse Color" << "Color";
        } else if (chanLower == "bump" || chanLower == "bump_map") {
            candidates << "Bump Map" << "Bump Strength" << "Bump";
        } else if (chanLower == "normal" || chanLower == "normal_map") {
            candidates << "Normal Value Map" << "Normal Map" << "Normal";
        } else if (chanLower == "displacement" || chanLower == "displacement_map") {
            candidates << "Displacement Map" << "Displacement Strength" << "Displacement";
        } else if (chanLower == "specular" || chanLower == "specular_map") {
            candidates << "Specular Value Map" << "Specular Color" << "Specular Weight" << "Specular";
        } else if (chanLower == "specular_color") {
            candidates << "Specular Color Map" << "Specular Color";
        } else if (chanLower == "glossiness" || chanLower == "glossiness_map") {
            candidates << "Glossiness Value Map" << "Glossiness" << "Roughness" << "Roughness Value Map";
        } else if (chanLower == "reflection" || chanLower == "reflection_map") {
            candidates << "Reflection Value Map" << "Reflection Weight" << "Reflection";
        } else if (chanLower == "refraction" || chanLower == "refraction_map") {
            candidates << "Refraction Value Map" << "Refraction Weight" << "Refraction";
        } else if (chanLower == "opacity" || chanLower == "opacity_map") {
            candidates << "Opacity Value Map" << "Opacity Weight" << "Opacity" << "Transparency";
        } else if (chanLower == "ambient" || chanLower == "ambient_map") {
            candidates << "Ambient Value Map" << "Ambient Color" << "Ambient";
        } else {
            candidates << channel;
        }
        candidates.removeDuplicates();

        bool setAny = false;
        for (int i = 0; i < shape->getNumMaterials(); ++i) {
            DzMaterial* mat = shape->getMaterial(i);
            if (!mat) continue;

            for (const QString& candidate : candidates) {
                DzProperty* prop = mat->findProperty(candidate);
                if (!prop) continue;

                DzImageProperty* imgProp = qobject_cast<DzImageProperty*>(prop);
                if (imgProp) {
                    imgProp->setValue(filePath);
                    setAny = true;
                    break;
                }
                DzStringProperty* sProp = qobject_cast<DzStringProperty*>(prop);
                if (sProp) {
                    sProp->setValue(filePath);
                    setAny = true;
                    break;
                }
            }
        }

        if (setAny) {
            return OkResponse(id, std::string("{\"set\":true,\"channel\":\"") +
                JsonEscape(channel) + "\",\"file\":\"" + JsonEscape(filePath) + "\"}");
        }
        return ErrorResponse(id, QString("Channel '%1' not found on node %2").arg(channel, nodeId));
    }

    if (command == "get_material_channels") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));

        DzObject* obj = node->getObject();
        if (!obj) return ErrorResponse(id, "Node has no geometry object");

        std::ostringstream oss;
        oss << "{\"channels\":[";
        bool firstMat = true;

        for (int i = 0; i < obj->getNumShapes(); ++i) {
            DzShape* shape = obj->getShape(i);
            if (!shape) continue;

            for (int j = 0; j < shape->getNumMaterials(); ++j) {
                DzMaterial* mat = shape->getMaterial(j);
                if (!mat) continue;

                if (!firstMat) oss << ",";
                firstMat = false;

                oss << "{\"material\":\"" << JsonEscape(mat->getName()) << "\",";
                oss << "\"label\":\"" << JsonEscape(mat->getLabel()) << "\",";
                oss << "\"channels\":{";

                // Check for standard texture channels
                DzDefaultMaterial* dfltMat = qobject_cast<DzDefaultMaterial*>(mat);
                bool firstChan = true;

                auto addChannel = [&](const char* name, DzTexture* tex, double value) {
                    if (!firstChan) oss << ",";
                    firstChan = false;
                    oss << "\"" << name << "\":{\"value\":" << value;
                    if (tex) {
                        QString fname = tex->getFilename();
                        oss << ",\"texture\":\"" << JsonEscape(fname) << "\"";
                    } else {
                        oss << ",\"texture\":null";
                    }
                    oss << "}";
                };

                if (dfltMat) {
                    addChannel("diffuse", dfltMat->getDiffuseValueMap(), dfltMat->getDiffuseStrength());
                    addChannel("bump", dfltMat->getBumpMap(), dfltMat->getBumpStrength());
                    addChannel("normal", dfltMat->getNormalValueMap(), 1.0);
                    addChannel("displacement", dfltMat->getDisplacementMap(), dfltMat->getDisplacementStrength());
                    addChannel("specular", dfltMat->getSpecularValueMap(), dfltMat->getSpecularStrength());
                    addChannel("specular_color", dfltMat->getSpecularColorMap(), 1.0);
                    addChannel("glossiness", dfltMat->getGlossinessValueMap(), dfltMat->getGlossinessStrength());
                    addChannel("reflection", dfltMat->getReflectionValueMap(), dfltMat->getReflectionStrength());
                    addChannel("refraction", dfltMat->getRefractionValueMap(), dfltMat->getRefractionStrength());
                    addChannel("opacity", dfltMat->getOpacityMap(), 1.0);
                    addChannel("ambient", dfltMat->getAmbientValueMap(), dfltMat->getAmbientStrength());
                }

                oss << "}}";
            }
        }
        oss << "]}";
        return OkResponse(id, oss.str());
    }

    // ─── Node Transform Commands ───────────────────────────────────────────────

    if (command == "get_node_transform") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
        DzVec3 pos = node->getWSPos();
        DzQuat rot = node->getWSRot();
        DzMatrix3 scaleMat = node->getWSScale();
        const float* r0 = scaleMat.rowPointer(0);
        const float* r1 = scaleMat.rowPointer(1);
        const float* r2 = scaleMat.rowPointer(2);
        std::ostringstream oss;
        oss << "{";
        oss << "\"node_id\":\"" << JsonEscape(node->getName()) << "\",";
        oss << "\"position\":[" << pos.m_x << "," << pos.m_y << "," << pos.m_z << "],";
        oss << "\"rotation\":[" << rot.m_x << "," << rot.m_y << "," << rot.m_z << "," << rot.m_w << "],";
        oss << "\"scale\":[" << r0[0] << "," << r0[1] << "," << r0[2] << ",";
        oss << r1[0] << "," << r1[1] << "," << r1[2] << ",";
        oss << r2[0] << "," << r2[1] << "," << r2[2] << "]";
        oss << "}";
        return OkResponse(id, oss.str());
    }

    if (command == "set_node_transform") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
        QString posStr = ExtractArgString(line, "position");
        if (!posStr.isEmpty()) {
            // Position as JSON array: [x, y, z]
            posStr.remove('[').remove(']');
            QStringList parts = posStr.split(',');
            if (parts.size() == 3) {
                DzVec3 pos(parts[0].trimmed().toFloat(), parts[1].trimmed().toFloat(), parts[2].trimmed().toFloat());
                node->setWSPos(pos);
            }
        }
        QString rotStr = ExtractArgString(line, "rotation");
        if (!rotStr.isEmpty()) {
            // Rotation as JSON array: [x, y, z, w] (quaternion)
            rotStr.remove('[').remove(']');
            QStringList parts = rotStr.split(',');
            if (parts.size() == 4) {
                DzQuat rot(parts[0].trimmed().toFloat(), parts[1].trimmed().toFloat(), parts[2].trimmed().toFloat(), parts[3].trimmed().toFloat());
                node->setWSRot(rot);
            }
        }
        QString scaleStr = ExtractArgString(line, "scale");
        if (!scaleStr.isEmpty()) {
            // Scale as JSON array of 9 values (matrix) or single uniform value
            scaleStr.remove('[').remove(']');
            QStringList parts = scaleStr.split(',');
            if (parts.size() == 1) {
                float s = parts[0].trimmed().toFloat();
                float vals[12] = {s, 0, 0, 0, s, 0, 0, 0, s, 0, 0, 0};
                DzMatrix3 scaleMat(vals);
                node->setWSScale(scaleMat);
            } else if (parts.size() == 9) {
                float vals[12] = {
                    parts[0].trimmed().toFloat(), parts[1].trimmed().toFloat(), parts[2].trimmed().toFloat(), 0,
                    parts[3].trimmed().toFloat(), parts[4].trimmed().toFloat(), parts[5].trimmed().toFloat(), 0,
                    parts[6].trimmed().toFloat(), parts[7].trimmed().toFloat(), parts[8].trimmed().toFloat(), 0
                };
                DzMatrix3 scaleMat(vals);
                node->setWSScale(scaleMat);
            }
        }
        return OkResponse(id, std::string("{\"node\":\"") + JsonEscape(node->getName()) + "\",\"updated\":true}");
    }

    // ─── Skeleton / Bone Commands ──────────────────────────────────────────────

    if (command == "list_bones") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString figureId = ExtractArgString(line, "figure_id");
        DzNode* node = figureId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(figureId);
        if (!node) return ErrorResponse(id, QString("Figure not found: %1").arg(figureId));
        DzFigure* figure = qobject_cast<DzFigure*>(node);
        if (!figure) return ErrorResponse(id, "Node is not a figure");

        QObjectList bones = figure->getAllBones();
        std::ostringstream oss;
        oss << "{\"bones\":[";
        bool first = true;
        for (QObject* obj : bones) {
            DzBone* bone = qobject_cast<DzBone*>(obj);
            if (!bone) continue;
            if (!first) oss << ",";
            first = false;
            DzVec3 pos = bone->getWSPos();
            DzQuat rot = bone->getWSRot();
            oss << "{\"name\":\"" << JsonEscape(bone->getName()) << "\",";
            oss << "\"position\":[" << pos.m_x << "," << pos.m_y << "," << pos.m_z << "],";
            oss << "\"rotation\":[" << rot.m_x << "," << rot.m_y << "," << rot.m_z << "," << rot.m_w << "]}";
        }
        oss << "],\"count\":" << (first ? 0 : bones.size()) << "}";
        return OkResponse(id, oss.str());
    }

    if (command == "set_bone_transform") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString figureId = ExtractArgString(line, "figure_id");
        QString boneName = ExtractArgString(line, "bone_name");

        DzNode* node = figureId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(figureId);
        if (!node) return ErrorResponse(id, QString("Figure not found: %1").arg(figureId));
        DzFigure* figure = qobject_cast<DzFigure*>(node);
        if (!figure) return ErrorResponse(id, "Node is not a figure");

        DzBone* bone = figure->findBone(boneName);
        if (!bone) return ErrorResponse(id, QString("Bone not found: %1").arg(boneName));

        BeginUndoBatch();

        QString posStr = ExtractArgString(line, "position");
        if (!posStr.isEmpty()) {
            posStr.remove('[').remove(']');
            QStringList parts = posStr.split(',');
            if (parts.size() == 3) {
                bone->setWSPos(DzVec3(parts[0].trimmed().toFloat(), parts[1].trimmed().toFloat(), parts[2].trimmed().toFloat()));
            }
        }
        QString rotStr = ExtractArgString(line, "rotation");
        if (!rotStr.isEmpty()) {
            rotStr.remove('[').remove(']');
            QStringList parts = rotStr.split(',');
            if (parts.size() == 4) {
                bone->setWSRot(DzQuat(parts[0].trimmed().toFloat(), parts[1].trimmed().toFloat(), parts[2].trimmed().toFloat(), parts[3].trimmed().toFloat()));
            }
        }

        AcceptUndoBatch(QString("Set bone transform: %1").arg(boneName));
        return OkResponse(id, std::string("{\"bone\":\"") + JsonEscape(boneName) + "\",\"updated\":true}");
    }

    // ─── Keyframe Management Commands ──────────────────────────────────────────

    if (command == "list_keyframes") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        QString propName = ExtractArgString(line, "property");

        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));

        DzProperty* prop = node->findProperty(propName);
        if (!prop) return ErrorResponse(id, QString("Property not found: %1").arg(propName));

        DzFloatProperty* fProp = qobject_cast<DzFloatProperty*>(prop);
        if (!fProp) return ErrorResponse(id, QString("Property %1 is not a float property").arg(propName));

        DzTime ticksPerFrame = dzScene->getTimeStep();
        int numKeys = fProp->getNumKeys();

        std::ostringstream oss;
        oss << "{\"node\":\"" << JsonEscape(node->getName()) << "\",";
        oss << "\"property\":\"" << JsonEscape(propName) << "\",";
        oss << "\"keyframes\":[";
        for (int i = 0; i < numKeys; ++i) {
            if (i > 0) oss << ",";
            DzTime t = fProp->getKeyTime(i);
            double val = fProp->getKeyValue(i);
            float frame = (float)t / (float)ticksPerFrame;
            oss << "{\"index\":" << i << ",\"frame\":" << frame << ",\"value\":" << val << "}";
        }
        oss << "],\"count\":" << numKeys << "}";
        return OkResponse(id, oss.str());
    }

    if (command == "delete_keyframes") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        QString propName = ExtractArgString(line, "property");
        QString startStr = ExtractArgString(line, "start");
        QString endStr = ExtractArgString(line, "end");

        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));

        DzProperty* prop = node->findProperty(propName);
        if (!prop) return ErrorResponse(id, QString("Property not found: %1").arg(propName));

        int deleted;
        if (startStr.isEmpty() && endStr.isEmpty()) {
            // Delete all keyframes
            deleted = prop->deleteKeys(DzTimeRange(DzTime(0), DzTime(0x7FFFFFFF)));
        } else {
            DzTime ticksPerFrame = dzScene->getTimeStep();
            DzTime startTime = static_cast<DzTime>(startStr.toInt()) * ticksPerFrame;
            DzTime endTime = endStr.isEmpty() ? startTime + ticksPerFrame : static_cast<DzTime>(endStr.toInt()) * ticksPerFrame;
            deleted = prop->deleteKeys(DzTimeRange(startTime, endTime));
        }

        return OkResponse(id, std::string("{\"deleted\":") + std::to_string(deleted) + "}");
    }

    // ─── Viewport Control ──────────────────────────────────────────────────────

    if (command == "set_viewport_mode") {
        if (!dzApp) return ErrorResponse(id, "No app");
        DzMainWindow* mainWindow = dzApp->getInterface();
        if (!mainWindow) return ErrorResponse(id, "No main window");
        DzViewportMgr* viewportMgr = mainWindow->getViewportMgr();
        if (!viewportMgr) return ErrorResponse(id, "No viewport manager");
        DzViewport* viewport = viewportMgr->getActiveViewport();
        if (!viewport) return ErrorResponse(id, "No active viewport");
        Dz3DViewport* viewport3d = viewport->get3DViewport();
        if (!viewport3d) return ErrorResponse(id, "No 3D viewport");

        QString mode = ExtractArgString(line, "mode").toLower();
        Dz3DViewport::ShadeStyle style = viewport3d->getShadeStyle();

        if (mode == "texture") style = Dz3DViewport::Textured;
        else if (mode == "wire_textured") style = Dz3DViewport::WireTextured;
        else if (mode == "shaded") style = Dz3DViewport::SmoothShaded;
        else if (mode == "wire_shaded") style = Dz3DViewport::WireShaded;
        else if (mode == "wireframe") style = Dz3DViewport::Wireframe;
        else if (mode == "lit_wireframe") style = Dz3DViewport::LitWireframe;
        else if (mode == "hidden_line") style = Dz3DViewport::HiddenLine;
        else if (mode == "wire_box") style = Dz3DViewport::WireBox;
        else if (mode == "solid_box") style = Dz3DViewport::SolidBox;
        else return ErrorResponse(id, QString("Unknown viewport mode: %1 (try: texture, shaded, wireframe, lit_wireframe, hidden_line)").arg(mode));

        viewport3d->setShadeStyle(style);
        return OkResponse(id, std::string("{\"mode\":\"") + JsonEscape(mode) + "\"}");
    }

    // ─── Modifier Stack Commands ───────────────────────────────────────────────

    if (command == "list_modifiers") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));

        DzObject* obj = node->getObject();
        if (!obj) return OkResponse(id, "{\"modifiers\":[],\"count\":0}");

        int numMods = obj->getNumModifiers();
        std::ostringstream oss;
        oss << "{\"modifiers\":[";
        for (int i = 0; i < numMods; ++i) {
            DzModifier* mod = obj->getModifier(i);
            if (!mod) continue;
            if (i > 0) oss << ",";
            oss << "{\"index\":" << i << ",\"name\":\"" << JsonEscape(mod->getName()) << "\",";
            oss << "\"label\":\"" << JsonEscape(mod->getLabel()) << "\"}";
        }
        oss << "],\"count\":" << numMods << "}";
        return OkResponse(id, oss.str());
    }

    return ErrorResponse(id, QString("Unknown command: %1").arg(command));
}

static void BridgeServerLoop() {
#ifdef _WIN32
    WSADATA wsaData;
    if (WSAStartup(MAKEWORD(2, 2), &wsaData) != 0) {
        LOG_ERROR("WSAStartup failed");
        return;
    }
#endif

    g_listenSocket = socket(AF_INET, SOCK_STREAM, 0);
    if (g_listenSocket == INVALID_BRIDGE_SOCKET) {
#ifdef _WIN32
        WSACleanup();
#endif
        return;
    }

    sockaddr_in service;
    service.sin_family = AF_INET;
    service.sin_addr.s_addr = inet_addr(g_state.host.toUtf8().constData());
    service.sin_port = htons(static_cast<unsigned short>(g_state.port));

    if (bind(g_listenSocket, reinterpret_cast<sockaddr*>(&service), sizeof(service)) < 0) {
        LOG_ERROR("Failed to bind {}:{}", g_state.host.toStdString(), g_state.port);
        CloseBridgeSocket(g_listenSocket);
        g_listenSocket = INVALID_BRIDGE_SOCKET;
#ifdef _WIN32
        WSACleanup();
#endif
        return;
    }

    listen(g_listenSocket, SOMAXCONN);
    LOG_INFO("Listening on {}:{}", g_state.host.toStdString(), g_state.port);

    while (g_serverRunning.load()) {
        BridgeSocket client = accept(g_listenSocket, nullptr, nullptr);
        if (client == INVALID_BRIDGE_SOCKET) continue;

#ifdef _WIN32
        DWORD timeoutMs = 5000;
        setsockopt(client, SOL_SOCKET, SO_RCVTIMEO, (const char*)&timeoutMs, sizeof(timeoutMs));
#else
        struct timeval tv;
        tv.tv_sec = 5;
        tv.tv_usec = 0;
        setsockopt(client, SOL_SOCKET, SO_RCVTIMEO, &tv, sizeof(tv));
#endif

        std::string line;
        char ch;
        const size_t MAX_REQUEST_SIZE = 1 * 1024 * 1024;
        while (recv(client, &ch, 1, 0) == 1) {
            if (ch == '\n') break;
            if (line.size() >= MAX_REQUEST_SIZE) {
                line.clear();
                break;
            }
            line.push_back(ch);
        }

        std::string response = DispatchRequest(line);
#ifdef _WIN32
        send(client, response.c_str(), static_cast<int>(response.size()), 0);
#else
        send(client, response.c_str(), response.size(), 0);
#endif
        CloseBridgeSocket(client);
    }

    if (g_listenSocket != INVALID_BRIDGE_SOCKET) {
        CloseBridgeSocket(g_listenSocket);
        g_listenSocket = INVALID_BRIDGE_SOCKET;
    }
#ifdef _WIN32
    WSACleanup();
#endif
}

const char* GetPluginName() { return "DazPilot Bridge"; }
const char* GetPluginDescription() { return "AI-powered scene editing bridge for Daz Studio"; }
const char* GetPluginVersion() { return "0.5.3"; }
int GetPluginType() { return 1; }

bool PluginInitialize() {
    dazpilot::Log::init();
    g_scriptExecutor = new ScriptExecutor();
    
    // Resolve bridge_config.json path dynamically from shared OS AppData
    QString configPath;
#ifdef _WIN32
    char* appdata = getenv("APPDATA");
    if (appdata) {
        configPath = QString(appdata) + "/com.dazpilot.desktop/bridge_config.json";
    }
#else
    char* home = getenv("HOME");
    if (home) {
        configPath = QString(home) + "/Library/Application Support/com.dazpilot.desktop/bridge_config.json";
    }
#endif

    if (!configPath.isEmpty()) {
        auto config = dazpilot::json_util::readJsonFile(configPath.toStdString());
        if (config) {
            g_state.port = config->value("port", 8765);
            g_state.host = QString::fromStdString(config->value("host", "127.0.0.1"));
        }
    }
    
    g_serverRunning = true;
    g_serverThread = std::thread(BridgeServerLoop);
    LOG_INFO("Plugin initialized. Listening on {}:{}", g_state.host.toStdString(), g_state.port);
    return true;
}

void PluginCleanup() {
    DisconnectFromDazPilot();
    if (g_scriptExecutor) {
        delete g_scriptExecutor;
        g_scriptExecutor = nullptr;
    }
    LOG_INFO("Plugin cleanup complete");
}

const char* GetMenuName() { return "DazPilot Bridge"; }

void ExecuteMenuAction(const char* action) {
    LOG_INFO("Menu action: {}", action ? action : "");
}

bool ConnectToDazPilot(const char* host, int port) {
    g_state.host = (host && strlen(host) > 0) ? QString(host) : "127.0.0.1";
    g_state.port = (port > 0) ? port : 8765;
    return IsConnectedToDazPilot();
}

void DisconnectFromDazPilot() {
    g_serverRunning = false;
    if (g_listenSocket != INVALID_BRIDGE_SOCKET) {
        CloseBridgeSocket(g_listenSocket);
        g_listenSocket = INVALID_BRIDGE_SOCKET;
    }
    if (g_serverThread.joinable()) {
        g_serverThread.join();
    }
}

bool IsConnectedToDazPilot() {
    return g_serverRunning.load();
}

const char* GetSceneInfo() {
    g_state.lastResponse = QString::fromUtf8(SceneInfoData().c_str());
    return g_state.lastResponse.toUtf8().constData();
}

const char* GetNodeList() {
    g_state.lastResponse = QString::fromUtf8(NodeListData(false).c_str());
    return g_state.lastResponse.toUtf8().constData();
}

const char* GetSelectedNodes() {
    g_state.lastResponse = QString::fromUtf8(NodeListData(true).c_str());
    return g_state.lastResponse.toUtf8().constData();
}

bool SelectNode(const char* nodeId) {
    return SelectNodeInDaz(QString(nodeId ? nodeId : ""));
}

bool LoadAsset(const char* assetPath) {
    return OpenContentFile(QString(assetPath ? assetPath : ""), true);
}

bool ApplyPose(const char* poseFile, const char* figureId) {
    if (figureId && strlen(figureId) > 0 && dzScene) {
        DzNode* target = dzScene->findNode(QString(figureId));
        if (target) {
            dzScene->setPrimarySelection(target);
        }
    }
    return OpenContentFile(QString(poseFile ? poseFile : ""), true);
}

bool RenderPreview() {
    LOG_INFO("Render preview requested");
    return true;
}

const char* GetCameras() {
    g_state.lastResponse = QString::fromUtf8(CamerasData().c_str());
    return g_state.lastResponse.toUtf8().constData();
}

const char* ExecuteCommand(const char* command, const char* args) {
    std::string request = "{\"id\":\"direct\",\"command\":\"";
    request += command ? command : "";
    request += "\",\"args\":";
    request += args ? args : "{}";
    request += "}";
    g_state.lastResponse = QString::fromUtf8(DispatchRequest(request).c_str()).trimmed();
    return g_state.lastResponse.toUtf8().constData();
}

const char* CaptureViewport() {
    QString path = dzApp ? dzApp->getTempRenderFilename() : "";
    if (!CaptureActiveViewport(path).empty()) {
        g_state.lastResponse = QString("{\"status\":\"ok\",\"path\":\"%1\"}").arg(path);
    } else {
        g_state.lastResponse = "{\"status\":\"error\",\"error\":\"Viewport capture failed\"}";
    }
    return g_state.lastResponse.toUtf8().constData();
}

class DazPilotBridgeDzPlugin : public DzPlugin {
public:
    DazPilotBridgeDzPlugin()
        : DzPlugin(
              "DazPilot Bridge",
              "DazPilot",
              "TCP bridge for DazPilot scene editing and viewport sync.",
              0,
              5,
              3,
              0) {}

protected:
    void startup() override { PluginInitialize(); }
    void shutdown() override { PluginCleanup(); }
};

DZ_CUSTOM_PLUGIN_DEFINITION(DazPilotBridgeDzPlugin);
DZ_PLUGIN_CLASS_GUID(DazPilotPane, 2D5B8E01-A301-48CD-AF81-C3BB80EC4AA6);
DZ_PLUGIN_CLASS_GUID(DazPilotPhyModifier, F9EC5E01-A301-48CD-AF81-C3BB80EC4AA6);
DZ_PLUGIN_CLASS_GUID(DazPilotPhyModifierIO, 1C884DA8-6C3C-4364-81B4-272501D5DDD8);
DZ_PLUGIN_REGISTER_MODIFIER_EXTRA_OBJECT_IO("dazpilot_phy", DazPilotPhyModifierIO, DazPilotPhyModifier);
