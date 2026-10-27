# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Actual responsibility-owned comparison projections retain finite meaning."""
# ruff: noqa: PT009, PT027 -- stdlib host controls

from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path

import msgspec
from jsonschema import Draft202012Validator

from scripts.generated.evidence_comparison_output import (
    EvidenceComparison,
    EvidenceComparisonClassification,
    EvidenceComparisonCorrespondence,
    EvidenceComparisonSide,
    EvidenceMeasurementConditions,
)
from scripts.generated.evidence_comparison_request_input import (
    EvidenceComparisonRequest,
    EvidenceComparisonSection,
    EvidenceSectionSelection,
)
from scripts.tests.test_evidence_comparison_report import fixture

ROOT = Path(__file__).resolve().parents[2]


class OwnerBoundaryTests(unittest.TestCase):
    def test_actual_request_schema_and_msgspec_reject_unknown_section_keys(self) -> None:
        schema = json.loads((ROOT / "scripts/generated/evidence_schema/evidence-comparison-request.input.schema.json").read_bytes())
        validator = Draft202012Validator(schema)
        request = {
            "version": 1,
            "baseline": {"generations": ["baseline"], "crate_name": None, "context_id": None},
            "candidate": {"generations": ["candidate"], "crate_name": None, "context_id": None},
            "sections": {},
        }
        for section in EvidenceComparisonSection:
            request["sections"] = {section.value: {"limit": 1, "offset": 0}}
            validator.validate(request)
            decoded = msgspec.json.decode(msgspec.json.encode(request), type=EvidenceComparisonRequest)
            self.assertEqual(list(decoded.sections), [section])
            typed = msgspec.structs.replace(decoded, sections={section: EvidenceSectionSelection(limit=2, offset=0)})
            self.assertEqual(json.loads(msgspec.json.encode(typed))["sections"], {section.value: {"limit": 2, "offset": 0}})
        request["sections"] = {"not_a_section": {"limit": 1, "offset": 0}}
        self.assertTrue(list(validator.iter_errors(request)))
        with self.assertRaises(msgspec.ValidationError):
            msgspec.json.decode(msgspec.json.encode(request), type=EvidenceComparisonRequest)

    def test_actual_result_schema_and_python_keep_closed_judgments(self) -> None:
        schema = json.loads((ROOT / "scripts/generated/evidence_schema/evidence-comparison.output.schema.json").read_bytes())
        validator = Draft202012Validator(schema)
        original = fixture()
        validator.validate(original)
        result = msgspec.json.decode(msgspec.json.encode(original), type=EvidenceComparison)
        assert result.subjects is not None
        assert result.samples is not None
        self.assertIs(result.subjects.rows[0].classification, EvidenceComparisonClassification.RESOLVED_FAILURE)
        self.assertIs(result.subjects.rows[0].correspondence, EvidenceComparisonCorrespondence.EXACT_COMPATIBLE_OWNER_IDENTITY)
        self.assertIs(result.samples.rows[0].side, EvidenceComparisonSide.BASELINE)
        for record in (EvidenceComparisonClassification, EvidenceComparisonCorrespondence, EvidenceComparisonSide, EvidenceMeasurementConditions):
            for value in record:
                self.assertIs(msgspec.json.decode(msgspec.json.encode(value.value), type=record), value)
            with self.assertRaises(msgspec.ValidationError):
                msgspec.json.decode(b'"future_closed_value"', type=record)
        for section, field in (("subjects", "classification"), ("subjects", "correspondence"), ("samples", "side")):
            invalid = copy.deepcopy(original)
            invalid[section]["rows"][0][field] = "future_closed_value"
            self.assertTrue(list(validator.iter_errors(invalid)))
            with self.assertRaises(msgspec.ValidationError):
                msgspec.json.decode(msgspec.json.encode(invalid), type=EvidenceComparison)
        original["samples"]["rows"][0]["fact"]["status"] = "original_future_owner_status"
        validator.validate(original)
        result = msgspec.json.decode(msgspec.json.encode(original), type=EvidenceComparison)
        assert result.samples is not None
        self.assertEqual(result.samples.rows[0].fact.status, "original_future_owner_status")
