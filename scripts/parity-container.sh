#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
# Keep Linux container environments separate from the host environment.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
image="${PSE_SOLVER_IMAGE:-$(python3 scripts/solver-images.py ref dev)}"
docker run --rm --user "$(id -u):$(id -g)" \
  -v "$root:/work" -w /work \
  -e UV_PROJECT_ENVIRONMENT=/work/.venv-parity-container \
  -e UV_CACHE_DIR=/work/target/parity-container/uv-cache \
  -e CARGO_HOME=/work/target/parity-container/cargo-home \
  -e CARGO_TARGET_DIR=/work/target/parity-container \
  "$image" bash -c 'set -euo pipefail
    uv sync --locked --group parity --python 3.13
    uv run --no-sync --python 3.13 pytest --parity "$@"' bash "$@"
