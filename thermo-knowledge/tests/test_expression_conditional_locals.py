# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""A conditional local can be the test of another conditional (expressions.md section 5): the
comparison of a conditional expression with something else is stated as the predicate that selects
each branch, so the condition stays boolean structure and the compiled function is the size of the
flattened form's. Each numerical test compares the evaluator with a calculation written here
directly with NumPy."""

from __future__ import annotations

import time

import numpy as np
import pytest
import sympy
from expression_support import scenario

from thermo_knowledge.expression.evaluate import bind
from thermo_knowledge.expression.lowering import lower_relation, substitute
from thermo_knowledge.expression.parameters import InMemorySource

LIMITS = dict(T_hi=900.0, T_mid=500.0, p_b=2.0e6)


def small(form: str):  # noqa: ANN201
    source = InMemorySource(slots={(f"{form}.limits", ("x",)): LIMITS})
    return bind(scenario("conditional_locals"), form, source=source, roles={"i": "x"})


def small_grid() -> tuple[np.ndarray, np.ndarray]:
    """Points in every branch and on every boundary: the limits themselves and the doubles
    either side of the temperature limits, and the boundary pressure itself."""
    temperatures = [300.0, 499.9, 500.0, float(np.nextafter(500.0, 1e9)), 700.0]
    temperatures += [899.9, 900.0, float(np.nextafter(900.0, 1e9)), 1200.0]
    pressures = [1.0e6, float(np.nextafter(2.0e6, 0.0)), 2.0e6, float(np.nextafter(2.0e6, 1e9)), 3.0e6]
    T, p = np.meshgrid(temperatures, pressures)
    return T.ravel(), p.ravel()


def small_reference(T: np.ndarray, p: np.ndarray) -> dict[str, np.ndarray]:
    region = np.where(T > 900.0, 5, np.where(T > 500.0, np.where(p < 2.0e6, 2, 3), 1))
    y1 = np.exp(T / 500.0)
    y2 = T / 900.0 + p / 2.0e6
    y3 = np.log(T / 500.0) * p / 2.0e6
    y5 = np.sqrt(T / 900.0)
    y = np.select([region == 5, region == 3, region == 2], [y5, y3, y2], y1)
    return {
        "region": region.astype(float),
        "y": y,
        "other": np.where(region != 2, 1.0, 0.0),
        "band": np.where((region > 1) & (region <= 3), 1.0, 0.0),
    }


@pytest.mark.parametrize("form", ["nested_local", "flat"])
@pytest.mark.parametrize("output", ["region", "y", "other", "band"])
def test_a_conditional_local_tested_by_another_conditional_gives_the_flattened_values(
    form: str, output: str
) -> None:
    T, p = small_grid()
    want = small_reference(T, p)[output]
    got = small(form).evaluate(output, T=T, p=p)
    np.testing.assert_allclose(got, want, rtol=1e-13, atol=0.0)


def test_the_branches_are_chosen_on_both_sides_of_every_boundary() -> None:
    T = np.array([499.9, 500.0, float(np.nextafter(500.0, 1e9)), 900.0, float(np.nextafter(900.0, 1e9))])
    p = np.full(T.shape, 1.0e6)
    assert small("nested_local").evaluate("region", T=T, p=p).tolist() == [1.0, 1.0, 2.0, 2.0, 5.0]
    T = np.full(3, 700.0)
    p = np.array([float(np.nextafter(2.0e6, 0.0)), 2.0e6, float(np.nextafter(2.0e6, 1e9))])
    assert small("nested_local").evaluate("region", T=T, p=p).tolist() == [2.0, 3.0, 3.0]


def test_no_condition_of_the_compiled_expression_holds_a_conditional_expression() -> None:
    expression = small("nested_local").expression("y")
    assert not expression.has(sympy.ITE)
    for piece in expression.atoms(sympy.Piecewise):
        for _, condition in piece.args:
            assert not condition.has(sympy.Piecewise)


# -- the three-level nesting of a regional formulation -------------------------------------------

REGIONAL = dict(T_13=623.15, T_23=863.15, T_25=1073.15, p_b=1.0e6)
# the saturation pressure of a regional formulation: a quartic of a ratio with a square root
SATURATION = {
    f"n{k}": n
    for k, n in enumerate(
        [1096.63, 188677.0, 10.736, -9157.73, -4623823.0, 18.782, -2022.39, 577807.0, -0.27024, 732.31],
        start=1,
    )
}


def regional(form: str):  # noqa: ANN201
    source = InMemorySource(
        slots={
            (f"{form}.limits", ("x",)): REGIONAL,
            (f"{form}.sat", ("x",)): SATURATION,
        }
    )
    return bind(scenario("conditional_locals"), form, source=source, roles={"i": "x"})


def saturation(T: np.ndarray) -> np.ndarray:
    n = SATURATION
    theta = T + n["n9"] / (T - n["n10"])
    A = theta**2 + n["n1"] * theta + n["n2"]
    B = n["n3"] * theta**2 + n["n4"] * theta + n["n5"]
    C = n["n6"] * theta**2 + n["n7"] * theta + n["n8"]
    return REGIONAL["p_b"] * (2 * C / (-B + np.sqrt(B**2 - 4 * A * C))) ** 4


def boundary(T: np.ndarray) -> np.ndarray:
    x = T / REGIONAL["T_13"]
    return REGIONAL["p_b"] * (1 + 0.5 * x + 0.1 * x**2)


def regional_grid() -> tuple[np.ndarray, np.ndarray]:
    """Every region, and each side of the saturation pressure and of the boundary pressure."""
    points = []
    for T in (400.0, 599.0, 623.15):  # region 1 or 2 by the saturation pressure
        sat = float(saturation(np.array(T)))
        points += [(T, sat * 0.99), (T, sat * 1.01)]
    for T in (float(np.nextafter(623.15, 1e9)), 700.0, 863.15):  # region 2 or 3 by the boundary
        bnd = float(boundary(np.array(T)))
        points += [(T, bnd * 0.99), (T, bnd * 1.01)]
    points += [(float(np.nextafter(863.15, 1e9)), 5.0e6), (1073.15, 5.0e6)]  # region 2
    points += [(float(np.nextafter(1073.15, 1e9)), 5.0e6), (1200.0, 1.0e5)]  # region 5
    T, p = (np.array(v) for v in zip(*points, strict=True))
    return T, p


def regional_reference(T: np.ndarray, p: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    x = T / REGIONAL["T_13"]
    region = np.select(
        [T > 1073.15, T > 863.15, (T > 623.15) & (p < boundary(T)), T > 623.15, p > saturation(T)],
        [5, 2, 2, 3, 1],
        2,
    )
    v1 = p / REGIONAL["p_b"] + x
    v2 = np.exp(-x) * p / REGIONAL["p_b"]
    v3 = np.log(x) + p / boundary(T)
    v5 = np.sqrt(x) * p / REGIONAL["p_b"]
    y = np.select([region == 5, region == 2, region == 3], [v5, v2, v3], v1)
    return region.astype(float), y


def test_three_levels_of_conditional_compile_and_evaluate_within_a_small_time_budget() -> None:
    T, p = regional_grid()
    started = time.perf_counter()
    bound = regional("three_levels")
    region = bound.evaluate("region", T=T, p=p)
    y = bound.evaluate("y", T=T, p=p)
    elapsed = time.perf_counter() - started
    # Flattened, the same two outputs take a fraction of a second. The condition the test of the
    # local used to leave in the expression was simplified algebraically (the relation of the
    # saturation pressure was cancelled and expanded) and did not finish in minutes.
    assert elapsed < 10.0, elapsed
    want_region, want_y = regional_reference(T, p)
    assert region.tolist() == want_region.tolist()
    np.testing.assert_allclose(y, want_y, rtol=1e-13, atol=0.0)
    assert {int(r) for r in region} == {1, 2, 3, 5}  # every region is reached


def test_three_levels_of_conditional_give_the_values_of_the_flattened_form() -> None:
    T, p = regional_grid()
    nested, flat = regional("three_levels"), regional("three_levels_flat")
    for output in ("region", "y"):
        np.testing.assert_allclose(
            nested.evaluate(output, T=T, p=p), flat.evaluate(output, T=T, p=p), rtol=1e-13, atol=0.0
        )
    assert not nested.expression("y").has(sympy.ITE)


# -- the lowering itself ---------------------------------------------------------------------------

T_SYMBOL = sympy.Symbol("T", real=True)
P_SYMBOL = sympy.Symbol("p", real=True)
OPERATORS = [sympy.Lt, sympy.Le, sympy.Gt, sympy.Ge, sympy.Eq, sympy.Ne]


def numeric(expr: sympy.Basic, T: np.ndarray, p: np.ndarray) -> np.ndarray:
    return np.asarray(sympy.lambdify([T_SYMBOL, P_SYMBOL], expr, modules="numpy", cse=False)(T, p))


@pytest.mark.parametrize("relation", OPERATORS)
def test_a_comparison_with_a_conditional_expression_selects_as_numpy_compares_it(
    relation: type[sympy.Basic],
) -> None:
    # the last branch has no condition that always holds: where none holds (T = 0.5) the value is NaN
    pieces = sympy.Piecewise(
        (sympy.Integer(1), T_SYMBOL > 2),
        (P_SYMBOL * 2, sympy.Eq(T_SYMBOL, 1)),
        (sympy.Integer(3), T_SYMBOL < 0),
    )
    T = np.array([3.0, 1.0, -1.0, 0.5, 3.0, 1.0, 0.5])
    p = np.array([0.0, 1.0, 5.0, 2.0, 1.0, 2.0, 3.0])
    for left, right in ((pieces, sympy.Integer(2)), (sympy.Integer(2), pieces), (pieces, P_SYMBOL)):
        lowered = lower_relation(relation, left, right)
        assert not lowered.has(sympy.Piecewise)
        want = numeric(relation(left, right, evaluate=False), T, p)
        got = numeric(lowered, T, p)
        assert got.tolist() == want.tolist(), (relation, left, right)


def test_a_conditional_expression_substituted_for_a_symbol_is_compared_as_a_predicate() -> None:
    r = sympy.Symbol("r", real=True)
    chosen = sympy.Piecewise((1, T_SYMBOL > 1), (P_SYMBOL, True))
    outer = sympy.Piecewise((10 * P_SYMBOL, sympy.Eq(r, 1)), (T_SYMBOL, r < P_SYMBOL), (0, True))
    replaced = substitute(outer, {r: chosen})
    assert not replaced.has(sympy.ITE)
    for _, condition in replaced.atoms(sympy.Piecewise).pop().args:
        assert not condition.has(sympy.Piecewise)
    T = np.array([2.0, 0.0, 0.0, 2.0])
    p = np.array([5.0, 1.0, 3.0, 0.5])
    want = np.where(
        np.select([T > 1, True], [1.0, p]) == 1.0,
        10 * p,
        np.where(np.select([T > 1, True], [1.0, p]) < p, T, 0.0),
    )
    assert numeric(replaced, T, p).tolist() == want.tolist()
