#pragma once

#ifdef _WIN32
#include <winsock2.h>
#include <ws2tcpip.h>
#endif

#include <QtCore/QObject>
#include <QtCore/QString>
#include <QtCore/QStringList>
#include <QtCore/QList>
#include <QtCore/QByteArray>
#include <QtGui/QImage>
#include <QtNetwork/QTcpServer>
#include <QtNetwork/QTcpSocket>

#include "dzapp.h"
#include "dzcontentmgr.h"
#include "dzmainwindow.h"
#include "dzviewportmgr.h"
#include "dzviewport.h"
#include "dz3dviewport.h"
#include "dzscene.h"
#include "dznode.h"
#include "dzcamera.h"
#include "dzlight.h"
#include "dzfigure.h"
#include "dzpointlight.h"
#include "dzspotlight.h"
#include "dzdistantlight.h"
#include "dzbasiccamera.h"
#include "dzproperty.h"
#include "dzfloatproperty.h"
#include "dzboolproperty.h"
#include "dzcolorproperty.h"
#include "dzstringproperty.h"
#include "dzobject.h"
#include "dzshape.h"
#include "dzmaterial.h"
#if defined(_WIN32)
#  ifdef DAZPILOTBRIDGE_EXPORTS
#    define DAZPILOTBRIDGE_API __declspec(dllexport)
#  else
#    define DAZPILOTBRIDGE_API __declspec(dllimport)
#  endif
#else
#  define DAZPILOTBRIDGE_API __attribute__((visibility("default")))
#endif

struct SceneInfo {
    QString name;
    QString figure;
    int nodeCount;
    int lightCount;
    int cameraCount;
    QString selectedNode;
};

struct SceneNode {
    QString name;
    QString type;
    QString id;
    bool selected;
};

struct DazPilotBridgeState {
    QTcpServer* server;
    QList<QTcpSocket*> clients;
    QString host;
    int port;
    QString lastResponse;
};

extern "C" {
    DAZPILOTBRIDGE_API const char* GetPluginName();
    DAZPILOTBRIDGE_API const char* GetPluginDescription();
    DAZPILOTBRIDGE_API const char* GetPluginVersion();
    DAZPILOTBRIDGE_API int GetPluginType();
    DAZPILOTBRIDGE_API bool PluginInitialize();
    DAZPILOTBRIDGE_API void PluginCleanup();
    DAZPILOTBRIDGE_API const char* GetMenuName();
    DAZPILOTBRIDGE_API void ExecuteMenuAction(const char* action);
    
    DAZPILOTBRIDGE_API bool ConnectToDazPilot(const char* host, int port);
    DAZPILOTBRIDGE_API void DisconnectFromDazPilot();
    DAZPILOTBRIDGE_API bool IsConnectedToDazPilot();
    
    DAZPILOTBRIDGE_API const char* GetSceneInfo();
    DAZPILOTBRIDGE_API const char* GetNodeList();
    DAZPILOTBRIDGE_API const char* GetSelectedNodes();
    DAZPILOTBRIDGE_API bool SelectNode(const char* nodeId);
    DAZPILOTBRIDGE_API bool LoadAsset(const char* assetPath);
    DAZPILOTBRIDGE_API bool ApplyPose(const char* poseFile, const char* figureId);
    DAZPILOTBRIDGE_API bool RenderPreview();
    DAZPILOTBRIDGE_API const char* GetCameras();
    DAZPILOTBRIDGE_API const char* ExecuteCommand(const char* command, const char* args);
    DAZPILOTBRIDGE_API const char* CaptureViewport();
}
