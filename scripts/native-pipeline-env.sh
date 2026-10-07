#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
# Source-pinned pipeline libraries share the existing native cache and BLAS provider.
# Manual exports only: sourcing creates no operation. Outside an admitted owner,
# every use gets full verification and managed generations remain pinned;
# `scripts/pse-env --native` owns setup and command lifetime for ordinary recipes.
set -euo pipefail
pse_pipeline_root="$(git rev-parse --show-toplevel)"
eval "$(python3 "$pse_pipeline_root/scripts/pse_env.py" --print)"
pse_pipeline_exports="$("${PSE_NATIVE_SETUP_PYTHON:-$pse_pipeline_root/.venv/bin/python}" "$pse_pipeline_root/scripts/native_operation.py" --capabilities uno,petsc --shell)"
eval "$pse_pipeline_exports"
unset pse_pipeline_exports
