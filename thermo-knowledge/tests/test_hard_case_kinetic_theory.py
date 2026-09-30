# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Hard case (m), plan 24 packet TK2: kinetic-theory transport.

Dilute-gas viscosity and thermal conductivity from Chapman-Enskog theory: a species has a molar mass
and the Lennard-Jones diameter and well depth over k; the reduced collision integral is a correlation
form (Neufeld's fit) referenced through a sub-form slot of the transport form; the Boltzmann and
Avogadro constants are convention facts of the parameterisation, stated as two separate facts, so two
parameterisations that took them from different editions cannot be evaluated together; Wilke's rule
combines the viscosities of the pure components through a sub-form slot.

The numbers are synthetic. The independent calculation is the viscosity, the conductivity, the
collision integral and Wilke's rule written in numpy.
"""

from __future__ import annotations

import uuid
from collections.abc import Iterator
from dataclasses import dataclass
from pathlib import Path

import numpy as np
import psycopg
import pytest
from hard_case_support import CHECKS, at, build, failing
from mapping_support import real_declaration, writer

from thermo_knowledge import db
from thermo_knowledge.canonical.values import Quantity
from thermo_knowledge.canonical.writer import CanonicalWriter, ValidationError
from thermo_knowledge.declaration import Declaration
from thermo_knowledge.expression.compiled import CompileCache
from thermo_knowledge.expression.evaluate import EvaluationRefusal, bind
from thermo_knowledge.qualify.source import DatabaseSource, SubformBinding
from thermo_knowledge.testing import TestDatabase
from thermo_knowledge.verify.run import run_check

CACHE = CompileCache()

# -- the constants of two editions ---------------------------------------------------------------------

EDITIONS = {  # the Boltzmann constant in J/K and the Avogadro constant in 1/mol
    "exact": (1.380649e-23, 6.02214076e23),
    "earlier": (1.38064852e-23, 6.022140857e23),
}

# -- the synthetic gases: M in kg/mol, sigma in angstrom, epsilon over k in K, and the edition of each ----

GASES = {
    "gas-a": (0.0400, 3.40, 120.0, "exact"),
    "gas-b": (0.0280, 3.70, 95.0, "exact"),
    "gas-c": (0.0160, 3.80, 148.0, "earlier"),
}
NEUFELD = dict(A=1.10, B=0.15, C=0.50, D=0.70, E=1.60, F=2.4, G=2.0, H=3.9)
TEMPERATURES = np.array([150.0, 300.0, 450.0, 800.0])

# -- the independent calculation -------------------------------------------------------------------------


def omega(T_star: np.ndarray) -> np.ndarray:
    c = NEUFELD
    return (
        c["A"] / T_star ** c["B"]
        + c["C"] / np.exp(c["D"] * T_star)
        + c["E"] / np.exp(c["F"] * T_star)
        + c["G"] / np.exp(c["H"] * T_star)
    )


def viscosity(gas: str, T: np.ndarray, edition: str | None = None) -> np.ndarray:
    M, sigma, eps, own = GASES[gas]
    kB, NA = EDITIONS[edition or own]
    m = M / NA
    return 5.0 / 16.0 * np.sqrt(np.pi * m * kB * T) / (np.pi * (sigma * 1e-10) ** 2 * omega(T / eps))


def conductivity(gas: str, T: np.ndarray, edition: str | None = None) -> np.ndarray:
    M, _, _, own = GASES[gas]
    kB, NA = EDITIONS[edition or own]
    return 15.0 / 4.0 * kB / (M / NA) * viscosity(gas, T, edition)


def wilke(names: list[str], x: np.ndarray, T: float, edition: str | None = None) -> float:
    eta = np.array([float(viscosity(n, np.array(T), edition)) for n in names])
    M = np.array([GASES[n][0] for n in names])
    total = 0.0
    for i in range(len(names)):
        denominator = 0.0
        for j in range(len(names)):
            phi = (1.0 + np.sqrt(eta[i] / eta[j]) * (M[j] / M[i]) ** 0.25) ** 2 / np.sqrt(8.0 * (1.0 + M[i] / M[j]))
            denominator += x[j] * phi
        total += x[i] * eta[i] / denominator
    return float(total)


# -- the fixture ---------------------------------------------------------------------------------------------


@dataclass
class World:
    decl: Declaration
    database: TestDatabase
    ids: dict[str, uuid.UUID]


def convention_set(w: CanonicalWriter, edition: str) -> uuid.UUID:
    kB, NA = EDITIONS[edition]
    return w.kind(
        "convention_set",
        {
            "key": f"constants-{edition}",
            "revision": "1",
            "temperature_scale": "its_90",
            "boltzmann_constant": Quantity(kB, "J/K"),
            "avogadro_constant": Quantity(NA, "1/mol"),
        },
        origins=at(f"constants-{edition}"),
    )


def write_world(w: CanonicalWriter, ids: dict[str, uuid.UUID]) -> None:
    for name in GASES:
        ids[name] = w.kind("species", {"canonical_key": name, "label": name}, origins=at(name))
    neufeld = w.kind(
        "parameterization",
        {"key": "neufeld", "revision": "1", "title": "collision integral", "coherence": "independent_records"},
        origins=at("neufeld"),
    )
    ids["neufeld"] = neufeld
    w.parameter_set(
        parameterization=neufeld, slot_group="neufeld_collision_integral.correlation", subjects=[],
        slots=dict(NEUFELD), origins=at("neufeld-set"),
    )
    for edition in EDITIONS:
        ids[f"constants_{edition}"] = convention_set(w, edition)
        ids[f"pure_{edition}"] = w.kind(
            "parameterization",
            {
                "key": f"transport-{edition}",
                "revision": "1",
                "title": f"pure gases with the {edition} constants",
                "coherence": "independent_records",
                "convention_set": ids[f"constants_{edition}"],
            },
            origins=at(f"transport-{edition}"),
        )
    for name, (mass, sigma, eps, edition) in GASES.items():
        w.parameter_set(
            parameterization=ids[f"pure_{edition}"], slot_group="chapman_enskog_transport.pure", subjects=[ids[name]],
            slots={"M": Quantity(mass, "kg/mol"), "sigma": Quantity(sigma, "angstrom"), "epsilon_over_k": Quantity(eps, "K")},
            origins=at(f"pure-{name}"),
        )


@pytest.fixture(scope="module")
def decl() -> Declaration:
    return real_declaration()


@pytest.fixture(scope="module")
def world(decl: Declaration, tmp_path_factory: pytest.TempPathFactory) -> Iterator[World]:
    ids: dict[str, uuid.UUID] = {}
    database = build(tmp_path_factory.mktemp("kinetic"), lambda w: write_world(w, ids), decl)
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


def source(world: World, conn: psycopg.Connection, editions: tuple[str, ...]) -> DatabaseSource:
    parameterizations = tuple(world.ids[f"pure_{e}"] for e in editions)
    neufeld = SubformBinding("neufeld_collision_integral", (world.ids["neufeld"],))
    return DatabaseSource(
        conn,
        world.decl,
        list(parameterizations),
        subforms={
            "chapman_enskog_transport.collision_integral": [neufeld],
            "wilke_mixture_viscosity.pure": [SubformBinding("chapman_enskog_transport", parameterizations)],
        },
    )


def transport(world: World, conn: psycopg.Connection, gas: str):  # noqa: ANN201
    return bind(
        world.decl, "chapman_enskog_transport", source=source(world, conn, (GASES[gas][3],)), roles={"i": str(world.ids[gas])}, cache=CACHE
    )


def test_the_fixture_satisfies_every_invariant(conn: psycopg.Connection) -> None:
    assert failing(conn) == {}


# -- what the source holds -------------------------------------------------------------------------------------


def test_each_gas_has_a_molar_mass_and_two_lennard_jones_parameters(world: World, conn: psycopg.Connection) -> None:
    for name, (mass, sigma, eps, edition) in GASES.items():
        row = conn.execute(
            'SELECT p."M", p.sigma, p.epsilon_over_k, s.parameterization FROM param."chapman_enskog_transport__pure" p '
            "JOIN tk.parameter_set s ON s.id = p.id WHERE p.i = %s",
            (world.ids[name],),
        ).fetchone()
        assert row[:3] == pytest.approx((mass, sigma * 1e-10, eps), rel=1e-12)
        assert row[3] == world.ids[f"pure_{edition}"]
    slots = {s.name for group in world.decl.forms["chapman_enskog_transport"].slot_groups for s in group.slots}
    assert slots == {"M", "sigma", "epsilon_over_k"}  # the constants have no slot: they belong to the convention set


def test_the_boltzmann_and_avogadro_constants_are_two_convention_facts_the_form_declares(world: World, conn: psycopg.Connection) -> None:
    facts = {
        r[0]
        for r in conn.execute(
            "SELECT name FROM meta.form_convention WHERE form = 'chapman_enskog_transport'"
        ).fetchall()
    }
    assert facts == {"boltzmann_constant", "avogadro_constant"}
    for edition, (kB, NA) in EDITIONS.items():
        row = conn.execute(
            "SELECT boltzmann_constant, avogadro_constant, gas_constant FROM tk.convention_set WHERE id = %s", (world.ids[f"constants_{edition}"],)
        ).fetchone()
        assert row == (kB, NA, None)  # a gas constant is not derived from them: it is a third fact, not stated here
    assert world.decl.forms["wilke_mixture_viscosity"].conventions == ()  # the mixture form reads none itself


def test_the_collision_integral_is_a_form_of_its_own_in_a_subform_slot(world: World) -> None:
    (slot,) = world.decl.forms["chapman_enskog_transport"].subforms
    assert (slot.name, slot.accepts, slot.multiplicity, slot.per) == ("collision_integral", "collision_integral", "one", "model")
    assert world.decl.forms["neufeld_collision_integral"].implements == "collision_integral"
    (group,) = world.decl.forms["neufeld_collision_integral"].slot_groups
    assert group.subjects == ()  # the coefficients belong to no species


# -- evaluation ---------------------------------------------------------------------------------------------------


def test_the_collision_integral_matches_numpy(world: World, conn: psycopg.Connection) -> None:
    bound = bind(world.decl, "neufeld_collision_integral", source=DatabaseSource(conn, world.decl, [world.ids["neufeld"]]), cache=CACHE)
    T_star = np.geomspace(0.3, 30.0, 9)
    found = np.asarray(bound.evaluate("omega", T_star=T_star), dtype=float).reshape(-1)
    np.testing.assert_allclose(found, omega(T_star), rtol=1e-12)


@pytest.mark.parametrize("gas", list(GASES))
def test_the_viscosity_matches_numpy(world: World, conn: psycopg.Connection, gas: str) -> None:
    found = np.asarray(transport(world, conn, gas).evaluate("eta", T=TEMPERATURES), dtype=float).reshape(-1)
    np.testing.assert_allclose(found, viscosity(gas, TEMPERATURES), rtol=1e-11)
    assert np.all(np.diff(found) > 0)  # a dilute gas grows more viscous when heated (over this range)


@pytest.mark.parametrize("gas", list(GASES))
def test_the_thermal_conductivity_is_fifteen_quarters_k_over_m_times_the_viscosity(world: World, conn: psycopg.Connection, gas: str) -> None:
    bound = transport(world, conn, gas)
    found = np.asarray(bound.evaluate("lam", T=TEMPERATURES), dtype=float).reshape(-1)
    np.testing.assert_allclose(found, conductivity(gas, TEMPERATURES), rtol=1e-11)
    eta = np.asarray(bound.evaluate("eta", T=TEMPERATURES), dtype=float).reshape(-1)
    M = GASES[gas][0]
    kB, NA = EDITIONS[GASES[gas][3]]
    np.testing.assert_allclose(found / eta, 15.0 / 4.0 * kB * NA / M, rtol=1e-11)


def test_a_different_edition_of_the_constants_changes_the_value(world: World) -> None:
    exact = viscosity("gas-a", TEMPERATURES, "exact")
    earlier = viscosity("gas-a", TEMPERATURES, "earlier")
    assert np.all(np.abs(exact / earlier - 1.0) > 1e-9) and np.all(np.abs(exact / earlier - 1.0) < 1e-6)


@pytest.mark.parametrize("names", [["gas-a", "gas-b"], ["gas-b", "gas-a"]])
def test_the_wilke_mixture_matches_numpy_in_either_order_of_the_components(world: World, conn: psycopg.Connection, names: list[str]) -> None:
    bound = bind(
        world.decl, "wilke_mixture_viscosity", source=source(world, conn, ("exact",)),
        sets={"components": [str(world.ids[n]) for n in names]}, cache=CACHE,
    )
    for T in (250.0, 600.0):
        for x_first in (0.1, 0.5, 0.9):
            x = np.array([x_first, 1.0 - x_first])
            found = float(
                np.asarray(
                    bound.evaluate(
                        "eta",
                        T=np.array([T]),
                        x={str(world.ids[n]): np.array([v]) for n, v in zip(names, x, strict=True)},
                        M={str(world.ids[n]): np.array([GASES[n][0]]) for n in names},
                    )
                ).reshape(-1)[0]
            )
            assert found == pytest.approx(wilke(names, x, T), rel=1e-11)


def test_the_wilke_rule_reduces_to_the_pure_viscosity_for_a_pure_component(world: World, conn: psycopg.Connection) -> None:
    names = ["gas-a", "gas-b"]
    bound = bind(
        world.decl, "wilke_mixture_viscosity", source=source(world, conn, ("exact",)),
        sets={"components": [str(world.ids[n]) for n in names]}, cache=CACHE,
    )
    found = float(
        np.asarray(
            bound.evaluate(
                "eta",
                T=np.array([300.0]),
                x={str(world.ids["gas-a"]): np.array([1.0]), str(world.ids["gas-b"]): np.array([0.0])},
                M={str(world.ids[n]): np.array([GASES[n][0]]) for n in names},
            )
        ).reshape(-1)[0]
    )
    assert found == pytest.approx(float(viscosity("gas-a", np.array(300.0))), rel=1e-12)


# -- the editions of the constants ----------------------------------------------------------------------------------------


def test_components_whose_parameterisations_state_different_constants_cannot_be_combined(world: World, conn: psycopg.Connection) -> None:
    names = ["gas-a", "gas-c"]  # the first from the exact edition, the second from the earlier one
    bound = bind(
        world.decl, "wilke_mixture_viscosity", source=source(world, conn, ("exact", "earlier")),
        sets={"components": [str(world.ids[n]) for n in names]}, cache=CACHE,
    )
    with pytest.raises(EvaluationRefusal, match="boltzmann_constant|avogadro_constant"):
        bound.evaluate(
            "eta",
            T=np.array([300.0]),
            x={str(world.ids[n]): np.array([0.5]) for n in names},
            M={str(world.ids[n]): np.array([GASES[n][0]]) for n in names},
        )


# -- what the model refuses ---------------------------------------------------------------------------------------------------------


def test_a_diameter_needs_a_length_and_a_constant_needs_its_own_dimension(decl: Declaration) -> None:
    w = writer(decl)
    gas = w.kind("species", {"canonical_key": "g", "label": "g"}, origins=at("g"))
    p = w.kind("parameterization", {"key": "p", "revision": "1", "title": "p", "coherence": "independent_records"}, origins=at("p"))
    with pytest.raises(ValidationError, match="cannot be converted"):
        w.parameter_set(
            parameterization=p, slot_group="chapman_enskog_transport.pure", subjects=[gas],
            slots={"M": Quantity(0.04, "kg/mol"), "sigma": Quantity(3.4, "K"), "epsilon_over_k": Quantity(120.0, "K")}, origins=at("wrong"),
        )
    with pytest.raises(ValidationError, match="cannot be converted"):
        w.kind(
            "convention_set",
            {"key": "bad", "revision": "1", "temperature_scale": "its_90", "boltzmann_constant": Quantity(1.38e-23, "J/mol")},
            origins=at("bad"),
        )


def test_the_verify_check_flags_a_parameterisation_that_does_not_state_the_constants_a_form_reads(decl: Declaration, tmp_path: Path) -> None:
    ids: dict[str, uuid.UUID] = {}

    def emit(w: CanonicalWriter) -> None:
        gas = w.kind("species", {"canonical_key": "g", "label": "g"}, origins=at("g"))
        only_boltzmann = w.kind(
            "convention_set",
            {"key": "half", "revision": "1", "temperature_scale": "its_90", "boltzmann_constant": Quantity(1.380649e-23, "J/K")},
            origins=at("half"),
        )
        for key, conventions in (("half", only_boltzmann), ("none", None)):
            ids[key] = w.kind(
                "parameterization",
                {"key": key, "revision": "1", "title": key, "coherence": "independent_records", **({"convention_set": conventions} if conventions else {})},
                origins=at(f"p-{key}"),
            )
            w.parameter_set(
                parameterization=ids[key], slot_group="chapman_enskog_transport.pure", subjects=[gas],
                slots={"M": Quantity(0.04, "kg/mol"), "sigma": Quantity(3.4, "angstrom"), "epsilon_over_k": Quantity(120.0, "K")},
                origins=at(f"set-{key}"),
            )

    database = build(tmp_path, emit, decl)
    try:
        with psycopg.connect(database.url, autocommit=True) as connection:
            found = failing(connection)
            assert set(found) == {"parameterization_has_conventions"}
            flagged = run_check(connection, CHECKS["parameterization_has_conventions"], shown=10)
            assert sorted(set(flagged.ids)) == sorted(str(ids[k]) for k in ("half", "none"))
    finally:
        database.remove()
