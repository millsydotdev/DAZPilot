#!/usr/bin/env bash
# Collects installable release files from merged CI artifacts into a flat directory.
set -euo pipefail

SOURCE="${1:-all-artifacts}"
DEST="${2:-release-staging}"

rm -rf "$DEST"
mkdir -p "$DEST"

# Target triple from a Tauri artifact path (…/target/<triple>/release/bundle/…).
target_triple_from_path() {
  local path="$1"
  if [[ "$path" =~ /target/([^/]+)/release/ ]]; then
    echo "${BASH_REMATCH[1]}"
  fi
}

unique_name() {
  local file="$1"
  local base
  base="$(basename "$file")"
  local triple
  triple="$(target_triple_from_path "$file")"

  if [[ -z "$triple" ]]; then
    echo "$base"
    return
  fi

  case "$base" in
    DAZPilot.app.tar.gz)
      echo "DAZPilot_${triple}.app.tar.gz"
      ;;
    DAZPilot.app.tar.gz.sig)
      echo "DAZPilot_${triple}.app.tar.gz.sig"
      ;;
    DAZPilot.dmg)
      echo "DAZPilot_${triple}.dmg"
      ;;
    DAZPilot.dmg.sig)
      echo "DAZPilot_${triple}.dmg.sig"
      ;;
    *)
      if [[ "$base" == *"${triple}"* ]]; then
        echo "$base"
      else
        local stem="${base%.*}"
        local ext="${base##*.}"
        if [[ "$base" == *.*.* ]]; then
          local mid="${base#*.}"
          mid="${mid%.*}"
          stem="${base%%.${mid}.${ext}}"
          echo "${stem}_${triple}.${mid}.${ext}"
        else
          echo "${stem}_${triple}.${ext}"
        fi
      fi
      ;;
  esac
}

while IFS= read -r -d '' file; do
  name="$(unique_name "$file")"
  dest="$DEST/$name"
  if [[ -f "$dest" ]]; then
    echo "ERROR: duplicate staged asset: $name" >&2
    echo "  from: $file" >&2
    echo "  existing source already copied" >&2
    exit 1
  fi
  cp -f "$file" "$dest"
  echo "Staged: $name"
done < <(
  find "$SOURCE" -type f \
    -path '*/release/bundle/*' \
    ! -name '*.d' \
    ! -name '*.rlib' \
    ! -name '*.pdb' \
    -print0
)

count="$(find "$DEST" -type f | wc -l | tr -d ' ')"
if [[ "$count" -eq 0 ]]; then
  echo "ERROR: no release files staged from $SOURCE" >&2
  exit 1
fi

echo "Staged $count file(s) in $DEST"
