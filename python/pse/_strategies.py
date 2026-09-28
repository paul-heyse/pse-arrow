# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Owned explicit graph and numerical strategies over native preparation."""

import attrs
import msgspec

from pse import codec
from pse._build import (
    DiagnosticReport,
    NativeRoute,
    NativeStrategyAttempt,
    SolveSettings,
    _NativePreparedFlow,
    _NativePreparedStrategy,
    _NativeStrategyResult,
)


class _AnalysisDocument(msgspec.Struct, frozen=True, forbid_unknown_fields=True):
    """Transport envelope; Rust admits the typed analysis inside it."""

    payload: dict[str, object]


@attrs.frozen
class PreparedFlow:
    """Selected physical graph with explicit cost, grouping and tear policy."""

    _handle: _NativePreparedFlow

    def graph(self) -> dict[str, object]:
        """Return source-attributed nodes, scalar ports, connections and decisions."""
        return codec.decode_json(
            self._handle.graph_json().encode(), _AnalysisDocument
        ).payload

    def select_tears(self, method: str, settings: SolveSettings) -> "StrategyResult":
        """Run the explicitly selected native MILP or policy-respecting heuristic."""
        return StrategyResult(self._handle.select_tears(method, settings))


@attrs.frozen
class PreparedStrategy:
    """Prepared explicit cone, causal map or transactional initialization."""

    _handle: _NativePreparedStrategy

    @property
    def routes(self) -> tuple[NativeRoute, ...]:
        """Typed routes selected before execution, without implicit failure fallback."""
        return tuple(self._handle.routes)

    def run(self) -> "StrategyResult":
        """Run one finite strategy and retain its joined reports."""
        return StrategyResult(self._handle.run())


@attrs.frozen
class StrategyResult:
    """Owned native attempts, including failures and stage overlays."""

    _handle: _NativeStrategyResult

    def tears(self) -> dict[str, object] | None:
        """Selected decisions, authored cost and independently checked acyclic order."""
        data = self._handle.tears_json()
        return (
            None
            if data is None
            else codec.decode_json(data.encode(), _AnalysisDocument).payload
        )

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

    def initialization(self) -> dict[str, object] | None:
        """Original values, committed unknowns and temporary stage evidence."""
        data = self._handle.initialization_json()
        return (
            None
            if data is None
            else codec.decode_json(data.encode(), _AnalysisDocument).payload
        )


