#pragma once
#include <spdlog/spdlog.h>
#include <spdlog/sinks/stdout_color_sinks.h>
#include <memory>

namespace dazpilot {

class Log {
public:
    static void init();
    static std::shared_ptr<spdlog::logger>& getLogger() { return s_logger; }

private:
    static std::shared_ptr<spdlog::logger> s_logger;
};

}

#define LOG_TRACE(...)    ::dazpilot::Log::getLogger()->trace(__VA_ARGS__)
#define LOG_DEBUG(...)    ::dazpilot::Log::getLogger()->debug(__VA_ARGS__)
#define LOG_INFO(...)     ::dazpilot::Log::getLogger()->info(__VA_ARGS__)
#define LOG_WARN(...)     ::dazpilot::Log::getLogger()->warn(__VA_ARGS__)
#define LOG_ERROR(...)    ::dazpilot::Log::getLogger()->error(__VA_ARGS__)
#define LOG_CRITICAL(...) ::dazpilot::Log::getLogger()->critical(__VA_ARGS__)
