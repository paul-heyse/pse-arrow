# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Native binding declarations and host requests; no execution composition."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Annotated

from pydantic import Field, JsonValue, model_validator

from scripts.capabilities.model import Contract, Prerequisite
from scripts.capabilities.workers import WorkerCall
from scripts.telemetry_investigation import QueryAnswer

Positive = Annotated[int, Field(ge=1)]
Text = Annotated[str, Field(min_length=1)]
Row = Annotated[int, Field(ge=0, le=(1 << 64) - 1)]


class Resources(Contract):
    memory_limit_bytes: Positive = 16 << 30
    threads: Annotated[int, Field(ge=1, le=16)] = 16
    max_spill_bytes: Positive = 1 << 30


class Source(Contract):
    physical: JsonValue
    documents: JsonValue
    resources: Resources = Field(default_factory=Resources)


class Preparation(Source):
    request: JsonValue


class ModelingComparison(Preparation):
    other_physical: JsonValue
    other_documents: JsonValue


class Deployment(Contract):
    state: Text
    database: Text
    resources: Resources = Field(default_factory=Resources)


class Metadata(Deployment):
    request: JsonValue


class Select(Metadata):
    physical: JsonValue


class Revision(Deployment):
    physical: JsonValue
    revision: Text
    logical: Annotated[list[Text], Field(min_length=1, max_length=32)]


class RevisionPreparation(Deployment):
    physical: JsonValue
    revision: Text
    request: JsonValue


class Bundle(Contract):
    path: Text
    maximum_document_bytes: Annotated[int, Field(ge=1, le=16 << 20)] = 16 << 20


class Diagnostic(Contract):
    checkpoint: Text
    request: JsonValue


class DiagnosticProcesses(Contract):
    request: JsonValue
    directory: Text | None = None


class DiagnosticCheckpoint(Contract):
    checkpoint: Text
    expected: JsonValue | None = None


class Evidence(Contract):
    path: Text
    request: JsonValue


class Invocation(Contract):
    path: Text


class Result(Deployment):
    run: Text
    attempt: Text
    relation: Text
    start: Row
    end: Row
    maximum_bytes: Annotated[int, Field(ge=4096, le=65536)] = 65536


class ResultInspection(Deployment):
    request: JsonValue


class EvidenceCompare(Contract):
    request: JsonValue
    snapshots: Annotated[list[JsonValue], Field(min_length=1, max_length=64)]
    observations: Annotated[list[JsonValue], Field(max_length=64)] = Field(
        default_factory=list
    )


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


@dataclass(frozen=True)
class Binding:
    operation: str
    title: str
    request: type[Contract]
    documents: dict[str, str]
    receiver: str
    description: str
    domain: str
    effects: str
    output_document: str | None = None
    prepare_import: bool = False

    @property
    def worker(self) -> WorkerCall:
        return WorkerCall(
            "native", self.operation, "attached" if self.prepare_import else "detached"
        )

    @property
    def requirements(self) -> tuple[Prerequisite, ...]:
        extension = Prerequisite(
            kind="python-extension",
            owner="scripts/python_extension.py",
            mode="native",
            validated=True,
            preparation=("python_extension_native_stage",),
        )
        if issubclass(self.request, Deployment):
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


# These original native documents/bytes are kept intact at both transport stages.
EXACT_READS = frozenset(
    {
        "evidence_compare",
        # The explicit capture action also preserves its original whole document.
        "telemetry_profile_capture",
        "result_read",
        "result_inspect",
        "model_declarations",
        "model_inspect",
        "model_compare",
        "diagnostic_inspect",
        "diagnostic_processes",
        "diagnostic_checkpoint",
    }
)


BINDINGS = (
    Binding(
        "model_declarations",
        "Read native declaration page",
        Preparation,
        {
            "physical": "project-draft-documents",
            "documents": "project-draft-documents",
            "request": "modeling-declaration-request",
        },
        "pse.ModelingPackage.declaration_page",
        "Admit finite supplied scientific source and read an exact selected declaration page; local revision handles expire with the child.",
        "projects",
        "Native source admission and selected declaration observation; no solve.",
        "modeling-declaration-page",
        prepare_import=True,
    ),
    Binding(
        "model_inspect",
        "Inspect native declared model",
        Preparation,
        {
            "physical": "project-draft-documents",
            "documents": "project-draft-documents",
            "request": "modeling-inspection-request",
        },
        "pse.ModelingPackage.inspect",
        "Inspect selected instantiated lineage under an explicit native preparation policy; contextual admission can prepare providers, but no procedure is dispatched.",
        "projects",
        "Native source/provider admission and selected observation, not universally passive.",
        "modeling-inspection",
        prepare_import=True,
    ),
    Binding(
        "model_compare",
        "Compare native authored model sources",
        ModelingComparison,
        {
            "physical": "project-draft-documents",
            "documents": "project-draft-documents",
            "other_physical": "project-draft-documents",
            "other_documents": "project-draft-documents",
            "request": "modeling-comparison-request",
        },
        "pse.ModelingPackage.comparison_observation",
        "Admit two finite sources in one Runtime and compare selected native changes/details; source-only mode preserves unknown scientific impact across physical contexts.",
        "projects",
        "Native source admission and finite comparison; no solve or numerical equivalence claim.",
        "modeling-comparison-observation",
        prepare_import=True,
    ),
    Binding(
        "evidence_compare",
        "Compare explicit baseline and candidate evidence",
        EvidenceCompare,
        {"request": "evidence-comparison-request", "snapshots": "evidence-snapshot"},
        "pse.diagnostics.compare_evidence",
        "Compare supplied original evidence snapshots through the native finite owner. Select exact baseline/candidate generations and explicit correspondence where owner identities differ. Preserve full denominators, ambiguity, unsupported measurement associations and separately bounded sections. Opens no store and performs no scientific execution. Continue only on the exact returned cut.",
        "diagnostics",
        "Bounded supplied-only native evidence analysis; no build or service startup.",
        "evidence-comparison",
    ),
    Binding(
        "source_admit",
        "Admit original authored source",
        Source,
        {"physical": "project-draft-documents", "documents": "project-draft-documents"},
        "pse.Runtime.physical_from_documents + modeling_from_documents",
        "Admit original raw package bytes through the native physical and modeling source loaders. JSON schemas describe the byte container; authoring.schema is guidance for applicable authored TOML sections, not a parser for TOML/.pse/Parquet. The returned local revision is an observation and cannot be reopened after this call.",
        "projects",
        "Prepare native import providers through their existing owner as needed, then admit ephemeral source; no canonical database or publication.",
        prepare_import=True,
    ),
    Binding(
        "preparation_discover",
        "Inspect preparation availability",
        Preparation,
        {
            "physical": "project-draft-documents",
            "documents": "project-draft-documents",
            "request": "preparation-discovery-request",
        },
        "pse.ModelingPackage.preparation_discovery",
        "Admit original supplied source and inspect exact preparation availability within the same Runtime lifetime. This passively observes availability without preparing mathematics or opening canonical storage.",
        "projects",
        "Prepare native import providers through their existing owner as needed, then admit source and passively observe preparations.",
        "preparation-discovery-report",
        prepare_import=True,
    ),
    Binding(
        "project_select",
        "Select exact project source",
        Select,
        {"physical": "project-draft-documents", "request": "project-select-request"},
        "pse.Runtime.project_select",
        "Select a branch or bookmark through its actual native receiver, freezing source and physical identities. Uses the explicitly selected existing canonical database and original physical source bytes.",
        "projects",
        "Read selected canonical store; native physical admission, no publication.",
        "project-selection",
    ),
    Binding(
        "project_metadata",
        "Read project metadata page",
        Metadata,
        {"request": "project-metadata-request"},
        "pse.Runtime.project_metadata",
        "Read the original bounded project/draft/bookmark/savepoint/diagnostic metadata page, preserving the issuing store and owner-bound continuation.",
        "projects",
        "Read explicitly selected existing canonical database; no provisioning.",
        "project-metadata-page",
    ),
    Binding(
        "revision_source",
        "Inspect exact retained source revision",
        Revision,
        {"physical": "project-draft-documents"},
        "pse.Runtime.modeling_revision + ModelingPackage.project_source_detail",
        "Reopen the explicitly retained canonical revision with its original physical closure and read selected logical source detail. Source and binary identities remain native; a local revision token from an earlier ephemeral call cannot be used here.",
        "projects",
        "Protected canonical source read and native source admission; no source edit or publication.",
    ),
    Binding(
        "revision_preparation_discover",
        "Inspect retained revision preparations",
        RevisionPreparation,
        {
            "physical": "project-draft-documents",
            "request": "preparation-discovery-request",
        },
        "pse.Runtime.modeling_revision + ModelingPackage.preparation_discovery",
        "Reopen an exact retained canonical revision and passively inspect its preparation/publication headers. Durable presence preserves unchecked receiving eligibility; no numerical preparation is requested.",
        "projects",
        "Protected canonical source/preparation metadata read, no numerical preparation.",
        "preparation-discovery-report",
    ),
    Binding(
        "project_bundle_inspect",
        "Inspect supplied project pack offline",
        Bundle,
        {},
        "pse.ProjectBundle.manifest + digest",
        "Verify and inspect a supplied copied project closure through the ordinary native bundle owner. Reads its original manifest and digest without opening a canonical database or provisioning storage.",
        "projects",
        "Read and digest-check supplied project pack only.",
        "project-manifest",
    ),
    Binding(
        "research_read",
        "Read original scientific research metadata",
        Metadata,
        {"request": "research-read-request"},
        "pse.Runtime.research_read",
        "Read bounded immutable scientific research dataset, selection or publication metadata through the original native owner. Metadata grants no measurement or result lifetime.",
        "projects",
        "Bounded read of explicitly selected canonical research metadata.",
        "research-read-result",
    ),
    Binding(
        "diagnostic_processes",
        "Discover published native diagnostic processes",
        DiagnosticProcesses,
        {"request": "diagnostic-list-request"},
        "pse.diagnostics.list_processes",
        "Read one bounded page of native process checkpoint summaries from the selected diagnostic directory through the original native reader. Retains process/start locators, recorded target runtime_metrics and operation IDs when available. Directory discovery is not an atomic snapshot or live-process/attachment authority; no Runtime construction, service startup or capture.",
        "diagnostics",
        "Read bounded selected native diagnostic process summaries only; no subject execution or settlement.",
        "diagnostic-process-page",
    ),
    Binding(
        "diagnostic_checkpoint",
        "Read an exact native diagnostic checkpoint",
        DiagnosticCheckpoint,
        {"expected": "checkpoint-identity"},
        "pse.diagnostics.read_checkpoint",
        "Decode one exact bounded published native checkpoint, including target runtime_metrics and registration coverage, without requiring an operation ID. Optional expected identity validates the same decoded image's original diagnostic process/start association. Recorded observations do not prove current liveness or grant attachment, cancellation or settlement authority. No Runtime construction, service startup or capture.",
        "diagnostics",
        "Read supplied native checkpoint and optionally validate its exact original association; no execution or publication.",
        "process-checkpoint",
    ),
    Binding(
        "diagnostic_inspect",
        "Inspect published diagnostic checkpoint",
        Diagnostic,
        {"request": "inspection-request"},
        "pse.diagnostics.inspect_checkpoint",
        "Inspect an exact published checkpoint through native decoding and interpretation, including target runtime_metrics with original target locator, executor role and registration coverage when present. These are recorded target observations, not calling/helper-process metrics. Does not construct a Runtime, start a service, or cancel the observed subject.",
        "diagnostics",
        "Read supplied native checkpoint only.",
        "inspection-document",
    ),
    Binding(
        "evidence_inspect",
        "Inspect supplied frozen evidence",
        Evidence,
        {"request": "frozen-inspection-request"},
        "pse.diagnostics.read_evidence + inspect_frozen",
        "Read a supplied bounded saved evidence set and interpret its immutable members at recorded source time. Embedded paths do not authorize lookup; canonical storage is not acquired.",
        "diagnostics",
        "Read and interpret supplied frozen evidence only.",
        "frozen-inspection-document",
    ),
    Binding(
        "invocation_read",
        "Read original native invocation evidence",
        Invocation,
        {},
        "pse.diagnostics.read_invocation",
        "Decode descriptive invocation facts through the native saved-evidence reader. This grants no live result handle and does not replay the invocation.",
        "diagnostics",
        "Read supplied invocation evidence only.",
        "invocation-evidence",
    ),
    Binding(
        "result_read",
        "Read exact stored scientific result window",
        Result,
        {},
        "pse.Runtime.results",
        "Read a bounded original run/attempt/relation row window through the protected native Arrow reader. Returns Arrow IPC stream bytes, preserving IEEE payloads, units and schema metadata. A detail bound asks for a narrower exact window; it never silently substitutes rows or attempts.",
        "projects",
        "Protected read of explicitly selected canonical results; no scientific execution or publication.",
    ),
    Binding(
        "result_inspect",
        "Inspect protected scientific result shape or readable selected values",
        ResultInspection,
        {"request": "result-inspection-request"},
        "pse.Runtime.inspect_result",
        "Inspect an exact protected run/attempt/relation. Schema view does not acquire data batches. Values view selects bounded fields and rows natively, retains original identity, units, basis and quality context, and represents null/nonfinite/unsupported cells explicitly. Exact original Arrow IPC remains native_result_read. No scientific usability is inferred.",
        "projects",
        "Protected read of the explicitly selected existing canonical result; no execution, provision or publication.",
        "result-inspection-document",
    ),
)


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


def worker_bindings() -> tuple[Binding, ...]:
    """The complete fixed native worker routes, from declaration owners only."""
    return (*BINDINGS, PROFILE_BINDING)
