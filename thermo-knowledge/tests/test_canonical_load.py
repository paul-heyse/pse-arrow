# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Canonical Parquet loads through `tk build`: every column type reaches its table and the
commit validates every constraint."""

from __future__ import annotations

import uuid
from datetime import UTC, datetime
from pathlib import Path

import psycopg
from build_support import count, fingerprint, inputs_of, write_source
from mapping_support import SATURATION, extended_declaration, origin, real_declaration

from thermo_knowledge.build import build_database
from thermo_knowledge.canonical.values import Quantity, QuantityArray
from thermo_knowledge.canonical.writer import (
    CanonicalWriter,
    FamilyRow,
    TabulatedAxis,
    TabulatedFunction,
    TabulatedSeries,
)
from thermo_knowledge.declaration import Declaration
from thermo_knowledge.testing import TestDatabase


def emit_saturation(w: CanonicalWriter, fluids: int = 2) -> list[uuid.UUID]:
    param = w.kind(
        "parameterization",
        {"key": "k", "revision": "r", "title": "t", "coherence": "independent_records"},
        origins=[origin("a.json#/0", "fitted")],
    )
    sets = []
    for n in range(fluids):
        sp = w.kind(
            "species",
            {"canonical_key": f"K{n}", "label": f"K{n}"},
            origins=[origin(f"a.json#/{n}")],
        )
        sets.append(
            w.parameter_set(
                parameterization=param,
                slot_group=SATURATION,
                subjects=[sp],
                slots={"T_r": Quantity(400.0 + n, "K"), "p_r": Quantity(1e6, "Pa")},
                families={
                    "term": [FamilyRow({"k": k}, {"n": float(k), "t": 1.0}) for k in (1, 2, 3)]
                },
                origins=[origin(f"a.json#/{n}", "fitted")],
            )
        )
    return sets


def test_canonical_parquet_loads_with_every_constraint_satisfied_at_commit(
    tmp_path: Path,
) -> None:
    sets: list[uuid.UUID] = []
    write_source(tmp_path, "src", lambda w: sets.extend(emit_saturation(w)))
    with TestDatabase() as database:
        result = build_database(database.url, real_declaration(), inputs_of(tmp_path))
        assert result.tables["param.vapor_pressure_exp_series_tau__pure__term"] == 6
        decl = real_declaration()
        declared = sum(1 for e in decl.entities if decl.kinds[e.kind].provenance.mode == "own")
        # the declared entities' records, then two species, a parameterization and two sets
        assert count(database.url, "prov.record") == result.tables["prov.record"] == declared + 5
        with psycopg.connect(database.url) as conn:
            found = conn.execute(
                "SELECT p.id, p.\"T_r\", p.p_r, (SELECT string_agg(DISTINCT o.role::text, ',') "
                "FROM prov.record_origin o WHERE o.record = p.id) "
                'FROM param.vapor_pressure_exp_series_tau__pure p ORDER BY p."T_r"'
            ).fetchall()
            assert [row[0] for row in found] == sets
            assert found[0][1:] == (400.0, 1e6, "fitted")
            (terms,) = conn.execute(
                "SELECT count(*) FROM param.vapor_pressure_exp_series_tau__pure__term"
            ).fetchone()  # type: ignore[misc]
            assert terms == 6
            (origins,) = conn.execute(
                "SELECT count(*) FROM prov.record_origin o JOIN prov.record r ON r.id = o.record"
            ).fetchone()  # type: ignore[misc]
            assert origins == result.tables["prov.record_origin"]
            (hash_length,) = conn.execute(
                "SELECT min(octet_length(tree_hash)) FROM prov.carrier"
            ).fetchone()  # type: ignore[misc]
            assert hash_length == 32


def test_array_timestamp_hash_and_enum_columns_load(tmp_path: Path) -> None:
    decl: Declaration = extended_declaration(tmp_path / "decl")

    def fill(w: CanonicalWriter) -> None:
        species = w.kind(
            "species", {"canonical_key": "K", "label": "K"}, origins=[origin("a.json#/0")]
        )
        param = w.kind(
            "parameterization",
            {"key": "k", "revision": "r", "title": "t", "coherence": "independent_records"},
            origins=[origin("a.json#/0", "fitted")],
        )
        w.parameter_set(
            parameterization=param,
            slot_group="fixture_profile.pure",
            subjects=[species],
            slots={
                "profile": TabulatedFunction(
                    "linear",
                    axes=[TabulatedAxis("Temperature", QuantityArray([1.0, 2.0, 3.0], "K"))],
                    series=[
                        TabulatedSeries("a", "Pressure", QuantityArray([1.0, 2.0, 3.0], "kPa")),
                        TabulatedSeries("b", "Pressure", QuantityArray([4.0, 5.0, 6.0], "kPa")),
                    ],
                )
            },
            origins=[origin("a.json#/1", "fitted")],
        )

    write_source(tmp_path / "canonical", "src", fill, decl=decl, declaration=fingerprint(decl))
    with TestDatabase() as database:
        build_database(database.url, decl, inputs_of(tmp_path / "canonical"))
        with psycopg.connect(database.url) as conn:
            ((points,),) = conn.execute("SELECT points::float8[] FROM tk.tabulated_axis").fetchall()
            assert points == [1.0, 2.0, 3.0]
            found = conn.execute(
                'SELECT name, "values"::float8[] FROM tk.tabulated_series ORDER BY name'
            ).fetchall()
            assert found == [("a", [1000.0, 2000.0, 3000.0]), ("b", [4000.0, 5000.0, 6000.0])]
            ((interpolation,),) = conn.execute(
                "SELECT interpolation FROM tk.tabulated_function"
            ).fetchall()
            assert interpolation == "linear"
            ((retrieved,),) = conn.execute("SELECT retrieved FROM prov.carrier").fetchall()
            assert retrieved.astimezone(UTC) == datetime(2026, 9, 30, 9, 32, 9, tzinfo=UTC)


def test_repeated_assertions_of_one_subject_load_as_occurrences(tmp_path: Path) -> None:
    def fill(w: CanonicalWriter) -> None:
        param = w.kind(
            "parameterization",
            {"key": "k", "revision": "r", "title": "t", "coherence": "independent_records"},
            origins=[origin("a.json#/0", "fitted")],
        )
        sp = w.kind("species", {"canonical_key": "K", "label": "K"}, origins=[origin("a.json#/0")])
        for occurrence, pressure in ((None, 1e6), (2, 2e6), (3, 3e6)):
            w.parameter_set(
                parameterization=param,
                slot_group=SATURATION,
                subjects=[sp],
                slots={"T_r": Quantity(400.0, "K"), "p_r": Quantity(pressure, "Pa")},
                origins=[origin(f"a.json#/{occurrence or 1}", "fitted")],
                occurrence=occurrence,
            )

    write_source(tmp_path, "src", fill)
    with TestDatabase() as database:
        build_database(database.url, real_declaration(), inputs_of(tmp_path))
        with psycopg.connect(database.url) as conn:
            found = conn.execute(
                "SELECT s.occurrence, p.p_r FROM tk.parameter_set s "
                "JOIN param.vapor_pressure_exp_series_tau__pure p ON p.id = s.id ORDER BY s.occurrence"
            ).fetchall()
            assert found == [(1, 1e6), (2, 2e6), (3, 3e6)]


def test_every_transposition_rule_loads_under_the_generated_canonical_checks(
    tmp_path: Path,
) -> None:
    """The writer's canonical orientation is the one the generated DDL checks."""
    decl: Declaration = extended_declaration(tmp_path / "decl")

    def fill(w: CanonicalWriter) -> None:
        keys = "ABC"
        members = sorted(
            w.kind(
                "species", {"canonical_key": key, "label": key}, origins=[origin(f"a.json#/{n}")]
            )
            for n, key in enumerate(keys)
        )
        param = w.kind(
            "parameterization",
            {"key": "k", "revision": "r", "title": "t", "coherence": "independent_records"},
            origins=[origin("a.json#/0", "fitted")],
        )
        for group, slots in (
            ("fixture_symmetric.pair", {"k": 1.0, "T_ref": Quantity(300.0, "K")}),
            ("fixture_reciprocal.pair", {"r": 2.0, "other": 1.0}),
            ("fixture_margules.pair", {"h0": 2.0, "h1": 0.5, "other": 1.0}),
        ):
            w.parameter_set(
                parameterization=param,
                slot_group=group,
                subjects=[members[2], members[0]],
                slots=slots,
                origins=[origin("a.json#/1", "fitted")],
            )
        w.parameter_set(
            parameterization=param,
            slot_group="fixture_parity.pair",
            subjects=[members[2], members[1]],
            slots={},
            families={"term": [FamilyRow({"order": n}, {"c": 1.0}) for n in range(3)]},
            origins=[origin("a.json#/2", "fitted")],
        )
        w.parameter_set(
            parameterization=param,
            slot_group="fixture_group.triple",
            subjects=[members[2], members[0], members[1]],
            slots={"v": 1.0},
            origins=[origin("a.json#/3", "fitted")],
        )
        w.parameter_set(
            parameterization=param,
            slot_group="fixture_plain.pure",
            subjects=[members[0]],
            slots={"T_c": Quantity(300.0, "K")},
            families={
                "piece": [
                    FamilyRow(
                        {"n": 1}, {"T_low": Quantity(200.0, "K"), "T_high": Quantity(400.0, "K")}
                    )
                ]
            },
            origins=[origin("a.json#/4", "fitted")],
        )

    write_source(tmp_path / "canonical", "src", fill, decl=decl, declaration=fingerprint(decl))
    with TestDatabase() as database:
        counts = build_database(database.url, decl, inputs_of(tmp_path / "canonical")).tables
        assert (
            counts["param.fixture_symmetric__pair"] == 1
            and counts["param.fixture_group__triple"] == 1
        )
        assert counts["param.fixture_plain__pure__piece"] == 1
        assert count(database.url, "param.fixture_reciprocal__pair") == 1
        assert count(database.url, "param.fixture_margules__pair") == 1
        with psycopg.connect(database.url) as conn:
            # written as (high, low): the stored row is the swapped pair, (h0 + h1, -h1)
            assert conn.execute("SELECT h0, h1 FROM param.fixture_margules__pair").fetchall() == [
                (2.5, -0.5)
            ]


def test_the_subject_keys_of_top_level_and_nested_sets_verify_and_a_changed_key_is_found(
    tmp_path: Path,
) -> None:
    """A set held by a slot outside any family and sets held by the rows of a family are all
    keyed by their parent, slot and index; the verify check recomputes each key from the rows."""
    from thermo_knowledge import config
    from thermo_knowledge.canonical.writer import NestedSet
    from thermo_knowledge.verify.checks import VERIFY_DIR, load_checks
    from thermo_knowledge.verify.run import run_check

    check = {c.target: c for c in load_checks(config.TREE_DIR / VERIFY_DIR)[0]}[
        "subject_key_matches_subjects"
    ]
    decl = extended_declaration(tmp_path / "decl")
    pair, linear = "fixture_nested_pair.pair", "fixture_linear.global"

    def fill(w: CanonicalWriter) -> None:
        param = w.kind(
            "parameterization",
            {"key": "k", "revision": "r", "title": "t", "coherence": "independent_records"},
            origins=[origin("a.json#/0", "fitted")],
        )
        first, second = (
            w.kind(
                "species",
                {"canonical_key": key, "label": key},
                origins=[origin(f"a.json#/{position}")],
            )
            for position, key in ((1, "A"), (2, "B"))
        )
        low, high = sorted((first, second), key=str)
        w.parameter_set(
            parameterization=param,
            slot_group=pair,
            subjects=[low, high],
            slots={
                "k": 1.0,
                "f": NestedSet("fixture_constant.global", {"c": 5.0}),
            },
            families={
                "term": [
                    FamilyRow(
                        {"order": n},
                        {
                            "c": 0.5,
                            "g": NestedSet(
                                linear, {"a": float(n), "b": 1.0, "T_ref": Quantity(300.0, "K")}
                            ),
                        },
                    )
                    for n in (0, 1)
                ]
            },
            origins=[origin("a.json#/3", "fitted")],
        )

    write_source(tmp_path / "canonical", "src", fill, decl=decl, declaration=fingerprint(decl))
    with TestDatabase() as database:
        build_database(database.url, decl, inputs_of(tmp_path / "canonical"))
        with psycopg.connect(database.url) as conn:
            (sets,) = conn.execute("SELECT count(*) FROM tk.parameter_set").fetchone()  # type: ignore[misc]
            (nested,) = conn.execute(
                "SELECT count(*) FROM tk.parameter_set WHERE parent IS NOT NULL"
            ).fetchone()  # type: ignore[misc]
            assert (sets, nested) == (4, 3)  # the pair, its constant and its two family values
            result = run_check(conn, check)
            assert result.error is None and result.violations == 0, result.rows
            conn.execute(
                "UPDATE tk.parameter_set SET subject_key = 'changed ' || id::text WHERE id IN ("
                "SELECT id FROM tk.parameter_set WHERE parent IS NOT NULL "
                "ORDER BY subject_key LIMIT 2)"
            )
            conn.execute(
                "UPDATE tk.parameter_set SET subject_key = 'top' WHERE parent IS NULL"
            )
            broken = run_check(conn, check)
            assert broken.error is None and broken.violations == 3
            conn.rollback()
