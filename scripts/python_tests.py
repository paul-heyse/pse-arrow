# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Run the selected Python partition in its declared process allocation."""

from __future__ import annotations

import os
import sys
from pathlib import Path

from scripts import native_tests, surreal_server
from scripts.test_run import run_python

ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    extra = sys.argv[1:]
    managed = "--managed-primary-route" in extra
    child = "--managed-primary-child" in extra
    functional_child = "--functional-observer-child" in extra
    if child and not managed:
        raise ValueError("managed primary child requires its declared route")
    if managed and not child:
        state = os.environ.get("PSE_SURREAL_STATE")
        if not state:
            raise ValueError("managed primary route requires PSE_SURREAL_STATE")

        worker = native_tests.worker_binary([], os.environ)
        with native_tests.worker_binding(worker):
            status = surreal_server.observer(
                surreal_server.reference_state(Path(state)),
                [
                    sys.executable,
                    "-m",
                    "scripts.python_tests",
                    *extra,
                    "--managed-primary-child",
                ],
                profile="reference",
            )
    elif (
        not managed
        and not functional_child
        and "--unit-only" not in extra
        and os.environ.get("PSE_NATIVE_OPERATION")
    ):
        state = Path(os.environ["PSE_SURREAL_STATE"])
        status = surreal_server.observer(
            state,
            [
                sys.executable,
                "-m",
                "scripts.python_tests",
                *extra,
                "--functional-observer-child",
            ],
            profile="exclusive-observer",
        )
    else:
        extra = [
            argument
            for argument in extra
            if argument
            not in {
                "--managed-primary-route",
                "--managed-primary-child",
                "--functional-observer-child",
            }
        ]
        # The existing selection owner consumes the last user marker expression,
        # then intersects it with this process's mandatory resource partition.
        selected = (
            "unit or component or integration" if managed else "unit or component"
        )
        if "--unit-only" in extra:
            selected = "unit"
            extra.remove("--unit-only")
        command = native_tests.python_command(["-m", selected, *extra], managed=managed)

        status = run_python(command)
    # Match shell signal exit semantics rather than SystemExit's 256-N conversion.
    return 128 - status if status < 0 else status


if __name__ == "__main__":
    raise SystemExit(main())
