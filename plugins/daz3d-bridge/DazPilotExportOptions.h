#pragma once

#include <QString>
#include <QStringList>
#include <QSize>

class DzFileIOSettings;

class DazPilotExportOptions {
public:
    DazPilotExportOptions() {
        resetToDefaults();
    }
    ~DazPilotExportOptions() {}

    // Export flow settings (mapped to/from JSON command)
    bool selectedOnly;
    bool includeMaterials;
    bool includeAnimations;
    bool bakeTextures;

    // Asset info
    QString assetType; // e.g., "SkeletalMesh", "StaticMesh", "Animation", "Pose", "Light"
    QString exportFilename; // without extension
    QString exportSubfolder; // subfolder within root folder
    QString rootFolder; // destination root folder
    QString productName;
    QString productComponentName;
    QStringList morphList; // list of morphs to export
    bool useRelativePaths; // use relative paths in DTU

    // Normal maps
    bool generateNormalMaps; // generate normal maps from height maps
    bool undoNormalMaps; // remove generated normal maps after export

    // FBX
    QString exportFbx; // override filename of exported fbx

    // LOD
    bool enableLodGeneration; // enable level-of-detail generation
    int lodMethodIndex; // 0: PreGenerated, 1: Decimator
    QString lodMethodString; // e.g., "Decimator"
    int numberOfLods; // total number of LOD levels (including Base LOD)

    // Texture settings
    bool convertToPng;
    bool convertToJpg; // overrides convertToPng if true
    bool exportAllTextures;
    bool combineDiffuseAndAlphaMaps;
    bool resizeTextures;
    int targetTextureWidth;
    int targetTextureHeight;
    QSize targetTextureSize; // default 4096x4096
    bool multiplyTextureValues;
    bool recompressIfFileSizeTooBig;
    int fileSizeThresholdToInitiateRecompression; // in bytes
    bool forceReEncoding;

    // Bake settings
    bool bakeMakeupOverlay;
    bool bakeTranslucency;
    bool bakeSpecularToMetallic;
    bool bakeRefractionWeight;

    // Bake modes: 0: NeverBake, 1: AlwaysBake, -1: Ask
    int bakeInstancesMode; // 0: Never, 1: Always
    int bakePivotPointsMode; // 0: Never, 1: Always
    int bakeRigidFollowNodesMode; // 0: Never, 1: Always

    // Animation settings (if needed)
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

    // Parse a JSON settings string and populate this struct
    static DazPilotExportOptions fromJson(const QString& json);

    // Apply these options to a DzFileIOSettings object for the exporter
    void applyToSettings(DzFileIOSettings* settings) const;

    // Clear all options to defaults
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
        lodMethodIndex = 0; // PreGenerated
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
        fileSizeThresholdToInitiateRecompression = 10*1024*1024; // 10 MB
        forceReEncoding = false;

        bakeMakeupOverlay = false;
        bakeTranslucency = false;
        bakeSpecularToMetallic = false;
        bakeRefractionWeight = false;

        bakeInstancesMode = 0; // Never
        bakePivotPointsMode = 0; // Never
        bakeRigidFollowNodesMode = 0; // Never

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
