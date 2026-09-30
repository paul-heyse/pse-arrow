# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""`position(t, s)`: the position a constituent array gives a species, read from the array's stored
members (expressions.md sections 3 and 5). Evaluated here over an in-memory source with the
committed Redlich-Kister form of a compound-energy array; the database-backed source is exercised
by the CALPHAD hard case."""

from __future__ import annotations

import numpy as np
import pytest
from mapping_support import real_declaration

from thermo_knowledge.expression.evaluate import EvaluationRefusal, bind
from thermo_knowledge.expression.parameters import InMemorySource

ARRAY, FE, CR = "array-1", "species-fe", "species-cr"
GROUP = "cef_redlich_kister.interaction"
L0, L1 = -15000.0, 6000.0


def source(positions: dict[tuple[str, str], tuple[int, ...]]) -> InMemorySource:
    return InMemorySource(
        families={(GROUP, "order", (ARRAY,)): {(0,): {"L": L0}, (1,): {"L": L1}}},
        positions=positions,
        declaration=real_declaration(),
    )


def energy(found: InMemorySource, order: tuple[str, str]) -> float:
    bound = bind(
        real_declaration(),
        "cef_redlich_kister",
        source=found,
        roles={"t": ARRAY},
        sets={"mixing": list(order)},
    )
    y = {FE: np.array([0.3]), CR: np.array([0.7])}
    return float(np.asarray(bound.evaluate("G", y=y, spectator=np.array([1.0]))).reshape(-1)[0])


def test_the_species_at_position_one_is_the_first_of_the_difference() -> None:
    found = energy(source({(ARRAY, FE): (1,), (ARRAY, CR): (2,)}), (FE, CR))
    assert found == pytest.approx(0.3 * 0.7 * (L0 + L1 * (0.3 - 0.7)), rel=1e-13)


def test_the_array_that_asserts_the_species_the_other_way_changes_the_sign_of_the_odd_term() -> None:
    found = energy(source({(ARRAY, CR): (1,), (ARRAY, FE): (2,)}), (FE, CR))
    assert found == pytest.approx(0.3 * 0.7 * (L0 - L1 * (0.3 - 0.7)), rel=1e-13)


def test_the_order_the_set_is_given_in_does_not_change_the_position() -> None:
    positions = {(ARRAY, FE): (1,), (ARRAY, CR): (2,)}
    assert energy(source(positions), (CR, FE)) == pytest.approx(energy(source(positions), (FE, CR)), rel=1e-13)


@pytest.mark.parametrize(
    ("positions", "message"),
    [
        ({(ARRAY, CR): (2,)}, "places it nowhere"),
        ({(ARRAY, FE): (1, 1), (ARRAY, CR): (2,)}, r"places it 2 times \(\(1, 1\)\)"),
    ],
    ids=["a species the array does not place", "a species placed on two site classes"],
)
def test_a_species_without_a_single_position_is_refused_and_named(
    positions: dict[tuple[str, str], tuple[int, ...]], message: str
) -> None:
    with pytest.raises(EvaluationRefusal, match=rf"position of species {FE} in constituent array {ARRAY} is undefined: the array {message}"):
        energy(source(positions), (FE, CR))
