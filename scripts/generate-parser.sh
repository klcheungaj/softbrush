#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "$0")/.." && pwd)
cd "$repo_root"

output_directory=$repo_root/src/generated
check_generated=false

if [[ $# -gt 1 ]]; then
  echo "usage: $0 [--check]" >&2
  exit 2
fi

case ${1:-} in
  "") ;;
  --check)
    check_generated=true
    output_directory=$(mktemp -d)
    trap 'rm -rf "$output_directory"' EXIT
    # mod.rs is project-owned; all other files must be reproduced by codegen.
    cp "$repo_root/src/generated/mod.rs" "$output_directory/mod.rs"
    ;;
  *)
    echo "usage: $0 [--check]" >&2
    exit 2
    ;;
esac

cargo run --quiet \
  --locked \
  --manifest-path "$repo_root/tools/parser-generator/Cargo.toml" \
  -- "$repo_root/grammar/Tcl.g4" "$output_directory"

if [[ $check_generated == true ]]; then
  if diff --recursive --unified "$repo_root/src/generated" "$output_directory"; then
    echo "generated parser is current"
  else
    status=$?
    if [[ $status -eq 1 ]]; then
      echo "error: generated parser differs from grammar/Tcl.g4" >&2
      echo "run scripts/generate-parser.sh and commit the result" >&2
    fi
    exit "$status"
  fi
fi
