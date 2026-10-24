# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Content projections retain exact owners, visibility and effect boundaries."""
# ruff: noqa: PT009, PT027 -- stdlib host controls

from __future__ import annotations

import json
import tempfile
import unittest
from hashlib import sha256
from pathlib import Path
from typing import TYPE_CHECKING
from unittest.mock import patch

from fastmcp import Client, FastMCP
from fastmcp.server.context import Context  # noqa: TC002 -- evaluated by FastMCP
from fastmcp.server.middleware import Middleware
from fastmcp.server.transforms.visibility import save_visibility_rules
from fastmcp.tools import Tool
from mcp import MCPError
from mcp_types import TextContent, TextResourceContents

from scripts.capabilities import presentation
from scripts.capabilities.catalogue import Catalogue
from scripts.capabilities.guidance import Guidance, register_guides
from scripts.capabilities.model import Capability, Domain, Guide, Workflow, WorkflowStep

if TYPE_CHECKING:
    from fastmcp.client.client import CallToolResult
    from fastmcp.prompts import PromptResult
    from fastmcp.resources import ResourceResult
    from fastmcp.server.middleware import CallNext, MiddlewareContext
    from mcp_types import GetPromptRequestParams, ReadResourceRequestParams

ROOT = Path(__file__).resolve().parents[2]
PROJECTIONS = {"list_resources", "read_resource", "list_prompts", "get_prompt"}


def projected_text(result: CallToolResult) -> str:
    content = result.content[0]
    if not isinstance(content, TextContent):
        raise TypeError("Expected a text projection")
    return content.text


class ObservedContent(Middleware):
    def __init__(self) -> None:
        self.reads: list[str] = []
        self.prompts: list[str] = []

    async def on_read_resource(
        self,
        context: MiddlewareContext[ReadResourceRequestParams],
        call_next: CallNext[ReadResourceRequestParams, ResourceResult],
    ) -> ResourceResult:
        self.reads.append(context.message.uri)
        return await call_next(context)

    async def on_get_prompt(
        self,
        context: MiddlewareContext[GetPromptRequestParams],
        call_next: CallNext[GetPromptRequestParams, PromptResult],
    ) -> PromptResult:
        self.prompts.append(context.message.name)
        return await call_next(context)


class PresentationTests(unittest.IsolatedAsyncioTestCase):
    async def test_content_projections_are_truthfully_read_only(self) -> None:
        async with Client(self.server) as client:
            tools = await client.list_tools()
        projections = [tool for tool in tools if tool.name in PROJECTIONS]
        self.assertEqual({tool.name for tool in projections}, PROJECTIONS)
        for tool in projections:
            self.assertIsNotNone(tool.annotations)
            assert tool.annotations is not None
            self.assertTrue(tool.annotations.read_only_hint)
            self.assertFalse(tool.annotations.destructive_hint)

    async def asyncSetUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        (self.root / "AGENTS.md").write_text("# Root instructions\n")
        skill_path = ".codex/skills/plan-execution/SKILL.md"
        (self.root / skill_path).parent.mkdir(parents=True)
        self.skill = (ROOT / skill_path).read_text()
        (self.root / skill_path).write_text(self.skill)
        self.guides = (
            Guide(id="root", title="Root", path="AGENTS.md"),
            Guide(id="planning", title="Plan execution", path=skill_path, kind="skill"),
        )
        self.calls = 0

        def owned_read(limit: int = 3) -> dict[str, int]:
            """The canonical fixture read."""
            self.calls += 1
            return {"items": limit}

        self.tool = Tool.from_function(owned_read)
        self.capability = Capability(
            id="owned_read",
            title="Owned read",
            description="Inspect the owned source",
            domain="source",
            workflows=("inspect",),
            guides=("planning",),
            owner="owner.py",
            source="owner.py",
            exposure="tool",
            effects="Read source only",
        )
        self.workflow = Workflow(
            id="inspect",
            title="Inspect a plan",
            description="Plan the investigation without authorizing implementation.",
            guides=("planning",),
            steps=(WorkflowStep(id="source", capabilities=("owned_read",), purpose="Inspect the selected source"),),
        )
        self.catalogue = Catalogue(
            (Domain(id="source", title="Source", description="Source reads"),),
            (self.workflow,),
            self.guides,
            (self.capability,),
            {self.capability.id: self.tool},
        )
        self.guidance = Guidance(self.root, self.guides)
        self.server = FastMCP("presentation-test", strict_input_validation=True)
        self.observed = ObservedContent()
        self.server.add_middleware(self.observed)
        self.guide_keys = register_guides(self.server, self.guidance)
        self.server.disable(keys={value for key, value in self.guide_keys.items() if key.split("/", 1)[0] == "planning"})
        self.server.add_tool(self.tool)
        self.server.disable(keys={self.tool.key})
        with patch.object(
            self.guidance,
            "instruction_bundle",
            side_effect=AssertionError("registration must not read instructions"),
        ):
            self.presentation = presentation.register(
                self.server, self.catalogue, self.guidance
            )

        @self.server.tool
        async def select(ctx: Context, keys: list[str]) -> None:
            await save_visibility_rules(ctx, [{"enabled": True, "keys": keys}])

        self.client = Client(self.server)
        await self.client.__aenter__()
        self.addAsyncCleanup(self.client.__aexit__, None, None, None)

    async def select(self, *, exclude: set[str] | None = None) -> None:
        keys = {
            self.tool.key,
            *self.guide_keys.values(),
            self.presentation.workflow_keys["inspect"],
        } - (exclude or set())
        await self.client.call_tool("select", {"keys": sorted(keys)})

    async def test_bootstrap_lists_small_projections_and_lazy_content_template(
        self,
    ) -> None:
        with patch.object(
            self.guidance,
            "instruction_bundle",
            side_effect=AssertionError("listing must not read instructions"),
        ):
            tools = await self.client.list_tools()
            resources = json.loads(
                projected_text(await self.client.call_tool("list_resources", {}))
            )
            prompts = json.loads(
                projected_text(await self.client.call_tool("list_prompts", {}))
            )
        self.assertEqual({tool.name for tool in tools}, PROJECTIONS | {"select"})
        self.assertLess(sum(len(tool.model_dump_json()) for tool in tools), 10 * 1024)
        self.assertEqual(prompts, [])
        self.assertEqual(
            {item.get("uri") or item.get("uri_template") for item in resources},
            {"pse://guidance/root", "pse://capabilities/{capability}"},
        )
        self.assertEqual(self.calls, 0)

    async def test_exact_template_and_projection_share_loaded_description_schema(
        self,
    ) -> None:
        uri = "pse://capabilities/owned_read"
        native = (await self.client.read_resource(uri))[0]
        self.assertIsInstance(native, TextResourceContents)
        if not isinstance(native, TextResourceContents):
            self.fail("Expected JSON text")
        projected = await self.client.call_tool("read_resource", {"uri": uri})
        expected = self.catalogue.describe("owned_read", include_schema=True)
        self.assertEqual(json.loads(native.text), expected)
        self.assertEqual(json.loads(projected_text(projected)), expected)
        self.assertEqual(expected["input_schema"], self.tool.parameters)
        self.assertEqual(self.observed.reads, [uri, uri])
        self.assertEqual(self.calls, 0)
        for unknown in ("missing", "owned-read", "owned_read/extra"):
            with self.subTest(identity=unknown), self.assertRaises(MCPError):
                await self.client.read_resource("pse://capabilities/" + unknown)

    async def test_resource_and_prompt_refusals_keep_codes_without_raw_exception_text(
        self,
    ) -> None:
        with (
            patch.object(
                self.catalogue,
                "describe",
                side_effect=ValueError("PRIVATE SOURCE password=value"),
            ),
            self.assertRaises(MCPError) as failure,
        ):
            await self.client.read_resource("pse://capabilities/owned_read")
        self.assertIn("pse.capabilities.unknown_capability", str(failure.exception))
        self.assertNotIn("PRIVATE SOURCE", str(failure.exception))
        self.assertNotIn("password", str(failure.exception))
        await self.select()
        with (
            patch.object(
                self.guidance,
                "instruction_bundle",
                side_effect=ValueError("PRIVATE SOURCE password=value"),
            ),
            self.assertRaises(MCPError) as failure,
        ):
            await self.client.get_prompt("workflow_inspect")
        self.assertIn("pse.capabilities.guidance_unavailable", str(failure.exception))
        self.assertNotIn("PRIVATE SOURCE", str(failure.exception))
        self.assertNotIn("password", str(failure.exception))
        self.assertEqual(self.calls, 0)

    async def test_native_guide_and_projected_guide_keep_identical_owned_content(
        self,
    ) -> None:
        await self.select()
        uri = "pse://guidance/planning"
        native = (await self.client.read_resource(uri))[0]
        if not isinstance(native, TextResourceContents):
            self.fail("Expected owned Markdown in JSON text")
        projected = await self.client.call_tool("read_resource", {"uri": uri})
        self.assertEqual(json.loads(projected_text(projected)), json.loads(native.text))
        self.assertEqual(json.loads(native.text)["content"], self.skill)
        self.assertEqual(self.observed.reads, [uri, uri])

    async def test_hidden_guides_and_prompts_refuse_direct_and_projected_retrieval(
        self,
    ) -> None:
        with patch.object(
            self.guidance,
            "instruction_bundle",
            side_effect=AssertionError("hidden retrieval must not read bodies"),
        ):
            with self.assertRaises(MCPError):
                await self.client.read_resource("pse://guidance/planning")
            with self.assertRaises(MCPError):
                await self.client.get_prompt("workflow_inspect")
            for tool, arguments in (
                ("read_resource", {"uri": "pse://guidance/planning"}),
                ("get_prompt", {"name": "workflow_inspect"}),
            ):
                result = await self.client.call_tool(
                    tool, arguments, raise_on_error=False
                )
                self.assertTrue(result.is_error)

    async def test_prompt_checks_all_related_visibility_before_reading_guides(
        self,
    ) -> None:
        for missing in (self.tool.key, self.guide_keys["planning"]):
            await self.select(exclude={missing})
            with (
                self.subTest(missing=missing),
                patch.object(
                    self.guidance,
                    "instruction_bundle",
                    side_effect=AssertionError("must check all visibility first"),
                ),
            ):
                with self.assertRaisesRegex(MCPError, "workflow_hidden"):
                    await self.client.get_prompt("workflow_inspect")
                result = await self.client.call_tool(
                    "get_prompt", {"name": "workflow_inspect"}, raise_on_error=False
                )
                self.assertTrue(result.is_error)

    async def test_selected_prompt_is_exact_pure_and_identical_through_projection(
        self,
    ) -> None:
        await self.select()
        before = {tool.name for tool in await self.client.list_tools()}
        with (
            patch(
                "subprocess.run", side_effect=AssertionError("must not run commands")
            ),
            patch(
                "subprocess.Popen",
                side_effect=AssertionError("must not start services"),
            ),
            patch("os.system", side_effect=AssertionError("must not execute skills")),
        ):
            native = await self.client.get_prompt("workflow_inspect")
            projected = await self.client.call_tool(
                "get_prompt", {"name": "workflow_inspect"}
            )
        message = native.messages[0]
        if not isinstance(message.content, TextContent):
            self.fail("Expected generated prompt JSON text")
        payload = json.loads(message.content.text)
        self.assertEqual(payload["revision"], self.catalogue.revision)
        self.assertEqual(payload["workflow"], self.workflow.model_dump(mode="json"))
        self.assertEqual(
            payload["capabilities"],
            [
                {
                    "id": "owned_read",
                    "title": self.capability.title,
                    "exposure": "tool",
                    "effects": self.capability.effects,
                    "description_uri": "pse://capabilities/owned_read",
                }
            ],
        )
        instructions = {item["id"]: item for item in payload["instruction_resources"]}
        self.assertFalse(payload["instructions_embedded"])
        self.assertEqual(set(instructions), {"root", "planning"})
        self.assertTrue(all("content" not in item for item in instructions.values()))
        self.assertEqual(
            instructions["planning"]["digest"], sha256(self.skill.encode()).hexdigest()
        )
        self.assertEqual(instructions["planning"]["uri"], "pse://guidance/planning")
        self.assertEqual(self.guidance.read("planning")["content"], self.skill)
        self.assertLess(len(message.content.text.encode()), len(self.skill.encode()))
        self.assertEqual(
            json.loads(projected_text(projected)),
            {"messages": [{"role": message.role, "content": message.content.text}]},
        )
        self.assertEqual(
            self.observed.prompts, ["workflow_inspect", "workflow_inspect"]
        )
        self.assertEqual(before, {tool.name for tool in await self.client.list_tools()})
        self.assertEqual(self.calls, 0)
        self.assertLess(
            len(message.content.text.encode()), presentation.MAX_PROMPT_BYTES
        )

    async def test_prompt_bound_refuses_instead_of_truncating_owned_instructions(
        self,
    ) -> None:
        await self.select()
        with (
            patch.object(presentation, "MAX_PROMPT_BYTES", 100),
            self.assertRaisesRegex(MCPError, "prompt_too_large"),
        ):
            await self.client.get_prompt("workflow_inspect")
        self.assertEqual(self.guidance.read("planning")["content"], self.skill)


if __name__ == "__main__":
    unittest.main()
