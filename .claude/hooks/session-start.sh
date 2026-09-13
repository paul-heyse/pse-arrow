#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
#
# SessionStart: put the working copy's actual state in front of the agent.
#
# An agent that opens with `just test` in a tree with no .venv and no solver
# libraries burns a minute and gets an error it then has to diagnose. Two seconds
# here avoids that, and doctor.py is stdlib-only so it runs before bootstrap.
set -euo pipefail
cd "${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel)}"

status="$(python3 scripts/doctor.py --format=text 2>&1 || true)"

# Hooks speak to the model through hookSpecificOutput.additionalContext.
python3 - "$status" <<'PY'
import json
import sys

print(
    json.dumps(
        {
            "hookSpecificOutput": {
                "hookEventName": "SessionStart",
                "additionalContext": (
                    "Environment status (scripts/doctor.py):\n"
                    + sys.argv[1]
                    + "\n\nFix anything blocking with `just bootstrap` before running a gate."
                ),
            }
        }
    )
)
PY
