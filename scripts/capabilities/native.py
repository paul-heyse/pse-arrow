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
from scripts.capabilities.diagnostics import safe_path, schema_keys
from scripts.capabilities.model import Capability
from scripts.capabilities.native_contracts import (
    BINDINGS,
    EXACT_READS,
    Binding,
)
from scripts.capabilities.native_schema import JSON_OBJECT, SchemaSet
from scripts.capabilities.process import run_worker
from scripts.capabilities.responses import ReadResult, bounded
from scripts.capabilities.workers import WORKERS

_PENDING: set[asyncio.Task[dict[str, JsonValue]]] = set()


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


def document_output_schema(schemas: SchemaSet, slug: str) -> dict[str, JsonValue]:
    """Compose the transport with its original directional Rust document schema."""
    document = schemas.document(slug, "output")
    definitions = document.pop("$defs", {})
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
        WORKERS["native"].module,
    ]
    with (
        tempfile.TemporaryDirectory(prefix="pse-native-admission-") as directory,
        tempfile.TemporaryFile() as output,
        tempfile.TemporaryFile() as errors,
    ):
        admitted = execution_basis.publish(
            Path(directory), root, "native", operation, request
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

    def bind_context(self, root: Path, binding: Binding, schemas: SchemaSet) -> None:
        self._root = root
        self._binding = binding
        self._schemas = schemas

    async def run(self, arguments: dict[str, object]) -> ToolResult:
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
                if self._binding.prepare_import:
                    # Actions use the shared settled transport: cancellation
                    # signals only this admitted helper and joins native drain.
                    value = await run_worker(self._root, self._binding.worker, data)
                else:
                    task = asyncio.create_task(
                        asyncio.to_thread(
                            run_child, self._root, self._binding.operation, data
                        )
                    )
                    _PENDING.add(task)
                    task.add_done_callback(_PENDING.discard)
                    # Cancelling a read detaches this observer. Its native read
                    # and source protection finish under their existing owner.
                    value = await asyncio.shield(task)
        response = (
            intact_result(value, self.name)
            if self._binding.operation in EXACT_READS
            else bounded(value, self.name)
        )
        return ToolResult(structured_content=response.model_dump(mode="json"))


def bindings(root: Path) -> tuple[tuple[Capability, ...], dict[str, Tool]]:
    schemas = SchemaSet(root)
    specs: list[Capability] = []
    tools: dict[str, Tool] = {}
    for binding in BINDINGS:
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
            worker_calls=(binding.worker,),
            requirements=binding.requirements,
            prerequisites=(
                (
                    "Existing compatible native Python extension; this selected action may prepare native import providers through the existing environment owner.",
                )
                if binding.prepare_import
                else (
                    "Existing compatible prebuilt extension and already prepared linked import environment. Explicitly prepare with just py-sync-native-correctness, attach from scripts/pse-env --native -- <endpoint>, and reattach after preparation. This read never builds import providers.",
                )
            ),
            guides=("root", "tooling"),
        )
        tool = NativeTool(
            name=name,
            description=binding.description,
            parameters=parameters,
            output_schema=(
                document_output_schema(schemas, binding.output_document)
                if binding.operation in EXACT_READS
                and binding.output_document is not None
                else ReadResult.model_json_schema(mode="serialization")
            ),
            tags=spec.tags,
            meta={
                "pse.native": {
                    "receiver": binding.receiver,
                    "contracts": contracts,
                    "schema_set_fingerprint": schemas.fingerprint,
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
                read_only_hint=not binding.prepare_import,
                destructive_hint=False,
                idempotent_hint=True,
                open_world_hint=False,
            ),
        )
        tool.bind_context(root, binding, schemas)
        specs.append(spec)
        tools[name] = tool

    schema_specs, schema_tools = schema_bindings.bindings(root)
    return (*specs, *schema_specs), {**tools, **schema_tools}
