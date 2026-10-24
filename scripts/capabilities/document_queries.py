# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Bounded host queries consume the existing source, observation and report owners."""

from __future__ import annotations

from typing import TYPE_CHECKING, Annotated, Literal

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
    def document_scope_query(
        path: str | None = None,
        state: Literal["open", "blocked", "deferred", "complete", "unknown"]
        | None = None,
        offset: Offset = 0,
        limit: Limit = 20,
    ) -> answers.DocumentScopeResult | answers.DetailRefused:
        """Page actual document-owned scope and exact optional state/path filters. This projection neither schedules work nor authorizes edits, decisions or retirement."""
        from fnmatch import (  # noqa: PLC0415 -- lazy operation owner.
            fnmatchcase,
        )

        from scripts import (  # noqa: PLC0415 -- lazy operation owner.
            document_lifecycle,
            document_metadata,
        )

        def read() -> dict:
            snapshot = document_lifecycle.snapshot(
                root, "scope", document_metadata.load_policy(root)
            )
            items = snapshot["items"]
            if path is not None:
                items = [item for item in items if fnmatchcase(item["document"], path)]
            if state is not None:
                items = [item for item in items if item["state"] == state]
            return {
                **snapshot,
                "items": items[offset : offset + limit],
                "total": len(items),
                "offset": offset,
                "next_offset": offset + limit if offset + limit < len(items) else None,
            }

        return owner_read("document_scope_query", read, project=answers.document_scope)

    entries = (
        (
            document_scope_query,
            "governance",
            "documentation-lifecycle",
            "scripts/document_lifecycle.py",
            ("docs", "decisions"),
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
