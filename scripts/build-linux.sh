#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "$0")/.." && pwd)
cd "$repo_root"

usage() {
  cat >&2 <<'EOF'
usage: scripts/build-linux.sh [--musl|--gnu] [--arch x86_64|aarch64] [-- CARGO_ARGS...]

Builds a static musl release by default. --gnu builds an unoptimized GNU
executable for debugging. Arguments after -- are forwarded to cargo build.
EOF
}

libc=musl
profile=release
case $(uname -m) in
  x86_64 | amd64) architecture=x86_64 ;;
  aarch64 | arm64) architecture=aarch64 ;;
  *)
    echo "error: unsupported Linux architecture: $(uname -m)" >&2
    exit 1
    ;;
esac
cargo_arguments=()

while [[ $# -gt 0 ]]; do
  case $1 in
    --musl)
      libc=musl
      profile=release
      shift
      ;;
    --gnu)
      libc=gnu
      profile=debug
      shift
      ;;
    --arch)
      if [[ $# -lt 2 ]]; then
        usage
        exit 2
      fi
      architecture=$2
      shift 2
      ;;
    --)
      shift
      cargo_arguments=("$@")
      break
      ;;
    -h | --help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown argument: $1" >&2
      usage
      exit 2
      ;;
  esac
done

case $architecture in
  x86_64 | aarch64) ;;
  *)
    echo "error: unsupported Linux architecture: $architecture" >&2
    exit 2
    ;;
esac

target=$architecture-unknown-linux-$libc
target_libdir=$(rustc --print target-libdir --target "$target" 2>/dev/null || true)
if [[ -z $target_libdir || ! -d $target_libdir ]]; then
  echo "error: Rust target $target is not installed" >&2
  echo "install it with: rustup target add $target" >&2
  exit 1
fi

if [[ $libc == musl ]]; then
  if [[ -n ${MUSL_CC:-} ]]; then
    musl_cc=$MUSL_CC
  elif command -v "$architecture-linux-musl-gcc" >/dev/null 2>&1; then
    musl_cc=$architecture-linux-musl-gcc
  elif command -v musl-gcc >/dev/null 2>&1; then
    musl_cc=musl-gcc
  elif command -v cc >/dev/null 2>&1 && [[ $(cc -dumpmachine) == *musl* ]]; then
    # Alpine's native compiler already targets musl.
    musl_cc=cc
  else
    echo "error: no $architecture musl C compiler found" >&2
    echo "install musl-tools, build in Alpine, or set MUSL_CC" >&2
    exit 1
  fi

  linker_variable=CARGO_TARGET_${target^^}_LINKER
  linker_variable=${linker_variable//-/_}
  cc_variable=CC_${target//-/_}
  export "$linker_variable=$musl_cc"
  export "$cc_variable=$musl_cc"
fi

profile_arguments=()
if [[ $profile == release ]]; then
  profile_arguments+=(--release)
fi
cargo build --locked "${profile_arguments[@]}" --target "$target" "${cargo_arguments[@]}"

echo "built target/$target/$profile/softbrush_ls"
