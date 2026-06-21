#include "Log.h"

namespace dazstudio_mcp {

std::shared_ptr<spdlog::logger> Log::s_logger = nullptr;

void Log::init() {
    spdlog::set_pattern("[%T] [%^%l%$] %v");
    s_logger = spdlog::stdout_color_mt("DAZMCP");
    s_logger->set_level(spdlog::level::trace);
}

}
