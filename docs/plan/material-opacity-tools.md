# Material Opacity Micro-Tools

Each of these is an **independent system** — a bridge command + action type + (optionally) agent capability entry. The AI planner composes them naturally for use cases like see-through effects, but they're all individually useful for other purposes.

---

## `SetBodyOpacity`

Sets uniform opacity across all material surfaces on a figure's current shape. Useful for ghost/fade/see-through effects.

**Bridge command:** `set_body_opacity`
**Parameters:** `node_id`, `value` (float 0.0–1.0)
**C++ impl:** Iterates `shape->getMaterial(i)`, finds `"Opacity"` property, casts to `DzFloatProperty`, calls `setValue(value)`.
**Alternatives:** `set_material_property` (if targeting a single surface)

## `SetSurfaceOpacity`

Sets opacity on material surfaces matching a name/pattern. Pattern can be exact name or substring match. Selective control — e.g., only make the torso transparent while leaving limbs solid.

**Bridge command:** `set_surface_opacity`
**Parameters:** `node_id`, `surface_pattern` (string), `value` (float 0.0–1.0)
**C++ impl:** Iterates materials, matches `mat->getName()` or `mat->getLabel()` against pattern (case-insensitive substring), sets Opacity on matches.

## `GetInternalSurfaces`

Returns a list of material surface names that are likely internal anatomy (skeleton). Pattern-matched heuristically against known internal surface naming conventions.

**Bridge command:** `get_internal_surfaces`
**Parameters:** `node_id`
**Returns:** `["Skull", "Ribcage", "Pelvis", "Spine", ...]`
**C++ impl:** Iterates materials, matches names against internal keywords: "Skull", "Bone", "Rib", "Spine", "Pelvis", "Clavicle", "Scapula", "Skeleton", etc.

## `ShowAnatomy`

Convenience tool — calls `GetInternalSurfaces` then `SetSurfaceOpacity` for each result to value 1.0. Makes skeleton fully visible regardless of what happened to body opacity.

**Bridge command:** `show_anatomy`
**Parameters:** `node_id`
**C++ impl:** Reuses `get_internal_surfaces` logic + `set_surface_opacity` loop.

## `PlaceAssetInside`

Loads an asset and positions it inside a figure at the torso/abdomen area. Independent — useful for any "object inside body" scenario, not just opacity effects.

**Bridge command:** `place_asset_inside`
**Parameters:** `figure_id`, `asset_path`
**C++ impl:** `load_asset(asset_path)` → find loaded node → compute stomach center from figure bounding box → `set_node_transform` to position → parent to figure.
**Alternatives:** `load_asset` + `set_node_transform` (manually positioned)

---

## How the AI Composes Them

The examples below are just natural compositions. Each tool works independently.

### Example: See-through body with visible skeleton

```
SetBodyOpacity(figure, 0.15)
SetSurfaceOpacity(figure, "Stomach", 0.02)
SetSurfaceOpacity(figure, "Torso", 0.05)
ShowAnatomy(figure)
PlaceAssetInside(figure, "/alien_prop.duf")
```

### Example: Ghost effect (only opacity, no skeleton)

```
SetBodyOpacity(figure, 0.3)
```

### Example: X-ray arm (one limb, no skeleton)

```
SetSurfaceOpacity(figure, "Forearm", 0.1)
SetSurfaceOpacity(figure, "Hand", 0.1)
```

---

## No UV/Texture Work Required

All opacity control uses `DzFloatProperty` on the existing `Opacity` material channel. The figure's UVs are untouched — we never assign a texture map unless the user explicitly wants a gradient opacity map for smoother falloff.

---

## Files

| File | Change |
|---|---|
| `plugins/daz3d-bridge/DazPilotBridgePlugin.cpp` | Add 5 independent `if (command == "...")` handlers |
| `src-tauri/src/knowledge/workflow_knowledge.rs` | Add 5 ActionType variants + command mappings |
| `src-tauri/src/ai_action.rs` | Add 5 `execute_*` functions + dispatch entries |
