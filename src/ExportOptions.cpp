#include "ExportOptions.h"
#ifndef DAZSTUDIO_MCP_TESTS
#include "dzfileiosettings.h"
#endif
#include <nlohmann/json.hpp>

using json = nlohmann::json;

ExportOptions ExportOptions::fromJson(const QString& jsonStr) {
    ExportOptions opts;
    try {
        json j = json::parse(jsonStr.toStdString());

        opts.selectedOnly = j.value("selected_only", false);
        opts.includeMaterials = j.value("include_materials", true);
        opts.includeAnimations = j.value("include_animations", true);
        opts.bakeTextures = j.value("bake_textures", false);

        opts.generateNormalMaps = j.value("generate_normal_maps", false);
        opts.exportAllTextures = j.value("export_all_textures", false);
        opts.combineDiffuseAndAlphaMaps = j.value("combine_diffuse_and_alpha_maps", false);
        opts.resizeTextures = j.value("resize_textures", false);

        opts.targetTextureWidth = j.value("target_texture_width", 4096);
        opts.targetTextureHeight = j.value("target_texture_height", 4096);

        opts.bakeMakeupOverlay = j.value("bake_makeup_overlay", false);
        opts.bakeTranslucency = j.value("bake_translucency", false);
        opts.bakeSpecularToMetallic = j.value("bake_specular_to_metallic", false);
        opts.bakeRefractionWeight = j.value("bake_refraction_weight", false);

        opts.assetType = QString::fromStdString(j.value("asset_type", "SkeletalMesh"));
        opts.exportFilename = QString::fromStdString(j.value("export_filename", ""));
    } catch (...) {
    }
    return opts;
}

#ifndef DAZSTUDIO_MCP_TESTS
void ExportOptions::applyToSettings(DzFileIOSettings* settings) const {
    if (!settings) return;

    settings->setBoolValue("RunSilent", true);
    settings->setFloatValue("Scale", 1.0f);
    settings->setBoolValue("LatAxis", true);

    settings->setBoolValue("SelectedOnly", selectedOnly);
    settings->setBoolValue("WriteMaterial", includeMaterials);
    settings->setBoolValue("WriteTextures", includeMaterials);
    settings->setBoolValue("EmbedTextures", includeMaterials);
    settings->setBoolValue("WriteAnimations", includeAnimations);

    settings->setBoolValue("BakeTextures", bakeTextures);
    settings->setBoolValue("GenerateNormalMaps", generateNormalMaps);
    settings->setBoolValue("ExportAllTextures", exportAllTextures);
    settings->setBoolValue("CombineDiffuseAndAlphaMaps", combineDiffuseAndAlphaMaps);
    settings->setBoolValue("ResizeTextures", resizeTextures);
    settings->setIntValue("TargetTextureWidth", targetTextureWidth);
    settings->setIntValue("TargetTextureHeight", targetTextureHeight);

    settings->setBoolValue("BakeMakeupOverlay", bakeMakeupOverlay);
    settings->setBoolValue("BakeTranslucency", bakeTranslucency);
    settings->setBoolValue("BakeSpecularToMetallic", bakeSpecularToMetallic);
    settings->setBoolValue("BakeRefractionWeight", bakeRefractionWeight);
}
#endif
