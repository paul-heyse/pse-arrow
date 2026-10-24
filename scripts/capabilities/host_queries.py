# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Bounded host queries consume the existing source, observation and report owners."""

from __future__ import annotations

import asyncio
import os
from pathlib import Path
from typing import Annotated, Literal

from fastmcp.tools import Tool
from mcp.types import ToolAnnotations
from pydantic import Field

from scripts.capabilities.model import Capability
from scripts.capabilities.operations import owner_read
from scripts.capabilities import read_contracts as answers

Limit = Annotated[int, Field(ge=1, le=100)]
Offset = Annotated[int, Field(ge=0)]
Tail = Annotated[int, Field(ge=0, le=100)]
Wait = Annotated[float, Field(ge=0, le=30)]
Paths = Annotated[list[str], Field(max_length=64)]


def bindings(root: Path) -> tuple[tuple[Capability, ...], dict[str, Tool]]:
    def environment_explain() -> answers.EnvironmentResult | answers.DetailRefused:
        """Observe checkout/tool placement without evaluating local shell, building or allocating work. Credentials are not request fields."""
        from scripts import (  # noqa: PLC0415 -- lazy operation owner.
            pse_env,
        )

        return owner_read(
            "environment_explain",
            lambda: pse_env.operation_description(root, os.environ, None),
            project=answers.environment,
        )

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
                tuple(paths or ()), root=root, concept=concept, identity=identity,
                excerpt_offset=excerpt_offset, excerpt_limit=excerpt_limit
            ),
            project=answers.knowledge,
        )

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

    def product_run_read(
        invocation: str,
        registry: str | None = None,
        subject: str | None = None,
        failures: bool = False,
        offset: Offset = 0,
        limit: Limit = 20,
        tail: Tail = 0,
    ) -> answers.ProductRunResult | answers.DetailRefused:
        """Read one authentic invocation and selected subjects, independent workload/assurance/drain and protected detail. Reader success is not a product pass; no rerun or implicit retry."""
        from scripts import (  # noqa: PLC0415 -- lazy operation owner.
            run_inspection,
        )

        return owner_read(
            "product_run_read",
            lambda: run_inspection.product_result(
                invocation,
                registry=Path(registry) if registry is not None else None,
                subject=subject,
                failures=failures,
                offset=offset,
                limit=limit,
                tail=tail,
            ),
            project=answers.product_run,
        )

    def assessment_read(
        directory: str,
        gate: str | None = None,
        failures: bool = False,
        tail: Tail = 0,
        source_detail: bool = False,
        full_export: bool = False,
        historical: bool = False,
    ) -> answers.AssessmentResult | answers.DetailRefused:
        """Read a selected assessment while borrowing its original report and reused origins. Qualification, incomplete scope and native causes survive; no printed-path scraping."""
        from scripts import (  # noqa: PLC0415 -- lazy operation owner.
            validation_result,
        )

        return owner_read(
            "assessment_read",
            lambda: validation_result.read_assessment(
                Path(directory),
                gate=gate,
                failures=failures,
                tail=tail,
                source_detail=source_detail,
                full_export=full_export,
                historical=historical,
            ),
            project=answers.assessment,
        )

    async def workload_watch(
        kind: Literal["run", "attempt", "unit"],
        handle: str,
        registry: str | None = None,
        cursor: str | None = None,
        max_wait: Wait = 0,
    ) -> answers.WorkloadWatchResult | answers.DetailRefused:
        """Detached bounded observation of one exact native owner. Cursor is bound to its selection; cancelling this read never cancels, drains or settles the subject."""
        from scripts import (  # noqa: PLC0415 -- lazy operation owner.
            activity_watch,
        )

        def read() -> answers.WorkloadWatchResult | answers.DetailRefused:
            selected = activity_watch.selection(
                kind, handle, Path(registry) if registry is not None else None
            )
            return owner_read(
                "workload_watch",
                lambda: activity_watch.watch(
                    selected, cursor=cursor, max_wait=max_wait
                ),
                project=answers.watch,
            )

        return await asyncio.to_thread(read)

    def storage_observe() -> answers.StorageResult | answers.DetailRefused:
        """Observe configured store identities only, without recursive size traversal, reclamation or an assertion of disposable storage."""
        from scripts import (  # noqa: PLC0415 -- lazy operation owner.
            build_storage,
            pse_env,
        )

        return owner_read(
            "storage_observe",
            lambda: {
                "items": build_storage.observe(
                    build_storage.stores(
                        root, pse_env.base_environment(root, os.environ)
                    ),
                    metadata_only=True,
                ),
                "reclamation_authorized": False,
            },
            project=answers.storage,
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

    def coverage_correspondence_read(directory: str, base: str) -> answers.CoverageResult | answers.DetailRefused:
        """Inspect a retained coverage capture against an exact Git base. Verify captured report/population digests and retain source freshness, uncovered regions and correspondence limits; coverage is not behavioral adequacy."""
        from scripts import (  # noqa: PLC0415 -- lazy operation owner.
            coverage_correspondence,
        )

        return owner_read(
            "coverage_correspondence_read",
            lambda: coverage_correspondence.inspect(root / directory, base),
            project=answers.coverage,
        )

    entries = (
        (
            environment_explain,
            "environment",
            "environment-preparation",
            "scripts/pse_env.py",
            ("environment",),
        ),
        (
            knowledge_source_corpus,
            "knowledge",
            "knowledge-recall",
            "scripts/knowledge_store.py",
            ("root",),
        ),
        (
            document_scope_query,
            "governance",
            "documentation-lifecycle",
            "scripts/document_lifecycle.py",
            ("docs", "decisions"),
        ),
        (
            product_run_read,
            "execution",
            "workload-observation",
            "scripts/run_inspection.py",
            ("qualification",),
        ),
        (
            assessment_read,
            "validation",
            "qualification-assessment",
            "scripts/validation_result.py",
            ("qualification",),
        ),
        (
            workload_watch,
            "execution",
            "workload-observation",
            "scripts/activity_watch.py",
            ("environment",),
        ),
        (
            storage_observe,
            "environment",
            "build-storage",
            "scripts/build_storage.py",
            ("environment",),
        ),
        (
            library_source_packet,
            "knowledge",
            "library-research",
            "scripts/workspace_sources.py",
            ("root",),
        ),
        (
            coverage_correspondence_read,
            "validation",
            "verification-techniques",
            "scripts/coverage_correspondence.py",
            ("qualification",),
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
