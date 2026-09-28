# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Source-independent joined computation and publication handles."""
from collections.abc import Mapping
from typing import TypeAlias
import attrs
import msgspec
from pse import codec
from pse._build import (DiagnosticReport, NativeEligibility, NativeRoute,
    ProgressEvent, _NativePreparedOperation, _NativePublicationAttempt,
    _NativeRunHandle, _NativeRunResult, _NativeStart)
from pse._inspection import TableStream
from pse.contracts import runtime as result_contracts
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
        """Typed eligibility row of every assessed backend, with registry reason codes."""
        return tuple(self._handle.eligibility)

    def with_start(self, seed: _NativeStart) -> "PreparedOperation":
        """Select a compatible owned numerical seed for an algebraic solve."""
        return PreparedOperation(self._handle.with_start(seed))

    def with_primal_start(self, values: Mapping[SemanticId, float]) -> "PreparedOperation":
        """Select every original free coordinate without retaining native state."""
        return PreparedOperation(self._handle.with_primal_start({key.to_hex(): value for key, value in values.items()}))

    def start(self) -> "RunHandle":
        """Start one admitted native operation."""
        return RunHandle(self._handle.start())


@attrs.frozen
class PublicationTicket:
    """Serialized pre-effect recovery request; settlement validates native witnesses."""

    json: bytes


class PublicationRoot(msgspec.Struct, frozen=True):
    """Exact immutable control selection."""

    location: str
    version: int


class PublicationCommitted(
    msgspec.Struct, tag="committed", tag_field="status", frozen=True
):
    """Matching transaction and complete request witnesses establish this exact root."""

    root: PublicationRoot


class PublicationNoncommit(
    msgspec.Struct, tag="proved_noncommit", tag_field="status", frozen=True
):
    """Positive durable evidence excludes the conditional publication."""

    reason: str


class PublicationConflict(
    msgspec.Struct, tag="conflict", tag_field="status", frozen=True
):
    """An observed identity or exact-parent conflict."""

    reason: str


class PublicationUnresolved(
    msgspec.Struct, tag="unresolved", tag_field="status", frozen=True
):
    """Evidence is incomplete; preserve the ticket and unresolved members."""

    reason: str


PublicationSettlement: TypeAlias = (
    PublicationCommitted
    | PublicationNoncommit
    | PublicationConflict
    | PublicationUnresolved
)


@attrs.frozen
class PublicationRequest:
    """Caller-stable identities and exact parent for one intended publication."""

    base: str
    workspace_id: SemanticId
    publication_id: SemanticId
    attempt_id: SemanticId
    parent: SemanticId | None = None


@attrs.frozen
class PublicationAttempt:
    """Explicit single-use write command over immutable results."""

    _handle: _NativePublicationAttempt

    @property
    def ticket(self) -> PublicationTicket:
        """Save before commit; it remains available after the command is consumed."""
        return PublicationTicket(self._handle.ticket())

    @property
    def attempt_id(self) -> SemanticId:
        """Native settlement identity retained after a failed commit."""
        return SemanticId.from_hex(self._handle.attempt_id)

    @property
    def publication_id(self) -> SemanticId:
        """Identity of the proposed control publication."""
        return SemanticId.from_hex(self._handle.publication_id)

    def commit(self) -> tuple[str, int]:
        """Return exact control URI/version for pse.open; never retry implicitly."""
        return self._handle.commit()


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
    def run_id(self) -> SemanticId:
        """Unique execution identity."""
        return SemanticId.from_hex(self._handle.run_id)

    @property
    def attempt_id(self) -> SemanticId | None:
        """Durable attempt that recorded this run; ``None`` for an ephemeral run."""
        attempt = self._handle.attempt_id
        return None if attempt is None else SemanticId.from_hex(attempt)

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
        self, base: str, workspace_id: SemanticId, *, parent: SemanticId | None = None
    ) -> PublicationAttempt:
        """Prepare control-last publication; this call does not write."""
        return PublicationAttempt(
            self._handle.prepare_publication(
                base,
                workspace_id.to_hex(),
                parent=None if parent is None else parent.to_hex(),
            )
        )

    def prepare_publication_request(
        self, request: PublicationRequest
    ) -> PublicationAttempt:
        """Prepare a retained request identity without performing any writes."""
        return PublicationAttempt(
            self._handle.prepare_publication(
                request.base,
                request.workspace_id.to_hex(),
                parent=None if request.parent is None else request.parent.to_hex(),
                publication_id=request.publication_id.to_hex(),
                attempt_id=request.attempt_id.to_hex(),
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
    def attempt_id(self) -> SemanticId | None:
        """Durable attempt minted before any effect; ``None`` for an ephemeral run."""
        attempt = self._handle.attempt_id
        return None if attempt is None else SemanticId.from_hex(attempt)

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


