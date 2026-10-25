# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent presence and lifetime controls for the schema-derived client."""

from __future__ import annotations

import tempfile
import sys
import os
import shutil
import subprocess
import json
from contextlib import asynccontextmanager
import types
import unittest
from typing import cast
from pathlib import Path
from unittest.mock import patch

from fastmcp.tools import Tool
from fastmcp import Client
from fastmcp.client.client import CallToolResult
from mcp.types import TextContent, ToolAnnotations

from scripts import capability_cli_runtime as runtime
from scripts.capabilities.cli_codegen import check, publish, render_tools
from scripts.capabilities.execution_basis import digest, wire
from scripts.capabilities.model import Capability
from scripts.capabilities.model import Domain
from scripts.capabilities.catalogue import Catalogue


def echo(value: str | None = "DEFAULT", items: list[str] | None = None, flag: bool = False, count: int = 7) -> dict[str, object]:
    return {"value": value, "items": items, "flag": flag, "count": count}


class GeneratedClientTests(unittest.TestCase):
    def test_lifetime_is_authoritative_wire_metadata(self) -> None:
        tool = Tool.from_function(echo)
        spec = Capability(id="echo", title="Echo", description="Fixture", domain="workspace", owner="fixture", source="fixture", exposure="tool", effects="read", invocation_lifetime="single")
        domains = (Domain(id="workspace", title="Workspace", description="Fixture"),)
        Catalogue(domains, (), (), (spec,), {"echo": tool})
        before = digest(wire(tool))
        self.assertEqual(tool.meta["pse.invocation_lifetime"], "single")
        Catalogue(domains, (), (), (spec.model_copy(update={"invocation_lifetime": "endpoint"}),), {"echo": tool})
        self.assertNotEqual(before, digest(wire(tool)))

    def generated(self) -> tuple[str, types.ModuleType]:
        tool = Tool.from_function(echo, annotations=ToolAnnotations(readOnlyHint=True))
        spec = Capability(id="echo", title="Echo", description="Fixture", domain="workspace", owner="fixture", source="fixture", exposure="tool", effects="read", invocation_lifetime="single")
        content = render_tools({"echo": tool}, {"echo": spec}, {"echo": wire(tool)})
        module = types.ModuleType("generated_client_fixture")
        # Cyclopts resolves postponed annotations using the registered module.
        sys.modules[module.__name__] = module
        self.addCleanup(sys.modules.pop, module.__name__, None)
        exec(compile(content, "<generated-client>", "exec", dont_inherit=True), module.__dict__)
        self.assertEqual(module.OPERATIONS["echo"]["wire_revision"], digest(wire(tool)))
        return content, module

    def test_presence_is_not_filtered_or_replaced_by_schema_defaults(self) -> None:
        _, module = self.generated()
        calls = []
        with patch.object(module, "call", lambda name, arguments, expected: calls.append((name, arguments))):
            app = module.register()
            app(["echo"], exit_on_error=False, result_action="return_value")
            app(["echo", "--value", "null", "--items", "[]", "--flag", "--count", "0"], exit_on_error=False, result_action="return_value")
            app(["echo", "--no-flag"], exit_on_error=False, result_action="return_value")
        self.assertEqual(calls[0], ("echo", {}))
        self.assertEqual(calls[1], ("echo", {"value": None, "items": [], "flag": True, "count": 0}))
        self.assertEqual(calls[2], ("echo", {"flag": False}))

    def test_exact_generator_inventory_and_nonmutating_comparison(self) -> None:
        content, _ = self.generated()
        self.assertNotIn(str(Path.cwd()), content)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "output"
            self.assertTrue(check(content, root))
            self.assertFalse(root.exists())
            publish(content, root)
            self.assertFalse(check(content, root))
            (root / "unexpected.py").write_text("not owned", encoding="utf-8")
            with self.assertRaises(ValueError):
                publish(content, root)
            self.assertEqual((root / "unexpected.py").read_text(), "not owned")

    def test_raw_request_preserves_presence_and_rejects_nonobjects(self) -> None:
        self.assertEqual(runtime.parse_request('{"value":null,"items":[],"flag":false,"count":0}'), {"value": None, "items": [], "flag": False, "count": 0})
        with self.assertRaises(ValueError):
            runtime.parse_request("[]")
        with self.assertRaises(runtime.ClientRefusedError):
            runtime.parse_request('{"value":1,"value":2}')
        with self.assertRaises(ValueError):
            runtime.parse_request('{"value":NaN}')

    def test_offline_diagnosis_survives_broken_serving_sources_and_missing_generation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            scripts = root / "scripts"
            scripts.mkdir()
            (scripts / "__init__.py").write_text("")
            for name in ("capability_cli.py", "capability_cli_runtime.py"):
                shutil.copyfile(runtime.checkout_root() / "scripts" / name, scripts / name)
            (scripts / "capabilities").mkdir()
            (scripts / "capabilities/__init__.py").write_text("raise RuntimeError('serving source must never import')\n")
            (scripts / "capabilities/server.py").write_text("invalid syntax !!!\n")
            result = subprocess.run([sys.executable, "-m", "scripts.capability_cli", "diagnose"], cwd=root, env={**os.environ, "PYTHONPATH": str(root)}, text=True, capture_output=True, check=False, timeout=20)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(json.loads(result.stdout)["connection"], "not_probed")
            self.assertFalse(json.loads(result.stdout)["generated_cli_available"])


class FixedClient:
    def __init__(self, revision: str = "wire") -> None:
        self.revision = revision
        self.calls: list[str] = []

    async def call_tool(self, name: str, arguments: object, **options: object) -> CallToolResult:
        self.calls.append(name)
        if name == "capabilities_describe":
            return CallToolResult(content=[], structured_content={"interface": {"operation_wire_revision": self.revision, "restart_required": False}}, meta=None)
        if name == "capabilities_select":
            return CallToolResult(content=[], structured_content={"selected": True}, meta=None)
        return CallToolResult(content=[TextContent(type="text", text="original content")], structured_content={"reference": "original-owner"}, meta={"observation": "original"})


class ConnectionTests(unittest.IsolatedAsyncioTestCase):
    async def test_describe_refusal_preserves_original_and_never_selects(self) -> None:
        class RefusingClient(FixedClient):
            async def call_tool(self, name: str, arguments: object, **options: object) -> CallToolResult:
                self.calls.append(name)
                return CallToolResult(content=[TextContent(type="text", text='{"code":"original-refusal"}')], structured_content={"code": "original-refusal"}, meta={"owner": "original"}, is_error=True)
        client = RefusingClient()
        result = await runtime.invoke_on(cast(Client, client), "echo", {}, {"wire_revision": "wire", "lifetime": "single", "read_only": True})
        self.assertTrue(result["is_error"])
        self.assertEqual(result["meta"], {"owner": "original"})
        self.assertEqual(client.calls, ["capabilities_describe"])

    async def test_regular_file_jsonl_retains_one_connection_and_settles_eof(self) -> None:
        client = FixedClient()
        lifecycle = []
        @asynccontextmanager
        async def connected(root: Path):
            lifecycle.append("opened")
            try:
                yield cast(Client, client)
            finally:
                lifecycle.append("closed")
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "requests.jsonl"
            source.write_text('{"id":"first","capability":"echo","arguments":{}}\n{"id":"second","capability":"echo","arguments":{}}\n')
            replies = []
            with source.open() as stream, patch.object(runtime.sys, "stdin", stream), patch.object(runtime, "connection", connected), patch.object(runtime, "emit", replies.append):
                await runtime.session({"echo": {"wire_revision": "wire", "lifetime": "endpoint", "read_only": True}})
        self.assertEqual(lifecycle, ["opened", "closed"])
        self.assertEqual([row.get("id") for row in replies[1:]], ["first", "second"])
        self.assertEqual(client.calls.count("capabilities_invoke_read"), 2)

    async def test_one_client_selects_and_preserves_complete_result(self) -> None:
        client = FixedClient()
        result = await runtime.invoke_on(cast(Client, client), "echo", {}, {"wire_revision": "wire", "lifetime": "single", "read_only": True})
        self.assertEqual(client.calls, ["capabilities_describe", "capabilities_select", "capabilities_invoke_read"])
        self.assertEqual(result["structured_content"], {"reference": "original-owner"})
        self.assertEqual(result["meta"], {"observation": "original"})
        self.assertEqual(result["content"][0]["text"], "original content")

    async def test_stale_contract_and_endpoint_lifetime_refuse_before_effects(self) -> None:
        client = FixedClient("changed")
        expected = {"wire_revision": "wire", "lifetime": "single", "read_only": False}
        with self.assertRaises(runtime.ClientRefusedError):
            await runtime.invoke_on(cast(Client, client), "echo", {}, expected)
        self.assertEqual(client.calls, ["capabilities_describe"])
        client = FixedClient()
        expected["lifetime"] = "endpoint"
        with self.assertRaises(runtime.ClientRefusedError):
            await runtime.invoke_on(cast(Client, client), "echo", {}, expected)
        self.assertEqual(client.calls, [])
        await runtime.invoke_on(cast(Client, client), "echo", {}, expected, retained=True)
        self.assertEqual(client.calls[-1], "capabilities_invoke_action")


class DiagnosticProbeTests(unittest.IsolatedAsyncioTestCase):
    async def test_missing_launch_and_failed_initialization_are_independent(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with patch.object(runtime, "checkout_root", return_value=root), patch.object(runtime, "connection") as connect:
                missing = await runtime.probe()
            self.assertEqual(missing["classification"], "missing_checkout_interpreter")
            self.assertEqual(missing["stage"], "configuration")
            connect.assert_not_called()
            interpreter = root / ".venv/bin/python"
            interpreter.parent.mkdir(parents=True)
            interpreter.touch()
            @asynccontextmanager
            async def broken(_root: Path):
                raise OSError("sensitive details must not escape")
                yield  # pragma: no cover -- defines the failing context manager.
            with patch.object(runtime, "checkout_root", return_value=root), patch.object(runtime, "connection", broken):
                failed = await runtime.probe()
            self.assertEqual(failed["stage"], "endpoint_initialization")
            self.assertEqual(failed["connection"], "unavailable")
            self.assertNotIn("sensitive details", json.dumps(failed))

    async def test_stale_handshake_is_not_reported_as_refreshed_attachment(self) -> None:
        class StaleClient(FixedClient):
            async def call_tool(self, name: str, arguments: object, **options: object) -> CallToolResult:
                self.calls.append(name)
                return CallToolResult(content=[], structured_content={"interface": {"restart_required": True}}, meta=None)
        client = StaleClient()
        @asynccontextmanager
        async def connected(_root: Path):
            yield cast(Client, client)
        with patch.object(runtime, "connection", connected):
            report = await runtime.probe()
        self.assertEqual(report["classification"], "stale_loaded_interface")
        self.assertEqual(client.calls, ["capabilities_domains"])
        self.assertIn("no setup", report["effect"])


if __name__ == "__main__":
    unittest.main()
