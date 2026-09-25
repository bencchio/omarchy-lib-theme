#!/usr/bin/env bash
# Builds and installs the library outside a distribution package — the fast
# path for trying a build before packaging it, or for a system without one
# (see README.md § Installing).
set -euo pipefail

PREFIX="${HOME}/.local"

_usage() {
	cat <<-EOF >&2
	Usage: $(basename "$0") [--prefix <path>]
	  --prefix  install prefix (default: ${HOME}/.local)
	EOF
	exit 1
}

while [[ $# -gt 0 ]]; do
	case "$1" in
		--prefix) shift; PREFIX="${1:-}" ;;
		--prefix=*) PREFIX="${1#--prefix=}" ;;
		-h|--help) _usage ;;
		*) echo "install.sh: unknown option: $1" >&2; _usage ;;
	esac
	shift
done

[[ -n "${PREFIX}" ]] || { echo "install.sh: --prefix cannot be empty" >&2; exit 1; }

for tool in cargo cmake; do
	command -v "${tool}" >/dev/null 2>&1 \
		|| { echo "install.sh: missing required tool '${tool}' — install it first (e.g. pacman -S rust cmake on Arch)" >&2; exit 1; }
done

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cmake -S "${SCRIPT_DIR}" -B "${SCRIPT_DIR}/build" \
	-DCMAKE_INSTALL_PREFIX="${PREFIX}" \
	-DCMAKE_BUILD_TYPE=Release
cmake --build "${SCRIPT_DIR}/build"
cmake --install "${SCRIPT_DIR}/build"

echo "install.sh: installed under ${PREFIX}"
echo "install.sh: if that prefix isn't on your system's default search path, export PKG_CONFIG_PATH=\"${PREFIX}/lib/pkgconfig\" and CMAKE_PREFIX_PATH=\"${PREFIX}\""
