# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The writer a reader emits its tables through.

It enforces the contract of a staged table: the table is declared, every batch matches the
declared schema, every row names a payload file as its `_artifact`, every `_locator` has the form
`<artifact>#<position>` and is unique within the table. One Parquet file is written per table.
`finish` accounts for every payload file: a file is `read` when a row cites it (or the reader
says it opened it), `partly_read` or `skipped` when the reader says so with a reason; a payload
file nothing accounts for is an error.
"""

from __future__ import annotations

import hashlib
from collections.abc import Iterable, Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path

import pyarrow as pa
import pyarrow.parquet as pq

from thermo_knowledge.staging import schema as schema_module
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.manifest import FileRecord, TableRecord
from thermo_knowledge.staging.schema import ARTIFACT, LOCATOR

BATCH_ROWS = 50_000
COMPRESSION = "zstd"
_CHUNK = 1024 * 1024


def parquet_name(table: str) -> str:
    return f"{table}.parquet"


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while chunk := handle.read(_CHUNK):
            digest.update(chunk)
    return digest.hexdigest()


@dataclass(frozen=True)
class WriterResult:
    tables: dict[str, TableRecord]
    payload: list[FileRecord]


class Writer:
    """Collects the tables of one reader run into `directory`."""

    def __init__(
        self,
        directory: Path,
        schemas: Mapping[str, pa.Schema],
        payload_files: Sequence[str],
    ) -> None:
        schema_module.check_declared(schemas)
        self._directory = directory
        self._schemas = dict(schemas)
        self._payload = tuple(payload_files)
        self._payload_set = frozenset(payload_files)
        self._writers: dict[str, pq.ParquetWriter] = {}
        self._rows: dict[str, int] = {}
        self._locators: dict[str, set[str]] = {name: set() for name in schemas}
        self._cited: set[str] = set()
        self._opened: set[str] = set()
        self._skipped: dict[str, str] = {}
        self._partly: dict[str, str] = {}

    @property
    def payload_files(self) -> tuple[str, ...]:
        """Every file the source's payload globs match, sorted."""
        return self._payload

    # -- emitting ----------------------------------------------------------------------

    def rows(
        self, table: str, rows: Iterable[Mapping[str, object]], *, batch_rows: int = BATCH_ROWS
    ) -> int:
        """Write rows (column name to Python value) to `table`; returns how many.

        A row with a key the schema does not declare, a value of the wrong kind or a null in a
        non-nullable column is refused.
        """
        schema = self._declared(table)
        names = set(schema.names)
        written = 0
        pending: list[Mapping[str, object]] = []
        for row in rows:
            unknown = sorted(set(row) - names)
            if unknown:
                raise StagingError(
                    f"table {table}: row has columns the schema does not declare: "
                    f"{', '.join(unknown)}"
                )
            for field in schema:
                _check_value(table, field, row.get(field.name))
            pending.append(row)
            if len(pending) >= batch_rows:
                written += self._flush(table, pending)
                pending = []
        if pending:
            written += self._flush(table, pending)
        return written

    def batch(self, table: str, batch: pa.RecordBatch | pa.Table) -> int:
        """Write an Arrow batch whose schema equals the declared schema."""
        schema = self._declared(table)
        if not batch.schema.equals(schema, check_metadata=False):
            raise StagingError(
                f"table {table}: the batch does not match the declared schema\n"
                f"  declared: {schema.to_string(show_field_metadata=False)}\n"
                f"  batch:    {batch.schema.to_string(show_field_metadata=False)}"
            )
        records = batch.to_batches() if isinstance(batch, pa.Table) else [batch]
        total = 0
        for record in records:
            self._write(table, pa.RecordBatch.from_arrays(record.columns, schema=schema))
            total += record.num_rows
        return total

    def _flush(self, table: str, rows: list[Mapping[str, object]]) -> int:
        schema = self._schemas[table]
        batch = pa.RecordBatch.from_pylist([dict(row) for row in rows], schema=schema)
        self._write(table, batch)
        return batch.num_rows

    def _write(self, table: str, batch: pa.RecordBatch) -> None:
        schema = self._schemas[table]
        for field in schema:
            if not field.nullable and batch.column(field.name).null_count:
                raise StagingError(f"table {table}: null in the non-nullable column {field.name}")
        artifacts = batch.column(ARTIFACT).to_pylist()
        locators = batch.column(LOCATOR).to_pylist()
        seen = self._locators[table]
        for artifact, locator in zip(artifacts, locators, strict=True):
            if artifact is None or artifact not in self._payload_set:
                raise StagingError(
                    f"table {table}: row cites {artifact!r}, which is not a payload file "
                    "of the source"
                )
            if locator is None or not locator.startswith(f"{artifact}#"):
                raise StagingError(
                    f"table {table}: _locator {locator!r} must start with "
                    f"'<_artifact>#' ({artifact}#)"
                )
            if locator in seen:
                raise StagingError(f"table {table}: duplicate _locator {locator!r}")
            seen.add(locator)
        self._cited.update(artifact for artifact in artifacts if artifact is not None)
        writer = self._writers.get(table)
        if writer is None:
            writer = pq.ParquetWriter(
                self._directory / parquet_name(table), schema, compression=COMPRESSION
            )
            self._writers[table] = writer
        writer.write_batch(batch)
        self._rows[table] = self._rows.get(table, 0) + batch.num_rows

    def _declared(self, table: str) -> pa.Schema:
        try:
            return self._schemas[table]
        except KeyError:
            declared = ", ".join(self._schemas)
            raise StagingError(
                f"table {table!r} is emitted without a declared schema; declared tables: {declared}"
            ) from None

    # -- payload accounting ---------------------------------------------------------------

    def opened(self, artifact: str) -> None:
        """The reader read `artifact` and it yielded no rows."""
        self._known(artifact)
        self._opened.add(artifact)

    def skipped(self, artifact: str, reason: str) -> None:
        """The reader did not read `artifact`; `reason` says why."""
        self._known(artifact)
        _reason(artifact, reason)
        self._skipped[artifact] = reason

    def partly_read(self, artifact: str, reason: str) -> None:
        """The reader kept only part of `artifact`'s content as columns; `reason` says what."""
        self._known(artifact)
        _reason(artifact, reason)
        self._partly[artifact] = reason

    def _known(self, artifact: str) -> None:
        if artifact not in self._payload_set:
            raise StagingError(f"{artifact!r} is not a payload file of the source")

    # -- finishing -------------------------------------------------------------------------

    def finish(self) -> WriterResult:
        """Close the tables (declared tables nothing was written to are written empty) and
        account for the payload; raises for an unaccounted payload file."""
        for table, schema in self._schemas.items():
            if table not in self._writers:
                self._writers[table] = pq.ParquetWriter(
                    self._directory / parquet_name(table), schema, compression=COMPRESSION
                )
                self._rows[table] = 0
        for writer in self._writers.values():
            writer.close()
        records = [self._file_record(path) for path in self._payload]
        unaccounted = [record.path for record in records if record.status == "unaccounted"]
        if unaccounted:
            shown = ", ".join(unaccounted[:10])
            more = f" (and {len(unaccounted) - 10} more)" if len(unaccounted) > 10 else ""
            raise StagingError(
                f"{len(unaccounted)} payload file(s) are unaccounted for: no row cites them and "
                f"the reader did not list them as skipped with a reason: {shown}{more}"
            )
        tables = {
            table: TableRecord(
                file=parquet_name(table),
                rows=self._rows[table],
                schema_fingerprint=schema_module.fingerprint(schema),
                content_hash=sha256_file(self._directory / parquet_name(table)),
            )
            for table, schema in self._schemas.items()
        }
        payload = [
            FileRecord(record.path, record.status, record.reason)  # type: ignore[arg-type]
            for record in records
        ]
        return WriterResult(tables, payload)

    def abort(self) -> None:
        """Close any open Parquet file after a failed run."""
        for writer in self._writers.values():
            try:
                writer.close()
            except Exception:  # noqa: BLE001 - best-effort cleanup of a failed run
                pass

    def _file_record(self, path: str) -> _Account:
        cited = path in self._cited
        if path in self._skipped:
            if cited:
                raise StagingError(f"{path}: listed as skipped but rows cite it")
            return _Account(path, "skipped", self._skipped[path])
        if path in self._partly:
            return _Account(path, "partly_read", self._partly[path])
        if cited or path in self._opened:
            return _Account(path, "read", None)
        return _Account(path, "unaccounted", None)


@dataclass(frozen=True)
class _Account:
    path: str
    status: str
    reason: str | None


def _reason(artifact: str, reason: str) -> None:
    if not reason.strip():
        raise StagingError(f"{artifact}: a skipped or partly read file needs a reason")


def _check_value(table: str, field: pa.Field, value: object) -> None:
    if value is None:
        if not field.nullable:
            raise StagingError(f"table {table}: null in the non-nullable column {field.name}")
        return
    if not _fits(field.type, value):
        raise StagingError(
            f"table {table}, column {field.name}: value {value!r} ({type(value).__name__}) "
            f"does not match the declared type {field.type}"
        )


def _fits(dtype: pa.DataType, value: object) -> bool:
    if pa.types.is_list(dtype):
        if not isinstance(value, list | tuple):
            return False
        return all(item is None or _fits(dtype.value_type, item) for item in value)
    if pa.types.is_boolean(dtype):
        return isinstance(value, bool)
    if isinstance(value, bool):
        return False
    if pa.types.is_string(dtype):
        return isinstance(value, str)
    if pa.types.is_integer(dtype):
        return isinstance(value, int)
    if pa.types.is_floating(dtype):
        return isinstance(value, int | float)
    return False
