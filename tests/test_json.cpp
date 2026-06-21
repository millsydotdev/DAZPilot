#include "doctest.h"
#include <nlohmann/json.hpp>
#include <cstdio>
#include <fstream>

using json = nlohmann::json;

TEST_CASE("nlohmann json can parse and query data") {
    json data = json::parse(R"({"name": "test", "value": 42, "items": [1, 2, 3]})");
    CHECK(data["name"] == "test");
    CHECK(data["value"] == 42);
    CHECK(data["items"].size() == 3);
    CHECK(data["items"][0] == 1);
    CHECK(data["items"][2] == 3);
}

TEST_CASE("nlohmann json can serialize") {
    json obj;
    obj["hello"] = "world";
    obj["count"] = 123;
    std::string serialized = obj.dump();
    CHECK(serialized == R"({"count":123,"hello":"world"})");
}

TEST_CASE("nlohmann json handles missing keys") {
    json data = json::parse(R"({"exists": true})");
    CHECK(data.contains("exists"));
    CHECK_FALSE(data.contains("missing"));
    CHECK(data.value("missing", "default") == "default");
}

TEST_CASE("nlohmann json can read from file") {
    std::string testPath = std::tmpnam(nullptr);
    {
        std::ofstream file(testPath);
        file << R"({"port": 8765, "host": "127.0.0.1"})";
    }
    std::ifstream file(testPath);
    json config;
    file >> config;
    CHECK(config["port"] == 8765);
    CHECK(config["host"] == "127.0.0.1");
    std::remove(testPath.c_str());
}
