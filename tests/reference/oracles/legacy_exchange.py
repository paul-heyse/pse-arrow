# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Finite private JSON response boundary for the four fixed scientific cases."""

from __future__ import annotations

from typing import TYPE_CHECKING, Protocol, TypeVar

import msgspec

if TYPE_CHECKING:
    from collections.abc import Iterable, Mapping
    from pathlib import Path

Request = TypeVar("Request")


class Comparison(Protocol):
    """The variable-length fields shared by all four legacy outcomes."""

    @property
    def id(self) -> str:
        """Return the original candidate identity."""
        ...

    @property
    def status(self) -> str:
        """Return the explicit comparison disposition."""
        ...

    @property
    def reasons(self) -> list[str]:
        """Return any retained failure reason."""
        ...


def response_limit(candidate_count: int) -> int:
    """Size the response budget from the admitted candidate population."""
    if not 1 <= candidate_count <= 32:
        message = "reference candidate population outside its finite extent"
        raise ValueError(message)
    return (64 << 10) + candidate_count * (8 << 10)


def read_request(path: Path, request_type: type[Request]) -> Request:
    """Check the input extent before decoding the case's typed request."""
    limit = 2 << 20
    with path.open("rb") as stream:
        encoded = stream.read(limit + 1)
    if len(encoded) > limit:
        message = "reference request exceeds its byte limit"
        raise ValueError(message)
    return msgspec.json.decode(encoded, type=request_type)


def admit_candidates(identities: Iterable[str], candidate_count: int) -> None:
    """Check population and candidate identity bytes before the oracle runs."""
    response_limit(candidate_count)
    if any(not 1 <= len(identity.encode()) <= 128 for identity in identities):
        message = "reference candidate identity outside its finite extent"
        raise ValueError(message)


def write_response(
    path: Path,
    response: msgspec.Struct,
    candidate_count: int,
    environment: Mapping[str, str],
    comparisons: Iterable[Comparison],
) -> None:
    """Admit variable-length fields and encode before publishing a new response."""
    limit = response_limit(candidate_count)
    if len(environment) > 48 or sum(
        len(name.encode()) + len(value.encode()) for name, value in environment.items()
    ) > (8 << 10):
        message = "reference environment exceeds its extent"
        raise ValueError(message)
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
            message = "reference comparison exceeds its extent"
            raise ValueError(message)
    if count != candidate_count:
        message = "reference comparison population differs"
        raise ValueError(message)
    # UTF-8 text may expand up to sixfold under JSON escaping. The fixed cases'
    # numerical records have already passed their case/coordinate checks.
    payload = msgspec.json.encode(response)
    if len(payload) > limit:
        message = "reference response exceeds its byte limit"
        raise ValueError(message)
    with path.open("xb") as stream:
        stream.write(payload)
