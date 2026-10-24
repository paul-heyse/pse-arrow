# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Real source-only host bindings; no compiler, native extension or store startup."""

from __future__ import annotations

from typing import TYPE_CHECKING, Annotated

from fastmcp.tools import Tool
from mcp.types import ToolAnnotations
from pydantic import Field

from scripts.capabilities import read_contracts
from scripts.capabilities.model import Capability
from scripts.capabilities.responses import owner_read

if TYPE_CHECKING:
    from pathlib import Path

Bound = Annotated[int, Field(ge=1, le=100)]
Offset = Annotated[int, Field(ge=0)]
PathList = Annotated[list[str], Field(max_length=8)]


def bindings(root: Path) -> tuple[tuple[Capability, ...], dict[str, Tool]]:
    def workspace_producer_relationships(
        paths: PathList, limit: Bound = 40
    ) -> read_contracts.ProducerRelationshipsResult | read_contracts.DetailRefused:
        """Find declared producer relationships for exact source/output paths. Unknown edges are not evidence of no impact; generated files name edit owners."""
        from scripts.workspace_sources import relationships  # noqa: PLC0415

        return owner_read(
            "workspace_producer_relationships",
            lambda: relationships(root=root, exact_paths=tuple(paths), limit=limit),
            project=read_contracts.producer_relationships,
        )

    rows = (
        (
            workspace_producer_relationships,
            "Declared producer relationships",
            "scripts/workspace_sources.py",
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
            effects="Read selected source/Git/metadata only; no build, installation, service startup or native execution.",
        )
        tool.tags.update(spec.tags)
        specs.append(spec)
        tools[spec.id] = tool
    return tuple(specs), tools
