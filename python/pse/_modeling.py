# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Owned generic modeling operations over registry-generated source and result contracts."""

from collections.abc import Mapping, Sequence
from typing import TYPE_CHECKING, overload

import attrs
import msgspec

from pse import codec
from pse._build import (
    NativeAttempt,
    EngineSettings,
    SimulationSettings,
    _NativeModelingTrajectory,
    _NativeModelingNonlinearExplanation,
    _NativeModelingNativeAnalysis,
    _NativeModelingElasticAttempt,
    ModelingDiagnosticSettings,
    ModelingLimits,
    ModelingFixturePolicy,
    ModelingEventSettings,
    ModelingModeSettings,
    DiagnosticReport,
    _NativeModelingDiagnostics,
    _NativeModelingDiagnosticSamples,
    _NativeModelingConformance,
    _NativeModelingPackage,
    _NativeModelingResult,
    _NativeModelingInitialization,
    _NativeModelingInitializationAttempt,
    _NativeModelingStudy,
)
from pse._inspection import TableStream
from pse._runs import PreparedOperation, StudyHandle, Workspace
from pse._strategies import PreparedFlow, PreparedStrategy, _AnalysisDocument
from pse.contracts.authored import (
    AuthoredModelingDeclarationsRow,
    AuthoredFitCasesRow,
    AuthoredObservationsRow,
    AuthoredDatasetsRow,
)
from pse.contracts.documents import PointOverlay, SolveSettings

if TYPE_CHECKING:
    from pse._workflow import Runtime
from pse.contracts.enums import ModelingAnalysisRoute
from pse.contracts.identities import DeclarationId, FitId, InstanceId, RunId
from pse.contracts.values import ContentHash, SemanticId


class _DeclarationEdit(msgspec.Struct, frozen=True, forbid_unknown_fields=True):
    declarations: tuple[dict[str, object], ...]


class _FitData(msgspec.Struct, frozen=True, forbid_unknown_fields=True):
    fits: tuple[dict[str, object], ...]
    observations: tuple[dict[str, object], ...]
    datasets: tuple[dict[str, object], ...]


@attrs.frozen
class ModelingResult:
    """One original solve outcome plus independent physical model qualification."""

    _handle: _NativeModelingResult

    @property
    def accepted(self) -> bool:
        return self._handle.accepted

    @property
    def run_id(self) -> RunId:
        return RunId(SemanticId.from_hex(self._handle.run_id))

    @property
    def outcome_kind(self) -> str:
        return self._handle.outcome_kind

    @property
    def validation_error(self) -> DiagnosticReport | None:
        return self._handle.validation_error

    def attempt(self) -> NativeAttempt | None:
        """The unchanged native attempt, absent for constant evaluation or refusal."""
        return self._handle.attempt()

    def failure(self) -> DiagnosticReport | None:
        return self._handle.failure()

    def table(self, name: str) -> TableStream:
        """Export runtime.modeling_checks or runtime.modeling_reports with owned buffers."""
        return TableStream(self._handle.table(name))


@attrs.frozen
class ModelingNativeAnalysis:
    """Native analysis evidence; the generated relation defines its scientific scope."""

    _handle: _NativeModelingNativeAnalysis

    @property
    def relation(self) -> str:
        return self._handle.relation

    def attempts(self) -> tuple[NativeAttempt, ...]:
        return tuple(self._handle.attempt(i) for i in range(self._handle.attempt_count))

    def table(self) -> TableStream:
        """Source-coordinate IIS/rays/ranges or scaled-Jacobian certificates and limits."""
        return TableStream(self._handle.table())


@attrs.frozen
class ModelingElasticAttempt:
    """One diagnostic solve over explicit omitted rows and physical elastic weights."""

    _handle: _NativeModelingElasticAttempt

    @property
    def observation(self) -> str:
        return self._handle.observation

    @property
    def penalty(self) -> float | None:
        return self._handle.penalty

    def omitted(self) -> tuple[SemanticId, ...]:
        return tuple(SemanticId.from_hex(v) for v in self._handle.omitted())

    def result(self) -> ModelingResult | None:
        result = self._handle.result()
        return None if result is None else ModelingResult(result)

    def failure(self) -> DiagnosticReport | None:
        return self._handle.failure()


@attrs.frozen
class ModelingNonlinearExplanation:
    """Local diagnostic evidence under unchanged bounds; never a global infeasibility proof."""

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
        return tuple(SemanticId.from_hex(v) for v in self._handle.candidate_rows())

    def background_variables(self) -> tuple[SemanticId, ...]:
        return tuple(
            SemanticId.from_hex(v) for v in self._handle.background_variables()
        )

    def attempts(self) -> tuple[ModelingElasticAttempt, ...]:
        return tuple(ModelingElasticAttempt(v) for v in self._handle.attempts())


@attrs.frozen
class ModelingConformance:
    """Shared checks and completed fixture solves, including explicit coverage gaps."""

    _handle: _NativeModelingConformance

    @classmethod
    def pure(
        cls,
        documents: Sequence[Mapping[str, str]],
        physical: Mapping[str, str],
        settings: EngineSettings,
        *,
        maximum_fixtures: int = 1024,
        maximum_checks: int = 16384,
        limits: ModelingLimits | None = None,
    ) -> "ModelingConformance":
        """Run explicit pure fixtures without creating a workflow Runtime."""
        return cls(
            _NativeModelingConformance.pure(
                [dict(bundle) for bundle in documents],
                dict(physical),
                settings,
                maximum_fixtures=maximum_fixtures,
                maximum_checks=maximum_checks,
                limits=limits,
            )
        )

    @property
    def passed(self) -> bool:
        return self._handle.passed

    @property
    def complete(self) -> bool:
        return self._handle.complete

    def table(self) -> TableStream:
        return TableStream(self._handle.table())

    def findings(self) -> TableStream:
        return TableStream(self._handle.findings())

    def failure(self, ordinal: int) -> DiagnosticReport:
        return self._handle.failure(ordinal)

    def trajectory(self, fixture_id: DeclarationId) -> "ModelingTrajectory":
        return ModelingTrajectory(self._handle.trajectory(fixture_id.to_hex()))

    def initialization(self, fixture_id: DeclarationId) -> "ModelingInitialization":
        return ModelingInitialization(self._handle.initialization(fixture_id.to_hex()))

    def fixtures(self) -> tuple[DeclarationId, ...]:
        return tuple(
            DeclarationId(SemanticId.from_hex(value))
            for value in self._handle.fixtures()
        )

    def fixture_statuses(self) -> TableStream:
        """All discovered fixtures, including identities beyond a detailed check limit."""
        return TableStream(self._handle.fixture_statuses())

    def result(self, fixture_id: DeclarationId) -> ModelingResult:
        return ModelingResult(self._handle.result(fixture_id.to_hex()))


@attrs.frozen
class ModelingInitializationAttempt:
    """One unchanged specification attempt, retaining preparation and interruption failures."""

    _handle: _NativeModelingInitializationAttempt

    @property
    def kind(self) -> str:
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
    """Ordered initialization history; only an accepted original model commits values."""

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
        values = self._handle.committed_values()
        return (
            None
            if values is None
            else {SemanticId.from_hex(k): v for k, v in values.items()}
        )


@attrs.frozen
class ModelingStudy:
    """Independent outcomes with explicit accepted-predecessor dependencies."""

    _handle: _NativeModelingStudy

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

    def result(self, index: int) -> ModelingResult | None:
        handle = self._handle.result(index)
        return None if handle is None else ModelingResult(handle)

    def failure(self, index: int) -> DiagnosticReport | None:
        return self._handle.failure(index)


@attrs.frozen
class ModelingDiagnostics:
    """Bounded point-specific evidence with named source coordinates."""

    _handle: _NativeModelingDiagnostics

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
        return tuple(SemanticId.from_hex(v) for v in rows), tuple(
            SemanticId.from_hex(v) for v in columns
        )

    def singular_modes(
        self,
    ) -> tuple[tuple[float, tuple[float, ...], tuple[float, ...]], ...]:
        return tuple(
            (value, tuple(left), tuple(right))
            for value, left, right in self._handle.singular_modes()
        )


@attrs.frozen
class ModelingTrajectory:
    """Completed physical samples and the actual native termination, including partial runs."""

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
    def termination(self) -> str:
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
    def stop(self) -> str:
        return self._handle.stop

    def ids(self) -> tuple[SemanticId, ...]:
        return tuple(SemanticId.from_hex(value) for value in self._handle.ids())

    def result(self, index: int) -> ModelingDiagnostics | None:
        handle = self._handle.result(index)
        return None if handle is None else ModelingDiagnostics(handle)

    def failure(self, index: int) -> DiagnosticReport | None:
        return self._handle.failure(index)


@attrs.frozen
class ModelingPackage:
    """Immutable authored package sharing the deployment's compiler and execution service."""

    _handle: _NativeModelingPackage

    def with_fit_data(
        self,
        fits: tuple[AuthoredFitCasesRow, ...],
        observations: tuple[AuthoredObservationsRow, ...],
        datasets: tuple[AuthoredDatasetsRow, ...],
    ) -> "ModelingPackage":
        """Admit registry-owned measurement bindings in an immutable package view."""
        converter = codec.converter()
        converter.register_unstructure_hook(SemanticId, SemanticId.to_hex)
        converter.register_unstructure_hook(ContentHash, ContentHash.to_prefixed)
        data = _FitData(
            tuple(converter.unstructure(row) for row in fits),
            tuple(converter.unstructure(row) for row in observations),
            tuple(converter.unstructure(row) for row in datasets),
        )
        return ModelingPackage(self._handle.with_fit_data(codec.encode_json(data)))

    def prepare_fit(
        self,
        fit_id: FitId,
        settings: SolveSettings,
        simulations: Mapping[InstanceId, SimulationSettings] | None = None,
        *,
        modes: Mapping[InstanceId, Sequence[ModelingModeSettings]] | None = None,
        rank_tolerance: float = 1e-8,
        max_cells: int = 1000000,
    ) -> "PreparedOperation":
        """Compile shared parameters over authored algebraic or integrated experiments.

        Experiment settings are keyed by the experiment's instance: its ``experiment_id``.
        """
        profiles = [(key.to_hex(), value) for key, value in (simulations or {}).items()]
        return PreparedOperation(
            self._handle.prepare_fit(
                fit_id.to_hex(),
                codec.encode_json(settings),
                profiles,
                modes=[
                    (key.to_hex(), list(values))
                    for key, values in (modes or {}).items()
                ],
                rank_tolerance=rank_tolerance,
                max_cells=max_cells,
            )
        )

    def with_limits(self, limits: ModelingLimits) -> "ModelingPackage":
        """Share the admitted revision while selecting explicit expansion limits."""
        return ModelingPackage(self._handle.with_limits(limits))

    def diagnose_samples(
        self,
        case_id: DeclarationId,
        settings: SolveSettings,
        diagnostics: ModelingDiagnosticSettings,
        samples: tuple[tuple[SemanticId, dict[SemanticId, float]], ...],
        *,
        maximum_samples: int = 128,
        time_limit: float = 60.0,
    ) -> ModelingDiagnosticSamples:
        """Inspect explicit candidate overrides while preserving frozen case inputs."""
        return ModelingDiagnosticSamples(
            self._handle.diagnose_samples(
                case_id.to_hex(),
                codec.encode_json(settings),
                diagnostics,
                [
                    (
                        name.to_hex(),
                        {key.to_hex(): value for key, value in point.items()},
                    )
                    for name, point in samples
                ],
                maximum_samples,
                time_limit,
            )
        )

    def explain_nonlinear(
        self,
        case_id: DeclarationId,
        settings: SolveSettings,
        nominals: dict[SemanticId, float],
        *,
        penalty_tolerance: float,
        maximum_attempts: int,
        time_limit: float,
    ) -> ModelingNonlinearExplanation:
        """Explain local infeasibility by bounded deletion on the l1 exact penalty."""
        return ModelingNonlinearExplanation(
            self._handle.explain_nonlinear(
                case_id.to_hex(),
                codec.encode_json(settings),
                {key.to_hex(): value for key, value in nominals.items()},
                penalty_tolerance=penalty_tolerance,
                maximum_attempts=maximum_attempts,
                time_limit=time_limit,
            )
        )

    def prepare_simulation(
        self,
        case_id: DeclarationId,
        settings: SimulationSettings,
        *,
        modes: tuple[ModelingModeSettings, ...] | None = None,
    ) -> PreparedOperation:
        """Prepare an authored trajectory for cancellable execution and publication."""
        return PreparedOperation(
            self._handle.prepare_simulation(
                case_id.to_hex(), settings, None if modes is None else list(modes)
            )
        )

    def simulate(
        self,
        case_id: DeclarationId,
        settings: SimulationSettings,
        *,
        modes: tuple[ModelingModeSettings, ...] | None = None,
    ) -> ModelingTrajectory:
        """Integrate the authored case with the existing native dynamics engine."""
        return ModelingTrajectory(
            self._handle.simulate(
                case_id.to_hex(), settings, None if modes is None else list(modes)
            )
        )

    def diagnose(
        self,
        case_id: DeclarationId,
        settings: SolveSettings,
        diagnostics: ModelingDiagnosticSettings,
    ) -> ModelingDiagnostics:
        """Inspect the declared candidate without requesting a native solve."""
        return ModelingDiagnostics(
            self._handle.diagnose(
                case_id.to_hex(), codec.encode_json(settings), diagnostics
            )
        )

    def diagnose_linear(
        self,
        case_id: DeclarationId,
        settings: SolveSettings,
        *,
        rays: bool = False,
        iis: bool = False,
        ranging: bool = False,
        relaxation: tuple[float, float, float] | None = None,
        lower_penalties: dict[SemanticId, float] | None = None,
        upper_penalties: dict[SemanticId, float] | None = None,
        row_penalties: dict[SemanticId, float] | None = None,
        maximum_entries: int = 100_000,
    ) -> ModelingNativeAnalysis:
        """Run native affine-model analyses, retaining MIP relaxation scope explicitly.

        Relaxation penalties name lower bounds, upper bounds and rows, in that order.
        Negative penalties forbid violation. Each optional local map must name every
        coordinate in its family; no physical weight is inferred.
        """

        def encoded(values: dict[SemanticId, float] | None) -> dict[str, float] | None:
            return (
                None
                if values is None
                else {key.to_hex(): value for key, value in values.items()}
            )

        return ModelingNativeAnalysis(
            self._handle.diagnose_linear(
                case_id.to_hex(),
                codec.encode_json(settings),
                rays=rays,
                iis=iis,
                ranging=ranging,
                relaxation=relaxation,
                lower_penalties=encoded(lower_penalties),
                upper_penalties=encoded(upper_penalties),
                row_penalties=encoded(row_penalties),
                maximum_entries=maximum_entries,
            )
        )

    def diagnose_jacobian(
        self,
        case_id: DeclarationId,
        settings: SolveSettings,
        *,
        maximum_rows: int = 32,
        maximum_entries: int = 100_000,
        maximum_attempts: int = 64,
        multiplier_bound: float = 10.0,
        tolerance: float = 1e-7,
        rank_relative: float = 1e-8,
    ) -> ModelingNativeAnalysis:
        """Bounded LP/MILP evidence about the local scaled Jacobian, not nonlinear feasibility."""
        return ModelingNativeAnalysis(
            self._handle.diagnose_jacobian(
                case_id.to_hex(),
                codec.encode_json(settings),
                maximum_rows=maximum_rows,
                maximum_entries=maximum_entries,
                maximum_attempts=maximum_attempts,
                multiplier_bound=multiplier_bound,
                tolerance=tolerance,
                rank_relative=rank_relative,
            )
        )

    def with_declarations(
        self, declarations: Sequence[AuthoredModelingDeclarationsRow]
    ) -> "ModelingPackage":
        """Admit edited generated declarations while retaining the original revision."""
        converter = codec.converter()
        converter.register_unstructure_hook(SemanticId, SemanticId.to_hex)
        converter.register_unstructure_hook(ContentHash, ContentHash.to_prefixed)
        data = _DeclarationEdit(
            tuple(converter.unstructure(row) for row in declarations)
        )
        return ModelingPackage(self._handle.with_declarations(codec.encode_json(data)))

    def declarations(self) -> tuple[AuthoredModelingDeclarationsRow, ...]:
        return tuple(
            codec.converter().structure(
                msgspec.json.decode(self._handle.declarations()),
                list[AuthoredModelingDeclarationsRow],
            )
        )

    def inspect(
        self, case_id: DeclarationId, settings: SolveSettings
    ) -> dict[str, object]:
        """Inspect instantiated member lineage and declared topology before execution."""
        return codec.decode_json(
            self._handle.inspect(case_id.to_hex(), codec.encode_json(settings)),
            _AnalysisDocument,
        ).payload

    def prepare_flow(
        self,
        case_id: DeclarationId,
        selection: Mapping[str, object],
        settings: SolveSettings,
    ) -> PreparedFlow:
        """Project explicitly selected authored nodes, ports and connection policies."""
        return PreparedFlow(
            self._handle.prepare_flow(
                case_id.to_hex(),
                codec.encode_json(_AnalysisDocument(dict(selection))),
                codec.encode_json(settings),
            )
        )

    def prepare_recycle(
        self,
        case_id: DeclarationId,
        selection: Mapping[str, object],
        request: Mapping[str, object],
        settings: SolveSettings,
    ) -> PreparedStrategy:
        """Compile explicit causal directions from this immutable authored model."""
        return PreparedStrategy(
            self._handle.prepare_recycle(
                case_id.to_hex(),
                codec.encode_json(_AnalysisDocument(dict(selection))),
                codec.encode_json(_AnalysisDocument(dict(request))),
                codec.encode_json(settings),
            )
        )

    def prepare_block_initialization(
        self,
        case_id: DeclarationId,
        settings: SolveSettings,
        stages: Sequence[Mapping[SemanticId, float]],
    ) -> PreparedStrategy:
        """Prepare ordered blocks and transactional overlays over original bindings."""
        return PreparedStrategy(
            self._handle.prepare_block_initialization(
                case_id.to_hex(),
                codec.encode_json(settings),
                [{k.to_hex(): v for k, v in stage.items()} for stage in stages],
            )
        )

    def prepare_solve(
        self,
        case_id: DeclarationId,
        settings: SolveSettings,
        *,
        route: ModelingAnalysisRoute = ModelingAnalysisRoute.STEADY,
    ) -> PreparedOperation:
        """Prepare an authored algebraic case for owned execution and publication."""
        return PreparedOperation(
            self._handle.prepare_solve(
                case_id.to_hex(), codec.encode_json(settings), route=route.value
            )
        )

    def solve_case(
        self,
        case_id: DeclarationId,
        settings: SolveSettings,
        *,
        route: ModelingAnalysisRoute = ModelingAnalysisRoute.STEADY,
    ) -> ModelingResult:
        """Use source fixture specifications and model starts through the native solver pipeline."""
        return ModelingResult(
            self._handle.solve_case(
                case_id.to_hex(), codec.encode_json(settings), route=route.value
            )
        )

    def initialize(
        self,
        case_id: DeclarationId,
        settings: SolveSettings,
        *,
        stages: tuple[str, ...] = (),
        homotopy: bool = False,
        initial_step: float = 0.25,
        minimum_step: float = 1e-6,
        growth: float = 1.5,
        maximum_attempts: int = 128,
        time_limit: float = 60.0,
    ) -> ModelingInitialization:
        """Run bounded stages and adaptive homotopy with original-model qualification."""
        return ModelingInitialization(
            self._handle.initialize(
                case_id.to_hex(),
                codec.encode_json(settings),
                stages=list(stages),
                homotopy=homotopy,
                initial_step=initial_step,
                minimum_step=minimum_step,
                growth=growth,
                maximum_attempts=maximum_attempts,
                time_limit=time_limit,
            )
        )

    @overload
    def study(
        self,
        case_ids: tuple[DeclarationId, ...],
        settings: SolveSettings,
        *,
        predecessors: tuple[int | None, ...] = (),
        maximum_points: int = 1024,
    ) -> ModelingStudy: ...

    @overload
    def study(
        self,
        case_ids: tuple[DeclarationId, ...],
        settings: SolveSettings,
        *,
        predecessors: tuple[int | None, ...] = (),
        maximum_points: int = 1024,
        runtime: "Runtime",
        workspace: Workspace,
        overlays: tuple[PointOverlay, ...] = (),
        max_tries: int = 1,
        priority: int = 0,
    ) -> StudyHandle: ...

    def study(
        self,
        case_ids: tuple[DeclarationId, ...],
        settings: SolveSettings,
        *,
        predecessors: tuple[int | None, ...] = (),
        maximum_points: int = 1024,
        runtime: "Runtime | None" = None,
        workspace: Workspace | None = None,
        overlays: tuple[PointOverlay, ...] = (),
        max_tries: int = 1,
        priority: int = 0,
    ) -> "ModelingStudy | StudyHandle":
        """Execute authored cases; failures do not suppress independent points.

        Without ``runtime`` the points run in order in this process and the
        returned study holds their results (the library path). With a durable
        ``runtime`` the study is stored in its operational store and run by
        workers: a point with a predecessor waits for it and starts from its
        stored solution, and the study publishes once in ``workspace``.

        Args:
            case_ids: The authored case of each point.
            settings: The solve settings of every point.
            predecessors: For each point, the earlier point it starts from.
            maximum_points: The largest in-process study accepted.
            runtime: A durable runtime whose workers run the study.
            workspace: The registered workspace the study publishes in.
            overlays: For each point, the case values and parameters it replaces.
            max_tries: How often each point and the finalization may be tried.
            priority: The priority of the study's jobs.

        Returns:
            The in-process study, or the durable study's handle.
        """
        if runtime is None:
            if workspace is not None or overlays:
                raise ValueError(
                    "workspace and overlays select a durable study; pass runtime"
                )
            return ModelingStudy(
                self._handle.study(
                    [case.to_hex() for case in case_ids],
                    codec.encode_json(settings),
                    predecessors=list(predecessors),
                    maximum_points=maximum_points,
                )
            )
        if workspace is None:
            raise ValueError("a durable study publishes in a workspace")
        return StudyHandle(
            self._handle.start_study(
                runtime._handle,  # noqa: SLF001 - same native boundary
                msgspec.json.encode(workspace),
                [case.to_hex() for case in case_ids],
                codec.encode_json(settings),
                predecessors=list(predecessors),
                overlays=[codec.encode_json(overlay) for overlay in overlays],
                max_tries=max_tries,
                priority=priority,
            )
        )

    def conform(
        self,
        settings: SolveSettings,
        *,
        maximum_fixtures: int = 1024,
        maximum_checks: int = 16384,
        derivative_cells: int = 100000,
        derivative_step: float = 1e-6,
        derivative_tolerance: float = 1e-4,
        fixture_policies: Mapping[DeclarationId, ModelingFixturePolicy] | None = None,
    ) -> ModelingConformance:
        """Discover authored tests and run bounded shared checks without importing IDAES."""
        return ModelingConformance(
            self._handle.conform(
                codec.encode_json(settings),
                maximum_fixtures=maximum_fixtures,
                maximum_checks=maximum_checks,
                derivative_cells=derivative_cells,
                derivative_step=derivative_step,
                derivative_tolerance=derivative_tolerance,
                fixture_policies={
                    key.to_hex(): value
                    for key, value in (fixture_policies or {}).items()
                },
            )
        )
