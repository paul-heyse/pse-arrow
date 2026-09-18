#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
# Compile with the exact container C interface; execute in that container's runtime.
set -euo pipefail
root="$(git rev-parse --show-toplevel)"
cd "$root"
source "$root/scripts/native-solver-env.sh"
exec cargo nextest run --locked -p pse-backend-native \
  --features ipopt,pse-relations/force-validate "$@"
