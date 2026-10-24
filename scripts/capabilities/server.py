# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""One cheap local endpoint; exact reveal delegates visibility to FastMCP."""

from __future__ import annotations

import asyncio
import json
from contextlib import asynccontextmanager
from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING, Annotated

from fastmcp import FastMCP
from fastmcp.exceptions import ToolError
from fastmcp.server.context import (
    Context,  # noqa: TC002 -- FastMCP evaluates tool annotations at registration.
)
from fastmcp.server.transforms.visibility import (
    get_visibility_rules,
    save_visibility_rules,
)
from fastmcp.tools import (
    ToolResult,  # noqa: TC002 -- FastMCP evaluates tool return annotations.
)
from pydantic import Field, JsonValue

from scripts.capabilities.coherence import LOADED, Interface, install
from scripts.capabilities import (
    host_actions,
    host_queries,
    native,
    operations,
    observation,
    presentation,
    qualification,
    specialists,
)
from scripts.capabilities.catalogue import JSON_OBJECT, Catalogue
from scripts.capabilities.guidance import Guidance, register_guides
from scripts.capabilities.inventory import inventory

if TYPE_CHECKING:
    from collections.abc import AsyncGenerator

    from fastmcp.tools import Tool

    from scripts.capabilities.model import Capability

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
If this host keeps its initial callable inventory, use capabilities_invoke_read for reads or
capabilities_invoke_action for explicit actions, with the exact selected capability and its
described input schema. Both delegate to that same registered tool, including visibility,
validation, middleware and native admission. Neither accepts an arbitrary command. A lost
response does not undo an action; inspect its authentic native handle before deciding next work.
"""


def refusal(code: str, error: ValueError) -> ToolError:
    return ToolError(json.dumps({"code": code, "message": str(error)}, sort_keys=True))


def bindings(
    root: Path, *, owner: specialists.Specialists
) -> tuple[tuple[Capability, ...], dict[str, Tool]]:
    """Compose actual owner bindings; duplicate identities remain composition errors."""
    specs = []
    tools = {}
    for factory in (
        operations.bindings,
        host_queries.bindings,
        host_actions.bindings,
        qualification.bindings,
        native.bindings,
        lambda path: specialists.bindings(path, owner=owner),
    ):
        declared, bound = factory(root)
        if tools.keys() & bound.keys():
            raise ValueError("duplicate operation binding")
        specs.extend(declared)
        tools.update(bound)
    return tuple(specs), tools


@dataclass(frozen=True)
class Runtime:
    server: FastMCP
    catalogue: Catalogue
    guidance: Guidance
    specialists: specialists.Specialists
    interface: Interface


def build(root: Path = ROOT) -> Runtime:
    root = root.resolve(strict=True)
    owner = specialists.Specialists(root)
    domains, workflows, guides, declared = inventory(root)
    bound, tools = bindings(root, owner=owner)
    catalogue = Catalogue(
        domains, workflows, guides, (*declared, *bound), tools,
        loaded_source_revision=LOADED.revision,
    )
    guidance = Guidance(root, guides)

    @asynccontextmanager
    async def lifetime(_server: FastMCP) -> AsyncGenerator[dict[str, JsonValue]]:
        try:
            yield {}
        finally:
            await owner.close()

    server = FastMCP(
        "pse-capabilities",
        instructions=INSTRUCTIONS,
        strict_input_validation=True,
        dereference_schemas=False,
        mask_error_details=True,
        lifespan=lifetime,
    )
    keys = register_guides(server, guidance)
    for tool in tools.values():
        server.add_tool(tool)
        server.disable(keys={tool.key})
    for identity, key in keys.items():
        if identity != "root":
            server.disable(keys={key})
    presentation_view = presentation.register(server, catalogue, guidance)
    lock = asyncio.Lock()

    @server.tool(annotations={"readOnlyHint": True, "destructiveHint": False})
    def capabilities_domains(
        domain: str | None = None, offset: Offset = 0, limit: Limit = 20
    ) -> dict[str, JsonValue]:
        """Browse exact domains, or page the capabilities of one domain. This reads loaded declarations, never probes or starts services."""
        try:
            return {**catalogue.browse("domain", domain, offset, limit),
                    "interface": JSON_OBJECT.validate_python(interface.status())}
        except ValueError as error:
            raise refusal("pse.capabilities.unknown_domain", error) from error

    @server.tool(annotations={"readOnlyHint": True, "destructiveHint": False})
    def capabilities_workflows(
        workflow: str | None = None, offset: Offset = 0, limit: Limit = 20
    ) -> dict[str, JsonValue]:
        """Browse exact workflows, or page their canonical capability identities. A workflow guides composition; it does not schedule work."""
        try:
            return {**catalogue.browse("workflow", workflow, offset, limit),
                    "interface": JSON_OBJECT.validate_python(interface.status())}
        except ValueError as error:
            raise refusal("pse.capabilities.unknown_workflow", error) from error

    @server.tool(annotations={"readOnlyHint": True, "destructiveHint": False})
    async def capabilities_describe(
        capability: str, ctx: Context, include_schema: bool = False
    ) -> dict[str, JsonValue]:
        """Describe one exact capability, including hidden/deferred tools. Rich schemas come from actual callable bindings, never guessed CLI grammar."""
        try:
            result = catalogue.describe(capability, include_schema=include_schema)
        except ValueError as error:
            raise refusal("pse.capabilities.unknown_capability", error) from error
        rules = await get_visibility_rules(ctx)
        selected = {
            key
            for rule in rules
            if rule.get("enabled") is True
            for key in rule.get("keys", [])
        }
        tool = tools.get(capability)
        result["visible"] = tool is not None and tool.key in selected
        result["interface"] = JSON_OBJECT.validate_python(interface.status(capability))
        return result

    async def replace(ctx: Context, selected_keys: set[str]) -> tuple[bool, list[str]]:
        rules = (
            [
                JSON_OBJECT.validate_python(
                    {"enabled": True, "keys": sorted(selected_keys)}
                )
            ]
            if selected_keys
            else []
        )
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
                    raise ToolError(
                        "pse.capabilities.selection_failed: visibility was not committed"
                    ) from error
                return True, [
                    "selection committed; list-change notification unavailable: "
                    + type(error).__name__
                ]
        return True, []

    @server.tool(
        annotations={
            "readOnlyHint": True,
            "destructiveHint": False,
            "idempotentHint": True,
            "openWorldHint": False,
        }
    )
    async def capabilities_select(
        ctx: Context,
        domains: Ids | None = None,
        workflows: Ids | None = None,
        capabilities: Ids | None = None,
    ) -> dict[str, JsonValue]:
        """Replace this attachment's view with exact selected domains/workflows/capabilities. No operation executes; checkout/service/permission context is unchanged."""
        try:
            selected = catalogue.select(
                domains or [], workflows or [], capabilities or []
            )
        except ValueError as error:
            raise refusal("pse.capabilities.unknown_selection", error) from error
        selected_keys = {
            tools[key].key for key in selected.capabilities if key in tools
        }
        selected_keys.update(keys[key] for key in selected.guides if key in keys)
        selected_keys.update(presentation_view.workflow_keys[key] for key in workflows or [])
        try:
            instructions = guidance.instruction_bundle(
                [key for key in selected.guides if key != "root"]
            )
        except ValueError as error:
            raise refusal("pse.capabilities.guidance_unavailable", error) from error
        changed, notifications = await replace(ctx, selected_keys)
        remaining = 256 * 1024
        for instruction in instructions:
            content = instruction.get("content")
            if isinstance(content, str):
                size = len(content.encode())
                if size > remaining:
                    instruction["content"] = None
                    instruction["complete"] = False
                    instruction["omissions"] = [
                        "selection response instruction bound; read this resource explicitly"
                    ]
                else:
                    remaining -= size
        return JSON_OBJECT.validate_python(
            {
                "revision": catalogue.revision,
                "interface": interface.status(),
                "changed": changed,
                "capabilities": list(selected.capabilities),
                "callable_tools": [
                    key for key in selected.capabilities if key in tools
                ],
                "non_callable": [
                    catalogue.describe(key)
                    for key in selected.capabilities
                    if key not in tools
                ],
                "guides": [
                    {"id": key, "uri": catalogue.guides[key].uri}
                    for key in selected.guides
                ],
                "instructions": instructions,
                "notification_omissions": notifications,
                "compatibility_invocation": "capabilities_invoke_read",
                "action_invocation": "capabilities_invoke_action",
                "context": {
                    "checkout": str(root),
                    "service_selection": "unchanged",
                    "effects": "presentation only",
                },
            }
        )

    @server.tool(
        annotations={
            "readOnlyHint": True,
            "destructiveHint": False,
            "idempotentHint": True,
            "openWorldHint": False,
        }
    )
    async def capabilities_reset(ctx: Context) -> dict[str, JsonValue]:
        """Restore bootstrap for this attachment. Native work is neither cancelled nor replayed; source/project/service context remains unchanged."""
        changed, notifications = await replace(ctx, set())
        return JSON_OBJECT.validate_python(
            {
                "revision": catalogue.revision,
                "interface": interface.status(),
                "changed": changed,
                "capabilities": [],
                "notification_omissions": notifications,
            }
        )

    @server.tool(
        annotations={
            "readOnlyHint": True,
            "destructiveHint": False,
            "openWorldHint": False,
        }
    )
    async def capabilities_invoke_read(
        capability: str, arguments: dict[str, JsonValue]
    ) -> ToolResult:
        """Invoke one exact selected read-only canonical tool when the host keeps a static tool list. First describe its real schema and select it. No shell/SQL/argv or bootstrap dispatch is accepted."""
        tool = tools.get(capability)
        if (
            tool is None
            or tool.annotations is None
            or tool.annotations.read_only_hint is not True
        ):
            raise ToolError(
                "pse.capabilities.read_binding_required: select an actual read-only canonical tool"
            )
        # Public FastMCP dispatch retains visibility, context, strict FunctionTool
        # input validation, middleware, original result and future native admission.
        return await server.call_tool(capability, arguments)

    @server.tool(
        annotations={
            "readOnlyHint": False,
            "destructiveHint": True,
            "idempotentHint": False,
            "openWorldHint": True,
        }
    )
    async def capabilities_invoke_action(
        capability: str, arguments: dict[str, JsonValue]
    ) -> ToolResult:
        """Invoke one exact selected canonical action after describing its schema/effects. Delegates its native validation, admission and lifetime. No arbitrary shell, method or bootstrap dispatch. Cancellation or transport failure does not establish rollback; use the owner's handle and settlement reader."""
        tool = tools.get(capability)
        if (
            tool is None
            or tool.annotations is None
            or tool.annotations.read_only_hint is not False
        ):
            raise ToolError(
                "pse.capabilities.action_binding_required: select an actual canonical action"
            )
        return await server.call_tool(capability, arguments)

    interface = install(server, LOADED, catalogue)
    observation.install(
        server,
        revision=catalogue.revision,
        operation_names=frozenset(tools) | frozenset({
            "capabilities_domains", "capabilities_workflows", "capabilities_describe",
            "capabilities_select", "capabilities_reset", "capabilities_invoke_read",
            "capabilities_invoke_action", "list_resources", "read_resource",
            "list_prompts", "get_prompt",
        }),
        resource_names={
            **{guide.uri: guide.id for guide in guides},
            presentation.CAPABILITY_TEMPLATE: "capability_description",
        },
        prompt_names=frozenset("workflow_" + identity for identity in catalogue.workflows),
    )
    return Runtime(server, catalogue, guidance, owner, interface)
