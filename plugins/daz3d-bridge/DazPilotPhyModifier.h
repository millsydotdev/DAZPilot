#pragma once

/*****************************
    DazPilotPhy Modifier
    Real-Time Mass-Spring Geometry Modifier for Daz Studio
    Copyright (C) DazPilot. All Rights Reserved.
*****************************/

#include "dzmodifier.h"
#include "idzsceneasset.h"
#include "dzassetextraobjectio.h"
#include "dzfloatproperty.h"
#include <QtCore/QVector>
#include <QtCore/QObject>

// ─── Per-Vertex Physics State ─────────────────────────────────────────────────
struct DazPilotPhyVertexState {
    float restPos[3];   // Captured rest position (world-space)
    float velocity[3];  // Current velocity
};

// ─── DazPilotPhy Modifier ─────────────────────────────────────────────────────────
class DazPilotPhyModifier : public DzModifier, public DzSceneAsset {
    Q_OBJECT
    Q_INTERFACES(IDzSceneAsset)
    friend class DazPilotPhyModifierIO;

public:
    DazPilotPhyModifier();
    ~DazPilotPhyModifier();

    // ── DzModifier interface ─────────────────────────────────────────────────
    virtual DzError apply(DzVertexMesh& geom, DzNode& node) override;
    virtual DzError applyInverse(DzVertexMesh& geom, DzNode& node) override;

    // ── DzSceneAsset interface ───────────────────────────────────────────────
    virtual const QObject* toQObject() const override { return this; }
    virtual QObject*       toQObject()       override { return this; }
    virtual AssetType      getAssetType() const override { return ModifierAsset; }

    // ── Parameter accessors (called by bridge commands) ──────────────────────
    void setStiffness(float v);
    void setDamping(float v);
    void setMass(float v);
    void resetSimulation();

    float getStiffness() const;
    float getDamping()   const;
    float getMass()      const;

signals:
    void assetModified();
    void assetWasSaved();

private:
    bool                      m_initialized;
    QVector<DazPilotPhyVertexState> m_states;

    float m_stiffness;   // Spring constant   — default 12.0
    float m_damping;     // Velocity damper   — default 4.0
    float m_mass;        // Per-vertex mass   — default 0.5

    DzFloatProperty* m_stiffnessProp;
    DzFloatProperty* m_dampingProp;
    DzFloatProperty* m_massProp;
};

// ─── DazPilotPhy IO (serialisation for .duf scene files) ─────────────────────────
class DazPilotPhyModifierIO : public DzExtraModifierIO {
    Q_OBJECT
public:
    DazPilotPhyModifierIO();
    ~DazPilotPhyModifierIO();

    virtual DzModifier*         createModifier(const QString& name, DzObject* tgtObject) const override;
    virtual DzError             writeExtraDefinition(QObject* obj, IDzJsonIO* io, const DzFileIOSettings* opts) const override;
    virtual DzError             writeExtraInstance(QObject* obj, IDzJsonIO* io, const DzFileIOSettings* opts) const override;
    virtual DzAssetJsonObject*  startDefinitionRead(DzAssetJsonItem* parentItem) override;
    virtual DzAssetJsonObject*  startInstanceRead(DzAssetJsonItem* parentItem)   override;
    virtual DzError             applyDefinitionToObject(QObject* obj, const DzFileIOSettings* opts) const override;
    virtual DzError             applyInstanceToObject(QObject* obj, const DzFileIOSettings* opts)   const override;

    struct Context;
    Context* m_context;
};
