# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Sub-form calls through contracts with sets and indexed arguments (expressions.md sections 2
and 3): a cubic equation of state whose mixing rule is a sub-form over the components. Each
numerical test compares the evaluator with a calculation written here directly with NumPy."""

from __future__ import annotations

import numpy as np
import pytest
from expression_support import scenario

from thermo_knowledge.expression.evaluate import EvaluationRefusal, bind
from thermo_knowledge.expression.parameters import FormChoice, InMemorySource

R = 8.314462618
NAMES = ["ethane", "methane", "propane"]
CRITICAL = {
    "methane": dict(T_c=190.56, p_c=4.599e6, omega=0.011),
    "ethane": dict(T_c=305.32, p_c=4.872e6, omega=0.099),
    "propane": dict(T_c=369.83, p_c=4.248e6, omega=0.152),
}
CONSTANTS = dict(omega_a=0.42748, omega_b=0.08664, R=R)
# Held in the canonical orientation: the subject with the smaller identifier first.
KIJ = {("ethane", "methane"): 0.012, ("ethane", "propane"): 0.008, ("methane", "propane"): 0.021}
TAU = {
    ("ethane", "methane"): 0.8,
    ("methane", "ethane"): -0.3,
    ("ethane", "propane"): 1.1,
    ("propane", "ethane"): 0.2,
    ("methane", "propane"): -0.6,
    ("propane", "methane"): 0.45,
}
ALPHA = {pair: 0.25 + 0.05 * n for n, pair in enumerate(TAU)}
ALPHA_DIAGONAL = 0.3


def close(got: np.ndarray, want: np.ndarray, rtol: float = 1e-12) -> None:
    np.testing.assert_allclose(got, want, rtol=rtol, atol=0.0)


def values() -> tuple[np.ndarray, np.ndarray, dict[str, np.ndarray]]:
    T = np.array([250.0, 300.0, 350.0, 400.0])
    v = np.array([1.2e-3, 2.0e-3, 5.0e-3, 2.0e-2])
    x = {
        "ethane": np.array([0.2, 0.3, 0.1, 0.5]),
        "methane": np.array([0.5, 0.3, 0.6, 0.2]),
        "propane": np.array([0.3, 0.4, 0.3, 0.3]),
    }
    return T, v, x


# -- independent reference calculations -----------------------------------------------------------


def pure_parameters(T: np.ndarray) -> tuple[dict[str, np.ndarray], dict[str, float]]:
    a: dict[str, np.ndarray] = {}
    b: dict[str, float] = {}
    for name, c in CRITICAL.items():
        m = 0.48508 + 1.55171 * c["omega"] - 0.15613 * c["omega"] ** 2
        alpha = (1 + m * (1 - np.sqrt(T / c["T_c"]))) ** 2
        a[name] = 0.42748 * R**2 * c["T_c"] ** 2 / c["p_c"] * alpha
        b[name] = 0.08664 * R * c["T_c"] / c["p_c"]
    return a, b


def nrtl_excess(T: np.ndarray, x: dict[str, np.ndarray]) -> np.ndarray:
    def tau(i: str, j: str) -> float:
        return 0.0 if i == j else TAU[(i, j)]

    def alpha(i: str, j: str) -> float:
        return ALPHA_DIAGONAL if i == j else ALPHA[(i, j)]

    total = np.zeros_like(T)
    for i in x:
        numerator = sum(x[j] * tau(j, i) * np.exp(-alpha(j, i) * tau(j, i)) for j in x)
        denominator = sum(x[k] * np.exp(-alpha(k, i) * tau(k, i)) for k in x)
        total = total + x[i] * numerator / denominator
    return R * T * total


def pressure(T: np.ndarray, v: np.ndarray, a: np.ndarray, b: np.ndarray) -> np.ndarray:
    return R * T / (v - b) - a / (v * (v + b))


def vdw_reference(T: np.ndarray, v: np.ndarray, x: dict[str, np.ndarray]) -> np.ndarray:
    a, b = pure_parameters(T)
    a_mix = 0.0
    for i in x:
        for j in x:
            k = 0.0 if i == j else KIJ[tuple(sorted((i, j)))]
            a_mix = a_mix + x[i] * x[j] * np.sqrt(a[i] * a[j]) * (1 - k)
    b_mix = sum(x[i] * b[i] for i in x)
    return pressure(T, v, a_mix, b_mix)


def huron_vidal_reference(
    T: np.ndarray, v: np.ndarray, x: dict[str, np.ndarray], *, covolume_fractions: bool
) -> np.ndarray:
    a, b = pure_parameters(T)
    b_mix = sum(x[i] * b[i] for i in x)
    composition = {i: x[i] * b[i] / b_mix for i in x} if covolume_fractions else x
    a_mix = b_mix * (sum(x[i] * a[i] / b[i] for i in x) - nrtl_excess(T, composition) / np.log(2))
    return pressure(T, v, a_mix, b_mix)


# -- parameter sources ---------------------------------------------------------------------------


def nrtl_choice() -> FormChoice:
    slots = {("nrtl.core", ()): {"R": R}}
    for i in NAMES:
        for j in NAMES:
            slots[("nrtl.pair", (i, j))] = (
                {"tau": 0.0, "alpha": ALPHA_DIAGONAL}
                if i == j
                else {"tau": TAU[(i, j)], "alpha": ALPHA[(i, j)]}
            )
    return FormChoice("nrtl", InMemorySource(slots=slots))


def vdw_choice(pairs: dict[tuple[str, str], float]) -> FormChoice:
    return FormChoice(
        "mixing_vdw",
        InMemorySource(slots={("mixing_vdw.pair", pair): {"k": k} for pair, k in pairs.items()}),
    )


def huron_vidal_choice(form: str, excess: tuple[FormChoice, ...]) -> FormChoice:
    return FormChoice(form, InMemorySource(subforms={(f"{form}.excess", ()): excess}))


def cubic(mixing: tuple[FormChoice, ...], order: list[str] | None = None):  # noqa: ANN201
    slots = {("cubic_srk_mixed.core", ()): CONSTANTS}
    slots.update({("cubic_srk_mixed.pure", (name,)): c for name, c in CRITICAL.items()})
    source = InMemorySource(slots=slots, subforms={("cubic_srk_mixed.mixing", ()): mixing})
    return bind(
        scenario("cubic_mixing"),
        "cubic_srk_mixed",
        source=source,
        sets={"components": order or NAMES},
    )


# -- the two mixing rules ------------------------------------------------------------------------


def test_the_quadratic_van_der_waals_rule_with_a_symmetric_binary_parameter() -> None:
    T, v, x = values()
    bound = cubic((vdw_choice(KIJ),))
    close(bound.evaluate("p", T=T, v=v, x=x), vdw_reference(T, v, x))


@pytest.mark.parametrize(
    "order", [["methane", "ethane", "propane"], ["propane", "methane", "ethane"]]
)
def test_the_callee_reads_its_pair_parameters_for_the_components_the_caller_passed(
    order: list[str],
) -> None:
    T, v, x = values()
    bound = cubic((vdw_choice(KIJ),), order)
    close(bound.evaluate("p", T=T, v=v, x=x), vdw_reference(T, v, x))


def test_a_pair_the_mixing_rule_needs_but_the_source_lacks_is_refused() -> None:
    T, v, x = values()
    partial = {pair: k for pair, k in KIJ.items() if pair != ("ethane", "propane")}
    with pytest.raises(EvaluationRefusal, match=r"mixing_vdw\.pair.*ethane, propane"):
        cubic((vdw_choice(partial),)).evaluate("p", T=T, v=v, x=x)


def test_only_the_components_the_caller_passed_are_looked_up() -> None:
    T, v, x = values()
    two = {name: x[name] / (x["ethane"] + x["methane"]) for name in ("ethane", "methane")}
    bound = cubic(
        (vdw_choice({("ethane", "methane"): KIJ[("ethane", "methane")]}),), ["ethane", "methane"]
    )
    close(bound.evaluate("p", T=T, v=v, x=two), vdw_reference(T, v, two))


def test_a_huron_vidal_rule_calls_an_excess_gibbs_form_with_the_composition_passed_through() -> (
    None
):
    T, v, x = values()
    bound = cubic((huron_vidal_choice("mixing_huron_vidal", (nrtl_choice(),)),))
    close(
        bound.evaluate("p", T=T, v=v, x=x), huron_vidal_reference(T, v, x, covolume_fractions=False)
    )


@pytest.mark.parametrize("form", ["mixing_hv_remapped", "mixing_hv_inline"])
def test_a_remapped_composition_by_an_indexed_local_or_by_a_comprehension(form: str) -> None:
    T, v, x = values()
    bound = cubic((huron_vidal_choice(form, (nrtl_choice(),)),))
    got = bound.evaluate("p", T=T, v=v, x=x)
    close(got, huron_vidal_reference(T, v, x, covolume_fractions=True))
    # the remapping matters: the same rule without it gives another pressure
    assert not np.allclose(got, huron_vidal_reference(T, v, x, covolume_fractions=False), rtol=1e-6)


def test_the_two_remappings_agree_with_each_other_to_rounding() -> None:
    T, v, x = values()
    local = cubic((huron_vidal_choice("mixing_hv_remapped", (nrtl_choice(),)),))
    inline = cubic((huron_vidal_choice("mixing_hv_inline", (nrtl_choice(),)),))
    close(local.evaluate("p", T=T, v=v, x=x), inline.evaluate("p", T=T, v=v, x=x), rtol=1e-13)


def test_the_nrtl_pairs_of_the_inner_form_are_those_of_the_caller_components() -> None:
    T, v, x = values()
    reversed_order = ["propane", "methane", "ethane"]
    bound = cubic((huron_vidal_choice("mixing_huron_vidal", (nrtl_choice(),)),), reversed_order)
    close(
        bound.evaluate("p", T=T, v=v, x=x), huron_vidal_reference(T, v, x, covolume_fractions=False)
    )


def test_a_missing_excess_form_is_refused_naming_the_sub_form_slot() -> None:
    T, v, x = values()
    bound = cubic((huron_vidal_choice("mixing_huron_vidal", ()),))
    with pytest.raises(EvaluationRefusal, match=r"mixing_huron_vidal\.excess"):
        bound.evaluate("p", T=T, v=v, x=x)


def test_a_missing_mixing_choice_is_refused_naming_the_sub_form_slot() -> None:
    T, v, x = values()
    with pytest.raises(EvaluationRefusal, match=r"cubic_srk_mixed\.mixing"):
        cubic(()).evaluate("p", T=T, v=v, x=x)


def test_a_composition_without_a_value_for_a_component_is_refused() -> None:
    T, v, x = values()
    with pytest.raises(EvaluationRefusal, match=r"x\[propane\]"):
        cubic((vdw_choice(KIJ),)).evaluate(
            "p", T=T, v=v, x={k: x[k] for k in ("ethane", "methane")}
        )
