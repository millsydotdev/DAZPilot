#pragma once
#include <nlohmann/json.hpp>
#include <string>
#include <optional>
#include <fstream>

using json = nlohmann::json;

namespace dazstudio_mcp {

namespace json_util {

inline std::optional<json> readJsonFile(const std::string& path) {
    std::ifstream file(path);
    if (!file.is_open()) {
        return std::nullopt;
    }
    try {
        json j;
        file >> j;
        return j;
    } catch (const json::parse_error&) {
        return std::nullopt;
    }
}

inline std::string prettyPrint(const json& j, int indent = 4) {
    return j.dump(indent);
}

}

}
