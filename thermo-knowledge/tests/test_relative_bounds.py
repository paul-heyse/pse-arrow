# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""A validity bound stated relative to another observable of the subject (plan 24, packet TK2g,
alignment item 19): the clause stores the offset as stated, the named observable has the quantity
type of the clause's observable, and a point's coverage is decided only when the evaluation has a
value of the reference observable.

The fixture is the Alibakhshi validity range of `chemicals`: from the normal boiling temperature
minus 50 K to the critical temperature minus 100 K, with neither value in the file.
"""

from __future__ import annotations

import uuid
from collections.abc import Iterator
from dataclasses import dataclass

import numpy as np
import psycopg
import pytest

from build_support import fingerprint, inputs_of, write_source
from mapping_support import origin, writer
from mechanisms_support import entity_id, mechanism_declaration
from thermo_knowledge import config, db
from thermo_knowledge.build import build_database
from thermo_knowledge.canonical.values import Quantity
from thermo_knowledge.canonical.writer import CanonicalWriter, ValidationError
from thermo_knowledge.declaration import Declaration
from thermo_knowledge.expression.evaluate import bind
from thermo_knowledge.expression.validity import Membership
from thermo_knowledge.qualify.source import DatabaseSource
from thermo_knowledge.testing import TestDatabase
from thermo_knowledge.verify.checks import VERIFY_DIR, load_checks
from thermo_knowledge.verify.run import run_check, run_checks

INSIDE, OUTSIDE = int(Membership.INSIDE), int(Membership.OUTSIDE)
UNDETERMINED = int(Membership.UNDETERMINED)
CHECKS = {c.target: c for c in load_checks(config.TREE_DIR / VERIFY_DIR)[0]}
RELATIVE = "relative_bounds_fixture.pure"
PLAIN = "plain_bounds_fixture.pure"
BOILING, CRITICAL = 350.0, 500.0  # the region is 300 K to 400 K


@dataclass
class World:
    decl: Declaration
    database: TestDatabase
    ids: dict[str, uuid.UUID]


def at(tag: str) -> list:  # noqa: ANN401
    return [origin(f"a.json#/{tag}", "published")]


def clause(decl: Declaration, **bounds: object) -> dict[str, object]:
    return {"observable": entity_id(decl, "observable", "temperature"), **bounds}


def alibakhshi(decl: Declaration) -> list[dict[str, object]]:
    """Tb - 50 K to Tc - 100 K: each bound an offset from a different observable."""
    return [
        clause(
            decl,
            lower=Quantity(-50.0, "K"),
            lower_relative_to=entity_id(decl, "observable", "normal_boiling_temperature"),
            upper=Quantity(-100.0, "K"),
            upper_relative_to=entity_id(decl, "observable", "critical_temperature"),
        )
    ]


def write_world(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    species = {
        key: w.kind("species", {"canonical_key": key, "label": key}, origins=at(f"s-{key}"))
        for key in ("with_reference", "plain")
    }
    p = w.kind(
        "parameterization",
        {"key": "chemicals", "revision": "1", "title": "t", "coherence": "independent_records"},
        origins=at("p"),
    )
    ids["parameterization"] = p
    ids["species_with_reference"], ids["species_plain"] = (
        species["with_reference"],
        species["plain"],
    )
    ids["set_with_reference"] = w.parameter_set(
        parameterization=p,
        slot_group=RELATIVE,
        subjects=[species["with_reference"]],
        slots={
            "a": 20.0,
            "b": Quantity(3000.0, "K"),
            "T_b": Quantity(BOILING, "K"),
            "T_c": Quantity(CRITICAL, "K"),
        },
        origins=at("set-with-reference"),
    )
    ids["set_plain"] = w.parameter_set(
        parameterization=p,
        slot_group=PLAIN,
        subjects=[species["plain"]],
        slots={"a": 20.0, "b": Quantity(3000.0, "K")},
        origins=at("set-plain"),
    )
    for key in ("with_reference", "plain"):
        w.validity_region(
            ids[f"set_{key}"],
            {"kind": "fitted_range"},
            alibakhshi(decl),
            origins=at(f"region-{key}"),
        )


@pytest.fixture(scope="module")
def decl(tmp_path_factory: pytest.TempPathFactory) -> Declaration:
    return mechanism_declaration(tmp_path_factory.mktemp("relative-declaration"))


@pytest.fixture(scope="module")
def world(decl: Declaration, tmp_path_factory: pytest.TempPathFactory) -> Iterator[World]:
    canonical = tmp_path_factory.mktemp("relative-canonical")
    ids: dict[str, uuid.UUID] = {}
    write_source(
        canonical,
        "src",
        lambda w: write_world(w, decl, ids),
        decl=decl,
        declaration=fingerprint(decl),
    )
    with TestDatabase() as database:
        build_database(database.url, decl, inputs_of(canonical))
        yield World(decl, database, ids)


@pytest.fixture
def conn(world: World) -> Iterator[psycopg.Connection]:
    connection = db.connect(world.database.url)
    connection.execute("SELECT 1")
    try:
        yield connection
    finally:
        connection.rollback()
        connection.close()


def bound(world: World, conn: psycopg.Connection, form: str, species: str):  # noqa: ANN201
    source = DatabaseSource(conn, world.decl, [world.ids["parameterization"]])
    return bind(world.decl, form, source=source, roles={"i": str(world.ids[f"species_{species}"])})


TEMPERATURES = np.array([299.0, 300.0, 350.0, 400.0, 401.0])


# -- the clause holds the offsets as stated -----------------------------------------------------


def test_the_clause_stores_the_offsets_and_the_observables_they_are_relative_to(
    world: World, conn: psycopg.Connection
) -> None:
    row = conn.execute(
        "SELECT c.lower, c.upper, lo.key, hi.key FROM tk.region_clause c "
        "JOIN tk.observable lo ON lo.id = c.lower_relative_to "
        "JOIN tk.observable hi ON hi.id = c.upper_relative_to "
        "JOIN tk.validity_region r ON r.id = c.region WHERE r.record = %s",
        (world.ids["set_with_reference"],),
    ).fetchone()
    assert row == (-50.0, -100.0, "normal_boiling_temperature", "critical_temperature")


def test_every_fixture_satisfies_every_invariant(world: World, conn: psycopg.Connection) -> None:
    results = run_checks(conn, list(CHECKS.values()))
    assert [(r.check.target, r.violations, r.error) for r in results if not r.passed] == []


# -- evaluation ---------------------------------------------------------------------------------


def test_a_point_is_inside_or_outside_when_the_selected_set_holds_the_reference_values(
    world: World, conn: psycopg.Connection
) -> None:
    """The set holds Tb = 350 K and Tc = 500 K, so the region is [300 K, 400 K]: the bounds are
    decided with no value passed in."""
    form = bound(world, conn, "relative_bounds_fixture", "with_reference")
    codes = form.validity("p_sat", "fitted_range", T=TEMPERATURES)
    assert codes.tolist() == [OUTSIDE, INSIDE, INSIDE, INSIDE, OUTSIDE]


def test_a_value_passed_in_decides_a_set_that_holds_none(
    world: World, conn: psycopg.Connection
) -> None:
    form = bound(world, conn, "plain_bounds_fixture", "plain")
    codes = form.validity(
        "p_sat",
        "fitted_range",
        T=TEMPERATURES,
        references={"normal_boiling_temperature": BOILING, "critical_temperature": CRITICAL},
    )
    assert codes.tolist() == [OUTSIDE, INSIDE, INSIDE, INSIDE, OUTSIDE]


def test_a_point_is_undetermined_when_no_value_of_the_reference_is_available(
    world: World, conn: psycopg.Connection
) -> None:
    form = bound(world, conn, "plain_bounds_fixture", "plain")
    assert form.validity("p_sat", "fitted_range", T=TEMPERATURES).tolist() == [UNDETERMINED] * 5


def test_one_missing_reference_leaves_the_region_undetermined_but_a_failing_clause_still_decides(
    world: World, conn: psycopg.Connection
) -> None:
    """With only the boiling temperature known the upper bound cannot be placed: the region is
    undetermined (its one clause cannot be decided), never inside."""
    form = bound(world, conn, "plain_bounds_fixture", "plain")
    codes = form.validity(
        "p_sat", "fitted_range", T=TEMPERATURES, references={"normal_boiling_temperature": BOILING}
    )
    assert codes.tolist() == [UNDETERMINED] * 5


def test_a_passed_value_is_used_in_place_of_the_value_the_selected_set_holds(
    world: World, conn: psycopg.Connection
) -> None:
    form = bound(world, conn, "relative_bounds_fixture", "with_reference")
    codes = form.validity(
        "p_sat",
        "fitted_range",
        T=TEMPERATURES,
        references={"normal_boiling_temperature": 360.0, "critical_temperature": 510.0},
    )
    assert codes.tolist() == [
        OUTSIDE,
        OUTSIDE,
        INSIDE,
        INSIDE,
        INSIDE,
    ]  # the region is [310 K, 410 K]


def test_a_kind_of_region_no_record_states_is_still_reported_not_stated(
    world: World, conn: psycopg.Connection
) -> None:
    """Relative bounds change how a clause is decided, not what a missing kind means."""
    form = bound(world, conn, "plain_bounds_fixture", "plain")
    assert (
        form.validity("p_sat", "validated_range", T=TEMPERATURES).tolist() == [3] * 5
    )  # not stated


# -- the requirements ---------------------------------------------------------------------------


def region_for(
    w: CanonicalWriter, decl: Declaration, clauses: list[dict[str, object]], key: str
) -> uuid.UUID:
    species = w.kind("species", {"canonical_key": key, "label": key}, origins=at(f"s-{key}"))
    p = w.kind(
        "parameterization",
        {"key": key, "revision": "1", "title": "t", "coherence": "independent_records"},
        origins=at(f"p-{key}"),
    )
    made = w.parameter_set(
        parameterization=p,
        slot_group=PLAIN,
        subjects=[species],
        slots={"a": 1.0, "b": Quantity(1.0, "K")},
        origins=at(f"set-{key}"),
    )
    return w.validity_region(made, {"kind": "fitted_range"}, clauses, origins=at(f"region-{key}"))


def fresh(decl: Declaration) -> CanonicalWriter:
    return writer(decl)


def test_bounds_against_one_reference_must_be_ordered_and_bounds_against_two_are_not_compared(
    decl: Declaration,
) -> None:
    tb = entity_id(decl, "observable", "normal_boiling_temperature")
    tc = entity_id(decl, "observable", "critical_temperature")
    w = fresh(decl)
    region_for(w, decl, alibakhshi(decl), "different-references")  # -50 from Tb, -100 from Tc
    with pytest.raises(ValidationError, match="same reference"):
        region_for(
            w,
            decl,
            [
                clause(
                    decl,
                    lower=Quantity(-50.0, "K"),
                    lower_relative_to=tb,
                    upper=Quantity(-100.0, "K"),
                    upper_relative_to=tb,
                )
            ],
            "same-reference",
        )
    region_for(  # both absolute: ordered as before
        w, decl, [clause(decl, lower=Quantity(300.0, "K"), upper=Quantity(400.0, "K"))], "absolute"
    )
    with pytest.raises(ValidationError, match="exceeds"):
        region_for(
            w,
            decl,
            [clause(decl, lower=Quantity(400.0, "K"), upper=Quantity(300.0, "K"))],
            "reversed",
        )
    region_for(  # one relative and one absolute are not compared either
        w,
        decl,
        [
            clause(
                decl, lower=Quantity(-50.0, "K"), lower_relative_to=tc, upper=Quantity(400.0, "K")
            )
        ],
        "mixed",
    )


def test_an_offset_may_be_negative_although_temperature_is_absolute(decl: Declaration) -> None:
    w = fresh(decl)
    tb = entity_id(decl, "observable", "normal_boiling_temperature")
    region_for(
        w, decl, [clause(decl, lower=Quantity(-50.0, "K"), lower_relative_to=tb)], "negative-offset"
    )


def test_an_offset_is_in_the_unit_the_clause_observable_is_stored_in(decl: Declaration) -> None:
    """An offset of 50 degrees Celsius of difference is 50 K: the writer converts it like any
    bound of the clause's observable."""
    w = fresh(decl)
    tb = entity_id(decl, "observable", "normal_boiling_temperature")
    region_for(
        w,
        decl,
        [clause(decl, lower=Quantity(-50.0, "delta_degC"), lower_relative_to=tb)],
        "celsius-offset",
    )
    rows = w.tables()["tk.region_clause"].to_pylist()
    assert [row["lower"] for row in rows] == pytest.approx([-50.0])


def test_a_reference_observable_of_another_quantity_type_is_refused_by_the_check(
    decl: Declaration, tmp_path_factory: pytest.TempPathFactory
) -> None:
    critical_pressure = entity_id(decl, "observable", "critical_pressure")
    canonical = tmp_path_factory.mktemp("relative-refused")

    def fill(w: CanonicalWriter) -> None:
        region_for(
            w,
            decl,
            [clause(decl, lower=Quantity(-50.0, "K"), lower_relative_to=critical_pressure)],
            "wrong-type",
        )
        tb = entity_id(decl, "observable", "normal_boiling_temperature")
        region_for(  # a reference named for a bound that is not stated
            w, decl, [clause(decl, upper=Quantity(400.0, "K"), lower_relative_to=tb)], "no-bound"
        )
        region_for(w, decl, alibakhshi(decl), "sound")

    write_source(canonical, "src", fill, decl=decl, declaration=fingerprint(decl))
    with TestDatabase() as database:
        build_database(database.url, decl, inputs_of(canonical))
        with db.connect(database.url) as conn:
            found = run_check(conn, CHECKS["region_clause.relative_bound_matches_observable"])
    reasons = sorted(row[-1] for row in found.rows)
    assert reasons == [
        "the lower bound is not stated",
        "the observable critical_pressure has another quantity type than the clause's observable",
    ]
