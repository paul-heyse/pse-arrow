#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
set -euo pipefail
source "$(git rev-parse --show-toplevel)/scripts/native-execution-env.sh"
exec "$@"
