# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Text files kept verbatim: one row per line, so a file that is not decomposed into columns is
still read losslessly.

A file that is valid UTF-8 is decoded as UTF-8; any other is decoded as latin-1, which maps each
byte to one character, so the bytes are recoverable. Lines are split on `\\n` only; a trailing
`\\r` stays in the text.
"""

from __future__ import annotations

from pathlib import Path

from thermo_knowledge.readers.coolprop.common import (
    NOT_APPLICABLE,
    Col,
    Sink,
    integer,
    locator,
    merge_schemas,
    text,
)
from thermo_knowledge.staging.schema import BOOL, INT64, STRING, table_schema

SCHEMAS = merge_schemas(
    {
        "text_files": table_schema(
            Col(
                "encoding",
                STRING,
                "decoding applied: utf-8, or latin-1 for other bytes",
                NOT_APPLICABLE,
            ).field(),
            Col("line_count", INT64, "number of lines", NOT_APPLICABLE).field(),
            Col("byte_count", INT64, "size of the file", "bytes").field(),
            Col(
                "ends_with_newline",
                BOOL,
                "whether the last line ends with a newline",
                NOT_APPLICABLE,
            ).field(),
        ),
        "text_lines": table_schema(
            integer("line", source_name="line number", unit=NOT_APPLICABLE).field(),
            text("text", source_name="line content without its \\n").field(),
        ),
    }
)


def read_text_lines(tree: Path, artifact: str, sink: Sink) -> None:
    data = (tree / artifact).read_bytes()
    try:
        content, encoding = data.decode("utf-8"), "utf-8"
    except UnicodeDecodeError:
        content, encoding = data.decode("latin-1"), "latin-1"
    lines = content.split("\n")
    ends_with_newline = content.endswith("\n")
    if ends_with_newline or content == "":
        lines.pop()
    last = f"L1-L{len(lines)}" if lines else "L0"
    sink.add(
        "text_files",
        {
            "_artifact": artifact,
            "_locator": locator(artifact, last),
            "encoding": encoding,
            "line_count": len(lines),
            "byte_count": len(data),
            "ends_with_newline": ends_with_newline,
        },
    )
    for number, line in enumerate(lines, start=1):
        sink.add(
            "text_lines",
            {
                "_artifact": artifact,
                "_locator": locator(artifact, f"L{number}"),
                "line": number,
                "text": line,
            },
        )
