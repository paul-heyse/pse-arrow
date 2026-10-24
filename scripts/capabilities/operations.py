# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Real source-only host bindings; no compiler, native extension or store startup."""

from __future__ import annotations

import json
from typing import TYPE_CHECKING, Annotated

from fastmcp.exceptions import ToolError
from fastmcp.tools import Tool
from mcp.types import ToolAnnotations
from pydantic import Field, JsonValue, TypeAdapter

from scripts.capabilities.model import Capability, Contract

if TYPE_CHECKING:
    from collections.abc import Callable
    from pathlib import Path

JSON = TypeAdapter(JsonValue)
Bound = Annotated[int, Field(ge=1, le=100)]
Offset = Annotated[int, Field(ge=0)]
PathList = Annotated[list[str], Field(max_length=8)]


class ReadResult(Contract):
    operation: str
    complete: bool
    omissions: list[str]
    payload: dict[str, JsonValue]


def bounded(result: object, operation: str) -> ReadResult:
    value = JSON.validate_python(result)
    if not isinstance(value, dict):
        raise TypeError("host owner must return an object")
    omissions: list[str] = []
    remaining = 128 * 1024

    def walk(item: JsonValue, location: str, depth: int) -> JsonValue:
        nonlocal remaining
        if remaining <= 0 or depth > 12:
            omissions.append(location + ": detail bound exceeded; narrow the request")
            return None
        if isinstance(item, str):
            size = len(json.dumps(item, ensure_ascii=True).encode())
            if size > min(16384, remaining):
                omissions.append(
                    location + ": string detail omitted; narrow the request"
                )
                return None
            remaining -= size
            return item
        if isinstance(item, list):
            if len(item) > 100:
                omissions.append(
                    location + ": showing first 100 items; narrow or page owner query"
                )
            return [
                walk(child, f"{location}[{index}]", depth + 1)
                for index, child in enumerate(item[:100])
            ]
        if isinstance(item, dict):
            if item.get("truncated") is True or item.get("complete") is False:
                omissions.append(
                    location + ": owner reports incomplete or truncated detail"
                )
            gaps = item.get("gaps")
            if isinstance(gaps, list) and gaps:
                omissions.append(
                    location + ": owner reports gaps; inspect original gaps"
                )
            result = {}
            for index, (key, child) in enumerate(item.items()):
                if index == 100:
                    omissions.append(location + ": object detail bound exceeded")
                    break
                remaining -= len(json.dumps(key).encode()) + 4
                if key == "text" and location.startswith("$.source_facts"):
                    omissions.append(
                        location
                        + ".text: raw source body omitted; inspect selected source owner"
                    )
                    continue
                result[key] = walk(child, location + "." + key, depth + 1)
            return result
        remaining -= 32
        return item

    projected = walk(value, "$", 0)
    if not isinstance(projected, dict):
        raise TypeError("host projection must remain an object")
    native_omissions = value.get("omissions", [])
    if isinstance(native_omissions, list):
        omissions = [
            item for item in native_omissions if isinstance(item, str)
        ] + omissions
    return ReadResult(
        operation=operation,
        complete=not omissions and value.get("complete") is not False,
        omissions=list(dict.fromkeys(omissions)),
        payload=projected,
    )


def owner_read(operation: str, read: Callable[[], object]) -> ReadResult:
    try:
        value = read()
    except (ValueError, OSError) as error:
        code = (
            "input_refused" if isinstance(error, ValueError) else "source_unavailable"
        )
        raise ToolError(
            json.dumps(
                {
                    "code": "pse.workspace." + code,
                    "operation": operation,
                    "cause": type(error).__name__,
                    "message": str(error),
                },
                sort_keys=True,
            )
        ) from error
    return bounded(value, operation)


def bindings(root: Path) -> tuple[tuple[Capability, ...], dict[str, Tool]]:
    def workspace_task_context(
        task: str | None = None, paths: PathList | None = None
    ) -> ReadResult:
        """Read an exact task/package and selected source paths. Derived facts retain native status, source protection and omissions; no work is executed."""
        from scripts.workspace_tasks import task_packet  # noqa: PLC0415

        return owner_read(
            "workspace_task_context",
            lambda: task_packet(task, tuple(paths or ()), root=root),
        )

    def workspace_concepts_query(
        concept: str,
        identity: str | None = None,
        document: str | None = None,
        limit: Bound = 20,
        offset: Offset = 0,
    ) -> ReadResult:
        """Query exact owner-native concepts and identities. No similarity or lexical fallback; incomplete source relationships remain explicit."""
        from scripts.workspace_domains import query  # noqa: PLC0415

        return owner_read(
            "workspace_concepts_query",
            lambda: query(
                concept,
                identity,
                root=root,
                document=document,
                limit=limit,
                offset=offset,
            ),
        )

    def workspace_producer_relationships(
        paths: PathList, limit: Bound = 40
    ) -> ReadResult:
        """Find declared producer relationships for exact source/output paths. Unknown edges are not evidence of no impact; generated files name edit owners."""
        from scripts.workspace_sources import relationships  # noqa: PLC0415

        return owner_read(
            "workspace_producer_relationships",
            lambda: relationships(root=root, exact_paths=tuple(paths), limit=limit),
        )

    def workspace_recipe_describe(recipe: str) -> ReadResult:
        """Describe an exact public recipe from its current declaration, including native grammar and limitations. This never runs that recipe."""
        from scripts.capabilities.inventory import describe_recipe  # noqa: PLC0415

        return owner_read(
            "workspace_recipe_describe", lambda: describe_recipe(root, recipe)
        )

    def workspace_inventory(
        offset: Offset = 0, limit: Bound = 50, kind: str | None = None
    ) -> ReadResult:
        """Explicitly inspect current public recipe/script/native/skill entry points. This bounded read never executes the discovered operations."""
        from scripts.capabilities.inventory import inspect_entrypoints  # noqa: PLC0415

        return owner_read(
            "workspace_inventory",
            lambda: inspect_entrypoints(root, offset=offset, limit=limit, kind=kind),
        )

    rows = (
        (
            workspace_task_context,
            "Task context",
            "scripts/workspace_tasks.py",
            ("inspect-plan",),
        ),
        (
            workspace_concepts_query,
            "Exact native concepts",
            "scripts/workspace_domains.py",
            ("inspect-plan", "plan-functional-review"),
        ),
        (
            workspace_producer_relationships,
            "Declared producer relationships",
            "scripts/workspace_sources.py",
            ("change-generated-contract",),
        ),
        (
            workspace_recipe_describe,
            "Exact native recipe",
            "justfile",
            ("inspect-plan",),
        ),
        (
            workspace_inventory,
            "Current entry-point inventory",
            "scripts/capabilities/inventory.py",
            ("inspect-plan",),
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
