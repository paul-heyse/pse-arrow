# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Compose query bindings; individual owners retain their actual source closures."""

from __future__ import annotations

from typing import TYPE_CHECKING

from scripts.capabilities import (
    coverage_queries,
    document_queries,
    environment_queries,
    knowledge_queries,
    rust_references,
    telemetry_queries,
    workload_queries,
)

if TYPE_CHECKING:
    from pathlib import Path

    from fastmcp.tools import Tool

    from scripts.capabilities.model import Capability


def bindings(root: Path) -> tuple[tuple[Capability, ...], dict[str, Tool]]:
    specs = []
    tools: dict[str, Tool] = {}
    for factory in (
        environment_queries.bindings,
        knowledge_queries.bindings,
        document_queries.bindings,
        workload_queries.bindings,
        coverage_queries.bindings,
        rust_references.bindings,
        telemetry_queries.bindings,
    ):
        declared, bound = factory(root)
        specs.extend(declared)
        tools.update(bound)
    return tuple(specs), tools
