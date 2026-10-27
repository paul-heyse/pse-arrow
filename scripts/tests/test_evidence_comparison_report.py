# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Rendering controls use the original generated comparison codec, not a mirror."""
# ruff: noqa: PT009, PT027 -- standard-library artifact controls

from __future__ import annotations

import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import evidence_comparison_report as report


def fixture() -> dict:
    """Controlled rendering data; these counts are not executed product evidence."""
    denominator = {
        "selected_tests": 3,
        "declared_crate_tests": 0,
        "declared_context_tests": 0,
        "failures": 1,
        "successes": 1,
        "unexecuted": 1,
        "other_outcomes": 0,
        "samples": 2,
        "selected_members": 5,
    }
    summary = dict.fromkeys(
        (
            "paired",
            "new_failures",
            "resolved_failures",
            "persistent_failures",
            "unexecuted",
            "inconclusive",
            "unchanged_successes",
            "unmatched_baseline",
            "unmatched_candidate",
            "ambiguous",
            "mixed_outcome_cohorts",
        ),
        0,
    )
    summary.update(
        paired=1,
        resolved_failures=1,
        unexecuted=1,
        unmatched_baseline=1,
        unmatched_candidate=1,
        changed_selection_denominator=False,
        selection_membership_changed_or_unresolved=True,
    )
    counts = {"failures": 0, "successes": 0, "unexecuted": 0, "other": 0}
    identity = {
        "authority": "controlled-rendering-fixture",
        "codec": "owner-codec",
        "kind": "rust_test",
        "crate_name": None,
        "binary_id": "original-binary",
        "test_id": "failure-to-pass",
    }
    page = {
        "cut": "controlled-rendering-cut",
        "total_rows": 3,
        "offset": 0,
        "limit": 1,
        "complete": False,
        "next_offset": 1,
    }
    samples = []
    for side, values in (("baseline", (10.0, 30.0)), ("candidate", (11.0, 29.0))):
        for index, value in enumerate(values):
            fact = dict.fromkeys(
                (
                    "crate_name",
                    "test_id",
                    "detail_ref",
                    "binary_id",
                    "context_id",
                    "context_json",
                    "link_relation",
                    "link_target",
                    "population_id",
                    "sample_id",
                    "value_ns",
                )
            )
            fact.update(
                member=f"sample:{index}",
                kind="criterion_sample",
                status="measured",
                population_id="original-population",
                sample_id=f"owner-row:{index}",
                value_ns=value,
            )
            samples.append(
                {"side": side, "generation": side + "-generation", "fact": fact}
            )
    source = {
        "generation": "original-generation",
        "authority": "controlled-rendering-fixture",
        "codec": "owner-codec",
        "source_id": "original-owner-source",
        "snapshot_digest": "a" * 64,
        "coverage": "partial",
        "expected_members": 5,
        "freshness": "unknown",
    }
    return {
        "version": 1,
        "cut": page["cut"],
        "baseline_sources": [source],
        "candidate_sources": [source],
        "baseline_denominator": denominator,
        "candidate_denominator": denominator,
        "summary": summary,
        "measurement_summary": {
            "populations": 1,
            "comparable": 0,
            "incompatible": 1,
            "unavailable": 0,
            "unavailable_association_populations": 1,
            "verdict": "inconclusive",
        },
        "interpretation": "Controlled rendering fixture <script>alert('source')</script>; no actual product outcomes.",
        "subjects": {
            **page,
            "rows": [
                {
                    "cohort": "resolved-cohort",
                    "correspondence": "exact_compatible_owner_identity",
                    "classification": "resolved_failure",
                    "baseline": identity,
                    "candidate": identity,
                    "baseline_outcomes": {**counts, "failures": 1},
                    "candidate_outcomes": {**counts, "successes": 1},
                    "mixed_outcomes": False,
                }
            ],
        },
        "members": None,
        "samples": {
            **page,
            "rows": samples,
            "total_rows": 4,
            "limit": 4,
            "complete": True,
            "next_offset": None,
        },
        "measurements": None,
        "contexts": None,
        "associations": None,
    }


class EvidenceComparisonReportTests(unittest.TestCase):
    def setUp(self) -> None:
        self.root = Path(self.enterContext(tempfile.TemporaryDirectory()))
        self.source = self.root / "original.json"
        self.source.write_text(json.dumps(fixture()))
        self.enterContext(patch.dict(sys.modules, {"pse": None}))

    def test_headless_report_preserves_full_denominators_limits_original_ids_and_escaping(
        self,
    ) -> None:
        before = self.source.read_bytes()
        with patch.object(
            report, "_plots", side_effect=AssertionError("plots not selected")
        ):
            result = report.write_report(self.source, self.root / "report", plots=False)
        body = Path(report.text_value(result["html"])).read_text()
        for value in (
            "selected_tests",
            "resolved_failures",
            "unmatched_candidate",
            "unexecuted",
            "inconclusive",
            "original-binary",
            "owner-row:0",
            "1/3 rows",
            "not requested",
            "../original.json",
            "Original source cut",
        ):
            self.assertIn(value, body)
        self.assertNotIn("<script>", body)
        self.assertIn("&lt;script&gt;", body)
        self.assertIn("none (end of section)", body)
        self.assertEqual(result["original_sha256"], hashlib.sha256(before).hexdigest())
        self.assertEqual(self.source.read_bytes(), before)
        self.assertEqual(result["plots"], [])

    def test_standalone_plots_use_native_counts_and_leave_inconclusive_interpretation(
        self,
    ) -> None:
        result = report.write_report(self.source, self.root / "report")
        paths = [Path(name) for name in report._strings(result["plots"])]  # noqa: SLF001 -- strict original report list decoder
        self.assertEqual(
            {path.name for path in paths},
            {"outcomes.svg", "outcomes.png", "samples.svg", "samples.png"},
        )
        for path in paths:
            self.assertGreater(path.stat().st_size, 1000)
            if path.suffix == ".png":
                self.assertTrue(path.read_bytes().startswith(b"\x89PNG"))
        self.assertIn("inconclusive", (self.root / "report/samples.svg").read_text())
        self.assertIn("no pairing", (self.root / "report/samples.svg").read_text())

    def test_foreign_cut_and_unknown_scientific_fields_refuse_before_publication(
        self,
    ) -> None:
        value = fixture()
        report.object_value(value["subjects"])["cut"] = "other-cut"
        self.source.write_text(json.dumps(value))
        with self.assertRaisesRegex(ValueError, "another comparison cut"):
            report.write_report(self.source, self.root / "report", plots=False)
        self.assertFalse((self.root / "report").exists())
        value = fixture()
        value["invented_scientific_verdict"] = "pass"
        self.source.write_text(json.dumps(value))
        with self.assertRaises(ValueError):
            report.write_report(self.source, self.root / "report", plots=False)
        self.assertFalse((self.root / "report").exists())

    def test_existing_destination_is_preserved(self) -> None:
        selected = self.root / "report"
        selected.mkdir()
        original = selected / "original.txt"
        original.write_text("preserve")
        with self.assertRaises(FileExistsError):
            report.write_report(self.source, selected, plots=False)
        self.assertEqual(original.read_text(), "preserve")

    def test_native_local_refs_and_resource_id_resolve_without_network_retrieval(
        self,
    ) -> None:
        original = report.SchemaSet(report.ROOT, family="evidence")
        schema = original.document("evidence-comparison", "output")
        schema["$id"] = "https://example.invalid/native-owned-resource"
        with patch.object(report, "SchemaSet") as owner:
            owner.return_value.document.return_value = schema
            owner.return_value.identity.return_value = original.identity(
                "evidence-comparison", "output"
            )
            result = report.write_report(
                self.source, self.root / "local-resource", plots=False
            )
        self.assertTrue(Path(report.text_value(result["html"])).exists())
        report.object_value(schema["properties"])["summary"] = {
            "$ref": "https://example.invalid/never-retrieved"
        }
        with patch.object(report, "SchemaSet") as owner:
            owner.return_value.document.return_value = schema
            with self.assertRaisesRegex(ValueError, "native output schema"):
                report.write_report(self.source, self.root / "remote-ref", plots=False)
        self.assertFalse((self.root / "remote-ref").exists())


if __name__ == "__main__":
    unittest.main()
