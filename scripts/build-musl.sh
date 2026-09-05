#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "$0")/.." && pwd)
cd "$repo_root"

target=x86_64-unknown-linux-musl

if [[ -n ${MUSL_CC:-} ]]; then
  musl_cc=$MUSL_CC
elif command -v x86_64-linux-musl-gcc >/dev/null 2>&1; then
  musl_cc=x86_64-linux-musl-gcc
elif command -v musl-gcc >/dev/null 2>&1; then
  musl_cc=musl-gcc
elif command -v cc >/dev/null 2>&1 && [[ $(cc -dumpmachine) == *musl* ]]; then
  # Alpine's native compiler already targets musl.
  musl_cc=cc
else
  echo "error: no x86_64 musl C compiler found" >&2
  echo "install musl-tools, or set MUSL_CC to a musl-targeting compiler" >&2
  exit 1
fi

target_libdir=$(rustc --print target-libdir --target "$target" 2>/dev/null || true)
if [[ -z $target_libdir || ! -d $target_libdir ]]; then
  echo "error: Rust target $target is not installed" >&2
  echo "install it with: rustup target add $target" >&2
  exit 1
fi

# Cargo uses the linker variable for the final executable. The cc crate used by
# libmimalloc-sys reads the second variable while compiling mimalloc itself.
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=$musl_cc
export CC_x86_64_unknown_linux_musl=$musl_cc

cargo build --locked --release --target "$target" "$@"

echo "built target/$target/release/softbrush_ls"
