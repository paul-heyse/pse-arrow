# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""A sub-form chosen per ternary (plan 24, packet TK2g, proof fixture 1; the mechanism
Thermochimica's dispositions assume).

`subject_subform_choice` names the form that fills a per-subject sub-form slot for one subject
under a parameterisation. Here the subject is a constituent array of three species and the two
forms are the Kohler and the Toop extrapolation of a ternary excess energy from the binaries of
its pairs: one ternary subsystem is extrapolated one way, another the other way, by the records
alone. The evaluation reads the choice of each array from the database.
"""

from __future__ import annotations

import uuid
from collections.abc import Iterator
from dataclasses import dataclass

import numpy as np
import psycopg
import pytest
from build_support import fingerprint, inputs_of, write_source
from mapping_support import origin
from mechanisms_support import entity_id, mechanism_declaration

from thermo_knowledge import config, db
from thermo_knowledge.build import build_database
from thermo_knowledge.canonical.values import Quantity
from thermo_knowledge.canonical.writer import CanonicalWriter
from thermo_knowledge.declaration import Declaration
from thermo_knowledge.expression.evaluate import EvaluationRefusal, bind
from thermo_knowledge.qualify.source import DatabaseSource, SubformBinding
from thermo_knowledge.testing import TestDatabase
from thermo_knowledge.verify.checks import VERIFY_DIR, load_checks
from thermo_knowledge.verify.run import run_checks

CHECKS = {c.target: c for c in load_checks(config.TREE_DIR / VERIFY_DIR)[0]}
SLOT = "ternary_excess_fixture.extrapolation"
ROOT = "ternary_excess_fixture"
# the binaries, as asserted: (i, j) -> (L0, L1) in J/mol
BINARIES = {
    ("a", "b"): (4000.0, 900.0),
    ("a", "c"): (-2500.0, 400.0),
    ("b", "c"): (1500.0, -700.0),
    ("a", "d"): (3000.0, -300.0),
    ("b", "d"): (-1000.0, 650.0),
}
ARRAYS = {"first": ("a", "b", "c"), "second": ("a", "b", "d")}
CHOSEN = {"first": "kohler_fixture", "second": "toop_fixture"}


@dataclass
class World:
    decl: Declaration
    database: TestDatabase
    ids: dict[str, uuid.UUID]


def at(tag: str) -> list:  # noqa: ANN401
    return [origin(f"a.json#/{tag}", "published")]


def write_world(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    species = {
        name: w.kind("species", {"canonical_key": name, "label": name}, origins=at(f"s-{name}"))
        for name in "abcd"
    }
    ids.update({f"species_{name}": value for name, value in species.items()})
    system = w.kind("chemical_system", {"key": "abcd", "revision": "1"}, origins=at("system"))
    phase = w.kind(
        "phase_definition",
        {
            "system": system,
            "key": "LIQUID",
            "aggregation": entity_id(decl, "aggregation", "liquid"),
            "structure": "sublattice",
        },
        origins=at("phase"),
    )
    site = w.kind(
        "site_class",
        {"phase": phase, "index": 1, "ratio_kind": "constant", "ratio": 1},
        origins=at("site"),
    )
    p = w.kind(
        "parameterization",
        {"key": "ternaries", "revision": "1", "title": "t", "coherence": "independent_records"},
        origins=at("p"),
    )
    ids["parameterization"] = p
    for name, members in ARRAYS.items():
        array = w.kind(
            "constituent_array",
            {"phase": phase, "canonical_key": "LIQUID:" + ",".join(members)},
            origins=at(f"array-{name}"),
        )
        ids[f"array_{name}"] = array
        for position, member in enumerate(members, start=1):
            w.relation(
                "constituent_array_member",
                {"array": array, "site_class": site, "position": position},
                {"species": species[member]},
                at="a.json#/member",
            )
        w.subform_choice(
            parameterization=p,
            slot=SLOT,
            subjects=[array],
            form=CHOSEN[name],
            at="a.json#/choice",
        )
    for (first, second), (l0, l1) in BINARIES.items():
        w.parameter_set(
            parameterization=p,
            slot_group="redlich_kister_fixture.pair",
            subjects=[species[first], species[second]],
            slots={"L0": Quantity(l0, "J/mol"), "L1": Quantity(l1, "J/mol")},
            origins=at(f"binary-{first}{second}"),
        )


@pytest.fixture(scope="module")
def world(tmp_path_factory: pytest.TempPathFactory) -> Iterator[World]:
    decl = mechanism_declaration(tmp_path_factory.mktemp("ternary-declaration"))
    canonical = tmp_path_factory.mktemp("ternary-canonical")
    ids: dict[str, uuid.UUID] = {}
    write_source(canonical, "src", lambda w: write_world(w, decl, ids), decl=decl, declaration=fingerprint(decl))
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


# -- the reference: the two extrapolations written out --------------------------------------------


def binary(i: str, j: str, xi: float) -> float:
    """The excess energy of the binary of i and j at the mole fraction xi of i, from the pair as
    asserted (L1 changes sign when the pair is read the other way round)."""
    if (i, j) in BINARIES:
        l0, l1 = BINARIES[(i, j)]
    else:
        l0, l1 = BINARIES[(j, i)][0], -BINARIES[(j, i)][1]
    return xi * (1 - xi) * (l0 + l1 * (2 * xi - 1))


def kohler(names: tuple[str, ...], x: dict[str, float]) -> float:
    total = 0.0
    for index, i in enumerate(names):
        for j in names[index + 1 :]:
            total += (x[i] + x[j]) ** 2 * binary(i, j, x[i] / (x[i] + x[j]))
    return total


def toop(names: tuple[str, ...], x: dict[str, float], special: str) -> float:
    others = [n for n in names if n != special]
    total = sum(x[j] / (1 - x[special]) * binary(special, j, x[special]) for j in others)
    (j, k) = others
    return total + (x[j] + x[k]) ** 2 * binary(j, k, x[j] / (x[j] + x[k]))


COMPOSITION = {"a": 0.2, "b": 0.5, "c": 0.3}


def source_for(world: World, conn: psycopg.Connection) -> DatabaseSource:
    p = world.ids["parameterization"]
    binding = [SubformBinding("redlich_kister_fixture", (p,))]
    return DatabaseSource(
        conn,
        world.decl,
        [p],
        subforms={"kohler_fixture.binary": binding, "toop_fixture.binary": binding},
    )


def ternary(
    world: World,
    conn: psycopg.Connection,
    array: str,
    order: tuple[str, ...],
    x: dict[str, float],
    *,
    special: str | None = None,
) -> float:
    """The excess energy of the ternary `array` with its components passed in `order`; the species
    singled out by the array (its first) unless `special` names another."""
    members = ARRAYS[array]
    chosen = special or members[0]
    ids = {name: str(world.ids[f"species_{name}"]) for name in order}
    bound = bind(
        world.decl,
        ROOT,
        source=source_for(world, conn),
        roles={"t": str(world.ids[f"array_{array}"])},
        sets={"components": [ids[name] for name in order]},
    )
    return float(
        np.asarray(
            bound.evaluate(
                "gE",
                x={ids[name]: np.array([x[name]]) for name in order},
                first={ids[name]: np.array([1.0 if name == chosen else 0.0]) for name in order},
            )
        ).reshape(-1)[0]
    )


# -- tests ----------------------------------------------------------------------------------------


def test_every_fixture_satisfies_every_invariant(world: World, conn: psycopg.Connection) -> None:
    results = run_checks(conn, list(CHECKS.values()))
    assert [(r.check.target, r.violations, r.error) for r in results if not r.passed] == []


def test_each_ternary_array_has_its_own_choice_of_form(world: World, conn: psycopg.Connection) -> None:
    rows = conn.execute(
        "SELECT sc.subject_key, f.name FROM tk.subject_subform_choice sc JOIN meta.form f ON f.id = sc.form "
        "ORDER BY f.name"
    ).fetchall()
    assert len(rows) == 2
    by_form = {form: key for key, form in rows}
    assert set(by_form) == {"kohler_fixture", "toop_fixture"}
    assert by_form["kohler_fixture"] != by_form["toop_fixture"]


def test_the_subject_of_a_choice_is_the_constituent_array(world: World, conn: psycopg.Connection) -> None:
    from thermo_knowledge import identity

    keys = {key for (key,) in conn.execute("SELECT subject_key FROM tk.subject_subform_choice").fetchall()}
    assert keys == {identity.canonical_encoding([world.ids[f"array_{name}"]]) for name in ARRAYS}


def test_the_ternary_chosen_to_be_extrapolated_by_kohler_is(world: World, conn: psycopg.Connection) -> None:
    found = ternary(world, conn, "first", ("a", "b", "c"), COMPOSITION)
    assert found == pytest.approx(kohler(("a", "b", "c"), COMPOSITION), rel=1e-12)


def test_the_ternary_chosen_to_be_extrapolated_by_toop_is(world: World, conn: psycopg.Connection) -> None:
    x = {"a": 0.25, "b": 0.45, "d": 0.30}
    found = ternary(world, conn, "second", ("a", "b", "d"), x)
    assert found == pytest.approx(toop(("a", "b", "d"), x, "a"), rel=1e-12)


def test_the_two_extrapolations_differ_for_the_same_binaries_and_composition(
    world: World, conn: psycopg.Connection
) -> None:
    x = {"a": 0.25, "b": 0.45, "d": 0.30}
    assert abs(kohler(("a", "b", "d"), x) - toop(("a", "b", "d"), x, "a")) > 1.0, "the choice matters"


def test_kohler_does_not_depend_on_the_order_the_components_are_passed_in(
    world: World, conn: psycopg.Connection
) -> None:
    reference = ternary(world, conn, "first", ("a", "b", "c"), COMPOSITION)
    for order in (("c", "a", "b"), ("b", "c", "a")):
        assert ternary(world, conn, "first", order, COMPOSITION) == pytest.approx(reference, rel=1e-12)


def test_toop_follows_the_species_the_array_puts_first(world: World, conn: psycopg.Connection) -> None:
    x = {"a": 0.25, "b": 0.45, "d": 0.30}
    first = ternary(world, conn, "second", ("a", "b", "d"), x, special="a")
    other = ternary(world, conn, "second", ("a", "b", "d"), x, special="b")
    assert first == pytest.approx(toop(("a", "b", "d"), x, "a"), rel=1e-12)
    assert other == pytest.approx(toop(("a", "b", "d"), x, "b"), rel=1e-12)
    assert abs(first - other) > 1.0


def test_a_ternary_with_no_choice_is_refused_naming_the_sub_form_slot(
    world: World, conn: psycopg.Connection
) -> None:
    x = {"a": 0.2, "b": 0.5, "c": 0.3}
    ids = {name: str(world.ids[f"species_{name}"]) for name in "abc"}
    unchosen = uuid.uuid4()
    bound = bind(
        world.decl,
        ROOT,
        source=source_for(world, conn),
        roles={"t": str(unchosen)},
        sets={"components": [ids[name] for name in "abc"]},
    )
    with pytest.raises(EvaluationRefusal, match=r"ternary_excess_fixture\.extrapolation"):
        bound.evaluate(
            "gE",
            x={ids[name]: np.array([x[name]]) for name in "abc"},
            first={ids[name]: np.array([1.0 if name == "a" else 0.0]) for name in "abc"},
        )
