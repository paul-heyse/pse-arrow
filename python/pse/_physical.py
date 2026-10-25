# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Arrow transport protocols for registry-governed physical inputs."""

from typing import Protocol, TypeAlias

import attrs

from pse._build import _NativePhysicalContext
from pse.contracts.values import ContentHash
from pse.contracts.documents import PhysicalInspectionRequest, PhysicalInspectionDocument
from pse.contracts.identities import QuantityTypeId, ReferenceStateId, UnitId


class ArrowArrayInput(Protocol):
    """An Arrow C Data producer, including a record batch."""

    def __arrow_c_array__(
        self, requested_schema: object = None
    ) -> tuple[object, object]:
        """Export a schema and array capsule."""
        ...


class ArrowStreamInput(Protocol):
    """An Arrow C Stream producer, including a table or batch reader."""

    def __arrow_c_stream__(self, requested_schema: object = None) -> object:
        """Export an Arrow stream capsule."""
        ...


ArrowInput: TypeAlias = ArrowArrayInput | ArrowStreamInput


class ArrowSchema(Protocol):
    """A complete schema imported through Arrow's C Schema interface."""

    def __arrow_c_schema__(self) -> object:
        """Export the registered schema capsule, including metadata."""
        ...


@attrs.frozen
class PhysicalContext:
    """Admitted quantities, units, prerequisites and original source rows."""

    _handle: _NativePhysicalContext

    @property
    def identity(self) -> ContentHash:
        """Complete native compiler context identity."""
        return self._handle.identity

    def describe(self, request: PhysicalInspectionRequest) -> PhysicalInspectionDocument:
        """Read a bounded page of this exact admitted physical context."""
        return self._handle.describe(request)

    def resolve_quantity(self, name: str) -> QuantityTypeId:
        """Resolve one declared physical quantity name within this context."""
        return self._handle.resolve_quantity(name)

    def resolve_reference_state(self, name: str) -> ReferenceStateId:
        """Resolve one declared reference-state name within this context."""
        return self._handle.resolve_reference_state(name)

    def resolve_unit(self, symbol: str) -> UnitId:
        """Resolve one exact declared representation symbol within this context."""
        return self._handle.resolve_unit(symbol)
