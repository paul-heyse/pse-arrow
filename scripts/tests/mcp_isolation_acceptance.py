# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit disposable-checkout MCP/CLI edit and language acceptance.

No production source is edited. Every fixture is removed after the endpoint closes;
original structured replies and independent byte assertions remain in fresh output.
"""

from __future__ import annotations

import argparse
import asyncio
import hashlib
import json
import subprocess
import time
from pathlib import Path

from fastmcp import Client
from fastmcp.client.transports import StdioTransport
from pydantic import JsonValue, TypeAdapter

from scripts.worktree import command_environment

ROOT = Path(__file__).resolve().parents[2]
JSON_OBJECT = TypeAdapter(dict[str, JsonValue])
SAMPLE = "import json\nfrom pathlib import Path\nvalue=json.loads(Path('fixture.json').read_text())\n"
AFTER = "import json\nfrom pathlib import Path\nvalue=json.loads(Path('fixture.json').read_bytes())\n"


def write(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2, allow_nan=False) + "\n")


def require(condition: object, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def unwrapped(value: object) -> dict:
    if not isinstance(value, dict):
        raise TypeError("Expected an original structured object")
    if set(value) == {"result"}:
        value = value["result"]
    if not isinstance(value, dict):
        raise TypeError("Expected unwrapped structured object")
    return value


async def selected(client: Client, capability: str, arguments: dict) -> dict:
    description = await client.call_tool(
        "capabilities_describe", {"capability": capability, "include_schema": True}
    )
    described = unwrapped(description.structured_content)
    await client.call_tool("capabilities_select", {"capabilities": [capability]})
    # The canonical tool annotations are the owner of effect classification.
    actual = next(
        tool
        for tool in await client.list_tools(cache_mode="refresh")
        if tool.name == capability
    )
    if actual.annotations is None:
        raise AssertionError("Canonical tool has no effect annotations")
    annotations = JSON_OBJECT.validate_python(
        actual.annotations.model_dump(mode="json", by_alias=True), strict=True
    )
    invoker = (
        "capabilities_invoke_read"
        if annotations.get("readOnlyHint") is True
        else "capabilities_invoke_action"
    )
    result = await client.call_tool(
        invoker,
        {"capability": capability, "arguments": arguments},
        raise_on_error=False,
    )
    return {
        "capability": capability,
        "arguments": arguments,
        "is_error": result.is_error,
        "structured_content": result.structured_content,
        "content": [block.model_dump(mode="json") for block in result.content],
        "interface": described["interface"],
    }


def client(root: Path) -> Client:
    return Client(
        StdioTransport(
            command=str(root / ".venv/bin/python"),
            args=["-m", "scripts.capabilities"],
            cwd=str(root),
            env=command_environment(root),
            keep_alive=False,
        ),
        timeout=120,
    )


async def edit_stdio(root: Path, output: Path) -> dict:
    path = root / "mcp-assessment-structural.py"
    path.write_text(SAMPLE)
    calls = []
    try:
        async with client(root) as endpoint:
            first = await selected(
                endpoint, "structural_json_source_bytes_preview", {"paths": [path.name]}
            )
            calls.append(first)
            require(not first["is_error"], "Structural preview refused")
            handle = unwrapped(first["structured_content"])["payload"]["preview_sha256"]
            for capability, arguments in (
                ("edit_preview_status", {"preview_sha256": handle}),
                (
                    "edit_preview_read",
                    {"preview_sha256": handle, "file_index": 0, "side": "after"},
                ),
                (
                    "edit_preview_apply",
                    {
                        "preview_sha256": handle,
                        "receipt": "mcp-assessment-structural-receipt.json",
                    },
                ),
                ("edit_preview_reconcile", {"preview_sha256": handle}),
                ("edit_preview_release", {"preview_sha256": handle}),
            ):
                result = await selected(endpoint, capability, arguments)
                calls.append(result)
                require(not result["is_error"], capability + " refused")
            require(
                path.read_text() == AFTER,
                "Applied bytes differ from independently expected migration",
            )
            retained = await selected(
                endpoint, "edit_preview_read", {"preview_sha256": handle}
            )
            calls.append(retained)
            require(
                retained["is_error"], "Released preview unexpectedly remained readable"
            )
            path.write_text(SAMPLE)
            first = await selected(
                endpoint, "structural_json_source_bytes_preview", {"paths": [path.name]}
            )
            calls.append(first)
            handle = unwrapped(first["structured_content"])["payload"]["preview_sha256"]
            path.write_text("changed_by_owner=1\n")
            stale = await selected(
                endpoint,
                "edit_preview_apply",
                {
                    "preview_sha256": handle,
                    "receipt": "mcp-assessment-stale-receipt.json",
                },
            )
            calls.append(stale)
            require(stale["is_error"], "Stale preview was applied")
            require(
                path.read_text() == "changed_by_owner=1\n",
                "Stale preview changed source bytes",
            )
            reconciled = await selected(
                endpoint, "edit_preview_reconcile", {"preview_sha256": handle}
            )
            calls.append(reconciled)
            require(
                not reconciled["is_error"], "Diverged preview could not be reconciled"
            )
            released = await selected(
                endpoint, "edit_preview_release", {"preview_sha256": handle}
            )
            calls.append(released)
            protected = await selected(
                endpoint,
                "structural_json_source_bytes_preview",
                {"paths": ["scripts/generated/capabilities/cli.py"]},
            )
            calls.append(protected)
            require(protected["is_error"], "Generated-path protection did not refuse")
        async with client(root) as fresh:
            lost = await selected(
                fresh, "edit_preview_read", {"preview_sha256": handle}
            )
            calls.append(lost)
            require(lost["is_error"], "Endpoint-local handle crossed endpoint lifetime")
        return {
            "status": "passed",
            "scope": "real STDIO protected structural edit and endpoint custody",
            "independent_expected_after_sha256": hashlib.sha256(
                AFTER.encode()
            ).hexdigest(),
        }
    finally:
        write(output / "edit-stdio-calls.json", calls)
        path.unlink(missing_ok=True)
        (root / "mcp-assessment-structural-receipt.json").unlink(missing_ok=True)
        (root / "mcp-assessment-stale-receipt.json").unlink(missing_ok=True)


def edit_cli(root: Path, output: Path) -> dict:
    path = root / "mcp-assessment-cli.py"
    path.write_text(SAMPLE)
    calls = []
    environment = command_environment(root)
    with (output / "cli-session.stderr").open("w") as error:
        process = subprocess.Popen(
            [str(root / ".venv/bin/python"), "-m", "scripts.capability_cli", "session"],
            cwd=root,
            env=environment,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=error,
            text=True,
        )
        try:
            input_pipe, output_pipe = process.stdin, process.stdout
            if input_pipe is None or output_pipe is None:
                raise AssertionError("CLI pipe missing")
            welcome = json.loads(output_pipe.readline())
            require(
                welcome.get("session") == "connected", "CLI session did not connect"
            )
            calls.append(welcome)

            def request(capability: str, arguments: dict) -> dict:
                identity = str(len(calls))
                input_pipe.write(
                    json.dumps(
                        {
                            "id": identity,
                            "capability": capability,
                            "arguments": arguments,
                        }
                    )
                    + "\n"
                )
                input_pipe.flush()
                reply = json.loads(output_pipe.readline())
                calls.append(reply)
                require(reply["id"] == identity, "CLI reply identity changed")
                require(
                    not reply.get("is_error")
                    and not reply.get("result", {}).get("is_error"),
                    "CLI request refused: " + capability,
                )
                return unwrapped(reply["result"]["structured_content"])

            preview = request(
                "structural_json_source_bytes_preview", {"paths": [path.name]}
            )
            handle = preview["payload"]["preview_sha256"]
            detail = request(
                "edit_preview_read",
                {"preview_sha256": handle, "file_index": 0, "side": "after"},
            )
            require(
                detail["payload"]["text"] == AFTER,
                "CLI preview differs from independently expected text",
            )
            request(
                "edit_preview_apply",
                {
                    "preview_sha256": handle,
                    "receipt": "mcp-assessment-cli-receipt.json",
                },
            )
            require(path.read_text() == AFTER, "CLI apply changed wrong bytes")
            request("edit_preview_reconcile", {"preview_sha256": handle})
            request("edit_preview_release", {"preview_sha256": handle})
            input_pipe.close()
            code = process.wait(timeout=30)
            require(code == 0, "CLI did not close successfully on EOF")
            return {
                "status": "passed",
                "pid": process.pid,
                "exit_code": code,
                "eof_settled": True,
                "scope": "one real foreground CLI endpoint; original edit custody",
            }
        finally:
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=20)
            write(output / "edit-cli-calls.json", calls)
            path.unlink(missing_ok=True)
            (root / "mcp-assessment-cli-receipt.json").unlink(missing_ok=True)


async def navigation(root: Path, output: Path) -> dict:
    path = root / "mcp_assessment_navigation.py"
    path.write_text(
        "def add_one(value: int) -> int:\n    return value + 1\n\nresult = add_one(2)\n"
    )
    calls = []
    statuses = {}
    try:
        async with client(root) as endpoint:
            coordinates = {
                "language": "python",
                "file_path": path.name,
                "line": 4,
                "character": 12,
            }
            requests = [
                (
                    "navigation_tool_support",
                    {"language": "python", "file_path": path.name},
                ),
                ("navigation_definition", coordinates),
                (
                    "navigation_references",
                    {
                        **coordinates,
                        "line": 1,
                        "character": 6,
                        "include_declaration": True,
                    },
                ),
                ("navigation_hover", coordinates),
                ("navigation_type_definition", coordinates),
                ("navigation_declaration", coordinates),
                ("navigation_implementations", coordinates),
                ("navigation_completions", coordinates),
                ("navigation_signature", coordinates),
                ("navigation_symbols", {"language": "python", "file_path": path.name}),
                (
                    "navigation_diagnostics",
                    {"language": "python", "file_path": path.name},
                ),
                (
                    "navigation_workspace_symbols",
                    {"language": "python", "query": "add_one", "limit": 10},
                ),
                ("navigation_call_hierarchy", coordinates),
                (
                    "navigation_code_actions",
                    {
                        "language": "python",
                        "file_path": path.name,
                        "start_line": 1,
                        "start_character": 1,
                        "end_line": 1,
                        "end_character": 8,
                    },
                ),
            ]
            for capability, arguments in requests:
                reply = await selected(endpoint, capability, arguments)
                calls.append(reply)
                statuses[capability] = "refused" if reply["is_error"] else "delivered"
                write(output / "navigation-calls.json", calls)
            # A successful transport wrapper does not establish a successful LSP answer.
            definition = next(
                row for row in calls if row["capability"] == "navigation_definition"
            )
            native = (
                unwrapped(definition["structured_content"])["payload"]["native"]
                if not definition["is_error"]
                else {}
            )
            require(
                not definition["is_error"] and not native.get("isError"),
                "Python definition lookup failed",
            )
            require(
                path.name in json.dumps(native),
                "Definition does not name independent expected source",
            )
            require(
                '"line": 1' in json.dumps(native),
                "Definition does not name independently expected declaration line",
            )
            stale = await selected(
                endpoint,
                "navigation_definition",
                {**coordinates, "source_sha256": "0" * 64},
            )
            calls.append(stale)
            require(stale["is_error"], "Stale source hash was accepted")
        return {
            "status": "passed",
            "scope": "real Python language provider over fresh STDIO; per-native replies retain support/refusal",
            "operations": statuses,
        }
    finally:
        write(output / "navigation-calls.json", calls)
        path.unlink(missing_ok=True)


async def navigation_extended(root: Path, output: Path) -> dict:
    path = root / "mcp_assessment_navigation_extended.py"
    original = (
        "def add_one(value: int) -> int:\n    return value + 1\n\nresult = add_one(2)\n"
    )
    path.write_text(original)
    calls = []
    try:
        async with client(root) as endpoint:
            hierarchy = await selected(
                endpoint,
                "navigation_call_hierarchy",
                {
                    "language": "python",
                    "file_path": path.name,
                    "line": 1,
                    "character": 6,
                },
            )
            calls.append(hierarchy)
            item = unwrapped(hierarchy["structured_content"])["payload"]["native"][
                "result"
            ]["items"][0]
            for capability in (
                "navigation_incoming_calls",
                "navigation_outgoing_calls",
            ):
                reply = await selected(
                    endpoint, capability, {"language": "python", "item": item}
                )
                calls.append(reply)
                require(
                    not reply["is_error"]
                    and not unwrapped(reply["structured_content"])["payload"]["native"][
                        "isError"
                    ],
                    capability + " failed",
                )
            renamed = await selected(
                endpoint,
                "navigation_rename_preview",
                {
                    "language": "python",
                    "file_path": path.name,
                    "line": 1,
                    "character": 6,
                    "new_name": "add_two",
                    "owned": [path.name],
                },
            )
            calls.append(renamed)
            require(not renamed["is_error"], "Rename preview failed")
            handle = unwrapped(renamed["structured_content"])["payload"]["preview"][
                "preview_sha256"
            ]
            for capability, arguments in (
                (
                    "edit_preview_read",
                    {"preview_sha256": handle, "file_index": 0, "side": "after"},
                ),
                (
                    "edit_preview_apply",
                    {
                        "preview_sha256": handle,
                        "receipt": "mcp-assessment-rename-receipt.json",
                    },
                ),
                ("edit_preview_reconcile", {"preview_sha256": handle}),
                ("edit_preview_release", {"preview_sha256": handle}),
            ):
                reply = await selected(endpoint, capability, arguments)
                calls.append(reply)
                require(not reply["is_error"], capability + " failed after rename")
            require(
                path.read_text() == original.replace("add_one", "add_two"),
                "Rename result differs from independently expected source bytes",
            )
        return {
            "status": "passed",
            "scope": "real Python incoming/outgoing calls and rename apply, independent exact expected source bytes",
        }
    finally:
        write(output / "navigation-extended-calls.json", calls)
        path.unlink(missing_ok=True)
        (root / "mcp-assessment-rename-receipt.json").unlink(missing_ok=True)


async def navigation_rust(root: Path, output: Path) -> dict:
    path = "crates/pse-ids/src/float.rs"
    calls = []
    try:
        async with client(root) as endpoint:
            coordinates = {
                "language": "rust",
                "file_path": path,
                "line": 42,
                "character": 13,
            }
            requests = [
                ("navigation_tool_support", {"language": "rust", "file_path": path}),
                ("navigation_definition", coordinates),
                ("navigation_hover", coordinates),
                ("navigation_references", {**coordinates, "include_declaration": True}),
                ("navigation_type_definition", coordinates),
                ("navigation_declaration", coordinates),
                ("navigation_implementations", coordinates),
                ("navigation_completions", coordinates),
                ("navigation_signature", {**coordinates, "line": 41, "character": 17}),
                ("navigation_symbols", {"language": "rust", "file_path": path}),
                ("navigation_diagnostics", {"language": "rust", "file_path": path}),
                (
                    "navigation_workspace_symbols",
                    {"language": "rust", "query": "canonical_f64_bits", "limit": 10},
                ),
                (
                    "navigation_call_hierarchy",
                    {**coordinates, "line": 40, "character": 10},
                ),
                (
                    "navigation_code_actions",
                    {
                        "language": "rust",
                        "file_path": path,
                        "start_line": 40,
                        "start_character": 1,
                        "end_line": 40,
                        "end_character": 40,
                    },
                ),
            ]
            for capability, arguments in requests:
                calls.append(await selected(endpoint, capability, arguments))
                write(output / "navigation-rust-calls.json", calls)
            definition = next(
                row for row in calls if row["capability"] == "navigation_definition"
            )
            require(not definition["is_error"], "Rust definition transport failed")
            native = unwrapped(definition["structured_content"])["payload"]["native"]
            require(not native["isError"], "Rust native definition failed")
            require(
                "float.rs" in json.dumps(native) and '"line": 23' in json.dumps(native),
                "Rust definition differs from expected constant declaration",
            )
            hierarchy = next(
                row for row in calls if row["capability"] == "navigation_call_hierarchy"
            )
            items = (
                unwrapped(hierarchy["structured_content"])["payload"]["native"][
                    "result"
                ].get("items", [])
                if not hierarchy["is_error"]
                else []
            )
            if items:
                for capability in (
                    "navigation_incoming_calls",
                    "navigation_outgoing_calls",
                ):
                    calls.append(
                        await selected(
                            endpoint, capability, {"language": "rust", "item": items[0]}
                        )
                    )
            return {
                "status": "passed",
                "scope": "real Rust language provider over fresh STDIO; independent constant definition line assertion",
                "hierarchy_items": len(items),
            }
    finally:
        write(output / "navigation-rust-calls.json", calls)


async def code_action(root: Path, output: Path) -> dict:
    path = "crates/pse-ids/src/float.rs"
    original = (root / path).read_text()
    calls = []
    try:
        async with client(root) as endpoint:
            request = {
                "language": "rust",
                "file_path": path,
                "start_line": 40,
                "start_character": 1,
                "end_line": 40,
                "end_character": 40,
            }
            available = await selected(endpoint, "navigation_code_actions", request)
            calls.append(available)
            require(not available["is_error"], "Rust code-action discovery failed")
            actions = unwrapped(available["structured_content"])["payload"]["native"][
                "result"
            ]["actions"]
            index = next(
                i
                for i, action in enumerate(actions)
                if action.get("title") == "Change Visibility to pub(crate)"
            )
            preview = await selected(
                endpoint,
                "navigation_action_preview",
                {**request, "owned": [path], "action_index": index},
            )
            calls.append(preview)
            require(not preview["is_error"], "Rust selected code-action preview failed")
            handle = unwrapped(preview["structured_content"])["payload"]["preview"][
                "preview_sha256"
            ]
            pieces = []
            offset = 0
            while offset is not None:
                detail = await selected(
                    endpoint,
                    "edit_preview_read",
                    {
                        "preview_sha256": handle,
                        "file_index": 0,
                        "side": "after",
                        "offset": offset,
                    },
                )
                calls.append(detail)
                require(not detail["is_error"], "Rust action detail failed")
                body = unwrapped(detail["structured_content"])["payload"]
                require(
                    body["offset"] == offset,
                    "Preview continuation changed selected offset",
                )
                pieces.append(body["text"])
                offset = body["next_offset"]
            expected = original.replace(
                "pub fn canonical_f64_bits", "pub(crate) fn canonical_f64_bits", 1
            )
            require(
                "".join(pieces) == expected,
                "Code-action preview differs from independent visibility change",
            )
            calls.append(
                await selected(
                    endpoint, "edit_preview_release", {"preview_sha256": handle}
                )
            )
            require(
                (root / path).read_text() == original,
                "Code-action preview mutated source",
            )
        return {
            "status": "passed",
            "scope": "real Rust selected source code-action preview, independently expected full source; no apply",
        }
    finally:
        write(output / "code-action-calls.json", calls)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--worktree", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument(
        "--stage",
        choices=(
            "edit_stdio",
            "edit_cli",
            "navigation_python",
            "navigation_extended",
            "navigation_rust",
            "code_action",
        ),
        action="append",
    )
    args = parser.parse_args()
    root = args.worktree.resolve(strict=True)
    require(
        root != ROOT and (root / ".git").is_file(),
        "Supply the owned disposable worktree",
    )
    args.output.mkdir(parents=True, exist_ok=False)
    result = {
        "source_revision": subprocess.check_output(
            ["git", "rev-parse", "HEAD"], text=True
        ).strip(),
        "worktree": str(root),
        "baseline_failures": 0,
        "stages": {},
    }
    for name, action in (
        ("edit_stdio", lambda: asyncio.run(edit_stdio(root, args.output))),
        ("edit_cli", lambda: edit_cli(root, args.output)),
        ("navigation_python", lambda: asyncio.run(navigation(root, args.output))),
        (
            "navigation_extended",
            lambda: asyncio.run(navigation_extended(root, args.output)),
        ),
        ("navigation_rust", lambda: asyncio.run(navigation_rust(root, args.output))),
        ("code_action", lambda: asyncio.run(code_action(root, args.output))),
    ):
        if args.stage and name not in args.stage:
            continue
        started = time.monotonic()
        answer: dict[str, object]
        try:
            answer = action()
        except Exception as error:
            answer = {
                "status": "failed",
                "error_type": type(error).__name__,
                "message": str(error),
            }
        answer["elapsed_seconds"] = time.monotonic() - started
        result["stages"][name] = answer
        write(args.output / "acceptance.json", result)
    return int(any(stage["status"] != "passed" for stage in result["stages"].values()))


if __name__ == "__main__":
    raise SystemExit(main())
