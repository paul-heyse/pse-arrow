#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
#
# Fetch the pinned reading copies into external/ (gitignored). They are never vendored:
# a submodule would pin hundreds of MB onto every checkout and CI job, and the parity
# harness installs idaes-pse from PyPI anyway. Tags come from the pins, not from here:
#   arrow-rs / datafusion  <- Cargo.lock (the resolved `arrow` and `datafusion` versions)
#   idaes-pse              <- pyproject.toml [dependency-groups].parity (idaes-pse==X)
# tests/governance and the adr-lint job check that this script and the pins agree.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
mkdir -p external

lock_version() {
  # First `[[package]]` named $1 in Cargo.lock.
  awk -v name="$1" '
    $0 == "[[package]]" { inpkg = 1; next }
    inpkg && $1 == "name" && $3 == "\"" name "\"" { found = 1; next }
    inpkg && found && $1 == "version" { gsub(/"/, "", $3); print $3; exit }
    /^$/ { inpkg = 0; found = 0 }
  ' Cargo.lock
}

ARROW_TAG="${ARROW_TAG:-$(lock_version arrow)}"
DATAFUSION_TAG="${DATAFUSION_TAG:-$(lock_version datafusion)}"
IDAES_TAG="${IDAES_TAG:-$(grep -oE 'idaes-pse==[0-9.]+' pyproject.toml | head -n1 | cut -d= -f3)}"

# `--print` reports the tags the pins resolve to without touching the network; the
# governance workflow compares the IDAES one against the parity pin.
if [ "${1:-}" = "--print" ]; then
  printf 'ARROW_TAG=%s\nDATAFUSION_TAG=%s\nIDAES_TAG=%s\n' "$ARROW_TAG" "$DATAFUSION_TAG" "$IDAES_TAG"
  exit 0
fi

fetch() { # name url tag
  local name="$1" url="$2" tag="$3" dir="external/$1"
  if [ -d "$dir/.git" ]; then
    local have
    have="$(git -C "$dir" describe --tags --exact-match 2>/dev/null || echo none)"
    if [ "$have" = "$tag" ]; then echo "external/$name already at $tag"; return; fi
    echo "external/$name is at $have, want $tag: refetching"; rm -rf "$dir"
  fi
  echo "external/$name <- $url @ $tag"
  git clone --quiet --depth 1 --branch "$tag" "$url" "$dir"
}

fetch arrow-rs   https://github.com/apache/arrow-rs.git   "$ARROW_TAG"
fetch datafusion https://github.com/apache/datafusion.git "$DATAFUSION_TAG"
fetch idaes-pse  https://github.com/IDAES/idaes-pse.git   "$IDAES_TAG"
echo "done: arrow-rs $ARROW_TAG, datafusion $DATAFUSION_TAG, idaes-pse $IDAES_TAG"
