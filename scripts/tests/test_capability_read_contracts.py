# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent expectations for selected answers, native observations and bounds."""

# ruff: noqa: PT009, PT027 -- stdlib tooling tests
from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from jsonschema import Draft202012Validator
from pydantic import TypeAdapter, ValidationError

from scripts import (
    knowledge_store,
    run_inspection,
    test_resources,
    test_run,
    validation_result,
)
from scripts.capabilities import read_contracts as answers
from scripts.tests import assessment_fixture


class ContractTests(unittest.TestCase):
    def assert_schema(
        self, result: answers.Contract, result_type: type[answers.Contract]
    ) -> dict:
        serialized = json.loads(result.model_dump_json())
        Draft202012Validator(
            TypeAdapter(result_type | answers.DetailRefused).json_schema(
                mode="serialization"
            )
        ).validate(serialized)
        return serialized

    def test_storage_complete_partial_and_unavailable_are_observations_not_reclamation(
        self,
    ) -> None:
        def row(status: str) -> dict:
            return {
                "name": "selected",
                "path": "/selected",
                "ownership": "owner",
                "active_use": "unknown",
                "resource_identity": "unobserved",
                "reclaimable_bytes": None,
                "status": status,
            }

        for statuses, expected in (
            (["metadata-only"], "complete"),
            (["metadata-only", "unavailable"], "partial"),
            (["unavailable"], "unavailable"),
        ):
            with self.subTest(statuses=statuses):
                result = answers.storage(
                    {
                        "items": [row(status) for status in statuses],
                        "reclamation_authorized": False,
                    }
                )
                serialized = self.assert_schema(result, answers.StorageResult)
                self.assertEqual(serialized["answer"]["population"], expected)
                self.assertFalse(serialized["payload"]["reclamation_authorized"])
                self.assertIsNone(
                    serialized["payload"]["items"][0]["reclaimable_bytes"]
                )

    def test_required_fact_is_never_dropped_or_coerced_to_a_null(self) -> None:
        payload = {"items": [], "reclamation_authorized": False}
        with self.assertRaises(ValidationError):
            answers.storage({"items": None, "reclamation_authorized": False})
        with self.assertRaises(ValidationError):
            answers.storage({"items": []})
        with self.assertRaises(ValidationError):
            answers.storage({**payload, "unadvertised_primary_field": "unexpected"})
        result = answers.storage(payload)
        if not isinstance(result, answers.StorageResult):
            self.fail("Expected the selected storage answer")
        self.assertEqual(result.payload.items, [])

    def test_oversized_selected_answer_is_explicit_refusal_without_rewritten_payload(
        self,
    ) -> None:
        payload = {
            "items": [
                {
                    "name": "selected",
                    "path": "\u03b1" * 100000,
                    "ownership": "owner",
                    "active_use": "unknown",
                    "resource_identity": "unobserved",
                    "reclaimable_bytes": None,
                    "status": "metadata-only",
                }
            ],
            "reclamation_authorized": False,
        }
        original = json.dumps(payload)
        result = answers.storage(payload)
        serialized = self.assert_schema(result, answers.StorageResult)
        self.assertEqual(serialized["outcome"], "detail-refused")
        self.assertFalse(serialized["complete"])
        self.assertNotIn("payload", serialized)
        self.assertGreater(serialized["encoded_bytes"], serialized["maximum_bytes"])
        self.assertEqual(original, json.dumps(payload))

    def test_selected_source_excerpts_have_exact_population_and_full_import_stays_full(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "owner.md").write_text("\n".join(f"line {n}" for n in range(200)))
            complete = knowledge_store.corpus(("owner.md",), root=root)
            selected = knowledge_store.corpus(
                ("owner.md",), root=root, excerpt_offset=1, excerpt_limit=1
            )
            self.assertNotIn("excerpt_page", complete)
            self.assertEqual(
                selected["excerpt_page"],
                {
                    "offset": 1,
                    "limit": 1,
                    "total": len(complete["excerpts"]),
                    "next_offset": 2,
                },
            )
            self.assertEqual(
                [
                    {
                        key: value
                        for key, value in item.items()
                        if key != "text_complete"
                    }
                    for item in selected["excerpts"]
                ],
                complete["excerpts"][1:2],
            )
            self.assertEqual(selected["sources"], complete["sources"])
            result = answers.knowledge(selected)
            serialized = self.assert_schema(result, answers.KnowledgeResult)
            self.assertEqual(serialized["answer"]["population"], "partial")
            self.assertEqual(serialized["payload"]["excerpt_page"]["total"], 5)

    def test_final_concept_page_retains_partial_population_even_without_next_cursor(
        self,
    ) -> None:
        payload = {
            "schema": "workspace-domains-v1",
            "subjects": [],
            "edges": [],
            "productions": [],
            "source_facts": [],
            "section_ranges": [],
            "native_facts": [],
            "complete": True,
            "omissions": [],
            "unknowns": [],
            "coverage": "exact selected owners",
            "query": {"concept": "document"},
            "items": [],
            "next_offset": None,
            "adapter_status": "available",
            "population": {
                "matched_count": 2,
                "returned_count": 0,
                "offset": 2,
                "limit": 1,
                "complete": True,
            },
        }
        result = answers.concept_query(payload)
        self.assert_schema(result, answers.ConceptQueryResult)
        if not isinstance(result, answers.ConceptQueryResult):
            self.fail("Expected the selected concept answer")
        self.assertEqual(result.answer.population, "partial")
        self.assertFalse(result.complete)
        self.assertTrue(result.payload.population.complete)
        self.assertIsNone(result.payload.next_offset)

    def test_original_excerpt_text_bound_is_explicit_optional_detail_omission(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "owner.md").write_text("x" * 20000)
            selected = knowledge_store.corpus(("owner.md",), root=root, excerpt_limit=1)
            result = answers.knowledge(selected)
            self.assert_schema(result, answers.KnowledgeResult)
            if not isinstance(result, answers.KnowledgeResult):
                self.fail("Expected the selected knowledge answer")
            self.assertEqual(result.answer.population, "complete")
            self.assertEqual(result.answer.detail, "omitted")
            self.assertFalse(result.payload.excerpts[0].text_complete)
            self.assertEqual(len(result.payload.excerpts[0].text), 16000)
            self.assertEqual(result.payload.sources[0].locator, "repo:owner.md")

    def test_product_population_and_workload_outcome_remain_independent(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            registry = Path(directory) / "registry"
            with patch.object(test_resources, "registry", return_value=registry):
                invocation = test_resources.invocation(
                    "python", [{"name": "one"}, {"name": "two"}]
                )
                missing = answers.product_run(
                    run_inspection.product_result(
                        str(invocation), registry=registry, limit=1
                    )
                )
                self.assert_schema(missing, answers.ProductRunResult)
                if not isinstance(missing, answers.ProductRunResult):
                    self.fail("Expected the selected ProductRunResult answer")
                self.assertEqual(missing.answer.population, "partial")
                self.assertEqual(missing.payload.selected_count, 2)
                self.assertEqual(missing.payload.subjects[0].outcome, "unreported")
                test_run.finish(
                    invocation,
                    {
                        "exit_code": 1,
                        "results": [
                            {"nodeid": "one", "status": "failure"},
                            {"nodeid": "two", "status": "passed"},
                        ],
                        "report_errors": [],
                    },
                    "runner",
                )
                failed = answers.product_run(
                    run_inspection.product_result(
                        str(invocation), registry=registry, subject="one"
                    )
                )
                self.assert_schema(failed, answers.ProductRunResult)
                if not isinstance(failed, answers.ProductRunResult):
                    self.fail("Expected the selected ProductRunResult answer")
                self.assertEqual(failed.payload.workload_exit, 1)
                if failed.payload.terminal is None:
                    self.fail("Expected the original terminal observation")
                self.assertTrue(failed.payload.terminal.complete)
                self.assertEqual(failed.payload.subjects[0].outcome, "failure")
                self.assertEqual(failed.payload.page.subject_count, 1)
                self.assertEqual(failed.payload.selected_count, 2)

    def test_assessment_summary_is_complete_population_with_optional_detail_omitted(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            run = root / "assessment"
            run.mkdir()
            (run / "unit.log").write_text("done\n")
            record = {
                "version": 6,
                "mode": "local",
                "baseline_failures": 0,
                "complete": True,
                "required_checks_covered": True,
                "source_unchanged": True,
                "provenance_errors": [],
                "input_coverage": True,
                "evidence": "Tested",
                "parent": None,
                "scope": [{"name": "unit", "role": "required"}],
                "checks": [
                    {
                        "gate": "unit",
                        "status": "passed",
                        "role": "required",
                        "log": "unit.log",
                        "artifacts": {},
                        "evidence_kind": "executed",
                        "exit_code": 0,
                        "report_errors": [],
                        "results": [],
                    }
                ],
            }
            with patch.object(
                test_resources, "registry", return_value=root / "registry"
            ):
                assessment_fixture.publish(run, record)
                summary = answers.assessment(validation_result.read_assessment(run))
                selected = answers.assessment(
                    validation_result.read_assessment(run, gate="unit")
                )
            self.assert_schema(summary, answers.AssessmentResult)
            if not isinstance(summary, answers.AssessmentResult):
                self.fail("Expected the selected AssessmentResult answer")
            self.assert_schema(selected, answers.AssessmentResult)
            if not isinstance(selected, answers.AssessmentResult):
                self.fail("Expected the selected AssessmentResult answer")
            self.assertEqual(summary.answer.population, "complete")
            self.assertEqual(summary.answer.detail, "omitted")
            self.assertFalse(summary.complete)
            self.assertTrue(summary.payload.assessment.complete)
            self.assertEqual(selected.answer.detail, "selected")
            self.assertTrue(selected.payload.checks[0].qualified)


if __name__ == "__main__":
    unittest.main()
