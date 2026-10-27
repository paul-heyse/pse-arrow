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
from scripts.capabilities.native_extension import (
    ExtensionSelectionError,
    extension_currency,
)
from scripts.capabilities.native_schema import JSON_OBJECT
from scripts.capabilities.responses import bounded
from scripts.capabilities.telemetry_profile_worker import capture
from scripts.capabilities.worker_progress import ProgressReporter
from scripts.capabilities.workers import WORKERS

if TYPE_CHECKING:
    from collections.abc import Mapping

    from jsonschema.exceptions import ValidationError as SchemaValidationError

    from pse import DiagnosticReport, ModelingPackage, PhysicalContext, Runtime
    from scripts.capabilities.native_contracts import Result
    from scripts.capabilities.native_read_context import NativeReadContext

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


def failure_remedy(boundary_class: object, failure_class: object) -> str:
    """Describe the typed failure without inferring effect absence or replay permission."""
    selected = boundary_class if isinstance(boundary_class, str) else failure_class
    if not isinstance(selected, str):
        return "Inspect original owner state and the typed failure before deciding whether another action is safe. No automatic retry is authorized."
    if selected in {
        "resource_limit",
        "runtime.resource_limit",
        "runtime.timeout",
        "solve.limit",
    }:
        return "Inspect original owner state and the operation's resource, effort and deadline allowances before selecting another action. No automatic retry is authorized."
    if selected in {"cancelled", "runtime.cancelled"}:
        return "Inspect original owner state and confirm that admitted work has drained before selecting another action. Cancellation does not prove that effects are absent."
    if selected in {
        "internal",
        "internal.invariant",
        "infrastructure",
        "runtime.infrastructure",
    }:
        return "Inspect original owner state and the retained failure diagnostics; establish effect and integrity state before selecting another action. Diagnose the owning service before changing supplied input."
    if selected in {"unsupported", "capability.backend"}:
        return "Inspect the selected operation's published capabilities and configuration. Preserve the original operation and settle any effects before selecting another action."
    if selected in {
        "trial_rejected",
        "nonfinite",
        "numerical",
        "inconclusive",
        "solve.evaluation_error",
        "solve.solver_error",
    }:
        return "Inspect original owner state, native observations and original scientific assessments. Retained diagnostic values do not grant scientific permission or automatic retry."
    if selected in {
        "invalid_model",
        "incompatible",
        "input",
        "authoring.parse",
        "authoring.reference",
        "compile.math",
        "config.invalid",
        "user.model",
    }:
        return "Correct the selected input using its published schema and attributed scientific requirements. Inspect original owner state before deciding whether another action is safe."
    return "Inspect original owner state and the typed failure before deciding whether another action is safe. No automatic retry is authorized."


def scientific_error(
    error: Exception, *, operation: str | None = None
) -> dict[str, object]:
    """Retain owner facts and authentic recovery references, redact supplied text."""
    report = native_report(error)
    value: dict[str, object] = {
        "cause": type(error).__name__,
        "message": "Native operation failed; supplied values and private diagnostics are redacted.",
        "remedy": "Inspect original owner state and the typed failure before deciding whether another action is safe. No automatic retry is authorized.",
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
        "boundary_class",
        "severity",
        "stage",
        "rule",
        "source_refusal_proved",
    ):
        item = getattr(report, field, None)
        if item is not None:
            value[field] = item
    value["remedy"] = failure_remedy(
        value.get("boundary_class"), value.get("failure_class")
    )
    observations = getattr(report, "observations", None)
    if isinstance(observations, dict):
        effect = getattr(observations.get("admission_effect"), "value", None)
        if isinstance(effect, str) and effect in {
            "no_publication",
            "source_acknowledged",
            "unresolved_publication",
        }:
            value["admission_effect"] = effect
        integrity = getattr(observations.get("integrity_failure"), "value", None)
        if integrity is True:
            value["integrity_failure"] = True
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
    if value.get("admission_effect") == "source_acknowledged" and isinstance(
        observations, dict
    ):
        revision = getattr(observations.get("source_revision"), "value", None)
        if (
            isinstance(revision, str)
            and revision
            and len(revision) <= 160
            and all(c.isascii() and (c.isalnum() or c in "-_:") for c in revision)
        ):
            recovery["source_revision"] = revision
    if recovery:
        value["recovery"] = recovery
        value["remedy"] = (
            value["remedy"]
            + " Inspect the original owner using these issued references; settle uncertain work before any repeat action. No automatic retry is authorized."
        )
    if value.get("admission_effect") == "unresolved_publication":
        value["remedy"] = (
            value["remedy"]
            + " Read back the original publication occurrence; its outcome remains unresolved."
        )
    value["complete"] = (
        len(report.causes) <= 16
        and len(report.source_locations) <= 16
        and len(report.contexts) <= 16
        and len(report.related) <= 8
    )
    return value


def _source_identity(packages: list[dict[str, bytes]]) -> str:
    """Hash bounded original bytes; package order and document names retain meaning."""
    digest = hashlib.sha256()
    for package in packages:
        digest.update(len(package).to_bytes(8, "big"))
        for name, content in sorted(package.items()):
            encoded = name.encode()
            digest.update(len(encoded).to_bytes(8, "big"))
            digest.update(encoded)
            digest.update(len(content).to_bytes(8, "big"))
            digest.update(content)
    return digest.hexdigest()


def execute(
    operation: str,
    arguments: Mapping[str, object],
    *,
    scratch: Path,
    context: NativeReadContext | None = None,
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
    header = host.validate_arguments(operation, supplied)
    if (
        context is not None
        and host.binding_for(operation).read_eligibility != "exact_context"
    ):
        raise ValueError("operation does not declare retained read eligibility")
    current = None
    if os.environ.get("PSE_CAPABILITY_ADMISSION"):
        binding = next(
            (item for item in host.worker_bindings() if item.operation == operation),
            None,
        )
        requirement = binding.extension_requirement if binding is not None else None
        if requirement is not None:
            try:
                current = extension_currency(
                    python_extension.bind_interpreter(os.environ),
                    requirement.mode,
                    validated=requirement.validated,
                )
            except ExtensionSelectionError as error:
                return dict(error.detail())
            if current.state != "current":
                return {
                    "outcome": "prerequisite_unavailable",
                    "code": "pse.native.artifact_not_current",
                    "currency": current.model_dump(mode="json"),
                }
            from scripts.capabilities.native_read_image import (  # noqa: PLC0415
                LoadedReadImage,
                ReadImageChangedError,
            )

            try:
                LoadedReadImage.admit(current).attach()
            except ReadImageChangedError:
                return {
                    "outcome": "stale_interface",
                    "code": "pse.native.unbound_loaded_artifact",
                    "remedy": "Drain the original owner and attach a fresh worker; no receiver was entered.",
                }
    import pse  # noqa: PLC0415 -- selected owner loads only at this boundary
    from pse import (  # noqa: PLC0415 -- selected owner loads only at this boundary
        codec,
        diagnostics,
    )
    from pse.contracts import (  # noqa: PLC0415 -- selected owner loads only at this boundary
        documents as contracts,
    )

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

    if (
        os.environ.get("PSE_CAPABILITY_ADMISSION") or context is not None
    ) and (context is None or not context.interface_compatible):
        if JSON_OBJECT.validate_json(
            codec.encode_json(pse.host_operation_catalogue())
        ) != host.native_catalogue():
            return {
                "outcome": "stale_interface",
                "code": "pse.native.operation_catalogue_mismatch",
                "remedy": "Select a compatible publication and reattach the endpoint; no receiver was entered.",
            }
        if context is not None:
            context.interface_compatible = True
    activate_native()
    physical_packages: list[dict[str, bytes]] = []
    packages: list[dict[str, bytes]] = []
    resources: host.Resources | None = None
    configured: host.ConfiguredCapabilities | None = None
    capability_settings: contracts.BackendSettings | None = None
    deployment: host.Deployment | None = None
    discovery: contracts.PreparationDiscoveryRequest | None = None
    selection: contracts.ProjectSelectRequest | None = None
    metadata: contracts.ProjectMetadataRequest | None = None
    research: contracts.ResearchReadRequest | None = None
    revision: host.Revision | host.RevisionPreparation | None = None
    result: host.Result | None = None
    result_inspection: contracts.ResultInspectionRequest | None = None
    physical_inspection: contracts.PhysicalInspectionRequest | None = None
    model_request: object = None
    other_physical_packages: list[dict[str, bytes]] = []
    other_packages: list[dict[str, bytes]] = []
    # Decode every generated request before Runtime construction or store access.
    if operation == "telemetry_profile_capture":
        return capture(supplied)
    if operation in {"model_declarations", "model_inspect", "model_compare"}:
        source = cast("host.ModelingComparison | host.Preparation", header)
        physical_packages = source_documents(source.physical)
        packages = source_documents(source.documents)
        resources = source.resources
        request_type = host.selected_document_type(operation, "request")
        model_request = decoded(source.request, request_type)
        if isinstance(source, host.ModelingComparison):
            other_physical_packages = source_documents(source.other_physical)
            other_packages = source_documents(source.other_documents)
            if len(other_physical_packages) != 1:
                raise ValueError(
                    "comparison physical source requires one complete package"
                )
    elif operation in {"source_admit", "preparation_discover"}:
        source = cast("host.Preparation | host.Source", header)
        physical_packages = source_documents(source.physical)
        packages = source_documents(source.documents)
        resources = source.resources
        if isinstance(source, host.Preparation):
            discovery = decoded(
                source.request,
                host.document_type(
                    operation, "request", contracts.PreparationDiscoveryRequest
                ),
            )
    elif operation == "project_select":
        selected = cast("host.Select", header)
        selection = decoded(
            selected.request,
            host.document_type(operation, "request", contracts.ProjectSelectRequest),
        )
        physical_packages = source_documents(selected.physical)
        deployment = selected
    elif operation in {"revision_source", "revision_preparation_discover"}:
        revision = cast("host.RevisionPreparation | host.Revision", header)
        physical_packages = source_documents(revision.physical)
        deployment = revision
        if isinstance(revision, host.RevisionPreparation):
            discovery = decoded(
                revision.request,
                host.document_type(
                    operation, "request", contracts.PreparationDiscoveryRequest
                ),
            )
    elif operation == "project_bundle_inspect":
        bundle_request = cast("host.Bundle", header)
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
        page = cast("host.Metadata", header)
        deployment = page
        if operation == "research_read":
            research = decoded(
                page.request,
                host.document_type(operation, "request", contracts.ResearchReadRequest),
            )
        else:
            metadata = decoded(
                page.request,
                host.document_type(
                    operation, "request", contracts.ProjectMetadataRequest
                ),
            )
    elif operation == "diagnostic_processes":
        processes = cast("host.DiagnosticProcesses", header)
        listing = decoded(
            processes.request,
            host.document_type(operation, "request", contracts.DiagnosticListRequest),
        )
        return {
            "outcome": "observed",
            "document": codec.document_builtins(
                diagnostics.list_processes(listing, directory=processes.directory)
            ),
        }
    elif operation == "diagnostic_checkpoint":
        checkpoint = cast("host.DiagnosticCheckpoint", header)
        expected = (
            None
            if checkpoint.expected is None
            else decoded(
                checkpoint.expected,
                host.document_type(operation, "expected", contracts.CheckpointIdentity),
            )
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
        diagnostic = cast("host.Diagnostic", header)
        inspection = decoded(
            diagnostic.request,
            host.document_type(operation, "request", contracts.InspectionRequest),
        )
        return {
            "outcome": "observed",
            "document": codec.document_builtins(
                diagnostics.inspect_checkpoint(Path(diagnostic.checkpoint), inspection)
            ),
        }
    elif operation == "evidence_inspect":
        frozen = cast("host.Evidence", header)
        frozen_inspection = decoded(
            frozen.request,
            host.document_type(operation, "request", contracts.FrozenInspectionRequest),
        )
        evidence = diagnostics.read_evidence(Path(frozen.path))
        return {
            "outcome": "observed",
            "document": codec.document_builtins(
                diagnostics.inspect_frozen(evidence, frozen_inspection)
            ),
        }
    elif operation == "invocation_read":
        invocation = cast("host.Invocation", header)
        return {
            "outcome": "observed",
            "document": codec.document_builtins(
                diagnostics.read_invocation(Path(invocation.path))
            ),
        }
    elif operation == "physical_inspect":
        selected_physical = cast("host.PhysicalInspection", header)
        physical_packages = source_documents(selected_physical.physical)
        physical_inspection = decoded(
            selected_physical.request,
            host.document_type(
                operation, "request", contracts.PhysicalInspectionRequest
            ),
        )
        resources = selected_physical.resources
    elif operation == "result_inspect":
        selected_result = cast("host.ResultInspection", header)
        result_inspection = decoded(
            selected_result.request,
            host.document_type(operation, "request", contracts.ResultInspectionRequest),
        )
        deployment = selected_result
    elif operation == "configured_capabilities":
        configured = cast("host.ConfiguredCapabilities", header)
        resources = configured.resources
        if configured.settings is not None:
            if configured.backend is None:
                raise ValueError(
                    "configured settings require an explicitly selected backend"
                )
            capability_settings = decoded(
                configured.settings,
                host.document_type(operation, "settings", contracts.BackendSettings),
            )
    elif operation == "result_read":
        result = cast("host.Result", header)
        if result.end < result.start:
            raise ValueError("reversed original result window")
        deployment = result
    else:
        raise ValueError("unknown fixed native operation")
    if (
        operation
        in {
            "source_admit",
            "physical_inspect",
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
    if resources is None:
        raise TypeError("runtime receiver requires its declared resource header")
    spill = scratch / "spill"
    spill.mkdir(exist_ok=context is not None)
    settings = pse.EngineSettings(
        memory_limit_bytes=resources.memory_limit_bytes,
        threads=resources.threads,
        spill_dir=str(spill),
        max_spill_bytes=resources.max_spill_bytes,
        batch_size=host.generated.DEFAULT_BATCH_SIZE,
    )

    def create_runtime() -> Runtime:
        return pse.Runtime(
            settings,
            ephemeral=deployment is None,
            substrate=deployment.state if deployment is not None else None,
            database=deployment.database if deployment is not None else None,
        )

    runtime_identity = hashlib.sha256(
        json.dumps(
            {
                "resources": resources.model_dump(mode="json"),
                "state": deployment.state if deployment is not None else None,
                "database": deployment.database if deployment is not None else None,
            },
            sort_keys=True,
            allow_nan=False,
        ).encode()
    ).hexdigest()
    runtime = (
        create_runtime()
        if context is None
        else context.runtime(
            runtime_identity, create_runtime, lambda owned: owned.close()
        )
    )

    def retained_physical(document: dict[str, bytes]) -> PhysicalContext:
        def create() -> PhysicalContext:
            return runtime.physical_from_documents(document)

        return (
            create()
            if context is None
            else context.physical(_source_identity([document]), create)
        )

    def modeling(
        documents: list[dict[str, bytes]],
        original_physical: dict[str, bytes],
        admitted_physical: PhysicalContext,
    ) -> ModelingPackage:
        def create() -> ModelingPackage:
            return runtime.modeling_from_documents(documents, admitted_physical)

        identity = (
            _source_identity([original_physical]) + ":" + _source_identity(documents)
        )
        return create() if context is None else context.modeling(identity, create)

    try:
        if configured is not None:
            if configured.backend is None:
                rows = runtime.capabilities()
            else:
                from pse.contracts.enums import (  # noqa: PLC0415
                    NativeBackend,
                )

                rows = (
                    runtime.capability_for(
                        NativeBackend(configured.backend), settings=capability_settings
                    ),
                )
            return {
                "outcome": "observed",
                "document": [codec.document_builtins(row) for row in rows],
            }
        if physical_inspection is not None:
            physical = retained_physical(physical_packages[0])
            return {
                "outcome": "observed",
                "document": codec.document_builtins(
                    physical.describe(physical_inspection)
                ),
            }
        if operation in {"model_declarations", "model_inspect", "model_compare"}:
            physical = retained_physical(physical_packages[0])
            package = modeling(packages, physical_packages[0], physical)
            if operation == "model_declarations":
                observed = package.declaration_page(
                    cast("contracts.ModelingDeclarationRequest", model_request)
                )
            elif operation == "model_inspect":
                observed = package.inspect(
                    cast("contracts.ModelingInspectionRequest", model_request)
                )
            else:
                other_physical = retained_physical(other_physical_packages[0])
                other = modeling(
                    other_packages, other_physical_packages[0], other_physical
                )
                observed = package.comparison_observation(
                    other, cast("contracts.ModelingComparisonRequest", model_request)
                )
            return {
                "outcome": "observed",
                "document": codec.document_builtins(observed),
            }
        if operation in {"source_admit", "preparation_discover"}:
            physical = retained_physical(physical_packages[0])
            package = modeling(packages, physical_packages[0], physical)
            response: dict[str, object] = {
                "outcome": "admitted",
                "revision": {
                    "identity": package.revision_id,
                    "lifetime": "completed_child"
                    if context is None
                    else "foreground_session",
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
            physical = retained_physical(physical_packages[0])
            return {
                "outcome": "selected",
                "document": codec.document_builtins(
                    runtime.project_select(selection, physical)
                ),
            }
        if revision is not None:
            physical = retained_physical(physical_packages[0])
            revision_id = revision.revision

            def create_revision() -> ModelingPackage:
                return runtime.modeling_revision(revision_id, physical)

            package = (
                create_revision()
                if context is None
                else context.modeling(
                    "revision:"
                    + revision_id
                    + ":"
                    + _source_identity(physical_packages),
                    create_revision,
                )
            )
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
        if context is None:
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


def project_result(operation: str, value: object) -> dict[str, JsonValue]:
    """One strict original-document/detail projection shared by both transports."""
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
    from scripts.capabilities.native_contracts import (  # noqa: PLC0415
        binding_for,
    )

    try:
        preserve = binding_for(operation).preserve_document
    except ValueError:
        preserve = False
    if preserve:
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
    return JSON_OBJECT.validate_python(value)


def main(argv: list[str] | None = None) -> int:
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
    value = project_result(operation, value)
    if destination is None:
        print(json.dumps(value, allow_nan=False))
    else:
        with destination.open("x") as output:
            json.dump({"result": value}, output, allow_nan=False)
    return 0


if __name__ == "__main__":
    from scripts.telemetry_process import run_helper

    raise SystemExit(run_helper("pse-native-worker", main))
