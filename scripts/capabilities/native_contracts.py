# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Thin host adapters over generated Rust operation/header declarations."""
from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING, TypeVar, cast

from pydantic import JsonValue, model_validator

from scripts.capabilities.model import Contract, Prerequisite
from scripts.capabilities.native_schema import JSON_OBJECT
from scripts.capabilities.workers import WorkerCall
from scripts.generated import native_operations as generated
from scripts.generated.native_operations import (
    Resources as Resources,
    Source as Source,
    Preparation as Preparation,
    ModelingComparison as ModelingComparison,
    Deployment as Deployment,
    Metadata as Metadata,
    Select as Select,
    Revision as Revision,
    RevisionPreparation as RevisionPreparation,
    Bundle as Bundle,
    Diagnostic as Diagnostic,
    DiagnosticProcesses as DiagnosticProcesses,
    DiagnosticCheckpoint as DiagnosticCheckpoint,
    Evidence as Evidence,
    Invocation as Invocation,
    Result as Result,
    PhysicalInspection as PhysicalInspection,
    ResultInspection as ResultInspection,
    EvidenceCompare as EvidenceCompare,
    ConfiguredCapabilities as ConfiguredCapabilities,
)
from scripts.telemetry_investigation import QueryAnswer

if TYPE_CHECKING:
    from collections.abc import Mapping

T = TypeVar("T")


class ProfileCaptureRequest(generated.ProfileCaptureRequest):
    """Host association only; scientific documents still use their native owners."""

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


@dataclass(frozen=True)
class Binding:
    """Existing host interface, derived entirely from one generated declaration."""

    declaration: generated.OperationDeclaration
    request: type[Contract]

    @property
    def operation(self) -> str:
        return self.declaration.operation

    @property
    def title(self) -> str:
        return self.declaration.title

    @property
    def documents(self) -> dict[str, str]:
        return {item.field: item.document.slug for item in self.declaration.documents}

    @property
    def document_types(self) -> dict[str, str]:
        return {item.field: item.document.name for item in self.declaration.documents}

    @property
    def receiver(self) -> str:
        return self.declaration.receiver

    @property
    def description(self) -> str:
        return self.declaration.description

    @property
    def domain(self) -> str:
        return self.declaration.domain

    @property
    def effects(self) -> str:
        return self.declaration.effects

    @property
    def output_document(self) -> str | None:
        reference = self.declaration.output_document
        return reference.slug if reference is not None else None

    @property
    def output_many(self) -> bool:
        return self.declaration.output_many

    @property
    def prepare_import(self) -> bool:
        return self.declaration.prepare_import

    @property
    def preserve_document(self) -> bool:
        return self.declaration.preserve_document

    @property
    def read_eligibility(self) -> str:
        return self.declaration.read_eligibility

    @property
    def context_fields(self) -> tuple[str, ...]:
        return tuple(self.declaration.context_fields)

    @property
    def continuations(self) -> tuple[generated.ContinuationField, ...]:
        return tuple(self.declaration.continuations)

    @property
    def cursor_inputs(self) -> tuple[tuple[str, ...], ...]:
        return tuple(tuple(item.input) for item in self.declaration.continuations)

    @property
    def cursor_outputs(self) -> tuple[tuple[str, ...], ...]:
        return tuple(tuple(item.output) for item in self.declaration.continuations)

    @property
    def worker(self) -> WorkerCall:
        return WorkerCall(
            self.declaration.worker,
            self.operation,
            self.declaration.invocation_lifetime,
        )

    @property
    def requirements(self) -> tuple[Prerequisite, ...]:
        if self.declaration.prerequisites == "evidence_executable":
            return (
                Prerequisite(
                    kind="executable",
                    owner="xtask/src/bin/pse-evidence.rs",
                    provider="pse-evidence",
                    preparation=("just", "evidence-build"),
                ),
            )
        extension = Prerequisite(
            kind="python-extension",
            owner="scripts/python_extension.py",
            mode="native",
            validated=True,
            preparation=("python_extension_native_stage",),
        )
        if self.declaration.prerequisites == "extension_and_canonical":
            return (
                extension,
                Prerequisite(
                    kind="selected-service",
                    owner="scripts/surreal_server.py",
                    provider="canonical-request",
                    preparation=("just", "surreal-start"),
                ),
            )
        return (extension,)


_ENVELOPES = {**generated.ENVELOPES, "ProfileCaptureRequest": ProfileCaptureRequest}
_WORKER_BINDINGS = tuple(
    Binding(row, _ENVELOPES[row.envelope]) for row in generated.OPERATION_DECLARATIONS
)
_BY_OPERATION = {binding.operation: binding for binding in _WORKER_BINDINGS}
if len(_BY_OPERATION) != len(_WORKER_BINDINGS):
    raise ValueError("generated native operation identities must be unique")
BINDINGS = tuple(binding for binding in _WORKER_BINDINGS if binding.declaration.public_binding)
PROFILE_BINDING = next(binding for binding in _WORKER_BINDINGS if binding.request is ProfileCaptureRequest)
# Original-document preservation is a projection, never read-lifetime policy.
EXACT_READS = frozenset(binding.operation for binding in _WORKER_BINDINGS if binding.preserve_document)


def worker_bindings() -> tuple[Binding, ...]:
    """The complete fixed worker routes from responsibility-owned declarations."""
    return _WORKER_BINDINGS


def binding_for(operation: str) -> Binding:
    """Refuse unknown fixed operation identities before receiver construction."""
    try:
        return _BY_OPERATION[operation]
    except KeyError as error:
        raise ValueError("unknown fixed native operation") from error


def validate_arguments(operation: str, arguments: Mapping[str, object]) -> Contract:
    """Materialize only this operation's generated original header defaults/bounds."""
    return binding_for(operation).request.model_validate(arguments, strict=True)


def selected_document_type(operation: str, field: str) -> type[object]:
    """Resolve an original scientific owner lazily, only inside its selected worker."""
    from pse.contracts import documents  # noqa: PLC0415 -- original body owner after native admission

    name = binding_for(operation).document_types[field]
    return cast("type[object]", getattr(documents, name))


def document_type(operation: str, field: str, expected: type[T]) -> type[T]:
    """Keep each small typed receiver glue honest against its generated declaration."""
    if selected_document_type(operation, field) is not expected:
        raise TypeError("receiver document type differs from its owned operation declaration")
    return expected


def native_catalogue() -> dict[str, JsonValue]:
    """Expected compiled native metadata, excluding the independent tooling family."""
    return JSON_OBJECT.validate_python(generated.NATIVE_CATALOGUE.model_dump(mode="json"))
