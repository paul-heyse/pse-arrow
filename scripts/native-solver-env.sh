#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
# Source this file to compile against the immutable solver image's C interface.
set -euo pipefail
# CI/devcontainer already exports the admitted image's interface and runtime paths.
# An explicit local prefix is also checked by pse-ipopt-sys; do not require nested Docker.
if [[ -n "${IPOPT_DIR:-}" ]]; then
  return 0
fi
pse_solver_root="$(git rev-parse --show-toplevel)"
source "$pse_solver_root/scripts/build-env.sh"
pse_solver_prefix="$(python3 "$pse_solver_root/scripts/native_cache.py" solver)"
export IPOPT_DIR="$pse_solver_prefix"
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="bash $pse_solver_root/scripts/native-solver-runner.sh"
