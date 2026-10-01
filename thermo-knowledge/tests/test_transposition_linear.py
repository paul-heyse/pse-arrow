# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The `linear` transposition rule (meta-model section 4.3): what the loader refuses, what the
swap does to values, how the canonical writer stores a pair given in the other orientation and how
the evaluator reads a pair in either orientation."""

from __future__ import annotations

import uuid
from pathlib import Path

import numpy as np
import pytest

from declaration_support import copy_full, full_declaration
from expression_support import scenario
from mapping_support import extended_declaration, origin, writer
from thermo_knowledge import transposition
from thermo_knowledge.canonical.writer import CanonicalWriter, CompetingAssertion, ValidationError
from thermo_knowledge.declaration import Declaration, Diagnostic, load_declaration
from thermo_knowledge.expression.compiled import CompileCache
from thermo_knowledge.expression.evaluate import bind
from thermo_knowledge.expression.parameters import InMemorySource

MARGULES = [[1, 1], [0, -1]]

# -- the matrix --------------------------------------------------------------------------------


@pytest.mark.parametrize(
    "matrix",
    [
        [[1, 1], [0, -1]],  # the Margules swap
        [[1, 0], [0, 1]],
        [[0, 1], [1, 0]],
        [[-1]],
        [[0.5, 0.5], [1.5, -0.5]],
        [[0.1, 0.9], [1.1, -0.1]],  # exact on the declared decimals, not on their doubles
        [[1, 0, 0], [0, 0, 1], [0, 1, 0]],
    ],
)
def test_an_involution_applied_twice_is_the_identity(matrix: list[list[float]]) -> None:
    assert transposition.is_involution(matrix)


@pytest.mark.parametrize(
    "matrix",
    [
        [[1, 1], [0, 1]],
        [[2, 0], [0, 1]],
        [[1, 1, 0], [0, -1, 0]],  # not square
        [[1, 1], [0]],
        [[float("nan"), 0.0], [0.0, 1.0]],
    ],
)
def test_anything_else_is_not(matrix: list[list[float]]) -> None:
    assert not transposition.is_involution(matrix)


def test_the_decimal_trap_is_a_float_trap() -> None:
    """Why the check is rational: in doubles the declared matrix does not square to one."""
    m = np.array([[0.1, 0.9], [1.1, -0.1]])
    assert not np.array_equal(m @ m, np.eye(2))
    assert transposition.is_involution([[0.1, 0.9], [1.1, -0.1]])


# -- the loader --------------------------------------------------------------------------------

FORM = """
module = "zz_linear"
uses = ["physical", "identity", "framework", "vocab", "contracts", "forms"]
doc = "A linear transposition under test."

[forms.lin]
doc = "d"
implements = "binary_parameter"
completeness = "fully_declared"
status = "catalogued"

[forms.lin.slot_groups.pair]
doc = "d"
subject.i = {{ type = "species_form", doc = "d" }}
subject.j = {{ type = "species_form", doc = "d" }}
transposition = {transposition}

[forms.lin.slot_groups.pair.slots]
{slots}
"""
SCALARS = 'h0 = { type = "Scalar", doc = "d" }\nh1 = { type = "Scalar", doc = "d" }'


def load(tmp_path: Path, transposition_text: str, slots: str = SCALARS) -> list[Diagnostic]:
    tree = copy_full(tmp_path)
    (tree / "forms" / "zz_linear.toml").write_text(
        FORM.format(transposition=transposition_text, slots=slots), encoding="utf-8"
    )
    return list(load_declaration(tree / "model", tree / "forms", contract=None).diagnostics)


def rule(matrix: str, slots: str = '["h0", "h1"]', extra: str = "") -> str:
    return f'{{ rule = "linear", roles = ["i", "j"], slots = {slots}, matrix = {matrix}{extra} }}'


def refusals(tmp_path: Path, transposition_text: str, slots: str = SCALARS) -> list[str]:
    found = load(tmp_path, transposition_text, slots)
    assert all(d.code == "bad-transposition" for d in found), found
    assert all(d.construct == "forms.lin.slot_groups.pair.transposition" for d in found)
    return [d.message for d in found]


def test_a_well_formed_linear_rule_is_accepted(tmp_path: Path) -> None:
    assert load(tmp_path, rule("[[1, 1], [0, -1]]")) == []
    decl = full_declaration()
    swap = decl.forms["margules"].slot_groups[0].transposition
    assert swap is not None and swap.rule == "linear"
    assert swap.slots == ("h0", "h1") and swap.matrix == ((1, 1), (0, -1))


def test_a_rule_that_is_not_an_involution_is_refused(tmp_path: Path) -> None:
    (message,) = refusals(tmp_path, rule("[[1, 1], [0, 1]]"))
    assert "not an involution" in message and "applying it twice" in message


def test_a_matrix_that_is_not_square_over_the_slots_is_refused(tmp_path: Path) -> None:
    (message,) = refusals(tmp_path / "wide", rule("[[1, 1, 0], [0, -1, 0]]"))
    assert "square over the 2 slots" in message and "2 x 3" in message
    (message,) = refusals(tmp_path / "big", rule("[[1, 0, 0], [0, -1, 0], [0, 0, 1]]"))
    assert "square over the 2 slots" in message and "3 x 3" in message
    (message,) = refusals(tmp_path / "ragged", rule("[[1, 1], [0]]"))
    assert "square over the 2 slots" in message
    (message,) = refusals(
        tmp_path / "missing", '{ rule = "linear", roles = ["i", "j"], slots = ["h0", "h1"] }'
    )
    assert "square over the 2 slots" in message and "missing" in message


def test_slots_are_required_and_must_exist(tmp_path: Path) -> None:
    (message,) = refusals(tmp_path / "unknown", rule("[[1, 1], [0, -1]]", '["h0", "nothing"]'))
    assert "`nothing` is not a slot" in message
    (message,) = refusals(
        tmp_path / "none", '{ rule = "linear", roles = ["i", "j"], matrix = [[1]] }'
    )
    assert "names the slots its matrix multiplies" in message
    (message,) = refusals(tmp_path / "twice", rule("[[1, 0], [0, 1]]", '["h0", "h0"]'))
    assert "a slot is named twice" in message


def test_a_slot_a_linear_combination_cannot_take_is_refused(tmp_path: Path) -> None:
    stateful = 'h0 = { type = "Scalar", doc = "d" }\nh1 = { type = "Scalar", presence = "stateful", doc = "d" }'
    (message,) = refusals(tmp_path / "stateful", rule("[[1, 1], [0, -1]]"), stateful)
    assert "`h1` is stateful" in message
    enum = 'h0 = { type = "Scalar", doc = "d" }\nh1 = { type = "phase_class", doc = "d" }'
    (message,) = refusals(tmp_path / "enum", rule("[[1, 1], [0, -1]]"), enum)
    assert "`h1` is phase_class, not a quantity" in message


def test_a_row_combines_slots_of_one_dimension_only(tmp_path: Path) -> None:
    mixed = 'h0 = { type = "Scalar", doc = "d" }\nh1 = { type = "Temperature", doc = "d" }'
    (message,) = refusals(tmp_path / "mixed", rule("[[1, 1], [0, -1]]"), mixed)
    assert "the row for `h0` combines `h1`, which has another dimension" in message
    # slots of different dimensions are fine while no row combines them
    assert load(tmp_path / "apart", rule("[[1, 0], [0, -1]]"), mixed) == []
    assert load(tmp_path / "swap", rule("[[0, 1], [1, 0]]"), mixed)[0].code == "bad-transposition"


def test_matrix_and_slots_belong_to_their_rules(tmp_path: Path) -> None:
    (message,) = refusals(
        tmp_path / "symmetric", '{ rule = "symmetric", roles = ["i", "j"], matrix = [[1]] }'
    )
    assert "`matrix` belongs to `linear`" in message
    (message,) = refusals(
        tmp_path / "slots", '{ rule = "symmetric", roles = ["i", "j"], slots = ["h0"] }'
    )
    assert "`slots` belongs to `reciprocal` and `linear`" in message
    (message,) = refusals(
        tmp_path / "roles", '{ rule = "linear", roles = ["i"], slots = ["h0"], matrix = [[1]] }'
    )
    assert "names exactly two roles" in message


def test_a_relation_takes_a_linear_rule_over_its_value_columns(tmp_path: Path) -> None:
    tree = copy_full(tmp_path)
    (tree / "model" / "zz_case.toml").write_text(
        """
module = "zz_relation"
schema = "tk"
uses = ["physical", "identity", "framework", "vocab"]
doc = "A relation with a linear rule."

[relations.margules_value]
doc = "d"
provenance = "none"
absence = "optional"
transposition = { rule = "linear", roles = ["i", "j"], slots = ["h0", "h1"], matrix = [[1, 1], [0, -1]] }

[relations.margules_value.keys]
i = { type = "species", doc = "d" }
j = { type = "species", doc = "d" }

[relations.margules_value.columns]
h0 = { type = "Scalar", doc = "d" }
h1 = { type = "Scalar", doc = "d" }
""",
        encoding="utf-8",
    )
    assert load_declaration(tree / "model", tree / "forms", contract=None).diagnostics == ()


# -- swapped values ----------------------------------------------------------------------------


def group_with(decl: Declaration, form: str) -> transposition.Owner:
    return decl.forms[form].slot_groups[0]


def test_the_swap_multiplies_the_listed_slots_by_the_matrix_and_leaves_the_rest() -> None:
    decl = full_declaration()
    group = group_with(decl, "margules")
    assert transposition.swapped_values(group, {"h0": 2.0, "h1": 0.5}) == {"h0": 2.5, "h1": -0.5}
    # a value the rule does not name is carried along
    assert transposition.swapped_values(group, {"h0": 2.0, "h1": 0.5, "z": 9.0})["z"] == 9.0


def test_a_swap_applied_twice_returns_the_values() -> None:
    group = group_with(full_declaration(), "margules")
    values = {"h0": -1.3, "h1": 4.75}
    back = transposition.swapped_values(group, transposition.swapped_values(group, values))
    assert back == pytest.approx(values, rel=1e-15)
    exact = {"h0": -1.25, "h1": 4.75}
    assert transposition.swapped_values(group, transposition.swapped_values(group, exact)) == exact


def test_reciprocal_and_symmetric_groups_take_the_same_entry_point() -> None:
    decl = full_declaration()
    ratio = group_with(decl, "ratio")
    assert transposition.swapped_values(ratio, {"r": 4.0}) == {"r": 0.25}
    assert transposition.swapped_values(group_with(decl, "kij"), {"k": 3.0}) == {"k": 3.0}


# -- the canonical writer ----------------------------------------------------------------------

GROUP = "fixture_margules.pair"


@pytest.fixture(scope="module")
def extended(tmp_path_factory: pytest.TempPathFactory) -> Declaration:
    return extended_declaration(tmp_path_factory.mktemp("extended"))


def pair(w: CanonicalWriter) -> tuple[uuid.UUID, uuid.UUID, uuid.UUID]:
    a = w.kind("species", {"canonical_key": "A", "label": "A"}, origins=[origin("a.json#/0")])
    b = w.kind("species", {"canonical_key": "B", "label": "B"}, origins=[origin("a.json#/1")])
    low, high = sorted((a, b))
    parameterization = w.kind(
        "parameterization",
        {"key": "k", "revision": "r", "title": "t", "coherence": "independent_records"},
        origins=[origin("a.json#/0", "fitted")],
    )
    return low, high, parameterization


def write(
    w: CanonicalWriter,
    param: uuid.UUID,
    subjects: list[uuid.UUID],
    h0: float,
    h1: float,
    locator: str,
) -> uuid.UUID:
    return w.parameter_set(
        parameterization=param,
        slot_group=GROUP,
        subjects=subjects,
        slots={"h0": h0, "h1": h1, "other": 3.0},
        origins=[origin(locator, "fitted")],
    )


def test_a_pair_given_in_the_other_orientation_is_stored_as_asserted(
    extended: Declaration,
) -> None:
    w = writer(extended)
    low, high, param = pair(w)
    given = write(w, param, [high, low], 2.0, 0.5, "a.json#/3")
    (row,) = w.tables()["param.fixture_margules__pair"].to_pylist()
    assert row["id"] == given and (row["i"], row["j"]) == (low, high)
    assert (row["h0"], row["h1"]) == (2.0, 0.5), "the asserted numbers, not (h0 + h1, -h1)"
    assert row["other"] == 3.0
    assert row["arrangement"] == 1, "asserted for the swapped order"


def test_both_orientations_of_one_fact_are_one_set_with_their_own_assertions(
    extended: Declaration,
) -> None:
    w = writer(extended)
    low, high, param = pair(w)
    first = write(w, param, [low, high], 2.5, -0.5, "a.json#/2")
    with pytest.raises(CompetingAssertion):
        write(w, param, [high, low], 2.0, 0.5, "a.json#/3")
    assert w.rows("param.fixture_margules__pair") == 1
    assert first == write(w, param, [low, high], 2.5, -0.5, "a.json#/4")


def test_a_pair_given_in_the_canonical_orientation_is_stored_as_given(
    extended: Declaration,
) -> None:
    w = writer(extended)
    low, high, param = pair(w)
    write(w, param, [low, high], 1.25, -4.0, "a.json#/2")
    (row,) = w.tables()["param.fixture_margules__pair"].to_pylist()
    assert (row["h0"], row["h1"], row["arrangement"]) == (1.25, -4.0, 0)


def test_the_diagonal_is_refused_for_a_linear_group(extended: Declaration) -> None:
    w = writer(extended)
    low, _, param = pair(w)
    before = w.counts()
    with pytest.raises(ValidationError, match="forbids the diagonal"):
        write(w, param, [low, low], 1.0, 1.0, "a.json#/5")
    assert w.counts() == before


# -- the evaluator -----------------------------------------------------------------------------


def excess(h0: float, h1: float, first: str, second: str, xi: np.ndarray) -> np.ndarray:
    declaration = scenario("transposition")
    source = InMemorySource(
        slots={("margules_form.pair", ("a", "b")): {"h0": h0, "h1": h1}}, declaration=declaration
    )
    bound = bind(declaration, "margules_form", source=source, roles={"i": first, "j": second})
    return bound.evaluate("gE", xi=xi)


def test_both_orientations_give_the_excess_energy_of_the_same_mixture() -> None:
    h0, h1 = 1.3, -0.45
    xa = np.linspace(0.05, 0.95, 7)
    xb = 1.0 - xa
    # the independent calculation: g^E / RT = x_a x_b (h0 + h1 x_b), a mixture of a and b
    expected = xa * xb * (h0 + h1 * xb)
    held = excess(h0, h1, "a", "b", xa)
    swapped = excess(h0, h1, "b", "a", xb)
    np.testing.assert_allclose(held, expected, rtol=1e-14, atol=0)
    np.testing.assert_allclose(swapped, expected, rtol=1e-14, atol=0)
    np.testing.assert_allclose(swapped, held, rtol=1e-14, atol=0)


def test_the_swap_is_an_operation_on_the_values_and_not_on_the_structure() -> None:
    cache = CompileCache()
    declaration = scenario("transposition")
    source = InMemorySource(
        slots={("margules_form.pair", ("a", "b")): {"h0": 1.25, "h1": -0.5}},
        declaration=declaration,
    )
    held = bind(
        declaration, "margules_form", source=source, roles={"i": "a", "j": "b"}, cache=cache
    )
    swapped = bind(
        declaration, "margules_form", source=source, roles={"i": "b", "j": "a"}, cache=cache
    )
    xi = np.array([0.5])
    held.evaluate("gE", xi=xi)
    swapped.evaluate("gE", xi=xi)
    assert cache.compilations == 1, "both orientations are one expression"
    assert sorted(held.parameters.values()) == [-0.5, 1.25]
    assert sorted(swapped.parameters.values()) == [0.5, 0.75]  # (h0 + h1, -h1) is what was passed
