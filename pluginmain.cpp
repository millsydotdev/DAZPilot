// Minimal Daz Studio plug-in that starts the MCP server
#include <QtCore/QObject>
#include "dzplugin.h"
#include "MCPServer.h"
#include <QtCore/QFile>
#include <QtCore/QTextStream>
#include <QtCore/QDir>

class MCPBridgePlugin : public DzPlugin
{
    Q_OBJECT
public:
    MCPBridgePlugin()
        : DzPlugin("MCP Bridge Plugin", "OpenAI / Custom Build") {}

    // -----------------------------------------------------------------
    // Called by Daz Studio when the plug‑in is loaded
    // -----------------------------------------------------------------
    void startup() override
    {
        // Debug marker – confirms the plug‑in was loaded
        QFile debugFile(QString::fromLocal8Bit(qgetenv("TEMP")) + "/MCPBridge_started.txt");
        if (debugFile.open(QIODevice::WriteOnly | QIODevice::Text)) {
            QTextStream out(&debugFile);
            out << "started\n";
            debugFile.close();
        }
        server = new MCPServer();
        server->start();
    }

    void shutdown() override
    {
        if (server) {
            server->quit();
            server->wait();
            delete server;
            server = nullptr;
        }
    }

private:
    MCPServer *server = nullptr;
};

// -----------------------------------------------------------------
// Export the plug‑in definition for Daz Studio
// -----------------------------------------------------------------
// Note: exports are defined via MCPBridge.def to ensure correct
// C++ mangled names that Daz Studio expects.
__declspec(dllexport) DzVersion getSDKVersion() { return DZ_SDK_VERSION; }
__declspec(dllexport) DzPlugin* getPluginDefinition()
{
    static MCPBridgePlugin s_pluginDef;
    return &s_pluginDef;
}

// Include the MOC file (must be after the export block)
#include "pluginmain.moc"
