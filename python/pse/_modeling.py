# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Bounded study browsing and passive reconnectable observation."""

from collections.abc import Mapping, Sequence

import attrs

from pse import codec
from pse._build import (
    DiagnosticReport,
    EngineSettings,
    ModelingDiagnosticSettings,
    ModelingLimits,
    NativeAttempt,
    SimulationSettings,
    _NativeLocalStudy,
    _NativeModelingConformance,
    _NativeModelingDiagnostics,
    _NativeModelingDiagnosticSamples,
    _NativeModelingElasticAttempt,
    _NativeModelingInitialization,
    _NativeModelingInitializationAttempt,
    _NativeModelingKnowledge,
    _NativeModelingNativeAnalysis,
    _NativeModelingNonlinearExplanation,
    _NativeModelingPackage,
    _NativeModelingResult,
    _NativeModelingTrajectory,
    _NativeRunResult,
    _NativeStudyReport,
)
from pse._inspection import TableStream
from pse._physical import ArrowInput, PhysicalContext
from pse._runs import PreparedOperation, RunResult, StoredResult
from pse._strategies import PreparedFlow, PreparedStrategy
from pse.codec._generated import encode_declarations, encode_fits
from pse.contracts.authored import (
    AuthoredFitCasesRow,
    AuthoredModelingDeclarationsRow,
)
from pse.contracts.documents import (
    AdmittedBinding,
    Conclusion,
    ConformanceControls,
    DiagnosticOrigin,
    DiagnosticOriginOutput,
    DiagnosticSamplesControls,
    FlowSelectionDocument,
    InitializationOverrides,
    JacobianDiagnosticControls,
    KnowledgeControls,
    LazyStudyDefinition,
    LinearDiagnosticControls,
    ManagedStudyDiagnosticsDocumentOutput,
    ModelingNonlinearPolicy,
    ModelingInspectionRequest,
    ModelingDeclarationRequest,
    ModelingDeclarationPageOutput,
    ModelingComparisonRequest,
    ModelingComparisonObservationOutput,
    ModelingSourceMapDocumentOutput,
    OperationSourceOutput,
    PhysicalFitRequest,
    PointOverlay,
    PreparationCounts,
    PreparationDiscoveryReportOutput,
    PreparationDiscoveryRequest,
    PreparationDiscoveryRequestOutput,
    PreparationSettings,
    PreparationSettingsOutput,
    ProjectComparisonOutput,
    ProjectSourceDetailOutput,
    PureConformanceControls,
    RecycleRequest,
    SolveSettings,
    SourceDelta,
    StudyPointDefinition,
    StudyRequest,
    StudyRunControls,
)
from pse.contracts.documents import (
    CompletionOutput as Completion,
)
from pse.contracts.documents import (
    ModelingInspectionOutput as ModelingInspection,
)
from pse.contracts.documents import (
    PointOutcomeOutput as PointOutcome,
)
from pse.contracts.enums import (
    ModelingDiagnosticSampleStop,
    ModelingElasticObservation,
    ModelingInitializationStep,
    NativeRunState,
    TrajectoryTermination,
)
from pse.contracts.identities import DeclarationId, FitId, RunId
from pse.contracts.values import ContentHash, SemanticId


class LocalStudy:
    """Owned local scientific intent; recording is an explicit runtime operation."""

    # This is an opaque behavioral owner, not a serializable row contract.
    __slots__ = ("_handle",)

    def __init__(self, handle: _NativeLocalStudy) -> None:
        self._handle = handle

    def execute(self, *, controls: StudyRunControls | None = None) -> "StudyReport":
        """Execute using retained owners after the originating package is released."""
        return StudyReport(
            self._handle.execute(
                controls=None if controls is None else codec.encode_json(controls)
            )
        )

    @property
    def points(self) -> tuple[StudyPointDefinition, ...]:
        """Physically admitted explicit occurrences in authored order."""
        return tuple(self._handle.points)

    @property
    def variants(self) -> LazyStudyDefinition | None:
        """The retained bounded lazy input, without constructing variants."""
        return self._handle.variants

    @property
    def modeling_revision(self) -> str:
        """Attribution of the actual retained local model owner."""
        return self._handle.modeling_revision


@attrs.frozen
class ModelingResult:
    """One original solve outcome plus independent physical model qualification."""

    _handle: _NativeModelingResult

    @property
    def accepted(self) -> bool:
        return self._handle.accepted

    @property
    def run_id(self) -> RunId:
        return self._handle.run_id

    @property
    def outcome_kind(self) -> NativeRunState:
        return self._handle.outcome_kind

    @property
    def validation_error(self) -> DiagnosticReport | None:
        return self._handle.validation_error

    @property
    def bound_source(self) -> AdmittedBinding | None:
        """Exact parent, physical context and admitted transient coordinates."""
        return self._handle.bound_source()

    @property
    def binding_identity(self) -> ContentHash | None:
        """Canonical binding identity, excluding submitted spelling and units."""
        return self._handle.binding_identity

    def attempt(self) -> NativeAttempt | None:
        """The unchanged native attempt, absent for constant evaluation or refusal."""
        return self._handle.attempt()

    def failure(self) -> DiagnosticReport | None:
        return self._handle.failure()

    def table(self, name: str) -> TableStream:
        """Export runtime.modeling_checks or runtime.modeling_reports.

        The exported stream owns its buffers.
        """
        return TableStream(self._handle.table(name))


@attrs.frozen
class ModelingNativeAnalysis:
    """Native analysis evidence; the generated relation defines its scientific scope."""

    _handle: _NativeModelingNativeAnalysis

    @property
    def origin(self) -> DiagnosticOriginOutput | None:
        """Original invocation/failure link, independent of diagnostic success."""
        return self._handle.origin

    @property
    def relation(self) -> str:
        return self._handle.relation

    def attempts(self) -> tuple[NativeAttempt, ...]:
        return tuple(self._handle.attempt(i) for i in range(self._handle.attempt_count))

    def table(self) -> TableStream:
        """Source-coordinate IIS, rays and ranges, or scaled-Jacobian certificates.

        The table also records the analysis limits.
        """
        return TableStream(self._handle.table())


@attrs.frozen
class ModelingElasticAttempt:
    """One diagnostic solve over explicit omitted rows and physical elastic weights."""

    _handle: _NativeModelingElasticAttempt

    @property
    def observation(self) -> ModelingElasticObservation:
        return self._handle.observation

    @property
    def penalty(self) -> float | None:
        return self._handle.penalty

    def omitted(self) -> tuple[SemanticId, ...]:
        return tuple(self._handle.omitted())

    def result(self) -> ModelingResult | None:
        result = self._handle.result()
        return None if result is None else ModelingResult(result)

    def failure(self) -> DiagnosticReport | None:
        return self._handle.failure()


@attrs.frozen
class ModelingNonlinearExplanation:
    """Local diagnostic evidence under unchanged bounds; never a global infeasibility.

    The evidence does not prove that the model is infeasible everywhere.
    """

    _handle: _NativeModelingNonlinearExplanation

    def findings(self) -> TableStream:
        """Structured stop and attempt causes, joined by failure ordinal."""
        return TableStream(self._handle.findings())

    def table(self) -> TableStream:
        """Export the registry-defined history with independent buffer ownership."""
        return TableStream(self._handle.table())

    @property
    def complete(self) -> bool:
        return self._handle.complete

    @property
    def stop(self) -> DiagnosticReport | None:
        return self._handle.stop

    def candidate_rows(self) -> tuple[SemanticId, ...]:
        return tuple(self._handle.candidate_rows())

    def background_variables(self) -> tuple[SemanticId, ...]:
        return tuple(self._handle.background_variables())

    def attempts(self) -> tuple[ModelingElasticAttempt, ...]:
        return tuple(ModelingElasticAttempt(v) for v in self._handle.attempts())


@attrs.frozen
class ModelingConformance:
    """Shared checks and completed fixture solves, including explicit coverage gaps."""

    _handle: _NativeModelingConformance

    @classmethod
    def pure(
        cls,
        documents: Sequence[Mapping[str, str | bytes]],
        physical: Mapping[str, str | bytes],
        settings: EngineSettings,
        *,
        controls: PureConformanceControls | None = None,
        limits: ModelingLimits | None = None,
        preparation: PreparationSettings | PreparationSettingsOutput | None = None,
    ) -> "ModelingConformance":
        """Run native pure conformance; an explicit preparation replaces ``limits``."""
        return cls(
            _NativeModelingConformance.pure(
                [dict(bundle) for bundle in documents],
                dict(physical),
                settings,
                controls=None if controls is None else codec.encode_json(controls),
                limits=limits,
                preparation=preparation,
            )
        )

    @property
    def passed(self) -> bool:
        return self._handle.passed

    @property
    def complete(self) -> bool:
        return self._handle.complete

    @property
    def selected(self) -> bool:
        """Whether the run executed a fixture selection.

        A selection assesses no package coverage.
        """
        return self._handle.selected

    def table(self) -> TableStream:
        return TableStream(self._handle.table())

    def findings(self) -> TableStream:
        return TableStream(self._handle.findings())

    def parity(self) -> TableStream:
        """Oracle parity by unit, oracle and release for every oracle fixture."""
        return TableStream(self._handle.parity())

    def admission(self, name: str) -> TableStream:
        """Export retained runtime.route_decisions or runtime.structural_assessments."""
        return TableStream(self._handle.admission(name))

    def failure(self, ordinal: int) -> DiagnosticReport:
        return self._handle.failure(ordinal)

    def trajectory(self, fixture_id: DeclarationId) -> "ModelingTrajectory":
        return ModelingTrajectory(self._handle.trajectory(fixture_id))

    def initialization(self, fixture_id: DeclarationId) -> "ModelingInitialization":
        return ModelingInitialization(self._handle.initialization(fixture_id))

    def fixtures(self) -> tuple[DeclarationId, ...]:
        return tuple(self._handle.fixtures())

    def fixture_statuses(self) -> TableStream:
        """All discovered fixtures, including those beyond the detailed check limit."""
        return TableStream(self._handle.fixture_statuses())

    def result(self, fixture_id: DeclarationId) -> ModelingResult:
        return ModelingResult(self._handle.result(fixture_id))


@attrs.frozen
class ModelingInitializationAttempt:
    """One unchanged specification attempt.

    The attempt retains its preparation and interruption failures.
    """

    _handle: _NativeModelingInitializationAttempt

    @property
    def kind(self) -> ModelingInitializationStep:
        return self._handle.kind

    @property
    def stage(self) -> str | None:
        return self._handle.stage

    @property
    def fraction(self) -> float | None:
        return self._handle.fraction

    @property
    def accepted(self) -> bool:
        return self._handle.accepted

    @property
    def interruption(self) -> DiagnosticReport | None:
        return self._handle.interruption

    @property
    def preparation_error(self) -> DiagnosticReport | None:
        return self._handle.preparation_error

    def result(self) -> ModelingResult | None:
        handle = self._handle.result()
        return None if handle is None else ModelingResult(handle)


@attrs.frozen
class ModelingInitialization:
    """Ordered initialization history; only an accepted original model commits."""

    _handle: _NativeModelingInitialization

    def findings(self) -> TableStream:
        return TableStream(self._handle.findings())

    def table(self) -> TableStream:
        """Export the registry-defined history with independent buffer ownership."""
        return TableStream(self._handle.table())

    @property
    def completed(self) -> bool:
        return self._handle.completed

    @property
    def failure(self) -> DiagnosticReport | None:
        return self._handle.failure

    def attempts(self) -> tuple[ModelingInitializationAttempt, ...]:
        return tuple(ModelingInitializationAttempt(a) for a in self._handle.attempts())

    def committed_values(self) -> dict[SemanticId, float] | None:
        return self._handle.committed_values()


@attrs.frozen
class CompactStudyResult:
    """Original occurrence completion and explicitly selected bounded detail."""

    _handle: _NativeStudyReport
    _index: int

    @property
    def completion(self) -> Completion:
        return self._handle.compact_completion(self._index)

    @property
    def table_names(self) -> tuple[str, ...]:
        return tuple(self._handle.compact_table_names(self._index))

    def table(self, name: str) -> TableStream:
        """Read original selected chunks; unavailable detail remains a refusal."""
        return TableStream(self._handle.compact_table(self._index, name))


@attrs.frozen
class StudyReport:
    """Independent outcomes with explicit accepted-predecessor dependencies."""

    _handle: _NativeStudyReport

    def diagnostic_selection(self) -> ManagedStudyDiagnosticsDocumentOutput | None:
        """Read managed child evidence; absent for in-process studies."""
        return self._handle.diagnostic_selection()

    def findings(self) -> TableStream:
        return TableStream(self._handle.findings())

    def table(self) -> TableStream:
        """Export the registry-defined history with independent buffer ownership."""
        return TableStream(self._handle.table())

    @property
    def count(self) -> int:
        return self._handle.count

    @property
    def unattempted(self) -> int:
        return self._handle.unattempted

    @property
    def preparations(self) -> PreparationCounts:
        """Caller-process preparation counts; excludes an external primary worker."""
        return self._handle.preparations

    def result(
        self, index: int
    ) -> RunResult | StoredResult | CompactStudyResult | None:
        stored = self._handle.stored_result(index)
        if stored is not None:
            return StoredResult(stored)
        if self._handle.is_compact(index):
            return CompactStudyResult(self._handle, index)
        handle = self._handle.result(index)
        return None if handle is None else RunResult(handle)

    def outcome(self, index: int) -> PointOutcome:
        return self._handle.outcome(index)

    @property
    def conclusion(self) -> Conclusion:
        return self._handle.conclusion

    def failure(self, index: int) -> DiagnosticReport | None:
        return self._handle.failure(index)


@attrs.frozen
class SingularMode:
    """Residual-qualified mode of the report's declared scaled local operator.

    A rectangular null mode has an absent equation on one side. Physical right
    perturbations multiply variable nominals; left multipliers divide row nominals.
    """

    value: float
    left: tuple[float, ...]
    right: tuple[float, ...]
    left_residual: float | None
    right_residual: float | None
    normalization_error: float
    residual_bound: float


@attrs.frozen
class ModelingDiagnostics:
    """Bounded point-specific evidence with named source coordinates."""

    _handle: _NativeModelingDiagnostics

    @property
    def origin(self) -> DiagnosticOriginOutput | None:
        """Original invocation/failure link, independent of diagnostic success."""
        return self._handle.origin

    def table(self, name: str = "runtime.modeling_diagnostics") -> TableStream:
        """Export attributed numerical evidence or runtime.modeling_findings."""
        return TableStream(self._handle.table(name))

    @property
    def complete(self) -> bool:
        return self._handle.complete

    @property
    def profile(self) -> str:
        return self._handle.profile

    @property
    def rank(self) -> int | None:
        return self._handle.rank

    @property
    def cutoff(self) -> float | None:
        return self._handle.cutoff

    def findings(self) -> tuple[DiagnosticReport, ...]:
        return tuple(self._handle.findings())

    def statistics(self) -> dict[str, int]:
        return self._handle.statistics()

    def coordinates(self) -> tuple[tuple[SemanticId, ...], tuple[SemanticId, ...]]:
        rows, columns = self._handle.coordinates()
        return tuple(rows), tuple(columns)

    def singular_modes(
        self,
    ) -> tuple[SingularMode, ...]:
        return tuple(
            SingularMode(
                value,
                tuple(left),
                tuple(right),
                left_residual,
                right_residual,
                normalization_error,
                residual_bound,
            )
            for (
                value,
                left,
                right,
                left_residual,
                right_residual,
                normalization_error,
                residual_bound,
            ) in self._handle.singular_modes()
        )


@attrs.frozen
class ModelingTrajectory:
    """Completed physical samples and the actual native termination.

    Partial runs are included.
    """

    _handle: _NativeModelingTrajectory

    @property
    def accepted(self) -> bool:
        """Completed integration with all requested samples passing model checks."""
        return self._handle.accepted

    @property
    def checks_complete(self) -> bool:
        return self._handle.checks_complete

    @property
    def validation_error(self) -> DiagnosticReport | None:
        return self._handle.validation_error

    @property
    def termination(self) -> TrajectoryTermination:
        return self._handle.termination

    @property
    def completed_time(self) -> float:
        return self._handle.completed_time

    @property
    def samples(self) -> int:
        return self._handle.samples

    def failure(self) -> DiagnosticReport | None:
        return self._handle.failure()

    def table(self, name: str = "runtime.simulation_samples") -> TableStream:
        return TableStream(self._handle.table(name))


@attrs.frozen
class ModelingDiagnosticSamples:
    """Named sample findings with independent results and explicit stopping evidence."""

    _handle: _NativeModelingDiagnosticSamples

    @property
    def origin(self) -> DiagnosticOriginOutput | None:
        """Original invocation/failure link, independent of diagnostic success."""
        return self._handle.origin

    def findings(self) -> TableStream:
        """Structured sample failures, joined to outcomes by failure ordinal."""
        return TableStream(self._handle.findings())

    def table(self) -> TableStream:
        """Export named sample outcomes, including incomplete campaign status."""
        return TableStream(self._handle.table())

    @property
    def unattempted(self) -> int:
        return self._handle.unattempted

    @property
    def stop(self) -> ModelingDiagnosticSampleStop:
        return self._handle.stop

    def ids(self) -> tuple[SemanticId, ...]:
        return tuple(self._handle.ids())

    def result(self, index: int) -> ModelingDiagnostics | None:
        handle = self._handle.result(index)
        return None if handle is None else ModelingDiagnostics(handle)

    def failure(self, index: int) -> DiagnosticReport | None:
        return self._handle.failure(index)


@attrs.frozen
class ModelingKnowledge:
    """Read-only admitted cells retaining their exact source revision."""

    _handle: _NativeModelingKnowledge

    @property
    def source_revision(self) -> ContentHash:
        return self._handle.source_revision

    def table(self) -> TableStream:
        """Generated runtime.modeling_knowledge rows in canonical units."""
        return TableStream(self._handle.table())


@attrs.frozen
class ModelingPackage:
    """Immutable authored package.

    It shares the deployment's compiler and execution service.
    """

    _handle: _NativeModelingPackage

    @property
    def revision_id(self) -> str:
        """Opaque exact parent token for edits of this admitted receiver."""
        return self._handle.revision_id

    @property
    def physical_context(self) -> PhysicalContext:
        """Reuse this model's admitted physical closure without ambient input files."""
        return PhysicalContext(self._handle.physical_context)

    @property
    def operation_source(self) -> OperationSourceOutput:
        """Exact source and physical identity for reconstructible study inputs."""
        return self._handle.operation_source

    @property
    def canonical_revision(self) -> str | None:
        """Immutable retained source selection, suitable for reopening after restart."""
        return self._handle.canonical_revision

    @property
    def canonical_problem(self) -> str | None:
        """Canonical problem identity shared by this source's revisions."""
        return self._handle.canonical_problem

    def knowledge(
        self,
        owner_id: DeclarationId | None = None,
        *,
        controls: KnowledgeControls | None = None,
    ) -> ModelingKnowledge:
        """Inspect admitted records, constants and tables under native limits."""
        return ModelingKnowledge(
            self._handle.knowledge(
                owner_id,
                controls=None if controls is None else codec.encode_json(controls),
            )
        )

    def with_fit_declarations(
        self, fits: tuple[AuthoredFitCasesRow, ...]
    ) -> "ModelingPackage":
        """Attach fit selection intent; measurements remain admitted typed records."""
        return ModelingPackage(self._handle.with_fit_declarations(encode_fits(fits)))

    def prepare_fit(
        self, fit_id: FitId, request: PhysicalFitRequest
    ) -> "PreparedOperation":
        """Admit owned fit settings against the selected model and experiments."""
        return PreparedOperation(self._handle.prepare_fit(fit_id, request))

    def with_limits(self, limits: ModelingLimits) -> "ModelingPackage":
        """Share the admitted revision while selecting explicit expansion limits."""
        return ModelingPackage(self._handle.with_limits(limits))

    def diagnose_samples(
        self,
        case_id: DeclarationId,
        settings: SolveSettings,
        diagnostics: ModelingDiagnosticSettings,
        samples: ArrowInput,
        *,
        controls: DiagnosticSamplesControls | None = None,
        origin: DiagnosticOrigin | DiagnosticOriginOutput | None = None,
    ) -> ModelingDiagnosticSamples:
        """Inspect overrides while preserving the prepared case's frozen inputs."""
        return ModelingDiagnosticSamples(
            self._handle.diagnose_samples(
                case_id,
                settings,
                diagnostics,
                samples,
                controls=None if controls is None else codec.encode_json(controls),
                origin=origin,
            )
        )

    def explain_nonlinear(
        self,
        case_id: DeclarationId,
        settings: SolveSettings,
        policy: ModelingNonlinearPolicy,
    ) -> ModelingNonlinearExplanation:
        """Explain local infeasibility using the actual native penalty policy."""
        return ModelingNonlinearExplanation(
            self._handle.explain_nonlinear(case_id, settings, codec.encode_json(policy))
        )

    def prepare_simulation(
        self, case_id: DeclarationId, settings: SimulationSettings
    ) -> PreparedOperation:
        """Prepare an authored trajectory for cancellable canonical execution.

        The case's fixture declares its modes, events and scheduled inputs.
        """
        return PreparedOperation(self._handle.prepare_simulation(case_id, settings))

    def simulate(
        self, case_id: DeclarationId, settings: SimulationSettings
    ) -> ModelingTrajectory:
        """Integrate the authored case with the existing native dynamics engine."""
        return ModelingTrajectory(self._handle.simulate(case_id, settings))

    def diagnostic_source_map(
        self,
        case_id: DeclarationId,
        settings: SolveSettings,
        *,
        maximum_bytes: int = 16 << 20,
        time_limit: float = 30.0,
    ) -> ModelingSourceMapDocumentOutput:
        """Read static attribution under one preparation and copy clock."""
        return self._handle.diagnostic_source_map(
            case_id, settings, maximum_bytes, time_limit
        )

    def diagnose(
        self,
        case_id: DeclarationId,
        settings: SolveSettings,
        diagnostics: ModelingDiagnosticSettings,
        *,
        origin: DiagnosticOrigin | DiagnosticOriginOutput | None = None,
    ) -> ModelingDiagnostics:
        """Inspect the declared candidate without requesting a native solve."""
        return ModelingDiagnostics(
            self._handle.diagnose(
                case_id,
                settings,
                diagnostics,
                origin=origin,
            )
        )

    def diagnose_linear(
        self,
        case_id: DeclarationId,
        settings: SolveSettings,
        *,
        controls: LinearDiagnosticControls | None = None,
        origin: DiagnosticOrigin | DiagnosticOriginOutput | None = None,
    ) -> ModelingNativeAnalysis:
        """Run native affine diagnostics with explicit physical penalties."""
        return ModelingNativeAnalysis(
            self._handle.diagnose_linear(
                case_id,
                settings,
                controls=None if controls is None else codec.encode_json(controls),
                origin=origin,
            )
        )

    def diagnose_jacobian(
        self,
        case_id: DeclarationId,
        settings: SolveSettings,
        *,
        controls: JacobianDiagnosticControls | None = None,
        origin: DiagnosticOrigin | DiagnosticOriginOutput | None = None,
    ) -> ModelingNativeAnalysis:
        """Bounded native evidence about the local scaled Jacobian."""
        return ModelingNativeAnalysis(
            self._handle.diagnose_jacobian(
                case_id,
                settings,
                controls=None if controls is None else codec.encode_json(controls),
                origin=origin,
            )
        )

    def with_declarations(
        self, declarations: Sequence[AuthoredModelingDeclarationsRow]
    ) -> "ModelingPackage":
        """Admit edited generated declarations while retaining the original revision."""
        return ModelingPackage(
            self._handle.with_declarations(encode_declarations(declarations))
        )

    def declaration_page(self, request: ModelingDeclarationRequest) -> ModelingDeclarationPageOutput:
        """Read a finite native declaration page with exact source attribution."""
        return self._handle.declaration_page(request)

    def declarations(self) -> tuple[AuthoredModelingDeclarationsRow, ...]:
        document = self._handle.declarations()
        return codec.document_rows(
            document.declarations, AuthoredModelingDeclarationsRow
        )

    def with_delta(self, delta: SourceDelta) -> "ModelingPackage":
        """Atomically edit this exact parent; omitted source objects stay inherited."""
        return ModelingPackage(self._handle.with_delta(codec.encode_json(delta)))

    def reload_from_parent(
        self, documents: Sequence[Mapping[str, str | bytes]]
    ) -> "ModelingPackage":
        """Reload complete ordered document inventories against this exact parent.

        The returned revision keeps this problem and its admitted physical context.
        Omitted documents are removed; unrelated fits supplied through IR
        stay inherited.
        """
        return ModelingPackage(
            self._handle.reload_from_parent([dict(bundle) for bundle in documents])
        )

    def comparison_observation(self, other: "ModelingPackage", request: ModelingComparisonRequest) -> ModelingComparisonObservationOutput:
        """Read a finite exact comparison with native source-only/context semantics."""
        return self._handle.comparison_observation(other._handle, request)  # noqa: SLF001 -- same native receiver composition.

    def project_compare(
        self, other: "ModelingPackage", *, source_only: bool = False
    ) -> ProjectComparisonOutput:
        """Compare admitted objects without numerical preparation."""
        return self._handle.project_compare(other._handle, source_only=source_only)  # noqa: SLF001 -- package-native facade composition

    def project_source_detail(
        self, logical: Sequence[str]
    ) -> tuple[ProjectSourceDetailOutput, ...]:
        """Read authored detail and exact binary content identities."""
        return tuple(self._handle.project_source_detail(list(logical)))

    def source_tables(self) -> dict[SemanticId, TableStream]:
        """Export the complete immutable source inventory as explicit Arrow tables."""
        return {
            relation: TableStream(stream)
            for relation, stream in self._handle.source_tables().items()
        }

    def preparation_discovery(
        self, request: PreparationDiscoveryRequest | PreparationDiscoveryRequestOutput
    ) -> PreparationDiscoveryReportOutput:
        """Observe retained preparations and publication headers without preparing them.

        Availability is observational. Durable presence leaves receiving eligibility
        unchecked, and an absent row supplies no cache-miss explanation.
        """
        return self._handle.preparation_discovery(request)

    def inspect(
        self, request: ModelingInspectionRequest
    ) -> ModelingInspection:
        """Inspect a finite native page under its explicit preparation/source policy."""
        return self._handle.inspect(request)

    def prepare_flow(
        self,
        case_id: DeclarationId,
        selection: FlowSelectionDocument,
        settings: SolveSettings,
    ) -> PreparedFlow:
        """Project explicitly selected authored nodes, ports and connection policies."""
        return PreparedFlow(
            self._handle.prepare_flow(
                case_id,
                codec.encode_json(selection),
                settings,
            )
        )

    def prepare_recycle(
        self,
        case_id: DeclarationId,
        selection: FlowSelectionDocument,
        request: RecycleRequest,
        settings: SolveSettings,
    ) -> PreparedStrategy:
        """Compile admitted causal directions from this immutable authored model."""
        return PreparedStrategy(
            self._handle.prepare_recycle(
                case_id,
                codec.encode_json(selection),
                codec.encode_json(request),
                settings,
            )
        )

    def prepare_block_initialization(
        self,
        case_id: DeclarationId,
        settings: SolveSettings,
        stages: ArrowInput,
    ) -> PreparedStrategy:
        """Prepare ordered blocks and transactional overlays over original bindings."""
        return PreparedStrategy(
            self._handle.prepare_block_initialization(
                case_id,
                settings,
                stages,
            )
        )

    def prepare_solve(
        self,
        case_id: DeclarationId,
        settings: SolveSettings,
        *,
        preparation: PreparationSettings | PreparationSettingsOutput | None = None,
        covariance_source: RunResult | None = None,
    ) -> PreparedOperation:
        """Prepare an authored solve with optional compiler and expansion policy."""

        def bind(source: _NativeRunResult | None) -> PreparedOperation:
            return PreparedOperation(
                self._handle.prepare_solve(
                    case_id,
                    settings,
                    preparation=preparation,
                    covariance_source=source,
                )
            )

        if covariance_source is None:
            return bind(None)
        return covariance_source.bind_parameter_covariance(bind)

    def solve_bound(
        self,
        case_id: DeclarationId,
        overlay: PointOverlay,
        settings: SolveSettings,
        *,
        preparation: PreparationSettings | PreparationSettingsOutput | None = None,
        covariance_source: RunResult | None = None,
    ) -> ModelingResult:
        """Solve transient typed values against this immutable declared parent.

        Parameter and fixed-coordinate assignments replace values; free-coordinate
        assignments replace starts. The returned receipt retains parent and attribution.
        """

        def bind(source: _NativeRunResult | None) -> ModelingResult:
            return ModelingResult(
                self._handle.solve_bound(
                    case_id,
                    codec.encode_json(overlay),
                    settings,
                    preparation=preparation,
                    covariance_source=source,
                )
            )

        if covariance_source is None:
            return bind(None)
        return covariance_source.bind_parameter_covariance(bind)

    def solve_case(
        self,
        case_id: DeclarationId,
        settings: SolveSettings,
        *,
        preparation: PreparationSettings | PreparationSettingsOutput | None = None,
        covariance_source: RunResult | None = None,
    ) -> ModelingResult:
        """Solve a case through the native solver pipeline.

        The case uses its source fixture specifications and model starts. An explicit
        preparation replaces this package's compiler and expansion defaults.
        """

        def bind(source: _NativeRunResult | None) -> ModelingResult:
            return ModelingResult(
                self._handle.solve_case(
                    case_id,
                    settings,
                    preparation=preparation,
                    covariance_source=source,
                )
            )

        if covariance_source is None:
            return bind(None)
        return covariance_source.bind_parameter_covariance(bind)

    def initialize(
        self,
        case_id: DeclarationId,
        settings: SolveSettings,
        *,
        overrides: InitializationOverrides | None = None,
    ) -> ModelingInitialization:
        """Run authored stages under native overrides and retain original bindings."""
        return ModelingInitialization(
            self._handle.initialize(
                case_id,
                settings,
                overrides=None if overrides is None else codec.encode_json(overrides),
            )
        )

    def admit_study(self, request: StudyRequest) -> "LocalStudy":
        """Retain admitted scientific intent and its exact immutable source owners."""
        return LocalStudy(self._handle.admit_study(codec.encode_json(request)))

    def study(
        self, local: "LocalStudy", *, controls: StudyRunControls | None = None
    ) -> StudyReport:
        """Execute owned local occurrences without initializing a canonical store."""
        if controls is not None and not isinstance(controls, StudyRunControls):
            message = "local studies require StudyRunControls"
            raise TypeError(message)
        return StudyReport(
            self._handle.study(
                local._handle,  # noqa: SLF001 - same native ownership boundary
                controls=None if controls is None else codec.encode_json(controls),
            )
        )

    def conform(
        self,
        settings: SolveSettings,
        *,
        controls: ConformanceControls | None = None,
        preparation: PreparationSettings | PreparationSettingsOutput | None = None,
    ) -> ModelingConformance:
        """Run bounded checks with optional complete compiler and expansion policy."""
        return ModelingConformance(
            self._handle.conform(
                settings,
                controls=None if controls is None else codec.encode_json(controls),
                preparation=preparation,
            )
        )
