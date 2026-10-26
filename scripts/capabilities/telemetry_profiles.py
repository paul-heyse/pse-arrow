# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit telemetry-to-native-capture association; native contracts stay original."""

from __future__ import annotations

from typing import TYPE_CHECKING, Literal

from mcp.types import ToolAnnotations

from scripts.capabilities.model import Capability, Contract
from scripts.capabilities.native import NativeTool, document_output_schema
from scripts.capabilities.native_contracts import PROFILE_BINDING
from scripts.capabilities.native_schema import SchemaSet, object_value
from scripts.telemetry_investigation import QueryRequest, QuerySource

if TYPE_CHECKING:
    from pathlib import Path

    from fastmcp.tools import Tool


class CaptureContext(Contract):
    """Retained telemetry selection; it supplies no native attachment authority."""

    request: QueryRequest
    source: QuerySource
    status: Literal["available", "partial"]
    meaning: str
    association: Literal["operator_selected_native_target"] = (
        "operator_selected_native_target"
    )
    identity_admission: Literal[
        "original_checkpoint_preflight_and_capture_target_postcheck"
    ] = "original_checkpoint_preflight_and_capture_target_postcheck"
    atomic_identity_admission: Literal[False] = False


def bindings(root: Path) -> tuple[tuple[Capability, ...], dict[str, Tool]]:
    """Composition hook; source-only schema construction imports no pse facade."""
    schemas = SchemaSet(root)
    binding = PROFILE_BINDING
    host_schema = binding.request.model_json_schema()
    parameters = schemas.envelope(
        host_schema, binding.documents, tuple(host_schema.get("required", ()))
    )
    output = document_output_schema(schemas, "capture-document")
    context = CaptureContext.model_json_schema(mode="serialization")
    context_id = "urn:pse:host:telemetry-profile-context:v1"
    context["$id"] = context_id
    object_value(output["$defs"])["telemetry-profile-context"] = context
    payload = object_value(object_value(output["properties"])["payload"])
    branches = payload["oneOf"]
    if not isinstance(branches, list) or not branches:
        raise TypeError("original capture response schema has no observed branch")
    observed = object_value(branches[0])
    object_value(observed["properties"])["telemetry_context"] = {"$ref": context_id}
    observed["required"] = ["outcome", "document", "telemetry_context"]
    spec = Capability(
        id="telemetry_profile_capture",
        title=binding.title,
        description=binding.description,
        domain="diagnostics",
        family="active-diagnostics",
        owner=binding.receiver,
        source="scripts/capabilities/telemetry_profiles.py",
        exposure="tool",
        invocation_lifetime="single",
        effects=binding.effects,
        worker_calls=(binding.worker,),
        requirements=binding.requirements,
        prerequisites=(
            "Compatible prepared correctness extension and the exact original checkpoint identity/OperationId. Selected CPU capture additionally needs the existing native perf/Samply permissions and tools; none are installed or permission-tuned by this action.",
        ),
        guides=("diagnostics",),
    )
    tool = NativeTool(
        name=spec.id,
        description=binding.description,
        parameters=parameters,
        output_schema=output,
        tags=spec.tags,
        meta={
            "pse.native": {
                "receiver": binding.receiver,
                "schema_set_fingerprint": schemas.fingerprint,
                "contracts": {
                    **{
                        name: schemas.identity(slug)
                        for name, slug in binding.documents.items()
                    },
                    "output_document": schemas.identity("capture-document", "output"),
                },
            }
        },
        annotations=ToolAnnotations(
            read_only_hint=binding.effect_facts.read_only,
            destructive_hint=binding.effect_facts.destructive,
            idempotent_hint=binding.effect_facts.idempotent,
            open_world_hint=False,
        ),
    )
    tool.bind_context(root, binding, schemas)
    return (spec,), {spec.id: tool}
