#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
pse_xtask="${PSE_XTASK_BINARY:-$root/target/debug/xtask}"
[[ -x "$pse_xtask" ]] || { echo "solver-rebuild-check: explicitly build xtask first" >&2; exit 125; }
image="$("$pse_xtask" solver-images runtime ci)"
report="$(mktemp -d)"
trap 'rm -rf "$report"' EXIT
docker build --no-cache --target solvers -t pse-solvers:rebuild \
  -f docker/solvers/Dockerfile docker/solvers
checksums() {
  docker run --rm --entrypoint sh "$1" -c \
    'cd /opt/pse-solvers/lib && sha256sum lib*.so* | sort'
}
docker pull --quiet "$image" > /dev/null
checksums "$image" > "$report/published.sha256"
checksums pse-solvers:rebuild > "$report/rebuild.sha256"
diff -u "$report/published.sha256" "$report/rebuild.sha256"
echo 'solvers: rebuilt libraries match the published image byte for byte'
