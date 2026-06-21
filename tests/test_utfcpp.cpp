#include "doctest.h"
#include "utf8.h"
#include <string>

TEST_CASE("utf8::append encodes codepoints correctly") {
    std::string result;
    utf8::append(0x0041, std::back_inserter(result));
    CHECK(result == "A");

    result.clear();
    utf8::append(0x00E9, std::back_inserter(result));
    CHECK(result == "\xC3\xA9");

    result.clear();
    utf8::append(0x4E16, std::back_inserter(result));
    CHECK(result == "\xE4\xB8\x96");
}

TEST_CASE("utf8::peek_next decodes correctly") {
    std::string input = "A\xC3\xA9\xE4\xB8\x96";
    auto it = input.begin();
    CHECK(utf8::peek_next(it, input.end()) == 0x0041);
    std::advance(it, 1);
    CHECK(utf8::peek_next(it, input.end()) == 0x00E9);
    std::advance(it, 2);
    CHECK(utf8::peek_next(it, input.end()) == 0x4E16);
}

TEST_CASE("utf8::is_valid validates correctly") {
    CHECK(utf8::is_valid("hello"));
    CHECK(utf8::is_valid("\xC3\xA9\xC3\xA0"));
    CHECK_FALSE(utf8::is_valid("\xFF\xFE"));
    CHECK_FALSE(utf8::is_valid("\xC0\xAF"));
}

TEST_CASE("utf8::distance counts codepoints") {
    std::string ascii = "hello";
    CHECK(utf8::distance(ascii.begin(), ascii.end()) == 5);

    std::string mixed = "A\xC3\xA9\xE4\xB8\x96";
    CHECK(utf8::distance(mixed.begin(), mixed.end()) == 3);
}
