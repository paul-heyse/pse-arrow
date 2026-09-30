# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""One explicit Arrow schema per projected table (pipeline section 2, canonical rows).

The columns, their order and their nullability are those of the generated DDL: the schemas are
read from the generator's description of every table (`generate.plan.build_plan`), so the two
cannot disagree, and a column whose value PostgreSQL computes (a generated column) is not
carried. Only the type of a column is translated, by the closed mapping below; a SQL type the
mapping does not know is an error, never a guess.

| PostgreSQL | Arrow |
|---|---|
| `uuid` | `uuid` (the canonical extension type over `fixed_size_binary(16)`) |
| `text`, identifier-scheme domains, enums | `string` |
| `bigint`, `boolean`, `date`, `timestamptz` | `int64`, `bool`, `date32`, `timestamp[us, UTC]` |
| quantity domains, `finite_real` | `float64` |
| `meta.hash` | `fixed_size_binary(32)` |
| `meta.closed_interval` (a range) | `struct<lower: float64, upper: float64>` |
| an array of any of these | `list` of it |

Carrying a `uuid` as Arrow's `uuid` type: it is stored in Parquet as 16 bytes with the `UUID`
logical type, survives a Parquet round trip as the same type, and the ADBC PostgreSQL driver
ingests it into a `uuid` column (its binary `COPY` writes the 16 bytes PostgreSQL's `uuid` wire
format is). A `string` does not work: PostgreSQL's binary `uuid` format is not text.
"""

from __future__ import annotations

import pyarrow as pa

from thermo_knowledge.declaration import model as m
from thermo_knowledge.generate import ir, naming
from thermo_knowledge.generate.plan import build_plan
from thermo_knowledge.generate.sqltext import qname

UUID = pa.uuid()
HASH = pa.binary(32)
INTERVAL = pa.struct([pa.field("lower", pa.float64()), pa.field("upper", pa.float64())])
TIMESTAMP = pa.timestamp("us", tz="UTC")

DOC = "doc"
"""Field metadata key holding the column's documentation."""


def table_name(schema: str, table: str) -> str:
    """`schema.table`, the name a canonical table has in a file name and in a mapping."""
    return f"{schema}.{table}"


def sql_to_arrow(decl: m.Declaration) -> dict[str, pa.DataType]:
    """Arrow type of every SQL column type the projection of `decl` uses."""
    types: dict[str, pa.DataType] = {
        "uuid": UUID,
        "text": pa.string(),
        "bigint": pa.int64(),
        "boolean": pa.bool_(),
        "date": pa.date32(),
        "timestamptz": TIMESTAMP,
        qname("meta", "hash"): HASH,
        qname("meta", "finite_real"): pa.float64(),
        qname("meta", "closed_interval"): INTERVAL,
    }
    for scheme in decl.schemes.values():
        types[qname("meta", naming.scheme_domain(scheme.name))] = pa.string()
    for quantity in decl.quantity_types.values():
        types[qname("meta", naming.quantity_domain(quantity.name))] = pa.float64()
    for enum in decl.enums.values():
        types[qname("meta", enum.name)] = pa.string()
    return types


def arrow_type(types: dict[str, pa.DataType], sql_type: str) -> pa.DataType:
    """The Arrow type of one SQL column type (an array of a known type included)."""
    if sql_type.endswith("[]"):
        return pa.list_(arrow_type(types, sql_type.removesuffix("[]")))
    try:
        return types[sql_type]
    except KeyError:
        raise KeyError(f"no Arrow type is defined for the SQL type {sql_type}") from None


def table_schema(types: dict[str, pa.DataType], table: ir.Table) -> pa.Schema:
    """The Arrow schema of one planned table: its stored columns, in DDL order."""
    return pa.schema(
        [
            pa.field(
                column.name,
                arrow_type(types, column.sql_type),
                nullable=not column.not_null,
                metadata={DOC: column.comment},
            )
            for column in table.columns
            if column.generated is None
        ]
    )


def canonical_schemas(decl: m.Declaration) -> dict[str, pa.Schema]:
    """The Arrow schema of every table a canonical row can be written to, keyed by
    `schema.table`: every kind, refinement, relation, slot-group, family and `Set<K>` child
    table, and `prov.record`. The reified `meta` tables are not canonical rows: a build inserts
    them from the declaration."""
    types = sql_to_arrow(decl)
    return {
        table_name(table.schema, table.name): table_schema(types, table)
        for table in build_plan(decl).tables
        if table.schema != m.META_SCHEMA
    }
