# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Compose workspace operations without owning their implementation dependencies."""
from __future__ import annotations
from pathlib import Path
from fastmcp.tools import Tool
from scripts.capabilities.model import Capability
from scripts.capabilities import context_bindings, concept_bindings, producer_bindings, entrypoint_bindings

def bindings(root: Path) -> tuple[tuple[Capability, ...], dict[str, Tool]]:
    specs = []
    tools: dict[str, Tool] = {}
    for factory in (context_bindings.bindings, concept_bindings.bindings, producer_bindings.bindings, entrypoint_bindings.bindings):
        declared, bound = factory(root)
        specs.extend(declared)
        tools.update(bound)
    return tuple(specs), tools
