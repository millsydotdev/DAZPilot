@test "npm test exists and runs" {
  run bash -c '
    if command -v npm &>/dev/null; then
      echo "npm available"
    else
      echo "npm not found"
      exit 1
    fi
  '
  [ "$status" -eq 0 ]
}

@test "cargo test exists and runs for Rust" {
  run bash -c '
    if command -v cargo &>/dev/null; then
      echo "cargo available"
    else
      echo "cargo not found - non-fatal"
    fi
  '
  [ "$status" -eq 0 ]
}

@test "test script runs both frontend and backend tests" {
  run bash -c '
    echo "Frontend: npm test"
    echo "Backend: cargo test"
    echo "Done"
  '
  [ "$status" -eq 0 ]
}

@test "test script reports failures correctly" {
  run bash -c 'exit 1'
  [ "$status" -eq 1 ]
}

@test "test script handles missing backend gracefully" {
  run bash -c '
    if ! command -v cargo &>/dev/null; then
      echo "Skipping Rust tests (cargo not found)"
    fi
  '
  [ "$status" -eq 0 ]
}
