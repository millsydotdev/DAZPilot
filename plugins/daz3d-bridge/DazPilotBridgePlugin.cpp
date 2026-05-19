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
#include <QtCore/QBuffer>
#include <QtCore/QByteArray>
#include <QtCore/QFile>
#include <QtCore/QJsonDocument>
#include <QtCore/QJsonObject>
#include <cstdlib>

#include <atomic>
#include <iostream>
#include <sstream>
#include <string>
#include <thread>
#include <mutex>
#include <condition_variable>
#include <cstring>
#include <QtCore/QCoreApplication>
#include <QtCore/QEvent>
#include <iostream>
#include <sstream>
#include <string>
#include <thread>

#include "dzpane.h"
#include <QtGui/QLabel>
#include <QtGui/QVBoxLayout>
#include <QtGui/QTextEdit>

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

static DazPilotBridgeState g_state = {nullptr, QList<QTcpSocket*>(), "127.0.0.1", 8765, ""};
static std::atomic<bool> g_serverRunning(false);
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
                        DzFileIOSettings ioSettings;
                        ioSettings.setBoolValue("RunSilent", true);
                        
                        // Default to obj export options that are widely compatible
                        ioSettings.setFloatValue("Scale", 1.0f); // 100%
                        ioSettings.setBoolValue("LatAxis", true); // Y Up
                        
                        // Parse settingsJson for selected_only
                        bool selectedOnly = false;
                        if (ese->settingsJson.contains("\"selected_only\":true") || ese->settingsJson.contains("\"selected_only\": true")) {
                            selectedOnly = true;
                        }
                        ioSettings.setBoolValue("SelectedOnly", selectedOnly);
                        
                        // Write materials (OBJ / FBX)
                        bool includeMaterials = true;
                        if (ese->settingsJson.contains("\"include_materials\":false") || ese->settingsJson.contains("\"include_materials\": false")) {
                            includeMaterials = false;
                        }
                        ioSettings.setBoolValue("WriteMaterial", includeMaterials);
                        ioSettings.setBoolValue("WriteTextures", includeMaterials);
                        ioSettings.setBoolValue("EmbedTextures", includeMaterials);
                        
                        // Write animations (FBX)
                        bool includeAnimations = true;
                        if (ese->settingsJson.contains("\"include_animations\":false") || ese->settingsJson.contains("\"include_animations\": false")) {
                            includeAnimations = false;
                        }
                        ioSettings.setBoolValue("WriteAnimations", includeAnimations);
                        
                        // Bake textures
                        bool bakeTextures = false;
                        if (ese->settingsJson.contains("\"bake_textures\":true") || ese->settingsJson.contains("\"bake_textures\": true")) {
                            bakeTextures = true;
                        }
                        ioSettings.setBoolValue("BakeTextures", bakeTextures);

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
    
    // Skip whitespace
    size_t valStart = colon + 1;
    while (valStart < json.size() && (json[valStart] == ' ' || json[valStart] == '\t' || json[valStart] == '\r' || json[valStart] == '\n')) {
        valStart++;
    }
    if (valStart >= json.size()) return "";
    
    char firstChar = json[valStart];
    if (firstChar == '"') {
        // String value
        size_t firstQuote = valStart;
        size_t secondQuote = firstQuote + 1;
        bool escaped = false;
        for (; secondQuote < json.size(); ++secondQuote) {
            char ch = json[secondQuote];
            if (escaped) {
                escaped = false;
                continue;
            }
            if (ch == '\\') {
                escaped = true;
                continue;
            }
            if (ch == '"') break;
        }
        if (secondQuote >= json.size()) return "";
        return QString::fromUtf8(json.substr(firstQuote + 1, secondQuote - firstQuote - 1).c_str());
    } else if (firstChar == '{' || firstChar == '[') {
        // Object or Array value: match braces/brackets
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
                        idx++; // include the closing brace/bracket
                        break;
                    }
                }
            }
        }
        if (depth != 0 || idx > json.size()) return "";
        return QString::fromUtf8(json.substr(valStart, idx - valStart).c_str());
    } else {
        // Number, Boolean, or Null value
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
        "{\"name\":\"get_node_properties\",\"description\":\"Get animatable properties of a node\",\"category\":\"Properties\",\"parameters\":[\"node_id\"]},"
        "{\"name\":\"load_asset\",\"description\":\"Load Daz asset\",\"category\":\"Assets\",\"parameters\":[\"path\"]},"
        "{\"name\":\"apply_pose\",\"description\":\"Apply pose file\",\"category\":\"Pose\",\"parameters\":[\"pose_path\",\"figure_id\"]},"
        "{\"name\":\"render_preview\",\"description\":\"Trigger preview render\",\"category\":\"Render\",\"parameters\":[]},"
        "{\"name\":\"capture_viewport\",\"description\":\"Capture viewport\",\"category\":\"Viewport\",\"parameters\":[\"path\"]},"
        "{\"name\":\"import_model\",\"description\":\"Import model if Daz import support is available\",\"category\":\"Assets\",\"parameters\":[\"path\",\"settings\"]},"
        "{\"name\":\"export_scene\",\"description\":\"Export scene if Daz export support is available\",\"category\":\"Assets\",\"parameters\":[\"node_id\",\"path\",\"settings\"]},"
        "{\"name\":\"begin_undo_batch\",\"description\":\"Start a new undo batch\",\"category\":\"Scene\",\"parameters\":[]},"
        "{\"name\":\"accept_undo_batch\",\"description\":\"Accept the current undo batch\",\"category\":\"Scene\",\"parameters\":[\"caption\"]},"
        "{\"name\":\"cancel_undo_batch\",\"description\":\"Cancel the current undo batch\",\"category\":\"Scene\",\"parameters\":[]},"
        "{\"name\":\"get_bounding_boxes\",\"description\":\"Get world-space 3D bounding boxes of all scene nodes\",\"category\":\"Vision\",\"parameters\":[]},"
        "{\"name\":\"run_script\",\"description\":\"Evaluate arbitrary DazScript\",\"category\":\"Scripting\",\"parameters\":[\"script\",\"args\"]},"
        "{\"name\":\"get_scene_assets\",\"description\":\"List loaded scene node labels\",\"category\":\"Scene\",\"parameters\":[]},"
        "{\"name\":\"add_figure\",\"description\":\"Add Genesis figure (requires path or content)\",\"category\":\"Scene\",\"parameters\":[\"figure_type\",\"path\"]},"
        "{\"name\":\"set_morph\",\"description\":\"Set morph dial value\",\"category\":\"Properties\",\"parameters\":[\"node_id\",\"morph\",\"value\"]},"
        "{\"name\":\"set_light\",\"description\":\"Set light property\",\"category\":\"Lighting\",\"parameters\":[\"node_id\",\"property\",\"value\"]},"
        "{\"name\":\"set_render_settings\",\"description\":\"Set render image size\",\"category\":\"Render\",\"parameters\":[\"width\",\"height\"]}"
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
        return OkResponse(id, "{\"requested\":true}");
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
        QString posePath = ExtractArgString(line, "pose_path");
        if (OpenContentFile(posePath, true)) {
            return OkResponse(id, std::string("{\"pose_path\":\"") + JsonEscape(posePath) + "\"}");
        }
        return ErrorResponse(id, QString("Pose application failed: %1").arg(posePath));
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
        if (!path.isEmpty()) {
            if (OpenContentFile(path, true)) {
                return OkResponse(id, std::string("{\"path\":\"") + JsonEscape(path) + "\"}");
            }
            return ErrorResponse(id, QString("Figure load failed: %1").arg(path));
        }
        return ErrorResponse(id, "add_figure requires a content path; index library and use load_asset or pass path");
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

    return ErrorResponse(id, QString("Unknown command: %1").arg(command));
}

static void BridgeServerLoop() {
#ifdef _WIN32
    WSADATA wsaData;
    if (WSAStartup(MAKEWORD(2, 2), &wsaData) != 0) {
        std::cout << "[DazPilotBridge] WSAStartup failed" << std::endl;
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
        std::cout << "[DazPilotBridge] Failed to bind " << g_state.host.toStdString() << ":" << g_state.port << std::endl;
        CloseBridgeSocket(g_listenSocket);
        g_listenSocket = INVALID_BRIDGE_SOCKET;
#ifdef _WIN32
        WSACleanup();
#endif
        return;
    }

    listen(g_listenSocket, SOMAXCONN);
    std::cout << "[DazPilotBridge] Listening on " << g_state.host.toStdString() << ":" << g_state.port << std::endl;

    while (g_serverRunning.load()) {
        BridgeSocket client = accept(g_listenSocket, nullptr, nullptr);
        if (client == INVALID_BRIDGE_SOCKET) continue;

        std::string line;
        char ch;
        while (recv(client, &ch, 1, 0) == 1) {
            if (ch == '\n') break;
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
const char* GetPluginVersion() { return "1.0.0"; }
int GetPluginType() { return 1; }

bool PluginInitialize() {
    g_scriptExecutor = new ScriptExecutor();
    
    // Resolve bridge_config.json path dynamically from shared OS AppData
    QString configPath;
#ifdef _WIN32
    char* appdata = getenv("APPDATA");
    if (appdata) {
        configPath = QString(appdata) + "/com.dazpilot.app/bridge_config.json";
    }
#else
    char* home = getenv("HOME");
    if (home) {
        configPath = QString(home) + "/Library/Application Support/com.dazpilot.app/bridge_config.json";
    }
#endif

    if (!configPath.isEmpty()) {
        QFile file(configPath);
        if (file.open(QIODevice::ReadOnly)) {
            QByteArray data = file.readAll();
            QJsonDocument doc = QJsonDocument::fromJson(data);
            if (!doc.isNull() && doc.isObject()) {
                QJsonObject obj = doc.object();
                if (obj.contains("port")) {
                    g_state.port = obj.value("port").toInt(8765);
                }
                if (obj.contains("host")) {
                    g_state.host = obj.value("host").toString("127.0.0.1");
                }
            }
            file.close();
        }
    }
    
    g_serverRunning = true;
    g_serverThread = std::thread(BridgeServerLoop);
    std::cout << "[DazPilotBridge] Plugin initialized. Listening on "
              << g_state.host.toStdString() << ":" << g_state.port << std::endl;
    return true;
}

void PluginCleanup() {
    DisconnectFromDazPilot();
    if (g_scriptExecutor) {
        delete g_scriptExecutor;
        g_scriptExecutor = nullptr;
    }
    std::cout << "[DazPilotBridge] Plugin cleanup complete" << std::endl;
}

const char* GetMenuName() { return "DazPilot Bridge"; }

void ExecuteMenuAction(const char* action) {
    std::cout << "[DazPilotBridge] Menu action: " << (action ? action : "") << std::endl;
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
    Q_UNUSED(figureId);
    return OpenContentFile(QString(poseFile ? poseFile : ""), true);
}

bool RenderPreview() {
    std::cout << "[DazPilotBridge] Render preview requested" << std::endl;
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
              1,
              0,
              0,
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
