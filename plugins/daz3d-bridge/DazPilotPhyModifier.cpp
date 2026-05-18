/*****************************
    DazPilotPhy Modifier
    Real-Time Mass-Spring Geometry Modifier for Daz Studio
    Copyright (C) DazPilot. All Rights Reserved.

    DazPilotPhy implements a per-vertex mass-spring simulation that runs
    entirely inside Daz Studio's geometry pipeline. Every frame, spring forces
    pull displaced vertices back toward their rest positions while exponential
    damping prevents infinite oscillation — producing physically believable
    jiggle and secondary motion without dForce.
*****************************/

#include "DazPilotPhyModifier.h"
#include "dzvertexmesh.h"
#include "dzobject.h"
#include "idzjsonio.h"
#include <QtCore/QCoreApplication>
#include <cstring>
#include <cmath>

// ─── Constants ────────────────────────────────────────────────────────────────
static constexpr float FIXED_DT       = 1.0f / 60.0f; // 60 Hz simulation
static constexpr float MAX_VELOCITY   = 5.0f;          // Clamp runaway velocities
static constexpr char  MODIFIER_NAME[] = "DazPilotPhy";

// ─── Constructor ──────────────────────────────────────────────────────────────
DazPilotPhyModifier::DazPilotPhyModifier()
    : m_initialized(false)
    , m_stiffness(12.0f)
    , m_damping(4.0f)
    , m_mass(0.5f)
    , m_stiffnessProp(nullptr)
    , m_dampingProp(nullptr)
    , m_massProp(nullptr)
{
    setObjectName(MODIFIER_NAME);

    // Register animatable Daz properties so they appear in the Scene pane
    // and can be driven by the timeline or ERC links.
    m_stiffnessProp = new DzFloatProperty("Stiffness", true, false, 12.0f);
    m_stiffnessProp->setMin(0.1f);
    m_stiffnessProp->setMax(50.0f);
    addProperty(m_stiffnessProp);

    m_dampingProp = new DzFloatProperty("Damping", true, false, 4.0f);
    m_dampingProp->setMin(0.0f);
    m_dampingProp->setMax(20.0f);
    addProperty(m_dampingProp);

    m_massProp = new DzFloatProperty("Mass", true, false, 0.5f);
    m_massProp->setMin(0.01f);
    m_massProp->setMax(5.0f);
    addProperty(m_massProp);
}

DazPilotPhyModifier::~DazPilotPhyModifier() {}

// ─── Parameter setters ────────────────────────────────────────────────────────
void DazPilotPhyModifier::setStiffness(float v) {
    m_stiffness = v;
    if (m_stiffnessProp) m_stiffnessProp->setValue(v);
}
void DazPilotPhyModifier::setDamping(float v) {
    m_damping = v;
    if (m_dampingProp) m_dampingProp->setValue(v);
}
void DazPilotPhyModifier::setMass(float v) {
    m_mass = qMax(0.001f, v);
    if (m_massProp) m_massProp->setValue(m_mass);
}
float DazPilotPhyModifier::getStiffness() const { return m_stiffness; }
float DazPilotPhyModifier::getDamping()   const { return m_damping;   }
float DazPilotPhyModifier::getMass()      const { return m_mass;      }

void DazPilotPhyModifier::resetSimulation() {
    m_initialized = false;
    m_states.clear();
}

// ─── apply() — called every frame by the geometry pipeline ───────────────────
DzError DazPilotPhyModifier::apply(DzVertexMesh& geom, DzNode& node) {
    DzPnt3* verts = geom.getVerticesPtr();
    const int n   = geom.getNumVertices();
    if (!verts || n == 0) return DZ_NO_ERROR;

    // Read live property values (allows timeline-driven parameters)
    const float k    = m_stiffnessProp ? m_stiffnessProp->getValue() : m_stiffness;
    const float b    = m_dampingProp   ? m_dampingProp->getValue()   : m_damping;
    const float mVal = m_massProp      ? qMax<float>(0.001f, m_massProp->getValue()) : m_mass;

    // ── First frame: capture rest positions ───────────────────────────────────
    if (!m_initialized || m_states.size() != n) {
        m_states.resize(n);
        for (int i = 0; i < n; i++) {
            m_states[i].restPos[0]  = verts[i][0];
            m_states[i].restPos[1]  = verts[i][1];
            m_states[i].restPos[2]  = verts[i][2];
            m_states[i].velocity[0] = 0.0f;
            m_states[i].velocity[1] = 0.0f;
            m_states[i].velocity[2] = 0.0f;
        }
        m_initialized = true;
        return DZ_NO_ERROR; // Skip physics on first frame
    }

    // ── Mass-Spring Euler Integration ─────────────────────────────────────────
    // For each vertex:
    //   F = k*(restPos - currentPos) - b*velocity   [spring + damper]
    //   a = F / m
    //   velocity += a * dt
    //   position += velocity * dt
    for (int i = 0; i < n; i++) {
        DazPilotPhyVertexState& s = m_states[i];

        // Displacement from rest
        float dx = s.restPos[0] - verts[i][0];
        float dy = s.restPos[1] - verts[i][1];
        float dz = s.restPos[2] - verts[i][2];

        // Acceleration (F/m)
        float ax = (k * dx - b * s.velocity[0]) / mVal;
        float ay = (k * dy - b * s.velocity[1]) / mVal;
        float az = (k * dz - b * s.velocity[2]) / mVal;

        // Integrate velocity
        s.velocity[0] += ax * FIXED_DT;
        s.velocity[1] += ay * FIXED_DT;
        s.velocity[2] += az * FIXED_DT;

        // Clamp velocity to prevent explosion on first large displacement
        float spd = std::sqrtf(s.velocity[0]*s.velocity[0] + s.velocity[1]*s.velocity[1] + s.velocity[2]*s.velocity[2]);
        if (spd > MAX_VELOCITY) {
            float scale = MAX_VELOCITY / spd;
            s.velocity[0] *= scale;
            s.velocity[1] *= scale;
            s.velocity[2] *= scale;
        }

        // Integrate position
        verts[i][0] += s.velocity[0] * FIXED_DT;
        verts[i][1] += s.velocity[1] * FIXED_DT;
        verts[i][2] += s.velocity[2] * FIXED_DT;
    }

    return DZ_NO_ERROR;
}

// ─── applyInverse() — restore mesh to unmodified state ───────────────────────
DzError DazPilotPhyModifier::applyInverse(DzVertexMesh& geom, DzNode& node) {
    // Daz calls this when the modifier is evaluated in reverse order
    // (e.g. when saving or exporting). We restore rest positions.
    if (!m_initialized) return DZ_NO_ERROR;
    DzPnt3* verts = geom.getVerticesPtr();
    const int n   = geom.getNumVertices();
    if (!verts || m_states.size() != n) return DZ_NO_ERROR;

    for (int i = 0; i < n; i++) {
        verts[i][0] = m_states[i].restPos[0];
        verts[i][1] = m_states[i].restPos[1];
        verts[i][2] = m_states[i].restPos[2];
    }
    return DZ_NO_ERROR;
}

// ─── DazPilotPhy Modifier IO ──────────────────────────────────────────────────────
struct DazPilotPhyModifierIO::Context {
    float stiffness = 12.0f;
    float damping   = 4.0f;
    float mass      = 0.5f;
};

DazPilotPhyModifierIO::DazPilotPhyModifierIO() : m_context(nullptr) {}
DazPilotPhyModifierIO::~DazPilotPhyModifierIO() { delete m_context; }

DzModifier* DazPilotPhyModifierIO::createModifier(const QString& name, DzObject* tgtObject) const {
    DazPilotPhyModifier* mod = new DazPilotPhyModifier();
    tgtObject->addModifier(mod);
    return mod;
}

DzError DazPilotPhyModifierIO::writeExtraDefinition(QObject* obj, IDzJsonIO* io, const DzFileIOSettings* opts) const {
    io->addMember("vendor", "DazPilot");
    io->addMember("plugin",  "DazPilotPhy");
    return DZ_NO_ERROR;
}

DzError DazPilotPhyModifierIO::writeExtraInstance(QObject* obj, IDzJsonIO* io, const DzFileIOSettings* opts) const {
    DazPilotPhyModifier* mod = qobject_cast<DazPilotPhyModifier*>(obj);
    if (!mod) return DZ_NO_ERROR;
    io->addMember("stiffness", (double)mod->getStiffness());
    io->addMember("damping",   (double)mod->getDamping());
    io->addMember("mass",      (double)mod->getMass());
    return DZ_NO_ERROR;
}

DzAssetJsonObject* DazPilotPhyModifierIO::startDefinitionRead(DzAssetJsonItem* parentItem) {
    return nullptr;
}

DzAssetJsonObject* DazPilotPhyModifierIO::startInstanceRead(DzAssetJsonItem* parentItem) {
    m_context = new Context();
    return nullptr; // Simple numeric fields read by applyInstanceToObject
}

DzError DazPilotPhyModifierIO::applyDefinitionToObject(QObject* obj, const DzFileIOSettings* opts) const {
    return DZ_NO_ERROR;
}

DzError DazPilotPhyModifierIO::applyInstanceToObject(QObject* obj, const DzFileIOSettings* opts) const {
    DazPilotPhyModifier* mod = qobject_cast<DazPilotPhyModifier*>(obj);
    if (mod && m_context) {
        mod->setStiffness(m_context->stiffness);
        mod->setDamping(m_context->damping);
        mod->setMass(m_context->mass);
    }
    return DZ_NO_ERROR;
}

#include "moc_DazPilotPhyModifier.cpp"
