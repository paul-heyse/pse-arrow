# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Exact Delta publication inspection through the Arrow capsule protocol."""

from pathlib import Path
from types import TracebackType
from typing import Self

import attrs
import pyarrow as pa

from pse._build import (
    CacheReport,
    DiagnosticReport,
    EngineSettings,
    ResourceReport,
    TableName,
    _NativePublication,
    _NativeTableStream,
    _open_publication,
)
from pse._transfer import FieldTransfer, compare_schemas


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


@attrs.frozen
class Publication:
    """One exact Delta root; subsequent writes cannot update this selection."""

    _handle: _NativePublication

    @property
    def location(self) -> str:
        """The absolute URI of the selected Delta control table."""
        return self._handle.location

    @property
    def version(self) -> int:
        """The exact selected Delta control version."""
        return self._handle.version

    def tables(self) -> tuple[TableName, ...]:
        """Return exact catalog, schema and table components for every member."""
        return tuple(self._handle.tables())

    def table(self, catalog: str, schema: str, table: str) -> TableStream:
        """Stream an exact native member under the publication's contracts.

        Args:
            catalog: Literal native catalog name; no SQL case normalization.
            schema: Literal native schema name.
            table: Literal native table name.

        Returns:
            An owned Arrow stream that outlives this publication handle.
        """
        return TableStream(self._handle.table(catalog, schema, table))

    def resource_usage(self) -> ResourceReport:
        """Observe actual pool ownership, including arrays from closed handles."""
        return self._handle.resource_usage()

    def cache_usage(self) -> tuple[CacheReport, ...]:
        """Observe shared cache ownership; unavailable native counters are None."""
        return tuple(self._handle.cache_usage())

    def close(self) -> None:
        """Release this selection; existing streams retain independent ownership."""
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


def open(
    location: str | Path, *, version: int, settings: EngineSettings
) -> Publication:
    """Open an exact existing Delta publication under the shared process budget.

    Args:
        location: Absolute Delta control-table URI, or a local pathlib Path.
        version: Exact nonnegative Delta control version to select.
        settings: Explicit Rust-validated memory, thread, spill and batch bounds.

    Returns:
        A selected publication with lazy, contract-checked native member reads.
    """
    uri = location.resolve().as_uri() if isinstance(location, Path) else location
    return Publication(_open_publication(uri, version, settings))
