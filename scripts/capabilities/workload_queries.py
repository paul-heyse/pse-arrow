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

from scripts.capabilities import read_contracts as answers
from scripts.capabilities.model import Capability
from scripts.capabilities.responses import owner_read

Limit = Annotated[int, Field(ge=1, le=100)]
Offset = Annotated[int, Field(ge=0)]
Tail = Annotated[int, Field(ge=0, le=100)]
Wait = Annotated[float, Field(ge=0, le=30)]
Paths = Annotated[list[str], Field(max_length=64)]


def bindings(root: Path) -> tuple[tuple[Capability, ...], dict[str, Tool]]:
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

    entries = (
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
