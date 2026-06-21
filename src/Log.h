#pragma once
#include <spdlog/spdlog.h>
#include <spdlog/sinks/stdout_color_sinks.h>
#include <memory>

namespace dazstudio_mcp {

class Log {
public:
    static void init();
    static std::shared_ptr<spdlog::logger>& getLogger() { return s_logger; }

private:
    static std::shared_ptr<spdlog::logger> s_logger;
};

}

#define LOG_TRACE(...)    ::dazstudio_mcp::Log::getLogger()->trace(__VA_ARGS__)
#define LOG_DEBUG(...)    ::dazstudio_mcp::Log::getLogger()->debug(__VA_ARGS__)
#define LOG_INFO(...)     ::dazstudio_mcp::Log::getLogger()->info(__VA_ARGS__)
#define LOG_WARN(...)     ::dazstudio_mcp::Log::getLogger()->warn(__VA_ARGS__)
#define LOG_ERROR(...)    ::dazstudio_mcp::Log::getLogger()->error(__VA_ARGS__)
#define LOG_CRITICAL(...) ::dazstudio_mcp::Log::getLogger()->critical(__VA_ARGS__)
