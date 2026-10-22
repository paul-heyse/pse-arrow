# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent invocation documents retain result, readability and protection meanings."""
# ruff: noqa: PT009, PT027

from __future__ import annotations

import contextlib
import io
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import resource_store, run_inspection, test_resources, test_run, validation_result


class RunInspectionTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.registry = self.root / "registry"
        selected = patch.object(test_resources, "registry", return_value=self.registry)
        selected.start()
        self.addCleanup(selected.stop)
        self.path = test_resources.invocation("python", [{"name": "suite::one"}, {"name": "suite::two"}])

    def freeze(self, *, code: int = 0, errors: list[str] | None = None, rows: list | None = None, **extra: object) -> None:
        test_run.finish(self.path, {"exit_code": code, "results": rows if rows is not None else [
            {"nodeid": "suite::one", "status": "passed"}, {"nodeid": "suite::two", "status": "passed"}],
            "report_errors": errors or [], **extra}, "runner")

    def read(self, **options: object) -> dict:
        return run_inspection.product_result(str(self.path), **options)

    def test_pass_and_complete_failure_have_authentic_frozen_identity(self) -> None:
        self.freeze(code=1, rows=[{"nodeid": "suite::one", "status": "failure"}, {"nodeid": "suite::two", "status": "passed"}])
        result = self.read()
        self.assertTrue(result["terminal"]["complete"])
        self.assertEqual(result["workload_exit"], 1)
        self.assertEqual(result["terminal_identity"], test_resources.storage().terminal(self.path.stem)[0])
        self.assertEqual(result["selected_unreported"], [])
        self.assertEqual(self.read(failures=True)["subjects"][0]["outcome"], "failure")
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            self.assertEqual(validation_result.main([str(self.path), "--kind", "product-test", "--json"]), 0)
        self.assertEqual(json.loads(output.getvalue())["workload_exit"], 1)

    def test_interruption_and_missing_terminal_never_invent_subject_outcomes(self) -> None:
        missing = self.read()
        self.assertEqual(missing["terminal_status"], "missing")
        self.assertIsNone(missing["terminal"])
        self.freeze(code=130, errors=["missing report"], rows=[{"nodeid": "suite::one", "status": "passed"}], process={"forwarded_signal": 2}, owned_drain="unresolved")
        result = self.read()
        self.assertFalse(result["terminal"]["complete"])
        self.assertEqual(result["selected_unreported"], ["suite::two"])
        self.assertEqual(result["subjects"][1]["outcome"], "unreported")
        self.assertEqual(result["process"]["forwarded_signal"], 2)
        self.assertEqual(result["owned_drain"], "unresolved")

    def test_frozen_report_mismatch_and_malformed_authority_are_explicit(self) -> None:
        self.freeze(rows=[{"nodeid": "other", "status": "passed"}])
        result = self.read()
        self.assertFalse(result["terminal"]["complete"])
        self.assertIn("outside selected catalog", result["report_errors"][0])
        state_path = self.registry / "invocation-state" / self.path.name
        image = json.loads(state_path.read_text())
        image["terminal"]["complete"] = True
        resource_store.publish(state_path, image)
        with self.assertRaisesRegex(ValueError, "Invalid frozen"):
            self.read()

    def test_bounded_exact_selection_and_registry_refusal(self) -> None:
        self.freeze()
        self.assertEqual(self.read(offset=1, limit=1)["subjects"][0]["identity"], "suite::two")
        exact = run_inspection.product_result(self.path.stem, registry=self.registry)
        self.assertEqual(exact["invocation"], self.path.stem)
        with self.assertRaisesRegex(ValueError, "selected registry"):
            self.read(registry=self.root / "unrelated")
        with self.assertRaisesRegex(ValueError, "selected catalog"):
            self.read(subject="invented")

    def test_detached_metadata_does_not_initialize_or_modify_registry(self) -> None:
        before = {p: p.read_bytes() for p in self.registry.rglob("*") if p.is_file()}
        with patch.object(resource_store.FileStore, "ensure", side_effect=AssertionError("ensure")):
            self.read()
        self.assertEqual(before, {p: p.read_bytes() for p in self.registry.rglob("*") if p.is_file()})

    def test_payload_tail_holds_report_borrow_until_read_and_releases_it(self) -> None:
        directory = self.root / "build" / "report"
        directory.mkdir(parents=True)
        log = directory / "failure.log"
        log.write_text("old\nnew\n")
        resource = test_resources.register_report(directory, root=self.root)
        self.freeze(code=1, artifacts={"failure.log": test_resources.file_digest(log)}, artifact_base=str(directory))
        original = run_inspection.artifact_observation
        def guarded(path: Path, digest: str, lines: int, record: dict | None) -> dict:
            self.assertTrue(test_resources.resource_status(resource)["borrows"])
            return original(path, digest, lines, record)
        with patch.object(run_inspection, "artifact_observation", side_effect=guarded):
            self.assertEqual(self.read(tail=1)["payloads"][0]["tail"], ["new"])
        self.assertEqual(test_resources.resource_status(resource)["borrows"], {})
        with patch.object(run_inspection, "artifact_observation", side_effect=OSError("read error")), self.assertRaises(OSError):
            self.read(tail=1)
        self.assertEqual(test_resources.resource_status(resource)["borrows"], {})
        log.write_text("changed payload")
        self.assertEqual(self.read(tail=1)["payloads"][0]["status"], "changed")
        log.unlink()
        log.symlink_to(self.root / "outside")
        (self.root / "outside").write_text("not owned")
        self.assertEqual(self.read(tail=1)["payloads"][0]["status"], "unavailable")


if __name__ == "__main__":
    unittest.main()
