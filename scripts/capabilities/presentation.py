# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Source-only content views of the loaded catalogue and owned guidance."""

from __future__ import annotations

import json
from dataclasses import dataclass
from functools import partial
from typing import TYPE_CHECKING

from fastmcp.exceptions import PromptError, ResourceError
from fastmcp.prompts import Prompt
from fastmcp.server.transforms import PromptsAsTools, ResourcesAsTools, Transform
from mcp.types import ToolAnnotations
from pydantic import (
    JsonValue,
)

from scripts.capabilities.diagnostics import refusal
from scripts.capabilities.model import Contract

if TYPE_CHECKING:
    from collections.abc import Sequence

    from fastmcp import FastMCP
    from fastmcp.server.transforms import GetToolNext
    from fastmcp.tools import Tool
    from fastmcp.utilities.versions import VersionSpec

    from scripts.capabilities.catalogue import Catalogue
    from scripts.capabilities.guidance import Guidance

MAX_PROMPT_BYTES = 256 * 1024
CAPABILITY_TEMPLATE = "pse://capabilities/{capability}"
CONTENT_TOOLS = frozenset(
    {"list_resources", "read_resource", "list_prompts", "get_prompt"}
)


class ContentAnnotations(Transform):
    """Describe the effects of this endpoint's pure content projections.

    Framework prompt projections have no default effect annotations. Preserve
    their actual dispatch and schemas, adding only the truthful local contract.
    This transform never changes action-tool annotations or host permissions.
    """

    @staticmethod
    def annotate(tool: Tool) -> Tool:
        if tool.name not in CONTENT_TOOLS:
            return tool
        return tool.model_copy(
            update={
                "annotations": ToolAnnotations(
                    readOnlyHint=True,
                    destructiveHint=False,
                    idempotentHint=True,
                    openWorldHint=False,
                )
            }
        )

    async def list_tools(self, tools: Sequence[Tool]) -> Sequence[Tool]:
        return [self.annotate(tool) for tool in tools]

    async def get_tool(
        self, name: str, call_next: GetToolNext, *, version: VersionSpec | None = None
    ) -> Tool | None:
        tool = await call_next(name, version=version)
        return self.annotate(tool) if tool is not None else None


class InstructionReference(Contract):
    id: str
    uri: str
    path: str
    digest: str | None
    complete: bool
    restart_required: bool
    omissions: list[str]
    supporting_manifest: str | None = None
    package_digest: str | None = None


class SelectionContext(Contract):
    checkout: str
    service_selection: str = "unchanged"
    effects: str = "presentation only"


class SelectionReceipt(Contract):
    revision: str
    interface: dict[str, JsonValue]
    changed: bool
    capabilities: list[str]
    callable_tools: list[str]
    non_callable: list[str]
    workflow_steps: list[str]
    instruction_resources: list[InstructionReference]
    instructions_complete: bool
    instructions_embedded: bool = False
    instruction_access: str = "Read each complete guide resource and, when relevant, its captured supporting-file manifest. Retrieval does not activate a skill or authorize work."
    notification_omissions: list[str]
    compatibility_invocation: str = "capabilities_invoke_read"
    action_invocation: str = "capabilities_invoke_action"
    context: SelectionContext


@dataclass(frozen=True)
class Presentation:
    """Keys the composition root includes in its atomic workflow selection."""

    workflow_keys: dict[str, str]


def register(server: FastMCP, catalogue: Catalogue, guidance: Guidance) -> Presentation:
    """Register derived content and framework tool projections on this endpoint.

    Capability metadata, like exact description, is readable before selection.
    Workflow prompts start hidden and never select or invoke their capabilities.
    Guide resources are registered by their existing composition owner.
    """

    @server.resource(
        CAPABILITY_TEMPLATE,
        name="capability_description",
        description="Exact loaded capability metadata and its actual callable schemas; no operation executes.",
        mime_type="application/json",
    )
    def capability_description(capability: str) -> dict[str, object]:
        try:
            return dict(catalogue.describe(capability, include_schema=True))
        except ValueError as error:
            raise ResourceError(
                str(refusal("pse.capabilities.unknown_capability"))
            ) from error

    async def workflow_content(workflow: str) -> str:
        selection = catalogue.select([], [workflow], [])
        # Check every dependency before touching guide bodies. A prompt enabled
        # on its own must not become a route around another component's visibility.
        for identity in selection.capabilities:
            if identity in catalogue.tools and await server.get_tool(identity) is None:
                raise PromptError(
                    "pse.capabilities.workflow_hidden: select the complete workflow"
                )
        for identity in selection.guides:
            if await server.get_resource(catalogue.guides[identity].uri) is None:
                raise PromptError(
                    "pse.capabilities.workflow_hidden: select the complete workflow"
                )
        try:
            instructions = guidance.references(selection.guides)
        except ValueError as error:
            raise PromptError(
                str(refusal("pse.capabilities.guidance_unavailable"))
            ) from error
        result = {
            "revision": catalogue.revision,
            "workflow": catalogue.workflows[workflow].model_dump(mode="json"),
            "capabilities": [
                {
                    "id": key,
                    "title": catalogue.capabilities[key].title,
                    "exposure": catalogue.capabilities[key].exposure,
                    "effects": catalogue.capabilities[key].effects,
                    "description_uri": "pse://capabilities/" + key,
                }
                for key in selection.capabilities
            ],
            "instruction_resources": instructions,
            "workflow_steps": list(selection.steps),
            "instructions_embedded": False,
            "instruction_access": "Read complete owned instructions through the listed resources. This prompt is a compact workflow index; no instruction body is truncated or duplicated here.",
            "effects": "Instruction retrieval only; no selection or workflow execution.",
        }
        content = json.dumps(result, sort_keys=True, allow_nan=False)
        if len(content.encode()) > MAX_PROMPT_BYTES:
            raise PromptError(
                "pse.capabilities.prompt_too_large: read the selected guide resources individually"
            )
        return content

    workflow_keys: dict[str, str] = {}
    for identity, workflow in catalogue.workflows.items():
        prompt = Prompt.from_function(
            partial(workflow_content, identity),
            name="workflow_" + identity,
            title=workflow.title,
            description=workflow.description,
            tags={"workflow:" + identity},
        )
        server.add_prompt(prompt)
        server.disable(keys={prompt.key})
        workflow_keys[identity] = prompt.key

    # These route through the same server's public dispatch, retaining its
    # visibility, authorization and middleware for both native and tool clients.
    server.add_transform(ResourcesAsTools(server))
    server.add_transform(PromptsAsTools(server))
    server.add_transform(ContentAnnotations())
    return Presentation(workflow_keys)
