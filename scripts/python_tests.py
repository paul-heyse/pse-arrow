# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Run the selected Python partition in its declared process allocation."""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

from scripts import native_tests

ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    extra = sys.argv[1:]
    managed = "--managed-primary-route" in extra
    child = "--managed-primary-child" in extra
    if child and not managed:
        raise ValueError("managed primary child requires its declared route")
    if managed and not child:
        state = os.environ.get("PSE_SURREAL_STATE")
        if not state:
            raise ValueError("managed primary route requires PSE_SURREAL_STATE")
        from scripts import (  # noqa: PLC0415 -- selected managed placement owner
            surreal_server,
        )

        status = surreal_server.observer(
            Path(state),
            [
                sys.executable,
                "-m",
                "scripts.python_tests",
                *extra,
                "--managed-primary-child",
            ],
        )
    else:
        extra = [
            argument
            for argument in extra
            if argument
            not in {
                "--managed-primary-route",
                "--managed-primary-child",
            }
        ]
        # The existing selection owner consumes the last user marker expression,
        # then intersects it with this process's mandatory resource partition.
        selected = (
            "unit or component or integration" if managed else "unit or component"
        )
        command = native_tests.python_command(["-m", selected, *extra], managed=managed)
        status = subprocess.call(command, cwd=ROOT)
    # Match shell signal exit semantics rather than SystemExit's 256-N conversion.
    return 128 - status if status < 0 else status


if __name__ == "__main__":
    raise SystemExit(main())
