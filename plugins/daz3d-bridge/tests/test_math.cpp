#include "doctest.h"
#include <climits>

// Simple function to test
int add(int a, int b) {
    return a + b;
}

TEST_CASE("Testing the add function") {
    CHECK(add(2, 3) == 5);
    CHECK(add(-1, 1) == 0);
    CHECK(add(0, 0) == 0);
}

TEST_CASE("Testing edge cases") {
    CHECK(add(1000, -1000) == 0);
    CHECK(add(INT_MAX, 0) == INT_MAX);
    CHECK(add(INT_MIN, 0) == INT_MIN);
}