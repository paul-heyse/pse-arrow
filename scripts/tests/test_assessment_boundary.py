# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Protected assessment selections survive the actual typed MCP read boundary."""

# ruff: noqa: PT009, PT027 -- stdlib cross-owner controls
from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from fastmcp import Client
from fastmcp.exceptions import ToolError
from jsonschema import Draft202012Validator

from scripts import assessment_store, test_resources, validation_result
from scripts.capabilities.server import build
from scripts.tests import assessment_fixture

ROOT = Path(__file__).resolve().parents[2]


class AssessmentBoundaryTests(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.run = self.root / "build" / "assessment"
        self.run.mkdir(mode=0o700, parents=True)
        self.enterContext(
            patch.object(test_resources, "registry", return_value=self.root / "registry")
        )
        self.receipt = assessment_fixture.publish(
            self.run,
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
                            {"nodeid": str(i), "status": "passed"}
                            for i in range(10000)
                        ],
                    },
                ],
            },
        )
        (self.run / "provenance.json").write_text("{}")
        self.resource = test_resources.register_report(self.run, root=self.root)
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
            item for item in await self.client.list_tools()
            if item.name == "assessment_read"
        )
        self.schema = tool.output_schema

    async def read(self, **selection: object) -> dict:
        result = await self.client.call_tool(
            "capabilities_invoke_read",
            {
                "capability": "assessment_read",
                "arguments": {"directory": str(self.run), **selection},
            },
        )
        self.assertIsNotNone(result.structured_content)
        Draft202012Validator(self.schema).validate(result.structured_content)
        return result.structured_content["result"]

    async def test_complete_selected_answer_keeps_failed_native_outcome_and_custody(
        self,
    ) -> None:
        image = assessment_store.root(self.run)
        chosen = image["checks"][0]["detail"]
        original = assessment_store.detail

        def selected_only(run: Path, reference: object) -> dict:
            self.assertEqual(reference, chosen, "unselected detail was acquired")
            self.assertTrue(test_resources.resource_status(self.resource)["borrows"])
            return original(run, reference)

        with (
            patch.object(assessment_store, "detail", side_effect=selected_only) as read,
            patch("subprocess.run", side_effect=AssertionError("offline read ran setup")),
        ):
            native = validation_result.read_assessment(self.run, gate="selected")
            selected = await self.read(gate="selected")
        self.assertEqual(read.call_count, 2)
        self.assertTrue(selected["complete"])
        self.assertEqual(selected["answer"]["population"], "complete")
        self.assertEqual(selected["answer"]["detail"], "selected")
        self.assertEqual(selected["payload"]["assessment"]["source_cut_ref"], image["source_cut"])
        self.assertEqual(selected["payload"]["scope"], native["scope"])
        check = selected["payload"]["checks"][0]
        self.assertEqual(check["record"]["results"], native["checks"][0]["record"]["results"])
        self.assertEqual(check["record"]["report_errors"], ["original scientific refusal"])
        self.assertFalse(check["qualified"])
        self.assertEqual(selected["payload"]["assessment"]["reader_custody"]["status"], "sealed")
        self.assertFalse(test_resources.resource_status(self.resource)["borrows"])

    async def test_selected_missing_detail_refuses_without_changing_summary(self) -> None:
        image = assessment_store.root(self.run)
        path = self.run / "details" / (image["checks"][0]["detail"]["sha256"] + ".json")
        path.unlink()
        with self.assertRaises(ToolError):
            await self.read(gate="selected")
        summary = await self.read()
        self.assertEqual(summary["answer"]["detail"], "omitted")
        self.assertEqual(summary["payload"]["checks"][0]["record"]["status"], "failed")
        self.assertEqual(summary["payload"]["assessment"]["source_cut_ref"], image["source_cut"])
        self.assertFalse(test_resources.resource_status(self.resource)["borrows"])

    async def test_source_and_full_export_are_explicit_and_never_silently_truncated(
        self,
    ) -> None:
        image = assessment_store.root(self.run)
        original = assessment_store.detail
        with patch.object(assessment_store, "detail", wraps=original) as read:
            source = await self.read(source_detail=True)
        self.assertEqual([call.args[1] for call in read.call_args_list], [image["source_cut"]])
        self.assertEqual(source["outcome"], "detail-refused")
        self.assertFalse(source["complete"])
        with patch.object(assessment_store, "detail", wraps=original) as read:
            full = validation_result.read_assessment(self.run, full_export=True)
        self.assertEqual(
            [call.args[1] for call in read.call_args_list],
            [image["source_cut"], *(check["detail"] for check in image["checks"])],
        )
        self.assertEqual(full["assessment"]["source_files"], self.receipt["source_files"])
        self.assertEqual([row["record"] for row in full["checks"]], self.receipt["checks"])
        # Export did not mutate either the summary or immutable source attribution.
        self.assertEqual(assessment_store.root(self.run), image)
        self.assertFalse(test_resources.resource_status(self.resource)["borrows"])


if __name__ == "__main__":
    unittest.main()
