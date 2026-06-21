#pragma once

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
#include "dzrendermgr.h"
#include "dzrenderoptions.h"
#include "dzscene.h"
#include "dzskeleton.h"
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
#include "dzdefaultmaterial.h"
#include "dzimageproperty.h"
#include "dzbone.h"
#include "DAZStudioMCP.h"

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

struct MCPState {
    QTcpServer* server;
    int port;
};
