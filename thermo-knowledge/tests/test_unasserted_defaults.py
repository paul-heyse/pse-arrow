# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""A subject the source holds no set for takes the defaults of a policy that applies stated defaults
to unasserted subjects (plan 24, packet TK2, hard case (c); meta-model section 5, selection policies).

A table a source lists only in part (the bonding pairs of an association matrix, the pairs of a
binary table that interact) is read as complete by an expression over all pairs. The policy's
`unasserted = stated_default` states what a pair that is not listed takes, for every slot of the
slot group it covers; the database-backed source supplies it when it has found no set in any order
of the pair. A policy that refuses, covers another slot group or states no default for one slot of
the group supplies nothing, and the evaluation refuses as it does without a policy.
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
from thermo_knowledge import db
from thermo_knowledge.build import build_database
from thermo_knowledge.canonical.values import Quantity
from thermo_knowledge.canonical.writer import CanonicalWriter
from thermo_knowledge.declaration import Declaration
from thermo_knowledge.expression.evaluate import EvaluationRefusal, bind
from thermo_knowledge.qualify.source import DatabaseSource
from thermo_knowledge.testing import TestDatabase

NRTL = "nrtl_fixture.pair"
OTHER = "pcsaft_binary_fixture.pair"
ALPHA, A, B = 0.3, 0.8, 50.0


@dataclass
class World:
    decl: Declaration
    database: TestDatabase
    ids: dict[str, uuid.UUID]


def at(tag: str) -> list:  # noqa: ANN401
    return [origin(f"a.json#/{tag}", "published")]


def write_world(w: CanonicalWriter, ids: dict[str, uuid.UUID]) -> None:
    a, b, c = (
        w.kind("species", {"canonical_key": n, "label": n}, origins=at(f"s-{n}")) for n in "abc"
    )
    ids.update(a=a, b=b, c=c)
    p = w.kind(
        "parameterization",
        {"key": "listed", "revision": "1", "title": "t", "coherence": "independent_records"},
        origins=at("p"),
    )
    ids["parameterization"] = p
    w.parameter_set(  # the only pair the source lists
        parameterization=p,
        slot_group=NRTL,
        subjects=[a, b],
        slots={"a": 1.5, "b": Quantity(300.0, "K"), "alpha": 0.2},
        origins=at("listed"),
    )

    def policy(
        key: str, unasserted: str, slots: tuple[str, ...], scope: str | None, group: str = NRTL
    ) -> None:
        ids[key] = w.kind(
            "selection_policy",
            {
                "key": key,
                "revision": "1",
                "unasserted": unasserted,
                **({} if scope is None else {"scope_slot_group": scope}),
            },
            origins=at(f"policy-{key}"),
        )
        for slot in slots:
            value = {"a": A, "b": Quantity(B, "K"), "alpha": ALPHA, "k_ij": 0.0, "l_ij": 0.0}[slot]
            w.relation(
                "policy_default",
                {"policy": ids[key], "slot": f"{group}.{slot}"},
                {"state": "known", "value": value},
                at="a.json#/default",
            )

    policy("scoped", "stated_default", ("a", "b", "alpha"), NRTL)
    policy("unscoped", "stated_default", ("a", "b", "alpha"), None)
    policy("refusing", "refuse", ("a", "b", "alpha"), NRTL)
    policy("other_group", "stated_default", ("k_ij", "l_ij"), OTHER, OTHER)
    policy("incomplete", "stated_default", ("a", "alpha"), NRTL)


@pytest.fixture(scope="module")
def world(tmp_path_factory: pytest.TempPathFactory) -> Iterator[World]:
    decl = mechanism_declaration(tmp_path_factory.mktemp("unasserted-declaration"))
    canonical = tmp_path_factory.mktemp("unasserted-canonical")
    ids: dict[str, uuid.UUID] = {}
    write_source(
        canonical, "src", lambda w: write_world(w, ids), decl=decl, declaration=fingerprint(decl)
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


def source(world: World, conn: psycopg.Connection, policy: str | None) -> DatabaseSource:
    return DatabaseSource(
        conn,
        world.decl,
        [world.ids["parameterization"]],
        policy=None if policy is None else world.ids[policy],
    )


def unlisted(world: World) -> tuple[str, str]:
    return str(world.ids["a"]), str(world.ids["c"])


@pytest.mark.parametrize("policy", ["scoped", "unscoped"])
def test_a_policy_that_covers_the_group_supplies_its_defaults_for_a_pair_that_is_not_listed(
    world: World, conn: psycopg.Connection, policy: str
) -> None:
    found = source(world, conn, policy)
    assert found.slot_values(NRTL, unlisted(world)) is None  # no set: the source still says so
    assert found.default_slot_values(NRTL, unlisted(world)) == {"a": A, "b": B, "alpha": ALPHA}


@pytest.mark.parametrize("policy", [None, "refusing", "other_group", "incomplete"])
def test_a_policy_that_refuses_covers_another_group_or_leaves_a_slot_without_a_default_supplies_none(
    world: World, conn: psycopg.Connection, policy: str | None
) -> None:
    assert source(world, conn, policy).default_slot_values(NRTL, unlisted(world)) is None


def nrtl(
    world: World, found: DatabaseSource, first: str, second: str, temperature: np.ndarray
) -> np.ndarray:
    bound = bind(
        world.decl,
        "nrtl_fixture",
        source=found,
        roles={"i": str(world.ids[first]), "j": str(world.ids[second])},
    )
    return bound.evaluate("G", T=temperature)


def test_an_expression_over_an_unlisted_pair_takes_the_policys_defaults(
    world: World, conn: psycopg.Connection
) -> None:
    temperature = np.array([280.0, 350.0])
    got = nrtl(world, source(world, conn, "scoped"), "a", "c", temperature)
    np.testing.assert_allclose(got, np.exp(-ALPHA * (A + B / temperature)), rtol=1e-14)


def test_a_listed_pair_keeps_its_own_values_under_the_same_policy(
    world: World, conn: psycopg.Connection
) -> None:
    temperature = np.array([300.0])
    got = nrtl(world, source(world, conn, "scoped"), "a", "b", temperature)
    np.testing.assert_allclose(got, np.exp(-0.2 * (1.5 + 300.0 / temperature)), rtol=1e-14)


@pytest.mark.parametrize("policy", [None, "refusing", "incomplete"])
def test_without_defaults_for_unasserted_pairs_the_evaluation_refuses_the_unlisted_pair(
    world: World, conn: psycopg.Connection, policy: str | None
) -> None:
    with pytest.raises(EvaluationRefusal, match="nrtl_fixture.pair"):
        nrtl(world, source(world, conn, policy), "a", "c", np.array([300.0]))
