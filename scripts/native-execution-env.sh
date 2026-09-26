#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
# Shared linked development environment for builds, tests and Python contracts.
set -euo pipefail
pse_native_root="$(git rev-parse --show-toplevel)"
source "$pse_native_root/scripts/build-env.sh"
source "$pse_native_root/scripts/native-solver-env.sh"
source "$pse_native_root/scripts/native-math-env.sh"
if [[ -f "$pse_native_root/.envrc.local" ]]; then
    # Optional checkout-local overrides are not committed.
    # shellcheck source=/dev/null
    source "$pse_native_root/.envrc.local"
fi
unset CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER
export LD_LIBRARY_PATH="$IPOPT_DIR/lib:${LD_LIBRARY_PATH:-}"
export OMP_NUM_THREADS=1 OPENBLAS_NUM_THREADS=1 MKL_NUM_THREADS=1
