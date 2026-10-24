# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Acceptance evidence controls; these synthetic events are not actual-host proof."""
# ruff: noqa: PT009, PT027 -- stdlib host controls

from __future__ import annotations

import copy
import hashlib
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts.tests import capability_codex_check as check


def fixture(output: Path) -> tuple[list[dict], dict, str]:
    guide = "# Plan execution\nPrepare the approach; do not execute implementation.\n"
    native = {
        "invocation": "a" * 32,
        "registry": "/registry",
        "run_path": "/registry/invocations/" + "a" * 32 + ".json",
        "run_kind": "rust",
        "phase": "frozen",
        "terminal_status": "available",
        "selected_count": 1,
        "selected_unreported_count": 0,
        "subjects": [{"identity": "pse-ids::lib::" + check.TEST, "selection": {"name": check.TEST, "binary_id": "pse-ids::lib"}, "outcome": "pass"}],
        "terminal_identity": "immutable-native-terminal",
        "workload_exit": 0,
        "assurance": "pass",
        "owned_drain": "complete",
        "report_errors": [],
    }
    description = {"revision": "loaded-revision", "capability": {"id": "rust_test_run"}, "input_schema": {"type": "object"}, "output_schema": {"type": "object"}}
    arguments = {"request": {"package": "pse-ids", "kind": "unit", "expression": check.EXPRESSION, "output": str(output / "rust-test")}}
    action = {"operation": "rust_test_run", "result": {"exit_code": 0, "selected_count": 1, "invocation_ownership": "owned", "invocation": native["run_path"], "catalogue_reader": {"invocation": native["run_path"], "registry": "/registry"}}}

    def rendered(workflow: str) -> dict:
        return {"messages": [{"role": "user", "content": json.dumps({"workflow": {"id": workflow}, "instructions": [{"content": guide}]})}]}

    calls = [
        ("capabilities_domains", {}, {"kind": "domain", "items": [{"id": "source"}]}),
        ("capabilities_workflows", {}, {"kind": "workflow", "items": [{"id": check.WORKFLOW}]}),
        ("capabilities_describe", {"capability": "skill-plan-execution", "include_schema": True}, {"capability": {"id": "skill-plan-execution"}}),
        ("capabilities_describe", {"capability": "rust_test_run", "include_schema": True}, description),
        ("capabilities_select", {"workflows": [check.WORKFLOW], "capabilities": ["skill-plan-execution"]}, {"capabilities": ["skill-plan-execution"]}),
        ("read_resource", {"uri": check.GUIDE_URI}, {"content": guide, "digest": hashlib.sha256(guide.encode()).hexdigest(), "complete": True}),
        ("read_resource", {"uri": check.TEMPLATE_URI}, description),
        ("get_prompt", {"name": "workflow_" + check.WORKFLOW}, rendered(check.WORKFLOW)),
        ("capabilities_reset", {}, {"capabilities": []}),
        ("capabilities_select", {"capabilities": ["rust_test_run", "product_run_read"]}, {"callable_tools": ["rust_test_run", "product_run_read"]}),
        ("capabilities_describe", {"capability": "product_run_read", "include_schema": True}, {"input_schema": {"type": "object"}}),
        ("capabilities_invoke_action", {"capability": "rust_test_run", "arguments": arguments}, action),
        ("capabilities_invoke_read", {"capability": "product_run_read", "arguments": {"invocation": native["run_path"], "registry": "/registry", "limit": 10, "tail": 0}}, {"operation": "product_run_read", "payload": native}),
        ("capabilities_reset", {}, {"capabilities": []}),
        ("capabilities_select", {"workflows": [check.NEXT_WORKFLOW]}, {"capabilities": []}),
        ("get_prompt", {"name": "workflow_" + check.NEXT_WORKFLOW}, rendered(check.NEXT_WORKFLOW)),
        ("capabilities_reset", {}, {"capabilities": []}),
    ]
    events: list[dict] = [{"type": "thread.started", "thread_id": "fresh-thread"}, {"type": "turn.started"}]
    for number, (tool, supplied, result) in enumerate(calls):
        item = {"id": "item_" + str(number), "type": "mcp_tool_call", "server": check.SERVER, "tool": tool, "arguments": supplied, "result": None, "error": None, "status": "in_progress"}
        events.append({"type": "item.started", "item": item})
        completed = {**item, "status": "completed", "result": {"content": [{"type": "text", "text": json.dumps(result)}]}}
        if tool != "read_resource" and tool != "get_prompt":
            completed["result"]["structured_content"] = result
        events.append({"type": "item.completed", "item": completed})
    events.extend([{"type": "item.completed", "item": {"type": "agent_message", "text": "Everything passed"}}, {"type": "turn.completed"}])
    return events, native, guide


class CodexEvidenceTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.output = Path(temporary.name)
        self.events, self.native, self.guide = fixture(self.output)

    def transcript(self, events: list[dict]) -> list[check.Call]:
        path = self.output / "events.jsonl"
        path.write_text("".join(json.dumps(event) + "\n" for event in events))
        return check.transcript(path)

    def test_completed_journey_checks_original_reader_not_final_model_prose(self) -> None:
        calls = self.transcript(self.events)
        with patch.object(check.run_inspection, "product_result", return_value=self.native) as reader:
            outcome, authentic = check.assess(calls, self.output, self.guide)
        self.assertEqual(outcome["selected_count"], 1)
        self.assertEqual(outcome["failures"], 0)
        self.assertEqual(outcome["passed"], 1)
        self.assertEqual(len(outcome["stages"]), 17)
        self.assertEqual(authentic, self.native)
        reader.assert_called_once_with(self.native["run_path"], registry=Path("/registry"), limit=10)

    def test_claimed_pass_without_actual_calls_is_refused(self) -> None:
        events = [self.events[0], self.events[-2], self.events[-1]]
        with self.assertRaisesRegex(ValueError, "missing.*domains"):
            check.assess(self.transcript(events), self.output, self.guide)

    def test_noncanonical_calls_edits_and_incomplete_calls_refuse(self) -> None:
        for item in (
            {"id": "outside", "type": "command_execution", "command": "true"},
            {"id": "outside", "type": "file_change", "changes": []},
            {"id": "outside", "type": "mcp_tool_call", "server": "other", "tool": "read"},
            {"id": "outside", "type": "mcp_tool_call", "server": check.SERVER, "tool": "contracts_generate"},
        ):
            with self.subTest(item=item), self.assertRaises(ValueError):
                self.transcript([*self.events, {"type": "item.started", "item": item}])
        with self.assertRaisesRegex(ValueError, "did not finish"):
            self.transcript(self.events[:-1])
        pending = copy.deepcopy(self.events[2])
        pending["item"]["id"] = "unfinished"
        with self.assertRaisesRegex(ValueError, "did not finish"):
            self.transcript([*self.events, pending])

    def test_authentic_failed_or_broadened_population_overrules_model_claim(self) -> None:
        calls = self.transcript(self.events)
        for changes in (
            {"selected_count": 2},
            {"phase": "open"},
            {"workload_exit": 1},
            {"assurance": "failure"},
            {"owned_drain": "unobserved"},
            {"subjects": [{"selection": {"name": check.TEST, "binary_id": "pse-ids::lib"}, "outcome": "fail"}]},
        ):
            with self.subTest(changes=changes), patch.object(check.run_inspection, "product_result", return_value={**self.native, **changes}), self.assertRaises(ValueError):
                check.assess(calls, self.output, self.guide)

    def test_missing_prompt_and_second_action_cannot_pass(self) -> None:
        calls = self.transcript(self.events)
        with self.assertRaisesRegex(ValueError, "planning_prompt"):
            check.assess([call for call in calls if call.tool != "get_prompt"], self.output, self.guide)
        action = next(call for call in calls if call.operation == "rust_test_run")
        with self.assertRaisesRegex(ValueError, "exactly one"):
            check.assess([*calls, action], self.output, self.guide)
        altered = copy.deepcopy(calls)
        action = next(call for call in altered if call.operation == "rust_test_run")
        check.object_value(action.operation_arguments["request"])["expression"] = "all()"
        with self.assertRaisesRegex(ValueError, "exact authorized"):
            check.assess(altered, self.output, self.guide)

    def test_timeout_drain_reaps_owned_direct_process(self) -> None:
        process = subprocess.Popen([sys.executable, "-c", "import time; time.sleep(60)"], start_new_session=True)
        try:
            check.drain(process, grace=0.2)
            self.assertIsNotNone(process.returncode)
            self.assertFalse(check.live_group(process.pid))
        finally:
            if process.returncode is None:
                process.kill()
                process.wait(timeout=5)


if __name__ == "__main__":
    unittest.main()
