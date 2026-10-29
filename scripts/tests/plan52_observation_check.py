# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit bounded kernel pressure consumer controls; no global stress or full suite.

Run with PSE_PLAN52_OBSERVATION_RECEIPTS set to a fresh absolute directory through
scripts/pse-env --resource-class light -- .venv/bin/python -m unittest
scripts.tests.plan52_observation_check. Receipts survive failures.
"""

# ruff: noqa: PT009 -- stdlib setup controls
from __future__ import annotations

import json
import os
import subprocess
import sys
import time
import unittest
import uuid
from pathlib import Path

import msgspec

from scripts import (
    activity_watch as watch,
)
from scripts import (
    host_admission,
    operation_lifetime,
)
from scripts import (
    pressure_observation as pressure,
)

ROOT = Path(__file__).resolve().parents[2]


def wait_file(path: Path, deadline: float) -> None:
    while not path.exists():
        if time.monotonic() >= deadline:
            raise TimeoutError(f"bounded control did not observe {path}")
        time.sleep(0.02)


def worker(mode: str, directory: Path) -> int:
    deadline = time.monotonic() + 15
    if mode == "pressure":
        operation_lifetime.write_json(
            directory / "ready.json",
            {
                "group": operation_lifetime.process_group(os.getpid()),
                "pid": os.getpid(),
            },
        )
        wait_file(directory / "cpu-start", deadline)
        stop = time.monotonic() + 1.5
        value = 1
        while time.monotonic() < stop:
            value = (value * 7 + 1) % 999983
        (directory / "cpu-finished").write_text(str(value))
        wait_file(directory / "release", deadline)
        return 0
    raise ValueError("unknown pressure worker mode")


class ObservationControls(unittest.TestCase):
    def setUp(self) -> None:
        selected = os.environ.get("PSE_PLAN52_OBSERVATION_RECEIPTS")
        if selected is None:
            self.fail("set a fresh PSE_PLAN52_OBSERVATION_RECEIPTS directory")
        self.directory = Path(selected) / self._testMethodName
        self.assertTrue(self.directory.is_absolute())
        self.directory.mkdir(parents=True, exist_ok=False)

    def launch(self, mode: str) -> list[str]:
        return [
            sys.executable,
            "-m",
            "scripts.tests.plan52_observation_check",
            "--worker",
            mode,
            str(self.directory),
        ]

    def test_actual_idle_and_finite_cpu_quota(self) -> None:
        allocation = host_admission.inherit(os.environ)
        if allocation is None:
            self.fail("bounded control requires an outer admitted scope")
        unit = "pse-pressure-control-" + uuid.uuid4().hex + ".scope"
        command = [
            "systemd-run",
            "--user",
            "--scope",
            "--quiet",
            "--collect",
            "--no-ask-password",
            "--expand-environment=no",
            "--unit=" + unit,
            "--slice=" + host_admission.allocation_slice(allocation),
            "--property=CPUQuota=10%",
            "--property=MemoryMax=" + str(min(allocation.profile.memory, 256 << 20)),
            "--property=MemorySwapMax=0",
            "--",
            *self.launch("pressure"),
        ]
        with (
            (self.directory / "worker.log").open("w") as log,
            subprocess.Popen(
                command,
                cwd=ROOT,
                stdout=log,
                stderr=subprocess.STDOUT,
                env=operation_lifetime.manager_control_environment(),
            ) as child,
        ):
            try:
                wait_file(self.directory / "ready.json", time.monotonic() + 5)
                group = json.loads((self.directory / "ready.json").read_text())["group"]
                first = pressure.capture(group)
                time.sleep(0.35)
                idle = pressure.capture(group)
                (self.directory / "cpu-start").touch()
                wait_file(self.directory / "cpu-finished", time.monotonic() + 5)
                busy = pressure.capture(group)
                idle_delta = pressure.delta(first.selected, idle.selected)
                busy_delta = pressure.delta(idle.selected, busy.selected)
                operation_lifetime.write_json(
                    self.directory / "pressure.json",
                    {
                        "unit": unit,
                        "command": command,
                        "first": msgspec.to_builtins(first),
                        "idle": msgspec.to_builtins(idle),
                        "busy": msgspec.to_builtins(busy),
                        "idle_delta": idle_delta,
                        "busy_delta": busy_delta,
                        "interpretation": "selected finite quota control only; host PSI has concurrent unrelated work; no general causal inference",
                    },
                )
                self.assertEqual(
                    busy.selected.fields["cpu.max"].values["raw"], "10000 100000"
                )
                self.assertGreater(
                    busy_delta["fields"]["cpu.stat"]["nr_throttled"]["delta"], 0
                )
                self.assertGreater(
                    busy_delta["fields"]["cpu.stat"]["throttled_usec"]["delta"], 0
                )
                self.assertGreater(
                    busy_delta["fields"]["cpu.pressure"]["some.total"]["delta"], 0
                )
                self.assertLess(
                    idle_delta["fields"]["cpu.stat"]["usage_usec"]["per_second"], 50000
                )
                self.assertGreater(
                    busy_delta["fields"]["cpu.stat"]["usage_usec"]["per_second"],
                    idle_delta["fields"]["cpu.stat"]["usage_usec"]["per_second"],
                )
                self.assertEqual(
                    busy.selected.fields["direct_member_affinity"].status, "available"
                )
                self.assertTrue(busy.selected.ancestors)
            finally:
                (self.directory / "release").touch()
                try:
                    child.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    child.terminate()
                    child.wait(timeout=3)
            self.assertEqual(child.returncode, 0)
        gone = watch.snapshot({"kind": "unit", "handle": unit}, time.monotonic() + 1)
        self.assertEqual(gone["state"], "observation-unavailable")


if __name__ == "__main__":
    if len(sys.argv) == 4 and sys.argv[1] == "--worker":
        raise SystemExit(worker(sys.argv[2], Path(sys.argv[3])))
    unittest.main()
