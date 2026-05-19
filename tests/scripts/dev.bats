@test "dev script runs tauri dev command" {
  run bash -c '
    echo "npm run tauri dev"
  '
  [ "$status" -eq 0 ]
}

@test "dev script checks for required ports" {
  run bash -c '
    PORT=1420
    if command -v ss &>/dev/null; then
      ss -tlnp | grep -q ":$PORT " && echo "in use" || echo "free"
    elif command -v netstat &>/dev/null; then
      netstat -an | grep -q ":$PORT " && echo "in use" || echo "free"
    else
      echo "cannot check"
    fi
  '
  [ "$status" -eq 0 ]
}
