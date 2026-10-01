# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""A YAML 1.2 core-schema loader over PyYAML, which implements YAML 1.1.

Cantera's files are read by yaml-cpp, a YAML 1.2 parser, so the same text must resolve the same
way here. The differences that matter for these files:

- booleans are only `true` and `false` (in three spellings each); PyYAML's `no`, `NO`, `n`, `N`,
  `yes`, `on` and `off` are strings, and `name: NO` and `name: N` are species names;
- numbers follow the 1.2 core grammar: no underscores, no sexagesimal (`1:30`), no leading-zero
  octal, and `1e-3` (no dot) is a float;
- there is no timestamp type (a `date:` value stays text).

A key repeated in one mapping keeps its last value, as every YAML loader does; each discarded
earlier occurrence is returned with its line so that the reader can stage it.
Anchors and aliases resolve as usual; the merge key `<<` is not special in YAML 1.2 and is kept
as an ordinary key.
"""

from __future__ import annotations

import math
import re
from dataclasses import dataclass
from typing import Any

import yaml

_BOOL = re.compile(r"^(?:true|True|TRUE|false|False|FALSE)$")
_NULL = re.compile(r"^(?:null|Null|NULL|~|)$")
_INT = re.compile(r"^(?:[-+]?[0-9]+|0o[0-7]+|0x[0-9a-fA-F]+)$")
_FLOAT = re.compile(
    r"^(?:[-+]?(?:\.[0-9]+|[0-9]+(?:\.[0-9]*)?)(?:[eE][-+]?[0-9]+)?"
    r"|[-+]?\.(?:inf|Inf|INF)|\.(?:nan|NaN|NAN))$"
)
_FIRST = {
    "bool": list("tTfF"),
    "null": ["~", "n", "N", ""],
    "int": list("-+0123456789"),
    "float": list("-+0123456789."),
}


class Yaml12Loader(yaml.SafeLoader):
    """`yaml.SafeLoader` with the YAML 1.2 core schema."""

    def __init__(self, stream: str) -> None:
        super().__init__(stream)
        self.duplicates: list[DuplicateKey] = []


Yaml12Loader.yaml_implicit_resolvers = {}
for _tag, _expression in (
    ("tag:yaml.org,2002:bool", _BOOL),
    ("tag:yaml.org,2002:null", _NULL),
    ("tag:yaml.org,2002:int", _INT),
    ("tag:yaml.org,2002:float", _FLOAT),
):
    _name = _tag.rsplit(":", 1)[1]
    for _char in _FIRST[_name]:
        Yaml12Loader.yaml_implicit_resolvers.setdefault(_char, []).append((_tag, _expression))


def _construct_bool(loader: yaml.SafeLoader, node: yaml.ScalarNode) -> bool:
    return str(node.value).lower() == "true"


def _construct_int(loader: yaml.SafeLoader, node: yaml.ScalarNode) -> int:
    text = str(node.value)
    sign = -1 if text.startswith("-") else 1
    body = text.lstrip("+-")
    if body.startswith("0o"):
        return sign * int(body[2:], 8)
    if body.startswith("0x"):
        return sign * int(body[2:], 16)
    return sign * int(body)


def _construct_float(loader: yaml.SafeLoader, node: yaml.ScalarNode) -> float:
    text = str(node.value).lower()
    if text.endswith(".inf"):
        return -math.inf if text.startswith("-") else math.inf
    if text == ".nan":
        return math.nan
    return float(text)


@dataclass(frozen=True)
class DuplicateKey:
    """An earlier occurrence of a key that a later one in the same mapping replaced."""

    key: object
    value: object
    line: int
    """1-based line of the discarded occurrence."""
    replaced_at: int
    """1-based line of the occurrence that replaced it."""


def _construct_mapping(
    loader: Yaml12Loader, node: yaml.MappingNode, deep: bool = False
) -> dict[Any, Any]:
    seen: dict[Any, tuple[int, yaml.Node]] = {}
    for key_node, value_node in node.value:
        if not isinstance(key_node, yaml.ScalarNode):
            continue
        key = loader.construct_object(key_node, deep=True)
        if key in seen:
            first_line, first_value = seen[key]
            loader.duplicates.append(
                DuplicateKey(
                    key,
                    loader.construct_object(first_value, deep=True),
                    first_line,
                    key_node.start_mark.line + 1,
                )
            )
        seen[key] = (key_node.start_mark.line + 1, value_node)
    return yaml.constructor.SafeConstructor.construct_mapping(loader, node, deep)


Yaml12Loader.add_constructor("tag:yaml.org,2002:bool", _construct_bool)
Yaml12Loader.add_constructor("tag:yaml.org,2002:int", _construct_int)
Yaml12Loader.add_constructor("tag:yaml.org,2002:float", _construct_float)
Yaml12Loader.add_constructor("tag:yaml.org,2002:map", _construct_mapping)


def load(text: str) -> tuple[Any, list[DuplicateKey]]:
    """The YAML document `text` with YAML 1.2 core-schema scalars, and the discarded earlier
    occurrences of any duplicated keys (the later occurrence wins, as in every YAML loader)."""
    loader = Yaml12Loader(text)
    try:
        return loader.get_single_data(), loader.duplicates
    finally:
        loader.dispose()
