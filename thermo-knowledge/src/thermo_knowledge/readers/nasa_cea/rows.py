# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Row emission for the NASA CEA reader: a buffered sink over the writer and the provenance
columns of a row located by line."""

from __future__ import annotations

from thermo_knowledge.readers.nasa_cea.fortran import Line
from thermo_knowledge.staging.writer import Writer

Value = object
BATCH = 20_000


class Rows:
    """Collects rows per table and hands them to the writer in batches."""

    def __init__(self, writer: Writer) -> None:
        self._writer = writer
        self._buffers: dict[str, list[dict[str, Value]]] = {}

    def add(self, table: str, line: Line, /, *, suffix: str = "", **columns: Value) -> None:
        """A row located at `line` (`<artifact>#L<line>`, plus `suffix` when one line yields
        several rows)."""
        buffer = self._buffers.setdefault(table, [])
        buffer.append(
            {
                "_artifact": line.artifact,
                "_locator": f"{line.artifact}#L{line.number}{suffix}",
                **columns,
            }
        )
        if len(buffer) >= BATCH:
            self._flush(table)

    def frame(
        self,
        line: Line,
        kind: str,
        *,
        temperatures: list[float | None] | None = None,
        date: str | None = None,
    ) -> None:
        self.add(
            "frame_lines",
            line,
            kind=kind,
            text=line.text,
            temperatures=temperatures,
            date=date,
        )

    def _flush(self, table: str) -> None:
        buffer = self._buffers.get(table)
        if buffer:
            self._writer.rows(table, buffer)
            self._buffers[table] = []

    def flush(self) -> None:
        for table in list(self._buffers):
            self._flush(table)
