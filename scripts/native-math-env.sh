#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
# Source before the selected native solver feature build. Build upstream vendored
# KLU explicitly: suitesparse_sys 0.1.4 exports DEP paths before its vendor build
# and otherwise enables CHOLMOD/CUDA. No upstream source is modified.
# Manual exports only: outside an admitted operation, each use is fully checked
# and owned generations remain conservatively pinned. Use native_exec.sh to own
# setup and command lifetime; pipeline libraries have their separate helper.
set -euo pipefail
pse_math_root="$(git rev-parse --show-toplevel)"
source "$pse_math_root/scripts/build-env.sh"
pse_native_exports="$("${PSE_NATIVE_SETUP_PYTHON:-$pse_math_root/.venv/bin/python}" "$pse_math_root/scripts/native_operation.py" --capabilities klu,isolation --shell)"
eval "$pse_native_exports"
unset pse_native_exports
