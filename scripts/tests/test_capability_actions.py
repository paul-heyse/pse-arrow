# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Exact action dispatch retains the registered contract, visibility and effects."""

# ruff: noqa: PT009, PT027 -- stdlib endpoint controls
from __future__ import annotations

import unittest
from pathlib import Path
from unittest.mock import patch

from fastmcp import Client
from fastmcp.exceptions import ToolError
from fastmcp.tools import Tool

from scripts.capabilities import server
from scripts.capabilities.model import Capability, Contract, Domain, Guide

ROOT = Path(__file__).resolve().parents[2]


class Presence(Contract):
    limit: int | None = None
    labels: list[str] = []


class ActionTests(unittest.IsolatedAsyncioTestCase):
    async def test_action_reaches_only_visible_receiver_without_normalizing_presence(self) -> None:
        calls = []

        def edit(request: Presence) -> dict:
            """Actual independent action fixture."""
            value = request.model_dump(exclude_unset=True)
            calls.append(value)
            return {"received": value, "owner": "native-fixture"}

        tool = Tool.from_function(edit, annotations={"readOnlyHint": False, "destructiveHint": True})
        spec = Capability(id="edit", title="Edit", description="Edit", domain="editing", guides=("root",),
            owner="fixture", source="fixture", exposure="tool", effects="Writes the selected fixture")
        index = ((Domain(id="editing",title="Editing",description="Editing"),), (),
            (Guide(id="root",title="Root",path="AGENTS.md"),), ())
        with patch.object(server,"inventory",return_value=index), patch.object(server,"bindings",return_value=((spec,),{"edit":tool})):
            runtime = server.build(ROOT)
        async with Client(runtime.server) as client:
            bootstrap = {entry.name: entry for entry in await client.list_tools()}
            self.assertFalse(bootstrap["capabilities_invoke_action"].annotations.read_only_hint)
            with self.assertRaises(ToolError):
                await client.call_tool("capabilities_invoke_action",{"capability":"edit","arguments":{"request":{}}})
            self.assertEqual(calls, [])
            await client.call_tool("capabilities_select",{"capabilities":["edit"]})
            for request in ({}, {"limit":None}, {"labels":[]}):
                result = await client.call_tool("capabilities_invoke_action",{"capability":"edit","arguments":{"request":request}})
                self.assertEqual(result.structured_content,{"received":request,"owner":"native-fixture"})
            self.assertEqual(calls,[{}, {"limit":None}, {"labels":[]}])
            for request in ({"limit":"3"},{"unexpected":True}):
                with self.assertRaises(ToolError):
                    await client.call_tool("capabilities_invoke_action",{"capability":"edit","arguments":{"request":request}})
            with self.assertRaises(ToolError):
                await client.call_tool("capabilities_invoke_read",{"capability":"edit","arguments":{"request":{}}})
            with self.assertRaises(ToolError):
                await client.call_tool("capabilities_invoke_action",{"capability":"capabilities_reset","arguments":{}})
            self.assertEqual(len(calls),3)
            await client.call_tool("capabilities_reset",{})
            with self.assertRaises(ToolError):
                await client.call_tool("capabilities_invoke_action",{"capability":"edit","arguments":{"request":{}}})
            self.assertEqual(len(calls),3)


if __name__ == "__main__":
    unittest.main()
