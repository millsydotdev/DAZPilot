#include "doctest.h"
#include "JsonUtil.h"
#include <cstdio>
#include <fstream>

using json = nlohmann::json;

TEST_CASE("JsonUtil::readJsonFile returns nullopt for missing file") {
    auto result = dazstudio_mcp::json_util::readJsonFile("nonexistent_file_12345.json");
    CHECK_FALSE(result.has_value());
}

TEST_CASE("JsonUtil::readJsonFile returns parsed JSON for valid file") {
    std::string testPath = std::tmpnam(nullptr);
    {
        std::ofstream file(testPath);
        file << R"({"key": "value", "num": 42})";
    }
    auto result = dazstudio_mcp::json_util::readJsonFile(testPath);
    REQUIRE(result.has_value());
    CHECK((*result)["key"] == "value");
    CHECK((*result)["num"] == 42);
    std::remove(testPath.c_str());
}

TEST_CASE("JsonUtil::readJsonFile returns nullopt for malformed file") {
    std::string testPath = std::tmpnam(nullptr);
    {
        std::ofstream file(testPath);
        file << "{ broken json }";
    }
    auto result = dazstudio_mcp::json_util::readJsonFile(testPath);
    CHECK_FALSE(result.has_value());
    std::remove(testPath.c_str());
}

TEST_CASE("JsonUtil::prettyPrint formats JSON with indentation") {
    json j;
    j["name"] = "test";
    j["value"] = 42;
    std::string printed = dazstudio_mcp::json_util::prettyPrint(j);
    CHECK(printed.find("  ") != std::string::npos);
    CHECK(printed.find("\"name\"") != std::string::npos);
    CHECK(printed.find("\"value\"") != std::string::npos);
}

TEST_CASE("JsonUtil::prettyPrint with custom indent") {
    json j;
    j["x"] = 1;
    std::string printed = dazstudio_mcp::json_util::prettyPrint(j, 2);
    json parsed = json::parse(printed);
    CHECK(parsed["x"] == 1);
}

TEST_CASE("JsonUtil::config.json round-trip via readJsonFile") {
    std::string testPath = std::tmpnam(nullptr);
    {
        std::ofstream file(testPath);
        file << R"({"host": "127.0.0.1", "port": 8765, "auto_connect": true})";
    }
    auto result = dazstudio_mcp::json_util::readJsonFile(testPath);
    REQUIRE(result.has_value());
    CHECK((*result)["host"] == "127.0.0.1");
    CHECK((*result)["port"] == 8765);
    CHECK((*result)["auto_connect"] == true);
    std::remove(testPath.c_str());
}

TEST_CASE("JsonUtil handles empty file") {
    std::string testPath = std::tmpnam(nullptr);
    {
        std::ofstream file(testPath);
    }
    auto result = dazstudio_mcp::json_util::readJsonFile(testPath);
    CHECK_FALSE(result.has_value());
    std::remove(testPath.c_str());
}

TEST_CASE("JsonUtil handles deeply nested JSON") {
    std::string testPath = std::tmpnam(nullptr);
    {
        std::ofstream file(testPath);
        file << R"({"level1": {"level2": {"level3": {"value": 42}}}})";
    }
    auto result = dazstudio_mcp::json_util::readJsonFile(testPath);
    REQUIRE(result.has_value());
    CHECK((*result)["level1"]["level2"]["level3"]["value"] == 42);
    std::remove(testPath.c_str());
}
