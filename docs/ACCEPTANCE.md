# Bridge Acceptance

## Automated (CI / local, no Daz Studio required)

```powershell
npm run acceptance
```

This runs:

1. `npm run check` (Rust clippy, typecheck, lint, format check, Rust fmt, frontend tests)
2. `cargo test acceptance_` with `DAZPILOT_DEV_MOCK_BRIDGE=1`

Rust tests validate full command-name parity between `DazPilotBridgePlugin.cpp` and `mcp_client.rs`; mock bridge tests exercise representative workflow commands including `get_scene_assets`, `add_figure`, `set_morph`, `set_light`, and `set_render_settings`.

## Manual (live Daz Studio)

Requires Daz Studio with `DazPilotBridge.dll` installed and listening on `127.0.0.1:8765`.

### Setup

| # | Step | Expected |
| --- | --- | --- |
| 0.1 | Build bridge: `npm run plugin:rebuild` | `DazPilotBridge.dll` in `dist/Release/` |
| 0.2 | Install plugin (auto or copy to Daz plugins folder) | DLL in Daz plugins dir |
| 0.3 | Start Daz Studio | Bridge log: listening on 8765 |
| 0.4 | Start DazPilot → connect bridge | Launcher shows Daz Studio connected |

### Scene & Node Commands

| # | Command / action | Expected |
| --- | --- | --- |
| 1.1 | `get_scene_info` | Scene summary with node count, render engine, dimensions |
| 1.2 | `list_nodes` | List of all scene nodes with labels and types |
| 1.3 | `get_selected_nodes` | Currently selected node(s) |
| 1.4 | `select_node` with a node ID | Node becomes selected in Daz Studio |
| 1.5 | `add_node` (null, camera, point_light, spot_light, distant_light) | New node appears in scene |
| 1.6 | `delete_node` with an existing node ID | Node removed from scene |
| 1.7 | `get_geoshells` | List of geometry shells in scene |
| 1.8 | `get_scene_assets` | List of loaded asset paths in scene |

### Figure & Morph Commands

| # | Command / action | Expected |
| --- | --- | --- |
| 2.1 | `add_figure` genesis8 | Genesis 8 figure appears in scene |
| 2.2 | `add_figure` genesis9 | Genesis 9 figure appears in scene |
| 2.3 | `get_figure_morphs` on a figure | All morph dials and current values |
| 2.4 | `set_morph` with morph name and 0.5 | Morph dial updates in Daz Studio |
| 2.5 | `apply_morph` with figure_id, morph_id, value | Morph applies correctly |
| 2.6 | `get_active_expressions` on a figure | Current expression dial values |
| 2.7 | `apply_expression` with expression_id and value | Expression updates on figure |
| 2.8 | `get_fitted_items` on a figure | List of fitted clothing/accessories |
| 2.9 | `get_material_zones` on a figure | Material zone names |

### Property & Material Commands

| # | Command / action | Expected |
| --- | --- | --- |
| 3.1 | `get_node_properties` on a node | Animatable properties with current values |
| 3.2 | `set_property` with property name and new value | Property updates in Daz Studio |
| 3.3 | `set_material_property` with property and value | Material property updates in viewport |

### Asset & Pose Commands

| # | Command / action | Expected |
| --- | --- | --- |
| 4.1 | `load_asset` with a .duf path | Asset appears in scene |
| 4.2 | `apply_pose` with pose_path and figure_id | Pose applies to figure |
| 4.3 | `import_model` with an OBJ/FBX path | Model imports into scene |

### Scene Export

| # | Command / action | Expected |
| --- | --- | --- |
| 5.1 | `export_scene` with node_id, path, format=obj | OBJ file written to disk |
| 5.2 | `export_scene` with format=fbx | FBX file written to disk |
| 5.3 | `export_scene` with format=gltf | GLTF file written to disk |
| 5.4 | `export_scene` with selected_only=true | Only selected node exported |
| 5.5 | `export_scene` via DazScript fallback (bridge export disabled) | Falls back and succeeds |

### Lighting & Rendering

| # | Command / action | Expected |
| --- | --- | --- |
| 6.1 | `set_light` with intensity property | Light intensity changes |
| 6.2 | `set_light` with color property | Light color changes |
| 6.3 | `set_render_settings` with width, height | Render resolution updates |
| 6.4 | `render_preview` | Render starts or preview updates |
| 6.5 | `get_cameras` | List of scene cameras |

### Viewport

| # | Command / action | Expected |
| --- | --- | --- |
| 7.1 | `capture_viewport` with path | Viewport image written to path |
| 7.2 | `viewport_click` with x, y coordinates | Node under cursor selected |
| 7.3 | `get_bounding_boxes` | 3D bounding boxes for all nodes |

### Animation

| # | Command / action | Expected |
| --- | --- | --- |
| 8.1 | `set_timeline_range` with start_frame, end_frame | Timeline range updates |
| 8.2 | `seek_to_frame` with frame number | Timeline cursor moves |
| 8.3 | `set_keyframe` with node_id, property, frame, value, interpolation | Keyframe set on timeline |
| 8.4 | `play_timeline` | Timeline plays |
| 8.5 | `pause_timeline` | Timeline pauses |
| 8.6 | `stop_timeline` | Timeline stops, resets to frame 0 |
| 8.7 | `get_timeline_state` | Current frame, range, fps, playing state |

### Physics

| # | Command / action | Expected |
| --- | --- | --- |
| 9.1 | `run_dforce_simulation` with node_id, start_frame, end_frame | dForce simulation runs on figure |

### Scripting

| # | Command / action | Expected |
| --- | --- | --- |
| 10.1 | `run_script` with valid DazScript | Script executes, returns success |

### Undo Batching

| # | Command / action | Expected |
| --- | --- | --- |
| 11.1 | `begin_undo_batch` | Undo batch starts |
| 11.2 | Execute operations | Operations grouped in one undo step |
| 11.3 | `accept_undo_batch` with caption | Undo step added with caption |
| 11.4 | `begin_undo_batch` → `cancel_undo_batch` | Operations rolled back |

### AI Action Pipeline (end-to-end)

| # | Prompt | Expected |
| --- | --- | --- |
| 12.1 | "list nodes" | Heuristic parser or AI returns structured action, executes, reply |
| 12.2 | "add a Genesis 9 figure" | Figure added to scene |
| 12.3 | "set morph Head_Height to 0.3" | Morph value changes |
| 12.4 | "load [indexed asset name]" | Asset loaded into scene |
| 12.5 | "capture the viewport" | Viewport image captured and displayed |
| 12.6 | "render preview at 1920x1080" | Render settings applied, preview triggered |
| 12.7 | "create a scene with a figure, light, and camera" | Multi-step action plan, all elements created |
| 12.8 | "export the scene as OBJ" | Scene exported to file |

### Recording Results

When cutting a release, go through every row and record:

```
v0.5.1 acceptance results:
  Passed: 1.1–1.8, 2.1–2.9, 3.1–3.3, 4.1–4.3, ...
  Failed: 5.2 (FBX exporter not found in Daz SDK)
  Notes: Export with selected_only=true needs SDK testing
```
