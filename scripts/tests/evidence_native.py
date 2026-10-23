# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit built-artifact controls: PSE_EVIDENCE_TEST_TOOL selects the built binary.

Run explicitly with python -m unittest scripts.tests.evidence_native. This module is
outside default test discovery because building the optional operator is an explicit
operation, never an inspection side effect. Missing selection fails, never skips.
"""

# ruff: noqa: PT009
from __future__ import annotations

import hashlib
import json
import os
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import (
    active_diagnostics,
    evidence_projection,
    evidence_sources,
    storage_health,
    surreal_server,
    test_resources,
    test_run,
)

ROOT = Path(__file__).resolve().parents[2]


class EvidenceNativeTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tool = Path(os.environ["PSE_EVIDENCE_TEST_TOOL"]).absolute()
        self.assertTrue(
            self.tool.is_file(), "build explicit operator before this selected control"
        )
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.registry = self.root / "registry"
        selection = patch.object(test_resources, "registry", return_value=self.registry)
        selection.start()
        self.addCleanup(selection.stop)
        self.environment = {
            key: value
            for key, value in os.environ.items()
            if key
            not in {
                "PSE_TOOLING_ACTIVITY_FD",
                "PSE_TOOLING_SERVICE_INSTANCE_ID",
                "SURREAL_USER",
                "SURREAL_PASS",
            }
        }
        self.environment["SYMBOLICA_LICENSE"] = ""
        self.environment["PSE_SURREAL_STATE"] = str(self.root / "absent-service")

    def native(
        self,
        snapshots: list[evidence_projection.EvidenceSnapshot],
        operation: str,
        **filters: object,
    ) -> dict:
        question = self.root / "question.json"
        question.write_text(json.dumps({"operation": operation, **filters}))
        arguments = [
            str(self.tool),
            "evidence",
            "--state",
            str(self.root / "absent-service"),
            "offline",
            "--question",
            str(question),
        ]
        for offset, snapshot in enumerate(snapshots):
            path = self.root / f"snapshot-{offset}.json"
            path.write_text(json.dumps(snapshot))
            arguments += ["--input", str(path)]
        completed = subprocess.run(
            arguments,
            cwd=ROOT,
            env=self.environment,
            text=True,
            capture_output=True,
            check=False,
            timeout=30,
        )
        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertFalse(
            (self.root / "absent-service").exists(),
            "offline dispatch must precede service configuration",
        )
        return json.loads(completed.stdout)

    def test_real_offline_executable_authentic_failed_and_unreported_denominator(
        self,
    ) -> None:
        handle = test_resources.invocation(
            "python",
            [
                {"name": "suite::failed", "crate_name": "declared-owner"},
                {"name": "suite::missing", "crate_name": "declared-owner"},
                {"name": "suite::skipped"},
            ],
        )
        test_run.finish(
            handle,
            {
                "exit_code": 130,
                "results": [
                    {"nodeid": "suite::failed", "status": "failure"},
                    {"nodeid": "suite::skipped", "status": "skipped"},
                ],
                "report_errors": ["interrupted"],
            },
            "runner",
        )
        snapshot = evidence_projection.invocation_snapshot(
            str(handle), registry=self.registry
        )
        original = {
            str(path): path.read_bytes()
            for path in self.registry.rglob("*")
            if path.is_file()
        }
        result = self.native(
            [snapshot, snapshot], "failures", crate_name="declared-owner"
        )
        self.assertEqual(result["denominator"][0]["selected_tests"], 2)
        self.assertEqual([row["test_id"] for row in result["rows"]], ["suite::failed"])
        self.assertEqual(result["sources"][0]["coverage"], "partial")
        result = self.native([snapshot], "missing")
        self.assertEqual(result["denominator"][0]["selected_tests"], 3)
        self.assertEqual([row["test_id"] for row in result["rows"]], ["suite::missing"])
        # Public thin CLI adapts the genuine owner directly, without extension/build/service.
        completed = subprocess.run(
            [
                sys.executable,
                "-m",
                "scripts.evidence_store",
                "--tool",
                str(self.tool),
                "offline",
                "--invocation",
                str(handle),
                "--registry",
                str(self.registry),
                "--operation",
                "distribution",
            ],
            cwd=ROOT,
            env=self.environment,
            text=True,
            capture_output=True,
            check=False,
            timeout=30,
        )
        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertEqual(
            json.loads(completed.stdout)["denominator"][0]["selected_tests"], 3
        )
        self.assertEqual(
            original,
            {
                str(path): path.read_bytes()
                for path in self.registry.rglob("*")
                if path.is_file()
            },
        )

    def test_real_native_linked_standalone_and_explicit_released_detail(self) -> None:
        invocation = test_resources.invocation("python", [{"name": "suite::failed"}])
        test_run.finish(
            invocation,
            {
                "exit_code": 1,
                "results": [{"nodeid": "suite::failed", "status": "failure"}],
                "report_errors": [],
            },
            "runner",
        )
        snapshots = []
        for name, association in [
            ("linked", {"registry": str(self.registry), "identity": invocation.stem}),
            ("nearby", None),
        ]:
            directory = self.root / name
            directory.mkdir(mode=0o700)
            (directory / "capture.txt").write_text("protected selected detail\n")
            active_diagnostics.publish(
                directory,
                {
                    "schema": active_diagnostics.SCHEMA,
                    "action": "control",
                    "status": "tool_failure",
                    "target_outcome": "unknown",
                    "invocation": association,
                    "started_monotonic_ns": 123,
                },
            )
            snapshots.append(evidence_sources.diagnostic_snapshot(directory))
        snapshots.append(
            evidence_projection.invocation_snapshot(
                str(invocation), registry=self.registry
            )
        )
        result = self.native(snapshots, "diagnostics")
        self.assertEqual(len(result["linked_actions"]), 1)
        self.assertEqual(result["linked_actions"][0]["tool_outcome"], "tool_failure")
        self.assertEqual(
            len([row for row in result["rows"] if row["kind"] == "diagnostic_tool"]), 2
        )
        command = [
            sys.executable,
            "-m",
            "scripts.evidence_store",
            "detail",
            "--diagnostic",
            str(self.root / "linked"),
            "--member",
            "capture.txt",
        ]
        completed = subprocess.run(
            command,
            cwd=ROOT,
            env=self.environment,
            text=True,
            capture_output=True,
            check=False,
            timeout=30,
        )
        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertEqual(
            json.loads(completed.stdout)["tail"], ["protected selected detail"]
        )
        (self.root / "linked/capture.txt").unlink()
        result = self.native(snapshots, "diagnostics")
        self.assertEqual(
            len(result["linked_actions"]), 1, "released detail retains frozen summary"
        )
        completed = subprocess.run(
            command,
            cwd=ROOT,
            env=self.environment,
            text=True,
            capture_output=True,
            check=False,
            timeout=30,
        )
        self.assertEqual(json.loads(completed.stdout)["status"], "missing")

    def test_storage_health_original_offline_source_keeps_real_links_and_unknown_science(
        self,
    ) -> None:
        # Capture actual independent kernel/host fields with no service available.
        with (
            patch.object(
                surreal_server,
                "config_for",
                side_effect=surreal_server.SupervisorError("service unavailable"),
            ),
            patch.object(
                surreal_server,
                "metric_snapshot",
                side_effect=surreal_server.SupervisorError("service unavailable"),
            ),
        ):
            record = storage_health.capture(
                self.root / "absent-service", time.monotonic() + 5
            )
        original = self.root / "original-health.json"
        import msgspec  # noqa: PLC0415 -- original host codec, no native extension

        original.write_bytes(msgspec.json.encode(record))
        snapshot = evidence_sources.storage_health_snapshot(original)
        reference = next(
            f["detail_ref"] for f in snapshot["facts"] if f["kind"] == "diagnostic_tool"
        )
        directory = self.root / "linked-health-action"
        directory.mkdir(mode=0o700)
        active_diagnostics.publish(
            directory,
            {
                "schema": active_diagnostics.SCHEMA,
                "action": "control",
                "status": "tool_failure",
                "target_outcome": "unknown",
                "storage_health": reference,
            },
        )
        snapshots = [snapshot, evidence_sources.diagnostic_snapshot(directory)]
        result = self.native(snapshots, "diagnostics")
        tools = [row for row in result["rows"] if row["kind"] == "diagnostic_tool"]
        self.assertEqual({row["status"] for row in tools}, {"partial", "tool_failure"})
        self.assertTrue(
            all(
                row["status"] == "unknown"
                for row in result["rows"]
                if row["kind"] == "diagnostic_science"
            )
        )
        self.assertEqual(
            result["linked_actions"],
            [],
            "a storage-health link is not a test/invocation association",
        )
        links = result["declared_links"]
        self.assertEqual(
            len(
                [
                    row
                    for row in links
                    if row["link_relation"] == "storage_health"
                    and row["link_target"] == reference
                ]
            ),
            2,
        )
        self.assertFalse(
            any(
                row["link_relation"] in {"service", "process", "invocation"}
                for row in links
            )
        )
        self.assertEqual(record.host["MemTotal_bytes"].status, "available")
        self.assertFalse((self.root / "absent-service").exists())

    def test_real_native_measurement_contexts_actual_samples_and_unknown_membership(
        self,
    ) -> None:
        snapshots = []
        for name, provider, profile in [
            ("first", "provider-a", "release"),
            ("other", "provider-b", "performance"),
        ]:
            directory = self.root / name
            directory.mkdir()
            csv = directory / "raw.csv"
            csv.write_text(
                "sample_measured_value,iteration_count,unit\n100,10,ns\n300,10,ns\n"
            )
            report = {
                "schema": "process-cost-v5",
                "measured": True,
                "profile": {"cargo_profile": profile, "features": ["native-process"]},
                "selected_cases": ["case", "censored", "failed"],
                "cases": [
                    {
                        "id": "case",
                        "workload": {"size": 2},
                        "provider": provider,
                        "artifacts": {
                            "raw.csv": hashlib.sha256(csv.read_bytes()).hexdigest()
                        },
                    },
                    {"id": "censored", "status": "censored"},
                    {"id": "failed", "status": "failed"},
                ],
            }
            path = directory / "case-measure.json"
            path.write_text(json.dumps(report))
            snapshots.append(
                evidence_sources.measurement_snapshot(path, criterion={"case": csv})
            )
        result = self.native([*snapshots, snapshots[0]], "measurements")
        self.assertEqual(len(result["rows"]), 2)
        self.assertEqual([row["sample_count"] for row in result["rows"]], [2, 2])
        self.assertEqual([row["mean_ns"] for row in result["rows"]], [20.0, 20.0])
        self.assertEqual(
            {row["status"] for row in result["populations"]},
            {"measured", "censored", "failed"},
        )
        self.assertTrue(
            all(row["status"] == "unavailable" for row in result["association"])
        )
        self.assertTrue(all(not row["comparable"] for row in result["comparability"]))


if __name__ == "__main__":
    unittest.main()
