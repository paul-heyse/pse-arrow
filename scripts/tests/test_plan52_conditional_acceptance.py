# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Tamper controls for inquiry evidence; synthetic events are not host evidence."""

from __future__ import annotations

import copy
import hashlib
import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from scripts.tests import plan52_conditional_acceptance as inquiry


class ConditionalSetupPolicyTests(unittest.IsolatedAsyncioTestCase):
    async def test_exact_setup_is_once_only_and_forbidden_inside_composition(self) -> None:
        guard = inquiry.InquiryHostGuard(frozenset(), prepare_native=True)
        context = SimpleNamespace(message=SimpleNamespace(name=inquiry.PREPARE, arguments=inquiry.PREPARATION))

        async def next_call(_context: object) -> str:
            return "original-owner"

        self.assertEqual(await guard.on_call_tool(context, next_call), "original-owner")
        with self.assertRaisesRegex(inquiry.ToolError, "must_not_be_replayed"):
            await guard.on_call_tool(context, next_call)
        guard = inquiry.InquiryHostGuard(frozenset(), prepare_native=True)
        token = inquiry.composition.ACTIVE.set(inquiry.composition.Policy())
        try:
            with self.assertRaisesRegex(inquiry.ToolError, "only_explicit"):
                await guard.on_call_tool(context, next_call)
        finally:
            inquiry.composition.ACTIVE.reset(token)


class ConditionalEvidenceTests(unittest.TestCase):
    def test_missing_initializer_capability_refuses_before_fixture_provisioning(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "not-provisioned"
            with patch.object(inquiry.subprocess, "run", return_value=subprocess.CompletedProcess([], 2)) as run:
                with self.assertRaisesRegex(RuntimeError, "no fixture provisioned"):
                    inquiry.native_fixture(output)
            self.assertFalse(output.exists())
            self.assertEqual(run.call_args.args[0][-2:], ["canonical-init", "--help"])

    def test_independent_expected_answer_cannot_share_exposed_input_custody(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            source = directory / "comparison-inputs.json"
            source.write_text(json.dumps({"comparison": {}, "expected": {"invented": "answer"}}))
            with self.assertRaisesRegex(AssertionError, "separate custody"):
                inquiry.load_control(directory)
            source.write_text(json.dumps({"comparison": {}}))
            (directory / "comparison-expected.json").write_text(json.dumps({"cut": "original-cut"}))
            self.assertEqual(inquiry.load_control(directory)["expected"], {"cut": "original-cut"})
            self.assertNotIn("expected", json.loads(source.read_text()))

    def provenance(self) -> tuple[dict[str, object], list[object]]:
        expected = [{"identity": "native::subject", "outcome": "unreported", "assurance": "incomplete"}]
        structured = {"result": {"payload": {
            "terminal_identity": "native-terminal", "registry": "/original/registry",
            "subjects": [{"identity": "native::subject", "outcome": "unreported"}],
            "assurance": "incomplete",
        }}}
        answer = {"complete": False, "derived": copy.deepcopy(expected), "observations": [{
            "capability": "product_run_read", "outcome": "observed", "structured": structured,
            "sha256": hashlib.sha256(inquiry.composition.encoded(structured)).hexdigest(),
        }]}
        return answer, expected

    def test_rejects_forged_derived_summary_despite_valid_original_provenance(self) -> None:
        answer, expected = self.provenance()
        inquiry.validate_provenance(answer, expected)
        answer["derived"] = [{"identity": "native::subject", "outcome": "pass", "assurance": "pass"}]
        with self.assertRaisesRegex(AssertionError, "derived answer"):
            inquiry.validate_provenance(answer, expected)

    def test_rejects_missing_provenance_and_relabelled_completeness(self) -> None:
        answer, expected = self.provenance()
        answer["observations"] = []
        with self.assertRaisesRegex(AssertionError, "retained native facts"):
            inquiry.validate_provenance(answer, expected)
        answer, expected = self.provenance()
        answer["complete"] = True
        with self.assertRaisesRegex(AssertionError, "promoted to complete"):
            inquiry.validate_provenance(answer, expected)

    def test_rejects_tampered_callback_bytes(self) -> None:
        answer, expected = self.provenance()
        answer["observations"][0]["structured"]["result"]["payload"]["terminal_identity"] = "other-terminal"
        with self.assertRaisesRegex(AssertionError, "digest"):
            inquiry.validate_provenance(answer, expected)

    def test_repeated_focused_read_retains_provenance_without_inventing_another_subject(self) -> None:
        answer, expected = self.provenance()
        answer["observations"].append(copy.deepcopy(answer["observations"][0]))
        inquiry.validate_provenance(answer, expected)
        repeated = answer["observations"][1]
        repeated["structured"]["result"]["payload"]["assurance"] = "pass"
        repeated["sha256"] = hashlib.sha256(inquiry.composition.encoded(repeated["structured"])).hexdigest()
        with self.assertRaisesRegex(AssertionError, "repeated native observations disagree"):
            inquiry.validate_provenance(answer, expected)

    def test_rejects_unfinished_calls_and_noncanonical_execution(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "events.jsonl"
            events = [
                {"type": "thread.started"},
                {"type": "item.started", "item": {"type": "mcp_tool_call", "id": "one",
                    "server": inquiry.SERVER, "tool": "product_run_read", "arguments": {}}},
                {"type": "turn.completed"},
            ]
            path.write_text("\n".join(json.dumps(row) for row in events))
            with self.assertRaisesRegex(AssertionError, "complete fresh turn"):
                inquiry.host_calls(path)
            events[1] = {"type": "item.completed", "item": {"type": "command_execution", "command": "echo fabricated"}}
            path.write_text("\n".join(json.dumps(row) for row in events))
            with self.assertRaisesRegex(AssertionError, "outside"):
                inquiry.host_calls(path)


class CurrentNativeSetupObservationTests(unittest.TestCase):
    def fixture(self, *, before_current: bool) -> tuple[list[dict[str, object]], list[dict[str, object]], list[dict[str, object]], dict[str, object]]:
        def comparison(identity: str, payload: dict[str, object]) -> dict[str, object]:
            return {"id":identity,"tool":inquiry.F01_COMPARE,"arguments":{},"result":{"structured_content":{"payload":payload}}}
        before=comparison("before",{"outcome":"observed","document":{"cut":"original"}} if before_current else {"outcome":"prerequisite_unavailable","code":"pse.native.artifact_not_current"})
        preparation={"id":"setup","tool":"capabilities_invoke_action","arguments":{"capability":inquiry.PREPARE,"arguments":inquiry.PREPARATION},"result":{"structured_content":{"exit_code":0,"operation":"prepare","mode":"native","profile":"dev","validated":True}}}
        after=comparison("after",{"outcome":"observed","document":{"cut":"original"}})
        return [before,preparation,after],[before,after],([before,after] if before_current else [after]),preparation

    def test_actual_current_basis_does_not_require_a_fabricated_stale_refusal(self) -> None:
        self.assertEqual(inquiry.native_setup_observation(*self.fixture(before_current=True))["before_state"],"observed_current")

    def test_exact_artifact_stale_basis_is_retained(self) -> None:
        self.assertEqual(inquiry.native_setup_observation(*self.fixture(before_current=False))["before_state"],"artifact_not_current")

    def test_an_unrelated_native_refusal_cannot_be_repaired_as_currency(self) -> None:
        calls,comparisons,observed,preparation=self.fixture(before_current=False)
        comparisons[0]["result"]["structured_content"]["payload"]["code"]="pse.native.unbound_loaded_artifact"
        with self.assertRaisesRegex(AssertionError,"unrelated native refusal"):
            inquiry.native_setup_observation(calls,comparisons,observed,preparation)


if __name__ == "__main__":
    unittest.main()
