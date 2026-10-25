# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit telemetry-to-native-capture association; native contracts stay original."""

from __future__ import annotations

from typing import TYPE_CHECKING, Literal

from mcp.types import ToolAnnotations
from pydantic import JsonValue, model_validator

from scripts.capabilities.model import Capability, Contract
from scripts.capabilities.native import NativeTool, document_output_schema
from scripts.capabilities.native_contracts import Binding
from scripts.capabilities.native_schema import SchemaSet, object_value
from scripts.telemetry_investigation import QueryAnswer, QueryRequest, QuerySource

if TYPE_CHECKING:
    from pathlib import Path

    from fastmcp.tools import Tool


class ProfileCaptureRequest(Contract):
    """Host association only; both native documents are decoded by their owner."""

    request: JsonValue
    expected: JsonValue
    suspect: QueryAnswer

    @model_validator(mode="after")
    def observed_source(self) -> ProfileCaptureRequest:
        if self.expected is None:
            raise ValueError("Capture requires an original checkpoint identity")
        if (
            self.suspect.source is None
            or self.suspect.status not in {"available", "partial"}
            or not self.suspect.records
        ):
            raise ValueError(
                "Select an actual bounded telemetry observation with its original source"
            )
        return self


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


PROFILE_BINDING = Binding(
    operation="telemetry_profile_capture",
    title="Capture an explicitly selected native telemetry suspect",
    request=ProfileCaptureRequest,
    documents={"request": "capture-request", "expected": "checkpoint-identity"},
    receiver="pse.diagnostics.read_checkpoint + capture + CaptureSession.wait/stop",
    description="Associate a selected bounded telemetry observation with an explicit original native CaptureRequest and CheckpointIdentity. Start only the requested finite Timeline/CPU capture against its exact checkpoint/OperationId, then return the intact original CaptureDocument. The original native owner validates operation/liveness; preflight and postcheck preserve process/start association but are not atomic expected-identity admission. Elapsed telemetry time is not CPU time, and late attachment cannot recover preceding samples. Cancelling this attached helper drains capture only; it never cancels or restarts science, starts telemetry services, changes permissions/tuning or launches a profiler UI. Existing output directories refuse. Capture failure, symbols, overlap and loss remain independent of scientific outcomes.",
    domain="diagnostics",
    effects="Explicit bounded native Timeline/CPU capture and fresh retained output only; native capture owns samplers/exporters and drains them independently of target science.",
    output_document="capture-document",
    prepare_import=True,
)


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
            read_only_hint=False,
            destructive_hint=False,
            idempotent_hint=False,
            open_world_hint=False,
        ),
    )
    tool.bind_context(root, binding, schemas)
    return (spec,), {spec.id: tool}
