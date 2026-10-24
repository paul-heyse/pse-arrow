# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Protected assessment selections survive the actual typed MCP read boundary."""

# ruff: noqa: PT009, PT027 -- stdlib cross-owner controls
from __future__ import annotations

import tempfile
import unittest
from dataclasses import asdict
from pathlib import Path
from unittest.mock import patch

from fastmcp import Client
from fastmcp.exceptions import ToolError
from jsonschema import Draft202012Validator

from scripts import (
    assessment_store,
    test_resources,
    validation_receipts,
    validation_result,
)
from scripts.capabilities.server import build
from scripts.tests import assessment_fixture
from scripts.validation_scope import Gate, input_identity

ROOT = Path(__file__).resolve().parents[2]


class AssessmentBoundaryTests(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.assessment_dir = self.root / "build" / "assessment"
        self.assessment_dir.mkdir(mode=0o700, parents=True)
        self.enterContext(
            patch.object(
                test_resources, "registry", return_value=self.root / "registry"
            )
        )
        self.receipt = assessment_fixture.publish(
            self.assessment_dir,
            {
                "complete": True,
                "required_checks_covered": True,
                "source_files": {str(i): "a" * 64 for i in range(20000)},
                "scope": [{"name": "selected"}, {"name": "unrelated"}],
                "checks": [
                    {
                        "gate": "selected",
                        "status": "failed",
                        "exit_code": 1,
                        "report_errors": ["original scientific refusal"],
                        "results": [{"nodeid": "real-subject", "status": "failure"}],
                    },
                    {
                        "gate": "unrelated",
                        "status": "passed",
                        "results": [
                            {"nodeid": str(i), "status": "passed"} for i in range(10000)
                        ],
                    },
                ],
            },
        )
        (self.assessment_dir / "provenance.json").write_text("{}")
        self.resource = test_resources.register_report(
            self.assessment_dir, root=self.root
        )
        test_resources.finish_report(
            self.resource,
            self.receipt,
            roles={"checks.json": "receipt", "provenance.json": "provenance"},
            required_provenance=["provenance.json"],
        )
        self.client = Client(build(ROOT).server)
        await self.client.__aenter__()
        self.addAsyncCleanup(self.client.__aexit__, None, None, None)
        await self.client.call_tool(
            "capabilities_select", {"capabilities": ["assessment_read"]}
        )
        tool = next(
            item
            for item in await self.client.list_tools()
            if item.name == "assessment_read"
        )
        self.schema = tool.output_schema

    async def read(self, **selection: object) -> dict:
        result = await self.client.call_tool(
            "capabilities_invoke_read",
            {
                "capability": "assessment_read",
                "arguments": {"directory": str(self.assessment_dir), **selection},
            },
        )
        self.assertIsNotNone(result.structured_content)
        if self.schema is None:
            self.fail("Expected the registered output schema")
        Draft202012Validator(self.schema).validate(result.structured_content)
        return result.structured_content["result"]

    async def test_complete_selected_answer_keeps_failed_native_outcome_and_custody(
        self,
    ) -> None:
        image = assessment_store.root(self.assessment_dir)
        chosen = image["checks"][0]["detail"]
        original = assessment_store.detail

        def selected_only(run: Path, reference: object) -> dict:
            self.assertEqual(reference, chosen, "unselected detail was acquired")
            self.assertTrue(test_resources.resource_status(self.resource)["borrows"])
            return original(run, reference)

        with (
            patch.object(assessment_store, "detail", side_effect=selected_only) as read,
            patch(
                "subprocess.run", side_effect=AssertionError("offline read ran setup")
            ),
        ):
            native = validation_result.read_assessment(
                self.assessment_dir, gate="selected"
            )
            selected = await self.read(gate="selected")
        self.assertEqual(read.call_count, 2)
        self.assertTrue(selected["complete"])
        self.assertEqual(selected["answer"]["population"], "complete")
        self.assertEqual(selected["answer"]["detail"], "selected")
        self.assertEqual(
            selected["payload"]["assessment"]["source_cut_ref"], image["source_cut"]
        )
        self.assertEqual(selected["payload"]["scope"], native["scope"])
        check = selected["payload"]["checks"][0]
        self.assertEqual(
            check["record"]["results"], native["checks"][0]["record"]["results"]
        )
        self.assertEqual(
            check["record"]["report_errors"], ["original scientific refusal"]
        )
        self.assertFalse(check["qualified"])
        self.assertEqual(
            selected["payload"]["assessment"]["reader_custody"]["status"], "sealed"
        )
        self.assertFalse(test_resources.resource_status(self.resource)["borrows"])

    async def test_selected_missing_detail_refuses_without_changing_summary(
        self,
    ) -> None:
        image = assessment_store.root(self.assessment_dir)
        path = (
            self.assessment_dir
            / "details"
            / (image["checks"][0]["detail"]["sha256"] + ".json")
        )
        path.unlink()
        with self.assertRaises(ToolError):
            await self.read(gate="selected")
        summary = await self.read()
        self.assertEqual(summary["answer"]["detail"], "omitted")
        self.assertEqual(summary["payload"]["checks"][0]["record"]["status"], "failed")
        self.assertEqual(
            summary["payload"]["assessment"]["source_cut_ref"], image["source_cut"]
        )
        self.assertFalse(test_resources.resource_status(self.resource)["borrows"])

    async def test_source_and_full_export_are_explicit_and_never_silently_truncated(
        self,
    ) -> None:
        image = assessment_store.root(self.assessment_dir)
        original = assessment_store.detail
        with patch.object(assessment_store, "detail", wraps=original) as read:
            source = await self.read(source_detail=True)
        self.assertEqual(
            [call.args[1] for call in read.call_args_list], [image["source_cut"]]
        )
        self.assertEqual(source["outcome"], "detail-refused")
        self.assertFalse(source["complete"])
        with patch.object(assessment_store, "detail", wraps=original) as read:
            full = validation_result.read_assessment(
                self.assessment_dir, full_export=True
            )
        self.assertEqual(
            [call.args[1] for call in read.call_args_list],
            [image["source_cut"], *(check["detail"] for check in image["checks"])],
        )
        self.assertEqual(
            full["assessment"]["source_files"], self.receipt["source_files"]
        )
        self.assertEqual(
            [row["record"] for row in full["checks"]], self.receipt["checks"]
        )
        # Export did not mutate either the summary or immutable source attribution.
        self.assertEqual(assessment_store.root(self.assessment_dir), image)
        self.assertFalse(test_resources.resource_status(self.resource)["borrows"])

    async def test_reused_gate_keeps_older_origin_protected_before_publication_and_read(
        self,
    ) -> None:
        origin = self.root / "build" / "older-origin"
        origin.mkdir(mode=0o700)
        declaration = asdict(Gate("selected"))
        snapshot = {"model": "original-model-digest"}
        receipt = assessment_fixture.publish(
            origin,
            {
                "complete": True,
                "required_checks_covered": True,
                "input_coverage": True,
                "scope": [declaration],
                "checks": [
                    {
                        "gate": "selected",
                        "status": "passed",
                        "exit_code": 0,
                        "invocation": declaration,
                        "inputs": input_identity(
                            declaration["input_scope"], snapshot, {}
                        ),
                        "report_errors": [],
                        "results": [{"nodeid": "original-subject", "status": "passed"}],
                    }
                ],
            },
        )
        (origin / "provenance.json").write_text("{}")
        resource = test_resources.register_report(origin, root=self.root)
        test_resources.finish_report(
            resource,
            receipt,
            roles={"checks.json": "receipt", "provenance.json": "provenance"},
            required_provenance=["provenance.json"],
        )
        consumer = self.root / "build" / "reused-assessment"
        consumer.mkdir(mode=0o700)
        retained = validation_receipts.reuse_checks(
            origin,
            snapshot,
            {},
            [declaration],
            {"selected"},
            set(),
            None,
            consumer=consumer,
        )
        # The real reuse owner establishes persistent protection before the
        # consumer publishes its root containing original evidence references.
        self.assertIn(
            str(consumer), test_resources.resource_status(resource)["references"]
        )
        assessment_fixture.publish(consumer, {**receipt, "checks": retained})
        (consumer / "provenance.json").write_text("{}")
        current = test_resources.register_report(consumer, root=self.root)
        test_resources.finish_report(
            current,
            {**receipt, "checks": retained},
            roles={"checks.json": "receipt", "provenance.json": "provenance"},
            required_provenance=["provenance.json"],
        )
        original = validation_result.selected_result

        def protected(
            run: Path,
            receipt: dict,
            *,
            gate: str | None,
            failures: bool,
            tail: int,
            origins: dict[str, Path],
        ) -> dict:
            for identity in (current, resource):
                self.assertTrue(test_resources.resource_status(identity)["borrows"])
            return original(
                run, receipt, gate=gate, failures=failures, tail=tail, origins=origins
            )

        self.assessment_dir = consumer
        with patch.object(validation_result, "selected_result", side_effect=protected):
            selected = await self.read(gate="selected")
        check = selected["payload"]["checks"][0]
        self.assertTrue(selected["complete"])
        self.assertTrue(check["qualified"])
        self.assertEqual(check["origin_path"], str(origin))
        self.assertEqual(check["record"]["results"], receipt["checks"][0]["results"])
        self.assertEqual(check["record"]["evidence_kind"], "unchanged-input-reuse")
        for identity in (current, resource):
            self.assertFalse(test_resources.resource_status(identity)["borrows"])
        self.assertIn(
            str(consumer), test_resources.resource_status(resource)["references"]
        )


if __name__ == "__main__":
    unittest.main()
