# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Pure native schema inspection without execution adapter imports."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path
from typing import Annotated, Literal

from fastmcp.tools import Tool
from mcp.types import ToolAnnotations
from pydantic import Field

from scripts.capabilities.diagnostics import refusal
from scripts.capabilities.model import Capability
from scripts.capabilities.native_schema import SchemaSet
from scripts.capabilities.responses import ReadResult, bounded


def bindings(root: Path) -> tuple[tuple[Capability, ...], dict[str, Tool]]:
    schemas = SchemaSet(root)
    specs: list[Capability] = []
    tools: dict[str, Tool] = {}

    def native_contract_schema(
        slug: Annotated[str, Field(pattern=r"^[a-z][a-z0-9-]{0,95}$")],
        direction: Literal["input", "output"] | None = None,
        pointer: Annotated[str, Field(max_length=4096)] = "",
    ) -> ReadResult:
        """Read an exact generated native schema or authoring guidance with no native import, provider setup or service. Native schemas require their input/output direction; the exact authoring selector has no direction. Large roots report omissions: select an original JSON Pointer to inspect detail. Local references always retain their original root_ref scope. Authoring guidance describes applicable logical source sections; raw TOML/.pse/Parquet use native original-byte loaders."""
        try:
            return bounded(
                schemas.read(slug, direction, pointer), "native_contract_schema"
            )
        except (ValueError, OSError) as error:
            raise refusal(
                "pse.native_schema.read_refused", operation="native_contract_schema"
            ) from error

    schema_spec = Capability(
        id="native_contract_schema",
        title="Read exact native schema or authoring guidance",
        description=native_contract_schema.__doc__ or "Read generated schema",
        domain="generation",
        family="generated-contracts",
        guides=("root", "tooling"),
        owner="crates/pse-codegen/src/codegen/documents.rs",
        source="scripts/capabilities/schema_bindings.py",
        exposure="tool",
        effects="Read prepared generated schema resources only; no native import, installation, build or service.",
    )
    schema_tool = Tool.from_function(
        native_contract_schema,
        annotations=ToolAnnotations(
            read_only_hint=True,
            destructive_hint=False,
            idempotent_hint=True,
            open_world_hint=False,
        ),
        tags=schema_spec.tags,
        meta={
            "pse.native_schema": {
                "schema_set_fingerprint": schemas.fingerprint,
                "authoring_selector": {
                    "slug": "authoring",
                    "direction": None,
                    "applicability": "Logical source sections only; not a native raw-bundle wire document.",
                },
            }
        },
    )
    specs.append(schema_spec)
    tools[schema_spec.id] = schema_tool
    return tuple(specs), tools
