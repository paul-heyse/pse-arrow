# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Bounded host queries consume the existing source, observation and report owners."""

from __future__ import annotations

from typing import TYPE_CHECKING, Annotated

from fastmcp.tools import Tool
from mcp.types import ToolAnnotations
from pydantic import Field

from scripts.capabilities import read_contracts as answers
from scripts.capabilities.model import Capability
from scripts.capabilities.responses import owner_read

if TYPE_CHECKING:
    from pathlib import Path


Limit = Annotated[int, Field(ge=1, le=100)]
Offset = Annotated[int, Field(ge=0)]
Tail = Annotated[int, Field(ge=0, le=100)]
Wait = Annotated[float, Field(ge=0, le=30)]
Paths = Annotated[list[str], Field(max_length=64)]


def bindings(root: Path) -> tuple[tuple[Capability, ...], dict[str, Tool]]:
    def knowledge_source_corpus(
        paths: Paths | None = None,
        concept: str | None = None,
        identity: str | None = None,
        excerpt_offset: Offset = 0,
        excerpt_limit: Limit = 8,
    ) -> answers.KnowledgeResult | answers.DetailRefused:
        """Read exact source-native concepts OR explicit source paths with original attribution. No store, ranked search or session scan. Missing and empty selections retain the owner's refusal."""
        from scripts import (  # noqa: PLC0415 -- lazy operation owner.
            knowledge_store,
        )

        return owner_read(
            "knowledge_source_corpus",
            lambda: knowledge_store.corpus(
                tuple(paths or ()),
                root=root,
                concept=concept,
                identity=identity,
                excerpt_offset=excerpt_offset,
                excerpt_limit=excerpt_limit,
            ),
            project=answers.knowledge,
        )

    def library_source_packet(
        package: str,
        references: Annotated[list[str], Field(max_length=32)] | None = None,
        features: Annotated[list[str], Field(max_length=128)] | None = None,
        target: str | None = None,
        cargo_metadata: str | None = None,
        rustdoc: str | None = None,
    ) -> answers.LibraryResult | answers.DetailRefused:
        """Read an exact dependency pin/lock and selected source/API evidence. Supplied build metadata keeps its provenance and unknowns; this never resolves packages, downloads, compiles or infers missing library capability."""
        from scripts import (  # noqa: PLC0415 -- lazy operation owner.
            workspace_sources,
        )

        return owner_read(
            "library_source_packet",
            lambda: workspace_sources.library_packet(
                package,
                root=root,
                references=tuple(references or ()),
                features=tuple(features or ()),
                target=target,
                cargo_metadata=cargo_metadata,
                rustdoc=rustdoc,
            ),
            project=answers.library,
        )

    entries = (
        (
            knowledge_source_corpus,
            "knowledge",
            "knowledge-recall",
            "scripts/knowledge_store.py",
            ("root",),
        ),
        (
            library_source_packet,
            "knowledge",
            "library-research",
            "scripts/workspace_sources.py",
            ("root",),
        ),
    )
    tools: dict[str, Tool] = {}
    specs = []
    for function, domain, family, source, guides in entries:
        tool = Tool.from_function(
            function,
            annotations=ToolAnnotations(
                readOnlyHint=True, destructiveHint=False, openWorldHint=False
            ),
        )
        tools[tool.name] = tool
        specs.append(
            Capability(
                id=tool.name,
                title=tool.name.replace("_", " "),
                description=tool.description or "",
                domain=domain,
                family=family,
                owner=source,
                source=source,
                guides=guides,
                exposure="tool",
                effects="Bounded owner read; no subject execution or disposal",
            )
        )
    return tuple(specs), tools
