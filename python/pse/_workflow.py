# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Typed declarations and owned handles over the single native computation pipeline."""

from collections.abc import Mapping, Sequence
from pathlib import Path

import attrs

from pse import codec
from pse._analyses import Analysis
from pse._artifacts import (
    ArtifactOwnerKind,
    ArtifactSettlement,
    ScientificArtifact,
    ScientificArtifactKind,
)
from pse._build import (
    EngineSettings,
    _NativePhysicalContext,
    _NativeRuntime,
)
from pse._inspection import TableStream
from pse._modeling import LocalStudy, ModelingPackage, StudyReport
from pse._physical import ArrowSchema
from pse._research import ResearchRuntime
from pse._runs import (
    PreparedOperation,
    ProgressStream,
    RunHandle,
    StudyHandle,
)
from pse._strategies import (
    PreparedStrategy,
)
from pse._study_workbench import StudyWorkbenchMixin
from pse.contracts.documents import (
    AnalysisControls,
    AnalysisDocument,
    AnalysisDocumentOutput,
    ArtifactIntent,
    BackendSettings,
    ConicRequest,
    ContextCut,
    ContextSelection,
    ContextSelectionOutput,
    DiagnosticExecutionDocumentOutput,
    DiagnosticExecutionRequest,
    DiagnosticReproducer,
    DiagnosticReproducerOutput,
    FrozenEvidenceSet,
    FrozenEvidenceSetOutput,
    HistoryObligation,
    HistoryOwner,
    HistoryReclamationPage,
    PackExportRequest,
    PackIndexOutput,
    ParameterCovarianceOutput,
    ProjectAcknowledgementOutput,
    ProjectBranchPageOutput,
    ProjectBranchRecordOutput,
    ProjectContextOutput,
    ProjectCursor,
    ProjectCursorBody,
    ProjectDraftDeltaOutput,
    ProjectDraftRecordOutput,
    ProjectDraftSaveIntent,
    ProjectDraftSaveIntentOutput,
    ProjectDraftVersion,
    ProjectDraftVersionOutput,
    ProjectMergeIntent,
    ProjectMergeIntentOutput,
    ProjectMergePreview,
    ProjectMergeResolution,
    ProjectMetadataPageOutput,
    ProjectMetadataRequest,
    ProjectMetadataRequestOutput,
    ProjectNavigationRequest,
    ProjectPublication,
    ProjectPublicationOutput,
    ProjectRecord,
    ProjectSelection,
    ProjectSelectRequest,
    ReproductionDocumentOutput,
    ResourceReportOutput,
    ResultReclamationPage,
    RunControls,
    SolverCapability,
    SolveSettings,
    SourceDelta,
    SourceDeltaOutput,
    SourceReclamationPage,
    SourceSettlementOutput,
    StudyDefinition,
    StudyDefinitionOutput,
    StudyRunControls,
    TaskDumpDocumentOutput,
    TaskDumpRequest,
)
from pse.contracts.enums import (
    CanonicalTerminalOutcome,
    NativeBackend,
    PhysicalInputKind,
)
from pse.contracts.identities import (
    SolutionId,
    StudyId,
)
from pse.contracts.values import ContentHash, SemanticId
from pse.diagnostics import Diagnostics


@attrs.frozen(init=False)
class Runtime(ResearchRuntime, StudyWorkbenchMixin):
    """One native deployment budget with explicit local or durable composition."""

    _handle: _NativeRuntime

    def __init__(
        self,
        settings: EngineSettings,
        *,
        substrate: str | None = None,
        database: str | None = None,
        producer: str | None = None,
        ephemeral: bool = False,
    ) -> None:
        """Attach to the shared deployment under an explicit durability class.

        Args:
            settings: The deployment budget shared by compilation, native work
                and reads.
            substrate: Supervisor state directory for canonical scientific storage;
                optional for explicit ephemeral computation.
            database: Optional explicitly selected existing canonical database.
            producer: Optional qualified mathematical producer receipt path.
            ephemeral: Explicitly admit models and retain outcomes locally, without
                connecting during construction.
                Application execution durably records outcomes in canonical storage.
        """
        object.__setattr__(
            self,
            "_handle",
            _NativeRuntime(
                settings,
                substrate=substrate,
                database=database,
                producer=producer,
                ephemeral=ephemeral,
            ),
        )

    @staticmethod
    def initialize_database(
        settings: EngineSettings, *, substrate: str, database: str
    ) -> None:
        """Explicitly provision a selected database before opening its runtime."""
        _NativeRuntime.initialize_database(
            settings, substrate=substrate, database=database
        )

    def close(self) -> None:
        """Release this caller and its aliases.

        Admitted work continues through drain.
        """
        self._handle.close()

    def run_reproducer(
        self, request: DiagnosticReproducer | DiagnosticReproducerOutput
    ) -> ReproductionDocumentOutput:
        """Execute a selected reproducer under one fresh operation and deadline."""
        return self._handle.run_reproducer(request)

    def diagnose_documents(
        self, request: DiagnosticExecutionRequest
    ) -> DiagnosticExecutionDocumentOutput:
        """Run original-source diagnostics under one admission-to-result deadline."""
        return self._handle.diagnose_documents(request)

    def task_dump(self, request: TaskDumpRequest) -> TaskDumpDocumentOutput:
        """Re-poll the actual executor; timeout does not cancel its issued dump."""
        return self._handle.task_dump(request)

    def task_dump_status(self) -> TaskDumpDocumentOutput | None:
        """Read retained issued-request state without requesting another dump."""
        return self._handle.task_dump_status()

    def observations(self) -> Diagnostics:
        """Read preparation and execution diagnostics independently of this caller."""
        return Diagnostics(self._handle.observations())

    @property
    def durable(self) -> bool:
        """Whether execution records canonical runs and terminal observations."""
        return self._handle.durable

    def modeling_revision(
        self, revision: str, physical: "PhysicalContext"
    ) -> ModelingPackage:
        """Reopen a retained source revision from canonical storage."""
        return ModelingPackage(
            self._handle.modeling_revision(
                revision,
                physical._handle,  # noqa: SLF001 - same native boundary
            )
        )

    def source_settlement(self, problem: str, operation: str) -> SourceSettlementOutput:
        """Observe the original source acknowledgement without replaying the edit.

        An absent acknowledgement leaves the original outcome unproved.
        """
        return self._handle.source_settlement(problem, operation)

    def resource_usage(self) -> ResourceReportOutput:
        """Observe the shared deployment's accounted pool and process memory."""
        return self._handle.resource_usage()

    def run_record(self, run: str) -> TableStream:
        """Read the exact registry canonical run row without result hydration."""
        return TableStream(self._handle.run_record(run))

    def attempt_record(self, attempt: str) -> TableStream:
        """Read one actual attempt's lifecycle and recorded terminal class."""
        return TableStream(self._handle.attempt_record(attempt))

    def result_manifest(self, attempt: str) -> TableStream:
        """Read the exact admitted closed descriptor through its Arrow schema."""
        return TableStream(self._handle.result_manifest(attempt))

    def fit_covariance(self, run: str, attempt: str) -> ParameterCovarianceOutput:
        """Hydrate the actual qualified covariance of one exact recorded fit."""
        return self._handle.fit_covariance(run, attempt)

    def recover_artifact_intent(self, intent: ArtifactIntent) -> ArtifactSettlement:
        """Recover an exact original intent document for settlement."""
        return ArtifactSettlement(self._handle.recover_artifact_intent(intent))

    def scientific_artifact(self, key: str) -> ScientificArtifact:
        """Reopen one active independent artifact after producer retirement."""
        return ScientificArtifact(self._handle.scientific_artifact(key))

    def promote_seed_artifact(
        self, solution: SolutionId, *, owner_kind: ArtifactOwnerKind, owner: str
    ) -> ScientificArtifact:
        """Copy one freshly qualified original start and its reuse assessment."""
        return ScientificArtifact(
            self._handle.promote_seed_artifact(solution, owner_kind, owner)
        )

    def promote_scientific_artifact(
        self,
        run: str,
        attempt: str,
        kind: ScientificArtifactKind,
        *,
        owner_kind: ArtifactOwnerKind,
        owner: str,
    ) -> ScientificArtifact:
        """Preserve an interpreted scientific product; insufficient context refuses."""
        return ScientificArtifact(
            self._handle.promote_scientific_artifact(
                run, attempt, kind, owner_kind, owner
            )
        )

    def analysis(self, key: str) -> Analysis:
        """Reopen one exact active scientific graph without hydrating results."""
        return Analysis(self._handle.analysis(key))

    def result_analysis(
        self, run: str, attempt: str, controls: AnalysisControls
    ) -> Analysis:
        """Persist family-qualified response evidence and exact result provenance."""
        return Analysis(
            self._handle.result_analysis(run, attempt, codec.encode_json(controls))
        )

    def retain_history_obligation(self, obligation: HistoryObligation) -> None:
        """Protect exact audit or unsettled-effect detail independently of payloads."""
        self._handle.retain_history_obligation(codec.encode_json(obligation))

    def release_history_obligation(self, obligation: HistoryObligation) -> None:
        """Release this exact attributed obligation; other owners still protect it."""
        self._handle.release_history_obligation(codec.encode_json(obligation))

    def release_history(self, owner: HistoryOwner) -> None:
        """Irreversibly release settled detail separately from payload retirement."""
        self._handle.release_history(codec.encode_json(owner))

    def reclaim_history_page(self, owner: HistoryOwner) -> HistoryReclamationPage:
        """Collect one bounded history page while preserving permanent replay fences."""
        return self._handle.reclaim_history_page(owner)

    def forget_study_results(self, study: str) -> None:
        """Withdraw a terminal study's explicit retention obligation."""
        self._handle.forget_study_results(study)

    def forget_analysis_results(self, analysis: str) -> bool:
        """Remove a bounded page; repeat until creation settles and returns True."""
        return self._handle.forget_analysis_results(analysis)

    def reclaim_run_results(self, run: str) -> ResultReclamationPage:
        """Retire a terminal run and reclaim one bounded page of scientific data."""
        return self._handle.reclaim_run_results(run)

    def reclaim_source_page(self, problem: str) -> SourceReclamationPage:
        """Continue bounded source cleanup from its stored cursor.

        Repeat until ``after`` is None.
        """
        return self._handle.reclaim_source_page(problem)

    def latest_results(
        self,
        problem: str,
        relation: str,
        *,
        classes: Sequence[CanonicalTerminalOutcome] = (),
        start: int = 0,
        end: int = 18446744073709551615,
    ) -> TableStream:
        """Select the newest admitted semantic run in explicit terminal classes.

        Empty classes include every recorded terminal outcome. A requested class
        never changes the scientific meaning of a failed or partial result.
        """
        return TableStream(
            self._handle.latest_results(
                problem, relation, classes=list(classes), start=start, end=end
            )
        )

    def export_review_pack(
        self,
        destination: str | Path,
        evidence: "FrozenEvidenceSet | FrozenEvidenceSetOutput",
        analysis: "AnalysisDocument | AnalysisDocumentOutput",
        request: "PackExportRequest",
    ) -> "PackIndexOutput":
        """Export protected scientific selections and release their readers."""
        return self._handle.export_review_pack(
            Path(destination), evidence, analysis, request
        )

    def results(
        self,
        run: str,
        attempt: str,
        relation: str,
        *,
        start: int = 0,
        end: int = 18446744073709551615,
    ) -> TableStream:
        """Stream one exact admitted run/attempt's registry result relation.

        The identities survive restart. ``start..end`` selects half-open recorded
        row coordinates; partial or failed outcomes keep their own available
        observations. Missing coverage does not select another attempt's values.
        The stream retains source/result protection until its unread work closes.
        Values preserve their exact IEEE payloads, units and registry metadata.
        """
        return TableStream(
            self._handle.results(run, attempt, relation, start=start, end=end)
        )

    def output_results(
        self,
        run: str,
        attempt: str,
        relation: str,
        output: SemanticId,
        field: str,
        *,
        partition: str = "0",
        start: int = 0,
        end: int = (1 << 64) - 1,
        minimum: float | None = None,
        maximum: float | None = None,
        missing: bool | None = None,
    ) -> TableStream:
        """Read one declared output's original scientific rows.

        Scalar bounds filter finite values; trajectory bounds filter time.
        Missing scalar values require an explicit missing predicate. Units and
        quality remain in the original rows; numeric indexes never replace bits.
        """
        return TableStream(
            self._handle.output_results(
                run,
                attempt,
                relation,
                output,
                field,
                partition=partition,
                start=start,
                end=end,
                minimum=minimum,
                maximum=maximum,
                missing=missing,
            )
        )

    def export_results(
        self,
        run: str,
        attempt: str,
        relation: str,
        destination: str | Path,
        *,
        start: int = 0,
        end: int = 18446744073709551615,
    ) -> None:
        """Export exact results as IPC with recorded source and terminal lineage.

        The final path appears after successful completion. An interrupted export
        retains a sibling ``.incomplete`` file. Existing destinations are refused.
        """
        self._handle.export_results(
            run, attempt, relation, str(destination), start=start, end=end
        )

    def progress(self, run: str, attempt: str) -> ProgressStream:
        """Read the exact terminal attempt's bounded recorded event history."""
        return ProgressStream(self._handle.progress(run, attempt))

    def record_study(self, local: LocalStudy) -> StudyDefinitionOutput:
        """Publish exact retained inputs and return a portable recorded definition."""
        return self._handle.record_study(
            local._handle,  # noqa: SLF001 - same native ownership boundary
        )

    def execute_study(
        self,
        definition: StudyDefinition | StudyDefinitionOutput,
        *,
        controls: StudyRunControls | None = None,
    ) -> StudyReport:
        """Join recorded durable execution and retain its original result report."""
        return StudyReport(
            self._handle.execute_study(
                codec.encode_json(definition),
                controls=None if controls is None else codec.encode_json(controls),
            )
        )

    def start_study(
        self, definition: StudyDefinition | StudyDefinitionOutput
    ) -> StudyHandle:
        """Start durable workers from a genuinely recorded portable definition."""
        return StudyHandle(self._handle.start_study(codec.encode_json(definition)))

    def study(self, study_id: StudyId) -> StudyHandle:
        """Return a handle on a durable study of this runtime's store."""
        return StudyHandle(self._handle.study(study_id))

    def work(self, *, maximum_actions: int | None = None) -> int:
        """Serve native ready points and finalization until idle or the action limit.

        Args:
            maximum_actions: Stop after this many native worker actions.

        Returns:
            The number of native worker actions processed.
        """
        return self._handle.work(maximum_actions=maximum_actions)

    def physical_from_documents(
        self, documents: Mapping[str, str | bytes]
    ) -> "PhysicalContext":
        """Admit actual physical package rows using the native source loader."""
        return PhysicalContext(self._handle.physical_from_documents(dict(documents)))

    def modeling_from_documents(
        self,
        documents: Sequence[Mapping[str, str | bytes]],
        physical: "PhysicalContext",
    ) -> ModelingPackage:
        """Admit an exact package closure with manifest-owned physical type aliases."""
        return ModelingPackage(
            self._handle.modeling_from_documents(
                [dict(bundle) for bundle in documents],
                physical._handle,  # noqa: SLF001 - same native boundary
            )
        )

    def project_context_cut(self, project: str) -> ContextCut:
        """Capture the project's recorded context position, independent of source sequence."""
        return self._handle.project_context_cut(project)

    def project_context_select(
        self, cut: ContextCut, scope: str, effective_at_us: int
    ) -> ContextSelectionOutput | None:
        """Select at an explicit UTC microsecond instant and original recorded cut."""
        return self._handle.project_context_select(cut, scope, effective_at_us)

    def project_context_record(self, key: str) -> ProjectContextOutput | None:
        """Read original immutable metadata, including tombstones and hidden projects."""
        return self._handle.project_context_record(key)

    def project_context_reopen(
        self, selection: ContextSelection | ContextSelectionOutput
    ) -> ModelingPackage:
        """Reopen original persisted source and physical closures without supplied documents."""
        return ModelingPackage(self._handle.project_context_reopen(selection))

    def project_context_package(
        self,
        selection: ContextSelection | ContextSelectionOutput,
        physical: "PhysicalContext",
    ) -> ModelingPackage:
        """Load this exact selection through ordinary source admission; never select latest."""
        return ModelingPackage(
            self._handle.project_context_package(
                selection,
                physical._handle,  # noqa: SLF001 - same native boundary
            )
        )

    def project_context_append(
        self,
        operation: str,
        project: ProjectRecord,
        scope: str,
        source: ModelingPackage,
        effective_from_us: int,
        *,
        effective_to_us: int | None = None,
        corrects: str | None = None,
        deleted: bool = False,
    ) -> ProjectContextOutput:
        """Append an exact source reference with UTC half-open interval and correction ancestry.

        Retain the original project descriptor and operation through uncertain completion.
        Scientific values remain in the admitted source, not this context record.
        """
        return self._handle.project_context_append(
            operation,
            project,
            scope,
            source._handle,  # noqa: SLF001 - same native boundary
            effective_from_us,
            effective_to_us=effective_to_us,
            corrects=corrects,
            deleted=deleted,
        )

    def project_create(
        self, operation: str, project: str, title: str
    ) -> ProjectAcknowledgementOutput:
        """Create project organization using an original caller-issued operation."""
        return self._handle.project_create(operation, project, title)

    def project_record(self, project: str) -> ProjectRecord | None:
        """Read project metadata without hydrating scientific payloads."""
        return self._handle.project_record(project)

    def project_branch_record(self, branch: str) -> ProjectBranchRecordOutput | None:
        """Read branch ownership; its source problem owns the single head."""
        return self._handle.project_branch_record(branch)

    def project_branches(
        self, project: str, *, cursor: ProjectCursor | ProjectCursorBody | None = None
    ) -> ProjectBranchPageOutput:
        """Read one ordered page with an authority/generation-fenced continuation."""
        if isinstance(cursor, ProjectCursorBody):
            cursor = ProjectCursor(
                issuing_store=cursor.issuing_store,
                project=cursor.project,
                generation=cursor.generation,
                after=cursor.after,
            )
        return self._handle.project_branches(project, cursor=cursor)

    def project_fork(
        self,
        project: str,
        branch: str,
        problem: str,
        operation: str,
        source: ModelingPackage,
    ) -> str:
        """Fork an exact admitted source closure into an independent writable branch."""
        return self._handle.project_fork(
            project,
            branch,
            problem,
            operation,
            source._handle,  # noqa: SLF001 - same native boundary
        )

    def project_publication_intent(
        self,
        project: str,
        branch: str,
        expected: str,
        operation: str,
        *,
        draft: str | None = None,
    ) -> ProjectPublicationOutput:
        """Issue the original exact source/draft CAS; retain it through uncertainty."""
        return self._handle.project_publication_intent(
            project, branch, expected, operation, draft=draft
        )

    def project_draft_save_intent(
        self,
        operation: str,
        project: str,
        branch: str,
        key: str,
        base: str,
        content: bytes,
        *,
        expected: int | None = None,
    ) -> ProjectDraftSaveIntentOutput:
        """Read-only issue of raw custody; content may be malformed or empty.

        Keep this original intent until its artifact and project acknowledgements settle.
        Scientific publication separately requires the ProjectDraftDocuments codec.
        """
        return self._handle.project_draft_save_intent(
            operation, project, branch, key, base, content, expected=expected
        )

    def project_draft_save(
        self,
        intent: ProjectDraftSaveIntent | ProjectDraftSaveIntentOutput,
        content: bytes,
    ) -> ProjectAcknowledgementOutput:
        """Save only original issued bytes; retries preserve artifact/operation identity."""
        return self._handle.project_draft_save(intent, content)

    def project_draft_record(self, key: str) -> ProjectDraftRecordOutput | None:
        """Read raw custody metadata without granting a live file read."""
        return self._handle.project_draft_record(key)

    def project_draft_bytes(self, key: str) -> bytes:
        """Read this exact raw body through its current protection owner."""
        return bytes(self._handle.project_draft_bytes(key))

    def project_select(
        self, request: ProjectSelectRequest, physical: "PhysicalContext"
    ) -> ProjectSelection:
        """Freeze an exact branch or bookmark source with its physical closure identity."""
        return self._handle.project_select(request, physical._handle)  # noqa: SLF001

    def project_open_selection(
        self, selection: ProjectSelection, physical: "PhysicalContext"
    ) -> ModelingPackage:
        """Reopen retained science after labels or the branch head have changed."""
        return ModelingPackage(
            self._handle.project_open_selection(selection, physical._handle)  # noqa: SLF001
        )

    def project_navigate(
        self,
        operation: str,
        project: ProjectRecord,
        request: ProjectNavigationRequest,
    ) -> ProjectAcknowledgementOutput:
        """Apply an explicit name, bookmark, savepoint or visibility operation.

        Keep the original project generation and request when settling a retry.
        """
        return self._handle.project_navigate(operation, project, request)

    def project_metadata(
        self, request: ProjectMetadataRequest | ProjectMetadataRequestOutput
    ) -> ProjectMetadataPageOutput:
        """Read bounded live headers; raw bodies and scientific payloads stay separate."""
        return self._handle.project_metadata(request)

    def project_draft_version(self, key: str) -> ProjectDraftVersionOutput | None:
        """Select an exact immutable raw version after later head changes."""
        return self._handle.project_draft_version(key)

    def project_draft_version_bytes(self, key: str) -> bytes:
        """Read original raw custody without moving a current draft head."""
        return bytes(self._handle.project_draft_version_bytes(key))

    def project_draft_delta(
        self, source: ModelingPackage, delta: SourceDelta | SourceDeltaOutput
    ) -> ProjectDraftDeltaOutput:
        """Capture typed edits against their original exact source and physical context."""
        return self._handle.project_draft_delta(source._handle, delta)  # noqa: SLF001

    def project_admit_draft(
        self, operation: str, key: str, physical: "PhysicalContext"
    ) -> ModelingPackage:
        """Admit locally and retain diagnostics; the branch head stays unchanged."""
        return ModelingPackage(
            self._handle.project_admit_draft(operation, key, physical._handle)  # noqa: SLF001
        )

    def project_acknowledgement(
        self, operation: str
    ) -> ProjectAcknowledgementOutput | None:
        """Observe the original immutable receipt; absence does not prove rollback."""
        return self._handle.project_acknowledgement(operation)

    def project_merge_preview(
        self,
        base: ModelingPackage,
        target: ModelingPackage,
        proposal: ModelingPackage,
        *,
        resolutions: Sequence[ProjectMergeResolution] = (),
    ) -> ProjectMergePreview:
        """Inspect three-way logical conflicts before issuing a publication decision."""
        return self._handle.project_merge_preview(
            base._handle, target._handle, proposal._handle, list(resolutions)
        )

    def project_merge_intent(
        self,
        publication: ProjectPublication | ProjectPublicationOutput,
        base: ModelingPackage,
        target: ModelingPackage,
        proposal: ModelingPackage,
        *,
        resolutions: Sequence[ProjectMergeResolution] = (),
    ) -> ProjectMergeIntentOutput:
        """Retain exact admitted revision selections and explicit conflict choices."""
        return self._handle.project_merge_intent(
            publication,
            base._handle,
            target._handle,
            proposal._handle,
            list(resolutions),
        )

    def project_merge_draft_intent(
        self,
        publication: ProjectPublication | ProjectPublicationOutput,
        base: ModelingPackage,
        target: ModelingPackage,
        version: ProjectDraftVersion | ProjectDraftVersionOutput,
        *,
        resolutions: Sequence[ProjectMergeResolution] = (),
    ) -> ProjectMergeIntentOutput:
        """Merge an exact saved version without publishing a temporary source revision."""
        return self._handle.project_merge_draft_intent(
            publication, base._handle, target._handle, version, list(resolutions)
        )

    def project_undo_intent(
        self,
        publication: ProjectPublication | ProjectPublicationOutput,
        target: ModelingPackage,
        restore: ModelingPackage,
    ) -> ProjectMergeIntentOutput:
        """Retain an ordinary compensating source publication, never a head rewind."""
        return self._handle.project_undo_intent(
            publication, target._handle, restore._handle
        )

    def project_publish_merge(
        self,
        intent: ProjectMergeIntent | ProjectMergeIntentOutput,
        physical: "PhysicalContext",
    ) -> ModelingPackage:
        """Publish or settle the original immutable merge/undo decision."""
        return ModelingPackage(
            self._handle.project_publish_merge(intent, physical._handle)
        )

    def project_publish_draft(
        self,
        intent: ProjectPublication | ProjectPublicationOutput,
        physical: "PhysicalContext",
    ) -> ModelingPackage:
        """Admit the saved document container, then atomically publish its exact generation."""
        return ModelingPackage(
            self._handle.project_publish_draft(
                intent,
                physical._handle,  # noqa: SLF001 - same native boundary
            )
        )

    def physical_input_schema(self, kind: PhysicalInputKind) -> ArrowSchema:
        """Return the registered Arrow schema for a physical input role.

        ``pyarrow.schema(...)`` imports its complete native schema and metadata.
        """
        return self._handle.physical_input_schema(kind)

    def capabilities(self) -> tuple[SolverCapability, ...]:
        """Discover linked native libraries without PATH or optional Python probes."""
        return tuple(self._handle.capabilities())

    def capability_for(
        self, backend: NativeBackend, *, settings: BackendSettings | None = None
    ) -> SolverCapability:
        """Inspect the linked adapter's actual support for the selected settings.

        A profile embedded in ``SolveSettings.settings`` is a domain body. Select
        its current standalone interpretation explicitly before passing it here::

            selected = codec.convert_document(
                codec.document_builtins(solve_settings.settings), BackendSettings
            )
            runtime.capability_for(backend, settings=selected)
        """
        return self._handle.capability_for(backend, settings=settings)

    def clear_program_cache(self) -> None:
        """Release retained programs while keeping active prepared workers valid."""
        self._handle.clear_program_cache()

    def prepare_conic(
        self,
        request: ConicRequest,
        physical: "PhysicalContext",
        settings: SolveSettings,
    ) -> PreparedStrategy:
        """Admit explicit library cone geometry and an exact quadratic witness."""
        return PreparedStrategy(
            self._handle.prepare_conic(
                codec.encode_json(request),
                physical._handle,  # noqa: SLF001 - same native boundary
                settings,
            )
        )

    def start(
        self, cases: Sequence[PreparedOperation], *, controls: RunControls | None = None
    ) -> RunHandle:
        """Run a finite sequence on the existing completion-owned native pipeline."""
        return RunHandle(
            self._handle.start(
                [c._handle for c in cases],  # noqa: SLF001 - same native boundary
                controls=None if controls is None else codec.encode_json(controls),
            )
        )


@attrs.frozen
class PhysicalContext:
    """Admitted quantities, units, prerequisites and original source rows."""

    _handle: _NativePhysicalContext

    @property
    def identity(self) -> ContentHash:
        """Complete native compiler context identity."""
        return self._handle.identity
