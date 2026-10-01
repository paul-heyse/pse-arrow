# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Hard case (i), plan 24 packet TK2: petroleum pseudo-components.

An assay is a distillation curve as a dataset: the cumulative volume distilled, the temperature it
distilled at, and the specific gravity and molar mass of the cut at each point. A characterisation
derivation turns it into pseudo-components, one for each cut (kind `assay_cut`), each with a
molar mass, a normal boiling temperature and a specific gravity (`assay_cut`, origin role
`derived`). An estimation derivation gives the critical temperature, critical pressure and acentric
factor of every cut through correlation forms (`estimated_critical_constants`, origin role
`estimated`); it consumes the coefficient sets of the correlations, so the lineage names the forms.
A lump of two cuts is a pseudo-component (kind `lump`) that a lumping derivation produced from the
two, with the mole fractions of the lump as the weights of its inputs.

The numbers are synthetic. The independent calculations are the power laws, the sum of terms and
Edmister's closed form written in numpy, and the moments the lumping rule states.
"""

from __future__ import annotations

import uuid
from collections.abc import Iterator
from dataclasses import dataclass
from pathlib import Path

import numpy as np
import psycopg
import pytest

from hard_case_support import CHECKS, at, build, failing, observable_id
from mapping_support import carrier, real_declaration, writer
from thermo_knowledge import db, identity
from thermo_knowledge.canonical.values import Quantity
from thermo_knowledge.canonical.writer import CanonicalWriter, ValidationError
from thermo_knowledge.declaration import Declaration
from thermo_knowledge.expression.compiled import CompileCache
from thermo_knowledge.expression.evaluate import EvaluationRefusal, bind
from thermo_knowledge.qualify.source import DatabaseSource
from thermo_knowledge.testing import TestDatabase
from thermo_knowledge.verify.run import run_check

CACHE = CompileCache()
CARRIER = identity.identifier("source", [carrier("src", "a.json").key])
ATMOSPHERE = 101325.0  # Pa, defined

# -- the assay: a distillation curve with the properties of each cut -----------------------------

VOLUME = (0.10, 0.30, 0.55, 0.80, 0.95)  # cumulative volume fraction distilled at the cut point
TEMPERATURE = (385.0, 430.0, 490.0, 560.0, 650.0)  # K, mean boiling point of the cut
GRAVITY = (0.70, 0.74, 0.79, 0.85, 0.92)
MOLAR_MASS_G = (98.0, 128.0, 175.0, 250.0, 390.0)  # g/mol, stated so: the writer stores kg/mol
PRESSURE = 101.325  # kPa, the pressure of the distillation

# -- the correlations' coefficients ----------------------------------------------------------------

RD_TC = dict(a=18.0, b=0.60, c=0.32)  # Tc = a (Tb / K)^b SG^c, a in K
RD_PC = dict(a=4.2e10, b=-1.6, c=0.5)  # Pc = a (Tb / K)^b SG^c, a in Pa
KL_TC = dict(k0=100.0, k1=300.0, k2=0.55, k3=0.10, k4=3.0e4, k5=-2.0e4)  # k4, k5 in K^2

# -- the lump ------------------------------------------------------------------------------------

LUMP_OF = (1, 2)  # the second and the third cut, by position
LUMP_FRACTIONS = (0.6, 0.4)  # mole fractions of the lump

# -- the independent calculation -----------------------------------------------------------------


def riazi_daubert(coefficients: dict[str, float], Tb: np.ndarray, SG: np.ndarray) -> np.ndarray:
    return (
        coefficients["a"]
        * np.asarray(Tb) ** coefficients["b"]
        * np.asarray(SG) ** coefficients["c"]
    )


def kesler_lee(Tb: np.ndarray, SG: np.ndarray) -> np.ndarray:
    k = KL_TC
    return k["k0"] + k["k1"] * SG + (k["k2"] + k["k3"] * SG) * Tb + (k["k4"] + k["k5"] * SG) / Tb


def edmister(Tb: np.ndarray, Tc: np.ndarray, Pc: np.ndarray) -> np.ndarray:
    return 3.0 / 7.0 * np.log10(Pc / ATMOSPHERE) / (Tc / Tb - 1.0) - 1.0


def lumped(
    masses: np.ndarray, boiling: np.ndarray, gravities: np.ndarray, z: np.ndarray
) -> tuple[float, float, float]:
    """The lumping rule of the derivation: molar mass and boiling point are mole-weighted means and
    the gravity follows from additive volumes, SG = M / sum(z_i M_i / SG_i)."""
    mass = float(np.sum(z * masses))
    return mass, float(np.sum(z * boiling)), float(mass / np.sum(z * masses / gravities))


def estimate(Tb: float, SG: float) -> tuple[float, float, float]:
    Tc = float(riazi_daubert(RD_TC, np.array(Tb), np.array(SG)))
    Pc = float(riazi_daubert(RD_PC, np.array(Tb), np.array(SG)))
    return Tc, Pc, float(edmister(np.array(Tb), np.array(Tc), np.array(Pc)))


# -- the fixture ---------------------------------------------------------------------------------


@dataclass
class World:
    decl: Declaration
    database: TestDatabase
    ids: dict[str, uuid.UUID]


def parameterization(w: CanonicalWriter, key: str, role: str = "published") -> uuid.UUID:
    return w.kind(
        "parameterization",
        {"key": key, "revision": "1", "title": key, "coherence": "independent_records"},
        origins=at(f"parameterization-{key}", role),
    )


def column(
    w: CanonicalWriter,
    decl: Declaration,
    dataset: uuid.UUID,
    ordinal: int,
    role: str,
    observable: str,
    **extra: object,
) -> uuid.UUID:
    return w.kind(
        "dataset_column",
        {
            "dataset": dataset,
            "ordinal": ordinal,
            "role": role,
            "observable": observable_id(decl, observable),
            **extra,
        },
        at="a.json#/columns",
    )


def cut_slots(mass: float, boiling: float, gravity: float) -> dict[str, object]:
    return {
        "molar_mass": Quantity(mass, "kg/mol"),
        "normal_boiling_temperature": Quantity(boiling, "K"),
        "specific_gravity": gravity,
    }


def critical_slots(Tc: float, Pc: float, omega: float) -> dict[str, object]:
    return {
        "critical_temperature": Quantity(Tc, "K"),
        "critical_pressure": Quantity(Pc, "Pa"),
        "acentric_factor": omega,
    }


def write_world(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    # the assay: a distillation curve with the properties of each cut
    dataset = w.kind(
        "dataset",
        {
            "carrier": CARRIER,
            "local_key": "assay",
            "kind": "measured",
            "title": "Distillation assay of a crude",
            "method": "true boiling point distillation",
        },
        origins=at("assay", "measured"),
    )
    ids["assay"] = dataset
    column(
        w,
        decl,
        dataset,
        1,
        "constraint",
        "pressure",
        constant=Quantity(PRESSURE, "kPa"),
        constant_digits=6,
    )
    columns = {
        name: column(w, decl, dataset, ordinal, role, observable, **extra)
        for name, ordinal, role, observable, extra in (
            ("volume", 2, "variable", "cumulative_distilled_volume_fraction", {}),
            ("temperature", 3, "property", "temperature", {}),
            ("gravity", 4, "property", "specific_gravity", {}),
            ("mass", 5, "property", "molar_mass", {}),
        )
    }
    for index in range(len(VOLUME)):
        point = w.kind("data_point", {"dataset": dataset, "index": index + 1}, at="a.json#/points")
        for name, value in (
            ("volume", Quantity(VOLUME[index], "dimensionless")),
            ("temperature", Quantity(TEMPERATURE[index], "K")),
            ("gravity", Quantity(GRAVITY[index], "dimensionless")),
            ("mass", Quantity(MOLAR_MASS_G[index], "g/mol")),
        ):
            w.relation(
                "datum",
                {"point": point, "column": columns[name]},
                {"state": "known", "value": value, "digits": 3},
                at="a.json#/datum",
            )
    # the characterisation of the assay into cuts
    characterisation = w.kind(
        "derivation",
        {
            "key": "characterisation",
            "kind": "characterisation",
            "method": "one pseudo-component for each cut point of the curve, with the properties reported at it",
        },
        origins=at("characterisation", "published"),
    )
    ids["characterisation"] = characterisation
    w.relation(
        "derivation_input",
        {"derivation": characterisation, "record": dataset},
        {},
        at="a.json#/lineage",
    )
    assay = parameterization(w, "assay-cuts", "derived")
    w.relation(
        "derivation_output",
        {"derivation": characterisation, "record": assay},
        {},
        at="a.json#/lineage",
    )
    cuts = []
    for index in range(len(VOLUME)):
        cut = w.kind(
            "pseudo_component",
            {
                "canonical_key": f"cut-{index + 1}",
                "label": f"cut {index + 1}",
                "kind": "assay_cut",
                "produced_by": characterisation,
            },
            origins=at(f"cut-{index + 1}", "derived"),
        )
        cuts.append(cut)
        ids[f"cut_{index}"] = cut
        ids[f"cut_set_{index}"] = w.parameter_set(
            parameterization=assay,
            slot_group="assay_cut.cut",
            subjects=[cut],
            slots=cut_slots(MOLAR_MASS_G[index] * 1e-3, TEMPERATURE[index], GRAVITY[index]),
            origins=at(f"cut-set-{index + 1}", "derived"),
        )
        w.relation(
            "derivation_output",
            {"derivation": characterisation, "record": cut},
            {},
            at="a.json#/lineage",
        )
        w.relation(
            "derivation_output",
            {"derivation": characterisation, "record": ids[f"cut_set_{index}"]},
            {},
            at="a.json#/lineage",
        )
    # the lump of two cuts
    lumping = w.kind(
        "derivation",
        {
            "key": "lumping",
            "kind": "characterisation",
            "method": "lump of cuts: mole-weighted molar mass and boiling point, volume-additive gravity",
        },
        origins=at("lumping", "published"),
    )
    ids["lumping"] = lumping
    members = [cuts[i] for i in LUMP_OF]
    masses = np.array([MOLAR_MASS_G[i] * 1e-3 for i in LUMP_OF])
    mass, boiling, gravity = lumped(
        masses,
        np.array([TEMPERATURE[i] for i in LUMP_OF]),
        np.array([GRAVITY[i] for i in LUMP_OF]),
        np.array(LUMP_FRACTIONS),
    )
    lump = w.kind(
        "pseudo_component",
        {
            "canonical_key": "lump-2-3",
            "label": "lump of cuts 2 and 3",
            "kind": "lump",
            "produced_by": lumping,
        },
        origins=at("lump", "derived"),
    )
    ids["lump"] = lump
    lump_set = w.parameter_set(
        parameterization=assay,
        slot_group="assay_cut.cut",
        subjects=[lump],
        slots=cut_slots(mass, boiling, gravity),
        origins=at("lump-set", "derived"),
    )
    ids["lump_set"] = lump_set
    for member, fraction in zip(members, LUMP_FRACTIONS, strict=True):
        w.relation(
            "derivation_input",
            {"derivation": lumping, "record": member},
            {"weight": fraction},
            at="a.json#/lineage",
        )
    w.relation(
        "derivation_output", {"derivation": lumping, "record": lump}, {}, at="a.json#/lineage"
    )
    w.relation(
        "derivation_output", {"derivation": lumping, "record": lump_set}, {}, at="a.json#/lineage"
    )
    # the correlations
    tc = parameterization(w, "riazi-daubert-tc")
    pc = parameterization(w, "riazi-daubert-pc")
    kl = parameterization(w, "kesler-lee-tc")
    ids["rd_tc"] = w.parameter_set(
        parameterization=tc,
        slot_group="riazi_daubert_critical_temperature.correlation",
        subjects=[],
        slots={"a": Quantity(RD_TC["a"], "K"), "b": RD_TC["b"], "c": RD_TC["c"]},
        origins=at("rd-tc"),
    )
    ids["rd_pc"] = w.parameter_set(
        parameterization=pc,
        slot_group="riazi_daubert_critical_pressure.correlation",
        subjects=[],
        slots={"a": Quantity(RD_PC["a"], "Pa"), "b": RD_PC["b"], "c": RD_PC["c"]},
        origins=at("rd-pc"),
    )
    ids["kl_tc"] = w.parameter_set(
        parameterization=kl,
        slot_group="kesler_lee_critical_temperature.correlation",
        subjects=[],
        slots={
            "k0": Quantity(KL_TC["k0"], "K"),
            "k1": Quantity(KL_TC["k1"], "K"),
            "k2": KL_TC["k2"],
            "k3": KL_TC["k3"],
            "k4": Quantity(KL_TC["k4"], "K**2"),
            "k5": Quantity(KL_TC["k5"], "K**2"),
        },
        origins=at("kl-tc"),
    )
    # the estimation of the critical constants of every cut and of the lump
    estimation = w.kind(
        "derivation",
        {
            "key": "estimation",
            "kind": "estimation",
            "method": "riazi_daubert_critical_temperature, riazi_daubert_critical_pressure and edmister_acentric_factor",
        },
        origins=at("estimation", "published"),
    )
    ids["estimation"] = estimation
    constants = parameterization(w, "estimated-constants", "estimated")
    w.relation(
        "derivation_output",
        {"derivation": estimation, "record": constants},
        {},
        at="a.json#/lineage",
    )
    for name in ("rd_tc", "rd_pc"):
        w.relation(
            "derivation_input",
            {"derivation": estimation, "record": ids[name]},
            {},
            at="a.json#/lineage",
        )
    subjects = [(f"cut_{i}", TEMPERATURE[i], GRAVITY[i]) for i in range(len(VOLUME))] + [
        ("lump", boiling, gravity)
    ]
    for name, Tb, SG in subjects:
        ids[f"critical_{name}"] = w.parameter_set(
            parameterization=constants,
            slot_group="estimated_critical_constants.critical",
            subjects=[ids[name]],
            slots=critical_slots(*estimate(Tb, SG)),
            origins=at(f"critical-{name}", "estimated"),
        )
        w.relation(
            "derivation_input",
            {
                "derivation": estimation,
                "record": ids[name.replace("cut_", "cut_set_") if name != "lump" else "lump_set"],
            },
            {},
            at="a.json#/lineage",
        )
        w.relation(
            "derivation_output",
            {"derivation": estimation, "record": ids[f"critical_{name}"]},
            {},
            at="a.json#/lineage",
        )
    ids["assay_parameterization"] = assay
    ids["constants_parameterization"] = constants
    ids["tc_parameterization"] = tc
    ids["pc_parameterization"] = pc
    ids["kl_parameterization"] = kl


@pytest.fixture(scope="module")
def decl() -> Declaration:
    return real_declaration()


@pytest.fixture(scope="module")
def world(decl: Declaration, tmp_path_factory: pytest.TempPathFactory) -> Iterator[World]:
    ids: dict[str, uuid.UUID] = {}
    database = build(
        tmp_path_factory.mktemp("petroleum"), lambda w: write_world(w, decl, ids), decl
    )
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


def evaluate(
    world: World,
    conn: psycopg.Connection,
    form: str,
    parameterizations: list[uuid.UUID],
    output: str,
    roles: dict[str, str] | None = None,
    **arguments: np.ndarray,
) -> np.ndarray:
    bound = bind(
        world.decl,
        form,
        source=DatabaseSource(conn, world.decl, parameterizations),
        roles=roles,
        cache=CACHE,
    )
    return np.asarray(bound.evaluate(output, **arguments), dtype=float).reshape(-1)


def test_the_fixture_satisfies_every_invariant(conn: psycopg.Connection) -> None:
    assert failing(conn) == {}


# -- the assay and its characterisation -----------------------------------------------------------


def test_the_assay_is_a_dataset_whose_columns_are_typed_by_observable_and_role(
    world: World, conn: psycopg.Connection
) -> None:
    rows = conn.execute(
        "SELECT c.ordinal, c.role::text, o.key, c.constant FROM ev.dataset_column c JOIN tk.observable o ON o.id = c.observable "
        "WHERE c.dataset = %s ORDER BY c.ordinal",
        (world.ids["assay"],),
    ).fetchall()
    assert [r[:3] for r in rows] == [
        (1, "constraint", "pressure"),
        (2, "variable", "cumulative_distilled_volume_fraction"),
        (3, "property", "temperature"),
        (4, "property", "specific_gravity"),
        (5, "property", "molar_mass"),
    ]
    assert rows[0][3] == pytest.approx(PRESSURE * 1e3, rel=1e-12)  # the atmosphere, in pascal
    stored = conn.execute(
        "SELECT p.index, c.ordinal, d.value FROM ev.datum d JOIN ev.data_point p ON p.id = d.point "
        "JOIN ev.dataset_column c ON c.id = d.column WHERE p.dataset = %s ORDER BY p.index, c.ordinal",
        (world.ids["assay"],),
    ).fetchall()
    by_column = {ordinal: [v for _, o, v in stored if o == ordinal] for ordinal in (2, 3, 4, 5)}
    np.testing.assert_allclose(by_column[2], VOLUME, rtol=1e-12)
    np.testing.assert_allclose(by_column[3], TEMPERATURE, rtol=1e-12)
    np.testing.assert_allclose(by_column[4], GRAVITY, rtol=1e-12)
    np.testing.assert_allclose(
        by_column[5], np.array(MOLAR_MASS_G) * 1e-3, rtol=1e-12
    )  # g/mol stated, kg/mol stored


def test_the_characterisation_produced_each_cut_from_the_curve(
    world: World, conn: psycopg.Connection
) -> None:
    producer = world.ids["characterisation"]
    assert (
        scalar(conn, "SELECT kind::text FROM prov.derivation WHERE id = %s", producer)
        == "characterisation"
    )
    assert (
        scalar(conn, "SELECT record FROM prov.derivation_input WHERE derivation = %s", producer)
        == world.ids["assay"]
    )
    cuts = conn.execute(
        "SELECT e.canonical_key, p.kind::text, p.produced_by, e.label FROM tk.pseudo_component p "
        "JOIN tk.material_entity e ON e.id = p.id WHERE p.kind::text = 'assay_cut' ORDER BY e.canonical_key"
    ).fetchall()
    assert [c[0] for c in cuts] == [f"cut-{i}" for i in range(1, 6)]
    assert {c[2] for c in cuts} == {producer}
    produced = {
        r[0]
        for r in conn.execute(
            "SELECT record FROM prov.derivation_output WHERE derivation = %s", (producer,)
        ).fetchall()
    }
    assert {world.ids[f"cut_{i}"] for i in range(5)} | {
        world.ids[f"cut_set_{i}"] for i in range(5)
    } <= produced
    origin_roles = {
        r[0]
        for r in conn.execute(
            "SELECT role::text FROM prov.record_origin WHERE record = ANY(%s)",
            ([world.ids[f"cut_set_{i}"] for i in range(5)],),
        ).fetchall()
    }
    assert origin_roles == {"derived"}


def test_each_cut_carries_the_properties_its_point_of_the_curve_reported(
    world: World, conn: psycopg.Connection
) -> None:
    for index in range(5):
        row = conn.execute(
            'SELECT molar_mass, normal_boiling_temperature, specific_gravity FROM param."assay_cut__cut" WHERE id = %s',
            (world.ids[f"cut_set_{index}"],),
        ).fetchone()
        assert row == pytest.approx(
            (MOLAR_MASS_G[index] * 1e-3, TEMPERATURE[index], GRAVITY[index]), rel=1e-12
        )
        bound = bind(
            world.decl,
            "assay_cut",
            source=DatabaseSource(conn, world.decl, [world.ids["assay_parameterization"]]),
            roles={"i": str(world.ids[f"cut_{index}"])},
            cache=CACHE,
        )
        for output, want in (
            ("M", MOLAR_MASS_G[index] * 1e-3),
            ("Tb", TEMPERATURE[index]),
            ("SG", GRAVITY[index]),
        ):
            assert float(np.asarray(bound.evaluate(output)).reshape(-1)[0]) == pytest.approx(
                want, rel=1e-12
            )


def test_a_pseudo_component_has_no_formula_and_a_cut_without_a_set_is_refused(
    world: World, conn: psycopg.Connection
) -> None:
    assert (
        scalar(
            conn,
            "SELECT count(*) FROM tk.composition WHERE entity = ANY(%s)",
            [world.ids[f"cut_{i}"] for i in range(5)],
        )
        == 0
    )
    stranger = uuid.uuid4()
    bound = bind(
        world.decl,
        "assay_cut",
        source=DatabaseSource(conn, world.decl, [world.ids["assay_parameterization"]]),
        roles={"i": str(stranger)},
        cache=CACHE,
    )
    with pytest.raises(EvaluationRefusal, match="assay_cut.cut"):
        bound.evaluate("M")


# -- the estimation through correlation forms -----------------------------------------------------


def test_the_power_laws_match_numpy(world: World, conn: psycopg.Connection) -> None:
    Tb = np.array([350.0, 420.0, 505.0, 610.0, 720.0])
    SG = np.array([0.68, 0.74, 0.81, 0.88, 0.95])
    found = evaluate(
        world,
        conn,
        "riazi_daubert_critical_temperature",
        [world.ids["tc_parameterization"]],
        "Tc",
        Tb=Tb,
        SG=SG,
    )
    np.testing.assert_allclose(found, riazi_daubert(RD_TC, Tb, SG), rtol=1e-12)
    found = evaluate(
        world,
        conn,
        "riazi_daubert_critical_pressure",
        [world.ids["pc_parameterization"]],
        "Pc",
        Tb=Tb,
        SG=SG,
    )
    np.testing.assert_allclose(found, riazi_daubert(RD_PC, Tb, SG), rtol=1e-12)


def test_two_correlation_forms_implement_one_contract_and_differ(
    world: World, conn: psycopg.Connection
) -> None:
    decl = world.decl
    assert (
        decl.forms["riazi_daubert_critical_temperature"].implements
        == decl.forms["kesler_lee_critical_temperature"].implements
    )
    Tb = np.array([350.0, 420.0, 505.0, 610.0, 720.0])
    SG = np.array([0.68, 0.74, 0.81, 0.88, 0.95])
    found = evaluate(
        world,
        conn,
        "kesler_lee_critical_temperature",
        [world.ids["kl_parameterization"]],
        "Tc",
        Tb=Tb,
        SG=SG,
    )
    np.testing.assert_allclose(found, kesler_lee(Tb, SG), rtol=1e-12)
    assert np.all(
        np.abs(found - riazi_daubert(RD_TC, Tb, SG)) > 1.0
    )  # alternatives: the choice of form matters


def test_edmister_has_no_coefficient_and_matches_numpy(
    world: World, conn: psycopg.Connection
) -> None:
    assert [g.name for g in world.decl.forms["edmister_acentric_factor"].slot_groups] == []
    Tb = np.array([350.0, 420.0, 505.0, 610.0])
    Tc = np.array([520.0, 610.0, 700.0, 790.0])
    Pc = np.array([3.4e6, 2.6e6, 1.9e6, 1.4e6])
    found = evaluate(world, conn, "edmister_acentric_factor", [], "omega", Tb=Tb, Tc=Tc, Pc=Pc)
    np.testing.assert_allclose(found, edmister(Tb, Tc, Pc), rtol=1e-12)


def test_the_estimated_constants_are_what_the_correlation_forms_give_at_the_stored_cut_properties(
    world: World, conn: psycopg.Connection
) -> None:
    for name in [f"cut_{i}" for i in range(5)] + ["lump"]:
        Tb, SG = (
            float(v)
            for v in conn.execute(
                'SELECT normal_boiling_temperature, specific_gravity FROM param."assay_cut__cut" WHERE id = %s',
                (world.ids["lump_set" if name == "lump" else name.replace("cut_", "cut_set_")],),
            ).fetchone()  # type: ignore[union-attr]
        )
        Tc = evaluate(
            world,
            conn,
            "riazi_daubert_critical_temperature",
            [world.ids["tc_parameterization"]],
            "Tc",
            Tb=np.array([Tb]),
            SG=np.array([SG]),
        )[0]
        Pc = evaluate(
            world,
            conn,
            "riazi_daubert_critical_pressure",
            [world.ids["pc_parameterization"]],
            "Pc",
            Tb=np.array([Tb]),
            SG=np.array([SG]),
        )[0]
        omega = evaluate(
            world,
            conn,
            "edmister_acentric_factor",
            [],
            "omega",
            Tb=np.array([Tb]),
            Tc=np.array([Tc]),
            Pc=np.array([Pc]),
        )[0]
        stored = conn.execute(
            'SELECT critical_temperature, critical_pressure, acentric_factor FROM param."estimated_critical_constants__critical" WHERE id = %s',
            (world.ids[f"critical_{name}"],),
        ).fetchone()
        assert stored == pytest.approx((Tc, Pc, omega), rel=1e-11)
        bound = bind(
            world.decl,
            "estimated_critical_constants",
            source=DatabaseSource(conn, world.decl, [world.ids["constants_parameterization"]]),
            roles={"i": str(world.ids[name])},
            cache=CACHE,
        )
        assert float(np.asarray(bound.evaluate("omega")).reshape(-1)[0]) == pytest.approx(
            omega, rel=1e-11
        )


def test_the_estimation_names_the_correlation_forms_through_the_sets_it_consumed(
    world: World, conn: psycopg.Connection
) -> None:
    estimation = world.ids["estimation"]
    assert (
        scalar(conn, "SELECT kind::text FROM prov.derivation WHERE id = %s", estimation)
        == "estimation"
    )
    forms = {
        r[0]
        for r in conn.execute(
            "SELECT g.form FROM prov.derivation_input i JOIN tk.parameter_set s ON s.id = i.record "
            "JOIN meta.slot_group g ON g.id = s.slot_group WHERE i.derivation = %s",
            (estimation,),
        ).fetchall()
    }
    assert forms == {
        "riazi_daubert_critical_temperature",
        "riazi_daubert_critical_pressure",
        "assay_cut",
    }
    # the Kesler-Lee coefficients are a parameterisation no derivation consumed
    assert (
        scalar(
            conn, "SELECT count(*) FROM prov.derivation_input WHERE record = %s", world.ids["kl_tc"]
        )
        == 0
    )
    roles = {
        r[0]
        for r in conn.execute(
            "SELECT role::text FROM prov.record_origin WHERE record = ANY(%s)",
            ([world.ids[f"critical_cut_{i}"] for i in range(5)] + [world.ids["critical_lump"]],),
        ).fetchall()
    }
    assert roles == {"estimated"}
    assert (
        scalar(
            conn, "SELECT count(*) FROM prov.derivation_output WHERE derivation = %s", estimation
        )
        == 7
    )  # six sets and their parameterisation


# -- the lump --------------------------------------------------------------------------------------


def test_a_lump_is_a_pseudo_component_its_lumping_derivation_produced_from_two_cuts(
    world: World, conn: psycopg.Connection
) -> None:
    lumping = world.ids["lumping"]
    kind, producer = conn.execute(
        "SELECT kind::text, produced_by FROM tk.pseudo_component WHERE id = %s",
        (world.ids["lump"],),
    ).fetchone()  # type: ignore[misc]
    assert (kind, producer) == ("lump", lumping)
    inputs = dict(
        conn.execute(
            "SELECT record, weight FROM prov.derivation_input WHERE derivation = %s", (lumping,)
        ).fetchall()
    )
    assert inputs == {
        world.ids[f"cut_{i}"]: w for i, w in zip(LUMP_OF, LUMP_FRACTIONS, strict=True)
    }
    assert sum(inputs.values()) == pytest.approx(1.0, rel=1e-12)


def test_the_lump_holds_the_moments_its_rule_states(world: World, conn: psycopg.Connection) -> None:
    members = conn.execute(
        'SELECT molar_mass, normal_boiling_temperature, specific_gravity FROM param."assay_cut__cut" WHERE id = ANY(%s) ORDER BY molar_mass',
        ([world.ids[f"cut_set_{i}"] for i in LUMP_OF],),
    ).fetchall()
    arrays = [np.array(column) for column in zip(*members, strict=True)]
    mass, boiling, gravity = lumped(arrays[0], arrays[1], arrays[2], np.array(LUMP_FRACTIONS))
    row = conn.execute(
        'SELECT molar_mass, normal_boiling_temperature, specific_gravity FROM param."assay_cut__cut" WHERE id = %s',
        (world.ids["lump_set"],),
    ).fetchone()
    assert row == pytest.approx((mass, boiling, gravity), rel=1e-12)
    assert arrays[0][0] < mass < arrays[0][1]  # between the molar masses of its cuts
    assert mass == pytest.approx(
        sum(z * m for z, m in zip(LUMP_FRACTIONS, arrays[0], strict=True)), rel=1e-12
    )


# -- what the model refuses ------------------------------------------------------------------------


def test_the_writer_refuses_a_constant_of_the_wrong_dimension_a_negative_temperature_and_an_unknown_kind(
    decl: Declaration,
) -> None:
    w = writer(decl)
    cut = w.kind(
        "pseudo_component",
        {"canonical_key": "c", "label": "c", "kind": "assay_cut"},
        origins=at("c", "published"),
    )
    p = parameterization(w, "p")
    with pytest.raises(ValidationError, match="cannot be converted"):
        w.parameter_set(
            parameterization=p,
            slot_group="estimated_critical_constants.critical",
            subjects=[cut],
            slots=critical_slots(500.0, 3.0e6, 0.3)
            | {"critical_temperature": Quantity(500.0, "Pa")},
            origins=at("wrong-unit", "published"),
        )
    with pytest.raises(ValidationError, match="critical_temperature"):
        w.parameter_set(
            parameterization=p,
            slot_group="estimated_critical_constants.critical",
            subjects=[cut],
            slots=critical_slots(-5.0, 3.0e6, 0.3),
            origins=at("negative", "published"),
        )
    with pytest.raises(ValidationError, match="not a member of enum `pseudo_component_kind`"):
        w.kind(
            "pseudo_component",
            {"canonical_key": "d", "label": "d", "kind": "distillation_cut"},
            origins=at("d", "published"),
        )
    with pytest.raises(
        ValidationError,
        match="is a species, and the role of `assay_cut.cut` is of kind `pseudo_component`",
    ):
        species = w.kind(
            "species", {"canonical_key": "s", "label": "s"}, origins=at("s", "published")
        )
        w.parameter_set(
            parameterization=p,
            slot_group="assay_cut.cut",
            subjects=[species],
            slots=cut_slots(0.1, 400.0, 0.7),
            origins=at("on-species", "published"),
        )


def test_the_verify_checks_flag_lineage_that_does_not_hold(
    decl: Declaration, tmp_path: Path
) -> None:
    """A set with the role `estimated` that no derivation produced, an estimation with no method, a
    pseudo-component whose producer is not the derivation that has it as an output (and the reverse:
    a produced pseudo-component that names none), and a record with two producers."""
    ids: dict[str, uuid.UUID] = {}

    def emit(w: CanonicalWriter) -> None:
        p = parameterization(w, "p", "derived")
        producing = w.kind(
            "derivation",
            {"key": "producing", "kind": "characterisation"},
            origins=at("producing", "published"),
        )
        other = w.kind(
            "derivation",
            {"key": "other", "kind": "characterisation"},
            origins=at("other", "published"),
        )
        w.relation(
            "derivation_output", {"derivation": producing, "record": p}, {}, at="a.json#/lineage"
        )
        named_not_output = w.kind(
            "pseudo_component",
            {
                "canonical_key": "named",
                "label": "named",
                "kind": "assay_cut",
                "produced_by": producing,
            },
            origins=at("named", "published"),
        )
        ids["named_not_output"] = named_not_output
        output_not_named = w.kind(
            "pseudo_component",
            {"canonical_key": "unnamed", "label": "unnamed", "kind": "assay_cut"},
            origins=at("unnamed", "published"),
        )
        ids["output_not_named"] = output_not_named
        w.relation(
            "derivation_output",
            {"derivation": producing, "record": output_not_named},
            {},
            at="a.json#/lineage",
        )
        other_output = w.kind(
            "pseudo_component",
            {
                "canonical_key": "other-output",
                "label": "other output",
                "kind": "assay_cut",
                "produced_by": producing,
            },
            origins=at("other-output", "published"),
        )
        ids["other_output"] = other_output
        w.relation(
            "derivation_output",
            {"derivation": other, "record": other_output},
            {},
            at="a.json#/lineage",
        )
        w.kind(
            "derivation",
            {"key": "estimation", "kind": "estimation"},
            origins=at("estimation", "published"),
        )
        cut = w.kind(
            "pseudo_component",
            {"canonical_key": "fine", "label": "fine", "kind": "assay_cut"},
            origins=at("fine", "published"),
        )
        w.parameter_set(
            parameterization=p,
            slot_group="estimated_critical_constants.critical",
            subjects=[cut],
            slots=critical_slots(500.0, 3.0e6, 0.3),
            origins=at("orphan", "estimated"),
        )
        shared = parameterization(w, "shared", "derived")
        for derivation in (producing, other):  # two producers of one record
            w.relation(
                "derivation_output",
                {"derivation": derivation, "record": shared},
                {},
                at="a.json#/lineage",
            )

    database = build(tmp_path, emit, decl)
    try:
        with psycopg.connect(database.url, autocommit=True) as connection:
            found = failing(connection)
            assert found["producing_derivation"] == 1  # the estimated set nobody produced
            assert found["derivation.estimation_names_method"] == 1
            assert found["derivation_output.one_producer_per_record"] == 1
            assert found["pseudo_component.produced_by_matches_lineage"] == 3
            flagged = run_check(
                connection, CHECKS["pseudo_component.produced_by_matches_lineage"], shown=10
            )
            assert sorted(flagged.ids) == sorted(
                str(ids[k]) for k in ("named_not_output", "output_not_named", "other_output")
            )
    finally:
        database.remove()
