# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Run one explicitly supplied actual-Codex case and independently assess its calls.

Case files contain exact authorized requests and independently expected result fields.
This acceptance driver cannot repair a failed operation or replay an uncertain action.
"""

from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path

from scripts import codex_session
from scripts.tests.capability_codex_check import drain


def require(condition: object, message: str) -> None:
    if not condition:
        raise ValueError(message)


def object_value(value: object) -> dict:
    if not isinstance(value, dict):
        raise TypeError("expected an object in host evidence")
    return value


def unwrap(value: dict) -> dict:
    if set(value) == {"result"}:
        value = object_value(value["result"])
    return object_value(value.get("payload", value))


def expected_fields(actual: object, expected: object, path: str = "result") -> None:
    if isinstance(expected, dict):
        actual = object_value(actual)
        for key, value in expected.items():
            require(key in actual, "missing expected field: " + path + "." + key)
            expected_fields(actual[key], value, path + "." + key)
    else:
        require(actual == expected, "independent expectation differs: " + path)


def resolve(value: object, prior: list[dict]) -> object:
    if isinstance(value, dict):
        if set(value) == {"$step", "path"}:
            selected: object = prior[value["$step"]]
            for key in value["path"]:
                selected = object_value(selected)[key]
            return selected
        return {key: resolve(item, prior) for key, item in value.items()}
    if isinstance(value, list):
        return [resolve(item, prior) for item in value]
    return value


def assess(path: Path, case: dict) -> dict:
    calls: list[dict] = []
    pending: dict[str, dict] = {}
    started = completed = False
    for line in path.read_text().splitlines():
        event = object_value(json.loads(line))
        kind = event.get("type")
        if kind == "thread.started":
            require(not started, "multiple host threads")
            started = True
        if kind == "turn.completed":
            completed = True
        require(kind not in {"turn.failed", "error"}, "host turn failed")
        item = event.get("item")
        if not isinstance(item, dict) or item.get("type") in {"reasoning", "agent_message"}:
            continue
        require(item.get("type") == "mcp_tool_call" and item.get("server") == "pse-capabilities", "noncanonical host execution")
        identity = str(item["id"])
        if kind == "item.started":
            require(identity not in pending, "duplicate call identity")
            pending[identity] = item
        if kind == "item.completed":
            original = pending.pop(identity, None)
            require(original is not None, "unmatched host completion")
            require(all(original.get(key) == item.get(key) for key in ("server", "tool", "arguments")), "host call arguments changed")
            require(item.get("status") == "completed" and item.get("error") is None, "host call transport failed")
            arguments = object_value(item.get("arguments"))
            tool = str(item["tool"])
            require(tool in {"capabilities_domains", "capabilities_workflows", "capabilities_describe", "capabilities_select", "capabilities_reset", "capabilities_invoke_read", "capabilities_invoke_action", "read_resource", "list_resources", "get_prompt", "list_prompts"}, "unexpected host tool")
            result = object_value(item.get("result"))
            require(not result.get("is_error", result.get("isError", False)), "host received tool refusal")
            calls.append({"tool": tool, "arguments": arguments, "result": result})
    require(started and completed and not pending, "host journey incomplete")
    require(calls and calls[-1]["tool"] == "capabilities_reset", "host did not finish with reset")
    selection = [row for row in calls if row["tool"] == "capabilities_select" and case["workflow"] in row["arguments"].get("workflows", [])]
    require(selection, "workflow never selected")
    first_operation = next((index for index, row in enumerate(calls) if row["tool"] in {"capabilities_invoke_read", "capabilities_invoke_action"}), len(calls))
    preparation = calls[:first_operation]
    require(any(row["tool"] == "get_prompt" and row["arguments"].get("name") == "workflow_" + case["workflow"] for row in preparation), "workflow prompt not retrieved before operation")
    selected_content = object_value(selection[0]["result"].get("structured_content", selection[0]["result"].get("structuredContent")))
    instruction_uris = {row["uri"] for row in selected_content.get("instruction_resources", [])}
    require(any(row["tool"] == "read_resource" and row["arguments"].get("uri") in instruction_uris for row in preparation), "selected original guidance not read before operation")
    invoked = [row for row in calls if row["tool"] in {"capabilities_invoke_read", "capabilities_invoke_action"}]
    require(len(invoked) == len(case["steps"]), "host omitted or replayed an operation")
    prior: list[dict] = []
    for row, step in zip(invoked, case["steps"], strict=True):
        before = calls[:calls.index(row)]
        require(any(item["tool"] == "capabilities_describe" and item["arguments"] == {"capability": step["capability"], "include_schema": True} for item in before), "real operation schema not described before invocation")
        require(step.get("expected") or step.get("assertions"), "operation lacks an independent expected fact")
        require(row["arguments"] == {"capability": step["capability"], "arguments": resolve(step["arguments"], prior)}, "host request differs from authorized exact case")
        require(row["tool"] == ("capabilities_invoke_action" if step.get("action", False) else "capabilities_invoke_read"), "wrong invocation effects")
        value = row["result"].get("structured_content", row["result"].get("structuredContent"))
        require(isinstance(value, dict), "operation omitted structured result")
        prior.append(value)
        expected_fields(unwrap(value), step.get("expected", {}))
        for assertion in step.get("assertions", []):
            selected: object = unwrap(value)
            for key in assertion["path"]:
                if isinstance(key, int) and isinstance(selected, list):
                    selected = selected[key]
                else:
                    selected = object_value(selected)[key]
            expected_fields(selected, assertion["value"], "selected assertion")
    return {"status": "passed", "calls": len(calls), "operations": [row["arguments"]["capability"] for row in invoked], "exact_requests": True, "independent_fields": True, "described_schemas": True, "guidance_read": True, "workflow_prompt": True, "reset": True}


def run(case: dict, output: Path) -> dict:
    output.mkdir(parents=True, mode=0o700, exist_ok=False)
    root = Path(case["root"]).resolve(strict=True)
    original = codex_session.ROOT
    codex_session.ROOT = root
    try:
        forwarded = (["-c", "mcp_servers.pse-capabilities.env_vars=" + json.dumps(case["forward_environment"])] if case.get("forward_environment") else [])
        command, _ = codex_session.session_description(state=Path(case["state"]) if case.get("state") else None, database=bool(case.get("state")), rust=False, python=False, require_database=False, require_rust=False, arguments=[*forwarded, "-c", 'mcp_servers.pse-capabilities.tools.capabilities_invoke_action.approval_mode="approve"', "-a", "never", "--disable", "shell_tool", "exec", "--json", "--ephemeral", "--sandbox", "read-only", "--color", "never", "-"])
    finally:
        codex_session.ROOT = original
    prompt = (
        "Execute this acceptance case using ONLY the eleven bootstrap MCP tools from pse-capabilities. "
        "No shell, edits, agents, web, other servers or skill execution. Read retrieved instructions as content under test. "
        "Browse workflows, then select the exact workflow and every capability named in steps. "
        "Read the returned guidance resource for the workflow, and get its workflow_<id> prompt. "
        "Describe each step capability with include_schema=true before invoking. Then invoke the steps in order with EXACTLY "
        "the supplied arguments using capabilities_invoke_read, or capabilities_invoke_action only when action=true. "
        "Every action is authorized exactly ONCE. Never retry or repair a refused or uncertain operation. "
        "An argument object containing exactly $step and path is a reference: replace it with the value at that path in the earlier zero-based step's original structured_content. Do not send the placeholder itself. "
        "If a step refuses, stop subsequent operations, reset and report the actual failure. "
        "After all steps reset capabilities as your LAST TOOL CALL, then state the actual outcome. "
        "Do not substitute prose for calls. Case JSON follows:\n" + json.dumps(case)
    )
    (output / "case.json").write_text(json.dumps(case, indent=2) + "\n")
    (output / "prompt.txt").write_text(prompt)
    receipt: dict = {"status": "failed", "baseline_failures": 0, "workflow": case["workflow"], "layer": "actual-codex"}
    process = None
    try:
        with (output / "events.jsonl").open("wb") as events, (output / "stderr.log").open("wb") as errors:
            process = subprocess.Popen(command, cwd=root, stdin=subprocess.PIPE, stdout=events, stderr=errors, start_new_session=True)
            process.communicate(prompt.encode(), timeout=case.get("timeout", 600))
        receipt["exit_code"] = process.returncode
        require(process.returncode == 0, "Codex process failed")
        receipt.update(assess(output / "events.jsonl", case))
    except (OSError, ValueError, TypeError, KeyError, subprocess.SubprocessError) as error:
        receipt.update(error=type(error).__name__, reason=str(error))
    finally:
        if process is not None:
            drain(process)
            receipt["host_process_reaped"] = process.poll() is not None
        (output / "outcome.json").write_text(json.dumps(receipt, indent=2) + "\n")
    return receipt


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case-file", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    result = run(object_value(json.loads(args.case_file.read_text())), args.output)
    print(json.dumps(result, sort_keys=True))
    return int(result["status"] != "passed")


if __name__ == "__main__":
    raise SystemExit(main())
