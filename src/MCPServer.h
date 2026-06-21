#pragma once
#include <QtCore/QObject>
#include <QtCore/QThread>
#include <QtCore/QByteArray>
#include <QtCore/QString>
#include <QtNetwork/QTcpServer>
#include <QtNetwork/QTcpSocket>

class MCPServer : public QThread
{
    Q_OBJECT
public:
    explicit MCPServer(QObject *parent = nullptr);
    ~MCPServer();

protected:
    void run() override;

private slots:
    void onNewConnection();
    void onReadyRead();
    void onDisconnected();

private:
    void sendJsonRpc(int id, const QString &resultJson);
    void sendError(int id, int code, const QString &message);
    void sendRaw(const QByteArray &data);

    // MCP tool handlers
    QString handleInitialize(const QString &params);
    QString handleToolsList();
    QString handleToolsCall(const QString &name, const QString &args);
    QString handleResourcesList();
    QString handleResourcesRead(const QString &uri);

    // Daz SDK helpers
    QString sceneInfo();
    QString listNodes();
    QString getNodeProperties(const QString &nodeId);
    QString getSceneAssets();
    QString getCameras();
    QString selectNode(const QString &nodeId);
    QString setProperty(const QString &nodeId, const QString &prop, const QString &value);
    QString setMorph(const QString &nodeId, const QString &morph, double value);

    // MCP framing via Content-Length
    int parseFrame();
    QByteArray m_recvBuf;

    QTcpServer *m_server;
    QTcpSocket *m_socket;
    QByteArray m_writeBuf;
};
