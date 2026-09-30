# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Hard case (e), plan 24 packet TK2: a CALPHAD sublattice phase with piecewise parameters.

A chemical system with an ordered phase (two substitutional site classes and a vacancy class) and
the disordered phase it is the ordered form of (`disordered_partner`), a liquid with one site class,
site classes with their ratios and admissible occupants, and constituent arrays whose species keep
the order the source asserted. Endmember energies are named temperature functions in half-open
pieces that the endmembers of several phases reference; a Redlich-Kister interaction of two species
is carried by an array and two arrays that assert the species in opposite order carry opposite
odd-order terms for one physical interaction (alignment item 14); the binary of a pair of species
is held once with parity, and its odd terms change sign when it is read the other way; the magnetic
contribution has a Curie temperature, a moment and the structure factor; and two ternary subsystems
choose the Kohler and the Toop extrapolation, the Toop form singling out the species the array puts
first, which it reads from the array's member positions.

Every number is synthetic. Each form is evaluated at points in every piece and against the
published formulas written in numpy here.
"""

from __future__ import annotations

import uuid
from collections.abc import Iterator
from dataclasses import dataclass
from pathlib import Path

import numpy as np
import psycopg
import pytest
from hard_case_support import CHECKS, at, build, entity_id, failing
from mapping_support import real_declaration, writer
from mechanisms_support import broken, replace

from thermo_knowledge import db
from thermo_knowledge.canonical.values import Quantity
from thermo_knowledge.canonical.writer import (
    CanonicalWriter,
    FamilyRow,
    SetReference,
    ValidationError,
)
from thermo_knowledge.declaration import Declaration
from thermo_knowledge.declaration.diagnostics import Code
from thermo_knowledge.expression.compiled import CompileCache
from thermo_knowledge.expression.evaluate import EvaluationRefusal, bind
from thermo_knowledge.qualify.source import DatabaseSource, SubformBinding
from thermo_knowledge.testing import TestDatabase

CACHE = CompileCache()
R = 8.314462618  # the gas constant of the parameterisation's convention set, J/(mol K)
ELEMENTS = ("Fe", "Cr", "Ni", "Mo")
SPECIES = (*ELEMENTS, "Va")

# -- the independent calculation ---------------------------------------------------------------

UNITS = {
    "a": "J/mol",
    "b": "J/(mol*K)",
    "c": "J/(mol*K)",
    "d": "J/(mol*K^2)",
    "e": "J*K/mol",
    "f": "J/(mol*K^3)",
    "g": "J/(mol*K^7)",
    "h": "J*K^9/mol",
}


def polynomial(c: dict[str, float], T: np.ndarray) -> np.ndarray:
    """The temperature polynomial of one piece, written term by term."""
    return (
        c["a"] + c["b"] * T + c["c"] * T * np.log(T) + c["d"] * T**2 + c["e"] / T
        + c["f"] * T**3 + c["g"] * T**7 + c["h"] / T**9
    )


FUNCTIONS = {
    "fe_bcc": [
        dict(T_low=298.15, T_high=1000.0, a=-8000.0, b=130.0, c=-24.0, d=-4.0e-3, e=2.5e5, f=1.0e-7, g=0.0, h=0.0),
        dict(T_low=1000.0, T_high=3000.0, a=-12000.0, b=170.0, c=-28.0, d=1.0e-3, e=-1.0e6, f=0.0, g=1.0e-21, h=1.0e29),
    ],
    "cr_bcc": [
        dict(T_low=298.15, T_high=2000.0, a=-9500.0, b=120.0, c=-22.0, d=-3.0e-3, e=2.0e5, f=2.0e-8, g=0.0, h=0.0),
        dict(T_low=2000.0, T_high=3500.0, a=-30000.0, b=260.0, c=-38.0, d=2.0e-3, e=0.0, f=0.0, g=0.0, h=0.0),
    ],
    "fe_cr_b2": [
        dict(T_low=298.15, T_high=700.0, a=-4000.0, b=40.0, c=-7.0, d=-1.0e-3, e=0.0, f=0.0, g=0.0, h=0.0),
        dict(T_low=700.0, T_high=1500.0, a=-4500.0, b=46.0, c=-8.0, d=0.0, e=1.0e5, f=0.0, g=0.0, h=0.0),
        dict(T_low=1500.0, T_high=3000.0, a=-2000.0, b=30.0, c=-5.5, d=5.0e-4, e=0.0, f=0.0, g=0.0, h=0.0),
    ],
}


def function_value(name: str, T: np.ndarray) -> np.ndarray:
    """The function at `T`: the piece whose half-open interval holds it (the last piece is closed)."""
    pieces = FUNCTIONS[name]
    T = np.asarray(T, dtype=float)
    chosen = np.searchsorted([p["T_low"] for p in pieces[1:]], T, side="right")
    out = np.empty_like(T)
    for index, piece in enumerate(pieces):
        mask = chosen == index
        if mask.any():
            out[mask] = polynomial(piece, T[mask])
    return out


MAGNETIC = {  # array -> (Tc in K, beta in Bohr magnetons): a negative value is an antiferromagnetic one
    "endmember_fe": (1043.0, 2.22),
    "endmember_cr": (-311.5, -0.25),
}
AFM, P_STRUCTURE = -1.0, 0.4


def magnetic(tc: float, beta: float, afm: float, p: float, T: np.ndarray) -> np.ndarray:
    """The Inden-Hillert-Jarl contribution, from the published function."""
    tc = tc / afm if tc < 0 else tc
    beta = beta / afm if beta < 0 else beta
    tau = np.asarray(T, dtype=float) / tc
    big_a = 518.0 / 1125.0 + 11692.0 / 15975.0 * (1.0 / p - 1.0)
    with np.errstate(over="ignore", invalid="ignore"):
        below = 1.0 - (
            79.0 / (140.0 * p * tau) + 474.0 / 497.0 * (1.0 / p - 1.0) * (tau**3 / 6 + tau**9 / 135 + tau**15 / 600)
        ) / big_a
        above = -(tau**-5 / 10 + tau**-15 / 315 + tau**-25 / 1500) / big_a
    return R * T * np.log(beta + 1.0) * np.where(tau < 1.0, below, above)


# the binaries as asserted: (i, j) -> the Redlich-Kister coefficients L0, L1, L2 in J/mol
BINARIES = {
    ("Fe", "Cr"): (4000.0, 900.0, -350.0),
    ("Fe", "Ni"): (-2500.0, 400.0, 0.0),
    ("Cr", "Ni"): (1500.0, -700.0, 120.0),
    ("Fe", "Mo"): (3000.0, -300.0, 80.0),
    ("Cr", "Mo"): (-1000.0, 650.0, -40.0),
}


def binary(i: str, j: str, xi: float) -> float:
    """The excess energy of the binary of i and j at the mole fraction xi of i from the pair as
    asserted: the odd-order terms change sign when the pair is read the other way round."""
    if (i, j) in BINARIES:
        coefficients = BINARIES[(i, j)]
    else:
        coefficients = tuple(c * (-1) ** k for k, c in enumerate(BINARIES[(j, i)]))
    z = 2 * xi - 1
    return xi * (1 - xi) * sum(c * z**k for k, c in enumerate(coefficients))


def kohler(names: tuple[str, ...], x: dict[str, float]) -> float:
    return sum(
        (x[i] + x[j]) ** 2 * binary(i, j, x[i] / (x[i] + x[j]))
        for index, i in enumerate(names)
        for j in names[index + 1 :]
    )


def toop(names: tuple[str, ...], x: dict[str, float], special: str) -> float:
    (j, k) = [n for n in names if n != special]
    return (
        sum(x[other] / (1 - x[special]) * binary(special, other, x[special]) for other in (j, k))
        + (x[j] + x[k]) ** 2 * binary(j, k, x[j] / (x[j] + x[k]))
    )


# -- the fixture -------------------------------------------------------------------------------

PHASES = {  # phase -> (aggregation, structure, [(site ratio, occupants)])
    "BCC_A2": ("crystalline", "sublattice", [(1.0, ("Fe", "Cr", "Ni", "Mo")), (3.0, ("Va",))]),
    "BCC_B2": (
        "crystalline",
        "sublattice",
        [(0.5, ("Fe", "Cr", "Ni", "Mo")), (0.5, ("Fe", "Cr", "Ni", "Mo")), (3.0, ("Va",))],
    ),
    "LIQUID": ("liquid", "none", [(1.0, ("Fe", "Cr", "Ni", "Mo"))]),
}
ARRAYS = {  # array -> (phase, species per site class in the asserted order)
    "endmember_fe": ("BCC_A2", [("Fe",), ("Va",)]),
    "endmember_cr": ("BCC_A2", [("Cr",), ("Va",)]),
    "interaction_fecr": ("BCC_A2", [("Fe", "Cr"), ("Va",)]),
    "interaction_crfe": ("BCC_A2", [("Cr", "Fe"), ("Va",)]),
    "b2_fecr": ("BCC_B2", [("Fe",), ("Cr",), ("Va",)]),
    "b2_crfe": ("BCC_B2", [("Cr",), ("Fe",), ("Va",)]),
    "b2_fefe": ("BCC_B2", [("Fe",), ("Fe",), ("Va",)]),
    "kohler": ("LIQUID", [("Fe", "Cr", "Ni")]),
    "toop_fe_first": ("LIQUID", [("Fe", "Cr", "Mo")]),
    "toop_cr_first": ("LIQUID", [("Cr", "Fe", "Mo")]),
}
ENDMEMBER_FUNCTION = {
    "endmember_fe": "fe_bcc",
    "endmember_cr": "cr_bcc",
    "b2_fecr": "fe_cr_b2",
    "b2_crfe": "fe_cr_b2",
}
INTERACTIONS = {"interaction_fecr": (-15000.0, 6000.0), "interaction_crfe": (-15000.0, -6000.0)}
CHOICES = {
    "kohler": "ternary_kohler",
    "toop_fe_first": "ternary_toop",
    "toop_cr_first": "ternary_toop",
}


def array_key(phase: str, classes: list[tuple[str, ...]]) -> str:
    """The canonical key by the declared ordering convention: the phase key, then a colon and the
    species of each site class in position order, the classes in ascending index."""
    return phase + "".join(":" + ",".join(members) for members in classes)


@dataclass
class World:
    decl: Declaration
    database: TestDatabase
    ids: dict[str, uuid.UUID]


def write_system(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    species = {}
    for name in SPECIES:
        species[name] = w.kind("species", {"canonical_key": name, "label": name}, origins=at(f"s-{name}"))
        if name in ELEMENTS:
            w.relation(
                "composition",
                {"entity": species[name], "quantity": entity_id(decl, "element", name)},
                {"value": 1},
                at="a.json#/composition",
            )
    ids.update({f"species_{name}": value for name, value in species.items()})
    system = w.kind("chemical_system", {"key": "fe-cr-ni-mo", "revision": "1"}, origins=at("system"))
    ids["system"] = system
    for name in ELEMENTS:
        w.relation(
            "system_conserves",
            {"system": system, "quantity": entity_id(decl, "element", name)},
            {},
            at="a.json#/conserves",
        )
    classes: dict[tuple[str, int], uuid.UUID] = {}
    for phase, (aggregation, structure, sites) in PHASES.items():
        ids[f"phase_{phase}"] = w.kind(
            "phase_definition",
            {
                "system": system,
                "key": phase,
                "aggregation": entity_id(decl, "aggregation", aggregation),
                "structure": structure,
            },
            origins=at(f"phase-{phase}"),
        )
        for index, (ratio, occupants) in enumerate(sites, start=1):
            site = w.kind(
                "site_class",
                {
                    "phase": ids[f"phase_{phase}"],
                    "index": index,
                    "ratio_kind": "constant",
                    "ratio": ratio,
                },
                origins=at(f"site-{phase}-{index}"),
            )
            classes[(phase, index)] = site
            for occupant in occupants:
                w.relation(
                    "site_occupant", {"site_class": site, "occupant": species[occupant]}, {}, at="a.json#/occupant"
                )
    ids.update({f"site_{phase}_{index}": value for (phase, index), value in classes.items()})
    # the ordered phase is the ordered form of the disordered one: its model is the disordered
    # phase's plus an ordering contribution
    w.relation(
        "disordered_partner",
        {"ordered": ids["phase_BCC_B2"], "disordered": ids["phase_BCC_A2"]},
        {"never_disorder": False},
        at="a.json#/partner",
    )


def write_arrays(w: CanonicalWriter, ids: dict[str, uuid.UUID]) -> None:
    for name, (phase, placed) in ARRAYS.items():
        array = w.kind(
            "constituent_array",
            {"phase": ids[f"phase_{phase}"], "canonical_key": array_key(phase, placed)},
            origins=at(f"array-{name}"),
        )
        ids[f"array_{name}"] = array
        for index, members in enumerate(placed, start=1):
            for position, member in enumerate(members, start=1):
                w.relation(
                    "constituent_array_member",
                    {"array": array, "site_class": ids[f"site_{phase}_{index}"], "position": position},
                    {"species": ids[f"species_{member}"]},
                    at="a.json#/member",
                )


def write_conventions(w: CanonicalWriter, ids: dict[str, uuid.UUID]) -> None:
    reference = w.kind(
        "energy_reference",
        {
            "key": "ser",
            "enthalpy": "stable_element_reference",
            "entropy": "third_law",
            "temperature": Quantity(298.15, "K"),
            "pressure": Quantity(1.0, "bar"),
        },
        origins=at("energy-reference"),
    )
    ids["conventions"] = w.kind(
        "convention_set",
        {
            "key": "calphad",
            "revision": "1",
            "temperature_scale": "its_90",
            "energy_reference": reference,
            "gas_constant": Quantity(R, "J/(mol*K)"),
        },
        origins=at("conventions"),
    )


def piece_row(piece: dict[str, float]) -> dict[str, object]:
    return {
        "T_low": Quantity(piece["T_low"], "K"),
        "T_high": Quantity(piece["T_high"], "K"),
        **{name: Quantity(piece[name], unit) for name, unit in UNITS.items()},
    }


def write_parameters(w: CanonicalWriter, ids: dict[str, uuid.UUID]) -> None:
    p = w.kind(
        "parameterization",
        {
            "key": "calphad-fixture",
            "revision": "1",
            "title": "A compound-energy database",
            "coherence": "independent_records",
            "convention_set": ids["conventions"],
            "chemical_system": ids["system"],
        },
        origins=at("parameterization"),
    )
    ids["parameterization"] = p
    functions = {}
    for name, pieces in FUNCTIONS.items():
        component = w.kind(
            "model_component", {"parameterization": p, "name": name}, origins=at(f"function-{name}")
        )
        functions[name] = component
        w.parameter_set(
            parameterization=p,
            slot_group="gibbs_polynomial.function",
            subjects=[component],
            slots={},
            families={"piece": [FamilyRow({"n": n}, piece_row(piece)) for n, piece in enumerate(pieces, 1)]},
            origins=at(f"function-{name}-pieces"),
        )
    for array, function in ENDMEMBER_FUNCTION.items():
        ids[f"set_{array}"] = w.parameter_set(
            parameterization=p,
            slot_group="cef_endmember.endmember",
            subjects=[ids[f"array_{array}"]],
            slots={"function": SetReference(p, "gibbs_polynomial.function", [functions[function]])},
            origins=at(f"endmember-{array}"),
        )
    for array, (l0, l1) in INTERACTIONS.items():
        w.parameter_set(
            parameterization=p,
            slot_group="cef_redlich_kister.interaction",
            subjects=[ids[f"array_{array}"]],
            slots={},
            families={
                "order": [
                    FamilyRow({"k": 0}, {"L": Quantity(l0, "J/mol")}),
                    FamilyRow({"k": 1}, {"L": Quantity(l1, "J/mol")}),
                ]
            },
            origins=at(f"interaction-{array}"),
        )
    for array, (tc, beta) in MAGNETIC.items():
        w.parameter_set(
            parameterization=p,
            slot_group="cef_magnetic_ihj.magnetic",
            subjects=[ids[f"array_{array}"]],
            slots={"Tc": Quantity(tc, "K"), "beta": beta},
            origins=at(f"magnetic-{array}"),
        )
    w.parameter_set(
        parameterization=p,
        slot_group="cef_magnetic_ihj.structure",
        subjects=[ids["phase_BCC_A2"]],
        slots={"afm": AFM, "p": P_STRUCTURE},
        origins=at("magnetic-structure"),
    )
    for (first, second), coefficients in BINARIES.items():
        w.parameter_set(
            parameterization=p,
            slot_group="redlich_kister_pair.pair",
            subjects=[ids[f"species_{first}"], ids[f"species_{second}"]],  # the order the source asserts
            slots={},
            families={
                "order": [
                    FamilyRow({"k": k}, {"L": Quantity(c, "J/mol")}) for k, c in enumerate(coefficients)
                ]
            },
            origins=at(f"binary-{first}{second}"),
        )
    for array, form in CHOICES.items():
        w.subform_choice(
            parameterization=p,
            slot="ternary_excess_choice.extrapolation",
            subjects=[ids[f"array_{array}"]],
            form=form,
            at="a.json#/choice",
        )


def write_world(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    write_system(w, decl, ids)
    write_arrays(w, ids)
    write_conventions(w, ids)
    write_parameters(w, ids)


@pytest.fixture(scope="module")
def decl() -> Declaration:
    return real_declaration()


@pytest.fixture(scope="module")
def world(decl: Declaration, tmp_path_factory: pytest.TempPathFactory) -> Iterator[World]:
    ids: dict[str, uuid.UUID] = {}
    database = build(tmp_path_factory.mktemp("calphad"), lambda w: write_world(w, decl, ids), decl)
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


def source(world: World, conn: psycopg.Connection) -> DatabaseSource:
    p = world.ids["parameterization"]
    binary_forms = [SubformBinding("redlich_kister_pair", (p,))]
    return DatabaseSource(
        conn,
        world.decl,
        [p],
        subforms={"ternary_kohler.binary": binary_forms, "ternary_toop.binary": binary_forms},
    )


def number(value: object) -> float:
    return float(np.asarray(value, dtype=float).reshape(-1)[0])


# -- the structure ------------------------------------------------------------------------------


def test_the_fixture_satisfies_every_invariant(conn: psycopg.Connection) -> None:
    assert failing(conn) == {}


def test_the_ordered_phase_names_the_disordered_phase_it_is_the_ordered_form_of(
    world: World, conn: psycopg.Connection
) -> None:
    rows = conn.execute("SELECT ordered, disordered, never_disorder FROM tk.disordered_partner").fetchall()
    assert rows == [(world.ids["phase_BCC_B2"], world.ids["phase_BCC_A2"], False)]
    systems = conn.execute(
        "SELECT DISTINCT system FROM tk.phase_definition WHERE id = ANY(%s)",
        ([world.ids["phase_BCC_B2"], world.ids["phase_BCC_A2"]],),
    ).fetchall()
    assert len(systems) == 1


def test_site_classes_have_their_ratios_and_their_admissible_occupants(
    world: World, conn: psycopg.Connection
) -> None:
    ratios = {
        (phase, index): ratio
        for phase, index, ratio in conn.execute(
            'SELECT p.key, s."index", s.ratio FROM tk.site_class s JOIN tk.phase_definition p ON p.id = s.phase'
        )
    }
    assert ratios == {
        (phase, index): ratio
        for phase, (_, _, sites) in PHASES.items()
        for index, (ratio, _) in enumerate(sites, start=1)
    }
    occupants = {
        row[0]: set(row[1])
        for row in conn.execute(
            "SELECT so.site_class, array_agg(e.canonical_key) FROM tk.site_occupant so "
            "JOIN tk.material_entity e ON e.id = so.occupant GROUP BY so.site_class"
        )
    }
    assert occupants[world.ids["site_BCC_B2_3"]] == {"Va"}
    assert occupants[world.ids["site_BCC_B2_1"]] == {"Fe", "Cr", "Ni", "Mo"}


def test_the_members_of_an_array_keep_the_order_the_source_asserted(
    world: World, conn: psycopg.Connection
) -> None:
    def order(array: str) -> list[tuple[int, str]]:
        return [
            (position, key)
            for position, key in conn.execute(
                'SELECT m."position", e.canonical_key FROM tk.constituent_array_member m '
                'JOIN tk.material_entity e ON e.id = m.species WHERE m."array" = %s AND m.site_class = %s '
                'ORDER BY m."position"',
                (world.ids[f"array_{array}"], world.ids["site_BCC_A2_1"]),
            )
        ]

    assert order("interaction_fecr") == [(1, "Fe"), (2, "Cr")]
    assert order("interaction_crfe") == [(1, "Cr"), (2, "Fe")]
    keys = {
        key
        for (key,) in conn.execute(
            "SELECT canonical_key FROM tk.constituent_array WHERE id = ANY(%s)",
            ([world.ids["array_interaction_fecr"], world.ids["array_interaction_crfe"]],),
        )
    }
    assert keys == {"BCC_A2:Fe,Cr:Va", "BCC_A2:Cr,Fe:Va"}  # two arrays, two subjects


def test_the_two_orders_of_one_array_are_two_subjects_not_one(world: World) -> None:
    assert world.ids["array_interaction_fecr"] != world.ids["array_interaction_crfe"]


def test_the_functions_are_half_open_pieces_in_kelvin(world: World, conn: psycopg.Connection) -> None:
    rows = conn.execute(
        'SELECT "n", "T_low", "T_high" FROM param."gibbs_polynomial__function__piece" ORDER BY "T_low", "n"'
    ).fetchall()
    expected = sorted(
        (n, piece["T_low"], piece["T_high"])
        for pieces in FUNCTIONS.values()
        for n, piece in enumerate(pieces, start=1)
    )
    assert sorted(rows) == expected
    # the interval column is a half-open range: [298.15, 1000) and [1000, 3000) do not overlap
    bounds = conn.execute(
        "SELECT pg_get_constraintdef(oid) FROM pg_constraint "
        "WHERE conrelid = 'param.gibbs_polynomial__function__piece'::regclass AND contype = 'x'"
    ).fetchall()
    assert bounds, "the pieces of one set are held apart by an exclusion constraint"


def test_two_endmembers_of_different_phases_reference_one_function_set(
    world: World, conn: psycopg.Connection
) -> None:
    targets = conn.execute(
        'SELECT "function", count(*) FROM param."cef_endmember__endmember" GROUP BY "function"'
    ).fetchall()
    counts = sorted(count for _, count in targets)
    assert counts == [1, 1, 2]  # the two B2 endmembers share the function of the ordered pair


def test_the_ternary_of_each_array_has_its_own_choice_of_extrapolation(
    world: World, conn: psycopg.Connection
) -> None:
    rows = conn.execute(
        "SELECT f.name, count(*) FROM tk.subject_subform_choice c JOIN meta.form f ON f.id = c.form "
        "GROUP BY f.name ORDER BY f.name"
    ).fetchall()
    assert rows == [("ternary_kohler", 1), ("ternary_toop", 2)]


# -- the endmember energies, against numpy ------------------------------------------------------

POINTS = {
    "endmember_fe": [298.15, 500.0, 999.999, 1000.0, 1001.0, 2000.0, 3000.0],
    "endmember_cr": [298.15, 1999.0, 2000.0, 2500.0, 3500.0],
    "b2_fecr": [298.15, 699.0, 700.0, 1499.0, 1500.0, 3000.0],
    "b2_crfe": [500.0, 700.0, 1500.0],
}


@pytest.mark.parametrize("array", sorted(POINTS))
def test_each_endmember_energy_matches_the_independent_calculation_in_every_piece(
    world: World, conn: psycopg.Connection, array: str
) -> None:
    found = bind(
        world.decl,
        "cef_endmember",
        source=source(world, conn),
        roles={"t": str(world.ids[f"array_{array}"])},
        cache=CACHE,
    )
    T = np.array(POINTS[array])
    got = np.asarray(found.evaluate("G", T=T), dtype=float).reshape(-1)
    np.testing.assert_allclose(got, function_value(ENDMEMBER_FUNCTION[array], T), rtol=1e-12)


def test_the_pieces_are_half_open_a_boundary_point_belongs_to_the_piece_above(
    world: World, conn: psycopg.Connection
) -> None:
    found = bind(
        world.decl,
        "cef_endmember",
        source=source(world, conn),
        roles={"t": str(world.ids["array_endmember_fe"])},
        cache=CACHE,
    )
    at_boundary = number(found.evaluate("G", T=np.array([1000.0])))
    lower, upper = FUNCTIONS["fe_bcc"]
    assert at_boundary == pytest.approx(float(polynomial(upper, np.array([1000.0]))[0]), rel=1e-13)
    assert abs(at_boundary - float(polynomial(lower, np.array([1000.0]))[0])) > 1.0, "the pieces differ there"


def test_a_temperature_outside_every_piece_is_refused_naming_the_function(
    world: World, conn: psycopg.Connection
) -> None:
    found = bind(
        world.decl,
        "cef_endmember",
        source=source(world, conn),
        roles={"t": str(world.ids["array_endmember_fe"])},
        cache=CACHE,
    )
    for T in (250.0, 3500.0):
        with pytest.raises(EvaluationRefusal, match=r"outside every piece of family `gibbs_polynomial.function.piece`"):
            found.evaluate("G", T=np.array([T]))


# -- the interaction and its sign under a swap --------------------------------------------------


def interaction(world: World, conn: psycopg.Connection, array: str, order: tuple[str, str], y: dict[str, float]) -> float:
    ids = {name: str(world.ids[f"species_{name}"]) for name in order}
    found = bind(
        world.decl,
        "cef_redlich_kister",
        source=source(world, conn),
        roles={"t": str(world.ids[f"array_{array}"])},
        sets={"mixing": [ids[name] for name in order]},
        cache=CACHE,
    )
    return number(
        found.evaluate("G", y={ids[name]: np.array([y[name]]) for name in order}, spectator=np.array([1.0]))
    )


Y = {"Fe": 0.3, "Cr": 0.7}


def expected_interaction(first: str, second: str, l0: float, l1: float) -> float:
    return Y[first] * Y[second] * (l0 + l1 * (Y[first] - Y[second]))


def test_the_interaction_takes_its_sign_from_the_position_the_array_gives_each_species(
    world: World, conn: psycopg.Connection
) -> None:
    l0, l1 = INTERACTIONS["interaction_fecr"]
    found = interaction(world, conn, "interaction_fecr", ("Fe", "Cr"), Y)
    assert found == pytest.approx(expected_interaction("Fe", "Cr", l0, l1), rel=1e-13)
    # the array that asserts the species the other way round has the opposite L1 and the same energy
    l0_swapped, l1_swapped = INTERACTIONS["interaction_crfe"]
    assert l1_swapped == -l1 and l0_swapped == l0
    swapped = interaction(world, conn, "interaction_crfe", ("Cr", "Fe"), Y)
    assert swapped == pytest.approx(expected_interaction("Cr", "Fe", l0_swapped, l1_swapped), rel=1e-13)
    assert swapped == pytest.approx(found, rel=1e-13)


def test_the_order_the_caller_passes_the_mixing_species_in_does_not_matter(
    world: World, conn: psycopg.Connection
) -> None:
    straight = interaction(world, conn, "interaction_fecr", ("Fe", "Cr"), Y)
    reversed_ = interaction(world, conn, "interaction_fecr", ("Cr", "Fe"), Y)
    assert reversed_ == pytest.approx(straight, rel=1e-13)


def test_reading_one_array_with_the_other_arrays_coefficient_would_change_the_energy() -> None:
    """The control: with the asserted sign ignored the two orders would disagree."""
    l0, l1 = INTERACTIONS["interaction_fecr"]
    assert abs(expected_interaction("Fe", "Cr", l0, l1) - expected_interaction("Fe", "Cr", l0, -l1)) > 100.0


# -- the binary of a pair, held once with parity --------------------------------------------------


def pair_energy(world: World, conn: psycopg.Connection, i: str, j: str, xi: float) -> float:
    found = bind(
        world.decl,
        "redlich_kister_pair",
        source=source(world, conn),
        roles={"i": str(world.ids[f"species_{i}"]), "j": str(world.ids[f"species_{j}"])},
        cache=CACHE,
    )
    return number(found.evaluate("g", xi=np.array([xi])))


@pytest.mark.parametrize("pair", sorted(BINARIES))
def test_a_binary_read_in_either_order_is_the_same_function_of_the_composition(
    world: World, conn: psycopg.Connection, pair: tuple[str, str]
) -> None:
    i, j = pair
    for xi in (0.15, 0.5, 0.8):
        asserted = pair_energy(world, conn, i, j, xi)
        swapped = pair_energy(world, conn, j, i, 1 - xi)
        assert asserted == pytest.approx(binary(i, j, xi), rel=1e-12)
        assert swapped == pytest.approx(binary(j, i, 1 - xi), rel=1e-12)
        assert swapped == pytest.approx(asserted, rel=1e-12, abs=1e-9)


def test_the_odd_orders_of_a_binary_are_stored_as_asserted_with_the_arrangement_recorded(
    world: World, conn: psycopg.Connection
) -> None:
    rows = conn.execute(
        'SELECT p."arrangement", o."k", o."L" FROM param."redlich_kister_pair__pair" p '
        'JOIN param."redlich_kister_pair__pair__order" o ON o.set_id = p.id '
        "JOIN tk.material_entity a ON a.id = p.i JOIN tk.material_entity b ON b.id = p.j "
        "WHERE a.canonical_key || b.canonical_key IN ('FeCr', 'CrFe') ORDER BY o.\"k\""
    ).fetchall()
    assert [(k, value) for _, k, value in rows] == [(0, 4000.0), (1, 900.0), (2, -350.0)]  # not negated
    assert {arrangement for arrangement, _, _ in rows} <= {0, 1}


# -- the magnetic contribution --------------------------------------------------------------------


@pytest.mark.parametrize("array", sorted(MAGNETIC))
def test_the_magnetic_contribution_matches_the_published_function_below_and_above_the_curie_temperature(
    world: World, conn: psycopg.Connection, array: str
) -> None:
    found = bind(
        world.decl,
        "cef_magnetic_ihj",
        source=source(world, conn),
        roles={"t": str(world.ids[f"array_{array}"]), "ph": str(world.ids["phase_BCC_A2"])},
        cache=CACHE,
    )
    tc, beta = MAGNETIC[array]
    T = np.array([100.0, 300.0, 700.0, 0.999 * abs(tc), 1.001 * abs(tc), 1.4 * abs(tc), 1800.0])
    got = np.asarray(found.evaluate("G", T=T), dtype=float).reshape(-1)
    np.testing.assert_allclose(got, magnetic(tc, beta, AFM, P_STRUCTURE, T), rtol=1e-11)


def test_the_function_is_continuous_across_the_curie_temperature() -> None:
    T = np.array([1043.0 * (1 - 1e-9), 1043.0 * (1 + 1e-9)])
    below, above = magnetic(1043.0, 2.22, AFM, P_STRUCTURE, T)
    assert below == pytest.approx(above, rel=1e-6)


def test_a_negative_curie_temperature_is_divided_by_the_antiferromagnetic_factor(
    world: World, conn: psycopg.Connection
) -> None:
    """Cr is stored with Tc = -311.5 K and a negative moment: the form reads Tc / afm = 311.5 K."""
    found = bind(
        world.decl,
        "cef_magnetic_ihj",
        source=source(world, conn),
        roles={"t": str(world.ids["array_endmember_cr"]), "ph": str(world.ids["phase_BCC_A2"])},
        cache=CACHE,
    )
    at_311 = number(found.evaluate("G", T=np.array([311.5 * 1.000001])))
    ignoring_sign = number(magnetic(311.5, 0.25, 1.0, P_STRUCTURE, np.array([311.5 * 1.000001])))
    assert at_311 == pytest.approx(ignoring_sign, rel=1e-12)


# -- the ternaries --------------------------------------------------------------------------------


def ternary(
    world: World, conn: psycopg.Connection, array: str, order: tuple[str, ...], x: dict[str, float]
) -> float:
    ids = {name: str(world.ids[f"species_{name}"]) for name in order}
    found = bind(
        world.decl,
        "ternary_excess_choice",
        source=source(world, conn),
        roles={"t": str(world.ids[f"array_{array}"])},
        sets={"components": [ids[name] for name in order]},
        cache=CACHE,
    )
    return number(found.evaluate("gE", x={ids[name]: np.array([x[name]]) for name in order}))


KOHLER_X = {"Fe": 0.2, "Cr": 0.5, "Ni": 0.3}
TOOP_X = {"Fe": 0.25, "Cr": 0.45, "Mo": 0.30}


def test_the_ternary_that_chose_kohler_is_extrapolated_by_kohler(world: World, conn: psycopg.Connection) -> None:
    found = ternary(world, conn, "kohler", ("Fe", "Cr", "Ni"), KOHLER_X)
    assert found == pytest.approx(kohler(("Fe", "Cr", "Ni"), KOHLER_X), rel=1e-12)
    for order in (("Ni", "Fe", "Cr"), ("Cr", "Ni", "Fe")):
        assert ternary(world, conn, "kohler", order, KOHLER_X) == pytest.approx(found, rel=1e-12)


def test_the_ternary_that_chose_toop_singles_out_the_species_the_array_puts_first(
    world: World, conn: psycopg.Connection
) -> None:
    first_fe = ternary(world, conn, "toop_fe_first", ("Fe", "Cr", "Mo"), TOOP_X)
    first_cr = ternary(world, conn, "toop_cr_first", ("Fe", "Cr", "Mo"), TOOP_X)
    assert first_fe == pytest.approx(toop(("Fe", "Cr", "Mo"), TOOP_X, "Fe"), rel=1e-12)
    assert first_cr == pytest.approx(toop(("Fe", "Cr", "Mo"), TOOP_X, "Cr"), rel=1e-12)
    assert abs(first_fe - first_cr) > 1.0, "the species the array puts first changes the energy"


def test_the_species_singled_out_does_not_depend_on_the_order_the_components_are_passed_in(
    world: World, conn: psycopg.Connection
) -> None:
    reference = ternary(world, conn, "toop_cr_first", ("Fe", "Cr", "Mo"), TOOP_X)
    for order in (("Mo", "Fe", "Cr"), ("Cr", "Mo", "Fe")):
        assert ternary(world, conn, "toop_cr_first", order, TOOP_X) == pytest.approx(reference, rel=1e-12)


def test_the_two_extrapolations_differ_for_the_same_binaries_and_composition() -> None:
    assert abs(kohler(("Fe", "Cr", "Mo"), TOOP_X) - toop(("Fe", "Cr", "Mo"), TOOP_X, "Fe")) > 1.0


def test_a_species_the_array_does_not_place_is_refused_with_its_position_undefined(
    world: World, conn: psycopg.Connection
) -> None:
    ids = {name: str(world.ids[f"species_{name}"]) for name in ("Fe", "Cr", "Ni")}
    found = bind(
        world.decl,
        "ternary_excess_choice",
        source=source(world, conn),
        roles={"t": str(world.ids["array_toop_fe_first"])},  # Fe, Cr, Mo: no Ni
        sets={"components": [ids["Fe"], ids["Cr"], ids["Ni"]]},
    )
    with pytest.raises(EvaluationRefusal, match="position of species .* is undefined: the array places it nowhere"):
        found.evaluate("gE", x={ids[n]: np.array([v]) for n, v in (("Fe", 0.2), ("Cr", 0.5), ("Ni", 0.3))})


def test_a_species_the_array_places_twice_has_no_single_position(world: World, conn: psycopg.Connection) -> None:
    """An endmember of the ordered phase with iron on both substitutional site classes."""
    ids = {name: str(world.ids[f"species_{name}"]) for name in ("Fe", "Cr", "Mo")}
    found = bind(
        world.decl,
        "ternary_toop",
        source=source(world, conn),
        roles={"t": str(world.ids["array_b2_fefe"])},
        sets={"components": [ids["Fe"], ids["Cr"], ids["Mo"]]},
    )
    with pytest.raises(EvaluationRefusal, match=r"places it 2 times \(\(1, 1\)\)"):
        found.evaluate("gE", x={ids[n]: np.array([v]) for n, v in (("Fe", 0.2), ("Cr", 0.5), ("Mo", 0.3))})


def test_a_ternary_with_no_choice_is_refused_naming_the_sub_form_slot(
    world: World, conn: psycopg.Connection
) -> None:
    ids = {name: str(world.ids[f"species_{name}"]) for name in ("Fe", "Cr", "Ni")}
    found = bind(
        world.decl,
        "ternary_excess_choice",
        source=source(world, conn),
        roles={"t": str(uuid.uuid4())},
        sets={"components": list(ids.values())},
    )
    with pytest.raises(EvaluationRefusal, match=r"ternary_excess_choice\.extrapolation"):
        found.evaluate("gE", x={i: np.array([1 / 3]) for i in ids.values()})


# -- what the model refuses -----------------------------------------------------------------------


def system_and_phases(w: CanonicalWriter, decl: Declaration) -> tuple[dict[str, uuid.UUID], dict[str, uuid.UUID]]:
    ids: dict[str, uuid.UUID] = {}
    write_system(w, decl, ids)
    return ids, {name: ids[f"species_{name}"] for name in SPECIES}


def test_the_ordered_phase_cannot_be_its_own_disordered_partner(decl: Declaration) -> None:
    w = writer(decl)
    ids, _ = system_and_phases(w, decl)
    with pytest.raises(ValidationError, match="diagonal"):
        w.relation(
            "disordered_partner",
            {"ordered": ids["phase_BCC_A2"], "disordered": ids["phase_BCC_A2"]},
            {"never_disorder": True},
            at="a.json#/partner",
        )


def test_pieces_of_a_function_that_overlap_are_refused(decl: Declaration) -> None:
    w = writer(decl)
    ids: dict[str, uuid.UUID] = {}
    write_system(w, decl, ids)
    write_conventions(w, ids)
    p = w.kind(
        "parameterization",
        {"key": "p", "revision": "1", "title": "p", "coherence": "independent_records"},
        origins=at("p"),
    )
    component = w.kind("model_component", {"parameterization": p, "name": "f"}, origins=at("f"))
    lower, upper = FUNCTIONS["fe_bcc"]
    with pytest.raises(ValidationError, match="pieces overlap"):
        w.parameter_set(
            parameterization=p,
            slot_group="gibbs_polynomial.function",
            subjects=[component],
            slots={},
            families={
                "piece": [
                    FamilyRow({"n": 1}, piece_row(lower)),
                    FamilyRow({"n": 2}, piece_row({**upper, "T_low": 900.0})),
                ]
            },
            origins=at("overlap"),
        )


def test_an_endmember_cannot_reference_a_set_of_another_contract(decl: Declaration) -> None:
    w = writer(decl)
    ids: dict[str, uuid.UUID] = {}
    write_system(w, decl, ids)
    write_arrays(w, ids)
    p = w.kind(
        "parameterization",
        {"key": "p", "revision": "1", "title": "p", "coherence": "independent_records"},
        origins=at("p"),
    )
    with pytest.raises(ValidationError, match="implements"):
        w.parameter_set(
            parameterization=p,
            slot_group="cef_endmember.endmember",
            subjects=[ids["array_endmember_fe"]],
            slots={"function": SetReference(p, "cef_endmember.endmember", [ids["array_endmember_cr"]])},
            origins=at("bad-reference"),
        )


def test_the_checks_of_verify_flag_each_broken_array_and_partner(decl: Declaration, tmp_path: Path) -> None:
    """One violation of each structure the case is about, each found by the check that states
    it: an ordered phase whose partner belongs to another system, an array whose key is not the
    ordering convention applied to its members, positions with a gap, a site class of another
    phase and a species that may not occupy its site class."""

    def emit(w: CanonicalWriter) -> None:
        ids: dict[str, uuid.UUID] = {}
        write_system(w, decl, ids)
        other = w.kind("chemical_system", {"key": "other", "revision": "1"}, origins=at("other-system"))
        foreign = w.kind(
            "phase_definition",
            {
                "system": other,
                "key": "OTHER",
                "aggregation": entity_id(decl, "aggregation", "crystalline"),
                "structure": "sublattice",
            },
            origins=at("foreign-phase"),
        )
        w.relation(
            "disordered_partner",
            {"ordered": ids["phase_LIQUID"], "disordered": foreign},
            {"never_disorder": False},
            at="a.json#/foreign-partner",
        )
        phase = ids["phase_LIQUID"]
        site = ids["site_LIQUID_1"]

        def member(array: uuid.UUID, position: int, name: str, site_class: uuid.UUID = site) -> None:
            w.relation(
                "constituent_array_member",
                {"array": array, "site_class": site_class, "position": position},
                {"species": ids[f"species_{name}"]},
                at="a.json#/member",
            )

        wrong_key = w.kind(
            "constituent_array", {"phase": phase, "canonical_key": "LIQUID:Cr,Fe"}, origins=at("wrong-key")
        )
        member(wrong_key, 1, "Fe")
        member(wrong_key, 2, "Cr")
        gap = w.kind("constituent_array", {"phase": phase, "canonical_key": "LIQUID:Fe,Cr"}, origins=at("gap"))
        member(gap, 1, "Fe")
        member(gap, 3, "Cr")
        foreign_class = w.kind(
            "site_class",
            {"phase": foreign, "index": 1, "ratio_kind": "constant", "ratio": 1.0},
            origins=at("foreign-site"),
        )
        w.relation(
            "site_occupant", {"site_class": foreign_class, "occupant": ids["species_Fe"]}, {}, at="a.json#/occupant"
        )
        elsewhere = w.kind(
            "constituent_array", {"phase": phase, "canonical_key": "LIQUID:Fe"}, origins=at("elsewhere")
        )
        member(elsewhere, 1, "Fe", foreign_class)
        stranger = w.kind(
            "constituent_array", {"phase": phase, "canonical_key": "LIQUID:Va"}, origins=at("stranger")
        )
        member(stranger, 1, "Va")

    database = build(tmp_path, emit, decl)
    try:
        with psycopg.connect(database.url) as connection:
            assert failing(connection) == {
                "disordered_partner.same_system": 1,
                "constituent_array.key_follows_members": 1,
                "constituent_array_member.positions_contiguous": 1,
                "constituent_array_member.site_class_of_array_phase": 1,
                "constituent_array_member.species_may_occupy_site_class": 1,
            }
            assert {name for name in CHECKS if name.startswith("constituent_array")} >= {
                "constituent_array.key_follows_members",
                "constituent_array_member.positions_contiguous",
            }
    finally:
        database.remove()


def test_the_position_function_takes_an_array_then_a_species(tmp_path: Path) -> None:
    result = broken(tmp_path, {"forms/calphad.toml": replace("position(t, s) != 1", "position(s, t) != 1")})
    assert result.declaration is None
    assert any(
        d.code is Code.SLOT_SUBJECTS and "constituent_array" in d.message for d in result.diagnostics
    )


def test_the_position_function_is_refused_for_an_expression_operand(tmp_path: Path) -> None:
    result = broken(tmp_path, {"forms/calphad.toml": replace("position(t, s) != 1", "position(t, s + 1) != 1")})
    assert any(d.code is Code.SLOT_SUBJECTS for d in result.diagnostics)
