# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The JSON parameter files of the property tree: one row per leaf value, in the file's own keys.

The files under `general_helmholtz/components/parameters/` hold the equation-of-state parameters
of each fluid (`<fluid>.json`: coefficient tables keyed by term number, reference strings, the
auxiliary and transport correlations) and the maps and global constants that select the compiled
expression graphs (`<fluid>_parameters.json`). The files are parameter data, so they are read as
they are written; the compiled expression graphs themselves (`.nl`) are code and are not payload.

A file is a tree of objects, arrays and scalars. Every scalar, and every empty object or array,
is a leaf and becomes one row; nothing is renamed, reordered or interpreted:

- `path` holds the keys and positions from the file root to the leaf, exactly as the file spells
  them (a key verbatim, a position as its decimal index), and `path_steps` says for each step
  whether it is a `key` or an `index`, so a key that looks like a number is never mistaken for a
  position. `pointer` is the same path as an RFC 6901 JSON pointer and is the position in the
  row's locator. `key` and `position` are the leaf's own last step.
- `kind` says what the leaf is (`number`, `string`, `boolean`, `null`, `empty_object`,
  `empty_array`). A number keeps the spelling the file uses in `text` and its value in `number`
  (a float64; an integer beyond 2^53 would round, and the spelling then stays exact). An integer
  and a real are both `number`: `integer` records whether the file wrote an integer literal.
- A string leaf's `text` is the string itself.

A file that is not valid JSON, has an object with a repeated key, or writes a number that is not
finite is refused.
"""

from __future__ import annotations

import json
import math
from collections.abc import Iterator
from dataclasses import dataclass

import pyarrow as pa

from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.schema import BOOL, FLOAT64, INT64, STRING, column, table_schema

NOT_APPLICABLE = "not applicable"
NOT_STATED = "not stated"

SUFFIX = ".json"
KINDS = ("number", "string", "boolean", "null", "empty_object", "empty_array")
KEY = "key"
INDEX = "index"


def _text(
    name: str, source_name: str, *, nullable: bool = True, note: str | None = None
) -> pa.Field:
    return column(
        name, STRING, source_name=source_name, unit=NOT_APPLICABLE, note=note, nullable=nullable
    )


JSON_ENTRIES = table_schema(
    column(
        "path",
        pa.list_(STRING),
        source_name="keys and positions from the file root to the leaf",
        unit=NOT_APPLICABLE,
        note="a key verbatim; a position as its decimal index; path_steps says which",
        nullable=False,
    ),
    column(
        "path_steps",
        pa.list_(STRING),
        source_name="whether each step of path is a key or an index",
        unit=NOT_APPLICABLE,
        nullable=False,
    ),
    _text("pointer", "path as an RFC 6901 JSON pointer", nullable=False),
    _text("key", "the leaf's own key", note="null when the leaf is an array element or the root"),
    column(
        "position",
        INT64,
        source_name="the leaf's own position in its array",
        unit=NOT_APPLICABLE,
        note="null when the leaf is an object member or the root",
    ),
    _text(
        "kind",
        "number, string, boolean, null, empty_object or empty_array",
        nullable=False,
    ),
    column(
        "number",
        FLOAT64,
        source_name="the leaf as a number",
        unit=NOT_STATED,
        note="set for kind number; the file states no unit of its values",
    ),
    column(
        "integer",
        BOOL,
        source_name="whether the file wrote the number as an integer literal",
        unit=NOT_APPLICABLE,
        note="set for kind number",
    ),
    column(
        "flag",
        BOOL,
        source_name="the leaf as a boolean",
        unit=NOT_APPLICABLE,
        note="set for kind boolean",
    ),
    _text(
        "text",
        "the leaf as the file writes it",
        nullable=False,
        note=(
            "a string leaf is the string itself; a number is its spelling in the file (69e6, "
            "0.0000345); true, false, null, {} and [] for the other kinds"
        ),
    ),
)

SCHEMAS = {"json_entries": JSON_ENTRIES}


@dataclass(frozen=True)
class _Number:
    """A number as the file spells it."""

    text: str
    integer: bool


class _Object(tuple):
    """An object as its member pairs in file order."""

    __slots__ = ()


def _object(pairs: list[tuple[str, object]]) -> _Object:
    seen: set[str] = set()
    for key, _ in pairs:
        if key in seen:
            raise ValueError(f"the object has the key {key!r} more than once")
        seen.add(key)
    return _Object(pairs)


def _refuse_constant(name: str) -> object:
    raise ValueError(f"the constant {name} is not a JSON number")


def parse(artifact: str, text: str) -> object:
    """The document with numbers kept as spelled and objects as ordered pairs."""
    try:
        return json.loads(
            text,
            object_pairs_hook=_object,
            parse_float=lambda spelling: _Number(spelling, integer=False),
            parse_int=lambda spelling: _Number(spelling, integer=True),
            parse_constant=_refuse_constant,
        )
    except json.JSONDecodeError as error:
        raise StagingError(
            f"{artifact}#L{error.lineno}: cannot be parsed as JSON: {error.msg}"
        ) from error
    except ValueError as error:
        raise StagingError(f"{artifact}: {error}") from error


def pointer_of(path: tuple[str, ...]) -> str:
    """The RFC 6901 pointer of a path (`~` is `~0`, `/` is `~1`)."""
    return "".join("/" + step.replace("~", "~0").replace("/", "~1") for step in path)


def _leaves(
    node: object, path: tuple[str, ...], steps: tuple[str, ...]
) -> Iterator[tuple[tuple[str, ...], tuple[str, ...], object]]:
    if isinstance(node, _Object):
        if not node:
            yield path, steps, node
        for key, value in node:
            yield from _leaves(value, (*path, key), (*steps, KEY))
    elif isinstance(node, list):
        if not node:
            yield path, steps, node
        for position, value in enumerate(node):
            yield from _leaves(value, (*path, str(position)), (*steps, INDEX))
    else:
        yield path, steps, node


def _leaf_fields(artifact: str, pointer: str, leaf: object) -> dict[str, object]:
    """`kind`, `number`, `integer`, `flag` and `text` of a leaf."""
    fields: dict[str, object] = {"number": None, "integer": None, "flag": None}
    if isinstance(leaf, _Number):
        value = float(leaf.text)
        if not math.isfinite(value):
            raise StagingError(f"{artifact}#{pointer}: the number {leaf.text} is not finite")
        fields.update(kind="number", number=value, integer=leaf.integer, text=leaf.text)
    elif isinstance(leaf, bool):
        fields.update(kind="boolean", flag=leaf, text="true" if leaf else "false")
    elif leaf is None:
        fields.update(kind="null", text="null")
    elif isinstance(leaf, str):
        fields.update(kind="string", text=leaf)
    elif isinstance(leaf, _Object):
        fields.update(kind="empty_object", text="{}")
    else:
        fields.update(kind="empty_array", text="[]")
    return fields


def json_rows(artifact: str, text: str) -> Iterator[dict[str, object]]:
    """The rows of one JSON file, one per leaf in file order."""
    for path, steps, leaf in _leaves(parse(artifact, text), (), ()):
        pointer = pointer_of(path)
        last = steps[-1] if steps else None
        yield {
            "_artifact": artifact,
            "_locator": f"{artifact}#{pointer}",
            "path": list(path),
            "path_steps": list(steps),
            "pointer": pointer,
            "key": path[-1] if last == KEY else None,
            "position": int(path[-1]) if last == INDEX else None,
            **_leaf_fields(artifact, pointer, leaf),
        }
