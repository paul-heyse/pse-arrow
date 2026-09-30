# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The database-backed parameter source against a database built from fixture forms: it reads
what the in-memory source holds for the same fixture, reports a missing set as missing, follows
nested sets, redirects and the sub-form choices it is given, and reads in a bounded number of
queries."""

from __future__ import annotations

import uuid
from collections.abc import Iterator
from pathlib import Path

import numpy as np
import psycopg
import pytest
from mapping_support import real_declaration
from qualify_support import QFIX_A, QFIX_B, World, build_world, fixture_declaration

from thermo_knowledge import db, transposition
from thermo_knowledge.declaration import Declaration
from thermo_knowledge.expression.evaluate import EvaluationRefusal, bind
from thermo_knowledge.expression.parameters import FormChoice, InMemorySource
from thermo_knowledge.qualify.source import (
    AmbiguousOccurrence,
    DatabaseSource,
    SourceError,
    SubformBinding,
)
from thermo_knowledge.testing import TestDatabase

PAIR = "qfix_pair_form.pair"
PURE = "qfix_pair_form.pure"
SHIFT = "qfix_pair_form.shift"
STATEFUL = "qfix_stateful.core"


@pytest.fixture(scope="module")
def decl(tmp_path_factory: pytest.TempPathFactory) -> Declaration:
    return fixture_declaration(tmp_path_factory.mktemp("qualify-declaration"))


@pytest.fixture(scope="module")
def world(
    decl: Declaration, tmp_path_factory: pytest.TempPathFactory
) -> Iterator[tuple[World, TestDatabase]]:
    with TestDatabase() as database:
        yield build_world(tmp_path_factory.mktemp("canonical"), decl, database.url), database


@pytest.fixture
def conn(world: tuple[World, TestDatabase]) -> Iterator[psycopg.Connection]:
    with db.connect(world[1].url) as connection:
        yield connection


def parameterization(conn: psycopg.Connection, key: str) -> uuid.UUID:
    row = conn.execute("SELECT id FROM tk.parameterization WHERE key = %s", (key,)).fetchone()
    assert row is not None
    return row[0]


def pair_source(conn: psycopg.Connection, decl: Declaration) -> DatabaseSource:
    return DatabaseSource(
        conn,
        decl,
        [parameterization(conn, "qfix")],
        subforms={SHIFT: [SubformBinding("qfix_linear", (parameterization(conn, "shift"),))]},
    )


def canonical_pair(world: World) -> tuple[str, str]:
    a, b = world.species[QFIX_A], world.species[QFIX_B]
    ordered, _ = transposition.canonical_orientation(  # type: ignore[type-var]
        next(g for g in world.decl.slot_groups if g.qualified == PAIR), (a, b)
    )
    return str(ordered[0]), str(ordered[1])


def in_memory(world: World) -> InMemorySource:
    """What the fixture holds, written out by hand."""
    sa, sb = str(world.species[QFIX_A]), str(world.species[QFIX_B])
    pair = canonical_pair(world)
    nested = FormChoice(
        "qfix_linear", InMemorySource(slots={("qfix_linear.core", ()): {"a": 0.25, "b": 0.002}})
    )
    shift = FormChoice(
        "qfix_linear", InMemorySource(slots={("qfix_linear.core", ()): {"a": 10.0, "b": -0.01}})
    )
    slots, families = {(PAIR, pair): {"k": 1.75}}, {}
    for position, subject in enumerate((sa, sb), 1):
        slots[(PURE, (subject,))] = {"w": 0.5 * position}
        families[(PURE, "term", (subject,))] = {
            (n,): {"c": 0.1 * n * position, "e": 0.5 * n} for n in (1, 2)
        }
    return InMemorySource(
        slots=slots,
        families=families,
        nested={(PAIR, "f", pair): nested},
        subforms={(SHIFT, ()): (shift,)},
    )


def values_of(source: DatabaseSource | InMemorySource, world: World, i: str, j: str) -> np.ndarray:
    roles = {"i": str(world.species[i]), "j": str(world.species[j])}
    bound = bind(world.decl, "qfix_pair_form", source=source, roles=roles)
    return bound.evaluate("y", T=np.array([250.0, 300.0, 444.4]))


@pytest.mark.parametrize(("i", "j"), [(QFIX_A, QFIX_B), (QFIX_B, QFIX_A)])
def test_the_database_source_gives_the_evaluator_what_the_in_memory_source_does(
    world: tuple[World, TestDatabase], conn: psycopg.Connection, i: str, j: str
) -> None:
    built, _ = world
    through_database = values_of(pair_source(conn, built.decl), built, i, j)
    through_memory = values_of(in_memory(built), built, i, j)
    np.testing.assert_array_equal(through_database, through_memory)
    assert np.all(np.isfinite(through_database))


def test_slot_values_family_rows_and_nested_sets_equal_the_in_memory_ones(
    world: tuple[World, TestDatabase], conn: psycopg.Connection
) -> None:
    built, _ = world
    source, memory = pair_source(conn, built.decl), in_memory(built)
    sa = str(built.species[QFIX_A])
    pair = canonical_pair(built)
    assert source.slot_values(PURE, (sa,)) == memory.slot_values(PURE, (sa,))
    assert source.slot_values(PAIR, pair) == memory.slot_values(PAIR, pair)
    assert source.family_rows(PURE, "term", (sa,)) == memory.family_rows(PURE, "term", (sa,))
    nested, expected = source.nested_set(PAIR, "f", pair), memory.nested_set(PAIR, "f", pair)
    assert nested is not None and expected is not None and nested.form == expected.form
    assert nested.source.slot_values("qfix_linear.core", ()) == expected.source.slot_values(
        "qfix_linear.core", ()
    )
    (shift,) = source.subform_choices(SHIFT, ())
    assert shift.form == "qfix_linear"
    assert shift.source.slot_values("qfix_linear.core", ()) == {"a": 10.0, "b": -0.01}
    assert source.subform_choices("qfix_pair_form.other", ()) == ()


def test_a_transposable_set_is_found_in_the_stored_orientation_only(
    world: tuple[World, TestDatabase], conn: psycopg.Connection
) -> None:
    built, _ = world
    source = pair_source(conn, built.decl)
    first, second = canonical_pair(built)
    assert source.slot_values(PAIR, (first, second)) == {"k": 1.75}
    assert source.slot_values(PAIR, (second, first)) is None  # the evaluator tries the swap itself


def test_a_missing_set_is_reported_as_missing_and_never_defaulted(
    world: tuple[World, TestDatabase], conn: psycopg.Connection
) -> None:
    built, _ = world
    source = pair_source(conn, built.decl)
    stranger = str(uuid.uuid4())
    sa = str(built.species[QFIX_A])
    assert source.slot_values(PURE, (stranger,)) is None
    assert source.slot_values(PURE, ("not-an-identifier",)) is None
    assert source.family_rows(PURE, "term", (stranger,)) is None
    assert source.nested_set(PAIR, "f", (sa, stranger)) is None
    assert source.default_slot_values(PURE, (stranger,)) is None
    roles = {"i": sa, "j": stranger}
    with pytest.raises(
        EvaluationRefusal, match=r"no parameter set of slot group `qfix_pair_form.pair`"
    ):
        bind(built.decl, "qfix_pair_form", source=source, roles=roles).evaluate(
            "y", T=np.array([300.0])
        )


def test_a_database_built_from_another_declaration_is_refused(
    world: tuple[World, TestDatabase], conn: psycopg.Connection
) -> None:
    with pytest.raises(SourceError, match="rebuild it with `tk build`"):
        DatabaseSource(conn, real_declaration(), [])


def test_a_redirect_is_followed_and_a_withheld_slot_is_absent(
    world: tuple[World, TestDatabase], conn: psycopg.Connection
) -> None:
    built, _ = world
    redirect = DatabaseSource(conn, built.decl, [parameterization(conn, "redirect")])
    assert redirect.slot_values(STATEFUL, ()) == {"a": 2.0, "b": 5.0}
    withheld = DatabaseSource(conn, built.decl, [parameterization(conn, "withheld")])
    assert withheld.slot_values(STATEFUL, ()) == {"b": 7.0}
    T = np.array([300.0])
    want = 2.0 + 5.0 * T
    got = bind(built.decl, "qfix_stateful", source=redirect).evaluate("y", T=T)
    np.testing.assert_allclose(got, want, rtol=1e-15)
    with pytest.raises(EvaluationRefusal, match="holds no value for slot `a`"):
        bind(built.decl, "qfix_stateful", source=withheld).evaluate("y", T=T)


def test_the_first_parameterization_that_holds_a_set_answers(
    world: tuple[World, TestDatabase], conn: psycopg.Connection
) -> None:
    built, _ = world
    base, redirect = parameterization(conn, "base"), parameterization(conn, "redirect")
    assert DatabaseSource(conn, built.decl, [redirect, base]).slot_values(STATEFUL, ())["b"] == 5.0  # type: ignore[index]
    assert DatabaseSource(conn, built.decl, [base, redirect]).slot_values(STATEFUL, ())["b"] == 3.0  # type: ignore[index]
    assert DatabaseSource(conn, built.decl, []).slot_values(STATEFUL, ()) is None


SATURATION = "vapor_pressure_exp_series_tau.pure"


def test_several_occurrences_for_a_subject_are_refused_until_one_is_chosen(
    world: tuple[World, TestDatabase], conn: psycopg.Connection
) -> None:
    built, _ = world
    twice, single = parameterization(conn, "sat_twice"), parameterization(conn, "sat")
    sa, sb = str(built.species[QFIX_A]), str(built.species[QFIX_B])
    source = DatabaseSource(conn, built.decl, [twice])
    with pytest.raises(
        AmbiguousOccurrence, match=r"holds 2 sets for the subject \(" + sa + r"\).*occurrences 1, 2"
    ) as caught:
        source.slot_values(SATURATION, (sa,))
    assert SATURATION in str(caught.value) and "occurrence = <n>" in str(caught.value)
    assert isinstance(caught.value, SourceError)
    # a subject with one occurrence needs no choice, but a read that touches the other still refuses
    assert DatabaseSource(conn, built.decl, [twice]).slot_values(SATURATION, (sb,)) is not None
    with pytest.raises(AmbiguousOccurrence):
        DatabaseSource(conn, built.decl, [twice]).prefetch(SATURATION, [(sa,), (sb,)])
    # the stated occurrence picks that set; a subject without it is missing, never another occurrence
    for occurrence in (1, 2):
        chosen = DatabaseSource(conn, built.decl, [twice], occurrences={twice: occurrence})
        assert chosen.set_id(SATURATION, (sa,)) == built.repeated[f"sp-a#{occurrence}"]
    assert (
        DatabaseSource(conn, built.decl, [twice], occurrences={twice: 1}).set_id(SATURATION, (sb,))
        == built.repeated["sp-b#1"]
    )
    second = DatabaseSource(conn, built.decl, [twice], occurrences={twice: 2})
    assert second.slot_values(SATURATION, (sb,)) is None
    assert second.slot_values(SATURATION, (sa,))["p_r"] == pytest.approx(5.0e6 * 1.01)  # type: ignore[index]
    # an occurrence applies to its own parameterization only; the next one answers where it has none
    fallback = DatabaseSource(conn, built.decl, [twice, single], occurrences={twice: 2})
    assert fallback.set_id(SATURATION, (sb,)) == built.sets[QFIX_B]
    # a parameterization that answers first hides the ambiguity of a later one
    assert (
        DatabaseSource(conn, built.decl, [single, twice]).set_id(SATURATION, (sa,))
        == built.sets[QFIX_A]
    )


class Counting:
    """A connection that counts the statements run through `execute`."""

    def __init__(self, inner: psycopg.Connection) -> None:
        self.inner = inner
        self.statements = 0

    def execute(self, *args: object, **kwargs: object) -> psycopg.Cursor:
        self.statements += 1
        return self.inner.execute(*args, **kwargs)  # type: ignore[arg-type]


def test_reads_are_bounded_by_the_subject_not_by_the_slots(
    world: tuple[World, TestDatabase], conn: psycopg.Connection
) -> None:
    built, _ = world
    counted = Counting(conn)
    source = DatabaseSource(counted, built.decl, [parameterization(conn, "qfix")])  # type: ignore[arg-type]
    before = counted.statements
    sa = str(built.species[QFIX_A])
    source.slot_values(PURE, (sa,))
    source.slot_values(PURE, (sa,))
    assert counted.statements - before == 1  # one row holds every slot, and it is remembered
    source.family_rows(PURE, "term", (sa,))
    source.family_rows(PURE, "term", (sa,))
    assert counted.statements - before == 2  # one more for the one family

    def prefetched(subjects: list[str]) -> int:
        fresh = Counting(conn)
        other = DatabaseSource(fresh, built.decl, [parameterization(conn, "qfix")])  # type: ignore[arg-type]
        start = fresh.statements
        other.prefetch(PURE, [(s,) for s in subjects])
        for subject in subjects:
            assert other.slot_values(PURE, (subject,)) is not None
            assert other.family_rows(PURE, "term", (subject,)) is not None
        return fresh.statements - start

    sb = str(built.species[QFIX_B])
    assert prefetched([sa]) == prefetched([sa, sb]) == 2
