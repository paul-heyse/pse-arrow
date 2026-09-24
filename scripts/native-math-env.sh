#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
# Source before the selected native solver feature build. Build upstream vendored
# KLU explicitly: suitesparse_sys 0.1.4 exports DEP paths before its vendor build
# and otherwise enables CHOLMOD/CUDA. No upstream source is modified.
set -euo pipefail
pse_math_root="$(git rev-parse --show-toplevel)"
source "$pse_math_root/scripts/build-env.sh"
pse_native_exports="$(python3 "$pse_math_root/scripts/native_cache.py" compiler-shell)"
eval "$pse_native_exports"
unset pse_native_exports
pse_klu_prefix="$(python3 "$pse_math_root/scripts/native_cache.py" klu)"
export SUITESPARSE_INCLUDE_DIR="$pse_klu_prefix/include/suitesparse"
export SUITESPARSE_LIBRARY_DIR="$pse_klu_prefix/lib"
