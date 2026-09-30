# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""A combining rule declared as a form and referenced by a policy for unasserted pairs (plan 24,
packet TK2g, proof fixture 3; the mechanism the Clapeyron, teqp and FeOS dispositions assume).

The Lorentz-Berthelot rule gives the segment diameter and the dispersion energy of a pair from
those of its two species. It is an ordinary form; a selection policy whose `unasserted` is
`named_rule` names it in `rule_form`, for the slot group of the pair table it is scoped to. The
evaluator never applies a rule by itself: a pair the source leaves unasserted is refused until a
selection applies the named form.
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
from mechanisms_support import mechanism_declaration

from thermo_knowledge import config, db
from thermo_knowledge.build import build_database
from thermo_knowledge.canonical.values import Quantity
from thermo_knowledge.canonical.writer import CanonicalWriter
from thermo_knowledge.declaration import Declaration
from thermo_knowledge.expression.evaluate import bind
from thermo_knowledge.qualify.source import DatabaseSource
from thermo_knowledge.testing import TestDatabase
from thermo_knowledge.verify.checks import VERIFY_DIR, load_checks
from thermo_knowledge.verify.run import run_check, run_checks

CHECKS = {c.target: c for c in load_checks(config.TREE_DIR / VERIFY_DIR)[0]}
TABLE = "pair_dispersion_table_fixture.pair"
RULE = "lorentz_berthelot_fixture"
SPECIES = {"a": (3.5, 120.0), "b": (3.0, 150.0), "c": (4.1, 300.0)}  # sigma in angstrom, epsilon over k in K
ASSERTED = {("a", "b"): (3.3, 145.0)}  # a pair the source states, off the rule
ANGSTROM = 1e-10


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
        for name in SPECIES
    }
    ids.update({f"species_{name}": value for name, value in species.items()})
    p = w.kind(
        "parameterization",
        {"key": "saft-like", "revision": "1", "title": "t", "coherence": "independent_records"},
        origins=at("p"),
    )
    ids["parameterization"] = p
    for name, (sigma, epsilon) in SPECIES.items():
        w.parameter_set(
            parameterization=p,
            slot_group="lorentz_berthelot_fixture.pure",
            subjects=[species[name]],
            slots={"sigma": Quantity(sigma, "angstrom"), "epsilon": Quantity(epsilon, "K")},
            origins=at(f"pure-{name}"),
        )
    for (first, second), (sigma, epsilon) in ASSERTED.items():
        w.parameter_set(
            parameterization=p,
            slot_group=TABLE,
            subjects=[species[first], species[second]],
            slots={"sigma": Quantity(sigma, "angstrom"), "epsilon": Quantity(epsilon, "K")},
            origins=at(f"pair-{first}{second}"),
        )
    ids["policy"] = w.kind(
        "selection_policy",
        {
            "key": "combine-unasserted-pairs",
            "revision": "1",
            "unasserted": "named_rule",
            "rule_form": RULE,
            "scope_slot_group": TABLE,
        },
        origins=at("policy"),
    )
    w.relation(
        "policy_precedence",
        {"policy": ids["policy"], "parameterization": p},
        {"value": 1},
        at="a.json#/rank",
    )


@pytest.fixture(scope="module")
def decl(tmp_path_factory: pytest.TempPathFactory) -> Declaration:
    return mechanism_declaration(tmp_path_factory.mktemp("combining-declaration"))


@pytest.fixture(scope="module")
def world(decl: Declaration, tmp_path_factory: pytest.TempPathFactory) -> Iterator[World]:
    canonical = tmp_path_factory.mktemp("combining-canonical")
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


def rule(world: World, conn: psycopg.Connection, first: str, second: str) -> tuple[float, float]:
    source = DatabaseSource(conn, world.decl, [world.ids["parameterization"]])
    bound = bind(
        world.decl,
        RULE,
        source=source,
        sets={"components": [str(world.ids[f"species_{first}"]), str(world.ids[f"species_{second}"])]},
    )
    return tuple(  # type: ignore[return-value]
        float(np.asarray(bound.evaluate(output)).reshape(-1)[0]) for output in ("sigma", "epsilon")
    )


def test_every_fixture_satisfies_every_invariant(world: World, conn: psycopg.Connection) -> None:
    results = run_checks(conn, list(CHECKS.values()))
    assert [(r.check.target, r.violations, r.error) for r in results if not r.passed] == []


def test_the_policy_names_the_combining_form(world: World, conn: psycopg.Connection) -> None:
    row = conn.execute(
        "SELECT p.unasserted::text, f.name, f.implements, sg.qualified_name FROM tk.selection_policy p "
        "JOIN meta.form f ON f.id = p.rule_form JOIN meta.slot_group sg ON sg.id = p.scope_slot_group "
        "WHERE p.id = %s",
        (world.ids["policy"],),
    ).fetchone()
    assert row == ("named_rule", RULE, "pair_dispersion_fixture", TABLE)


def test_the_rule_form_gives_the_lorentz_berthelot_values_of_an_unasserted_pair(
    world: World, conn: psycopg.Connection
) -> None:
    sigma, epsilon = rule(world, conn, "a", "c")
    assert sigma == pytest.approx((3.5 + 4.1) / 2 * ANGSTROM, rel=1e-12)
    assert epsilon == pytest.approx(np.sqrt(120.0 * 300.0), rel=1e-12)


def test_the_rule_does_not_depend_on_the_order_of_the_pair(world: World, conn: psycopg.Connection) -> None:
    assert rule(world, conn, "a", "c") == pytest.approx(rule(world, conn, "c", "a"), rel=1e-14)


def test_a_pair_the_source_states_is_read_from_the_table_and_is_not_the_rule_value(
    world: World, conn: psycopg.Connection
) -> None:
    source = DatabaseSource(conn, world.decl, [world.ids["parameterization"]])
    subjects = (str(world.ids["species_a"]), str(world.ids["species_b"]))
    stated = source.slot_values(TABLE, subjects)
    assert stated == pytest.approx({"sigma": 3.3 * ANGSTROM, "epsilon": 145.0})
    sigma, epsilon = rule(world, conn, "a", "b")
    assert abs(epsilon - stated["epsilon"]) > 1.0  # the source's value is not what the rule gives


def test_an_unasserted_pair_has_no_set_until_a_selection_applies_the_rule(
    world: World, conn: psycopg.Connection
) -> None:
    source = DatabaseSource(conn, world.decl, [world.ids["parameterization"]])
    subjects = (str(world.ids["species_a"]), str(world.ids["species_c"]))
    assert source.slot_values(TABLE, subjects) is None
    assert source.default_slot_values(TABLE, subjects) is None  # the evaluator applies no rule of its own


@pytest.fixture(scope="module")
def violations(
    decl: Declaration, tmp_path_factory: pytest.TempPathFactory
) -> Iterator[tuple[psycopg.Connection, dict[str, uuid.UUID]]]:
    ids: dict[str, uuid.UUID] = {}

    def fill(w: CanonicalWriter) -> None:
        for key, form, scope in (
            ("sound", RULE, TABLE),
            ("wrong-contract", "nrtl_excess_gibbs", TABLE),
            ("unscoped", "nrtl_excess_gibbs", None),
        ):
            ids[key] = w.kind(
                "selection_policy",
                {
                    "key": key,
                    "revision": "1",
                    "unasserted": "named_rule",
                    "rule_form": form,
                    **({} if scope is None else {"scope_slot_group": scope}),
                },
                origins=at(f"policy-{key}"),
            )

    canonical = tmp_path_factory.mktemp("combining-violations")
    write_source(canonical, "src", fill, decl=decl, declaration=fingerprint(decl))
    with TestDatabase() as database:
        build_database(database.url, decl, inputs_of(canonical))
        with db.connect(database.url) as conn:
            yield conn, ids


def test_a_rule_form_of_another_contract_than_the_scope_is_flagged(
    violations: tuple[psycopg.Connection, dict[str, uuid.UUID]],
) -> None:
    conn, ids = violations
    found = run_check(conn, CHECKS["selection_policy.rule_form_implements_the_scope_contract"])
    assert [str(row[0]) for row in found.rows] == [str(ids["wrong-contract"])]
