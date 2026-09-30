# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Hard case (a), plan 24 packet TK2: a Helmholtz-energy fluid and a GERG-2008-style multifluid
mixture.

A pure fluid: the ideal-gas part (lead, logarithmic and Planck-Einstein terms) and the residual part
(polynomial, exponential, Gaussian and non-analytic terms) as additive term families of one slot
group, the reducing temperature and density as slots and the gas constant a fact of the
parameterisation's convention set. A mixture: the reducing function parameters (beta_T, gamma_T,
beta_v, gamma_v) of each pair with the reciprocal transposition on the betas, stored as the source
asserted them, so that a pair a source lists in the other order keeps its numbers and records the
arrangement; one departure-function set that two different pair sets reference, each pair with its
own scale factor F_ij.

The numbers are synthetic. The independent calculation of the pure fluid is written in numpy, with
the derivative of the residual part taken by complex step; that of the reducing functions is the
published sum over unordered pairs.
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
from hard_case_support import at, build, failing
from mapping_support import real_declaration, writer

from thermo_knowledge import db
from thermo_knowledge.canonical.values import Quantity
from thermo_knowledge.canonical.writer import CanonicalWriter, FamilyRow, SetReference, ValidationError
from thermo_knowledge.declaration import Declaration
from thermo_knowledge.expression.compiled import CompileCache
from thermo_knowledge.expression.evaluate import EvaluationRefusal, bind
from thermo_knowledge.qualify.source import DatabaseSource
from thermo_knowledge.testing import TestDatabase

CACHE = CompileCache()
R_PURE = 8.314472  # the gas constant of the parameterisation of the pure fluid
R_OTHER = 8.314462618

# -- the synthetic fluid ------------------------------------------------------------------------

FLUID = dict(T_r=190.0, rho_r=10_000.0, a1=-4.2, a2=3.1, c=2.5)
PLANCK_EINSTEIN = [(1.5, 3.2), (-0.8, 10.0)]  # (n, theta)
POWER = [(0.9, 1.0, 0.25), (-1.6, 1.0, 1.1), (0.04, 4.0, 0.6)]  # (n, d, t)
EXPONENTIAL = [(-0.3, 1.0, 1.5, 1.0), (0.25, 3.0, 1.1, 2.0), (-0.04, 2.0, 4.0, 3.0)]  # (n, d, t, l)
GAUSSIAN = [(0.12, 2.0, 1.0, 0.9, 0.5, 1.2, 1.1), (-0.05, 1.0, 3.0, 1.4, 0.8, 0.9, 0.7)]  # (n, d, t, eta, epsilon, beta, gamma)
NONANALYTIC = [(-0.6, 0.3, 0.85, 0.3, 0.3, 0.25, 10.0, 30.0), (0.2, 0.4, 0.9, 0.35, 0.2, 0.3, 12.0, 40.0)]  # (n, a, b, beta, A, B, C, D)

POINTS = [(T, rho) for T in (150.0, 190.0, 250.0, 400.0) for rho in (1_500.0, 4_000.0, 9_000.0, 12_000.0, 20_000.0)]


def alpha_ideal(delta, tau):  # noqa: ANN001, ANN201
    f = FLUID
    return (
        np.log(delta) + f["a1"] + f["a2"] * tau + f["c"] * np.log(tau)
        + sum(n * np.log(1 - np.exp(-theta * tau)) for n, theta in PLANCK_EINSTEIN)
    )


def alpha_residual(delta, tau):  # noqa: ANN001, ANN201
    """Written for complex arguments, so that its derivative can be taken by complex step."""
    total = 0 * delta
    for n, d, t in POWER:
        total = total + n * delta**d * tau**t
    for n, d, t, l in EXPONENTIAL:
        total = total + n * delta**d * tau**t * np.exp(-(delta**l))
    for n, d, t, eta, epsilon, beta, gamma in GAUSSIAN:
        total = total + n * delta**d * tau**t * np.exp(-eta * (delta - epsilon) ** 2 - beta * (tau - gamma) ** 2)
    for n, a, b, beta, A, B, C, D in NONANALYTIC:
        square = (delta - 1) ** 2
        theta = (1 - tau) + A * square ** (1 / (2 * beta))
        big_delta = theta**2 + B * square**a
        total = total + n * big_delta**b * delta * np.exp(-C * (delta - 1) ** 2 - D * (tau - 1) ** 2)
    return total


def pure_fluid(T, rho, gas):  # noqa: ANN001, ANN201
    """alpha_0, alpha_r, the residual molar energy and the pressure."""
    T, rho = np.asarray(T, dtype=float), np.asarray(rho, dtype=float)
    delta, tau = rho / FLUID["rho_r"], FLUID["T_r"] / T
    h = 1e-30
    alpha_r = alpha_residual(delta, tau)
    derivative = np.imag(alpha_residual(delta + 1j * h, tau)) / h
    return (
        alpha_ideal(delta, tau),
        alpha_r,
        gas * T * alpha_r,
        rho * gas * T * (1 + delta * derivative),
    )


# -- the synthetic mixture ----------------------------------------------------------------------

CRITICAL = {"A": (190.0, 10_100.0), "B": (305.0, 6_900.0), "C": (126.0, 11_200.0)}  # (T_c, rho_c)
# the pairs as the source asserts them: the first component first; (beta_T, gamma_T, beta_v, gamma_v)
PAIRS = {("A", "B"): (1.02, 1.05, 1.10, 1.04), ("C", "A"): (0.97, 0.99, 0.93, 1.01), ("B", "C"): (1.00, 1.02, 1.05, 0.98)}
DEPARTURE_POWER = [(0.30, 1.0, 0.8), (-0.12, 2.0, 1.5), (0.05, 3.0, 3.0)]
DEPARTURE_GAUSSIAN = [(0.02, 1.0, 2.0, 0.9, 0.6, 1.1, 0.8), (-0.01, 2.0, 3.0, 1.3, 0.7, 0.9, 0.5)]  # (n, d, t, eta, epsilon, beta, gamma)
SCALES = {("A", "B"): 0.92, ("A", "C"): 1.07}  # F_ij of the two pairs that have a departure function
COMPOSITIONS = [
    {"A": 0.6, "B": 0.3, "C": 0.1},
    {"A": 0.2, "B": 0.5, "C": 0.3},
    {"A": 0.05, "B": 0.05, "C": 0.9},
]


def reduced(composition: dict[str, float]) -> tuple[float, float]:
    """T_r and rho_r of the published reducing functions, summed over the unordered pairs as the
    source asserts them (beta for the first component first)."""
    x = composition
    t_r = sum(x[i] ** 2 * CRITICAL[i][0] for i in x)
    v_r = sum(x[i] ** 2 / CRITICAL[i][1] for i in x)
    for (i, j), (beta_t, gamma_t, beta_v, gamma_v) in PAIRS.items():
        t_r += 2 * beta_t * gamma_t * (x[i] + x[j]) / (beta_t**2 * x[i] + x[j]) * x[i] * x[j] * np.sqrt(CRITICAL[i][0] * CRITICAL[j][0])
        v_r += (
            2 * beta_v * gamma_v * (x[i] + x[j]) / (beta_v**2 * x[i] + x[j]) * x[i] * x[j]
            * 0.125 * (CRITICAL[i][1] ** (-1 / 3) + CRITICAL[j][1] ** (-1 / 3)) ** 3
        )
    return float(t_r), float(1 / v_r)


def reduced_with_the_pair_swapped(composition: dict[str, float]) -> tuple[float, float]:
    """The same sums with every pair written the other way round: its first component second and
    its betas inverted."""
    swapped = {(j, i): (1 / bt, gt, 1 / bv, gv) for (i, j), (bt, gt, bv, gv) in PAIRS.items()}
    x = composition
    t_r = sum(x[i] ** 2 * CRITICAL[i][0] for i in x)
    v_r = sum(x[i] ** 2 / CRITICAL[i][1] for i in x)
    for (i, j), (beta_t, gamma_t, beta_v, gamma_v) in swapped.items():
        t_r += 2 * beta_t * gamma_t * (x[i] + x[j]) / (beta_t**2 * x[i] + x[j]) * x[i] * x[j] * np.sqrt(CRITICAL[i][0] * CRITICAL[j][0])
        v_r += (
            2 * beta_v * gamma_v * (x[i] + x[j]) / (beta_v**2 * x[i] + x[j]) * x[i] * x[j]
            * 0.125 * (CRITICAL[i][1] ** (-1 / 3) + CRITICAL[j][1] ** (-1 / 3)) ** 3
        )
    return float(t_r), float(1 / v_r)


def departure(delta, tau):  # noqa: ANN001, ANN201
    total = sum(n * delta**d * tau**t for n, d, t in DEPARTURE_POWER)
    for n, d, t, eta, epsilon, beta, gamma in DEPARTURE_GAUSSIAN:
        total = total + n * delta**d * tau**t * np.exp(-eta * (delta - epsilon) ** 2 - beta * (delta - gamma))
    return total


# -- the fixture --------------------------------------------------------------------------------


@dataclass
class World:
    decl: Declaration
    database: TestDatabase
    ids: dict[str, uuid.UUID]


def family(rows: list[tuple[float, ...]], names: tuple[str, ...]) -> list[FamilyRow]:
    return [FamilyRow({"k": k}, dict(zip(names, row, strict=True))) for k, row in enumerate(rows, 1)]


def write_fluid(w: CanonicalWriter, ids: dict[str, uuid.UUID], species: uuid.UUID, gas: float, key: str) -> None:
    conventions = w.kind(
        "convention_set",
        {"key": key, "revision": "1", "temperature_scale": "its_90", "gas_constant": Quantity(gas, "J/(mol*K)")},
        origins=at(f"conventions-{key}"),
    )
    p = w.kind(
        "parameterization",
        {"key": key, "revision": "1", "title": key, "coherence": "independent_records", "convention_set": conventions},
        origins=at(f"parameterization-{key}"),
    )
    ids[f"pure_{key}"] = p
    f = FLUID
    ids[f"fluid_{key}"] = w.parameter_set(
        parameterization=p,
        slot_group="helmholtz_pure_fluid.pure",
        subjects=[species],
        slots={
            "T_r": Quantity(f["T_r"], "K"),
            "rho_r": Quantity(f["rho_r"], "mol/m^3"),
            "a1": f["a1"],
            "a2": f["a2"],
            "c": f["c"],
        },
        families={
            "planck_einstein": family(PLANCK_EINSTEIN, ("n", "theta")),
            "power": family(POWER, ("n", "d", "t")),
            "exponential": family(EXPONENTIAL, ("n", "d", "t", "l")),
            "gaussian": family(GAUSSIAN, ("n", "d", "t", "eta", "epsilon", "beta", "gamma")),
            "nonanalytic": family(NONANALYTIC, ("n", "a", "b", "beta", "A", "B", "C", "D")),
        },
        origins=at(f"fluid-{key}"),
    )


def write_world(w: CanonicalWriter, ids: dict[str, uuid.UUID]) -> None:
    species = {name: w.kind("species", {"canonical_key": name, "label": name}, origins=at(f"s-{name}")) for name in CRITICAL}
    ids.update({f"species_{name}": found for name, found in species.items()})
    write_fluid(w, ids, species["A"], R_PURE, "gerg-gas-constant")
    write_fluid(w, ids, species["A"], R_OTHER, "codata-gas-constant")
    mixture = w.kind(
        "parameterization",
        {"key": "gerg-fixture", "revision": "1", "title": "A multifluid mixture", "coherence": "jointly_fitted"},
        origins=at("mixture"),
    )
    ids["mixture"] = mixture
    for name, (T_c, rho_c) in CRITICAL.items():
        w.parameter_set(
            parameterization=mixture,
            slot_group="multifluid_reducing.pure",
            subjects=[species[name]],
            slots={"T_c": Quantity(T_c, "K"), "rho_c": Quantity(rho_c, "mol/m^3")},
            origins=at(f"critical-{name}"),
        )
    for (first, second), (beta_t, gamma_t, beta_v, gamma_v) in PAIRS.items():
        ids[f"reducing_{first}{second}"] = w.parameter_set(
            parameterization=mixture,
            slot_group="multifluid_reducing.pair",
            subjects=[species[first], species[second]],  # the order the source asserts
            slots={"beta_T": beta_t, "gamma_T": gamma_t, "beta_v": beta_v, "gamma_v": gamma_v},
            origins=at(f"reducing-{first}{second}"),
        )
    function = w.kind(
        "model_component", {"parameterization": mixture, "name": "departure-10"}, origins=at("departure-10")
    )
    ids["departure"] = w.parameter_set(
        parameterization=mixture,
        slot_group="multifluid_departure_terms.core",
        subjects=[function],
        slots={},
        families={
            "power": family(DEPARTURE_POWER, ("n", "d", "t")),
            "gaussian": family(DEPARTURE_GAUSSIAN, ("n", "d", "t", "eta", "epsilon", "beta", "gamma")),
        },
        origins=at("departure-10-terms"),
    )
    for (first, second), scale in SCALES.items():
        ids[f"pair_{first}{second}"] = w.parameter_set(
            parameterization=mixture,
            slot_group="multifluid_pair_departure_scaled.pair",
            subjects=[species[first], species[second]],
            slots={
                "F": scale,
                "departure": SetReference(mixture, "multifluid_departure_terms.core", [function]),
            },
            origins=at(f"pair-{first}{second}"),
        )


@pytest.fixture(scope="module")
def decl() -> Declaration:
    return real_declaration()


@pytest.fixture(scope="module")
def world(decl: Declaration, tmp_path_factory: pytest.TempPathFactory) -> Iterator[World]:
    ids: dict[str, uuid.UUID] = {}
    database = build(tmp_path_factory.mktemp("helmholtz"), lambda w: write_world(w, ids), decl)
    try:
        yield World(decl, database, ids)
    finally:
        database.remove()


@pytest.fixture
def conn(world: World) -> Iterator[psycopg.Connection]:
    connection = db.connect(world.database.url)
    connection.execute("SELECT 1")
    try:
        yield connection
    finally:
        connection.rollback()
        connection.close()


def scalar(conn: psycopg.Connection, query: str, *params: object) -> object:
    row = conn.execute(query, params).fetchone()  # type: ignore[arg-type]
    assert row is not None
    return row[0]


def fluid(world: World, conn: psycopg.Connection, key: str):  # noqa: ANN201
    source = DatabaseSource(conn, world.decl, [world.ids[f"pure_{key}"]])
    return bind(world.decl, "helmholtz_pure_fluid", source=source, roles={"i": str(world.ids["species_A"])}, cache=CACHE)


def mixture(world: World, conn: psycopg.Connection, order: tuple[str, ...]):  # noqa: ANN201
    source = DatabaseSource(conn, world.decl, [world.ids["mixture"]])
    members = [str(world.ids[f"species_{name}"]) for name in order]
    return bind(world.decl, "multifluid_reducing", source=source, sets={"components": members}, cache=CACHE)


def reducing(world: World, conn: psycopg.Connection, order: tuple[str, ...], composition: dict[str, float]) -> tuple[float, float]:
    found = mixture(world, conn, order)
    x = {str(world.ids[f"species_{name}"]): np.array([composition[name]]) for name in order}
    t_r = float(np.asarray(found.evaluate("T_r", x=x)).reshape(-1)[0])
    rho_r = float(np.asarray(found.evaluate("rho_r", x=x)).reshape(-1)[0])
    return t_r, rho_r


# -- the structure ------------------------------------------------------------------------------


def test_the_fixture_satisfies_every_invariant(conn: psycopg.Connection) -> None:
    assert failing(conn) == {}


def test_the_term_kinds_of_the_fluid_are_families_of_one_slot_group(world: World, conn: psycopg.Connection) -> None:
    counts = {
        name: scalar(conn, f'SELECT count(*) FROM param."helmholtz_pure_fluid__pure__{name}" WHERE set_id = %s', world.ids["fluid_gerg-gas-constant"])
        for name in ("planck_einstein", "power", "exponential", "gaussian", "nonanalytic")
    }
    assert counts == {"planck_einstein": 2, "power": 3, "exponential": 3, "gaussian": 2, "nonanalytic": 2}


def test_the_gas_constant_is_a_fact_of_the_convention_set_and_not_a_slot(decl: Declaration) -> None:
    form = decl.forms["helmholtz_pure_fluid"]
    assert [c.name for c in form.conventions] == ["gas_constant"]
    (group,) = form.slot_groups
    assert {slot.name for slot in group.slots} == {"T_r", "rho_r", "a1", "a2", "c"}


def test_the_betas_are_stored_as_the_source_asserted_them_and_the_arrangement_is_recorded(
    world: World, conn: psycopg.Connection
) -> None:
    """The pair asserted with C first keeps its published betas; the subjects are in the canonical
    order (the smaller identifier first), so the row says for which arrangement the values hold."""
    for (first, second), (beta_t, gamma_t, beta_v, gamma_v) in PAIRS.items():
        i, j, arrangement, bt, gt, bv, gv = conn.execute(
            'SELECT i, j, arrangement, "beta_T", "gamma_T", "beta_v", "gamma_v" FROM param."multifluid_reducing__pair" WHERE id = %s',
            (world.ids[f"reducing_{first}{second}"],),
        ).fetchone()  # type: ignore[misc]
        ids = (world.ids[f"species_{first}"], world.ids[f"species_{second}"])
        assert (i, j) == tuple(sorted(ids, key=str))
        assert arrangement == (0 if ids[0] < ids[1] else 1)
        assert (bt, gt, bv, gv) == (beta_t, gamma_t, beta_v, gamma_v)  # never rewritten
    arrangements = {
        scalar(conn, 'SELECT arrangement FROM param."multifluid_reducing__pair" WHERE id = %s', world.ids[f"reducing_{a}{b}"])
        for a, b in PAIRS
    }
    assert arrangements == {0, 1}  # the fixture has a pair in each arrangement


def test_one_departure_set_is_referenced_by_two_different_pair_sets(world: World, conn: psycopg.Connection) -> None:
    targets = conn.execute('SELECT id, "departure" FROM param."multifluid_pair_departure_scaled__pair"').fetchall()
    assert len(targets) == 2
    assert {target for _, target in targets} == {world.ids["departure"]}
    assert scalar(conn, 'SELECT count(*) FROM param."multifluid_departure_terms__core"') == 1
    scales = sorted(row[0] for row in conn.execute('SELECT "F" FROM param."multifluid_pair_departure_scaled__pair"'))
    assert scales == sorted(SCALES.values())  # each pair keeps its own scale factor


# -- the pure fluid against numpy ---------------------------------------------------------------


@pytest.mark.parametrize("key, gas", [("gerg-gas-constant", R_PURE), ("codata-gas-constant", R_OTHER)])
def test_the_pure_fluid_matches_the_independent_calculation(world: World, conn: psycopg.Connection, key: str, gas: float) -> None:
    T, rho = (np.array(x) for x in zip(*POINTS, strict=True))
    found = fluid(world, conn, key)
    want_ideal, want_residual, want_energy, want_pressure = pure_fluid(T, rho, gas)
    out = {name: np.asarray(found.evaluate(name, T=T, rho=rho), dtype=float).reshape(-1) for name in ("alpha_0", "alpha_r", "a_r", "p")}
    np.testing.assert_allclose(out["alpha_0"], want_ideal, rtol=1e-12)
    np.testing.assert_allclose(out["alpha_r"], want_residual, rtol=1e-12)
    np.testing.assert_allclose(out["a_r"], want_energy, rtol=1e-12)
    np.testing.assert_allclose(out["p"], want_pressure, rtol=1e-9, atol=1e-6)


def test_the_gas_constant_enters_through_the_convention_set(world: World, conn: psycopg.Connection) -> None:
    """The same coefficients under the two editions differ in exactly the terms that carry R."""
    T, rho = np.array([250.0]), np.array([4_000.0])
    gerg = fluid(world, conn, "gerg-gas-constant")
    codata = fluid(world, conn, "codata-gas-constant")
    assert float(gerg.evaluate("alpha_r", T=T, rho=rho)[0]) == float(codata.evaluate("alpha_r", T=T, rho=rho)[0])
    ratio = float(gerg.evaluate("a_r", T=T, rho=rho)[0]) / float(codata.evaluate("a_r", T=T, rho=rho)[0])
    assert ratio == pytest.approx(R_PURE / R_OTHER, rel=1e-13)


# -- the mixture against numpy ------------------------------------------------------------------


@pytest.mark.parametrize("composition", COMPOSITIONS)
def test_the_reducing_functions_match_the_independent_calculation_in_every_order_of_the_components(
    world: World, conn: psycopg.Connection, composition: dict[str, float]
) -> None:
    """The components in each of the six orders read the pairs in the order asserted or the other
    one; the reciprocal transposition makes every order give the published sums."""
    want_t, want_rho = reduced(composition)
    swapped_t, swapped_rho = reduced_with_the_pair_swapped(composition)
    assert swapped_t == pytest.approx(want_t, rel=1e-13) and swapped_rho == pytest.approx(want_rho, rel=1e-13)
    for order in itertools.permutations("ABC"):
        t_r, rho_r = reducing(world, conn, order, composition)
        assert t_r == pytest.approx(want_t, rel=1e-12), order
        assert rho_r == pytest.approx(want_rho, rel=1e-12), order


def test_the_binary_reducing_temperature_is_the_same_in_both_orientations_of_its_pair(
    world: World, conn: psycopg.Connection
) -> None:
    """A binary of the pair asserted with C first: the components as [C, A] read the asserted
    orientation, as [A, C] the other one (the betas inverted)."""
    binary = {"A": 0.35, "C": 0.65}
    first, second = ("C", "A"), ("A", "C")
    (beta_t, gamma_t, *_), (i, j) = PAIRS[first], first
    asserted = (
        binary[i] ** 2 * CRITICAL[i][0] + binary[j] ** 2 * CRITICAL[j][0]
        + 2 * beta_t * gamma_t * (binary[i] + binary[j]) / (beta_t**2 * binary[i] + binary[j]) * binary[i] * binary[j]
        * np.sqrt(CRITICAL[i][0] * CRITICAL[j][0])
    )
    inverted = 1 / beta_t
    reversed_ = (
        binary[j] ** 2 * CRITICAL[j][0] + binary[i] ** 2 * CRITICAL[i][0]
        + 2 * inverted * gamma_t * (binary[j] + binary[i]) / (inverted**2 * binary[j] + binary[i]) * binary[j] * binary[i]
        * np.sqrt(CRITICAL[j][0] * CRITICAL[i][0])
    )
    assert reversed_ == pytest.approx(asserted, rel=1e-13)
    for order in (first, second):
        found = mixture(world, conn, order)
        x = {str(world.ids[f"species_{name}"]): np.array([binary[name]]) for name in order}
        assert float(found.evaluate("T_r", x=x)[0]) == pytest.approx(float(asserted), rel=1e-12)


def test_without_the_inversion_of_the_betas_the_reducing_temperature_differs(world: World, conn: psycopg.Connection) -> None:
    """The control: with the betas used as stored in both orders (no inversion) the sum differs,
    so the agreement above is the transposition at work."""
    composition = COMPOSITIONS[0]
    x = composition
    plain = sum(x[i] ** 2 * CRITICAL[i][0] for i in x)
    for (i, j), (bt, gt, *_) in PAIRS.items():
        plain += x[i] * x[j] * gt * np.sqrt(CRITICAL[i][0] * CRITICAL[j][0]) * (
            bt * (x[i] + x[j]) / (bt**2 * x[i] + x[j]) + bt * (x[i] + x[j]) / (bt**2 * x[j] + x[i])
        )
    assert abs(plain - reduced(composition)[0]) > 1e-6


@pytest.mark.parametrize("first, second", [("A", "B"), ("B", "A"), ("A", "C"), ("C", "A")])
def test_a_pair_with_a_shared_departure_function_gives_its_scaled_contribution_in_either_order(
    world: World, conn: psycopg.Connection, first: str, second: str
) -> None:
    source = DatabaseSource(conn, world.decl, [world.ids["mixture"]])
    found = bind(
        world.decl,
        "multifluid_pair_departure_scaled",
        source=source,
        roles={"i": str(world.ids[f"species_{first}"]), "j": str(world.ids[f"species_{second}"])},
        cache=CACHE,
    )
    delta, tau = np.array([0.3, 0.9, 1.6]), np.array([1.4, 1.0, 0.7])
    scale = SCALES.get((first, second)) or SCALES[(second, first)]
    np.testing.assert_allclose(
        np.asarray(found.evaluate("alpha_r", delta=delta, tau=tau), dtype=float).reshape(-1),
        scale * departure(delta, tau),
        rtol=1e-12,
    )


# -- what the model refuses ---------------------------------------------------------------------


def fresh(decl: Declaration) -> tuple[CanonicalWriter, dict[str, uuid.UUID], uuid.UUID]:
    w = writer(decl)
    species = {name: w.kind("species", {"canonical_key": name, "label": name}, origins=at(f"s-{name}")) for name in "AB"}
    p = w.kind(
        "parameterization",
        {"key": "p", "revision": "1", "title": "p", "coherence": "independent_records"},
        origins=at("p"),
    )
    return w, species, p


def test_a_pair_of_a_component_with_itself_is_refused(decl: Declaration) -> None:
    w, species, p = fresh(decl)
    with pytest.raises(ValidationError, match="forbids the diagonal"):
        w.parameter_set(
            parameterization=p,
            slot_group="multifluid_reducing.pair",
            subjects=[species["A"], species["A"]],
            slots={"beta_T": 1.0, "gamma_T": 1.0, "beta_v": 1.0, "gamma_v": 1.0},
            origins=at("diagonal"),
        )


def test_a_departure_reference_to_a_set_of_another_contract_is_refused(decl: Declaration) -> None:
    w, species, p = fresh(decl)
    function = w.kind("model_component", {"parameterization": p, "name": "f"}, origins=at("f"))
    w.parameter_set(
        parameterization=p,
        slot_group="multifluid_reducing.pure",
        subjects=[species["A"]],
        slots={"T_c": Quantity(190.0, "K"), "rho_c": Quantity(10_000.0, "mol/m^3")},
        origins=at("critical"),
    )
    with pytest.raises(ValidationError):
        w.parameter_set(
            parameterization=p,
            slot_group="multifluid_pair_departure_scaled.pair",
            subjects=[species["A"], species["B"]],
            slots={"F": 1.0, "departure": SetReference(p, "multifluid_reducing.pure", [species["A"]])},
            origins=at("pair"),
        )
    assert w.rows("param.multifluid_pair_departure_scaled__pair") == 0
    assert function is not None


def test_a_reducing_function_read_for_a_pair_that_is_not_held_is_refused(world: World, conn: psycopg.Connection) -> None:
    """A missing pair is not a zero: a mixture of a component the parameterisation has no pair
    for is refused, naming the slot group."""
    source = DatabaseSource(conn, world.decl, [world.ids["mixture"]])
    stranger = uuid.uuid4()
    found = bind(
        world.decl,
        "multifluid_reducing",
        source=source,
        sets={"components": [str(world.ids["species_A"]), str(stranger)]},
        cache=CACHE,
    )
    with pytest.raises(EvaluationRefusal, match="multifluid_reducing"):
        found.evaluate(
            "T_r", x={str(world.ids["species_A"]): np.array([0.5]), str(stranger): np.array([0.5])}
        )


def test_a_fluid_whose_convention_set_states_no_gas_constant_is_flagged_and_refused(
    decl: Declaration, tmp_path: Path
) -> None:
    ids: dict[str, uuid.UUID] = {}

    def emit(w: CanonicalWriter) -> None:
        species = w.kind("species", {"canonical_key": "A", "label": "A"}, origins=at("s-A"))
        ids["species"] = species
        conventions = w.kind(
            "convention_set",
            {"key": "bare", "revision": "1", "temperature_scale": "its_90"},
            origins=at("conventions"),
        )
        p = w.kind(
            "parameterization",
            {"key": "bare", "revision": "1", "title": "bare", "coherence": "independent_records", "convention_set": conventions},
            origins=at("p"),
        )
        ids["parameterization"] = p
        w.parameter_set(
            parameterization=p,
            slot_group="helmholtz_pure_fluid.pure",
            subjects=[species],
            slots={"T_r": Quantity(190.0, "K"), "rho_r": Quantity(10_000.0, "mol/m^3"), "a1": 0.0, "a2": 0.0, "c": 0.0},
            families={"power": family(POWER, ("n", "d", "t"))},
            origins=at("fluid"),
        )

    database = build(tmp_path, emit, decl)
    try:
        with psycopg.connect(database.url) as connection:
            assert failing(connection) == {"parameterization_has_conventions": 1}
            source = DatabaseSource(connection, decl, [ids["parameterization"]])
            found = bind(decl, "helmholtz_pure_fluid", source=source, roles={"i": str(ids["species"])})
            with pytest.raises(EvaluationRefusal, match="gas_constant"):
                found.evaluate("p", T=np.array([250.0]), rho=np.array([4_000.0]))
    finally:
        database.remove()
