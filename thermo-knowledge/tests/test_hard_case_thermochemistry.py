# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Hard case (g), plan 24 packet TK2: the standard-state thermochemistry of one species form in the
four ways the sources hold it. A two-piece NASA seven-coefficient set, a three-piece NASA
nine-coefficient set and a two-piece Shomate set of one species form, in one parameterisation
under a convention set that states its energy reference and gas constant; a JANAF-style evaluated
table (T, Cp, S, -(G-H(Tr))/T, H-H(Tr), formation enthalpy and Gibbs energy, log Kf) with its row
at 0 K where two quantities diverge; and an equivalence assessment between the table and the
NASA-7 fit.

Every number is synthetic: coefficients are invented and the pieces are made continuous by
construction, so the table, which is the fit rounded the way a printed table is, agrees with it.
The forms are the committed ones of `forms/standard_state.toml`. Each form is evaluated at points
in every piece and at each common piece boundary, and the stored pieces are checked for continuity
of Cp, H and S across the boundary; the independent calculation is written here in numpy.
"""

from __future__ import annotations

import math
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
from thermo_knowledge.expression.evaluate import EvaluationRefusal, PiecePolicy, bind
from thermo_knowledge.qualify.source import DatabaseSource
from thermo_knowledge.testing import TestDatabase

CARRIER = identity.identifier("source", [carrier("src", "a.json").key])
R = 8.314462618  # the gas constant of the parameterisation's convention set, J/(mol K)
R_TABLE = 8.314510  # the gas constant the evaluated table assumes
T_REF = 298.15
CACHE = CompileCache()  # what one expansion compiles serves every later binding of the same structure

# -- the independent calculation ---------------------------------------------------------------


def nasa7_values(c: dict[str, float], T: np.ndarray, gas: float = R) -> tuple[np.ndarray, ...]:
    """Cp, H and S of one NASA-7 piece, from the published formulas written out term by term."""
    cp = gas * (c["a1"] + c["a2"] * T + c["a3"] * T**2 + c["a4"] * T**3 + c["a5"] * T**4)
    h = gas * (
        c["a1"] * T + c["a2"] * T**2 / 2 + c["a3"] * T**3 / 3 + c["a4"] * T**4 / 4
        + c["a5"] * T**5 / 5 + c["a6"]
    )
    s = gas * (
        c["a1"] * np.log(T) + c["a2"] * T + c["a3"] * T**2 / 2 + c["a4"] * T**3 / 3
        + c["a5"] * T**4 / 4 + c["a7"]
    )
    return cp, h, s


def nasa9_values(c: dict[str, float], T: np.ndarray, gas: float = R) -> tuple[np.ndarray, ...]:
    cp = gas * (
        c["a1"] / T**2 + c["a2"] / T + c["a3"] + c["a4"] * T + c["a5"] * T**2
        + c["a6"] * T**3 + c["a7"] * T**4
    )
    h = gas * (
        -c["a1"] / T + c["a2"] * np.log(T) + c["a3"] * T + c["a4"] * T**2 / 2
        + c["a5"] * T**3 / 3 + c["a6"] * T**4 / 4 + c["a7"] * T**5 / 5 + c["b1"]
    )
    s = gas * (
        -c["a1"] / (2 * T**2) - c["a2"] / T + c["a3"] * np.log(T) + c["a4"] * T
        + c["a5"] * T**2 / 2 + c["a6"] * T**3 / 3 + c["a7"] * T**4 / 4 + c["b2"]
    )
    return cp, h, s


def shomate_values(c: dict[str, float], T: np.ndarray, gas: float = 0.0) -> tuple[np.ndarray, ...]:
    cp = c["A"] + c["B"] * T + c["C"] * T**2 + c["D"] * T**3 + c["E"] / T**2
    h = c["A"] * T + c["B"] * T**2 / 2 + c["C"] * T**3 / 3 + c["D"] * T**4 / 4 - c["E"] / T + c["F"]
    s = (
        c["A"] * np.log(T) + c["B"] * T + c["C"] * T**2 / 2 + c["D"] * T**3 / 3
        - c["E"] / (2 * T**2) + c["G"]
    )
    return cp, h, s


def join(values, free: dict[str, float], lower: dict[str, float], names: tuple[str, str, str], at_T: float) -> dict[str, float]:  # noqa: ANN001
    """The piece above `lower` with the coefficients `free` given and the three constants `names`
    (the one that sets Cp, the one that sets H, the one that sets S) chosen so that Cp, H and S are
    continuous at `at_T`. Each constant enters its function linearly and alone, so it is found by
    evaluating the piece with the three at zero."""
    def at_boundary(piece: dict[str, float]) -> tuple[float, float, float]:
        cp, h, s = values(piece, np.array([at_T]))
        return float(cp[0]), float(h[0]), float(s[0])

    target = at_boundary(lower)
    piece = {**free, **dict.fromkeys(names, 0.0), "T_low": at_T}
    for position, name in enumerate(names):
        base = at_boundary(piece)
        unit = at_boundary({**piece, name: 1.0})[position] - base[position]
        piece[name] = (target[position] - base[position]) / unit
    return piece


NASA7_LOW = dict(T_low=200.0, T_high=1000.0, a1=3.6, a2=-1.0e-3, a3=2.0e-6, a4=-1.2e-9, a5=2.8e-13, a6=-30_000.0, a7=2.5)
NASA7_HIGH = {
    **join(
        nasa7_values,
        dict(a2=1.3e-3, a3=-4.1e-7, a4=6.0e-11, a5=-4.0e-15),
        NASA7_LOW,
        ("a1", "a6", "a7"),
        1000.0,
    ),
    "T_high": 6000.0,
}
NASA7 = [NASA7_LOW, NASA7_HIGH]

NASA9_1 = dict(T_low=200.0, T_high=1000.0, a1=1.0e4, a2=-150.0, a3=3.0, a4=2.0e-3, a5=-1.5e-6, a6=4.0e-10, a7=-3.0e-14, b1=-20_000.0, b2=8.0)
NASA9_2 = {
    **join(nasa9_values, dict(a1=5.0e3, a2=-50.0, a4=-1.0e-4, a5=3.0e-8, a6=-1.0e-11, a7=1.0e-15), NASA9_1, ("a3", "b1", "b2"), 1000.0),
    "T_high": 6000.0,
}
NASA9_3 = {
    **join(nasa9_values, dict(a1=-2.0e4, a2=200.0, a4=4.0e-4, a5=-2.0e-8, a6=5.0e-13, a7=-4.0e-18), NASA9_2, ("a3", "b1", "b2"), 6000.0),
    "T_high": 20_000.0,
}
NASA9 = [NASA9_1, NASA9_2, NASA9_3]

SHOMATE_1 = dict(T_low=298.0, T_high=1200.0, A=30.0, B=8.0e-3, C=-2.0e-6, D=3.0e-10, E=-1.5e5, F=-250_000.0, G=210.0)
SHOMATE_2 = {
    **join(shomate_values, dict(B=2.0e-3, C=-4.0e-7, D=5.0e-11, E=2.0e6), SHOMATE_1, ("A", "F", "G"), 1200.0),
    "T_high": 6000.0,
}
SHOMATE = [SHOMATE_1, SHOMATE_2]

FORMS = {
    "nasa7": (NASA7, nasa7_values, "nasa7.pure", 1),
    "nasa9": (NASA9, nasa9_values, "nasa9.pure", 1),
    "shomate": (SHOMATE, shomate_values, "shomate.pure", 0),
}


def piece_index(pieces: list[dict[str, float]], T: np.ndarray) -> np.ndarray:
    """The piece each temperature belongs to: half-open intervals, the last one closed."""
    inner = np.array([p["T_low"] for p in pieces[1:]])
    return np.searchsorted(inner, T, side="right")


def expected(name: str, T: np.ndarray, gas: float = R, *, lower: bool = False) -> tuple[np.ndarray, ...]:
    """Cp, H and S of a form at `T` from its pieces; at a boundary the upper piece, or the lower
    one under the other reading."""
    pieces, values, _, _ = FORMS[name]
    T = np.asarray(T, dtype=float)
    if lower:
        inner = np.array([p["T_high"] for p in pieces[:-1]])
        chosen = np.searchsorted(inner, T, side="left")
    else:
        chosen = piece_index(pieces, T)
    out = [np.empty_like(T) for _ in range(3)]
    for index, piece in enumerate(pieces):
        mask = chosen == index
        if mask.any():
            for target, value in zip(out, values(piece, T[mask], gas), strict=True):
                target[mask] = value
    return tuple(out)


# the evaluated table: the fit rounded as a printed table rounds it, and formation data made up
TABLE_T = [0.0, 200.0, 298.15, 500.0, 1000.0, 1500.0, 2000.0]


def formation_enthalpy(T: float) -> float:  # kJ/mol
    return -250.1 + 1.1e-3 * (T - T_REF)


def formation_gibbs(T: float) -> float:  # kJ/mol
    return -231.4 + 4.8e-2 * (T - T_REF)


def table_row(T: float) -> dict[str, float | None]:
    """One printed row in the units the table states, in J/(mol K) and kJ/mol; 0 K has values for
    Cp, S, H - H(Tr) and the formation quantities, and INFINITE (no value) where the quantity has
    1/T in it."""
    if T == 0.0:
        return dict(cp=0.0, s=0.0, g_function=None, dh=-9.905, dfh=-245.3, dfg=-245.3, log_kf=None)
    cp, h, s = (float(x[0]) for x in expected("nasa7", np.array([T])))
    h_ref = float(expected("nasa7", np.array([T_REF]))[1][0])
    dh = (h - h_ref) / 1000.0
    return dict(
        cp=round(cp, 3),
        s=round(s, 3),
        g_function=round(s - dh * 1000.0 / T, 3),
        dh=round(dh, 3),
        dfh=round(formation_enthalpy(T), 3),
        dfg=round(formation_gibbs(T), 3),
        log_kf=round(-formation_gibbs(T) * 1000.0 / (R_TABLE * T * math.log(10)), 3),
    )


# -- the fixture -------------------------------------------------------------------------------

COLUMNS = (  # (key, role, observable, unit)
    ("T", "variable", "temperature", "K"),
    ("cp", "property", "standard_molar_heat_capacity_isobaric", "J/(mol*K)"),
    ("s", "property", "standard_molar_entropy", "J/(mol*K)"),
    ("g_function", "property", "standard_gibbs_energy_function", "J/(mol*K)"),
    ("dh", "property", "standard_molar_enthalpy", "kJ/mol"),
    ("dfh", "property", "molar_enthalpy_of_formation", "kJ/mol"),
    ("dfg", "property", "molar_gibbs_energy_of_formation", "kJ/mol"),
    ("log_kf", "property", "log10_equilibrium_constant", "dimensionless"),
)


@dataclass
class World:
    decl: Declaration
    database: TestDatabase
    ids: dict[str, uuid.UUID]


def write_species(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    """Water and its elements in the gas phase, with their composition, and the formation reaction
    the table's last column is about."""
    gas = entity_id(decl, "aggregation", "gas")
    hydrogen, oxygen = entity_id(decl, "element", "H"), entity_id(decl, "element", "O")
    forms: dict[str, uuid.UUID] = {}
    for name, composition in (
        ("H2", {hydrogen: 2}),
        ("O2", {oxygen: 2}),
        ("H2O", {hydrogen: 2, oxygen: 1}),
    ):
        species = w.kind("species", {"canonical_key": name, "label": name}, origins=at(f"s-{name}"))
        forms[name] = w.kind(
            "species_form",
            {"canonical_key": f"{name} gas", "label": name, "species": species, "aggregation": gas},
            origins=at(f"f-{name}"),
        )
        for quantity, amount in composition.items():
            w.relation("composition", {"entity": species, "quantity": quantity}, {"value": amount}, at="a.json#/c")
    formation = w.kind(
        "reaction", {"canonical_key": "2 H2 + O2 = 2 H2O", "extent": "as_written"}, origins=at("formation")
    )
    for name, coefficient in (("H2", -2), ("O2", -1), ("H2O", 2)):
        w.relation(
            "reaction_participant",
            {"reaction": formation, "form": forms[name]},
            {"coefficient": coefficient},
            at="a.json#/p",
        )
    ids.update({f"form_{name}": found for name, found in forms.items()}, formation=formation)


def write_conventions(w: CanonicalWriter, ids: dict[str, uuid.UUID]) -> None:
    """The energy reference of formation data, and the two convention sets that assume it: the
    one of the coefficient sets and the one of the table, which differ in the gas constant."""
    reference = w.kind(
        "energy_reference",
        {
            "key": "formation-298",
            "enthalpy": "formation_from_elements",
            "entropy": "third_law",
            "temperature": Quantity(T_REF, "K"),
            "pressure": Quantity(1.0, "bar"),
        },
        origins=at("energy-reference"),
    )
    ids["energy_reference"] = reference
    for key, gas in (("coefficients", R), ("table", R_TABLE)):
        ids[f"conventions_{key}"] = w.kind(
            "convention_set",
            {
                "key": key,
                "revision": "1",
                "temperature_scale": "its_90",
                "energy_reference": reference,
                "gas_constant": Quantity(gas, "J/(mol*K)"),
            },
            origins=at(f"conventions-{key}"),
        )


def write_sets(w: CanonicalWriter, ids: dict[str, uuid.UUID]) -> None:
    """One parameterisation with three parameter sets of the species form, one per form."""
    p = w.kind(
        "parameterization",
        {
            "key": "thermochemistry",
            "revision": "1",
            "title": "Standard-state thermochemistry of water, three ways",
            "coherence": "independent_records",
            "convention_set": ids["conventions_coefficients"],
        },
        origins=at("parameterization"),
    )
    ids["parameterization"] = p
    subject = [ids["form_H2O"]]
    ids["set_nasa7"] = w.parameter_set(
        parameterization=p,
        slot_group="nasa7.pure",
        subjects=subject,
        slots={},
        families={"piece": [FamilyRow({"n": n}, nasa7_row(c)) for n, c in enumerate(NASA7, 1)]},
        origins=at("nasa7"),
    )
    ids["set_nasa9"] = w.parameter_set(
        parameterization=p,
        slot_group="nasa9.pure",
        subjects=subject,
        slots={},
        families={"piece": [FamilyRow({"n": n}, nasa9_row(c)) for n, c in enumerate(NASA9, 1)]},
        origins=at("nasa9"),
    )
    ids["set_shomate"] = w.parameter_set(
        parameterization=p,
        slot_group="shomate.pure",
        subjects=subject,
        slots={},
        families={
            "piece": [
                FamilyRow(
                    {"n": n},
                    {
                        "T_low": Quantity(c["T_low"], "K"),
                        "T_high": Quantity(c["T_high"], "K"),
                        "A": Quantity(c["A"], "J/(mol*K)"),
                        "B": Quantity(c["B"], "J/(mol*K^2)"),
                        "C": Quantity(c["C"], "J/(mol*K^3)"),
                        "D": Quantity(c["D"], "J/(mol*K^4)"),
                        "E": Quantity(c["E"], "J*K/mol"),
                        "F": Quantity(c["F"] / 1000.0, "kJ/mol"),  # the source states kJ/mol
                        "G": Quantity(c["G"], "J/(mol*K)"),
                    },
                )
                for n, c in enumerate(SHOMATE, 1)
            ]
        },
        origins=at("shomate"),
    )


def write_table(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    """The evaluated table as a dataset: the species form is its component, ideal gas at one bar
    its standard state, H - H(Tr) a difference from the same phase at Tr, and log Kf the
    observable of the formation reaction. The divergent cells of the 0 K row have no datum."""
    evaluated = at("janaf", "evaluated")
    dataset = w.kind(
        "dataset",
        {
            "carrier": CARRIER,
            "local_key": "h2o-gas",
            "kind": "evaluated",
            "convention_set": ids["conventions_table"],
            "reaction": ids["formation"],
        },
        origins=evaluated,
    )
    ids["dataset"] = dataset
    component = w.kind(
        "dataset_component",
        {"dataset": dataset, "ordinal": 1, "entity": ids["form_H2O"], "function": "component"},
        at="a.json#/janaf",
    )
    gas_phase = w.kind(
        "dataset_phase",
        {"dataset": dataset, "ordinal": 1, "aggregation": entity_id(decl, "aggregation", "gas")},
        at="a.json#/janaf",
    )
    ideal_gas = w.kind(
        "standard_state",
        {
            "key": "ideal-gas-1-bar",
            "kind": "pure_ideal_gas",
            "pressure_rule": "fixed",
            "pressure": Quantity(1.0, "bar"),
        },
        origins=at("standard-state"),
    )
    columns: dict[str, uuid.UUID] = {}
    for ordinal, (key, role, observable, _) in enumerate(COLUMNS, 1):
        attributes: dict[str, object] = {
            "dataset": dataset,
            "ordinal": ordinal,
            "role": role,
            "observable": observable_id(decl, observable),
        }
        if key != "T":
            attributes.update(component=component, phase=gas_phase)
        if key in ("cp", "s", "g_function", "dh"):
            attributes["standard_state"] = ideal_gas
        if key == "dh":  # H - H(Tr): the enthalpy against the same phase at Tr
            attributes.update(
                presentation="difference_from_reference",
                reference_state_kind="reference_phase_fixed_t_same_p",
                reference_temperature=Quantity(T_REF, "K"),
                reference_phase=gas_phase,
            )
        columns[key] = w.kind("dataset_column", attributes, at="a.json#/janaf")
    ids.update({f"column_{key}": found for key, found in columns.items()})
    for index, T in enumerate(TABLE_T, 1):
        point = w.kind("data_point", {"dataset": dataset, "index": index}, at="a.json#/janaf")
        ids[f"point_{index}"] = point
        row = {"T": T, **table_row(T)}
        for key, _, _, unit in COLUMNS:
            value = row[key]
            if value is None:
                continue  # INFINITE: the source states a divergence, and no datum says one
            w.relation(
                "datum",
                {"point": point, "column": columns[key]},
                {"state": "known", "value": Quantity(value, unit), "digits": 8},
                at="a.json#/janaf",
            )
    # the table and the NASA-7 fit give the same Cp, H and S to the rounding of the table
    w.relation(
        "equivalence_assessment",
        {"a": dataset, "b": ids["set_nasa7"]},
        {"level": "within_rounding"},
    )


def fill(decl: Declaration, ids: dict[str, uuid.UUID]):  # noqa: ANN201
    def emit(w: CanonicalWriter) -> None:
        write_species(w, decl, ids)
        write_conventions(w, ids)
        write_sets(w, ids)
        write_table(w, decl, ids)

    return emit


@pytest.fixture(scope="module")
def decl() -> Declaration:
    return real_declaration()


@pytest.fixture(scope="module")
def world(decl: Declaration, tmp_path_factory: pytest.TempPathFactory) -> Iterator[World]:
    ids: dict[str, uuid.UUID] = {}
    database = build(tmp_path_factory.mktemp("thermochemistry"), fill(decl, ids), decl)
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


def bound(world: World, conn: psycopg.Connection, form: str, policy: PiecePolicy | None = None):  # noqa: ANN201
    source = DatabaseSource(conn, world.decl, [world.ids["parameterization"]])
    return bind(world.decl, form, source=source, roles={"i": str(world.ids["form_H2O"])}, pieces=policy, cache=CACHE)


def evaluate(world: World, conn: psycopg.Connection, form: str, T: np.ndarray, policy: PiecePolicy | None = None):  # noqa: ANN201
    found = bound(world, conn, form, policy)
    return tuple(np.asarray(found.evaluate(name, T=T), dtype=float).reshape(-1) for name in ("cp", "h", "s"))


def scalar(conn: psycopg.Connection, query: str, *params: object) -> object:
    row = conn.execute(query, params).fetchone()  # type: ignore[arg-type]
    assert row is not None
    return row[0]


# -- the structure ------------------------------------------------------------------------------


def test_the_fixture_satisfies_every_invariant(conn: psycopg.Connection) -> None:
    assert failing(conn) == {}


def test_one_species_form_has_three_parameter_sets_in_one_parameterisation(
    world: World, conn: psycopg.Connection
) -> None:
    """The three forms describe one subject: sets of different slot groups coexist, each with its
    own pieces."""
    counts = {
        form: scalar(
            conn,
            f'SELECT count(*) FROM param."{form}__pure__piece" c JOIN tk.parameter_set s ON s.id = c.set_id '
            "WHERE s.parameterization = %s",
            world.ids["parameterization"],
        )
        for form in ("nasa7", "nasa9", "shomate")
    }
    assert counts == {"nasa7": 2, "nasa9": 3, "shomate": 2}
    subjects = {
        scalar(conn, f'SELECT i FROM param."{form}__pure"')
        for form in ("nasa7", "nasa9", "shomate")
    }
    assert subjects == {world.ids["form_H2O"]}


def test_the_convention_set_states_its_energy_reference_and_gas_constant(
    world: World, conn: psycopg.Connection
) -> None:
    row = conn.execute(
        "SELECT r.enthalpy::text, r.entropy::text, r.temperature, r.pressure, c.gas_constant "
        "FROM tk.convention_set c JOIN tk.energy_reference r ON r.id = c.energy_reference "
        "WHERE c.id = %s",
        (world.ids["conventions_coefficients"],),
    ).fetchone()
    assert row == ("formation_from_elements", "third_law", pytest.approx(T_REF), pytest.approx(1e5), pytest.approx(R))
    assert scalar(conn, "SELECT gas_constant FROM tk.convention_set WHERE id = %s", world.ids["conventions_table"]) == pytest.approx(R_TABLE)


def test_the_pieces_are_stored_in_kelvin_and_shomate_constants_in_coherent_units(
    world: World, conn: psycopg.Connection
) -> None:
    bounds = conn.execute(
        'SELECT "T_low", "T_high" FROM param."nasa9__pure__piece" ORDER BY "n"'
    ).fetchall()
    assert bounds == [(200.0, 1000.0), (1000.0, 6000.0), (6000.0, 20_000.0)]
    stored = scalar(conn, 'SELECT "F" FROM param."shomate__pure__piece" WHERE "n" = 1')
    assert stored == pytest.approx(SHOMATE_1["F"])  # kJ/mol in, J/mol stored


# -- evaluation against numpy -------------------------------------------------------------------

POINTS = {
    "nasa7": [200.0, 250.0, 600.0, 999.0, 1000.0, 1001.0, 3000.0, 6000.0],
    "nasa9": [200.0, 250.0, 900.0, 1000.0, 2500.0, 5999.0, 6000.0, 6500.0, 15_000.0, 20_000.0],
    "shomate": [298.0, 298.15, 1000.0, 1199.0, 1200.0, 1201.0, 5000.0, 6000.0],
}


@pytest.mark.parametrize("form", ["nasa7", "nasa9", "shomate"])
def test_each_form_matches_the_independent_calculation_in_every_piece(
    world: World, conn: psycopg.Connection, form: str
) -> None:
    T = np.array(POINTS[form])
    cp, h, s = evaluate(world, conn, form, T)
    want_cp, want_h, want_s = expected(form, T)
    np.testing.assert_allclose(cp, want_cp, rtol=1e-12)
    np.testing.assert_allclose(h, want_h, rtol=1e-11, atol=1e-6)
    np.testing.assert_allclose(s, want_s, rtol=1e-12)


@pytest.mark.parametrize("form", ["nasa7", "nasa9", "shomate"])
def test_the_pieces_join_continuously_at_every_common_boundary(
    world: World, conn: psycopg.Connection, form: str
) -> None:
    """Cp, H and S of the piece below and of the piece above agree at each boundary: in numpy, for
    the stored coefficients as evaluated by the library under each reading of the boundary, and
    a hair either side."""
    pieces = FORMS[form][0]
    for below, above in zip(pieces, pieces[1:], strict=False):
        boundary = np.array([below["T_high"]])
        assert above["T_low"] == below["T_high"]
        lower = evaluate(world, conn, form, boundary, PiecePolicy(boundary="lower_piece"))
        upper = evaluate(world, conn, form, boundary, PiecePolicy(boundary="upper_piece"))
        want_below = expected(form, boundary, lower=True)
        want_above = expected(form, boundary)
        for quantity in range(3):
            np.testing.assert_allclose(want_below[quantity], want_above[quantity], rtol=1e-12, atol=1e-7)
            np.testing.assert_allclose(lower[quantity], want_below[quantity], rtol=1e-12, atol=1e-7)
            np.testing.assert_allclose(upper[quantity], want_above[quantity], rtol=1e-12, atol=1e-7)
            np.testing.assert_allclose(lower[quantity], upper[quantity], rtol=1e-11, atol=1e-6)
        nearby = evaluate(world, conn, form, boundary * np.array([1 - 1e-9]))
        beyond = evaluate(world, conn, form, boundary * np.array([1 + 1e-9]))
        for quantity in range(3):
            np.testing.assert_allclose(nearby[quantity], beyond[quantity], rtol=1e-6, atol=1e-3)


def test_the_enthalpy_is_not_continuous_across_a_boundary_the_data_did_not_join() -> None:
    """The control of the continuity test: pieces that were not joined differ at the boundary."""
    unjoined = {**NASA7_HIGH, "a6": NASA7_HIGH["a6"] + 500.0}
    at_boundary = np.array([1000.0])
    assert abs(nasa7_values(unjoined, at_boundary)[1][0] - nasa7_values(NASA7_LOW, at_boundary)[1][0]) > 1000.0


def test_the_gas_constant_is_the_convention_sets_not_the_evaluators(
    world: World, conn: psycopg.Connection
) -> None:
    """NASA Cp/R coefficients reproduce their source only with the gas constant they were built
    with: the value the convention set states is what the evaluation multiplies by."""
    cp, _, _ = evaluate(world, conn, "nasa7", np.array([600.0]))
    assert cp[0] == pytest.approx(float(expected("nasa7", np.array([600.0]), R)[0][0]), rel=1e-13)
    assert cp[0] != pytest.approx(float(expected("nasa7", np.array([600.0]), R_TABLE)[0][0]), rel=1e-7)


# -- the evaluated table ------------------------------------------------------------------------


def datum(conn: psycopg.Connection, world: World, index: int, key: str) -> tuple[str, float | None] | None:
    row = conn.execute(
        "SELECT state::text, value FROM ev.datum WHERE point = %s AND \"column\" = %s",
        (world.ids[f"point_{index}"], world.ids[f"column_{key}"]),
    ).fetchone()
    return None if row is None else (row[0], row[1])


def test_the_table_is_an_evaluated_dataset_with_typed_columns(world: World, conn: psycopg.Connection) -> None:
    kind, convention = conn.execute(
        "SELECT kind::text, convention_set FROM ev.dataset WHERE id = %s", (world.ids["dataset"],)
    ).fetchone()  # type: ignore[misc]
    assert kind == "evaluated" and convention == world.ids["conventions_table"]
    columns = conn.execute(
        "SELECT c.role::text, o.key, c.presentation::text FROM ev.dataset_column c "
        "JOIN tk.observable o ON o.id = c.observable WHERE c.dataset = %s ORDER BY c.ordinal",
        (world.ids["dataset"],),
    ).fetchall()
    assert [(role, key) for role, key, _ in columns] == [(role, observable) for _, role, observable, _ in COLUMNS]
    assert [presentation for _, _, presentation in columns].count("difference_from_reference") == 1


def test_the_enthalpy_increment_is_a_difference_from_the_same_phase_at_the_reference_temperature(
    world: World, conn: psycopg.Connection
) -> None:
    row = conn.execute(
        "SELECT reference_state_kind::text, reference_temperature, reference_phase, phase, standard_state "
        'FROM ev.dataset_column WHERE id = %s',
        (world.ids["column_dh"],),
    ).fetchone()
    assert row is not None
    kind, reference_temperature, reference_phase, phase, standard_state = row
    assert kind == "reference_phase_fixed_t_same_p"
    assert reference_temperature == pytest.approx(T_REF)
    assert reference_phase == phase and standard_state is not None


def test_the_row_at_zero_kelvin_has_no_datum_where_the_source_says_infinite(
    world: World, conn: psycopg.Connection
) -> None:
    """Two cells of the 0 K row are INFINITE in the source: the Gibbs energy function and log Kf
    both carry 1/T. The representation is the absence of the datum row, which means "not
    asserted", while `not_measured` would say the value was sought and not found. No divergent
    datum state is needed: nothing a consumer can use is lost, because a divergence is not a
    number to interpolate, and the sign, which the file does not give, follows from the formation
    Gibbs energy in the same row."""
    assert datum(conn, world, 1, "g_function") is None
    assert datum(conn, world, 1, "log_kf") is None
    assert datum(conn, world, 1, "cp") == ("known", 0.0)
    assert datum(conn, world, 1, "dfg") == ("known", pytest.approx(-245_300.0))
    states = {row[0] for row in conn.execute("SELECT DISTINCT state::text FROM ev.datum")}
    assert states == {"known"}
    rows = scalar(conn, "SELECT count(*) FROM ev.datum")
    assert rows == len(COLUMNS) * len(TABLE_T) - 2
    # the sign of the divergence: the formation Gibbs energy is negative, so log Kf -> +infinity
    assert scalar(conn, 'SELECT value FROM ev.datum WHERE point = %s AND "column" = %s', world.ids["point_1"], world.ids["column_dfg"]) < 0  # type: ignore[operator]


def test_the_table_values_are_the_printed_ones_in_coherent_units(world: World, conn: psycopg.Connection) -> None:
    index = TABLE_T.index(500.0) + 1
    row = table_row(500.0)
    assert datum(conn, world, index, "cp") == ("known", pytest.approx(row["cp"]))
    assert datum(conn, world, index, "dh") == ("known", pytest.approx(row["dh"] * 1000.0))
    assert datum(conn, world, index, "log_kf") == ("known", pytest.approx(row["log_kf"]))


def test_log_kf_follows_from_the_formation_gibbs_energy_with_the_tables_gas_constant(
    world: World, conn: psycopg.Connection
) -> None:
    for index, T in enumerate(TABLE_T, 1):
        if T == 0.0:
            continue
        state, stored = datum(conn, world, index, "log_kf")  # type: ignore[misc]
        _, dfg = datum(conn, world, index, "dfg")  # type: ignore[misc]
        assert stored == pytest.approx(-dfg / (R_TABLE * T * math.log(10)), abs=5.1e-4)


def test_the_gibbs_energy_function_follows_from_entropy_and_enthalpy_increment(
    world: World, conn: psycopg.Connection
) -> None:
    for index, T in enumerate(TABLE_T, 1):
        if T == 0.0:
            continue
        (_, s), (_, dh), (_, g) = (datum(conn, world, index, key) for key in ("s", "dh", "g_function"))  # type: ignore[misc]
        # the table prints dh to 1 J/mol after rounding kJ/mol to three places: its share of the function
        assert g == pytest.approx(s - dh / T, abs=0.0011 + 0.5 / T)


def test_the_table_and_the_nasa7_fit_agree_to_the_rounding_of_the_table(
    world: World, conn: psycopg.Connection
) -> None:
    """The justification of the `within_rounding` assessment: at every tabulated temperature the
    fit, evaluated by the library from the stored set, reproduces the table's Cp and S to half
    a unit of the last printed digit, and H - H(Tr) likewise."""
    rows = [(index, T) for index, T in enumerate(TABLE_T, 1) if T >= NASA7_LOW["T_low"]]
    T = np.array([T for _, T in rows])
    cp, h, s = evaluate(world, conn, "nasa7", np.concatenate([T, [T_REF]]))
    h_ref = h[-1]
    for position, (index, _) in enumerate(rows):
        assert abs(datum(conn, world, index, "cp")[1] - cp[position]) <= 5.01e-4  # type: ignore[index]
        assert abs(datum(conn, world, index, "s")[1] - s[position]) <= 5.01e-4  # type: ignore[index]
        assert abs(datum(conn, world, index, "dh")[1] - (h[position] - h_ref)) <= 0.5001  # J/mol, printed in kJ/mol to 3 places


def test_the_assessment_is_stored_once_in_the_canonical_orientation(
    world: World, conn: psycopg.Connection
) -> None:
    rows = conn.execute("SELECT a, b, level::text, conversion FROM prov.equivalence_assessment").fetchall()
    assert len(rows) == 1
    a, b, level, conversion = rows[0]
    assert {a, b} == {world.ids["dataset"], world.ids["set_nasa7"]}
    assert a < b  # the symmetric rule stores the record with the smaller identifier first
    assert (level, conversion) == ("within_rounding", None)


# -- what the model refuses ---------------------------------------------------------------------


def test_pieces_that_overlap_on_the_temperature_axis_are_refused(decl: Declaration) -> None:
    w = writer(decl)
    p, subject = species_and_parameterization(w, decl)
    overlapping = [
        FamilyRow({"n": 1}, nasa7_row(NASA7_LOW)),
        FamilyRow({"n": 2}, nasa7_row({**NASA7_HIGH, "T_low": 900.0})),
    ]
    with pytest.raises(ValidationError, match="pieces overlap"):
        w.parameter_set(
            parameterization=p, slot_group="nasa7.pure", subjects=[subject], slots={},
            families={"piece": overlapping}, origins=at("overlap"),
        )
    assert w.rows("param.nasa7__pure") == 0


def test_a_piece_whose_lower_bound_is_not_below_its_upper_bound_is_refused(decl: Declaration) -> None:
    w = writer(decl)
    p, subject = species_and_parameterization(w, decl)
    with pytest.raises(ValidationError, match="is not below"):
        w.parameter_set(
            parameterization=p, slot_group="nasa7.pure", subjects=[subject], slots={},
            families={"piece": [FamilyRow({"n": 1}, nasa7_row({**NASA7_LOW, "T_high": 200.0}))]},
            origins=at("empty-piece"),
        )


def test_a_datum_states_a_value_exactly_when_it_reports_one(decl: Declaration) -> None:
    """The infinite cell cannot be written as a known datum with no value, nor as a measured-state
    datum with one: the only representation is no row."""
    w = writer(decl)
    ids: dict[str, uuid.UUID] = {}
    write_species(w, decl, ids)
    write_conventions(w, ids)
    dataset = w.kind(
        "dataset",
        {"carrier": CARRIER, "local_key": "x", "kind": "evaluated"},
        origins=at("x", "evaluated"),
    )
    column = w.kind(
        "dataset_column",
        {"dataset": dataset, "ordinal": 1, "role": "property", "observable": observable_id(decl, "log10_equilibrium_constant")},
        at="a.json#/x",
    )
    point = w.kind("data_point", {"dataset": dataset, "index": 1}, at="a.json#/x")
    for state, value, message in (
        ("known", None, "value is absent but state is `known`"),
        ("not_measured", Quantity(1.0, "dimensionless"), "value is present but state is `not_measured`"),
    ):
        with pytest.raises(ValidationError, match=message):
            w.relation("datum", {"point": point, "column": column}, {"state": state, "value": value}, at="a.json#/x")
    assert w.rows("ev.datum") == 0


def test_an_assessment_that_names_a_conversion_at_a_level_that_has_none_is_refused(decl: Declaration) -> None:
    w = writer(decl)
    with pytest.raises(ValidationError, match="conversion_iff_under_conversion"):
        w.relation(
            "equivalence_assessment",
            {"a": uuid.uuid4(), "b": uuid.uuid4()},
            {"level": "within_rounding", "conversion": uuid.uuid4()},
        )


def with_units(c: dict[str, float], units: dict[str, str]) -> dict[str, object]:
    return {"T_low": Quantity(c["T_low"], "K"), "T_high": Quantity(c["T_high"], "K"),
            **{name: Quantity(c[name], unit) for name, unit in units.items()}}


def nasa7_row(c: dict[str, float]) -> dict[str, object]:
    """The coefficients of a NASA-7 piece with the unit each has in kelvin: Cp/R is a polynomial
    in T, so a_k has the unit K^-(k-1)."""
    return with_units(
        c,
        {"a1": "dimensionless", "a2": "1/K", "a3": "1/K^2", "a4": "1/K^3", "a5": "1/K^4", "a6": "K", "a7": "dimensionless"},
    )


def nasa9_row(c: dict[str, float]) -> dict[str, object]:
    return with_units(
        c,
        {"a1": "K^2", "a2": "K", "a3": "dimensionless", "a4": "1/K", "a5": "1/K^2", "a6": "1/K^3",
         "a7": "1/K^4", "b1": "K", "b2": "dimensionless"},
    )


def species_and_parameterization(w: CanonicalWriter, decl: Declaration) -> tuple[uuid.UUID, uuid.UUID]:
    ids: dict[str, uuid.UUID] = {}
    write_species(w, decl, ids)
    write_conventions(w, ids)
    p = w.kind(
        "parameterization",
        {"key": "p", "revision": "1", "title": "p", "coherence": "independent_records",
         "convention_set": ids["conventions_coefficients"]},
        origins=at("p"),
    )
    return p, ids["form_H2O"]


def test_the_checks_of_verify_flag_a_missing_gas_constant_a_missing_reference_and_an_unbalanced_reaction(
    decl: Declaration, tmp_path: Path
) -> None:
    """Three violations of the structure the case is about, each found by the check that states
    it: a NASA parameterisation whose convention set does not state the gas constant, a column
    presented against a reference state with no reference state kind, and a formation reaction
    that does not balance."""

    ids: dict[str, uuid.UUID] = {}

    def emit(w: CanonicalWriter) -> None:
        write_species(w, decl, ids)
        bare = w.kind(
            "convention_set",
            {"key": "bare", "revision": "1", "temperature_scale": "its_90"},
            origins=at("bare"),
        )
        p = w.kind(
            "parameterization",
            {"key": "bare", "revision": "1", "title": "bare", "coherence": "independent_records",
             "convention_set": bare},
            origins=at("bare-p"),
        )
        w.parameter_set(
            parameterization=p, slot_group="nasa7.pure", subjects=[ids["form_H2O"]], slots={},
            families={"piece": [FamilyRow({"n": 1}, nasa7_row(NASA7_LOW))]}, origins=at("bare-nasa7"),
        )
        unbalanced = w.kind(
            "reaction", {"canonical_key": "2 H2 + O2 = H2O", "extent": "as_written"}, origins=at("unbalanced")
        )
        for name, coefficient in (("H2", -2), ("O2", -1), ("H2O", 1)):
            w.relation(
                "reaction_participant",
                {"reaction": unbalanced, "form": ids[f"form_{name}"]},
                {"coefficient": coefficient},
                at="a.json#/p",
            )
        dataset = w.kind(
            "dataset",
            {"carrier": CARRIER, "local_key": "no-reference", "kind": "evaluated"},
            origins=at("no-reference", "evaluated"),
        )
        w.kind(
            "dataset_column",
            {
                "dataset": dataset, "ordinal": 1, "role": "property",
                "observable": observable_id(decl, "standard_molar_enthalpy"),
                "presentation": "difference_from_reference",
            },
            at="a.json#/no-reference",
        )

    database = build(tmp_path, emit, decl)
    try:
        with psycopg.connect(database.url) as connection:
            assert failing(connection) == {
                "parameterization_has_conventions": 1,
                "dataset_column.reference_state_kind_when_relative": 1,
                "reaction.conserves_declared_quantities": 2,  # hydrogen and oxygen
            }
            source = DatabaseSource(connection, decl, [scalar(connection, "SELECT id FROM tk.parameterization")])  # type: ignore[list-item]
            found = bind(decl, "nasa7", source=source, roles={"i": str(ids["form_H2O"])})
            with pytest.raises(EvaluationRefusal, match="gas_constant"):
                found.evaluate("cp", T=np.array([300.0]))
    finally:
        database.remove()
