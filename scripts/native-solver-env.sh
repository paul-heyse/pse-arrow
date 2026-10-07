#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
# Source this file to compile against the immutable solver image's C interface.
# The image is the pinned dev image, or the immutable local override
# PSE_SOLVER_IMAGE (docker/solvers/README.md, "Local images").
# Manual exports only: outside an admitted operation, every use gets full
# verification and managed generations remain conservatively pinned. Prefer
# `scripts/pse-env --native` when setup and command lifetime must be scoped together.
set -euo pipefail
pse_solver_root="$(git rev-parse --show-toplevel)"
eval "$(python3 "$pse_solver_root/scripts/pse_env.py" --print)"
pse_solver_exports="$("${PSE_NATIVE_SETUP_PYTHON:-$pse_solver_root/.venv/bin/python}" "$pse_solver_root/scripts/native_operation.py" --capabilities solver --shell)"
eval "$pse_solver_exports"
unset pse_solver_exports
