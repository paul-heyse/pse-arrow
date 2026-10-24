# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Real source-only host bindings; no compiler, native extension or store startup."""

from __future__ import annotations

import json
import math
from typing import TYPE_CHECKING, Annotated, TypeVar, overload

from fastmcp.exceptions import ToolError
from fastmcp.tools import Tool
from mcp.types import ToolAnnotations
from pydantic import Field, JsonValue, TypeAdapter

from scripts.capabilities.model import Capability, Contract
from scripts.capabilities import read_contracts
from scripts.capabilities.diagnostics import refusal
from scripts.workspace_domain_contracts import ConceptQuery

T = TypeVar("T")

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
    # Reserve space for the envelope, structural punctuation and bounded loss
    # metadata; owner-provided omissions and keys share this response budget.
    remaining = 112 * 1024

    def omit(message: str) -> None:
        if len(omissions) < 31:
            omissions.append(message[:240])
        elif len(omissions) == 31:
            omissions.append(
                "Further detail omissions suppressed; narrow the owner query."
            )

    def walk(item: JsonValue, location: str, depth: int) -> JsonValue:
        nonlocal remaining
        if remaining <= 0 or depth > 12:
            omit(location + ": detail bound exceeded; narrow the request")
            return None
        if isinstance(item, str):
            size = len(json.dumps(item, ensure_ascii=True).encode())
            if size > min(16384, remaining):
                omit(location + ": string detail omitted; narrow the request")
                return None
            remaining -= size
            return item
        if isinstance(item, list):
            if len(item) > 100:
                omit(location + ": showing first 100 items; narrow or page owner query")
            return [
                walk(child, f"{location}[{index}]", depth + 1)
                for index, child in enumerate(item[:100])
            ]
        if isinstance(item, dict):
            if item.get("truncated") is True or item.get("complete") is False:
                omit(location + ": owner reports incomplete or truncated detail")
            gaps = item.get("gaps")
            if isinstance(gaps, list) and gaps:
                omit(location + ": owner reports gaps; inspect original gaps")
            result = {}
            for index, (key, child) in enumerate(item.items()):
                if index == 100:
                    omit(location + ": object detail bound exceeded")
                    break
                key_size = len(json.dumps(key, ensure_ascii=True).encode()) + 4
                if key_size > min(512, remaining):
                    omit(
                        f"{location}: object key at index {index} omitted; narrow the owner query"
                    )
                    continue
                remaining -= key_size
                if key == "text" and location.startswith("$.source_facts"):
                    omit(
                        location
                        + ".text: raw source body omitted; inspect selected source owner"
                    )
                    continue
                child_location = (location + "." + key)[:160]
                result[key] = walk(child, child_location, depth + 1)
            return result
        if isinstance(item, float) and not math.isfinite(item):
            raise ValueError("owner output contains a nonfinite JSON number")
        remaining -= 32
        return item

    projected = walk(value, "$", 0)
    if not isinstance(projected, dict):
        raise TypeError("host projection must remain an object")
    native_omissions = value.get("omissions", [])
    if isinstance(native_omissions, list):
        for item in native_omissions[:32]:
            if isinstance(item, str):
                omit(item)
    response = ReadResult(
        operation=operation,
        complete=not omissions and value.get("complete") is not False,
        omissions=list(dict.fromkeys(omissions)),
        payload=projected,
    )
    if (
        len(json.dumps(response.model_dump(mode="json"), ensure_ascii=True).encode())
        > 128 * 1024
    ):
        # A final encoded-size check covers escaping and every envelope byte.
        # Never return a supposedly bounded response whose metadata escapes it.
        return ReadResult(
            operation=operation[:128],
            complete=False,
            omissions=[
                "Encoded response exceeds the detail bound; narrow the owner query."
            ],
            payload={},
        )
    return response


@overload
def owner_read(operation: str, read: Callable[[], object]) -> ReadResult: ...


@overload
def owner_read(operation: str, read: Callable[[], dict], *, project: Callable[[dict], T]) -> T: ...


def owner_read(operation: str, read: Callable[[], object], *, project: Callable[[dict], object] | None = None) -> object:
    try:
        value = read()
    except (ValueError, OSError) as error:
        code = (
            "input_refused" if isinstance(error, ValueError) else "source_unavailable"
        )
        raise refusal("pse.workspace." + code, operation=operation) from error
    if project is not None:
        if not isinstance(value, dict):
            raise TypeError("host owner must return an object")
        return project(value)
    return bounded(value, operation)


def bindings(root: Path) -> tuple[tuple[Capability, ...], dict[str, Tool]]:
    def workspace_task_context(
        task: str | None = None, paths: PathList | None = None
    ) -> read_contracts.TaskContextResult | read_contracts.DetailRefused:
        """Read an exact task/package and selected source paths. Derived facts retain native status, source protection and omissions; no work is executed."""
        from scripts.workspace_tasks import task_packet  # noqa: PLC0415

        return owner_read(
            "workspace_task_context",
            lambda: task_packet(task, tuple(paths or ()), root=root),
            project=read_contracts.task_context,
        )

    def workspace_concepts_query(request: ConceptQuery) -> read_contracts.ConceptQueryResult | read_contracts.DetailRefused:
        """Query exact owner-defined concept alternatives, facets and original references.

        The concept tag selects required identity/origin/reference fields. Owner examples describe
        valid first requests; unknown native identities and unsupported adapters retain native truth.
        No similarity, lexical fallback or source/service preparation is performed.
        """
        from scripts.workspace_domains import query  # noqa: PLC0415

        return owner_read(
            "workspace_concepts_query", lambda: query(request, root=root),
            project=read_contracts.concept_query,
        )

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

    def workspace_recipe_describe(recipe: str) -> read_contracts.RecipeDescriptionResult | read_contracts.DetailRefused:
        """Describe an exact public recipe from its current declaration, including native grammar and limitations. This never runs that recipe."""
        from scripts.capabilities.inventory import describe_recipe  # noqa: PLC0415

        return owner_read(
            "workspace_recipe_describe", lambda: describe_recipe(root, recipe),
            project=read_contracts.recipe_description,
        )

    def workspace_inventory(
        offset: Offset = 0, limit: Bound = 50, kind: str | None = None
    ) -> read_contracts.InventoryResult | read_contracts.DetailRefused:
        """Explicitly inspect current public recipe/script/native/skill entry points. This bounded read never executes the discovered operations."""
        from scripts.capabilities.inventory import inspect_entrypoints  # noqa: PLC0415

        return owner_read(
            "workspace_inventory",
            lambda: inspect_entrypoints(root, offset=offset, limit=limit, kind=kind),
            project=read_contracts.inventory,
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
