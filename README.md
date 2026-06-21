# DazPilot

> **Disclaimer:** This plug‑in is **not** affiliated with, endorsed by, or sponsored by DAZ 3D.


MCP server for AI-driven DAZ Studio scene editing. Runs as a native DAZ Studio plugin (DLL) and exposes all scene control, rendering, dForce, asset, and animation commands through the [Model Context Protocol](https://modelcontextprotocol.io).

## How It Works

```
Claude Desktop  ──stdio──>  daz-mcp-client.py  ──HTTP POST /mcp──>  DazPilot (DLL inside DAZ Studio)
```

- The plugin hosts an HTTP server on `127.0.0.1:8765` serving a single endpoint `POST /mcp`
- Clients send [JSON-RPC 2.0](https://www.jsonrpc.org/specification) messages with MCP methods: `initialize`, `tools/list`, `tools/call`
- Each command maps to DAZ Studio SDK calls executed on DAZ's main thread
- The Python shim `daz-mcp-client.py` translates MCP stdio transport to HTTP for Claude Desktop

## 196 Commands

All categories — scene, selection, properties, materials, morphs, cameras, lights, rendering, assets, animation, dForce simulation, poses, clothing, hair, environment, export, viewport, rigging, scripting, mesh queries.

## Build

### Prerequisites
- CMake 3.15+
- C++17 compiler (MSVC 2019+)
- DAZ Studio 4.5+ SDK (place the SDK in a folder named `thirdparty` at the repository root; obtain it from DAZ or your own source, it is not pulled automatically)

### Quick Start
```powershell
cmake -B build -DDAZ_SDK_ROOT="C:\Program Files\DAZ 3D\DAZStudio4\SDK"
cmake --build build --config Release
```

Header-only test build (no SDK):
```powershell
cmake -B build -DBUILD_TESTS=ON
cmake --build build --config Release
```

## Install

Copy `DazPilot.dll` to your DAZ Studio plugins directory:
```
C:\Program Files\DAZ 3D\DAZStudio4\plugins\
```

Restart DAZ Studio. The server starts automatically on port 8765.

### Direct HTTP usage

If you prefer to talk to the MCP server directly (without the Python shim), you can POST JSON‑RPC requests to the plugin’s HTTP endpoint. The server runs at `http://127.0.0.1:8765/mcp` and follows the JSON‑RPC 2.0 specification.

### Using `curl`
```sh
curl -X POST http://127.0.0.1:8765/mcp \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
```
The response will be a JSON object containing the list of available commands.

### Using Python (`requests`)
```python
import requests
payload = {"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}}
resp = requests.post('http://127.0.0.1:8765/mcp', json=payload)
print(resp.json())
```

### Using JavaScript (Fetch API)
```js
fetch('http://127.0.0.1:8765/mcp', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ jsonrpc: '2.0', id: 1, method: 'tools/list', params: {} })
})
  .then(r => r.json())
  .then(console.log);
```

All other methods described in the **MCP Protocol** section work the same way – just replace the `method` and `params` fields. The plugin will reply with a JSON‑RPC response containing either `result` or `error`.


```json
{
  "mcpServers": {
    "dazpilot": {
      "command": "python",
      "args": ["path/to/daz-mcp-client.py"]
    }
  }
}
```

The shim forwards stdio JSON-RPC to `http://127.0.0.1:8765/mcp`.

## MCP Protocol

The plugin implements [Streamable HTTP](https://spec.modelcontextprotocol.io/2025-03-26/basic/transport/) transport:

- **Endpoint**: `POST http://127.0.0.1:8765/mcp`
- **Content-Type**: `application/json`
- **Body**: JSON-RPC 2.0 request object

### Methods

| Method | Purpose |
|--------|---------|
| `initialize` | Server capabilities & version |
| `tools/list` | List all 196 commands |
| `tools/call` | Execute a command |

### Example

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_scene_info","arguments":{}}}
```

## License

MIT
