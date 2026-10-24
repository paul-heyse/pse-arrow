# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Fixed native operations. This child alone imports the public scientific facade."""

from __future__ import annotations

import json
import sys
import tempfile
from pathlib import Path
from typing import TYPE_CHECKING, TypeVar, cast

if TYPE_CHECKING:
    from pse import DiagnosticReport, Runtime
    from scripts.capabilities.native import Result

T = TypeVar("T")


MAXIMUM_REQUEST_BYTES = 16 << 20


def decoded(value: object, document_type: type[T]) -> T:
    from pse import codec

    return codec.decode_json(json.dumps(value, allow_nan=False), document_type)


def source_documents(value: object) -> list[dict[str, bytes]]:
    from pse.contracts.documents import ProjectDraftDocuments

    document = decoded(value, ProjectDraftDocuments)
    return [
        {name: bytes(content) for name, content in package.items()}
        for package in document.packages
    ]


def scientific_error(error: Exception) -> dict[str, object]:
    """Keep native classifications and locations, never echo supplied source text."""
    import hashlib

    report = cast("DiagnosticReport | None", getattr(error, "report", None))
    value: dict[str, object] = {
        "cause": type(error).__name__,
        "message": "Native receiver refused the selected input; supplied values are redacted.",
    }
    if report is not None:
        for field in ("code", "reported_code", "failure_class", "stage", "rule", "source_refusal_proved"):
            item = getattr(report, field, None)
            if item is not None:
                value[field] = item
        value["causes"] = [
            {"index": index, "message_sha256": hashlib.sha256(item.message.encode()).hexdigest(), "message_redacted": True}
            for index, item in enumerate(report.causes[:16])
        ]
        value["locations"] = [
            {"source": item.source.to_hex(), "document": item.document.to_hex() if item.document is not None else None, "revision": item.revision.to_prefixed() if item.revision is not None else None, "start": item.start, "end": item.end}
            for item in report.source_locations[:16]
        ]
        value["cause_count"] = len(report.causes)
        value["location_count"] = len(report.source_locations)
        if len(report.causes) > 16 or len(report.source_locations) > 16:
            value["complete"] = False
            value["omissions"] = ["Native error cause/location detail is limited to the first 16 entries; supplied messages remain redacted."]
    else:
        # msgspec locations do not require returning its offending value/message.
        import re

        location = re.search(r"at `([^`]+)`", str(error))
        if location:
            value["location"] = location.group(1)
    return value


def execute(operation: str, arguments: dict[str, object], *, scratch: Path) -> dict[str, object]:
    import pse
    from pse import codec, diagnostics
    from pse.contracts import documents as contracts
    from scripts.capabilities import native as host

    supplied = dict(arguments)
    fingerprint = supplied.pop("schema_set_fingerprint", None)
    if not isinstance(fingerprint, str):
        raise ValueError("fixed native request requires its loaded schema fingerprint")
    if fingerprint != pse.SCHEMA_SET_FINGERPRINT:
        return {"outcome": "stale_interface", "code": "pse.native.schema_set_mismatch"}
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
    # Decode every generated request before Runtime construction or store access.
    if operation in {"source_admit", "preparation_discover"}:
        source = host.Preparation.model_validate(supplied, strict=True) if operation == "preparation_discover" else host.Source.model_validate(supplied, strict=True)
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
        revision = host.RevisionPreparation.model_validate(supplied, strict=True) if operation == "revision_preparation_discover" else host.Revision.model_validate(supplied, strict=True)
        physical_packages = source_documents(revision.physical)
        deployment = revision
        if isinstance(revision, host.RevisionPreparation):
            discovery = decoded(revision.request, contracts.PreparationDiscoveryRequest)
    elif operation == "project_bundle_inspect":
        bundle_request = host.Bundle.model_validate(supplied, strict=True)
        bundle = pse.ProjectBundle(Path(bundle_request.path), maximum_document_bytes=bundle_request.maximum_document_bytes)
        return {"outcome": "observed", "digest": bundle.digest, "document": codec.document_builtins(bundle.manifest)}
    elif operation in {"research_read", "project_metadata"}:
        page = host.Metadata.model_validate(supplied, strict=True)
        deployment = page
        if operation == "research_read":
            research = decoded(page.request, contracts.ResearchReadRequest)
        else:
            metadata = decoded(page.request, contracts.ProjectMetadataRequest)
    elif operation == "diagnostic_inspect":
        diagnostic = host.Diagnostic.model_validate(supplied, strict=True)
        inspection = decoded(diagnostic.request, contracts.InspectionRequest)
        return {"outcome": "observed", "document": codec.document_builtins(diagnostics.inspect_checkpoint(Path(diagnostic.checkpoint), inspection))}
    elif operation == "evidence_inspect":
        frozen = host.Evidence.model_validate(supplied, strict=True)
        frozen_inspection = decoded(frozen.request, contracts.FrozenInspectionRequest)
        evidence = diagnostics.read_evidence(Path(frozen.path))
        return {"outcome": "observed", "document": codec.document_builtins(diagnostics.inspect_frozen(evidence, frozen_inspection))}
    elif operation == "invocation_read":
        invocation = host.Invocation.model_validate(supplied, strict=True)
        return {"outcome": "observed", "document": codec.document_builtins(diagnostics.read_invocation(Path(invocation.path)))}
    elif operation == "result_read":
        result = host.Result.model_validate(supplied, strict=True)
        if result.end < result.start:
            raise ValueError("reversed original result window")
        deployment = result
    else:
        raise ValueError("unknown fixed native operation")
    if operation in {"source_admit", "preparation_discover", "project_select", "revision_source", "revision_preparation_discover"} and len(physical_packages) != 1:
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
        if operation in {"source_admit", "preparation_discover"}:
            physical = runtime.physical_from_documents(physical_packages[0])
            package = runtime.modeling_from_documents(packages, physical)
            response: dict[str, object] = {
                "outcome": "admitted",
                "revision": {"identity": package.revision_id, "lifetime": "completed_child", "reopenable": False},
                "durable": False,
                "source": codec.document_builtins(package.operation_source),
            }
            if discovery is not None:
                response["preparation"] = codec.document_builtins(package.preparation_discovery(discovery))
            return response
        if selection is not None:
            physical = runtime.physical_from_documents(physical_packages[0])
            return {"outcome": "selected", "document": codec.document_builtins(runtime.project_select(selection, physical))}
        if revision is not None:
            physical = runtime.physical_from_documents(physical_packages[0])
            package = runtime.modeling_revision(revision.revision, physical)
            if discovery is not None:
                return {"outcome": "observed", "document": codec.document_builtins(package.preparation_discovery(discovery))}
            if isinstance(revision, host.Revision):
                return {"outcome": "observed", "canonical_revision": package.canonical_revision, "source": [codec.document_builtins(item) for item in package.project_source_detail(revision.logical)]}
        if research is not None:
            return {"outcome": "observed", "document": codec.document_builtins(runtime.research_read(research))}
        if metadata is not None:
            return {"outcome": "observed", "document": codec.document_builtins(runtime.project_metadata(metadata))}
        if result is not None:
            return read_result(runtime, result)
    finally:
        runtime.close()
    raise ValueError("native operation did not return an outcome")


def read_result(runtime: Runtime, request: Result) -> dict[str, object]:
    """Copy a bounded Arrow window while its protected native reader is alive."""
    import base64

    import pyarrow as pa

    stream = runtime.results(request.run, request.attempt, request.relation, start=request.start, end=request.end)
    with stream:
        schema = pa.schema(stream)
        sink = pa.BufferOutputStream()
        rows = 0
        with pa.ipc.new_stream(sink, schema) as writer:
            for batch in stream:
                if batch.nbytes > request.maximum_bytes or sink.tell() + batch.nbytes + 4096 > request.maximum_bytes:
                    return {"outcome": "incomplete_observation", "complete": False, "omissions": ["Native result exceeds the selected detail bound; narrow the row window."], "code": "pse.native.result_detail_bound", "rows_copied": 0, "message": "Narrow the original row window or increase maximum_bytes."}
                writer.write_batch(batch)
                rows += batch.num_rows
        data = sink.getvalue()
        if len(data) > request.maximum_bytes:
            return {"outcome": "incomplete_observation", "complete": False, "omissions": ["Native result exceeds the selected detail bound; narrow the row window."], "code": "pse.native.result_detail_bound", "rows_copied": 0}
        encoded = base64.b64encode(data).decode("ascii")
        return {"outcome": "observed", "run": request.run, "attempt": request.attempt, "relation": request.relation, "start": request.start, "end": request.end, "rows": rows, "encoding": "arrow-ipc-stream-base64", "bytes": len(data), "chunks": [encoded[offset:offset + 8192] for offset in range(0, len(encoded), 8192)]}


def main(argv: list[str] | None = None) -> int:
    selected = sys.argv[1:] if argv is None else argv
    operation = "unknown"
    destination: Path | None = None
    try:
        if selected:
            if len(selected) != 3 or selected[0] not in {"source_admit", "preparation_discover"}:
                raise ValueError("select one fixed native action, request and response file")
            operation, source, target = selected
            destination = Path(target)
            with Path(source).open("rb") as input_file:
                request_bytes = input_file.read(MAXIMUM_REQUEST_BYTES + 1)
            if len(request_bytes) > MAXIMUM_REQUEST_BYTES:
                raise ValueError("native request exceeds transport byte bound")
            raw = json.dumps({"operation": operation, "arguments": json.loads(request_bytes)}).encode()
        else:
            raw = sys.stdin.buffer.read(MAXIMUM_REQUEST_BYTES + 1)
        if len(raw) > MAXIMUM_REQUEST_BYTES:
            raise ValueError("native request exceeds transport byte bound")
        payload = json.loads(raw)
        if not isinstance(payload, dict) or set(payload) != {"operation", "arguments"}:
            raise ValueError("fixed worker envelope requires operation and arguments")
        operation, arguments = payload["operation"], payload["arguments"]
        if not isinstance(operation, str) or not isinstance(arguments, dict):
            operation = "unknown"
            raise ValueError("invalid fixed worker envelope")
        with tempfile.TemporaryDirectory(prefix="pse-capability-native-") as directory:
            value = execute(operation, arguments, scratch=Path(directory))
    except (ImportError, ModuleNotFoundError) as error:
        value = {"outcome": "prerequisite_unavailable", "code": "pse.native.extension_unavailable", "cause": type(error).__name__, "remedy": "Explicitly prepare the compatible extension with just py-sync-native-correctness and reattach from a prepared native environment. Read operations never build import providers."}
    except Exception as error:
        import msgspec

        native = getattr(error, "report", None) is not None
        outcome = "native_refusal" if native else ("malformed_invocation" if isinstance(error, (ValueError, TypeError, msgspec.DecodeError)) else "unexpected_error")
        value = {"outcome": outcome, "error": scientific_error(error)}
    from scripts.capabilities.operations import bounded

    try:
        encoded = json.dumps(value, allow_nan=False)
    except (TypeError, ValueError):
        encoded = json.dumps({"outcome": "schema_failure", "code": "pse.native.non_json_producer_output", "message": "Native output was not an admitted JSON value; no object stringification or nonfinite normalization was applied."})
    projection = bounded(json.loads(encoded), "native_" + operation)
    value = projection.payload
    if projection.omissions:
        value["complete"] = False
        value["omissions"] = projection.omissions
    if destination is None:
        print(json.dumps(value, allow_nan=False))
    else:
        with destination.open("x") as output:
            json.dump({"result": value}, output, allow_nan=False)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
