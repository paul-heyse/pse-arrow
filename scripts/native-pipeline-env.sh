#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
# Source-pinned pipeline libraries share the existing native cache and BLAS provider.
set -euo pipefail
pse_pipeline_root="$(git rev-parse --show-toplevel)"
if [[ -z "${UNO_DIR:-}" ]]; then
    UNO_DIR="$(cd "$pse_pipeline_root" && python3 -m scripts.native_pipeline_cache uno)"
    export UNO_DIR
fi
if [[ -z "${PETSC_DIR:-}" ]]; then
    PETSC_DIR="$(cd "$pse_pipeline_root" && python3 -m scripts.native_pipeline_cache petsc)"
    export PETSC_DIR
fi
export LD_LIBRARY_PATH="$PETSC_DIR/lib:$UNO_DIR/lib:${LD_LIBRARY_PATH:-}"
