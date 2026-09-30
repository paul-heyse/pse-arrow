# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Hard case (n), plan 24 packet TK2: evidence, evaluated values and fit lineage.

A ThermoML-like binary vapour-liquid equilibrium dataset: two components with their functions, a
liquid and a vapour phase, a constraint column and value columns, uncertainty assessments of
several kinds (standard, expanded with its coverage factor and level of confidence, relative, a
repeatability, and one on the constant of a constraint), a censored value and a value that was not
measured, and a sample with purity statements and a purification history. An ATcT-style evaluated
value with its expanded uncertainty whose evaluating derivation names the release of the
thermochemical network. An ESPEI-style fit whose inputs are datasets, with free parameters, their
standard uncertainties and a correlation, and whose output sets have the origin role `fitted`. A
transformed biochemical quantity: a column of a transformed observable whose constraint columns
are the pH, the ionic strength and the pMg.

There is no evaluation. The tests show the rows the writer stores and the refusals of the writer
and of `tk verify`; every number is synthetic.
"""

from __future__ import annotations

import uuid
from collections.abc import Iterator
from dataclasses import dataclass
from pathlib import Path

import psycopg
import pytest
from hard_case_support import at, build, declaration_with_vocabulary, entity_id, failing, observable_id
from mapping_support import carrier, origin, writer

from thermo_knowledge import db, identity
from thermo_knowledge.canonical.values import Quantity
from thermo_knowledge.canonical.writer import CanonicalWriter, FamilyRow, ValidationError
from thermo_knowledge.declaration import Declaration
from thermo_knowledge.testing import TestDatabase

CARRIER = identity.identifier("source", [carrier("src", "a.json").key])
PAIR_SLOT = "redlich_kister_pair.pair.order.L"  # the slot a free parameter of the fit addresses


@dataclass
class World:
    decl: Declaration
    database: TestDatabase
    ids: dict[str, uuid.UUID]


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


def assessment(w: CanonicalWriter, target: uuid.UUID, ordinal: int, kind: str, **extra: object) -> uuid.UUID:
    return w.kind(
        "uncertainty_assessment",
        {"column": target, "ordinal": ordinal, "kind": kind, **extra},
        at="a.json#/uncertainty",
    )


def datum(
    w: CanonicalWriter,
    point: uuid.UUID,
    target: uuid.UUID,
    state: str,
    value: Quantity | None,
    digits: int | None = 4,
) -> None:
    w.relation(
        "datum",
        {"point": point, "column": target},
        {"state": state, "value": value, "digits": digits},
        at="a.json#/datum",
    )


# -- the ThermoML-like dataset -------------------------------------------------------------------

X1 = (0.1, 0.3, 0.5, 0.7, 0.9)  # liquid mole fraction of the first component, per point
Y1 = (0.35, 0.52, 0.61, None, 0.90)  # vapour mole fraction; not measured at the fourth point
PRESSURE = (40.1, 55.2, 61.0, 64.3, 100.0)  # total pressure in kPa; the fifth is only a limit


def write_species_and_samples(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    for name in ("ethanol", "water"):
        ids[f"species_{name}"] = w.kind(
            "species", {"canonical_key": name, "label": name}, origins=at(f"s-{name}")
        )
    sample = w.kind(
        "sample",
        {
            "carrier": CARRIER,
            "local_key": "ethanol-batch",
            "entity": ids["species_ethanol"],
            "source": "commercial",
            "status": "described",
            "supplier": "a supplier",
        },
        origins=at("sample"),
    )
    ids["sample"] = sample
    by_key = {e.name: e.id for e in decl.entities if e.kind in ("analysis_method", "purification_method")}
    for ordinal, values in enumerate(
        (
            {"basis": "mole", "value": 0.998, "digits": 3, "method": by_key["gas_chromatography"]},
            {"basis": "mass", "value": 0.9985, "digits": 4, "method_text": "gas chromatography, area percent"},
            {
                "basis": "mass",
                "value": 0.0002,
                "digits": 1,
                "method": by_key["karl_fischer_titration"],
                "impurity": ids["species_water"],
            },
        ),
        start=1,
    ):
        w.relation("purity_statement", {"sample": sample, "ordinal": ordinal}, values, at="a.json#/purity")
    w.relation(
        "purification_step",
        {"sample": sample, "step": 1},
        {"method": by_key["molecular_sieve"]},
        at="a.json#/purification",
    )
    w.relation(
        "purification_step",
        {"sample": sample, "step": 2},
        {"method_text": "degassed by repeated freeze-pump-thaw cycles"},
        at="a.json#/purification",
    )


def write_vle(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    dataset = w.kind(
        "dataset",
        {
            "carrier": CARRIER,
            "local_key": "vle",
            "kind": "measured",
            "title": "Isothermal vapour-liquid equilibrium of a binary",
            "method": "static apparatus",
        },
        origins=at("vle", "measured"),
    )
    ids["vle"] = dataset
    first = w.kind(
        "dataset_component",
        {"dataset": dataset, "ordinal": 1, "entity": ids["species_ethanol"], "sample": ids["sample"], "function": "component"},
        at="a.json#/components",
    )
    w.kind(
        "dataset_component",
        {"dataset": dataset, "ordinal": 2, "entity": ids["species_water"], "function": "component"},
        at="a.json#/components",
    )
    liquid = w.kind(
        "dataset_phase",
        {"dataset": dataset, "ordinal": 1, "aggregation": entity_id(decl, "aggregation", "liquid"), "label": "liquid"},
        at="a.json#/phases",
    )
    vapour = w.kind(
        "dataset_phase",
        {"dataset": dataset, "ordinal": 2, "aggregation": entity_id(decl, "aggregation", "gas"), "label": "vapour"},
        at="a.json#/phases",
    )
    mole_fraction = entity_id(decl, "composition_basis", "mole_fraction")
    ids["col_T"] = column(
        w, decl, dataset, 1, "constraint", "temperature", constant=Quantity(350.0, "K"), constant_digits=5
    )
    ids["col_x"] = column(
        w, decl, dataset, 2, "variable", "mole_fraction", component=first, phase=liquid, composition_basis=mole_fraction
    )
    ids["col_y"] = column(
        w, decl, dataset, 3, "property", "mole_fraction", component=first, phase=vapour, composition_basis=mole_fraction
    )
    ids["col_P"] = column(w, decl, dataset, 4, "property", "pressure", phase=liquid)
    points = []
    for index in range(1, len(X1) + 1):
        point = w.kind("data_point", {"dataset": dataset, "index": index}, at="a.json#/points")
        points.append(point)
        ids[f"point_{index}"] = point
        datum(w, point, ids["col_x"], "known", Quantity(X1[index - 1], "dimensionless"))
        y = Y1[index - 1]
        datum(w, point, ids["col_y"], "known" if y is not None else "not_measured", None if y is None else Quantity(y, "dimensionless"))
        censored = index == len(X1)
        datum(
            w,
            point,
            ids["col_P"],
            "censored_above" if censored else "known",
            Quantity(PRESSURE[index - 1], "kPa"),
            digits=None if censored else 4,
        )
    # the assessments: a column has as many as its source defines
    x_standard = assessment(w, ids["col_x"], 1, "standard", evaluator="the authors", method="balance and composition")
    y_repeatability = assessment(w, ids["col_y"], 1, "repeatability_of_mean")
    p_standard = assessment(w, ids["col_P"], 1, "standard", evaluator="the authors", method="gauge specification")
    p_expanded = assessment(
        w, ids["col_P"], 2, "expanded", coverage_factor=2.0, confidence_level=0.95, evaluator="the authors"
    )
    p_relative = assessment(w, ids["col_P"], 3, "relative")
    t_standard = assessment(w, ids["col_T"], 1, "standard", evaluator="the supplier of the thermometer")
    ids.update(p_standard=p_standard, p_expanded=p_expanded, p_relative=p_relative, t_standard=t_standard)
    w.relation(
        "column_uncertainty",
        {"assessment": t_standard},
        {"minus": Quantity(0.02, "K"), "plus": Quantity(0.02, "K")},
        at="a.json#/uncertainty",
    )
    for index, point in enumerate(points, start=1):
        w.relation(
            "datum_uncertainty",
            {"point": point, "assessment": x_standard},
            {"minus": Quantity(0.001, "dimensionless"), "plus": Quantity(0.001, "dimensionless")},
            at="a.json#/uncertainty",
        )
        if Y1[index - 1] is not None:
            w.relation(
                "datum_uncertainty",
                {"point": point, "assessment": y_repeatability},
                {"minus": Quantity(0.004, "dimensionless"), "plus": Quantity(0.004, "dimensionless")},
                at="a.json#/uncertainty",
            )
        if index < len(X1):  # a limit has no uncertainty of its own
            w.relation(
                "datum_uncertainty",
                {"point": point, "assessment": p_standard},
                {"minus": Quantity(0.15, "kPa"), "plus": Quantity(0.15, "kPa")},
                at="a.json#/uncertainty",
            )
            w.relation(
                "datum_uncertainty",
                {"point": point, "assessment": p_expanded},
                {"minus": Quantity(0.30, "kPa"), "plus": Quantity(0.30, "kPa")},
                at="a.json#/uncertainty",
            )
            w.relation(
                "datum_uncertainty",
                {"point": point, "assessment": p_relative},
                {"relative_minus": 0.002, "relative_plus": 0.002},
                at="a.json#/uncertainty",
            )


# -- the ATcT-style evaluated value ----------------------------------------------------------------


def write_evaluated(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    release = w.kind(
        "software_release",
        {"key": "atct-thermochemical-network-1.122", "title": "Thermochemical network", "version": "1.122"},
    )
    ids["network_release"] = release
    form = w.kind(
        "species_form",
        {
            "canonical_key": "water gas",
            "label": "water",
            "species": ids["species_water"],
            "aggregation": entity_id(decl, "aggregation", "gas"),
        },
        origins=at("f-water-gas"),
    )
    evaluation = w.kind(
        "derivation",
        {
            "key": "atct-evaluation",
            "kind": "evaluation",
            "method": "solution of the thermochemical network",
            "software": release,
        },
        origins=at("atct-evaluation", "published"),
    )
    ids["evaluation"] = evaluation
    dataset = w.kind(
        "dataset",
        {"carrier": CARRIER, "local_key": "atct", "kind": "evaluated", "title": "Evaluated enthalpy of formation"},
        origins=at("atct", "evaluated"),
    )
    ids["atct"] = dataset
    component = w.kind(
        "dataset_component",
        {"dataset": dataset, "ordinal": 1, "entity": form, "function": "component"},
        at="a.json#/components",
    )
    phase = w.kind(
        "dataset_phase",
        {"dataset": dataset, "ordinal": 1, "aggregation": entity_id(decl, "aggregation", "gas")},
        at="a.json#/phases",
    )
    ideal_gas = w.kind(
        "standard_state",
        {"key": "ideal-gas-1-bar", "kind": "pure_ideal_gas", "pressure_rule": "fixed", "pressure": Quantity(1.0, "bar")},
        origins=at("standard-state"),
    )
    column(w, decl, dataset, 1, "constraint", "temperature", constant=Quantity(298.15, "K"), constant_digits=5)
    value = column(
        w, decl, dataset, 2, "property", "molar_enthalpy_of_formation", component=component, phase=phase, standard_state=ideal_gas
    )
    point = w.kind("data_point", {"dataset": dataset, "index": 1}, at="a.json#/points")
    datum(w, point, value, "known", Quantity(-241.0, "kJ/mol"), digits=6)
    expanded = assessment(
        w, value, 1, "expanded", coverage_factor=2.0, confidence_level=0.95,
        evaluator="the thermochemical network", method="propagation through the network",
    )
    w.relation(
        "datum_uncertainty",
        {"point": point, "assessment": expanded},
        {"minus": Quantity(0.03, "kJ/mol"), "plus": Quantity(0.03, "kJ/mol")},
        at="a.json#/uncertainty",
    )
    w.relation("derivation_output", {"derivation": evaluation, "record": dataset}, {}, at="a.json#/lineage")


# -- the ESPEI-style fit ----------------------------------------------------------------------------

FITTED = {0: (4000.0, 250.0), 1: (900.0, 120.0)}  # order -> (L in J/mol, its standard uncertainty)
CORRELATION = -0.8


def write_fit(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    for name in ("Fe", "Cr"):
        ids[f"species_{name}"] = w.kind("species", {"canonical_key": name, "label": name}, origins=at(f"s-{name}"))
    p = w.kind(
        "parameterization",
        {"key": "fitted-binaries", "revision": "1", "title": "A fitted database", "coherence": "jointly_fitted"},
        origins=at("fitted-parameterization"),
    )
    ids["fitted_parameterization"] = p
    fitted = w.parameter_set(
        parameterization=p,
        slot_group="redlich_kister_pair.pair",
        subjects=[ids["species_Fe"], ids["species_Cr"]],
        slots={},
        families={"order": [FamilyRow({"k": k}, {"L": Quantity(value, "J/mol")}) for k, (value, _) in FITTED.items()]},
        origins=at("fitted-binary", "fitted"),
    )
    ids["fitted_set"] = fitted
    software = w.kind(
        "software_release", {"key": "espei-0.9", "title": "A fitting program", "version": "0.9"}
    )
    fit = w.kind(
        "fit",
        {
            "key": "espei-run",
            "kind": "fit",
            "method": "Markov chain Monte Carlo",
            "software": software,
            "objective": "log posterior of the equilibrium and thermochemical data",
            "outcome": "converged",
        },
        origins=at("fit", "published"),
    )
    ids["fit"] = fit
    for dataset, weight in ((ids["vle"], 1.0), (ids["atct"], 0.5)):
        w.relation(
            "derivation_input", {"derivation": fit, "record": dataset}, {"weight": weight}, at="a.json#/lineage"
        )
    for ordinal, (k, (_, uncertainty)) in enumerate(FITTED.items(), start=1):
        w.relation(
            "fit_free_parameter",
            {"fit": fit, "ordinal": ordinal},
            {
                "record": fitted,
                "slot": PAIR_SLOT,
                "index_key": identity.canonical_encoding([k]),
                "standard_uncertainty": Quantity(uncertainty, "J/mol"),
            },
            at="a.json#/free",
        )
    w.relation("fit_correlation", {"fit": fit, "a": 2, "b": 1}, {"coefficient": CORRELATION}, at="a.json#/free")
    w.relation("derivation_output", {"derivation": fit, "record": fitted}, {}, at="a.json#/lineage")


# -- the transformed biochemical quantity ------------------------------------------------------------


def write_biochemical(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    liquid = entity_id(decl, "aggregation", "liquid")
    forms = {}
    for name in ("ATP", "ADP", "Pi", "H2O"):
        species = w.kind("species", {"canonical_key": f"bio-{name}", "label": name}, origins=at(f"s-bio-{name}"))
        forms[name] = w.kind(
            "species_form",
            {"canonical_key": f"bio-{name} aqueous", "label": name, "species": species, "aggregation": liquid},
            origins=at(f"f-bio-{name}"),
        )
    reaction = w.kind(
        "reaction", {"canonical_key": "ATP + H2O = ADP + Pi", "extent": "as_written"}, origins=at("hydrolysis")
    )
    for name, coefficient in (("ATP", -1), ("H2O", -1), ("ADP", 1), ("Pi", 1)):
        w.relation(
            "reaction_participant", {"reaction": reaction, "form": forms[name]}, {"coefficient": coefficient}, at="a.json#/p"
        )
    dataset = w.kind(
        "dataset",
        {"carrier": CARRIER, "local_key": "hydrolysis", "kind": "measured", "reaction": reaction},
        origins=at("hydrolysis-data", "measured"),
    )
    ids["biochemical"] = dataset
    transformed = w.kind(
        "standard_state",
        {"key": "transformed-biochemical", "kind": "transformed_biochemical", "pressure_rule": "fixed", "pressure": Quantity(1.0, "bar")},
        origins=at("transformed-standard-state"),
    )
    column(w, decl, dataset, 1, "constraint", "temperature", constant=Quantity(298.15, "K"), constant_digits=5)
    column(w, decl, dataset, 2, "constraint", "ph", constant=Quantity(7.0, "dimensionless"), constant_digits=2)
    column(w, decl, dataset, 3, "constraint", "ionic_strength", constant=Quantity(0.25, "mol/kg"), constant_digits=2)
    column(w, decl, dataset, 4, "constraint", "pmg", constant=Quantity(3.0, "dimensionless"), constant_digits=2)
    ids["col_transformed"] = column(
        w, decl, dataset, 5, "property", "transformed_reaction_gibbs_energy", standard_state=transformed
    )
    point = w.kind("data_point", {"dataset": dataset, "index": 1}, at="a.json#/points")
    datum(w, point, ids["col_transformed"], "known", Quantity(-30.5, "kJ/mol"), digits=3)
    w.relation(
        "datum_uncertainty",
        {"point": point, "assessment": assessment(w, ids["col_transformed"], 1, "standard", evaluator="the authors")},
        {"minus": Quantity(0.4, "kJ/mol"), "plus": Quantity(0.4, "kJ/mol")},
        at="a.json#/uncertainty",
    )


def write_world(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    write_species_and_samples(w, decl, ids)
    write_vle(w, decl, ids)
    write_evaluated(w, decl, ids)
    write_fit(w, decl, ids)
    write_biochemical(w, decl, ids)


@pytest.fixture(scope="module")
def decl(tmp_path_factory: pytest.TempPathFactory) -> Declaration:
    return declaration_with_vocabulary(tmp_path_factory.mktemp("evidence-declaration"), "evidence")


@pytest.fixture(scope="module")
def world(decl: Declaration, tmp_path_factory: pytest.TempPathFactory) -> Iterator[World]:
    ids: dict[str, uuid.UUID] = {}
    database = build(tmp_path_factory.mktemp("evidence"), lambda w: write_world(w, decl, ids), decl)
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


# -- the VLE dataset ---------------------------------------------------------------------------------


def test_the_fixture_satisfies_every_invariant(conn: psycopg.Connection) -> None:
    assert failing(conn) == {}


def test_the_dataset_has_components_with_functions_and_phases(world: World, conn: psycopg.Connection) -> None:
    rows = conn.execute(
        "SELECT c.ordinal, e.canonical_key, c.function::text, c.sample IS NOT NULL FROM ev.dataset_component c "
        "JOIN tk.material_entity e ON e.id = c.entity WHERE c.dataset = %s ORDER BY c.ordinal",
        (world.ids["vle"],),
    ).fetchall()
    assert rows == [(1, "ethanol", "component", True), (2, "water", "component", False)]
    phases = conn.execute(
        "SELECT ordinal, label FROM ev.dataset_phase WHERE dataset = %s ORDER BY ordinal", (world.ids["vle"],)
    ).fetchall()
    assert phases == [(1, "liquid"), (2, "vapour")]


def test_the_columns_are_typed_by_role_observable_component_phase_and_basis(
    world: World, conn: psycopg.Connection
) -> None:
    rows = conn.execute(
        "SELECT c.ordinal, c.role::text, o.key, c.component IS NOT NULL, p.label, b.name "
        "FROM ev.dataset_column c JOIN tk.observable o ON o.id = c.observable "
        "LEFT JOIN ev.dataset_phase p ON p.id = c.phase LEFT JOIN tk.composition_basis b ON b.id = c.composition_basis "
        "WHERE c.dataset = %s ORDER BY c.ordinal",
        (world.ids["vle"],),
    ).fetchall()
    assert rows == [
        (1, "constraint", "temperature", False, None, None),
        (2, "variable", "mole_fraction", True, "liquid", "mole_fraction"),
        (3, "property", "mole_fraction", True, "vapour", "mole_fraction"),
        (4, "property", "pressure", False, "liquid", None),
    ]
    constant = conn.execute(
        'SELECT constant, constant_digits FROM ev.dataset_column WHERE id = %s', (world.ids["col_T"],)
    ).fetchone()
    assert constant == (pytest.approx(350.0), 5)


def test_a_column_has_as_many_assessments_as_its_source_defines_each_with_what_it_states(
    world: World, conn: psycopg.Connection
) -> None:
    rows = conn.execute(
        "SELECT a.ordinal, a.kind::text, a.coverage_factor, a.confidence_level FROM ev.uncertainty_assessment a "
        "WHERE a.column = %s ORDER BY a.ordinal",
        (world.ids["col_P"],),
    ).fetchall()
    assert rows == [(1, "standard", None, None), (2, "expanded", 2.0, 0.95), (3, "relative", None, None)]
    magnitudes = conn.execute(
        "SELECT a.ordinal, u.minus, u.plus, u.relative_minus, u.relative_plus FROM ev.datum_uncertainty u "
        "JOIN ev.uncertainty_assessment a ON a.id = u.assessment WHERE a.column = %s AND u.point = %s ORDER BY a.ordinal",
        (world.ids["col_P"], world.ids["point_2"]),
    ).fetchall()
    assert magnitudes == [
        (1, pytest.approx(150.0), pytest.approx(150.0), None, None),  # kPa stored in pascal
        (2, pytest.approx(300.0), pytest.approx(300.0), None, None),
        (3, None, None, 0.002, 0.002),
    ]


def test_the_constant_of_a_constraint_column_has_its_own_uncertainty(world: World, conn: psycopg.Connection) -> None:
    row = conn.execute(
        "SELECT a.kind::text, u.minus, u.plus FROM ev.column_uncertainty u "
        "JOIN ev.uncertainty_assessment a ON a.id = u.assessment WHERE a.column = %s",
        (world.ids["col_T"],),
    ).fetchone()
    assert row == ("standard", pytest.approx(0.02), pytest.approx(0.02))


def test_a_censored_value_is_a_limit_with_a_side_and_a_missing_one_has_no_value(
    world: World, conn: psycopg.Connection
) -> None:
    rows = conn.execute(
        'SELECT p.index, d.state::text, d.value FROM ev.datum d JOIN ev.data_point p ON p.id = d.point '
        'WHERE d."column" = %s ORDER BY p.index',
        (world.ids["col_P"],),
    ).fetchall()
    assert [(index, state) for index, state, _ in rows] == [(1, "known"), (2, "known"), (3, "known"), (4, "known"), (5, "censored_above")]
    assert rows[4][2] == pytest.approx(1.0e5)  # the limit, in pascal
    missing = scalar(
        conn,
        'SELECT count(*) FROM ev.datum WHERE "column" = %s AND state = \'not_measured\' AND value IS NULL',
        world.ids["col_y"],
    )
    assert missing == 1


def test_a_limit_carries_no_uncertainty_of_its_own_and_a_missing_value_none_at_all(
    world: World, conn: psycopg.Connection
) -> None:
    on_limit = scalar(
        conn, "SELECT count(*) FROM ev.datum_uncertainty WHERE point = %s", world.ids["point_5"]
    )
    assert on_limit == 2  # the liquid and the vapour composition; the limit of the pressure has none
    rows = conn.execute(
        "SELECT a.column FROM ev.datum_uncertainty u JOIN ev.uncertainty_assessment a ON a.id = u.assessment "
        "WHERE u.point = %s",
        (world.ids["point_5"],),
    ).fetchall()
    assert world.ids["col_P"] not in {column for (column,) in rows}
    fourth = conn.execute(
        "SELECT a.column FROM ev.datum_uncertainty u JOIN ev.uncertainty_assessment a ON a.id = u.assessment "
        "WHERE u.point = %s",
        (world.ids["point_4"],),
    ).fetchall()
    assert world.ids["col_y"] not in {column for (column,) in fourth}


def test_the_sample_states_its_purity_in_several_bases_and_its_purification_history(
    world: World, conn: psycopg.Connection
) -> None:
    rows = conn.execute(
        "SELECT s.ordinal, s.basis::text, s.value, s.digits, m.name, s.method_text, i.canonical_key "
        "FROM tk.purity_statement s LEFT JOIN tk.analysis_method m ON m.id = s.method "
        "LEFT JOIN tk.material_entity i ON i.id = s.impurity WHERE s.sample = %s ORDER BY s.ordinal",
        (world.ids["sample"],),
    ).fetchall()
    assert rows == [
        (1, "mole", pytest.approx(0.998), 3, "gas chromatography", None, None),
        (2, "mass", pytest.approx(0.9985), 4, None, "gas chromatography, area percent", None),
        (3, "mass", pytest.approx(0.0002), 1, "Karl Fischer titration", None, "water"),
    ]
    steps = conn.execute(
        "SELECT p.step, m.name, p.method_text FROM tk.purification_step p "
        "LEFT JOIN tk.purification_method m ON m.id = p.method WHERE p.sample = %s ORDER BY p.step",
        (world.ids["sample"],),
    ).fetchall()
    assert steps == [(1, "molecular sieve", None), (2, None, "degassed by repeated freeze-pump-thaw cycles")]


# -- the evaluated value -------------------------------------------------------------------------------


def test_the_evaluated_value_has_its_expanded_uncertainty_and_its_level_of_confidence(
    world: World, conn: psycopg.Connection
) -> None:
    row = conn.execute(
        "SELECT d.state::text, d.value, a.kind::text, a.coverage_factor, a.confidence_level, u.minus, u.plus "
        "FROM ev.datum d JOIN ev.uncertainty_assessment a ON a.column = d.\"column\" "
        "JOIN ev.datum_uncertainty u ON u.assessment = a.id AND u.point = d.point "
        "JOIN ev.dataset_column c ON c.id = d.\"column\" WHERE c.dataset = %s",
        (world.ids["atct"],),
    ).fetchone()
    assert row == (
        "known", pytest.approx(-241_000.0), "expanded", 2.0, 0.95, pytest.approx(30.0), pytest.approx(30.0)
    )


def test_the_version_of_the_thermochemical_network_is_the_release_of_the_evaluating_derivation(
    world: World, conn: psycopg.Connection
) -> None:
    row = conn.execute(
        "SELECT s.version, o.role::text FROM prov.derivation_output out "
        "JOIN prov.derivation d ON d.id = out.derivation JOIN prov.software_release s ON s.id = d.software "
        "JOIN prov.record_origin o ON o.record = out.record WHERE out.record = %s",
        (world.ids["atct"],),
    ).fetchone()
    assert row == ("1.122", "evaluated")
    assert scalar(conn, "SELECT kind::text FROM prov.derivation WHERE id = %s", world.ids["evaluation"]) == "evaluation"


# -- the fit -----------------------------------------------------------------------------------------


def test_the_inputs_of_the_fit_are_datasets_with_their_weights(world: World, conn: psycopg.Connection) -> None:
    rows = conn.execute(
        "SELECT i.record, i.weight, i.excluded, r.kind FROM prov.derivation_input i "
        "JOIN prov.record r ON r.id = i.record WHERE i.derivation = %s ORDER BY i.weight DESC",
        (world.ids["fit"],),
    ).fetchall()
    assert rows == [
        (world.ids["vle"], 1.0, False, "dataset"),
        (world.ids["atct"], 0.5, False, "dataset"),
    ]


def test_the_free_parameters_are_addressed_to_the_coefficient_with_their_standard_uncertainties(
    world: World, conn: psycopg.Connection
) -> None:
    rows = conn.execute(
        "SELECT f.ordinal, f.record, s.qualified_name, f.index_key, f.standard_uncertainty "
        "FROM prov.fit_free_parameter f JOIN meta.slot s ON s.id = f.slot WHERE f.fit = %s ORDER BY f.ordinal",
        (world.ids["fit"],),
    ).fetchall()
    assert rows == [
        (1, world.ids["fitted_set"], PAIR_SLOT, identity.canonical_encoding([0]), pytest.approx(250.0)),
        (2, world.ids["fitted_set"], PAIR_SLOT, identity.canonical_encoding([1]), pytest.approx(120.0)),
    ]


def test_the_correlation_is_stored_once_in_the_canonical_orientation(world: World, conn: psycopg.Connection) -> None:
    rows = conn.execute("SELECT a, b, coefficient FROM prov.fit_correlation WHERE fit = %s", (world.ids["fit"],)).fetchall()
    assert rows == [(1, 2, CORRELATION)]  # written as (2, 1)


def test_the_output_set_is_fitted_and_is_produced_by_the_fit(world: World, conn: psycopg.Connection) -> None:
    role = scalar(conn, "SELECT role::text FROM prov.record_origin WHERE record = %s", world.ids["fitted_set"])
    assert role == "fitted"
    producer = scalar(conn, "SELECT derivation FROM prov.derivation_output WHERE record = %s", world.ids["fitted_set"])
    assert producer == world.ids["fit"]
    assert scalar(conn, "SELECT count(*) FROM prov.fit WHERE id = %s", producer) == 1
    values = conn.execute(
        'SELECT o."k", o."L" FROM param."redlich_kister_pair__pair__order" o WHERE o.set_id = %s ORDER BY o."k"',
        (world.ids["fitted_set"],),
    ).fetchall()
    assert values == [(k, pytest.approx(value)) for k, (value, _) in FITTED.items()]


# -- the transformed quantity ---------------------------------------------------------------------------


def test_a_transformed_quantity_is_a_column_whose_constraints_are_the_conditions_that_define_it(
    world: World, conn: psycopg.Connection
) -> None:
    rows = conn.execute(
        "SELECT c.ordinal, c.role::text, o.key, o.relation::text, c.constant FROM ev.dataset_column c "
        "JOIN tk.observable o ON o.id = c.observable WHERE c.dataset = %s ORDER BY c.ordinal",
        (world.ids["biochemical"],),
    ).fetchall()
    assert rows == [
        (1, "constraint", "temperature", "absolute", pytest.approx(298.15)),
        (2, "constraint", "ph", "absolute", pytest.approx(7.0)),
        (3, "constraint", "ionic_strength", "absolute", pytest.approx(0.25)),  # mol/kg is the storage unit
        (4, "constraint", "pmg", "absolute", pytest.approx(3.0)),
        (5, "property", "transformed_reaction_gibbs_energy", "transformed", None),
    ]
    state = scalar(
        conn,
        "SELECT s.kind::text FROM ev.dataset_column c JOIN tk.standard_state s ON s.id = c.standard_state WHERE c.id = %s",
        world.ids["col_transformed"],
    )
    assert state == "transformed_biochemical"


# -- what the model refuses -------------------------------------------------------------------------------


def test_a_censored_value_needs_its_limit_and_a_missing_value_cannot_have_one(decl: Declaration) -> None:
    w = writer(decl)
    ids: dict[str, uuid.UUID] = {}
    write_species_and_samples(w, decl, ids)
    dataset = w.kind(
        "dataset", {"carrier": CARRIER, "local_key": "x", "kind": "measured"}, origins=at("x", "measured")
    )
    target = column(w, decl, dataset, 1, "property", "pressure")
    point = w.kind("data_point", {"dataset": dataset, "index": 1}, at="a.json#/x")
    for state, value in (("censored_above", None), ("censored_below", None), ("known", None)):
        with pytest.raises(ValidationError, match="value_matches_state"):
            datum(w, point, target, state, value)
    with pytest.raises(ValidationError, match="value_matches_state"):
        datum(w, point, target, "not_measured", Quantity(1.0, "kPa"))
    assert w.rows("ev.datum") == 0


def test_a_constraint_column_states_its_constant_and_the_others_do_not(decl: Declaration) -> None:
    w = writer(decl)
    dataset = w.kind(
        "dataset", {"carrier": CARRIER, "local_key": "x", "kind": "measured"}, origins=at("x", "measured")
    )
    with pytest.raises(ValidationError, match="a constraint column states its constant"):
        column(w, decl, dataset, 1, "constraint", "temperature")
    with pytest.raises(ValidationError, match="a constraint column states its constant"):
        column(w, decl, dataset, 2, "property", "pressure", constant=Quantity(1.0, "bar"))


def test_a_purification_step_names_a_listed_method_or_a_text_and_not_both_or_neither(decl: Declaration) -> None:
    w = writer(decl)
    ids: dict[str, uuid.UUID] = {}
    write_species_and_samples(w, decl, ids)
    method = next(e.id for e in decl.entities if e.kind == "purification_method")
    for step, values in ((3, {}), (4, {"method": method, "method_text": "both"})):
        with pytest.raises(ValidationError, match="one_method"):
            w.relation(
                "purification_step", {"sample": ids["sample"], "step": step}, values, at="a.json#/purification"
            )


def test_a_standard_uncertainty_of_a_free_parameter_needs_the_slot_that_fixes_its_unit(decl: Declaration) -> None:
    w = writer(decl)
    fit = w.kind("fit", {"key": "f", "kind": "fit", "outcome": "converged"}, origins=at("fit", "published"))
    with pytest.raises(ValidationError, match="its unit comes from `slot`"):
        w.relation(
            "fit_free_parameter",
            {"fit": fit, "ordinal": 1},
            {"record": uuid.uuid4(), "standard_uncertainty": Quantity(1.0, "J/mol")},
        )
    assert w.rows("prov.fit_free_parameter") == 0


def test_the_checks_of_verify_flag_each_broken_evidence_and_lineage_record(decl: Declaration, tmp_path: Path) -> None:
    """One violation of each structure the case is about, each found by the check that states
    it: magnitudes that do not fit the kind, an uncertainty of a value that was not measured, an
    uncertainty on the constant of a column that is not a constraint, a record with the role
    `fitted` that no derivation produced, a fitted record that an estimation and not a fit
    produced, a record with two producers and a lineage that loops."""

    def emit(w: CanonicalWriter) -> None:
        ids: dict[str, uuid.UUID] = {}
        write_species_and_samples(w, decl, ids)
        write_vle(w, decl, ids)
        broken = w.kind(
            "dataset", {"carrier": CARRIER, "local_key": "broken", "kind": "measured"}, origins=at("broken", "measured")
        )
        broken_pressure = column(w, decl, broken, 1, "property", "pressure")
        broken_temperature = column(w, decl, broken, 2, "property", "temperature")
        point = w.kind("data_point", {"dataset": broken, "index": 1}, at="a.json#/points")
        datum(w, point, broken_pressure, "known", Quantity(1.0, "bar"))
        datum(w, point, broken_temperature, "not_measured", None)
        expanded = assessment(w, broken_pressure, 1, "expanded", coverage_factor=2.0)
        w.relation(  # an expanded uncertainty states its sides in the unit of the quantity, not relative ones
            "datum_uncertainty",
            {"point": point, "assessment": expanded},
            {"relative_minus": 0.01, "relative_plus": 0.01},
            at="a.json#/uncertainty",
        )
        unmeasured = assessment(w, broken_temperature, 1, "standard")
        w.relation(
            "datum_uncertainty",
            {"point": point, "assessment": unmeasured},
            {"minus": Quantity(0.1, "K"), "plus": Quantity(0.1, "K")},
            at="a.json#/uncertainty",
        )
        w.relation(
            "column_uncertainty",
            {"assessment": assessment(w, broken_pressure, 2, "standard")},
            {"minus": Quantity(0.1, "bar"), "plus": Quantity(0.1, "bar")},
            at="a.json#/uncertainty",
        )
        # lineage
        w.kind(
            "parameterization",
            {"key": "orphan", "revision": "1", "title": "fitted, produced by nothing", "coherence": "independent_records"},
            origins=at("orphan", "fitted"),
        )
        estimated = w.kind(
            "parameterization",
            {"key": "estimated-as-fitted", "revision": "1", "title": "x", "coherence": "independent_records"},
            origins=at("estimated", "fitted"),
        )
        estimation = w.kind(
            "derivation", {"key": "estimation", "kind": "estimation", "method": "a group method"}, origins=at("estimation", "published")
        )
        w.relation("derivation_output", {"derivation": estimation, "record": estimated}, {}, at="a.json#/lineage")
        first = w.kind("derivation", {"key": "first", "kind": "conversion"}, origins=at("first", "published"))
        second = w.kind("derivation", {"key": "second", "kind": "conversion"}, origins=at("second", "published"))
        shared = w.kind(
            "parameterization",
            {"key": "shared", "revision": "1", "title": "x", "coherence": "independent_records"},
            origins=at("shared", "derived"),
        )
        for derivation in (first, second):
            w.relation("derivation_output", {"derivation": derivation, "record": shared}, {}, at="a.json#/lineage")
        # the loop: the first derivation consumes a record it produced
        w.relation("derivation_input", {"derivation": first, "record": shared}, {}, at="a.json#/lineage")

    database = build(tmp_path, emit, decl)
    try:
        with psycopg.connect(database.url) as connection:
            found = failing(connection)
        assert found["datum_uncertainty.magnitude_matches_kind"] == 1
        assert found["datum_uncertainty.datum_exists"] == 1
        assert found["column_uncertainty.column_is_constraint"] == 1
        assert found["producing_derivation"] == 1  # the fitted record that nothing produced
        assert found["producing_derivation_is_fit"] == 1  # the fitted record that an estimation produced
        assert found["derivation_output.one_producer_per_record"] >= 1
        assert found["derivation_input.lineage_acyclic"] >= 1
    finally:
        database.remove()
