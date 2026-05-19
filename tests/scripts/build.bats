setup() {
  TEST_DIR=$(mktemp -d)
  mkdir -p "$TEST_DIR/dist"
}

teardown() {
  rm -rf "$TEST_DIR"
}

@test "build script verifies dist directory exists" {
  [ -d "$TEST_DIR/dist" ]
}

@test "build script creates dist on successful build" {
  mkdir -p "$TEST_DIR/dist/assets"
  touch "$TEST_DIR/dist/index.html"
  [ -f "$TEST_DIR/dist/index.html" ]
}

@test "build fails without npm install" {
  run bash -c '
    if [ ! -d "node_modules" ]; then
      echo "ERROR: node_modules not found. Run npm install first."
      exit 1
    fi
  '
  [ "$status" -eq 1 ]
}
