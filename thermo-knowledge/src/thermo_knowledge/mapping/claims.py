# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The phase-1 output of a mapping: identity claims and the row ledger.

`tk map <id> --phase identity` writes, under `<canonical>/<id>/_identity/`:

| File | Holds |
|---|---|
| `claims.source_entity.parquet` | one row per (source entity, source row) that names it |
| `claims.identity_assertion.parquet` | one row per identifier a source row gives for an entity |
| `ledger.parquet` | the state each source row reached in phase 1 (emitted, lossy or held) |
| `manifest.json` | the reuse key, the carrier and the formula scopes |

A source entity cannot be a canonical row before resolution (its status, rule and target do not
exist yet), so the claims are not canonical tables: they carry the entity's key, the claimed
values and the source row each came from, and resolution turns them into the canonical
`source_entity` and `identity_assertion` rows.
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

import pyarrow as pa
import pyarrow.parquet as pq

from thermo_knowledge import pipeline_contract as pc

IDENTITY_DIR = "_identity"
SOURCE_ENTITY_FILE = "claims.source_entity.parquet"
ASSERTION_FILE = "claims.identity_assertion.parquet"
LEDGER_FILE = "ledger.parquet"

SOURCE_ENTITY_CLAIMS = pa.schema(
    [
        pa.field("manifest_id", pa.string(), nullable=False),
        pa.field("scope", pa.string(), nullable=False),
        pa.field("local_key", pa.string(), nullable=False),
        pa.field("aggregation", pa.string()),
        pa.field("polymorph", pa.string()),
        pa.field("stated_charge", pa.int64()),
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

LEDGER = pa.schema(
    [
        pa.field("table", pa.string(), nullable=False),
        pa.field("locator", pa.string(), nullable=False),
        pa.field("state", pa.string(), nullable=False),
        pa.field("lossy", pa.bool_(), nullable=False),
        pa.field("reason", pa.string()),
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
    aggregation: str | None
    polymorph: str | None
    stated_charge: int | None
    origin_role: str
    artifact: str
    locator: str

    @property
    def key(self) -> tuple[str, str, str]:
        return (self.manifest_id, self.scope, self.local_key)


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


def write_claims(
    directory: Path,
    entities: list[EntityClaim],
    assertions: list[AssertionClaim],
    ledger: list[dict[str, object]],
) -> dict[str, int]:
    """Write the three files under `directory`, rows in a fixed order; returns the row counts."""
    entity_rows = sorted(
        (
            {
                "manifest_id": c.manifest_id,
                "scope": c.scope,
                "local_key": c.local_key,
                "aggregation": c.aggregation,
                "polymorph": c.polymorph,
                "stated_charge": c.stated_charge,
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
    ledger_rows = sorted(ledger, key=lambda row: (str(row["table"]), str(row["locator"])))
    for file, schema, rows in (
        (SOURCE_ENTITY_FILE, SOURCE_ENTITY_CLAIMS, entity_rows),
        (ASSERTION_FILE, ASSERTION_CLAIMS, assertion_rows),
        (LEDGER_FILE, LEDGER, ledger_rows),
    ):
        pq.write_table(
            pa.Table.from_pylist(rows, schema=schema), directory / file, compression="zstd"
        )
    return {
        "source_entity_claims": len(entity_rows),
        "identity_assertion_claims": len(assertion_rows),
        "ledger": len(ledger_rows),
    }


def read_entity_claims(directory: Path) -> list[EntityClaim]:
    """The source-entity claims under `directory`."""
    return [
        EntityClaim(
            row["manifest_id"],
            row["scope"],
            row["local_key"],
            row["aggregation"],
            row["polymorph"],
            row["stated_charge"],
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


def read_ledger(directory: Path) -> list[dict[str, object]]:
    """The phase-1 row ledger under `directory`."""
    return pq.read_table(directory / LEDGER_FILE).to_pylist()
