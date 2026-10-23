# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Authentic owner-shaped evidence with independent selected population expectations."""
# ruff: noqa: PT009, PT027
from __future__ import annotations

import contextlib
import hashlib
import io
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import active_diagnostics, evidence_detail, evidence_projection, evidence_sources, evidence_store
from scripts import resource_store, test_resources, test_run


class EvidenceSourceTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.registry = self.root / "registry"
        selected = patch.object(test_resources, "registry", return_value=self.registry)
        selected.start()
        self.addCleanup(selected.stop)

    def test_declared_inventory_crate_mapping_and_foreign_selection_refused(self) -> None:
        original = test_resources.invocation("rust", [{"binary_id": "binary-special", "name": "alpha", "executable": "/explicit/binary"}])
        inventory = {"rust-suites": {"binary-special": {"binary-id": "binary-special", "binary-path": "/explicit/binary", "package-name": "pse-real-owner", "testcases": {"alpha": {"filter-match": {"status": "matches"}}}}}}
        path = self.root / "inventory.json"
        path.write_text(json.dumps(inventory))
        snapshot = evidence_projection.invocation_snapshot(str(original), registry=self.registry, inventory=path)
        subject = next(f for f in snapshot["facts"] if f["kind"] == "rust_test")
        self.assertEqual(subject["crate_name"], "pse-real-owner")
        self.assertEqual(subject["test_id"], "binary-special::alpha")
        inventory["rust-suites"]["binary-special"]["testcases"] = {"foreign": {"filter-match": {"status": "matches"}}}
        path.write_text(json.dumps(inventory))
        with self.assertRaisesRegex(ValueError, "differs"):
            evidence_projection.invocation_snapshot(str(original), registry=self.registry, inventory=path)

    def action(self, *, linked: bool) -> Path:
        path = self.root / ("linked" if linked else "standalone")
        path.mkdir(mode=0o700)
        (path / "capture.txt").write_text("authentic raw detail\n")
        invocation = None
        if linked:
            handle = test_resources.invocation("python", [{"name": "suite::failure"}])
            test_run.finish(handle, {"exit_code": 1, "results": [{"nodeid": "suite::failure", "status": "failure"}], "report_errors": []}, "runner")
            invocation = {"registry": str(self.registry), "identity": handle.stem}
        active_diagnostics.publish(path, {"schema": active_diagnostics.SCHEMA, "action": "control", "status": "tool_failure", "target_outcome": "unknown", "invocation": invocation})
        return path

    def test_declared_link_standalone_and_released_summary(self) -> None:
        linked, standalone = self.action(linked=True), self.action(linked=False)
        # Summary must never eagerly read capture bytes.
        with patch.object(active_diagnostics.run_inspection, "artifact_observation", side_effect=AssertionError("raw opened")):
            snapshot = evidence_sources.diagnostic_snapshot(linked)
        links = [f for f in snapshot["facts"] if f["kind"] == "declared_link"]
        self.assertEqual(len(links), 1)
        self.assertEqual(links[0]["link_relation"], "invocation")
        self.assertFalse([f for f in evidence_sources.diagnostic_snapshot(standalone)["facts"] if f["kind"] == "declared_link"])
        facts = {f["kind"]: f for f in snapshot["facts"]}
        self.assertEqual(facts["diagnostic_tool"]["status"], "tool_failure")
        self.assertEqual(facts["diagnostic_target"]["status"], "unknown")
        invocation = next((self.registry / "invocations").iterdir())
        source = evidence_projection.invocation_snapshot(str(invocation), registry=self.registry)
        declared = next(f for f in source["facts"] if f["kind"] == "declared_link")
        self.assertEqual(declared["link_target"], facts["diagnostic_tool"]["detail_ref"])
        self.assertEqual(evidence_detail.diagnostic_detail(linked, "capture.txt")["status"], "available")
        (linked / "capture.txt").unlink()
        self.assertEqual(evidence_sources.diagnostic_snapshot(linked), snapshot)
        self.assertEqual(evidence_detail.diagnostic_detail(linked, "capture.txt")["status"], "missing")

    def test_selected_copy_never_follows_historical_source_path(self) -> None:
        directory = self.root / "copy"
        directory.mkdir()
        (directory / "raw.txt").write_text("copied\n")
        (directory / "evidence-copy.json").write_text(json.dumps({"schema": "selected-evidence-copy-v1", "original_path": "/never/open", "members": {"raw.txt": {"sha256": hashlib.sha256(b"copied\n").hexdigest()}}}))
        self.assertEqual(evidence_detail.copied_detail(directory, "raw.txt")["tail"], ["copied"])
        (directory / "raw.txt").write_text("replaced\n")
        self.assertEqual(evidence_detail.copied_detail(directory, "raw.txt")["status"], "changed")
        self.assertEqual(evidence_detail.copied_detail(directory, "other")["status"], "unavailable")
        with self.assertRaises(ValueError):
            evidence_detail.copied_detail(directory, "../outside")

    def test_assessment_owner_codec_gate_denominator_and_reuse(self) -> None:
        report = {"version": 5, "scope": [{"name": "executed"}, {"name": "never"}], "complete": False, "source_unchanged": True, "baseline_failures": 0, "provenance_errors": [], "checks": [{"gate": "executed", "status": "failed", "log": "executed.log", "origin": "previous-owned-report", "selected": [{"nodeid": "suite::failed"}, {"nodeid": "suite::missing"}, {"nodeid": "suite::skipped"}], "results": [{"nodeid": "suite::failed", "status": "failure"}, {"nodeid": "suite::skipped", "status": "skipped"}]}]}
        (self.root / "checks.json").write_text(json.dumps(report))
        snapshot = evidence_sources.assessment_snapshot(self.root)
        self.assertEqual([f["status"] for f in snapshot["facts"] if f["kind"] == "python_test"], ["failure", "unreported", "skipped"])
        self.assertEqual([f["status"] for f in snapshot["facts"] if f["kind"] == "assessment_gate"], ["failed", "unreported"])
        self.assertEqual(next(f for f in snapshot["facts"] if f["kind"] == "declared_link")["status"], "reused")
        report["version"] = 99
        (self.root / "checks.json").write_text(json.dumps(report))
        with self.assertRaises(ValueError):
            evidence_sources.assessment_snapshot(self.root)

    def measurement(self) -> tuple[Path, Path]:
        csv = self.root / "raw.csv"
        csv.write_text("sample_measured_value,iteration_count,unit\n100,10,ns\n300,10,ns\n")
        report = {"schema": "process-cost-v5", "measured": True, "profile": {"cargo_profile": "release", "features": ["a"]}, "selected_cases": ["success", "censored", "failed", "missing"], "cases": [{"id": "success", "workload": {"size": 3}, "artifacts": {"copied/raw.csv": hashlib.sha256(csv.read_bytes()).hexdigest()}}, {"id": "censored", "status": "censored", "provider": "other"}, {"id": "failed", "status": "failed"}]}
        path = self.root / "case-measure.json"
        path.write_text(json.dumps(report))
        return path, csv

    def test_actual_samples_censoring_missing_contexts_unknown_invocation_join(self) -> None:
        path, csv = self.measurement()
        snapshot = evidence_sources.measurement_snapshot(path, criterion={"success": csv})
        populations = [f for f in snapshot["facts"] if f["kind"] == "measurement_population"]
        self.assertEqual([f["status"] for f in populations], ["measured", "censored", "failed", "unreported"])
        self.assertEqual([f["value_ns"] for f in snapshot["facts"] if f["kind"] == "criterion_sample"], [10.0, 30.0])
        self.assertTrue(all(f["status"] == "unavailable" for f in snapshot["facts"] if f["kind"] == "measurement_association"))
        self.assertEqual(len({f["context_id"] for f in populations}), 4)
        self.assertEqual(snapshot["coverage"], "partial")
        no_samples = evidence_sources.measurement_snapshot(path)
        self.assertFalse(any(f["kind"] == "criterion_sample" for f in no_samples["facts"]))
        csv.write_text("sample_measured_value,iteration_count,unit\n200,10,ns\n300,10,ns\n")
        with self.assertRaisesRegex(ValueError, "authentic"):
            evidence_sources.measurement_snapshot(path, criterion={"success": csv})

    def test_explicit_invocation_population_and_original_reported_statistics(self) -> None:
        path, _csv = self.measurement()
        report = json.loads(path.read_text())
        records = self.root / "observations.jsonl"
        records.write_text("\n".join(json.dumps(row) for row in [{"invocation_sequence": 0, "outcome": "panicked_incomplete", "seconds": 2}, {"invocation_sequence": 1, "outcome": "censored"}]))
        report["cases"][0].update(iterations=2, criterion_statistics={"mean_nanoseconds": 42.0}, artifacts={"observations.jsonl": hashlib.sha256(records.read_bytes()).hexdigest()})
        path.write_text(json.dumps(report))
        snapshot = evidence_sources.measurement_snapshot(path, invocations={"success": records})
        rows = [fact for fact in snapshot["facts"] if fact["kind"] == "harness_invocation"]
        self.assertEqual([fact["status"] for fact in rows], ["panicked_incomplete", "censored"])
        self.assertEqual(rows[0]["value_ns"], 2e9)
        self.assertNotIn("value_ns", rows[1])
        self.assertEqual(next(fact for fact in snapshot["facts"] if fact["kind"] == "measurement_summary")["value_ns"], 42.0)
        self.assertFalse(any(fact["kind"] == "criterion_sample" for fact in snapshot["facts"]))
        context = json.loads(next(fact["context_json"] for fact in snapshot["facts"] if fact["kind"] == "context"))
        self.assertIsNone(context["build"])
        self.assertIsNone(context["provider"])
        report["cases"][0]["iterations"] = 3
        path.write_text(json.dumps(report))
        with self.assertRaisesRegex(ValueError, "count"):
            evidence_sources.measurement_snapshot(path, invocations={"success": records})

    def test_actual_unmeasured_smoke_codec_cannot_become_performance_samples(self) -> None:
        path = self.root / "case-smoke.json"
        path.write_text(json.dumps({"schema": "case-smoke-v1", "measured": False, "selected_cases": ["one", "two"], "profile": "release", "recording_mode": "selected", "review_artifacts": False, "capture_from_start": False, "compiled_evaluator": False, "binaries": {"native_process": "/declared/binary"}}))
        snapshot = evidence_sources.measurement_snapshot(path)
        self.assertEqual([fact["status"] for fact in snapshot["facts"] if fact["kind"] == "measurement_population"], ["not_measured", "not_measured"])
        with self.assertRaisesRegex(ValueError, "no Criterion"):
            evidence_sources.measurement_snapshot(path, criterion={"one": self.root / "absent"})

    def test_host_owner_readonly_snapshot_and_foreign_version(self) -> None:
        directory = self.root / "host"
        resource_store.protected(directory)
        lock = directory / ".lock"
        lock.touch(mode=0o600)
        resource_store.publish(directory / "allocations.json", {"version": 1, "owners": {"a" * 32: {"boot": "00000000-0000-0000-0000-000000000000", "pid": 123, "start": "77", "class": "build", "lane": "build", "memory": 0, "slots": 1, "cores": [0], "exclusive": False, "deadline": 99999999999, "units": {}}}})
        before = (directory / "allocations.json").read_bytes()
        snapshot = evidence_sources.host_snapshot(directory)
        self.assertEqual(next(fact for fact in snapshot["facts"] if fact["kind"] == "host_allocation")["status"], "observed")
        self.assertEqual((directory / "allocations.json").read_bytes(), before)
        resource_store.publish(directory / "allocations.json", {"version": 2, "owners": {}})
        with self.assertRaises(ValueError):
            evidence_sources.host_snapshot(directory)

    def test_wrapper_offline_before_state_service_and_no_implicit_build(self) -> None:
        handle = test_resources.invocation("python", [{"name": "suite::missing"}])
        def run(tool: Path, arguments: list[str], *, deadline: float) -> int:
            source = Path(arguments[arguments.index("--input") + 1])
            self.assertEqual(json.loads(source.read_text())["facts"][1]["status"], "unreported")
            return 0
        with patch.object(evidence_store, "run_offline", side_effect=run), patch.object(evidence_store, "run_native", side_effect=AssertionError("service")):
            self.assertEqual(evidence_store.main(["offline", "--invocation", str(handle), "--registry", str(self.registry)]), 0)
        # An actual tiny supplied executable proves Python dispatch has no service or build.
        tool = self.root / "explicit-tool"
        tool.write_text(f'#!{sys.executable}\nimport sys\nassert sys.argv[1:3] == ["evidence", "offline"]\n')
        tool.chmod(0o700)
        with patch.object(evidence_store.surreal_server, "tooling_activity", side_effect=AssertionError("service")):
            self.assertEqual(evidence_store.main(["--tool", str(tool), "offline", "--invocation", str(handle), "--registry", str(self.registry)]), 0)
        with contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(evidence_store.main(["--tool", str(self.root / "absent"), "offline", "--invocation", str(handle), "--registry", str(self.registry)]), 1)


if __name__ == "__main__":
    unittest.main()
