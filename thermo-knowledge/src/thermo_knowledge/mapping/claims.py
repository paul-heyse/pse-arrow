# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The phase-1 output of a mapping: identity claims and the row ledger.

`tk map <id> --phase identity` writes, under `<canonical>/<id>/_identity/`:

| File | Holds |
|---|---|
| `claims.source_entity.parquet` | one row per (source entity, source row) that names it, with the entity's class |
| `claims.identity_assertion.parquet` | one row per identifier a source row gives for an entity |
| `claims.mixture_component.parquet` | one row per component a source row gives a defined mixture, with its fraction as decimal text |
| `ledger.parquet` | the state each source row reached in phase 1 (emitted, lossy or held) |
| `rules.parquet` | per value rule, the mapped rows phase 1 applied it to |
| `manifest.json` | the reuse key, the carrier and the formula scopes |

A source entity cannot be a canonical row before resolution (its status, rule and target do not
exist yet), so the claims are not canonical tables: they carry the entity's key, the claimed
values and the source row each came from, and resolution turns them into the canonical
`source_entity` and `identity_assertion` rows.
"""

from __future__ import annotations

import decimal
from collections.abc import Mapping
from dataclasses import dataclass
from pathlib import Path

import pyarrow as pa
import pyarrow.parquet as pq

from thermo_knowledge import pipeline_contract as pc

IDENTITY_DIR = "_identity"
SOURCE_ENTITY_FILE = "claims.source_entity.parquet"
ASSERTION_FILE = "claims.identity_assertion.parquet"
COMPONENT_FILE = "claims.mixture_component.parquet"
LEDGER_FILE = "ledger.parquet"
RULES_FILE = "rules.parquet"

SOURCE_ENTITY_CLAIMS = pa.schema(
    [
        pa.field("manifest_id", pa.string(), nullable=False),
        pa.field("scope", pa.string(), nullable=False),
        pa.field("local_key", pa.string(), nullable=False),
        pa.field("entity_class", pa.string(), nullable=False),
        pa.field("aggregation", pa.string()),
        pa.field("polymorph", pa.string()),
        pa.field("stated_charge", pa.int64()),
        pa.field("mixture_definition", pa.string()),
        pa.field("mole_basis", pa.bool_()),
        pa.field("origin_role", pa.string(), nullable=False),
        pa.field("_artifact", pa.string(), nullable=False),
        pa.field("_locator", pa.string(), nullable=False),
    ]
)

ASSERTION_CLAIMS = pa.schema(
    [
        pa.field("manifest_id", pa.string(), nullable=False),
        pa.field("scope", pa.string(), nullable=False),
        pa.field("local_key", pa.string(), nullable=False),
        pa.field("scheme", pa.string(), nullable=False),
        pa.field("value", pa.string(), nullable=False),
        pa.field("column", pa.string(), nullable=False),
        pa.field("_artifact", pa.string(), nullable=False),
        pa.field("_locator", pa.string(), nullable=False),
    ]
)

COMPONENT_CLAIMS = pa.schema(
    [
        pa.field("manifest_id", pa.string(), nullable=False),
        pa.field("scope", pa.string(), nullable=False),
        pa.field("local_key", pa.string(), nullable=False),
        pa.field("component_scope", pa.string(), nullable=False),
        pa.field("component_key", pa.string(), nullable=False),
        pa.field("fraction", pa.string(), nullable=False),
        pa.field("origin_role", pa.string(), nullable=False),
        pa.field("_artifact", pa.string(), nullable=False),
        pa.field("_locator", pa.string(), nullable=False),
    ]
)

LEDGER = pa.schema(
    [
        pa.field("table", pa.string(), nullable=False),
        pa.field("locator", pa.string(), nullable=False),
        pa.field("state", pa.string(), nullable=False),
        pa.field("lossy", pa.bool_(), nullable=False),
        pa.field("reason", pa.string()),
        pa.field("detail", pa.string()),
    ]
)

RULE_USE = pa.schema(
    [
        pa.field("table", pa.string(), nullable=False),
        pa.field("column", pa.string(), nullable=False),
        pa.field("rows", pa.int64(), nullable=False),
    ]
)

EMITTED = "emitted"
HELD = pc.ROW_STATE.member("held")
MAPPED = pc.TABLE_DISPOSITION.member("mapped")
DEFERRED = pc.TABLE_DISPOSITION.member("deferred")
OUT_OF_SCOPE = pc.TABLE_DISPOSITION.member("out_of_scope")


@dataclass(frozen=True)
class EntityClaim:
    """One source row's claim that a source entity exists."""

    manifest_id: str
    scope: str
    local_key: str
    entity_class: str
    aggregation: str | None
    polymorph: str | None
    stated_charge: int | None
    mixture_definition: str | None
    """For a defined mixture: whether its composition is by definition or by measurement."""
    mole_basis: bool | None
    """For a defined mixture: whether its fractions are mole fractions (else mass fractions)."""
    origin_role: str
    artifact: str
    locator: str

    @property
    def key(self) -> tuple[str, str, str]:
        return (self.manifest_id, self.scope, self.local_key)


@dataclass(frozen=True)
class ComponentClaim:
    """One source row's claim that a source entity is a component of a defined mixture, with its
    fraction as decimal text (`decimal_text`) on the mixture's basis."""

    manifest_id: str
    scope: str
    local_key: str
    component_scope: str
    component_key: str
    fraction: str
    origin_role: str
    artifact: str
    locator: str

    @property
    def mixture(self) -> tuple[str, str, str]:
        return (self.manifest_id, self.scope, self.local_key)

    @property
    def component(self) -> tuple[str, str, str]:
        return (self.manifest_id, self.component_scope, self.component_key)


@dataclass(frozen=True)
class AssertionClaim:
    """One identifier a source row gives for a source entity."""

    manifest_id: str
    scope: str
    local_key: str
    scheme: str
    value: str
    column: str
    artifact: str
    locator: str

    @property
    def entity(self) -> tuple[str, str, str]:
        return (self.manifest_id, self.scope, self.local_key)


def decimal_text(value: float | int | str) -> str:
    """A fraction as exact decimal text: the shortest decimal that reads back as the source's
    number (`repr` of a float), written without exponent or trailing zeros. The same number is
    always the same text, so a key built from it is stable."""
    number = decimal.Decimal(repr(value) if isinstance(value, float) else str(value))
    if not number.is_finite():
        raise ValueError(f"{value!r} is not a finite number")
    text = format(number.normalize(), "f")
    return "0" if text in ("-0", "") else text


def write_claims(
    directory: Path,
    entities: list[EntityClaim],
    assertions: list[AssertionClaim],
    components: list[ComponentClaim],
    ledger: list[dict[str, object]],
    rule_use: Mapping[tuple[str, str], int],
) -> dict[str, int]:
    """Write the claim files under `directory`, rows in a fixed order; returns the row counts."""
    entity_rows = sorted(
        (
            {
                "manifest_id": c.manifest_id,
                "scope": c.scope,
                "local_key": c.local_key,
                "entity_class": c.entity_class,
                "aggregation": c.aggregation,
                "polymorph": c.polymorph,
                "stated_charge": c.stated_charge,
                "mixture_definition": c.mixture_definition,
                "mole_basis": c.mole_basis,
                "origin_role": c.origin_role,
                "_artifact": c.artifact,
                "_locator": c.locator,
            }
            for c in entities
        ),
        key=lambda row: (row["scope"], row["local_key"], row["_locator"]),
    )
    assertion_rows = sorted(
        (
            {
                "manifest_id": c.manifest_id,
                "scope": c.scope,
                "local_key": c.local_key,
                "scheme": c.scheme,
                "value": c.value,
                "column": c.column,
                "_artifact": c.artifact,
                "_locator": c.locator,
            }
            for c in assertions
        ),
        key=lambda row: (row["scope"], row["local_key"], row["scheme"], row["value"], row["_locator"]),
    )
    component_rows = sorted(
        (
            {
                "manifest_id": c.manifest_id,
                "scope": c.scope,
                "local_key": c.local_key,
                "component_scope": c.component_scope,
                "component_key": c.component_key,
                "fraction": c.fraction,
                "origin_role": c.origin_role,
                "_artifact": c.artifact,
                "_locator": c.locator,
            }
            for c in components
        ),
        key=lambda row: (
            row["scope"],
            row["local_key"],
            row["component_scope"],
            row["component_key"],
            row["_locator"],
        ),
    )
    ledger_rows = sorted(ledger, key=lambda row: (str(row["table"]), str(row["locator"])))
    use_rows = [
        {"table": table, "column": column, "rows": rows}
        for (table, column), rows in sorted(rule_use.items())
    ]
    for file, schema, rows in (
        (SOURCE_ENTITY_FILE, SOURCE_ENTITY_CLAIMS, entity_rows),
        (ASSERTION_FILE, ASSERTION_CLAIMS, assertion_rows),
        (COMPONENT_FILE, COMPONENT_CLAIMS, component_rows),
        (LEDGER_FILE, LEDGER, ledger_rows),
        (RULES_FILE, RULE_USE, use_rows),
    ):
        pq.write_table(
            pa.Table.from_pylist(rows, schema=schema), directory / file, compression="zstd"
        )
    return {
        "source_entity_claims": len(entity_rows),
        "identity_assertion_claims": len(assertion_rows),
        "mixture_component_claims": len(component_rows),
        "ledger": len(ledger_rows),
        "rule_use": len(use_rows),
    }


def read_entity_claims(directory: Path) -> list[EntityClaim]:
    """The source-entity claims under `directory`."""
    return [
        EntityClaim(
            row["manifest_id"],
            row["scope"],
            row["local_key"],
            row["entity_class"],
            row["aggregation"],
            row["polymorph"],
            row["stated_charge"],
            row["mixture_definition"],
            row["mole_basis"],
            row["origin_role"],
            row["_artifact"],
            row["_locator"],
        )
        for row in pq.read_table(directory / SOURCE_ENTITY_FILE).to_pylist()
    ]


def read_assertion_claims(directory: Path) -> list[AssertionClaim]:
    """The identity-assertion claims under `directory`."""
    return [
        AssertionClaim(
            row["manifest_id"],
            row["scope"],
            row["local_key"],
            row["scheme"],
            row["value"],
            row["column"],
            row["_artifact"],
            row["_locator"],
        )
        for row in pq.read_table(directory / ASSERTION_FILE).to_pylist()
    ]


def read_component_claims(directory: Path) -> list[ComponentClaim]:
    """The mixture-component claims under `directory`."""
    return [
        ComponentClaim(
            row["manifest_id"],
            row["scope"],
            row["local_key"],
            row["component_scope"],
            row["component_key"],
            row["fraction"],
            row["origin_role"],
            row["_artifact"],
            row["_locator"],
        )
        for row in pq.read_table(directory / COMPONENT_FILE).to_pylist()
    ]


def read_rule_use(directory: Path) -> dict[tuple[str, str], int]:
    """The rows each value rule was applied to in phase 1, by (table, column)."""
    return {
        (row["table"], row["column"]): row["rows"]
        for row in pq.read_table(directory / RULES_FILE).to_pylist()
    }


def read_ledger(directory: Path) -> list[dict[str, object]]:
    """The phase-1 row ledger under `directory`."""
    return pq.read_table(directory / LEDGER_FILE).to_pylist()
