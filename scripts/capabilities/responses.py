# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Lightweight response boundaries, independent of registered operation owners."""

from __future__ import annotations

import json
import math
from typing import TYPE_CHECKING, TypeVar, overload

from fastmcp.exceptions import ToolError
from pydantic import JsonValue, TypeAdapter

from scripts.capabilities.diagnostics import refusal
from scripts.capabilities.model import Contract

if TYPE_CHECKING:
    from collections.abc import Callable
T = TypeVar("T")
JSON = TypeAdapter(JsonValue)


class ReadResult(Contract):
    operation: str
    complete: bool
    omissions: list[str]
    payload: dict[str, JsonValue]


def bounded(result: object, operation: str) -> ReadResult:
    value = JSON.validate_python(result)
    if not isinstance(value, dict):
        raise TypeError("host owner must return an object")
    omissions: list[str] = []
    # Reserve space for the envelope, structural punctuation and bounded loss
    # metadata; owner-provided omissions and keys share this response budget.
    remaining = 112 * 1024

    def omit(message: str) -> None:
        if len(omissions) < 31:
            omissions.append(message[:240])
        elif len(omissions) == 31:
            omissions.append(
                "Further detail omissions suppressed; narrow the owner query."
            )

    def walk(item: JsonValue, location: str, depth: int) -> JsonValue:
        nonlocal remaining
        if remaining <= 0 or depth > 12:
            omit(location + ": detail bound exceeded; narrow the request")
            return None
        if isinstance(item, str):
            size = len(json.dumps(item, ensure_ascii=True).encode())
            if size > min(16384, remaining):
                omit(location + ": string detail omitted; narrow the request")
                return None
            remaining -= size
            return item
        if isinstance(item, list):
            if len(item) > 100:
                omit(location + ": showing first 100 items; narrow or page owner query")
            return [
                walk(child, f"{location}[{index}]", depth + 1)
                for index, child in enumerate(item[:100])
            ]
        if isinstance(item, dict):
            if item.get("truncated") is True or item.get("complete") is False:
                omit(location + ": owner reports incomplete or truncated detail")
            gaps = item.get("gaps")
            if isinstance(gaps, list) and gaps:
                omit(location + ": owner reports gaps; inspect original gaps")
            result = {}
            for index, (key, child) in enumerate(item.items()):
                if index == 100:
                    omit(location + ": object detail bound exceeded")
                    break
                key_size = len(json.dumps(key, ensure_ascii=True).encode()) + 4
                if key_size > min(512, remaining):
                    omit(
                        f"{location}: object key at index {index} omitted; narrow the owner query"
                    )
                    continue
                remaining -= key_size
                if key == "text" and location.startswith("$.source_facts"):
                    omit(
                        location
                        + ".text: raw source body omitted; inspect selected source owner"
                    )
                    continue
                child_location = (location + "." + key)[:160]
                result[key] = walk(child, child_location, depth + 1)
            return result
        if isinstance(item, float) and not math.isfinite(item):
            raise ValueError("owner output contains a nonfinite JSON number")
        remaining -= 32
        return item

    projected = walk(value, "$", 0)
    if not isinstance(projected, dict):
        raise TypeError("host projection must remain an object")
    native_omissions = value.get("omissions", [])
    if isinstance(native_omissions, list):
        for item in native_omissions[:32]:
            if isinstance(item, str):
                omit(item)
    response = ReadResult(
        operation=operation,
        complete=not omissions and value.get("complete") is not False,
        omissions=list(dict.fromkeys(omissions)),
        payload=projected,
    )
    if (
        len(json.dumps(response.model_dump(mode="json"), ensure_ascii=True).encode())
        > 128 * 1024
    ):
        # A final encoded-size check covers escaping and every envelope byte.
        # Never return a supposedly bounded response whose metadata escapes it.
        return ReadResult(
            operation=operation[:128],
            complete=False,
            omissions=[
                "Encoded response exceeds the detail bound; narrow the owner query."
            ],
            payload={},
        )
    return response


@overload
def owner_read(operation: str, read: Callable[[], object]) -> ReadResult: ...


@overload
def owner_read(
    operation: str, read: Callable[[], object], *, project: Callable[[object], T]
) -> T: ...


def owner_read(
    operation: str,
    read: Callable[[], object],
    *,
    project: Callable[[object], object] | None = None,
) -> object:
    try:
        value = read()
    except (ValueError, OSError) as error:
        code = (
            "input_refused" if isinstance(error, ValueError) else "source_unavailable"
        )
        raise refusal("pse.workspace." + code, operation=operation) from error
    if project is not None:
        if not isinstance(value, dict):
            raise TypeError("host owner must return an object")
        return project(value)
    return bounded(value, operation)


class HostResult(Contract):
    """Projection completeness is independent from the native result's status."""

    operation: str
    result: dict[str, JsonValue]
    detail_complete: bool
    omissions: list[str]


def project(value: object, operation: str) -> HostResult:
    detail = bounded(value, operation)
    return HostResult(
        operation=operation,
        result=detail.payload,
        detail_complete=detail.complete,
        omissions=detail.omissions,
    )


def failure(operation: str, error: Exception) -> ToolError:
    return ToolError(
        json.dumps(
            {
                "code": "pse.host." + operation + ".refused",
                "cause": type(error).__name__,
                "stage": "host_action",
                "message": "The selected host request could not be served; supplied values are redacted.",
                "remedy": "Inspect the original owner and any issued output before deciding whether another action is safe. No automatic retry or rollback is inferred.",
            }
        )
    )
