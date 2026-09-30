# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""A small intermediate form of the generated DDL: tables, types and their rendering.

Everything the generator emits is first described here, so the same description yields the
SQL text, the identifiers checked against PostgreSQL's limits and the catalogue of tables the
build inserts into.
"""

from __future__ import annotations

from dataclasses import dataclass, field

from thermo_knowledge.generate.sqltext import columns, ident, literal, qname


@dataclass(frozen=True, kw_only=True)
class Column:
    name: str
    sql_type: str
    comment: str
    construct: str
    not_null: bool = True
    default: str | None = None
    generated: str | None = None


@dataclass(frozen=True, kw_only=True)
class Constraint:
    """A named constraint written inside `CREATE TABLE`.

    `index_backed` constraints (primary key, unique, exclusion) create an index whose name
    shares the schema's relation namespace.
    """

    name: str
    body: str
    construct: str
    index_backed: bool = False


@dataclass(frozen=True, kw_only=True)
class ForeignKey:
    name: str
    columns: tuple[str, ...]
    ref_schema: str
    ref_table: str
    ref_columns: tuple[str, ...]
    construct: str


@dataclass(kw_only=True)
class Table:
    schema: str
    name: str
    comment: str
    construct: str
    module: str | None = None
    columns: list[Column] = field(default_factory=list)
    constraints: list[Constraint] = field(default_factory=list)
    foreign_keys: list[ForeignKey] = field(default_factory=list)

    @property
    def qualified(self) -> str:
        return qname(self.schema, self.name)


@dataclass(frozen=True, kw_only=True)
class TypeDef:
    """A type or domain created by one statement and described by one comment."""

    object_kind: str  # `TYPE` or `DOMAIN`, as `COMMENT ON` spells it
    schema: str
    name: str
    create: str
    comment: str
    construct: str
    module: str | None = None


def primary_key(cols: list[str] | tuple[str, ...]) -> str:
    return f"PRIMARY KEY ({columns(cols)})"


def unique(cols: list[str] | tuple[str, ...]) -> str:
    return f"UNIQUE ({columns(cols)})"


def check(expression: str) -> str:
    return f"CHECK ({expression})"


def render_table(table: Table) -> str:
    """`CREATE TABLE` and the comments of the table and its columns."""
    lines: list[str] = []
    for column in table.columns:
        parts = [ident(column.name), column.sql_type]
        if column.generated is not None:
            parts.append(f"GENERATED ALWAYS AS ({column.generated}) STORED")
        if column.not_null:
            parts.append("NOT NULL")
        if column.default is not None:
            parts.append(f"DEFAULT {column.default}")
        lines.append("    " + " ".join(parts))
    for constraint in table.constraints:
        lines.append(f"    CONSTRAINT {ident(constraint.name)} {constraint.body}")
    statements = [f"CREATE TABLE {table.qualified} (\n" + ",\n".join(lines) + "\n);"]
    statements.append(f"COMMENT ON TABLE {table.qualified} IS {literal(table.comment)};")
    for column in table.columns:
        statements.append(
            f"COMMENT ON COLUMN {table.qualified}.{ident(column.name)} IS {literal(column.comment)};"
        )
    return "\n".join(statements)


def render_foreign_key(table: Table, fk: ForeignKey) -> str:
    return (
        f"ALTER TABLE {table.qualified} ADD CONSTRAINT {ident(fk.name)} "
        f"FOREIGN KEY ({columns(fk.columns)}) "
        f"REFERENCES {qname(fk.ref_schema, fk.ref_table)} ({columns(fk.ref_columns)}) "
        "DEFERRABLE INITIALLY DEFERRED;"
    )


def render_type(type_def: TypeDef) -> str:
    target = qname(type_def.schema, type_def.name)
    return (
        f"{type_def.create}\n"
        f"COMMENT ON {type_def.object_kind} {target} IS {literal(type_def.comment)};"
    )
