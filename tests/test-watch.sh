#!/usr/bin/env bash
# Watcher suite: the theme file changing on disk, against a temporary directory.
# Usage: tests/test-watch.sh
set -euo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

cd "${REPO_ROOT}"

cargo test --test watch
