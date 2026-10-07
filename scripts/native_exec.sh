#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
set -euo pipefail
pse_exec_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec bash "$pse_exec_root/scripts/memory-cap.sh" bash -c '
    pse_exec_root="$1"; pse_exec_capabilities="$2"; shift 2
    if [[ -f "$pse_exec_root/.envrc.local" ]]; then
        source "$pse_exec_root/.envrc.local"
    fi
    exec "${PSE_NATIVE_SETUP_PYTHON:-$pse_exec_root/.venv/bin/python}" "$pse_exec_root/scripts/native_operation.py" \
        --capabilities "$pse_exec_capabilities" -- "$@"
' native-exec "$pse_exec_root" "${PSE_NATIVE_CAPABILITIES-solver,klu,isolation,uno,petsc}" "$@"
