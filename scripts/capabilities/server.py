# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""One cheap local endpoint; exact reveal delegates visibility to FastMCP."""

from __future__ import annotations

import asyncio
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Annotated

from fastmcp import FastMCP
from fastmcp.exceptions import ToolError
from fastmcp.server.context import Context
from fastmcp.tools import ToolResult
from fastmcp.server.transforms.visibility import get_visibility_rules, save_visibility_rules
from pydantic import Field, JsonValue

from scripts.capabilities.catalogue import JSON_OBJECT, Catalogue
from scripts.capabilities.guidance import Guidance, register_guides
from scripts.capabilities.inventory import inventory
from scripts.capabilities.operations import bindings

ROOT = Path(__file__).resolve().parents[2]
Limit = Annotated[int, Field(ge=1, le=100)]
Offset = Annotated[int, Field(ge=0)]
Ids = Annotated[list[str], Field(max_length=32)]
INSTRUCTIONS = """This is the single structured PSE capability entry point. Browse exact domains or
workflows, describe a capability (including hidden ones), then select the relevant IDs. Selection
replaces the previous view; reset restores bootstrap. Selection does not execute work or change
checkout, project, credentials or service. Descriptions distinguish callable host operations,
retained native routes and planned adaptations. Obtain relevant instructions through returned
guidance or resources; retrieving a planning skill does not authorize or execute implementation.
Native owners retain source/status, admission and outcome truth. Catalogue availability is not a
live service-readiness claim. Use exact IDs; no semantic or ranked discovery is provided.
If this host keeps its initial callable inventory, use capabilities_invoke_read with the exact
selected capability and its described input schema. It delegates to that same registered tool,
including visibility, strict validation and middleware; it accepts no arbitrary command.
"""


def refusal(code: str, error: ValueError) -> ToolError:
    return ToolError(json.dumps({"code": code, "message": str(error)}, sort_keys=True))


@dataclass(frozen=True)
class Runtime:
    server: FastMCP
    catalogue: Catalogue
    guidance: Guidance


def build(root: Path = ROOT) -> Runtime:
    root = root.resolve(strict=True)
    domains, workflows, guides, declared = inventory(root)
    bound, tools = bindings(root)
    catalogue = Catalogue(domains, workflows, guides, (*declared, *bound), tools)
    guidance = Guidance(root, guides)
    server = FastMCP("pse-capabilities", instructions=INSTRUCTIONS, strict_input_validation=True, mask_error_details=True)
    keys = register_guides(server, guidance)
    for identity, tool in tools.items():
        server.add_tool(tool)
        server.disable(keys={tool.key})
    for identity, key in keys.items():
        if identity != "root":
            server.disable(keys={key})
    lock = asyncio.Lock()

    @server.tool(annotations={"readOnlyHint": True, "destructiveHint": False})
    def capabilities_domains(domain: str | None = None, offset: Offset = 0, limit: Limit = 20) -> dict[str, JsonValue]:
        """Browse exact domains, or page the capabilities of one domain. This reads loaded declarations, never probes or starts services."""
        try:
            return catalogue.browse("domain", domain, offset, limit)
        except ValueError as error:
            raise refusal("pse.capabilities.unknown_domain", error) from error

    @server.tool(annotations={"readOnlyHint": True, "destructiveHint": False})
    def capabilities_workflows(workflow: str | None = None, offset: Offset = 0, limit: Limit = 20) -> dict[str, JsonValue]:
        """Browse exact workflows, or page their canonical capability identities. A workflow guides composition; it does not schedule work."""
        try:
            return catalogue.browse("workflow", workflow, offset, limit)
        except ValueError as error:
            raise refusal("pse.capabilities.unknown_workflow", error) from error

    @server.tool(annotations={"readOnlyHint": True, "destructiveHint": False})
    async def capabilities_describe(capability: str, ctx: Context, include_schema: bool = False) -> dict[str, JsonValue]:
        """Describe one exact capability, including hidden/deferred tools. Rich schemas come from actual callable bindings, never guessed CLI grammar."""
        try:
            result = catalogue.describe(capability, include_schema=include_schema)
        except ValueError as error:
            raise refusal("pse.capabilities.unknown_capability", error) from error
        rules = await get_visibility_rules(ctx)
        selected = {key for rule in rules if rule.get("enabled") is True for key in rule.get("keys", [])}
        tool = tools.get(capability)
        result["visible"] = tool is not None and tool.key in selected
        return result

    async def replace(ctx: Context, selected_keys: set[str]) -> tuple[bool, list[str]]:
        rules: list[dict[str, JsonValue]] = [{"enabled": True, "keys": sorted(selected_keys)}] if selected_keys else []
        async with lock:
            previous = await get_visibility_rules(ctx)
            if previous == rules:
                return False, []
            try:
                # A single complete rule write avoids reset/enable intermediate states.
                await save_visibility_rules(ctx, rules)
            except Exception as error:
                # Framework saves first, then notifies. A disconnected observer must not
                # turn a committed selection into a claimed rollback or replay effects.
                actual = await get_visibility_rules(ctx)
                if actual != rules:
                    raise ToolError("pse.capabilities.selection_failed: visibility was not committed") from error
                return True, ["selection committed; list-change notification unavailable: " + type(error).__name__]
        return True, []

    @server.tool(annotations={"readOnlyHint": True, "destructiveHint": False, "idempotentHint": True, "openWorldHint": False})
    async def capabilities_select(
        ctx: Context,
        domains: Ids | None = None,
        workflows: Ids | None = None,
        capabilities: Ids | None = None,
    ) -> dict[str, JsonValue]:
        """Replace this attachment's view with exact selected domains/workflows/capabilities. No operation executes; checkout/service/permission context is unchanged."""
        try:
            selected = catalogue.select(domains or [], workflows or [], capabilities or [])
        except ValueError as error:
            raise refusal("pse.capabilities.unknown_selection", error) from error
        selected_keys = {tools[key].key for key in selected.capabilities if key in tools}
        selected_keys.update(keys[key] for key in selected.guides if key in keys)
        changed, notifications = await replace(ctx, selected_keys)
        instructions = guidance.instruction_bundle([key for key in selected.guides if key != "root"])
        remaining = 256 * 1024
        for instruction in instructions:
            content = instruction.get("content")
            if isinstance(content, str):
                size = len(content.encode())
                if size > remaining:
                    instruction["content"] = None
                    instruction["complete"] = False
                    instruction["omissions"] = ["selection response instruction bound; read this resource explicitly"]
                else:
                    remaining -= size
        return JSON_OBJECT.validate_python({
            "revision": catalogue.revision,
            "changed": changed,
            "capabilities": list(selected.capabilities),
            "callable_tools": [key for key in selected.capabilities if key in tools],
            "non_callable": [catalogue.describe(key) for key in selected.capabilities if key not in tools],
            "guides": [{"id": key, "uri": catalogue.guides[key].uri} for key in selected.guides],
            "instructions": instructions,
            "notification_omissions": notifications,
            "compatibility_invocation": "capabilities_invoke_read",
            "context": {"checkout": str(root), "service_selection": "unchanged", "effects": "presentation only"},
        })

    @server.tool(annotations={"readOnlyHint": True, "destructiveHint": False, "idempotentHint": True, "openWorldHint": False})
    async def capabilities_reset(ctx: Context) -> dict[str, JsonValue]:
        """Restore bootstrap for this attachment. Native work is neither cancelled nor replayed; source/project/service context remains unchanged."""
        changed, notifications = await replace(ctx, set())
        return {"revision": catalogue.revision, "changed": changed, "capabilities": [], "notification_omissions": notifications}

    @server.tool(annotations={"readOnlyHint": True, "destructiveHint": False, "openWorldHint": False})
    async def capabilities_invoke_read(capability: str, arguments: dict[str, JsonValue]) -> ToolResult:
        """Invoke one exact selected read-only canonical tool when the host keeps a static tool list. First describe its real schema and select it. No shell/SQL/argv or bootstrap dispatch is accepted."""
        tool = tools.get(capability)
        if tool is None or tool.annotations is None or tool.annotations.read_only_hint is not True:
            raise ToolError("pse.capabilities.read_binding_required: select an actual read-only canonical tool")
        # Public FastMCP dispatch retains visibility, context, strict FunctionTool
        # input validation, middleware, original result and future native admission.
        return await server.call_tool(capability, arguments)

    return Runtime(server, catalogue, guidance)
