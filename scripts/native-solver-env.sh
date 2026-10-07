#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
# Source this file to compile against the immutable solver image's C interface.
# The image is the pinned dev image, or the immutable local override
# PSE_SOLVER_IMAGE (docker/solvers/README.md, "Local images").
# Manual exports only: outside an admitted operation, every use gets full
# verification and managed generations remain conservatively pinned. Prefer
# native_exec.sh when setup and command lifetime must be scoped together.
set -euo pipefail
pse_solver_root="$(git rev-parse --show-toplevel)"
source "$pse_solver_root/scripts/build-env.sh"
pse_solver_exports="$("${PSE_NATIVE_SETUP_PYTHON:-$pse_solver_root/.venv/bin/python}" "$pse_solver_root/scripts/native_operation.py" --capabilities solver --shell)"
eval "$pse_solver_exports"
unset pse_solver_exports
