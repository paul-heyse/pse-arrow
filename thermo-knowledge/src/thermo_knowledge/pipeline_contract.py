# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The declared shape of the model that the pipeline's stage code depends on (meta-model
section 5), and the one place the code takes those names from.

`pipeline_contract.toml` lists each kind and relation the stages write or read by name, with
the attributes, keys and columns they use and their declared types, and the enums and members
they name. This module loads it once and exposes every entry as a constant:

```python
from thermo_knowledge import pipeline_contract as contract

contract.CARRIER.declared        # "carrier": the kind's declared name
contract.CARRIER.table           # "prov.carrier": the schema-qualified table
contract.CARRIER.tree_hash       # "tree_hash": a listed attribute; any other name is an error
```

A constant whose entry the file does not list fails when this module is imported; an attribute
the file does not list for an entry raises `ContractError` (an `AttributeError`) where it is
read. The declaration loader compares the loaded declaration with the same file
(`declaration/contract_check.py`), so a renamed or reshaped kind is refused at load and never
fails inside a stage.
"""

from __future__ import annotations

import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

CONTRACT_PATH = Path(__file__).with_name("pipeline_contract.toml")
"""The contract file, shipped with the package."""
CONTRACT_DOCUMENT = "thermo_knowledge/pipeline_contract.toml"
"""How diagnostics name the contract file."""

RESERVED = frozenset(
    {"declared", "table", "schema", "identity", "extends", "category", "columns", "listed", "qualified"}
)
"""Names an entry answers itself, so no listed column may use them."""


class ContractError(AttributeError):
    """The code asks for a name the pipeline contract does not list, or the contract file is
    malformed."""


@dataclass(frozen=True)
class Column:
    """One attribute, key or column the code uses, with the type it expects."""

    name: str
    type: str
    optional: bool
    defaulted: bool
    role: Literal["attribute", "key", "column"]


class Entry:
    """A kind or a relation the code depends on.

    `declared` is its name, `table` its schema-qualified table, and every listed attribute,
    key or column reads as its own name."""

    __slots__ = ("category", "columns", "declared", "extends", "identity", "_base", "schema")

    def __init__(
        self,
        declared: str,
        category: Literal["kind", "relation"],
        schema: str,
        columns: dict[str, Column],
        identity: tuple[str, ...] | None,
        extends: str | None,
    ) -> None:
        self.declared = declared
        self.category = category
        self.schema = schema
        self.columns = columns
        self.identity = identity
        self.extends = extends
        self._base: Entry | None = None
        """The entry of the kind this one extends: its attributes read through this entry too."""

    @property
    def table(self) -> str:
        """`schema.name`, the name of the table of the kind or relation."""
        return f"{self.schema}.{self.declared}"

    def __getattr__(self, column: str) -> str:
        if column.startswith("__"):
            raise AttributeError(column)
        entry: Entry | None = self
        found = None
        while entry is not None and found is None:
            found = entry.columns.get(column)
            entry = entry._base
        if found is None:
            listed = ", ".join(self.listed())
            raise ContractError(
                f"the pipeline contract lists no `{column}` for {self.category} "
                f"`{self.declared}` (it lists: {listed})"
            )
        return found.name

    def listed(self) -> list[str]:
        """The names this entry answers for: its own columns and those of the kinds it extends."""
        names: list[str] = []
        entry: Entry | None = self
        while entry is not None:
            names.extend(name for name in entry.columns if name not in names)
            entry = entry._base
        return names

    def qualified(self, column: str) -> str:
        """`schema.table.column` for a listed column."""
        return f"{self.table}.{getattr(self, column)}"

    def __repr__(self) -> str:
        return f"Entry({self.category} {self.declared})"


@dataclass(frozen=True)
class EnumEntry:
    """An enum the code depends on and the members it names."""

    declared: str
    members: tuple[str, ...]

    def member(self, name: str) -> str:
        """`name`, which the contract lists as a member of this enum."""
        if name not in self.members:
            raise ContractError(
                f"the pipeline contract lists no member `{name}` for enum `{self.declared}` "
                f"(it lists: {', '.join(self.members) or 'none'})"
            )
        return name


@dataclass(frozen=True)
class PipelineContract:
    """The parsed contract file."""

    kinds: dict[str, Entry]
    relations: dict[str, Entry]
    enums: dict[str, EnumEntry]

    def kind(self, name: str) -> Entry:
        found = self.kinds.get(name)
        if found is None:
            raise ContractError(f"the pipeline contract lists no kind `{name}`")
        return found

    def relation(self, name: str) -> Entry:
        found = self.relations.get(name)
        if found is None:
            raise ContractError(f"the pipeline contract lists no relation `{name}`")
        return found

    def enum(self, name: str) -> EnumEntry:
        found = self.enums.get(name)
        if found is None:
            raise ContractError(f"the pipeline contract lists no enum `{name}`")
        return found


def _columns(
    table: object, role: Literal["attribute", "key", "column"], where: str
) -> dict[str, Column]:
    if table is None:
        return {}
    if not isinstance(table, dict):
        raise ContractError(f"{where}: expected a table")
    found: dict[str, Column] = {}
    for name, body in table.items():
        if name in RESERVED:
            raise ContractError(f"{where}.{name}: `{name}` is reserved by the contract module")
        if not isinstance(body, dict) or not isinstance(body.get("type"), str):
            raise ContractError(f"{where}.{name}: states its `type`")
        extra = set(body) - {"type", "optional", "defaulted"}
        if extra:
            raise ContractError(f"{where}.{name}: unknown key {sorted(extra)[0]!r}")
        found[name] = Column(
            name=name,
            type=body["type"],
            optional=bool(body.get("optional", False)),
            defaulted=bool(body.get("defaulted", False)),
            role=role,
        )
    return found


def parse(text: str) -> PipelineContract:
    """The contract a file holds; raises `ContractError` when it is malformed."""
    data = tomllib.loads(text)
    unknown = set(data) - {"doc", "kinds", "relations", "enums"}
    if unknown:
        raise ContractError(f"unknown section {sorted(unknown)[0]!r}")
    kinds: dict[str, Entry] = {}
    for name, body in (data.get("kinds") or {}).items():
        where = f"kinds.{name}"
        extra = set(body) - {"schema", "identity", "extends", "attributes"}
        if extra or "schema" not in body:
            raise ContractError(f"{where}: states its `schema` and only `schema`, `identity`, "
                                "`extends` and `attributes`")
        kinds[name] = Entry(
            name,
            "kind",
            body["schema"],
            _columns(body.get("attributes"), "attribute", f"{where}.attributes"),
            tuple(body["identity"]) if "identity" in body else None,
            body.get("extends"),
        )
    relations: dict[str, Entry] = {}
    for name, body in (data.get("relations") or {}).items():
        where = f"relations.{name}"
        extra = set(body) - {"schema", "keys", "columns"}
        if extra or "schema" not in body:
            raise ContractError(f"{where}: states its `schema` and only `schema`, `keys` and `columns`")
        columns = _columns(body.get("keys"), "key", f"{where}.keys")
        columns.update(_columns(body.get("columns"), "column", f"{where}.columns"))
        relations[name] = Entry(name, "relation", body["schema"], columns, None, None)
    enums: dict[str, EnumEntry] = {}
    for name, body in (data.get("enums") or {}).items():
        enums[name] = EnumEntry(name, tuple(body.get("members", ())))
    for entry in kinds.values():
        if entry.extends is not None:
            entry._base = kinds.get(entry.extends)
            if entry._base is None:
                raise ContractError(
                    f"kinds.{entry.declared}: extends `{entry.extends}`, which the contract "
                    "does not list"
                )
    return PipelineContract(kinds, relations, enums)


def load(path: Path = CONTRACT_PATH) -> PipelineContract:
    """The contract in `path` (default: the one shipped with the package)."""
    return parse(path.read_text(encoding="utf-8"))


PACKAGED = load()
"""The contract shipped with the package: what the code's names are taken from and what the
declaration loader checks by default."""


def kind(name: str) -> Entry:
    """The entry of a kind the contract lists."""
    return PACKAGED.kind(name)


def relation(name: str) -> Entry:
    """The entry of a relation the contract lists."""
    return PACKAGED.relation(name)


def enum(name: str) -> EnumEntry:
    """The entry of an enum the contract lists."""
    return PACKAGED.enum(name)


# -- kinds --------------------------------------------------------------------------------------
SOURCE = kind("source")
CARRIER = kind("carrier")
SOFTWARE_RELEASE = kind("software_release")
ARTIFACT = kind("artifact")
IMPORT_RECORD = kind("import_record")
LICENCE = kind("licence")
RIGHTS_DETERMINATION = kind("rights_determination")
TABULATED_FUNCTION = kind("tabulated_function")
TABULATED_AXIS = kind("tabulated_axis")
TABULATED_SERIES = kind("tabulated_series")
PARAMETER_SET = kind("parameter_set")
PARAMETERIZATION = kind("parameterization")
CONVENTION_SET = kind("convention_set")
DERIVATION = kind("derivation")
FIT = kind("fit")
ENVELOPE = kind("envelope")
SOURCE_ENTITY = kind("source_entity")
IDENTITY_ASSERTION = kind("identity_assertion")
MATERIAL_ENTITY = kind("material_entity")
SPECIES = kind("species")
SPECIES_FORM = kind("species_form")
NAMING_SCHEME = kind("naming_scheme")
AGGREGATION = kind("aggregation")
MAPPING_RULE = kind("mapping_rule")
MAPPING_COVERAGE = kind("mapping_coverage")
HELD_ROW = kind("held_row")
QUALIFICATION_RUN = kind("qualification_run")
STANDARD_STATE = kind("standard_state")
DATASET_COLUMN = kind("dataset_column")
SITE_CLASS = kind("site_class")
ASSOCIATION_SITE = kind("association_site")

# -- relations ----------------------------------------------------------------------------------
RECORD_ORIGIN = relation("record_origin")
DERIVATION_OUTPUT = relation("derivation_output")
RESOLUTION_CANDIDATE = relation("resolution_candidate")
RUN_PARAMETER_SET = relation("run_parameter_set")
FIT_FREE_PARAMETER = relation("fit_free_parameter")

# -- enums --------------------------------------------------------------------------------------
ORIGIN_ROLE = enum("origin_role")
COHERENCE = enum("coherence")
DERIVATION_KIND = enum("derivation_kind")
FIT_OUTCOME = enum("fit_outcome")
COMPARISON_BASIS = enum("comparison_basis")
ENVELOPE_KIND = enum("envelope_kind")
RUN_OUTCOME = enum("run_outcome")
RESOLUTION_STATUS = enum("resolution_status")
RESOLUTION_RULE = enum("resolution_rule")
ROW_STATE = enum("row_state")
TABLE_DISPOSITION = enum("table_disposition")
