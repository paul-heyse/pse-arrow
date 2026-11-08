# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Finite private JSON response boundary for the four fixed scientific cases."""

from collections.abc import Iterable, Mapping
from pathlib import Path
from typing import Protocol

import msgspec


class Comparison(Protocol):
    @property
    def id(self) -> str: ...
    @property
    def status(self) -> str: ...
    @property
    def reasons(self) -> list[str]: ...


def response_limit(candidate_count: int) -> int:
    if not 1 <= candidate_count <= 32:
        raise ValueError("reference candidate population outside its finite extent")
    return (64 << 10) + candidate_count * (8 << 10)


def read_request[T](path: Path, request_type: type[T]) -> T:
    limit = 2 << 20
    with path.open("rb") as stream:
        encoded = stream.read(limit + 1)
    if len(encoded) > limit:
        raise ValueError("reference request exceeds its byte limit")
    return msgspec.json.decode(encoded, type=request_type)


def admit_candidates(identities: Iterable[str], candidate_count: int) -> None:
    response_limit(candidate_count)
    if any(not 1 <= len(identity.encode()) <= 128 for identity in identities):
        raise ValueError("reference candidate identity outside its finite extent")


def write_response(
    path: Path,
    response: msgspec.Struct,
    candidate_count: int,
    environment: Mapping[str, str],
    comparisons: Iterable[Comparison],
) -> None:
    limit = response_limit(candidate_count)
    if len(environment) > 48 or sum(
        len(name.encode()) + len(value.encode()) for name, value in environment.items()
    ) > (8 << 10):
        raise ValueError("reference environment exceeds its extent")
    count = 0
    for row in comparisons:
        count += 1
        if (
            count > candidate_count
            or not 1 <= len(row.id.encode()) <= 128
            or row.status not in {"passed", "failed", "missing"}
            or len(row.reasons) > 1
            or any(len(reason.encode()) > 1024 for reason in row.reasons)
        ):
            raise ValueError("reference comparison exceeds its extent")
    if count != candidate_count:
        raise ValueError("reference comparison population differs")
    # UTF-8 text may expand up to sixfold under JSON escaping. The fixed cases'
    # numerical records have already passed their case/coordinate checks.
    payload = msgspec.json.encode(response)
    if len(payload) > limit:
        raise ValueError("reference response exceeds its byte limit")
    with path.open("xb") as stream:
        stream.write(payload)
