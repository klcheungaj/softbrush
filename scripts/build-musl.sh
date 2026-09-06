#!/usr/bin/env bash
set -euo pipefail

script_directory=$(cd "$(dirname "$0")" && pwd)
exec "$script_directory/build-linux.sh" --musl -- "$@"
