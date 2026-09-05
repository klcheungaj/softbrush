#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 BINARY" >&2
  exit 2
fi

binary=$1
if [[ ! -x $binary ]]; then
  echo "error: musl binary is missing or not executable: $binary" >&2
  exit 1
fi

for tool in readelf nm; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "error: required verification tool is unavailable: $tool" >&2
    exit 1
  fi
done

verification_directory=$(mktemp -d)
trap 'rm -rf "$verification_directory"' EXIT

program_headers=$verification_directory/program-headers
dynamic_section=$verification_directory/dynamic-section
symbols=$verification_directory/symbols

readelf --wide --program-headers "$binary" >"$program_headers"
if grep -Eq '^[[:space:]]*INTERP[[:space:]]' "$program_headers"; then
  echo "error: $binary contains a PT_INTERP program header" >&2
  exit 1
fi

readelf --wide --dynamic "$binary" >"$dynamic_section"
if grep -Eq '\(NEEDED\)' "$dynamic_section"; then
  echo "error: $binary contains DT_NEEDED dynamic dependencies" >&2
  exit 1
fi

# Capture nm output before searching it: nm can otherwise receive SIGPIPE when
# grep exits after the first match, making a successful pipeline fail under
# pipefail.
nm "$binary" >"$symbols"
if ! grep -Eq '[[:space:]][[:alpha:]][[:space:]]mi_free$' "$symbols"; then
  echo "error: $binary does not expose the expected mimalloc symbol mi_free" >&2
  exit 1
fi

echo "verified static musl executable with mimalloc: $binary"
