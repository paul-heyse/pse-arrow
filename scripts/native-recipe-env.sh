#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
# Source once before any setup in a shebang recipe; arguments declare its roots.
pse_recipe_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
pse_recipe_capabilities="$1"
if ! "${PSE_NATIVE_SETUP_PYTHON:-$pse_recipe_root/.venv/bin/python}" "$pse_recipe_root/scripts/native_operation.py" \
    --active --capabilities "$pse_recipe_capabilities"; then
    exec env PSE_NATIVE_CAPABILITIES="$pse_recipe_capabilities" \
        bash "$pse_recipe_root/scripts/native_exec.sh" bash "$0" "${@:2}"
fi
unset pse_recipe_root pse_recipe_capabilities
