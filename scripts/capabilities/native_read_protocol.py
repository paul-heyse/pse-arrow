# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Private bounded foreground read frames; every request has independent admission."""

from __future__ import annotations

import asyncio
import hashlib
import json
import struct
import time
from typing import TYPE_CHECKING, Annotated, Literal, TypeVar

from pydantic import Field, JsonValue

from scripts.capabilities.execution_basis import Definition, Handoff
from scripts.capabilities.model import Contract

if TYPE_CHECKING:
    from typing import BinaryIO

MAXIMUM_REQUEST_BYTES = 16 << 20
MAXIMUM_RESPONSE_BYTES = 1 << 20
MAXIMUM_ADMISSION_BYTES = 256 << 10
MAXIMUM_REQUEST_FRAME_BYTES = MAXIMUM_REQUEST_BYTES + MAXIMUM_ADMISSION_BYTES + (4 << 10)
MAXIMUM_RESPONSE_FRAME_BYTES = MAXIMUM_RESPONSE_BYTES + (4 << 10)
SESSION_ENVIRONMENT = "PSE_NATIVE_READ_SESSION"
Token = Annotated[str, Field(pattern=r"^[a-f0-9]{32}$")]
Digest = Annotated[str, Field(pattern=r"^[a-f0-9]{64}$")]
T = TypeVar("T", bound=Contract)


class SessionProtocolError(ValueError):
    """A private transport mismatch; supplied bytes are never rendered."""


class ReadDeadlineExpired(SessionProtocolError):
    """The original observer budget expired before native receiver entry."""


class SessionAdmission(Contract):
    version: Literal[1] = 1
    incarnation: Token
    checkout: str
    worker: Literal["native-read", "native-read-prepared"] = "native-read"
    route_revision: Digest
    definition: Definition


class ReadRequest(Contract):
    version: Literal[1] = 1
    session: Token
    request: Token
    context: Digest
    operation: Annotated[str, Field(min_length=1, max_length=96)]
    arguments: dict[str, JsonValue]
    admission: Handoff
    deadline_monotonic_ns: Annotated[int, Field(ge=0)] | None = None

    def original_document(self) -> bytes:
        """The same independently admitted operation bytes used by the one-shot route."""
        return operation_document(self.operation, self.arguments)

    def require_live_deadline(self) -> None:
        # A queued/read observer never receives a fresh budget upon dispatch.
        if self.deadline_monotonic_ns is not None and time.monotonic_ns() >= self.deadline_monotonic_ns:
            raise ReadDeadlineExpired("native read original deadline expired")


class ReadResponse(Contract):
    version: Literal[1] = 1
    session: Token
    request: Token
    context: Digest
    request_sha256: Digest
    deadline_monotonic_ns: Annotated[int, Field(ge=0)] | None
    retired: bool = False
    result: dict[str, JsonValue]

    def require_request(self, request: ReadRequest) -> None:
        if (
            self.session != request.session
            or self.request != request.request
            or self.context != request.context
            or self.deadline_monotonic_ns != request.deadline_monotonic_ns
            or self.request_sha256 != hashlib.sha256(request.original_document()).hexdigest()
        ):
            raise SessionProtocolError("native read response belongs to another admitted request")


def operation_document(operation: str, arguments: dict[str, JsonValue]) -> bytes:
    return json.dumps({"operation": operation, "arguments": arguments}, allow_nan=False,
                      ensure_ascii=True, separators=(",", ":"), sort_keys=True).encode()


def _unique(pairs: list[tuple[str, JsonValue]]) -> dict[str, JsonValue]:
    result: dict[str, JsonValue] = {}
    for name, value in pairs:
        if name in result:
            raise SessionProtocolError("duplicate private frame key")
        result[name] = value
    return result


def _constant(_value: str) -> None:
    raise SessionProtocolError("nonfinite private frame value")


def decode(raw: bytes, document: type[T], maximum: int) -> T:
    if not raw or len(raw) > maximum:
        raise SessionProtocolError("private native read frame exceeds its original bound")
    try:
        value = json.loads(raw, object_pairs_hook=_unique, parse_constant=_constant)
        return document.model_validate(value, strict=True)
    except (ValueError, TypeError, UnicodeDecodeError) as error:
        raise SessionProtocolError("private native read frame refused") from error


def encoded(document: Contract, maximum: int) -> bytes:
    raw = document.model_dump_json().encode()
    if not raw or len(raw) > maximum:
        raise SessionProtocolError("private native read frame exceeds its original bound")
    return struct.pack("!I", len(raw)) + raw


def _exact(stream: BinaryIO, size: int) -> bytes:
    chunks: list[bytes] = []
    remaining = size
    while remaining:
        chunk = stream.read(remaining)
        if not chunk:
            raise EOFError("native read frame ended before its declared extent")
        chunks.append(chunk)
        remaining -= len(chunk)
    return b"".join(chunks)


def read(stream: BinaryIO, document: type[T], maximum: int) -> T | None:
    prefix = stream.read(1)
    if not prefix:
        return None
    size = struct.unpack("!I", prefix + _exact(stream, 3))[0]
    if not 0 < size <= maximum:
        raise SessionProtocolError("native read frame length refused before allocation")
    return decode(_exact(stream, size), document, maximum)


def write(stream: BinaryIO, document: Contract, maximum: int) -> None:
    frame = encoded(document, maximum)
    offset = 0
    while offset < len(frame):
        size = stream.write(frame[offset:])
        if size is None or size <= 0:
            raise BrokenPipeError("native read frame output detached")
        offset += size
    stream.flush()


async def receive(reader: asyncio.StreamReader, document: type[T], maximum: int) -> T:
    prefix = await reader.readexactly(4)
    size = struct.unpack("!I", prefix)[0]
    if not 0 < size <= maximum:
        raise SessionProtocolError("native read frame length refused before allocation")
    return decode(await reader.readexactly(size), document, maximum)
