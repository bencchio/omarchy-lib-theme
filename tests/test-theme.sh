#!/usr/bin/env bash
# THEME domain suite: formatting, lints and library tests.
# Usage: tests/test-theme.sh
set -euo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

cd "${REPO_ROOT}"

cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --lib
cargo test --test styles
