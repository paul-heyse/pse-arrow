# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Immutable admitted snapshot inspection through the Arrow capsule protocol."""

from pathlib import Path
from types import TracebackType
from typing import Self

import attrs
import pyarrow as pa

from pse._build import (
    EngineSettings,
    _NativeSnapshot,
    _NativeStore,
    _NativeTableStream,
    _open_store,
)
from pse._transfer import FieldTransfer, compare_schemas


@attrs.frozen
class ResourceUsage:
    """Actual shared pool counters, separate from whole-process peak RSS.

    Attributes:
        limit_bytes: Configured finite accounted memory limit.
        reserved_bytes: Current actual reservations across all stores and arrays.
        peak_bytes: Peak accounted reservations since runtime construction.
        process_peak_rss_bytes: Independent process observation, when supported.
    """

    limit_bytes: int
    reserved_bytes: int
    peak_bytes: int
    process_peak_rss_bytes: int | None


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
class Snapshot:
    """An exact admitted manifest; mutable ref changes cannot update this handle."""

    _handle: _NativeSnapshot

    @property
    def snapshot_id(self) -> str:
        """The pinned snapshot's content identity."""
        return self._handle.snapshot_id

    @property
    def manifest_checksum(self) -> str:
        """The exact pinned manifest encoding identity."""
        return self._handle.manifest_checksum

    @property
    def revision_id(self) -> str | None:
        """The exact revision observed with the ref, if the ref carries one."""
        return self._handle.revision_id

    def tables(self) -> tuple[tuple[str, str], ...]:
        """Return the qualified relation and exact output port for each member."""
        return tuple(self._handle.tables())

    def table(
        self, relation: str, name: str | None = None, *, port: str | None = None
    ) -> TableStream:
        """Stream a named table in bounded batches, retaining its admitted owners.

        Args:
            relation: A qualified relation, or namespace when name is supplied.
            name: Optional relation name for the two-argument form.
            port: Exact output port, required when a stage repeats the relation.

        Returns:
            An immutable object implementing the Arrow C Stream protocol.
        """
        return TableStream(self._handle.table(relation, name, port=port))

    def close(self) -> None:
        """Release this handle; existing streams retain independent ownership."""
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
class Store:
    """A read-only capability over an existing local catalog."""

    _handle: _NativeStore

    def head(self, ref_name: str = "main") -> Snapshot:
        """Read and admit the exact ref target and its persisted context.

        Args:
            ref_name: Existing named ref; defaults to main.

        Returns:
            A snapshot pinned independently of later ref movement.
        """
        return Snapshot(self._handle.head(ref_name))

    def snapshot(self, snapshot_id: str, manifest_checksum: str) -> Snapshot:
        """Open an exact persisted snapshot and validate its actual context.

        Args:
            snapshot_id: Explicit prefixed snapshot identity.
            manifest_checksum: Explicit prefixed manifest encoding identity.

        Returns:
            The fully admitted immutable snapshot.
        """
        return Snapshot(self._handle.snapshot(snapshot_id, manifest_checksum))

    def resource_usage(self) -> ResourceUsage:
        """Observe the shared budget, including live arrays from closed handles."""
        return ResourceUsage(*self._handle.resource_usage())

    def cancel(self) -> None:
        """Cancel this store's active or subsequent opening operation."""
        self._handle.cancel()

    def close(self) -> None:
        """Prevent new operations; existing snapshots and streams remain pinned."""
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


def open(path: str | Path, *, settings: EngineSettings) -> Store:
    """Open an existing local store under one explicit shared process budget.

    Args:
        path: Existing catalog directory. Opening never creates the directory.
        settings: Explicit Rust-validated memory, thread, spill and batch bounds.

    Returns:
        An immutable inspection capability. Snapshot opening completes read-only
        admission of actual external data. Inspecting admitted tables performs no
        further source construction and exposes no compile or solve operation.
    """
    return Store(_open_store(str(path), settings))
