# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Fault controls for the assessor; synthetic events are never host evidence."""

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from scripts.tests.mcp_host_acceptance import assess


class HostEvidenceTests(unittest.TestCase):
    def fixture(self, *, value: str = "fresh", extra: bool = False, reset: bool = True) -> tuple[list[dict], dict]:
        arguments = {"capability": "native_result_inspect", "arguments": {"request": "fixture"}}
        case = {"workflow": "manage-project", "steps": [{"capability": "native_result_inspect", "arguments": {"request": "fixture"}, "assertions": [{"path": ["document", "rows", 0, "values", 0, "value"], "value": "fresh"}]}]}
        rows = [{"type": "thread.started"}]
        calls = [
            ("capabilities_select", {"workflows": ["manage-project"]}, {"instruction_resources": [{"uri": "pse://guidance/storage"}]}),
            ("read_resource", {"uri": "pse://guidance/storage"}, {"result": "original fixture instruction"}),
            ("get_prompt", {"name": "workflow_manage-project"}, {"result": "fixture prompt"}),
            ("capabilities_describe", {"capability": "native_result_inspect", "include_schema": True}, {}),
            ("capabilities_invoke_read", arguments, {"payload": {"document": {"rows": [{"values": [{"value": value}]}]}}}),
        ]
        if extra:
            calls.append(calls[-1])
        if reset:
            calls.append(("capabilities_reset", {}, {}))
        for index, (tool, supplied, structured) in enumerate(calls):
            item = {"id": str(index), "type": "mcp_tool_call", "server": "pse-capabilities", "tool": tool, "arguments": supplied}
            rows.append({"type": "item.started", "item": item})
            rows.append({"type": "item.completed", "item": {**item, "status": "completed", "error": None, "result": {"structured_content": structured}}})
        rows.append({"type": "turn.completed"})
        return rows, case

    def check(self, rows: list[dict], case: dict) -> dict:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "events.jsonl"
            path.write_text("\n".join(json.dumps(row) for row in rows) + "\n")
            return assess(path, case)

    def test_exact_nested_value_and_original_request_pass(self) -> None:
        rows, case = self.fixture()
        self.assertEqual(self.check(rows, case)["status"], "passed")

    def test_delivered_wrong_cell_is_not_a_pass(self) -> None:
        rows, case = self.fixture(value="stale")
        with self.assertRaisesRegex(ValueError, "expectation differs"):
            self.check(rows, case)

    def test_repeated_operation_cannot_be_hidden_by_success(self) -> None:
        rows, case = self.fixture(extra=True)
        with self.assertRaisesRegex(ValueError, "omitted or replayed"):
            self.check(rows, case)

    def test_missing_reset_is_not_completed_acceptance(self) -> None:
        rows, case = self.fixture(reset=False)
        with self.assertRaisesRegex(ValueError, "finish with reset"):
            self.check(rows, case)

    def test_missing_terminal_cannot_be_counted_as_completed_host(self) -> None:
        rows, case = self.fixture()
        with self.assertRaisesRegex(ValueError, "incomplete"):
            self.check(rows[:-1], case)

    def test_noncanonical_execution_cannot_enter_the_evidence(self) -> None:
        rows, case = self.fixture()
        rows.insert(1, {"type": "item.completed", "item": {"type": "command_execution"}})
        with self.assertRaisesRegex(ValueError, "noncanonical"):
            self.check(rows, case)

    def test_missing_schema_description_cannot_pass(self) -> None:
        rows, case = self.fixture()
        rows = [row for row in rows if row.get("item", {}).get("tool") != "capabilities_describe"]
        with self.assertRaisesRegex(ValueError, "schema not described"):
            self.check(rows, case)

    def test_missing_owned_guidance_cannot_pass(self) -> None:
        rows, case = self.fixture()
        rows = [row for row in rows if row.get("item", {}).get("tool") != "read_resource"]
        with self.assertRaisesRegex(ValueError, "guidance not read"):
            self.check(rows, case)
