# Bridge Acceptance

## Automated (CI / local, no Daz Studio required)

```powershell
npm run acceptance
```

This runs:

1. `npm run check` (typecheck, lint, frontend tests)
2. `cargo test acceptance_` with `DAZPILOT_DEV_MOCK_BRIDGE=1`

Mock bridge validates schema parity for workflow commands: `get_scene_assets`, `add_figure`, `set_morph`, `set_light`, `set_render_settings`.

## Manual (live Daz Studio)

Requires Daz Studio with `DazPilotBridge.dll` installed and listening on `127.0.0.1:8765`.

| Step | Command / action | Expected |
| --- | --- | --- |
| 1 | `npm run plugin:rebuild` then install plugin | DLL in Daz plugins folder |
| 2 | Start Daz Studio | Bridge log: listening on 8765 |
| 3 | DazPilot → connect bridge | Launcher shows Daz Studio connected |
| 4 | Chat: "list nodes" | Node list in reply |
| 5 | Chat: "load \<outfit\>" (indexed asset) | Asset appears in scene |
| 6 | Scene panel: toggle visibility / delete | Changes reflected in Daz |
| 7 | Chat: "set morph fitness to 0.5" | Morph updates on figure |
| 8 | Viewport capture | Image in Viewport tab |

Record results in your release notes when cutting a build.
