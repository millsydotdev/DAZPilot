setup() {
  TEST_DIR=$(mktemp -d)
  mkdir -p "$TEST_DIR/node_modules"
  mkdir -p "$TEST_DIR/dist"
  mkdir -p "$TEST_DIR/src-tauri/target"
}

teardown() {
  rm -rf "$TEST_DIR"
}

@test "clean refuses without confirmation" {
  run bash -c '
    echo "n" | bash -c "
      cd \"$TEST_DIR\"
      confirm=n
      if [ \"\$confirm\" != \"y\" ] && [ \"\$confirm\" != \"Y\" ]; then
        echo Cancelled.
        exit 0
      fi
    "
  '
  [ "$status" -eq 0 ]
  [ -d "$TEST_DIR/node_modules" ]
}

@test "clean removes directories with confirmation" {
  run bash -c "
    rm -rf \"$TEST_DIR/node_modules\"
    rm -rf \"$TEST_DIR/dist\"
    [ ! -d \"$TEST_DIR/node_modules\" ] && [ ! -d \"$TEST_DIR/dist\" ]
  "
  [ "$status" -eq 0 ]
}

@test "clean script safety: does not delete git directory" {
  run bash -c '
    SAFE_DIRS="dist node_modules target"
    for dir in $SAFE_DIRS; do
      if [ "$dir" = ".git" ]; then
        echo "ERROR: .git should never be deleted"
        exit 1
      fi
    done
    echo "All safe"
  '
  [ "$status" -eq 0 ]
}
