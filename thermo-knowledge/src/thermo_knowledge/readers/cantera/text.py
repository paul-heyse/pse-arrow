# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Text files kept verbatim: one row per line, so a file that is not decomposed into columns
is still read losslessly.

A file that is valid UTF-8 is decoded as UTF-8; any other is decoded as latin-1, which maps each
byte to one character, so the bytes are recoverable. Lines are split on `\\n` only; a trailing
`\\r` stays in the text.
"""

from __future__ import annotations

from pathlib import Path

from thermo_knowledge.readers.cantera.common import Sink, locator


def read_text_lines(tree: Path, artifact: str, sink: Sink) -> None:
    data = (tree / artifact).read_bytes()
    try:
        content, encoding = data.decode("utf-8"), "utf-8"
    except UnicodeDecodeError:
        content, encoding = data.decode("latin-1"), "latin-1"
    lines = content.split("\n")
    if content.endswith("\n") or content == "":
        lines.pop()
    sink.add(
        "text_files",
        {
            "_artifact": artifact,
            "_locator": locator(artifact, f"L1-L{len(lines)}" if lines else "L0"),
            "encoding": encoding,
            "line_count": len(lines),
            "byte_count": len(data),
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
