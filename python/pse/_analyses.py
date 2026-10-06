# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Exact retained scientific graph pages with method and input provenance."""

import attrs

from pse._build import _NativeAnalysis
from pse._inspection import TableStream


@attrs.frozen
class Analysis:
    """A restartable persisted analysis; numeric evidence stays in source rows."""

    _handle: _NativeAnalysis

    @property
    def key(self) -> str:
        """Immutable method, configuration and selected-input identity."""
        return self._handle.key

    def header(self) -> TableStream:
        """Read the registry method and source lineage header."""
        return TableStream(self._handle.header())

    def nodes(self, *, after: str | None = None) -> TableStream:
        """Read at most 64 nodes after the exact native key cursor."""
        return TableStream(self._handle.nodes(after=after))

    def edges(self, *, after: str | None = None) -> TableStream:
        """Read at most 64 directed edges with original evidence references."""
        return TableStream(self._handle.edges(after=after))
