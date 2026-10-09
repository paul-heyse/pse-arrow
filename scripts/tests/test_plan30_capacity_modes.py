# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Opt-in actual widened and timing admission; harmless bounded processes only."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import time
import unittest
from pathlib import Path
from typing import TYPE_CHECKING, cast

if TYPE_CHECKING:
    from typing import TextIO

from scripts import host_admission as host
from scripts import native_operation as operation
from scripts.tests import test_plan30_placement as placement


@unittest.skipUnless(
    os.environ.get("PSE_PLAN30_PLACEMENT") == "1", "explicit actual-kernel control"
)
class CapacityModesTests(unittest.TestCase):
    def test_widened_functional_and_timing_lanes_preserve_actual_limits(self) -> None:
        directory = Path(os.environ["PSE_PLAN30_RECEIPTS"]).resolve()
        directory.mkdir(parents=True, exist_ok=False)
        children: list[subprocess.Popen] = []
        logs: list[TextIO] = []
        receipts: list[placement.PlacementSnapshot] = []

        def cleanup() -> None:
            for child_directory in directory.iterdir():
                if child_directory.is_dir():
                    (child_directory / "release-parent").touch()
            for child in children:
                try:
                    child.wait(timeout=15)
                except subprocess.TimeoutExpired:
                    child.kill()
                    child.wait(timeout=5)
            for receipt in receipts:
                deadline = time.monotonic() + 15
                while operation.populated(receipt["group"]):
                    if time.monotonic() >= deadline:
                        self.fail("owned harmless control failed to drain")
                    time.sleep(0.05)
                host.reconcile(Path(receipt["allocation_directory"]))
            for log in logs:
                log.close()

        self.addCleanup(cleanup)

        def launch(
            name: str, requested_class: str, requested_memory: str
        ) -> placement.PlacementSnapshot:
            child_directory = directory / name
            child_directory.mkdir()
            environment = dict(os.environ)
            for key in (
                host.MARKER,
                "PSE_ADMISSION_DEADLINE",
                "PSE_REQUIRE_STORE",
                "PSE_NATIVE_OPERATION",
                "PSE_NATIVE_HANDOFF",
            ):
                environment.pop(key, None)
            environment["PSE_MEMORY_MAX"] = requested_memory
            log = (child_directory / "caller.log").open("w")
            logs.append(log)
            child = subprocess.Popen(
                [
                    str(placement.ROOT / "scripts/pse-env"),
                    "--resource-class",
                    requested_class,
                    "--",
                    sys.executable,
                    "-m",
                    "scripts.tests.test_plan30_placement",
                    "--worker",
                    "hold",
                    str(child_directory),
                ],
                env=environment,
                cwd=placement.ROOT,
                stdout=log,
                stderr=subprocess.STDOUT,
            )
            children.append(child)
            placement.wait_file(child_directory / "ready.json", time.monotonic() + 45)
            receipt = cast(
                "placement.PlacementSnapshot",
                json.loads((child_directory / "ready.json").read_text()),
            )
            receipts.append(receipt)
            return receipt

        wide = launch("wide", "functional", "80G")
        timing = launch("timing", "timing", "56G")
        for receipt, memory, cores in (
            (wide, 80, host.select("wide").cores),
            (timing, 56, host.select("timing").cores),
        ):
            assert int(receipt["memory_max"]) == memory * host.GIB
            assert int(receipt["ancestor_memory_max"]) == memory * host.GIB
            assert receipt["swap_max"] == "0"
            assert set(receipt["affinity_cpus"]) == set(host.cpu_set(cores))
            nested = json.loads(
                (
                    directory / ("wide" if memory == 80 else "timing") / "nested.json"
                ).read_text()
            )
            assert nested["nonce"] == receipt["nonce"]
            assert nested["deadline"] == receipt["deadline"]
            assert operation.populated(receipt["group"])
        assert set(wide["affinity_cpus"]).isdisjoint(timing["affinity_cpus"])
        environment = dict(os.environ)
        for key in (
            host.MARKER,
            "PSE_ADMISSION_DEADLINE",
            "PSE_MEMORY_MAX",
            "PSE_REQUIRE_STORE",
            "PSE_NATIVE_OPERATION",
            "PSE_NATIVE_HANDOFF",
        ):
            environment.pop(key, None)
        started = time.monotonic()
        refused = subprocess.run(
            [
                str(placement.ROOT / "scripts/pse-env"),
                "--resource-class",
                "reference",
                "--",
                "true",
            ],
            env=environment,
            cwd=placement.ROOT,
            capture_output=True,
            text=True,
            timeout=host.policy()["admission_seconds"] + 15,
            check=False,
        )
        assert refused.returncode == 125
        assert "exclusive" in refused.stderr
        assert time.monotonic() - started >= host.policy()["admission_seconds"] - 0.5
        operation.write_json(
            directory / "result.json",
            {
                "result": "passed",
                "scope": "actual widened/timing kernel placement and reference exclusion; no scientific or performance claim",
                "callers": receipts,
                "reference_refusal": {
                    "returncode": refused.returncode,
                    "stderr": refused.stderr,
                },
            },
        )


if __name__ == "__main__":
    unittest.main()
