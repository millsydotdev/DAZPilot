#include "doctest.h"
#include "ExportOptions.h"
#include <nlohmann/json.hpp>

using json = nlohmann::json;

TEST_CASE("ExportOptions::fromJson returns defaults for empty JSON") {
    auto opts = ExportOptions::fromJson("{}");
    CHECK(opts.selectedOnly == false);
    CHECK(opts.includeMaterials == true);
    CHECK(opts.includeAnimations == true);
    CHECK(opts.bakeTextures == false);
    CHECK(opts.targetTextureWidth == 4096);
    CHECK(opts.targetTextureHeight == 4096);
    CHECK(opts.assetType == "SkeletalMesh");
}

TEST_CASE("ExportOptions::fromJson parses all boolean fields") {
    std::string jsonStr = R"({
        "selected_only": true,
        "include_materials": false,
        "include_animations": false,
        "bake_textures": true,
        "generate_normal_maps": true,
        "export_all_textures": true,
        "combine_diffuse_and_alpha_maps": true,
        "resize_textures": true,
        "bake_makeup_overlay": true,
        "bake_translucency": true,
        "bake_specular_to_metallic": true,
        "bake_refraction_weight": true
    })";
    auto opts = ExportOptions::fromJson(QString::fromStdString(jsonStr));
    CHECK(opts.selectedOnly == true);
    CHECK(opts.includeMaterials == false);
    CHECK(opts.includeAnimations == false);
    CHECK(opts.bakeTextures == true);
    CHECK(opts.generateNormalMaps == true);
    CHECK(opts.exportAllTextures == true);
    CHECK(opts.combineDiffuseAndAlphaMaps == true);
    CHECK(opts.resizeTextures == true);
    CHECK(opts.bakeMakeupOverlay == true);
    CHECK(opts.bakeTranslucency == true);
    CHECK(opts.bakeSpecularToMetallic == true);
    CHECK(opts.bakeRefractionWeight == true);
}

TEST_CASE("ExportOptions::fromJson parses numeric fields") {
    std::string jsonStr = R"({
        "target_texture_width": 2048,
        "target_texture_height": 1024
    })";
    auto opts = ExportOptions::fromJson(QString::fromStdString(jsonStr));
    CHECK(opts.targetTextureWidth == 2048);
    CHECK(opts.targetTextureHeight == 1024);
}

TEST_CASE("ExportOptions::fromJson parses string fields") {
    std::string jsonStr = R"({
        "asset_type": "StaticMesh",
        "export_filename": "my_export"
    })";
    auto opts = ExportOptions::fromJson(QString::fromStdString(jsonStr));
    CHECK(opts.assetType == "StaticMesh");
    CHECK(opts.exportFilename == "my_export");
}

TEST_CASE("ExportOptions::fromJson handles invalid JSON gracefully") {
    auto opts = ExportOptions::fromJson("not valid json{{{");
    CHECK(opts.selectedOnly == false);
    CHECK(opts.includeMaterials == true);
    CHECK(opts.assetType == "SkeletalMesh");
}

TEST_CASE("ExportOptions::fromJson handles partial JSON") {
    std::string jsonStr = R"({"selected_only": true})";
    auto opts = ExportOptions::fromJson(QString::fromStdString(jsonStr));
    CHECK(opts.selectedOnly == true);
    CHECK(opts.includeMaterials == true); // default
    CHECK(opts.targetTextureWidth == 4096); // default
}

TEST_CASE("ExportOptions::fromJson handles null values") {
    std::string jsonStr = R"({"selected_only": null, "asset_type": null})";
    auto opts = ExportOptions::fromJson(QString::fromStdString(jsonStr));
    CHECK(opts.selectedOnly == false); // null -> default
    CHECK(opts.assetType == "SkeletalMesh");
}
