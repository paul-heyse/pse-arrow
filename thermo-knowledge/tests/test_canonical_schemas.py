# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The Arrow schema of every projected table agrees with the generated DDL."""

from __future__ import annotations

import io

import psycopg
import pyarrow as pa
import pyarrow.parquet as pq
import pytest
from declaration_support import NO_PHYSICAL, full_declaration
from mapping_support import real_declaration

from thermo_knowledge.canonical.schemas import INTERVAL, UUID, canonical_schemas, table_name
from thermo_knowledge.declaration import Declaration
from thermo_knowledge.generate.plan import build_plan
from thermo_knowledge.build import build_database
from thermo_knowledge.testing import TestDatabase

BASE_TYPES = {
    "uuid": UUID,
    "text": pa.string(),
    "bigint": pa.int64(),
    "boolean": pa.bool_(),
    "date": pa.date32(),
    "timestamp with time zone": pa.timestamp("us", tz="UTC"),
    "double precision": pa.float64(),
}


def expected_family(column: tuple[object, ...]) -> str:
    """The Arrow type family a PostgreSQL column must be carried as."""
    _, _, _, _, data_type, udt_name, domain_name = column
    if data_type == "ARRAY":
        return "list"
    if domain_name == "hash":
        return "fixed_size_binary(32)"
    if data_type == "bytea":
        return "binary"
    if data_type == "USER-DEFINED":
        return "struct" if udt_name in ("closed_interval", "float8range") else "string"
    return str(BASE_TYPES[str(data_type)])


def arrow_family(dtype: pa.DataType) -> str:
    if pa.types.is_list(dtype):
        return "list"
    if pa.types.is_struct(dtype):
        return "struct"
    if pa.types.is_fixed_size_binary(dtype) and not isinstance(dtype, pa.ExtensionType):
        return f"fixed_size_binary({dtype.byte_width})"
    return str(dtype)


def columns_of(url: str) -> dict[str, list[tuple[object, ...]]]:
    with psycopg.connect(url) as conn:
        found = conn.execute(
            "SELECT table_schema, table_name, column_name, is_nullable, data_type, udt_name, "
            "domain_name, is_generated FROM information_schema.columns "
            "WHERE table_schema IN ('tk', 'prov', 'ev', 'qual', 'param') "
            "AND (table_schema, table_name) IN "
            "(SELECT table_schema, table_name FROM information_schema.tables WHERE table_type = 'BASE TABLE') "
            "ORDER BY table_schema, table_name, ordinal_position"
        ).fetchall()
    tables: dict[str, list[tuple[object, ...]]] = {}
    for schema, table, name, nullable, data_type, udt, domain, generated in found:
        if generated == "ALWAYS":
            continue
        tables.setdefault(table_name(schema, table), []).append(
            (name, nullable == "YES", table, schema, data_type, udt, domain)
        )
    return tables


def check_against_database(decl: Declaration) -> int:
    schemas = canonical_schemas(decl)
    with TestDatabase() as database:
        build_database(database.url, decl, tree=None if decl is real_declaration() else NO_PHYSICAL)
        database_columns = columns_of(database.url)
    assert set(schemas) == set(database_columns)
    for name, schema in schemas.items():
        declared = database_columns[name]
        assert schema.names == [column[0] for column in declared], name
        for field, column in zip(schema, declared, strict=True):
            assert field.nullable is column[1], f"{name}.{field.name} nullability"
            assert arrow_family(field.type) == expected_family(column), f"{name}.{field.name}"
    return len(schemas)


def test_every_table_of_the_committed_model_matches_the_database() -> None:
    assert check_against_database(real_declaration()) > 100


def test_every_table_of_the_fixture_model_matches_the_database() -> None:
    """The fixture declaration has a set, a range, an array and optional columns."""
    decl = full_declaration()
    assert any(
        pa.types.is_struct(field.type)
        for schema in canonical_schemas(decl).values()
        for field in schema
    )
    check_against_database(decl)


def test_a_slot_group_and_its_family_have_their_generated_columns() -> None:
    schemas = canonical_schemas(real_declaration())
    group = schemas["param.vapor_pressure_exp_series_tau__pure"]
    assert group.names == ["id", "slot_group", "i", "T_r", "p_r"]
    term = schemas["param.vapor_pressure_exp_series_tau__pure__term"]
    assert term.names == ["set_id", "k", "n", "t"]
    assert term.field("k").type == pa.int64()
    assert schemas["prov.record"].names == ["id", "kind"]


def test_a_generated_interval_column_is_not_carried() -> None:
    """The exclusion-constraint range of a piecewise family is computed by PostgreSQL."""
    plan = {table_name(t.schema, t.name): t for t in build_plan(full_declaration()).tables}
    piece = plan["param.nasa7__pure__piece"]
    assert any(column.generated for column in piece.columns)
    assert "interval" not in canonical_schemas(full_declaration())["param.nasa7__pure__piece"].names


def test_identifiers_are_arrow_uuids_and_survive_parquet() -> None:
    schema = canonical_schemas(real_declaration())["tk.species"]
    assert schema.field("id").type == UUID
    assert schema.field("inchikey").type == pa.string()
    table = pa.Table.from_pylist(
        [{"id": None, "charge": 1.0, "inchikey": None}],
        schema=pa.schema(
            [
                pa.field("id", UUID),
                pa.field("charge", pa.float64()),
                pa.field("inchikey", pa.string()),
            ]
        ),
    )
    buffer = io.BytesIO()
    pq.write_table(table, buffer)
    buffer.seek(0)
    assert pq.read_table(buffer).schema.field("id").type == UUID


def test_the_interval_carriage_is_a_pair_of_bounds() -> None:
    assert [field.name for field in INTERVAL] == ["lower", "upper"]


def test_an_unknown_sql_type_is_an_error_not_a_guess() -> None:
    from thermo_knowledge.canonical.schemas import arrow_type, sql_to_arrow

    with pytest.raises(KeyError, match="no Arrow type"):
        arrow_type(sql_to_arrow(real_declaration()), '"meta"."nothing"')
