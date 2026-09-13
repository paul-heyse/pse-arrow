# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Strict structuring for the Python boundary (blueprint §21.5).

Two serialisation libraries with one division of labour: attrs classes describe
the relation-row contracts and are structured by cattrs; msgspec describes the
manifests and other documents that are read from and written to disk as JSON.

Both are strict. cattrs' default converter *ignores* unknown keys, which turns a
renamed column into a silently missing value; :func:`converter` therefore
installs a structure-hook factory that rebuilds every attrs hook with
``_cattrs_forbid_extra_keys=True``. msgspec structs declare
``forbid_unknown_fields=True`` for the same reason. A payload that does not
match the contract is an error at the boundary, never a default further in.
"""

from collections.abc import Iterable, Mapping
from typing import TypeVar

import attrs
import cattrs
import msgspec
import msgspec.json
from cattrs.gen import make_dict_structure_fn

__all__ = [
    "Manifest",
    "converter",
    "decode_json",
    "encode_json",
    "render_errors",
    "structure_rows",
]

T = TypeVar("T")
S = TypeVar("S", bound=msgspec.Struct)


class Manifest(msgspec.Struct, frozen=True, forbid_unknown_fields=True):
    """The artifact manifest a snapshot is published with (blueprint §21.5).

    Phase 0 carries only the fingerprint fields the Python side verifies; the
    generator grows this struct from the registry in phase 1.

    Attributes:
        schema_version: The manifest generation, so a reader can refuse a
            future one instead of guessing.
        artifact_id: The snapshot's semantic identity, in short form.
        content_hash: blake3 digest over the snapshot's relations.
        lockfile_hash: The toolchain fingerprint the snapshot was produced with.
    """

    schema_version: int
    artifact_id: str
    content_hash: str
    lockfile_hash: str


def converter() -> cattrs.Converter:
    """Build a cattrs converter that refuses unknown keys.

    A fresh converter per call: converters carry mutable hook registries, and a
    module-level singleton would let one caller's registration change another
    caller's parsing.

    Returns:
        A converter whose attrs structure hooks are generated with
        ``_cattrs_forbid_extra_keys=True`` (blueprint §21.5).
    """
    conv = cattrs.Converter()
    conv.register_structure_hook_factory(
        attrs.has,
        lambda cls: make_dict_structure_fn(cls, conv, _cattrs_forbid_extra_keys=True),
    )
    return conv


def structure_rows(rows: Iterable[Mapping[str, object]], cls: type[T]) -> list[T]:
    """Structure an iterable of row mappings into contract instances.

    Args:
        rows: The mappings to structure, typically ``RecordBatch.to_pylist()``.
        cls: The attrs contract class each row must satisfy.

    Returns:
        One instance of ``cls`` per row, in input order.

    Raises:
        cattrs.BaseValidationError: If any row carries an unknown key, is
            missing a field, or holds a value of the wrong type. Render it with
            :func:`render_errors`.
    """
    conv = converter()
    return [conv.structure(row, cls) for row in rows]


def render_errors(exc: Exception) -> list[str]:
    """Flatten a cattrs validation error into one finding per offending field.

    Args:
        exc: The exception raised by :func:`structure_rows`, normally a
            ``cattrs.BaseValidationError``.

    Returns:
        One human-readable finding per leaf error, each naming the path to the
        field it is about. Never a single flattened "structuring failed".
    """
    return list(cattrs.transform_error(exc))


def decode_json(data: bytes | str, type: type[S]) -> S:
    """Decode JSON into a msgspec struct, refusing unknown fields.

    Args:
        data: The JSON document.
        type: The msgspec struct the document must match.

    Returns:
        The decoded struct.

    Raises:
        msgspec.ValidationError: If the document does not match the struct.
        msgspec.DecodeError: If the document is not well-formed JSON.
    """
    return msgspec.json.decode(data, type=type)


def encode_json(obj: msgspec.Struct) -> bytes:
    """Encode a msgspec struct as JSON bytes.

    Args:
        obj: The struct to encode.

    Returns:
        The encoded document, as bytes.
    """
    return msgspec.json.encode(obj)
