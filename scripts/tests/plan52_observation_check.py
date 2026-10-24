# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit bounded kernel/build consumer controls; no global stress or full suite.

Run with PSE_PLAN52_OBSERVATION_RECEIPTS set to a fresh absolute directory through
scripts/pse-env --resource-class light -- .venv/bin/python -m unittest
scripts.tests.plan52_observation_check. Receipts survive failures.
"""

# ruff: noqa: PT009 -- stdlib setup controls
from __future__ import annotations

import json
import os
import signal
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
    build_observation,
    host_admission,
    native_operation,
    test_resources,
    test_run,
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
        native_operation.write_json(
            directory / "ready.json",
            {"group": native_operation.process_group(os.getpid()), "pid": os.getpid()},
        )
        wait_file(directory / "cpu-start", deadline)
        stop = time.monotonic() + 1.5
        value = 1
        while time.monotonic() < stop:
            value = (value * 7 + 1) % 999983
        (directory / "cpu-finished").write_text(str(value))
        wait_file(directory / "release", deadline)
        return 0
    path = test_resources.invocation("python", [{"name": "actual::arithmetic"}])
    native_operation.write_json(
        directory / "ready.json",
        {"run": str(path), "registry": str(test_resources.registry())},
    )
    wait_file(directory / "build-start", deadline)
    environment = {
        **os.environ,
        "PSE_TEST_INVOCATION": str(path),
        "RUSTC_WRAPPER": "",
        "RUSTC_WORKSPACE_WRAPPER": "",
    }
    code = build_observation.run_command(
        [
            "cargo",
            "build",
            "--offline",
            "--manifest-path",
            str(directory / "fixture/Cargo.toml"),
            "--message-format=json",
        ],
        env=environment,
    )
    if code:
        return code
    (directory / "test-started").write_text("actual Python child test")
    wait_file(directory / "test-release", deadline)
    actual = subprocess.run(
        [sys.executable, "-c", "assert sum([1, 2, 3]) == 6"], check=False, timeout=3
    )
    test_run.finish(
        path,
        {
            "exit_code": actual.returncode,
            "results": [
                {
                    "nodeid": "actual::arithmetic",
                    "status": "passed" if actual.returncode == 0 else "failure",
                }
            ],
        },
        "runner",
    )
    return actual.returncode


class ObservationControls(unittest.TestCase):
    def setUp(self) -> None:
        selected = os.environ.get("PSE_PLAN52_OBSERVATION_RECEIPTS")
        self.assertIsNotNone(
            selected, "set a fresh PSE_PLAN52_OBSERVATION_RECEIPTS directory"
        )
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
        self.assertIsNotNone(
            allocation, "bounded control requires an outer admitted scope"
        )
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
                env=native_operation.manager_control_environment(),
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
                native_operation.write_json(
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
                cli = subprocess.run(
                    [
                        sys.executable,
                        "scripts/activity.py",
                        "--json",
                        "--unit",
                        unit,
                        "--pressure",
                    ],
                    cwd=ROOT,
                    capture_output=True,
                    text=True,
                    timeout=3,
                    check=False,
                )
                (self.directory / "activity.json").write_text(cli.stdout)
                self.assertIn(
                    cli.returncode, (0, 1)
                )  # partial kernel fields remain explicit
                projected = json.loads(cli.stdout)
                self.assertTrue(projected["units_available"])
                self.assertEqual(projected["unit_selection"], [unit])
                self.assertEqual(
                    projected["pressure"]["selected"][group]["identity"]["inode"],
                    busy.selected.identity.inode,
                )
                self.assertEqual(
                    projected["pressure"]["selected"][group]["fields"][
                        "direct_member_affinity"
                    ]["status"],
                    "available",
                )
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

    def test_actual_build_test_terminal_and_observer_interrupt_reconnect(self) -> None:
        fixture = self.directory / "fixture"
        (fixture / "src").mkdir(parents=True)
        (fixture / "Cargo.toml").write_text(
            '[package]\nname="observation-control"\nversion="0.0.0"\nedition="2024"\n'
        )
        (fixture / "src/main.rs").write_text(
            'fn main() { println!("actual build"); }\n'
        )
        (fixture / "build.rs").write_text(
            "fn main() { std::thread::sleep(std::time::Duration::from_millis(600)); }\n"
        )
        environment = {
            **os.environ,
            "PSE_TEST_RESOURCE_STATE": str(self.directory / "registry"),
            "XDG_STATE_HOME": str(self.directory / "state"),
        }
        observed = []
        with (
            (self.directory / "worker.log").open("w") as log,
            subprocess.Popen(
                self.launch("build"),
                cwd=ROOT,
                env=environment,
                stdout=log,
                stderr=subprocess.STDOUT,
            ) as child,
        ):
            try:
                wait_file(self.directory / "ready.json", time.monotonic() + 5)
                ready = json.loads((self.directory / "ready.json").read_text())
                selected = watch.selection("run", ready["run"], Path(ready["registry"]))
                # Exact same environment selects the build snapshots without
                # initializing admission or compiler configuration in the watch.
                from unittest.mock import patch

                with patch.dict(os.environ, environment, clear=True):
                    initial = watch.watch(selected, max_wait=0)
                    observed.append(initial)
                    command = [
                        sys.executable,
                        "scripts/activity.py",
                        "--watch",
                        "--run",
                        ready["run"],
                        "--registry",
                        ready["registry"],
                        "--cursor",
                        initial["cursor"],
                    ]
                    with (
                        (self.directory / "watcher.log").open("w") as output,
                        subprocess.Popen(
                            command,
                            cwd=ROOT,
                            env=environment,
                            stdout=output,
                            stderr=subprocess.STDOUT,
                        ) as observer,
                    ):
                        time.sleep(0.35)
                        observer.send_signal(signal.SIGINT)
                        self.assertEqual(observer.wait(timeout=3), 130)
                    self.assertIsNone(
                        child.poll(),
                        "observer cancellation must leave the producer active",
                    )
                    self.assertIn(
                        "watcher-detached", (self.directory / "watcher.log").read_text()
                    )
                    (self.directory / "build-start").touch()
                    deadline = time.monotonic() + 8
                    cursor = initial["cursor"]
                    while not (self.directory / "test-started").exists():
                        result = watch.watch(selected, cursor=cursor, max_wait=0.2)
                        cursor = result["cursor"]
                        observed.append(result)
                        self.assertLess(time.monotonic(), deadline)
                    result = watch.watch(selected, cursor=cursor, max_wait=0)
                    observed.append(result)
                    self.assertEqual(result["snapshot"]["state"], "running")
                    self.assertTrue(
                        any(
                            item.get("status") == "returned"
                            for item in result["snapshot"]["builds"]
                        )
                    )
                    (self.directory / "test-release").touch()
                    while observed[-1]["snapshot"]["state"] != "result-complete":
                        observed.append(
                            watch.watch(
                                selected, cursor=observed[-1]["cursor"], max_wait=0.2
                            )
                        )
                        self.assertLess(time.monotonic(), deadline)
                    self.assertIsNotNone(
                        observed[-1]["snapshot"]["result_reference"][
                            "terminal_identity"
                        ]
                    )
                    self.assertTrue(
                        any(item["snapshot"].get("builds") for item in observed)
                    )
                    self.assertEqual(child.wait(timeout=3), 0)
                native_operation.write_json(
                    self.directory / "watch.json",
                    {
                        "observations": observed,
                        "command": command,
                        "workload_exit": child.returncode,
                    },
                )
            finally:
                if child.poll() is None:
                    child.terminate()
                    child.wait(timeout=3)


if __name__ == "__main__":
    if len(sys.argv) == 4 and sys.argv[1] == "--worker":
        raise SystemExit(worker(sys.argv[2], Path(sys.argv[3])))
    unittest.main()
