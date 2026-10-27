# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Source-only native bindings with fixed, admitted public-facade children."""

from __future__ import annotations

import asyncio
import json
import os
import subprocess
import tempfile
from pathlib import Path

from fastmcp.tools import Tool, ToolResult
from mcp.types import ToolAnnotations
from pydantic import JsonValue, PrivateAttr, ValidationError

from scripts.capabilities import execution_basis, schema_bindings
from scripts.capabilities.cleanup import settle
from scripts.capabilities.diagnostics import safe_path, schema_keys
from scripts.capabilities.model import Capability
from scripts.capabilities.native_contracts import (
    BINDINGS,
    Binding,
)
from scripts.capabilities.native_read_sessions import ReadSessions
from scripts.capabilities.native_schema import JSON_OBJECT, SchemaSet
from scripts.capabilities.process import run_worker
from scripts.capabilities.responses import ReadResult, bounded
from scripts.capabilities.workers import WorkerCall


def intact_result(value: dict[str, JsonValue], operation: str) -> ReadResult:
    """Preserve exact selected native documents/IPC, or refuse them as a whole."""
    if len(json.dumps(value, ensure_ascii=True, allow_nan=False).encode()) > 120 * 1024:
        capture = operation in {
            "telemetry_profile_capture",
            "native_telemetry_profile_capture",
        }
        diagnostic = operation.startswith("native_diagnostic_") or capture
        value = {
            "outcome": "detail_refused",
            "code": "pse.native.observation_detail_bound"
            if diagnostic
            else "pse.native.result_detail_bound",
            "remedy": "Read the original retained CaptureDocument in the explicitly requested capture output; no capture facts were truncated."
            if capture
            else "Narrow the original diagnostic selection or use its explicit native reader; no target facts were truncated."
            if diagnostic
            else "Narrow the selected field/row window; exact IPC remains available through the protected reader.",
        }
    omissions = value.get("omissions", [])
    return ReadResult(
        operation=operation,
        complete=value.get("complete") is not False
        and value.get("outcome") not in {"detail_refused", "incomplete_observation"},
        omissions=[item for item in omissions if isinstance(item, str)]
        if isinstance(omissions, list)
        else [],
        payload=value,
    )


def document_output_schema(
    schemas: SchemaSet, slug: str, *, many: bool = False
) -> dict[str, JsonValue]:
    """Compose the transport with its original directional Rust document schema."""
    document = schemas.document(slug, "output")
    definitions = document.pop("$defs", {})
    if many:
        document = {"type": "array", "items": document}
    observed = {
        "type": "object",
        "additionalProperties": False,
        "properties": {"outcome": {"const": "observed"}, "document": document},
        "required": ["outcome", "document"],
    }
    refusal = {
        "type": "object",
        "properties": {"outcome": {"type": "string", "not": {"const": "observed"}}},
        "required": ["outcome"],
    }
    schema = ReadResult.model_json_schema(mode="serialization")
    schema["$defs"] = definitions
    schema["properties"]["payload"] = {"oneOf": [observed, refusal]}
    return JSON_OBJECT.validate_python(schema)


def run_child(
    root: Path, operation: str, arguments: dict[str, JsonValue]
) -> dict[str, JsonValue]:
    """Invoke one fixed module through the existing native environment owner."""
    try:
        request = json.dumps(
            {"operation": operation, "arguments": arguments}, allow_nan=False
        ).encode()
    except (ValueError, TypeError):
        return {"outcome": "malformed_invocation", "code": "pse.native.non_json_input"}
    if len(request) > 16 << 20:
        return {"outcome": "malformed_invocation", "code": "pse.native.request_bound"}
    binding = next((item for item in BINDINGS if item.operation == operation), None)
    if binding is None or binding.prepare_import:
        return {
            "outcome": "malformed_invocation",
            "code": "pse.native.unknown_fixed_read",
        }
    command = [
        str(root / "scripts/pse-env"),
        "--",
        str(root / ".venv/bin/python"),
        "-m",
        binding.worker.declaration.module,
    ]
    with (
        tempfile.TemporaryDirectory(prefix="pse-native-admission-") as directory,
        tempfile.TemporaryFile() as output,
        tempfile.TemporaryFile() as errors,
    ):
        admitted = execution_basis.publish(
            Path(directory), root, binding.worker.worker, operation, request
        )
        try:
            completed = subprocess.run(
                command,
                input=request,
                stdout=output,
                stderr=errors,
                cwd=root,
                env={**os.environ, **admitted},
                check=False,
            )
        except OSError as error:
            return {
                "outcome": "prerequisite_unavailable",
                "code": "pse.native.child_unavailable",
                "cause": type(error).__name__,
            }
        if completed.returncode:
            return {
                "outcome": "transport_failure",
                "code": "pse.native.child_failed",
                "exit_status": completed.returncode,
                "message": "Inspect the native environment owner; child stderr is withheld because it can include source or credentials.",
            }
        output.seek(0)
        data = output.read((1 << 20) + 1)
    if len(data) > 1 << 20:
        return {
            "outcome": "incomplete_observation",
            "code": "pse.native.response_bound",
            "complete": False,
            "omissions": [
                "Native response exceeds the transport bound; narrow the selected observation."
            ],
        }
    try:
        return JSON_OBJECT.validate_json(data, strict=True)
    except (ValueError, UnicodeDecodeError):
        return {
            "outcome": "transport_failure",
            "code": "pse.native.invalid_child_response",
        }


class NativeTool(Tool):
    _root: Path = PrivateAttr()
    _binding: Binding = PrivateAttr()
    _schemas: SchemaSet = PrivateAttr()
    _reads: ReadSessions = PrivateAttr()

    def bind_context(
        self,
        root: Path,
        binding: Binding,
        schemas: SchemaSet,
        reads: ReadSessions | None = None,
    ) -> None:
        self._root = root
        self._binding = binding
        self._schemas = schemas
        self._reads = reads if reads is not None else ReadSessions(root)

    def read_lifetime(self) -> ReadSessions:
        """The endpoint owns this lifetime across both registered dispatch routes."""
        return self._reads

    def select_read_lifetime(self, owner: ReadSessions) -> None:
        self._reads = owner

    async def run(self, arguments: dict[str, object]) -> ToolResult:
        if self._reads.admission_closed:
            return ToolResult(
                structured_content=intact_result(
                    {
                        "outcome": "transport_failure",
                        "code": "pse.native.endpoint_closed_before_admission",
                        "native_receiver": "not_entered",
                    },
                    self.name,
                ).model_dump(mode="json")
            )
        value: dict[str, JsonValue]
        try:
            request = self._binding.request.model_validate(arguments, strict=True)
        except ValidationError as error:
            value = {
                "outcome": "malformed_invocation",
                "code": "pse.native.host_envelope_refused",
                "fields": [
                    {
                        "location": list[JsonValue](
                            safe_path(
                                list(item["loc"]),
                                schema_keys(self._binding.request.model_json_schema()),
                            )
                        ),
                        "code": item["type"],
                    }
                    for item in error.errors(include_input=False)[:32]
                ],
            }
        else:
            data = request.model_dump(mode="json")
            if self._binding.operation == "result_read" and data["end"] < data["start"]:
                value = {
                    "outcome": "malformed_invocation",
                    "code": "pse.native.reversed_result_window",
                }
            else:
                data["schema_set_fingerprint"] = self._schemas.fingerprint
                if (
                    self._binding.read_eligibility == "exact_context"
                    and self._reads.foreground_selected
                ):
                    value = await self._reads.run(self._binding, data)
                elif self._binding.prepare_import:
                    # Actions use the shared settled transport: cancellation
                    # signals only this admitted helper and joins native drain.
                    value = await self._reads.observe(
                        run_worker(self._root, self._binding.worker, data)
                    )
                else:
                    task = self._reads.observe(
                        asyncio.to_thread(
                            run_child, self._root, self._binding.operation, data
                        )
                    )
                    # Cancelling a read detaches this observer. Its native read
                    # and source protection finish under their existing owner.
                    value = await asyncio.shield(task)
        response = (
            intact_result(value, self.name)
            if self._binding.preserve_document
            else bounded(value, self.name)
        )
        return ToolResult(structured_content=response.model_dump(mode="json"))


def open_bound(tools: dict[str, Tool]) -> None:
    """Select retention only under an actual foreground endpoint lifetime."""
    replacements: dict[ReadSessions, ReadSessions] = {}
    for tool in tools.values():
        if isinstance(tool, NativeTool):
            owner = tool.read_lifetime()
            if owner.closed:
                if owner not in replacements:
                    replacements[owner] = owner.successor()
                owner = replacements[owner]
                tool.select_read_lifetime(owner)
            owner.foreground_selected = True


async def close_bound(tools: dict[str, Tool]) -> None:
    """Settle every distinct native read owner after endpoint EOF, including dedicated reads."""
    owners = {
        tool.read_lifetime() for tool in tools.values() if isinstance(tool, NativeTool)
    }
    for owner in owners:
        owner.begin_close()
    await settle(*(owner.close() for owner in owners))


def begin_close_bound(tools: dict[str, Tool]) -> None:
    """Synchronously close admission before the endpoint yields to any drain."""
    for tool in tools.values():
        if isinstance(tool, NativeTool):
            tool.read_lifetime().begin_close()


def bindings(root: Path) -> tuple[tuple[Capability, ...], dict[str, Tool]]:
    reads = ReadSessions(root)
    product_schemas = SchemaSet(root)
    evidence_schemas = SchemaSet(root, family="evidence")
    specs: list[Capability] = []
    tools: dict[str, Tool] = {}
    for binding in BINDINGS:
        schemas = (
            evidence_schemas
            if binding.declaration.worker == "evidence"
            else product_schemas
        )
        host_schema = binding.request.model_json_schema()
        parameters = schemas.envelope(
            host_schema, binding.documents, tuple(host_schema.get("required", ()))
        )
        contracts = {
            field: schemas.identity(slug) for field, slug in binding.documents.items()
        }
        if binding.output_document:
            contracts["output_document"] = schemas.identity(
                binding.output_document, "output"
            )
        name = "native_" + binding.operation
        spec = Capability(
            id=name,
            title=binding.title,
            description=binding.description,
            domain=binding.domain,
            family=(
                "scientific-diagnostics"
                if binding.domain == "diagnostics"
                else "projects-revisions"
            ),
            owner=binding.receiver,
            source="scripts/capabilities/native.py",
            exposure="tool",
            invocation_lifetime="single",
            effects=binding.effects,
            worker_calls=(
                WorkerCall(
                    "native-read-prepared" if binding.prepare_import else "native-read",
                    binding.operation,
                    "detached",
                ),
            )
            if binding.read_eligibility == "exact_context"
            else (binding.worker,),
            requirements=binding.requirements,
            prerequisites=(
                (
                    "Existing explicitly built pse-evidence executable; no scientific extension or service is used.",
                )
                if binding.declaration.worker == "evidence"
                else (
                    "Current installed correctness extension in its recorded mode/profile; source admission may prepare native providers through their existing owner only when that installed selection needs them.",
                )
                if binding.prepare_import
                else (
                    "Current installed correctness extension and its original recorded mode/profile/provider evidence. For a native-mode receipt attach from its already admitted native environment; this passive read prepares no providers.",
                )
            ),
            guides=("root", "tooling"),
        )
        tool = NativeTool(
            name=name,
            description=binding.description,
            parameters=parameters,
            output_schema=(
                document_output_schema(
                    schemas, binding.output_document, many=binding.output_many
                )
                if binding.output_document is not None
                else ReadResult.model_json_schema(mode="serialization")
            ),
            tags=spec.tags,
            meta={
                "pse.native": {
                    "receiver": binding.receiver,
                    "contracts": contracts,
                    "schema_set_fingerprint": schemas.fingerprint,
                    "read_eligibility": binding.read_eligibility,
                    "preserve_document": binding.preserve_document,
                    "authoring_guidance": (
                        {
                            "path": "docs/generated/schema/authoring.schema.json",
                            "scope": "Applicable authored source sections only; native original-byte loaders own admission.",
                        }
                        if "physical" in binding.documents
                        else None
                    ),
                }
            },
            annotations=ToolAnnotations(
                read_only_hint=binding.effect_facts.read_only,
                destructive_hint=binding.effect_facts.destructive,
                idempotent_hint=binding.effect_facts.idempotent,
                open_world_hint=False,
            ),
        )
        tool.bind_context(root, binding, schemas, reads)
        specs.append(spec)
        tools[name] = tool

    schema_specs, schema_tools = schema_bindings.bindings(root)
    return (*specs, *schema_specs), {**tools, **schema_tools}
