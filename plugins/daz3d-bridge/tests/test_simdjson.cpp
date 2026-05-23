#include "doctest.h"
#include "simdjson.h"

TEST_CASE("simdjson can parse simple JSON") {
    simdjson::ondemand::parser parser;
    auto json = "{\"name\": \"test\", \"value\": 42}"_padded;
    simdjson::ondemand::document doc = parser.iterate(json);
    CHECK(std::string_view(doc["name"]) == "test");
    CHECK(int64_t(doc["value"]) == 42);
}

TEST_CASE("simdjson handles arrays") {
    simdjson::ondemand::parser parser;
    auto json = "[10, 20, 30]"_padded;
    simdjson::ondemand::document doc = parser.iterate(json);
    int64_t sum = 0;
    for (auto elem : doc.get_array()) {
        sum += int64_t(elem);
    }
    CHECK(sum == 60);
}

TEST_CASE("simdjson handles nested objects") {
    simdjson::ondemand::parser parser;
    auto json = R"({"outer": {"inner": "value", "count": 42}})"_padded;
    auto doc = parser.iterate(json);
    std::string_view inner = doc["outer"]["inner"];
    CHECK(inner == "value");
    int64_t count = doc["outer"]["count"];
    CHECK(count == 42);
}
