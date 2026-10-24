# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Pinned semantic navigation, real requests and source-bound edit previews."""

from __future__ import annotations

import argparse
import asyncio
import json
import os
import shutil
import subprocess
import tempfile
import tomllib
from contextlib import asynccontextmanager
from pathlib import Path
from typing import TYPE_CHECKING

from scripts import build_environment, diagnostic_trace

if TYPE_CHECKING:
    from collections.abc import Mapping

ROOT = Path(__file__).resolve().parents[1]


def configuration(
    root: Path,
    language: str = "python",
    environment: Mapping[str, str] | None = None,
    *,
    fixture: bool = False,
) -> str:
    """Resolve execution paths once; neither import nor configuration starts a server."""
    if language not in {"python", "rust"}:
        raise ValueError("navigation language must be python or rust")
    if language == "rust":
        if fixture and (
            not (root / "rust-toolchain.toml").is_file()
            or (root / "rust-toolchain.toml").read_bytes()
            != (ROOT / "rust-toolchain.toml").read_bytes()
        ):
            raise ValueError(
                "Rust fixture must carry a source copy of this checkout's toolchain declaration"
            )
        settings = (
            {"cargo": {"targetDir": True}}
            if fixture
            else tomllib.loads((root / "rust-analyzer.toml").read_text())
        )
        analyzer = shutil.which("rust-analyzer")
        if analyzer is None:
            raise ValueError(
                "pinned rust-analyzer is unavailable; install the pinned rustup component"
            )
        selected = build_environment.language_server_environment(
            os.environ if environment is None else environment
        )
        server_environment = "[lsp_servers.env]\n" + "".join(
            json.dumps(key) + " = " + json.dumps(value) + "\n"
            for key, value in selected.items()
        )
        native = (
            '[[lsp_servers]]\nlanguage_id = "rust"\ncommand = '
            + json.dumps(analyzer)
            + '\nargs = []\nfile_patterns = ["**/*.rs"]\ntimeout_seconds = 45\nrequest_timeout_seconds = 45\n'
            + server_environment
            + "[lsp_servers.initialization_options]\ncheck.enable = false\n"
        )
        for group, values in settings.items():
            for key, value in values.items():
                native += f"{group}.{key} = " + json.dumps(value) + "\n"
    else:
        native = (
            '[[lsp_servers]]\nlanguage_id = "python"\n'
            "command = "
            + json.dumps(str((ROOT if fixture else root) / ".venv/bin/pyrefly"))
            + "\n"
            'args = ["lsp"]\nfile_patterns = ["**/*.py", "**/*.pyi"]\n'
            "timeout_seconds = 30\nrequest_timeout_seconds = 45\n"
        )
    return (
        "[workspace]\nroots = " + json.dumps([str(root)]) + "\n"
        'position_encodings = ["utf-8", "utf-16"]\n'
        "max_documents = 100\nmax_file_size = 10485760\n"
        "indexing_ready_timeout_seconds = 45\n" + native
    )


@asynccontextmanager
async def connection(root: Path, language: str, *, fixture: bool = False):
    """Own exactly one bridge transport; acquire only when explicitly entered."""
    from fastmcp import Client
    from fastmcp.client.transports import StdioTransport

    with tempfile.TemporaryDirectory(prefix="pse-navigation-") as directory:
        config = Path(directory) / "mcpls.toml"
        config.write_text(configuration(root, language, fixture=fixture))
        transport = StdioTransport(
            command=str(diagnostic_trace.binary_path("mcpls")),
            args=["--config", str(config)],
            cwd=str(root),
            keep_alive=False,
        )
        try:
            async with Client(transport, timeout=100, init_timeout=100) as client:
                yield client
        finally:
            await transport.close()


async def perform(client, calls: list[dict]) -> list[dict]:
    """Preserve real protocol/domain errors and bounded startup-readiness retries."""
    from mcp import MCPError

    results = []
    for call in calls:
        readiness = []
        deadline = asyncio.get_running_loop().time() + 45
        while True:
            try:
                response = await client.call_tool(
                    call["name"], call.get("arguments", {}), raise_on_error=False
                )
                break
            except MCPError as error:
                detail = {"code": error.code, "message": error.message, "data": error.data}
                if error.code == -32051 and asyncio.get_running_loop().time() < deadline:
                    readiness.append(detail)
                    await asyncio.sleep(1)
                    continue
                results.append({"name": call["name"], "isError": True,
                                "result": {"error": detail}, "readiness": readiness})
                response = None
                break
        if response is None:
            continue
        payload = response.structured_content
        if payload is None:
            texts = [item.text for item in response.content if item.type == "text"]
            try:
                payload = json.loads("\n".join(texts))
            except json.JSONDecodeError:
                payload = {"text": texts}
        results.append({"name": call["name"], "isError": response.is_error,
                        "result": payload, "readiness": readiness,
                        "content": [item.model_dump(mode="json", by_alias=True,
                                                   exclude_none=True)
                                    for item in response.content]})
    return results


async def request(
    root: Path, language: str, calls: list[dict], *, fixture: bool = False
) -> dict:
    """Run finite real requests; mcpls positions use one-based UTF-16."""
    async with connection(root, language, fixture=fixture) as client:
        catalogue = await client.list_tools()
        return {
            "language": language,
            "root": str(root.resolve()),
            "protocol": client.protocol_version,
            "tools": [{"name": tool.name, "inputSchema": tool.input_schema}
                      for tool in catalogue],
            "results": await perform(client, calls),
        }


def edit_result(payload: dict) -> dict:
    """Convert mcpls one-based UTF-16 edit coordinates to the shared contract."""
    import copy

    from scripts import semantic_edits

    semantic_edits._complete(payload)
    selected = copy.deepcopy(payload)
    for change in selected.get("changes", []):
        for edit in change.get("edits", []):
            for endpoint in ("start", "end"):
                position = edit["range"][endpoint]
                if min(position["line"], position["character"]) < 1:
                    raise ValueError("invalid mcpls one-based position")
                position["line"] -= 1
                position["character"] -= 1
    return selected


def main() -> int:
    from scripts import semantic_edits

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "command",
        choices=("serve", "describe", "request", "preview", "apply", "reconcile"),
    )
    parser.add_argument("--language", choices=("python", "rust"), default="python")
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument(
        "--calls", type=Path, help="JSON list of MCP tool name/arguments requests"
    )
    parser.add_argument(
        "--owned", nargs="+", help="all owned source paths captured before requests"
    )
    parser.add_argument(
        "--preview", type=Path, help="reviewed preview file for apply/reconcile"
    )
    parser.add_argument(
        "--receipt", type=Path, help="application/reconciliation result destination"
    )
    parser.add_argument(
        "--action-index",
        type=int,
        help="select an editable code action from the final response",
    )
    parser.add_argument(
        "--fixture",
        action="store_true",
        help="small isolated workspace using checkout-pinned servers",
    )
    args = parser.parse_args()
    if args.command in {"apply", "reconcile"}:
        if not args.preview or not args.receipt:
            parser.error("apply/reconcile require --preview and --receipt")
        selected = json.loads(args.preview.read_bytes())
        result = (
            semantic_edits.apply(selected, args.receipt)
            if args.command == "apply"
            else semantic_edits.reconcile(selected)
        )
        if args.command == "reconcile":
            args.receipt.write_text(json.dumps(result, indent=2) + "\n")
        print(json.dumps(result, indent=2))
        return 0 if result["complete"] else 1
    binary = diagnostic_trace.binary_path("mcpls")
    if not binary.is_file() or (
        args.language == "python" and not (ROOT / ".venv/bin/pyrefly").is_file()
    ):
        parser.exit(
            125,
            "navigation unavailable: explicitly install mcpls and sync pinned checkout tools\n",
        )
    if args.command == "describe":
        print(
            json.dumps(
                {
                    "adapter": str(binary),
                    "server": str(ROOT / ".venv/bin/pyrefly")
                    if args.language == "python"
                    else shutil.which("rust-analyzer"),
                    "checkout": str(args.root),
                    "available": True,
                    "attached": "not observed",
                    "coverage": "readiness and operation support require real requests; edits require explicit owned source cuts",
                }
            )
        )
        return 0
    if args.command in {"request", "preview"}:
        if not args.calls:
            parser.error("request/preview require --calls")
        if args.command == "preview" and not args.owned:
            parser.error("preview requires --owned source paths before the request")
        cut = semantic_edits.capture(args.root, args.owned or [])
        observed = asyncio.run(
            request(
                args.root,
                args.language,
                json.loads(args.calls.read_bytes()),
                fixture=args.fixture,
            )
        )
        if args.command == "request":
            print(json.dumps(observed, indent=2))
            return 0 if all(not item["isError"] for item in observed["results"]) else 1
        last = observed["results"][-1]
        if last["isError"]:
            raise ValueError(
                "server refused edit request: " + json.dumps(last["result"])
            )
        payload = last["result"]
        semantic_edits._complete(payload)
        if args.action_index is not None:
            action = payload["actions"][args.action_index]
            if action.get("command"):
                raise ValueError(
                    "command-based code actions cannot be represented as a complete source edit"
                )
            payload = action["edit"]
        print(
            json.dumps(
                semantic_edits.preview(
                    args.root,
                    cut,
                    edit_result(payload),
                    producer="mcpls:" + args.language,
                ),
                indent=2,
            )
        )
        return 0
    with tempfile.TemporaryDirectory(prefix="pse-navigation-") as directory:
        config = Path(directory) / "mcpls.toml"
        config.write_text(configuration(args.root, args.language, fixture=args.fixture))
        return subprocess.call([str(binary), "--config", str(config)], cwd=args.root)


if __name__ == "__main__":
    raise SystemExit(main())
