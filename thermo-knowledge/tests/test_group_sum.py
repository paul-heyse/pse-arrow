# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Group counts read by an expression as an indexed contract argument (plan 24, packet TK2g,
proof fixture 2; the mechanism Clapeyron's `group_sum` dispositions assume).

The UNIFAC relative volume r and surface q of a molecule are sums over its groups of the number of
times each group occurs times the group's parameter. The count of each group is stored as the
`group_count` rows of the molecule's group assignment; the form reads them as an indexed argument
over the groups of the scheme, and the group parameters as a slot group of the groups.
"""

from __future__ import annotations

import uuid
from collections.abc import Iterator
from dataclasses import dataclass

import numpy as np
import psycopg
import pytest
from build_support import fingerprint, inputs_of, write_source
from mapping_support import carrier, origin
from mechanisms_support import mechanism_declaration

from thermo_knowledge import config, db, identity
from thermo_knowledge.build import build_database
from thermo_knowledge.canonical.writer import CanonicalWriter
from thermo_knowledge.declaration import Declaration
from thermo_knowledge.expression.evaluate import bind
from thermo_knowledge.qualify.source import DatabaseSource
from thermo_knowledge.testing import TestDatabase
from thermo_knowledge.verify.checks import VERIFY_DIR, load_checks
from thermo_knowledge.verify.run import run_checks

CARRIER = identity.identifier("source", [carrier("src", "a.json").key])
CHECKS = {c.target: c for c in load_checks(config.TREE_DIR / VERIFY_DIR)[0]}
FORM = "group_size_fixture"
# UNIFAC (Hansen et al. 1991): subgroup code, label, R, Q
GROUPS = {
    "1": ("CH3", 0.9011, 0.848),
    "2": ("CH2", 0.6744, 0.540),
    "14": ("OH", 1.0000, 1.200),
    "18": ("CH3CO", 1.6724, 1.488),  # in the scheme, in no molecule below
}
MOLECULES = {"ethanol": {"1": 1, "2": 1, "14": 1}, "propan-1-ol": {"1": 1, "2": 2, "14": 1}}


@dataclass
class World:
    decl: Declaration
    database: TestDatabase
    ids: dict[str, uuid.UUID]


def at(tag: str) -> list:  # noqa: ANN401
    return [origin(f"a.json#/{tag}", "published")]


def write_world(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    scheme = w.kind(
        "group_scheme", {"key": "unifac", "revision": "1", "role": "activity"}, origins=at("scheme")
    )
    groups = {
        code: w.kind(
            "group",
            {"scheme": scheme, "code": code, "label": label, "role": "group"},
            origins=at(f"group-{code}"),
        )
        for code, (label, _, _) in GROUPS.items()
    }
    ids.update({f"group_{code}": value for code, value in groups.items()})
    p = w.kind(
        "parameterization",
        {
            "key": "unifac-groups",
            "revision": "1",
            "title": "t",
            "coherence": "independent_records",
            "group_scheme": scheme,
        },
        origins=at("p"),
    )
    ids["parameterization"] = p
    for code, (_, r, q) in GROUPS.items():
        w.parameter_set(
            parameterization=p,
            slot_group=f"{FORM}.group",
            subjects=[groups[code]],
            slots={"R": r, "Q": q},
            origins=at(f"parameters-{code}"),
        )
    for name, counts in MOLECULES.items():
        species = w.kind("species", {"canonical_key": name, "label": name}, origins=at(f"s-{name}"))
        ids[f"species_{name}"] = species
        assignment = w.kind(
            "group_assignment",
            {"entity": species, "scheme": scheme, "asserted_by": CARRIER, "origin": "published"},
            origins=at(f"assignment-{name}"),
        )
        ids[f"assignment_{name}"] = assignment
        for code, count in counts.items():
            w.relation(
                "group_count",
                {"assignment": assignment, "group": groups[code]},
                {"value": count},
                at="a.json#/count",
            )


@pytest.fixture(scope="module")
def world(tmp_path_factory: pytest.TempPathFactory) -> Iterator[World]:
    decl = mechanism_declaration(tmp_path_factory.mktemp("group-declaration"))
    canonical = tmp_path_factory.mktemp("group-canonical")
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


def evaluate(world: World, conn: psycopg.Connection, molecule: str, order: list[str], output: str) -> float:
    """r or q of `molecule`, the counts read from its assignment's `group_count` rows for the
    groups of the scheme taken in `order`."""
    counts = dict(
        conn.execute(
            "SELECT gc.group, gc.value FROM tk.group_count gc WHERE gc.assignment = %s",
            (world.ids[f"assignment_{molecule}"],),
        ).fetchall()
    )
    members = [world.ids[f"group_{code}"] for code in order]
    source = DatabaseSource(conn, world.decl, [world.ids["parameterization"]])
    bound = bind(
        world.decl,
        FORM,
        source=source,
        roles={"i": str(world.ids[f"species_{molecule}"])},
        sets={"groups": [str(member) for member in members]},
    )
    found = bound.evaluate(
        output, nu={str(member): np.array([float(counts.get(member, 0))]) for member in members}
    )
    return float(np.asarray(found).reshape(-1)[0])


def test_every_fixture_satisfies_every_invariant(world: World, conn: psycopg.Connection) -> None:
    results = run_checks(conn, list(CHECKS.values()))
    assert [(r.check.target, r.violations, r.error) for r in results if not r.passed] == []


@pytest.mark.parametrize("molecule", list(MOLECULES))
def test_r_and_q_are_the_sums_of_the_group_counts_times_the_group_parameters(
    world: World, conn: psycopg.Connection, molecule: str
) -> None:
    order = list(GROUPS)
    r = sum(n * GROUPS[code][1] for code, n in MOLECULES[molecule].items())
    q = sum(n * GROUPS[code][2] for code, n in MOLECULES[molecule].items())
    assert evaluate(world, conn, molecule, order, "r") == pytest.approx(r, rel=1e-13)
    assert evaluate(world, conn, molecule, order, "q") == pytest.approx(q, rel=1e-13)


def test_ethanol_has_the_published_unifac_size_and_surface(world: World, conn: psycopg.Connection) -> None:
    assert evaluate(world, conn, "ethanol", list(GROUPS), "r") == pytest.approx(2.5755, abs=1e-12)
    assert evaluate(world, conn, "ethanol", list(GROUPS), "q") == pytest.approx(2.588, abs=1e-12)


def test_the_sum_does_not_depend_on_the_order_of_the_groups(world: World, conn: psycopg.Connection) -> None:
    reference = evaluate(world, conn, "propan-1-ol", list(GROUPS), "r")
    for order in (["14", "18", "2", "1"], ["2", "1", "14", "18"]):
        assert evaluate(world, conn, "propan-1-ol", order, "r") == pytest.approx(reference, rel=1e-13)


def test_a_group_that_is_absent_from_the_assignment_counts_zero(world: World, conn: psycopg.Connection) -> None:
    """The counts of the assignment are a relation whose absence means zero: the group `CH3CO`
    has no row for ethanol and contributes nothing, although it is in the scheme's list."""
    rows = conn.execute(
        "SELECT count(*) FROM tk.group_count WHERE assignment = %s AND \"group\" = %s",
        (world.ids["assignment_ethanol"], world.ids["group_18"]),
    ).fetchone()
    assert rows == (0,)
    with_it = evaluate(world, conn, "ethanol", list(GROUPS), "r")
    without = evaluate(world, conn, "ethanol", ["1", "2", "14"], "r")
    assert with_it == pytest.approx(without, rel=1e-13)
