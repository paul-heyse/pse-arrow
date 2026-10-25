# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit source-only two-checkout STDIO identity/reveal/restart acceptance.

The operator supplies a disposable, clean Git worktree of this repository.
Only its source-only environment query is temporarily edited and restored.
No native artifact, database, compiler or workload action is selected.
"""
from __future__ import annotations

import argparse
import asyncio
import json
import os
import subprocess
from pathlib import Path

from fastmcp import Client
from fastmcp.client.transports import StdioTransport

ROOT = Path(__file__).resolve().parents[2]


def transport(root: Path, *, explicit: bool) -> StdioTransport:
    return StdioTransport(
        command=str(root / ".venv/bin/python"), args=["-m", "scripts.capabilities"],
        cwd=str(root), keep_alive=False,
        env={**os.environ, "UV_PROJECT_ENVIRONMENT": str(root / ".venv"),
             "CARGO_BUILD_JOBS": "16" if explicit else "1",
             "PSE_CARGO_DEFAULT_JOBS": "32" if explicit else "1"},
    )


async def invoke(client: Client, capability: str, arguments: dict | None = None) -> dict:
    result = await client.call_tool("capabilities_invoke_read", {"capability": capability, "arguments": arguments or {}})
    if result.is_error or not isinstance(result.structured_content, dict):
        raise AssertionError("selected canonical read did not return its declared object")
    return result.structured_content


async def run(worktree: Path) -> dict:
    worktree = worktree.resolve(strict=True)
    if worktree == ROOT or not (worktree / ".git").is_file():
        raise ValueError("select a separate disposable Git worktree")
    common = lambda root: subprocess.check_output(["git", "rev-parse", "--path-format=absolute", "--git-common-dir"], cwd=root, text=True).strip()
    if common(ROOT) != common(worktree):
        raise ValueError("worktree belongs to another repository")
    dirty = subprocess.check_output(["git", "status", "--porcelain", "--untracked-files=all"], cwd=worktree, text=True)
    if dirty:
        raise ValueError("acceptance worktree has unowned edits")
    file = worktree / "scripts/capabilities/environment_queries.py"
    original = file.read_bytes()
    observations = []
    async with Client(transport(ROOT, explicit=True)) as main, Client(transport(worktree, explicit=False)) as other:
        for client, root in ((main, ROOT), (other, worktree)):
            selected = await client.call_tool("capabilities_select", {"capabilities": ["environment_explain"]})
            receipt = selected.structured_content
            if not isinstance(receipt, dict) or receipt["interface"]["checkout"] != str(root):
                raise AssertionError("attachment identity differs from selected checkout")
            observed = await invoke(client, "environment_explain")
            payload = observed["payload"]
            if payload["checkout"] != str(root) or payload["interpreter"] != str(root / ".venv/bin/python"):
                raise AssertionError("environment observation came from another checkout")
            observations.append({"checkout": str(root), "compiler": payload["compiler"], "interface": receipt["interface"]})
        first, second = (item["compiler"]["CARGO_BUILD_JOBS"] for item in observations)
        if first["value"] != "16" or first["source"] != "caller" or second["value"] != "32" or second["source"] != "default":
            raise AssertionError("explicit selection or refreshed derived default was lost")
        await other.call_tool("capabilities_reset", {})
        if "environment_explain" not in {tool.name for tool in await main.list_tools()} or "environment_explain" in {tool.name for tool in await other.list_tools()}:
            raise AssertionError("independent attachments shared reveal state")
        await other.call_tool("capabilities_select", {"capabilities": ["environment_explain"]})
        try:
            file.write_bytes(original + b"\n# Disposable acceptance source-basis change.\n")
            refused = await other.call_tool("capabilities_invoke_read", {"capability": "environment_explain", "arguments": {}}, raise_on_error=False)
            if not refused.is_error or "pse.capabilities.restart_required" not in str(refused):
                raise AssertionError("changed serving implementation was not refused")
            await invoke(main, "environment_explain")
        finally:
            file.write_bytes(original)
    async with Client(transport(worktree, explicit=False)) as restarted:
        await restarted.call_tool("capabilities_select", {"capabilities": ["environment_explain"]})
        await invoke(restarted, "environment_explain")
    if subprocess.check_output(["git", "status", "--porcelain", "--untracked-files=all"], cwd=worktree, text=True):
        raise AssertionError("owned source fixture was not restored")
    return {"status": "passed", "baseline": "zero failures", "observations": observations,
            "independent_reveal": True, "relevant_restart_refused": True, "reattachment_passed": True,
            "worktree_source_restored": True,
            "scope": "Two real source-only STDIO attachments with distinct checkout/interpreter/environment identity; no native artifact or scientific execution claim."}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--worktree", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    result = asyncio.run(run(args.worktree))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open("x") as destination:
        json.dump(result, destination, indent=2)
    print(result["status"])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
