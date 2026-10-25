# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Synthetic protocol controls only; these do not establish actual agent behavior."""
from __future__ import annotations

import copy
import json
import tempfile
import unittest
from pathlib import Path

from scripts.tests import capability_codex_check as original
from scripts.tests import plan52_agent_workflows_acceptance as check


class AgentWorkflowAcceptanceTests(unittest.TestCase):
    def events(self, *, action: bool = False) -> list[dict[str, object]]:
        item = {"id":"one","type":"mcp_tool_call","server":original.SERVER,
            "tool":"capabilities_invoke_action" if action else "capabilities_invoke_read",
            "arguments":{"capability":"contracts_generate" if action else "product_run_read","arguments":{}}}
        result = {"structured_content":{"operation":"contracts_generate","result":{}}} if action else {"structured_content":{"result":{"operation":"product_run_read","payload":{}}}}
        return [{"type":"thread.started"},{"type":"item.started","item":item},
            {"type":"item.completed","item":{**item,"status":"completed","error":None,"result":result}},
            {"type":"item.completed","item":{"id":"final","type":"agent_message","text":'{"case":"partial"}'}},
            {"type":"turn.completed"}]

    def parse(self, events: list[dict[str, object]], *, generated: bool = False) -> tuple[list[original.Call], dict[str, object]]:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary)/"events.jsonl"
            path.write_text("\n".join(json.dumps(event) for event in events)+"\n")
            return check.transcript(path,generated=generated)

    def test_original_read_wrapper_and_final_observation_are_preserved(self) -> None:
        calls, final = self.parse(self.events())
        self.assertEqual(calls[0].operation,"product_run_read")
        self.assertEqual(calls[0].result,{"operation":"product_run_read","payload":{}})
        self.assertEqual(final,{"case":"partial"})

    def test_planning_admission_refuses_actions_and_local_execution(self) -> None:
        with self.assertRaisesRegex(ValueError,"unauthorized action"):
            self.parse(self.events(action=True))
        events=self.events()
        events.insert(1,{"type":"item.completed","item":{"type":"command_execution","command":"echo not-authorized"}})
        with self.assertRaisesRegex(ValueError,"noncanonical"):
            self.parse(events)

    def test_readonly_rejects_a_direct_rust_action_too(self) -> None:
        events=self.events()
        for event in events:
            item=event.get("item")
            if isinstance(item,dict) and item.get("type")=="mcp_tool_call":
                item["tool"]="rust_test_run"
                item["arguments"]={"request":{}}
        with self.assertRaisesRegex(ValueError,"unauthorized action"):
            self.parse(events)

    def test_generated_case_admits_only_original_generator(self) -> None:
        calls,_=self.parse(self.events(action=True),generated=True)
        self.assertEqual(calls[0].operation,"contracts_generate")
        events=self.events(action=True)
        for event in events:
            item=event.get("item")
            if isinstance(item,dict) and item.get("type")=="mcp_tool_call":
                item["arguments"]["capability"]="rust_test_run"
        with self.assertRaisesRegex(ValueError,"unauthorized action"):
            self.parse(events,generated=True)

    def test_changed_arguments_and_incomplete_calls_are_rejected(self) -> None:
        events=self.events()
        events[2]=copy.deepcopy(events[2])
        events[2]["item"]["arguments"]["arguments"]={"changed":True}
        with self.assertRaisesRegex(ValueError,"arguments changed"):
            self.parse(events)
        events=self.events()
        del events[2]
        with self.assertRaisesRegex(ValueError,"incomplete"):
            self.parse(events)

    def test_partial_reader_success_does_not_admit_product_pass(self) -> None:
        arguments={"invocation":"original"}
        control={"arguments":arguments,"oracle":{"terminal_identity":"terminal"}}
        calls=[original.Call(0,"workflows","capabilities_workflows",{},{}),
            original.Call(1,"describe","capabilities_describe",{"capability":"product_run_read","include_schema":True},{"input_schema":{"type":"object"},"output_schema":{"type":"object"}}),
            original.Call(2,"read","capabilities_invoke_read",{"capability":"product_run_read","arguments":arguments},{"payload":{"terminal_identity":"terminal","selected_count":1,"terminal":{"complete":False},"workload_exit":None,"report_rows":[{"status":"not_run"}]}})]
        final={"case":"partial","executed":False,"product_pass":False,"terminal_complete":False,"source":check.PARTIAL}
        outcome=check.assess("partial",control,calls,final)
        self.assertEqual(outcome["status"],"passed")
        with self.assertRaisesRegex(ValueError,"overstated"):
            check.assess("partial",control,calls,{**final,"product_pass":True})

    def test_stale_codec_rejection_cannot_contain_current_items(self) -> None:
        arguments={"request":{"concept":"evidence-reference"}}
        calls=[original.Call(0,"workflows","capabilities_workflows",{},{}),
            original.Call(1,"describe","capabilities_describe",{"capability":"workspace_concepts_query","include_schema":True},{"input_schema":{"type":"object"},"output_schema":{"type":"object"}}),
            original.Call(2,"query","capabilities_invoke_read",{"capability":"workspace_concepts_query","arguments":arguments},{"payload":{"items":[],"adapter_status":"not_implemented"},"unknowns":["selected codec differs from original owner"]})]
        final={"case":"stale","basis_accepted":False,"current_evidence_established":False,"rerun":False}
        self.assertEqual(check.assess("stale",{"arguments":arguments},calls,final)["status"],"passed")
        calls[-1].result["payload"]["items"].append({"invented":"current"})
        with self.assertRaisesRegex(ValueError,"returned current evidence"):
            check.assess("stale",{"arguments":arguments},calls,final)


if __name__ == "__main__":
    unittest.main()
