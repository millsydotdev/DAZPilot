#include "doctest.h"
#include <spdlog/spdlog.h>
#include <spdlog/sinks/null_sink.h>
#include <memory>
#include <string>

TEST_CASE("spdlog null sink logging does not crash") {
    auto null_sink = std::make_shared<spdlog::sinks::null_sink_mt>();
    auto logger = std::make_shared<spdlog::logger>("log_test", null_sink);
    logger->set_level(spdlog::level::trace);
    logger->trace("trace message {}", 1);
    logger->debug("debug message {}", 2);
    logger->info("info message {}", 3);
    logger->warn("warn message {}", 4);
    logger->error("error message {}", 5);
    logger->critical("critical message {}", 6);
    CHECK(true);
}

TEST_CASE("spdlog level filtering works") {
    auto null_sink = std::make_shared<spdlog::sinks::null_sink_mt>();
    auto logger = std::make_shared<spdlog::logger>("filter_test", null_sink);
    logger->set_level(spdlog::level::warn);
    CHECK(static_cast<int>(logger->level()) == static_cast<int>(spdlog::level::warn));
}
