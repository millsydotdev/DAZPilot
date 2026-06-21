#include "MCPServer.moc"
#include "MCPServer.h"
#include <QtCore/QDebug>

#include "dzscene.h"
#include "dznode.h"
#include "dzobject.h"
#include "dzproperty.h"
#include "dzfloatproperty.h"
#include "dzboolproperty.h"
#include "dzcolorproperty.h"
#include "dzstringproperty.h"
#include "dzcamera.h"
#include "dzlight.h"
#include "dzfigure.h"

// ── Minimal JSON writer (avoids dependency on Qt4 JSON which may be missing) ─
class JsonWriter {
    QString m_buf;
public:
    JsonWriter() { m_buf.reserve(4096); }
    void beginObject() { m_buf += '{'; }
    void endObject() { m_buf += '}'; }
    void beginArray() { m_buf += '['; }
    void endArray() { m_buf += ']'; }
    void key(const QString &k) { comma(); m_buf += '"' + esc(k) + '"' + ':'; }
    void value(const QString &v) { comma(); m_buf += '"' + esc(v) + '"'; }
    void value(int v) { comma(); m_buf += QString::number(v); }
    void value(double v) { comma(); m_buf += QString::number(v, 'f', 6); }
    void value(bool v) { comma(); m_buf += v ? "true" : "false"; }
    void raw(const QString &r) { comma(); m_buf += r; }
    void null() { comma(); m_buf += "null"; }
    QString str() const { return m_buf; }
private:
    bool m_comma = false;
    void comma() {
        if (m_comma) m_buf += ',';
        m_comma = true;
    }
    static QString esc(const QString &s) {
        QString out;
        out.reserve(s.size() + 4);
        for (int i = 0; i < s.size(); ++i) {
            QChar c = s[i];
            if (c == '"') out += "\\\"";
            else if (c == '\\') out += "\\\\";
            else if (c == '\n') out += "\\n";
            else if (c == '\r') out += "\\r";
            else if (c == '\t') out += "\\t";
            else out += c;
        }
        return out;
    }
};

// ── Helpers ─────────────────────────────────────────────────────────────────
static QString nodeType(DzNode *node) {
    if (qobject_cast<DzFigure*>(node)) return "Figure";
    if (qobject_cast<DzLight*>(node)) return "Light";
    if (qobject_cast<DzCamera*>(node)) return "Camera";
    return "Node";
}

static QString propValueToJson(DzProperty *prop) {
    if (!prop) return "null";
    if (auto *f = qobject_cast<DzFloatProperty*>(prop))
        return QString::number(f->getValue());
    if (auto *b = qobject_cast<DzBoolProperty*>(prop))
        return b->getBoolValue() ? "true" : "false";
    if (auto *s = qobject_cast<DzStringProperty*>(prop))
        return "\"" + JsonWriter().value(s->getValue()).str() + "\"";
    if (auto *c = qobject_cast<DzColorProperty*>(prop)) {
        QColor col = c->getColorValue();
        return QString("\"#%1%2%3\"").arg(col.red(),2,16,QChar('0'))
            .arg(col.green(),2,16,QChar('0')).arg(col.blue(),2,16,QChar('0'));
    }
    return "null";
}

// ── Server implementation ───────────────────────────────────────────────────
MCPServer::MCPServer(QObject *parent)
    : QThread(parent), m_server(nullptr), m_socket(nullptr)
{}

MCPServer::~MCPServer() {
    quit();
    wait();
    if (m_server) { m_server->close(); delete m_server; m_server = nullptr; }
}

void MCPServer::run() {
    m_server = new QTcpServer();
    QObject::connect(m_server, SIGNAL(newConnection()), this, SLOT(onNewConnection()));
    if (!m_server->listen(QHostAddress::LocalHost, 5000)) {
        qWarning("MCP Server: Unable to listen on port 5000");
        return;
    }
    qDebug("MCP Server listening on port 5000");
    exec();
    m_server->close();
    delete m_server;
    m_server = nullptr;
}

void MCPServer::onNewConnection() {
    if (m_socket) {
        QTcpSocket *sock = m_server->nextPendingConnection();
        sock->disconnectFromHost();
        sock->deleteLater();
        return;
    }
    m_socket = m_server->nextPendingConnection();
    QObject::connect(m_socket, SIGNAL(readyRead()), this, SLOT(onReadyRead()));
    QObject::connect(m_socket, SIGNAL(disconnected()), this, SLOT(onDisconnected()));
    m_recvBuf.clear();
    qDebug("MCP Client connected");
}

void MCPServer::onDisconnected() {
    if (m_socket) { m_socket->deleteLater(); m_socket = nullptr; }
    m_recvBuf.clear();
    qDebug("MCP Client disconnected");
}

// ── MCP JSON‑RPC 2.0 handler ────────────────────────────────────────────────
void MCPServer::onReadyRead() {
    if (!m_socket) return;
    m_recvBuf.append(m_socket->readAll());
    while (true) {
        int frameLen = parseFrame();
        if (frameLen < 0) break;
        QByteArray body = m_recvBuf.left(frameLen);
        m_recvBuf.remove(0, frameLen);

        // Simple JSON key‑value extraction (enough for MCP header parsing)
        QString text = QString::fromUtf8(body);
        QString method = extractJsonStr(text, "method");
        int id = extractJsonInt(text, "id");
        QString paramsObj = extractJsonObj(text, "params");

        if (method == "initialize") {
            sendJsonRpc(id, handleInitialize());
        } else if (method == "initialized") {
            // notification – no response
        } else if (method == "tools/list") {
            sendJsonRpc(id, handleToolsList());
        } else if (method == "tools/call") {
            QString name = extractJsonStr(paramsObj, "name");
            QString args = extractJsonObj(paramsObj, "arguments");
            sendJsonRpc(id, handleToolsCall(name, args));
        } else if (method == "resources/list") {
            sendJsonRpc(id, handleResourcesList());
        } else if (method == "resources/read") {
            QString uri = extractJsonStr(paramsObj, "uri");
            sendJsonRpc(id, handleResourcesRead(uri));
        } else {
            sendError(id, -32601, "Method not found: " + method);
        }
    }
}

// ── Simple JSON extraction (no full parser needed for MCP framing) ──────────
static int findKey(const QString &text, const QString &key, int start = 0) {
    QString needle = "\"" + key + "\"";
    int pos = text.indexOf(needle, start);
    if (pos < 0) return -1;
    int colon = text.indexOf(':', pos + needle.size());
    if (colon < 0) return -1;
    return colon + 1;
}

static QString extractJsonStr(const QString &text, const QString &key) {
    int pos = findKey(text, key);
    if (pos < 0) return "";
    // Skip whitespace
    while (pos < text.size() && (text[pos] == ' ' || text[pos] == '\t' || text[pos] == '\r' || text[pos] == '\n'))
        ++pos;
    if (pos >= text.size() || text[pos] != '"') return "";
    ++pos;
    QString out;
    bool esc = false;
    for (; pos < text.size(); ++pos) {
        QChar c = text[pos];
        if (esc) { out += c; esc = false; continue; }
        if (c == '\\') { esc = true; continue; }
        if (c == '"') break;
        out += c;
    }
    return out;
}

static int extractJsonInt(const QString &text, const QString &key) {
    int pos = findKey(text, key);
    if (pos < 0) return -1;
    while (pos < text.size() && (text[pos] == ' ' || text[pos] == '\t'))
        ++pos;
    int sign = 1;
    if (pos < text.size() && text[pos] == '-') { sign = -1; ++pos; }
    int val = 0;
    while (pos < text.size() && text[pos] >= '0' && text[pos] <= '9') {
        val = val * 10 + (text[pos].unicode() - '0');
        ++pos;
    }
    return val * sign;
}

static QString extractJsonObj(const QString &text, const QString &key) {
    int pos = findKey(text, key);
    if (pos < 0) return "{}";
    while (pos < text.size() && (text[pos] == ' ' || text[pos] == '\t' || text[pos] == '\r' || text[pos] == '\n'))
        ++pos;
    if (pos >= text.size()) return "{}";
    QChar startCh = text[pos];
    QChar endCh = (startCh == '{') ? '}' : (startCh == '[') ? ']' : QChar();
    if (endCh.isNull()) return "{}";
    int depth = 0;
    bool inStr = false;
    bool esc = false;
    int startPos = pos;
    for (; pos < text.size(); ++pos) {
        QChar c = text[pos];
        if (esc) { esc = false; continue; }
        if (c == '\\') { esc = true; continue; }
        if (c == '"') { inStr = !inStr; continue; }
        if (inStr) continue;
        if (c == startCh) ++depth;
        if (c == endCh) {
            --depth;
            if (depth == 0) {
                return text.mid(startPos, pos - startPos + 1);
            }
        }
    }
    return "{}";
}

// ── MCP framing (Content-Length: N\r\n\r\n) ─────────────────────────────────
int MCPServer::parseFrame() {
    int idx = m_recvBuf.indexOf("\r\n\r\n");
    if (idx < 0) return -1;
    QByteArray header = m_recvBuf.left(idx);
    int bodyStart = idx + 4;
    int len = -1;
    QList<QByteArray> lines = header.split('\n');
    for (const QByteArray &line : lines) {
        if (line.toLower().startsWith("content-length:")) {
            len = line.mid(15).trimmed().toInt();
            break;
        }
    }
    if (len < 0) return -1;
    if (m_recvBuf.size() < bodyStart + len) return -1;
    return bodyStart + len;
}

// ── Response helpers ────────────────────────────────────────────────────────
void MCPServer::sendJsonRpc(int id, const QString &resultJson) {
    QString body = "{\"jsonrpc\":\"2.0\",\"id\":" + QString::number(id) +
                   ",\"result\":" + resultJson + "}";
    sendRaw(body.toUtf8());
}

void MCPServer::sendError(int id, int code, const QString &message) {
    if (id < 0) return;
    QString body = "{\"jsonrpc\":\"2.0\",\"id\":" + QString::number(id) +
                   ",\"error\":{\"code\":" + QString::number(code) +
                   ",\"message\":\"" + message + "\"}}";
    sendRaw(body.toUtf8());
}

void MCPServer::sendRaw(const QByteArray &data) {
    if (!m_socket) return;
    QByteArray frame = "Content-Length: " + QByteArray::number(data.size()) + "\r\n\r\n" + data;
    m_socket->write(frame);
    m_socket->flush();
}

// ── MCP handlers ────────────────────────────────────────────────────────────
QString MCPServer::handleInitialize() {
    // Use raw json string merge to avoid JSON writer overhead
    return "{\"protocolVersion\":\"2024-11-05\","
           "\"capabilities\":{\"tools\":{},\"resources\":{}},"
           "\"serverInfo\":{\"name\":\"DAZPilot-MCP\",\"version\":\"1.0.0\"}}";
}

QString MCPServer::handleToolsList() {
    JsonWriter w;
    w.beginObject();
    w.key("tools");
    w.beginArray();

    // scene_info
    w.beginObject();
    w.key("name"); w.value("scene_info");
    w.key("description"); w.value("Get scene summary");
    w.key("inputSchema");
    w.beginObject();
    w.key("type"); w.value("object");
    w.key("properties"); w.beginObject(); w.endObject();
    w.endObject();
    w.endObject();

    // list_nodes
    w.beginObject();
    w.key("name"); w.value("list_nodes");
    w.key("description"); w.value("List all scene nodes");
    w.key("inputSchema");
    w.beginObject();
    w.key("type"); w.value("object");
    w.key("properties"); w.beginObject(); w.endObject();
    w.endObject();
    w.endObject();

    // get_node_properties
    w.beginObject();
    w.key("name"); w.value("get_node_properties");
    w.key("description"); w.value("Get properties of a node");
    w.key("inputSchema");
    w.beginObject();
    w.key("type"); w.value("object");
    w.key("properties");
    w.beginObject();
    w.key("node_id"); w.beginObject(); w.key("type"); w.value("string"); w.key("description"); w.value("Node name"); w.endObject();
    w.endObject();
    w.endObject();
    w.endObject();

    // select_node
    w.beginObject();
    w.key("name"); w.value("select_node");
    w.key("description"); w.value("Select a node by name");
    w.key("inputSchema");
    w.beginObject();
    w.key("type"); w.value("object");
    w.key("properties");
    w.beginObject();
    w.key("node_id"); w.beginObject(); w.key("type"); w.value("string"); w.endObject();
    w.endObject();
    w.endObject();
    w.endObject();

    // set_property
    w.beginObject();
    w.key("name"); w.value("set_property");
    w.key("description"); w.value("Set a property value on a node");
    w.key("inputSchema");
    w.beginObject();
    w.key("type"); w.value("object");
    w.key("properties");
    w.beginObject();
    w.key("node_id"); w.beginObject(); w.key("type"); w.value("string"); w.endObject();
    w.key("property"); w.beginObject(); w.key("type"); w.value("string"); w.endObject();
    w.key("value"); w.beginObject(); w.key("type"); w.value("string"); w.endObject();
    w.endObject();
    w.endObject();
    w.endObject();

    // set_morph
    w.beginObject();
    w.key("name"); w.value("set_morph");
    w.key("description"); w.value("Set a morph value on a figure");
    w.key("inputSchema");
    w.beginObject();
    w.key("type"); w.value("object");
    w.key("properties");
    w.beginObject();
    w.key("node_id"); w.beginObject(); w.key("type"); w.value("string"); w.endObject();
    w.key("morph"); w.beginObject(); w.key("type"); w.value("string"); w.endObject();
    w.key("value"); w.beginObject(); w.key("type"); w.value("number"); w.endObject();
    w.endObject();
    w.endObject();
    w.endObject();

    // get_cameras
    w.beginObject();
    w.key("name"); w.value("get_cameras");
    w.key("description"); w.value("List all cameras in the scene");
    w.key("inputSchema");
    w.beginObject();
    w.key("type"); w.value("object");
    w.key("properties"); w.beginObject(); w.endObject();
    w.endObject();
    w.endObject();

    // get_scene_assets
    w.beginObject();
    w.key("name"); w.value("get_scene_assets");
    w.key("description"); w.value("List all assets in the scene");
    w.key("inputSchema");
    w.beginObject();
    w.key("type"); w.value("object");
    w.key("properties"); w.beginObject(); w.endObject();
    w.endObject();
    w.endObject();

    w.endArray();
    w.endObject();
    return w.str();
}

QString MCPServer::handleToolsCall(const QString &name, const QString &args) {
    QString result;
    if (name == "scene_info") {
        result = sceneInfo();
    } else if (name == "list_nodes") {
        result = listNodes();
    } else if (name == "get_node_properties") {
        QString nodeId = extractJsonStr(args, "node_id");
        result = getNodeProperties(nodeId);
    } else if (name == "select_node") {
        QString nodeId = extractJsonStr(args, "node_id");
        bool ok = selectNode(nodeId) == "true";
        result = ok ? "{\"selected\":true}" : "{\"selected\":false}";
    } else if (name == "set_property") {
        QString nodeId = extractJsonStr(args, "node_id");
        QString prop = extractJsonStr(args, "property");
        QString value = extractJsonStr(args, "value");
        bool ok = setProperty(nodeId, prop, value) == "true";
        result = ok ? "{\"set\":true}" : "{\"set\":false}";
    } else if (name == "set_morph") {
        QString nodeId = extractJsonStr(args, "node_id");
        QString morph = extractJsonStr(args, "morph");
        QString valueStr = extractJsonStr(args, "value");
        bool ok = setMorph(nodeId, morph, valueStr.toDouble()) == "true";
        result = ok ? "{\"set\":true}" : "{\"set\":false}";
    } else if (name == "get_cameras") {
        result = getCameras();
    } else if (name == "get_scene_assets") {
        result = getSceneAssets();
    } else {
        return "{\"content\":[{\"type\":\"text\",\"text\":\"{\\\"error\\\":\\\"Unknown tool\\\"}\"}]}";
    }
    return "{\"content\":[{\"type\":\"text\",\"text\":" + QString(result) + "}]}";
}

QString MCPServer::handleResourcesList() {
    return "{\"resources\":[]}";
}

QString MCPServer::handleResourcesRead(const QString &) {
    return "{\"contents\":[]}";
}

// ── Daz SDK tool implementations ────────────────────────────────────────────
QString MCPServer::sceneInfo() {
    if (!dzScene) return "{\"available\":false}";
    QString filename = dzScene->getFilename();
    if (filename.isEmpty()) filename = "Untitled";
    return "{\"scene\":\"" + filename + "\",\"nodes\":" +
           QString::number(dzScene->getNumNodes()) + ",\"lights\":" +
           QString::number(dzScene->getNumLights()) + ",\"cameras\":" +
           QString::number(dzScene->getNumCameras()) + "}";
}

QString MCPServer::listNodes() {
    JsonWriter w;
    w.beginObject();
    w.key("nodes");
    w.beginArray();
    if (dzScene) {
        int n = dzScene->getNumNodes();
        for (int i = 0; i < n; ++i) {
            DzNode *node = dzScene->getNode(i);
            if (!node) continue;
            w.beginObject();
            w.key("name"); w.value(node->getName());
            w.key("type"); w.value(nodeType(node));
            w.key("selected"); w.value(node->isSelected());
            w.endObject();
        }
    }
    w.endArray();
    w.endObject();
    return w.str();
}

QString MCPServer::getNodeProperties(const QString &nodeId) {
    if (!dzScene) return "{\"properties\":[]}";
    DzNode *node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
    if (!node) return "{\"properties\":[]}";
    JsonWriter w;
    w.beginObject();
    w.key("node"); w.value(node->getName());
    w.key("properties");
    w.beginArray();
    int n = node->getNumProperties();
    for (int i = 0; i < n; ++i) {
        DzProperty *prop = node->getProperty(i);
        if (!prop) continue;
        w.beginObject();
        w.key("name"); w.value(prop->getName());
        w.key("label"); w.value(prop->getLabel());
        w.key("path"); w.value(prop->getPath());
        w.key("value"); w.raw(propValueToJson(prop));
        w.endObject();
    }
    w.endArray();
    w.endObject();
    return w.str();
}

QString MCPServer::getSceneAssets() {
    JsonWriter w;
    w.beginObject();
    w.key("assets");
    w.beginArray();
    if (dzScene) {
        int n = dzScene->getNumNodes();
        for (int i = 0; i < n; ++i) {
            DzNode *node = dzScene->getNode(i);
            if (!node) continue;
            QString label = node->getLabel();
            if (label.isEmpty()) label = node->getName();
            if (label.isEmpty()) continue;
            w.value(label);
        }
    }
    w.endArray();
    w.endObject();
    return w.str();
}

QString MCPServer::getCameras() {
    JsonWriter w;
    w.beginObject();
    w.key("cameras");
    w.beginArray();
    if (dzScene) {
        int n = dzScene->getNumCameras();
        for (int i = 0; i < n; ++i) {
            DzCamera *cam = dzScene->getCamera(i);
            if (!cam) continue;
            w.beginObject();
            w.key("name"); w.value(cam->getName());
            w.endObject();
        }
    }
    w.endArray();
    w.endObject();
    return w.str();
}

QString MCPServer::selectNode(const QString &nodeId) {
    if (!dzScene || nodeId.isEmpty()) return "false";
    DzNode *node = dzScene->findNode(nodeId);
    if (!node) return "false";
    dzScene->setPrimarySelection(node);
    return "true";
}

QString MCPServer::setProperty(const QString &nodeId, const QString &prop, const QString &value) {
    if (!dzScene) return "false";
    DzNode *node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
    if (!node) return "false";
    DzProperty *p = node->findProperty(prop);
    if (!p) return "false";
    if (auto *f = qobject_cast<DzFloatProperty*>(p)) { f->setValue(value.toDouble()); return "true"; }
    if (auto *b = qobject_cast<DzBoolProperty*>(p)) { b->setBoolValue(value.toLower() == "true" || value == "1"); return "true"; }
    if (auto *s = qobject_cast<DzStringProperty*>(p)) { s->setValue(value); return "true"; }
    return "false";
}

QString MCPServer::setMorph(const QString &nodeId, const QString &morph, double value) {
    if (!dzScene) return "false";
    DzNode *node = nodeId.isEmpty() ? dzScene->getPrimarySelection() : dzScene->findNode(nodeId);
    if (!node) return "false";
    DzProperty *p = node->findProperty(morph, false);
    if (!p) return "false";
    if (auto *f = qobject_cast<DzFloatProperty*>(p)) { f->setValue(value); return "true"; }
    return "false";
}
