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

# The global Rust allocator only redirects Rust allocations. These strong C
# entry points prove that mimalloc's override layer also owns allocations made
# by musl and native dependencies.
mimalloc_symbols=(
  mi_malloc
  mi_free
  malloc
  calloc
  realloc
  free
  aligned_alloc
  posix_memalign
  memalign
  malloc_usable_size
  reallocarray
  __libc_malloc
  __libc_calloc
  __libc_realloc
  __libc_free
)
for symbol in "${mimalloc_symbols[@]}"; do
  if ! grep -Eq "[[:space:]]T[[:space:]]$symbol$" "$symbols"; then
    echo "error: $binary lacks mimalloc's strong allocator override: $symbol" >&2
    exit 1
  fi
done

# These symbols are implementation details of musl's mallocng allocator. Their
# presence means musl's allocator objects were linked alongside mimalloc.
musl_allocator_pattern='(__libc_malloc_impl|__malloc_alloc_meta|__malloc_context|__malloc_lock|__malloc_replaced|__malloc_size_classes)$'
if grep -Eq "[[:space:]][[:alpha:]][[:space:]]$musl_allocator_pattern" "$symbols"; then
  echo "error: $binary still contains the musl allocator implementation" >&2
  exit 1
fi

echo "verified static musl executable with complete mimalloc override: $binary"
