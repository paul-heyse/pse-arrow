#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
#
# Run a heavy native workload inside its own memory-capped systemd user scope. A runaway
# process is then OOM-killed alone, inside that scope, instead of exhausting the machine
# and taking the editor session and every other agent down with it (two such crashes on
# 2026-09-29: a conformance run grew to 156 GB outside the engine's accounted pool).
#
# PSE_MEMORY_MAX sets the cap (default 120G); PSE_MEMORY_MAX=off runs uncapped. Without a
# systemd user manager the command runs uncapped.
set -euo pipefail
cap="${PSE_MEMORY_MAX:-120G}"
if [[ "$cap" != off ]] && command -v systemd-run >/dev/null 2>&1 \
    && systemctl --user show-environment >/dev/null 2>&1; then
    exec systemd-run --user --scope --quiet --collect \
        -p MemoryMax="$cap" -p MemorySwapMax=0 -- "$@"
fi
exec "$@"
