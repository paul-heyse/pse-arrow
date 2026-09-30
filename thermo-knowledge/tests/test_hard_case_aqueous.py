# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Hard case (f), plan 24 packet TK2: aqueous and electrolyte chemistry.

A PHREEQC-style chemical system: master (basis) and secondary species, the defining reactions of
the secondary species with their equilibrium constants as analytic temperature expressions, the
alkalinity as a conserved quantity, an exchanger and a surface with their site classes, exchange
and surface species that carry a site total as a conserved quantity, the Pitzer parameters of
cation-anion pairs, like-ion pairs and triplets, a specific-interaction coefficient, and the
standard-state model of a species assembled from its own parts (a Helgeson-Kirkham-Flowers ion and
a species of constant energy, in one parameterisation). `tk verify` shows that every reaction
conserves elements, charge, alkalinity and sites.

Every number is synthetic. The forms are the committed ones of `forms/aqueous_electrolyte.toml`;
log K(T), the van 't Hoff form, the Pitzer B^phi and the interaction term are evaluated against
the published formulas written in numpy here.
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
from hard_case_support import at, build, entity_id, failing
from mapping_support import real_declaration, writer

from thermo_knowledge import db
from thermo_knowledge.canonical.values import Quantity
from thermo_knowledge.canonical.writer import CanonicalWriter, ValidationError
from thermo_knowledge.declaration import Declaration
from thermo_knowledge.expression.compiled import CompileCache
from thermo_knowledge.expression.evaluate import EvaluationRefusal, bind
from thermo_knowledge.qualify.source import DatabaseSource, SubformBinding
from thermo_knowledge.testing import TestDatabase

CACHE = CompileCache()
R = 8.314462618  # the gas constant of the convention set, J/(mol K)
SYSTEM = "aq-fixture"
ELEMENTS = ("H", "O", "Na", "Cl", "Ca", "C")

# name -> (composition, charge, alkalinity, role in the system, member role, aggregation). The
# alkalinity is the count of the proton condition: a species counts the protons it has taken or
# lost against the master species of its own kind (the surface species against the neutral site),
# so every reaction between species of one system conserves it.
SPECIES = {
    "H2O": ({"H": 2, "O": 1}, 0, 0, "basis", "solvent", "liquid"),
    "H+": ({"H": 1}, 1, -1, "basis", "solute", "liquid"),
    "Na+": ({"Na": 1}, 1, 0, "basis", "solute", "liquid"),
    "Cl-": ({"Cl": 1}, -1, 0, "basis", "solute", "liquid"),
    "Ca+2": ({"Ca": 1}, 2, 0, "basis", "solute", "liquid"),
    "CO3-2": ({"C": 1, "O": 3}, -2, 2, "basis", "solute", "liquid"),
    "OH-": ({"H": 1, "O": 1}, -1, 1, "secondary", "solute", "liquid"),
    "HCO3-": ({"H": 1, "C": 1, "O": 3}, -1, 1, "secondary", "solute", "liquid"),
    "CO2(aq)": ({"C": 1, "O": 2}, 0, 0, "secondary", "solute", "liquid"),
    "CaCO3(aq)": ({"Ca": 1, "C": 1, "O": 3}, 0, 2, "secondary", "solute", "liquid"),
    "X-": ({"X": 1}, -1, 0, "basis", "exchange_species", "adsorbed"),
    "NaX": ({"Na": 1, "X": 1}, 0, 0, "secondary", "exchange_species", "adsorbed"),
    "CaX2": ({"Ca": 1, "X": 2}, 0, 0, "secondary", "exchange_species", "adsorbed"),
    "Hfo_sOH": ({"H": 1, "O": 1, "Hfo_s": 1}, 0, 0, "basis", "surface_species", "surface"),
    "Hfo_sOH2+": ({"H": 2, "O": 1, "Hfo_s": 1}, 1, -1, "secondary", "surface_species", "surface"),
    "Hfo_sO-": ({"O": 1, "Hfo_s": 1}, -1, 1, "secondary", "surface_species", "surface"),
}
# the reactions as written: (equation, {participant: coefficient}, the species it defines, if it defines one)
REACTIONS = {
    "water": ("H2O = H+ + OH-", {"H2O": -1, "H+": 1, "OH-": 1}, "OH-"),
    "bicarbonate": ("CO3-2 + H+ = HCO3-", {"CO3-2": -1, "H+": -1, "HCO3-": 1}, "HCO3-"),
    "carbon dioxide": ("CO3-2 + 2H+ = CO2 + H2O", {"CO3-2": -1, "H+": -2, "CO2(aq)": 1, "H2O": 1}, "CO2(aq)"),
    "carbon dioxide from bicarbonate": (
        "HCO3- + H+ = CO2 + H2O",
        {"HCO3-": -1, "H+": -1, "CO2(aq)": 1, "H2O": 1},
        None,
    ),
    "calcium carbonate": ("Ca+2 + CO3-2 = CaCO3", {"Ca+2": -1, "CO3-2": -1, "CaCO3(aq)": 1}, "CaCO3(aq)"),
    "sodium exchange": ("Na+ + X- = NaX", {"Na+": -1, "X-": -1, "NaX": 1}, "NaX"),
    "calcium exchange": ("Ca+2 + 2X- = CaX2", {"Ca+2": -1, "X-": -2, "CaX2": 1}, "CaX2"),
    "surface protonation": ("Hfo_sOH + H+ = Hfo_sOH2+", {"Hfo_sOH": -1, "H+": -1, "Hfo_sOH2+": 1}, "Hfo_sOH2+"),
    "surface deprotonation": ("Hfo_sOH = Hfo_sO- + H+", {"Hfo_sOH": -1, "Hfo_sO-": 1, "H+": 1}, "Hfo_sO-"),
}
ANALYTIC = {  # reaction -> (A1, A2, A3, A4, A5, A6)
    "water": (-180.0, -0.0312, 9100.0, 64.5, -7.4e5, 1.0e-5),
    "bicarbonate": (95.0, 0.0182, -4200.0, -36.0, 1.9e5, 0.0),
    "carbon dioxide": (210.0, 0.0441, -9800.0, -80.5, 4.2e5, -2.0e-6),
}
VAN_T_HOFF = {  # reaction -> (log10 K at 298.15 K, reaction enthalpy in J/mol)
    "carbon dioxide from bicarbonate": (-3.7632, -9_000.0),
    "calcium carbonate": (3.22, 15_000.0),
    "sodium exchange": (0.0, -2_500.0),
    "calcium exchange": (0.8, -4_000.0),
    "surface protonation": (7.29, -30_000.0),
    "surface deprotonation": (-8.93, 55_000.0),
}
# Pitzer cation-anion pairs: (beta0, beta1, beta2, alpha1, alpha2, C_phi)
PITZER = {
    ("Na+", "Cl-"): (0.081, 0.33, 0.0, 2.0, 12.0, 0.0012),
    ("Ca+2", "Cl-"): (0.29, 1.6, 0.0, 2.0, 12.0, 0.0008),
    ("Ca+2", "CO3-2"): (0.15, 2.9, -42.0, 1.4, 12.0, 0.0),
}
THETA = {("Na+", "Ca+2"): 0.07}
PSI = {("Na+", "Ca+2", "Cl-"): -0.012}
SIT = {("Na+", "Cl-"): 0.04}
HKF = {  # the parameters of one HKF ion, synthetic
    "g_f": -261_900.0, "h_f": -240_300.0, "s": 58.4, "a1": 1.8e-6, "a2": -2.0e3,
    "a3": 1.1e-4, "a4": -2.6e4, "c1": 75.0, "c2": -1.1e4, "omega": 5.3e5,
}
CONSTANT_G = -386_000.0


# -- the independent calculation ---------------------------------------------------------------


def log_k_analytic(a: tuple[float, ...], T: np.ndarray) -> np.ndarray:
    return a[0] + a[1] * T + a[2] / T + a[3] * np.log10(T) + a[4] / T**2 + a[5] * T**2


def log_k_van_t_hoff(log_k_ref: float, delta_h: float, T: np.ndarray) -> np.ndarray:
    return log_k_ref - delta_h / (R * math.log(10.0)) * (1.0 / T - 1.0 / 298.15)


def b_phi(beta: tuple[float, ...], I: np.ndarray) -> np.ndarray:
    beta0, beta1, beta2, alpha1, alpha2, _ = beta
    return beta0 + beta1 * np.exp(-alpha1 * np.sqrt(I)) + beta2 * np.exp(-alpha2 * np.sqrt(I))


# -- the fixture -------------------------------------------------------------------------------


@dataclass
class World:
    decl: Declaration
    database: TestDatabase
    ids: dict[str, uuid.UUID]


def write_chemistry(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    """The species, the conserved quantities, the system with its members, reactions and
    conserved quantities, the exchanger and surface with their site classes, and the convention
    set that gives the standard state of each role."""
    quantity: dict[str, uuid.UUID] = {"charge": entity_id(decl, "conserved_quantity", "charge")}
    for element in ELEMENTS:
        quantity[element] = entity_id(decl, "element", element)
    quantity["alk"] = w.kind(
        "conserved_quantity", {"key": f"{SYSTEM}:Alk", "kind": "alkalinity"}, origins=at("alk")
    )
    for site in ("X", "Hfo_s"):
        quantity[site] = w.kind(
            "conserved_quantity", {"key": f"{SYSTEM}:{site}", "kind": "site_total"}, origins=at(f"site-{site}")
        )
    ids.update({f"quantity_{name}": value for name, value in quantity.items()})
    system = w.kind("chemical_system", {"key": SYSTEM, "revision": "1"}, origins=at("system"))
    ids["system"] = system
    for value in quantity.values():
        w.relation("system_conserves", {"system": system, "quantity": value}, {}, at="a.json#/conserves")
    forms: dict[str, uuid.UUID] = {}
    for name, (composition, charge, alkalinity, role, member_role, aggregation) in SPECIES.items():
        species = w.kind(
            "species", {"canonical_key": name, "label": name, "charge": charge}, origins=at(f"s-{name}")
        )
        for key, amount in composition.items():
            w.relation("composition", {"entity": species, "quantity": quantity[key]}, {"value": amount}, at="a.json#/c")
        w.relation("composition", {"entity": species, "quantity": quantity["charge"]}, {"value": charge}, at="a.json#/c")
        w.relation("composition", {"entity": species, "quantity": quantity["alk"]}, {"value": alkalinity}, at="a.json#/c")
        forms[name] = w.kind(
            "species_form",
            {
                "canonical_key": f"{name} {aggregation}",
                "label": name,
                "species": species,
                "aggregation": entity_id(decl, "aggregation", aggregation),
            },
            origins=at(f"f-{name}"),
        )
        ids[f"species_{name}"] = species
        ids[f"form_{name}"] = forms[name]
        w.relation(
            "system_member",
            {"system": system, "form": forms[name]},
            {"role": role, "member_role": entity_id(decl, "member_role", member_role)},
            at="a.json#/member",
        )
    states = {
        "solvent": w.kind(
            "standard_state",
            {"key": "solvent", "kind": "pure_real", "pressure_rule": "system_pressure"},
            origins=at("ss-solvent"),
        ),
        "solute": w.kind(
            "standard_state",
            {
                "key": "solute",
                "kind": "infinite_dilution",
                "scale": entity_id(decl, "composition_basis", "molality"),
                "solvent": forms["H2O"],
                "pressure_rule": "system_pressure",
            },
            origins=at("ss-solute"),
        ),
        "exchange_species": w.kind(
            "standard_state",
            {
                "key": "exchange",
                "kind": "site_reference",
                "scale": entity_id(decl, "composition_basis", "equivalent_fraction"),
                "pressure_rule": "system_pressure",
            },
            origins=at("ss-exchange"),
        ),
        "surface_species": w.kind(
            "standard_state",
            {
                "key": "surface",
                "kind": "site_reference",
                "scale": entity_id(decl, "composition_basis", "site_fraction"),
                "pressure_rule": "system_pressure",
            },
            origins=at("ss-surface"),
        ),
    }
    ids["conventions"] = w.kind(
        "convention_set",
        {
            "key": "aqueous",
            "revision": "1",
            "temperature_scale": "its_90",
            "gas_constant": Quantity(R, "J/(mol*K)"),
        },
        origins=at("conventions"),
    )
    for role, state in states.items():
        w.relation(
            "convention_standard_state",
            {"convention_set": ids["conventions"], "member_role": entity_id(decl, "member_role", role)},
            {"value": state},
            at="a.json#/standard-state",
        )
    for label, (equation, participants, defines) in REACTIONS.items():
        reaction = w.kind(
            "reaction",
            {"canonical_key": equation, "extent": "as_written", "equation": equation},
            origins=at(f"reaction-{label}"),
        )
        ids[f"reaction_{label}"] = reaction
        for name, coefficient in participants.items():
            w.relation(
                "reaction_participant",
                {"reaction": reaction, "form": forms[name]},
                {"coefficient": coefficient},
                at="a.json#/participant",
            )
        w.relation(
            "system_reaction",
            {"system": system, "reaction": reaction},
            {"defines": None if defines is None else forms[defines]},
            at="a.json#/system-reaction",
        )
    # the exchanger and the surface: a phase of structure surface with one class of sites each
    for phase, site, members in (
        ("EXCHANGER", "X", ("X-", "NaX", "CaX2")),
        ("HFO", "Hfo_s", ("Hfo_sOH", "Hfo_sOH2+", "Hfo_sO-")),
    ):
        definition = w.kind(
            "phase_definition",
            {
                "system": system,
                "key": phase,
                "aggregation": entity_id(decl, "aggregation", "adsorbed" if phase == "EXCHANGER" else "surface"),
                "structure": "surface",
            },
            origins=at(f"phase-{phase}"),
        )
        ids[f"phase_{phase}"] = definition
        site_class = w.kind(
            "site_class",
            {"phase": definition, "index": 1, "label": site, "ratio_kind": "constant", "ratio": 1.0},
            origins=at(f"site-class-{site}"),
        )
        ids[f"site_class_{site}"] = site_class
        for name in members:
            w.relation(
                "site_occupant",
                {"site_class": site_class, "occupant": ids[f"species_{name}"]},
                {},
                at="a.json#/occupant",
            )


def write_parameters(w: CanonicalWriter, ids: dict[str, uuid.UUID]) -> None:
    p = w.kind(
        "parameterization",
        {
            "key": "aq-fixture",
            "revision": "1",
            "title": "An aqueous database",
            "coherence": "independent_records",
            "convention_set": ids["conventions"],
            "chemical_system": ids["system"],
        },
        origins=at("parameterization"),
    )
    ids["parameterization"] = p
    for label, (a1, a2, a3, a4, a5, a6) in ANALYTIC.items():
        w.parameter_set(
            parameterization=p,
            slot_group="log_k_analytic.reaction",
            subjects=[ids[f"reaction_{label}"]],
            slots={
                "A1": a1,
                "A2": Quantity(a2, "1/K"),
                "A3": Quantity(a3, "K"),
                "A4": a4,
                "A5": Quantity(a5, "K^2"),
                "A6": Quantity(a6, "1/K^2"),
            },
            origins=at(f"log-k-{label}"),
        )
    for label, (log_k_ref, delta_h) in VAN_T_HOFF.items():
        w.parameter_set(
            parameterization=p,
            slot_group="log_k_van_t_hoff.reaction",
            subjects=[ids[f"reaction_{label}"]],
            slots={"log_k_ref": log_k_ref, "delta_h": Quantity(delta_h, "J/mol")},
            origins=at(f"log-k-{label}"),
        )
    for (cation, anion), (beta0, beta1, beta2, alpha1, alpha2, c_phi) in PITZER.items():
        w.parameter_set(
            parameterization=p,
            slot_group="pitzer_ion_pair.pair",
            subjects=[ids[f"species_{cation}"], ids[f"species_{anion}"]],
            slots={
                "beta0": Quantity(beta0, "kg/mol"),
                "beta1": Quantity(beta1, "kg/mol"),
                "beta2": Quantity(beta2, "kg/mol"),
                "alpha1": alpha1,
                "alpha2": alpha2,
                "C_phi": Quantity(c_phi, "kg^2/mol^2"),
            },
            origins=at(f"pitzer-{cation}-{anion}"),
        )
    for (first, second), theta in THETA.items():
        w.parameter_set(
            parameterization=p,
            slot_group="pitzer_theta.pair",
            subjects=[ids[f"species_{first}"], ids[f"species_{second}"]],
            slots={"theta": Quantity(theta, "kg/mol")},
            origins=at(f"theta-{first}-{second}"),
        )
    for (first, second, third), psi in PSI.items():
        w.parameter_set(
            parameterization=p,
            slot_group="pitzer_psi.triplet",
            subjects=[ids[f"species_{first}"], ids[f"species_{second}"], ids[f"species_{third}"]],
            slots={"psi": Quantity(psi, "kg^2/mol^2")},
            origins=at(f"psi-{first}-{second}-{third}"),
        )
    for (first, second), epsilon in SIT.items():
        w.parameter_set(
            parameterization=p,
            slot_group="sit_epsilon.pair",
            subjects=[ids[f"species_{first}"], ids[f"species_{second}"]],
            slots={"epsilon": Quantity(epsilon, "kg/mol")},
            origins=at(f"sit-{first}-{second}"),
        )


ASSEMBLY = {  # the assembly of each species form under the parameterisation: path -> (slot, form)
    "Na+": {
        "eos": ("aqueous_standard_state.eos", "hkf_standard_state"),
        "eos/epsilon": ("hkf_standard_state.epsilon", "solvent_permittivity_correlation"),
        "eos/density": ("hkf_standard_state.density", "solvent_density_correlation"),
    },
    "CO2(aq)": {"eos": ("aqueous_standard_state.eos", "standard_gibbs_constant")},
}


def write_assemblies(w: CanonicalWriter, ids: dict[str, uuid.UUID]) -> None:
    p = ids["parameterization"]
    for name, choices in ASSEMBLY.items():
        assembly = w.kind(
            "model_assembly",
            {"key": f"aqueous-{name}", "revision": "1", "title": name, "root": "aqueous_standard_state"},
            origins=at(f"assembly-{name}"),
        )
        ids[f"assembly_{name}"] = assembly
        for path, (slot, form) in choices.items():
            w.kind(
                "assembly_choice",
                {"assembly": assembly, "path": path, "ordinal": 1, "slot": slot, "form": form},
                at="a.json#/choice",
            )
        w.relation(
            "entity_model",
            {"parameterization": p, "entity": ids[f"form_{name}"]},
            {"value": assembly},
            at="a.json#/entity-model",
        )
    w.parameter_set(
        parameterization=p,
        slot_group="hkf_standard_state.pure",
        subjects=[ids["form_Na+"]],
        slots={
            "g_f": Quantity(HKF["g_f"], "J/mol"),
            "h_f": Quantity(HKF["h_f"], "J/mol"),
            "s": Quantity(HKF["s"], "J/(mol*K)"),
            "a1": Quantity(HKF["a1"], "m^3/mol"),
            "a2": Quantity(HKF["a2"], "J/mol"),
            "a3": Quantity(HKF["a3"], "m^3*K/mol"),
            "a4": Quantity(HKF["a4"], "J*K/mol"),
            "c1": Quantity(HKF["c1"], "J/(mol*K)"),
            "c2": Quantity(HKF["c2"], "J*K/mol"),
            "omega": Quantity(HKF["omega"], "J/mol"),
        },
        origins=at("hkf-Na+"),
    )
    w.parameter_set(
        parameterization=p,
        slot_group="standard_gibbs_constant.pure",
        subjects=[ids["form_CO2(aq)"]],
        slots={"g": Quantity(CONSTANT_G, "J/mol")},
        origins=at("constant-CO2"),
    )


def write_world(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    write_chemistry(w, decl, ids)
    write_parameters(w, ids)
    write_assemblies(w, ids)


@pytest.fixture(scope="module")
def decl() -> Declaration:
    return real_declaration()


@pytest.fixture(scope="module")
def world(decl: Declaration, tmp_path_factory: pytest.TempPathFactory) -> Iterator[World]:
    ids: dict[str, uuid.UUID] = {}
    database = build(tmp_path_factory.mktemp("aqueous"), lambda w: write_world(w, decl, ids), decl)
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


def source(world: World, conn: psycopg.Connection, **subforms: list[SubformBinding]) -> DatabaseSource:
    return DatabaseSource(conn, world.decl, [world.ids["parameterization"]], subforms=subforms or None)


def number(value: object) -> np.ndarray:
    return np.asarray(value, dtype=float).reshape(-1)


# -- the structure ------------------------------------------------------------------------------


def test_the_fixture_satisfies_every_invariant(conn: psycopg.Connection) -> None:
    assert failing(conn) == {}


def test_the_system_has_master_and_secondary_species_and_a_defining_reaction_for_each_secondary(
    world: World, conn: psycopg.Connection
) -> None:
    roles = dict(
        conn.execute(
            "SELECT e.label, m.role::text FROM tk.system_member m JOIN tk.material_entity e ON e.id = m.form"
        ).fetchall()
    )
    assert {name for name, role in roles.items() if role == "basis"} == {
        name for name, spec in SPECIES.items() if spec[3] == "basis"
    }
    defined = {
        label: name
        for label, name in conn.execute(
            "SELECT e.label, r.equation FROM tk.system_reaction s JOIN tk.material_entity e ON e.id = s.defines "
            "JOIN tk.reaction r ON r.id = s.reaction"
        ).fetchall()
    }
    assert set(defined) == {name for name, role in roles.items() if role == "secondary"}


def test_every_reaction_conserves_elements_charge_alkalinity_and_sites(
    world: World, conn: psycopg.Connection
) -> None:
    """The balance `tk verify` checks, written out: for each reaction and each quantity of the
    system the coefficient-weighted compositions sum to zero, and the three kinds of quantity
    that are not elements are among those counted."""
    nets = conn.execute(
        "SELECT q.key, r.canonical_key, sum(p.coefficient * c.value) "
        "FROM tk.reaction r JOIN tk.reaction_participant p ON p.reaction = r.id "
        "JOIN tk.species_form f ON f.id = p.form JOIN tk.composition c ON c.entity = f.species "
        "JOIN tk.conserved_quantity q ON q.id = c.quantity GROUP BY q.key, r.canonical_key"
    ).fetchall()
    assert nets and all(abs(net) < 1e-12 for _, _, net in nets)
    counted = {key for key, _, _ in nets}
    assert {"charge", f"{SYSTEM}:Alk", f"{SYSTEM}:X", f"{SYSTEM}:Hfo_s"} <= counted
    kinds = dict(conn.execute("SELECT key, kind::text FROM tk.conserved_quantity").fetchall())
    assert kinds[f"{SYSTEM}:Alk"] == "alkalinity"
    assert kinds[f"{SYSTEM}:X"] == kinds[f"{SYSTEM}:Hfo_s"] == "site_total"
    assert kinds["charge"] == "charge"


def test_the_exchange_and_surface_species_occupy_the_site_classes_of_their_phases(
    world: World, conn: psycopg.Connection
) -> None:
    occupants = {
        (phase, label): set(names)
        for phase, label, names in conn.execute(
            "SELECT p.key, s.label, array_agg(e.canonical_key) FROM tk.site_occupant o "
            "JOIN tk.site_class s ON s.id = o.site_class JOIN tk.phase_definition p ON p.id = s.phase "
            "JOIN tk.material_entity e ON e.id = o.occupant GROUP BY p.key, s.label"
        ).fetchall()
    }
    assert occupants == {
        ("EXCHANGER", "X"): {"X-", "NaX", "CaX2"},
        ("HFO", "Hfo_s"): {"Hfo_sOH", "Hfo_sOH2+", "Hfo_sO-"},
    }
    sites = conn.execute(
        "SELECT c.key, array_agg(e.canonical_key ORDER BY e.canonical_key) FROM tk.composition k "
        "JOIN tk.conserved_quantity c ON c.id = k.quantity JOIN tk.material_entity e ON e.id = k.entity "
        "WHERE c.kind = 'site_total' AND k.value > 0 GROUP BY c.key"
    ).fetchall()
    assert dict(sites) == {
        f"{SYSTEM}:X": ["CaX2", "NaX", "X-"],
        f"{SYSTEM}:Hfo_s": ["Hfo_sO-", "Hfo_sOH", "Hfo_sOH2+"],
    }


def test_a_participant_takes_the_standard_state_of_its_member_role(
    world: World, conn: psycopg.Connection
) -> None:
    kinds = dict(
        conn.execute(
            "SELECT m.name, s.kind::text FROM tk.convention_standard_state c "
            "JOIN tk.standard_state s ON s.id = c.value JOIN tk.member_role m ON m.id = c.member_role"
        ).fetchall()
    )
    assert kinds == {
        "solvent": "pure_real",
        "solute": "infinite_dilution",
        "exchange_species": "site_reference",
        "surface_species": "site_reference",
    }
    assert scalar(conn, "SELECT count(standard_state) FROM tk.reaction_participant") == 0  # none states its own


# -- log K and the interaction terms, against numpy -----------------------------------------------

TEMPERATURES = np.array([273.15, 298.15, 323.15, 373.15, 473.15, 573.15])


@pytest.mark.parametrize("label", sorted(ANALYTIC))
def test_the_analytic_log_k_matches_the_independent_calculation(
    world: World, conn: psycopg.Connection, label: str
) -> None:
    found = bind(
        world.decl,
        "log_k_analytic",
        source=source(world, conn),
        roles={"r": str(world.ids[f"reaction_{label}"])},
        cache=CACHE,
    )
    got = number(found.evaluate("log_k", T=TEMPERATURES))
    np.testing.assert_allclose(got, log_k_analytic(ANALYTIC[label], TEMPERATURES), rtol=1e-12)


@pytest.mark.parametrize("label", sorted(VAN_T_HOFF))
def test_the_van_t_hoff_log_k_matches_the_independent_calculation_with_the_convention_sets_gas_constant(
    world: World, conn: psycopg.Connection, label: str
) -> None:
    found = bind(
        world.decl,
        "log_k_van_t_hoff",
        source=source(world, conn),
        roles={"r": str(world.ids[f"reaction_{label}"])},
        cache=CACHE,
    )
    log_k_ref, delta_h = VAN_T_HOFF[label]
    got = number(found.evaluate("log_k", T=TEMPERATURES))
    np.testing.assert_allclose(got, log_k_van_t_hoff(log_k_ref, delta_h, TEMPERATURES), rtol=1e-12, atol=1e-13)
    assert got[1] == pytest.approx(log_k_ref, abs=1e-12)  # the constant at 298.15 K is the stored one


def test_a_reaction_written_on_another_basis_is_another_reaction_with_its_own_constant(
    world: World, conn: psycopg.Connection
) -> None:
    """Carbon dioxide is formed by two reactions, one written from the carbonate master species
    and one from bicarbonate: two reactions, two stored constants. They are consistent (the
    first is the second plus the constant of the bicarbonate reaction), but nothing re-bases one
    on the other: each constant is asked of the reaction it was written for."""
    both = conn.execute(
        "SELECT r.canonical_key FROM tk.reaction r JOIN tk.reaction_participant p ON p.reaction = r.id "
        "JOIN tk.species_form f ON f.id = p.form JOIN tk.material_entity e ON e.id = f.species "
        "WHERE e.canonical_key = 'CO2(aq)' ORDER BY r.canonical_key"
    ).fetchall()
    assert [key for (key,) in both] == ["CO3-2 + 2H+ = CO2 + H2O", "HCO3- + H+ = CO2 + H2O"]
    at_25 = np.array([298.15])

    def log_k(form: str, label: str) -> float:
        found = bind(
            world.decl,
            form,
            source=source(world, conn),
            roles={"r": str(world.ids[f"reaction_{label}"])},
            cache=CACHE,
        )
        return float(number(found.evaluate("log_k", T=at_25))[0])

    from_carbonate = log_k("log_k_analytic", "carbon dioxide")
    from_bicarbonate = log_k("log_k_van_t_hoff", "carbon dioxide from bicarbonate")
    assert from_carbonate != pytest.approx(from_bicarbonate, abs=0.1), "two constants, not one"
    assert from_carbonate == pytest.approx(log_k("log_k_analytic", "bicarbonate") + from_bicarbonate, abs=0.001)
    unwritten = bind(world.decl, "log_k_analytic", source=source(world, conn), roles={"r": str(uuid.uuid4())})
    with pytest.raises(EvaluationRefusal, match="log_k_analytic.reaction"):
        unwritten.evaluate("log_k", T=at_25)


@pytest.mark.parametrize("pair", sorted(PITZER))
def test_the_pitzer_osmotic_coefficients_match_the_independent_calculation(
    world: World, conn: psycopg.Connection, pair: tuple[str, str]
) -> None:
    cation, anion = pair
    found = bind(
        world.decl,
        "pitzer_ion_pair",
        source=source(world, conn),
        roles={"cation": str(world.ids[f"species_{cation}"]), "anion": str(world.ids[f"species_{anion}"])},
        cache=CACHE,
    )
    I = np.array([0.01, 0.1, 0.5, 1.0, 3.0, 6.0])
    np.testing.assert_allclose(number(found.evaluate("B_phi", I=I)), b_phi(PITZER[pair], I), rtol=1e-12)
    np.testing.assert_allclose(number(found.evaluate("C_phi", I=I)), PITZER[pair][5], rtol=1e-12)


def test_the_third_beta_changes_the_result_for_the_pair_of_ions_of_high_charge() -> None:
    I = np.array([0.05])
    with_beta2 = b_phi(PITZER[("Ca+2", "CO3-2")], I)
    without = b_phi((*PITZER[("Ca+2", "CO3-2")][:2], 0.0, *PITZER[("Ca+2", "CO3-2")][3:]), I)
    assert abs(with_beta2[0] - without[0]) > 1.0


def test_theta_and_psi_are_the_same_whichever_of_the_like_ions_is_named_first(
    world: World, conn: psycopg.Connection
) -> None:
    def evaluate(form: str, output: str, **roles: str) -> float:
        found = bind(world.decl, form, source=source(world, conn), roles=roles, cache=CACHE)
        return float(number(found.evaluate(output))[0])

    sodium, calcium, chloride = (str(world.ids[f"species_{n}"]) for n in ("Na+", "Ca+2", "Cl-"))
    for i, j in ((sodium, calcium), (calcium, sodium)):
        assert evaluate("pitzer_theta", "theta", i=i, j=j) == pytest.approx(THETA[("Na+", "Ca+2")], rel=1e-13)
        assert evaluate("pitzer_psi", "psi", i=i, j=j, k=chloride) == pytest.approx(
            PSI[("Na+", "Ca+2", "Cl-")], rel=1e-13
        )


def test_the_specific_interaction_term_matches_the_independent_calculation(
    world: World, conn: psycopg.Connection
) -> None:
    found = bind(
        world.decl,
        "sit_epsilon",
        source=source(world, conn),
        roles={"i": str(world.ids["species_Na+"]), "j": str(world.ids["species_Cl-"])},
        cache=CACHE,
    )
    m = np.array([0.05, 0.5, 2.0])
    np.testing.assert_allclose(number(found.evaluate("log10_gamma_term", m=m)), SIT[("Na+", "Cl-")] * m, rtol=1e-13)
    swapped = bind(
        world.decl,
        "sit_epsilon",
        source=source(world, conn),
        roles={"i": str(world.ids["species_Cl-"]), "j": str(world.ids["species_Na+"])},
    )
    np.testing.assert_allclose(number(swapped.evaluate("log10_gamma_term", m=m)), SIT[("Na+", "Cl-")] * m, rtol=1e-13)


# -- the standard state of a species, assembled for the species ----------------------------------


def assembly_bindings(world: World, conn: psycopg.Connection, name: str) -> dict[str, list[SubformBinding]]:
    rows = conn.execute(
        "SELECT sl.qualified_name, f.name FROM tk.assembly_choice c "
        "JOIN meta.subform_slot sl ON sl.id = c.slot JOIN meta.form f ON f.id = c.form "
        "WHERE c.assembly = %s ORDER BY c.path, c.ordinal",
        (scalar(
            conn,
            "SELECT value FROM tk.entity_model WHERE parameterization = %s AND entity = %s",
            world.ids["parameterization"],
            world.ids[f"form_{name}"],
        ),),
    ).fetchall()
    found: dict[str, list[SubformBinding]] = {}
    for slot, form in rows:
        found.setdefault(slot, []).append(SubformBinding(form, (world.ids["parameterization"],)))
    return found


def test_two_species_of_one_parameterisation_have_their_own_assemblies(
    world: World, conn: psycopg.Connection
) -> None:
    counts = dict(
        conn.execute(
            "SELECT e.value, count(*) FROM tk.entity_model e JOIN tk.assembly_choice c ON c.assembly = e.value "
            "WHERE e.parameterization = %s GROUP BY e.value",
            (world.ids["parameterization"],),
        ).fetchall()
    )
    assert counts == {world.ids["assembly_Na+"]: 3, world.ids["assembly_CO2(aq)"]: 1}
    paths = [
        row
        for row in conn.execute(
            "SELECT c.path, f.name FROM tk.assembly_choice c JOIN meta.form f ON f.id = c.form "
            "WHERE c.assembly = %s ORDER BY c.path",
            (world.ids["assembly_Na+"],),
        )
    ]
    assert paths == [
        ("eos", "hkf_standard_state"),
        ("eos/density", "solvent_density_correlation"),
        ("eos/epsilon", "solvent_permittivity_correlation"),
    ]


def test_the_species_of_constant_energy_evaluates_through_its_assembly(
    world: World, conn: psycopg.Connection
) -> None:
    found = bind(
        world.decl,
        "aqueous_standard_state",
        source=source(world, conn, **assembly_bindings(world, conn, "CO2(aq)")),
        roles={"i": str(world.ids["form_CO2(aq)"])},
        cache=CACHE,
    )
    got = number(found.evaluate("G0", T=np.array([298.15, 350.0]), P=np.array([1e5, 2e7])))
    np.testing.assert_allclose(got, CONSTANT_G, rtol=1e-14)


def test_the_hkf_ion_is_held_with_its_parameters_but_its_equations_live_in_code(
    world: World, conn: psycopg.Connection
) -> None:
    row = conn.execute('SELECT "g_f", "omega", "a1", "c2" FROM param."hkf_standard_state__pure"').fetchone()
    assert row == (
        pytest.approx(HKF["g_f"]), pytest.approx(HKF["omega"]), pytest.approx(HKF["a1"]), pytest.approx(HKF["c2"])
    )
    found = bind(
        world.decl,
        "aqueous_standard_state",
        source=source(world, conn, **assembly_bindings(world, conn, "Na+")),
        roles={"i": str(world.ids["form_Na+"])},
    )
    with pytest.raises(EvaluationRefusal, match=r"hkf_standard_state.*catalogued"):
        found.evaluate("G0", T=np.array([298.15]), P=np.array([1e5]))


def test_the_assembly_of_one_species_is_not_applied_to_another(world: World, conn: psycopg.Connection) -> None:
    found = bind(
        world.decl,
        "aqueous_standard_state",
        source=source(world, conn, **assembly_bindings(world, conn, "CO2(aq)")),
        roles={"i": str(world.ids["form_Na+"])},
    )
    with pytest.raises(EvaluationRefusal, match=r"standard_gibbs_constant.pure"):
        found.evaluate("G0", T=np.array([298.15]), P=np.array([1e5]))


# -- what the model refuses ---------------------------------------------------------------------


def test_the_checks_of_verify_flag_an_unbalanced_reaction_and_a_participant_with_no_standard_state(
    decl: Declaration, tmp_path: Path
) -> None:
    """Three violations: an exchange reaction whose product lacks the site its reactant carries
    (the site total does not balance, and the reaction belongs to a system that conserves it), the
    forms of a reaction of an equilibrium-constant set that have no role in the system, so no
    standard state under the convention set, and a chosen equation of state whose solvent slots
    are not decided."""

    def emit(w: CanonicalWriter) -> None:
        ids: dict[str, uuid.UUID] = {}
        write_chemistry(w, decl, ids)
        write_parameters(w, ids)

        def form(name: str, aggregation: str, composition: dict[str, int], charge: int = 0) -> uuid.UUID:
            species = ids.get(f"species_{name}") or w.kind(
                "species", {"canonical_key": name, "label": name, "charge": charge}, origins=at(f"s-{name}")
            )
            ids[f"species_{name}"] = species
            for key, amount in composition.items():
                w.relation(
                    "composition", {"entity": species, "quantity": ids[f"quantity_{key}"]}, {"value": amount}, at="a.json#/c"
                )
            return w.kind(
                "species_form",
                {
                    "canonical_key": f"{name} {aggregation}",
                    "label": name,
                    "species": species,
                    "aggregation": entity_id(decl, "aggregation", aggregation),
                },
                origins=at(f"f-{name}-{aggregation}"),
            )

        def reaction(key: str, participants: dict[uuid.UUID, int]) -> uuid.UUID:
            made = w.kind("reaction", {"canonical_key": key, "extent": "as_written"}, origins=at(f"reaction-{key}"))
            for participant, coefficient in participants.items():
                w.relation(
                    "reaction_participant",
                    {"reaction": made, "form": participant},
                    {"coefficient": coefficient},
                    at="a.json#/p",
                )
            return made

        sodium_no_site = form("NaX-without-site", "adsorbed", {"Na": 1})
        unbalanced = reaction(
            "Na+ + X- = NaX-without-site",
            {ids["form_Na+"]: -1, ids["form_X-"]: -1, sodium_no_site: 1},
        )
        w.relation("system_reaction", {"system": ids["system"], "reaction": unbalanced}, {}, at="a.json#/system-reaction")
        outside = reaction(
            "Mg+2 (liquid) = Mg+2 (adsorbed)",
            {form("Mg+2", "liquid", {"charge": 2}, 2): -1, form("Mg+2", "adsorbed", {}): 1},
        )
        w.parameter_set(
            parameterization=ids["parameterization"],
            slot_group="log_k_van_t_hoff.reaction",
            subjects=[outside],
            slots={"log_k_ref": 0.0, "delta_h": Quantity(0.0, "J/mol")},
            origins=at("log-k-outside"),
        )
        half = w.kind(
            "model_assembly",
            {"key": "half", "revision": "1", "title": "half", "root": "aqueous_standard_state"},
            origins=at("half-assembly"),
        )
        w.kind(
            "assembly_choice",
            {"assembly": half, "path": "eos", "ordinal": 1, "slot": "aqueous_standard_state.eos", "form": "hkf_standard_state"},
            at="a.json#/choice",
        )

    database = build(tmp_path, emit, decl)
    try:
        with psycopg.connect(database.url) as connection:
            assert failing(connection) == {
                "reaction.conserves_declared_quantities": 1,  # the site total X
                "system_reaction.conserves_system_quantities": 1,
                "reaction.standard_states_for_equilibrium_constant": 2,  # both forms of the outside reaction
                "model_assembly.single_slots_decided": 2,  # the solvent permittivity and density
            }
    finally:
        database.remove()


def test_a_pitzer_parameter_cannot_be_written_for_an_ion_with_itself(decl: Declaration) -> None:
    w = writer(decl)
    ids: dict[str, uuid.UUID] = {}
    write_chemistry(w, decl, ids)
    p = w.kind(
        "parameterization",
        {"key": "p", "revision": "1", "title": "p", "coherence": "independent_records"},
        origins=at("p"),
    )
    with pytest.raises(ValidationError, match="diagonal"):
        w.parameter_set(
            parameterization=p,
            slot_group="pitzer_theta.pair",
            subjects=[ids["species_Na+"], ids["species_Na+"]],
            slots={"theta": Quantity(0.07, "kg/mol")},
            origins=at("theta-diagonal"),
        )


def test_a_pitzer_coefficient_needs_the_unit_of_the_model(decl: Declaration) -> None:
    w = writer(decl)
    ids: dict[str, uuid.UUID] = {}
    write_chemistry(w, decl, ids)
    p = w.kind(
        "parameterization",
        {"key": "p", "revision": "1", "title": "p", "coherence": "independent_records"},
        origins=at("p"),
    )
    with pytest.raises(ValidationError, match="beta0"):
        w.parameter_set(
            parameterization=p,
            slot_group="pitzer_ion_pair.pair",
            subjects=[ids["species_Na+"], ids["species_Cl-"]],
            slots={
                "beta0": Quantity(0.081, "J/mol"),  # not kg/mol
                "beta1": Quantity(0.33, "kg/mol"),
                "beta2": Quantity(0.0, "kg/mol"),
                "alpha1": 2.0,
                "alpha2": 12.0,
                "C_phi": Quantity(0.0, "kg^2/mol^2"),
            },
            origins=at("beta-wrong-unit"),
        )
