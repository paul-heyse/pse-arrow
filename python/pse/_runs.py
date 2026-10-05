# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Source-independent joined computation and publication handles."""

from collections.abc import Mapping, Sequence
from types import TracebackType
from typing import Self, TypeAlias

import attrs

from pse import codec, contracts
from pse._build import (
    DiagnosticReport,
    _NativePreparedOperation,
    _NativeProgressStream,
    _NativePublicationAttempt,
    _NativeRunHandle,
    _NativeRunResult,
    _NativeStart,
    _NativeStudyHandle,
)
from pse._inspection import TableStream
from pse.contracts.documents import (
    Completion,
    CompositionRequest,
    EligibilityDocument,
    FitProfileDocument,
    NumericalStrategyDocument,
    PointStatus,
    ProgressEventDocument,
    Published,
    RouteDocument,
    StudyCancel,
    StudyStatus,
    StudyWaitControls,
    Workspace,
)
from pse.contracts.identities import (
    AttemptId,
    PublicationId,
    RunId,
    StudyId,
)
from pse.contracts.values import ContentHash, SemanticId

StudyPointStatus: TypeAlias = PointStatus


@attrs.frozen
class PreparedOperation:
    """Immutable authored solve, simulation or fitting request."""

    _handle: _NativePreparedOperation

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


@attrs.frozen
class PublicationTicket:
    """Serialized publication request, saved before any effect.

    Settle it after a commit whose outcome is unknown.
    """

    json: bytes


@attrs.frozen
class PublicationAttempt:
    """Explicit single-use publication of immutable results."""

    _handle: _NativePublicationAttempt

    @property
    def ticket(self) -> PublicationTicket:
        """Save before commit; it remains available after the command is consumed."""
        return PublicationTicket(self._handle.ticket())

    @property
    def attempt_id(self) -> AttemptId:
        """The durable attempt published: the publication attempt itself."""
        return AttemptId(SemanticId.from_hex(self._handle.attempt_id))

    @property
    def publication_id(self) -> PublicationId:
        """Identity of the proposed publication."""
        return PublicationId(SemanticId.from_hex(self._handle.publication_id))

    def commit(self) -> Published:
        """Register the intent, write the members and commit once; never retried.

        A conflict raises; re-prepare with the same ``publication_id`` against the new
        head. An unresolved outcome raises; settle the ticket.
        """
        return codec.decode_json(self._handle.commit(), Published)


@attrs.frozen
class RunResult:
    """Joined immutable outcome; failed and unattempted steps remain inspectable."""

    _handle: _NativeRunResult

    def available_start(self, step: int | None = None) -> _NativeStart | None:
        """Return available numerical seed data and its provenance."""
        return self._handle.available_start(step)

    @property
    def run_id(self) -> RunId:
        """Unique execution identity."""
        return RunId(SemanticId.from_hex(self._handle.run_id))

    @property
    def attempt_id(self) -> AttemptId | None:
        """Durable attempt that recorded this run; ``None`` for an ephemeral run."""
        attempt = self._handle.attempt_id
        return None if attempt is None else AttemptId(SemanticId.from_hex(attempt))

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
    def accuracy_goals(
        self,
    ) -> tuple[contracts.runtime.RuntimeAccuracyGoalAssessmentsRow, ...]:
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

    def prepare_publication(
        self,
        workspace: Workspace,
        *,
        parent: PublicationId | None = None,
        publication_id: PublicationId | None = None,
    ) -> PublicationAttempt:
        """Prepare the publication of this durable run in ``workspace``; no write.

        Args:
            workspace: A registered workspace.
            parent: The exact expected head; ``None`` for the first publication.
            publication_id: Reuse an identity when re-preparing after a conflict.

        Returns:
            A single-use attempt whose ticket exists before any effect.
        """
        return PublicationAttempt(
            self._handle.prepare_publication(
                codec.encode_json(workspace),
                parent=None if parent is None else parent.to_hex(),
                publication_id=None
                if publication_id is None
                else publication_id.to_hex(),
            )
        )


@attrs.frozen
class RunHandle:
    """Blocking and asyncio access to the same supervised native job."""

    _handle: _NativeRunHandle

    def cancel(self) -> None:
        """Request stop; join and terminal report ownership remain native."""
        self._handle.cancel()

    @property
    def attempt_id(self) -> AttemptId | None:
        """Durable attempt minted before any effect; ``None`` for an ephemeral run."""
        attempt = self._handle.attempt_id
        return None if attempt is None else AttemptId(SemanticId.from_hex(attempt))

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


@attrs.frozen
class ProgressStream:
    """A durable attempt's stored progress events and incumbents, in observation order.

    Events are read from the operational store a bounded page at a time. A followed
    stream waits for new events until the attempt stops working; otherwise it ends
    after the events stored when it reads them. Each event carries its ``step``,
    stream ``sequence`` and observation time ``at``; an incumbent of a
    branch-and-bound search is an event whose ``incumbent`` is set. ``close`` ends
    the stream, including a read that is waiting.
    """

    _handle: _NativeProgressStream

    @property
    def attempt_id(self) -> AttemptId:
        """The attempt whose streams these are."""
        return AttemptId(SemanticId.from_hex(self._handle.attempt_id))

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
    """A durable study run by workers: its status, cancellation and one publication."""

    _handle: _NativeStudyHandle

    @property
    def study_id(self) -> StudyId:
        """The study identity."""
        return StudyId(SemanticId.from_hex(self._handle.study_id))

    def status(self) -> StudyStatus:
        """Read the study and every point as the operational store holds them now."""
        return codec.decode_json(self._handle.status(), StudyStatus)

    def cancel(self) -> StudyCancel:
        """Cancel the points that have not started and stop the running tries.

        What completed is still published.
        """
        return codec.decode_json(self._handle.cancel(), StudyCancel)

    def result(self) -> Published | None:
        """The study's publication once committed; ``None`` before."""
        published = self._handle.result()
        return None if published is None else codec.decode_json(published, Published)

    def wait(self, *, controls: StudyWaitControls | None = None) -> Published:
        """Wait for the study's publication under Rust-owned observation timing."""
        return codec.decode_json(
            self._handle.wait(
                controls=None if controls is None else codec.encode_json(controls)
            ),
            Published,
        )
