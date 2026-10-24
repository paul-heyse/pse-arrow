# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Real source-only host bindings; no compiler, native extension or store startup."""

from __future__ import annotations

from typing import TYPE_CHECKING, Annotated

from fastmcp.tools import Tool
from mcp.types import ToolAnnotations
from pydantic import Field

from scripts.capabilities import read_contracts
from scripts.capabilities.responses import owner_read
from scripts.capabilities.model import Capability
from scripts.workspace_domain_contracts import (
    ConceptQuery,  # noqa: TC001 -- FastMCP evaluates tagged request annotations at registration.
)


if TYPE_CHECKING:
    from collections.abc import Callable
    from pathlib import Path

Bound = Annotated[int, Field(ge=1, le=100)]
Offset = Annotated[int, Field(ge=0)]
PathList = Annotated[list[str], Field(max_length=8)]


def bindings(root: Path) -> tuple[tuple[Capability, ...], dict[str, Tool]]:
    def workspace_concepts_query(
            request: ConceptQuery,
        ) -> read_contracts.ConceptQueryResult | read_contracts.DetailRefused:
            """Query exact owner-defined concept alternatives, facets and original references.

            The concept tag selects required identity/origin/reference fields. Owner examples describe
            valid first requests; unknown native identities and unsupported adapters retain native truth.
            No similarity, lexical fallback or source/service preparation is performed.
            """
            from scripts.workspace_domains import query  # noqa: PLC0415

            return owner_read(
                "workspace_concepts_query",
                lambda: query(request, root=root),
                project=read_contracts.concept_query,
            )

    rows = (
        (
                    workspace_concepts_query,
                    "Exact native concepts",
                    "scripts/workspace_domains.py",
                    ("inspect-plan", "plan-functional-review"),
                ),
    )
    specs = []
    tools: dict[str, Tool] = {}
    for function, title, source, workflows in rows:
        tool = Tool.from_function(
            function,
            annotations=ToolAnnotations(
                read_only_hint=True,
                destructive_hint=False,
                idempotent_hint=True,
                open_world_hint=False,
            ),
        )
        spec = Capability(
            id=tool.name,
            title=title,
            description=tool.description or title,
            domain="workspace",
            family="source-context",
            workflows=workflows,
            guides=("root", "tooling"),
            owner=source,
            source=source,
            exposure="tool",
            effects="Read selected source/Git/metadata only; no build, installation, service startup or native execution.",
        )
        tool.tags.update(spec.tags)
        specs.append(spec)
        tools[spec.id] = tool
    return tuple(specs), tools
