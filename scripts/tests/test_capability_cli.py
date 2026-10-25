# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent presence and lifetime controls for the schema-derived client."""

from __future__ import annotations

import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from contextlib import asynccontextmanager
from pathlib import Path
from typing import TYPE_CHECKING, cast
from unittest.mock import patch

import pytest
from fastmcp.client.client import CallToolResult
from fastmcp.tools import Tool
from mcp.types import TextContent, ToolAnnotations
from pydantic import ValidationError

from scripts import capability_cli_runtime as runtime
from scripts.capabilities.catalogue import Catalogue
from scripts.capabilities.cli_codegen import check, publish, render_tools
from scripts.capabilities.execution_basis import digest, wire
from scripts.capabilities.model import Capability, Domain

if TYPE_CHECKING:
    import types
    from collections.abc import AsyncGenerator

    from fastmcp import Client


def echo(
    value: str | None = "DEFAULT",
    items: list[str] | None = None,
    flag: bool = False,
    count: int = 7,
) -> dict[str, object]:
    return {"value": value, "items": items, "flag": flag, "count": count}


class GeneratedClientTests(unittest.TestCase):
    def test_connection_retains_original_selected_environment_without_logging(
        self,
    ) -> None:
        selected = {
            "PSE_CAPABILITIES_DATABASE_STATE": "/selected/state",
            "LD_LIBRARY_PATH": "/selected/native/lib",
            "OTEL_SDK_DISABLED": "true",
        }
        with (
            patch.dict(os.environ, selected),
            patch("fastmcp.Client") as client,
            patch("fastmcp.client.transports.StdioTransport") as transport,
        ):
            runtime.connection(Path("/selected/checkout"))
        assert client.call_count == 1
        assert transport.call_args is not None
        configuration = transport.call_args.kwargs
        assert configuration["cwd"] == "/selected/checkout"
        assert configuration["command"] == "/selected/checkout/.venv/bin/python"
        assert all(
            configuration["env"][key] == value for key, value in selected.items()
        )

    def test_lifetime_is_authoritative_wire_metadata(self) -> None:
        tool = Tool.from_function(echo)
        spec = Capability(
            id="echo",
            title="Echo",
            description="Fixture",
            domain="workspace",
            owner="fixture",
            source="fixture",
            exposure="tool",
            effects="read",
            invocation_lifetime="single",
        )
        domains = (Domain(id="workspace", title="Workspace", description="Fixture"),)
        Catalogue(domains, (), (), (spec,), {"echo": tool})
        before = digest(wire(tool))
        assert tool.meta is not None
        assert (tool.meta["pse.invocation_lifetime"]) == ("single")
        Catalogue(
            domains,
            (),
            (),
            (spec.model_copy(update={"invocation_lifetime": "endpoint"}),),
            {"echo": tool},
        )
        assert (before) != (digest(wire(tool)))

    def generated(self) -> tuple[str, types.ModuleType]:
        tool = Tool.from_function(echo, annotations=ToolAnnotations(readOnlyHint=True))
        spec = Capability(
            id="echo",
            title="Echo",
            description="Fixture",
            domain="workspace",
            owner="fixture",
            source="fixture",
            exposure="tool",
            effects="read",
            invocation_lifetime="single",
        )
        content = render_tools({"echo": tool}, {"echo": spec}, {"echo": wire(tool)})
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        source = Path(directory.name) / "generated_client_fixture.py"
        source.write_text(content, encoding="utf-8")
        spec = importlib.util.spec_from_file_location(
            "generated_client_fixture", source
        )
        assert spec is not None
        assert spec.loader is not None
        module = importlib.util.module_from_spec(spec)
        sys.modules[module.__name__] = module
        self.addCleanup(sys.modules.pop, module.__name__, None)
        spec.loader.exec_module(module)
        assert (module.OPERATIONS["echo"]["wire_revision"]) == (digest(wire(tool)))
        return content, module

    def test_presence_is_not_filtered_or_replaced_by_schema_defaults(self) -> None:
        _, module = self.generated()
        calls = []
        with patch.object(
            module,
            "call",
            lambda name, arguments, _expected: calls.append((name, arguments)),
        ):
            app = module.register()
            app(["echo"], exit_on_error=False, result_action="return_value")
            app(
                ["echo", "--value", "null", "--items", "[]", "--flag", "--count", "0"],
                exit_on_error=False,
                result_action="return_value",
            )
            app(
                ["echo", "--no-flag"], exit_on_error=False, result_action="return_value"
            )
        assert (calls[0]) == (("echo", {}))
        assert (calls[1]) == (
            ("echo", {"value": None, "items": [], "flag": True, "count": 0})
        )
        assert (calls[2]) == (("echo", {"flag": False}))

    def test_exact_generator_inventory_and_nonmutating_comparison(self) -> None:
        content, _ = self.generated()
        assert (str(Path.cwd())) not in (content)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "output"
            assert check(content, root)
            assert not (root.exists())
            publish(content, root)
            assert not (check(content, root))
            (root / "unexpected.py").write_text("not owned", encoding="utf-8")
            with pytest.raises(ValueError, match="unexpected generated CLI entry"):
                publish(content, root)
            assert ((root / "unexpected.py").read_text()) == ("not owned")

    def test_raw_request_preserves_presence_and_rejects_nonobjects(self) -> None:
        assert (
            runtime.parse_request('{"value":null,"items":[],"flag":false,"count":0}')
        ) == ({"value": None, "items": [], "flag": False, "count": 0})
        with pytest.raises(ValidationError, match="dictionary"):
            runtime.parse_request("[]")
        with pytest.raises(runtime.ClientRefusedError):
            runtime.parse_request('{"value":1,"value":2}')
        with pytest.raises(ValueError, match="Out of range float"):
            runtime.parse_request('{"value":NaN}')

    def test_offline_diagnosis_survives_broken_serving_sources_and_missing_generation(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            scripts = root / "scripts"
            scripts.mkdir()
            (scripts / "__init__.py").write_text("")
            for name in ("capability_cli.py", "capability_cli_runtime.py"):
                shutil.copyfile(
                    runtime.checkout_root() / "scripts" / name, scripts / name
                )
            (scripts / "capabilities").mkdir()
            (scripts / "capabilities/__init__.py").write_text(
                "raise RuntimeError('serving source must never import')\n"
            )
            (scripts / "capabilities/server.py").write_text("invalid syntax !!!\n")
            result = subprocess.run(
                [sys.executable, "-m", "scripts.capability_cli", "diagnose"],
                cwd=root,
                env={**os.environ, "PYTHONPATH": str(root)},
                text=True,
                capture_output=True,
                check=False,
                timeout=20,
            )
            assert (result.returncode) == (0), result.stderr
            assert (json.loads(result.stdout)["connection"]) == ("not_probed")
            assert not (json.loads(result.stdout)["generated_cli_available"])


class FixedClient:
    def __init__(self, revision: str = "wire") -> None:
        self.revision = revision
        self.calls: list[str] = []

    async def call_tool(
        self, name: str, _arguments: object, **_options: object
    ) -> CallToolResult:
        self.calls.append(name)
        if name == "capabilities_describe":
            return CallToolResult(
                content=[],
                structured_content={
                    "interface": {
                        "operation_wire_revision": self.revision,
                        "restart_required": False,
                    }
                },
                meta=None,
            )
        if name == "capabilities_select":
            return CallToolResult(
                content=[], structured_content={"selected": True}, meta=None
            )
        return CallToolResult(
            content=[TextContent(type="text", text="original content")],
            structured_content={"reference": "original-owner"},
            meta={"observation": "original"},
        )


class ConnectionTests(unittest.IsolatedAsyncioTestCase):
    async def test_describe_refusal_preserves_original_and_never_selects(self) -> None:
        class RefusingClient(FixedClient):
            async def call_tool(
                self, name: str, _arguments: object, **_options: object
            ) -> CallToolResult:
                self.calls.append(name)
                return CallToolResult(
                    content=[
                        TextContent(type="text", text='{"code":"original-refusal"}')
                    ],
                    structured_content={"code": "original-refusal"},
                    meta={"owner": "original"},
                    is_error=True,
                )

        client = RefusingClient()
        result = await runtime.invoke_on(
            cast("Client", client),
            "echo",
            {},
            {"wire_revision": "wire", "lifetime": "single", "read_only": True},
        )
        assert result["is_error"]
        assert (result["meta"]) == ({"owner": "original"})
        assert (client.calls) == (["capabilities_describe"])

    async def test_regular_file_jsonl_retains_one_connection_and_settles_eof(
        self,
    ) -> None:
        client = FixedClient()
        lifecycle = []

        @asynccontextmanager
        async def connected(_root: Path) -> AsyncGenerator[Client]:
            lifecycle.append("opened")
            try:
                yield cast("Client", client)
            finally:
                lifecycle.append("closed")

        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "requests.jsonl"
            source.write_text(
                '{"id":"first","capability":"echo","arguments":{}}\n{"id":"second","capability":"echo","arguments":{}}\n'
            )
            replies = []
            with (
                source.open() as stream,
                patch.object(runtime.sys, "stdin", stream),
                patch.object(runtime, "connection", connected),
                patch.object(runtime, "emit", replies.append),
            ):
                await runtime.session(
                    {
                        "echo": {
                            "wire_revision": "wire",
                            "lifetime": "endpoint",
                            "read_only": True,
                        }
                    }
                )
        assert (lifecycle) == (["opened", "closed"])
        assert ([row.get("id") for row in replies[1:]]) == (["first", "second"])
        assert (client.calls.count("capabilities_invoke_read")) == (2)

    async def test_one_client_selects_and_preserves_complete_result(self) -> None:
        client = FixedClient()
        result = await runtime.invoke_on(
            cast("Client", client),
            "echo",
            {},
            {"wire_revision": "wire", "lifetime": "single", "read_only": True},
        )
        assert (client.calls) == (
            [
                "capabilities_describe",
                "capabilities_select",
                "capabilities_invoke_read",
            ]
        )
        assert (result["structured_content"]) == ({"reference": "original-owner"})
        assert (result["meta"]) == ({"observation": "original"})
        content = result["content"]
        assert isinstance(content, list)
        assert content[0]["text"] == "original content"

    async def test_stale_contract_and_endpoint_lifetime_refuse_before_effects(
        self,
    ) -> None:
        client = FixedClient("changed")
        expected = {"wire_revision": "wire", "lifetime": "single", "read_only": False}
        with pytest.raises(runtime.ClientRefusedError):
            await runtime.invoke_on(cast("Client", client), "echo", {}, expected)
        assert (client.calls) == (["capabilities_describe"])
        client = FixedClient()
        expected["lifetime"] = "endpoint"
        with pytest.raises(runtime.ClientRefusedError):
            await runtime.invoke_on(cast("Client", client), "echo", {}, expected)
        assert (client.calls) == ([])
        await runtime.invoke_on(
            cast("Client", client), "echo", {}, expected, retained=True
        )
        assert (client.calls[-1]) == ("capabilities_invoke_action")


class DiagnosticProbeTests(unittest.IsolatedAsyncioTestCase):
    async def test_missing_launch_and_failed_initialization_are_independent(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with (
                patch.object(runtime, "checkout_root", return_value=root),
                patch.object(runtime, "connection") as connect,
            ):
                missing = await runtime.probe()
            assert (missing["classification"]) == ("missing_checkout_interpreter")
            assert (missing["stage"]) == ("configuration")
            connect.assert_not_called()
            interpreter = root / ".venv/bin/python"
            interpreter.parent.mkdir(parents=True)
            interpreter.touch()

            @asynccontextmanager
            async def broken(_root: Path) -> AsyncGenerator[Client]:
                raise OSError("sensitive details must not escape")
                yield  # pragma: no cover -- defines the failing context manager.

            with (
                patch.object(runtime, "checkout_root", return_value=root),
                patch.object(runtime, "connection", broken),
            ):
                failed = await runtime.probe()
            assert (failed["stage"]) == ("endpoint_initialization")
            assert (failed["connection"]) == ("unavailable")
            assert ("sensitive details") not in (json.dumps(failed))

    async def test_stale_handshake_is_not_reported_as_refreshed_attachment(
        self,
    ) -> None:
        class StaleClient(FixedClient):
            async def call_tool(
                self, name: str, _arguments: object, **_options: object
            ) -> CallToolResult:
                self.calls.append(name)
                return CallToolResult(
                    content=[],
                    structured_content={"interface": {"restart_required": True}},
                    meta=None,
                )

        client = StaleClient()

        @asynccontextmanager
        async def connected(_root: Path) -> AsyncGenerator[Client]:
            yield cast("Client", client)

        with patch.object(runtime, "connection", connected):
            report = await runtime.probe()
        assert (report["classification"]) == ("stale_loaded_interface")
        assert (client.calls) == (["capabilities_domains"])
        effect = report["effect"]
        assert isinstance(effect, str)
        assert "no setup" in effect


if __name__ == "__main__":
    unittest.main()
