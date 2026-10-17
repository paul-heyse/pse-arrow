# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Joined computation and exact canonical execution handles."""

from collections.abc import Mapping, Sequence
from types import TracebackType
from typing import Self, TypeAlias

import attrs
import pyarrow as pa

from pse import codec
from pse._analyses import Analysis
from pse._artifacts import ScientificArtifact
from pse._build import (
    DiagnosticReport,
    _NativeLiveSimulationCursor,
    _NativeLiveSimulationEvent,
    _NativePreparedOperation,
    _NativeProgressStream,
    _NativeRunHandle,
    _NativeRunResult,
    _NativeScientificReplay,
    _NativeStart,
    _NativeStoredResult,
    _NativeStudyHandle,
)
from pse._inspection import TableStream
from pse.contracts.documents import (
    AdviceMetric,
    AdviceObservation,
    AnalysisControls,
    AttemptSummary,
    ComparisonPolicy,
    Completion,
    CompositionRequest,
    DiagnosticOrigin,
    EligibilityDocument,
    FitProfileDocument,
    FrozenAdvice,
    InvocationComparisonDocument,
    InvocationEvidenceDocument,
    ManagedStudyDiagnosticsDocument,
    NumericalStrategyDocument,
    PointStatus,
    ProgressCheckpointDocument,
    ProgressCoverage,
    ProgressEventDocument,
    ReplayDocument,
    RouteDocument,
    RuntimeAccuracyGoalAssessmentsRow,
    SolverAdviceInputs,
    StudyCancel,
    StudyResults,
    StudyStatus,
    StudyWaitControls,
)
from pse.contracts.identities import (
    RunId,
    StudyId,
)
from pse.contracts.values import ContentHash, SemanticId
from pse.diagnostics import OperationObservations

StudyPointStatus: TypeAlias = PointStatus


@attrs.frozen
class StoredResult:
    """Exact persisted occurrence; scientific rows reopen from canonical storage."""

    _handle: _NativeStoredResult

    @property
    def run_id(self) -> RunId:
        """Producing scientific execution identity."""
        return RunId(SemanticId.from_hex(self._handle.run_id))

    @property
    def usable(self) -> bool:
        """Read original candidate permissions from retained completion."""
        return self._handle.usable

    @property
    def completion(self) -> Completion:
        """Original bounded scientific completion, without report hydration."""
        return codec.decode_json(self._handle.completion(), Completion)

    def diagnostics(self) -> tuple[DiagnosticReport, ...]:
        """Original scientific boundary observations."""
        return tuple(self._handle.diagnostics())

    @property
    def run_key(self) -> str:
        return self._handle.run_key

    @property
    def attempt_key(self) -> str:
        return self._handle.attempt_key

    def table(
        self, relation: str, *, start: int = 0, end: int = (1 << 64) - 1
    ) -> TableStream:
        return TableStream(self._handle.table(relation, start=start, end=end))

    def attempt_record(self) -> TableStream:
        """Recorded lifecycle and completion; terminal class alone is not usability."""
        return TableStream(self._handle.attempt_record())

    def progress(self) -> "ProgressStream":
        return ProgressStream(self._handle.progress())

    def export(
        self,
        relation: str,
        destination: str,
        *,
        start: int = 0,
        end: int = (1 << 64) - 1,
    ) -> None:
        self._handle.export(relation, destination, start=start, end=end)


@attrs.frozen
class PreparedOperation:
    """Immutable authored solve, simulation or fitting request."""

    _handle: _NativePreparedOperation

    @property
    def advice_inputs(self) -> SolverAdviceInputs | None:
        """Actual comparable cohort and baseline, when original equivalence is known."""
        return self._handle.advice_inputs

    def freeze_advice(
        self, metric: AdviceMetric, observations: Sequence[AdviceObservation]
    ) -> FrozenAdvice:
        """Freeze descriptive support with Rust's conservative fallback rules."""
        return self._handle.freeze_advice(
            codec.encode_json(metric), codec.encode_json(observations)
        )

    def dependency_analysis(self, controls: AnalysisControls) -> Analysis:
        """Persist original incidence and execution dependencies before dispatch."""
        return Analysis(self._handle.dependency_analysis(codec.encode_json(controls)))

    @property
    def identity(self) -> ContentHash:
        """Exact prepared source and profile identity."""
        return ContentHash.from_prefixed(self._handle.identity)

    @property
    def route(self) -> RouteDocument:
        """Admitted algebraic route, before native execution."""
        return self._handle.route

    @property
    def eligibility(self) -> tuple[EligibilityDocument, ...]:
        """Typed eligibility of every assessed backend, with registry reason codes."""
        return tuple(self._handle.eligibility)

    @property
    def strategy_profile(self) -> ContentHash:
        """Exact effective native profile identity for numerical declarations."""
        return ContentHash.from_prefixed(self._handle.strategy_profile)

    @property
    def composition_request(self) -> CompositionRequest:
        """Requested composition with preserved branch, start and work constraints."""
        return self._handle.composition_request

    @property
    def numerical_strategy(self) -> NumericalStrategyDocument:
        """Actual finite execution declaration, projected by Rust."""
        return codec.decode_json(
            self._handle.numerical_strategy, NumericalStrategyDocument
        )

    def with_numerical_strategy(
        self,
        declaration: NumericalStrategyDocument,
        rungs: Sequence["PreparedOperation"],
    ) -> "PreparedOperation":
        """Bind admitted original-system profiles to one declared numerical task."""
        return PreparedOperation(
            self._handle.with_numerical_strategy(
                codec.encode_json(declaration),
                [rung._handle for rung in rungs],  # noqa: SLF001 - same native boundary
            )
        )

    def with_artifact_start(self, artifact: ScientificArtifact) -> "PreparedOperation":
        """Acquire a retained qualified seed and check this target's compatibility."""
        return PreparedOperation(
            self._handle.with_artifact_start(artifact._handle)  # noqa: SLF001 - same native boundary
        )

    def with_start(self, seed: _NativeStart) -> "PreparedOperation":
        """Select a compatible owned numerical seed for an algebraic solve."""
        return PreparedOperation(self._handle.with_start(seed))

    def with_primal_start(
        self, values: Mapping[SemanticId, float]
    ) -> "PreparedOperation":
        """Select every original free coordinate without retaining native state."""
        return PreparedOperation(
            self._handle.with_primal_start(
                {key.to_hex(): value for key, value in values.items()}
            )
        )

    def start(self) -> "RunHandle":
        """Start one admitted native operation."""
        return RunHandle(self._handle.start())

    def start_live(
        self, *, capacity: int = 16, compact: bool = False
    ) -> tuple["RunHandle", "LiveSimulationCursor"]:
        """Subscribe before a simulation starts; each sample is provisional.

        Compact output omits full terminal payload retention. Consume or close the
        cursor while the run is active: its bounded queue applies backpressure.
        Explicit sample sensitivity requests require Full retention.
        """
        run, cursor = self._handle.start_live(capacity=capacity, compact=compact)
        return RunHandle(run), LiveSimulationCursor(cursor)


@attrs.frozen
class LiveSimulationEvent:
    """Provisional physical rows, actual recording ACK, or joined completion."""

    _handle: _NativeLiveSimulationEvent

    @property
    def kind(self) -> str:
        return self._handle.kind

    @property
    def accuracy_attempt(self) -> int | None:
        return self._handle.accuracy_attempt

    @property
    def sample_ordinal(self) -> int | None:
        return self._handle.sample_ordinal

    @property
    def canonical_attempt_key(self) -> str | None:
        return self._handle.canonical_attempt_key

    @property
    def recorded_rows(self) -> int | None:
        return self._handle.recorded_rows

    @property
    def produced_samples(self) -> int | None:
        return self._handle.produced_samples

    @property
    def completed_time(self) -> float | None:
        return self._handle.completed_time

    @property
    def checks_complete(self) -> bool | None:
        return self._handle.checks_complete

    @property
    def checks_satisfied(self) -> bool | None:
        return self._handle.checks_satisfied

    @property
    def termination(self) -> str | None:
        return self._handle.termination

    def table(self) -> TableStream | None:
        """The original checked provisional rows, when this event is a sample."""
        stream = self._handle.table()
        return None if stream is None else TableStream(stream)

    def result(self) -> "RunResult | None":
        """Joined science/recording result, available only on the joined event."""
        result = self._handle.result()
        return None if result is None else RunResult(result)

    def diagnostic(self) -> DiagnosticReport | None:
        return self._handle.diagnostic()


@attrs.frozen
class LiveSimulationCursor:
    """Bounded subscription; closing it detaches output without cancelling science."""

    _handle: _NativeLiveSimulationCursor

    def close(self) -> None:
        self._handle.close()

    def __iter__(self) -> Self:
        return self

    def __next__(self) -> LiveSimulationEvent:
        event = self._handle.next_event()
        if event is None:
            raise StopIteration
        return LiveSimulationEvent(event)

    def __aiter__(self) -> Self:
        return self

    async def __anext__(self) -> LiveSimulationEvent:
        event = await self._handle.next_async()
        if event is None:
            raise StopAsyncIteration
        return LiveSimulationEvent(event)

    def __enter__(self) -> Self:
        return self

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_value: BaseException | None,
        traceback: TracebackType | None,
    ) -> None:
        self.close()


@attrs.frozen
class RunResult:
    """Joined immutable outcome; failed and unattempted steps remain inspectable."""

    _handle: _NativeRunResult

    def diagnostic_origin(self, step: int | None = None) -> DiagnosticOrigin:
        """Retain the originating invocation/failure for explicit follow-up diagnostics."""
        return self._handle.diagnostic_origin(step)

    def replay(self) -> "ScientificReplay":
        """Retain exact inputs and observed decisions without mutable history."""
        return ScientificReplay(self._handle.replay())

    def available_start(self, step: int | None = None) -> _NativeStart | None:
        """Return available numerical seed data and its provenance."""
        return self._handle.available_start(step)

    @property
    def run_id(self) -> RunId:
        """Unique execution identity."""
        return RunId(SemanticId.from_hex(self._handle.run_id))

    @property
    def canonical_run_key(self) -> str | None:
        """Exact persistent run key; absent for explicit ephemeral execution."""
        return self._handle.canonical_run_key

    @property
    def canonical_attempt_key(self) -> str | None:
        """Exact producing attempt key, without truncation or ID reinterpretation."""
        return self._handle.canonical_attempt_key

    @property
    def usable(self) -> bool:
        """Whether every requested candidate satisfies its final usability policy."""
        return self._handle.usable

    @property
    def fit_profiles(self) -> FitProfileDocument | None:
        """Retained profile chains, with worker failures and actual parallelism."""
        data = self._handle.fit_profiles()
        return None if data is None else codec.decode_json(data, FitProfileDocument)

    @property
    def completion(self) -> Completion:
        """Read the immutable joined assessment without evaluating the model again."""
        return codec.decode_json(self._handle.completion(), Completion)

    @property
    def attempt_summary(self) -> AttemptSummary:
        """Bounded retained observations and decisions; performs no numerical work."""
        return self._handle.attempt_summary()

    @property
    def invocation_evidence(self) -> InvocationEvidenceDocument:
        """Read qualification/readiness endpoints and retained attempt evidence."""
        return self._handle.invocation_evidence()

    def invocation_arrow(self, *, memory_bytes: int = 16 << 20) -> pa.RecordBatch:
        """Project the actual joined invocation with independently missing endpoints."""
        return pa.record_batch(self._handle.invocation_arrow(memory_bytes))

    def explain_invocation(
        self, *, policy: ComparisonPolicy | None = None
    ) -> InvocationComparisonDocument:
        """Explain actual work and limitations without reevaluating the model."""
        return self._handle.explain_invocation(policy=policy)

    def compare_invocation(
        self, other: Self, *, policy: ComparisonPolicy | None = None
    ) -> InvocationComparisonDocument:
        """Compare actual products, work, outcomes and clock scopes of two results."""
        return self._handle.compare_invocation(other._handle, policy=policy)

    @property
    def accuracy_goals(
        self,
    ) -> tuple[RuntimeAccuracyGoalAssessmentsRow, ...]:
        """Retained engineering goal outcomes; reading this performs no solver work."""
        return tuple(self.completion.accuracy_goals)

    def diagnostics(self) -> tuple[DiagnosticReport, ...]:
        """Structured native admission and execution failures, preserving causes."""
        return tuple(self._handle.diagnostics())

    def tables(self) -> tuple[str, ...]:
        """Declared result and reconstruction-source relation names."""
        return tuple(self._handle.tables())

    def export_fit_parameters(self) -> TableStream:
        """Export canonical typed cells with producing fit/run/source identities.

        Requires a freshly qualified, locally identifiable estimate. The export
        can be admitted into a fitted bank with fit lineage; it performs no write.
        """
        return TableStream(self._handle.export_fit_parameters())

    def table(self, name: str) -> TableStream:
        """Return a one-consumption stream owning its final Arrow buffers."""
        return TableStream(self._handle.table(name))


@attrs.frozen
class ScientificReplay:
    """Immutable input export and tolerant rerun through the original supervisor."""

    _handle: _NativeScientificReplay

    def document(self, *, bytes: int = 16 * 1024 * 1024) -> ReplayDocument:
        """Export complete inputs and observed coverage within the given allocation."""
        return self._handle.document(bytes)

    def rerun(self) -> "RunHandle":
        """Start a new execution with its own identity and allowance."""
        return RunHandle(self._handle.rerun())

    def native_continuation(self) -> "RunHandle":
        """Request an actual library checkpoint; currently refuses explicitly."""
        return RunHandle(self._handle.native_continuation())


@attrs.frozen
class RunHandle:
    """Blocking and asyncio access to the same supervised native job."""

    _handle: _NativeRunHandle

    def observations(self) -> OperationObservations | None:
        """Read diagnostics without retaining this handle's cancellation lease."""
        reader = self._handle.observations()
        return None if reader is None else OperationObservations(reader)

    def cancel(self) -> None:
        """Request stop; join and terminal report ownership remain native."""
        self._handle.cancel()

    @property
    def canonical_run_key(self) -> str | None:
        """Exact persistent run key; absent for explicit ephemeral execution."""
        return self._handle.canonical_run_key

    @property
    def canonical_attempt_key(self) -> str | None:
        """Exact producing attempt key, without truncation or ID reinterpretation."""
        return self._handle.canonical_attempt_key

    def wait(self) -> RunResult:
        """Release Python while waiting; join cancellation before a signal escapes."""
        return RunResult(self._handle.wait())

    async def wait_async(self) -> RunResult:
        """Request native stop on cancellation; retain the report for later waiters."""
        return RunResult(await self._handle.wait_async())

    def result(self) -> RunResult | None:
        """Nonblocking terminal snapshot."""
        result = self._handle.result()
        return None if result is None else RunResult(result)

    def progress(self) -> tuple[tuple[ProgressEventDocument, ...], int]:
        """Observe bounded typed native events and actual dropped-event count."""
        events, dropped = self._handle.progress()
        return tuple(events), dropped

    @property
    def progress_count(self) -> tuple[int, int]:
        """Retained native events and dropped-event count."""
        return self._handle.progress_count

    def current_progress(self) -> ProgressCheckpointDocument:
        """Read whole current progress and coverage, including history zero."""
        return self._handle.current_progress()


@attrs.frozen
class ProgressStream:
    """An admitted terminal attempt's progress, one recorded batch at a time.

    Live observations are available on ``RunHandle``. Closing this history stream
    ends unread database work and releases its canonical result protection.
    """

    _handle: _NativeProgressStream

    @property
    def run_key(self) -> str:
        """Exact opaque producing run key."""
        return self._handle.run_key

    @property
    def attempt_key(self) -> str:
        """Exact opaque producing terminal attempt key."""
        return self._handle.attempt_key

    @property
    def coverage(self) -> ProgressCoverage:
        """Original sampling loss and writer coverage, separate from science."""
        return self._handle.coverage

    def __iter__(self) -> Self:
        return self

    def __next__(self) -> ProgressEventDocument:
        event = self._handle.next_event()
        if event is None:
            raise StopIteration
        return event

    def close(self) -> None:
        """Stop reading; a waiting read ends and later reads find nothing."""
        self._handle.close()

    def __enter__(self) -> Self:
        return self

    def __exit__(
        self,
        _exc_type: type[BaseException] | None,
        _exc_value: BaseException | None,
        _traceback: TracebackType | None,
    ) -> None:
        self.close()


@attrs.frozen
class StudyHandle:
    """A canonical study: occurrence status and exact retained result handles."""

    _handle: _NativeStudyHandle

    def diagnostic_selection(self) -> ManagedStudyDiagnosticsDocument:
        """Read bounded actual worker references for explicitly selected detail."""
        return self._handle.diagnostic_selection()

    @property
    def study_id(self) -> StudyId:
        """The study identity."""
        return StudyId(SemanticId.from_hex(self._handle.study_id))

    def status(self) -> StudyStatus:
        """Read exact canonical occurrence status and lineage."""
        return codec.decode_json(self._handle.status(), StudyStatus)

    def cancel(self) -> StudyCancel:
        """Cancel the points that have not started and stop the running tries.

        Completed observations retain their actual canonical result identities.
        """
        return codec.decode_json(self._handle.cancel(), StudyCancel)

    def result(self) -> StudyResults | None:
        """The admitted parent and exact occurrence results; ``None`` before."""
        published = self._handle.result()
        return None if published is None else codec.decode_json(published, StudyResults)

    def wait(self, *, controls: StudyWaitControls | None = None) -> StudyResults:
        """Wait for admitted canonical results under Rust-owned timing."""
        return codec.decode_json(
            self._handle.wait(
                controls=None if controls is None else codec.encode_json(controls)
            ),
            StudyResults,
        )
