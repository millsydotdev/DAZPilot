#include "doctest.h"
#include <glm/glm.hpp>
#include <glm/gtc/matrix_transform.hpp>
#include <glm/gtc/type_ptr.hpp>

TEST_CASE("glm vector operations") {
    glm::vec3 a(1, 2, 3);
    glm::vec3 b(4, 5, 6);
    glm::vec3 sum = a + b;
    CHECK(sum.x == 5);
    CHECK(sum.y == 7);
    CHECK(sum.z == 9);
}

TEST_CASE("glm dot and cross product") {
    glm::vec3 a(1, 0, 0);
    glm::vec3 b(0, 1, 0);
    float dot = glm::dot(a, b);
    CHECK(dot == 0);
    glm::vec3 cross = glm::cross(a, b);
    CHECK(cross.x == 0);
    CHECK(cross.y == 0);
    CHECK(cross.z == 1);
}

TEST_CASE("glm matrix transformations") {
    glm::vec3 position(1, 2, 3);
    glm::mat4 transform = glm::translate(glm::mat4(1.0f), position);
    glm::vec4 result = transform * glm::vec4(0, 0, 0, 1);
    CHECK(result.x == 1);
    CHECK(result.y == 2);
    CHECK(result.z == 3);
}

TEST_CASE("glm quaternion rotation") {
    glm::vec3 axis(0, 1, 0);
    glm::quat rotation = glm::angleAxis(glm::radians(90.0f), axis);
    glm::vec3 forward(1, 0, 0);
    glm::vec3 rotated = rotation * forward;
    CHECK(std::abs(rotated.x) < 0.001f);
    CHECK(std::abs(rotated.y) < 0.001f);
    CHECK(std::abs(rotated.z + 1.0f) < 0.001f);
}
