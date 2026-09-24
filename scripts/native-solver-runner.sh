#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
# Run native test binaries with the pinned Ipopt/MUMPS/netlib runtime.
set -euo pipefail
root="$(git rev-parse --show-toplevel)"
cd "$root"
image="$(python3 scripts/solver-images.py ref dev)"
mounts=(-v "$root:$root:ro")
target="${CARGO_TARGET_DIR:-$root/target}"
target="$(realpath "$target")"
case "$target/" in
  "$root/"*) ;;
  *) mounts+=(-v "$target:$target:ro") ;;
esac
exec docker run --rm --network none --user "$(id -u):$(id -g)" \
  -e PSE_SOLVER_MEASURE -e SYMBOLICA_LICENSE \
  "${mounts[@]}" -w "$root" "$image" "$@"
