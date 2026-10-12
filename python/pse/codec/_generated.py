# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Private, thread-owned conversion for the two generated source-edit boundaries."""

from collections.abc import Iterable
from math import isfinite
from threading import local
from typing import TYPE_CHECKING, TypeVar

import msgspec
import msgspec.inspect

from pse.contracts.authored import (
    AuthoredFitCasesRow,
    AuthoredModelingDeclarationsRow,
)
from pse.contracts.documents import DeclarationEdit, FitDeclarations
from pse.contracts.values import ContentHash, SemanticId

from . import _check_version_headers, converter, encode_json

if TYPE_CHECKING:
    import cattrs

T = TypeVar("T")


class _Converters(local):
    """Only this module can mutate these hooks; public converters remain fresh."""

    def __init__(self) -> None:
        self.rows: cattrs.Converter = converter()
        self.rows.register_unstructure_hook(SemanticId, SemanticId.to_hex)
        self.rows.register_unstructure_hook(ContentHash, ContentHash.to_prefixed)


_converters = _Converters()


def _check_finite(value: object, path: str) -> None:
    # JSON encoding normalizes nonfinite floats to null. Direct conversion must
    # instead refuse an invalid authored value before optional fields can hide it.
    if isinstance(value, float) and not isfinite(value):
        message = f"Expected finite number - at `{path}`"
        raise msgspec.ValidationError(message)
    if isinstance(value, dict):
        for key, child in value.items():
            _check_finite(child, f"{path}.{key}")
    elif isinstance(value, (tuple, list)):
        for index, child in enumerate(value):
            _check_finite(child, f"{path}[{index}]")


def _encode(
    rows: Iterable[object], row_type: type, field: str, document_type: type[T]
) -> bytes:
    plain_rows: list[object] = []
    for row in rows:
        if type(row) is not row_type:
            message = f"Expected generated {row_type.__name__} row"
            raise TypeError(message)
        plain_rows.append(_converters.rows.unstructure(row))
    plain = {field: plain_rows}
    _check_finite(plain, "$")
    _check_version_headers(plain, msgspec.inspect.type_info(document_type), "$")
    return encode_json(msgspec.convert(plain, type=document_type, strict=True))


def encode_declarations(rows: Iterable[AuthoredModelingDeclarationsRow]) -> bytes:
    """Strict generated DTO conversion followed by one JSON encoding."""
    return _encode(
        rows, AuthoredModelingDeclarationsRow, "declarations", DeclarationEdit
    )


def encode_fits(rows: Iterable[AuthoredFitCasesRow]) -> bytes:
    """Use the same isolated hooks for nested fit rows and exact identities."""
    return _encode(rows, AuthoredFitCasesRow, "fits", FitDeclarations)
