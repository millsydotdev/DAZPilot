#pragma once

#include <QString>
#include <QStringList>
#include <QSize>

class DzFileIOSettings;

class ExportOptions {
public:
    ExportOptions() {
        resetToDefaults();
    }
    ~ExportOptions() {}

    // Export flow settings (mapped to/from JSON command)
    bool selectedOnly;
    bool includeMaterials;
    bool includeAnimations;
    bool bakeTextures;

    // Asset info
    QString assetType;
    QString exportFilename;
    QString exportSubfolder;
    QString rootFolder;
    QString productName;
    QString productComponentName;
    QStringList morphList;
    bool useRelativePaths;

    // Normal maps
    bool generateNormalMaps;
    bool undoNormalMaps;

    // FBX
    QString exportFbx;

    // LOD
    bool enableLodGeneration;
    int lodMethodIndex;
    QString lodMethodString;
    int numberOfLods;

    // Texture settings
    bool convertToPng;
    bool convertToJpg;
    bool exportAllTextures;
    bool combineDiffuseAndAlphaMaps;
    bool resizeTextures;
    int targetTextureWidth;
    int targetTextureHeight;
    QSize targetTextureSize;
    bool multiplyTextureValues;
    bool recompressIfFileSizeTooBig;
    int fileSizeThresholdToInitiateRecompression;
    bool forceReEncoding;

    // Bake settings
    bool bakeMakeupOverlay;
    bool bakeTranslucency;
    bool bakeSpecularToMetallic;
    bool bakeRefractionWeight;

    // Bake modes
    int bakeInstancesMode;
    int bakePivotPointsMode;
    int bakeRigidFollowNodesMode;

    // Animation settings
    bool animationUseExperimentalTransfer;
    bool animationBake;
    bool animationTransferFace;
    bool animationExportActiveCurves;
    bool animationApplyBoneScale;

    // Post-process FBX
    bool postProcessFbx;
    bool removeDuplicateGeografts;
    bool experimentalFbxPostProcessing;

    // Morph settings
    bool morphLockBoneTranslation;
    bool enableAutoJcm;
    bool enableFakeDualQuat;
    bool allowMorphDoubleDipping;

    // Lod settings (additional)
    bool createLodGroup;

    static ExportOptions fromJson(const QString& json);
    void applyToSettings(DzFileIOSettings* settings) const;

    void resetToDefaults() {
        selectedOnly = false;
        includeMaterials = true;
        includeAnimations = true;
        bakeTextures = false;

        assetType = "SkeletalMesh";
        exportFilename = "";
        exportSubfolder = "";
        rootFolder = "";
        productName = "";
        productComponentName = "";
        morphList.clear();
        useRelativePaths = false;

        generateNormalMaps = false;
        undoNormalMaps = false;

        exportFbx = "";

        enableLodGeneration = false;
        lodMethodIndex = 0;
        lodMethodString = "PreGenerated";
        numberOfLods = 3;

        convertToPng = false;
        convertToJpg = false;
        exportAllTextures = false;
        combineDiffuseAndAlphaMaps = false;
        resizeTextures = false;
        targetTextureWidth = 4096;
        targetTextureHeight = 4096;
        targetTextureSize = QSize(4096, 4096);
        multiplyTextureValues = false;
        recompressIfFileSizeTooBig = false;
        fileSizeThresholdToInitiateRecompression = 10*1024*1024;
        forceReEncoding = false;

        bakeMakeupOverlay = false;
        bakeTranslucency = false;
        bakeSpecularToMetallic = false;
        bakeRefractionWeight = false;

        bakeInstancesMode = 0;
        bakePivotPointsMode = 0;
        bakeRigidFollowNodesMode = 0;

        animationUseExperimentalTransfer = false;
        animationBake = false;
        animationTransferFace = false;
        animationExportActiveCurves = false;
        animationApplyBoneScale = false;

        postProcessFbx = false;
        removeDuplicateGeografts = false;
        experimentalFbxPostProcessing = false;

        morphLockBoneTranslation = false;
        enableAutoJcm = false;
        enableFakeDualQuat = false;
        allowMorphDoubleDipping = false;

        createLodGroup = false;
    }
};
