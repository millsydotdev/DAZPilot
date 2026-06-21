#define STB_IMAGE_IMPLEMENTATION
#include "stb_image.h"
#include "doctest.h"
#include <cstring>

TEST_CASE("stb_image failure mode is graceful") {
    int x, y, n;
    unsigned char* data = stbi_load("nonexistent_file.png", &x, &y, &n, 0);
    CHECK(data == nullptr);
    CHECK(stbi_failure_reason() != nullptr);
}
