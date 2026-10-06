# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Canonical table inspection through the Arrow capsule protocol."""

from types import TracebackType
from typing import Self

import attrs
import pyarrow as pa

from pse._build import (
    DiagnosticReport,
    _NativeTableStream,
    EngineSettings,
    _registry_table,
)
from pse._transfer import FieldTransfer, compare_schemas


def registry_table(name: str, *, settings: EngineSettings) -> "TableStream":
    """Read one exact compiled reference relation through Arrow C Stream."""
    return TableStream(_registry_table(name, settings=settings))


@attrs.frozen
class TableStream:
    """One-consumption Arrow stream; close releases unread work, not live arrays."""

    _handle: _NativeTableStream

    def __arrow_c_schema__(self) -> object:
        """Export the exact admitted Arrow schema capsule."""
        return self._handle.__arrow_c_schema__()

    def __arrow_c_stream__(self, requested_schema: object | None = None) -> object:
        """Export once; requested schema casts are explicitly refused."""
        return self._handle.__arrow_c_stream__(requested_schema)

    @property
    def failure(self) -> DiagnosticReport | None:
        """Original terminal native failure, including causes and execution contexts."""
        return self._handle.failure

    def close(self) -> None:
        """Release unread work; readers reach EOF and live arrays stay valid."""
        self._handle.close()

    def cancel(self) -> None:
        """Make further consumption fail with the cancellation diagnostic."""
        self._handle.cancel()

    def extension_report(self, consumer_schema: pa.Schema) -> tuple[FieldTransfer, ...]:
        """Compare this source with the schema an actual consumer supplied.

        Args:
            consumer_schema: The consumer's observed schema, rather than an
                assumed registration.

        Returns:
            One observation per nested field, including lost metadata and mismatches.
        """
        return compare_schemas(pa.schema(self), consumer_schema)

    def __enter__(self) -> Self:
        return self

    def __exit__(
        self,
        _exc_type: type[BaseException] | None,
        _exc_value: BaseException | None,
        _traceback: TracebackType | None,
    ) -> None:
        self.close()

