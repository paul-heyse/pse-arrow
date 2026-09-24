#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
set -euo pipefail
# shellcheck source=scripts/build-env.sh
source "$(dirname "${BASH_SOURCE[0]}")/build-env.sh"
exec bash "$@"
