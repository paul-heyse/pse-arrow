# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The reference evaluator. Each numerical test compares it with a calculation written here
directly with NumPy; none compares the engine with its own output."""

from __future__ import annotations

import math

import numpy as np
import pytest
import scipy.integrate

from expression_support import scenario
from thermo_knowledge.expression.evaluate import EvaluationRefusal, bind
from thermo_knowledge.expression.parameters import FormChoice, InMemorySource

R = 8.314462618


def close(got: np.ndarray, want: np.ndarray, rtol: float = 1e-12) -> None:
    np.testing.assert_allclose(got, want, rtol=rtol, atol=0.0)


# -- Antoine: a pure role, a unit literal and exp ------------------------------------------------

ANTOINE = {"A": 11.6834, "B": 3816.44, "C": -46.13}


def antoine_source(form: str) -> InMemorySource:
    return InMemorySource(slots={(f"{form}.pure", ("water",)): ANTOINE})


def test_antoine_vapor_pressure_in_pascal() -> None:
    T = np.linspace(280.0, 400.0, 7)
    bound = bind(
        scenario("antoine"), "antoine", source=antoine_source("antoine"), roles={"i": "water"}
    )
    close(bound.evaluate("p_sat", T=T), np.exp(ANTOINE["A"] - ANTOINE["B"] / (T + ANTOINE["C"])))


def test_a_unit_literal_converts_to_storage_units() -> None:
    T = np.array([300.0, 350.0])
    bound = bind(
        scenario("antoine"),
        "antoine_bar",
        source=antoine_source("antoine_bar"),
        roles={"i": "water"},
    )
    want = 10 ** (ANTOINE["A"] - ANTOINE["B"] / (T + ANTOINE["C"])) * 1.0e5
    close(bound.evaluate("p_sat", T=T), want)


def test_a_derivative_through_a_local_is_a_chain_rule() -> None:
    T = np.array([300.0, 350.0, 380.0])
    bound = bind(
        scenario("antoine"),
        "antoine_slope",
        source=antoine_source("antoine_slope"),
        roles={"i": "water"},
    )
    p = np.exp(ANTOINE["A"] - ANTOINE["B"] / (T + ANTOINE["C"]))
    close(bound.evaluate("p_sat", T=T), p * ANTOINE["B"] / (T + ANTOINE["C"]) ** 2)


def test_a_missing_parameter_set_is_refused_naming_group_and_subject() -> None:
    bound = bind(scenario("antoine"), "antoine", source=InMemorySource(), roles={"i": "water"})
    with pytest.raises(EvaluationRefusal, match=r"antoine\.pure.*water"):
        bound.evaluate("p_sat", T=np.array([300.0]))


def test_binding_needs_a_subject_for_every_role() -> None:
    with pytest.raises(EvaluationRefusal, match="role"):
        bind(scenario("antoine"), "antoine", source=InMemorySource(), roles={})


def test_an_argument_without_a_value_is_refused() -> None:
    bound = bind(
        scenario("antoine"), "antoine", source=antoine_source("antoine"), roles={"i": "water"}
    )
    with pytest.raises(EvaluationRefusal, match="T"):
        bound.evaluate("p_sat")


# -- NASA-7: an interval family, selection with `at` ----------------------------------------------

LOW = dict(
    T_low=200.0,
    T_high=1000.0,
    a1=3.53,
    a2=-1.2e-3,
    a3=2.3e-6,
    a4=-1.1e-9,
    a5=3.1e-13,
    a6=-1044.0,
    a7=2.97,
)
HIGH = dict(
    T_low=1000.0,
    T_high=6000.0,
    a1=2.95,
    a2=1.4e-3,
    a3=-4.9e-7,
    a4=7.7e-11,
    a5=-5.2e-15,
    a6=-922.9,
    a7=5.87,
)


def nasa_reference(T: np.ndarray) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    def pick(name: str) -> np.ndarray:
        return np.where(T < 1000.0, LOW[name], HIGH[name])

    a1, a2, a3, a4, a5, a6, a7 = (pick(f"a{k}") for k in range(1, 8))
    cp = R * (a1 + a2 * T + a3 * T**2 + a4 * T**3 + a5 * T**4)
    h = R * T * (a1 + a2 * T / 2 + a3 * T**2 / 3 + a4 * T**3 / 4 + a5 * T**4 / 5 + a6 / T)
    s = R * (a1 * np.log(T) + a2 * T + a3 * T**2 / 2 + a4 * T**3 / 3 + a5 * T**4 / 4 + a7)
    return cp, h, s


def nasa_bound():  # noqa: ANN201
    source = InMemorySource(families={("nasa7.pure", "piece", ("n2",)): {(1,): LOW, (2,): HIGH}})
    return bind(scenario("nasa7"), "nasa7", source=source, roles={"i": "n2"})


def test_nasa7_outputs_match_a_direct_calculation_on_both_sides_of_the_breakpoint() -> None:
    bound = nasa_bound()
    T = np.array([200.0, 300.0, 999.999999, 1000.0, 1500.0, 5999.0, 6000.0])
    cp, h, s = nasa_reference(T)
    close(bound.evaluate("cp", T=T), cp)
    close(bound.evaluate("h", T=T), h)
    close(bound.evaluate("s", T=T), s)


def test_at_selects_the_upper_piece_at_the_breakpoint_and_the_closed_end_belongs_to_the_last() -> (
    None
):
    bound = nasa_bound()
    below, at, end = (np.array(v) for v in (999.9999999, 1000.0, 6000.0))
    cp_low = R * (
        LOW["a1"]
        + LOW["a2"] * 1000
        + LOW["a3"] * 1000**2
        + LOW["a4"] * 1000**3
        + LOW["a5"] * 1000**4
    )
    cp_high = R * (
        HIGH["a1"]
        + HIGH["a2"] * 1000
        + HIGH["a3"] * 1000**2
        + HIGH["a4"] * 1000**3
        + HIGH["a5"] * 1000**4
    )
    assert abs(cp_low - cp_high) > 1e-3  # the pieces differ at the breakpoint
    close(bound.evaluate("cp", T=below), cp_low, rtol=1e-6)
    close(bound.evaluate("cp", T=at), cp_high)
    assert np.isfinite(bound.evaluate("cp", T=end))


@pytest.mark.parametrize("T", [199.999, 6000.001, 50.0, 9000.0])
def test_a_temperature_outside_every_piece_is_refused(T: float) -> None:
    with pytest.raises(EvaluationRefusal, match=r"outside every piece.*nasa7\.pure\.piece.*n2"):
        nasa_bound().evaluate("cp", T=np.array([500.0, T]))


def test_overlapping_pieces_are_refused() -> None:
    overlapping = dict(HIGH, T_low=900.0)
    source = InMemorySource(
        families={("nasa7.pure", "piece", ("n2",)): {(1,): LOW, (2,): overlapping}}
    )
    bound = bind(scenario("nasa7"), "nasa7", source=source, roles={"i": "n2"})
    with pytest.raises(EvaluationRefusal, match="overlap"):
        bound.evaluate("cp", T=np.array([500.0]))


# -- A cubic equation of state for a mixture ------------------------------------------------------

CRITICAL = {
    "methane": dict(T_c=190.56, p_c=4.599e6, omega=0.011),
    "ethane": dict(T_c=305.32, p_c=4.872e6, omega=0.099),
    "propane": dict(T_c=369.83, p_c=4.248e6, omega=0.152),
}
CONSTANTS = dict(omega_a=0.42748, omega_b=0.08664, R=R)
FITTED = {"ethane": 0.72, "propane": 0.81}
# Held in the canonical orientation: the subject with the smaller identifier first.
KIJ = {("ethane", "methane"): 0.012, ("ethane", "propane"): 0.008}


def soave(Tr: np.ndarray, omega: float) -> np.ndarray:
    m = 0.48508 + 1.55171 * omega - 0.15613 * omega**2
    return (1 + m * (1 - np.sqrt(Tr))) ** 2


def srk_reference(
    T: np.ndarray,
    v: np.ndarray,
    x: dict[str, np.ndarray],
    fitted: dict[str, float],
    k: dict[frozenset[str], float],
) -> np.ndarray:
    names = list(x)
    a: dict[str, np.ndarray] = {}
    b = 0.0
    for name in names:
        c = CRITICAL[name]
        Tr = T / c["T_c"]
        alpha = (
            (1 + fitted[name] * (1 - np.sqrt(Tr))) ** 2 if name in fitted else soave(Tr, c["omega"])
        )
        a[name] = 0.42748 * R**2 * c["T_c"] ** 2 / c["p_c"] * alpha
        b = b + x[name] * 0.08664 * R * c["T_c"] / c["p_c"]
    mixture = 0.0
    for i in names:
        for j in names:
            kij = 0.0 if i == j else k.get(frozenset((i, j)), 0.0)
            mixture = mixture + x[i] * x[j] * np.sqrt(a[i] * a[j]) * (1 - kij)
    return R * T / (v - b) - mixture / (v * (v + b))


def cubic_source(
    fitted: dict[str, float], *, defaults: bool, pairs: dict[tuple[str, str], float]
) -> InMemorySource:
    slots = {("cubic_srk.core", ()): CONSTANTS}
    slots.update({("cubic_srk.pure", (name,)): values for name, values in CRITICAL.items()})
    slots.update({("cubic_srk.pair", pair): {"k": value} for pair, value in pairs.items()})
    subforms: dict[tuple[str, tuple[str, ...]], tuple[FormChoice, ...]] = {}
    for name in CRITICAL:
        if name in fitted:
            choice = FormChoice(
                "alpha_fitted",
                InMemorySource(slots={("alpha_fitted.pure", (name,)): {"c1": fitted[name]}}),
            )
        else:
            choice = FormChoice("alpha_soave", InMemorySource())
        subforms[("cubic_srk.alpha", (name,))] = (choice,)
    return InMemorySource(
        slots=slots,
        subforms=subforms,
        defaults={"cubic_srk.pair": {"k": 0.0}} if defaults else {},
        declaration=scenario("cubic"),
    )


def cubic_values() -> tuple[np.ndarray, np.ndarray, dict[str, np.ndarray]]:
    T = np.array([250.0, 300.0, 350.0, 400.0])
    v = np.array([1.2e-3, 2.0e-3, 5.0e-3, 2.0e-2])
    x = {
        "ethane": np.array([0.2, 0.3, 0.1, 0.5]),
        "methane": np.array([0.5, 0.3, 0.6, 0.2]),
        "propane": np.array([0.3, 0.4, 0.3, 0.3]),
    }
    return T, v, x


@pytest.mark.parametrize("fitted", [{}, FITTED, {"ethane": 0.72}], ids=["soave", "fitted", "mixed"])
def test_cubic_pressure_with_the_alpha_function_chosen_per_test(fitted: dict[str, float]) -> None:
    T, v, x = cubic_values()
    source = cubic_source(fitted, defaults=True, pairs=KIJ)
    bound = bind(scenario("cubic"), "cubic_srk", source=source, sets={"components": list(x)})
    want = srk_reference(T, v, x, fitted, {frozenset(pair): k for pair, k in KIJ.items()})
    close(bound.evaluate("p", T=T, v=v, x=x), want)


def test_pair_parameter_is_found_in_either_orientation_whatever_the_component_order() -> None:
    T, v, x = cubic_values()
    source = cubic_source(FITTED, defaults=True, pairs=KIJ)
    want = srk_reference(T, v, x, FITTED, {frozenset(p): k for p, k in KIJ.items()})
    for order in (["methane", "ethane", "propane"], ["propane", "methane", "ethane"]):
        bound = bind(scenario("cubic"), "cubic_srk", source=source, sets={"components": order})
        close(bound.evaluate("p", T=T, v=v, x=x), want)


def test_absent_pairs_take_the_explicit_default_and_nothing_else() -> None:
    T, v, x = cubic_values()
    source = cubic_source({}, defaults=True, pairs={("ethane", "methane"): 0.05})
    bound = bind(scenario("cubic"), "cubic_srk", source=source, sets={"components": list(x)})
    want = srk_reference(T, v, x, {}, {frozenset(("ethane", "methane")): 0.05})
    close(bound.evaluate("p", T=T, v=v, x=x), want)


def test_a_required_pair_that_is_missing_is_refused_not_zero() -> None:
    T, v, x = cubic_values()
    source = cubic_source({}, defaults=False, pairs={("ethane", "methane"): 0.05})
    bound = bind(scenario("cubic"), "cubic_srk", source=source, sets={"components": list(x)})
    with pytest.raises(EvaluationRefusal, match=r"cubic_srk\.pair.*ethane, propane"):
        bound.evaluate("p", T=T, v=v, x=x)


def test_a_missing_alpha_choice_is_refused_naming_the_slot_and_subject() -> None:
    T, v, x = cubic_values()
    source = cubic_source({}, defaults=True, pairs=KIJ)
    empty = InMemorySource(
        slots=source.slots, defaults=source.defaults, declaration=source.declaration
    )
    bound = bind(scenario("cubic"), "cubic_srk", source=empty, sets={"components": list(x)})
    with pytest.raises(EvaluationRefusal, match=r"cubic_srk\.alpha.*ethane"):
        bound.evaluate("p", T=T, v=v, x=x)


def test_the_set_members_must_be_given() -> None:
    with pytest.raises(EvaluationRefusal, match="components"):
        bind(scenario("cubic"), "cubic_srk", source=InMemorySource())


# -- A residual Helmholtz term list ---------------------------------------------------------------

POWER = {(1,): dict(n=0.52, d=1.0, t=0.25), (2,): dict(n=-0.31, d=2.0, t=1.5)}
EXPONENTIAL = {(1,): dict(n=0.21, d=2.0, t=0.5, l=1.0), (2,): dict(n=-0.08, d=3.0, t=2.0, l=2.0)}
REDUCING = dict(T_c=190.56, rho_c=10139.0, R=R)


def helmholtz_bound(choices: tuple[FormChoice, ...]):  # noqa: ANN201
    source = InMemorySource(
        slots={("helmholtz_eos.core", ()): REDUCING},
        subforms={("helmholtz_eos.residual", ()): choices},
    )
    return bind(scenario("helmholtz"), "helmholtz_eos", source=source)


def term_source(form: str, rows: dict[tuple[int, ...], dict[str, float]]) -> FormChoice:
    return FormChoice(form, InMemorySource(families={(f"{form}.core", "terms", ()): rows}))


def helmholtz_reference(
    T: np.ndarray, rho: np.ndarray, power: bool, exponential: bool
) -> np.ndarray:
    delta = rho / REDUCING["rho_c"]
    tau = REDUCING["T_c"] / T
    # d alpha_r / d delta, differentiated by hand term by term
    slope = 0.0
    if power:
        for row in POWER.values():
            slope = slope + row["n"] * row["d"] * delta ** (row["d"] - 1) * tau ** row["t"]
    if exponential:
        for row in EXPONENTIAL.values():
            damping = np.exp(-(delta ** row["l"]))
            slope = slope + row["n"] * tau ** row["t"] * damping * delta ** (row["d"] - 1) * (
                row["d"] - row["l"] * delta ** row["l"]
            )
    return rho * R * T * (1 + delta * slope)


@pytest.mark.parametrize(
    ("power", "exponential"),
    [(True, True), (True, False), (False, True)],
    ids="both power exp".split(),
)
def test_pressure_from_the_derivative_of_a_term_list(power: bool, exponential: bool) -> None:
    choices = tuple(
        choice
        for used, choice in (
            (power, term_source("term_power", POWER)),
            (exponential, term_source("term_exponential", EXPONENTIAL)),
        )
        if used
    )
    T = np.array([150.0, 190.56, 250.0, 400.0])
    rho = np.array([500.0, 6000.0, 10139.0, 15000.0])
    close(
        helmholtz_bound(choices).evaluate("p", T=T, rho=rho),
        helmholtz_reference(T, rho, power, exponential),
    )


def test_an_optional_term_list_may_be_empty_and_the_gas_is_ideal() -> None:
    T = np.array([200.0, 300.0])
    rho = np.array([10.0, 20.0])
    close(helmholtz_bound(()).evaluate("p", T=T, rho=rho), rho * R * T)


# -- NRTL and Redlich-Kister ---------------------------------------------------------------------

NAMES = ["a", "b", "c"]
TAU = {
    ("a", "b"): 0.8,
    ("b", "a"): -0.3,
    ("a", "c"): 1.1,
    ("c", "a"): 0.2,
    ("b", "c"): -0.6,
    ("c", "b"): 0.45,
}
ALPHA = {
    ("a", "b"): 0.3,
    ("b", "a"): 0.3,
    ("a", "c"): 0.25,
    ("c", "a"): 0.25,
    ("b", "c"): 0.35,
    ("c", "b"): 0.35,
}


def test_nrtl_excess_gibbs_energy_over_ordered_pairs_and_a_double_sum() -> None:
    T = np.array([290.0, 330.0, 360.0])
    x = {
        "a": np.array([0.2, 0.5, 0.1]),
        "b": np.array([0.5, 0.2, 0.3]),
        "c": np.array([0.3, 0.3, 0.6]),
    }
    slots = {("nrtl.core", ()): {"R": R}}
    for i in NAMES:
        for j in NAMES:
            slots[("nrtl.pair", (i, j))] = (
                {"tau": 0.0, "alpha": 0.3}
                if i == j
                else {"tau": TAU[(i, j)], "alpha": ALPHA[(i, j)]}
            )
    bound = bind(
        scenario("activity"), "nrtl", source=InMemorySource(slots=slots), sets={"components": NAMES}
    )
    want = np.zeros_like(T)
    for i in NAMES:
        numerator = sum(
            x[j]
            * (TAU[(j, i)] if j != i else 0.0)
            * math.exp(-(ALPHA[(j, i)] if j != i else 0.3) * (TAU[(j, i)] if j != i else 0.0))
            for j in NAMES
        )
        denominator = sum(
            x[k] * math.exp(-(ALPHA[(k, i)] if k != i else 0.3) * (TAU[(k, i)] if k != i else 0.0))
            for k in NAMES
        )
        want = want + x[i] * numerator / denominator
    close(bound.evaluate("gE", T=T, x=x), R * T * want)


def test_nrtl_pairs_are_ordered_so_the_reverse_pair_is_not_a_substitute() -> None:
    slots = {("nrtl.core", ()): {"R": R}}
    for i, j in (("a", "a"), ("b", "b"), ("a", "b")):
        slots[("nrtl.pair", (i, j))] = {"tau": 0.0 if i == j else 0.5, "alpha": 0.3}
    bound = bind(
        scenario("activity"),
        "nrtl",
        source=InMemorySource(slots=slots),
        sets={"components": ["a", "b"]},
    )
    x = {"a": np.array([0.4]), "b": np.array([0.6])}
    with pytest.raises(EvaluationRefusal, match=r"nrtl\.pair.*b, a"):
        bound.evaluate("gE", T=np.array([300.0]), x=x)


RK = {(0,): {"L": 1200.0}, (1,): {"L": -340.0}, (2,): {"L": 75.0}, (3,): {"L": 12.0}}


def rk_reference(x: np.ndarray) -> np.ndarray:
    return x * (1 - x) * sum(row["L"] * (2 * x - 1) ** k[0] for k, row in RK.items())


def rk_bound(i: str, j: str):  # noqa: ANN201
    source = InMemorySource(
        families={("redlich_kister.pair", "rk", ("a", "b")): RK}, declaration=scenario("activity")
    )
    return bind(scenario("activity"), "redlich_kister", source=source, roles={"i": i, "j": j})


def test_redlich_kister_with_the_pair_in_its_canonical_orientation() -> None:
    x = np.linspace(0.05, 0.95, 7)
    close(rk_bound("a", "b").evaluate("gE", xi=x), rk_reference(x))


def test_redlich_kister_in_the_other_orientation_equals_the_canonical_one() -> None:
    x = np.linspace(0.05, 0.95, 7)
    canonical = rk_bound("a", "b").evaluate("gE", xi=x)
    # the same binary seen from the second species: its mole fraction is 1 - x
    swapped = rk_bound("b", "a").evaluate("gE", xi=1 - x)
    close(swapped, canonical)
    close(swapped, rk_reference(x))


def test_a_forbidden_diagonal_is_refused() -> None:
    with pytest.raises(EvaluationRefusal, match="diagonal"):
        rk_bound("a", "a").evaluate("gE", xi=np.array([0.5]))


# -- A nested-set slot ---------------------------------------------------------------------------

C0, C1 = 0.5, 2.0e-3


def nested_bound(i: str, j: str):  # noqa: ANN201
    linear = FormChoice(
        "linear_in_t", InMemorySource(slots={("linear_in_t.core", ()): {"c0": C0, "c1": C1}})
    )
    source = InMemorySource(
        nested={("tau_nested.pair", "a", ("a", "b")): linear}, declaration=scenario("nested")
    )
    return bind(scenario("nested"), "tau_nested", source=source, roles={"i": i, "j": j})


@pytest.mark.parametrize(("i", "j"), [("a", "b"), ("b", "a")])
def test_a_pair_parameter_that_is_a_temperature_function_of_a_sub_form(i: str, j: str) -> None:
    T = np.array([250.0, 300.0, 450.0])
    close(nested_bound(i, j).evaluate("tau", T=T), C0 + C1 * T)


def test_a_missing_nested_set_is_refused() -> None:
    bound = bind(
        scenario("nested"), "tau_nested", source=InMemorySource(), roles={"i": "a", "j": "b"}
    )
    with pytest.raises(EvaluationRefusal, match=r"tau_nested\.pair.*a, b"):
        bound.evaluate("tau", T=np.array([300.0]))


# -- A definite integral and special functions ---------------------------------------------------


def test_a_definite_integral_with_a_closed_form() -> None:
    cp0, cp1, T_ref = 28.0, 0.004, 298.15
    source = InMemorySource(slots={("heating.core", ()): dict(cp0=cp0, cp1=cp1, T_ref=T_ref)})
    bound = bind(scenario("integral_special"), "heating", source=source)
    T = np.array([300.0, 500.0, 800.0])
    close(bound.evaluate("dh", T=T), cp0 * (T - T_ref) + cp1 / 2 * (T**2 - T_ref**2))


def test_a_definite_integral_without_a_closed_form_uses_quadrature() -> None:
    source = InMemorySource(slots={("heating.core", ()): dict(cp0=1.0, cp1=0.0, T_ref=1.0)})
    bound = bind(scenario("integral_special"), "heating", source=source)
    T = np.array([0.5, 1.0, 2.5, 7.0])
    want = np.array(
        [
            scipy.integrate.quad(
                lambda z: math.exp(math.sin(z)) / (1 + z**2), 0.0, t, epsrel=1e-13
            )[0]
            for t in T
        ]
    )
    close(bound.evaluate("dh_numeric", T=T), want, rtol=1e-9)


def special() -> object:
    return bind(scenario("integral_special"), "special", source=InMemorySource())


def test_erf_and_a_chebyshev_polynomial() -> None:
    x = np.array([-0.9, -0.3, 0.0, 0.25, 0.8, 1.0])
    bound = special()
    close(bound.evaluate("error", x=x), np.array([math.erf(v) for v in x]))  # type: ignore[attr-defined]
    close(bound.evaluate("cheb", x=x), 4 * x**3 - 3 * x)  # type: ignore[attr-defined]


def test_the_debye_function_is_the_integral_it_is_defined_by() -> None:
    x = np.array([0.1, 1.0, 2.5, 10.0])
    want = np.array(
        [
            3.0
            / v**3
            * scipy.integrate.quad(lambda t: t**3 / math.expm1(t), 0.0, v, epsrel=1e-13)[0]
            for v in x
        ]
    )
    close(special().evaluate("debye", x=x), want, rtol=1e-9)  # type: ignore[attr-defined]
    close(special().evaluate("debye", x=np.array(0.0)), np.array(1.0))  # type: ignore[attr-defined]


def test_the_expression_of_an_output_is_available_symbolically() -> None:
    bound = bind(
        scenario("antoine"), "antoine", source=antoine_source("antoine"), roles={"i": "water"}
    )
    free = bound.expression("p_sat").free_symbols
    parameters = bound.parameters
    # the argument, and one symbol for each stored value and the unit literal; no value is in it
    assert {str(s) for s in free - set(parameters)} == {"T"}
    assert sorted(parameters.values()) == sorted([*ANTOINE.values(), 1.0])
    assert set(parameters) <= free


# -- operators, functions, conditions and index arithmetic ---------------------------------------


def operators_bound(names: list[str]):  # noqa: ANN201
    return bind(
        scenario("operators"), "operators", source=InMemorySource(), sets={"components": names}
    )


T_GRID = np.array([250.0, 299.0, 350.0, 399.0, 400.0, 450.0, 500.0, 620.0])
Z = T_GRID / 1000


def test_min_max_and_a_conditional_on_temperature() -> None:
    bound = operators_bound(["a"])
    x = {"a": 0.5}
    close(bound.evaluate("clip", T=T_GRID, x=x), np.maximum(0, np.minimum(T_GRID - 300, 50)))
    close(
        bound.evaluate("branch", T=T_GRID, x=x),
        np.where(T_GRID < 400, T_GRID, 400 + (T_GRID - 400) / 2),
    )


def test_boolean_operators_and_comparisons() -> None:
    want = np.where(((T_GRID > 300) & (T_GRID < 500)) | (T_GRID == 250), 1.0, 0.0)
    close(operators_bound(["a"]).evaluate("logic", T=T_GRID, x={"a": 0.5}), want)


def test_elementary_functions() -> None:
    want = (
        np.sinh(Z)
        + np.cosh(Z)
        + np.tanh(Z)
        + np.sin(Z)
        + np.cos(Z)
        + np.arctan(Z)
        + np.log10(1 + Z**2)
        + np.abs(-Z)
    )
    close(operators_bound(["a"]).evaluate("trig", T=T_GRID, x={"a": 0.5}), want)


def test_a_sum_over_a_range_and_a_product_over_the_components() -> None:
    x = {
        "a": np.array([0.2, 0.3, 0.4]),
        "b": np.array([0.5, 0.1, 0.4]),
        "c": np.array([0.3, 0.6, 0.2]),
    }
    T = np.array([300.0, 500.0, 700.0])
    z = T / 1000
    want = sum(z**n / (n + 1) for n in range(4)) + (1 + x["a"]) * (1 + x["b"]) * (1 + x["c"])
    close(operators_bound(["a", "b", "c"]).evaluate("series", T=T, x=x), want)


def test_a_derivative_with_respect_to_an_indexed_argument() -> None:
    x = {"a": np.array([0.2, 0.3]), "b": np.array([0.5, 0.1]), "c": np.array([0.3, 0.6])}
    want = (1 + x["b"]) * (1 + x["c"]) + (1 + x["a"]) * (1 + x["c"]) + (1 + x["a"]) * (1 + x["b"])
    close(operators_bound(["a", "b", "c"]).evaluate("sens", T=np.array([300.0, 400.0]), x=x), want)


def test_a_square_root_of_an_even_dimension_is_a_dimension() -> None:
    close(operators_bound(["a"]).evaluate("root", T=T_GRID, x={"a": 1.0}), T_GRID)


def test_an_indexed_argument_without_a_value_for_a_member_is_refused() -> None:
    with pytest.raises(EvaluationRefusal, match=r"x\[b\]"):
        operators_bound(["a", "b"]).evaluate("series", T=T_GRID, x={"a": 0.5})


def test_an_unknown_argument_is_refused() -> None:
    with pytest.raises(EvaluationRefusal, match="not an argument"):
        operators_bound(["a"]).evaluate("clip", T=T_GRID, x={"a": 0.5}, y=1.0)


# -- reciprocal and permutation-group transposition ------------------------------------------------


def test_a_reciprocal_slot_inverts_when_the_pair_is_asked_for_swapped() -> None:
    source = InMemorySource(
        slots={("reciprocal_form.pair", ("a", "b")): {"r": 4.0, "s": 0.3}},
        declaration=scenario("transposition"),
    )
    T = np.array([300.0])
    held = bind(
        scenario("transposition"), "reciprocal_form", source=source, roles={"i": "a", "j": "b"}
    )
    swapped = bind(
        scenario("transposition"), "reciprocal_form", source=source, roles={"i": "b", "j": "a"}
    )
    close(held.evaluate("v", T=T), np.array([4.0 + 3.0]))
    close(swapped.evaluate("v", T=T), np.array([1 / 4.0 + 3.0]))


def test_a_permutation_group_finds_the_fact_under_an_arrangement_of_its_group_only() -> None:
    source = InMemorySource(
        slots={("permutation_form.triple", ("a", "b", "c")): {"w": 0.7}},
        declaration=scenario("transposition"),
    )
    T = np.array([300.0])

    def value(i: str, j: str, k: str) -> np.ndarray:
        bound = bind(
            scenario("transposition"),
            "permutation_form",
            source=source,
            roles={"i": i, "j": j, "k": k},
        )
        return bound.evaluate("v", T=T)

    close(value("a", "b", "c"), np.array([0.7]))
    close(value("b", "a", "c"), np.array([0.7]))
    with pytest.raises(EvaluationRefusal, match=r"permutation_form\.triple.*a, c, b"):
        value("a", "c", "b")


# -- a sub-form slot iterated per subject -----------------------------------------------------------


def term(slope: float, subject: str) -> FormChoice:
    return FormChoice(
        "linear_term", InMemorySource(slots={("linear_term.pure", (subject,)): {"c": slope}})
    )


def test_the_forms_chosen_for_a_subject_are_added() -> None:
    source = InMemorySource(
        subforms={
            ("total_form.terms", ("a",)): (term(2.0, "a"), term(0.5, "a")),
            ("total_form.terms", ("b",)): (term(3.0, "b"),),
        }
    )
    T = np.array([300.0, 500.0])
    total_a = bind(scenario("iteration"), "total_form", source=source, roles={"i": "a"})
    total_b = bind(scenario("iteration"), "total_form", source=source, roles={"i": "b"})
    total_c = bind(scenario("iteration"), "total_form", source=source, roles={"i": "c"})
    close(total_a.evaluate("amount", T=T), (2.0 + 0.5) * T)
    close(total_b.evaluate("amount", T=T), 3.0 * T)
    close(total_c.evaluate("amount", T=T), np.zeros(2))  # no form chosen: an empty sum


def test_a_conditional_on_a_subject_does_not_read_the_branch_it_does_not_take() -> None:
    # the diagonal of a symmetric, forbidden-diagonal group is never read when the branch
    # `0 if i == j else pair.k[i, j]` is static: see the cubic tests, where no diagonal row exists
    source = cubic_source({}, defaults=True, pairs={})
    T, v, x = cubic_values()
    bound = bind(scenario("cubic"), "cubic_srk", source=source, sets={"components": ["methane"]})
    value = bound.evaluate("p", T=T, v=v, x={"methane": np.ones(4)})
    assert np.all(np.isfinite(value))
