# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Selected host answers. Native facts stay attributable; bounds never rewrite them.

The records below describe host views, not scientific admission schemas. Opaque
owner declarations are explicitly named fields, rather than the primary answer.
"""

from __future__ import annotations

import json
from typing import Annotated, Literal, TypeVar

from pydantic import ConfigDict, Field, JsonValue, TypeAdapter, model_validator

from scripts.capabilities.model import Contract as BaseContract


class Contract(BaseContract):
    model_config = ConfigDict(
        extra="forbid",
        strict=True,
        frozen=True,
        validate_default=True,
        serialize_by_alias=True,
    )


ReferenceText = Annotated[str, Field(min_length=1, max_length=4096)]


class InvocationReference(Contract):
    kind: Literal["invocation"]
    invocation: ReferenceText
    registry: ReferenceText | None = None


class AssessmentReference(Contract):
    kind: Literal["assessment"]
    directory: ReferenceText
    gate: Annotated[str, Field(min_length=1, max_length=160)] | None = None


class NativeReaderReference(Contract):
    kind: Literal["native"]
    reader: Literal[
        "native_result_inspect",
        "native_diagnostic_inspect",
        "native_diagnostic_checkpoint",
    ]
    arguments: Annotated[
        dict[str, JsonValue],
        Field(
            max_length=32,
            description="Original reader envelope; copying it neither validates native eligibility nor executes the reader.",
        ),
    ]
    source_basis: Annotated[dict[str, JsonValue], Field(max_length=32)] = Field(
        default_factory=dict
    )


OwnerReference = Annotated[
    InvocationReference | AssessmentReference | NativeReaderReference,
    Field(discriminator="kind"),
]


class ReaderSuggestion(Contract):
    reference: OwnerReference
    reason: Annotated[str, Field(min_length=1, max_length=2048)]
    disposition: Literal["unexecuted"] = "unexecuted"


MAX_ANSWER_BYTES = 128 * 1024


JSON_RECORD = TypeAdapter(dict[str, JsonValue])


class OwnerRecord(Contract):
    """Original owner vocabulary; extensions must remain real JSON values."""

    model_config = ConfigDict(
        extra="allow", strict=True, frozen=True, serialize_by_alias=True
    )

    @model_validator(mode="before")
    @classmethod
    def json_record(cls, value: object) -> object:
        # BaseModel owns its mutable private storage annotation. Validate the
        # original wire record rather than illegally narrow that inherited slot.
        return JSON_RECORD.validate_python(value)


class ScopeItem(Contract):
    id: str
    native_id: str | None = None
    label: str
    document: str
    binding: str | None = None
    kind: str
    source: str
    source_line: int
    raw_status: str
    state: str
    native_state: str
    status_owner: str
    status_basis: str
    owner: str
    native_cells: dict[str, str]
    relationships: list[dict[str, str]]


class SourceFact(OwnerRecord):
    path: str
    digest: str | None = None
    availability: str | None = None
    role: str | None = None
    scope: str | None = None
    owner: str | None = None
    text: str | None = Field(
        default=None,
        description="Only when explicitly selected at the original source owner.",
    )


class RelationshipsPayload(Contract):
    symbol: str | None = None
    queries: list[str] = Field(default_factory=list)
    predicates: dict[str, JsonValue] | None = None
    exact_paths: list[str] = Field(default_factory=list)
    selection_basis: str = "no admitted paths"
    items: list[OwnerRecord]
    truncated: bool = False
    producers: list[JsonValue] = Field(default_factory=list)
    producer_documents: list[JsonValue] = Field(default_factory=list)
    current_source_facts: list[SourceFact] = Field(default_factory=list)
    revision: str | None = None
    gaps: list[str] = Field(default_factory=list)
    producer_freshness: str = "unobserved"
    coverage: str = "no admitted paths"
    availability: str


class GitRationale(Contract):
    items: list[dict[str, str]]
    basis: str
    mode: str
    query: str | None


class TaskContextPayload(Contract):
    schema_id: str = Field(alias="schema")
    task: str | None
    population_complete: bool
    unknowns: list[str]
    exclusions: list[str]
    native_scope: list[ScopeItem]
    sources: list[SourceFact]
    relationships: RelationshipsPayload
    symbol_leads: RelationshipsPayload | None
    dirty_delta: list[str]
    git_rationale: GitRationale
    evidence: list[dict[str, str]]
    omissions: list[str]
    authority: str


class ConceptSubject(OwnerRecord):
    id: str
    kind: str
    native_id: str
    owner: str


class ConceptPopulation(Contract):
    matched_count: int
    returned_count: int
    offset: int
    limit: int
    complete: bool


class ConceptPayload(Contract):
    schema_id: str = Field(alias="schema")
    subjects: list[ConceptSubject]
    edges: list[OwnerRecord]
    productions: list[OwnerRecord]
    source_facts: list[SourceFact]
    section_ranges: list[OwnerRecord]
    native_facts: list[OwnerRecord]
    complete: bool
    omissions: list[str]
    unknowns: list[str]
    coverage: str
    query: OwnerRecord
    items: list[ConceptSubject]
    next_offset: int | None
    adapter_status: str
    population: ConceptPopulation


class RecipePayload(Contract):
    schema_id: str = Field(alias="schema")
    name: str
    description: str
    source: OwnerRecord
    outer_parameters: list[OwnerRecord]
    attributes: list[JsonValue]
    body: list[JsonValue]
    dependencies: list[JsonValue]
    leaf_contracts: list[OwnerRecord]
    leaf_sources_omitted: int
    leaf_schema_complete: bool
    source_coherence: str
    source_coherence_reason: str
    contract: str
    usage_route: list[str]
    definition_route: list[str]
    source_inventory: str
    source_review: str
    execution: str
    capability: str | None
    classification: str
    work_owner: str | None
    reason: str


class Entrypoint(OwnerRecord):
    kind: str
    identity: str
    source: str
    capability: str | None
    classification: str
    work_owner: str | None
    reason: str


class InventoryPayload(Contract):
    schema_id: str = Field(alias="schema")
    source_inventory: str
    source_review: str
    classification_owner: str
    items: list[Entrypoint]
    offset: int
    total: int
    next_offset: int | None
    totals: dict[str, int]
    unclassified_counts: dict[str, int]
    complete: bool
    omissions: list[str]
    coverage: str
    execution: str


class Choice(Contract):
    value: str | None
    source: str
    local_unresolved: bool


class Placement(Contract):
    slice: str | None
    resource_class: str
    memory_request: str | None
    admission: str


class EnvironmentPayload(Contract):
    version: int
    kind: str
    selection_status: str
    checkout: str
    interpreter: str
    python_package: str
    path: list[str]
    tools: dict[str, str | None]
    compiler: dict[str, Choice]
    target_directory: str
    native_capabilities: list[str] | None
    providers: dict[str, OwnerRecord]
    native_runtime_choices: dict[str, Choice]
    placement: Placement
    command: list[str]
    build_context: OwnerRecord
    local_configuration: str
    refusals: list[str]


class CorpusSource(OwnerRecord):
    id: str
    kind: str
    locator: str
    version_digest: str
    basis: str
    title: str
    status: str


class CorpusExcerpt(Contract):
    text_complete: bool | None = None
    id: str
    source_id: str
    subject_id: str | None
    text: str
    locator: str
    basis: str


class ExcerptPage(Contract):
    offset: int
    limit: int
    total: int
    next_offset: int | None


class KnowledgePayload(Contract):
    version: int
    expected_active: str | None
    excerpt_page: ExcerptPage | None = None
    sources: list[CorpusSource]
    excerpts: list[CorpusExcerpt]
    subjects: list[ConceptSubject]
    edges: list[OwnerRecord]
    productions: list[OwnerRecord]


class DocumentScopePayload(Contract):
    version: int
    observed_at: str
    operation: str
    authority: str
    items: list[ScopeItem]
    total: int
    offset: int
    next_offset: int | None


class ProductSubject(Contract):
    identity: str
    selection: JsonValue
    outcome: str
    requirements: JsonValue


class ProductPage(Contract):
    offset: int
    limit: int
    subject_count: int
    resource_count: int
    artifact_count: int
    diagnostic_count: int
    report_row_count: int
    unreported_count: int
    next_offset: int | None


class ProductTerminal(OwnerRecord):
    complete: bool


class InvocationOwner(OwnerRecord):
    association: str
    liveness: str


class ExecutionExplanation(Contract):
    """Original owner counts and limits, not inferred scientific success."""

    selected_count: int
    reported_count: int
    unreported_count: int
    outcome_counts: dict[str, int]
    observed_stage: str | None
    stage_basis: str
    executed_count: int | None = None
    preparation_seconds: float | None = None
    execution_seconds: float | None = None
    owned_drain: str = "unobserved"
    next_readers: list[ReaderSuggestion] = Field(default_factory=list)
    limits: list[str] = Field(default_factory=list)


class ProductPayload(Contract):
    kind: str
    registry: str
    invocation: str
    run_kind: str
    run_path: str
    observation_mode: str
    phase: str
    terminal_status: str
    terminal_identity: str | None
    terminal: ProductTerminal | None
    lifecycle: JsonValue
    producer: InvocationOwner
    composer: InvocationOwner
    diagnostic_actions: list[OwnerRecord]
    selected_count: int
    selection_state: str
    refusal: JsonValue
    subjects: list[ProductSubject]
    selected_unreported: list[str]
    selected_unreported_count: int
    report_rows: list[OwnerRecord]
    report_errors: list[str]
    workload_exit: int | None
    assurance: str
    peer_invocations: dict[str, JsonValue]
    cleanup_observations: dict[str, JsonValue]
    setup_failures: dict[str, JsonValue]
    process: OwnerRecord
    owned_drain: str
    artifacts: dict[str, JsonValue]
    payloads: list[OwnerRecord]
    resources: list[OwnerRecord]
    page: ProductPage
    execution_explanation: ExecutionExplanation | None = None


class FileObservation(Contract):
    path: str
    status: str
    tail: list[str]
    error: str | None = None
    recorded_digest: str | None = None


class AssessmentRecord(OwnerRecord):
    gate: str
    status: str
    log: str
    role: str | None = None
    exit_code: int | None = None
    evidence_kind: str | None = None
    detail: OwnerRecord | None = None


class AssessmentCheck(Contract):
    record: AssessmentRecord
    qualified: bool
    origin_path: str
    log: FileObservation
    artifacts: list[FileObservation]


class AssessmentSummary(OwnerRecord):
    complete: bool
    source_unchanged: bool
    baseline_failures: int


class AssessmentPayload(Contract):
    run_path: str
    record_path: str
    assessment: AssessmentSummary
    scope: list[OwnerRecord]
    checks: list[AssessmentCheck]
    unattempted_scope: list[OwnerRecord]
    execution_explanation: ExecutionExplanation | None = None


class WatchSelection(Contract):
    kind: Literal["run", "attempt", "unit"]
    handle: str
    registry: str | None = None
    admission_directory: str | None = None


class WatchSnapshot(OwnerRecord):
    state: str
    result_reference: OwnerRecord | None
    run: ProductPayload | None = None


class WatchReconciliation(Contract):
    source: str
    history: str
    gap: str
    previous_observed_unix_ns: int | None


class WatchPayload(Contract):
    mode: Literal["detached-no-write"]
    selection: WatchSelection
    changed: bool
    reason: Literal["changed", "timeout"]
    cursor: str
    revision: str
    observed_unix_ns: int
    reconciliation: WatchReconciliation
    snapshot: WatchSnapshot


class StorageItem(Contract):
    name: str
    path: str
    ownership: str
    active_use: str
    resource_identity: str
    reclaimable_bytes: int | None
    resolved_path: str | None = None
    identity: list[int] | None = None
    status: Literal["alias", "metadata-only", "unavailable"]
    alias_of: str | None = None
    apparent_bytes: int | None = None
    allocated_bytes: int | None = None
    error: str | None = None


class StoragePayload(Contract):
    items: list[StorageItem]
    reclamation_authorized: Literal[False]


class LibraryPayload(Contract):
    version: int
    package: str
    resolved_package: str
    ecosystem: str
    declaration: JsonValue
    declaration_location: str
    exact_version: str | None
    resolved: list[OwnerRecord]
    selected_features: list[str]
    declared_features: list[str]
    feature_basis: str
    target: str | None
    observed_closure: OwnerRecord | None
    rustdoc: OwnerRecord | None
    patches: JsonValue
    local_sources: list[OwnerRecord]
    references: list[OwnerRecord]
    source_cut: list[OwnerRecord]
    revision: str | None
    population_complete: bool
    unresolved: list[str]
    unknowns: list[str]
    effects: str


class CoveredSubject(Contract):
    binary: str | None
    test: str | None
    outcome: Literal["failed", "not-run", "passed"]


class CoveragePayload(Contract):
    schema_id: str = Field(alias="schema")
    capture: str
    fresh: bool
    status: str
    population: list[CoveredSubject] | None
    population_status: str
    exclusions: str
    regions: list[OwnerRecord]
    selection: str
    limits: list[str]


class AnswerState(Contract):
    """Reader completeness is independent of the observed workload's outcome."""

    population: Literal["complete", "partial", "unavailable"]
    detail: Literal["selected", "omitted"] = "selected"
    reasons: list[str] = Field(default_factory=list)


class DetailRefused(Contract):
    operation: str
    outcome: Literal["detail-refused"] = "detail-refused"
    complete: Literal[False] = False
    omissions: list[str]
    encoded_bytes: int
    maximum_bytes: int = MAX_ANSWER_BYTES
    remedy: str = "Narrow the original owner's selection or use its explicit protected export; no facts were truncated."


class ReadAnswer(Contract):
    """Common operation identity required by the final-size safeguard."""

    operation: str


# Separate output families keep native facts discoverable in actual MCP schemas.
class TaskContextResult(ReadAnswer):
    operation: Literal["workspace_task_context"] = "workspace_task_context"
    complete: bool
    omissions: list[str]
    answer: AnswerState
    payload: TaskContextPayload


class ConceptQueryResult(ReadAnswer):
    operation: Literal["workspace_concepts_query"] = "workspace_concepts_query"
    complete: bool
    omissions: list[str]
    answer: AnswerState
    payload: ConceptPayload


class ProducerRelationshipsResult(ReadAnswer):
    operation: Literal["workspace_producer_relationships"] = (
        "workspace_producer_relationships"
    )
    complete: bool
    omissions: list[str]
    answer: AnswerState
    payload: RelationshipsPayload


class RecipeDescriptionResult(ReadAnswer):
    operation: Literal["workspace_recipe_describe"] = "workspace_recipe_describe"
    complete: bool
    omissions: list[str]
    answer: AnswerState
    payload: RecipePayload


class InventoryResult(ReadAnswer):
    operation: Literal["workspace_inventory"] = "workspace_inventory"
    complete: bool
    omissions: list[str]
    answer: AnswerState
    payload: InventoryPayload


class EnvironmentResult(ReadAnswer):
    operation: Literal["environment_explain"] = "environment_explain"
    complete: bool
    omissions: list[str]
    answer: AnswerState
    payload: EnvironmentPayload


class KnowledgeResult(ReadAnswer):
    operation: Literal["knowledge_source_corpus"] = "knowledge_source_corpus"
    complete: bool
    omissions: list[str]
    answer: AnswerState
    payload: KnowledgePayload


class DocumentScopeResult(ReadAnswer):
    operation: Literal["document_scope_query"] = "document_scope_query"
    complete: bool
    omissions: list[str]
    answer: AnswerState
    payload: DocumentScopePayload


class ProductRunResult(ReadAnswer):
    operation: Literal["product_run_read"] = "product_run_read"
    complete: bool
    omissions: list[str]
    answer: AnswerState
    payload: ProductPayload


class AssessmentResult(ReadAnswer):
    operation: Literal["assessment_read"] = "assessment_read"
    complete: bool
    omissions: list[str]
    answer: AnswerState
    payload: AssessmentPayload


class WorkloadWatchResult(ReadAnswer):
    operation: Literal["workload_watch"] = "workload_watch"
    complete: bool
    omissions: list[str]
    answer: AnswerState
    payload: WatchPayload


class StorageResult(ReadAnswer):
    operation: Literal["storage_observe"] = "storage_observe"
    complete: bool
    omissions: list[str]
    answer: AnswerState
    payload: StoragePayload


class LibraryResult(ReadAnswer):
    operation: Literal["library_source_packet"] = "library_source_packet"
    complete: bool
    omissions: list[str]
    answer: AnswerState
    payload: LibraryPayload


class CoverageResult(ReadAnswer):
    operation: Literal["coverage_correspondence_read"] = "coverage_correspondence_read"
    complete: bool
    omissions: list[str]
    answer: AnswerState
    payload: CoveragePayload


Result = TypeVar("Result", bound=ReadAnswer)


def finish_read(
    result_type: type[Result],
    payload: object,
    *,
    population: Literal["complete", "partial", "unavailable"] = "complete",
    reasons: list[str] | None = None,
    detail_omitted: bool = False,
) -> Result | DetailRefused:
    """Validate one selected answer, then refuse oversize without changing its facts."""
    omissions = list(dict.fromkeys(reasons or []))
    result = result_type.model_validate(
        {
            "complete": population == "complete" and not detail_omitted,
            "omissions": omissions,
            "answer": {
                "population": population,
                "detail": "omitted" if detail_omitted else "selected",
                "reasons": omissions,
            },
            "payload": payload,
        }
    )
    size = len(
        json.dumps(
            result.model_dump(mode="json"),
            ensure_ascii=True,
            allow_nan=False,
            separators=(",", ":"),
        ).encode()
    )
    if size > MAX_ANSWER_BYTES:
        return DetailRefused(
            operation=result.operation,
            omissions=[
                "Selected answer exceeds the encoded response bound; no partial payload was substituted."
            ],
            encoded_bytes=size,
        )
    return result


def task_context(value: object) -> TaskContextResult | DetailRefused:
    selected = TaskContextPayload.model_validate(value)
    return finish_read(
        TaskContextResult,
        selected,
        population="complete" if selected.population_complete else "partial",
        reasons=selected.omissions,
    )


def concept_query(value: object) -> ConceptQueryResult | DetailRefused:
    selected = ConceptPayload.model_validate(value)
    state = (
        "unavailable"
        if selected.adapter_status == "not_implemented"
        else "partial"
        if not selected.complete
        or selected.next_offset is not None
        or selected.population.offset > 0
        else "complete"
    )
    return finish_read(
        ConceptQueryResult, selected, population=state, reasons=selected.omissions
    )


def producer_relationships(
    value: object,
) -> ProducerRelationshipsResult | DetailRefused:
    selected = RelationshipsPayload.model_validate(value)
    return finish_read(
        ProducerRelationshipsResult,
        selected,
        population="partial"
        if selected.truncated
        else "unavailable"
        if selected.availability != "available"
        else "complete",
        reasons=selected.gaps,
    )


def recipe_description(value: object) -> RecipeDescriptionResult | DetailRefused:
    selected = RecipePayload.model_validate(value)
    return finish_read(
        RecipeDescriptionResult,
        selected,
        detail_omitted=bool(selected.leaf_sources_omitted),
        reasons=["Additional leaf sources are available at their original owner."]
        if selected.leaf_sources_omitted
        else [],
    )


def inventory(value: object) -> InventoryResult | DetailRefused:
    selected = InventoryPayload.model_validate(value)
    return finish_read(
        InventoryResult,
        selected,
        population="partial"
        if not selected.complete or selected.next_offset is not None or selected.offset
        else "complete",
        reasons=selected.omissions,
    )


def environment(value: object) -> EnvironmentResult | DetailRefused:
    selected = EnvironmentPayload.model_validate(value)
    return finish_read(
        EnvironmentResult,
        selected,
        population="partial"
        if selected.local_configuration == "unresolved: not executed"
        else "complete",
        reasons=selected.refusals,
    )


def knowledge(value: object) -> KnowledgeResult | DetailRefused:
    selected = KnowledgePayload.model_validate(value)
    page = selected.excerpt_page
    partial = page is not None and (page.offset > 0 or page.next_offset is not None)
    omitted = any(item.text_complete is False for item in selected.excerpts)
    reasons: list[str] = []
    if partial:
        reasons.append(
            "Only selected source excerpts are returned; source and subject attribution retain the inspected population."
        )
    if omitted:
        reasons.append(
            "An excerpt exceeded the original owner's 16KiB text bound; inspect its attributable source for full bytes."
        )
    return finish_read(
        KnowledgeResult,
        selected,
        population="partial" if partial else "complete",
        detail_omitted=omitted,
        reasons=reasons,
    )


def document_scope(value: object) -> DocumentScopeResult | DetailRefused:
    selected = DocumentScopePayload.model_validate(value)
    return finish_read(
        DocumentScopeResult,
        selected,
        population="partial"
        if selected.offset or selected.next_offset is not None
        else "complete",
    )


def product_run(value: object) -> ProductRunResult | DetailRefused:
    selected = ProductPayload.model_validate(value)
    page = selected.page
    incomplete = (
        selected.terminal_status != "available"
        or bool(selected.selected_unreported_count)
        or selected.selection_state != "collected"
    )
    paged = page.offset > 0 or page.next_offset is not None
    reasons: list[str] = []
    if incomplete:
        reasons.append(
            "The original invocation has missing terminal, collection or subject evidence; reader success is not a workload pass."
        )
    if paged:
        reasons.append(
            "The answer contains the selected subject/resource page; global counts remain authoritative."
        )
    return finish_read(
        ProductRunResult,
        selected,
        population="partial" if incomplete or paged else "complete",
        reasons=reasons,
    )


def assessment(value: object) -> AssessmentResult | DetailRefused:
    selected = AssessmentPayload.model_validate(value)
    summary = selected.assessment
    partial = not summary.complete or bool(selected.unattempted_scope)
    omitted = any(check.record.detail is not None for check in selected.checks)
    reasons: list[str] = []
    if partial:
        reasons.append(
            "The original assessment reports incomplete or unattempted scope; reader success is not qualification."
        )
    if omitted:
        reasons.append(
            "Unselected check bodies remain at their protected owner; retained summary records carry their exact detail references."
        )
    return finish_read(
        AssessmentResult,
        selected,
        population="partial" if partial else "complete",
        detail_omitted=omitted,
        reasons=reasons,
    )


def watch(value: object) -> WorkloadWatchResult | DetailRefused:
    selected = WatchPayload.model_validate(value)
    unavailable = selected.snapshot.state == "observation-unavailable"
    return finish_read(
        WorkloadWatchResult,
        selected,
        population="unavailable" if unavailable else "partial",
        reasons=[selected.reconciliation.gap],
    )


def storage(value: object) -> StorageResult | DetailRefused:
    selected = StoragePayload.model_validate(value)
    absent = [item for item in selected.items if item.status == "unavailable"]
    return finish_read(
        StorageResult,
        selected,
        population="unavailable"
        if absent and len(absent) == len(selected.items)
        else "partial"
        if absent
        else "complete",
        reasons=["Some configured stores could not be observed."] if absent else [],
    )


def library(value: object) -> LibraryResult | DetailRefused:
    selected = LibraryPayload.model_validate(value)
    return finish_read(
        LibraryResult,
        selected,
        population="complete" if selected.population_complete else "partial",
        reasons=selected.unresolved,
    )


def coverage(value: object) -> CoverageResult | DetailRefused:
    selected = CoveragePayload.model_validate(value)
    return finish_read(
        CoverageResult,
        selected,
        population="complete"
        if selected.fresh and selected.population_status == "observed"
        else "partial",
        reasons=selected.limits,
    )
