# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""A standard-state model assembled per species form (plan 24, packet TK2g, alignment item 16).

A substance of a ThermoFun-like database names up to three methods (a general equation of state, a
temperature correction, a pressure correction), and a solute equation of state needs a solvent
density and permittivity. `entity_model` holds the assembly of each species form under a
parameterisation, so two species of one parameterisation have different assemblies, and the
assembly names the form of each sub-form slot of a root form (the solvent forms are slots of the
equation of state, one level down, by `path`).

Which method code fills which slot is knowledge of the source, not of the model: `CODE_TO_FORM` and
`PART_TO_SLOT` below are the small mapping of this fixture, the content of a `mapping.toml`.
"""

from __future__ import annotations

import uuid
from collections.abc import Iterator
from dataclasses import dataclass

import numpy as np
import psycopg
import pytest
from build_support import fingerprint, inputs_of, write_source
from mapping_support import origin
from mechanisms_support import entity_id, mechanism_declaration

from thermo_knowledge import config, db
from thermo_knowledge.build import build_database
from thermo_knowledge.canonical.values import Quantity
from thermo_knowledge.canonical.writer import CanonicalWriter
from thermo_knowledge.declaration import Declaration
from thermo_knowledge.expression.evaluate import EvaluationRefusal, bind
from thermo_knowledge.qualify.source import DatabaseSource, SubformBinding
from thermo_knowledge.testing import TestDatabase
from thermo_knowledge.verify.checks import VERIFY_DIR, load_checks
from thermo_knowledge.verify.run import run_check, run_checks

CHECKS = {c.target: c for c in load_checks(config.TREE_DIR / VERIFY_DIR)[0]}
ROOT = "aqueous_ss_fixture"

# -- the mapping knowledge: what the source's method codes mean -----------------------------------

PART_TO_SLOT = {
    "method_genEoS": "aqueous_ss_fixture.eos",
    "method_corrT": "aqueous_ss_fixture.t_correction",
    "method_corrP": "aqueous_ss_fixture.p_correction",
}
CODE_TO_FORM = {
    "hkf": "hkf_fixture",
    "const": "constant_energy_fixture",
    "ds": "entropy_correction_fixture",
    "dv": "volume_correction_fixture",
}
SOLVENT_SLOTS_OF_FORM = {  # the sub-slots a chosen equation of state brings with it
    "hkf_fixture": {
        "hkf_fixture.epsilon": "permittivity_exponential_fixture",
        "hkf_fixture.density": "density_constant_fixture",
    }
}
SUBSTANCES = {
    "Na+": {"method_genEoS": "hkf", "method_corrT": "ds", "method_corrP": "dv"},
    "CO2(aq)": {"method_genEoS": "const"},
}
VALUES = {
    "Na+": {"g_ref": -261_900.0, "omega": 5.0e5, "a1": -0.5, "ds": -58.4, "dv": -1.1e-6},
    "CO2(aq)": {"g": -386_000.0},
}
SOLVENT = {"eps0": 78.0, "theta": 219.0, "rho": 997.0}


@dataclass
class World:
    decl: Declaration
    database: TestDatabase
    ids: dict[str, uuid.UUID]


def at(tag: str) -> list:  # noqa: ANN401
    return [origin(f"a.json#/{tag}", "published")]


def write_world(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    aqueous = entity_id(decl, "aggregation", "liquid")
    p = w.kind(
        "parameterization",
        {"key": "thermofun-like", "revision": "1", "title": "t", "coherence": "independent_records"},
        origins=at("p"),
    )
    ids["parameterization"] = p
    forms = {}
    for name in SUBSTANCES:
        species = w.kind("species", {"canonical_key": name, "label": name}, origins=at(f"s-{name}"))
        forms[name] = w.kind(
            "species_form",
            {"canonical_key": f"{name} aqueous", "label": name, "species": species, "aggregation": aqueous},
            origins=at(f"f-{name}"),
        )
        ids[f"form_{name}"] = forms[name]
    # the assembly of each substance is built from its method codes by the mapping above
    for name, methods in SUBSTANCES.items():
        key = "+".join(methods[part] for part in PART_TO_SLOT if part in methods)
        assembly = w.kind(
            "model_assembly",
            {"key": f"aqueous-{key}", "revision": "1", "title": key, "root": ROOT},
            origins=at(f"assembly-{key}"),
        )
        ids[f"assembly_{name}"] = assembly
        for part, code in methods.items():
            slot, form = PART_TO_SLOT[part], CODE_TO_FORM[code]
            path = slot.split(".")[1]
            w.kind(
                "assembly_choice",
                {"assembly": assembly, "path": path, "ordinal": 1, "slot": slot, "form": form},
                at="a.json#/choice",
            )
            for sub_slot, sub_form in SOLVENT_SLOTS_OF_FORM.get(form, {}).items():
                w.kind(
                    "assembly_choice",
                    {
                        "assembly": assembly,
                        "path": f"{path}/{sub_slot.split('.')[1]}",
                        "ordinal": 1,
                        "slot": sub_slot,
                        "form": sub_form,
                    },
                    at="a.json#/choice",
                )
        w.relation(
            "entity_model",
            {"parameterization": p, "entity": forms[name]},
            {"value": assembly},
            at="a.json#/entity-model",
        )
    # the parameter sets of the parts
    na, co2 = forms["Na+"], forms["CO2(aq)"]
    values = VALUES["Na+"]
    w.parameter_set(
        parameterization=p,
        slot_group="hkf_fixture.pure",
        subjects=[na],
        slots={
            "g_ref": Quantity(values["g_ref"], "J/mol"),
            "omega": Quantity(values["omega"], "J/mol"),
            "a1": Quantity(values["a1"], "J/mol"),
        },
        origins=at("hkf"),
    )
    w.parameter_set(
        parameterization=p,
        slot_group="entropy_correction_fixture.pure",
        subjects=[na],
        slots={"ds": Quantity(values["ds"], "J/(mol*K)")},
        origins=at("ds"),
    )
    w.parameter_set(
        parameterization=p,
        slot_group="volume_correction_fixture.pure",
        subjects=[na],
        slots={"dv": Quantity(values["dv"], "m^3/mol")},
        origins=at("dv"),
    )
    w.parameter_set(
        parameterization=p,
        slot_group="constant_energy_fixture.pure",
        subjects=[co2],
        slots={"g": Quantity(VALUES["CO2(aq)"]["g"], "J/mol")},
        origins=at("const"),
    )
    w.parameter_set(
        parameterization=p,
        slot_group="permittivity_exponential_fixture.core",
        subjects=[],
        slots={"eps0": SOLVENT["eps0"], "theta": Quantity(SOLVENT["theta"], "K")},
        origins=at("permittivity"),
    )
    w.parameter_set(
        parameterization=p,
        slot_group="density_constant_fixture.core",
        subjects=[],
        slots={"rho": Quantity(SOLVENT["rho"], "kg/m^3")},
        origins=at("density"),
    )


@pytest.fixture(scope="module")
def world(tmp_path_factory: pytest.TempPathFactory) -> Iterator[World]:
    decl = mechanism_declaration(tmp_path_factory.mktemp("assembly-declaration"))
    canonical = tmp_path_factory.mktemp("assembly-canonical")
    ids: dict[str, uuid.UUID] = {}
    write_source(canonical, "src", lambda w: write_world(w, decl, ids), decl=decl, declaration=fingerprint(decl))
    with TestDatabase() as database:
        build_database(database.url, decl, inputs_of(canonical))
        yield World(decl, database, ids)


@pytest.fixture
def conn(world: World) -> Iterator[psycopg.Connection]:
    connection = db.connect(world.database.url)
    connection.execute("SELECT 1")
    try:
        yield connection
    finally:
        connection.rollback()
        connection.close()


def assembly_of(world: World, conn: psycopg.Connection, name: str) -> uuid.UUID:
    row = conn.execute(
        "SELECT value FROM tk.entity_model WHERE parameterization = %s AND entity = %s",
        (world.ids["parameterization"], world.ids[f"form_{name}"]),
    ).fetchone()
    assert row is not None
    return row[0]


def bindings(world: World, conn: psycopg.Connection, name: str) -> dict[str, list[SubformBinding]]:
    """The forms the assembly of a species form puts in each sub-form slot, as the source reads
    them: this is the assembly read back out of the database."""
    rows = conn.execute(
        "SELECT sl.qualified_name, f.name FROM tk.assembly_choice c "
        "JOIN meta.subform_slot sl ON sl.id = c.slot JOIN meta.form f ON f.id = c.form "
        "WHERE c.assembly = %s ORDER BY c.path, c.ordinal",
        (assembly_of(world, conn, name),),
    ).fetchall()
    found: dict[str, list[SubformBinding]] = {}
    for slot, form in rows:
        found.setdefault(slot, []).append(SubformBinding(form, (world.ids["parameterization"],)))
    return found


def gibbs(world: World, conn: psycopg.Connection, name: str, T: float, P: float) -> float:
    source = DatabaseSource(
        conn, world.decl, [world.ids["parameterization"]], subforms=bindings(world, conn, name)
    )
    bound = bind(world.decl, ROOT, source=source, roles={"i": str(world.ids[f"form_{name}"])})
    return float(np.asarray(bound.evaluate("G0", T=np.array([T]), P=np.array([P]))).reshape(-1)[0])


def test_every_fixture_satisfies_every_invariant(world: World, conn: psycopg.Connection) -> None:
    results = run_checks(conn, list(CHECKS.values()))
    assert [(r.check.target, r.violations, r.error) for r in results if not r.passed] == []


def test_two_species_of_one_parameterisation_have_different_assemblies(
    world: World, conn: psycopg.Connection
) -> None:
    rows = conn.execute(
        "SELECT e.value, count(*) FROM tk.entity_model e JOIN tk.assembly_choice c ON c.assembly = e.value "
        "WHERE e.parameterization = %s GROUP BY e.value",
        (world.ids["parameterization"],),
    ).fetchall()
    counts = {assembly: n for assembly, n in rows}
    assert len(counts) == 2
    assert counts[world.ids["assembly_Na+"]] == 5  # eos, both corrections, and the two solvent slots of the eos
    assert counts[world.ids["assembly_CO2(aq)"]] == 1  # the equation of state alone


def test_a_solvent_slot_of_the_equation_of_state_is_reached_by_its_path(
    world: World, conn: psycopg.Connection
) -> None:
    rows = conn.execute(
        "SELECT c.path, sl.qualified_name, f.name FROM tk.assembly_choice c "
        "JOIN meta.subform_slot sl ON sl.id = c.slot JOIN meta.form f ON f.id = c.form "
        "WHERE c.assembly = %s ORDER BY c.path",
        (world.ids["assembly_Na+"],),
    ).fetchall()
    assert rows == [
        ("eos", "aqueous_ss_fixture.eos", "hkf_fixture"),
        ("eos/density", "hkf_fixture.density", "density_constant_fixture"),
        ("eos/epsilon", "hkf_fixture.epsilon", "permittivity_exponential_fixture"),
        ("p_correction", "aqueous_ss_fixture.p_correction", "volume_correction_fixture"),
        ("t_correction", "aqueous_ss_fixture.t_correction", "entropy_correction_fixture"),
    ]


def test_the_assembled_model_of_the_aqueous_ion_evaluates(world: World, conn: psycopg.Connection) -> None:
    T, P = 350.0, 2.0e7
    v = VALUES["Na+"]
    eps = SOLVENT["eps0"] * np.exp(-(T - 298.15) / SOLVENT["theta"])
    expected = (
        v["g_ref"] + v["omega"] * (1 / eps - 1) + v["a1"] * SOLVENT["rho"]  # the equation of state
        - v["ds"] * (T - 298.15)  # the temperature correction
        + v["dv"] * (P - 1.0e5)  # the pressure correction
    )
    assert gibbs(world, conn, "Na+", T, P) == pytest.approx(expected, rel=1e-13)


def test_the_second_species_uses_its_own_assembly_with_no_corrections_and_no_solvent(
    world: World, conn: psycopg.Connection
) -> None:
    assert gibbs(world, conn, "CO2(aq)", 350.0, 2.0e7) == VALUES["CO2(aq)"]["g"]


def test_the_assembly_of_one_species_is_not_applied_to_another(world: World, conn: psycopg.Connection) -> None:
    """The ion's assembly binds its corrections, but the second species holds no parameter set
    for them: the evaluation refuses rather than borrowing the ion's values."""
    source = DatabaseSource(
        conn, world.decl, [world.ids["parameterization"]], subforms=bindings(world, conn, "Na+")
    )
    bound = bind(world.decl, ROOT, source=source, roles={"i": str(world.ids["form_CO2(aq)"])})
    with pytest.raises(EvaluationRefusal, match="hkf_fixture.pure"):
        bound.evaluate("G0", T=np.array([350.0]), P=np.array([2.0e7]))


def test_the_code_to_slot_table_is_no_part_of_the_declaration(world: World) -> None:
    """The method codes of the source name no construct of the model: they are mapping
    knowledge, here `PART_TO_SLOT` and `CODE_TO_FORM`."""
    declared = {
        name
        for form in world.decl.forms.values()
        for name in (
            form.name,
            *(sub.name for sub in form.subforms),
            *(slot.name for group in form.slot_groups for slot in group.slots),
        )
    }
    assert declared.isdisjoint(PART_TO_SLOT)


def test_the_ordinals_of_the_choices_at_one_path_are_checked_like_those_of_a_subject_choice(
    world: World, conn: psycopg.Connection
) -> None:
    """A second choice for a slot of multiplicity one, a choice numbered from two and a gap are
    each flagged; the assembly as written has none."""
    contiguous = CHECKS["assembly_choice.ordinals_contiguous"]
    assert run_check(conn, contiguous).violations == 0
    conn.execute(
        "INSERT INTO tk.assembly_choice (id, assembly, path, ordinal, slot, form) "
        "SELECT gen_random_uuid(), assembly, path, 2, slot, form FROM tk.assembly_choice "
        "WHERE path = 'eos' AND assembly = %s",
        (world.ids["assembly_Na+"],),
    )
    flagged = run_check(conn, contiguous)
    assert flagged.error is None and flagged.violations == 1
    conn.execute("UPDATE tk.assembly_choice SET ordinal = 3 WHERE ordinal = 2")
    assert run_check(conn, contiguous).violations == 1
    conn.execute("DELETE FROM tk.assembly_choice WHERE ordinal = 3")
    conn.execute("UPDATE tk.assembly_choice SET ordinal = 2 WHERE path = 'eos' AND assembly = %s", (world.ids["assembly_Na+"],))
    assert run_check(conn, contiguous).violations == 1
