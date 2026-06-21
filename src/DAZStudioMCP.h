#pragma once

#if defined(_WIN32)
#  ifdef DAZSTUDIO_MCP_EXPORTS
#    define DAZSTUDIO_MCP_API __declspec(dllexport)
#  else
#    define DAZSTUDIO_MCP_API __declspec(dllimport)
#  endif
#else
#  define DAZSTUDIO_MCP_API __attribute__((visibility("default")))
#endif

extern "C" {
    DAZSTUDIO_MCP_API const char* GetPluginName();
    DAZSTUDIO_MCP_API const char* GetPluginDescription();
    DAZSTUDIO_MCP_API const char* GetPluginVersion();
    DAZSTUDIO_MCP_API int GetPluginType();
    DAZSTUDIO_MCP_API bool PluginInitialize();
    DAZSTUDIO_MCP_API void PluginCleanup();
    DAZSTUDIO_MCP_API const char* GetMenuName();
    DAZSTUDIO_MCP_API void ExecuteMenuAction(const char* action);
}
