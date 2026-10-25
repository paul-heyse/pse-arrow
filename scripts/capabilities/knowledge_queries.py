# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Bounded host queries consume the existing source, observation and report owners."""

from __future__ import annotations

import json
import os
from pathlib import Path
from typing import Annotated

from fastmcp.tools import Tool
from mcp.types import ToolAnnotations
from pydantic import Field, JsonValue

from scripts.capabilities import read_contracts as answers
from scripts.capabilities.model import Capability, Contract, Prerequisite
from scripts.capabilities.native import document_output_schema, intact_result
from scripts.capabilities.native_schema import JSON_OBJECT, SchemaSet, object_value
from scripts.capabilities.responses import ReadResult, owner_read

Limit = Annotated[int, Field(ge=1, le=100)]
Offset = Annotated[int, Field(ge=0)]
Tail = Annotated[int, Field(ge=0, le=100)]
Wait = Annotated[float, Field(ge=0, le=30)]
Paths = Annotated[list[str], Field(max_length=64)]
State = Annotated[str, Field(min_length=1, max_length=4096)]


class IndexedRequest(Contract):
    """Host routing only; the request body is declared and decoded by Rust."""

    request: JsonValue
    state: State | None = None
    timeout: Annotated[float, Field(gt=0, le=30)] = 30.0


def bindings(root: Path) -> tuple[tuple[Capability, ...], dict[str, Tool]]:
    def indexed_read(
        operation: str, request: JsonValue, state: str | None, timeout: float
    ) -> ReadResult:
        from scripts import knowledge_store  # noqa: PLC0415 -- selected original owner

        selected = state if state is not None else os.environ.get("PSE_SURREAL_STATE")
        if not selected:
            return intact_result(
                {
                    "outcome": "prerequisite_unavailable",
                    "code": "pse.knowledge.state_required",
                },
                operation,
            )
        if not isinstance(request, dict):
            return intact_result(
                {"outcome": "refused", "code": "pse.knowledge.input_refused"}, operation
            )
        try:
            value = knowledge_store.indexed(
                request,
                explain=operation == "knowledge_explain",
                state=Path(selected),
                tool=root / "target/debug/pse-evidence",
                root=root,
                timeout=timeout,
            )
        except (OSError, knowledge_store.NativeKnowledgeUnavailableError):
            return intact_result(
                {
                    "outcome": "prerequisite_unavailable",
                    "code": "pse.knowledge.owner_unavailable",
                    "remedy": "Inspect the selected service and original evidence executable; no source fallback was used.",
                },
                operation,
            )
        except ValueError:
            return intact_result(
                {"outcome": "refused", "code": "pse.knowledge.input_refused"}, operation
            )
        if len(json.dumps(value, allow_nan=False).encode()) > 120 * 1024:
            value = {
                "outcome": "detail_refused",
                "code": "pse.knowledge.response_bound",
                "remedy": "Narrow the native query page or use the original knowledge reader; no result facts were truncated.",
            }
        return intact_result(JSON_OBJECT.validate_python(value), operation)

    def knowledge_query(
        request: JsonValue,
        state: State | None = None,
        timeout: Annotated[float, Field(gt=0, le=30)] = 30.0,
    ) -> ReadResult:
        """Select an exact native indexed knowledge page; current source freshness is separate. No implicit import, refresh, service start or fallback."""
        return indexed_read("knowledge_query", request, state, timeout)

    def knowledge_explain(
        request: JsonValue,
        state: State | None = None,
        timeout: Annotated[float, Field(gt=0, le=30)] = 30.0,
    ) -> ReadResult:
        """Explain the same original native query without ANALYZE or source reopening; the raw version-specific plan is informational."""
        return indexed_read("knowledge_explain", request, state, timeout)

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
                invocation_lifetime="single",
            )
        )
    schemas = SchemaSet(root)
    for function, output in (
        (knowledge_query, "knowledge-page"),
        (knowledge_explain, "knowledge-explain"),
    ):
        tool = Tool.from_function(
            function,
            annotations=ToolAnnotations(
                readOnlyHint=True,
                destructiveHint=False,
                openWorldHint=False,
            ),
        )
        tool.parameters = schemas.envelope(
            IndexedRequest.model_json_schema(),
            {"request": "knowledge-query"},
            ("request",),
        )
        output_schema = document_output_schema(schemas, output)
        payload = object_value(object_value(output_schema["properties"])["payload"])
        alternatives = payload["oneOf"]
        if not isinstance(alternatives, list) or not alternatives:
            raise ValueError(
                "native output transport requires observed/refusal alternatives"
            )
        observed = object_value(alternatives[0])
        object_value(observed["properties"])["source_observations"] = {
            "type": "array",
            "items": {"type": "object"},
        }
        tool.output_schema = output_schema
        tool.meta = {
            "pse.native": {
                "contracts": {
                    "request": schemas.identity("knowledge-query"),
                    "output_document": schemas.identity(output, "output"),
                }
            }
        }
        tools[tool.name] = tool
        specs.append(
            Capability(
                id=tool.name,
                title=tool.name.replace("_", " "),
                description=tool.description or "",
                domain="knowledge",
                family="knowledge-recall",
                owner="crates/pse-operations/src/knowledge.rs",
                source="scripts/capabilities/knowledge_queries.py",
                exposure="tool",
                guides=("root", "tooling"),
                effects="Bounded original indexed query; no ANALYZE, import, refresh, startup or disposal",
                invocation_lifetime="single",
                requirements=(
                    Prerequisite(
                        kind="executable",
                        owner="xtask/src/evidence_tools.rs",
                        provider="pse-evidence",
                    ),
                    Prerequisite(
                        kind="selected-service",
                        owner="scripts/surreal_server.py",
                        provider="canonical-request",
                    ),
                ),
            )
        )
    return tuple(specs), tools
