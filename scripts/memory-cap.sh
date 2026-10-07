#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
#
# Run a heavy native workload inside its own memory-capped systemd user scope. A runaway
# process is then OOM-killed alone, inside that scope, instead of exhausting the machine
# and taking the editor session and every other agent down with it (two such crashes on
# 2026-09-29: a conformance run grew to 156 GB outside the engine's accounted pool).
#
# PSE_MEMORY_MAX sets the cap (default 120G); off preserves scope ownership without
# a memory limit. Without a systemd user manager, conservative generation pins remain.
set -euo pipefail
cap="${PSE_MEMORY_MAX:-120G}"
pse_cap_runtime="/run/user/$(id -u)"
if [[ -S "$pse_cap_runtime/bus" ]]; then
    export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-$pse_cap_runtime}"
    export DBUS_SESSION_BUS_ADDRESS="${DBUS_SESSION_BUS_ADDRESS:-unix:path=$pse_cap_runtime/bus}"
fi
pse_cap_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
pse_cap_python="${PSE_NATIVE_SETUP_PYTHON:-$pse_cap_root/.venv/bin/python}"
if [[ -x "$pse_cap_python" ]] && "$pse_cap_python" "$pse_cap_root/scripts/native_operation.py" --inside; then
    exec "$@"
fi
if command -v systemd-run >/dev/null 2>&1 \
    && systemctl --user show-environment >/dev/null 2>&1; then
    pse_cap_unit="pse-native-$(tr -d '-' </proc/sys/kernel/random/uuid).scope"
    pse_cap_properties=()
    if [[ "$cap" != off ]]; then
        pse_cap_properties=(-p MemoryMax="$cap" -p MemorySwapMax=0)
    fi
    exec systemd-run --user --scope --quiet --collect --unit="$pse_cap_unit" \
        "${pse_cap_properties[@]}" -- "$@"
fi
exec "$@"
