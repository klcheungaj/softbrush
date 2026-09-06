#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "$0")/.." && pwd)
verifier=$repo_root/scripts/verify-musl.sh
test_directory=$(mktemp -d)
trap 'rm -rf "$test_directory"' EXIT

binary=$test_directory/softbrush_ls
tools_directory=$test_directory/tools
symbols=$test_directory/symbols
stdout=$test_directory/stdout
stderr=$test_directory/stderr
mkdir -p "$tools_directory"
touch "$binary"
chmod +x "$binary"

cat >"$tools_directory/readelf" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
case $* in
  *--program-headers*) echo 'Program Headers:' ;;
  *--dynamic*) echo 'There is no dynamic section in this file.' ;;
  *) exit 2 ;;
esac
EOF

cat >"$tools_directory/nm" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
cat "$MOCK_NM_SYMBOLS"
EOF
chmod +x "$tools_directory/readelf" "$tools_directory/nm"

write_complete_mimalloc_symbols() {
  local symbol
  for symbol in \
    mi_malloc mi_free malloc calloc realloc free aligned_alloc posix_memalign \
    memalign malloc_usable_size reallocarray __libc_malloc __libc_calloc \
    __libc_realloc __libc_free; do
    printf '0000000000000000 T %s\n' "$symbol"
  done >"$symbols"
}

run_verifier() {
  PATH="$tools_directory:$PATH" MOCK_NM_SYMBOLS="$symbols" \
    "$verifier" "$binary" >"$stdout" 2>"$stderr"
}

# A fully static binary with every mimalloc override and no musl allocator
# internals must pass.
write_complete_mimalloc_symbols
run_verifier
grep -q 'complete mimalloc override' "$stdout"
test ! -s "$stderr"

# Removing any standard allocation entry point must fail.
grep -v ' T calloc$' "$symbols" >"$symbols.incomplete"
mv "$symbols.incomplete" "$symbols"
if run_verifier; then
  echo "error: verifier accepted a missing mimalloc calloc override" >&2
  exit 1
fi
grep -q "lacks mimalloc's strong allocator override: calloc" "$stderr"

# A musl allocator implementation linked beside mimalloc must fail even when
# all public allocation symbols are present.
write_complete_mimalloc_symbols
printf '0000000000000000 B __malloc_context\n' >>"$symbols"
if run_verifier; then
  echo "error: verifier accepted the musl allocator implementation" >&2
  exit 1
fi
grep -q 'still contains the musl allocator implementation' "$stderr"

echo "verified musl allocator regression checks"
