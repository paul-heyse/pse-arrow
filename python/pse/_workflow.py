# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Typed declarations and owned handles over the single native computation pipeline."""

from collections.abc import Mapping, Sequence

import attrs
import msgspec
import pyarrow as pa

from pse import codec
from pse._build import (
    EngineSettings,
    OperationalStore,
    _NativePhysicalContext,
    _NativeRuntime,
)
from pse._inspection import TableStream
from pse._modeling import ModelingPackage
from pse._strategies import PreparedFlow, PreparedStrategy, StrategyResult, _AnalysisDocument
from pse._runs import (
    PreparedOperation, RunHandle, RunResult, RunCompletion, PublicationTicket,
    PublicationRoot, PublicationCommitted, PublicationNoncommit, PublicationConflict,
    PublicationUnresolved, PublicationSettlement, PublicationRequest, PublicationAttempt,
)

from pse.contracts import runtime as result_contracts
from pse.contracts.documents import SolveSettings
from pse.contracts.enums import AttemptState
from pse.contracts.values import ContentHash, SemanticId

SolverCapability = result_contracts.RuntimeSolverCapabilitiesRow
OperationalAttempt = result_contracts.RuntimeOperationalAttemptsRow

@attrs.frozen(init=False)
class Runtime:
    """One native deployment budget shared with publication inspection."""

    _handle: _NativeRuntime

    def __init__(
        self, settings: EngineSettings, *, store: OperationalStore | None = None
    ) -> None:
        """Attach to the shared deployment under an explicit durability class.

        Args:
            settings: The deployment budget shared with publication inspection.
            store: With a store, every run is a durable attempt registered in it
                and may be published; without one, runs are ephemeral and cannot
                publish (ADR-0112 Outcome 16).
        """
        object.__setattr__(self, "_handle", _NativeRuntime(settings, store=store))

    @property
    def durable(self) -> bool:
        """Whether runs are durable attempts in an operational store."""
        return self._handle.durable

    def runs(
        self,
        *,
        run_id: SemanticId | None = None,
        states: Sequence[AttemptState] = (),
        limit: int = 100,
    ) -> tuple[OperationalAttempt, ...]:
        """List the store's durable attempts, newest first; they survive restarts.

        Args:
            run_id: Only the attempts of this run.
            states: Only attempts in these lifecycle states; every state when empty.
            limit: At most this many attempts.

        Returns:
            One registry ``runtime.operational_attempts`` row per attempt.
        """
        stream = TableStream(
            self._handle.runs(
                run_id=None if run_id is None else run_id.to_hex(),
                states=[AttemptState(state).value for state in states],
                limit=limit,
            )
        )
        return tuple(
            codec.structure_rows(pa.table(stream).to_pylist(), OperationalAttempt)
        )

    def physical_from_documents(
        self, documents: Mapping[str, str]
    ) -> "PhysicalContext":
        """Admit actual physical package rows using the native source loader."""
        return PhysicalContext(self._handle.physical_from_documents(dict(documents)))

    def modeling_from_documents(
        self,
        documents: Sequence[Mapping[str, str]],
        physical: "PhysicalContext",
    ) -> ModelingPackage:
        """Admit an exact package closure with manifest-owned physical type aliases."""
        return ModelingPackage(
            self._handle.modeling_from_documents(
                [dict(bundle) for bundle in documents], physical._handle
            )
        )

    def capabilities(self) -> tuple[SolverCapability, ...]:
        """Discover linked native libraries without PATH or optional Python probes."""
        return tuple(
            codec.converter().structure(
                msgspec.json.decode(self._handle.capabilities()), list[SolverCapability]
            )
        )

    def clear_program_cache(self) -> None:
        """Release retained programs while keeping active prepared workers valid."""
        self._handle.clear_program_cache()

    def settle_publication(self, ticket: PublicationTicket) -> PublicationSettlement:
        """Observe saved native witnesses without repeating a solve or publication."""
        return msgspec.json.decode(
            self._handle.settle_publication(ticket.json), type=PublicationSettlement
        )

    def prepare_conic(
        self,
        request: Mapping[str, object],
        physical: "PhysicalContext",
        settings: SolveSettings,
    ) -> PreparedStrategy:
        """Admit explicit library cone geometry and an exact quadratic witness."""
        return PreparedStrategy(
            self._handle.prepare_conic(
                codec.encode_json(_AnalysisDocument(dict(request))),
                physical._handle,  # noqa: SLF001 - same native boundary
                codec.encode_json(settings),
            )
        )

    def start(
        self, cases: Sequence[PreparedOperation], *, continue_independent: bool = False
    ) -> RunHandle:
        """Run a finite sequence on the existing completion-owned native pipeline."""
        return RunHandle(
            self._handle.start(
                [c._handle for c in cases],  # noqa: SLF001 - same native boundary
                continue_independent=continue_independent,
            )
        )


@attrs.frozen
class PhysicalContext:
    """Admitted quantities, units, prerequisites and original source rows."""

    _handle: _NativePhysicalContext

    @property
    def identity(self) -> ContentHash:
        """Complete native compiler context identity."""
        return ContentHash.from_prefixed(self._handle.identity)
