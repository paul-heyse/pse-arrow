# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Opt-in kernel placement control; no solver, store or memory-stress workload.

Run through a light pse-env wrapper with PSE_PLAN30_PLACEMENT=1 and a new
PSE_PLAN30_RECEIPTS directory. Receipts persist on failure for inspection.
"""

# ruff: noqa: PT009
from __future__ import annotations

import json
import os
import subprocess
import sys
import time
import unittest
from pathlib import Path
from typing import TYPE_CHECKING, TypedDict, cast

if TYPE_CHECKING:
    from typing import TextIO

from scripts import host_admission as host
from scripts import native_operation as operation

ROOT = Path(__file__).resolve().parents[2]
LIMIT = 180.0


class PlacementSnapshot(TypedDict):
    """Kernel observations emitted by the owned control child."""

    pid: int
    start: str
    nonce: str
    deadline: float
    group: str
    units: dict[str, host.BoundUnit]
    cores: list[int]
    memory_max: str
    swap_max: str
    cpus: str
    cpuset_directory: str
    affinity_cpus: list[int]
    cpu_max: str
    ancestor_memory_max: str
    allocation_directory: str


def cpu_list(value: str) -> set[int]:
    result: set[int] = set()
    for part in value.strip().split(","):
        if part:
            ends = part.split("-")
            result.update(range(int(ends[0]), int(ends[-1]) + 1))
    return result


def snapshot() -> PlacementSnapshot:
    allocation = host.inherit(os.environ)
    if allocation is None:
        raise RuntimeError("worker lacks actual allocation")
    group = operation.process_group(os.getpid())
    directory = Path("/sys/fs/cgroup") / group.lstrip("/")
    cpuset_directory = directory
    while not (cpuset_directory / "cpuset.cpus.effective").is_file():
        if cpuset_directory == Path("/sys/fs/cgroup"):
            raise RuntimeError("kernel exposes no effective cpuset")
        cpuset_directory = cpuset_directory.parent
    with host.allocation_metadata(allocation.directory) as state:
        owner = state["owners"][allocation.nonce]
        units = dict(owner["units"])
    return {
        "pid": os.getpid(),
        "start": operation.start_identity(os.getpid()),
        "nonce": allocation.nonce,
        "deadline": allocation.deadline,
        "group": group,
        "units": units,
        "cores": list(allocation.profile.cores),
        "memory_max": (directory / "memory.max").read_text().strip(),
        "swap_max": (directory / "memory.swap.max").read_text().strip(),
        "cpus": (cpuset_directory / "cpuset.cpus.effective").read_text().strip(),
        "cpuset_directory": str(cpuset_directory),
        "affinity_cpus": sorted(os.sched_getaffinity(0)),
        "cpu_max": (directory / "cpu.max").read_text().strip(),
        "ancestor_memory_max": (directory.parent / "memory.max").read_text().strip(),
        "allocation_directory": str(allocation.directory),
    }


def wait_file(path: Path, deadline: float) -> None:
    while not path.exists():
        if time.monotonic() >= deadline:
            raise TimeoutError(f"control file not observed: {path}")
        time.sleep(0.05)


def worker(mode: str, directory: Path) -> int:
    deadline = time.monotonic() + LIMIT
    if mode == "nested":
        operation.write_json(directory / "nested.json", snapshot())
        return 0
    if mode == "descendant":
        operation.write_json(directory / "descendant.json", snapshot())
        wait_file(directory / "release-descendant", deadline)
        return 0
    descendant = None
    if mode == "survivor":
        descendant = subprocess.Popen(
            [
                sys.executable,
                "-m",
                "scripts.tests.test_plan30_placement",
                "--worker",
                "descendant",
                str(directory),
            ],
            cwd=ROOT,
            start_new_session=True,
        )
        wait_file(directory / "descendant.json", deadline)
    nested = subprocess.run(
        [
            str(ROOT / "scripts/pse-env"),
            "--resource-class",
            "light",
            "--",
            sys.executable,
            "-m",
            "scripts.tests.test_plan30_placement",
            "--worker",
            "nested",
            str(directory),
        ],
        cwd=ROOT,
        timeout=20,
        check=False,
        capture_output=True,
        text=True,
    )
    (directory / "nested.log").write_text(nested.stdout + nested.stderr)
    if nested.returncode:
        raise RuntimeError(f"nested boundary failed: {nested.returncode}")
    operation.write_json(directory / "ready.json", snapshot())
    wait_file(directory / "release-parent", deadline)
    # The descendant deliberately survives this parent; its own finite lifetime
    # and release file bound it, and actual cgroup population remains authoritative.
    if descendant is not None:
        (directory / "parent-returning").touch()
    return 0


@unittest.skipUnless(
    os.environ.get("PSE_PLAN30_PLACEMENT") == "1", "explicit actual-kernel control"
)
class Plan30PlacementTests(unittest.TestCase):
    def test_independent_lanes_nested_reuse_refusal_and_surviving_descendant(
        self,
    ) -> None:
        self.directory = Path(os.environ["PSE_PLAN30_RECEIPTS"]).resolve()
        self.directory.mkdir(parents=True, exist_ok=False)
        self.children: list[subprocess.Popen] = []
        self.logs: list[TextIO] = []
        self.receipts: list[PlacementSnapshot] = []
        self.addCleanup(self.cleanup_owned)
        first, a = self.launch("first", "survivor")
        second, b = self.launch("second", "hold")
        self.verify_kernel(a)
        self.verify_kernel(b)
        self.assertNotEqual(a["nonce"], b["nonce"])
        self.assertTrue(set(a["cores"]).isdisjoint(b["cores"]))
        self.assertTrue(set(a["affinity_cpus"]).isdisjoint(b["affinity_cpus"]))
        for name, receipt in (("first", a), ("second", b)):
            nested = json.loads((self.directory / name / "nested.json").read_text())
            for field in (
                "nonce",
                "deadline",
                "group",
                "memory_max",
                "cpus",
                "affinity_cpus",
            ):
                self.assertEqual(
                    nested[field], cast("dict[str, object]", receipt)[field], field
                )

        # Kill only this test's allocation caller. The registered child scope
        # and actual descendant are deliberately left alive.
        first.kill()
        first.wait(timeout=10)
        (self.directory / "first/release-parent").touch()
        wait_file(self.directory / "first/parent-returning", time.monotonic() + 10)
        descendant = json.loads((self.directory / "first/descendant.json").read_text())
        self.assertEqual(
            operation.start_identity(descendant["pid"]), descendant["start"]
        )
        self.assertEqual(operation.process_group(descendant["pid"]), a["group"])
        self.assertTrue(operation.populated(a["group"]))
        allocation_directory = Path(a["allocation_directory"])
        host.reconcile(allocation_directory)
        with host.allocation_metadata(allocation_directory) as state:
            retained = state["owners"][a["nonce"]]
            self.assertFalse(host.caller_alive(retained))
            self.assertFalse(host.drained(retained))
            operation.write_json(self.directory / "retained-owner.json", retained)

        # Actual pse-env entry point uses the original policy admission clock;
        # do not shorten it by replacing the allocator or policy.
        start = time.monotonic()
        third = subprocess.run(
            self.command("true"),
            env=self.independent_environment(),
            cwd=ROOT,
            timeout=host.policy()["admission_seconds"] + 15,
            capture_output=True,
            text=True,
            check=False,
        )
        elapsed = time.monotonic() - start
        operation.write_json(
            self.directory / "third-refusal.json",
            {
                "returncode": third.returncode,
                "elapsed_seconds": elapsed,
                "stdout": third.stdout,
                "stderr": third.stderr,
            },
        )
        self.assertEqual(third.returncode, 125)
        self.assertIn(
            "Admission deadline exhausted: functional slots occupied", third.stderr
        )
        self.assertGreaterEqual(elapsed, host.policy()["admission_seconds"] - 0.5)
        self.assertLess(elapsed, host.policy()["admission_seconds"] + 15)
        self.assertTrue(operation.populated(a["group"]))
        host.reconcile(allocation_directory)
        with host.allocation_metadata(allocation_directory) as state:
            self.assertIn(a["nonce"], state["owners"])

        (self.directory / "first/release-descendant").touch()
        self.wait_drained(a)
        host.reconcile(allocation_directory)
        with host.allocation_metadata(allocation_directory) as state:
            self.assertNotIn(a["nonce"], state["owners"])
        replacement, c = self.launch("replacement", "hold")
        self.verify_kernel(c)
        self.assertNotEqual(c["nonce"], a["nonce"])
        self.assertTrue(set(c["cores"]).isdisjoint(b["cores"]))
        for name, child, receipt in (
            ("replacement", replacement, c),
            ("second", second, b),
        ):
            (self.directory / name / "release-parent").touch()
            self.assertEqual(child.wait(timeout=15), 0)
            self.wait_drained(receipt)
        operation.write_json(
            self.directory / "result.json",
            {
                "result": "passed",
                "scope": "finite actual-kernel placement controls only",
                "callers": self.receipts,
            },
        )

    def independent_environment(self) -> dict[str, str]:
        env = dict(os.environ)
        for key in (
            host.MARKER,
            "PSE_ADMISSION_DEADLINE",
            "PSE_MEMORY_MAX",
            "PSE_REQUIRE_STORE",
            "PSE_NATIVE_OPERATION",
            "PSE_NATIVE_HANDOFF",
        ):
            env.pop(key, None)
        env["PSE_RESOURCE_CLASS"] = "functional"
        env["PSE_MEMORY_MAX"] = str(host.settings("functional")["slot_gib"]) + "G"
        return env

    def command(self, *command: str) -> list[str]:
        return [
            str(ROOT / "scripts/pse-env"),
            "--resource-class",
            "functional",
            "--",
            *command,
        ]

    def launch(
        self, name: str, mode: str
    ) -> tuple[subprocess.Popen, PlacementSnapshot]:
        directory = self.directory / name
        directory.mkdir()
        log = (directory / "caller.log").open("w")
        self.logs.append(log)
        child = subprocess.Popen(
            self.command(
                sys.executable,
                "-m",
                "scripts.tests.test_plan30_placement",
                "--worker",
                mode,
                str(directory),
            ),
            env=self.independent_environment(),
            cwd=ROOT,
            stdout=log,
            stderr=subprocess.STDOUT,
        )
        self.children.append(child)
        wait_file(directory / "ready.json", time.monotonic() + 45)
        receipt = cast(
            "PlacementSnapshot", json.loads((directory / "ready.json").read_text())
        )
        self.receipts.append(receipt)
        return child, receipt

    def verify_kernel(self, receipt: PlacementSnapshot) -> None:
        self.assertEqual(
            int(receipt["memory_max"]),
            host.settings("functional")["slot_gib"] * host.GIB,
        )
        self.assertEqual(receipt["swap_max"], "0")
        self.assertEqual(
            int(receipt["ancestor_memory_max"]), int(receipt["memory_max"])
        )
        # This user manager does not delegate cpuset. The bind-child boundary
        # enforces the scheduler lane through affinity before running workloads;
        # the nearest inherited cpuset is recorded, not mistaken for that lane.
        self.assertTrue(
            set(receipt["affinity_cpus"]).issubset(cpu_list(receipt["cpus"]))
        )
        self.assertEqual(
            set(receipt["affinity_cpus"]), set(host.cpu_set(receipt["cores"]))
        )
        quota, period = map(int, receipt["cpu_max"].split())
        self.assertEqual(quota, period * len(receipt["cores"]))
        self.assertTrue(operation.populated(receipt["group"]))
        self.assertTrue(
            any(
                owner.get("group") == receipt["group"]
                for owner in receipt["units"].values()
            )
        )

    def wait_drained(self, receipt: PlacementSnapshot) -> None:
        deadline = time.monotonic() + 15
        while operation.populated(receipt["group"]):
            if time.monotonic() >= deadline:
                self.fail(f"owned group did not drain: {receipt['group']}")
            time.sleep(0.1)

    def cleanup_owned(self) -> None:
        for directory in self.directory.iterdir():
            if directory.is_dir():
                for name in ("release-parent", "release-descendant"):
                    (directory / name).touch()
        for child in self.children:
            if child.poll() is None:
                try:
                    child.wait(timeout=15)
                except subprocess.TimeoutExpired:
                    child.kill()
                    child.wait(timeout=5)
        # Only receipts authenticated by this test's launched child identify
        # cleanup targets; never signal another runtime unit or remove artifacts.
        for receipt in self.receipts:
            if operation.populated(receipt["group"]):
                for unit, owner in receipt["units"].items():
                    observed = operation.unit_observation(unit)
                    if owner.get("group") == receipt["group"] and observed.get(
                        "InvocationID"
                    ) == owner.get("invocation"):
                        subprocess.run(
                            [
                                "systemctl",
                                "--user",
                                "kill",
                                "--kill-whom=all",
                                "--signal=SIGKILL",
                                unit,
                            ],
                            capture_output=True,
                            check=False,
                            timeout=10,
                        )
                self.wait_drained(receipt)
            host.reconcile(Path(receipt["allocation_directory"]))
            host.retire_empty_allocation(
                host.Allocation(
                    Path(receipt["allocation_directory"]),
                    receipt["nonce"],
                    host.Profile(
                        "functional",
                        int(receipt["memory_max"]),
                        "functional",
                        1,
                        tuple(receipt["cores"]),
                    ),
                    receipt["deadline"],
                )
            )
        for log in self.logs:
            log.close()


if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--worker":
        raise SystemExit(worker(sys.argv[2], Path(sys.argv[3])))
    unittest.main()
