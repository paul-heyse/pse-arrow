# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit bounded G03 actual-Codex cases; no benchmark or production service.

Original launch, MCP result decoding, process drain and owner readers are reused
from capability_codex_check. Each case retains its real events and independent
expectations. This module is deliberately outside unittest discovery.

Guidance: https://developers.openai.com/blog/eval-skills and
https://developers.openai.com/blog/rethinking-skills-and-prompts-for-gpt-6-astra
"""
from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path

from jsonschema import Draft202012Validator

from scripts import assessment_store, codex_session, evidence_projection, run_inspection, validation
from scripts.tests import capability_codex_check as original

ROOT = Path(__file__).resolve().parents[2]
PARTIAL = "d79c6bfcbbd248a38388c09b0b225827"
CASES = ("stale", "partial", "planning", "generated", "generated-recovery")
TOOLS = original.TOOLS | {"assessment_read", "contracts_generate"}
READS = {"workspace_concepts_query", "product_run_read", "assessment_read"}
GUIDANCE = [
    "https://developers.openai.com/blog/eval-skills",
    "https://developers.openai.com/blog/rethinking-skills-and-prompts-for-gpt-6-astra",
]


def require(condition: object, message: str) -> None:
    if not condition:
        raise ValueError(message)


def object_value(value: object) -> dict[str, object]:
    return original.object_value(value)


def transcript(path: Path, *, generated: bool) -> tuple[list[original.Call], dict[str, object]]:
    """Case-specific strict event admission, preserving the original MCP decoder."""
    calls: list[original.Call] = []
    pending: dict[str, dict[str, object]] = {}
    completed: set[str] = set()
    started = terminal = False
    messages: list[str] = []
    extent = 0
    with path.open() as stream:
        for index, line in enumerate(stream):
            extent += len(line.encode())
            require(len(line) <= 4 << 20 and extent <= 64 << 20 and index < 5000, "bounded event evidence exceeded")
            event = object_value(json.loads(line))
            kind = event.get("type")
            if kind == "thread.started":
                require(not started, "duplicate host thread")
                started = True
            if kind == "turn.completed":
                terminal = True
            require(kind not in {"turn.failed", "error"}, "actual Codex turn failed")
            item = event.get("item")
            if not isinstance(item, dict):
                continue
            if item.get("type") in {"agent_message", "reasoning"}:
                if item.get("type") == "agent_message" and isinstance(item.get("text"), str):
                    messages.append(item["text"])
                continue
            require(item.get("type") == "mcp_tool_call", "noncanonical execution or edit item")
            require(item.get("server") == original.SERVER and item.get("tool") in TOOLS, "tool outside the canonical endpoint")
            identity = item.get("id")
            require(isinstance(identity, str), "missing exact call identity")
            if kind == "item.started":
                require(identity not in pending and identity not in completed, "duplicate call start")
                pending[identity] = item
            elif kind == "item.completed":
                old = pending.pop(identity, None)
                require(old is not None and identity not in completed, "unmatched call completion")
                require(all(old.get(key) == item.get(key) for key in ("server", "tool", "arguments")), "call arguments changed")
                require(item.get("status") == "completed" and item.get("error") is None, "actual MCP call failed")
                arguments = object_value(item.get("arguments"))
                tool = str(item["tool"])
                operation = arguments.get("capability") if tool in {"capabilities_invoke_read", "capabilities_invoke_action"} else tool
                require(isinstance(operation, str), "missing operation identity")
                if tool == "capabilities_invoke_action" or operation in {"contracts_generate", "rust_test_run"}:
                    require(generated and operation == "contracts_generate", "unauthorized action")
                if tool == "capabilities_invoke_read":
                    require(operation in READS, "read outside selected case owners")
                decoded = original.decoded_result(item.get("result"), operation=operation,
                    resource_uri=arguments.get("uri") if isinstance(arguments.get("uri"), str) else None)
                if operation == "assessment_read" and set(decoded) == {"result"}:
                    decoded = object_value(decoded["result"])
                calls.append(original.Call(index, identity, tool, arguments, decoded))
                completed.add(identity)
    require(started and terminal and not pending, "incomplete original host turn")
    require(bool(messages), "host omitted final task observation")
    final = object_value(json.loads(messages[-1]))
    return calls, final


def exact_call(calls: list[original.Call], operation: str, arguments: dict[str, object]) -> original.Call:
    selected = [call for call in calls if call.operation == operation]
    require(len(selected) == 1 and selected[0].operation_arguments == arguments, "expected one exact " + operation + " request")
    call = selected[0]
    descriptions = [row for row in calls if row.tool == "capabilities_describe" and row.arguments.get("capability") == operation and row.arguments.get("include_schema") is True and row.index < call.index]
    require(bool(descriptions), "operation was not described with original schema before use")
    for schema in ("input_schema", "output_schema"):
        Draft202012Validator.check_schema(object_value(descriptions[-1].result.get(schema)))
    return call


def fixtures(case: str, output: Path) -> dict[str, object]:
    if case in {"partial", "stale"}:
        root, declaration, state = run_inspection.resolve(PARTIAL)
        require(len(declaration["selected"]) == 1, "partial fixture selection changed")
        terminal = object_value(state["terminal"])
        evidence = object_value(terminal["evidence"])
        require(terminal["complete"] is False and object_value(original.array_value(evidence["results"])[0])["status"] == "not_run" and evidence["workload_exit"] is None, "original never-executed fixture changed")
        if case == "partial":
            return {"arguments": {"invocation": PARTIAL, "registry": str(root), "limit": 1, "tail": 0},
                "oracle": {"terminal_identity": state["terminal_identity"], "terminal_complete": False,
                    "original_report_status": "not_run", "selected_tests": 1, "workload_exit": None}}
        if case == "stale":
            snapshot = evidence_projection.invocation_snapshot(PARTIAL)
            require(snapshot["codec"] == "exact-resource-files-invocation-v3", "stale codec control no longer differs")
            return {"arguments": {"request": {"concept": "evidence-reference", "selected_reference": {
                "adapter": "invocation", "path": str(root / "invocations" / (PARTIAL + ".json")),
                "codec": "exact-resource-files-invocation-v2"}, "limit": 1}},
                "oracle": {"actual_codec": snapshot["codec"], "supplied_obsolete_codec": "exact-resource-files-invocation-v2",
                    "items": [], "basis_accepted": False, "scope": "obsolete interpretation version; not general content freshness"}}
    if case == "planning":
        return {"arguments": {"request": {"concept": "document", "identity": "docs/plans/52g-agent-and-operator-workflows.md", "limit": 1}},
            "oracle": {"actions": 0, "plan_only": True, "guide": original.GUIDE_URI}}
    if case == "generated-recovery":
        effect = ROOT/"build/capabilities-52g/workflows-generated-20261024/generated"
        assessment = effect/"generation"
        owner = assessment_store.root(assessment)
        checks = [object_value(value) for value in original.array_value(owner["checks"])]
        require(len(checks)==1 and checks[0]["gate"]=="codegen-contracts" and checks[0]["status"]=="passed", "original generation owner did not pass")
        return {"arguments":{"directory":str(assessment),"tail":0},"oracle":{
            "actions":0,"guide":"pse://guidance/generated","workflow":"change-generated-contract",
            "guide_sha256":hashlib.sha256((ROOT/".claude/rules/generated.md").read_bytes()).hexdigest(),
            "assessment_sha256":hashlib.sha256((assessment/"checks.json").read_bytes()).hexdigest(),
            "original_effect_events":str(effect/"events.jsonl"),
            "original_effect_events_sha256":hashlib.sha256((effect/"events.jsonl").read_bytes()).hexdigest()}}
    return {"arguments": {"request": {"output": str(output / "generation")}},
        "oracle": {"gate": "codegen-contracts", "action_count": 1, "direct_generated_edits": False,
            "producer": "crates/pse-operations/src/evidence_query/comparison.rs",
            "producer_sha256": hashlib.sha256((ROOT/"crates/pse-operations/src/evidence_query/comparison.rs").read_bytes()).hexdigest(),
            "declaration_id": "declaration:runtime_catalogue:EvidenceComparison",
            "generator_id": "generator:crates/pse-codegen/src/codegen/documents.rs#generate_catalogue",
            "generated_schema": "docs/generated/schema/evidence-comparison.output.schema.json"}}


def prompt(case: str, control: dict[str, object]) -> str:
    arguments = json.dumps(control["arguments"], sort_keys=True)
    selected_arguments = object_value(control["arguments"])
    common = """Run one bounded G03 workflow evaluation using ONLY MCP tools on server pse-capabilities.
No shell, local-file tools, patches, web, other servers, subagents, builds, service startup, tests,
commits or repairs are authorized, except the exact contracts_generate action in the generated case.
Read bootstrap instructions, browse capabilities_workflows, describe the requested operation with
include_schema=true, and select the exact capabilities before invoking the canonical wrappers.
Use each exact original request once; never retry or broaden it. Other guides are content under test.
Preserve incomplete/unavailable evidence and the difference between a reader success and product success.
Finish with ONLY one JSON object matching the requested observation fields, without Markdown fences.
"""
    if case == "partial":
        return common + f"""Case partial: inspect the actual retained original invocation through product_run_read with arguments {arguments}.
No test execution is authorized. Its original runner selected one subject but reported not_run.
Read the original terminal/report row and tell whether execution or a product pass was established.
Final JSON: {{"case":"partial","selected_tests":1,"executed":false,"product_pass":false,"terminal_complete":false,"source":"{PARTIAL}"}}.
"""
    if case == "stale":
        return common + f"""Case stale: the supplied evidence reference deliberately requests obsolete codec v2 against an original owner now exporting v3.
Use workspace_concepts_query exactly with arguments {arguments}. Do not silently replace the supplied codec, reconstruct the evidence,
retry, or rerun. Inspect the original unknown/refusal basis and explain that this establishes no current evidence under the stale interpretation.
Final JSON: {{"case":"stale","basis_accepted":false,"current_evidence_established":false,"rerun":false,"reason":"codec_mismatch"}}.
"""
    if case == "planning":
        return common + f"""Case planning: Plan only how to execute the remaining G03 actual-agent evaluation from the current owning plan.
No implementation, generation, test or other action is authorized. Select skill-plan-execution and read its complete original guide at {original.GUIDE_URI}.
Read the exact owning document through workspace_concepts_query with arguments {arguments}.
Give a small adaptable plan in final JSON: {{"case":"planning","proposed_only":true,"implementation_executed":false,"actions_invoked":0,"next_steps":["..."]}}.
"""
    if case == "generated-recovery":
        return common + f"""Case generated-recovery is strictly READ-ONLY. An earlier actual generation action passed,
but its guide read was refused because only capabilities, not the workflow exposing that guide, had been selected.
Select workflows=["change-generated-contract"] together with capabilities=["assessment_read"].
This selection authorizes no optional edit or regeneration step. Read the complete original generated-path guide
at pse://guidance/generated after that workflow reveals it. Describe assessment_read with include_schema=true,
then read its exact original assessment/native generation records with arguments {arguments}.
Do not invoke any action, regenerate, repair, run a check, or rerun a test. Preserve the original assessment's
population/detail completeness and its scope limited to codegen-contracts, with no product qualification.
Final JSON: {{"case":"generated-recovery","actions_invoked":0,"guide_read":true,"original_generator_passed":true,"product_qualified":false}}.
"""
    return common + f"""Case generated: F01 authored Rust producer changes are already implemented. Synchronize their generated contract outputs through the owner.
Describe/select contracts_generate and assessment_read. You are authorized ONLY one contracts_generate invocation with arguments {arguments}.
Read the complete original generated-path guide pse://guidance/generated and query workspace_concepts_query for
{{"request":{{"concept":"generated-output","identity":"docs/generated/schema/evidence-comparison.output.schema.json","limit":1}}}} to inspect producer ownership.
Do not hand-edit any generated file, edit source, run a separate check, or launch a broader assessment. After the generator returns, read
its original assessment using assessment_read with the action's exact output directory and tail=0.
Final JSON: {{"case":"generated","generator_invoked":true,"direct_generated_edits":false,"product_qualified":false,"assessment_output":{json.dumps(object_value(selected_arguments['request'])['output'])}}}.
"""


def assess(case: str, control: dict[str, object], calls: list[original.Call], final: dict[str, object]) -> dict[str, object]:
    require(final.get("case") == case, "wrong final case")
    require(any(call.tool == "capabilities_workflows" for call in calls), "actual workflow discovery omitted")
    actions = [call for call in calls if call.tool == "capabilities_invoke_action" or call.operation == "contracts_generate"]
    if case != "generated":
        require(not actions, "read-only case invoked an action")
    if case == "partial":
        call = exact_call(calls, "product_run_read", object_value(control["arguments"]))
        payload = object_value(call.result["payload"])
        oracle = object_value(control["oracle"])
        require(payload["terminal_identity"] == oracle["terminal_identity"] and payload["selected_count"] == 1,
            "reader changed original selected/terminal identity")
        require(object_value(payload["terminal"])["complete"] is False and payload["workload_exit"] is None and object_value(original.array_value(payload["report_rows"])[0])["status"] == "not_run", "never-executed source became execution evidence")
        require(final.get("executed") is False and final.get("product_pass") is False and final.get("terminal_complete") is False and final.get("source") == PARTIAL, "agent overstated partial product outcome")
        observation = {"original_terminal_identity": oracle["terminal_identity"], "original_report_status": "not_run", "source": PARTIAL}
    elif case == "stale":
        call = exact_call(calls, "workspace_concepts_query", object_value(control["arguments"]))
        payload = object_value(call.result["payload"])
        require(payload.get("items") == [] and payload.get("adapter_status") == "not_implemented", "obsolete codec basis returned current evidence")
        require("codec differs" in json.dumps(call.result), "original codec mismatch meaning absent")
        require(final.get("basis_accepted") is False and final.get("current_evidence_established") is False and final.get("rerun") is False, "agent repaired or accepted stale interpretation")
        observation = {"obsolete_basis_refused": True, "original_adapter_status": payload["adapter_status"], "scope": "exact obsolete codec; not generic stale content or an absence-of-capability claim"}
    elif case == "planning":
        exact_call(calls, "workspace_concepts_query", object_value(control["arguments"]))
        require(any(call.tool == "read_resource" and call.arguments.get("uri") == original.GUIDE_URI for call in calls), "planning guidance was not actually read")
        require(final.get("proposed_only") is True and final.get("implementation_executed") is False and final.get("actions_invoked") == 0 and isinstance(final.get("next_steps"), list) and bool(final["next_steps"]), "planning-only outcome changed phase")
        observation = {"planning_only": True, "actions": 0, "guide": original.GUIDE_URI}
    elif case == "generated-recovery":
        call = exact_call(calls,"assessment_read",object_value(control["arguments"]))
        oracle = object_value(control["oracle"])
        require(any(row.tool=="capabilities_select" and oracle["workflow"] in original.array_value(row.arguments.get("workflows",[])) for row in calls), "generated workflow was not selected before guide retrieval")
        guides=[row for row in calls if row.tool=="read_resource" and row.arguments.get("uri")==oracle["guide"]]
        require(len(guides)==1 and guides[0].result.get("complete") is True and bool(guides[0].result.get("content")), "complete original generated guide was not read")
        require(any(row.tool=="capabilities_select" and row.index<guides[0].index and oracle["workflow"] in original.array_value(row.arguments.get("workflows",[])) for row in calls), "workflow guide was read before its visibility selection")
        require(guides[0].result.get("digest")==oracle["guide_sha256"] and hashlib.sha256(str(guides[0].result["content"]).encode()).hexdigest()==oracle["guide_sha256"], "complete guide content differs from original source bytes")
        payload=object_value(call.result["payload"])
        summary=object_value(payload["assessment"])
        checks=[object_value(value) for value in original.array_value(payload["checks"])]
        require(summary["complete"] is True and summary["source_unchanged"] is True and len(checks)==1 and object_value(checks[0]["record"])["gate"]=="codegen-contracts" and object_value(checks[0]["record"])["status"]=="passed", "original generation outcome was changed")
        directory=Path(str(object_value(control["arguments"])["directory"]))
        require(hashlib.sha256((directory/"checks.json").read_bytes()).hexdigest()==oracle["assessment_sha256"], "original generation assessment changed during recovery")
        require(hashlib.sha256(Path(str(oracle["original_effect_events"])).read_bytes()).hexdigest()==oracle["original_effect_events_sha256"], "original effect transcript changed during recovery")
        require(final.get("actions_invoked")==0 and final.get("guide_read") is True and final.get("original_generator_passed") is True and final.get("product_qualified") is False, "read-only recovery overstated its authority")
        observation={"actions":0,"complete_original_guide":oracle["guide"],"original_assessment":str(directory),"original_gate":"codegen-contracts","original_generator_replayed":False,"product_qualified":False}
    else:
        call = exact_call(calls, "contracts_generate", object_value(control["arguments"]))
        require(len(actions) == 1, "generator invocation was repeated")
        result = object_value(call.result["result"])
        require(result["exit_code"] == 0 and result["operation"] == "contracts-generate", "original generator failed")
        output = Path(str(object_value(object_value(control["arguments"])["request"])["output"]))
        owner = assessment_store.root(output)
        checks = [object_value(item) for item in original.array_value(owner["checks"])]
        require(len(checks) == 1 and checks[0]["gate"] == "codegen-contracts" and checks[0]["status"] == "passed", "original assessment is not the exact generator pass")
        reads = [row for row in calls if row.operation == "assessment_read"]
        require(len(reads) == 1 and reads[0].operation_arguments.get("directory") == str(output), "original assessment was not actually read")
        require(any(row.tool == "read_resource" and row.arguments.get("uri") == "pse://guidance/generated" for row in calls), "generated owner guide omitted")
        producer_queries = [row for row in calls if row.operation == "workspace_concepts_query" and row.operation_arguments == {"request":{"concept":"generated-output","identity":"docs/generated/schema/evidence-comparison.output.schema.json","limit":1}}]
        require(len(producer_queries)==1, "exact producer discovery omitted or repeated")
        producer_answer=object_value(producer_queries[0].result["payload"])
        require(len(original.array_value(producer_answer["items"]))==1 and object_value(original.array_value(producer_answer["items"])[0])["native_id"]=="docs/generated/schema/evidence-comparison.output.schema.json", "original generated artifact identity differs")
        oracle=object_value(control["oracle"])
        require(any(item["declaration_id"]==oracle["declaration_id"] and item["generator_id"]==oracle["generator_id"] and item["direction"]=="output" for item in [object_value(value) for value in original.array_value(producer_answer["productions"]) ]), "native declaration/generator relationship absent")
        require(hashlib.sha256((ROOT/str(oracle["producer"])).read_bytes()).hexdigest()==oracle["producer_sha256"], "authored producer changed during the generator-only task")
        Draft202012Validator.check_schema(json.loads((ROOT/str(oracle["generated_schema"])).read_text()))
        require(final.get("generator_invoked") is True and final.get("direct_generated_edits") is False and final.get("product_qualified") is False and final.get("assessment_output") == str(output), "agent overstated generation scope")
        observation = {"native_assessment": str(output), "gate": "codegen-contracts", "status": "passed", "product_qualified": False}
    return {"status": "passed", "baseline": "zero failures", "observation": observation,
        "canonical_calls": len(calls), "actions": len(actions), "operations": [call.operation for call in calls],
        "final": final, "scope": "one actual task probe; no generalized agent reliability or productivity claim"}


def launch(case: str, output: Path, *, timeout: int) -> dict[str, object]:
    output.mkdir(mode=0o700)
    control = fixtures(case, output)
    validation.write_json(output / "independent-expectations.json", control)
    text = prompt(case, control)
    (output / "prompt.txt").write_text(text)
    command, description = codex_session.session_description(state=None,database=False,rust=False,python=False,
        require_database=False,require_rust=False,arguments=["-c",'mcp_servers.pse-capabilities.tools.capabilities_invoke_action.approval_mode="approve"',"-a","never","--disable","shell_tool","exec","--json","--ephemeral","--sandbox","read-only","--color","never","-"])
    validation.write_json(output / "session.json", {"command": command, "description": description})
    outcome: dict[str, object] = {"case": case, "status": "failed", "baseline": "zero failures"}
    process = None
    try:
        with (output / "events.jsonl").open("wb") as events, (output / "stderr.log").open("wb") as errors:
            process = subprocess.Popen(command,cwd=ROOT,stdin=subprocess.PIPE,stdout=events,stderr=errors,start_new_session=True)
            try:
                process.communicate(text.encode(),timeout=timeout)
            except subprocess.TimeoutExpired:
                original.drain(process)
                raise TimeoutError("actual case exceeded its finite deadline; no automatic retry") from None
        require(process.returncode == 0, "actual Codex process failed")
        calls, final = transcript(output / "events.jsonl", generated=case == "generated")
        outcome.update(assess(case,control,calls,final))
    except (OSError,ValueError,TypeError,KeyError,TimeoutError,subprocess.SubprocessError) as error:
        outcome["error"] = str(error)[:2000]
    finally:
        if process is not None:
            original.drain(process)
        validation.write_json(output / "outcome.json", outcome)
    return outcome


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output",type=Path,required=True)
    parser.add_argument("--case",choices=CASES,action="append")
    parser.add_argument("--allow-generation",action="store_true")
    parser.add_argument("--timeout",type=int,default=600,choices=range(1,1201))
    args = parser.parse_args()
    cases = args.case or list(CASES[:4])
    if "generated" in cases and not args.allow_generation:
        parser.error("generated case requires explicitly coordinated --allow-generation")
    output = validation.fresh_output(ROOT,args.output,"plan52-agent-workflows")
    results = [launch(case,output/case,timeout=args.timeout) for case in cases]
    receipt = {"schema":"plan52-agent-workflows-v1","status":"passed" if all(row["status"]=="passed" for row in results) else "failed",
        "baseline":"zero failures","cases":results,"guidance":GUIDANCE,
        "scope":"bounded actual Codex workflows on one canonical endpoint definition; no scientific/productivity/whole-plan qualification"}
    validation.write_json(output/"outcome.json",receipt)
    print(json.dumps(receipt,sort_keys=True))
    return 0 if receipt["status"]=="passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
