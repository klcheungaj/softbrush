#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 BINARY" >&2
  exit 2
fi

binary=$1
if [[ ! -x $binary ]]; then
  echo "error: macOS binary is missing or not executable: $binary" >&2
  exit 1
fi

for tool in file otool nm; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "error: required verification tool is unavailable: $tool" >&2
    exit 1
  fi
done

if ! file "$binary" | grep -q 'Mach-O 64-bit executable arm64'; then
  echo "error: $binary is not a macOS arm64 executable" >&2
  exit 1
fi

dependencies=$(otool -L "$binary" | tail -n +2 | awk '{print $1}')
while IFS= read -r dependency; do
  [[ -z $dependency ]] && continue
  case $dependency in
    /usr/lib/* | /System/Library/*) ;;
    *)
      echo "error: $binary has a non-system dynamic dependency: $dependency" >&2
      exit 1
      ;;
  esac
done <<<"$dependencies"

verification_directory=$(mktemp -d)
trap 'rm -rf "$verification_directory"' EXIT
nm "$binary" >"$verification_directory/symbols"
if ! grep -Eq '[[:space:]]_mi_free$' "$verification_directory/symbols"; then
  echo "error: $binary does not expose the expected mimalloc symbol _mi_free" >&2
  exit 1
fi

echo "verified macOS arm64 executable with only system dynamic libraries: $binary"
