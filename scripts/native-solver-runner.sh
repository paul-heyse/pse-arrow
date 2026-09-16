#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
# Run native test binaries with the pinned Ipopt/MUMPS/netlib runtime.
set -euo pipefail
root="$(git rev-parse --show-toplevel)"
cd "$root"
image="$(python3 scripts/solver-images.py ref dev)"
exec docker run --rm --network none --user "$(id -u):$(id -g)" \
  -v "$root:$root:ro" -w "$root" "$image" "$@"
