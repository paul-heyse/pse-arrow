# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Canonical exposure reaches real source owners and atomic framework visibility."""
# ruff: noqa: PT009, PT027 -- stdlib host controls

from __future__ import annotations

import asyncio
import json
import sys
import unittest
from pathlib import Path
from unittest.mock import patch

from fastmcp import Client
from fastmcp.client.transports import StdioTransport
from fastmcp.exceptions import ToolError
from fastmcp.tools import Tool
from mcp import MCPError
from pydantic import ValidationError

from scripts.capabilities.catalogue import Catalogue
from scripts.capabilities.model import Capability, Domain, Guide, Workflow
from scripts.capabilities.operations import bounded
from scripts.capabilities.server import build

ROOT = Path(__file__).resolve().parents[2]
BOOTSTRAP = {"capabilities_domains", "capabilities_workflows", "capabilities_describe", "capabilities_select", "capabilities_reset", "capabilities_invoke_read"}


def fixture(*, exposure="tool", guides=("root",), workflows=("inspect",)):
    def owned_read(limit: int = 3) -> dict[str, int]:
        """Actual fixture operation, independently expected output."""
        return {"items": limit}

    tool = Tool.from_function(owned_read)
    spec = Capability(id="owned_read", title="Read", description="Read", domain="source", workflows=workflows, guides=guides, owner="owner.py", source="owner.py", exposure=exposure, effects="Read only")
    return (Domain(id="source", title="Source", description="Source owners"),), (Workflow(id="inspect", title="Inspect", description="Inspect"),), (Guide(id="root", title="Root", path="AGENTS.md"),), spec, tool


class CatalogueTests(unittest.TestCase):
    def test_single_declaration_drives_actual_schema_and_membership(self) -> None:
        domains, workflows, guides, spec, tool = fixture()
        catalogue = Catalogue(domains, workflows, guides, (spec,), {spec.id: tool})
        self.assertEqual(catalogue.browse("domain", "source", 0, 1)["total"], 1)
        described = catalogue.describe(spec.id, include_schema=True)
        self.assertEqual(described["input_schema"]["properties"]["limit"]["type"], "integer")
        self.assertEqual(catalogue.select([], ["inspect"], []).capabilities, (spec.id,))
        with self.assertRaisesRegex(ValueError, "unknown domain"):
            catalogue.browse("domain", "missing", 0, 1)
        with self.assertRaisesRegex(ValueError, "unknown capabilities"):
            catalogue.select([], [], ["missing"])

    def test_bad_composition_refuses_duplicate_and_dangling_meanings(self) -> None:
        domains, workflows, guides, spec, tool = fixture()
        for bad_specs in ((spec, spec), (spec.model_copy(update={"domain": "missing"}),), (spec.model_copy(update={"workflows": ("missing",)}),), (spec.model_copy(update={"guides": ("missing",)}),)):
            with self.subTest(specs=bad_specs), self.assertRaises(ValueError):
                Catalogue(domains, workflows, guides, bad_specs, {spec.id: tool})
        with self.assertRaisesRegex(ValueError, "binding mismatch"):
            Catalogue(domains, workflows, guides, (spec,), {})
        with self.assertRaises(ValidationError):
            Capability.model_validate({**spec.model_dump(), "unknown": "not metadata"})

    def test_native_and_deferred_are_not_fake_callables(self) -> None:
        domains, workflows, guides, spec, _ = fixture(exposure="native")
        catalogue = Catalogue(domains, workflows, guides, (spec,), {})
        self.assertFalse(catalogue.describe(spec.id, include_schema=True)["callable"])
        self.assertIsNone(catalogue.describe(spec.id, include_schema=True)["input_schema"])
        with self.assertRaisesRegex(ValueError, "reason and owner"):
            Catalogue(domains, workflows, guides, (spec.model_copy(update={"exposure": "deferred"}),), {})

    def test_bounded_nested_output_retains_explicit_loss(self) -> None:
        result = bounded({"items": [{"detail": "x" * 20000}] * 101, "complete": True}, "read")
        self.assertFalse(result.complete)
        self.assertEqual(len(result.payload["items"]), 100)
        self.assertTrue(any("string detail" in item for item in result.omissions))
        self.assertLess(len(result.model_dump_json()), 128 * 1024)


class EndpointTests(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self) -> None:
        self.runtime = build(ROOT)
        self.client = Client(self.runtime.server)
        await self.client.__aenter__()
        self.addAsyncCleanup(self.client.__aexit__, None, None, None)

    async def listed(self):
        return {tool.name for tool in await self.client.list_tools(cache_mode="refresh")}

    async def refused(self, name, arguments):
        try:
            result = await self.client.call_tool(name, arguments, raise_on_error=False)
        except (MCPError, ToolError):
            return True
        return result.is_error

    async def test_discovery_guidance_and_real_native_status_read(self) -> None:
        self.assertEqual(await self.listed(), BOOTSTRAP)
        self.assertTrue(await self.refused("workspace_task_context", {}))
        domains = (await self.client.call_tool("capabilities_domains", {})).structured_content
        self.assertTrue(any(item["id"] == "workspace" for item in domains["items"]))
        description = (await self.client.call_tool("capabilities_describe", {"capability": "workspace_task_context", "include_schema": True})).structured_content
        self.assertTrue(description["callable"])
        self.assertFalse(description["visible"])
        self.assertIn("properties", description["input_schema"])
        selected = (await self.client.call_tool("capabilities_select", {"capabilities": ["workspace_task_context"]})).structured_content
        self.assertEqual(selected["callable_tools"], ["workspace_task_context"])
        self.assertTrue(selected["instructions"])
        actual = (await self.client.call_tool("workspace_task_context", {"task": "docs/plans/52-agent-effectiveness-and-engineering-workflows.md#CAP01", "paths": ["docs/plans/52i-canonical-capabilities-and-progressive-reveal.md"]})).structured_content
        self.assertEqual(actual["payload"]["schema"], "workspace-task-v1")
        self.assertEqual([item["native_id"] for item in actual["payload"]["native_scope"]], ["CAP01"])
        self.assertFalse(any("text" in source for source in actual["payload"]["sources"]))
        self.assertNotIn("pse._native", sys.modules)
        self.assertTrue(await self.refused("workspace_task_context", {"task": 12}))
        self.assertTrue(await self.refused("workspace_task_context", {"unknown": "field"}))

    async def test_exact_compatibility_invocation_preserves_visibility_and_strict_contract(self) -> None:
        arguments = {"paths": ["python/pse/_native.pyi"], "limit": 3}
        self.assertTrue(await self.refused("capabilities_invoke_read", {"capability": "workspace_producer_relationships", "arguments": arguments}))
        await self.client.call_tool("capabilities_select", {"capabilities": ["workspace_producer_relationships"]})
        direct = (await self.client.call_tool("workspace_producer_relationships", arguments)).structured_content
        compatible = (await self.client.call_tool("capabilities_invoke_read", {"capability": "workspace_producer_relationships", "arguments": arguments})).structured_content
        self.assertEqual(compatible, direct)
        self.assertTrue(await self.refused("capabilities_invoke_read", {"capability": "workspace_producer_relationships", "arguments": {"paths": ["python/pse/_native.pyi"], "limit": "3"}}))
        self.assertTrue(await self.refused("capabilities_invoke_read", {"capability": "capabilities_reset", "arguments": {}}))
        self.assertTrue(await self.refused("capabilities_invoke_read", {"capability": "sh", "arguments": {"command": "anything"}}))
        await self.client.call_tool("capabilities_reset", {})
        self.assertTrue(await self.refused("capabilities_invoke_read", {"capability": "workspace_producer_relationships", "arguments": arguments}))

    async def test_presentation_controls_are_read_only_to_the_external_world(self) -> None:
        tools = {tool.name: tool for tool in await self.client.list_tools(cache_mode="refresh")}
        for name in ("capabilities_select", "capabilities_reset"):
            self.assertTrue(tools[name].annotations.read_only_hint)
            self.assertFalse(tools[name].annotations.destructive_hint)
            self.assertFalse(tools[name].annotations.open_world_hint)

    async def test_unfamiliar_native_tool_has_real_owner_description_not_fake_handler(self) -> None:
        integration = (await self.client.call_tool("capabilities_domains", {"domain": "integration"})).structured_content
        self.assertIn("git-integration", {item["id"] for item in integration["items"]})
        described = (await self.client.call_tool("capabilities_describe", {"capability": "git-integration", "include_schema": True})).structured_content
        self.assertFalse(described["callable"])
        self.assertIsNone(described["input_schema"])
        await self.client.call_tool("capabilities_select", {"capabilities": ["workspace_recipe_describe"]})
        actual = (await self.client.call_tool("workspace_recipe_describe", {"recipe": "labels-sync"})).structured_content
        self.assertEqual(actual["payload"]["definition_route"], ["just", "--show", "labels-sync"])
        self.assertFalse(actual["payload"]["leaf_schema_complete"])
        self.assertTrue(actual["payload"]["leaf_contracts"])

    async def test_selection_replaces_resets_and_unknown_is_atomic(self) -> None:
        await self.client.call_tool("capabilities_select", {"capabilities": ["workspace_task_context"]})
        again = (await self.client.call_tool("capabilities_select", {"capabilities": ["workspace_task_context"]})).structured_content
        self.assertFalse(again["changed"])
        self.assertTrue(await self.refused("capabilities_select", {"capabilities": ["workspace_concepts_query", "unknown"]}))
        self.assertEqual(await self.listed(), BOOTSTRAP | {"workspace_task_context"})
        await self.client.call_tool("capabilities_select", {"capabilities": ["workspace_producer_relationships"]})
        self.assertEqual(await self.listed(), BOOTSTRAP | {"workspace_producer_relationships"})
        await self.client.call_tool("capabilities_reset", {})
        self.assertEqual(await self.listed(), BOOTSTRAP)
        self.assertTrue(await self.refused("workspace_producer_relationships", {"paths": ["python/pse/_native.pyi"]}))

    async def test_concurrent_selections_never_accumulate_union(self) -> None:
        await asyncio.gather(self.client.call_tool("capabilities_select", {"capabilities": ["workspace_task_context"]}), self.client.call_tool("capabilities_select", {"capabilities": ["workspace_concepts_query"]}))
        self.assertIn(await self.listed(), (BOOTSTRAP | {"workspace_task_context"}, BOOTSTRAP | {"workspace_concepts_query"}))
        await asyncio.gather(self.client.call_tool("capabilities_reset", {}), self.client.call_tool("capabilities_select", {"capabilities": ["workspace_task_context"]}))
        self.assertIn(await self.listed(), (BOOTSTRAP, BOOTSTRAP | {"workspace_task_context"}))

    async def test_notifications_do_not_claim_rollback_after_commit(self) -> None:
        from scripts.capabilities import server

        original = server.save_visibility_rules

        async def notify_failure(ctx, rules):
            await original(ctx, rules)
            raise RuntimeError("disconnected observer")

        with patch.object(server, "save_visibility_rules", side_effect=notify_failure):
            selected = (await self.client.call_tool("capabilities_select", {"capabilities": ["workspace_task_context"]})).structured_content
        self.assertTrue(selected["changed"])
        self.assertTrue(selected["notification_omissions"])
        self.assertEqual(await self.listed(), BOOTSTRAP | {"workspace_task_context"})

    async def test_source_only_startup_does_not_inspect_inventory_or_call_work(self) -> None:
        with patch("subprocess.run", side_effect=AssertionError("startup must not run a command")):
            runtime = build(ROOT)
        self.assertNotIn("pse._native", sys.modules)
        self.assertEqual(set(runtime.catalogue.tools), {"workspace_task_context", "workspace_concepts_query", "workspace_producer_relationships", "workspace_recipe_describe", "workspace_inventory"})


class StdioTests(unittest.IsolatedAsyncioTestCase):
    async def test_two_real_processes_reveal_and_teardown_independently(self) -> None:
        transports = [StdioTransport(command=sys.executable, args=["-m", "scripts.capabilities"], cwd=str(ROOT), keep_alive=False) for _ in range(2)]
        try:
            async with Client(transports[0], timeout=20) as first, Client(transports[1], timeout=20) as second:
                self.assertEqual({item.name for item in await first.list_tools(cache_mode="refresh")}, BOOTSTRAP)
                await first.call_tool("capabilities_select", {"capabilities": ["workspace_producer_relationships"]})
                self.assertIn("workspace_producer_relationships", {item.name for item in await first.list_tools(cache_mode="refresh")})
                self.assertEqual({item.name for item in await second.list_tools(cache_mode="refresh")}, BOOTSTRAP)
                actual = (await first.call_tool("workspace_producer_relationships", {"paths": ["python/pse/_native.pyi"], "limit": 3})).structured_content
                self.assertEqual(actual["operation"], "workspace_producer_relationships")
                self.assertIsInstance(actual["payload"], dict)
                await first.call_tool("capabilities_reset", {})
                self.assertEqual({item.name for item in await first.list_tools(cache_mode="refresh")}, BOOTSTRAP)
        finally:
            for transport in transports:
                await transport.close()


if __name__ == "__main__":
    unittest.main()
