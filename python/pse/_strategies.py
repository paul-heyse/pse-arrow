# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Owned explicit graph and numerical strategies over native preparation."""

import attrs

from pse import codec
from pse._build import (
    DiagnosticReport,
    NativeStrategyAttempt,
    _NativePreparedFlow,
    _NativePreparedStrategy,
    _NativeStrategyResult,
)
from pse.contracts.documents import (
    FlowGraphDocument,
    InitializationDocument,
    RouteDocument,
    SolveSettings,
    TearSelectionDocument,
)
from pse.contracts.enums import TearMethod


@attrs.frozen
class PreparedFlow:
    """Selected physical graph with explicit cost, grouping and tear policy."""

    _handle: _NativePreparedFlow

    def graph(self) -> FlowGraphDocument:
        """Return source-attributed nodes, scalar ports, connections and decisions."""
        return codec.decode_json(self._handle.graph(), FlowGraphDocument)

    def select_tears(
        self, method: TearMethod, settings: SolveSettings
    ) -> "StrategyResult":
        """Run the explicitly selected native MILP or policy-respecting heuristic."""
        return StrategyResult(
            self._handle.select_tears(method, codec.encode_json(settings))
        )


@attrs.frozen
class PreparedStrategy:
    """Prepared explicit cone, causal map or transactional initialization."""

    _handle: _NativePreparedStrategy

    @property
    def routes(self) -> tuple[RouteDocument, ...]:
        """Typed routes selected before execution, without implicit failure fallback."""
        return tuple(self._handle.routes)

    def run(self) -> "StrategyResult":
        """Run one finite strategy and retain its joined reports."""
        return StrategyResult(self._handle.run())


@attrs.frozen
class StrategyResult:
    """Owned native attempts, including failures and stage overlays."""

    _handle: _NativeStrategyResult

    def tears(self) -> TearSelectionDocument | None:
        """Selected decisions, authored cost and independently checked acyclic order."""
        data = self._handle.tears()
        return None if data is None else codec.decode_json(data, TearSelectionDocument)

    def attempts(self) -> tuple[NativeStrategyAttempt, ...]:
        """Every attempt in order: its native report or the typed failure before one."""
        return tuple(self._handle.attempts())

    def failures(self) -> tuple[tuple[int, DiagnosticReport], ...]:
        """Typed failures before a native report, indexed like ``attempts()``."""
        failures: list[tuple[int, DiagnosticReport]] = []
        for index, attempt in enumerate(self.attempts()):
            failure = attempt.failure
            if failure is not None:
                failures.append((index, failure))
        return tuple(failures)

    def initialization(self) -> InitializationDocument | None:
        """Original values, committed unknowns and temporary stage evidence."""
        data = self._handle.initialization()
        return None if data is None else codec.decode_json(data, InitializationDocument)
