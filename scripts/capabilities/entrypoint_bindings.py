# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Real source-only host bindings; no compiler, native extension or store startup."""

from __future__ import annotations

from typing import TYPE_CHECKING, Annotated

from fastmcp.tools import Tool
from mcp.types import ToolAnnotations
from pydantic import Field

from scripts.capabilities import read_contracts
from scripts.capabilities.entrypoints import describe_recipe, inspect_entrypoints
from scripts.capabilities.model import Capability
from scripts.capabilities.responses import owner_read

if TYPE_CHECKING:
    from pathlib import Path

Bound = Annotated[int, Field(ge=1, le=100)]
Offset = Annotated[int, Field(ge=0)]
PathList = Annotated[list[str], Field(max_length=8)]


def bindings(root: Path) -> tuple[tuple[Capability, ...], dict[str, Tool]]:
    def workspace_recipe_describe(
        recipe: str,
    ) -> read_contracts.RecipeDescriptionResult | read_contracts.DetailRefused:
        """Describe an exact public recipe from its current declaration, including native grammar and limitations. This never runs that recipe."""
        return owner_read(
            "workspace_recipe_describe",
            lambda: describe_recipe(root, recipe),
            project=read_contracts.recipe_description,
        )

    def workspace_inventory(
        offset: Offset = 0, limit: Bound = 50, kind: str | None = None
    ) -> read_contracts.InventoryResult | read_contracts.DetailRefused:
        """Explicitly inspect current public recipe/script/native/skill entry points. This bounded read never executes the discovered operations."""
        return owner_read(
            "workspace_inventory",
            lambda: inspect_entrypoints(root, offset=offset, limit=limit, kind=kind),
            project=read_contracts.inventory,
        )

    rows = (
        (
            workspace_recipe_describe,
            "Exact native recipe",
            "justfile",
        ),
        (
            workspace_inventory,
            "Current entry-point inventory",
            "scripts/capabilities/entrypoints.py",
        ),
    )
    specs = []
    tools: dict[str, Tool] = {}
    for function, title, source in rows:
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
            guides=("root", "tooling"),
            owner=source,
            source=source,
            exposure="tool",
            invocation_lifetime="single",
            effects="Read selected source/Git/metadata only; no build, installation, service startup or native execution.",
        )
        tool.tags.update(spec.tags)
        specs.append(spec)
        tools[spec.id] = tool
    return tuple(specs), tools
