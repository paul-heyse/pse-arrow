# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The generated DDL in a real database: build, `meta` rows, fingerprint and constraints."""

from __future__ import annotations

import uuid
from collections.abc import Iterator
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path

import psycopg
import pytest
from declaration_support import NO_PHYSICAL, copy_full, empty_declaration, full_declaration
from psycopg import errors

from thermo_knowledge import identity
from thermo_knowledge.declaration import Declaration, load_declaration
from thermo_knowledge.generate import (
    SCHEMA_PATH,
    declaration_fingerprint,
    generate,
    schema_fingerprint,
)
from thermo_knowledge.generate.reified import insert_batches, serialise
from thermo_knowledge.generate.meta_tables import META_TABLES
from thermo_knowledge.generate.plan import GENERATED_SCHEMAS
from thermo_knowledge.build import BuildResult, build_database
from thermo_knowledge.build.database import DatabaseRefusedError
from thermo_knowledge.schema_build import compare_fingerprint
from thermo_knowledge.testing import TestDatabase


@dataclass
class Built:
    url: str
    decl: Declaration
    result: BuildResult


@pytest.fixture(scope="module")
def built() -> Iterator[Built]:
    """The full fixture declaration built once; tests only read it or roll back."""
    decl = full_declaration()
    with TestDatabase() as database:
        yield Built(database.url, decl, build_database(database.url, decl, tree=NO_PHYSICAL))


@pytest.fixture
def conn(built: Built) -> Iterator[psycopg.Connection]:
    connection = psycopg.connect(built.url)
    # An open transaction makes every `connection.transaction()` below a savepoint, and the
    # final rollback discards whatever a test inserted.
    connection.execute("SELECT 1")
    try:
        yield connection
    finally:
        connection.rollback()
        connection.close()


def new_id() -> uuid.UUID:
    return uuid.uuid4()


def rejected(
    connection: psycopg.Connection,
    error: type[psycopg.Error],
    statement: str,
    params: tuple[object, ...],
    constraint: str | None = None,
) -> None:
    """Run `statement` in a savepoint; it must fail with `error` (naming `constraint`)."""
    with pytest.raises(error) as raised:
        with connection.transaction():
            connection.execute(statement, params)  # type: ignore[arg-type]
    if constraint is not None:
        assert raised.value.diag.constraint_name == constraint


def accepted(connection: psycopg.Connection, statement: str, params: tuple[object, ...]) -> None:
    with connection.transaction():
        connection.execute(statement, params)  # type: ignore[arg-type]


# -- build --------------------------------------------------------------------------------


def test_build_creates_every_schema_and_meta_table(built: Built, conn: psycopg.Connection) -> None:
    schemas = {
        name
        for (name,) in conn.execute(
            "SELECT nspname FROM pg_namespace WHERE nspname = ANY(%s)", (list(GENERATED_SCHEMAS),)
        )
    }
    assert schemas == set(GENERATED_SCHEMAS)
    tables = {
        name
        for (name,) in conn.execute("SELECT tablename FROM pg_tables WHERE schemaname = 'meta'")
    }
    # the tables the declaration defines, and the build's own record of its sources
    assert tables == set(META_TABLES) | {"build_source"}
    assert built.result.meta_rows > 0
    # 18 rows in the entity tables (children of Set attributes included) and the two
    # prov.record rows of the declared entities of `own` kinds
    assert built.result.entity_rows == 20


def test_meta_rows_reify_the_declaration(built: Built, conn: psycopg.Connection) -> None:
    decl = built.decl

    def count(table: str) -> int:
        row = conn.execute(f"SELECT count(*) FROM meta.{table}").fetchone()  # noqa: S608
        assert row is not None
        return row[0]

    assert count("kind") == len(decl.kinds)
    assert count("enum") == len(decl.enums)
    assert count("quantity_type") == len(decl.quantity_types)
    assert count("contract") == len(decl.contracts)
    assert count("form") == len(decl.forms)
    assert count("slot_group") == len(decl.slot_groups)
    assert count("family") == 1
    assert count("subform_slot") == 1
    assert count("relation") == len(decl.relations)
    assert count("module") == len(decl.modules)
    assert count("entity") == len(decl.entities)
    assert count("enum_member") == sum(len(e.members) for e in decl.enums.values())


def test_meta_rows_carry_typed_structure(built: Built, conn: psycopg.Connection) -> None:
    row = conn.execute(
        "SELECT unit, scale, production FROM meta.quantity_type WHERE name = 'Temperature'"
    ).fetchone()
    assert row == ("K", "absolute", "Temperature")
    dims = dict(
        conn.execute(
            "SELECT dimension, exponent FROM meta.unit_dimension WHERE unit = 'J / K ** 3 / mol'"
        ).fetchall()
    )
    assert dims == {"mass": 1, "length": 2, "time": -2, "substance": -1, "temperature": -3}
    assert conn.execute(
        "SELECT unit FROM meta.quantity_expression WHERE expression = 'MolarCp / Temperature^2'"
    ).fetchone() == ("J / K ** 3 / mol",)
    assert conn.execute(
        "SELECT extends, root, provenance, origin FROM meta.kind WHERE name = 'antoine__pure'"
    ).fetchone() == ("parameter_set", "parameter_set", "own", "slot_group")
    assert conn.execute(
        "SELECT array_agg(attribute ORDER BY position) FROM meta.kind_identity "
        "WHERE kind = 'parameter_set'"
    ).fetchone() == (["parameterization", "slot_group", "subject_key"],)
    assert conn.execute(
        "SELECT container, element_kind, element FROM meta.attribute "
        "WHERE kind = 'species' AND name = 'validity'"
    ).fetchone() == ("range", "quantity", "Temperature")
    assert conn.execute(
        "SELECT presence, shape FROM meta.slot WHERE qualified_name = 'critical.global.T_c'"
    ).fetchone() == ("stateful", "quantity")
    assert conn.execute(
        "SELECT shape, accepts, element FROM meta.slot "
        "WHERE qualified_name = 'cubic.pure.alpha_coefficients'"
    ).fetchone() == ("nested_set", "alpha_function", None)
    assert conn.execute(
        "SELECT interval_lower, interval_upper FROM meta.family "
        "WHERE qualified_name = 'nasa7.pure.piece'"
    ).fetchone() == ("T_low", "T_high")
    assert conn.execute(
        "SELECT rule, diagonal FROM meta.transposition WHERE owner = 'kij.pair'"
    ).fetchone() == ("symmetric", "forbidden")
    assert conn.execute(
        "SELECT count(*) FROM meta.transposition_permutation WHERE owner = 'triple'"
    ).fetchone() == (6,)
    assert conn.execute(
        "SELECT role, kind FROM meta.framework_role WHERE role = 'parameter_set'"
    ).fetchone() == ("parameter_set", "parameter_set")
    assert conn.execute("SELECT count(*) FROM meta.trace WHERE trace = 'IC-13'").fetchone() == (1,)
    assert conn.execute(
        "SELECT mark FROM meta.pse_mark WHERE construct = 'kinds.species'"
    ).fetchone() == ("gap:species_marks",)


def test_meta_ids_are_the_deterministic_identifiers(conn: psycopg.Connection) -> None:
    assert conn.execute(
        "SELECT id FROM meta.slot_group WHERE qualified_name = 'antoine.pure'"
    ).fetchone() == (identity.meta_identifier("slot_group", "antoine.pure"),)
    assert conn.execute(
        "SELECT id FROM meta.slot WHERE qualified_name = 'nasa7.pure.piece.T_low'"
    ).fetchone() == (identity.meta_identifier("slot", "nasa7.pure.piece.T_low"),)
    assert conn.execute(
        "SELECT id FROM meta.subform_slot WHERE qualified_name = 'cubic.alpha'"
    ).fetchone() == (identity.meta_identifier("subform_slot", "cubic.alpha"),)


def test_declared_entities_are_inserted_with_deterministic_ids(conn: psycopg.Connection) -> None:
    gas = identity.identifier("aggregation", ["gas"])
    assert conn.execute("SELECT id FROM tk.aggregation WHERE name = 'gas'").fetchone() == (gas,)
    assert conn.execute("SELECT aggregation FROM tk.phase WHERE name = 'vapor'").fetchone() == (
        gas,
    )
    assert conn.execute(
        "SELECT quantity FROM tk.observable WHERE name = 'vapor_pressure'"
    ).fetchone() == (identity.meta_identifier("quantity_type", "Pressure"),)
    assert conn.execute(
        "SELECT temperature_range::text FROM tk.element_group WHERE name = 'light'"
    ).fetchone() == ("[200,400.5]",)
    assert conn.execute("SELECT levels FROM tk.element_group WHERE name = 'light'").fetchone() == (
        [1, 2],
    )
    assert conn.execute("SELECT count(*) FROM tk.element_group__members").fetchone() == (2,)
    assert conn.execute(
        "SELECT atomic_number, molar_mass FROM tk.element WHERE name = 'He'"
    ).fetchone() == (
        2,
        0.0040026,
    )


# -- fingerprint --------------------------------------------------------------------------


def test_fingerprint_is_recorded_on_schema_tk_and_checks(
    built: Built, conn: psycopg.Connection
) -> None:
    (comment,) = conn.execute(
        "SELECT obj_description(oid, 'pg_namespace') FROM pg_namespace WHERE nspname = 'tk'"
    ).fetchone()  # type: ignore[misc]
    expected = schema_fingerprint(
        generate(built.decl)[SCHEMA_PATH], b"", serialise(insert_batches(built.decl))
    )
    assert comment.endswith(f"tk-schema-fingerprint sha256:{expected}")
    assert built.result.fingerprint == expected
    comparison = compare_fingerprint(built.url, built.decl, tree=NO_PHYSICAL)
    assert comparison.matches and comparison.recorded == expected
    assert "matches" in comparison.message


def test_fingerprint_fails_after_the_declaration_changes(built: Built, tmp_path: Path) -> None:
    tree = copy_full(tmp_path)
    (tree / "model" / "extra.toml").write_text(
        'module = "extra"\nschema = "tk"\ndoc = "d"\n\n'
        '[kinds.extra_kind]\ndoc = "d"\nidentity = ["a"]\nprovenance = "none"\n\n'
        '[kinds.extra_kind.attributes]\na = { type = "Text", doc = "d" }\n'
    )
    changed = load_declaration(tree / "model", tree / "forms", contract=None).require()
    comparison = compare_fingerprint(built.url, changed, tree=NO_PHYSICAL)
    assert not comparison.matches
    assert comparison.recorded == built.result.fingerprint
    assert comparison.recorded != comparison.expected
    assert "rebuild" in comparison.message


def test_physical_sql_takes_part_in_the_fingerprint_and_is_applied(tmp_path: Path) -> None:
    decl = full_declaration()
    tree = tmp_path
    (tree / "sql").mkdir()
    (tree / "sql" / "physical.sql").write_text(
        'CREATE INDEX species_tag_idx ON "tk"."species" ("tag");\n'
    )
    with TestDatabase() as database:
        result = build_database(database.url, decl, tree=tree)
        assert result.fingerprint == declaration_fingerprint(
            decl, (tree / "sql" / "physical.sql").read_bytes()
        )
        with psycopg.connect(database.url) as conn:
            assert conn.execute(
                "SELECT count(*) FROM pg_indexes WHERE indexname = 'species_tag_idx'"
            ).fetchone() == (1,)
        assert compare_fingerprint(database.url, decl, tree=tree).matches
        assert not compare_fingerprint(
            database.url, decl, tree=NO_PHYSICAL
        ).matches  # without physical.sql


def test_unbuilt_database_records_no_fingerprint() -> None:
    with TestDatabase() as database:
        with psycopg.connect(database.url, autocommit=True) as conn:
            conn.execute("CREATE SCHEMA tk")
        comparison = compare_fingerprint(database.url, full_declaration(), tree=NO_PHYSICAL)
        assert comparison.recorded is None and not comparison.matches
        assert "no schema fingerprint" in comparison.message


# -- building over an existing schema -----------------------------------------------------


def test_a_build_replaces_the_schemas_of_an_earlier_one(tmp_path: Path) -> None:
    with TestDatabase() as database:
        first = build_database(database.url, empty_declaration(tmp_path), tree=NO_PHYSICAL)
        assert compare_fingerprint(
            database.url, empty_declaration(tmp_path / "again"), tree=NO_PHYSICAL
        ).matches
        second = build_database(database.url, full_declaration(), tree=NO_PHYSICAL)
        assert first.fingerprint != second.fingerprint
        assert compare_fingerprint(database.url, full_declaration(), tree=NO_PHYSICAL).matches
        with psycopg.connect(database.url) as conn:
            assert conn.execute("SELECT count(*) FROM tk.species").fetchone() == (1,)
            assert conn.execute("SELECT count(*) FROM meta.build_source").fetchone() == (0,)


def test_a_failed_build_leaves_an_earlier_one_as_it_was(tmp_path: Path) -> None:
    with TestDatabase() as database:
        build_database(database.url, full_declaration(), tree=NO_PHYSICAL)
        (tmp_path / "sql").mkdir()
        (tmp_path / "sql" / "physical.sql").write_text("THIS IS NOT SQL;\n")
        with pytest.raises(DatabaseRefusedError, match="syntax"):
            build_database(database.url, full_declaration(), tree=tmp_path)
        assert compare_fingerprint(database.url, full_declaration(), tree=NO_PHYSICAL).matches
        with psycopg.connect(database.url) as conn:
            assert conn.execute("SELECT count(*) FROM tk.species").fetchone() == (1,)


def test_a_failed_first_build_leaves_no_generated_schema(tmp_path: Path) -> None:
    (tmp_path / "sql").mkdir()
    (tmp_path / "sql" / "physical.sql").write_text("THIS IS NOT SQL;\n")
    with TestDatabase() as database:
        with pytest.raises(DatabaseRefusedError, match="syntax"):
            build_database(database.url, full_declaration(), tree=tmp_path)
        with psycopg.connect(database.url) as conn:
            assert conn.execute(
                "SELECT count(*) FROM pg_namespace WHERE nspname = ANY(%s)",
                (list(GENERATED_SCHEMAS),),
            ).fetchone() == (0,)


# -- catalogue of the built schema --------------------------------------------------------


def test_every_table_and_column_has_a_comment(conn: psycopg.Connection) -> None:
    missing = conn.execute(
        """
        SELECT n.nspname || '.' || c.relname
        FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = ANY(%s) AND c.relkind = 'r'
          AND obj_description(c.oid, 'pg_class') IS NULL
        UNION ALL
        SELECT n.nspname || '.' || c.relname || '.' || a.attname
        FROM pg_attribute a JOIN pg_class c ON c.oid = a.attrelid
          JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = ANY(%s) AND c.relkind = 'r' AND a.attnum > 0 AND NOT a.attisdropped
          AND col_description(c.oid, a.attnum) IS NULL
        """,
        (list(GENERATED_SCHEMAS), list(GENERATED_SCHEMAS)),
    ).fetchall()
    assert missing == []


def test_constraints_are_named_deterministically_and_foreign_keys_are_deferrable(
    conn: psycopg.Connection,
) -> None:
    rows = conn.execute(
        """
        SELECT k.conname, k.contype, k.condeferrable, k.condeferred
        FROM pg_constraint k JOIN pg_namespace n ON n.oid = k.connamespace
        WHERE n.nspname = ANY(%s) AND k.contype IN ('p', 'u', 'f', 'c', 'x')
        """,
        (list(GENERATED_SCHEMAS),),
    ).fetchall()
    assert rows
    # PostgreSQL's own names (`t_pkey`, `t_col_fkey`, `t_col_check`) never contain `__`
    assert [name for name, *_ in rows if "__" not in name] == []
    assert all(deferrable and deferred for _, kind, deferrable, deferred in rows if kind == "f")


def test_no_index_exists_except_those_constraints_imply(conn: psycopg.Connection) -> None:
    stray = conn.execute(
        """
        SELECT c.relname
        FROM pg_index i JOIN pg_class c ON c.oid = i.indexrelid
          JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = ANY(%s)
          AND NOT EXISTS (SELECT 1 FROM pg_constraint k WHERE k.conindid = i.indexrelid)
        """,
        (list(GENERATED_SCHEMAS),),
    ).fetchall()
    assert stray == []


# -- constraints behave -------------------------------------------------------------------


def insert_kij(connection: psycopg.Connection, i: uuid.UUID, j: uuid.UUID) -> None:
    connection.execute(
        "INSERT INTO param.kij__pair (id, i, j, k) VALUES (%s, %s, %s, 0.1)", (new_id(), i, j)
    )


def test_symmetric_pair_in_the_non_canonical_orientation_is_rejected(
    conn: psycopg.Connection,
) -> None:
    low, high = sorted([new_id(), new_id()])
    statement = "INSERT INTO param.kij__pair (id, i, j, k) VALUES (%s, %s, %s, 0.1)"
    rejected(
        conn, errors.CheckViolation, statement, (new_id(), high, low), "kij__pair__ck__canonical"
    )
    accepted(conn, statement, (new_id(), low, high))


def test_forbidden_diagonal_is_rejected_and_allowed_diagonal_is_not(
    conn: psycopg.Connection,
) -> None:
    same = new_id()
    rejected(
        conn,
        errors.CheckViolation,
        "INSERT INTO param.kij__pair (id, i, j, k) VALUES (%s, %s, %s, 0.1)",
        (new_id(), same, same),
        "kij__pair__ck__diagonal",
    )
    accepted(
        conn,
        "INSERT INTO tk.triple (id, a, b, c, value, arrangement) VALUES (%s, %s, %s, %s, 1, 0)",
        (new_id(), same, same, same),
    )


def test_reciprocal_and_parity_relations_store_one_orientation(conn: psycopg.Connection) -> None:
    low, high = sorted([new_id(), new_id()])
    rejected(
        conn,
        errors.CheckViolation,
        "INSERT INTO param.ratio__pair (id, i, j, r, arrangement) VALUES (%s, %s, %s, 2, 0)",
        (new_id(), high, low),
        "ratio__pair__ck__canonical",
    )
    rejected(
        conn,
        errors.CheckViolation,
        "INSERT INTO param.ratio__pair (id, i, j, r, arrangement) VALUES (%s, %s, %s, 2, 2)",
        (new_id(), low, high),
        "ratio__pair__ck__arrangement_range",
    )
    accepted(
        conn,
        "INSERT INTO param.ratio__pair (id, i, j, r, arrangement) VALUES (%s, %s, %s, 2, 1)",
        (new_id(), low, high),
    )
    rejected(
        conn,
        errors.CheckViolation,
        'INSERT INTO tk.signed_interaction (id, i, j, "order", value, arrangement) '
        "VALUES (%s, %s, %s, 1, 1, 0)",
        (new_id(), high, low),
        "signed_interaction__ck__canonical",
    )
    rejected(
        conn,
        errors.CheckViolation,
        'INSERT INTO tk.signed_interaction (id, i, j, "order", value, arrangement) '
        "VALUES (%s, %s, %s, 1, 1, -1)",
        (new_id(), low, high),
        "signed_interaction__ck__arrangement_range",
    )
    accepted(
        conn,
        'INSERT INTO tk.signed_interaction (id, i, j, "order", value, arrangement) '
        "VALUES (%s, %s, %s, 1, 1, 1)",
        (new_id(), low, high),
    )
    rejected(
        conn,
        errors.CheckViolation,
        "INSERT INTO tk.interaction (id, i, j, value) VALUES (%s, %s, %s, 1)",
        (new_id(), high, low),
        "interaction__ck__canonical",
    )


def test_permutation_group_keeps_the_canonical_representative(conn: psycopg.Connection) -> None:
    a, b, c = sorted([new_id(), new_id(), new_id()])
    statement = "INSERT INTO tk.triple (id, a, b, c, value, arrangement) VALUES (%s, %s, %s, %s, 1, %s)"
    accepted(conn, statement, (new_id(), a, b, c, 0))
    for arrangement in ((b, a, c), (a, c, b), (c, b, a), (b, c, a), (c, a, b)):
        rejected(
            conn,
            errors.CheckViolation,
            statement,
            (new_id(), *arrangement, 0),
            "triple__ck__canonical",
        )
    # the group the two declared generators make has six arrangements: numbers 0 to 5
    x, y, z = sorted([new_id(), new_id(), new_id()])
    accepted(conn, statement, (new_id(), x, y, z, 5))
    rejected(
        conn,
        errors.CheckViolation,
        statement,
        (new_id(), a, b, c, 6),
        "triple__ck__arrangement_range",
    )


STATE_INSERT = (
    'INSERT INTO param.critical__global (id, "T_c", "T_c__state", "T_c__redirect", kind, '
    "gas_table, aggregation) VALUES (%s, %s, %s::meta.value_state, %s, 'gas', %s, %s)"
)


def test_stateful_slot_requires_a_value_exactly_when_known(conn: psycopg.Connection) -> None:
    def row(value: float | None, state: str, redirect: uuid.UUID | None = None, id_=None):  # noqa: ANN202
        return (id_ or new_id(), value, state, redirect, new_id(), new_id())

    check = "critical__global__ck__T_c__state"
    rejected(conn, errors.CheckViolation, STATE_INSERT, row(None, "known"), check)
    rejected(conn, errors.CheckViolation, STATE_INSERT, row(300.0, "not_applicable"), check)
    rejected(conn, errors.CheckViolation, STATE_INSERT, row(300.0, "withheld"), check)
    rejected(conn, errors.CheckViolation, STATE_INSERT, row(None, "redirect"), check)
    rejected(conn, errors.CheckViolation, STATE_INSERT, row(300.0, "redirect", new_id()), check)
    rejected(
        conn, errors.CheckViolation, STATE_INSERT, row(None, "not_applicable", new_id()), check
    )
    accepted(conn, STATE_INSERT, row(300.0, "known"))
    accepted(conn, STATE_INSERT, row(None, "not_applicable"))
    accepted(conn, STATE_INSERT, row(None, "withheld"))
    accepted(conn, STATE_INSERT, row(None, "redirect", new_id()))


def test_a_redirect_may_not_point_at_its_own_row(conn: psycopg.Connection) -> None:
    own = new_id()
    rejected(
        conn,
        errors.CheckViolation,
        STATE_INSERT,
        (own, None, "redirect", own, new_id(), new_id()),
        "critical__global__ck__T_c__redirect",
    )


def test_a_slot_group_row_is_tied_to_its_slot_group(conn: psycopg.Connection) -> None:
    rejected(
        conn,
        errors.CheckViolation,
        "INSERT INTO param.kij__pair (id, slot_group, i, j, k) VALUES (%s, %s, %s, %s, 1)",
        (new_id(), new_id(), *sorted([new_id(), new_id()])),
        "kij__pair__ck__slot_group",
    )


PIECE = 'INSERT INTO param.nasa7__pure__piece (set_id, n, "T_low", "T_high", a1) VALUES (%s, %s, %s, %s, 0)'


def test_overlapping_intervals_in_one_set_are_rejected(conn: psycopg.Connection) -> None:
    owner = new_id()
    accepted(conn, PIECE, (owner, 1, 200.0, 1000.0))
    rejected(
        conn,
        errors.ExclusionViolation,
        PIECE,
        (owner, 2, 900.0, 1100.0),
        "nasa7__pure__piece__xc__interval",
    )
    # pieces that meet at a boundary do not overlap; another set is independent
    accepted(conn, PIECE, (owner, 2, 1000.0, 6000.0))
    accepted(conn, PIECE, (new_id(), 1, 200.0, 1000.0))


def test_interval_bounds_and_indices_are_checked(conn: psycopg.Connection) -> None:
    owner = new_id()
    # inverted bounds cannot even form the interval column; equal bounds form an empty one
    rejected(conn, errors.DataException, PIECE, (owner, 1, 1000.0, 200.0))
    rejected(
        conn,
        errors.CheckViolation,
        PIECE,
        (owner, 1, 500.0, 500.0),
        "nasa7__pure__piece__ck__interval",
    )
    rejected(
        conn,
        errors.CheckViolation,
        PIECE,
        (owner, 0, 200.0, 300.0),
        "nasa7__pure__piece__ck__n__minimum",
    )
    # an absolute temperature is not negative, and 0 K is a valid lower bound
    rejected(conn, errors.CheckViolation, PIECE, (owner, 1, -1.0, 300.0))
    accepted(conn, PIECE, (owner, 1, 0.0, 300.0))


def test_row_without_its_record_is_rejected_at_commit(built: Built) -> None:
    identifier = new_id()
    label_insert = "INSERT INTO tk.parameterization (id, label) VALUES (%s, %s)"
    with psycopg.connect(built.url) as connection:
        connection.execute(label_insert, (identifier, "orphan"))  # deferred: accepted here
        with pytest.raises(errors.ForeignKeyViolation) as raised:
            connection.commit()
        assert raised.value.diag.constraint_name == "parameterization__fk__id"
    with psycopg.connect(built.url) as connection:
        connection.execute(
            "INSERT INTO prov.record (id, kind) VALUES (%s, 'parameterization')", (identifier,)
        )
        connection.execute(label_insert, (identifier, "registered"))
        connection.commit()  # with its record, the row commits
        connection.execute("DELETE FROM tk.parameterization WHERE id = %s", (identifier,))
        connection.execute("DELETE FROM prov.record WHERE id = %s", (identifier,))
        connection.commit()


def test_relations_with_own_provenance_register_their_rows(conn: psycopg.Connection) -> None:
    (constraint,) = conn.execute(
        "SELECT conname FROM pg_constraint WHERE conrelid = 'tk.interaction'::regclass "
        "AND contype = 'f' AND confrelid = 'prov.record'::regclass"
    ).fetchone()  # type: ignore[misc]
    assert constraint == "interaction__fk__id"
    assert conn.execute(
        "SELECT count(*) FROM pg_constraint WHERE conrelid = 'tk.formula'::regclass "
        "AND confrelid = 'prov.record'::regclass"
    ).fetchone() == (0,)  # inherit:j shares the species' record
    assert conn.execute(
        "SELECT count(*) FROM pg_constraint WHERE conrelid = 'tk.class_weight'::regclass "
        "AND confrelid = 'prov.record'::regclass"
    ).fetchone() == (0,)  # provenance none


def test_a_ddl_check_rejects_a_violating_row(conn: psycopg.Connection) -> None:
    species = "INSERT INTO tk.species (id, tag, smiles, t_min, t_max) VALUES (%s, %s, 'C', %s, %s)"
    rejected(
        conn, errors.CheckViolation, species, (new_id(), "t", 400.0, 300.0), "species__ck__t_window"
    )
    accepted(conn, species, (new_id(), "t1", 300.0, 400.0))
    accepted(conn, species, (new_id(), "t2", 300.0, 300.0))
    rejected(
        conn,
        errors.CheckViolation,
        species,
        (new_id(), "", 300.0, 400.0),
        "species__ck__tag_present",
    )


def test_other_ddl_checks(conn: psycopg.Connection) -> None:
    rejected(
        conn,
        errors.CheckViolation,
        "INSERT INTO tk.species (id, tag) VALUES (%s, 't')",
        (new_id(),),
        "species__ck__one_structure",
    )
    rejected(
        conn,
        errors.CheckViolation,
        "INSERT INTO tk.species (id, tag, smiles, inchikey) VALUES (%s, 't', 'C', 'AAAAAAAAAAAAAA-BBBBBBBBBB-C')",
        (new_id(),),
        "species__ck__one_structure",
    )
    rejected(
        conn,
        errors.CheckViolation,
        "INSERT INTO tk.species (id, tag, smiles, weight) VALUES (%s, 't', 'C', -1)",
        (new_id(),),
        "species__ck__weight_nonnegative",
    )
    rejected(
        conn,
        errors.CheckViolation,
        "INSERT INTO tk.species_form (id, species, label, positive_marker) VALUES (%s, %s, 'x', 0)",
        (new_id(), new_id()),
        "species_form__ck__marker_positive",
    )


def test_domains_refuse_non_finite_and_negative_values_and_accept_zero(
    conn: psycopg.Connection,
) -> None:
    accepted(
        conn, "INSERT INTO tk.species (id, tag, smiles, t_min) VALUES (%s, 'z', 'C', 0)", (new_id(),)
    )
    for bad in ("'NaN'", "'Infinity'", "-5"):
        rejected(
            conn,
            errors.CheckViolation,
            f"INSERT INTO tk.species (id, tag, smiles, t_min) VALUES (%s, 't', 'C', {bad})",  # noqa: S608
            (new_id(),),
        )
    rejected(
        conn,
        errors.CheckViolation,
        "INSERT INTO tk.species (id, tag, smiles, cp_curvature) VALUES (%s, 't', 'C', 'NaN')",
        (new_id(),),
    )
    rejected(
        conn,
        errors.CheckViolation,
        "INSERT INTO tk.species (id, tag, smiles, uncertainty) VALUES (%s, 't', 'C', 'Infinity')",
        (new_id(),),
    )
    accepted(
        conn,
        "INSERT INTO tk.species (id, tag, smiles, uncertainty, cp_curvature) "
        "VALUES (%s, 't', 'C', -1.5, -2.5)",
        (new_id(),),
    )


def test_range_and_array_columns(conn: psycopg.Connection) -> None:
    base = "INSERT INTO tk.species (id, tag, smiles, {column}) VALUES (%s, %s, 'C', {value})"
    accepted(conn, base.format(column="validity", value="'[300,400]'"), (new_id(), "r1"))
    for bad in ("'(300,400]'", "'[300,400)'", "'[300,)'", "'empty'", "'[-1,400]'"):
        rejected(
            conn, errors.CheckViolation, base.format(column="validity", value=bad), (new_id(), "r2")
        )
    # an absolute range may start at zero
    accepted(conn, base.format(column="validity", value="'[0,400]'"), (new_id(), "r1z"))
    accepted(
        conn,
        base.format(column="reference_temperatures", value="ARRAY[300, 350]"),
        (new_id(), "r3"),
    )
    rejected(
        conn,
        errors.CheckViolation,
        base.format(column="reference_temperatures", value="ARRAY[300, NULL]::double precision[]"),
        (new_id(), "r4"),
        "species__ck__reference_temperatures__no_null",
    )


def test_unique_and_identity_constraints(conn: psycopg.Connection) -> None:
    accepted(conn, "INSERT INTO tk.species (id, tag, smiles) VALUES (%s, 'u', 'C')", (new_id(),))
    rejected(
        conn,
        errors.UniqueViolation,
        "INSERT INTO tk.species (id, tag, smiles) VALUES (%s, 'u', 'CC')",
        (new_id(),),
        "species__uq__tag_charge",
    )
    accepted(
        conn,
        "INSERT INTO tk.species (id, tag, smiles, inchikey) VALUES (%s, 'v', NULL, 'KEY-1')",
        (new_id(),),
    )
    rejected(
        conn,
        errors.UniqueViolation,
        "INSERT INTO tk.species (id, tag, smiles, inchikey) VALUES (%s, 'w', NULL, 'KEY-1')",
        (new_id(),),
        "species__uq__inchikey",
    )
    accepted(
        conn, "INSERT INTO tk.material_entity (id, canonical_key) VALUES (%s, 'k')", (new_id(),)
    )
    rejected(
        conn,
        errors.UniqueViolation,
        "INSERT INTO tk.material_entity (id, canonical_key) VALUES (%s, 'k')",
        (new_id(),),
        "material_entity__identity",
    )
    rejected(
        conn,
        errors.CheckViolation,
        "INSERT INTO tk.species (id, tag, smiles, inchikey) VALUES (%s, 'x', NULL, '')",
        (new_id(),),
    )


def test_relation_keys_of_each_type(conn: psycopg.Connection) -> None:
    accepted(
        conn, "INSERT INTO tk.class_weight (id, class, value) VALUES (%s, 'gas', 1)", (new_id(),)
    )
    rejected(
        conn,
        errors.UniqueViolation,
        "INSERT INTO tk.class_weight (id, class, value) VALUES (%s, 'gas', 2)",
        (new_id(),),
        "class_weight__pk",
    )
    rejected(
        conn,
        errors.InvalidTextRepresentation,
        "INSERT INTO tk.class_weight (id, class, value) VALUES (%s, 'plasma', 2)",
        (new_id(),),
    )
    accepted(conn, "INSERT INTO tk.rank_weight (id, rank, value) VALUES (%s, 3, 1)", (new_id(),))
    rejected(
        conn,
        errors.UniqueViolation,
        "INSERT INTO tk.rank_weight (id, rank, value) VALUES (%s, 3, 2)",
        (new_id(),),
        "rank_weight__pk",
    )
    accepted(
        conn,
        "INSERT INTO tk.alias (id, key, alias) VALUES (%s, 'KEY-1', 'water')",
        (new_id(),),
    )
    accepted(
        conn,
        "INSERT INTO tk.measurement (id, record, quantity, index, value) VALUES (%s, %s, %s, 1, 3.5)",
        (new_id(), new_id(), identity.meta_identifier("quantity_type", "Pressure")),
    )
    rejected(
        conn,
        errors.CheckViolation,
        "INSERT INTO tk.measurement (id, record, quantity, index, value) VALUES (%s, %s, %s, 2, 'NaN')",
        (new_id(), new_id(), identity.meta_identifier("quantity_type", "Pressure")),
    )
    assert conn.execute(
        "SELECT flag, note FROM tk.measurement ORDER BY index LIMIT 1"
    ).fetchone() == (False, None)


def test_references_are_foreign_keys_checked_on_demand(conn: psycopg.Connection) -> None:
    conn.execute("SET CONSTRAINTS ALL IMMEDIATE")
    rejected(
        conn,
        errors.ForeignKeyViolation,
        "INSERT INTO tk.species_form (id, species, label) VALUES (%s, %s, 'x')",
        (new_id(), new_id()),
    )
    rejected(
        conn,
        errors.ForeignKeyViolation,
        "INSERT INTO tk.species (id, tag, smiles, unit_of) VALUES (%s, 't', 'C', %s)",
        (new_id(), new_id()),
    )
    rejected(
        conn,
        errors.ForeignKeyViolation,
        "INSERT INTO tk.measurement (id, record, quantity, index, value) VALUES (%s, %s, %s, 1, 1)",
        (new_id(), new_id(), new_id()),
    )


def test_a_complete_parameter_set_loads_and_commits(built: Built) -> None:
    """One chain through every layer: records, a root and a refinement, a parameter set tied
    to its slot group, and its slot-group row; a wrong slot group is refused at commit."""
    ids = {name: new_id() for name in ("entity", "form", "param", "set", "param2")}
    slot_group = identity.meta_identifier("slot_group", "antoine.pure")

    def load(connection: psycopg.Connection, group: uuid.UUID, row_group: uuid.UUID) -> None:
        for kind, key in (
            ("material_entity", ids["entity"]),
            ("species_form", ids["form"]),
            ("parameterization", ids["param"]),
            ("parameter_set", ids["set"]),
        ):
            connection.execute("INSERT INTO prov.record (id, kind) VALUES (%s, %s)", (key, kind))
        connection.execute(
            "INSERT INTO tk.material_entity (id, canonical_key) VALUES (%s, %s)",
            (ids["entity"], f"chain-{ids['entity']}"),
        )
        connection.execute(
            "INSERT INTO tk.species (id, tag, smiles) VALUES (%s, %s, 'C')",
            (ids["entity"], f"tag-{ids['entity']}"),
        )
        connection.execute(
            "INSERT INTO tk.species_form (id, species, label) VALUES (%s, %s, 'gas')",
            (ids["form"], ids["entity"]),
        )
        connection.execute(
            "INSERT INTO tk.parameterization (id, label) VALUES (%s, 'p')", (ids["param"],)
        )
        connection.execute(
            "INSERT INTO tk.parameter_set (id, parameterization, slot_group, subject_key) "
            "VALUES (%s, %s, %s, 'k')",
            (ids["set"], ids["param"], group),
        )
        connection.execute(
            'INSERT INTO param.antoine__pure (id, slot_group, i, "A", "B", "C") '
            "VALUES (%s, %s, %s, 1, 2, 3)",
            (ids["set"], row_group, ids["form"]),
        )

    with psycopg.connect(built.url) as connection:
        load(connection, slot_group, slot_group)
        connection.commit()
        assert connection.execute(
            "SELECT count(*) FROM param.antoine__pure a JOIN tk.parameter_set p USING (id)"
        ).fetchone() == (1,)
        for table in (
            "param.antoine__pure",
            "tk.parameter_set",
            "tk.parameterization",
            "tk.species_form",
            "tk.species",
            "tk.material_entity",
        ):
            connection.execute(
                f"DELETE FROM {table} WHERE id IN (%s, %s, %s, %s)",  # noqa: S608
                (ids["set"], ids["param"], ids["form"], ids["entity"]),
            )
        connection.execute(
            "DELETE FROM prov.record WHERE id IN (%s, %s, %s, %s)",
            (ids["set"], ids["param"], ids["form"], ids["entity"]),
        )
        connection.commit()

    # a parameter set of another slot group cannot hold an antoine row
    other_group = identity.meta_identifier("slot_group", "soave.pure")
    with psycopg.connect(built.url) as connection:
        load(connection, other_group, slot_group)
        with pytest.raises(errors.ForeignKeyViolation) as raised:
            connection.commit()
        assert raised.value.diag.constraint_name == "antoine__pure__fk__id"


# -- scalar types and Meta references -----------------------------------------------------


def test_scalar_types_project_to_their_postgresql_types(conn: psycopg.Connection) -> None:
    columns = dict(
        conn.execute(
            """
            SELECT a.attname, format_type(a.atttypid, a.atttypmod)
            FROM pg_attribute a WHERE a.attrelid = 'tk.species'::regclass AND a.attnum > 0
            """
        ).fetchall()
    )
    assert columns["curated"] == "boolean"
    assert columns["curated_on"] == "date"
    assert columns["curated_at"] == "timestamp with time zone"
    assert columns["checksum"] == "meta.hash"
    assert columns["note"] == "text"
    assert columns["tag"] == "text"
    assert columns["unit_of"] == "uuid"
    assert columns["inchikey"] == "meta.id_inchikey"
    assert columns["charge"] == "meta.charge_number"
    assert columns["uncertainty"] == "meta.finite_real"
    assert columns["cp_curvature"] == "meta.finite_real"
    assert columns["phase"] == "meta.phase_class"
    assert "elements" not in columns  # Set<K> is a child table, not a column


def test_hash_domain_requires_32_bytes_and_booleans_default(conn: psycopg.Connection) -> None:
    insert = "INSERT INTO tk.species (id, tag, smiles, checksum) VALUES (%s, %s, 'C', %s)"
    accepted(conn, insert, (new_id(), "h1", bytes(32)))
    rejected(conn, errors.CheckViolation, insert, (new_id(), "h2", bytes(5)))
    accepted(conn, "INSERT INTO tk.species (id, tag, smiles) VALUES (%s, 'h3', 'C')", (new_id(),))
    assert conn.execute("SELECT curated FROM tk.species WHERE tag = 'h3'").fetchone() == (False,)
    accepted(
        conn,
        "INSERT INTO tk.species (id, tag, smiles, curated_on, curated_at, note) "
        "VALUES (%s, 'h4', 'C', '2026-09-30', '2026-09-30T12:00:00Z', 'as published')",
        (new_id(),),
    )


def test_declared_entity_with_every_scalar_type(conn: psycopg.Connection) -> None:
    row = conn.execute(
        "SELECT released_on, built_at, checksum, stable, preferred_phase::text, key, scale, "
        "scale_units, remark FROM tk.release WHERE name = 'r1'"
    ).fetchone()
    assert row is not None
    released_on, built_at, checksum, stable, phase, key, scale, units, remark = row
    assert str(released_on) == "2026-09-30"
    assert built_at == datetime(2026, 9, 30, 12, 0, tzinfo=UTC)  # an instant, whatever the zone
    assert bytes(checksum) == bytes(range(32))
    assert (stable, phase, key, scale, units, remark) == (
        True,
        "liquid",
        "AAAAAAAAAAAAAA-BBBBBBBBBB-C",
        2.0,
        "{1,2.5}",  # an array of a domain comes back as its text form
        "as published",
    )
    assert conn.execute("SELECT id FROM tk.release").fetchone() == (
        identity.identifier("release", ["r1"]),
    )


def test_every_meta_construct_is_a_foreign_key_to_its_meta_table(conn: psycopg.Connection) -> None:
    targets = dict(
        conn.execute(
            """
            SELECT a.attname, k.confrelid::regclass::text
            FROM pg_constraint k
              JOIN pg_attribute a ON a.attrelid = k.conrelid AND a.attnum = k.conkey[1]
            WHERE k.conrelid = 'tk.meta_probe'::regclass AND k.contype = 'f'
            """
        ).fetchall()
    )
    assert targets == {
        "m_quantity_type": "meta.quantity_type",
        "m_kind": "meta.kind",
        "m_contract": "meta.contract",
        "m_form": "meta.form",
        "m_slot_group": "meta.slot_group",
        "m_slot": "meta.slot",
        "m_family": "meta.family",
        "m_subform_slot": "meta.subform_slot",
    }
    conn.execute("SET CONSTRAINTS ALL IMMEDIATE")
    ids = identity.meta_identifier
    accepted(
        conn,
        "INSERT INTO tk.meta_probe (id, label, m_quantity_type, m_kind, m_contract, m_form, "
        "m_slot_group, m_slot, m_family, m_subform_slot) VALUES (%s, 'all', %s, %s, %s, %s, %s, %s, %s, %s)",
        (
            new_id(),
            ids("quantity_type", "Temperature"),
            ids("kind", "species"),
            ids("contract", "heat_capacity"),
            ids("form", "nasa7"),
            ids("slot_group", "nasa7.pure"),
            ids("slot", "nasa7.pure.piece.a1"),
            ids("family", "nasa7.pure.piece"),
            ids("subform_slot", "cubic.alpha"),
        ),
    )
    rejected(
        conn,
        errors.ForeignKeyViolation,
        "INSERT INTO tk.meta_probe (id, label, m_slot) VALUES (%s, 'bad', %s)",
        (new_id(), ids("slot_group", "nasa7.pure")),  # a slot_group id is not a slot id
    )


# -- own-kind entities, observables, Array<Real>, fingerprint of the content ----------------


def test_declared_entities_of_own_kinds_register_their_records(
    built: Built, conn: psycopg.Connection
) -> None:
    water = identity.identifier("material_entity", ["O"])
    fit = identity.identifier("parameterization", ["default"])
    rows = dict(conn.execute("SELECT id, kind FROM prov.record").fetchall())
    assert rows == {water: "species", fit: "parameterization"}
    assert conn.execute("SELECT id FROM tk.species WHERE tag = 'water'").fetchone() == (water,)
    assert conn.execute("SELECT canonical_key FROM tk.material_entity").fetchone() == ("O",)
    assert conn.execute("SELECT count(*) FROM tk.species__elements").fetchone() == (1,)
    # declaration-provenance entities register nothing
    assert conn.execute("SELECT count(*) FROM tk.element").fetchone() == (2,)


def test_observables_are_referenced_by_the_entitys_identifier(conn: psycopg.Connection) -> None:
    vapor_pressure = identity.identifier("observable", ["vapor_pressure"])
    critical = identity.identifier("observable", ["critical_temperature"])
    assert conn.execute(
        "SELECT observable FROM meta.contract_output WHERE contract = 'pure_vapor_pressure'"
    ).fetchone() == (vapor_pressure,)
    assert conn.execute(
        "SELECT observable FROM meta.slot WHERE qualified_name = 'critical.global.T_c'"
    ).fetchone() == (critical,)
    assert (
        conn.execute("SELECT count(*) FROM meta.slot WHERE observable IS NULL").fetchone()
        == conn.execute(
            "SELECT count(*) FROM meta.slot WHERE qualified_name <> 'critical.global.T_c'"
        ).fetchone()
    )
    targets = conn.execute(
        """
        SELECT k.conrelid::regclass::text, k.confrelid::regclass::text
        FROM pg_constraint k WHERE k.conname IN
          ('contract_output__fk__observable', 'slot__fk__observable') ORDER BY 1
        """
    ).fetchall()
    assert targets == [
        ("meta.contract_output", "tk.observable"),
        ("meta.slot", "tk.observable"),
    ]
    conn.execute("SET CONSTRAINTS ALL IMMEDIATE")
    rejected(
        conn,
        errors.ForeignKeyViolation,
        "UPDATE meta.slot SET observable = %s WHERE qualified_name = 'critical.global.T_c'",
        (new_id(),),
    )


def test_the_observable_role_is_reified(conn: psycopg.Connection) -> None:
    assert conn.execute(
        "SELECT kind FROM meta.framework_role WHERE role = 'observable'"
    ).fetchone() == ("observable",)


def test_a_linear_transposition_is_reified_with_its_slots_and_matrix(
    conn: psycopg.Connection,
) -> None:
    assert conn.execute(
        "SELECT rule, diagonal, by_index FROM meta.transposition "
        "WHERE owner_type = 'slot_group' AND owner = 'margules.pair'"
    ).fetchone() == ("linear", "forbidden", None)
    assert conn.execute(
        "SELECT position, slot FROM meta.transposition_slot WHERE owner = 'margules.pair' ORDER BY position"
    ).fetchall() == [(1, "h0"), (2, "h1")]
    assert conn.execute(
        "SELECT row_position, column_position, coefficient FROM meta.transposition_matrix "
        "WHERE owner = 'margules.pair' ORDER BY row_position, column_position"
    ).fetchall() == [(1, 1, 1.0), (1, 2, 1.0), (2, 1, 0.0), (2, 2, -1.0)]
    # the slots of a reciprocal rule are reified too, and carry no matrix
    assert conn.execute(
        "SELECT position, slot FROM meta.transposition_slot WHERE owner = 'ratio.pair'"
    ).fetchall() == [(1, "r")]
    assert conn.execute(
        "SELECT count(*) FROM meta.transposition_matrix WHERE owner = 'ratio.pair'"
    ).fetchone() == (0,)


def test_a_linear_group_stores_one_orientation(conn: psycopg.Connection) -> None:
    low, high = sorted([new_id(), new_id()])
    statement = (
        "INSERT INTO param.margules__pair (id, i, j, h0, h1, arrangement) "
        "VALUES (%s, %s, %s, 1, 2, 0)"
    )
    rejected(
        conn,
        errors.CheckViolation,
        statement,
        (new_id(), high, low),
        "margules__pair__ck__canonical",
    )
    accepted(conn, statement, (new_id(), low, high))


def test_array_of_real_projects_to_the_finite_real_domain(conn: psycopg.Connection) -> None:
    assert conn.execute(
        "SELECT format_type(atttypid, atttypmod) FROM pg_attribute "
        "WHERE attrelid = 'tk.release'::regclass AND attname = 'readings'"
    ).fetchone() == ("meta.finite_real[]",)
    assert conn.execute("SELECT readings::text FROM tk.release").fetchone() == ("{0.5,-1}",)
    base = "INSERT INTO tk.species (id, tag, smiles, samples) VALUES (%s, %s, 'C', {value})"
    accepted(conn, base.format(value="ARRAY[-1.5, 2]"), (new_id(), "s1"))
    rejected(
        conn,
        errors.CheckViolation,
        base.format(value="ARRAY[1, 'NaN']::double precision[]"),
        (new_id(), "s2"),
    )
    rejected(
        conn,
        errors.CheckViolation,
        base.format(value="ARRAY[1, NULL]::double precision[]"),
        (new_id(), "s3"),
        "species__ck__samples__no_null",
    )


@pytest.mark.parametrize(
    ("path", "old", "new"),
    [
        ("model/physical.toml", 'doc = "A gas or vapour."', 'doc = "A gas."'),
        (
            "model/identity.toml",
            'doc = "A chemical element: a declared vocabulary."',
            'doc = "An element."',
        ),
        ("model/vocab.toml", 'doc = "Hydrogen."', 'doc = "The lightest element."'),
        ("model/vocab.toml", "atomic_number = 2", "atomic_number = 3"),
    ],
)
def test_the_fingerprint_check_fails_when_only_content_changes(
    built: Built, path: str, old: str, new: str, tmp_path: Path
) -> None:
    tree = copy_full(tmp_path)
    text = (tree / path).read_text()
    assert old in text
    (tree / path).write_text(text.replace(old, new))
    changed = load_declaration(tree / "model", tree / "forms", contract=None).require()
    # the DDL text is what it was unless a doc reaches a comment; the fingerprint must differ anyway
    comparison = compare_fingerprint(built.url, changed, tree=NO_PHYSICAL)
    assert not comparison.matches
    assert comparison.recorded == built.result.fingerprint


def test_a_doc_that_does_not_reach_the_ddl_still_changes_the_fingerprint(
    built: Built, tmp_path: Path
) -> None:
    tree = copy_full(tmp_path)
    path = tree / "model" / "vocab.toml"
    path.write_text(path.read_text().replace('doc = "Hydrogen."', 'doc = "The lightest element."'))
    changed = load_declaration(tree / "model", tree / "forms", contract=None).require()
    assert generate(changed) == generate(built.decl)  # entity docs are not in the DDL
    assert not compare_fingerprint(built.url, changed, tree=NO_PHYSICAL).matches


# -- invariants of relations and the check forms `present_iff` and `within` -----------------------


READING = (
    "INSERT INTO tk.reading (id, record, n, state, value) VALUES (%s, %s, %s, %s::meta.phase_class, %s)"
)


def test_a_relation_carries_its_declared_checks(conn: psycopg.Connection) -> None:
    record = new_id()
    accepted(conn, READING, (new_id(), record, 1, "gas", 0.5))
    accepted(conn, READING, (new_id(), record, 2, "gas", -0.5))  # the bounds are closed
    accepted(conn, READING, (new_id(), record, 3, "gas", 1))
    accepted(conn, READING, (new_id(), record, 4, "liquid", None))
    # present_iff: a value exactly when the state is one of the listed members
    rejected(
        conn,
        errors.CheckViolation,
        READING,
        (new_id(), record, 5, "gas", None),
        "reading__ck__value_only_when_compressible",
    )
    rejected(
        conn,
        errors.CheckViolation,
        READING,
        (new_id(), record, 6, "liquid", 0.5),
        "reading__ck__value_only_when_compressible",
    )
    # within: a closed interval, and a half-open one with one bound omitted
    rejected(
        conn, errors.CheckViolation, READING, (new_id(), record, 7, "gas", 1.5),
        "reading__ck__value_in_range",
    )
    rejected(
        conn, errors.CheckViolation, READING, (new_id(), record, 8, "gas", -1.5),
        "reading__ck__value_in_range",
    )
    rejected(
        conn, errors.CheckViolation, READING, (new_id(), record, 9, "gas", -0.75),
        "reading__ck__value_not_below",
    )


def test_a_kind_carries_the_same_check_forms(conn: psycopg.Connection) -> None:
    insert = "INSERT INTO tk.instrument (id, name, state, \"offset\") VALUES (%s, %s, %s::meta.calibration, %s)"
    accepted(conn, insert, (new_id(), "a", "calibrated", 0.25))
    accepted(conn, insert, (new_id(), "b", "uncalibrated", None))
    rejected(
        conn, errors.CheckViolation, insert, (new_id(), "c", "calibrated", None),
        "instrument__ck__offset_when_calibrated",
    )
    rejected(
        conn, errors.CheckViolation, insert, (new_id(), "d", "uncalibrated", 0.25),
        "instrument__ck__offset_when_calibrated",
    )
    rejected(
        conn, errors.CheckViolation, insert, (new_id(), "e", "calibrated", 2.0),
        "instrument__ck__offset_small",
    )


def test_requirements_of_kinds_and_relations_are_reified_alike(
    built: Built, conn: psycopg.Connection
) -> None:
    rows = conn.execute(
        "SELECT owner_type, owner, name, enforced, check_rule, check_lower, check_upper "
        "FROM meta.requirement WHERE owner IN ('reading', 'instrument', 'species') ORDER BY 1, 2, 3"
    ).fetchall()
    by_key = {(r[0], r[1], r[2]): r[3:] for r in rows}
    assert by_key[("relation", "reading", "value_only_when_compressible")] == (
        "ddl", "present_iff", None, None
    )
    assert by_key[("relation", "reading", "value_in_range")] == ("ddl", "within", -1.0, 1.0)
    assert by_key[("relation", "reading", "value_not_below")] == ("ddl", "within", -0.5, None)
    assert by_key[("relation", "reading", "readings_distinct")] == ("verify", None, None, None)
    assert by_key[("kind", "instrument", "offset_small")] == ("ddl", "within", -1.0, 1.0)
    assert by_key[("kind", "species", "tag_present")][1] == "nonempty"
    columns = conn.execute(
        "SELECT requirement, position, attribute FROM meta.requirement_attribute "
        "WHERE owner_type = 'relation' AND owner = 'reading' ORDER BY requirement, position"
    ).fetchall()
    assert ("value_only_when_compressible", 1, "value") in columns
    assert ("value_only_when_compressible", 2, "state") in columns
    members = conn.execute(
        "SELECT member FROM meta.requirement_member WHERE owner = 'reading' "
        "AND requirement = 'value_only_when_compressible' ORDER BY position"
    ).fetchall()
    assert members == [("gas",)]


def test_enum_facets_are_reified(conn: psycopg.Connection) -> None:
    facets = conn.execute("SELECT enum, name, ordinal FROM meta.enum_facet").fetchall()
    assert facets == [("phase_class", "compressible", 1)]
    assert conn.execute("SELECT enum, member, facet FROM meta.enum_member_facet").fetchall() == [
        ("phase_class", "gas", "compressible")
    ]


def test_a_contract_argument_basis_references_the_declared_entity(
    built: Built, conn: psycopg.Connection
) -> None:
    (basis,) = conn.execute(
        "SELECT basis FROM meta.contract_argument WHERE contract = 'mixture_heat_capacity' "
        "AND name = 'x'"
    ).fetchone()  # type: ignore[misc]
    (name,) = conn.execute("SELECT name FROM tk.composition_basis WHERE id = %s", (basis,)).fetchone()  # type: ignore[misc]
    assert name == "mole_fraction"
    constraint = conn.execute(
        "SELECT count(*) FROM pg_constraint WHERE conname = 'contract_argument__fk__basis'"
    ).fetchone()
    assert constraint == (1,)
