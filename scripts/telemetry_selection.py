# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Finite telemetry selections in the original native member-pack custody."""

from __future__ import annotations

import base64
import hashlib
import tempfile
import time
from pathlib import Path
from typing import Annotated, Literal

from pydantic import Field, JsonValue

from scripts.capabilities.read_contracts import Contract
from scripts.telemetry_investigation import QueryAnswer, schema_description

MAX_DOCUMENT_BYTES = 8 << 20


class Investigation(Contract):
    version: Literal[1] = 1
    created_at_ns: int
    schema_description: dict[str, JsonValue]
    selections: Annotated[list[QueryAnswer], Field(min_length=1, max_length=16)]
    missing_context: Annotated[list[str], Field(max_length=64)] = Field(
        default_factory=list
    )
    interpretation: str = "Finite copied observations; separate source cuts. Sealing does not establish live population completeness, clock alignment or scientific outcome."


class SelectionReference(Contract):
    version: Literal[1] = 1
    directory: Annotated[str, Field(min_length=1, max_length=4096)]
    sha256: Annotated[str, Field(pattern=r"^[0-9a-f]{64}$")]


class Cursor(Contract):
    version: Literal[1] = 1
    sha256: Annotated[str, Field(pattern=r"^[0-9a-f]{64}$")]
    selection: Annotated[int, Field(ge=0, le=15)]
    offset: Annotated[int, Field(ge=0, le=1000)]
    limit: Annotated[int, Field(ge=1, le=1000)]


class SelectionPage(Contract):
    reference: SelectionReference
    selection: int
    request: dict[str, JsonValue]
    source: dict[str, JsonValue] | None
    records: list[dict[str, JsonValue]]
    copied_rows: int
    returned_rows: int
    copied_selection_complete: bool
    live_population_complete: Literal[False] = False
    next_cursor: str | None
    missing_context: list[str]
    interpretation: str


def export(
    destination: Path,
    selections: list[QueryAnswer],
    *,
    maximum_output_bytes: int = MAX_DOCUMENT_BYTES,
    missing_context: list[str] | None = None,
) -> SelectionReference:
    """Explicit action: copy supplied answers, not a second query or native operation."""
    from pse import diagnostics  # noqa: PLC0415 -- explicit native publication only
    from pse.contracts import documents as d  # noqa: PLC0415

    if (
        type(maximum_output_bytes) is not int
        or not 1 <= maximum_output_bytes <= MAX_DOCUMENT_BYTES
    ):
        raise ValueError("Select a finite investigation output allowance up to 8 MiB")
    document = Investigation(
        created_at_ns=time.time_ns(),
        schema_description=schema_description(),
        selections=selections,
        missing_context=missing_context or [],
    )
    raw = document.model_dump_json().encode()
    if len(raw) > maximum_output_bytes:
        raise ValueError("Investigation exceeds the selected output allowance")
    destination = destination.absolute()
    if ".." in destination.parts or any(
        path.is_symlink() for path in (destination, *destination.parents)
    ):
        raise ValueError("Select an ordinary explicit investigation destination")
    with tempfile.TemporaryDirectory(
        prefix="pse-telemetry-copy-", dir=destination.parent
    ) as staging:
        supplied = Path(staging) / "investigation.json"
        supplied.write_bytes(raw)
        diagnostics.export_member_pack(
            destination,
            d.PackExportRequest(
                selection=d.PackSelection(
                    members=(
                        d.PackMemberSelection(
                            id="investigation",
                            role="query",
                            required=True,
                            format="json",
                            document_version=1,
                            source="openobserve-1.1.2-v1; individually recorded source cuts",
                        ),
                    ),
                    maximum_output_bytes=maximum_output_bytes,
                    maximum_document_bytes=MAX_DOCUMENT_BYTES,
                    missing_context=tuple(document.missing_context),
                ),
                files=(
                    d.PackFileInput(
                        member="investigation",
                        path=str(supplied),
                        readable_prefix_bytes=len(raw),
                    ),
                ),
            ),
        )
    return SelectionReference(
        directory=str(destination), sha256=hashlib.sha256(raw).hexdigest()
    )


def read_document(reference: SelectionReference) -> Investigation:
    """Original protected member reader plus the selected immutable byte identity."""
    from pse import diagnostics  # noqa: PLC0415 -- original native custody reader

    directory = Path(reference.directory)
    if (
        not directory.is_absolute()
        or ".." in directory.parts
        or any(path.is_symlink() for path in (directory, *directory.parents))
    ):
        raise ValueError("Select the original supplied investigation directory")
    raw = diagnostics.pack_member(
        directory, "investigation", maximum_bytes=MAX_DOCUMENT_BYTES
    )
    if hashlib.sha256(raw).hexdigest() != reference.sha256:
        raise ValueError("Selected investigation digest differs from supplied member")
    return Investigation.model_validate_json(raw)


def page(
    reference: SelectionReference,
    *,
    selection: int = 0,
    limit: int = 100,
    cursor: str | None = None,
) -> SelectionPage:
    """Stable pages of one exact copied population; no current clock/backend reads."""
    if (
        type(selection) is not int
        or type(limit) is not int
        or not 0 <= selection < 16
        or not 1 <= limit <= 1000
    ):
        raise ValueError("Invalid copied selection page")
    offset = 0
    if cursor is not None:
        if len(cursor) > 2048:
            raise ValueError("Continuation exceeds its bound")
        try:
            continuation = Cursor.model_validate_json(
                base64.urlsafe_b64decode(cursor.encode())
            )
        except (ValueError, UnicodeError) as error:
            raise ValueError("Invalid copied selection continuation") from error
        if (continuation.sha256, continuation.selection, continuation.limit) != (
            reference.sha256,
            selection,
            limit,
        ):
            raise ValueError("Continuation belongs to a different immutable selection")
        offset = continuation.offset
    document = read_document(reference)
    if selection >= len(document.selections):
        raise ValueError("Unselected investigation member")
    answer = document.selections[selection]
    if offset > len(answer.records):
        raise ValueError("Continuation is outside the supplied copied population")
    records = answer.records[offset : offset + limit]
    end = offset + len(records)
    next_cursor = (
        base64.urlsafe_b64encode(
            Cursor(
                sha256=reference.sha256, selection=selection, offset=end, limit=limit
            )
            .model_dump_json()
            .encode()
        ).decode()
        if end < len(answer.records)
        else None
    )
    return SelectionPage(
        reference=reference,
        selection=selection,
        request=answer.request.model_dump(mode="json"),
        source=answer.source.model_dump(mode="json")
        if answer.source is not None
        else None,
        records=records,
        copied_rows=len(answer.records),
        returned_rows=len(records),
        copied_selection_complete=end == len(answer.records),
        next_cursor=next_cursor,
        missing_context=[
            *document.missing_context,
            *answer.limits,
            *([answer.reason] if answer.reason else []),
        ],
        interpretation=document.interpretation,
    )


def resource_reference(reference: SelectionReference) -> str:
    return base64.urlsafe_b64encode(reference.model_dump_json().encode()).decode()


def from_resource(value: str) -> SelectionReference:
    if len(value) > 8192:
        raise ValueError("Selected reference exceeds its bound")
    return SelectionReference.model_validate_json(
        base64.urlsafe_b64decode(value.encode())
    )
