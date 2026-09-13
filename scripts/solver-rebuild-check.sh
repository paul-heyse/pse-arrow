#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
image="${PSE_SOLVER_IMAGE:-$(python3 scripts/solver-images.py ref ci)}"
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
