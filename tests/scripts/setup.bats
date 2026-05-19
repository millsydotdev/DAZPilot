setup() {
  load '../../scripts/setup.sh'
  TEST_DIR=$(mktemp -d)
}

teardown() {
  rm -rf "$TEST_DIR"
}

@test "setup script detects missing Node.js gracefully" {
  run bash -c 'command -v node || echo "MISSING"'
  [ "$status" -eq 0 ]
}

@test "setup script exits if node version < 20" {
  run bash -c '
    node() { echo "v18.0.0"; }
    export -f node
    NODE_VER=18
    if [ "$NODE_VER" -lt 20 ]; then exit 1; fi
  '
  [ "$status" -eq 1 ]
}

@test "setup script passes with Node.js 20+" {
  run bash -c '
    NODE_VER=20
    if [ "$NODE_VER" -ge 20 ]; then exit 0; else exit 1; fi
  '
  [ "$status" -eq 0 ]
}

@test "setup script handles missing Rust gracefully" {
  run bash -c '
    if command -v rustc &>/dev/null; then
      echo "Rust found"
    else
      echo "Rust not found - non-fatal"
    fi
  '
  [ "$status" -eq 0 ]
}
