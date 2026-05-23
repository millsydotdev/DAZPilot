#include "doctest.h"
#include "JsonUtil.h"
#include <cstdio>
#include <fstream>

TEST_CASE("JsonUtil readJsonFile reads valid JSON") {
    std::string path = std::tmpnam(nullptr);
    {
        std::ofstream f(path);
        f << R"({"port": 8765, "host": "127.0.0.1"})";
    }
    auto result = dazpilot::json_util::readJsonFile(path);
    REQUIRE(result.has_value());
    CHECK((*result)["port"] == 8765);
    CHECK((*result)["host"] == "127.0.0.1");
    std::remove(path.c_str());
}

TEST_CASE("JsonUtil readJsonFile returns nullopt for missing file") {
    auto result = dazpilot::json_util::readJsonFile("/nonexistent/path.json");
    CHECK_FALSE(result.has_value());
}

TEST_CASE("JsonUtil readJsonFile returns nullopt for invalid JSON") {
    std::string path = std::tmpnam(nullptr);
    {
        std::ofstream f(path);
        f << "not valid json";
    }
    auto result = dazpilot::json_util::readJsonFile(path);
    CHECK_FALSE(result.has_value());
    std::remove(path.c_str());
}

TEST_CASE("JsonUtil prettyPrint formats correctly") {
    json j = {{"name", "test"}, {"value", 42}};
    std::string printed = dazpilot::json_util::prettyPrint(j);
    CHECK(printed.find("\"name\": \"test\"") != std::string::npos);
    CHECK(printed.find("\"value\": 42") != std::string::npos);
}
