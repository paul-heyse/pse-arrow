# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Hard case (h), plan 24 packet TK2: a regional formulation, in the structure of IAPWS-IF97.

Regions are (predicate, form) pairs over the state (T, p): region 1 and 2 Gibbs forms in (p, T),
region 3 a Helmholtz form in (rho, T) whose density is found by an implicit solve at the given
(T, p), the saturation equation of region 4, region 5, the quadratic boundary between regions 2 and
3, and the wrapper form that states the predicates and chooses the form of each region; the model
assembly of the formulation puts the forms in the wrapper's sub-form slots. Each form's term lists
are families. A backward equation T(p, h) is its own form, linked to its forward form by
`auxiliary_of` and carrying an accuracy statement for its consistency tolerance. A verification
dataset is linked, by `dataset_verifies`, to the model assembly and to the parameterisation whose
implementation it checks (alignment item 9).

Every coefficient is synthetic: the structure is that of the release and the numbers are made up
(the backward coefficients are a least-squares fit, to the synthetic forward equation, of the
temperature, computed once with numpy). The independent calculation of each form is written here in
numpy, term by term, with the derivatives done by hand.
"""

from __future__ import annotations

import uuid
from collections.abc import Iterator
from dataclasses import dataclass
from pathlib import Path

import numpy as np
import psycopg
import pytest
from hard_case_support import at, build, entity_id, failing, observable_id
from mapping_support import carrier, origin, real_declaration, writer

from thermo_knowledge import db, identity
from thermo_knowledge.canonical.values import Quantity
from thermo_knowledge.canonical.writer import CanonicalWriter, FamilyRow, ValidationError
from thermo_knowledge.declaration import Declaration
from thermo_knowledge.expression.compiled import CompileCache
from thermo_knowledge.expression.evaluate import bind
from thermo_knowledge.expression.validity import Membership
from thermo_knowledge.qualify.source import DatabaseSource, SubformBinding
from thermo_knowledge.testing import TestDatabase

CARRIER = identity.identifier("source", [carrier("src", "a.json").key])
CACHE = CompileCache()
R = 8.314462618
M = 0.018015268
RS = R / M  # the specific gas constant the coefficient sets assume

# -- the synthetic coefficients ----------------------------------------------------------------

REGION1 = dict(p_star=16.53e6, T_star=1386.0, shift_pi=7.1, shift_tau=1.222)
REGION1_TERMS = [(-0.48, 0, 2), (-0.0258, 1, 1), (-0.0320, 1, 0), (-1.0e-4, 2, 1)]  # (n, I, J)
REGION2 = dict(p_star=1.0e6, T_star=540.0, shift_tau=0.5)
REGION2_IDEAL = [(-8.9, 0), (3.3, 1), (-0.62, -1)]  # (n, J)
REGION2_RESIDUAL = [(-2.0e-3, 1, 0), (3.0e-4, 1, 2), (-4.0e-5, 2, 1)]
REGION3 = dict(rho_star=322.0, T_star=647.096, n1=1.1)
REGION3_TERMS = [(0.30, 1, 0), (0.05, 2, 1), (-0.02, 1, 2), (-0.002, 3, 0)]
REGION5 = dict(p_star=1.0e6, T_star=1000.0)
REGION5_IDEAL = [(-8.0, 0), (2.9, 1), (-0.03, -2)]
REGION5_RESIDUAL = [(-1.0e-4, 1, 0), (2.0e-5, 1, 1), (-1.0e-6, 2, 0)]
SATURATION = dict(
    p_star=1.0e6,
    T_star=1.0,
    n=[1096.63, 188677.0, 10.736, -9157.73, -4623823.0, 18.782, -2022.39, 577807.0, -0.27024, 732.31],
)
BOUNDARY = (-182.859, 0.241390, 1.0e-4)  # megapascal, kelvin
LIMITS = dict(T_13=623.15, T_23=863.15, T_25=1073.15)
BACKWARD = dict(p_star=1.0e6, h_star=2.5e6, shift=1.0)
BACKWARD_TERMS = [(268.6431, 0, 0), (255.8539, 0, 1), (448.4219, 0, 3), (-0.5071, 1, 1)]  # (n, I, J)
TOLERANCE = 0.75  # kelvin, the consistency tolerance the backward equation states
BOX = dict(T=(300.0, 450.0), p=(1.0e6, 20.0e6))  # where the backward equation is stated to hold

# -- the independent calculation ---------------------------------------------------------------


def region1(T, p):  # noqa: ANN001, ANN201
    c = REGION1
    pi, tau = p / c["p_star"], c["T_star"] / T
    g_pi = sum(-n * I * (c["shift_pi"] - pi) ** (I - 1) * (tau - c["shift_tau"]) ** J for n, I, J in REGION1_TERMS)
    g_tau = sum(n * J * (c["shift_pi"] - pi) ** I * (tau - c["shift_tau"]) ** (J - 1) for n, I, J in REGION1_TERMS)
    return RS * T / p * pi * g_pi, RS * T * tau * g_tau


def gibbs_ideal_residual(c, ideal, residual, T, p):  # noqa: ANN001, ANN201
    pi, tau = p / c["p_star"], c["T_star"] / T
    shift = c.get("shift_tau", 0.0)
    g_pi = 1 / pi + sum(n * I * pi ** (I - 1) * (tau - shift) ** J for n, I, J in residual)
    g_tau = sum(n * J * tau ** (J - 1) for n, J in ideal) + sum(
        n * J * pi**I * (tau - shift) ** (J - 1) for n, I, J in residual
    )
    return RS * T / p * pi * g_pi, RS * T * tau * g_tau


def region2(T, p):  # noqa: ANN001, ANN201
    return gibbs_ideal_residual(REGION2, REGION2_IDEAL, REGION2_RESIDUAL, T, p)


def region5(T, p):  # noqa: ANN001, ANN201
    return gibbs_ideal_residual(REGION5, REGION5_IDEAL, REGION5_RESIDUAL, T, p)


def z3(delta, tau):  # noqa: ANN001, ANN201
    """p / (rho R T) of region 3: n1 + sum n I delta^I tau^J."""
    return REGION3["n1"] + sum(n * I * delta**I * tau**J for n, I, J in REGION3_TERMS)


def region3_density(T: float, p: float) -> float:
    """The density at which region 3's pressure is `p`, by bisection: the pressure increases with
    density, so the root is unique."""
    tau = REGION3["T_star"] / T
    low, high = 1e-3, 1500.0
    for _ in range(200):
        mid = 0.5 * (low + high)
        if mid * RS * T * z3(mid / REGION3["rho_star"], tau) < p:
            low = mid
        else:
            high = mid
    return 0.5 * (low + high)


def region3(T, p):  # noqa: ANN001, ANN201
    T, p = np.broadcast_arrays(np.asarray(T, dtype=float), np.asarray(p, dtype=float))
    rho = np.array([region3_density(t, q) for t, q in zip(T.ravel(), p.ravel(), strict=True)]).reshape(T.shape)
    delta, tau = rho / REGION3["rho_star"], REGION3["T_star"] / T
    phi_tau = sum(n * J * delta**I * tau ** (J - 1) for n, I, J in REGION3_TERMS)
    return 1 / rho, RS * T * (tau * phi_tau + z3(delta, tau))


def saturation(T):  # noqa: ANN001, ANN201
    n = SATURATION["n"]
    theta = T / SATURATION["T_star"] + n[8] / (T / SATURATION["T_star"] - n[9])
    a = theta**2 + n[0] * theta + n[1]
    b = n[2] * theta**2 + n[3] * theta + n[4]
    c = n[5] * theta**2 + n[6] * theta + n[7]
    return SATURATION["p_star"] * (2 * c / (-b + np.sqrt(b * b - 4 * a * c))) ** 4


def boundary(T):  # noqa: ANN001, ANN201
    return (BOUNDARY[0] + BOUNDARY[1] * T + BOUNDARY[2] * T**2) * 1.0e6


def backward(p, h):  # noqa: ANN001, ANN201
    c = BACKWARD
    return sum(n * (p / c["p_star"]) ** I * (h / c["h_star"] + c["shift"]) ** J for n, I, J in BACKWARD_TERMS)


def region_of(T: float, p: float) -> int:
    """The predicates of the regions, written as the release states them."""
    if T > LIMITS["T_25"]:
        return 5
    if T > LIMITS["T_23"]:
        return 2
    if T > LIMITS["T_13"]:
        return 2 if p < boundary(T) else 3
    return 1 if p > saturation(T) else 2


EQUATIONS = {1: region1, 2: region2, 3: region3, 5: region5}


def state(T: float, p: float) -> tuple[int, float, float]:
    region = region_of(T, p)
    v, h = EQUATIONS[region](np.float64(T), np.float64(p))
    return region, float(v), float(h)


# points on either side of every boundary the predicates have: (T, p)
def selection_points() -> list[tuple[float, float]]:
    t13, t23, t25 = LIMITS["T_13"], LIMITS["T_23"], LIMITS["T_25"]
    return [
        (400.0, 1.5 * saturation(400.0)),  # above the saturation pressure: liquid
        (400.0, 0.5 * saturation(400.0)),  # below it: vapour
        (t13, 1.001 * saturation(t13)),  # at the highest temperature of region 1, either side of ps
        (t13, 0.999 * saturation(t13)),
        (t13, 10.0e6),  # the same pressure either side of the 1/3 limit in temperature
        (t13 + 0.01, 10.0e6),
        (t13 + 0.01, 0.999 * boundary(t13 + 0.01)),  # either side of the 2/3 boundary
        (t13 + 0.01, 1.001 * boundary(t13 + 0.01)),
        (700.0, 0.999 * boundary(700.0)),
        (700.0, 1.001 * boundary(700.0)),
        (t23, 0.999 * boundary(t23)),  # at the top of the boundary, either side of it
        (t23, 1.001 * boundary(t23)),
        (t23 + 0.01, 1.001 * boundary(t23)),  # above it every state is region 2
        (t25, 30.0e6),  # either side of the 2/5 limit in temperature
        (t25 + 0.01, 30.0e6),
        (1500.0, 10.0e6),
    ]


# -- the fixture -------------------------------------------------------------------------------


@dataclass
class World:
    decl: Declaration
    database: TestDatabase
    ids: dict[str, uuid.UUID]


def rows(terms: list[tuple[float, ...]], names: tuple[str, ...]) -> list[FamilyRow]:
    return [FamilyRow({"k": k}, dict(zip(names, term, strict=True))) for k, term in enumerate(terms, 1)]


def write_model(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    water = w.kind("species", {"canonical_key": "H2O", "label": "water"}, origins=at("water"))
    ids["water"] = water
    reference = w.kind(
        "energy_reference",
        {
            "key": "iapws-triple-point",
            "enthalpy": "at_state",
            "entropy": "at_state",
            "temperature": Quantity(273.16, "K"),
            "pressure": Quantity(611.657, "Pa"),
            "state": "saturated liquid at the triple point",
            "aggregation": entity_id(decl, "aggregation", "liquid"),
            "datum_energy": "internal_energy",
            "specific_energy_value": Quantity(0.0, "J/kg"),
            "specific_entropy_value": Quantity(0.0, "J/(kg*K)"),
        },
        origins=at("energy-reference"),
    )
    conventions = w.kind(
        "convention_set",
        {
            "key": "if97",
            "revision": "1",
            "temperature_scale": "its_90",
            "energy_reference": reference,
            "gas_constant": Quantity(R, "J/(mol*K)"),
        },
        origins=at("conventions"),
    )
    p = w.kind(
        "parameterization",
        {
            "key": "if97-fixture",
            "revision": "1",
            "title": "A regional formulation of water",
            "coherence": "jointly_fitted",
            "convention_set": conventions,
        },
        origins=at("parameterization"),
    )
    ids["parameterization"] = p
    molar_mass = Quantity(M, "kg/mol")

    def put(form: str, slots: dict[str, object], families: dict[str, list[FamilyRow]] | None = None) -> uuid.UUID:
        found = w.parameter_set(
            parameterization=p,
            slot_group=f"{form}.basic" if form != "if97_regions" else "if97_regions.limits",
            subjects=[water],
            slots=slots,
            families=families,
            origins=at(form),
        )
        ids[form] = found
        return found

    c = REGION1
    put(
        "if97_gibbs_region1",
        {"p_star": Quantity(c["p_star"], "Pa"), "T_star": Quantity(c["T_star"], "K"),
         "shift_pi": c["shift_pi"], "shift_tau": c["shift_tau"], "M": molar_mass},
        {"term": rows(REGION1_TERMS, ("n", "I", "J"))},
    )
    c = REGION2
    put(
        "if97_gibbs_region2",
        {"p_star": Quantity(c["p_star"], "Pa"), "T_star": Quantity(c["T_star"], "K"),
         "shift_tau": c["shift_tau"], "M": molar_mass},
        {"ideal": rows(REGION2_IDEAL, ("n", "J")), "residual": rows(REGION2_RESIDUAL, ("n", "I", "J"))},
    )
    c = REGION3
    put(
        "if97_helmholtz_region3",
        {"rho_star": Quantity(c["rho_star"], "kg/m^3"), "T_star": Quantity(c["T_star"], "K"),
         "n1": c["n1"], "M": molar_mass},
        {"term": rows(REGION3_TERMS, ("n", "I", "J"))},
    )
    c = REGION5
    put(
        "if97_gibbs_region5",
        {"p_star": Quantity(c["p_star"], "Pa"), "T_star": Quantity(c["T_star"], "K"), "M": molar_mass},
        {"ideal": rows(REGION5_IDEAL, ("n", "J")), "residual": rows(REGION5_RESIDUAL, ("n", "I", "J"))},
    )
    c = SATURATION
    put(
        "if97_saturation",
        {"p_star": Quantity(c["p_star"], "Pa"), "T_star": Quantity(c["T_star"], "K"),
         **{f"n{k}": value for k, value in enumerate(c["n"], 1)}},
    )
    put("if97_boundary_23", {f"n{k}": value for k, value in enumerate(BOUNDARY, 1)})
    put(
        "if97_backward_region1_t_ph",
        {"p_star": Quantity(BACKWARD["p_star"], "Pa"), "h_star": Quantity(BACKWARD["h_star"], "J/kg"),
         "shift": BACKWARD["shift"]},
        {"term": rows(BACKWARD_TERMS, ("n", "I", "J"))},
    )
    put("if97_regions", {key: Quantity(value, "K") for key, value in LIMITS.items()})
    # the range each region states for its own equation (the limits of the release)
    temperature, pressure = observable_id(decl, "temperature"), observable_id(decl, "pressure")
    for form, (low, high), pressure_limit in (
        ("if97_gibbs_region1", (273.15, 623.15), 100.0),
        ("if97_gibbs_region2", (273.15, 1073.15), 100.0),
        ("if97_helmholtz_region3", (623.15, 863.15), 100.0),
        ("if97_gibbs_region5", (1073.15, 2273.15), 50.0),
    ):
        w.validity_region(
            ids[form],
            {"kind": "recommended_range"},
            [
                {"observable": temperature, "lower": Quantity(low, "K"), "upper": Quantity(high, "K")},
                {"observable": pressure, "upper": Quantity(pressure_limit, "MPa")},
            ],
            origins=at(f"range-{form}"),
        )
    # the backward equation: its box, its link to the forward equation and its tolerance
    box = w.validity_region(
        ids["if97_backward_region1_t_ph"],
        {"kind": "recommended_range"},
        [
            {"observable": temperature, "lower": Quantity(BOX["T"][0], "K"), "upper": Quantity(BOX["T"][1], "K")},
            {"observable": pressure, "lower": Quantity(BOX["p"][0], "Pa"), "upper": Quantity(BOX["p"][1], "Pa")},
        ],
        origins=at("range-backward"),
    )
    ids["auxiliary_backward"] = w.relation(
        "auxiliary_of",
        {"auxiliary": ids["if97_backward_region1_t_ph"], "target": ids["if97_gibbs_region1"]},
        {"kind": "approximate_inverse", "max_relative_error": 2.0e-3},
        at="a.json#/auxiliary",
    )
    ids["accuracy"] = w.relation(
        "accuracy_statement",
        {"record": ids["if97_backward_region1_t_ph"], "observable": temperature, "ordinal": 1},
        {
            "region": box,
            "against": ids["if97_gibbs_region1"],
            "kind": "interval",
            "statistic": "bound",
            "magnitude": Quantity(TOLERANCE, "K"),
        },
        at="a.json#/accuracy",
    )
    # the assembly of the formulation and its choices, one per sub-form slot of the wrapper
    assembly = w.kind(
        "model_assembly",
        {"key": "if97-fixture", "revision": "1", "title": "A regional formulation of water", "root": "if97_regions"},
        origins=at("assembly"),
    )
    ids["assembly"] = assembly
    for slot, form in (
        ("region1", "if97_gibbs_region1"),
        ("region2", "if97_gibbs_region2"),
        ("region3", "if97_helmholtz_region3"),
        ("region5", "if97_gibbs_region5"),
        ("saturation", "if97_saturation"),
        ("boundary", "if97_boundary_23"),
    ):
        w.kind(
            "assembly_choice",
            {"assembly": assembly, "path": slot, "ordinal": 1, "slot": f"if97_regions.{slot}", "form": form},
            at="a.json#/choice",
        )
    # the boundary between regions 2 and 3 bounds the assembly's regions, it does not approximate anything
    w.relation(
        "auxiliary_of",
        {"auxiliary": ids["if97_boundary_23"], "target": assembly},
        {"kind": "validity_boundary"},
        at="a.json#/auxiliary",
    )


VERIFICATION = [(300.0, 3.0e6), (500.0, 3.0e6), (400.0, 1.0e4), (650.0, 30.0e6), (700.0, 10.0e6), (700.0, 80.0e6), (1500.0, 10.0e6)]


def nine_digits(value: float) -> float:
    return float(f"{value:.8e}")


def write_verification(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    """The check table of the formulation: the density and enthalpy its reference implementation
    gives, to nine digits, at (T, p), and the records whose implementation it checks."""
    dataset = w.kind(
        "dataset",
        {"carrier": CARRIER, "local_key": "check-values", "kind": "verification"},
        origins=at("check-values", "computed"),
    )
    ids["verification"] = dataset
    w.relation("dataset_verifies", {"dataset": dataset, "target": ids["assembly"]}, {}, at="a.json#/check")
    w.relation("dataset_verifies", {"dataset": dataset, "target": ids["parameterization"]}, {}, at="a.json#/check")
    units = (("temperature", "variable", "K"), ("pressure", "variable", "MPa"),
             ("mass_density", "property", "kg/m^3"), ("specific_enthalpy", "property", "kJ/kg"))
    columns = {
        name: w.kind(
            "dataset_column",
            {"dataset": dataset, "ordinal": ordinal, "role": role, "observable": observable_id(decl, name)},
            at="a.json#/check",
        )
        for ordinal, (name, role, _) in enumerate(units, 1)
    }
    for index, (T, p) in enumerate(VERIFICATION, 1):
        point = w.kind("data_point", {"dataset": dataset, "index": index}, at="a.json#/check")
        _, v, h = state(T, p)
        for (name, _, unit), value in zip(units, (T, p / 1e6, nine_digits(1 / v), nine_digits(h / 1e3)), strict=True):
            w.relation(
                "datum",
                {"point": point, "column": columns[name]},
                {"state": "known", "value": Quantity(value, unit), "digits": 9},
                at="a.json#/check",
            )


@pytest.fixture(scope="module")
def decl() -> Declaration:
    return real_declaration()


@pytest.fixture(scope="module")
def world(decl: Declaration, tmp_path_factory: pytest.TempPathFactory) -> Iterator[World]:
    ids: dict[str, uuid.UUID] = {}

    def emit(w: CanonicalWriter) -> None:
        write_model(w, decl, ids)
        write_verification(w, decl, ids)

    database = build(tmp_path_factory.mktemp("if97"), emit, decl)
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


def form_bound(world: World, conn: psycopg.Connection, form: str, **options):  # noqa: ANN003, ANN201
    source = DatabaseSource(conn, world.decl, [world.ids["parameterization"]], **options)
    return bind(world.decl, form, source=source, roles={"i": str(world.ids["water"])}, cache=CACHE)


def assembly_bindings(world: World, conn: psycopg.Connection) -> dict[str, list[SubformBinding]]:
    """The forms the assembly puts in each sub-form slot of its root, read back from the database."""
    found = conn.execute(
        "SELECT sl.qualified_name, f.name FROM tk.assembly_choice c JOIN meta.subform_slot sl ON sl.id = c.slot "
        "JOIN meta.form f ON f.id = c.form WHERE c.assembly = %s ORDER BY c.path, c.ordinal",
        (world.ids["assembly"],),
    ).fetchall()
    return {slot: [SubformBinding(form, (world.ids["parameterization"],))] for slot, form in found}


def formulation(world: World, conn: psycopg.Connection):  # noqa: ANN201
    return form_bound(world, conn, "if97_regions", subforms=assembly_bindings(world, conn))


def values(found, output: str, **arguments: np.ndarray) -> np.ndarray:  # noqa: ANN001
    return np.asarray(found.evaluate(output, **arguments), dtype=float).reshape(-1)


def scalar(conn: psycopg.Connection, query: str, *params: object) -> object:
    row = conn.execute(query, params).fetchone()  # type: ignore[arg-type]
    assert row is not None
    return row[0]


# -- the structure ------------------------------------------------------------------------------


def test_the_fixture_satisfies_every_invariant(conn: psycopg.Connection) -> None:
    assert failing(conn) == {}


def test_the_forms_of_the_regions_are_declared_with_their_term_lists_as_families(decl: Declaration) -> None:
    for name, families in {
        "if97_gibbs_region1": {"term"},
        "if97_gibbs_region2": {"ideal", "residual"},
        "if97_helmholtz_region3": {"term"},
        "if97_gibbs_region5": {"ideal", "residual"},
        "if97_backward_region1_t_ph": {"term"},
    }.items():
        (group,) = decl.forms[name].slot_groups
        assert {family.name for family in group.families} == families
        assert decl.forms[name].status == "expressed"
    assert {s.name for s in decl.forms["if97_regions"].subforms} == {
        "region1", "region2", "region3", "region5", "saturation", "boundary"
    }


def test_the_assembly_chooses_one_form_for_each_slot_of_the_wrapper(world: World, conn: psycopg.Connection) -> None:
    found = {slot: binding[0].form for slot, binding in assembly_bindings(world, conn).items()}
    assert found == {
        "if97_regions.region1": "if97_gibbs_region1",
        "if97_regions.region2": "if97_gibbs_region2",
        "if97_regions.region3": "if97_helmholtz_region3",
        "if97_regions.region5": "if97_gibbs_region5",
        "if97_regions.saturation": "if97_saturation",
        "if97_regions.boundary": "if97_boundary_23",
    }


def test_the_convention_set_gives_the_zero_of_internal_energy_and_the_gas_constant(
    world: World, conn: psycopg.Connection
) -> None:
    row = conn.execute(
        "SELECT r.enthalpy::text, r.datum_energy::text, r.specific_energy_value, r.specific_entropy_value, c.gas_constant "
        "FROM tk.convention_set c JOIN tk.energy_reference r ON r.id = c.energy_reference WHERE c.key = 'if97'"
    ).fetchone()
    assert row == ("at_state", "internal_energy", 0.0, 0.0, pytest.approx(R))


def test_each_regions_range_is_a_validity_region_of_its_parameter_set(world: World, conn: psycopg.Connection) -> None:
    rows = conn.execute(
        "SELECT o.key, c.lower, c.upper FROM tk.validity_region r JOIN tk.region_clause c ON c.region = r.id "
        "JOIN tk.observable o ON o.id = c.observable WHERE r.record = %s ORDER BY c.ordinal",
        (world.ids["if97_gibbs_region5"],),
    ).fetchall()
    assert rows == [("temperature", 1073.15, 2273.15), ("pressure", None, 50.0e6)]


# -- each form against numpy --------------------------------------------------------------------

POINTS = {
    "if97_gibbs_region1": ([280.0, 300.0, 400.0, 600.0], [1.0e5, 3.0e6, 20.0e6, 80.0e6]),
    "if97_gibbs_region2": ([300.0, 500.0, 700.0, 1000.0], [3.0e3, 0.1e6, 10.0e6, 20.0e6]),
    "if97_helmholtz_region3": ([650.0, 700.0, 800.0, 863.15], [30.0e6, 60.0e6, 90.0e6, 50.0e6]),
    "if97_gibbs_region5": ([1100.0, 1500.0, 2000.0, 2273.15], [1.0e6, 10.0e6, 40.0e6, 5.0e6]),
}
FORWARD = {"if97_gibbs_region1": region1, "if97_gibbs_region2": region2,
           "if97_helmholtz_region3": region3, "if97_gibbs_region5": region5}


@pytest.mark.parametrize("form", sorted(POINTS))
def test_each_region_matches_the_independent_calculation(world: World, conn: psycopg.Connection, form: str) -> None:
    T, p = (np.array(x) for x in POINTS[form])
    found = form_bound(world, conn, form)
    want_v, want_h = FORWARD[form](T, p)
    np.testing.assert_allclose(values(found, "v", T=T, p=p), want_v, rtol=1e-10)
    np.testing.assert_allclose(values(found, "h", T=T, p=p), want_h, rtol=1e-10, atol=1e-4)


def test_the_density_of_region_3_solves_the_pressure_equation(world: World, conn: psycopg.Connection) -> None:
    """The implicit solve: the library's density, put back into the pressure equation written in
    numpy, gives the pressure that was asked for."""
    T, p = (np.array(x) for x in POINTS["if97_helmholtz_region3"])
    rho = 1.0 / values(form_bound(world, conn, "if97_helmholtz_region3"), "v", T=T, p=p)
    tau = REGION3["T_star"] / T
    pressure = rho * RS * T * z3(rho / REGION3["rho_star"], tau)
    np.testing.assert_allclose(pressure, p, rtol=1e-9)


def test_the_saturation_equation_and_the_boundary_match_the_independent_calculation(
    world: World, conn: psycopg.Connection
) -> None:
    T = np.array([273.15, 300.0, 400.0, 500.0, 600.0, 623.15, 640.0])
    np.testing.assert_allclose(values(form_bound(world, conn, "if97_saturation"), "p_sat", T=T), saturation(T), rtol=1e-11)
    assert np.all(np.diff(saturation(np.linspace(273.15, 640.0, 300))) > 0)
    T = np.array([623.15, 700.0, 800.0, 863.15])
    np.testing.assert_allclose(values(form_bound(world, conn, "if97_boundary_23"), "p", T=T), boundary(T), rtol=1e-12)


def test_the_boundary_starts_at_the_saturation_pressure_and_ends_at_the_pressure_limit() -> None:
    assert boundary(LIMITS["T_13"]) == pytest.approx(saturation(LIMITS["T_13"]), rel=1e-4)
    assert boundary(LIMITS["T_23"]) == pytest.approx(100.0e6, rel=1e-5)


# -- the region selection -----------------------------------------------------------------------


def test_the_wrapper_selects_the_region_on_either_side_of_every_boundary(world: World, conn: psycopg.Connection) -> None:
    points = selection_points()
    T, p = (np.array(x) for x in zip(*points, strict=True))
    found = formulation(world, conn)
    regions = values(found, "region", T=T, p=p)
    wanted = [region_of(t, q) for t, q in points]
    assert regions.astype(int).tolist() == wanted
    assert set(wanted) == {1, 2, 3, 5}  # every region is reached
    # the pairs of neighbours that differ in region: each boundary has both sides in the list
    assert [(wanted[n], wanted[n + 1]) for n in (0, 2, 4, 6, 8, 10, 11, 13)] == [
        (1, 2), (1, 2), (1, 3), (2, 3), (2, 3), (2, 3), (3, 2), (2, 5)
    ]


def test_the_wrapper_gives_the_volume_and_enthalpy_of_the_region_it_selects(world: World, conn: psycopg.Connection) -> None:
    points = selection_points()
    T, p = (np.array(x) for x in zip(*points, strict=True))
    found = formulation(world, conn)
    wanted = [state(t, q) for t, q in points]
    np.testing.assert_allclose(values(found, "v", T=T, p=p), [v for _, v, _ in wanted], rtol=1e-9)
    np.testing.assert_allclose(values(found, "h", T=T, p=p), [h for _, _, h in wanted], rtol=1e-9, atol=1e-3)


def test_the_wrapper_reads_more_stored_values_than_numpy_broadcasts_at_once(
    world: World, conn: psycopg.Connection
) -> None:
    """The expansion of the wrapper reads the coefficients of every region: more stored values
    than the 64 arrays `numpy.broadcast` takes, which the evaluator's shape computation once
    could not pass."""
    formulation(world, conn).evaluate("v", T=np.array([400.0]), p=np.array([3.0e6]))
    widest = max(len(getattr(entry, "symbols", ())) for entry in CACHE._entries.values())  # noqa: SLF001
    assert widest > 64


def test_a_point_is_inside_the_range_its_region_states_and_outside_the_others(world: World, conn: psycopg.Connection) -> None:
    region1_form = form_bound(world, conn, "if97_gibbs_region1")
    inside = region1_form.validity("v", "recommended_range", T=np.array([300.0, 600.0]), p=np.array([3.0e6, 90.0e6]))
    outside = region1_form.validity("v", "recommended_range", T=np.array([700.0, 300.0]), p=np.array([3.0e6, 150.0e6]))
    assert inside.tolist() == [int(Membership.INSIDE)] * 2
    assert outside.tolist() == [int(Membership.OUTSIDE)] * 2


# -- the backward equation ----------------------------------------------------------------------


def test_the_backward_equation_matches_the_independent_calculation(world: World, conn: psycopg.Connection) -> None:
    p = np.array([1.0e6, 5.0e6, 20.0e6])
    h = np.array([-1.9e6, -1.5e6, -1.2e6])
    np.testing.assert_allclose(
        values(form_bound(world, conn, "if97_backward_region1_t_ph"), "T", p=p, h=h), backward(p, h), rtol=1e-12
    )


def test_the_backward_equation_inverts_the_forward_one_within_the_stated_tolerance(
    world: World, conn: psycopg.Connection
) -> None:
    """The consistency the accuracy statement claims: T(p, h(T, p)) is T to within the tolerance
    over the box the equation is stated for, in numpy and through the library, and the tolerance
    is not vacuous (the equation is not exact)."""
    T, p = (x.ravel() for x in np.meshgrid(np.linspace(*BOX["T"], 14), np.linspace(*BOX["p"], 10)))
    _, h = region1(T, p)
    error = np.abs(backward(p, h) - T)
    assert 0.1 < error.max() <= TOLERANCE
    h_library = values(form_bound(world, conn, "if97_gibbs_region1"), "h", T=T, p=p)
    round_trip = values(form_bound(world, conn, "if97_backward_region1_t_ph"), "T", p=p, h=h_library)
    np.testing.assert_allclose(round_trip, backward(p, h), rtol=1e-9)
    assert np.abs(round_trip - T).max() <= TOLERANCE


def test_the_backward_equation_is_linked_to_its_forward_form_and_states_its_tolerance(
    world: World, conn: psycopg.Connection
) -> None:
    kind, target, relative = conn.execute(
        "SELECT kind::text, target, max_relative_error FROM tk.auxiliary_of WHERE auxiliary = %s",
        (world.ids["if97_backward_region1_t_ph"],),
    ).fetchone()  # type: ignore[misc]
    assert (kind, target, relative) == ("approximate_inverse", world.ids["if97_gibbs_region1"], 2.0e-3)
    statement = conn.execute(
        "SELECT o.key, a.kind::text, a.statistic::text, a.magnitude, a.against FROM tk.accuracy_statement a "
        "JOIN tk.observable o ON o.id = a.observable WHERE a.id = %s",
        (world.ids["accuracy"],),
    ).fetchone()
    assert statement == ("temperature", "interval", "bound", pytest.approx(TOLERANCE), world.ids["if97_gibbs_region1"])
    lower, upper = conn.execute(
        "SELECT c.lower, c.upper FROM tk.accuracy_statement a JOIN tk.region_clause c ON c.region = a.region "
        "JOIN tk.observable o ON o.id = c.observable WHERE a.id = %s AND o.key = 'temperature'",
        (world.ids["accuracy"],),
    ).fetchone()  # type: ignore[misc]
    assert (lower, upper) == BOX["T"]
    assert scalar(conn, "SELECT kind::text FROM tk.auxiliary_of WHERE auxiliary = %s", world.ids["if97_boundary_23"]) == "validity_boundary"


# -- the verification dataset -------------------------------------------------------------------


def test_the_verification_dataset_names_the_assembly_and_the_parameterisation_it_checks(
    world: World, conn: psycopg.Connection
) -> None:
    targets = {
        target: kind
        for target, kind in conn.execute(
            "SELECT v.target, r.kind FROM ev.dataset_verifies v JOIN prov.record r ON r.id = v.target WHERE v.dataset = %s",
            (world.ids["verification"],),
        ).fetchall()
    }
    assert targets == {world.ids["assembly"]: "model_assembly", world.ids["parameterization"]: "parameterization"}
    assert scalar(conn, "SELECT kind::text FROM ev.dataset WHERE id = %s", world.ids["verification"]) == "verification"


def test_the_formulation_reproduces_its_check_values_from_their_inputs(world: World, conn: psycopg.Connection) -> None:
    """A qualification run of the verified assembly: inputs and outputs as the table gives them."""
    table = conn.execute(
        "SELECT p.index, o.key, d.value FROM ev.data_point p JOIN ev.datum d ON d.point = p.id "
        "JOIN ev.dataset_column c ON c.id = d.\"column\" JOIN tk.observable o ON o.id = c.observable "
        "WHERE p.dataset = %s ORDER BY p.index",
        (world.ids["verification"],),
    ).fetchall()
    by_point: dict[int, dict[str, float]] = {}
    for index, key, value in table:
        by_point.setdefault(index, {})[key] = value
    by_key = [by_point[index] for index in sorted(by_point)]
    assert len(by_key) == len(VERIFICATION)
    T = np.array([row["temperature"] for row in by_key])
    p = np.array([row["pressure"] for row in by_key])
    found = formulation(world, conn)
    density = 1.0 / values(found, "v", T=T, p=p)
    enthalpy = values(found, "h", T=T, p=p)
    np.testing.assert_allclose(density, [row["mass_density"] for row in by_key], rtol=1e-8)
    np.testing.assert_allclose(enthalpy, [row["specific_enthalpy"] for row in by_key], rtol=1e-8)
    assert {region_of(t, q) for t, q in VERIFICATION} == {1, 2, 3, 5}


# -- what the model refuses ---------------------------------------------------------------------


def test_the_verify_checks_flag_a_verification_dataset_without_a_target_a_link_from_a_measured_dataset_and_a_target_of_another_kind(
    decl: Declaration, tmp_path: Path
) -> None:
    def emit(w: CanonicalWriter) -> None:
        water = w.kind("species", {"canonical_key": "H2O", "label": "water"}, origins=at("water"))
        assembly = w.kind(
            "model_assembly",
            {"key": "a", "revision": "1", "title": "a", "root": "if97_regions"},
            origins=at("assembly"),
        )
        unlinked = w.kind(
            "dataset",
            {"carrier": CARRIER, "local_key": "unlinked", "kind": "verification"},
            origins=at("unlinked", "computed"),
        )
        measured = w.kind(
            "dataset",
            {"carrier": CARRIER, "local_key": "measured", "kind": "measured"},
            origins=at("measured", "measured"),
        )
        w.relation("dataset_verifies", {"dataset": measured, "target": assembly}, {}, at="a.json#/measured")
        wrong = w.kind(
            "dataset",
            {"carrier": CARRIER, "local_key": "wrong-target", "kind": "verification"},
            origins=at("wrong", "computed"),
        )
        w.relation("dataset_verifies", {"dataset": wrong, "target": water}, {}, at="a.json#/wrong")
        assert unlinked != wrong

    database = build(tmp_path, emit, decl)
    try:
        with psycopg.connect(database.url) as connection:
            assert failing(connection) == {
                "dataset.verification_has_a_target": 1,
                "dataset_verifies.dataset_is_verification": 1,
                "dataset_verifies.target_is_assembly_or_parameterization": 1,
            }
    finally:
        database.remove()


def test_a_coefficient_set_that_leaves_out_its_molar_mass_is_refused(decl: Declaration) -> None:
    w = writer(decl)
    water = w.kind("species", {"canonical_key": "H2O", "label": "water"}, origins=at("water"))
    p = w.kind(
        "parameterization",
        {"key": "p", "revision": "1", "title": "p", "coherence": "independent_records"},
        origins=at("p"),
    )
    with pytest.raises(ValidationError, match="M"):
        w.parameter_set(
            parameterization=p,
            slot_group="if97_gibbs_region5.basic",
            subjects=[water],
            slots={"p_star": Quantity(1.0e6, "Pa"), "T_star": Quantity(1000.0, "K")},
            families={"ideal": rows(REGION5_IDEAL, ("n", "J")), "residual": rows(REGION5_RESIDUAL, ("n", "I", "J"))},
            origins=at("region5"),
        )
    assert w.rows("param.if97_gibbs_region5__basic") == 0


def test_a_reducing_enthalpy_in_the_unit_of_a_temperature_is_refused(decl: Declaration) -> None:
    w = writer(decl)
    water = w.kind("species", {"canonical_key": "H2O", "label": "water"}, origins=at("water"))
    p = w.kind(
        "parameterization",
        {"key": "p", "revision": "1", "title": "p", "coherence": "independent_records"},
        origins=at("p"),
    )
    with pytest.raises(ValidationError, match="h_star"):
        w.parameter_set(
            parameterization=p,
            slot_group="if97_backward_region1_t_ph.basic",
            subjects=[water],
            slots={"p_star": Quantity(1.0e6, "Pa"), "h_star": Quantity(2500.0, "K"), "shift": 1.0},
            families={"term": rows(BACKWARD_TERMS, ("n", "I", "J"))},
            origins=at("backward"),
        )


def test_a_relation_of_an_unknown_kind_between_a_backward_set_and_its_forward_set_is_refused(decl: Declaration) -> None:
    w = writer(decl)
    with pytest.raises(ValidationError, match="kind"):
        w.relation(
            "auxiliary_of",
            {"auxiliary": uuid.uuid4(), "target": uuid.uuid4()},
            {"kind": "exact_forward"},
            at="a.json#/auxiliary",
        )
