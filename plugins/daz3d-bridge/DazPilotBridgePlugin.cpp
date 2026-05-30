#include "DazPilotBridgePlugin.h"
#include "dzplugin.h"
#include "dzundostack.h"
#include "dzexportmgr.h"
#include "dzfloatproperty.h"
#include "dzenumproperty.h"
#include "dznumericproperty.h"
#include "dzselectionmap.h"
#include "dzfacetmesh.h"
#include "dzvertexmesh.h"
#include "dzinstancenode.h"
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
#include <QtCore/QTimer>

#include "dzpane.h"
#include <QtGui/QApplication>
#include <QtGui/QClipboard>
#include <QtGui/QHBoxLayout>
#include <QtGui/QLabel>
#include <QtGui/QPushButton>
#include <QtGui/QVBoxLayout>
#include <QtGui/QTextEdit>

static DazPilotBridgeState g_state = {nullptr, QList<QTcpSocket*>(), "127.0.0.1", 8765, ""};
static std::atomic<bool> g_serverRunning(false);
static QString g_lastAppInboxStatus = "No context sent yet.";

static QString SendViewportContextToAppInbox();

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

        m_bridgeStatusLabel = new QLabel("Bridge: Stopped", this);
        layout->addWidget(m_bridgeStatusLabel);

        m_appStatusLabel = new QLabel("DazPilot app: Waiting for handoff", this);
        layout->addWidget(m_appStatusLabel);

        m_lastResultLabel = new QLabel(g_lastAppInboxStatus, this);
        m_lastResultLabel->setWordWrap(true);
        layout->addWidget(m_lastResultLabel);

        QPushButton* refreshBtn = new QPushButton("Refresh Status", this);
        connect(refreshBtn, SIGNAL(clicked()), this, SLOT(refreshStatus()));
        layout->addWidget(refreshBtn);

        QPushButton* sendBtn = new QPushButton("Send Viewport to DazPilot", this);
        connect(sendBtn, SIGNAL(clicked()), this, SLOT(sendViewportToDazPilot()));
        layout->addWidget(sendBtn);

        QPushButton* copyBtn = new QPushButton("Copy Diagnostics", this);
        connect(copyBtn, SIGNAL(clicked()), this, SLOT(copyDiagnostics()));
        layout->addWidget(copyBtn);

        layout->addWidget(new QLabel("Log:", this));
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
            m_bridgeStatusLabel->setText(QString("Bridge: Listening on %1:%2")
                .arg(g_state.host).arg(g_state.port));
            m_bridgeStatusLabel->setStyleSheet("color: #00aa55;");
        } else {
            m_bridgeStatusLabel->setText("Bridge: Stopped");
            m_bridgeStatusLabel->setStyleSheet("color: #cc3333;");
        }
        m_appStatusLabel->setText(g_lastAppInboxStatus.startsWith("Sent")
            ? "DazPilot app: Connected"
            : "DazPilot app: Waiting for handoff");
        m_lastResultLabel->setText(g_lastAppInboxStatus);
    }

    void refreshStatus() {
        updateStatus();
        m_logArea->append("Status refreshed.");
    }

    void sendViewportToDazPilot() {
        g_lastAppInboxStatus = SendViewportContextToAppInbox();
        updateStatus();
        m_logArea->append(g_lastAppInboxStatus);
    }

    void copyDiagnostics() {
        QString diagnostics = QString("DazPilot Bridge\nBridge running: %1\nHost: %2\nPort: %3\nApp inbox: 127.0.0.1:8766\nLast result: %4")
            .arg(g_serverRunning.load() ? "yes" : "no")
            .arg(g_state.host)
            .arg(g_state.port)
            .arg(g_lastAppInboxStatus);
        QApplication::clipboard()->setText(diagnostics);
        m_logArea->append("Diagnostics copied.");
    }

private:
    QLabel* m_bridgeStatusLabel;
    QLabel* m_appStatusLabel;
    QLabel* m_lastResultLabel;
    QTextEdit* m_logArea;
};

class DazPilotPaneAction : public DzPaneAction {
    Q_OBJECT
public:
    DazPilotPaneAction() : DzPaneAction("DazPilotPane") {}

    QString getActionGroup() const override { return tr("DazPilot"); }
    QString getDefaultMenuPath() const override { return tr("&DazPilot"); }
    QString getDefaultToolBar() const override { return tr("DazPilot"); }
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

static QString SendViewportContextToAppInbox() {
    QString capturePath = QDir::temp().filePath(
        QString("dazpilot_viewport_%1.png").arg(QDateTime::currentMSecsSinceEpoch()));
    std::string captureJson = CaptureActiveViewport(capturePath);
    bool captured = !captureJson.empty();

    QString sceneInfo = QString::fromUtf8(SceneInfoData().c_str());
    QString payload = QString(
        "{\"request_type\":\"analyze_context\","
        "\"context_scope\":\"viewport\","
        "\"context_label\":\"Viewport\","
        "\"payload_id\":\"%1\","
        "\"summary\":\"Viewport context from DAZ Studio\","
        "\"payload\":{"
            "\"scene\":%2,"
            "\"viewport_capture_path\":\"%3\","
            "\"viewport_captured\":%4,"
            "\"plugin_version\":\"%5\","
            "\"bridge_host\":\"%6\","
            "\"bridge_port\":%7"
        "}}")
        .arg(captured ? capturePath : QString("none"))
        .arg(sceneInfo)
        .arg(JsonEscape(captured ? capturePath : QString()).c_str())
        .arg(captured ? "true" : "false")
        .arg(GetPluginVersion())
        .arg(g_state.host)
        .arg(g_state.port);

    BridgeSocket sock = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    if (sock == INVALID_BRIDGE_SOCKET) {
        return "App inbox unavailable: could not create socket.";
    }

    sockaddr_in addr;
    std::memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_port = htons(8766);
    addr.sin_addr.s_addr = inet_addr("127.0.0.1");

    if (::connect(sock, reinterpret_cast<sockaddr*>(&addr), sizeof(addr)) != 0) {
        CloseBridgeSocket(sock);
        return "App inbox unavailable: start DazPilot desktop app.";
    }

    QByteArray bytes = payload.toUtf8();
    int sent = send(sock, bytes.constData(), static_cast<int>(bytes.size()), 0);
    CloseBridgeSocket(sock);
    if (sent <= 0) {
        return "App inbox unavailable: send failed.";
    }

    return captured
        ? QString("Sent viewport context to DazPilot: %1").arg(capturePath)
        : "Sent DAZ session context to DazPilot without viewport capture.";
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
        "{\"name\":\"set_viewport_mode\",\"description\":\"Set viewport display mode (texture, shaded, wireframe, lit_wireframe, hidden_line, smooth_lit)\",\"category\":\"Viewport\",\"parameters\":[\"mode\"]},"
        "{\"name\":\"set_visibility\",\"description\":\"Show or hide a scene node\",\"category\":\"Scene\",\"parameters\":[\"node_id\",\"visible\"]},"
        "{\"name\":\"delete_nodes\",\"description\":\"Delete multiple nodes from the scene\",\"category\":\"Scene\",\"parameters\":[\"node_ids\"]},"
        "{\"name\":\"duplicate_nodes\",\"description\":\"Duplicate one or more nodes\",\"category\":\"Scene\",\"parameters\":[\"node_ids\",\"copies\"]},"
        "{\"name\":\"rename_node\",\"description\":\"Rename a scene node\",\"category\":\"Scene\",\"parameters\":[\"node_id\",\"new_name\"]},"
        "{\"name\":\"group_nodes\",\"description\":\"Parent nodes under a new null node\",\"category\":\"Scene\",\"parameters\":[\"parent_id\",\"child_ids\"]},"
        "{\"name\":\"merge_scene\",\"description\":\"Merge a scene file into the current scene\",\"category\":\"Scene\",\"parameters\":[\"filepath\"]},"
        "{\"name\":\"get_scene_stats\",\"description\":\"Get scene statistics: node counts by type\",\"category\":\"Scene\",\"parameters\":[]},"
        "{\"name\":\"list_figures\",\"description\":\"List all figure nodes in the scene\",\"category\":\"Figure\",\"parameters\":[\"include_details\"]},"
        "{\"name\":\"remove_figure\",\"description\":\"Remove a figure from the scene\",\"category\":\"Figure\",\"parameters\":[\"figure_id\"]},"
        "{\"name\":\"apply_figure_preset\",\"description\":\"Apply a figure/character preset\",\"category\":\"Figure\",\"parameters\":[\"figure_id\",\"preset_path\"]},"
        "{\"name\":\"list_props\",\"description\":\"List all props in the scene (non-figure, non-light, non-camera nodes)\",\"category\":\"Props\",\"parameters\":[\"category\"]},"
        "{\"name\":\"load_prop\",\"description\":\"Load a prop from the content library\",\"category\":\"Props\",\"parameters\":[\"name\",\"category\",\"position\"]},"
        "{\"name\":\"position_prop\",\"description\":\"Set a prop's world-space position\",\"category\":\"Props\",\"parameters\":[\"node_id\",\"position\"]},"
        "{\"name\":\"rotate_prop\",\"description\":\"Set a prop's world-space rotation (quaternion)\",\"category\":\"Props\",\"parameters\":[\"node_id\",\"rotation\"]},"
        "{\"name\":\"scale_prop\",\"description\":\"Set a prop's scale (uniform or per-axis)\",\"category\":\"Props\",\"parameters\":[\"node_id\",\"scale\"]},"
        "{\"name\":\"select_all\",\"description\":\"Select all nodes in the scene\",\"category\":\"Selection\",\"parameters\":[]},"
        "{\"name\":\"deselect_all\",\"description\":\"Deselect all nodes\",\"category\":\"Selection\",\"parameters\":[]},"
        "{\"name\":\"invert_selection\",\"description\":\"Invert the current node selection\",\"category\":\"Selection\",\"parameters\":[]},"
        "{\"name\":\"select_children\",\"description\":\"Select all children of a node\",\"category\":\"Selection\",\"parameters\":[\"node_id\"]},"
        "{\"name\":\"select_parent\",\"description\":\"Select the parent of the current selection\",\"category\":\"Selection\",\"parameters\":[\"node_id\"]},"
        "{\"name\":\"get_selection_count\",\"description\":\"Get the number of selected nodes\",\"category\":\"Selection\",\"parameters\":[]},"
        "{\"name\":\"create_camera\",\"description\":\"Create a new camera in the scene\",\"category\":\"Camera\",\"parameters\":[\"name\",\"focal_length\",\"f_stop\"]},"
        "{\"name\":\"delete_camera\",\"description\":\"Delete a camera from the scene\",\"category\":\"Camera\",\"parameters\":[\"camera_name\"]},"
        "{\"name\":\"set_camera_target\",\"description\":\"Set camera aim/focus point\",\"category\":\"Camera\",\"parameters\":[\"camera_name\",\"target\"]},"
        "{\"name\":\"get_camera_properties\",\"description\":\"Get detailed properties of a camera\",\"category\":\"Camera\",\"parameters\":[\"camera_name\"]},"
        "{\"name\":\"render\",\"description\":\"Start a full render\",\"category\":\"Render\",\"parameters\":[\"mode\",\"quality\",\"width\",\"height\"]},"
        "{\"name\":\"cancel_render\",\"description\":\"Cancel the current render\",\"category\":\"Render\",\"parameters\":[]},"
        "{\"name\":\"set_render_engine\",\"description\":\"Set the active render engine\",\"category\":\"Render\",\"parameters\":[\"engine\",\"use_gpu\"]},"
        "{\"name\":\"set_render_output\",\"description\":\"Set render output path and format\",\"category\":\"Render\",\"parameters\":[\"format\",\"path\",\"filename\"]},"
        "{\"name\":\"set_resolution\",\"description\":\"Set render resolution (width x height)\",\"category\":\"Render\",\"parameters\":[\"width\",\"height\"]},"
        "{\"name\":\"set_denoising\",\"description\":\"Enable/configure denoising for renders\",\"category\":\"Render\",\"parameters\":[\"enabled\",\"strength\",\"mode\"]},"
        "{\"name\":\"render_region\",\"description\":\"Render a specific region of the viewport\",\"category\":\"Render\",\"parameters\":[\"x\",\"y\",\"width\",\"height\",\"quality\"]},"
        "{\"name\":\"queue_render\",\"description\":\"Queue a render pass\",\"category\":\"Render\",\"parameters\":[\"pass_name\"]},"
        "{\"name\":\"export_fbx\",\"description\":\"Export scene to FBX format\",\"category\":\"Export\",\"parameters\":[\"filepath\"]},"
        "{\"name\":\"export_obj\",\"description\":\"Export scene to OBJ format\",\"category\":\"Export\",\"parameters\":[\"filepath\"]},"
        "{\"name\":\"export_gltf\",\"description\":\"Export scene to glTF format\",\"category\":\"Export\",\"parameters\":[\"filepath\",\"binary\"]},"
        "{\"name\":\"export_collada\",\"description\":\"Export scene to Collada/DAE format\",\"category\":\"Export\",\"parameters\":[\"filepath\"]},"
        "{\"name\":\"export_usd\",\"description\":\"Export scene to USD format\",\"category\":\"Export\",\"parameters\":[\"filepath\"]},"
        "{\"name\":\"export_selected\",\"description\":\"Export only selected nodes\",\"category\":\"Export\",\"parameters\":[\"filepath\",\"format\"]},"
        "{\"name\":\"batch_export\",\"description\":\"Batch export multiple nodes to a directory\",\"category\":\"Export\",\"parameters\":[\"dir\",\"format\"]},"
        "{\"name\":\"export_animation\",\"description\":\"Export animation to file\",\"category\":\"Export\",\"parameters\":[\"filepath\",\"figure_id\"]},"
        "{\"name\":\"set_display_mode\",\"description\":\"Set viewport display mode\",\"category\":\"Viewport\",\"parameters\":[\"mode\",\"viewport\"]},"
        "{\"name\":\"set_viewport_quality\",\"description\":\"Set viewport quality and texture resolution\",\"category\":\"Viewport\",\"parameters\":[\"quality\",\"texture_resolution\",\"anti_aliasing\"]},"
        "{\"name\":\"toggle_guide\",\"description\":\"Toggle viewport guide visibility\",\"category\":\"Viewport\",\"parameters\":[\"guide\",\"show\"]},"
        "{\"name\":\"set_viewport_camera\",\"description\":\"Set viewport to use a specific camera\",\"category\":\"Viewport\",\"parameters\":[\"camera\",\"viewport\"]},"
        "{\"name\":\"set_viewport_lighting\",\"description\":\"Set viewport lighting mode\",\"category\":\"Viewport\",\"parameters\":[\"lighting\",\"ambient_intensity\"]},"
        "{\"name\":\"center_view\",\"description\":\"Center viewport on a node\",\"category\":\"Viewport\",\"parameters\":[\"node_id\"]},"
        "{\"name\":\"set_environment\",\"description\":\"Set environment/HDRI\",\"category\":\"Environment\",\"parameters\":[\"type\",\"preset\",\"intensity\",\"rotation\"]},"
        "{\"name\":\"add_ground\",\"description\":\"Add ground plane to scene\",\"category\":\"Environment\",\"parameters\":[\"type\",\"size\"]},"
        "{\"name\":\"set_fog\",\"description\":\"Set fog in the scene\",\"category\":\"Environment\",\"parameters\":[\"enabled\",\"density\",\"color\",\"distance\"]},"
        "{\"name\":\"set_sun\",\"description\":\"Set sun direction and intensity\",\"category\":\"Environment\",\"parameters\":[\"direction\",\"intensity\"]},"
        "{\"name\":\"set_time_of_day\",\"description\":\"Set environment time of day\",\"category\":\"Environment\",\"parameters\":[\"time\"]},"
        "{\"name\":\"add_env_light\",\"description\":\"Add an environment fill light\",\"category\":\"Environment\",\"parameters\":[\"type\",\"intensity\"]},"
        "{\"name\":\"rotate_environment\",\"description\":\"Rotate environment/HDRI\",\"category\":\"Environment\",\"parameters\":[\"rotation\"]},"
        "{\"name\":\"get_environment_info\",\"description\":\"Get current environment info\",\"category\":\"Environment\",\"parameters\":[]},"
        "{\"name\":\"clear_environment\",\"description\":\"Clear environment settings\",\"category\":\"Environment\",\"parameters\":[\"hdri\",\"ground\",\"fog\"]},"
        "{\"name\":\"list_poses\",\"description\":\"List available pose presets\",\"category\":\"Pose\",\"parameters\":[]},"
        "{\"name\":\"save_pose\",\"description\":\"Save current pose as a preset\",\"category\":\"Pose\",\"parameters\":[\"figure_id\",\"name\"]},"
        "{\"name\":\"blend_poses\",\"description\":\"Blend two poses on a figure\",\"category\":\"Pose\",\"parameters\":[\"figure_id\",\"pose_a\",\"pose_b\",\"blend\"]},"
        "{\"name\":\"mirror_pose\",\"description\":\"Mirror pose from one side to the other\",\"category\":\"Pose\",\"parameters\":[\"figure_id\"]},"
        "{\"name\":\"asymmetric_pose\",\"description\":\"Apply different poses to left/right sides\",\"category\":\"Pose\",\"parameters\":[\"figure_id\",\"left\",\"right\"]},"
        "{\"name\":\"reset_pose\",\"description\":\"Reset figure to default pose\",\"category\":\"Pose\",\"parameters\":[\"figure_id\",\"pose_type\"]},"
        "{\"name\":\"random_pose\",\"description\":\"Apply a random pose to a figure\",\"category\":\"Pose\",\"parameters\":[\"figure_id\",\"category\",\"intensity\"]},"
        "{\"name\":\"batch_set_morphs\",\"description\":\"Set multiple morph values at once\",\"category\":\"Morphs\",\"parameters\":[\"figure_id\",\"morphs\"]},"
        "{\"name\":\"symmetry_morphs\",\"description\":\"Mirror morph values symmetrically\",\"category\":\"Morphs\",\"parameters\":[\"figure_id\",\"direction\"]},"
        "{\"name\":\"randomize_morphs\",\"description\":\"Randomize morph values on a figure\",\"category\":\"Morphs\",\"parameters\":[\"figure_id\",\"intensity\"]},"
        "{\"name\":\"save_morph_preset\",\"description\":\"Save current morph values as a preset\",\"category\":\"Morphs\",\"parameters\":[\"figure_id\",\"preset_name\"]},"
        "{\"name\":\"load_morph_preset\",\"description\":\"Load a morph preset onto a figure\",\"category\":\"Morphs\",\"parameters\":[\"figure_id\",\"preset_name\",\"blend\"]},"
        "{\"name\":\"reset_morphs\",\"description\":\"Reset all morph values on a figure\",\"category\":\"Morphs\",\"parameters\":[\"figure_id\"]},"
        "{\"name\":\"load_hair\",\"description\":\"Load a hair asset onto a figure\",\"category\":\"Hair\",\"parameters\":[\"name\",\"figure_id\"]},"
        "{\"name\":\"style_hair\",\"description\":\"Apply a hair style preset\",\"category\":\"Hair\",\"parameters\":[\"hair_id\",\"preset\"]},"
        "{\"name\":\"set_hair_color\",\"description\":\"Set hair color\",\"category\":\"Hair\",\"parameters\":[\"hair_id\",\"color\",\"highlights\"]},"
        "{\"name\":\"apply_hair_physics\",\"description\":\"Enable/configure hair physics\",\"category\":\"Hair\",\"parameters\":[\"hair_id\",\"enable\",\"stiffness\"]},"
        "{\"name\":\"set_hair_length\",\"description\":\"Set hair length (scale)\",\"category\":\"Hair\",\"parameters\":[\"hair_id\",\"length\",\"scale_factor\"]},"
        "{\"name\":\"set_hair_volume\",\"description\":\"Set hair volume\",\"category\":\"Hair\",\"parameters\":[\"hair_id\",\"volume\"]},"
        "{\"name\":\"list_hair_presets\",\"description\":\"Search content library for hair presets\",\"category\":\"Hair\",\"parameters\":[]},"
        "{\"name\":\"remove_hair\",\"description\":\"Remove a hair asset from the scene\",\"category\":\"Hair\",\"parameters\":[\"hair_id\"]},"
        "{\"name\":\"set_hair_shader\",\"description\":\"Set the hair shader type and gloss\",\"category\":\"Hair\",\"parameters\":[\"hair_id\",\"shader_type\",\"gloss\"]},"
        "{\"name\":\"apply_hair_preset\",\"description\":\"Apply a full hair preset (color + style)\",\"category\":\"Hair\",\"parameters\":[\"hair_id\",\"preset\"]},"
        "{\"name\":\"load_clothing\",\"description\":\"Load a clothing item onto a figure\",\"category\":\"Clothing\",\"parameters\":[\"name\",\"figure_id\",\"fit_mode\"]},"
        "{\"name\":\"fit_clothing\",\"description\":\"Auto-fit clothing to a figure\",\"category\":\"Clothing\",\"parameters\":[\"clothing_id\",\"figure_id\",\"fit_type\"]},"
        "{\"name\":\"remove_clothing\",\"description\":\"Remove a clothing item from the scene\",\"category\":\"Clothing\",\"parameters\":[\"clothing_id\"]},"
        "{\"name\":\"list_worn_items\",\"description\":\"List all clothing items worn by a figure\",\"category\":\"Clothing\",\"parameters\":[\"figure_id\"]},"
        "{\"name\":\"set_clothing_params\",\"description\":\"Set clothing adjustment parameters\",\"category\":\"Clothing\",\"parameters\":[\"clothing_id\",\"parameter\",\"value\"]},"
        "{\"name\":\"suggest_outfit\",\"description\":\"Search content library for outfit suggestions matching a style\",\"category\":\"Clothing\",\"parameters\":[\"figure_id\",\"style\"]},"
        "{\"name\":\"simulate_physics\",\"description\":\"Run physics simulation (dForce)\",\"category\":\"Physics\",\"parameters\":[\"node_ids\",\"frames\",\"start_frame\"]},"
        "{\"name\":\"set_wind\",\"description\":\"Set wind direction/speed for physics\",\"category\":\"Physics\",\"parameters\":[\"direction\",\"speed\",\"turbulence\"]},"
        "{\"name\":\"set_gravity\",\"description\":\"Set gravity strength and direction\",\"category\":\"Physics\",\"parameters\":[\"strength\",\"direction\",\"node_id\"]},"
        "{\"name\":\"add_collision\",\"description\":\"Add collision to a node for physics\",\"category\":\"Physics\",\"parameters\":[\"node_id\",\"shape\",\"friction\"]},"
        "{\"name\":\"bake_physics\",\"description\":\"Bake physics simulation to keyframes\",\"category\":\"Physics\",\"parameters\":[\"node_id\",\"sample_rate\"]},"
        "{\"name\":\"set_physics_props\",\"description\":\"Set physics properties on a node\",\"category\":\"Physics\",\"parameters\":[\"node_id\",\"mass\",\"stiffness\",\"damping\"]},"
        "{\"name\":\"remove_physics\",\"description\":\"Remove physics from a node\",\"category\":\"Physics\",\"parameters\":[\"node_id\",\"remove_modifiers\"]},"
        "{\"name\":\"get_joint_list\",\"description\":\"Get list of joints in a figure's skeleton\",\"category\":\"Rigging\",\"parameters\":[\"figure_id\"]},"
        "{\"name\":\"set_joint_rotation\",\"description\":\"Set a joint's rotation\",\"category\":\"Rigging\",\"parameters\":[\"figure_id\",\"joint\",\"rotation\"]},"
        "{\"name\":\"set_ik_fk_blend\",\"description\":\"Set IK/FK blend for a limb\",\"category\":\"Rigging\",\"parameters\":[\"figure_id\",\"limb\",\"blend\"]},"
        "{\"name\":\"add_joint\",\"description\":\"Add a custom joint to a figure\",\"category\":\"Rigging\",\"parameters\":[\"figure_id\",\"joint_name\",\"parent_joint\"]},"
        "{\"name\":\"set_transform\",\"description\":\"Set node transform (pos/rot/scale)\",\"category\":\"Transform\",\"parameters\":[\"node_id\",\"position\",\"rotation\",\"scale\"]},"
        "{\"name\":\"reset_transform\",\"description\":\"Reset node transform to defaults\",\"category\":\"Transform\",\"parameters\":[\"node_id\"]},"
        "{\"name\":\"align_nodes\",\"description\":\"Align nodes to a target\",\"category\":\"Transform\",\"parameters\":[\"target_node\",\"axes\",\"nodes\"]},"
        "{\"name\":\"distribute_nodes\",\"description\":\"Distribute nodes along an axis\",\"category\":\"Transform\",\"parameters\":[\"axis\",\"spacing\",\"nodes\"]},"
        "{\"name\":\"snap_to_ground\",\"description\":\"Snap a node to the ground plane\",\"category\":\"Transform\",\"parameters\":[\"node_id\"]},"
        "{\"name\":\"set_pivot\",\"description\":\"Set a node's pivot point\",\"category\":\"Transform\",\"parameters\":[\"node_id\",\"pivot\"]},"
        "{\"name\":\"select_by_type\",\"description\":\"Select all nodes of a given type\",\"category\":\"Selection\",\"parameters\":[\"type\"]},"
        "{\"name\":\"select_hierarchy\",\"description\":\"Select a node and all its children\",\"category\":\"Selection\",\"parameters\":[\"node_id\"]},"
        "{\"name\":\"save_selection\",\"description\":\"Save named selection set\",\"category\":\"Selection\",\"parameters\":[\"name\"]},"
        "{\"name\":\"load_selection\",\"description\":\"Load named selection set\",\"category\":\"Selection\",\"parameters\":[\"name\"]},"
        "{\"name\":\"viewport_click\",\"description\":\"Pick and select a node in the viewport at given coordinates\",\"category\":\"Viewport\",\"parameters\":[\"x\",\"y\"]},"
        "{\"name\":\"get_material_properties\",\"description\":\"Get material properties of a node\",\"category\":\"Materials\",\"parameters\":[\"node_id\"]},"
        "{\"name\":\"delete_node\",\"description\":\"Delete a single node from the scene\",\"category\":\"Scene\",\"parameters\":[\"node_id\"]},"
        "{\"name\":\"get_geoshells\",\"description\":\"Get all Geometry Shells in the scene\",\"category\":\"Scene\",\"parameters\":[]},"
        "{\"name\":\"apply_phy_modifier\",\"description\":\"Apply DazPilot physics modifier to a node\",\"category\":\"Physics\",\"parameters\":[\"node_id\",\"stiffness\",\"damping\",\"mass\"]},"
        "{\"name\":\"remove_phy_modifier\",\"description\":\"Remove DazPilot physics modifier from a node\",\"category\":\"Physics\",\"parameters\":[\"node_id\"]},"
        "{\"name\":\"set_phy_modifier_params\",\"description\":\"Update DazPilot physics modifier parameters\",\"category\":\"Physics\",\"parameters\":[\"node_id\",\"stiffness\",\"damping\",\"mass\"]},"
        "{\"name\":\"set_keyframe\",\"description\":\"Set an animatable float property keyframe at a specific frame\",\"category\":\"Animation\",\"parameters\":[\"node_id\",\"property\",\"frame\",\"value\",\"interpolation\"]},"
        "{\"name\":\"set_timeline_range\",\"description\":\"Set the Daz Studio play range and animation range\",\"category\":\"Animation\",\"parameters\":[\"start_frame\",\"end_frame\"]},"
        "{\"name\":\"seek_to_frame\",\"description\":\"Move the Daz Studio timeline cursor to a specific frame\",\"category\":\"Animation\",\"parameters\":[\"frame\"]},"
        "{\"name\":\"run_dforce_simulation\",\"description\":\"Run a dForce physics simulation via inline DazScript\",\"category\":\"Animation\",\"parameters\":[\"node_id\",\"start_frame\",\"end_frame\"]},"
        "{\"name\":\"set_camera_transform\",\"description\":\"Position and orient a camera\",\"category\":\"Camera\",\"parameters\":[\"camera_id\",\"position\",\"target\"]},"
        "{\"name\":\"set_focal_length\",\"description\":\"Set camera focal length in mm\",\"category\":\"Camera\",\"parameters\":[\"camera_id\",\"focal_length\"]},"
        "{\"name\":\"set_aperture\",\"description\":\"Set camera aperture and depth of field\",\"category\":\"Camera\",\"parameters\":[\"camera_id\",\"f_stop\",\"enable_dof\",\"focus_distance\"]},"
        "{\"name\":\"focus_camera\",\"description\":\"Point camera to look at a target node\",\"category\":\"Camera\",\"parameters\":[\"camera_id\",\"target\",\"offset\"]},"
        "{\"name\":\"selection_map_list\",\"description\":\"List all selection maps on a node\",\"category\":\"Selection\",\"parameters\":[\"node_id\"]},"
        "{\"name\":\"selection_map_get_pairs\",\"description\":\"Get face group/node pairs in a selection map\",\"category\":\"Selection\",\"parameters\":[\"node_id\",\"map_name\"]},"
        "{\"name\":\"selection_map_add_pair\",\"description\":\"Add a face group/node pair to a selection map\",\"category\":\"Selection\",\"parameters\":[\"node_id\",\"map_name\",\"face_group\",\"target_node\"]},"
        "{\"name\":\"selection_map_remove_pair\",\"description\":\"Remove a pair from a selection map by index\",\"category\":\"Selection\",\"parameters\":[\"node_id\",\"map_name\",\"pair_index\"]},"
        "{\"name\":\"selection_map_clear\",\"description\":\"Clear all pairs from a selection map\",\"category\":\"Selection\",\"parameters\":[\"node_id\",\"map_name\"]},"
        "{\"name\":\"set_node_selectable\",\"description\":\"Set whether a node is selectable in the viewport\",\"category\":\"Scene\",\"parameters\":[\"node_id\",\"selectable\"]},"
        "{\"name\":\"set_render_visible\",\"description\":\"Set whether a node is visible in renders\",\"category\":\"Scene\",\"parameters\":[\"node_id\",\"visible\"]},"
        "{\"name\":\"parent_node\",\"description\":\"Reparent a node under another node\",\"category\":\"Scene\",\"parameters\":[\"node_id\",\"parent_id\"]},"
        "{\"name\":\"unparent_node\",\"description\":\"Remove a node from its parent\",\"category\":\"Scene\",\"parameters\":[\"node_id\"]},"
        "{\"name\":\"mesh_get_vertex_count\",\"description\":\"Get the number of vertices in a node mesh\",\"category\":\"Mesh\",\"parameters\":[\"node_id\"]},"
        "{\"name\":\"mesh_get_face_count\",\"description\":\"Get the number of faces in a node mesh\",\"category\":\"Mesh\",\"parameters\":[\"node_id\"]},"
        "{\"name\":\"get_shape_materials\",\"description\":\"List material names on a node shape\",\"category\":\"Materials\",\"parameters\":[\"node_id\"]},"
        "{\"name\":\"lock_property\",\"description\":\"Lock a property on a node\",\"category\":\"Properties\",\"parameters\":[\"node_id\",\"property\",\"locked\"]}"
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

    // ─── Phase 3: Render & Export Commands ─────────────────────────────────────

    if (command == "render") {
        if (!dzApp) return ErrorResponse(id, "No app");
        DzRenderMgr* renderMgr = dzApp->getRenderMgr();
        if (!renderMgr) return ErrorResponse(id, "No render manager");
        QString mode = ExtractArgString(line, "mode").toLower();
        QString quality = ExtractArgString(line, "quality");
        QString wStr = ExtractArgString(line, "width");
        QString hStr = ExtractArgString(line, "height");
        DzRenderOptions* options = renderMgr->getRenderOptions();
        if (options) {
            if (!wStr.isEmpty() && !hStr.isEmpty())
                options->setImageSize(QSize(wStr.toInt(), hStr.toInt()));
            if (!quality.isEmpty()) {
                int q = quality.toInt();
                if (q > 0) { options->setPixelSamples(q, q); }
            }
        }
        renderMgr->doRender();
        return OkResponse(id, "{\"started\":true,\"render_id\":\"render_001\"}");
    }

    if (command == "cancel_render") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString script = "RenderMgr.cancelRender();\n";
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, "{\"cancelled\":true}");
    }

    if (command == "set_render_engine") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString engine = ExtractArgString(line, "engine");
        if (engine.isEmpty()) return ErrorResponse(id, "engine required");
        QString script = QString("RenderSettings.setRenderer('%1');\n").arg(engine);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, std::string("{\"engine_set\":true,\"engine\":\"") + JsonEscape(engine) + "\"}");
    }

    if (command == "set_render_output") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString path = ExtractArgString(line, "path");
        QString filename = ExtractArgString(line, "filename");
        QString script = QString(
            "RenderSettings.setOutputPath('%1');\n"
            "RenderSettings.setOutputFileName('%2');\n"
        ).arg(path, filename);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, std::string("{\"configured\":true,\"path\":\"") +
                          JsonEscape(path) + "\",\"filename\":\"" + JsonEscape(filename) + "\"}");
    }

    if (command == "set_resolution") {
        if (!dzApp) return ErrorResponse(id, "No app");
        DzRenderMgr* renderMgr = dzApp->getRenderMgr();
        if (!renderMgr) return ErrorResponse(id, "No render manager");
        DzRenderOptions* options = renderMgr->getRenderOptions();
        if (!options) return ErrorResponse(id, "No render options");
        int width = ExtractArgString(line, "width").toInt();
        int height = ExtractArgString(line, "height").toInt();
        if (width > 0 && height > 0) {
            options->setImageSize(QSize(width, height));
        }
        return OkResponse(id, std::string("{\"resolution_set\":true,\"width\":") +
                          std::to_string(width) + ",\"height\":" + std::to_string(height) + "}");
    }

    if (command == "set_denoising") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString enabledStr = ExtractArgString(line, "enabled");
        bool enabled = (enabledStr.toLower() == "true");
        QString script = QString("RenderSettings.setDenoisingEnabled(%1);\n").arg(enabled ? "true" : "false");
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, "{\"set\":true}");
    }

    if (command == "render_region") {
        // DAZ C++ SDK has no direct region render API — use DzScript
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString xStr = ExtractArgString(line, "x");
        QString yStr = ExtractArgString(line, "y");
        QString wStr = ExtractArgString(line, "width");
        QString hStr = ExtractArgString(line, "height");
        QString script = QString(
            "var vp = App.getActiveViewport();\n"
            "if (vp) {\n"
            "  App.renderRegion(vp, %1, %2, %3, %4);\n"
            "}\n"
        ).arg(xStr, yStr, wStr, hStr);
        QString result;
        std::mutex mtx;
        std::condition_variable cv;
        bool done = false;
        RunScriptEvent* event = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, event);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, std::string("{\"rendered\":true,\"region\":[") +
                          xStr.toStdString() + "," + yStr.toStdString() + "," + wStr.toStdString() + "," + hStr.toStdString() + "]}");
    }

    if (command == "queue_render") {
        // No direct C++ SDK queue API — use DzScript
        QString passName = ExtractArgString(line, "pass_name");
        DzScript dzScript;
        dzScript.addLine(QString("App.queueRender('%1');\n").arg(passName));
        if (dzScript.execute()) {
            return OkResponse(id, std::string("{\"queued\":true,\"pass_name\":\"") +
                              JsonEscape(passName) + "\",\"position\":1}");
        }
        return ErrorResponse(id, "Queue render failed");
    }

    if (command == "export_fbx" || command == "export_obj" ||
        command == "export_gltf" || command == "export_collada" ||
        command == "export_usd") {
        // Format-specific exports use DzScript (same pattern as export_scene)
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString filepath = ExtractArgString(line, "filepath");
        if (filepath.isEmpty()) return ErrorResponse(id, "filepath is required");
        QString fmt;
        if (command == "export_fbx") fmt = "FBX";
        else if (command == "export_obj") fmt = "OBJ";
        else if (command == "export_gltf") fmt = "glTF";
        else if (command == "export_collada") fmt = "Collada";
        else if (command == "export_usd") fmt = "USD";
        QString binaryStr = ExtractArgString(line, "binary");
        QString settings = QString("{\"format\":\"%1\",\"binary\":%2}")
            .arg(fmt, (binaryStr.toLower() == "true" ? "true" : "false"));
        QString result;
        std::mutex mtx;
        std::condition_variable cv;
        bool done = false;
        ExportSceneEvent* event = new ExportSceneEvent(filepath, settings, &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, event);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        if (result.contains("\"success\":true")) {
            return OkResponse(id, std::string("{\"exported\":true,\"filepath\":\"") + JsonEscape(filepath) + "\"}");
        }
        return ErrorResponse(id, QString("%1 export failed").arg(fmt));
    }

    if (command == "export_selected") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString filepath = ExtractArgString(line, "filepath");
        QString format = ExtractArgString(line, "format");
        if (filepath.isEmpty()) return ErrorResponse(id, "filepath is required");
        QString settings = QString("{\"format\":\"%1\",\"selected_only\":true}").arg(format.isEmpty() ? "FBX" : format);
        QString result;
        std::mutex mtx;
        std::condition_variable cv;
        bool done = false;
        ExportSceneEvent* event = new ExportSceneEvent(filepath, settings, &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, event);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        if (result.contains("\"success\":true")) {
            return OkResponse(id, std::string("{\"exported\":true,\"filepath\":\"") + JsonEscape(filepath) + "\"}");
        }
        return ErrorResponse(id, "Selected export failed");
    }

    if (command == "batch_export") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString dir = ExtractArgString(line, "dir");
        QString format = ExtractArgString(line, "format");
        if (dir.isEmpty()) return ErrorResponse(id, "dir is required");
        QString settings = QString("{\"format\":\"%1\",\"batch_dir\":\"%2\"}")
            .arg(format.isEmpty() ? "FBX" : format, dir);
        QString result;
        std::mutex mtx;
        std::condition_variable cv;
        bool done = false;
        ExportSceneEvent* event = new ExportSceneEvent(dir, settings, &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, event);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        if (result.contains("\"success\":true")) {
            return OkResponse(id, std::string("{\"exported\":true,\"dir\":\"") + JsonEscape(dir) + "\"}");
        }
        return ErrorResponse(id, "Batch export failed");
    }

    if (command == "export_animation") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString filepath = ExtractArgString(line, "filepath");
        QString figureId = ExtractArgString(line, "figure_id");
        if (filepath.isEmpty()) return ErrorResponse(id, "filepath is required");
        QString settings = QString("{\"format\":\"FBX\",\"selected_only\":false,\"animation_only\":true}");
        if (!figureId.isEmpty()) {
            settings = QString("{\"format\":\"FBX\",\"figure_id\":\"%1\",\"animation_only\":true}").arg(figureId);
        }
        QString result;
        std::mutex mtx;
        std::condition_variable cv;
        bool done = false;
        ExportSceneEvent* event = new ExportSceneEvent(filepath, settings, &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, event);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        if (result.contains("\"success\":true")) {
            return OkResponse(id, std::string("{\"exported\":true,\"filepath\":\"") + JsonEscape(filepath) + "\"}");
        }
        return ErrorResponse(id, "Animation export failed");
    }

    // ─── Phase 7: Physics, Rigging, Transform & Selection Remaining ───────────

    if (command == "simulate_physics") {
        // Redirect to run_dforce_simulation — same functionality
        QString nodeIdsStr = ExtractArgString(line, "node_ids");
        QString framesStr = ExtractArgString(line, "frames");
        QString startFrameStr = ExtractArgString(line, "start_frame");
        int frames = framesStr.toInt();
        if (frames <= 0) frames = 30;
        int startFrame = startFrameStr.toInt();
        // Use first node_id if provided
        QString nodeId;
        if (!nodeIdsStr.isEmpty()) {
            QString trimmed = nodeIdsStr.trimmed();
            if (trimmed.startsWith('[')) trimmed = trimmed.mid(1);
            if (trimmed.endsWith(']')) trimmed = trimmed.left(trimmed.length() - 1);
            QStringList ids = trimmed.split(',', QString::SkipEmptyParts);
            if (!ids.isEmpty()) nodeId = ids[0].trimmed().remove('"').remove('\'');
        }
        if (!dzApp) return ErrorResponse(id, "No app");
        QString script;
        if (!nodeId.isEmpty()) {
            script = QString(
                "var node = Scene.findNode('%1');\n"
                "if (node) App.getSimulator().simulate(node, %2, %3);\n"
            ).arg(nodeId).arg(startFrame).arg(startFrame + frames);
        } else {
            script = QString(
                "App.getSimulator().simulate(null, %1, %2);\n"
            ).arg(startFrame).arg(startFrame + frames);
        }
        DzScript dzScript;
        dzScript.addLine(script);
        if (dzScript.execute()) {
            return OkResponse(id, std::string("{\"simulated\":true,\"frames\":" + std::to_string(frames) + "}"));
        }
        return ErrorResponse(id, "Physics simulation failed");
    }

    if (command == "set_wind") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString direction = ExtractArgString(line, "direction");
        QString speed = ExtractArgString(line, "speed");
        QString turbulence = ExtractArgString(line, "turbulence");
        QString script = QString(
            "var wind = Scene.getWind();\n"
            "if (wind) {\n"
            "  if ('%1'.length > 0) wind.setDirection(new DzVec3(%1));\n"
            "  if ('%2'.length > 0) wind.setSpeed(%2);\n"
            "  if ('%3'.length > 0) wind.setTurbulence(%3);\n"
            "}\n"
        ).arg(direction, speed, turbulence);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, "{\"set\":true}");
    }

    if (command == "set_gravity") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString strength = ExtractArgString(line, "strength");
        QString direction = ExtractArgString(line, "direction");
        QString nodeId = ExtractArgString(line, "node_id");
        QString script = QString(
            "var grav = Scene.getGravity();\n"
            "if (grav) {\n"
            "  if ('%1'.length > 0) grav.setStrength(%1);\n"
            "  if ('%2'.length > 0) grav.setDirection(new DzVec3(%2));\n"
            "}\n"
        ).arg(strength, direction);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, "{\"set\":true}");
    }

    if (command == "add_collision") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString nodeId = ExtractArgString(line, "node_id");
        QString shape = ExtractArgString(line, "shape");
        QString friction = ExtractArgString(line, "friction");
        QString script = QString(
            "var node = Scene.findNode('%1');\n"
            "if (node) {\n"
            "  node.addCollisionShape('%2');\n"
            "  if ('%3'.length > 0) node.setFriction(%3);\n"
            "}\n"
        ).arg(nodeId, shape, friction);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, std::string("{\"added\":true,\"node_id\":\"") + JsonEscape(nodeId) + "\"}");
    }

    if (command == "bake_physics") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString nodeId = ExtractArgString(line, "node_id");
        QString sampleRate = ExtractArgString(line, "sample_rate");
        QString script = QString(
            "var node = Scene.findNode('%1');\n"
            "if (node) {\n"
            "  App.getSimulator().bakeSimulation(node, %2);\n"
            "}\n"
        ).arg(nodeId, sampleRate.isEmpty() ? "1" : sampleRate);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, std::string("{\"baked\":true,\"node_id\":\"") + JsonEscape(nodeId) + "\"}");
    }

    if (command == "set_physics_props") {
        // Reuse the existing phy modifier infrastructure
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        QString mass = ExtractArgString(line, "mass");
        QString stiffness = ExtractArgString(line, "stiffness");
        QString damping = ExtractArgString(line, "damping");
        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
        DzObject* obj = node->getObject();
        if (!obj) return ErrorResponse(id, "Node has no geometry object");
        DzModifier* baseMod = obj->findModifier("DazPilotPhy");
        DazPilotPhyModifier* mod = qobject_cast<DazPilotPhyModifier*>(baseMod);
        if (!mod) {
            // Create one
            mod = new DazPilotPhyModifier();
            obj->addModifier(mod);
        }
        if (!stiffness.isEmpty()) mod->setStiffness(stiffness.toFloat());
        if (!damping.isEmpty()) mod->setDamping(damping.toFloat());
        if (!mass.isEmpty()) mod->setMass(mass.toFloat());
        mod->resetSimulation();
        return OkResponse(id, std::string("{\"set\":true,\"node\":\"") + JsonEscape(node->getName()) + "\"}");
    }

    if (command == "remove_physics") {
        QString nodeId = ExtractArgString(line, "node_id");
        QString removeModifiers = ExtractArgString(line, "remove_modifiers");
        if (!dzScene) return ErrorResponse(id, "No scene");
        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
        DzObject* obj = node->getObject();
        if (!obj) return ErrorResponse(id, "Node has no geometry object");
        if (removeModifiers.toLower() != "false") {
            // Remove all physics modifiers
            DzModifier* mod = obj->findModifier("DazPilotPhy");
            if (mod) obj->removeModifier(mod);
            mod = obj->findModifier("dForce");
            if (mod) obj->removeModifier(mod);
        }
        return OkResponse(id, std::string("{\"removed\":true,\"node\":\"") + JsonEscape(node->getName()) + "\"}");
    }

    // ─── Rigging Commands ───────────────────────────────────────────────────

    if (command == "get_joint_list") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString figureId = ExtractArgString(line, "figure_id");
        DzNode* node = figureId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(figureId);
        if (!node) return ErrorResponse(id, QString("Figure not found: %1").arg(figureId));
        DzFigure* figure = qobject_cast<DzFigure*>(node);
        if (!figure) return ErrorResponse(id, "Node is not a figure");
        QObjectList bones = figure->getAllBones();
        std::ostringstream oss;
        oss << "{\"joints\":[";
        for (int i = 0; i < bones.size(); ++i) {
            DzBone* bone = qobject_cast<DzBone*>(bones[i]);
            if (!bone) continue;
            if (i > 0) oss << ",";
            oss << "{\"name\":\"" << JsonEscape(bone->getName()) << "\"";
            DzNode* parent = bone->getNodeParent();
            oss << ",\"parent\":\"" << (parent ? JsonEscape(parent->getName()) : "") << "\"";
            oss << ",\"type\":\"rotation\"}";
        }
        oss << "],\"total\":" << bones.size() << "}";
        return OkResponse(id, oss.str());
    }

    if (command == "set_joint_rotation") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString figureId = ExtractArgString(line, "figure_id");
        QString jointName = ExtractArgString(line, "joint");
        QString rotationStr = ExtractArgString(line, "rotation");
        DzNode* node = figureId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(figureId);
        if (!node) return ErrorResponse(id, QString("Figure not found: %1").arg(figureId));
        DzFigure* figure = qobject_cast<DzFigure*>(node);
        if (!figure) return ErrorResponse(id, "Node is not a figure");
        DzBone* bone = figure->findBone(jointName);
        if (!bone) return ErrorResponse(id, QString("Joint not found: %1").arg(jointName));
        if (!rotationStr.isEmpty()) {
            rotationStr.remove('[').remove(']');
            QStringList parts = rotationStr.split(',');
            if (parts.size() == 4) {
                bone->setWSRot(DzQuat(
                    parts[0].trimmed().toFloat(),
                    parts[1].trimmed().toFloat(),
                    parts[2].trimmed().toFloat(),
                    parts[3].trimmed().toFloat()
                ));
            } else if (parts.size() == 3) {
                // Euler angles — convert to quaternion via DzScript
                if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
                QString script = QString(
                    "var fig = Scene.findNode('%1');\n"
                    "var bone = fig.findBone('%2');\n"
                    "if (bone) bone.setRotation(%3, %4, %5);\n"
                ).arg(figureId, jointName, parts[0].trimmed(), parts[1].trimmed(), parts[2].trimmed());
                QString result;
                std::mutex mtx; std::condition_variable cv; bool done = false;
                RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
                QCoreApplication::postEvent(g_scriptExecutor, evt);
                std::unique_lock<std::mutex> lock(mtx);
                cv.wait(lock, [&done]{ return done; });
            }
        }
        return OkResponse(id, std::string("{\"set\":true,\"joint\":\"") + JsonEscape(jointName) + "\"}");
    }

    if (command == "set_ik_fk_blend") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString figureId = ExtractArgString(line, "figure_id");
        QString limb = ExtractArgString(line, "limb");
        double blend = ExtractArgString(line, "blend").toDouble();
        QString script = QString(
            "var fig = Scene.findNode('%1');\n"
            "if (fig) {\n"
            "  fig.setIKFKBlend('%2', %3);\n"
            "}\n"
        ).arg(figureId, limb, QString::number(blend));
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, std::string("{\"set\":true,\"limb\":\"") + JsonEscape(limb) + "\",\"blend\":" + std::to_string(blend) + "}");
    }

    if (command == "add_joint") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString figureId = ExtractArgString(line, "figure_id");
        QString jointName = ExtractArgString(line, "joint_name");
        QString parentJoint = ExtractArgString(line, "parent_joint");
        if (jointName.isEmpty()) return ErrorResponse(id, "joint_name is required");
        QString script = QString(
            "var fig = Scene.findNode('%1');\n"
            "if (fig) {\n"
            "  var parent = fig.findBone('%2');\n"
            "  var newJoint = fig.addBone('%3', parent);\n"
            "}\n"
        ).arg(figureId, parentJoint, jointName);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, std::string("{\"added\":true,\"joint\":\"") + JsonEscape(jointName) + "\"}");
    }

    // ─── Transform Commands ─────────────────────────────────────────────────

    if (command == "set_transform") {
        // Redirect to set_node_transform
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
        QString posStr = ExtractArgString(line, "position");
        if (!posStr.isEmpty()) {
            posStr.remove('[').remove(']');
            QStringList parts = posStr.split(',');
            if (parts.size() == 3) {
                node->setWSPos(DzVec3(parts[0].trimmed().toFloat(), parts[1].trimmed().toFloat(), parts[2].trimmed().toFloat()));
            }
        }
        QString rotStr = ExtractArgString(line, "rotation");
        if (!rotStr.isEmpty()) {
            rotStr.remove('[').remove(']');
            QStringList parts = rotStr.split(',');
            if (parts.size() == 4) {
                node->setWSRot(DzQuat(parts[0].trimmed().toFloat(), parts[1].trimmed().toFloat(), parts[2].trimmed().toFloat(), parts[3].trimmed().toFloat()));
            }
        }
        QString scaleStr = ExtractArgString(line, "scale");
        if (!scaleStr.isEmpty()) {
            scaleStr.remove('[').remove(']');
            QStringList parts = scaleStr.split(',');
            if (parts.size() == 1) {
                float s = parts[0].trimmed().toFloat();
                float vals[12] = {s, 0, 0, 0, s, 0, 0, 0, s, 0, 0, 0};
                node->setWSScale(DzMatrix3(vals));
            } else if (parts.size() == 3) {
                float vals[12] = {parts[0].trimmed().toFloat(),0,0,0,0,parts[1].trimmed().toFloat(),0,0,0,0,parts[2].trimmed().toFloat(),0};
                node->setWSScale(DzMatrix3(vals));
            }
        }
        return OkResponse(id, std::string("{\"transformed\":true,\"node\":\"") + JsonEscape(node->getName()) + "\"}");
    }

    if (command == "reset_transform") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
        node->setWSPos(DzVec3(0, 0, 0));
        node->setWSRot(DzQuat(0, 0, 0, 1));
        float identityVals[12] = {1,0,0,0,0,1,0,0,0,0,1,0};
        node->setWSScale(DzMatrix3(identityVals));
        return OkResponse(id, std::string("{\"reset\":true,\"node\":\"") + JsonEscape(node->getName()) + "\"}");
    }

    if (command == "align_nodes") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString targetNodeStr = ExtractArgString(line, "target_node");
        QString axesStr = ExtractArgString(line, "axes");
        QString nodesStr = ExtractArgString(line, "nodes");
        if (targetNodeStr.isEmpty()) return ErrorResponse(id, "target_node is required");
        DzNode* target = dzScene->findNode(targetNodeStr);
        if (!target) return ErrorResponse(id, QString("Target node not found: %1").arg(targetNodeStr));
        // Parse which axes to align
        bool alignX = axesStr.contains('x', Qt::CaseInsensitive) || axesStr.isEmpty();
        bool alignY = axesStr.contains('y', Qt::CaseInsensitive) || axesStr.isEmpty();
        bool alignZ = axesStr.contains('z', Qt::CaseInsensitive) || axesStr.isEmpty();
        // Parse node list
        QStringList nodeIds;
        if (!nodesStr.isEmpty()) {
            QString trimmed = nodesStr.trimmed();
            if (trimmed.startsWith('[')) trimmed = trimmed.mid(1);
            if (trimmed.endsWith(']')) trimmed = trimmed.left(trimmed.length() - 1);
            nodeIds = trimmed.split(',', QString::SkipEmptyParts);
        } else {
            // Use selection
            for (int i = 0; i < dzScene->getNumSelectedNodes(); ++i) {
                DzNode* sel = dzScene->getSelectedNode(i);
                if (sel && sel != target) nodeIds << sel->getName();
            }
        }
        DzVec3 targetPos = target->getWSPos();
        int aligned = 0;
        for (const QString& nid : nodeIds) {
            QString tid = nid.trimmed().remove('"').remove('\'');
            if (tid.isEmpty()) continue;
            DzNode* n = dzScene->findNode(tid);
            if (!n || n == target) continue;
            DzVec3 pos = n->getWSPos();
            if (alignX) pos.m_x = targetPos.m_x;
            if (alignY) pos.m_y = targetPos.m_y;
            if (alignZ) pos.m_z = targetPos.m_z;
            n->setWSPos(pos);
            aligned++;
        }
        return OkResponse(id, std::string("{\"aligned\":true,\"count\":") + std::to_string(aligned) + "}");
    }

    if (command == "distribute_nodes") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString axis = ExtractArgString(line, "axis").toLower();
        double spacing = ExtractArgString(line, "spacing").toDouble();
        if (spacing <= 0) spacing = 100.0;
        QString nodesStr = ExtractArgString(line, "nodes");
        QStringList nodeIds;
        if (!nodesStr.isEmpty()) {
            QString trimmed = nodesStr.trimmed();
            if (trimmed.startsWith('[')) trimmed = trimmed.mid(1);
            if (trimmed.endsWith(']')) trimmed = trimmed.left(trimmed.length() - 1);
            nodeIds = trimmed.split(',', QString::SkipEmptyParts);
        } else {
            for (int i = 0; i < dzScene->getNumSelectedNodes(); ++i) {
                nodeIds << dzScene->getSelectedNode(i)->getName();
            }
        }
        // Collect nodes
        QList<DzNode*> nodes;
        for (const QString& nid : nodeIds) {
            QString tid = nid.trimmed().remove('"').remove('\'');
            if (tid.isEmpty()) continue;
            DzNode* n = dzScene->findNode(tid);
            if (n) nodes.append(n);
        }
        if (nodes.size() < 2) return ErrorResponse(id, "Need at least 2 nodes to distribute");
        double startPos = 0.0;
        int axisIdx = (axis == "x") ? 0 : (axis == "y" ? 1 : 2);
        for (int i = 0; i < nodes.size(); ++i) {
            DzVec3 pos = nodes[i]->getWSPos();
            if (axisIdx == 0) pos.m_x = startPos + i * spacing;
            else if (axisIdx == 1) pos.m_y = startPos + i * spacing;
            else pos.m_z = startPos + i * spacing;
            nodes[i]->setWSPos(pos);
        }
        return OkResponse(id, std::string("{\"distributed\":true,\"count\":") + std::to_string(nodes.size()) + "}");
    }

    if (command == "snap_to_ground") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
        DzVec3 pos = node->getWSPos();
        pos.m_y = 0.0; // Snap to ground (Y=0)
        node->setWSPos(pos);
        return OkResponse(id, std::string("{\"snapped\":true,\"node\":\"") + JsonEscape(node->getName()) + "\"}");
    }

    if (command == "set_pivot") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString nodeId = ExtractArgString(line, "node_id");
        QString pivotStr = ExtractArgString(line, "pivot");
        if (pivotStr.isEmpty()) return ErrorResponse(id, "pivot is required");
        QString script = QString(
            "var node = Scene.findNode('%1');\n"
            "if (node) {\n"
            "  node.setPivot(new DzVec3(%2));\n"
            "}\n"
        ).arg(nodeId, pivotStr);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, std::string("{\"set\":true,\"node\":\"") + JsonEscape(nodeId) + "\"}");
    }

    // ─── Additional Selection Commands ──────────────────────────────────────

    if (command == "select_by_type") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString type = ExtractArgString(line, "type").toLower();
        dzScene->selectAllNodes(false);
        int matched = 0;
        for (int i = 0; i < dzScene->getNumNodes(); ++i) {
            DzNode* node = dzScene->getNode(i);
            if (!node) continue;
            bool match = false;
            if (type == "figure" && qobject_cast<DzFigure*>(node)) match = true;
            else if (type == "camera" && qobject_cast<DzCamera*>(node)) match = true;
            else if (type == "light" && qobject_cast<DzLight*>(node)) match = true;
            else if (type == "bone" && qobject_cast<DzBone*>(node)) match = true;
            else if (type == "prop" && !qobject_cast<DzFigure*>(node) && !qobject_cast<DzCamera*>(node) && !qobject_cast<DzLight*>(node) && !qobject_cast<DzBone*>(node)) match = true;
            if (match) {
                dzScene->setPrimarySelection(node);
                matched++;
            }
        }
        return OkResponse(id, std::string("{\"selected\":true,\"type\":\"") + JsonEscape(type) + "\",\"matched\":" + std::to_string(matched) + "}");
    }

    if (command == "select_hierarchy") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
        dzScene->selectAllNodes(false);
        // Recursively select node and all children
        std::function<void(DzNode*)> selectRecursive = [&](DzNode* n) {
            if (!n) return;
            dzScene->setPrimarySelection(n);
            for (int i = 0; i < n->getNumNodeChildren(); ++i) {
                selectRecursive(n->getNodeChild(i));
            }
        };
        selectRecursive(node);
        return OkResponse(id, std::string("{\"selected\":true,\"node\":\"") + JsonEscape(node->getName()) + "\",\"nodes_selected\":" + std::to_string(dzScene->getNumSelectedNodes()) + "}");
    }

    if (command == "save_selection") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString name = ExtractArgString(line, "name");
        if (name.isEmpty()) return ErrorResponse(id, "name is required");
        QString script = QString(
            "Scene.saveSelection('%1');\n"
        ).arg(name);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, std::string("{\"saved\":true,\"name\":\"") + JsonEscape(name) + "\"}");
    }

    if (command == "load_selection") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString name = ExtractArgString(line, "name");
        if (name.isEmpty()) return ErrorResponse(id, "name is required");
        QString script = QString(
            "Scene.loadSelection('%1');\n"
        ).arg(name);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, std::string("{\"loaded\":true,\"name\":\"") + JsonEscape(name) + "\",\"nodes_restored\":" + std::to_string(dzScene->getNumSelectedNodes()) + "}");
    }

    // ─── Phase 6: Hair & Clothing Commands ────────────────────────────────────

    if (command == "load_hair") {
        if (!dzApp) return ErrorResponse(id, "No app");
        QString name = ExtractArgString(line, "name");
        QString figureId = ExtractArgString(line, "figure_id");
        if (name.isEmpty()) return ErrorResponse(id, "name is required");
        DzContentMgr* contentMgr = dzApp->getContentMgr();
        if (!contentMgr) return ErrorResponse(id, "No content manager");
        // Search for hair content
        QStringList filters; filters << "*.duf";
        QString foundPath;
        for (int i = 0; i < contentMgr->getNumContentDirectories(); ++i) {
            QString dir = contentMgr->getContentDirectoryPath(i);
            QDirIterator it(dir, filters, QDir::Files, QDirIterator::Subdirectories);
            while (it.hasNext()) {
                QString fp = it.next();
                if (it.fileName().contains(name, Qt::CaseInsensitive) &&
                    fp.contains("Hair", Qt::CaseInsensitive)) {
                    foundPath = fp; break;
                }
            }
            if (!foundPath.isEmpty()) break;
        }
        if (foundPath.isEmpty()) return ErrorResponse(id, QString("Hair not found: %1").arg(name));
        if (!figureId.isEmpty() && dzScene) {
            DzNode* fig = dzScene->findNode(figureId);
            if (fig) dzScene->setPrimarySelection(fig);
        }
        if (OpenContentFile(foundPath, true)) {
            return OkResponse(id, std::string("{\"loaded\":true,\"name\":\"") + JsonEscape(name) +
                              "\",\"path\":\"" + JsonEscape(foundPath) + "\"}");
        }
        return ErrorResponse(id, QString("Failed to load hair: %1").arg(foundPath));
    }

    if (command == "style_hair") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString hairId = ExtractArgString(line, "hair_id");
        QString preset = ExtractArgString(line, "preset");
        if (preset.isEmpty()) return ErrorResponse(id, "preset is required");
        QString script = QString(
            "var hair = Scene.findNode('%1');\n"
            "if (hair) {\n"
            "  App.applyHairStyle(hair, '%2');\n"
            "}\n"
        ).arg(hairId, preset);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, std::string("{\"styled\":true,\"hair_id\":\"") + JsonEscape(hairId) + "\"}");
    }

    if (command == "set_hair_color") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString hairId = ExtractArgString(line, "hair_id");
        QString color = ExtractArgString(line, "color");
        QString highlights = ExtractArgString(line, "highlights");
        QString script = QString(
            "var hair = Scene.findNode('%1');\n"
            "if (hair) {\n"
            "  hair.setHairColor(new DzColor(%2));\n"
            "  if ('%3'.length > 0) hair.setHighlightColor(new DzColor(%3));\n"
            "}\n"
        ).arg(hairId, color, highlights);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, std::string("{\"set\":true,\"hair_id\":\"") + JsonEscape(hairId) + "\"}");
    }

    if (command == "apply_hair_physics") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString hairId = ExtractArgString(line, "hair_id");
        QString enable = ExtractArgString(line, "enable");
        QString stiffness = ExtractArgString(line, "stiffness");
        QString script = QString(
            "var hair = Scene.findNode('%1');\n"
            "if (hair) {\n"
            "  var phy = hair.getHairPhysics();\n"
            "  if (phy) {\n"
            "    phy.setEnabled(%2);\n"
            "    if ('%3'.length > 0) phy.setStiffness(%3);\n"
            "  }\n"
            "}\n"
        ).arg(hairId, enable.toLower() == "true" ? "true" : "false", stiffness);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, "{\"applied\":true}");
    }

    if (command == "set_hair_length") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString hairId = ExtractArgString(line, "hair_id");
        QString length = ExtractArgString(line, "length");
        QString scaleFactor = ExtractArgString(line, "scale_factor");
        QString script = QString(
            "var hair = Scene.findNode('%1');\n"
            "if (hair) {\n"
            "  if ('%2'.length > 0) hair.setHairLength(%2);\n"
            "  if ('%3'.length > 0) hair.setScale(%3, %3, %3);\n"
            "}\n"
        ).arg(hairId, length, scaleFactor.isEmpty() ? length : scaleFactor);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, std::string("{\"set\":true,\"hair_id\":\"") + JsonEscape(hairId) + "\"}");
    }

    if (command == "set_hair_volume") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString hairId = ExtractArgString(line, "hair_id");
        QString volume = ExtractArgString(line, "volume");
        QString script = QString(
            "var hair = Scene.findNode('%1');\n"
            "if (hair) {\n"
            "  hair.setHairVolume(%2);\n"
            "}\n"
        ).arg(hairId, volume);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, std::string("{\"set\":true,\"hair_id\":\"") + JsonEscape(hairId) + "\"}");
    }

    if (command == "list_hair_presets") {
        if (!dzApp) return ErrorResponse(id, "No app");
        DzContentMgr* contentMgr = dzApp->getContentMgr();
        if (!contentMgr) return ErrorResponse(id, "No content manager");
        QStringList filters; filters << "*.duf";
        std::ostringstream oss;
        oss << "{\"presets\":[";
        bool first = true;
        for (int i = 0; i < contentMgr->getNumContentDirectories(); ++i) {
            QString dir = contentMgr->getContentDirectoryPath(i);
            QDirIterator it(dir, filters, QDir::Files, QDirIterator::Subdirectories);
            while (it.hasNext()) {
                QString fp = it.next();
                if (!fp.contains("Hair", Qt::CaseInsensitive)) continue;
                if (!first) oss << ",";
                first = false;
                oss << "{\"name\":\"" << JsonEscape(it.fileName()) << "\",\"path\":\"" << JsonEscape(fp) << "\"}";
            }
        }
        oss << "],\"total\":" << 0 << "}";
        std::string result = oss.str();
        // Fix total
        int total = 0;
        for (int i = 0; i < contentMgr->getNumContentDirectories(); ++i) {
            QString dir = contentMgr->getContentDirectoryPath(i);
            QDirIterator it(dir, filters, QDir::Files, QDirIterator::Subdirectories);
            while (it.hasNext()) { QString fp = it.next(); if (fp.contains("Hair", Qt::CaseInsensitive)) total++; }
        }
        std::string totalStr = "\"total\":" + std::to_string(total) + "}";
        size_t pos = result.rfind("\"total\":");
        if (pos != std::string::npos) result.replace(pos, result.length() - pos - 1, totalStr);
        return OkResponse(id, result);
    }

    if (command == "remove_hair") {
        QString hairId = ExtractArgString(line, "hair_id");
        if (DeleteNode(hairId)) {
            return OkResponse(id, std::string("{\"removed\":true,\"hair_id\":\"") + JsonEscape(hairId) + "\"}");
        }
        return ErrorResponse(id, QString("Hair not found: %1").arg(hairId));
    }

    if (command == "set_hair_shader") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString hairId = ExtractArgString(line, "hair_id");
        QString shaderType = ExtractArgString(line, "shader_type");
        QString gloss = ExtractArgString(line, "gloss");
        QString script = QString(
            "var hair = Scene.findNode('%1');\n"
            "if (hair) {\n"
            "  hair.setHairShader('%2');\n"
            "  if ('%3'.length > 0) hair.setGloss(%3);\n"
            "}\n"
        ).arg(hairId, shaderType, gloss);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, std::string("{\"set\":true,\"hair_id\":\"") + JsonEscape(hairId) + "\"}");
    }

    if (command == "apply_hair_preset") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString hairId = ExtractArgString(line, "hair_id");
        QString preset = ExtractArgString(line, "preset");
        QString script = QString(
            "var hair = Scene.findNode('%1');\n"
            "if (hair) {\n"
            "  App.applyHairPreset(hair, '%2');\n"
            "}\n"
        ).arg(hairId, preset);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, std::string("{\"applied\":true,\"hair_id\":\"") + JsonEscape(hairId) + "\"}");
    }

    // ─── Clothing Commands ────────────────────────────────────────────────────

    if (command == "load_clothing") {
        if (!dzApp) return ErrorResponse(id, "No app");
        QString name = ExtractArgString(line, "name");
        QString figureId = ExtractArgString(line, "figure_id");
        if (name.isEmpty()) return ErrorResponse(id, "name is required");
        DzContentMgr* contentMgr = dzApp->getContentMgr();
        if (!contentMgr) return ErrorResponse(id, "No content manager");
        QStringList filters; filters << "*.duf";
        QString foundPath;
        for (int i = 0; i < contentMgr->getNumContentDirectories(); ++i) {
            QString dir = contentMgr->getContentDirectoryPath(i);
            QDirIterator it(dir, filters, QDir::Files, QDirIterator::Subdirectories);
            while (it.hasNext()) {
                QString fp = it.next();
                if (it.fileName().contains(name, Qt::CaseInsensitive)) {
                    foundPath = fp; break;
                }
            }
            if (!foundPath.isEmpty()) break;
        }
        if (foundPath.isEmpty()) return ErrorResponse(id, QString("Clothing not found: %1").arg(name));
        if (!figureId.isEmpty() && dzScene) {
            DzNode* fig = dzScene->findNode(figureId);
            if (fig) dzScene->setPrimarySelection(fig);
        }
        if (OpenContentFile(foundPath, true)) {
            return OkResponse(id, std::string("{\"loaded\":true,\"name\":\"") + JsonEscape(name) +
                              "\",\"path\":\"" + JsonEscape(foundPath) + "\"}");
        }
        return ErrorResponse(id, QString("Failed to load clothing: %1").arg(foundPath));
    }

    if (command == "fit_clothing") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString clothingId = ExtractArgString(line, "clothing_id");
        QString figureId = ExtractArgString(line, "figure_id");
        QString fitType = ExtractArgString(line, "fit_type");
        if (fitType.isEmpty()) fitType = "auto";
        // DzFigure has no fitItemTo() in C++ SDK — use DzScript
        QString script = QString(
            "var clothing = Scene.findNode('%1');\n"
            "var figure = Scene.findNode('%2');\n"
            "if (clothing && figure) {\n"
            "  figure.fitItem(clothing, '%3');\n"
            "}\n"
        ).arg(clothingId, figureId, fitType);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, "{\"fitted\":true}");
    }

    if (command == "remove_clothing") {
        QString clothingId = ExtractArgString(line, "clothing_id");
        if (DeleteNode(clothingId)) {
            return OkResponse(id, std::string("{\"removed\":true,\"clothing_id\":\"") + JsonEscape(clothingId) + "\"}");
        }
        return ErrorResponse(id, QString("Clothing not found: %1").arg(clothingId));
    }

    if (command == "list_worn_items") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString figureId = ExtractArgString(line, "figure_id");
        DzNode* fig = figureId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(figureId);
        if (!fig) return ErrorResponse(id, QString("Figure not found: %1").arg(figureId));
        DzFigure* figure = qobject_cast<DzFigure*>(fig);
        if (!figure) return ErrorResponse(id, "Node is not a figure");
        // List child nodes that are likely clothing (not bones)
        std::ostringstream oss;
        oss << "{\"items\":[";
        bool firstItem = true;
        for (int i = 0; i < figure->getNumNodeChildren(); ++i) {
            DzNode* child = figure->getNodeChild(i);
            if (!child) continue;
            if (qobject_cast<DzBone*>(child)) continue;
            if (!firstItem) oss << ",";
            firstItem = false;
            oss << "{\"node_id\":\"" << JsonEscape(child->getName()) << "\"";
            oss << ",\"name\":\"" << JsonEscape(child->getLabel()) << "\"";
            oss << ",\"type\":\"clothing\"";
            oss << ",\"fit\":\"fitted\"}";
        }
        oss << "],\"total\":" << (firstItem ? 0 : 0) << "}";
        std::string result = oss.str();
        int totalItems = 0;
        for (int i = 0; i < figure->getNumNodeChildren(); ++i) {
            DzNode* child = figure->getNodeChild(i);
            if (child && !qobject_cast<DzBone*>(child)) totalItems++;
        }
        std::string totalStr = "\"total\":" + std::to_string(totalItems) + "}";
        size_t pos2 = result.rfind("\"total\":");
        if (pos2 != std::string::npos) result.replace(pos2, result.length() - pos2 - 1, totalStr);
        return OkResponse(id, result);
    }

    if (command == "set_clothing_params") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString clothingId = ExtractArgString(line, "clothing_id");
        QString param = ExtractArgString(line, "parameter");
        QString value = ExtractArgString(line, "value");
        QString script = QString(
            "var clothing = Scene.findNode('%1');\n"
            "if (clothing) {\n"
            "  clothing.setParameter('%2', %3);\n"
            "}\n"
        ).arg(clothingId, param, value);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, std::string("{\"set\":true,\"clothing_id\":\"") + JsonEscape(clothingId) + "\"}");
    }

    if (command == "suggest_outfit") {
        // Content search for clothing matching a style
        if (!dzApp) return ErrorResponse(id, "No app");
        QString style = ExtractArgString(line, "style");
        DzContentMgr* contentMgr = dzApp->getContentMgr();
        if (!contentMgr) return ErrorResponse(id, "No content manager");
        QStringList filters; filters << "*.duf";
        std::ostringstream oss;
        oss << "{\"suggestions\":[";
        bool first = true;
        for (int i = 0; i < contentMgr->getNumContentDirectories(); ++i) {
            QString dir = contentMgr->getContentDirectoryPath(i);
            QDirIterator it(dir, filters, QDir::Files, QDirIterator::Subdirectories);
            while (it.hasNext()) {
                QString fp = it.next();
                if (!style.isEmpty() && !fp.contains(style, Qt::CaseInsensitive)) continue;
                if (!first) oss << ",";
                first = false;
                oss << "{\"name\":\"" << JsonEscape(it.fileName()) << "\",\"path\":\"" << JsonEscape(fp) << "\"}";
            }
        }
        oss << "],\"count\":" << 0 << "}";
        std::string result = oss.str();
        int total = 0;
        for (int i = 0; i < contentMgr->getNumContentDirectories(); ++i) {
            QString dir = contentMgr->getContentDirectoryPath(i);
            QDirIterator it(dir, filters, QDir::Files, QDirIterator::Subdirectories);
            while (it.hasNext()) {
                QString fp = it.next();
                if (!style.isEmpty() && !fp.contains(style, Qt::CaseInsensitive)) continue;
                total++;
            }
        }
        std::string countStr = "\"count\":" + std::to_string(total) + "}";
        size_t pos = result.rfind("\"count\":");
        if (pos != std::string::npos) result.replace(pos, result.length() - pos - 1, countStr);
        return OkResponse(id, result);
    }

    // ─── Phase 5: Pose & Morph Commands ───────────────────────────────────────

    if (command == "list_poses") {
        // Reuse content search for pose files
        if (!dzApp) return ErrorResponse(id, "No app");
        DzContentMgr* contentMgr = dzApp->getContentMgr();
        if (!contentMgr) return ErrorResponse(id, "No content manager");
        QStringList filters; filters << "*.duf" << "*.pz2";
        std::ostringstream oss;
        oss << "{\"poses\":[";
        bool first = true;
        for (int i = 0; i < contentMgr->getNumContentDirectories(); ++i) {
            QString dir = contentMgr->getContentDirectoryPath(i);
            QDirIterator it(dir, filters, QDir::Files, QDirIterator::Subdirectories);
            while (it.hasNext()) {
                QString fp = it.next();
                QString fn = it.fileName();
                if (!first) oss << ",";
                first = false;
                oss << "{\"name\":\"" << JsonEscape(fn) << "\",\"path\":\"" << JsonEscape(fp) << "\"}";
            }
        }
        oss << "],\"total\":" << (first ? 0 : 1) << "}";
        std::string result = oss.str();
        // Fix total count
        int total = 0;
        for (int i = 0; i < contentMgr->getNumContentDirectories(); ++i) {
            QString dir = contentMgr->getContentDirectoryPath(i);
            QDirIterator it(dir, filters, QDir::Files, QDirIterator::Subdirectories);
            while (it.hasNext()) { it.next(); total++; }
        }
        std::string totalStr = "\"total\":" + std::to_string(total) + "}";
        size_t pos = result.rfind("\"total\":");
        if (pos != std::string::npos) result.replace(pos, result.length() - pos - 1, totalStr);
        return OkResponse(id, result);
    }

    if (command == "save_pose") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString figureId = ExtractArgString(line, "figure_id");
        QString name = ExtractArgString(line, "name");
        if (name.isEmpty()) return ErrorResponse(id, "name is required");
        QString script = QString(
            "var fig = Scene.findNode('%1');\n"
            "if (fig) {\n"
            "  App.savePosePreset(fig, '%2');\n"
            "}\n"
        ).arg(figureId, name);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, std::string("{\"saved\":true,\"preset_name\":\"") + JsonEscape(name) + "\"}");
    }

    if (command == "blend_poses") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString figureId = ExtractArgString(line, "figure_id");
        QString poseA = ExtractArgString(line, "pose_a");
        QString poseB = ExtractArgString(line, "pose_b");
        double blend = ExtractArgString(line, "blend").toDouble();
        if (blend < 0.0) blend = 0.5;
        QString script = QString(
            "var fig = Scene.findNode('%1');\n"
            "if (fig) {\n"
            "  App.loadPosePreset(fig, '%2', %3);\n"
            "  App.loadPosePreset(fig, '%4', %5);\n"
            "}\n"
        ).arg(figureId, poseA, QString::number(1.0 - blend), poseB, QString::number(blend));
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, "{\"applied\":true}");
    }

    if (command == "mirror_pose") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString figureId = ExtractArgString(line, "figure_id");
        QString script = QString(
            "var fig = Scene.findNode('%1');\n"
            "if (fig) {\n"
            "  fig.mirrorPose();\n"
            "}\n"
        ).arg(figureId);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, "{\"mirrored\":true}");
    }

    if (command == "asymmetric_pose") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString figureId = ExtractArgString(line, "figure_id");
        QString left = ExtractArgString(line, "left");
        QString right = ExtractArgString(line, "right");
        QString script = QString(
            "var fig = Scene.findNode('%1');\n"
            "if (fig) {\n"
            "  App.loadPosePreset(fig, 'l:' + '%2');\n"
            "  App.loadPosePreset(fig, 'r:' + '%3');\n"
            "}\n"
        ).arg(figureId, left, right);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, "{\"applied\":true}");
    }

    if (command == "reset_pose") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString figureId = ExtractArgString(line, "figure_id");
        QString poseType = ExtractArgString(line, "pose_type");
        if (poseType.isEmpty()) poseType = "zero";
        QString script = QString(
            "var fig = Scene.findNode('%1');\n"
            "if (fig) {\n"
            "  fig.resetPose('%2');\n"
            "}\n"
        ).arg(figureId, poseType);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, "{\"reset\":true}");
    }

    if (command == "random_pose") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString figureId = ExtractArgString(line, "figure_id");
        QString category = ExtractArgString(line, "category");
        double intensity = ExtractArgString(line, "intensity").toDouble();
        if (intensity <= 0.0) intensity = 0.5;
        QString script = QString(
            "var fig = Scene.findNode('%1');\n"
            "if (fig) {\n"
            "  App.randomizePose(fig, %2);\n"
            "}\n"
        ).arg(figureId, intensity);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, "{\"applied\":true}");
    }

    if (command == "batch_set_morphs") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString figureId = ExtractArgString(line, "figure_id");
        QString morphsStr = ExtractArgString(line, "morphs");
        // morphs is a JSON object: {"morph1": 0.5, "morph2": -0.3}
        // Parse and set each morph
        // Since we don't have a JSON parser inline, use DzScript
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString script = QString(
            "var fig = Scene.findNode('%1');\n"
            "if (fig) {\n"
            "  var morphs = %2;\n"
            "  for (var m in morphs) {\n"
            "    fig.setMorph(m, morphs[m]);\n"
            "  }\n"
            "}\n"
        ).arg(figureId, morphsStr);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, "{\"applied\":true}");
    }

    if (command == "symmetry_morphs") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString figureId = ExtractArgString(line, "figure_id");
        QString direction = ExtractArgString(line, "direction").toLower();
        QString script = QString(
            "var fig = Scene.findNode('%1');\n"
            "if (fig) {\n"
            "  fig.symmetrizeMorphs('%2');\n"
            "}\n"
        ).arg(figureId, direction);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, "{\"applied\":true}");
    }

    if (command == "randomize_morphs") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString figureId = ExtractArgString(line, "figure_id");
        double intensity = ExtractArgString(line, "intensity").toDouble();
        if (intensity <= 0.0) intensity = 0.3;
        QString script = QString(
            "var fig = Scene.findNode('%1');\n"
            "if (fig) {\n"
            "  fig.randomizeMorphs(%2);\n"
            "}\n"
        ).arg(figureId, intensity);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, "{\"applied\":true}");
    }

    if (command == "save_morph_preset") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString figureId = ExtractArgString(line, "figure_id");
        QString presetName = ExtractArgString(line, "preset_name");
        if (presetName.isEmpty()) return ErrorResponse(id, "preset_name is required");
        QString script = QString(
            "var fig = Scene.findNode('%1');\n"
            "if (fig) {\n"
            "  App.saveMorphPreset(fig, '%2');\n"
            "}\n"
        ).arg(figureId, presetName);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, std::string("{\"saved\":true,\"preset_name\":\"") + JsonEscape(presetName) + "\"}");
    }

    if (command == "load_morph_preset") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString figureId = ExtractArgString(line, "figure_id");
        QString presetName = ExtractArgString(line, "preset_name");
        double blend = ExtractArgString(line, "blend").toDouble();
        if (blend <= 0.0) blend = 1.0;
        if (presetName.isEmpty()) return ErrorResponse(id, "preset_name is required");
        QString script = QString(
            "var fig = Scene.findNode('%1');\n"
            "if (fig) {\n"
            "  App.loadMorphPreset(fig, '%2', %3);\n"
            "}\n"
        ).arg(figureId, presetName, QString::number(blend));
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, std::string("{\"loaded\":true,\"preset_name\":\"") + JsonEscape(presetName) + "\"}");
    }

    if (command == "reset_morphs") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString figureId = ExtractArgString(line, "figure_id");
        QString script = QString(
            "var fig = Scene.findNode('%1');\n"
            "if (fig) {\n"
            "  fig.resetMorphs();\n"
            "}\n"
        ).arg(figureId);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, "{\"reset\":true}");
    }

    // ─── Phase 4: Viewport & Environment Commands ─────────────────────────────

    if (command == "set_display_mode") {
        if (!dzApp) return ErrorResponse(id, "No app");
        DzMainWindow* mw = dzApp->getInterface();
        if (!mw) return ErrorResponse(id, "No main window");
        DzViewportMgr* vm = mw->getViewportMgr();
        if (!vm) return ErrorResponse(id, "No viewport manager");
        DzViewport* vp = vm->getActiveViewport();
        if (!vp) return ErrorResponse(id, "No active viewport");
        Dz3DViewport* vp3d = vp->get3DViewport();
        if (!vp3d) return ErrorResponse(id, "No 3D viewport");
        QString mode = ExtractArgString(line, "mode").toLower();
        Dz3DViewport::ShadeStyle style = vp3d->getShadeStyle();
        if (mode == "texture") style = Dz3DViewport::Textured;
        else if (mode == "shaded") style = Dz3DViewport::SmoothShaded;
        else if (mode == "wireframe") style = Dz3DViewport::Wireframe;
        else if (mode == "lit_wireframe") style = Dz3DViewport::LitWireframe;
        else if (mode == "hidden_line") style = Dz3DViewport::HiddenLine;
        else if (mode == "wire_box") style = Dz3DViewport::WireBox;
        else return ErrorResponse(id, QString("Unknown mode: %1").arg(mode));
        vp3d->setShadeStyle(style);
        return OkResponse(id, std::string("{\"mode_set\":true,\"mode\":\"") + JsonEscape(mode) + "\"}");
    }

    if (command == "set_viewport_quality") {
        if (!dzApp) return ErrorResponse(id, "No app");
        DzMainWindow* mw = dzApp->getInterface();
        if (!mw) return ErrorResponse(id, "No main window");
        DzViewportMgr* vm = mw->getViewportMgr();
        if (!vm) return ErrorResponse(id, "No viewport manager");
        DzViewport* vp = vm->getActiveViewport();
        if (!vp) return ErrorResponse(id, "No active viewport");
        Dz3DViewport* vp3d = vp->get3DViewport();
        if (!vp3d) return ErrorResponse(id, "No 3D viewport");
        // Dz3DViewport has no setTextureResolution — use script fallback
        QString texRes = ExtractArgString(line, "texture_resolution");
        if (!texRes.isEmpty() && g_scriptExecutor) {
            QString script = QString("var vp = Scene.getActiveViewport();\n"
                "if (vp) vp.setTextureResolution(%1);\n").arg(texRes);
            QString result;
            std::mutex mtx; std::condition_variable cv; bool done = false;
            RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
            QCoreApplication::postEvent(g_scriptExecutor, evt);
            std::unique_lock<std::mutex> lock(mtx);
            cv.wait(lock, [&done]{ return done; });
        }
        return OkResponse(id, "{\"applied\":true}");
    }

    if (command == "toggle_guide") {
        // No C++ API for guides — use DzScript
        QString guide = ExtractArgString(line, "guide");
        QString showStr = ExtractArgString(line, "show");
        bool show = showStr.toLower() != "false";
        DzScript dzScript;
        dzScript.addLine(QString(
            "var g = App.getGuide('%1');\n"
            "if (g) g.setVisible(%2);\n"
        ).arg(guide).arg(show ? "true" : "false"));
        dzScript.execute();
        return OkResponse(id, std::string("{\"toggled\":true,\"guide\":\"") +
                          JsonEscape(guide) + "\",\"visible\":" + (show ? "true" : "false") + "}");
    }

    if (command == "set_viewport_camera") {
        if (!dzApp) return ErrorResponse(id, "No app");
        DzMainWindow* mw = dzApp->getInterface();
        if (!mw) return ErrorResponse(id, "No main window");
        DzViewportMgr* vm = mw->getViewportMgr();
        if (!vm) return ErrorResponse(id, "No viewport manager");
        DzViewport* vp = vm->getActiveViewport();
        if (!vp) return ErrorResponse(id, "No active viewport");
        Dz3DViewport* vp3d = vp->get3DViewport();
        if (!vp3d) return ErrorResponse(id, "No 3D viewport");
        QString camera = ExtractArgString(line, "camera");
        if (camera.isEmpty()) return ErrorResponse(id, "camera is required");
        DzCamera* cam = dzScene ? dzScene->findCamera(camera) : nullptr;
        if (!cam) return ErrorResponse(id, QString("Camera not found: %1").arg(camera));
        vp3d->setCamera(cam);
        return OkResponse(id, std::string("{\"camera_set\":true,\"camera\":\"") + JsonEscape(camera) + "\"}");
    }

    if (command == "set_viewport_lighting") {
        if (!dzApp) return ErrorResponse(id, "No app");
        DzMainWindow* mw = dzApp->getInterface();
        if (!mw) return ErrorResponse(id, "No main window");
        DzViewportMgr* vm = mw->getViewportMgr();
        if (!vm) return ErrorResponse(id, "No viewport manager");
        DzViewport* vp = vm->getActiveViewport();
        if (!vp) return ErrorResponse(id, "No active viewport");
        Dz3DViewport* vp3d = vp->get3DViewport();
        if (!vp3d) return ErrorResponse(id, "No 3D viewport");
        // Dz3DViewport has no setLightingMode — use script fallback
        QString lighting = ExtractArgString(line, "lighting").toLower();
        if (!g_scriptExecutor) {
            return OkResponse(id, std::string("{\"lighting_set\":false,\"error\":\"set_lighting_mode requires script executor\"}"));
        }
        QString script = QString("var vp = Scene.getActiveViewport();\n"
            "if (vp) vp.setLightingMode('%1');\n").arg(lighting);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, std::string("{\"lighting_set\":true,\"lighting\":\"") + JsonEscape(lighting) + "\"}");
    }

    if (command == "center_view") {
        if (!dzApp) return ErrorResponse(id, "No app");
        DzMainWindow* mw = dzApp->getInterface();
        if (!mw) return ErrorResponse(id, "No main window");
        DzViewportMgr* vm = mw->getViewportMgr();
        if (!vm) return ErrorResponse(id, "No viewport manager");
        DzViewport* vp = vm->getActiveViewport();
        if (!vp) return ErrorResponse(id, "No active viewport");
        Dz3DViewport* vp3d = vp->get3DViewport();
        if (!vp3d) return ErrorResponse(id, "No 3D viewport");
        QString nodeId = ExtractArgString(line, "node_id");
        if (!nodeId.isEmpty() && dzScene) {
            DzNode* node = dzScene->findNode(nodeId);
            if (node) {
                vp3d->frameCameraOnBox(node->getWSBoundingBox());
                return OkResponse(id, std::string("{\"centered\":true,\"node\":\"") + JsonEscape(nodeId) + "\"}");
            }
            return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
        }
        vp3d->frameCamera();
        return OkResponse(id, "{\"centered\":true}");
    }

    // ─── Environment Commands (all use DzScript — no C++ SDK environment API) ──

    if (command == "set_environment") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString envType = ExtractArgString(line, "type");
        QString preset = ExtractArgString(line, "preset");
        QString intensity = ExtractArgString(line, "intensity");
        QString rotation = ExtractArgString(line, "rotation");
        QString script = QString(
            "var env = Scene.getEnvironment();\n"
            "if (env) {\n"
            "  if ('%1'.length > 0) env.setType('%1');\n"
            "  if ('%2'.length > 0) env.loadPreset('%2');\n"
            "  if ('%3'.length > 0) env.setIntensity(%3);\n"
            "  if ('%4'.length > 0) env.setRotation(%4);\n"
            "}\n"
        ).arg(envType, preset, intensity, rotation);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, "{\"applied\":true}");
    }

    if (command == "add_ground") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString gtype = ExtractArgString(line, "type");
        QString size = ExtractArgString(line, "size");
        QString script = QString(
            "var ground = Scene.addGround();\n"
            "if (ground) {\n"
            "  if ('%1'.length > 0) ground.setType('%1');\n"
            "  if ('%2'.length > 0) ground.setSize(%2);\n"
            "}\n"
        ).arg(gtype, size);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, "{\"created\":true}");
    }

    if (command == "set_fog") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString enabled = ExtractArgString(line, "enabled");
        QString density = ExtractArgString(line, "density");
        QString color = ExtractArgString(line, "color");
        QString distance = ExtractArgString(line, "distance");
        QString script = QString(
            "if ('%1'.length > 0) Scene.setFogEnabled(%1);\n"
            "if ('%2'.length > 0) Scene.setFogDensity(%2);\n"
            "if ('%3'.length > 0) Scene.setFogColor(new DzColor(%3));\n"
            "if ('%4'.length > 0) Scene.setFogDistance(%4);\n"
        ).arg(enabled, density, color, distance);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, "{\"applied\":true}");
    }

    if (command == "set_sun") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString direction = ExtractArgString(line, "direction");
        QString intensity = ExtractArgString(line, "intensity");
        QString script = QString(
            "var sun = Scene.getSun();\n"
            "if (sun) {\n"
            "  if ('%1'.length > 0) sun.setDirection(new DzVec3(%1));\n"
            "  if ('%2'.length > 0) sun.setIntensity(%2);\n"
            "}\n"
        ).arg(direction, intensity);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, "{\"applied\":true}");
    }

    if (command == "set_time_of_day") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString time = ExtractArgString(line, "time");
        QString script = QString(
            "var env = Scene.getEnvironment();\n"
            "if (env) env.setTimeOfDay(%1);\n"
        ).arg(time);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, std::string("{\"applied\":true,\"time\":") + time.toStdString() + "}");
    }

    if (command == "add_env_light") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString ltype = ExtractArgString(line, "type");
        QString intensity = ExtractArgString(line, "intensity");
        QString script = QString(
            "var light = Scene.addLight('%1');\n"
            "if (light) {\n"
            "  if ('%2'.length > 0) light.setIntensity(%2);\n"
            "}\n"
        ).arg(ltype, intensity);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, "{\"created\":true}");
    }

    if (command == "rotate_environment") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString rotation = ExtractArgString(line, "rotation");
        QString script = QString(
            "var env = Scene.getEnvironment();\n"
            "if (env) env.setRotation(%1);\n"
        ).arg(rotation);
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, std::string("{\"rotation\":") + rotation.toStdString() + "}");
    }

    if (command == "get_environment_info") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString script = QString(
            "var env = Scene.getEnvironment();\n"
            "var info = {};\n"
            "if (env) {\n"
            "  info.current_preset = env.getPreset();\n"
            "  info.intensity = env.getIntensity();\n"
            "  info.rotation = env.getRotation();\n"
            "}\n"
            "JSON.stringify(info);\n"
        );
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        if (result.isEmpty()) result = "{}";
        return OkResponse(id, result.toStdString());
    }

    if (command == "clear_environment") {
        if (!g_scriptExecutor) return ErrorResponse(id, "ScriptExecutor not initialized");
        QString hdri = ExtractArgString(line, "hdri");
        QString ground = ExtractArgString(line, "ground");
        QString fog = ExtractArgString(line, "fog");
        QString script;
        if (hdri.toLower() != "false") script += "Scene.getEnvironment().clear();\n";
        if (ground.toLower() != "false") script += "Scene.removeGround();\n";
        if (fog.toLower() != "false") script += "Scene.setFogEnabled(false);\n";
        QString result;
        std::mutex mtx; std::condition_variable cv; bool done = false;
        RunScriptEvent* evt = new RunScriptEvent(script, "{}", &result, &mtx, &cv, &done);
        QCoreApplication::postEvent(g_scriptExecutor, evt);
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [&done]{ return done; });
        return OkResponse(id, "{\"cleared\":true}");
    }

    // ─── Phase 2: Selection & Camera Commands ──────────────────────────────────

    if (command == "select_all") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        dzScene->selectAllNodes(true);
        int count = dzScene->getNumSelectedNodes();
        return OkResponse(id, std::string("{\"selected_count\":") + std::to_string(count) + "}");
    }

    if (command == "deselect_all") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        dzScene->selectAllNodes(false);
        return OkResponse(id, "{\"deselected\":true}");
    }

    if (command == "invert_selection") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        // No C++ SDK invert method — use DzScript
        DzScript dzScript;
        dzScript.addLine(
            "var sel = Scene.getSelectedNodeList();\n"
            "Scene.selectAllNodes(false);\n"
            "for (var i = 0; i < Scene.getNumNodes(); i++) {\n"
            "  var n = Scene.getNode(i);\n"
            "  if (sel.indexOf(n) < 0) {\n"
            "    Scene.setPrimarySelection(n);\n"
            "  }\n"
            "}\n"
        );
        if (dzScript.execute()) {
            return OkResponse(id, std::string("{\"inverted\":true,\"selected_count\":") +
                              std::to_string(dzScene->getNumSelectedNodes()) + "}");
        }
        return ErrorResponse(id, "Invert selection failed");
    }

    if (command == "select_children") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
        int numChildren = node->getNumNodeChildren();
        dzScene->selectAllNodes(false);
        for (int i = 0; i < numChildren; ++i) {
            DzNode* child = node->getNodeChild(i);
            if (child) {
                dzScene->setPrimarySelection(child);
            }
        }
        std::ostringstream oss;
        oss << "{\"parent\":\"" << JsonEscape(node->getName()) << "\",\"children_selected\":" << numChildren << "}";
        return OkResponse(id, oss.str());
    }

    if (command == "select_parent") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
        DzNode* parent = node->getNodeParent();
        if (!parent) return ErrorResponse(id, "Node has no parent");
        dzScene->selectAllNodes(false);
        dzScene->setPrimarySelection(parent);
        return OkResponse(id, std::string("{\"parent\":\"") + JsonEscape(parent->getName()) + "\"}");
    }

    if (command == "get_selection_count") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        int count = dzScene->getNumSelectedNodes();
        return OkResponse(id, std::string("{\"selected_count\":") + std::to_string(count) + "}");
    }

    if (command == "create_camera") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString name = ExtractArgString(line, "name");
        if (name.isEmpty()) name = "New Camera";
        DzBasicCamera* cam = new DzBasicCamera(DzCamera::PERSPECTIVE_CAMERA, false);
        cam->setName(name);
        QString focalLen = ExtractArgString(line, "focal_length");
        if (!focalLen.isEmpty()) cam->setFocalLength(focalLen.toDouble());
        QString fStop = ExtractArgString(line, "f_stop");
        if (!fStop.isEmpty()) cam->setFStop(fStop.toDouble());
        dzScene->addNode(cam);
        return OkResponse(id, std::string("{\"created\":true,\"camera\":\"") + JsonEscape(name) + "\"}");
    }

    if (command == "delete_camera") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString cameraName = ExtractArgString(line, "camera_name");
        if (cameraName.isEmpty()) return ErrorResponse(id, "camera_name is required");
        if (DeleteNode(cameraName)) {
            return OkResponse(id, std::string("{\"deleted\":true,\"camera\":\"") + JsonEscape(cameraName) + "\"}");
        }
        return ErrorResponse(id, QString("Camera not found: %1").arg(cameraName));
    }

    if (command == "set_camera_target") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString cameraName = ExtractArgString(line, "camera_name");
        QString targetStr = ExtractArgString(line, "target");
        if (targetStr.isEmpty()) return ErrorResponse(id, "target is required [x, y, z]");
        DzCamera* cam = nullptr;
        if (cameraName.isEmpty()) {
            // Use active viewport camera
            if (dzApp) {
                DzMainWindow* mw = dzApp->getInterface();
                if (mw) {
                    DzViewportMgr* vm = mw->getViewportMgr();
                    if (vm) {
                        DzViewport* vp = vm->getActiveViewport();
                        if (vp) {
                            Dz3DViewport* vp3d = vp->get3DViewport();
                            if (vp3d) cam = vp3d->getCamera();
                        }
                    }
                }
            }
        } else {
            cam = dzScene->findCamera(cameraName);
        }
        if (!cam) return ErrorResponse(id, QString("Camera not found: %1").arg(cameraName));
        targetStr.remove('[').remove(']');
        QStringList parts = targetStr.split(',');
        if (parts.size() != 3) return ErrorResponse(id, "target must be [x, y, z]");
        cam->aimAt(DzVec3(
            parts[0].trimmed().toFloat(),
            parts[1].trimmed().toFloat(),
            parts[2].trimmed().toFloat()
        ));
        return OkResponse(id, std::string("{\"camera\":\"") + JsonEscape(cam->getName()) +
                          "\",\"target\":[" + targetStr.toStdString() + "]}");
    }

    if (command == "get_camera_properties") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString cameraName = ExtractArgString(line, "camera_name");
        DzCamera* cam = nullptr;
        if (cameraName.isEmpty()) {
            if (dzApp) {
                DzMainWindow* mw = dzApp->getInterface();
                if (mw) { DzViewportMgr* vm = mw->getViewportMgr();
                if (vm) { DzViewport* vp = vm->getActiveViewport();
                if (vp) { Dz3DViewport* vp3d = vp->get3DViewport();
                if (vp3d) cam = vp3d->getCamera(); }}}
            }
        } else {
            cam = dzScene->findCamera(cameraName);
        }
        if (!cam) return ErrorResponse(id, QString("Camera not found: %1").arg(cameraName));
        std::ostringstream oss;
        oss << "{\"camera\":\"" << JsonEscape(cam->getName()) << "\"";
        oss << ",\"focal_length\":" << cam->getFocalLength();
        oss << ",\"focal_distance\":" << cam->getFocalDistance();
        oss << ",\"aspect_ratio\":" << cam->getAspectRatio();
        DzBasicCamera* basicCam = qobject_cast<DzBasicCamera*>(cam);
        if (basicCam) {
            oss << ",\"f_stop\":" << basicCam->getFStop();
            oss << ",\"depth_of_field\":" << (basicCam->getDepthOfField() ? "true" : "false");
        }
        DzVec3 fp = cam->getFocalPoint();
        oss << ",\"focal_point\":[" << fp.m_x << "," << fp.m_y << "," << fp.m_z << "]";
        oss << ",\"type\":";
        switch (cam->getType()) {
            case DzCamera::PERSPECTIVE_CAMERA: oss << "\"perspective\""; break;
            case DzCamera::ORTHO_CAMERA: oss << "\"orthographic\""; break;
            default: oss << "\"unknown\""; break;
        }
        oss << "}";
        return OkResponse(id, oss.str());
    }

    // ─── Phase 1: Scene & Node Commands ────────────────────────────────────────

    if (command == "set_visibility") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
        QString visibleStr = ExtractArgString(line, "visible").toLower();
        bool visible = (visibleStr == "true" || visibleStr == "1");
        node->setVisible(visible);
        return OkResponse(id, std::string("{\"node\":\"") + JsonEscape(node->getName()) +
                          "\",\"visible\":" + (visible ? "true" : "false") + "}");
    }

    if (command == "delete_nodes") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeIdsStr = ExtractArgString(line, "node_ids");
        nodeIdsStr = nodeIdsStr.trimmed();
        if (nodeIdsStr.startsWith('[') && nodeIdsStr.endsWith(']')) {
            nodeIdsStr = nodeIdsStr.mid(1, nodeIdsStr.length() - 2);
        }
        QStringList ids = nodeIdsStr.split(',', QString::SkipEmptyParts);
        int deleted = 0;
        for (const QString& nid : ids) {
            QString tid = nid.trimmed();
            if (!tid.isEmpty()) {
                tid.remove('"').remove('\'');
                if (DeleteNode(tid)) deleted++;
            }
        }
        std::ostringstream oss;
        oss << "{\"deleted\":" << deleted << ",\"total\":" << ids.size() << "}";
        return OkResponse(id, oss.str());
    }

    if (command == "duplicate_nodes") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeIdsStr = ExtractArgString(line, "node_ids");
        nodeIdsStr = nodeIdsStr.trimmed();
        if (nodeIdsStr.startsWith('[') && nodeIdsStr.endsWith(']')) {
            nodeIdsStr = nodeIdsStr.mid(1, nodeIdsStr.length() - 2);
        }
        QStringList ids = nodeIdsStr.split(',', QString::SkipEmptyParts);
        int copies = ExtractArgString(line, "copies").toInt();
        if (copies < 1) copies = 1;
        if (copies > 10) copies = 10;

        // DzScene has no duplicateNode in C++ SDK, so use DzScript
        QString script;
        for (const QString& nid : ids) {
            QString tid = nid.trimmed().remove('"').remove('\'');
            if (tid.isEmpty()) continue;
            script += QString(
                "var src = Scene.findNode('%1');\n"
                "if (src) {\n"
                "  for (var i = 0; i < %2; i++) {\n"
                "    var dup = Scene.duplicateNode(src);\n"
                "    if (dup) dup.setName(src.getName() + '_copy_' + i);\n"
                "  }\n"
                "}\n"
            ).arg(tid).arg(copies);
        }

        if (script.isEmpty()) return ErrorResponse(id, "No valid node IDs provided");

        DzScript dzScript;
        dzScript.addLine(script);
        if (dzScript.execute()) {
            return OkResponse(id, std::string("{\"duplicated\":true,\"copies\":") +
                              std::to_string(copies * ids.size()) + "}");
        }
        return ErrorResponse(id, "Duplicate failed");
    }

    if (command == "rename_node") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        QString newName = ExtractArgString(line, "new_name");
        if (newName.isEmpty()) return ErrorResponse(id, "new_name is required");
        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
        QString oldName = node->getName();
        node->setName(newName);
        return OkResponse(id, std::string("{\"old_name\":\"") + JsonEscape(oldName) +
                          "\",\"new_name\":\"" + JsonEscape(newName) + "\"}");
    }

    if (command == "group_nodes") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString parentId = ExtractArgString(line, "parent_id");
        QString childIdsStr = ExtractArgString(line, "child_ids");
        childIdsStr = childIdsStr.trimmed();
        if (childIdsStr.startsWith('[') && childIdsStr.endsWith(']')) {
            childIdsStr = childIdsStr.mid(1, childIdsStr.length() - 2);
        }
        QStringList childIds = childIdsStr.split(',', QString::SkipEmptyParts);

        DzNode* parentNode = nullptr;
        if (parentId.isEmpty()) {
            // Create a new null node as parent
            parentNode = new DzNode();
            parentNode->setName("Group");
            dzScene->addNode(parentNode);
        } else {
            parentNode = dzScene->findNode(parentId);
            if (!parentNode) return ErrorResponse(id, QString("Parent node not found: %1").arg(parentId));
        }

        int grouped = 0;
        for (const QString& cid : childIds) {
            QString tid = cid.trimmed().remove('"').remove('\'');
            if (tid.isEmpty()) continue;
            DzNode* child = dzScene->findNode(tid);
            if (child && child != parentNode) {
                parentNode->addNodeChild(child);
                grouped++;
            }
        }
        std::ostringstream oss;
        oss << "{\"parent\":\"" << JsonEscape(parentNode->getName()) << "\",\"grouped\":" << grouped << "}";
        return OkResponse(id, oss.str());
    }

    if (command == "merge_scene") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString filepath = ExtractArgString(line, "filepath");
        if (filepath.isEmpty()) return ErrorResponse(id, "filepath is required");
        DzError err = dzScene->loadScene(filepath, DzScene::MergeFile);
        if (err == DZ_NO_ERROR) {
            return OkResponse(id, std::string("{\"merged\":true,\"path\":\"") + JsonEscape(filepath) + "\"}");
        }
        return ErrorResponse(id, QString("Merge failed: error code %1").arg((int)err));
    }

    if (command == "get_scene_stats") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        int totalNodes = 0, figures = 0, cameras = 0, lights = 0, props = 0, bones = 0, others = 0;
        for (int i = 0; i < dzScene->getNumNodes(); ++i) {
            DzNode* node = dzScene->getNode(i);
            if (!node) continue;
            totalNodes++;
            if (qobject_cast<DzFigure*>(node))           { figures++; }
            else if (qobject_cast<DzCamera*>(node))       { cameras++; }
            else if (qobject_cast<DzLight*>(node))        { lights++; }
            else if (qobject_cast<DzBone*>(node))         { bones++; }
            else                                          { props++; }
        }
        std::ostringstream oss;
        oss << "{\"total_nodes\":" << totalNodes;
        oss << ",\"figures\":" << figures;
        oss << ",\"cameras\":" << cameras;
        oss << ",\"lights\":" << lights;
        oss << ",\"props\":" << props;
        oss << ",\"bones\":" << bones;
        oss << ",\"others\":" << others << "}";
        return OkResponse(id, oss.str());
    }

    if (command == "list_figures") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        bool includeDetails = ExtractArgString(line, "include_details").toLower() == "true";
        std::ostringstream oss;
        oss << "{\"figures\":[";
        int figCount = 0;
        for (int i = 0; i < dzScene->getNumNodes(); ++i) {
            DzNode* node = dzScene->getNode(i);
            if (!node) continue;
            DzFigure* figure = qobject_cast<DzFigure*>(node);
            if (!figure) continue;
            if (figCount > 0) oss << ",";
            oss << "{\"name\":\"" << JsonEscape(node->getName()) << "\"";
            if (includeDetails) {
                DzVec3 pos = node->getWSPos();
                oss << ",\"label\":\"" << JsonEscape(node->getLabel()) << "\"";
                oss << ",\"position\":[" << pos.m_x << "," << pos.m_y << "," << pos.m_z << "]";
                oss << ",\"num_bones\":" << figure->getAllBones().size();
            }
            oss << "}";
            figCount++;
        }
        oss << "],\"count\":" << figCount << "}";
        return OkResponse(id, oss.str());
    }

    if (command == "remove_figure") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString figureId = ExtractArgString(line, "figure_id");
        if (DeleteNode(figureId)) {
            return OkResponse(id, "{\"removed\":true}");
        }
        return ErrorResponse(id, QString("Figure not found: %1").arg(figureId));
    }

    if (command == "apply_figure_preset") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString figureId = ExtractArgString(line, "figure_id");
        QString presetPath = ExtractArgString(line, "preset_path");
        if (presetPath.isEmpty()) return ErrorResponse(id, "preset_path is required");
        if (!figureId.isEmpty()) {
            DzNode* target = dzScene->findNode(figureId);
            if (target) {
                dzScene->setPrimarySelection(target);
            }
        }
        if (OpenContentFile(presetPath, true)) {
            return OkResponse(id, std::string("{\"applied\":true,\"preset\":\"") + JsonEscape(presetPath) + "\"}");
        }
        return ErrorResponse(id, QString("Failed to apply preset: %1").arg(presetPath));
    }

    if (command == "list_props") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString category = ExtractArgString(line, "category").toLower();
        std::ostringstream oss;
        oss << "{\"props\":[";
        int propCount = 0;
        for (int i = 0; i < dzScene->getNumNodes(); ++i) {
            DzNode* node = dzScene->getNode(i);
            if (!node) continue;
            if (qobject_cast<DzFigure*>(node)) continue;
            if (qobject_cast<DzCamera*>(node)) continue;
            if (qobject_cast<DzLight*>(node)) continue;
            if (qobject_cast<DzBone*>(node)) continue;

            QString name = node->getName();
            if (!category.isEmpty() && !name.contains(category, Qt::CaseInsensitive)) continue;

            if (propCount > 0) oss << ",";
            DzVec3 pos = node->getWSPos();
            oss << "{\"name\":\"" << JsonEscape(name) << "\"";
            oss << ",\"label\":\"" << JsonEscape(node->getLabel()) << "\"";
            oss << ",\"position\":[" << pos.m_x << "," << pos.m_y << "," << pos.m_z << "]}";
            propCount++;
        }
        oss << "],\"count\":" << propCount << "}";
        return OkResponse(id, oss.str());
    }

    if (command == "load_prop") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString name = ExtractArgString(line, "name");
        QString category = ExtractArgString(line, "category");
        QString positionStr = ExtractArgString(line, "position");

        if (name.isEmpty()) return ErrorResponse(id, "name is required");

        // Search content library for the prop
        if (!dzApp) return ErrorResponse(id, "No app");
        DzContentMgr* contentMgr = dzApp->getContentMgr();
        if (!contentMgr) return ErrorResponse(id, "No content manager");

        QStringList filters;
        if (!category.isEmpty()) {
            filters << QString("*.duf");
        } else {
            filters << "*.duf";
        }

        QString foundPath;
        for (int i = 0; i < contentMgr->getNumContentDirectories(); ++i) {
            QString dir = contentMgr->getContentDirectoryPath(i);
            QDirIterator it(dir, filters, QDir::Files, QDirIterator::Subdirectories);
            while (it.hasNext()) {
                QString filePath = it.next();
                QString fileName = it.fileName();
                if (fileName.contains(name, Qt::CaseInsensitive) &&
                    (category.isEmpty() || filePath.contains(category, Qt::CaseInsensitive))) {
                    foundPath = filePath;
                    break;
                }
            }
            if (!foundPath.isEmpty()) break;
        }

        if (foundPath.isEmpty()) {
            return ErrorResponse(id, QString("Prop not found: %1").arg(name));
        }

        if (!OpenContentFile(foundPath, true)) {
            return ErrorResponse(id, QString("Failed to load prop: %1").arg(foundPath));
        }

        // If position specified, move the most recently added node
        if (!positionStr.isEmpty()) {
            DzNode* lastNode = dzScene->getNode(dzScene->getNumNodes() - 1);
            if (lastNode) {
                positionStr.remove('[').remove(']');
                QStringList parts = positionStr.split(',');
                if (parts.size() == 3) {
                    lastNode->setWSPos(DzVec3(
                        parts[0].trimmed().toFloat(),
                        parts[1].trimmed().toFloat(),
                        parts[2].trimmed().toFloat()
                    ));
                }
            }
        }

        return OkResponse(id, std::string("{\"loaded\":true,\"path\":\"") + JsonEscape(foundPath) + "\"}");
    }

    if (command == "position_prop") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
        QString posStr = ExtractArgString(line, "position");
        if (posStr.isEmpty()) return ErrorResponse(id, "position is required");
        posStr.remove('[').remove(']');
        QStringList parts = posStr.split(',');
        if (parts.size() != 3) return ErrorResponse(id, "position must be [x, y, z]");
        node->setWSPos(DzVec3(
            parts[0].trimmed().toFloat(),
            parts[1].trimmed().toFloat(),
            parts[2].trimmed().toFloat()
        ));
        return OkResponse(id, std::string("{\"node\":\"") + JsonEscape(node->getName()) +
                          "\",\"position\":[" + posStr.toStdString() + "]}");
    }

    if (command == "rotate_prop") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
        QString rotStr = ExtractArgString(line, "rotation");
        if (rotStr.isEmpty()) return ErrorResponse(id, "rotation is required");
        rotStr.remove('[').remove(']');
        QStringList parts = rotStr.split(',');
        if (parts.size() != 4) return ErrorResponse(id, "rotation must be [x, y, z, w] (quaternion)");
        node->setWSRot(DzQuat(
            parts[0].trimmed().toFloat(),
            parts[1].trimmed().toFloat(),
            parts[2].trimmed().toFloat(),
            parts[3].trimmed().toFloat()
        ));
        return OkResponse(id, std::string("{\"node\":\"") + JsonEscape(node->getName()) +
                          "\",\"rotation\":[" + rotStr.toStdString() + "]}");
    }

    if (command == "scale_prop") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
        QString scaleStr = ExtractArgString(line, "scale");
        if (scaleStr.isEmpty()) return ErrorResponse(id, "scale is required");
        scaleStr.remove('[').remove(']');
        QStringList parts = scaleStr.split(',');
        std::string scaleStd = scaleStr.toStdString();
        if (parts.size() == 1) {
            float s = parts[0].trimmed().toFloat();
            float vals[12] = {s, 0, 0, 0, s, 0, 0, 0, s, 0, 0, 0};
            node->setWSScale(DzMatrix3(vals));
        } else if (parts.size() == 3) {
            float vals[12] = {
                parts[0].trimmed().toFloat(), 0, 0, 0,
                0, parts[1].trimmed().toFloat(), 0, 0,
                0, 0, parts[2].trimmed().toFloat(), 0
            };
            node->setWSScale(DzMatrix3(vals));
        } else {
            return ErrorResponse(id, "scale must be a single number or [sx, sy, sz]");
        }
        return OkResponse(id, std::string("{\"node\":\"") + JsonEscape(node->getName()) +
                          "\",\"scale\":[" + scaleStd + "]}");
    }

    if (command == "set_camera_transform") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString cameraId = ExtractArgString(line, "camera_id");
        if (cameraId.isEmpty()) return ErrorResponse(id, "camera_id required");
        DzCamera* cam = nullptr;
        for (int i = 0; i < dzScene->getNumCameras(); i++) {
            DzCamera* c = dzScene->getCamera(i);
            if (c && c->getName() == cameraId) { cam = c; break; }
        }
        if (!cam) return ErrorResponse(id, QString("Camera not found: %1").arg(cameraId));
        DzNode* camNode = qobject_cast<DzNode*>(cam);
        if (!camNode) return ErrorResponse(id, "Camera is not a node");
        QString posStr = ExtractArgString(line, "position");
        if (!posStr.isEmpty()) {
            posStr.remove('[').remove(']');
            QStringList parts = posStr.split(',');
            if (parts.size() == 3) {
                camNode->setWSPos(DzVec3(
                    parts[0].trimmed().toFloat(),
                    parts[1].trimmed().toFloat(),
                    parts[2].trimmed().toFloat()
                ));
            }
        }
        QString targetStr = ExtractArgString(line, "target");
        if (!targetStr.isEmpty()) {
            targetStr.remove('[').remove(']');
            QStringList parts = targetStr.split(',');
            if (parts.size() == 3) {
                cam->aimAt(DzVec3(
                    parts[0].trimmed().toFloat(),
                    parts[1].trimmed().toFloat(),
                    parts[2].trimmed().toFloat()
                ));
            }
        }
        return OkResponse(id, std::string("{\"camera\":\"") + JsonEscape(cameraId) + "\"}");
    }
    if (command == "set_focal_length") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString cameraId = ExtractArgString(line, "camera_id");
        if (cameraId.isEmpty()) return ErrorResponse(id, "camera_id required");
        DzCamera* cam = nullptr;
        for (int i = 0; i < dzScene->getNumCameras(); i++) {
            DzCamera* c = dzScene->getCamera(i);
            if (c && c->getName() == cameraId) { cam = c; break; }
        }
        if (!cam) return ErrorResponse(id, QString("Camera not found: %1").arg(cameraId));
        QString focalLen = ExtractArgString(line, "focal_length");
        if (focalLen.isEmpty()) return ErrorResponse(id, "focal_length required");
        cam->setFocalLength(focalLen.toDouble());
        return OkResponse(id, std::string("{\"camera\":\"") + JsonEscape(cameraId) + "\",\"focal_length\":" + focalLen.toStdString() + "}");
    }
    if (command == "set_aperture") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString cameraId = ExtractArgString(line, "camera_id");
        if (cameraId.isEmpty()) return ErrorResponse(id, "camera_id required");
        DzCamera* cam = nullptr;
        for (int i = 0; i < dzScene->getNumCameras(); i++) {
            DzCamera* c = dzScene->getCamera(i);
            if (c && c->getName() == cameraId) { cam = c; break; }
        }
        if (!cam) return ErrorResponse(id, QString("Camera not found: %1").arg(cameraId));
        DzBasicCamera* basicCam = qobject_cast<DzBasicCamera*>(cam);
        if (!basicCam) return ErrorResponse(id, "Camera does not support aperture settings");
        QString fStop = ExtractArgString(line, "f_stop");
        if (!fStop.isEmpty()) basicCam->setFStop(fStop.toDouble());
        QString dofStr = ExtractArgString(line, "enable_dof");
        if (!dofStr.isEmpty()) {
            bool enableDof = (dofStr == "true" || dofStr == "1");
            basicCam->setDepthOfField(enableDof);
        }
        QString focusDist = ExtractArgString(line, "focus_distance");
        if (!focusDist.isEmpty()) cam->setFocalDistance(focusDist.toDouble());
        return OkResponse(id, std::string("{\"camera\":\"") + JsonEscape(cameraId) + "\",\"f_stop\":" + fStop.toStdString() + "}");
    }
    if (command == "focus_camera") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString cameraId = ExtractArgString(line, "camera_id");
        if (cameraId.isEmpty()) return ErrorResponse(id, "camera_id required");
        DzCamera* cam = nullptr;
        for (int i = 0; i < dzScene->getNumCameras(); i++) {
            DzCamera* c = dzScene->getCamera(i);
            if (c && c->getName() == cameraId) { cam = c; break; }
        }
        if (!cam) return ErrorResponse(id, QString("Camera not found: %1").arg(cameraId));
        QString targetId = ExtractArgString(line, "target");
        if (targetId.isEmpty()) return ErrorResponse(id, "target required");
        DzNode* targetNode = dzScene->findNode(targetId);
        if (!targetNode) return ErrorResponse(id, QString("Target node not found: %1").arg(targetId));
        DzVec3 targetPos = targetNode->getWSPos();
        QString offsetStr = ExtractArgString(line, "offset");
        if (!offsetStr.isEmpty()) {
            offsetStr.remove('[').remove(']');
            QStringList parts = offsetStr.split(',');
            if (parts.size() == 3) {
                targetPos = targetPos + DzVec3(
                    parts[0].trimmed().toFloat(),
                    parts[1].trimmed().toFloat(),
                    parts[2].trimmed().toFloat()
                );
            }
        }
        cam->aimAt(targetPos);
        return OkResponse(id, std::string("{\"camera\":\"") + JsonEscape(cameraId) + "\",\"target\":\"" + JsonEscape(targetId) + "\"}");
    }
    // --- Selection Map Commands ----------------------------------------------

    if (command == "selection_map_list") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
        DzSelectionMap* selMap = node->getSelectionMap();
        if (!selMap) return OkResponse(id, "{\"maps\":[],\"count\":0}");
        // DzSelectionMap itself is the map � return info
        int numPairs = selMap->getNumPairs();
        std::ostringstream oss;
        oss << "{\"map_name\":\"" << JsonEscape(selMap->getName()) << "\",";
        oss << "\"pairs\":[";
        for (int i = 0; i < numPairs; ++i) {
            if (i > 0) oss << ",";
            DzNode* pairNode = selMap->getPairNode(i);
            oss << "{\"index\":" << i;
            oss << ",\"face_group\":\"" << JsonEscape(selMap->getPairGroup(i)) << "\"";
            oss << ",\"node\":\"" << (pairNode ? JsonEscape(pairNode->getName()) : "null") << "\"}";
        }
        oss << "],\"count\":" << numPairs << "}";
        return OkResponse(id, oss.str());
    }

    if (command == "selection_map_get_pairs") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
        DzSelectionMap* selMap = node->getSelectionMap();
        if (!selMap) return ErrorResponse(id, "No selection map on node");
        int numPairs = selMap->getNumPairs();
        std::ostringstream oss;
        oss << "{\"pairs\":[";
        for (int i = 0; i < numPairs; ++i) {
            if (i > 0) oss << ",";
            DzNode* pairNode = selMap->getPairNode(i);
            oss << "{\"index\":" << i;
            oss << ",\"face_group\":\"" << JsonEscape(selMap->getPairGroup(i)) << "\"";
            oss << ",\"node\":\"" << (pairNode ? JsonEscape(pairNode->getName()) : "null") << "\"}";
        }
        oss << "],\"count\":" << numPairs << "}";
        return OkResponse(id, oss.str());
    }

    if (command == "selection_map_add_pair") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
        DzSelectionMap* selMap = node->getSelectionMap();
        if (!selMap) return ErrorResponse(id, "No selection map on node");
        QString faceGroup = ExtractArgString(line, "face_group");
        QString targetNodeName = ExtractArgString(line, "target_node");
        if (faceGroup.isEmpty()) return ErrorResponse(id, "face_group is required");
        DzNode* targetNode = targetNodeName.isEmpty() ? nullptr : dzScene->findNode(targetNodeName);
        if (!targetNodeName.isEmpty() && !targetNode) return ErrorResponse(id, QString("Target node not found: %1").arg(targetNodeName));
        // If no target node specified, use a null node (self-reference)
        BeginUndoBatch();
        DzError err = selMap->addPair(faceGroup, targetNode, true);
        AcceptUndoBatch(QString("Add selection map pair: %1 -> %2").arg(faceGroup, targetNodeName));
        if (err == DZ_NO_ERROR) {
            return OkResponse(id, std::string("{\"added\":true,\"face_group\":\"") + JsonEscape(faceGroup) +
                              "\",\"target_node\":\"" + JsonEscape(targetNodeName) + "\"}");
        }
        return ErrorResponse(id, "Failed to add selection map pair");
    }

    if (command == "selection_map_remove_pair") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
        DzSelectionMap* selMap = node->getSelectionMap();
        if (!selMap) return ErrorResponse(id, "No selection map on node");
        int pairIndex = ExtractArgString(line, "pair_index").toInt();
        if (pairIndex < 0 || pairIndex >= selMap->getNumPairs()) return ErrorResponse(id, "Invalid pair index");
        BeginUndoBatch();
        DzError err = selMap->removePair(pairIndex);
        AcceptUndoBatch(QString("Remove selection map pair at index %1").arg(pairIndex));
        if (err == DZ_NO_ERROR) {
            return OkResponse(id, std::string("{\"removed\":true,\"index\":") + std::to_string(pairIndex) + "}");
        }
        return ErrorResponse(id, "Failed to remove selection map pair");
    }

    if (command == "selection_map_clear") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
        DzSelectionMap* selMap = node->getSelectionMap();
        if (!selMap) return ErrorResponse(id, "No selection map on node");
        BeginUndoBatch();
        selMap->clearAll();
        AcceptUndoBatch("Clear selection map");
        return OkResponse(id, "{\"cleared\":true}");
    }

    // --- Node Selectability & Render Visibility -----------------------------

    if (command == "set_node_selectable") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
        QString selectableStr = ExtractArgString(line, "selectable").toLower();
        bool selectable = (selectableStr == "true" || selectableStr == "1");
        DzBoolProperty* prop = node->getSelectabilityControl();
        if (!prop) return ErrorResponse(id, "Node does not support selectability control");
        BeginUndoBatch();
        prop->setBoolValue(selectable);
        AcceptUndoBatch(QString("Set node selectable: %1").arg(node->getName()));
        return OkResponse(id, std::string("{\"node\":\"") + JsonEscape(node->getName()) +
                          "\",\"selectable\":" + (selectable ? "true" : "false") + "}");
    }

    if (command == "set_render_visible") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
        QString visibleStr = ExtractArgString(line, "visible").toLower();
        bool visible = (visibleStr == "true" || visibleStr == "1");
        BeginUndoBatch();
        node->setVisibleInRender(visible);
        AcceptUndoBatch(QString("Set render visible: %1").arg(node->getName()));
        return OkResponse(id, std::string("{\"node\":\"") + JsonEscape(node->getName()) +
                          "\",\"visible_in_render\":" + (visible ? "true" : "false") + "}");
    }

    // --- Parent / Unparent Commands -----------------------------------------

    if (command == "parent_node") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        QString parentId = ExtractArgString(line, "parent_id");
        if (nodeId.isEmpty() || parentId.isEmpty()) return ErrorResponse(id, "node_id and parent_id are required");
        DzNode* node = dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
        DzNode* parent = dzScene->findNode(parentId);
        if (!parent) return ErrorResponse(id, QString("Parent not found: %1").arg(parentId));
        if (node == parent) return ErrorResponse(id, "Cannot parent a node to itself");
        BeginUndoBatch();
        parent->addNodeChild(node);
        AcceptUndoBatch(QString("Parent %1 under %2").arg(nodeId, parentId));
        return OkResponse(id, std::string("{\"node\":\"") + JsonEscape(nodeId) +
                          "\",\"parent\":\"" + JsonEscape(parentId) + "\"}");
    }

    if (command == "unparent_node") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
        DzNode* parent = node->getNodeParent();
        if (!parent) return ErrorResponse(id, "Node has no parent");
        BeginUndoBatch();
        parent->removeNodeChild(node);
        AcceptUndoBatch(QString("Unparent %1").arg(nodeId));
        return OkResponse(id, std::string("{\"node\":\"") + JsonEscape(nodeId) +
                          "\",\"old_parent\":\"" + JsonEscape(parent->getName()) + "\"}");
    }

    // --- Mesh Inspection Commands -------------------------------------------

    if (command == "mesh_get_vertex_count") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
        DzObject* obj = node->getObject();
        if (!obj) return ErrorResponse(id, "Node has no geometry object");
        DzShape* shape = obj->getCurrentShape();
        if (!shape) return ErrorResponse(id, "Node has no shape");
        DzVertexMesh* mesh = shape->getModifiableGeom(false);
        if (!mesh) return ErrorResponse(id, "Node has no modifiable geometry");
        int vertexCount = mesh->getNumVertices();
        return OkResponse(id, std::string("{\"node\":\"") + JsonEscape(node->getName()) +
                          "\",\"vertex_count\":" + std::to_string(vertexCount) + "}");
    }

    if (command == "mesh_get_face_count") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
        DzObject* obj = node->getObject();
        if (!obj) return ErrorResponse(id, "Node has no geometry object");
        DzShape* shape = obj->getCurrentShape();
        if (!shape) return ErrorResponse(id, "Node has no shape");
        DzVertexMesh* mesh = shape->getModifiableGeom(false);
        DzFacetMesh* facetMesh = qobject_cast<DzFacetMesh*>(mesh);
        if (!facetMesh) return ErrorResponse(id, "Node mesh is not a facet mesh");
        int faceCount = facetMesh->getNumFacets();
        return OkResponse(id, std::string("{\"node\":\"") + JsonEscape(node->getName()) +
                          "\",\"face_count\":" + std::to_string(faceCount) + "}");
    }

    // --- Shape Materials Command --------------------------------------------

    if (command == "get_shape_materials") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
        DzObject* obj = node->getObject();
        if (!obj) return ErrorResponse(id, "Node has no geometry object");
        DzShape* shape = obj->getCurrentShape();
        if (!shape) return ErrorResponse(id, "Node has no shape");
        int numMaterials = shape->getNumMaterials();
        std::ostringstream oss;
        oss << "{\"node\":\"" << JsonEscape(node->getName()) << "\",";
        oss << "\"materials\":[";
        for (int i = 0; i < numMaterials; ++i) {
            DzMaterial* mat = shape->getMaterial(i);
            if (!mat) continue;
            if (i > 0) oss << ",";
            oss << "{\"index\":" << i;
            oss << ",\"name\":\"" << JsonEscape(mat->getName()) << "\"";
            oss << ",\"label\":\"" << JsonEscape(mat->getLabel()) << "\"}";
        }
        oss << "],\"count\":" << numMaterials << "}";
        return OkResponse(id, oss.str());
    }

    // --- Property Lock Commands ---------------------------------------------

    if (command == "lock_property") {
        if (!dzScene) return ErrorResponse(id, "No scene");
        QString nodeId = ExtractArgString(line, "node_id");
        DzNode* node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
        if (!node) return ErrorResponse(id, QString("Node not found: %1").arg(nodeId));
        QString propName = ExtractArgString(line, "property");
        if (propName.isEmpty()) return ErrorResponse(id, "property name is required");
        DzProperty* prop = node->findProperty(propName);
        if (!prop) return ErrorResponse(id, QString("Property not found: %1").arg(propName));
        QString lockedStr = ExtractArgString(line, "locked").toLower();
        bool locked = (lockedStr != "false" && lockedStr != "0");
        BeginUndoBatch();
        prop->lock(locked);
        AcceptUndoBatch(QString("Lock property: %1/%2").arg(nodeId, propName));
        return OkResponse(id, std::string("{\"node\":\"") + JsonEscape(node->getName()) +
                          "\",\"property\":\"" + JsonEscape(propName) +
                          "\",\"locked\":" + (locked ? "true" : "false") + "}");
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
const char* GetPluginVersion() { return "0.5.5"; }
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
DZ_PLUGIN_CLASS_GUID(DazPilotPaneAction, 7A9B8C42-2E7C-4E6A-91B6-5D6F9F7A0C12);
DZ_PLUGIN_CLASS_GUID(DazPilotPhyModifier, F9EC5E01-A301-48CD-AF81-C3BB80EC4AA6);
DZ_PLUGIN_CLASS_GUID(DazPilotPhyModifierIO, 1C884DA8-6C3C-4364-81B4-272501D5DDD8);
DZ_PLUGIN_REGISTER_MODIFIER_EXTRA_OBJECT_IO("dazpilot_phy", DazPilotPhyModifierIO, DazPilotPhyModifier);
