#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
#
# PostToolUse on Edit|Write: format the file that was just edited.
#
# Uses the pinned tools from .venv and rust-toolchain.toml, so what happens here is
# exactly what `just fmt-check` and CI check. Formatting-only failures stop existing.
#
# Never fails the tool call: a missing or unhappy formatter is not a reason to lose
# an edit. The baseline is zero, but the gate is `just fmt-check`, not this hook.
set -uo pipefail

root="${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
cd "$root" 2>/dev/null || exit 0

file="$(python3 -c 'import json,sys; print(json.load(sys.stdin).get("tool_input",{}).get("file_path",""))' 2>/dev/null)"
[ -n "$file" ] && [ -f "$file" ] || exit 0

rel="${file#"$root"/}"
rel="${rel#./}"

case "$rel" in
  # Generated trees carry `#![rustfmt::skip]` and are excluded from ruff's formatter;
  # reformatting them here would produce a spurious `just codegen-check` diff.
  docs/generated/*|crates/*/src/generated/*|python/pse/contracts/*|crates/pse-ipopt-sys/src/bindings.rs)
    exit 0
    ;;
  target/*|build/*|external/*)
    exit 0
    ;;
esac

case "$rel" in
  *.rs)
    cargo fmt -- "$file" >/dev/null 2>&1
    ;;
  *.py)
    [ -x .venv/bin/ruff ] && .venv/bin/ruff format -q "$file" >/dev/null 2>&1
    ;;
  *.toml)
    [ -x .venv/bin/taplo ] && .venv/bin/taplo fmt "$file" >/dev/null 2>&1
    ;;
esac
exit 0
