# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Canonical exposure reaches real source owners and atomic framework visibility."""
# ruff: noqa: PT009, PT027 -- stdlib host controls

from __future__ import annotations

import asyncio
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path
from typing import TYPE_CHECKING, Literal
from unittest.mock import patch

from fastmcp import Client
from fastmcp.client.transports import StdioTransport
from fastmcp.exceptions import ToolError
from fastmcp.tools import Tool, ToolResult
from jsonschema import Draft202012Validator
from mcp import MCPError
from mcp.types import ResourceLink, TextResourceContents
from pydantic import ValidationError

from scripts.capabilities import server
from scripts.capabilities.catalogue import Catalogue
from scripts.capabilities.model import Capability, Domain, Guide, Workflow, WorkflowStep
from scripts.capabilities.presentation import SelectionReceipt
from scripts.capabilities.responses import bounded
from scripts.capabilities.server import build

if TYPE_CHECKING:
    from fastmcp.client.client import CallToolResult
    from fastmcp.server.context import Context

ROOT = Path(__file__).resolve().parents[2]
BOOTSTRAP = {
    "capabilities_domains",
    "capabilities_workflows",
    "capabilities_describe",
    "capabilities_select",
    "capabilities_reset",
    "capabilities_invoke_read",
    "capabilities_invoke_action",
    "list_resources",
    "read_resource",
    "list_prompts",
    "get_prompt",
}


def object_value(value: object) -> dict:
    if not isinstance(value, dict):
        raise TypeError("Expected an object, got " + type(value).__name__)
    return value


def content(result: CallToolResult | ToolResult) -> dict:
    return object_value(result.structured_content)


def host_content(result: CallToolResult | ToolResult) -> dict:
    """Typed host-return unions have one framework-owned object wrapper."""
    structured = content(result)
    if set(structured) != {"result"}:
        raise TypeError("Expected the published host answer wrapper")
    return object_value(structured["result"])


def fixture(
    *,
    exposure: Literal["tool", "native", "deferred", "internal", "unsupported"] = "tool",
    guides: tuple[str, ...] = ("root",),
    workflows: tuple[str, ...] = ("inspect",),
) -> tuple[
    tuple[Domain, ...], tuple[Workflow, ...], tuple[Guide, ...], Capability, Tool
]:
    def owned_read(limit: int = 3) -> dict[str, int]:
        """Actual fixture operation, independently expected output."""
        return {"items": limit}

    tool = Tool.from_function(owned_read)
    spec = Capability(
        id="owned_read",
        title="Read",
        description="Read",
        domain="source",
        workflows=workflows,
        guides=guides,
        owner="owner.py",
        source="owner.py",
        exposure=exposure,
        effects="Read only",
    )
    return (
        (Domain(id="source", title="Source", description="Source owners"),),
        (
            Workflow(
                id="inspect",
                title="Inspect",
                description="Inspect",
                steps=(
                    WorkflowStep(
                        id="read",
                        capabilities=("owned_read",),
                        purpose="Inspect source",
                    ),
                ),
            ),
        ),
        (Guide(id="root", title="Root", path="AGENTS.md"),),
        spec,
        tool,
    )


class CatalogueTests(unittest.TestCase):
    def test_family_membership_comes_from_real_bindings_and_selects_the_same_operations(
        self,
    ) -> None:
        domains, workflows, guides, spec, tool = fixture()
        family = Capability(
            id="source-family",
            title="Source family",
            description="Retained owner",
            domain="source",
            owner="owner.py",
            source="owner.py",
            exposure="native",
            effects="Native owner",
            route=("native-owner",),
        )
        child = Capability.model_validate({**spec.model_dump(), "family": family.id})
        catalogue = Catalogue(
            domains, workflows, guides, (family, child), {child.id: tool}
        )
        self.assertEqual(catalogue.describe(family.id)["operations"], [child.id])
        self.assertEqual(
            set(catalogue.select([], [], [family.id]).capabilities),
            {family.id, child.id},
        )
        bad = Capability.model_validate(
            {**spec.model_dump(), "family": "unknown-family"}
        )
        with self.assertRaisesRegex(ValueError, "invalid capability family"):
            Catalogue(domains, workflows, guides, (bad,), {bad.id: tool})

    def test_single_declaration_drives_actual_schema_and_membership(self) -> None:
        domains, workflows, guides, spec, tool = fixture()
        catalogue = Catalogue(domains, workflows, guides, (spec,), {spec.id: tool})
        self.assertEqual(catalogue.browse("domain", "source", 0, 1)["total"], 1)
        described = catalogue.describe(spec.id, include_schema=True)
        schema = object_value(described["input_schema"])
        self.assertEqual(
            object_value(object_value(schema["properties"])["limit"])["type"], "integer"
        )
        self.assertEqual(catalogue.select([], ["inspect"], []).capabilities, (spec.id,))
        with self.assertRaisesRegex(ValueError, "unknown domain"):
            catalogue.browse("domain", "missing", 0, 1)
        with self.assertRaisesRegex(ValueError, "unknown capabilities"):
            catalogue.select([], [], ["missing"])

    def test_bad_composition_refuses_duplicate_and_dangling_meanings(self) -> None:
        domains, workflows, guides, spec, tool = fixture()
        for bad_specs in (
            (spec, spec),
            (spec.model_copy(update={"domain": "missing"}),),
            (spec.model_copy(update={"guides": ("missing",)}),),
        ):
            with self.subTest(specs=bad_specs), self.assertRaises(ValueError):
                Catalogue(domains, workflows, guides, bad_specs, {spec.id: tool})
        with self.assertRaisesRegex(ValueError, "binding mismatch"):
            Catalogue(domains, workflows, guides, (spec,), {})
        with self.assertRaises(ValidationError):
            Capability.model_validate({**spec.model_dump(), "unknown": "not metadata"})

    def test_workflow_steps_are_the_single_membership_and_tag_source(self) -> None:
        domains, workflows, guides, spec, tool = fixture()
        authored = spec.model_copy(update={"workflows": ()})
        catalogue = Catalogue(domains, workflows, guides, (authored,), {spec.id: tool})
        self.assertEqual(catalogue.capabilities[spec.id].workflows, ("inspect",))
        self.assertIn("workflow:inspect", tool.tags)
        self.assertEqual(catalogue.browse("workflow", "inspect", 0, 10)["total"], 1)
        for steps in (
            (
                WorkflowStep(
                    id="missing", capabilities=("unknown",), purpose="Missing owner"
                ),
            ),
            (
                WorkflowStep(id="same", capabilities=(spec.id,), purpose="First"),
                WorkflowStep(id="same", capabilities=(spec.id,), purpose="Repeated"),
            ),
            (
                WorkflowStep(
                    id="duplicate",
                    capabilities=(spec.id, spec.id),
                    purpose="Repeated capability",
                ),
            ),
            (
                WorkflowStep(
                    id="guide",
                    capabilities=(spec.id,),
                    purpose="Unknown guide",
                    guides=("unknown",),
                ),
            ),
        ):
            with self.subTest(steps=steps), self.assertRaises(ValueError):
                Catalogue(
                    domains,
                    (workflows[0].model_copy(update={"steps": steps}),),
                    guides,
                    (spec,),
                    {spec.id: tool},
                )

    def test_native_and_deferred_are_not_fake_callables(self) -> None:
        domains, workflows, guides, spec, _ = fixture(exposure="native")
        catalogue = Catalogue(domains, workflows, guides, (spec,), {})
        self.assertFalse(catalogue.describe(spec.id, include_schema=True)["callable"])
        self.assertIsNone(
            catalogue.describe(spec.id, include_schema=True)["input_schema"]
        )
        with self.assertRaisesRegex(ValueError, "reason and owner"):
            Catalogue(
                domains,
                workflows,
                guides,
                (spec.model_copy(update={"exposure": "deferred"}),),
                {},
            )

    def test_bounded_nested_output_retains_explicit_loss(self) -> None:
        result = bounded(
            {"items": [{"detail": "x" * 20000}] * 101, "complete": True}, "read"
        )
        self.assertFalse(result.complete)
        items = result.payload["items"]
        if not isinstance(items, list):
            self.fail("Expected an item array")
        self.assertEqual(len(items), 100)
        self.assertTrue(any("string detail" in item for item in result.omissions))
        self.assertLess(len(result.model_dump_json()), 128 * 1024)

    def test_encoded_bound_includes_owner_omissions_keys_and_envelope(self) -> None:
        for value in (
            {"omissions": ["x" * 200000]},
            {"x" * 200000: 1},
            {"omissions": ["\U0001f680" * 2000] * 100},
            {str(index): {"nested": "x" * 16000} for index in range(100)},
        ):
            with self.subTest(value_size=len(str(value))):
                result = bounded(value, "read")
                self.assertFalse(result.complete)
                self.assertTrue(result.omissions)
                self.assertLessEqual(
                    len(json.dumps(result.model_dump(mode="json")).encode()), 128 * 1024
                )
        for value in (float("nan"), float("inf"), -float("inf")):
            with (
                self.subTest(value=value),
                self.assertRaisesRegex(ValueError, "nonfinite"),
            ):
                bounded({"number": value}, "read")

    def test_original_truncation_and_nested_gaps_cannot_be_promoted_to_complete(
        self,
    ) -> None:
        for payload in (
            {"items": [], "truncated": True, "gaps": []},
            {
                "relationships": {
                    "items": [],
                    "truncated": False,
                    "gaps": ["missing producer"],
                }
            },
            {"nested": {"complete": False}},
        ):
            with self.subTest(payload=payload):
                result = bounded(payload, "read")
                self.assertFalse(result.complete)
                self.assertTrue(result.omissions)
                self.assertEqual(result.payload, payload)


class EndpointTests(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self) -> None:
        self.runtime = build(ROOT)
        self.client = Client(self.runtime.server)
        await self.client.__aenter__()
        self.addAsyncCleanup(self.client.__aexit__, None, None, None)

    async def listed(self) -> set[str]:
        return {
            tool.name for tool in await self.client.list_tools(cache_mode="refresh")
        }

    async def refused(self, name: str, arguments: dict) -> bool:
        try:
            result = await self.client.call_tool(name, arguments, raise_on_error=False)
        except (MCPError, ToolError):
            return True
        return result.is_error

    async def test_discovery_guidance_and_real_native_status_read(self) -> None:
        self.assertEqual(await self.listed(), BOOTSTRAP)
        self.assertTrue(await self.refused("workspace_task_context", {}))
        domains = content(await self.client.call_tool("capabilities_domains", {}))
        self.assertTrue(any(item["id"] == "workspace" for item in domains["items"]))
        description = content(
            await self.client.call_tool(
                "capabilities_describe",
                {"capability": "workspace_task_context", "include_schema": True},
            )
        )
        self.assertTrue(description["callable"])
        self.assertFalse(description["visible"])
        self.assertIn("properties", description["input_schema"])
        selected = content(
            await self.client.call_tool(
                "capabilities_select", {"capabilities": ["workspace_task_context"]}
            )
        )
        self.assertEqual(selected["callable_tools"], ["workspace_task_context"])
        self.assertTrue(selected["instruction_resources"])
        actual = host_content(
            await self.client.call_tool(
                "workspace_task_context",
                {
                    "task": "docs/plans/52-agent-effectiveness-and-engineering-workflows.md#CAP01",
                    "paths": [
                        "docs/plans/52i-canonical-capabilities-and-progressive-reveal.md"
                    ],
                },
            )
        )
        self.assertEqual(actual["payload"]["schema"], "workspace-task-v1")
        self.assertEqual(
            [item["native_id"] for item in actual["payload"]["native_scope"]], ["CAP01"]
        )
        self.assertFalse(
            any(
                source.get("text") is not None
                for source in actual["payload"]["sources"]
            )
        )
        self.assertNotIn("pse._native", sys.modules)
        self.assertTrue(await self.refused("workspace_task_context", {"task": 12}))
        self.assertTrue(
            await self.refused("workspace_task_context", {"unknown": "field"})
        )

    async def test_exact_compatibility_invocation_preserves_visibility_and_strict_contract(
        self,
    ) -> None:
        arguments = {"paths": ["python/pse/_native.pyi"], "limit": 3}
        self.assertTrue(
            await self.refused(
                "capabilities_invoke_read",
                {
                    "capability": "workspace_producer_relationships",
                    "arguments": arguments,
                },
            )
        )
        await self.client.call_tool(
            "capabilities_select",
            {"capabilities": ["workspace_producer_relationships"]},
        )
        direct = host_content(
            await self.client.call_tool("workspace_producer_relationships", arguments)
        )
        compatible = host_content(
            await self.client.call_tool(
                "capabilities_invoke_read",
                {
                    "capability": "workspace_producer_relationships",
                    "arguments": arguments,
                },
            )
        )
        self.assertEqual(compatible, direct)
        self.assertTrue(
            await self.refused(
                "capabilities_invoke_read",
                {
                    "capability": "workspace_producer_relationships",
                    "arguments": {"paths": ["python/pse/_native.pyi"], "limit": "3"},
                },
            )
        )
        self.assertTrue(
            await self.refused(
                "capabilities_invoke_read",
                {"capability": "capabilities_reset", "arguments": {}},
            )
        )
        self.assertTrue(
            await self.refused(
                "capabilities_invoke_read",
                {"capability": "sh", "arguments": {"command": "anything"}},
            )
        )
        await self.client.call_tool("capabilities_reset", {})
        self.assertTrue(
            await self.refused(
                "capabilities_invoke_read",
                {
                    "capability": "workspace_producer_relationships",
                    "arguments": arguments,
                },
            )
        )

    async def test_planning_workflow_retrieves_preparation_only_without_effects(
        self,
    ) -> None:
        with patch(
            "subprocess.Popen",
            side_effect=AssertionError("instruction retrieval started a process"),
        ):
            selected = content(
                await self.client.call_tool(
                    "capabilities_select", {"workflows": ["plan-functional-review"]}
                )
            )
        guides = {item["id"] for item in selected["instruction_resources"]}
        self.assertIn("skill-plan-functional-extension-review", guides)
        self.assertNotIn("skill-functional-extension-review", guides)
        self.assertNotIn("skill-execute-plan", guides)
        self.assertNotIn("pse._native", sys.modules)

    async def test_presentation_controls_are_read_only_to_the_external_world(
        self,
    ) -> None:
        tools = {
            tool.name: tool
            for tool in await self.client.list_tools(cache_mode="refresh")
        }
        for name in ("capabilities_select", "capabilities_reset"):
            annotations = tools[name].annotations
            if annotations is None:
                self.fail("Presentation controls need truthful annotations")
            self.assertTrue(annotations.read_only_hint)
            self.assertFalse(annotations.destructive_hint)
            self.assertFalse(annotations.open_world_hint)

    async def test_unfamiliar_native_tool_has_real_owner_description_not_fake_handler(
        self,
    ) -> None:
        integration = content(
            await self.client.call_tool(
                "capabilities_domains", {"domain": "integration"}
            )
        )
        self.assertIn("git-integration", {item["id"] for item in integration["items"]})
        described = content(
            await self.client.call_tool(
                "capabilities_describe",
                {"capability": "git-integration", "include_schema": True},
            )
        )
        self.assertFalse(described["callable"])
        self.assertIsNone(described["input_schema"])
        await self.client.call_tool(
            "capabilities_select", {"capabilities": ["workspace_recipe_describe"]}
        )
        actual = host_content(
            await self.client.call_tool(
                "workspace_recipe_describe", {"recipe": "labels-sync"}
            )
        )
        self.assertEqual(
            actual["payload"]["definition_route"], ["just", "--show", "labels-sync"]
        )
        self.assertFalse(actual["payload"]["leaf_schema_complete"])
        self.assertTrue(actual["payload"]["leaf_contracts"])

    async def test_selection_replaces_resets_and_unknown_is_atomic(self) -> None:
        await self.client.call_tool(
            "capabilities_select", {"capabilities": ["workspace_task_context"]}
        )
        again = content(
            await self.client.call_tool(
                "capabilities_select", {"capabilities": ["workspace_task_context"]}
            )
        )
        self.assertFalse(again["changed"])
        self.assertTrue(
            await self.refused(
                "capabilities_select",
                {"capabilities": ["workspace_concepts_query", "unknown"]},
            )
        )
        self.assertEqual(await self.listed(), BOOTSTRAP | {"workspace_task_context"})
        await self.client.call_tool(
            "capabilities_select",
            {"capabilities": ["workspace_producer_relationships"]},
        )
        self.assertEqual(
            await self.listed(), BOOTSTRAP | {"workspace_producer_relationships"}
        )
        await self.client.call_tool("capabilities_reset", {})
        self.assertEqual(await self.listed(), BOOTSTRAP)
        self.assertTrue(
            await self.refused(
                "workspace_producer_relationships",
                {"paths": ["python/pse/_native.pyi"]},
            )
        )

    async def test_concurrent_selections_never_accumulate_union(self) -> None:
        await asyncio.gather(
            self.client.call_tool(
                "capabilities_select", {"capabilities": ["workspace_task_context"]}
            ),
            self.client.call_tool(
                "capabilities_select", {"capabilities": ["workspace_concepts_query"]}
            ),
        )
        self.assertIn(
            await self.listed(),
            (
                BOOTSTRAP | {"workspace_task_context"},
                BOOTSTRAP | {"workspace_concepts_query"},
            ),
        )
        await asyncio.gather(
            self.client.call_tool("capabilities_reset", {}),
            self.client.call_tool(
                "capabilities_select", {"capabilities": ["workspace_task_context"]}
            ),
        )
        self.assertIn(
            await self.listed(), (BOOTSTRAP, BOOTSTRAP | {"workspace_task_context"})
        )

    async def test_failed_guidance_preparation_does_not_publish_selection(self) -> None:
        await self.client.call_tool(
            "capabilities_select", {"capabilities": ["workspace_task_context"]}
        )
        with patch.object(
            self.runtime.guidance,
            "instruction_bundle",
            side_effect=ValueError("missing selected guide"),
        ):
            self.assertTrue(
                await self.refused(
                    "capabilities_select",
                    {"capabilities": ["workspace_concepts_query"]},
                )
            )
        self.assertEqual(await self.listed(), BOOTSTRAP | {"workspace_task_context"})

    async def test_compact_workflow_receipt_exact_defaults_and_deliberate_full_guidance(
        self,
    ) -> None:
        result = await self.client.call_tool(
            "capabilities_select", {"workflows": ["inspect-plan"]}
        )
        receipt = content(result)
        SelectionReceipt.model_validate(receipt)
        tools = {
            tool.name: tool
            for tool in await self.client.list_tools(cache_mode="refresh")
        }
        self.assertEqual(
            set(receipt["callable_tools"]),
            {"workspace_task_context", "document_scope_query"},
        )
        self.assertEqual(receipt["workflow_steps"], ["inspect-plan/scope"])
        self.assertFalse(receipt["instructions_embedded"])
        self.assertTrue(receipt["instructions_complete"])
        self.assertNotIn("instructions", receipt)
        self.assertTrue(
            all("content" not in item for item in receipt["instruction_resources"])
        )
        self.assertEqual(
            {item.uri for item in result.content if isinstance(item, ResourceLink)},
            {item["uri"] for item in receipt["instruction_resources"]},
        )
        schema = tools["capabilities_select"].output_schema
        if schema is None:
            self.fail("Selection must publish its typed receipt schema.")
        Draft202012Validator(schema).validate(receipt)
        for reference in receipt["instruction_resources"]:
            resource = await self.client.read_resource(reference["uri"])
            text = resource[0]
            if not isinstance(text, TextResourceContents):
                self.fail("Owned guide resources must contain JSON text.")
            full = json.loads(text.text)
            self.assertEqual(full["digest"], reference["digest"])
            self.assertEqual(full["content"], (ROOT / reference["path"]).read_text())
        repeated = content(
            await self.client.call_tool(
                "capabilities_select", {"workflows": ["inspect-plan"]}
            )
        )
        self.assertFalse(repeated["changed"])
        self.assertEqual(
            repeated["instruction_resources"], receipt["instruction_resources"]
        )
        generated = content(
            await self.client.call_tool(
                "capabilities_select", {"workflows": ["change-generated-contract"]}
            )
        )
        self.assertEqual(
            set(generated["callable_tools"]),
            {
                "workspace_task_context",
                "workspace_producer_relationships",
                "native_contract_schema",
            },
        )
        self.assertNotIn("contracts_generate", generated["capabilities"])
        family = content(
            await self.client.call_tool(
                "capabilities_select", {"capabilities": ["generated-contracts"]}
            )
        )
        self.assertIn("contracts_generate", family["callable_tools"])

    async def test_optional_named_steps_are_explicit_and_unknown_step_is_atomic(
        self,
    ) -> None:
        selected = content(
            await self.client.call_tool(
                "capabilities_select",
                {"workflow_steps": {"inspect-plan": ["native-route"]}},
            )
        )
        self.assertEqual(
            set(selected["callable_tools"]),
            {
                "workspace_task_context",
                "document_scope_query",
                "workspace_recipe_describe",
            },
        )
        before = await self.listed()
        self.assertTrue(
            await self.refused(
                "capabilities_select",
                {"workflow_steps": {"inspect-plan": ["invented"]}},
            )
        )
        self.assertEqual(await self.listed(), before)
        self.assertTrue(
            await self.refused(
                "capabilities_select", {"workflow_steps": {"invented": ["scope"]}}
            )
        )
        self.assertEqual(await self.listed(), before)

    async def test_owned_skill_manifest_support_and_projection_retain_visibility(
        self,
    ) -> None:
        selected = content(
            await self.client.call_tool(
                "capabilities_select", {"capabilities": ["skill-create-plan"]}
            )
        )
        reference = next(
            item
            for item in selected["instruction_resources"]
            if item["id"] == "skill-create-plan"
        )
        manifest_result = await self.client.read_resource(
            reference["supporting_manifest"]
        )
        manifest_text = manifest_result[0]
        if not isinstance(manifest_text, TextResourceContents):
            self.fail("Skill manifests must contain JSON text.")
        manifest = json.loads(manifest_text.text)
        self.assertEqual(manifest["package_digest"], reference["package_digest"])
        support = next(
            item
            for item in manifest["files"]
            if item["path"] == "references/plan-structure.md"
        )
        native = await self.client.read_resource(support["uri"])
        supporting_text = native[0]
        if not isinstance(supporting_text, TextResourceContents):
            self.fail("The selected Markdown support resource must contain text.")
        self.assertEqual(
            supporting_text.text,
            (
                ROOT / ".codex/skills/create-plan/references/plan-structure.md"
            ).read_text(),
        )
        projected = await self.client.call_tool(
            "read_resource", {"uri": support["uri"]}
        )
        self.assertFalse(projected.is_error)
        metadata = supporting_text.meta
        if metadata is None:
            self.fail("Supporting text must retain its captured source metadata.")
        self.assertEqual(metadata["digest"], support["digest"])
        await self.client.call_tool("capabilities_reset", {})
        with self.assertRaises(MCPError):
            await self.client.read_resource(support["uri"])
        projected = await self.client.call_tool(
            "read_resource", {"uri": support["uri"]}, raise_on_error=False
        )
        self.assertTrue(projected.is_error)

    async def test_owner_input_refusal_keeps_structured_cause(self) -> None:
        await self.client.call_tool(
            "capabilities_select", {"capabilities": ["workspace_concepts_query"]}
        )
        result = await self.client.call_tool(
            "workspace_concepts_query",
            {"request": {"concept": "plan-package", "identity": "CON01"}},
            raise_on_error=False,
        )
        self.assertTrue(result.is_error)
        text = " ".join(item.text for item in result.content if hasattr(item, "text"))
        self.assertIn("pse.workspace.input_refused", text)
        diagnostic = json.loads(text)
        self.assertEqual(diagnostic["stage"], "owner_read")
        self.assertTrue(diagnostic["remedy"])
        self.assertNotIn("CON01", text)
        self.assertNotIn("cause", diagnostic)

    async def test_tagged_concept_request_returns_schema_conformant_selected_answer(
        self,
    ) -> None:
        await self.client.call_tool(
            "capabilities_select", {"capabilities": ["workspace_concepts_query"]}
        )
        tool = self.runtime.catalogue.tools["workspace_concepts_query"]
        request = {
            "request": {
                "concept": "research-question",
                "identity": "RQ-host-contract",
                "origin": "user-question",
                "question": "Which owner declares this field?",
            }
        }
        Draft202012Validator(tool.parameters).validate(request)
        result = await self.client.call_tool("workspace_concepts_query", request)
        self.assertIsNotNone(tool.output_schema)
        Draft202012Validator(tool.output_schema).validate(content(result))
        answer = host_content(result)
        self.assertEqual(answer["operation"], "workspace_concepts_query")
        self.assertTrue(answer["complete"])
        self.assertEqual(answer["answer"]["population"], "complete")
        selected = answer["payload"]["items"]
        self.assertEqual(len(selected), 1)
        self.assertEqual(selected[0]["native_id"], "RQ-host-contract")
        self.assertEqual(selected[0]["basis"], "user-question")
        self.assertEqual(selected[0]["label"], "Which owner declares this field?")
        for invalid in (
            {"concept": "research-question", "identity": "RQ-host-contract"},
            {"concept": "diagnostic", "selected_reference": None},
            {"concept": "decision", "topics": [""]},
            {"concept": "not-a-native-concept"},
        ):
            with self.subTest(request=invalid):
                self.assertTrue(
                    await self.refused("workspace_concepts_query", {"request": invalid})
                )
        self.assertTrue(
            await self.refused("workspace_concepts_query", {"concept": "decision"})
        )

    async def test_actual_tagged_input_diagnostics_redact_values_and_unknown_field_names(
        self,
    ) -> None:
        await self.client.call_tool(
            "capabilities_select", {"capabilities": ["workspace_concepts_query"]}
        )
        redaction_marker = "credential-value-must-not-escape"
        unknown = "credential-field-name-must-not-escape"
        result = await self.client.call_tool(
            "workspace_concepts_query",
            {
                "request": {
                    "concept": "research-question",
                    "identity": 7,
                    "origin": "user-question",
                    "question": redaction_marker,
                    unknown: redaction_marker,
                }
            },
            raise_on_error=False,
        )
        self.assertTrue(result.is_error)
        text = " ".join(item.text for item in result.content if hasattr(item, "text"))
        self.assertNotIn(redaction_marker, text)
        self.assertNotIn(unknown, text)
        diagnostic = json.loads(text)
        self.assertEqual(diagnostic["code"], "pse.capabilities.input_refused")
        self.assertEqual(diagnostic["stage"], "input_validation")
        self.assertTrue(diagnostic["remedy"])
        locations = diagnostic["locations"]
        self.assertTrue(
            any(
                item["path"][-1] == "identity" and item["code"] == "string_type"
                for item in locations
            )
        )
        self.assertTrue(
            any(
                item["path"][-1] == "<redacted-key>"
                and item["code"] == "extra_forbidden"
                for item in locations
            )
        )

    async def test_notifications_do_not_claim_rollback_after_commit(self) -> None:
        original = server.save_visibility_rules

        async def notify_failure(ctx: Context, rules: list[dict]) -> None:
            await original(ctx, rules)
            raise RuntimeError("disconnected observer")

        with patch.object(server, "save_visibility_rules", side_effect=notify_failure):
            selected = content(
                await self.client.call_tool(
                    "capabilities_select", {"capabilities": ["workspace_task_context"]}
                )
            )
        self.assertTrue(selected["changed"])
        self.assertTrue(selected["notification_omissions"])
        self.assertEqual(await self.listed(), BOOTSTRAP | {"workspace_task_context"})

    async def test_source_only_startup_does_not_inspect_inventory_or_call_work(
        self,
    ) -> None:
        with patch(
            "subprocess.run",
            side_effect=AssertionError("startup must not run a command"),
        ):
            runtime = build(ROOT)
        self.assertNotIn("pse._native", sys.modules)
        self.assertTrue(
            {
                "workspace_task_context",
                "workspace_concepts_query",
                "workspace_producer_relationships",
                "workspace_recipe_describe",
                "workspace_inventory",
            }
            <= set(runtime.catalogue.tools),
        )


class StdioTests(unittest.IsolatedAsyncioTestCase):
    async def test_fixed_launcher_survives_unavailable_workload_admission_and_store(
        self,
    ) -> None:
        executable = shutil.which("just")
        if executable is None:
            self.fail("Just is required to test its fixed launcher")
        with tempfile.TemporaryDirectory() as directory:
            transport = StdioTransport(
                command=executable,
                args=["capabilities"],
                cwd=str(ROOT),
                env={
                    "PSE_RESOURCE_CLASS": "invalid-workload-class",
                    "PSE_SURREAL_STATE": str(Path(directory) / "absent-store"),
                    "UV_PROJECT_ENVIRONMENT": str(
                        Path(directory) / "absent-foreign-venv"
                    ),
                },
                keep_alive=False,
                log_file=Path(directory) / "server-stderr.log",
            )
            try:
                async with Client(transport, timeout=20) as client:
                    self.assertEqual(
                        {
                            tool.name
                            for tool in await client.list_tools(cache_mode="refresh")
                        },
                        BOOTSTRAP,
                    )
                    await client.call_tool(
                        "capabilities_select",
                        {"capabilities": ["workspace_task_context"]},
                    )
                    actual = host_content(
                        await client.call_tool(
                            "capabilities_invoke_read",
                            {
                                "capability": "workspace_task_context",
                                "arguments": {
                                    "task": "docs/plans/52-agent-effectiveness-and-engineering-workflows.md#CAP01"
                                },
                            },
                        )
                    )
                    self.assertEqual(
                        actual["payload"]["native_scope"][0]["native_id"], "CAP01"
                    )
                    self.assertFalse((Path(directory) / "absent-store").exists())
                    self.assertFalse((Path(directory) / "absent-foreign-venv").exists())
            finally:
                await transport.close()

    async def test_two_real_processes_reveal_and_teardown_independently(self) -> None:
        directory = Path(self.enterContext(tempfile.TemporaryDirectory()))
        transports = [
            StdioTransport(
                command=sys.executable,
                args=["-m", "scripts.capabilities"],
                cwd=str(ROOT),
                keep_alive=False,
                log_file=directory / f"server-{index}-stderr.log",
            )
            for index in range(2)
        ]
        try:
            async with (
                Client(transports[0], timeout=20) as first,
                Client(transports[1], timeout=20) as second,
            ):
                self.assertEqual(
                    {
                        item.name
                        for item in await first.list_tools(cache_mode="refresh")
                    },
                    BOOTSTRAP,
                )
                await first.call_tool(
                    "capabilities_select",
                    {"capabilities": ["workspace_producer_relationships"]},
                )
                self.assertIn(
                    "workspace_producer_relationships",
                    {
                        item.name
                        for item in await first.list_tools(cache_mode="refresh")
                    },
                )
                self.assertEqual(
                    {
                        item.name
                        for item in await second.list_tools(cache_mode="refresh")
                    },
                    BOOTSTRAP,
                )
                actual = host_content(
                    await first.call_tool(
                        "workspace_producer_relationships",
                        {"paths": ["python/pse/_native.pyi"], "limit": 3},
                    )
                )
                self.assertEqual(
                    actual["operation"], "workspace_producer_relationships"
                )
                self.assertIsInstance(actual["payload"], dict)
                await first.call_tool("capabilities_reset", {})
                self.assertEqual(
                    {
                        item.name
                        for item in await first.list_tools(cache_mode="refresh")
                    },
                    BOOTSTRAP,
                )
        finally:
            for transport in transports:
                await transport.close()


if __name__ == "__main__":
    unittest.main()
