# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Source-independent joined computation and publication handles."""

from collections.abc import Mapping
from datetime import timedelta
from types import TracebackType
from typing import Self, TypeAlias

import attrs
import msgspec

from pse import codec
from pse._build import (
    DiagnosticReport,
    NativeEligibility,
    NativeRoute,
    ProgressEvent,
    _NativePreparedOperation,
    _NativeProgressStream,
    _NativePublicationAttempt,
    _NativeRunHandle,
    _NativeRunResult,
    _NativeStart,
    _NativeStudyHandle,
)
from pse._inspection import TableStream
from pse.contracts import runtime as result_contracts
from pse.contracts.enums import AttemptState, JobState, StudyPointState, StudyState
from pse.contracts.identities import (
    AttemptId,
    PublicationId,
    RunId,
    StudyId,
    WorkspaceId,
)
from pse.contracts.values import ContentHash, SemanticId


@attrs.frozen
class PreparedOperation:
    """Immutable authored solve, simulation or fitting request."""

    _handle: _NativePreparedOperation

    @property
    def identity(self) -> ContentHash:
        """Exact prepared source and profile identity."""
        return ContentHash.from_prefixed(self._handle.identity)

    @property
    def route(self) -> NativeRoute:
        """Admitted algebraic route, before native execution."""
        return self._handle.route

    @property
    def eligibility(self) -> tuple[NativeEligibility, ...]:
        """Typed eligibility of every assessed backend, with registry reason codes."""
        return tuple(self._handle.eligibility)

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


class Workspace(msgspec.Struct, frozen=True, forbid_unknown_fields=True):
    """A registered publication workspace: one history with one head.

    Its members are written under ``root_uri``.
    """

    workspace_id: str
    name: str
    root_uri: str

    @property
    def id(self) -> WorkspaceId:
        """The workspace identity."""
        return WorkspaceId(SemanticId.from_hex(self.workspace_id))


class Published(msgspec.Struct, frozen=True, forbid_unknown_fields=True):
    """A committed publication: the head of its workspace when committed."""

    publication_id: str
    workspace_id: str
    parent: str | None
    attempt_id: str

    @property
    def id(self) -> PublicationId:
        """The publication identity."""
        return PublicationId(SemanticId.from_hex(self.publication_id))


class ExportReceipt(msgspec.Struct, frozen=True, forbid_unknown_fields=True):
    """An export: the manifest location and the lease protecting its members.

    The lease holds until ``expires_at`` (microseconds since the Unix epoch) or its
    release.
    """

    publication_id: str
    destination: str
    lease_id: str
    expires_at: int


class PublicationCommitted(
    msgspec.Struct, tag="committed", tag_field="status", frozen=True
):
    """The ticket's publication is visible."""

    publication_id: str


class PublicationNoncommit(
    msgspec.Struct, tag="proved_noncommit", tag_field="status", frozen=True
):
    """Nothing was committed and nothing is in flight; the same ticket may commit."""


class PublicationConflict(
    msgspec.Struct, tag="conflict", tag_field="status", frozen=True
):
    """The ticket can never commit as prepared: re-prepare against ``head``."""

    reason: str
    head: str | None


class PublicationUnresolved(
    msgspec.Struct, tag="unresolved", tag_field="status", frozen=True
):
    """The catalog could not be reached; settle again later."""

    reason: str


PublicationSettlement: TypeAlias = (
    PublicationCommitted
    | PublicationNoncommit
    | PublicationConflict
    | PublicationUnresolved
)


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
        return msgspec.json.decode(self._handle.commit(), type=Published)


@attrs.frozen
class RunCompletion:
    """Registry-owned completion records shared with the durable Arrow projection."""

    solves: tuple[result_contracts.RuntimeSolveRunsRow, ...]
    computation: result_contracts.RuntimeComputationRunsRow | None
    lineage: tuple[result_contracts.RuntimeRunLineageRow, ...]
    assessments: tuple[result_contracts.RuntimeCandidateAssessmentsRow, ...]


@attrs.frozen
class RunResult:
    """Joined immutable outcome; failed and unattempted steps remain inspectable."""

    _handle: _NativeRunResult

    def available_start(self, step: int = 0) -> _NativeStart | None:
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
    def completion(self) -> RunCompletion:
        """Read the immutable joined assessment without evaluating the model again."""
        wire = msgspec.json.decode(self._handle.completion(), type=dict[str, object])
        wire.pop("diagnostics")
        return codec.converter().structure(wire, RunCompletion)

    def diagnostics(self) -> tuple[DiagnosticReport, ...]:
        """Structured native admission and execution failures, preserving causes."""
        return tuple(self._handle.diagnostics())

    def tables(self) -> tuple[str, ...]:
        """Declared result and reconstruction-source relation names."""
        return tuple(self._handle.tables())

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
                msgspec.json.encode(workspace),
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

    def progress(self) -> tuple[tuple[ProgressEvent, ...], int]:
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

    def __next__(self) -> ProgressEvent:
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


class StudyPointStatus(msgspec.Struct, frozen=True, forbid_unknown_fields=True):
    """One point of a durable study.

    It records its state, the point that seeds it, its latest try and why it failed
    or was cancelled.
    """

    point_index: int
    state: StudyPointState
    predecessor: int | None
    attempt_id: str
    attempt_state: AttemptState
    job_state: JobState
    error: str | None


class StudyStatus(msgspec.Struct, frozen=True, forbid_unknown_fields=True):
    """A durable study and its lifecycle.

    It is open while its points run, concluded once every point is terminal, and
    published once its one publication is committed.
    """

    study_id: str
    state: StudyState
    attempt_id: str
    attempt_state: AttemptState
    publication_id: str
    finalization: JobState
    finalization_error: str | None
    points: tuple[StudyPointStatus, ...]

    @property
    def id(self) -> StudyId:
        """The study identity."""
        return StudyId(SemanticId.from_hex(self.study_id))


class StudyCancel(msgspec.Struct, frozen=True, forbid_unknown_fields=True):
    """What cancelling a study did.

    It lists the points cancelled before they started and the running tries asked to
    stop, and says whether the study concluded now or had already.
    """

    cancelled: tuple[int, ...]
    stopping: tuple[int, ...]
    concluded: bool
    already_concluded: bool


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
        return msgspec.json.decode(self._handle.status(), type=StudyStatus)

    def cancel(self) -> StudyCancel:
        """Cancel the points that have not started and stop the running tries.

        What completed is still published.
        """
        return msgspec.json.decode(self._handle.cancel(), type=StudyCancel)

    def result(self) -> Published | None:
        """The study's publication once committed; ``None`` before."""
        published = self._handle.result()
        return (
            None
            if published is None
            else msgspec.json.decode(published, type=Published)
        )

    def wait(
        self,
        *,
        poll: timedelta = timedelta(milliseconds=500),
        timeout: timedelta | None = None,
    ) -> Published:
        """Wait until the study is published.

        Args:
            poll: How often the study's state is read.
            timeout: Give up after this long; wait indefinitely when ``None``.

        Returns:
            The study's one publication.
        """
        return msgspec.json.decode(
            self._handle.wait(
                poll_seconds=poll.total_seconds(),
                timeout_seconds=None if timeout is None else timeout.total_seconds(),
            ),
            type=Published,
        )
