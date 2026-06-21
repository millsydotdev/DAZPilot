#!/usr/bin/env python3
"""stdio-to-HTTP bridge for DAZStudio-MCP.

Claude Desktop spawns this script via stdio transport.
It forwards each JSON-RPC message as an HTTP POST to the
DAZStudio-MCP plugin running inside DAZ Studio.
"""

import json
import sys
import urllib.request
import urllib.error

SERVER_URL = "http://127.0.0.1:8765/mcp"


def main():
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue

        body = line.encode("utf-8")
        req = urllib.request.Request(
            SERVER_URL,
            data=body,
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        try:
            with urllib.request.urlopen(req, timeout=30) as resp:
                result = resp.read().decode("utf-8")
                sys.stdout.write(result + "\n")
                sys.stdout.flush()
        except urllib.error.HTTPError as e:
            error_body = e.read().decode("utf-8")
            sys.stdout.write(error_body + "\n")
            sys.stdout.flush()
        except urllib.error.URLError:
            err = json.dumps({
                "jsonrpc": "2.0",
                "id": None,
                "error": {
                    "code": -32099,
                    "message": "DAZStudio-MCP unreachable — is DAZ Studio running?"
                }
            })
            sys.stdout.write(err + "\n")
            sys.stdout.flush()


if __name__ == "__main__":
    main()
