# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Bounded host queries consume the existing source, observation and report owners."""

from __future__ import annotations

import os
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

    entries = (
        (
            environment_explain,
            "environment",
            "environment-preparation",
            "scripts/pse_env.py",
            ("environment",),
        ),
        (
            storage_observe,
            "environment",
            "build-storage",
            "scripts/build_storage.py",
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
                invocation_lifetime="single",
                effects="Bounded owner read; no subject execution or disposal",
            )
        )
    return tuple(specs), tools
