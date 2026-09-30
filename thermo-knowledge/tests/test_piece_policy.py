# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""What an evaluation does at and beyond the boundaries of the pieces of a family (plan 24, packet
TK2g, alignment item 15).

The stored pieces stay half-open intervals. A source reads them differently (pycalphad's reader:
half-open and zero outside; its model: extrapolated; Thermochimica: upper-inclusive and
extrapolated upward), so the reading is a policy of the qualification case: a boundary rule (which
side a boundary point belongs to) and an outside rule (refuse, or use the nearest piece). The
evaluator's piece lookup applies it and the stored data never changes.
"""

from __future__ import annotations

import uuid
from pathlib import Path

import numpy as np
import pytest
from expression_support import scenario

from thermo_knowledge.expression.compiled import CompileCache
from thermo_knowledge.expression.evaluate import EvaluationRefusal, PiecePolicy, bind
from thermo_knowledge.expression.parameters import InMemorySource
from thermo_knowledge.qualify import run
from thermo_knowledge.qualify.case import CaseError, load_case

R = 8.314462618
LOW = dict(T_low=200.0, T_high=1000.0, a1=3.53, a2=-1.2e-3, a3=2.3e-6, a4=-1.1e-9, a5=3.1e-13, a6=-1044.0, a7=2.97)
HIGH = dict(T_low=1000.0, T_high=6000.0, a1=2.95, a2=1.4e-3, a3=-4.9e-7, a4=7.7e-11, a5=-5.2e-15, a6=-922.9, a7=5.87)
UPPER, LOWER = "upper_piece", "lower_piece"


def cp_of(piece: dict[str, float], T: float) -> float:
    return R * (
        piece["a1"] + piece["a2"] * T + piece["a3"] * T**2 + piece["a4"] * T**3 + piece["a5"] * T**4
    )


def bound_with(pieces: dict[tuple[int, ...], dict[str, float]], policy: PiecePolicy | None, cache: CompileCache | None = None):  # noqa: ANN201
    source = InMemorySource(families={("nasa7.pure", "piece", ("n2",)): pieces})
    return bind(scenario("nasa7"), "nasa7", source=source, roles={"i": "n2"}, pieces=policy, cache=cache)


def cp(pieces: dict[tuple[int, ...], dict[str, float]], policy: PiecePolicy | None, T: float) -> float:
    return float(np.asarray(bound_with(pieces, policy).evaluate("cp", T=np.array([T]))).reshape(-1)[0])


TWO = {(1,): LOW, (2,): HIGH}


# -- the boundary rule ---------------------------------------------------------------------------


def test_a_point_at_the_common_boundary_belongs_to_the_piece_above_by_default() -> None:
    assert cp(TWO, None, 1000.0) == pytest.approx(cp_of(HIGH, 1000.0), rel=1e-13)
    assert cp(TWO, PiecePolicy(boundary=UPPER), 1000.0) == pytest.approx(cp_of(HIGH, 1000.0), rel=1e-13)


def test_a_point_at_the_common_boundary_belongs_to_the_piece_below_under_the_other_rule() -> None:
    assert abs(cp_of(LOW, 1000.0) - cp_of(HIGH, 1000.0)) > 1e-3  # the pieces differ there
    assert cp(TWO, PiecePolicy(boundary=LOWER), 1000.0) == pytest.approx(cp_of(LOW, 1000.0), rel=1e-13)


@pytest.mark.parametrize("boundary", [UPPER, LOWER])
@pytest.mark.parametrize("outside", ["refuse", "nearest"])
def test_a_point_off_the_boundary_is_in_the_same_piece_under_every_rule(boundary: str, outside: str) -> None:
    policy = PiecePolicy(boundary=boundary, outside=outside)  # type: ignore[arg-type]
    assert cp(TWO, policy, 999.0) == pytest.approx(cp_of(LOW, 999.0), rel=1e-13)
    assert cp(TWO, policy, 1001.0) == pytest.approx(cp_of(HIGH, 1001.0), rel=1e-13)


@pytest.mark.parametrize("boundary", [UPPER, LOWER])
def test_the_outermost_bounds_belong_to_the_outermost_pieces_under_either_rule(boundary: str) -> None:
    policy = PiecePolicy(boundary=boundary)  # type: ignore[arg-type]
    assert cp(TWO, policy, 200.0) == pytest.approx(cp_of(LOW, 200.0), rel=1e-13)
    assert cp(TWO, policy, 6000.0) == pytest.approx(cp_of(HIGH, 6000.0), rel=1e-13)


# -- the outside rule ----------------------------------------------------------------------------


@pytest.mark.parametrize("boundary", [UPPER, LOWER])
@pytest.mark.parametrize("T", [199.0, 6000.5, 9000.0])
def test_a_point_beyond_the_pieces_is_refused_by_default(boundary: str, T: float) -> None:
    with pytest.raises(EvaluationRefusal, match=r"outside every piece.*nasa7\.pure\.piece.*n2"):
        bound_with(TWO, PiecePolicy(boundary=boundary)).evaluate("cp", T=np.array([500.0, T]))  # type: ignore[arg-type]


@pytest.mark.parametrize("boundary", [UPPER, LOWER])
def test_a_point_above_the_top_piece_is_given_to_it_under_nearest(boundary: str) -> None:
    policy = PiecePolicy(boundary=boundary, outside="nearest")  # type: ignore[arg-type]
    assert cp(TWO, policy, 7500.0) == pytest.approx(cp_of(HIGH, 7500.0), rel=1e-13)


@pytest.mark.parametrize("boundary", [UPPER, LOWER])
def test_a_point_below_the_bottom_piece_is_given_to_it_under_nearest(boundary: str) -> None:
    policy = PiecePolicy(boundary=boundary, outside="nearest")  # type: ignore[arg-type]
    assert cp(TWO, policy, 120.0) == pytest.approx(cp_of(LOW, 120.0), rel=1e-13)


def test_the_extrapolated_and_the_refused_evaluation_agree_inside_the_pieces() -> None:
    temperatures = np.array([200.0, 450.0, 999.0, 1000.0, 2500.0, 6000.0])
    refusing = bound_with(TWO, PiecePolicy()).evaluate("cp", T=temperatures)
    extrapolating = bound_with(TWO, PiecePolicy(outside="nearest")).evaluate("cp", T=temperatures)
    np.testing.assert_array_equal(refusing, extrapolating)


# -- a gap between pieces ------------------------------------------------------------------------

GAP = {(1,): dict(LOW, T_high=900.0), (2,): dict(HIGH, T_low=1100.0)}


def test_a_point_in_a_gap_is_refused_unless_the_policy_gives_it_to_the_nearer_piece() -> None:
    with pytest.raises(EvaluationRefusal, match="outside every piece"):
        bound_with(GAP, None).evaluate("cp", T=np.array([950.0]))
    nearest = PiecePolicy(outside="nearest")
    assert cp(GAP, nearest, 950.0) == pytest.approx(cp_of(GAP[(1,)], 950.0), rel=1e-13)
    assert cp(GAP, nearest, 1050.0) == pytest.approx(cp_of(GAP[(2,)], 1050.0), rel=1e-13)


def test_the_middle_of_a_gap_follows_the_boundary_rule() -> None:
    assert cp(GAP, PiecePolicy(boundary=UPPER, outside="nearest"), 1000.0) == pytest.approx(
        cp_of(GAP[(2,)], 1000.0), rel=1e-13
    )
    assert cp(GAP, PiecePolicy(boundary=LOWER, outside="nearest"), 1000.0) == pytest.approx(
        cp_of(GAP[(1,)], 1000.0), rel=1e-13
    )


def test_a_single_piece_takes_every_point_under_nearest() -> None:
    one = {(1,): LOW}
    nearest = PiecePolicy(outside="nearest")
    assert cp(one, nearest, 50.0) == pytest.approx(cp_of(LOW, 50.0), rel=1e-13)
    assert cp(one, nearest, 5000.0) == pytest.approx(cp_of(LOW, 5000.0), rel=1e-13)


# -- the policy belongs to the evaluation, not to the data ---------------------------------------


def test_bindings_with_different_policies_share_a_cache_without_sharing_a_compilation() -> None:
    cache = CompileCache()
    default = bound_with(TWO, PiecePolicy(), cache)
    lower = bound_with(TWO, PiecePolicy(boundary=LOWER), cache)
    at_boundary = np.array([1000.0])
    assert default.evaluate("cp", T=at_boundary)[0] == pytest.approx(cp_of(HIGH, 1000.0), rel=1e-13)
    assert lower.evaluate("cp", T=at_boundary)[0] == pytest.approx(cp_of(LOW, 1000.0), rel=1e-13)


def test_the_stored_pieces_are_the_same_whatever_the_policy() -> None:
    pieces = {(1,): dict(LOW), (2,): dict(HIGH)}
    before = {key: dict(row) for key, row in pieces.items()}
    for policy in (PiecePolicy(), PiecePolicy(boundary=LOWER, outside="nearest")):
        bound_with(pieces, policy).evaluate("cp", T=np.array([1000.0, 7000.0 if policy.outside == "nearest" else 1000.0]))
    assert pieces == before


# -- the case format ----------------------------------------------------------------------------

CASE = """
doc = "a piecewise form, read under a policy"
form = "nasa7"
output = "cp"
basis = "source_library"

[parameterization]
key = "k"
revision = "r"

[subjects]
group = "nasa7.pure"

[subjects.library_key]
carrier = "src"
scope = "items"

[arguments.T]
values = [500.0, 1000.0, 7000.0]
unit = "K"

[comparison]
relative_tolerance = 1e-9

[harness]
library = "fake"
call = "anything"
{pieces}
"""


def case_file(directory: Path, pieces: str = "") -> Path:
    directory.mkdir(parents=True, exist_ok=True)
    path = directory / "case.toml"
    path.write_text(CASE.format(pieces=pieces))
    return path


def test_a_case_states_the_boundary_and_the_outside_rule(tmp_path: Path) -> None:
    case = load_case(case_file(tmp_path, '[pieces]\nboundary = "lower_piece"\noutside = "nearest"'))
    assert (case.spec.pieces.boundary, case.spec.pieces.outside) == (LOWER, "nearest")
    assert case.spec.pieces.__class__.__name__ == "PiecesSpec"


def test_a_case_that_states_no_policy_keeps_the_half_open_reading_and_refuses_beyond(tmp_path: Path) -> None:
    case = load_case(case_file(tmp_path))
    assert (case.spec.pieces.boundary, case.spec.pieces.outside) == (UPPER, "refuse")


@pytest.mark.parametrize(
    "pieces",
    ['[pieces]\nboundary = "left"', '[pieces]\noutside = "zero"', "[pieces]\nextrapolate = true"],
)
def test_a_case_refuses_a_rule_it_does_not_know(tmp_path: Path, pieces: str) -> None:
    with pytest.raises(CaseError):
        load_case(case_file(tmp_path, pieces))


def test_the_policy_changes_the_content_hash_of_the_case(tmp_path: Path) -> None:
    plain = load_case(case_file(tmp_path / "plain"))
    other = load_case(case_file(tmp_path / "other", '[pieces]\noutside = "nearest"'))
    assert plain.content_hash != other.content_hash


class _Source(InMemorySource):
    def prefetch(self, group: str, subjects: object) -> None:  # the database source reads ahead
        return None


def evaluate_case(tmp_path: Path, pieces: str) -> dict[uuid.UUID, np.ndarray | str]:
    decl = scenario("nasa7")
    case = load_case(case_file(tmp_path, pieces))
    group = next(g for g in decl.slot_groups if g.qualified == "nasa7.pure")
    subject = run.Subject(uuid.uuid4(), ("n2",), ("N2",))
    source = _Source(families={("nasa7.pure", "piece", ("n2",)): TWO})
    points = {subject.set_id: {"T": [500.0, 1000.0, 7000.0]}}
    return run.evaluate(source, decl, case, group, [subject], points).answers  # type: ignore[arg-type]


def test_a_run_evaluates_under_the_policy_of_its_case(tmp_path: Path) -> None:
    (refused,) = evaluate_case(tmp_path / "default", "").values()
    assert isinstance(refused, str) and "outside every piece" in refused
    policy = '[pieces]\nboundary = "lower_piece"\noutside = "nearest"'
    (answers,) = evaluate_case(tmp_path / "policy", policy).values()
    expected = [cp_of(LOW, 500.0), cp_of(LOW, 1000.0), cp_of(HIGH, 7000.0)]
    np.testing.assert_allclose(answers, expected, rtol=1e-13)
