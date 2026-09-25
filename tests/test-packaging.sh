#!/usr/bin/env bash
# Packaging suite: the release script refuses a tag that isn't a plain version.
# Usage: tests/test-packaging.sh
set -euo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

readonly SCRIPT="${REPO_ROOT}/packaging/github-release.sh"

_check() {
	local tag="$1" out
	if out="$("${SCRIPT}" "${tag}" 2>&1)"; then
		echo "test-packaging: expected a refusal for '${tag}', got exit 0" >&2
		exit 1
	fi
	[[ "${out}" == *"X.Y.Z"* ]] || {
		echo "test-packaging: the refusal for '${tag}' must name the expected form, got: ${out}" >&2
		exit 1
	}
}

_check '0.3.0;id'
_check '0.3.0$(id)'
_check '0.3.0$(touch${IFS}x)'

echo "test-packaging: OK"
