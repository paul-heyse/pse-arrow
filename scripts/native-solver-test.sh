#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
# Compile with the exact container C interface; execute in that container's runtime.
set -euo pipefail
root="$(git rev-parse --show-toplevel)"
cd "$root"
image="$(python3 scripts/solver-images.py ref dev)"
digest="${image##*@sha256:}"
prefix="$root/target/native-solver/$digest"
if [[ ! -f "$prefix/.complete" ]]; then
  mkdir -p "$prefix"
  docker run --rm --entrypoint tar "$image" -C /opt/pse-solvers -cf - . | tar -xf - -C "$prefix"
  touch "$prefix/.complete"
fi
export IPOPT_DIR="$prefix"
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="bash $root/scripts/native-solver-runner.sh"
exec cargo nextest run --locked -p pse-backend-native \
  --features ipopt,pse-relations/force-validate "$@"
