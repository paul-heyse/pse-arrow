# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Implicit blocks (expressions.md sections 3 to 5): root selection for a cubic equation of
state, association site fractions, derivatives by the implicit-function theorem and the
refusals of a solve that has no answer. Each numerical test compares the evaluator with a
calculation written here directly with NumPy."""

from __future__ import annotations

import numpy as np
import pytest
from expression_support import scenario

from thermo_knowledge.expression.evaluate import EvaluationRefusal, bind
from thermo_knowledge.expression.parameters import FormChoice, InMemorySource

R = 8.314462618
PROPANE = dict(T_c=369.83, p_c=4.248e6, omega=0.152)


def close(got: np.ndarray, want: np.ndarray, rtol: float = 1e-12) -> None:
    np.testing.assert_allclose(got, want, rtol=rtol, atol=0.0)


# -- the molar volume of a cubic equation of state -------------------------------------------------


def srk(T: float, p: float) -> tuple[float, float, np.ndarray]:
    """a, b and the real roots of the cubic in v, found here with NumPy."""
    m = 0.48508 + 1.55171 * PROPANE["omega"] - 0.15613 * PROPANE["omega"] ** 2
    alpha = (1 + m * (1 - np.sqrt(T / PROPANE["T_c"]))) ** 2
    a = 0.42748 * R**2 * PROPANE["T_c"] ** 2 / PROPANE["p_c"] * alpha
    b = 0.08664 * R * PROPANE["T_c"] / PROPANE["p_c"]
    # p (v - b) v (v + b) - R T v (v + b) + a (v - b), expanded by hand
    roots = np.roots([p, -R * T, a - R * T * b - p * b * b, -a * b])
    real = np.sort(roots[np.abs(roots.imag) < 1e-12 * np.abs(roots.real)].real)
    return a, b, real[(real > b) & (real < R * T / p + b)]


def log_fugacity_coefficient(T: float, p: float, v: float) -> float:
    a, b, _ = srk(T, p)
    Z = p * v / (R * T)
    A, B = a * p / (R * T) ** 2, b * p / (R * T)
    return Z - 1 - np.log(Z - B) - A / B * np.log(1 + B / Z)


def cubic_bound(form: str):  # noqa: ANN201
    source = InMemorySource(
        slots={
            (f"{form}.core", ()): dict(omega_a=0.42748, omega_b=0.08664, R=R),
            (f"{form}.pure", ("propane",)): PROPANE,
        }
    )
    return bind(scenario("implicit_cubic"), form, source=source, roles={"i": "propane"})


# three real roots (subcritical, between the spinodals), then one (supercritical)
SUBCRITICAL = [(300.0, 0.8e6), (300.0, 1.0e6), (300.0, 1.2e6), (320.0, 1.5e6), (250.0, 2.0e5)]
SUPERCRITICAL = [(450.0, 3.0e6), (400.0, 8.0e6), (500.0, 1.0e5)]


def grid(points: list[tuple[float, float]]) -> tuple[np.ndarray, np.ndarray]:
    return np.array([T for T, _ in points]), np.array([p for _, p in points])


def test_the_test_points_have_three_real_roots_below_the_critical_temperature_and_one_above() -> (
    None
):
    assert all(len(srk(T, p)[2]) == 3 for T, p in SUBCRITICAL)
    assert all(len(srk(T, p)[2]) == 1 for T, p in SUPERCRITICAL)


def test_smallest_is_the_liquid_like_root() -> None:
    T, p = grid(SUBCRITICAL)
    want = np.array([srk(t, q)[2][0] for t, q in SUBCRITICAL])
    close(cubic_bound("srk_smallest").evaluate("v", T=T, p=p), want, rtol=1e-10)


def test_largest_is_the_vapour_like_root() -> None:
    T, p = grid(SUBCRITICAL)
    want = np.array([srk(t, q)[2][-1] for t, q in SUBCRITICAL])
    close(cubic_bound("srk_largest").evaluate("v", T=T, p=p), want, rtol=1e-10)


@pytest.mark.parametrize("form", ["srk_smallest", "srk_largest", "srk_unique", "srk_stable"])
def test_a_supercritical_point_has_one_real_root_whatever_the_selection(form: str) -> None:
    T, p = grid(SUPERCRITICAL)
    want = np.array([srk(t, q)[2][0] for t, q in SUPERCRITICAL])
    close(cubic_bound(form).evaluate("v", T=T, p=p), want, rtol=1e-10)


def test_the_root_satisfies_the_equation_of_state() -> None:
    T, p = grid(SUBCRITICAL)
    a, b = zip(*((srk(t, q)[0], srk(t, q)[1]) for t, q in SUBCRITICAL))
    v = cubic_bound("srk_largest").evaluate("v", T=T, p=p)
    close(R * T / (v - np.array(b)) - np.array(a) / (v * (v + np.array(b))), p, rtol=1e-9)


def test_by_picks_the_root_of_lower_gibbs_energy() -> None:
    # Below the vapour pressure (about 1 MPa at 300 K) the vapour is stable, above it the liquid.
    T, p = grid(SUBCRITICAL[:3])
    roots = [srk(t, q)[2] for t, q in SUBCRITICAL[:3]]
    chosen = []
    for (t, q), real in zip(SUBCRITICAL[:3], roots):
        energy = [log_fugacity_coefficient(t, q, r) for r in real]
        chosen.append(real[int(np.argmin(energy))])
    assert chosen[0] == roots[0][-1] and chosen[2] == roots[2][0]  # the test discriminates
    close(cubic_bound("srk_stable").evaluate("v", T=T, p=p), np.array(chosen), rtol=1e-10)


def test_unique_refuses_where_there_are_several_roots_and_names_them() -> None:
    with pytest.raises(EvaluationRefusal, match=r"srk_unique.*volume.*T=300.*3 roots"):
        cubic_bound("srk_unique").evaluate("v", T=np.array([300.0]), p=np.array([1.0e6]))


# -- derivatives by the implicit-function theorem --------------------------------------------------


@pytest.mark.parametrize("form", ["srk_smallest", "srk_largest", "srk_stable"])
def test_the_derivative_of_the_volume_equals_a_finite_difference_of_the_re_solved_volume(
    form: str,
) -> None:
    bound = cubic_bound(form)
    T, p = grid(SUBCRITICAL[:3] + SUPERCRITICAL[:2])
    volume = lambda t, q: bound.evaluate("v", T=t, p=q)  # noqa: E731
    dp, dT = 1.0e-4 * p, 1.0e-4 * T
    fd_p = (volume(T, p + dp) - volume(T, p - dp)) / (2 * dp)
    fd_T = (volume(T + dT, p) - volume(T - dT, p)) / (2 * dT)
    close(bound.evaluate("dv_dp", T=T, p=p), fd_p, rtol=1e-6)
    close(bound.evaluate("dv_dT", T=T, p=p), fd_T, rtol=1e-6)


def test_the_derivative_is_not_the_one_that_freezes_the_unknown() -> None:
    bound = cubic_bound("srk_largest")
    T, p = np.array([300.0]), np.array([0.8e6])
    # freezing v would give d v / d p = 0: the output is the implicit-function derivative
    assert bound.evaluate("dv_dp", T=T, p=p)[0] < -1e-12


def test_the_implicit_derivative_matches_the_closed_form_of_the_equation_of_state() -> None:
    # F(v, T, p) = p - P(v, T) = 0, so dv/dp = -F_p / F_v with F_p = 1 and F_v = -dP/dv
    bound = cubic_bound("srk_largest")
    T, p = 300.0, 0.8e6
    a, b, roots = srk(T, p)
    v = roots[-1]
    F_v = R * T / (v - b) ** 2 - a * (2 * v + b) / (v * (v + b)) ** 2
    close(
        bound.evaluate("dv_dp", T=np.array([T]), p=np.array([p])), np.array([-1.0 / F_v]), rtol=1e-9
    )


# -- association site fractions --------------------------------------------------------------------


def association_bound(sites: list[str], delta: dict[tuple[str, str], float]):  # noqa: ANN201
    slots = {}
    for a in sites:
        for b in sites:
            if (a, b) in delta:
                slots[("site_fractions.pair", (a, b))] = {"delta": delta[(a, b)]}
            elif (b, a) in delta:
                continue  # held in the other orientation
            else:
                slots[("site_fractions.pair", (a, b))] = {"delta": 0.0}
    return bind(
        scenario("implicit_association"),
        "site_fractions",
        source=InMemorySource(slots=slots),
        sets={"sites": sites},
    )


def fixed_point(
    rho: float, x: dict[str, float], delta: dict[tuple[str, str], float], sites: list[str]
) -> dict[str, float]:
    """Wertheim's site fractions by damped successive substitution, written here."""

    def strength(a: str, b: str) -> float:
        return delta.get((a, b), delta.get((b, a), 0.0))

    X = {a: 0.5 for a in sites}
    for _ in range(5000):
        new = {
            a: 1.0 / (1.0 + rho * sum(x[b] * X[b] * strength(a, b) for b in sites)) for a in sites
        }
        if max(abs(new[a] - X[a]) for a in sites) < 1e-15:
            X = new
            break
        X = {a: 0.5 * X[a] + 0.5 * new[a] for a in sites}
    return X


def association_reference(
    rhos: np.ndarray, x: dict[str, float], delta: dict[tuple[str, str], float], sites: list[str]
) -> tuple[np.ndarray, np.ndarray]:
    a_assoc, bonded = [], []
    for rho in rhos:
        X = fixed_point(float(rho), x, delta, sites)
        a_assoc.append(sum(x[a] * (np.log(X[a]) - X[a] / 2 + 0.5) for a in sites))
        bonded.append(sum(x[a] * (1 - X[a]) for a in sites))
    return np.array(a_assoc), np.array(bonded)


TWO_SITE = (["A", "B"], {("A", "B"): 2.0e-4}, {"A": 1.0, "B": 1.0})
FOUR_SITE = (
    ["A", "B", "C", "D"],
    {
        ("A", "C"): 1.5e-4,
        ("A", "D"): 1.5e-4,
        ("B", "C"): 1.5e-4,
        ("B", "D"): 1.5e-4,
        ("A", "B"): 2.0e-5,
    },
    {"A": 1.0, "B": 1.0, "C": 0.6, "D": 0.4},
)
RHO = np.array([100.0, 800.0, 3000.0, 9000.0])


@pytest.mark.parametrize("scheme", [TWO_SITE, FOUR_SITE], ids=["two-site", "four-site"])
def test_association_site_fractions_match_a_fixed_point_iteration(scheme) -> None:  # noqa: ANN001
    sites, delta, x = scheme
    bound = association_bound(sites, delta)
    a_assoc, bonded = association_reference(RHO, x, delta, sites)
    close(bound.evaluate("a_assoc", rho=RHO, x=x), a_assoc, rtol=1e-9)
    close(bound.evaluate("bonded", rho=RHO, x=x), bonded, rtol=1e-9)


@pytest.mark.parametrize("scheme", [TWO_SITE, FOUR_SITE], ids=["two-site", "four-site"])
def test_the_derivative_through_a_coupled_system_equals_a_finite_difference_of_the_re_solved_output(
    scheme,
) -> None:  # noqa: ANN001
    sites, delta, x = scheme
    bound = association_bound(sites, delta)
    h = 1.0e-4 * RHO
    up = bound.evaluate("a_assoc", rho=RHO + h, x=x)
    down = bound.evaluate("a_assoc", rho=RHO - h, x=x)
    close(bound.evaluate("da_drho", rho=RHO, x=x), (up - down) / (2 * h), rtol=1e-6)


def test_the_derivative_also_matches_one_made_from_the_reference_iteration() -> None:
    sites, delta, x = FOUR_SITE
    h = 1.0e-3 * RHO
    up = association_reference(RHO + h, x, delta, sites)[0]
    down = association_reference(RHO - h, x, delta, sites)[0]
    close(
        association_bound(sites, delta).evaluate("da_drho", rho=RHO, x=x),
        (up - down) / (2 * h),
        rtol=1e-5,
    )


@pytest.mark.parametrize("scheme", [TWO_SITE, FOUR_SITE], ids=["two-site", "four-site"])
def test_the_derivative_with_respect_to_an_indexed_argument_runs_through_the_unknowns(
    scheme,
) -> None:  # noqa: ANN001
    sites, delta, x = scheme
    bound = association_bound(sites, delta)
    h = 1.0e-5
    moved = lambda step: {a: value + step for a, value in x.items()}  # noqa: E731
    up = bound.evaluate("a_assoc", rho=RHO, x=moved(h))
    down = bound.evaluate("a_assoc", rho=RHO, x=moved(-h))
    close(bound.evaluate("da_dx", rho=RHO, x=x), (up - down) / (2 * h), rtol=1e-6)


def test_without_association_every_site_is_free() -> None:
    sites, _, x = TWO_SITE
    bound = association_bound(sites, {})
    close(bound.evaluate("a_assoc", rho=RHO, x=x), np.zeros(4), rtol=0.0)
    close(bound.evaluate("bonded", rho=RHO, x=x), np.zeros(4), rtol=0.0)


def test_a_missing_site_composition_is_refused_before_anything_is_solved() -> None:
    sites, delta, _ = TWO_SITE
    with pytest.raises(EvaluationRefusal, match=r"x\[B\]"):
        association_bound(sites, delta).evaluate("a_assoc", rho=RHO, x={"A": 1.0})


# -- residuals that are not polynomials: a scan of the interval -------------------------------------


def scanned(form: str):  # noqa: ANN201
    return bind(scenario("implicit_scan"), form, source=InMemorySource())


def test_a_root_of_a_residual_that_is_not_a_polynomial_is_found_by_scanning() -> None:
    u = np.array([0.1, 1.0, 20.0])
    close(scanned("exponential").evaluate("value", u=u), np.log(u), rtol=1e-12)


def test_the_roots_of_a_sine_are_selected_by_rule() -> None:
    u = np.array([0.5, 0.9])
    first = np.arcsin(u)
    second = np.pi - first
    third = 2 * np.pi + first  # 6.81 for u = 0.5, beyond 7 for u = 0.9
    close(scanned("sine_smallest").evaluate("value", u=u), first, rtol=1e-12)
    close(
        scanned("sine_largest").evaluate("value", u=u), np.array([third[0], second[1]]), rtol=1e-12
    )
    # cos is least on the falling side: pi - asin(u), whichever other roots there are
    close(scanned("sine_by_cosine").evaluate("value", u=u), second, rtol=1e-12)


def test_unique_refuses_the_three_roots_of_the_sine_and_names_them() -> None:
    with pytest.raises(EvaluationRefusal, match=r"sine_unique.*u=0\.5.*3 roots.*0\.523598775598"):
        scanned("sine_unique").evaluate("value", u=np.array([0.5]))


def test_a_sign_change_across_a_pole_is_not_a_root() -> None:
    with pytest.raises(EvaluationRefusal, match=r"implicit block `root`.*u=0\.3.*no root"):
        scanned("pole").evaluate("value", u=np.array([0.3]))


def test_two_roots_near_a_small_lower_bound_are_told_apart_by_the_geometric_scan() -> None:
    u = np.array([1.0e-4, 3.0e-5])
    close(scanned("geometric_grid").evaluate("value", u=u), 2 * u, rtol=1e-10)


# -- an implicit block in a form called through a sub-form slot -----------------------------------


def compressibility_bound():  # noqa: ANN201
    callee = InMemorySource(
        slots={
            ("srk_largest.core", ()): dict(omega_a=0.42748, omega_b=0.08664, R=R),
            ("srk_largest.pure", ("propane",)): PROPANE,
        }
    )
    source = InMemorySource(
        slots={("srk_compressibility.core", ()): dict(R=R)},
        subforms={
            ("srk_compressibility.volume", ("propane",)): (FormChoice("srk_largest", callee),)
        },
    )
    return bind(
        scenario("implicit_cubic"), "srk_compressibility", source=source, roles={"i": "propane"}
    )


def test_a_caller_sees_the_unknown_of_the_callees_block_solved_at_its_own_arguments() -> None:
    T, p = grid(SUBCRITICAL[:3] + SUPERCRITICAL)
    want = np.array([srk(t, q)[2][-1] * q / (R * t) for t, q in zip(T, p)])
    close(compressibility_bound().evaluate("Z", T=T, p=p), want, rtol=1e-10)


def test_the_derivative_through_a_callees_block_equals_a_finite_difference() -> None:
    bound = compressibility_bound()
    T, p = grid(SUBCRITICAL[:3] + SUPERCRITICAL)
    dp = 1.0e-4 * p
    fd = (bound.evaluate("Z", T=T, p=p + dp) - bound.evaluate("Z", T=T, p=p - dp)) / (2 * dp)
    close(bound.evaluate("dZ_dp", T=T, p=p), fd, rtol=1e-6)


def test_a_refusal_inside_a_callees_block_names_the_callee_and_the_callers_point() -> None:
    bound = compressibility_bound()
    with pytest.raises(
        EvaluationRefusal, match=r"form `srk_largest`, implicit block `volume`, at .*T=300"
    ):
        bound.evaluate("Z", T=np.array([300.0]), p=np.array([-1.0]))


# -- a solve that has no answer is a refusal, never a NaN -------------------------------------------


def refusal(form: str, u: float, output: str = "value") -> EvaluationRefusal:
    bound = bind(scenario("implicit_refusals"), form, source=InMemorySource())
    with pytest.raises(EvaluationRefusal) as caught:
        bound.evaluate(output, u=np.array([u]))
    return caught.value


def test_a_polynomial_without_a_root_in_the_bounds_names_form_block_and_point() -> None:
    error = refusal("polynomial_without_a_root", 1.0)
    text = str(error)
    assert "form `polynomial_without_a_root`" in text and "implicit block `root`" in text
    assert "u=1" in text and "no root" in text and "[-2, 2]" in text


def test_a_scanned_residual_without_a_root_is_refused_too() -> None:
    assert "no root" in str(refusal("scanned_without_a_root", 1.0))


def test_a_root_outside_the_bounds_is_not_returned() -> None:
    # x - 2 + u = 0 has its root at 2 - u: inside [0, 1] only for u in [1, 2]
    bound = bind(scenario("implicit_refusals"), "root_outside_the_bounds", source=InMemorySource())
    close(bound.evaluate("value", u=np.array([1.25, 2.0])), np.array([0.75, 0.0]))
    text = str(refusal("root_outside_the_bounds", 0.0))
    assert "no root" in text and "u=0" in text and "[0, 1]" in text


def test_unique_refuses_two_roots_and_smallest_takes_the_lower() -> None:
    text = str(refusal("two_roots_but_unique", 0.25))
    assert "unique" in text and "2 roots" in text
    bound = bind(scenario("implicit_refusals"), "two_roots_smallest", source=InMemorySource())
    close(bound.evaluate("value", u=np.array([0.25, 1.0])), np.array([-0.5, -1.0]), rtol=1e-12)


def test_a_newton_type_solve_that_cannot_converge_is_refused() -> None:
    text = str(refusal("unbounded_without_a_root", 1.0))
    assert "did not converge" in text and "implicit block `root`" in text and "u=1" in text


def test_a_bounded_system_whose_residuals_vanish_nowhere_in_the_box_is_refused() -> None:
    text = str(refusal("bounded_system_without_a_root", 0.0))
    assert "did not converge" in text and "implicit block `system`" in text


def test_a_bounded_system_with_a_root_in_the_box_is_solved() -> None:
    bound = bind(
        scenario("implicit_refusals"), "bounded_system_without_a_root", source=InMemorySource()
    )
    close(bound.evaluate("value", u=np.array([1.25, 1.5])), np.array([1.5, 1.0]))


def test_an_empty_interval_is_refused_naming_both_bounds() -> None:
    text = str(refusal("empty_interval", 3.0))
    assert "empty" in text and "u=3" in text
