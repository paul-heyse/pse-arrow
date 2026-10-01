# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Values are stored as the source asserted them (meta-model section 4.3).

The canonical writer keeps the subjects in the canonical order and records the `arrangement` the
values were asserted for; it never transforms a number. A parameter source applies the group's
rule on reading, once, and returns the stored numbers unchanged when the order asked for is the
order asserted. The tests write sets through the writer, build them into a database and read
them through both sources and the evaluator, against independent NumPy calculations.
"""

from __future__ import annotations

import itertools
import uuid
from collections.abc import Iterator
from dataclasses import dataclass
from pathlib import Path

import numpy as np
import psycopg
import pytest

from build_support import fingerprint, inputs_of, write_source
from mapping_support import origin, writer
from qualify_support import fixture_declaration
from thermo_knowledge import db, transposition
from thermo_knowledge.build import build_database
from thermo_knowledge.canonical.writer import CanonicalWriter, FamilyRow
from thermo_knowledge.declaration import Declaration
from thermo_knowledge.expression.evaluate import bind
from thermo_knowledge.expression.parameters import InMemorySource
from thermo_knowledge.qualify.source import DatabaseSource
from thermo_knowledge.testing import TestDatabase

RATIO = "qfix_ratio_form.pair"
SERIES = "qfix_series_form.pair"
MARGULES = "qfix_margules_form.pair"
TRIPLE = "qfix_triple_form.triple"
SYMMETRIC = "qfix_pair_form.pair"

BETA = 0.9403
"""A published number whose reciprocal, inverted again, is not the number."""
TERMS = {(0,): 1.5, (1,): 2.5, (2,): 3.5, (3,): 4.5}
H0, H1 = 2.0, 0.5
T = np.array([250.0, 300.0, 444.4])


def test_the_published_number_is_not_recoverable_from_its_reciprocal() -> None:
    assert 1.0 / (1.0 / BETA) != BETA


@dataclass
class Built:
    decl: Declaration
    database: TestDatabase
    parameterization: uuid.UUID
    members: list[uuid.UUID]
    """Three species in canonical order."""


def groups(decl: Declaration) -> dict[str, transposition.Owner]:
    return {group.qualified: group for group in decl.slot_groups}


@pytest.fixture(scope="module")
def built(tmp_path_factory: pytest.TempPathFactory) -> Iterator[Built]:
    decl = fixture_declaration(tmp_path_factory.mktemp("orientation-declaration"))
    canonical = tmp_path_factory.mktemp("orientation-canonical")
    made: dict[str, object] = {}

    def fill(w: CanonicalWriter) -> None:
        members = sorted(
            w.kind(
                "species", {"canonical_key": key, "label": key}, origins=[origin(f"a.json#/{n}")]
            )
            for n, key in enumerate("ABC")
        )
        param = w.kind(
            "parameterization",
            {"key": "k", "revision": "r", "title": "t", "coherence": "independent_records"},
            origins=[origin("a.json#/0", "fitted")],
        )
        made.update(members=members, parameterization=param)
        low, middle, high = members
        fitted = origin("a.json#/9", "fitted")
        # asserted for the swapped order, the canonical order asserted for another pair
        for subjects, r in (([high, low], BETA), ([low, middle], 4.0)):
            w.parameter_set(
                parameterization=param,
                slot_group=RATIO,
                subjects=subjects,
                slots={"r": r},
                origins=[fitted],
            )
        w.parameter_set(
            parameterization=param,
            slot_group=SERIES,
            subjects=[high, low],
            slots={},
            families={"term": [FamilyRow({"order": k[0]}, {"c": c}) for k, c in TERMS.items()]},
            origins=[fitted],
        )
        w.parameter_set(
            parameterization=param,
            slot_group=MARGULES,
            subjects=[high, low],
            slots={"h0": H0, "h1": H1},
            origins=[fitted],
        )
        w.parameter_set(
            parameterization=param,
            slot_group=TRIPLE,
            subjects=[high, low, middle],
            slots={"v": 2.0},
            origins=[fitted],
        )

    write_source(canonical, "src", fill, decl=decl, declaration=fingerprint(decl))
    with TestDatabase() as database:
        build_database(database.url, decl, inputs_of(canonical))
        yield Built(decl, database, made["parameterization"], made["members"])  # type: ignore[arg-type]


@pytest.fixture
def conn(built: Built) -> Iterator[psycopg.Connection]:
    connection = db.connect(built.database.url)
    connection.execute("SELECT 1")  # an open transaction: what a test changes is rolled back
    try:
        yield connection
    finally:
        connection.rollback()
        connection.close()


def source(built: Built, conn: psycopg.Connection) -> DatabaseSource:
    return DatabaseSource(conn, built.decl, [built.parameterization])


def asserted_and_other(built: Built) -> tuple[tuple[str, str], tuple[str, str]]:
    low, _, high = (str(member) for member in built.members)
    return (high, low), (low, high)


# -- what is stored ---------------------------------------------------------------------------


def test_a_reciprocal_pair_is_stored_as_asserted_with_its_arrangement(
    built: Built, conn: psycopg.Connection
) -> None:
    low, middle, high = built.members
    rows = conn.execute(
        "SELECT i, j, r, arrangement FROM param.qfix_ratio_form__pair ORDER BY arrangement"
    ).fetchall()
    assert rows == [(low, middle, 4.0, 0), (low, high, BETA, 1)]


def test_a_parity_family_and_a_linear_group_are_stored_as_asserted(
    built: Built, conn: psycopg.Connection
) -> None:
    low, _, high = built.members
    assert conn.execute(
        'SELECT h.i, h.j, h.arrangement, t."order", t.c FROM param.qfix_series_form__pair h '
        'JOIN param.qfix_series_form__pair__term t ON t.set_id = h.id ORDER BY t."order"'
    ).fetchall() == [(low, high, 1, k[0], c) for k, c in TERMS.items()]
    assert conn.execute(
        "SELECT i, j, h0, h1, arrangement FROM param.qfix_margules_form__pair"
    ).fetchall() == [(low, high, H0, H1, 1)]


def test_a_permutation_group_records_the_arrangement_and_a_symmetric_group_has_none(
    built: Built, conn: psycopg.Connection
) -> None:
    low, middle, high = built.members
    canonical = (low, middle, high)
    asserted = (high, low, middle)
    number = transposition.arrangement_of(groups(built.decl)[TRIPLE], asserted, canonical)
    assert number != 0
    assert conn.execute(
        "SELECT a, b, c, v, arrangement FROM param.qfix_triple_form__triple"
    ).fetchall() == [(*canonical, 2.0, number)]
    columns = {
        row[0]: {
            name
            for (name,) in conn.execute(
                "SELECT column_name FROM information_schema.columns "
                "WHERE table_schema = 'param' AND table_name = %s",
                (row[0],),
            )
        }
        for row in [("qfix_pair_form__pair",), ("qfix_ratio_form__pair",)]
    }
    assert "arrangement" not in columns["qfix_pair_form__pair"]
    assert "arrangement" in columns["qfix_ratio_form__pair"]


# -- reading ----------------------------------------------------------------------------------


def hexes(values: dict[str, float]) -> dict[str, str]:
    return {name: number.hex() for name, number in values.items()}


def test_a_reciprocal_value_reads_back_bitwise_in_the_asserted_order_and_inverted_in_the_other(
    built: Built, conn: psycopg.Connection
) -> None:
    asserted, other = asserted_and_other(built)
    found = source(built, conn)
    same = found.slot_values(RATIO, asserted)
    assert same is not None and hexes(same) == {"r": BETA.hex()}
    swapped = found.slot_values(RATIO, other)
    assert swapped == {"r": 1.0 / BETA}
    # the pair asserted in its canonical order reads back as stored there and inverted the other way
    low, middle = str(built.members[0]), str(built.members[1])
    assert found.slot_values(RATIO, (low, middle)) == {"r": 4.0}
    assert found.slot_values(RATIO, (middle, low)) == {"r": 0.25}


def test_parity_rows_change_sign_only_when_read_in_the_other_order(
    built: Built, conn: psycopg.Connection
) -> None:
    asserted, other = asserted_and_other(built)
    found = source(built, conn)
    same = found.family_rows(SERIES, "term", asserted)
    assert same is not None
    assert {key: row["c"].hex() for key, row in same.items()} == {
        key: c.hex() for key, c in TERMS.items()
    }
    swapped = found.family_rows(SERIES, "term", other)
    assert swapped is not None
    assert {key: row["c"] for key, row in swapped.items()} == {
        (0,): 1.5,
        (1,): -2.5,
        (2,): 3.5,
        (3,): -4.5,
    }


def test_a_linear_group_reads_back_as_asserted_and_as_the_matrix_image_in_the_other_order(
    built: Built, conn: psycopg.Connection
) -> None:
    asserted, other = asserted_and_other(built)
    found = source(built, conn)
    assert found.slot_values(MARGULES, asserted) == {"h0": H0, "h1": H1}
    matrix = np.array([[1.0, 1.0], [0.0, -1.0]])  # (h0, h1) -> (h0 + h1, -h1)
    expected = matrix @ np.array([H0, H1])
    swapped = found.slot_values(MARGULES, other)
    assert swapped is not None
    assert [swapped["h0"], swapped["h1"]] == expected.tolist()


def test_a_permutation_group_reads_the_same_numbers_in_every_order(
    built: Built, conn: psycopg.Connection
) -> None:
    found = source(built, conn)
    for order in itertools.permutations(str(member) for member in built.members):
        assert found.slot_values(TRIPLE, order) == {"v": 2.0}


def test_the_in_memory_source_reads_what_the_database_source_reads(
    built: Built, conn: psycopg.Connection
) -> None:
    asserted, other = asserted_and_other(built)
    memory = InMemorySource(
        slots={
            (RATIO, asserted): {"r": BETA},
            (MARGULES, asserted): {"h0": H0, "h1": H1},
        },
        families={(SERIES, "term", asserted): {k: {"c": c} for k, c in TERMS.items()}},
        declaration=built.decl,
    )
    found = source(built, conn)
    for order in (asserted, other):
        assert memory.slot_values(RATIO, order) == found.slot_values(RATIO, order)
        assert memory.slot_values(MARGULES, order) == found.slot_values(MARGULES, order)
        assert memory.family_rows(SERIES, "term", order) == found.family_rows(SERIES, "term", order)


def test_each_rule_is_applied_exactly_once_by_the_evaluator(
    built: Built, conn: psycopg.Connection
) -> None:
    asserted, other = asserted_and_other(built)
    found = source(built, conn)

    def evaluate(form: str, order: tuple[str, str]) -> np.ndarray:
        roles = {"i": order[0], "j": order[1]}
        return bind(built.decl, form, source=found, roles=roles).evaluate("y", T=T)

    def close(got: np.ndarray, want: np.ndarray) -> None:
        np.testing.assert_allclose(got, want, rtol=1e-14, atol=0.0)

    close(evaluate("qfix_ratio_form", asserted), BETA * T)
    close(evaluate("qfix_ratio_form", other), T / BETA)  # applied once: not BETA * T again
    series = np.array([c for c in TERMS.values()])
    powers = np.array([T**n for n in range(4)])
    close(evaluate("qfix_series_form", asserted), series @ powers)
    close(evaluate("qfix_series_form", other), (series * [1, -1, 1, -1]) @ powers)
    close(evaluate("qfix_margules_form", asserted), H0 + H1 * T)
    close(evaluate("qfix_margules_form", other), (H0 + H1) - H1 * T)
    for order in itertools.permutations(str(member) for member in built.members):
        roles = dict(zip("abc", order, strict=True))
        bound = bind(built.decl, "qfix_triple_form", source=found, roles=roles)
        close(bound.evaluate("y", T=T), np.full_like(T, 2.0))


# -- the orientation is an identity decision, the numbers are not -------------------------------


def test_an_identity_change_that_flips_the_canonical_order_changes_no_stored_value() -> None:
    decl = _decl_cache()
    observed: list[tuple[tuple[uuid.UUID, uuid.UUID], float, int]] = []
    for exchanged in (False, True):
        w = writer(decl)
        first = w.kind("species", {"canonical_key": "P", "label": "P"}, origins=[origin()])
        second = w.kind(
            "species", {"canonical_key": "Q", "label": "Q"}, origins=[origin("a.json#/1")]
        )
        # the source asserts the ratio of its first entity to its second; an identity decision
        # that exchanges the two identifiers is the same fixture with the two exchanged
        subjects = [second, first] if exchanged else [first, second]
        w.parameter_set(
            parameterization=w.kind(
                "parameterization",
                {"key": "k", "revision": "r", "title": "t", "coherence": "independent_records"},
                origins=[origin("a.json#/0", "fitted")],
            ),
            slot_group=RATIO,
            subjects=subjects,
            slots={"r": BETA},
            origins=[origin("a.json#/2", "fitted")],
        )
        (row,) = w.tables()["param.qfix_ratio_form__pair"].to_pylist()
        observed.append(((row["i"], row["j"]), row["r"], row["arrangement"]))
    (columns_a, value_a, arrangement_a), (columns_b, value_b, arrangement_b) = observed
    assert columns_a == columns_b, "the canonical order does not depend on the order asserted"
    assert value_a.hex() == value_b.hex() == BETA.hex(), "every stored slot value is unchanged"
    assert {arrangement_a, arrangement_b} == {0, 1}, "only the arrangement flips"


_DECL: list[Declaration] = []


def _decl_cache() -> Declaration:
    if not _DECL:
        import tempfile

        _DECL.append(fixture_declaration(Path(tempfile.mkdtemp(prefix="tk-orientation-"))))
    return _DECL[0]


# -- the arrangement functions ------------------------------------------------------------------


def test_the_arrangement_of_a_permutation_group_and_the_order_it_names_are_inverse() -> None:
    group = groups(_decl_cache())[TRIPLE]
    canonical = ("a", "b", "c")
    numbers = []
    for asserted in itertools.permutations(canonical):
        number = transposition.arrangement_of(group, asserted, canonical)
        assert transposition.asserted_order(group, canonical, number) == asserted
        numbers.append(number)
    assert sorted(numbers) == list(range(transposition.arrangement_count(group)))
    assert transposition.arrangement_count(group) == 6


def test_the_declared_permutations_are_numbered_first_in_declared_order() -> None:
    group = groups(_decl_cache())[TRIPLE]
    rule = group.transposition
    assert rule is not None
    numbered = transposition.group_arrangements(rule)
    assert numbered[:2] == [("b", "a", "c"), ("a", "c", "b")]
    assert len(numbered) == 5


def test_a_two_role_rule_has_the_canonical_and_the_swapped_arrangement() -> None:
    group = groups(_decl_cache())[RATIO]
    assert transposition.arrangement_count(group) == 2
    assert transposition.arrangement_of(group, ("a", "b"), ("a", "b")) == 0
    assert transposition.arrangement_of(group, ("b", "a"), ("a", "b")) == 1
    assert transposition.asserted_order(group, ("a", "b"), 1) == ("b", "a")
    assert not transposition.stores_arrangement(groups(_decl_cache())[SYMMETRIC])
