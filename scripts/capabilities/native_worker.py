# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Fixed native operations. This child alone imports the public scientific facade."""

from __future__ import annotations

import base64
import hashlib
import json
import os
import sys
import tempfile
from collections import deque
from itertools import islice
from pathlib import Path
from typing import TYPE_CHECKING, TypeVar, cast

import msgspec
from pydantic import JsonValue

from scripts.capabilities.diagnostics import safe_path, schema_keys
from scripts.capabilities.execution_basis import AdmissionRefusedError
from scripts.capabilities.native_schema import JSON_OBJECT
from scripts.capabilities.responses import bounded
from scripts.capabilities.telemetry_profile_worker import capture
from scripts.capabilities.worker_progress import ProgressReporter
from scripts.capabilities.workers import WORKERS

if TYPE_CHECKING:
    from collections.abc import Mapping

    from jsonschema.exceptions import ValidationError as SchemaValidationError

    from pse import DiagnosticReport, Runtime
    from scripts.capabilities.native_contracts import Result

T = TypeVar("T")


MAXIMUM_REQUEST_BYTES = 16 << 20


def decoded(value: object, document_type: type[T]) -> T:
    from pse import (  # noqa: PLC0415 -- selected native or specialized test owner loads on demand
        codec,
    )
    from pse.contracts import (  # noqa: PLC0415 -- generated root aliases load only in the selected native child.
        documents,
    )

    try:
        return codec.decode_json(json.dumps(value, allow_nan=False), document_type)
    except msgspec.DecodeError as error:
        if native_report(error) is not None:
            raise
        # Decode/admission runs first. Only a failed generated decode gets a
        # structured schema explanation; it never grants native admission.
        from jsonschema import Draft202012Validator  # noqa: PLC0415

        from scripts.capabilities.native_schema import SchemaSet  # noqa: PLC0415

        schemas = SchemaSet(Path(__file__).resolve().parents[2])
        # Tagged generated roots may be union aliases, which have no __name__.
        # Match the actual exported value to the catalogue rather than inventing
        # a parallel type/slug map or flattening its branch error into prose.
        schema = None
        for name, exported in vars(documents).items():
            if exported is document_type:
                schema = schemas.input_for_type(name)
                if schema is not None:
                    break
        if schema is not None:
            keys = schema_keys(schema)
            roots = Draft202012Validator(schema).iter_errors(value)
            pending: deque[tuple[SchemaValidationError, int | None]] = deque(
                (item, None) for item in islice(roots, 17)
            )
            locations: list[dict[str, object]] = []
            while pending and len(locations) < 16:
                item, parent = pending.popleft()
                index = len(locations)
                locations.append(
                    {
                        "path": safe_path(list(item.absolute_path), keys),
                        "schema_path": list(item.absolute_schema_path),
                        "rule": item.validator,
                        "parent": parent,
                    }
                )
                pending.extend((child, index) for child in islice(item.context, 17))
            raise StructuralInputError(locations, complete=not pending) from error
        raise


def source_documents(value: object) -> list[dict[str, bytes]]:
    from pse.contracts.documents import (  # noqa: PLC0415 -- selected native or specialized test owner loads on demand
        ProjectDraftDocuments,
    )

    document = decoded(value, ProjectDraftDocuments)
    return [
        {name: bytes(content) for name, content in package.items()}
        for package in document.packages
    ]


class StructuralInputError(ValueError):
    """Safe structural facts from the published owner schema after decode refusal."""

    def __init__(
        self, locations: list[dict[str, object]], *, complete: bool = True
    ) -> None:
        super().__init__("Generated input refused")
        self.locations = locations
        self.complete = complete


def native_report(error: BaseException) -> DiagnosticReport | None:
    """Follow explicit wrapping only, without parsing rendered exception text."""
    seen: set[int] = set()
    current: BaseException | None = error
    for _ in range(16):
        if current is None or id(current) in seen:
            break
        seen.add(id(current))
        report = getattr(current, "report", None)
        if report is not None:
            return cast("DiagnosticReport", report)
        current = current.__cause__
    return None


def scientific_error(
    error: Exception, *, operation: str | None = None
) -> dict[str, object]:
    """Retain owner facts and authentic recovery references, redact supplied text."""
    report = native_report(error)
    value: dict[str, object] = {
        "cause": type(error).__name__,
        "message": "Native receiver refused the selected input; supplied values are redacted.",
        "remedy": "Correct the selected input using its published schema. Inspect original owner state before deciding whether another action is safe.",
    }
    if isinstance(error, StructuralInputError):
        value.update(
            code="pse.native.input_schema_refused",
            stage="decode",
            locations=error.locations,
            complete=error.complete,
            location_limit=16,
        )
    if report is None:
        return value
    for field in (
        "code",
        "reported_code",
        "failure_class",
        "stage",
        "rule",
        "source_refusal_proved",
    ):
        item = getattr(report, field, None)
        if item is not None:
            value[field] = item
    value["causes"] = [
        {
            "index": index,
            "message_sha256": hashlib.sha256(item.message.encode()).hexdigest(),
            "message_redacted": True,
        }
        for index, item in enumerate(report.causes[:16])
    ]
    value["locations"] = [
        {
            "source": item.source.to_hex(),
            "document": item.document.to_hex() if item.document is not None else None,
            "revision": item.revision.to_prefixed()
            if item.revision is not None
            else None,
            "start": item.start,
            "end": item.end,
        }
        for item in report.source_locations[:16]
    ]
    value["cause_count"] = len(report.causes)
    value["location_count"] = len(report.source_locations)
    value["context_count"] = len(report.contexts)
    value["related_count"] = len(report.related)
    location = getattr(report, "decode_location", None)
    if location is not None:
        from pse import codec  # noqa: PLC0415
        from scripts.capabilities.native_contracts import (  # noqa: PLC0415 -- original closed worker declarations
            worker_bindings,
        )
        from scripts.capabilities.native_schema import SchemaSet  # noqa: PLC0415

        keys: frozenset[str] = frozenset({"version", "document_version"})
        if operation is not None:
            schemas = SchemaSet(Path(__file__).resolve().parents[2])
            for binding in worker_bindings():
                if binding.operation == operation:
                    for slug in binding.documents.values():
                        keys |= schema_keys(schemas.document(slug))
        projected = codec.document_builtins(location)
        if isinstance(projected, dict):
            # Native maps/enums may be keyed by supplied values. Only properties
            # of the actual selected owner schemas are safe public map keys.
            path = projected.get("path", [])
            if isinstance(path, list):
                for segment in path:
                    if isinstance(segment, dict):
                        if "key" in segment and segment["key"] not in keys:
                            segment["key"] = "<redacted-key>"
                        if "variant" in segment:
                            segment["variant"] = "<redacted-variant>"
            value["decode_location"] = projected
    value["contexts"] = [
        {
            "code": item.code,
            "message_redacted": True,
            "context_count": len(item.contexts),
            "annotation_count": len(item.diagnostics),
        }
        for item in report.contexts[:16]
    ]
    value["related"] = [
        {"code": item.code, "stage": item.stage, "message_redacted": True}
        for item in report.related[:8]
    ]
    recovery = {}
    for field in (
        "analysis_key",
        "artifact_key",
        "artifact_owner_kind",
        "artifact_owner",
        "artifact_read_key",
        "study_key",
        "source_problem",
        "source_operation",
    ):
        item = getattr(report, field, None)
        if (
            isinstance(item, str)
            and item
            and len(item) <= 160
            and all(c.isascii() and (c.isalnum() or c in "-_:.") for c in item)
        ):
            recovery[field] = item
    if recovery:
        value["recovery"] = recovery
        value["remedy"] = (
            "Inspect the original owner using these issued references; settle uncertain work before any repeat action. No automatic retry is authorized."
        )
    value["complete"] = (
        len(report.causes) <= 16
        and len(report.source_locations) <= 16
        and len(report.contexts) <= 16
        and len(report.related) <= 8
    )
    return value


def execute(
    operation: str, arguments: Mapping[str, object], *, scratch: Path
) -> dict[str, object]:
    from scripts import (  # noqa: PLC0415 -- selected owner loads only at this boundary
        python_extension,
    )
    from scripts.capabilities import (  # noqa: PLC0415 -- selected owner loads only at this boundary
        native_contracts as host,
    )

    supplied = dict(arguments)
    fingerprint = supplied.pop("schema_set_fingerprint", None)
    if not isinstance(fingerprint, str):
        raise TypeError("fixed native request requires its loaded schema fingerprint")
    current = None
    if os.environ.get("PSE_CAPABILITY_ADMISSION"):
        binding = next(
            (item for item in host.worker_bindings() if item.operation == operation),
            None,
        )
        requirement = (
            next(
                (
                    item
                    for item in binding.requirements
                    if item.kind == "python-extension"
                ),
                None,
            )
            if binding
            else None
        )
        if requirement is not None:
            current = python_extension.currency_check_installed(
                python_extension.bind_interpreter(os.environ),
                mode=requirement.mode or "dev",
                validated=requirement.validated is True,
            )
            if current.state != "current":
                return {
                    "outcome": "prerequisite_unavailable",
                    "code": "pse.native.artifact_not_current",
                    "currency": current.model_dump(mode="json"),
                }
            if (
                "pse._native" in sys.modules
                or current.artifact is None
                or current.artifact_stamp is None
            ):
                return {
                    "outcome": "stale_interface",
                    "code": "pse.native.unbound_loaded_artifact",
                    "remedy": "Use a fresh native worker after explicit preparation; no native receiver was entered.",
                }
    import pse  # noqa: PLC0415 -- selected owner loads only at this boundary
    from pse import (  # noqa: PLC0415 -- selected owner loads only at this boundary
        codec,
        diagnostics,
    )
    from pse.contracts import (  # noqa: PLC0415 -- selected owner loads only at this boundary
        documents as contracts,
    )

    if current is not None:
        loaded = sys.modules.get("pse._native")
        try:
            unchanged = (
                current.artifact is not None
                and loaded is not None
                and Path(str(getattr(loaded, "__file__", ""))).resolve()
                == Path(current.artifact.binary)
                and python_extension.artifact_stamp(current.artifact)
                == current.artifact_stamp
            )
        except (OSError, ValueError):
            unchanged = False
        if not unchanged:
            return {
                "outcome": "stale_interface",
                "code": "pse.native.artifact_changed_during_import",
                "remedy": "Let explicit preparation settle, then use a fresh worker; no native receiver was entered.",
            }
    if fingerprint != pse.SCHEMA_SET_FINGERPRINT:
        return {
            "outcome": "stale_interface",
            "code": "pse.native.schema_set_mismatch",
            "stage": "native_interface_selection",
            "remedy": "Explicitly prepare the compatible correctness extension with just py-sync-native-correctness, then restart or reattach this endpoint. This refusal does not build, retry or enter a native receiver.",
        }
    from scripts.telemetry_process import (  # noqa: PLC0415 -- selected owner loads only at this boundary
        activate_native,
    )

    activate_native()
    if operation == "evidence_compare":
        request = host.EvidenceCompare.model_validate(supplied)
        comparison = diagnostics.compare_evidence(
            json.dumps(request.request, allow_nan=False),
            json.dumps(request.snapshots, allow_nan=False),
            json.dumps(request.observations, allow_nan=False),
        )
        return {
            "outcome": "observed",
            "document": json.loads(codec.encode_json(comparison)),
        }
    physical_packages: list[dict[str, bytes]] = []
    packages: list[dict[str, bytes]] = []
    resources = host.Resources()
    deployment: host.Deployment | None = None
    discovery: contracts.PreparationDiscoveryRequest | None = None
    selection: contracts.ProjectSelectRequest | None = None
    metadata: contracts.ProjectMetadataRequest | None = None
    research: contracts.ResearchReadRequest | None = None
    revision: host.Revision | host.RevisionPreparation | None = None
    result: host.Result | None = None
    result_inspection: contracts.ResultInspectionRequest | None = None
    model_request: object = None
    other_physical_packages: list[dict[str, bytes]] = []
    other_packages: list[dict[str, bytes]] = []
    # Decode every generated request before Runtime construction or store access.
    if operation == "telemetry_profile_capture":
        return capture(supplied)
    if operation in {"model_declarations", "model_inspect", "model_compare"}:
        source = (host.ModelingComparison if operation == "model_compare" else host.Preparation).model_validate(supplied, strict=True)
        physical_packages = source_documents(source.physical)
        packages = source_documents(source.documents)
        resources = source.resources
        request_type = {"model_declarations": contracts.ModelingDeclarationRequest, "model_inspect": contracts.ModelingInspectionRequest, "model_compare": contracts.ModelingComparisonRequest}[operation]
        model_request = decoded(source.request, request_type)
        if isinstance(source, host.ModelingComparison):
            other_physical_packages = source_documents(source.other_physical)
            other_packages = source_documents(source.other_documents)
            if len(other_physical_packages) != 1:
                raise ValueError("comparison physical source requires one complete package")
    elif operation in {"source_admit", "preparation_discover"}:
        source = (
            host.Preparation.model_validate(supplied, strict=True)
            if operation == "preparation_discover"
            else host.Source.model_validate(supplied, strict=True)
        )
        physical_packages = source_documents(source.physical)
        packages = source_documents(source.documents)
        resources = source.resources
        if isinstance(source, host.Preparation):
            discovery = decoded(source.request, contracts.PreparationDiscoveryRequest)
    elif operation == "project_select":
        selected = host.Select.model_validate(supplied, strict=True)
        selection = decoded(selected.request, contracts.ProjectSelectRequest)
        physical_packages = source_documents(selected.physical)
        deployment = selected
    elif operation in {"revision_source", "revision_preparation_discover"}:
        revision = (
            host.RevisionPreparation.model_validate(supplied, strict=True)
            if operation == "revision_preparation_discover"
            else host.Revision.model_validate(supplied, strict=True)
        )
        physical_packages = source_documents(revision.physical)
        deployment = revision
        if isinstance(revision, host.RevisionPreparation):
            discovery = decoded(revision.request, contracts.PreparationDiscoveryRequest)
    elif operation == "project_bundle_inspect":
        bundle_request = host.Bundle.model_validate(supplied, strict=True)
        bundle = pse.ProjectBundle(
            Path(bundle_request.path),
            maximum_document_bytes=bundle_request.maximum_document_bytes,
        )
        return {
            "outcome": "observed",
            "digest": bundle.digest,
            "document": codec.document_builtins(bundle.manifest),
        }
    elif operation in {"research_read", "project_metadata"}:
        page = host.Metadata.model_validate(supplied, strict=True)
        deployment = page
        if operation == "research_read":
            research = decoded(page.request, contracts.ResearchReadRequest)
        else:
            metadata = decoded(page.request, contracts.ProjectMetadataRequest)
    elif operation == "diagnostic_processes":
        processes = host.DiagnosticProcesses.model_validate(supplied, strict=True)
        listing = decoded(processes.request, contracts.DiagnosticListRequest)
        return {
            "outcome": "observed",
            "document": codec.document_builtins(
                diagnostics.list_processes(listing, directory=processes.directory)
            ),
        }
    elif operation == "diagnostic_checkpoint":
        checkpoint = host.DiagnosticCheckpoint.model_validate(supplied, strict=True)
        expected = (
            None
            if checkpoint.expected is None
            else decoded(checkpoint.expected, contracts.CheckpointIdentity)
        )
        return {
            "outcome": "observed",
            "document": codec.document_builtins(
                diagnostics.read_checkpoint(
                    Path(checkpoint.checkpoint), expected=expected
                )
            ),
        }
    elif operation == "diagnostic_inspect":
        diagnostic = host.Diagnostic.model_validate(supplied, strict=True)
        inspection = decoded(diagnostic.request, contracts.InspectionRequest)
        return {
            "outcome": "observed",
            "document": codec.document_builtins(
                diagnostics.inspect_checkpoint(Path(diagnostic.checkpoint), inspection)
            ),
        }
    elif operation == "evidence_inspect":
        frozen = host.Evidence.model_validate(supplied, strict=True)
        frozen_inspection = decoded(frozen.request, contracts.FrozenInspectionRequest)
        evidence = diagnostics.read_evidence(Path(frozen.path))
        return {
            "outcome": "observed",
            "document": codec.document_builtins(
                diagnostics.inspect_frozen(evidence, frozen_inspection)
            ),
        }
    elif operation == "invocation_read":
        invocation = host.Invocation.model_validate(supplied, strict=True)
        return {
            "outcome": "observed",
            "document": codec.document_builtins(
                diagnostics.read_invocation(Path(invocation.path))
            ),
        }
    elif operation == "result_inspect":
        selected_result = host.ResultInspection.model_validate(supplied, strict=True)
        result_inspection = decoded(
            selected_result.request, contracts.ResultInspectionRequest
        )
        deployment = selected_result
    elif operation == "result_read":
        result = host.Result.model_validate(supplied, strict=True)
        if result.end < result.start:
            raise ValueError("reversed original result window")
        deployment = result
    else:
        raise ValueError("unknown fixed native operation")
    if (
        operation
        in {
            "source_admit",
            "model_declarations",
            "model_inspect",
            "model_compare",
            "preparation_discover",
            "project_select",
            "revision_source",
            "revision_preparation_discover",
        }
        and len(physical_packages) != 1
    ):
        raise ValueError("physical source requires one complete package")
    if deployment is not None:
        resources = deployment.resources
    spill = scratch / "spill"
    spill.mkdir()
    settings = pse.EngineSettings(
        memory_limit_bytes=resources.memory_limit_bytes,
        threads=resources.threads,
        spill_dir=str(spill),
        max_spill_bytes=resources.max_spill_bytes,
        batch_size=8192,
    )
    runtime = pse.Runtime(
        settings,
        ephemeral=deployment is None,
        substrate=deployment.state if deployment is not None else None,
        database=deployment.database if deployment is not None else None,
    )
    try:
        if operation in {"model_declarations", "model_inspect", "model_compare"}:
            physical = runtime.physical_from_documents(physical_packages[0])
            package = runtime.modeling_from_documents(packages, physical)
            if operation == "model_declarations":
                observed = package.declaration_page(cast("contracts.ModelingDeclarationRequest", model_request))
            elif operation == "model_inspect":
                observed = package.inspect(cast("contracts.ModelingInspectionRequest", model_request))
            else:
                other_physical = runtime.physical_from_documents(other_physical_packages[0])
                other = runtime.modeling_from_documents(other_packages, other_physical)
                observed = package.comparison_observation(other, cast("contracts.ModelingComparisonRequest", model_request))
            return {"outcome": "observed", "document": codec.document_builtins(observed)}
        if operation in {"source_admit", "preparation_discover"}:
            physical = runtime.physical_from_documents(physical_packages[0])
            package = runtime.modeling_from_documents(packages, physical)
            response: dict[str, object] = {
                "outcome": "admitted",
                "revision": {
                    "identity": package.revision_id,
                    "lifetime": "completed_child",
                    "reopenable": False,
                },
                "durable": False,
                "source": codec.document_builtins(package.operation_source),
            }
            if discovery is not None:
                response["preparation"] = codec.document_builtins(
                    package.preparation_discovery(discovery)
                )
            return response
        if selection is not None:
            physical = runtime.physical_from_documents(physical_packages[0])
            return {
                "outcome": "selected",
                "document": codec.document_builtins(
                    runtime.project_select(selection, physical)
                ),
            }
        if revision is not None:
            physical = runtime.physical_from_documents(physical_packages[0])
            package = runtime.modeling_revision(revision.revision, physical)
            if discovery is not None:
                return {
                    "outcome": "observed",
                    "document": codec.document_builtins(
                        package.preparation_discovery(discovery)
                    ),
                }
            if isinstance(revision, host.Revision):
                return {
                    "outcome": "observed",
                    "canonical_revision": package.canonical_revision,
                    "source": [
                        codec.document_builtins(item)
                        for item in package.project_source_detail(revision.logical)
                    ],
                }
        if research is not None:
            return {
                "outcome": "observed",
                "document": codec.document_builtins(runtime.research_read(research)),
            }
        if metadata is not None:
            return {
                "outcome": "observed",
                "document": codec.document_builtins(runtime.project_metadata(metadata)),
            }
        if result_inspection is not None:
            return {
                "outcome": "observed",
                "document": codec.document_builtins(
                    runtime.inspect_result(result_inspection)
                ),
            }
        if result is not None:
            return read_result(runtime, result)
    finally:
        runtime.close()
    raise ValueError("native operation did not return an outcome")


def read_result(runtime: Runtime, request: Result) -> dict[str, object]:
    """Copy a bounded Arrow window while its protected native reader is alive."""
    import pyarrow as pa  # noqa: PLC0415 -- load native/public owners only inside an actual call

    stream = runtime.results(
        request.run,
        request.attempt,
        request.relation,
        start=request.start,
        end=request.end,
    )
    with stream:
        schema = pa.schema(stream)
        sink = pa.BufferOutputStream()
        rows = 0
        with pa.ipc.new_stream(sink, schema) as writer:
            for batch in stream:
                if (
                    batch.nbytes > request.maximum_bytes
                    or sink.tell() + batch.nbytes + 4096 > request.maximum_bytes
                ):
                    return {
                        "outcome": "incomplete_observation",
                        "complete": False,
                        "omissions": [
                            "Native result exceeds the selected detail bound; narrow the row window."
                        ],
                        "code": "pse.native.result_detail_bound",
                        "rows_copied": 0,
                        "message": "Narrow the original row window or increase maximum_bytes.",
                    }
                writer.write_batch(batch)
                rows += batch.num_rows
        data = sink.getvalue()
        if len(data) > request.maximum_bytes:
            return {
                "outcome": "incomplete_observation",
                "complete": False,
                "omissions": [
                    "Native result exceeds the selected detail bound; narrow the row window."
                ],
                "code": "pse.native.result_detail_bound",
                "rows_copied": 0,
            }
        encoded = base64.b64encode(data).decode("ascii")
        return {
            "outcome": "observed",
            "run": request.run,
            "attempt": request.attempt,
            "relation": request.relation,
            "start": request.start,
            "end": request.end,
            "rows": rows,
            "encoding": "arrow-ipc-stream-base64",
            "bytes": len(data),
            "chunks": [
                encoded[offset : offset + 8192]
                for offset in range(0, len(encoded), 8192)
            ],
        }


def worker_request(selected: list[str]) -> tuple[str, dict[str, JsonValue]]:
    if selected:
        if len(selected) != 3 or selected[0] not in WORKERS["native"].operations:
            raise ValueError(
                "select one fixed native action, request and response file"
            )
        operation, source, _target = selected
        with Path(source).open("rb") as input_file:
            request_bytes = input_file.read(MAXIMUM_REQUEST_BYTES + 1)
        if len(request_bytes) > MAXIMUM_REQUEST_BYTES:
            raise ValueError("native request exceeds transport byte bound")
        from scripts.capabilities.execution_basis import (  # noqa: PLC0415 -- selected owner loads only at this boundary
            verify,
        )

        verify(Path(__file__).resolve().parents[2], "native", operation, request_bytes)
        arguments = JSON_OBJECT.validate_json(request_bytes, strict=True)
        return operation, arguments
    raw = sys.stdin.buffer.read(MAXIMUM_REQUEST_BYTES + 1)
    if len(raw) > MAXIMUM_REQUEST_BYTES:
        raise ValueError("native request exceeds transport byte bound")
    payload = JSON_OBJECT.validate_json(raw, strict=True)
    if set(payload) != {"operation", "arguments"}:
        raise ValueError("fixed worker envelope requires operation and arguments")
    operation, arguments = payload["operation"], payload["arguments"]
    if not isinstance(operation, str) or not isinstance(arguments, dict):
        raise TypeError("invalid fixed worker envelope")
    from scripts.capabilities.execution_basis import (  # noqa: PLC0415 -- selected owner loads only at this boundary
        verify,
    )

    verify(Path(__file__).resolve().parents[2], "native", operation, raw)
    return operation, arguments


def main(argv: list[str] | None = None) -> int:
    from scripts.capabilities.native_contracts import EXACT_READS  # noqa: PLC0415

    selected = sys.argv[1:] if argv is None else argv
    operation = "unknown"
    destination = (
        Path(selected[2])
        if len(selected) == 3 and selected[0] in WORKERS["native"].operations
        else None
    )
    try:
        operation, arguments = worker_request(selected)
        progress = (
            ProgressReporter(Path(selected[1]).parent / "progress.json", operation)
            if destination is not None
            else None
        )
        if progress is not None:
            progress.publish("native", "started")
        with tempfile.TemporaryDirectory(prefix="pse-capability-native-") as directory:
            value = execute(operation, arguments, scratch=Path(directory))
        if progress is not None:
            progress.publish("native", "completed")
    except (ImportError, ModuleNotFoundError) as error:
        value = {
            "outcome": "prerequisite_unavailable",
            "code": "pse.native.extension_unavailable",
            "cause": type(error).__name__,
            "remedy": "Explicitly prepare the compatible extension with just py-sync-native-correctness and reattach from a prepared native environment. Read operations never build import providers.",
        }
    except AdmissionRefusedError as error:
        value = {"outcome": "stale_interface", **error.detail()}
    except Exception as error:
        native = native_report(error) is not None
        outcome = (
            "native_refusal"
            if native
            else (
                "malformed_invocation"
                if isinstance(error, (ValueError, TypeError, msgspec.DecodeError))
                else "unexpected_error"
            )
        )
        value = {
            "outcome": outcome,
            "error": scientific_error(error, operation=operation),
        }
    try:
        encoded = json.dumps(value, allow_nan=False)
    except (TypeError, ValueError):
        encoded = json.dumps(
            {
                "outcome": "schema_failure",
                "code": "pse.native.non_json_producer_output",
                "message": "Native output was not an admitted JSON value; no object stringification or nonfinite normalization was applied.",
            }
        )
    if operation in EXACT_READS:
        from scripts.capabilities.native import intact_result  # noqa: PLC0415

        value = intact_result(
            JSON_OBJECT.validate_json(encoded), "native_" + operation
        ).payload
    else:
        projection = bounded(json.loads(encoded), "native_" + operation)
        value = projection.payload
        if projection.omissions:
            value["complete"] = False
            value["omissions"] = list[JsonValue](projection.omissions)
    if destination is None:
        print(json.dumps(value, allow_nan=False))
    else:
        with destination.open("x") as output:
            json.dump({"result": value}, output, allow_nan=False)
    return 0


if __name__ == "__main__":
    from scripts.telemetry_process import run_helper

    raise SystemExit(run_helper("pse-native-worker", main))
