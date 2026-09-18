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
pse_solver_image="$(python3 "$pse_solver_root/scripts/solver-images.py" ref dev)"
pse_solver_digest="${pse_solver_image##*@sha256:}"
pse_solver_prefix="$pse_solver_root/target/native-solver/$pse_solver_digest"
if [[ ! -f "$pse_solver_prefix/.complete" ]]; then
  mkdir -p "$pse_solver_prefix"
  docker run --rm --entrypoint tar "$pse_solver_image" -C /opt/pse-solvers -cf - . | tar -xf - -C "$pse_solver_prefix"
  touch "$pse_solver_prefix/.complete"
fi
export IPOPT_DIR="$pse_solver_prefix"
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="bash $pse_solver_root/scripts/native-solver-runner.sh"
