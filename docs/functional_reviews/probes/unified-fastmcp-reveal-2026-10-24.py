# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Synthetic reveal/guide/schema probe; no product imports or production effects.

Two independently owned STDIO server processes model two agent connections. This
does not test whether the Codex host refreshes its own callable-tool inventory.
"""

from __future__ import annotations

import asyncio
import importlib.metadata
import json
import logging
import sys
from pathlib import Path
from typing import Annotated

from fastmcp import Client, FastMCP
from fastmcp.cli.generate import generate_cli_script
from fastmcp.client.transports import StdioTransport
from fastmcp.exceptions import ToolError
from fastmcp.server.context import Context
from mcp import MCPError
from pydantic import BaseModel, Field


class Reply(BaseModel):
    value: int


class Exposure(BaseModel):
    family: str
    guide: str
    refresh_tools: bool


def server() -> FastMCP:
    instance = FastMCP(
        "synthetic-unified-review",
        strict_input_validation=True,
        mask_error_details=True,
    )

    @instance.tool(tags={"family:source"})
    async def source_inspect(limit: Annotated[int, Field(gt=0, le=10)]) -> Reply:
        """Read synthetic source detail; changes no source files."""
        return Reply(value=limit)

    @instance.resource("guide://source", tags={"family:source"})
    def source_guide() -> str:
        return "Source inspection is read-only. Select a positive limit up to ten."

    @instance.tool
    async def open_source(ctx: Context) -> Exposure:
        await ctx.enable_components(tags={"family:source"})
        return Exposure(family="source", guide="guide://source", refresh_tools=True)

    @instance.tool
    async def reset(ctx: Context) -> Exposure:
        await ctx.reset_visibility()
        return Exposure(family="root", guide="", refresh_tools=True)

    instance.disable(tags={"family:source"})
    return instance


async def refused(client: Client, name: str, arguments: dict) -> bool:
    try:
        result = await client.call_tool(name, arguments, raise_on_error=False)
    except (MCPError, ToolError):
        return True
    return result.is_error


async def probe() -> None:
    logging.disable(logging.CRITICAL)
    transports = [
        StdioTransport(
            command=sys.executable,
            args=[str(Path(__file__).resolve()), "--serve"],
            cwd=str(Path(__file__).resolve().parent),
            keep_alive=False,
        )
        for _ in range(2)
    ]
    try:
        async with (
            Client(transports[0], timeout=15, init_timeout=15) as first,
            Client(transports[1], timeout=15, init_timeout=15) as second,
        ):
            initial = await first.list_tools(cache_mode="refresh")
            assert {item.name for item in initial} == {"open_source", "reset"}
            assert await refused(first, "source_inspect", {"limit": 2})
            opened = await first.call_tool("open_source", {})
            assert opened.structured_content["guide"] == "guide://source"
            selected = await first.list_tools(cache_mode="refresh")
            assert {item.name for item in selected} == {
                "open_source",
                "reset",
                "source_inspect",
            }
            independent = await second.list_tools(cache_mode="refresh")
            assert {item.name for item in independent} == {"open_source", "reset"}
            resources = (await first.list_resources_mcp(cache_mode="refresh")).resources
            assert any(str(item.uri) == "guide://source" for item in resources)
            guide = await first.read_resource("guide://source")
            assert any("read-only" in item.text for item in guide)
            result = await first.call_tool("source_inspect", {"limit": 2})
            assert result.structured_content == {"value": 2}
            assert await refused(first, "source_inspect", {"limit": "2"})
            initial_cli = generate_cli_script(
                "probe", "probe.py", repr("probe.py"), set(), initial
            )
            selected_cli = generate_cli_script(
                "probe", "probe.py", repr("probe.py"), set(), selected
            )
            assert "async def source_inspect(" not in initial_cli
            assert "async def source_inspect(" in selected_cli
            await first.call_tool("reset", {})
            reset_tools = await first.list_tools(cache_mode="refresh")
            assert {item.name for item in reset_tools} == {"open_source", "reset"}
            assert await refused(first, "source_inspect", {"limit": 2})
            print(
                json.dumps(
                    {
                        "versions": {
                            name: importlib.metadata.version(name)
                            for name in ("fastmcp", "mcp", "pydantic")
                        },
                        "protocols": [first.protocol_version, second.protocol_version],
                        "initial": sorted(item.name for item in initial),
                        "revealed": sorted(item.name for item in selected),
                        "other_process": sorted(item.name for item in independent),
                        "explicit_refresh": True,
                        "guide_read": True,
                        "hidden_call_refused": True,
                        "strict_input_refused": True,
                        "generator_uses_visible_catalogue": True,
                        "reset": sorted(item.name for item in reset_tools),
                    },
                    sort_keys=True,
                )
            )
    finally:
        for transport in transports:
            await transport.close()


if __name__ == "__main__":
    if sys.argv[1:] == ["--serve"]:
        server().run(show_banner=False)
    elif sys.argv[1:]:
        raise SystemExit("usage: probe [--serve]")
    else:
        asyncio.run(probe())
