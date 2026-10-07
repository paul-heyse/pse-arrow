#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
# Shared linked development environment for builds, tests and Python contracts.
# Manual exports only: no operation is created by sourcing this file. Outside an
# admitted operation, assets are fully checked and generations stay pinned;
# native_exec.sh owns setup and command lifetime for ordinary recipes.
set -euo pipefail
pse_native_root="$(git rev-parse --show-toplevel)"
if [[ -f "$pse_native_root/.envrc.local" ]]; then
    # Optional checkout-local overrides are not committed.
    # shellcheck source=/dev/null
    source "$pse_native_root/.envrc.local"
fi
source "$pse_native_root/scripts/build-env.sh"
pse_execution_exports="$("${PSE_NATIVE_SETUP_PYTHON:-$pse_native_root/.venv/bin/python}" "$pse_native_root/scripts/native_operation.py" --capabilities solver,klu,isolation,uno,petsc --shell)"
eval "$pse_execution_exports"
unset pse_execution_exports
