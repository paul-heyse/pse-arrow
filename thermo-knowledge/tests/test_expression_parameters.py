# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Stored values are parameters of the compiled function, not constants in it (expressions.md
section 5): the formula is evaluated as written, and what is compiled is the structure of an
expansion, shared by every binding of that structure. Each numerical test compares the evaluator
with a calculation written here directly with NumPy."""

from __future__ import annotations

import numpy as np
import pytest
import sympy
from expression_support import scenario

from thermo_knowledge.expression.compiled import CompileCache
from thermo_knowledge.expression.evaluate import bind
from thermo_knowledge.expression.parameters import FormChoice, InMemorySource

R = 8.314462618
T_GRID = np.array([250.0, 299.0, 350.0, 450.0])


def close(got: np.ndarray, want: np.ndarray, rtol: float = 1e-13) -> None:
    np.testing.assert_allclose(got, want, rtol=rtol, atol=0.0)


# -- the defect: the formula as written, where theta is a few units in the last place -------------

CURVES = [
    # Tr, p, n, t: a reduced temperature of water's size, a coefficient and an exponent that
    # are not exactly representable, and a pair whose product Tr * n is 1 after rounding but
    # not exactly (3 * fl(1/3) is 1 - 2**-54: a fold in higher precision keeps the difference)
    (647.096, 2.2064e7, -7.85, 1 / 3),
    (3.0, 1.0e5, 1 / 3, 2 / 3),
    (512.64, 8.1e6, -8.3, 1 / 3),
]


def below(top: float, steps: int) -> np.ndarray:
    """`top` and the doubles below it, `steps` of them: the last place of `T` where theta is 1e-16."""
    found = [top]
    for _ in range(steps - 1):
        found.append(float(np.nextafter(found[-1], 0.0)))
    return np.array(found)


def as_written(T: np.ndarray, Tr: float, p: float, n: float, t: float) -> np.ndarray:
    return p * np.exp(Tr / T * n * (1 - T / Tr) ** t)


@pytest.mark.parametrize(("Tr", "p", "n", "t"), CURVES)
def test_the_formula_is_evaluated_as_written_where_theta_is_a_few_ulp(
    Tr: float, p: float, n: float, t: float
) -> None:
    source = InMemorySource(
        slots={("reduced_exponential.pure", ("x",)): dict(p=p, Tr=Tr, n=n, t=t)}
    )
    bound = bind(
        scenario("parameter_structure"), "reduced_exponential", source=source, roles={"i": "x"}
    )
    T = np.concatenate([below(float(np.nextafter(Tr, 0.0)), 6), [0.9 * Tr, 0.5 * Tr]])
    want = as_written(T, Tr, p, n, t)
    got = bound.evaluate("p", T=T)
    # Four units in the last place of the result. Theta and its power are the same operations on
    # the same doubles in both. What differs is the order of the product Tr / T * n * theta**t
    # (SymPy orders the factors), each rounding of which is half an ulp of a number below 1 in
    # size here, so the exponent moves by about 2e-16 of itself and the result by at most one
    # ulp of rounding, never the 1e-5 that a different theta gives.
    bound_ulp = 4 * np.spacing(want)
    assert np.all(np.abs(got - want) <= bound_ulp), (got - want) / np.spacing(want)
    # The test is sensitive: theta computed from a pre-combined constant, T * (1 / Tr), as a
    # folding evaluator did, is thousands of ulp away at one of these points for the first two.
    folded = p * np.exp(Tr / T * n * (1 - T * (1.0 / Tr)) ** t)
    worst = float(np.max(np.abs(folded - want) / np.spacing(want)))
    if (Tr, t) != (512.64, 1 / 3):
        assert worst > 1000


def test_no_stored_value_is_in_the_expression_and_none_is_combined_with_another() -> None:
    # 3 * fl(1/3) rounds to exactly 1.0 in double precision but is 1 - 2**-54 in exact arithmetic,
    # so a constant Tr * n formed before evaluation would not be the product the formula performs
    Tr, n = 3.0, 1 / 3
    source = InMemorySource(
        slots={("reduced_exponential.pure", ("x",)): dict(p=1.0e5, Tr=Tr, n=n, t=2 / 3)}
    )
    bound = bind(
        scenario("parameter_structure"), "reduced_exponential", source=source, roles={"i": "x"}
    )
    expression = bound.expression("p")
    assert not expression.atoms(sympy.Float)
    parameters = bound.parameters
    assert sorted(parameters.values()) == sorted([1.0e5, Tr, n, 2 / 3])
    assert set(parameters) <= expression.free_symbols
    assert len(parameters) == 4  # one symbol for each value read, however often it is read
    T = below(float(np.nextafter(Tr, 0.0)), 4)
    np.testing.assert_allclose(
        bound.evaluate("p", T=T), as_written(T, Tr, 1.0e5, n, 2 / 3), rtol=0, atol=0
    )


def test_an_integer_literal_stays_an_integer_and_a_unit_factor_is_a_parameter() -> None:
    bound = bind(
        scenario("antoine"),
        "antoine_bar",
        source=InMemorySource(slots={("antoine_bar.pure", ("w",)): dict(A=5.0, B=1500.0, C=-40.0)}),
        roles={"i": "w"},
    )
    expression = bound.expression("p_sat")
    assert not expression.atoms(sympy.Float)
    assert 1.0e5 in bound.parameters.values()  # the conversion factor of `unit('bar')`
    T = np.array([300.0, 350.0])
    close(bound.evaluate("p_sat", T=T), 10 ** (5.0 - 1500.0 / (T - 40.0)) * 1.0e5)
    # `T ** 2` of the NASA polynomial: the exponent is the integer written, not a float
    nasa = bind(
        scenario("nasa7"),
        "nasa7",
        source=InMemorySource(
            families={
                ("nasa7.pure", "piece", ("n2",)): {
                    (1,): dict(
                        T_low=200.0,
                        T_high=1000.0,
                        a1=1.0,
                        a2=1.0,
                        a3=1.0,
                        a4=1.0,
                        a5=1.0,
                        a6=1.0,
                        a7=1.0,
                    )
                }
            }
        ),
        roles={"i": "n2"},
    )
    exponents = {p.exp for p in nasa.expression("cp").atoms(sympy.Pow)}
    assert sympy.Integer(2) in exponents and all(e.is_Integer for e in exponents)


# -- the compile cache --------------------------------------------------------------------------


def series(
    cache: CompileCache, subject: str, Tr: float, p: float, terms: list[tuple[float, float]]
):  # noqa: ANN201
    rows = {(k,): dict(n=n, t=t) for k, (n, t) in enumerate(terms, 1)}
    source = InMemorySource(
        slots={("reduced_series.pure", (subject,)): dict(p=p, Tr=Tr)},
        families={("reduced_series.pure", "term", (subject,)): rows},
    )
    return bind(
        scenario("parameter_structure"),
        "reduced_series",
        source=source,
        roles={"i": subject},
        cache=cache,
    )


def series_reference(
    T: np.ndarray, Tr: float, p: float, terms: list[tuple[float, float]]
) -> np.ndarray:
    return p * np.exp(Tr / T * sum(n * (1 - T / Tr) ** t for n, t in terms))


TWO = [(-7.8, 1.0), (1.9, 1.5)]
THREE = [(-7.8, 1.0), (1.9, 1.5), (-1.2, 3.0)]


def test_subjects_of_equal_structure_compile_once_and_each_gets_its_own_answer() -> None:
    cache = CompileCache()
    curves = [
        (
            f"s{k}",
            400.0 + 25 * k,
            1.0e6 * (1 + k),
            [(n * (1 + 0.1 * k), t + 0.05 * k) for n, t in TWO],
        )
        for k in range(6)
    ]
    T = np.array([301.0, 350.0, 398.0])
    answers = []
    for subject, Tr, p, terms in curves:
        answers.append(series(cache, subject, Tr, p, terms).evaluate("p", T=T))
        close(answers[-1], series_reference(T, Tr, p, terms))
    assert (cache.compilations, cache.reused) == (1, 5)
    assert len({tuple(a) for a in answers}) == 6  # values differ, the compiled function is the same


def test_a_different_family_length_compiles_again_and_an_equal_one_does_not() -> None:
    cache = CompileCache()
    T = np.array([301.0, 350.0, 398.0])
    cases = [("a", TWO), ("b", THREE), ("c", TWO), ("d", THREE), ("e", [TWO[0]])]
    for subject, terms in cases:
        close(
            series(cache, subject, 450.0, 1.0e6, terms).evaluate("p", T=T),
            series_reference(T, 450.0, 1.0e6, terms),
        )
    assert cache.compilations == 3  # one each for 2, 3 and 1 terms


def test_the_cache_key_is_the_structure_and_no_value() -> None:
    cache = CompileCache()
    a = series(cache, "a", 450.0, 1.0e6, TWO)
    b = series(cache, "b", 451.0, 2.0e6, [(1.0, 2.0), (3.0, 4.0)])
    T = np.array([350.0])
    assert not np.array_equal(a.evaluate("p", T=T), b.evaluate("p", T=T))
    assert cache.compilations == 1
    assert a.expression("p") == b.expression("p")  # the expansions are the same expression
    assert list(a.parameters.values()) != list(b.parameters.values())


def test_a_static_conditional_reads_one_branch_and_the_branch_is_part_of_the_structure() -> None:
    cache = CompileCache()
    declaration = scenario("parameter_structure")

    def subject(name: str, a: float, b: float, form: str = "branch_on_slot"):  # noqa: ANN202
        source = InMemorySource(slots={(f"{form}.pure", (name,)): dict(a=a, b=b)})
        return bind(declaration, form, source=source, roles={"i": name}, cache=cache)

    T = np.array([300.0, 400.0])
    positive = [subject("p1", 2.0, -1.0), subject("p2", 5.0, 7.0), subject("p3", 0.5, 0.0)]
    for bound, (a, _) in zip(positive, [(2.0, 0), (5.0, 0), (0.5, 0)]):
        close(bound.evaluate("y", T=T), a * T)
    assert cache.compilations == 1
    other = subject("n1", -3.0, 4.0)
    close(other.evaluate("y", T=T), 4.0 * T)
    assert cache.compilations == 2  # the other branch reads another slot: another structure
    close(subject("n2", -1.0, 9.0).evaluate("y", T=T), 9.0 * T)
    close(subject("z", 0.0, 6.0).evaluate("y", T=T), 6.0 * T)  # a > 0 is false at zero
    assert cache.compilations == 2
    # the branch not taken is not expanded: no parameter of it is read
    assert len(positive[0].parameters) == 2 and len(other.parameters) == 3
    assert not any(
        isinstance(node, sympy.Piecewise)
        for node in sympy.preorder_traversal(positive[0].expression("y"))
    )


def test_a_local_that_depends_on_stored_values_alone_decides_at_bind_time() -> None:
    cache = CompileCache()
    declaration = scenario("parameter_structure")
    T = np.array([300.0, 400.0])

    def subject(name: str, a: float, b: float):  # noqa: ANN202
        source = InMemorySource(slots={("branch_on_local.pure", (name,)): dict(a=a, b=b)})
        return bind(declaration, "branch_on_local", source=source, roles={"i": name}, cache=cache)

    up = subject("u", 5.0, 2.0)
    close(up.evaluate("y", T=T), 3.0 * T)
    close(subject("v", 9.0, 1.0).evaluate("y", T=T), 8.0 * T)
    assert cache.compilations == 1
    down = subject("d", 1.0, 2.0)
    close(down.evaluate("y", T=T), 2.0 * T)
    assert cache.compilations == 2
    assert not up.expression("y").has(sympy.Piecewise) and not down.expression("y").has(
        sympy.Piecewise
    )


def test_a_conditional_on_an_argument_stays_in_the_expression_and_its_subjects_share_it() -> None:
    cache = CompileCache()
    declaration = scenario("parameter_structure")
    T = np.array([250.0, 299.0, 301.0, 450.0])

    def subject(name: str, a: float, b: float, T0: float):  # noqa: ANN202
        source = InMemorySource(
            slots={("branch_on_temperature.pure", (name,)): dict(a=a, b=b, T0=T0)}
        )
        return bind(
            declaration, "branch_on_temperature", source=source, roles={"i": name}, cache=cache
        )

    first = subject("a", 2.0, 3.0, 300.0)
    second = subject("b", 5.0, 7.0, 400.0)
    close(first.evaluate("y", T=T), np.where(T < 300.0, 2.0 * T, 3.0 * T))
    close(second.evaluate("y", T=T), np.where(T < 400.0, 5.0 * T, 7.0 * T))
    assert cache.compilations == 1
    assert first.expression("y").has(sympy.Piecewise)


def test_a_different_sub_form_choice_compiles_again() -> None:
    cache = CompileCache()
    declaration = scenario("iteration")

    def term(slope: float, subject: str) -> FormChoice:
        return FormChoice(
            "linear_term", InMemorySource(slots={("linear_term.pure", (subject,)): {"c": slope}})
        )

    source = InMemorySource(
        subforms={
            ("total_form.terms", ("a",)): (term(2.0, "a"), term(0.5, "a")),
            ("total_form.terms", ("b",)): (term(3.0, "b"),),
            ("total_form.terms", ("c",)): (term(4.0, "c"), term(1.5, "c")),
        }
    )
    T = np.array([300.0, 500.0])
    amounts = {}
    for name in "abc":
        amounts[name] = bind(
            declaration, "total_form", source=source, roles={"i": name}, cache=cache
        ).evaluate("amount", T=T)
    close(amounts["a"], 2.5 * T)
    close(amounts["b"], 3.0 * T)
    close(amounts["c"], 5.5 * T)
    assert cache.compilations == 2  # two contributions (a and c), one contribution (b)


def test_a_transposition_is_an_operation_on_the_value_and_not_on_the_structure() -> None:
    cache = CompileCache()
    declaration = scenario("transposition")
    source = InMemorySource(
        slots={("reciprocal_form.pair", ("a", "b")): {"r": 4.0, "s": 0.3}}, declaration=declaration
    )
    T = np.array([300.0])
    held = bind(
        declaration, "reciprocal_form", source=source, roles={"i": "a", "j": "b"}, cache=cache
    )
    swapped = bind(
        declaration, "reciprocal_form", source=source, roles={"i": "b", "j": "a"}, cache=cache
    )
    close(held.evaluate("v", T=T), np.array([4.0 + 3.0]))
    close(swapped.evaluate("v", T=T), np.array([1 / 4.0 + 3.0]))
    assert cache.compilations == 1
    assert sorted(held.parameters.values()) == [0.3, 4.0]
    assert sorted(swapped.parameters.values()) == [0.25, 0.3]  # the value passed is the reciprocal

    coefficients = {(0,): {"L": 1200.0}, (1,): {"L": -340.0}, (2,): {"L": 75.0}, (3,): {"L": 12.0}}
    activity = scenario("activity")
    rk_source = InMemorySource(
        families={("redlich_kister.pair", "rk", ("a", "b")): coefficients}, declaration=activity
    )
    x = np.linspace(0.05, 0.95, 5)
    canonical = bind(
        activity, "redlich_kister", source=rk_source, roles={"i": "a", "j": "b"}, cache=cache
    )
    flipped = bind(
        activity, "redlich_kister", source=rk_source, roles={"i": "b", "j": "a"}, cache=cache
    )
    before = cache.compilations
    canonical.evaluate("gE", xi=x)
    flipped.evaluate("gE", xi=1 - x)
    assert cache.compilations == before + 1
    assert sorted(canonical.parameters.values()) == [-340.0, 12.0, 75.0, 1200.0]
    # odd orders change sign, in the value passed
    assert sorted(flipped.parameters.values()) == [-12.0, 75.0, 340.0, 1200.0]


def test_the_pieces_of_an_interval_family_share_their_functions_across_subjects_and_their_order_is_structure() -> (
    None
):
    cache = CompileCache()
    declaration = scenario("nasa7")
    low = dict(
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
    high = dict(
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
    # the second species has its breakpoint elsewhere; the third holds its pieces in the other
    # order under their row numbers
    low2 = dict(low, T_high=1500.0, a1=3.1)
    high2 = dict(high, T_low=1500.0, a1=3.0)
    source = InMemorySource(
        families={
            ("nasa7.pure", "piece", ("n2",)): {(1,): low, (2,): high},
            ("nasa7.pure", "piece", ("o2",)): {(1,): low2, (2,): high2},
            ("nasa7.pure", "piece", ("he",)): {(1,): high2, (2,): low2},
        }
    )
    T = np.array([300.0, 1200.0, 1600.0, 5000.0])
    R_ = 8.314462618

    def cp(T: np.ndarray, pieces: list[dict[str, float]]) -> np.ndarray:
        pieces = sorted(pieces, key=lambda piece: piece["T_low"])
        out = np.empty_like(T)
        for k, value in enumerate(T):
            c = next(
                p
                for p in pieces
                if p["T_low"] <= value < p["T_high"] or (p is pieces[-1] and value <= p["T_high"])
            )
            out[k] = R_ * (
                c["a1"]
                + c["a2"] * value
                + c["a3"] * value**2
                + c["a4"] * value**3
                + c["a5"] * value**4
            )
        return out

    n2 = bind(declaration, "nasa7", source=source, roles={"i": "n2"}, cache=cache)
    o2 = bind(declaration, "nasa7", source=source, roles={"i": "o2"}, cache=cache)
    close(n2.evaluate("cp", T=T), cp(T, [low, high]), rtol=1e-12)
    close(o2.evaluate("cp", T=T), cp(T, [low2, high2]), rtol=1e-12)
    # the output and the condition that T lies in a piece: one function each, for both species
    assert (cache.compilations, cache.reused) == (2, 2)
    # the order of the pieces is decided from their bounds and is part of the structure
    he = bind(declaration, "nasa7", source=source, roles={"i": "he"}, cache=cache)
    close(he.evaluate("cp", T=T), cp(T, [low2, high2]), rtol=1e-12)
    assert cache.compilations == 3  # the output again; the condition is the same union of intervals


def test_an_implicit_block_is_compiled_once_for_every_binding_of_its_structure() -> None:
    cache = CompileCache()
    declaration = scenario("implicit_cubic")
    fluids = {
        "propane": dict(T_c=369.83, p_c=4.248e6, omega=0.152),
        "butane": dict(T_c=425.12, p_c=3.796e6, omega=0.200),
    }
    T, p = np.array([300.0, 450.0]), np.array([0.8e6, 3.0e6])

    def volume(name: str) -> np.ndarray:
        source = InMemorySource(
            slots={
                ("srk_largest.core", ()): dict(omega_a=0.42748, omega_b=0.08664, R=R),
                ("srk_largest.pure", (name,)): fluids[name],
            }
        )
        return bind(
            declaration, "srk_largest", source=source, roles={"i": name}, cache=cache
        ).evaluate("v", T=T, p=p)

    def equation_of_state(name: str, v: np.ndarray) -> np.ndarray:
        c = fluids[name]
        m = 0.48508 + 1.55171 * c["omega"] - 0.15613 * c["omega"] ** 2
        a = 0.42748 * R**2 * c["T_c"] ** 2 / c["p_c"] * (1 + m * (1 - np.sqrt(T / c["T_c"]))) ** 2
        b = 0.08664 * R * c["T_c"] / c["p_c"]
        return R * T / (v - b) - a / (v * (v + b))

    propane = volume("propane")
    after_first = cache.compilations
    butane = volume("butane")
    assert cache.compilations == after_first  # nothing compiled for the second fluid
    close(equation_of_state("propane", propane), p, rtol=1e-9)
    close(equation_of_state("butane", butane), p, rtol=1e-9)
    assert not np.allclose(propane, butane)
