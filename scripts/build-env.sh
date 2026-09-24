#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
# Source from direnv, recipe shells and native build entry points.
pse_build_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
pse_build_exports="$(python3 "$pse_build_root/scripts/build_environment.py" --shell)" || return
eval "$pse_build_exports"
unset pse_build_exports pse_build_root
