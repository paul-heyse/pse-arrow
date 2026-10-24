# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Kernel interpretation controls: unavailable, reset and placement are distinct."""

# ruff: noqa: PT009, PT027 -- stdlib setup controls
from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import pressure_observation as pressure

PSI = "some avg10=0.10 avg60=0.20 avg300=0.30 total=123\nfull avg10=0.00 avg60=0.00 avg300=0.00 total=4\n"


class PressureTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.proc = self.root / "proc"
        self.cgroup = self.root / "cgroup"
        boot = self.proc / "sys/kernel/random/boot_id"
        boot.parent.mkdir(parents=True)
        boot.write_text("original-boot")
        (self.proc / "pressure").mkdir()
        for resource in pressure.PSI:
            (self.proc / "pressure" / resource).write_text(PSI)
        for group in (
            self.cgroup,
            self.cgroup / "parent",
            self.cgroup / "parent/child",
        ):
            group.mkdir(exist_ok=True)
            (group / "cgroup.procs").write_text("")
            for resource in pressure.PSI:
                (group / (resource + ".pressure")).write_text(PSI)
            for name in pressure.COUNTERS:
                (group / name).write_text(
                    "8:0 rbytes=100 wbytes=200 rios=1 wios=2"
                    if name == "io.stat"
                    else "usage_usec 100\nnr_throttled 2\nthrottled_usec 20"
                )
            for name in pressure.PLACEMENT:
                (group / name).write_text("0-3" if "cpuset" in name else "max")
        (self.cgroup / "parent/cpu.max").write_text("10000 100000")
        self.directory = self.cgroup / "parent/child"

    def sample(self) -> pressure.Sample:
        return pressure.sample("/parent/child", cgroup=self.cgroup, proc=self.proc)

    def test_host_and_selected_pressure_keep_scope_and_ancestor_restrictions(
        self,
    ) -> None:
        host = pressure.sample(proc=self.proc)
        self.assertEqual(host.status, "observed")
        self.assertNotIn("full.total", host.fields["cpu.pressure"].values)
        selected = self.sample()
        self.assertEqual(selected.status, "observed")
        if selected.identity is None:
            self.fail("Expected the selected cgroup identity")
        self.assertEqual(selected.identity.group, "/parent/child")
        self.assertEqual(selected.fields["cpu.pressure"].values["full.total"], 4)
        self.assertEqual(selected.fields["io.stat"].values["8:0.rbytes"], 100)
        self.assertEqual(selected.fields["cpuset.cpus.effective"].values["raw"], "0-3")
        self.assertEqual([a.group for a in selected.ancestors], ["/parent", "/"])
        self.assertEqual(
            selected.ancestors[0].fields["cpu.max"].values["raw"], "10000 100000"
        )
        self.assertLessEqual(
            selected.started_monotonic_ns, selected.finished_monotonic_ns
        )

    def test_interval_counters_reset_missing_device_and_changed_incarnation(
        self,
    ) -> None:
        first = self.sample()
        (self.directory / "cpu.stat").write_text(
            "usage_usec 200\nnr_throttled 1\nthrottled_usec 60"
        )
        (self.directory / "io.stat").write_text("9:0 rbytes=999 wbytes=9")
        second = self.sample()
        diff = pressure.delta(first, second)
        self.assertGreater(diff["interval_seconds"], 0)
        self.assertEqual(diff["current_identity"]["group"], "/parent/child")
        self.assertEqual(
            diff["interval_start_monotonic_ns"], first.finished_monotonic_ns
        )
        cpu = diff["fields"]["cpu.stat"]
        self.assertEqual(cpu["usage_usec"]["delta"], 100)
        self.assertEqual(cpu["nr_throttled"]["status"], "reset")
        self.assertNotIn("per_second", cpu["nr_throttled"])
        self.assertEqual(
            diff["fields"]["io.stat"]["9:0.rbytes"]["status"], "unavailable"
        )
        if second.identity is None:
            self.fail("Expected the observed cgroup identity")
        second.identity.boot = "new-boot"
        self.assertEqual(pressure.delta(first, second)["status"], "unavailable")

    def test_permission_missing_field_and_invalid_records_are_explicit(self) -> None:
        original = pressure.read

        def denied(path: Path) -> str:
            if path.name == "io.stat":
                raise PermissionError("denied")
            return original(path)

        (self.directory / "memory.events").unlink()
        with patch.object(pressure, "read", side_effect=denied):
            selected = self.sample()
        self.assertEqual(selected.status, "partial")
        io_reason = selected.fields["io.stat"].reason
        memory_reason = selected.fields["memory.events"].reason
        if io_reason is None or memory_reason is None:
            self.fail("Expected original unavailable-field reasons")
        self.assertIn("PermissionError", io_reason)
        self.assertIn("FileNotFoundError", memory_reason)
        self.assertEqual(selected.fields["cpu.stat"].status, "available")
        for kind, value in (
            ("psi", "some avg10=nan avg60=0 avg300=0 total=0"),
            ("psi", "some total=1"),
            ("cpu.stat", "usage_usec -1"),
            ("io.stat", "8:0 rbytes=x"),
        ):
            with self.assertRaises(ValueError):
                pressure.parse(value, kind)

    def test_actual_affinity_is_bound_to_member_incarnation_and_membership(
        self,
    ) -> None:
        process = self.proc / "123"
        process.mkdir()
        (self.directory / "cgroup.procs").write_text("123")
        (process / "stat").write_text(
            "123 (name with spaces) " + " ".join(["S"] + ["0"] * 18 + ["987"])
        )
        (process / "cgroup").write_text("0::/parent/child")
        (process / "status").write_text(
            "Name: original\nCpus_allowed_list:\t8-15,24-31\n"
        )
        field = self.sample().fields["direct_member_affinity"]
        self.assertEqual(field.status, "available")
        self.assertEqual(field.values, {"123:987": "8-15,24-31"})
        (process / "cgroup").write_text("0::/different")
        self.assertEqual(
            self.sample().fields["direct_member_affinity"].status, "unavailable"
        )

    def test_disappearance_identity_change_and_traversal_discard_fields(self) -> None:
        for group in ("/missing", "/../escape", "relative"):
            sample = pressure.sample(group, cgroup=self.cgroup, proc=self.proc)
            self.assertEqual(sample.status, "unavailable")
            self.assertIsNone(sample.identity)
            self.assertEqual(sample.fields, {})
        original = pressure.identity
        count = 0

        def replaced(
            directory: Path, group: str | None, proc: Path
        ) -> pressure.Identity:
            nonlocal count
            value = original(directory, group, proc)
            if group == "/parent/child":
                count += 1
                if count > 1:
                    if value.inode is None:
                        self.fail("Expected the original selected inode")
                    value.inode += 1
            return value

        with patch.object(pressure, "identity", side_effect=replaced):
            self.assertEqual(self.sample().status, "unavailable")


if __name__ == "__main__":
    unittest.main()
