# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The declaration additions of plan 24, packet TK2f: what each construct holds, shown by a fixture
written through the canonical writer, loaded into a database and checked by `tk verify`, and by
the refusals of the invariants that state it.

The fixtures use the committed model and forms plus `fixtures/additions`: a few forms that use the
new constructs (an enum-typed slot of the COSMO-SAC dispersion class, a Flory-Huggins-style form
over abstract pseudo-components, a form that reads the Boltzmann and Avogadro constants) and the
vocabulary rows a mapping would declare (methods, observables that use the new members). The
violations of the invariants that the verify stage states are in `test_verify.py`, beside the
other checks; this file holds the conforming fixtures, the database constraints the writer
enforces, and the behaviour of the convention facts.
"""

from __future__ import annotations

import shutil
import uuid
from collections.abc import Iterator
from dataclasses import dataclass
from pathlib import Path

import numpy as np
import psycopg
import pytest

from build_support import fingerprint, inputs_of, write_source
from mapping_support import carrier, origin, writer
from thermo_knowledge import config, db, identity
from thermo_knowledge.build import build_database
from thermo_knowledge.canonical.provenance import SourceRef
from thermo_knowledge.canonical.values import Quantity
from thermo_knowledge.canonical.writer import CanonicalWriter, ValidationError
from thermo_knowledge.declaration import Declaration, load_declaration
from thermo_knowledge.declaration.types import registry
from thermo_knowledge.expression.evaluate import EvaluationRefusal, bind
from thermo_knowledge.qualify.source import DatabaseSource
from thermo_knowledge.testing import TestDatabase
from thermo_knowledge.verify.checks import VERIFY_DIR, load_checks
from thermo_knowledge.verify.run import run_check, run_checks

FIXTURES = Path(__file__).parent / "fixtures" / "additions"
PUBLISHED = origin("a.json#/0", "published")
CARRIER = identity.identifier("source", [carrier("src", "a.json").key])
CHECKS = {c.target: c for c in load_checks(config.TREE_DIR / VERIFY_DIR)[0]}


def additions_declaration(destination: Path) -> Declaration:
    """The committed declaration plus the fixture forms and vocabulary."""
    shutil.copytree(config.TREE_DIR / "model", destination / "model")
    shutil.copytree(config.TREE_DIR / "forms", destination / "forms")
    for form in (FIXTURES / "forms").glob("*.toml"):
        shutil.copy(form, destination / "forms" / form.name)
    for module in (FIXTURES / "model").glob("*.toml"):
        shutil.copy(module, destination / "model" / module.name)
    return load_declaration(destination / "model", destination / "forms").require()


def entity_id(decl: Declaration, kind: str, name: str) -> uuid.UUID:
    return next(e.id for e in decl.entities if e.kind == kind and e.name == name)


@dataclass
class World:
    decl: Declaration
    database: TestDatabase
    ids: dict[str, uuid.UUID]


def at(tag: str) -> list:  # noqa: ANN401
    """The origin of a record: a published row of `a.json`."""
    return [origin(f"a.json#/{tag}", "published")]


# -- the fixtures -------------------------------------------------------------------------------


def write_isotopes(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    """Heavy water: deuterium an isotope of hydrogen, and the exchange reaction of H2O and D2O."""
    hydrogen, oxygen, nitrogen = (entity_id(decl, "element", s) for s in ("H", "O", "N"))
    gas = entity_id(decl, "aggregation", "gas")
    deuterium = w.kind(
        "isotope",
        {
            "key": "2H",
            "kind": "isotope",
            "of_element": hydrogen,
            "mass_number": 2,
            "atomic_mass": Quantity(2.01410178, "g/mol"),
            "symbol": "D",
        },
        origins=at("isotope-d"),
    )
    tritium = w.kind(
        "isotope",
        {"key": "3H", "kind": "isotope", "of_element": hydrogen, "mass_number": 3, "symbol": "T"},
        origins=at("isotope-t"),
    )
    w.kind("conserved_quantity", {"key": "alk", "kind": "alkalinity"}, origins=at("alk"))
    w.kind(
        "conserved_quantity",
        {"key": "Nit", "kind": "decoupled_inventory", "of_element": nitrogen},
        origins=at("nit"),
    )
    w.kind("conserved_quantity", {"key": "LIGAND", "kind": "moiety"}, origins=at("ligand"))
    ids.update(deuterium=deuterium, tritium=tritium, hydrogen=hydrogen, oxygen=oxygen)
    forms: dict[str, uuid.UUID] = {}
    for name, composition in {
        "H2O": {hydrogen: 2, oxygen: 1},
        "D2O": {deuterium: 2, oxygen: 1},
        "HDO": {hydrogen: 1, deuterium: 1, oxygen: 1},
    }.items():
        species = w.kind("species", {"canonical_key": name, "label": name}, origins=at(name))
        forms[name] = w.kind(
            "species_form",
            {"canonical_key": f"{name} gas", "label": name, "species": species, "aggregation": gas},
            origins=at(name),
        )
        for quantity, amount in composition.items():
            w.relation(
                "composition",
                {"entity": species, "quantity": quantity},
                {"value": amount},
                at="a.json#/c",
            )
    exchange = w.kind(
        "reaction",
        {"canonical_key": "H2O + D2O = 2 HDO", "extent": "as_written"},
        origins=at("exchange"),
    )
    for name, coefficient in (("D2O", -1), ("H2O", -1), ("HDO", 2)):
        w.relation(
            "reaction_participant",
            {"reaction": exchange, "form": forms[name]},
            {"coefficient": coefficient},
            at="a.json#/p",
        )
    system = w.kind(
        "chemical_system", {"key": "heavy_water", "revision": "1"}, origins=at("heavy-water")
    )
    w.relation("system_conserves", {"system": system, "quantity": hydrogen}, at="a.json#/s")
    w.relation("system_conserves", {"system": system, "quantity": deuterium}, at="a.json#/s")
    w.relation("system_reaction", {"system": system, "reaction": exchange}, at="a.json#/s")


def write_conventions(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    """Temperature scales, energy datums that state values and the constants of convention sets."""
    liquid = entity_id(decl, "aggregation", "liquid")
    iir = w.kind(
        "energy_reference",
        {
            "key": "iir",
            "enthalpy": "at_state",
            "entropy": "at_state",
            "temperature": Quantity(273.15, "K"),
            "state": "saturated liquid",
            "aggregation": liquid,
            "datum_energy": "enthalpy",
            "specific_energy_value": Quantity(200.0, "kJ/kg"),
            "specific_entropy_value": Quantity(1.0, "kJ/(kg*K)"),
        },
        origins=at("iir"),
    )
    iapws95 = w.kind(
        "energy_reference",
        {
            "key": "iapws95",
            "enthalpy": "at_state",
            "entropy": "at_state",
            "temperature": Quantity(273.16, "K"),
            "pressure": Quantity(611.654771, "Pa"),
            "state": "liquid at the triple point",
            "aggregation": liquid,
            "datum_energy": "internal_energy",
            "energy_value": Quantity(0.0, "J/mol"),
            "entropy_value": Quantity(0.0, "J/(mol*K)"),
        },
        origins=at("iapws95"),
    )
    ids.update(iir=iir, iapws95=iapws95)
    for key, scale, reference in (
        ("ept", "ept_76", None),
        ("its27", "its_27", None),
        ("iir", "its_90", iir),
        ("iapws95", "its_90", iapws95),
    ):
        w.kind(
            "convention_set",
            {
                "key": key,
                "revision": "1",
                "temperature_scale": scale,
                "energy_reference": reference,
            },
            origins=at(f"conventions-{key}"),
        )
    # standard states that state no pressure and no scale, and a transformed one (items 32 and 33)
    w.kind(
        "standard_state",
        {"key": "unstated", "kind": "not_stated", "pressure_rule": "not_stated"},
        origins=at("ss-unstated"),
    )
    w.kind(
        "standard_state",
        {
            "key": "transformed",
            "kind": "transformed_biochemical",
            "pressure_rule": "fixed",
            "pressure": Quantity(1.0, "bar"),
        },
        origins=at("ss-transformed"),
    )
    water = w.kind(
        "species", {"canonical_key": "solvent water", "label": "water"}, origins=at("solvent")
    )
    w.kind(
        "standard_state",
        {
            "key": "dilute_without_scale",
            "kind": "infinite_dilution",
            "scale": entity_id(decl, "composition_basis", "not_stated"),
            "solvent": water,
            "pressure_rule": "not_stated",
        },
        origins=at("ss-dilute"),
    )
    # the gas constant, Boltzmann and Avogadro constants of the convention sets the form reads (item 28)
    boltzmann, avogadro, gas = 1.380649e-23, 6.02214076e23, 8.314462618
    for key, facts in {
        "kb_both": {
            "boltzmann_constant": Quantity(boltzmann, "J/K"),
            "avogadro_constant": Quantity(avogadro, "1/mol"),
        },
        "kb_same": {
            "boltzmann_constant": Quantity(boltzmann, "J/K"),
            "avogadro_constant": Quantity(avogadro, "1/mol"),
        },
        "kb_other_avogadro": {
            "boltzmann_constant": Quantity(boltzmann, "J/K"),
            "avogadro_constant": Quantity(6.0221e23, "1/mol"),
        },
        "kb_only_boltzmann": {"boltzmann_constant": Quantity(boltzmann, "J/K")},
        "kb_only_avogadro": {"avogadro_constant": Quantity(avogadro, "1/mol")},
        "kb_with_gas": {"gas_constant": Quantity(gas, "J/(mol*K)")},
    }.items():
        ids[f"conv_{key}"] = w.kind(
            "convention_set",
            {"key": key, "revision": "1", "temperature_scale": "its_90", **facts},
            origins=at(f"conv-{key}"),
        )


def write_groups(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    """A scheme that repeats a label under two codes, two decompositions by one carrier and the
    bonds between groups of one decomposition."""
    scheme = w.kind(
        "group_scheme",
        {"key": "unifac-dortmund", "revision": "1", "role": "activity"},
        origins=at("scheme"),
    )
    groups = {
        code: w.kind(
            "group",
            {"scheme": scheme, "code": code, "label": label, "role": "group"},
            origins=at(f"group-{code}"),
        )
        for code, label in (("9", "CHO"), ("10", "CHO"), ("1", "CH3"))
    }
    species = w.kind(
        "species", {"canonical_key": "propanal", "label": "propanal"}, origins=at("propanal")
    )
    ids.update(groups_scheme=scheme, groups_species=species)
    assignments = {}
    for occurrence in (1, 2):
        assignments[occurrence] = w.kind(
            "group_assignment",
            {
                "entity": species,
                "scheme": scheme,
                "asserted_by": CARRIER,
                "origin": "published",
                **({"occurrence": occurrence} if occurrence > 1 else {}),
            },
            origins=at(f"assignment-{occurrence}"),
        )
    first = assignments[1]
    for code, count in (("9", 1), ("1", 2)):
        w.relation(
            "group_count",
            {"assignment": first, "group": groups[code]},
            {"value": count},
            at="a.json#/n",
        )
    w.relation(
        "group_bond_count",
        {"assignment": first, "first": groups["9"], "second": groups["1"]},
        {"value": 1},
        at="a.json#/b",
    )
    w.relation(  # the two methyl groups are bonded to no one here, but a group may bond to its kind
        "group_bond_count",
        {"assignment": first, "first": groups["1"], "second": groups["1"]},
        {"value": 0},
        at="a.json#/b",
    )
    ids.update(assignment_first=assignments[1], assignment_second=assignments[2])


def write_phases(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    """An ordered phase and its disordered partner, and evidence reported per site class."""
    crystalline = entity_id(decl, "aggregation", "crystalline")
    system = w.kind("chemical_system", {"key": "fe-al", "revision": "1"}, origins=at("fe-al"))
    phases = {
        key: w.kind(
            "phase_definition",
            {"system": system, "key": key, "aggregation": crystalline, "structure": "sublattice"},
            origins=at(f"phase-{key}"),
        )
        for key in ("BCC_A2", "BCC_B2")
    }
    w.relation(
        "disordered_partner",
        {"ordered": phases["BCC_B2"], "disordered": phases["BCC_A2"]},
        {"never_disorder": False},
        at="a.json#/partner",
    )
    site_class = w.kind(
        "site_class",
        {"phase": phases["BCC_B2"], "index": 1, "ratio_kind": "constant", "ratio": 0.5},
        origins=at("sublattice"),
    )
    dataset = w.kind(
        "dataset",
        {"carrier": CARRIER, "local_key": "site-fractions", "kind": "measured"},
        origins=[origin("a.json#/site-fractions", "measured")],
    )
    phase = w.kind(
        "dataset_phase",
        {
            "dataset": dataset,
            "ordinal": 1,
            "aggregation": crystalline,
            "phase_definition": phases["BCC_B2"],
        },
        at="a.json#/site-fractions",
    )
    column = w.kind(
        "dataset_column",
        {
            "dataset": dataset,
            "ordinal": 1,
            "role": "property",
            "observable": decl.observable_entity("molar_density").id,  # type: ignore[union-attr]
            "phase": phase,
            "site_class": site_class,
        },
        at="a.json#/site-fractions",
    )
    ids.update(
        phase_ordered=phases["BCC_B2"], phase_disordered=phases["BCC_A2"], column_site=column
    )


def write_accuracy(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    """Accuracy statements: a relative bound of IAPWS-95 over a region of temperature and pressure,
    an IF97 consistency bound against IAPWS-95, a statement at one condition, a multiplicative
    factor and a magnitude whose meaning is not stated."""
    temperature = decl.observable_entity("temperature").id  # type: ignore[union-attr]
    pressure = decl.observable_entity("pressure").id  # type: ignore[union-attr]
    density = decl.observable_entity("molar_density").id  # type: ignore[union-attr]
    constant = decl.observable_entity("log10_equilibrium_constant").id  # type: ignore[union-attr]
    iapws95, if97 = (
        w.kind(
            "parameterization",
            {"key": key, "revision": "1", "title": key, "coherence": "independent_records"},
            origins=at(key),
        )
        for key in ("iapws95_formulation", "if97_formulation")
    )
    region = w.validity_region(
        iapws95,
        {"kind": "validated_range"},
        [
            {
                "observable": temperature,
                "lower": Quantity(273.16, "K"),
                "upper": Quantity(350.0, "K"),
            },
            {
                "observable": pressure,
                "lower": Quantity(0.1, "MPa"),
                "upper": Quantity(100.0, "MPa"),
            },
        ],
        origins=at("iapws95-region"),
    )
    reference = w.validity_region(
        iapws95,
        {"kind": "validated_range"},
        [
            {
                "observable": temperature,
                "lower": Quantity(298.15, "K"),
                "upper": Quantity(298.15, "K"),
            }
        ],
        origins=at("iapws95-reference"),
        ordinal=2,
    )
    statements = {
        "density": w.relation(
            "accuracy_statement",
            {"record": iapws95, "observable": density, "ordinal": 1},
            {
                "region": region,
                "kind": "relative",
                "statistic": "bound",
                "relative_magnitude": 1e-5,
                "sample_size": 120,
            },
            at="a.json#/accuracy-1",
        ),
        "consistency": w.relation(
            "accuracy_statement",
            {"record": if97, "observable": temperature, "ordinal": 1},
            {
                "against": iapws95,
                "kind": "interval",
                "statistic": "bound",
                "magnitude": Quantity(25.0, "mK"),
            },
            at="a.json#/accuracy-2",
        ),
        "at_a_condition": w.relation(
            "accuracy_statement",
            {"record": iapws95, "observable": constant, "ordinal": 1},
            {
                "region": reference,
                "kind": "standard",
                "statistic": "standard_deviation",
                "magnitude": 0.3,
            },
            at="a.json#/accuracy-3",
        ),
        "factor": w.relation(
            "accuracy_statement",
            {"record": iapws95, "observable": constant, "ordinal": 2},
            {"kind": "multiplicative_factor", "statistic": "bound", "relative_magnitude": 2.0},
            at="a.json#/accuracy-4",
        ),
        "unspecified": w.relation(
            "accuracy_statement",
            {"record": if97, "observable": density, "ordinal": 2},
            {"kind": "unspecified", "statistic": "maximum", "magnitude": Quantity(4.0, "mol/m^3")},
            at="a.json#/accuracy-5",
        ),
    }
    ids.update({f"accuracy_{key}": value for key, value in statements.items()})
    ids.update(iapws95=iapws95, if97=if97)


def write_levels(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    """A composite level of theory and a computation that names it."""
    release = w.kind(
        "software_release",
        {"key": "release:molpro:2022.1", "title": "Molpro", "version": "2022.1"},
    )
    frequency = w.kind(
        "level_of_theory",
        {"key": "b3lyp/6-31g(d)", "method": "B3LYP", "basis": "6-31G(d)", "software": release},
        origins=at("level-frequency"),
    )
    energy = w.kind(
        "level_of_theory",
        {
            "key": "ccsd(t)-f12/cc-pvtz-f12/ri",
            "method": "CCSD(T)-F12",
            "basis": "cc-pVTZ-F12",
            "auxiliary_basis": "cc-pVTZ-F12/JKFIT",
            "cabs": "cc-pVTZ-F12/OPTRI",
            "solvent": "water",
            "solvation_method": "COSMO",
            "arguments": "memory=2 GB",
            "software": release,
        },
        origins=at("level-energy"),
    )
    composite = w.kind(
        "level_of_theory",
        {
            "key": "composite:b3lyp/6-31g(d)+ccsd(t)-f12/cc-pvtz-f12/ri",
            "method": "composite",
            "frequency_level": frequency,
            "energy_level": energy,
        },
        origins=at("level-composite"),
    )
    derivation = w.kind(
        "derivation",
        {"key": "thermo-by-arkane", "kind": "computation", "level_of_theory": composite},
        origins=at("derivation"),
    )
    ids.update(level_composite=composite, level_energy=energy, derivation_computed=derivation)


def write_samples(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    """A sample with two purity statements and an impurity content, and a purification history, a
    mixture of a stated multicomponent kind and components that state their function."""
    species = w.kind(
        "species", {"canonical_key": "ethanol", "label": "ethanol"}, origins=at("ethanol")
    )
    water = w.kind("species", {"canonical_key": "water", "label": "water"}, origins=at("water"))
    sample = w.kind(
        "sample",
        {
            "carrier": CARRIER,
            "local_key": "1",
            "entity": species,
            "source": "standard_reference_material",
            "status": "described",
            "supplier": "NIST",
        },
        origins=at("sample-1"),
    )
    ids["sample"] = sample
    w.relation(
        "purity_statement",
        {"sample": sample, "ordinal": 1},
        {
            "basis": "mole",
            "value": 0.9995,
            "digits": 4,
            "method": entity_id(decl, "analysis_method", "gas_chromatography"),
        },
        at="a.json#/sample-1",
    )
    w.relation(
        "purity_statement",
        {"sample": sample, "ordinal": 2},
        {
            "basis": "mass",
            "value": 0.0003,
            "digits": 1,
            "method_text": "Karl Fischer titration",
            "impurity": water,
        },
        at="a.json#/sample-1",
    )
    w.relation(
        "purification_step",
        {"sample": sample, "step": 1},
        {"method": entity_id(decl, "purification_method", "fractional_distillation")},
        at="a.json#/sample-1",
    )
    w.relation(
        "purification_step",
        {"sample": sample, "step": 2},
        {"method_text": "dried over molecular sieves"},
        at="a.json#/sample-1",
    )
    w.kind(
        "sample",
        {
            "carrier": CARRIER,
            "local_key": "2",
            "entity": species,
            "source": "not_stated",
            "status": "described_previously",
        },
        origins=at("sample-2"),
    )
    mixture = w.kind(
        "defined_mixture",
        {
            "canonical_key": "brass",
            "label": "brass",
            "definition": "by_definition",
            "mole_basis": True,
            "multicomponent_kind": "alloy",
        },
        origins=at("brass"),
    )
    for name, fraction in (("ethanol", 0.5), ("water", 0.5)):
        w.relation(
            "mixture_component",
            {"mixture": mixture, "component": {"ethanol": species, "water": water}[name]},
            {"value": fraction},
            at="a.json#/brass",
        )
    dataset = w.kind(
        "dataset",
        {"carrier": CARRIER, "local_key": "biochemistry", "kind": "measured"},
        origins=[origin("a.json#/biochemistry", "measured")],
    )
    for ordinal, (function, speciation) in enumerate(
        (
            ("buffer", "equilibrium_mixture"),
            ("inert", "single_species"),
            ("cofactor", "not_stated"),
        ),
        1,
    ):
        w.kind(
            "dataset_component",
            {
                "dataset": dataset,
                "ordinal": ordinal,
                "entity": species,
                "function": function,
                "speciation": speciation,
            },
            at="a.json#/biochemistry",
        )


def write_evidence(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    """Values whose type is new: a negative surface excess, a kinematic viscosity and a quadrupole
    moment; and uncertainties whose kind is new."""
    dataset = w.kind(
        "dataset",
        {"carrier": CARRIER, "local_key": "new-types", "kind": "measured"},
        origins=[origin("a.json#/new-types", "measured")],
    )
    columns = {}
    for ordinal, name in enumerate(
        (
            "adsorbed_amount_excess",
            "fixture_kinematic_viscosity",
            "fixture_quadrupole_moment",
            "fixture_joule_thomson_coefficient",
            "fixture_enthalpy_of_solution",
            "fixture_transformed_gibbs_energy",
            "fixture_binary_diffusion",
        ),
        1,
    ):
        columns[name] = w.kind(
            "dataset_column",
            {
                "dataset": dataset,
                "ordinal": ordinal,
                "role": "property",
                "observable": decl.observable_entity(name).id,  # type: ignore[union-attr]
            },
            at="a.json#/new-types",
        )
    point = w.kind("data_point", {"dataset": dataset, "index": 1}, at="a.json#/new-types")
    for name, value in (
        ("adsorbed_amount_excess", Quantity(-0.25, "mmol/g")),
        ("fixture_kinematic_viscosity", Quantity(1.0e-6, "m^2/s")),
        ("fixture_quadrupole_moment", Quantity(-1.0e-39, "C*m^2")),
    ):
        w.relation(
            "datum",
            {"point": point, "column": columns[name]},
            {"state": "known", "value": value},
            at="a.json#/new-types",
        )
    for ordinal, kind in enumerate(("unspecified", "multiplicative_factor"), 1):
        assessment = w.kind(
            "uncertainty_assessment",
            {"column": columns["fixture_kinematic_viscosity"], "ordinal": ordinal, "kind": kind},
            at="a.json#/new-types",
        )
        magnitudes = (
            {"minus": Quantity(1.0e-8, "m^2/s"), "plus": Quantity(1.0e-8, "m^2/s")}
            if kind == "unspecified"
            else {"relative_minus": 1.1, "relative_plus": 1.1}
        )
        w.relation(
            "datum_uncertainty",
            {"point": point, "assessment": assessment},
            magnitudes,
            at="a.json#/new-types",
        )
    ids.update(evidence_dataset=dataset, evidence_point=point)


def write_primary_work(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    """An attribution that says where in the primary work the content is."""
    publication = w.citation("src", "dechema", doi="10.1000/dechema.1", year=1977)
    imported = w.import_record(SourceRef("src", "a.json", "a.json#/ipd"))
    ids["attribution"] = w.relation(
        "attribution",
        {"import_record": imported, "primary": publication},
        {"page": "217", "table": "3", "equation": "(12)", "figure": "4b"},
        at="a.json#/ipd",
    )


def write_forms(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    """Sets of the fixture forms: a dispersion class (item 23), a Flory-Huggins size and
    interaction under a stated composition basis and reference volume (item 10), and weights read
    with the Boltzmann and Avogadro constants of several convention sets (item 28)."""
    published = at("forms")
    cosmo = w.kind(
        "parameterization",
        {
            "key": "cosmo-sac-dsp",
            "revision": "1",
            "title": "COSMO-SAC dispersion",
            "coherence": "independent_records",
        },
        origins=published,
    )
    species = {
        name: w.kind(
            "species", {"canonical_key": f"k-{name}", "label": name}, origins=at(f"k-{name}")
        )
        for name in ("water", "ethanol", "oxygen")
    }
    for name, flag, energy in (("water", "water", 3.2e2), ("ethanol", "hb_donor_acceptor", 2.1e2)):
        ids[f"dsp_{name}"] = w.parameter_set(
            parameterization=cosmo,
            slot_group="cosmo_sac_dsp_fixture.pure",
            subjects=[species[name]],
            slots={"dispersion_class": flag, "energy_over_k": Quantity(energy, "K")},
            origins=published,
        )
    solvent, polymer = (
        w.kind(
            "pseudo_component",
            {"canonical_key": key, "label": key, "kind": "abstract_component"},
            origins=at(key),
        )
        for key in ("fh-solvent", "fh-polymer")
    )
    flory = w.kind(
        "parameterization",
        {
            "key": "flory-huggins",
            "revision": "1",
            "title": "Flory-Huggins",
            "coherence": "independent_records",
            "composition_basis": entity_id(decl, "composition_basis", "volume_fraction"),
            "reference_volume": Quantity(1.8e-5, "m^3/mol"),
        },
        origins=published,
    )
    ids["flory"] = flory
    for name, component, size in (("solvent", solvent, 1.0), ("polymer", polymer, 250.0)):
        ids[f"fh_size_{name}"] = w.parameter_set(
            parameterization=flory,
            slot_group="flory_huggins_fixture.pure",
            subjects=[component],
            slots={"size": size},
            origins=published,
        )
    ids["fh_chi"] = w.parameter_set(
        parameterization=flory,
        slot_group="flory_huggins_fixture.pair",
        subjects=[solvent, polymer],
        slots={"chi": 0.45},
        origins=published,
    )
    ids.update({f"species_{name}": value for name, value in species.items()})
    for key in (
        "kb_both",
        "kb_same",
        "kb_other_avogadro",
        "kb_only_boltzmann",
        "kb_only_avogadro",
        "kb_with_gas",
    ):
        parameterization = w.kind(
            "parameterization",
            {
                "key": f"p-{key}",
                "revision": "1",
                "title": key,
                "coherence": "independent_records",
                "convention_set": ids[f"conv_{key}"],
            },
            origins=published,
        )
        ids[f"p_{key}"] = parameterization
        for name, weight in (("water", 2.0), ("oxygen", 3.0)):
            if key == "kb_same" and name == "water":
                continue  # `kb_same` holds only oxygen, so a read of both draws on two parameterisations
            w.parameter_set(
                parameterization=parameterization,
                slot_group="additions_constants_sum.pure",
                subjects=[species[name]],
                slots={"w": weight},
                origins=published,
            )


@pytest.fixture(scope="module")
def world(tmp_path_factory: pytest.TempPathFactory) -> Iterator[World]:
    decl = additions_declaration(tmp_path_factory.mktemp("additions-declaration"))
    canonical = tmp_path_factory.mktemp("additions-canonical")
    ids: dict[str, uuid.UUID] = {}

    def fill(w: CanonicalWriter) -> None:
        for part in (
            write_isotopes,
            write_conventions,
            write_groups,
            write_phases,
            write_accuracy,
            write_levels,
            write_samples,
            write_evidence,
            write_primary_work,
            write_forms,
        ):
            part(w, decl, ids)

    write_source(canonical, "src", fill, decl=decl, declaration=fingerprint(decl))
    with TestDatabase() as database:
        build_database(database.url, decl, inputs_of(canonical))
        yield World(decl, database, ids)


@pytest.fixture
def conn(world: World) -> Iterator[psycopg.Connection]:
    connection = db.connect(world.database.url)
    connection.execute("SELECT 1")  # an open transaction: what a test changes is rolled back
    try:
        yield connection
    finally:
        connection.rollback()
        connection.close()


def same_unit(found: str, wanted: str) -> bool:
    """Whether two unit strings are the same unit (the declaration keeps its own rendering)."""
    units = registry()
    return bool(units.Unit(found) == units.Unit(wanted))


def scalar(conn: psycopg.Connection, query: str, *params: object) -> object:
    row = conn.execute(query, params).fetchone()  # type: ignore[arg-type]
    assert row is not None
    return row[0]


def fresh_writer(world: World) -> CanonicalWriter:
    return writer(world.decl)


def test_every_fixture_satisfies_every_invariant(world: World, conn: psycopg.Connection) -> None:
    results = run_checks(conn, list(CHECKS.values()))
    failing = [
        (r.check.target, r.violations, r.error)
        for r in results
        # the convention check flags the parameterisations that state no fact a form reads: the
        # fixtures of item 28 include two on purpose, and nothing else
        if not r.passed and r.check.target != "parameterization_has_conventions"
    ]
    assert failing == []
    flagged = run_check(conn, CHECKS["parameterization_has_conventions"])
    assert set(flagged.ids) == {
        str(world.ids[key])
        for key in ("p_kb_only_boltzmann", "p_kb_only_avogadro", "p_kb_with_gas")
    }


# -- 17: conserved quantities beyond elements ---------------------------------------------------


def test_an_isotope_is_a_conserved_quantity_of_an_element(
    world: World, conn: psycopg.Connection
) -> None:
    row = conn.execute(
        "SELECT q.key, q.kind::text, e.key, i.mass_number, i.atomic_mass, i.symbol "
        "FROM tk.isotope i JOIN tk.conserved_quantity q ON q.id = i.id "
        "JOIN tk.conserved_quantity e ON e.id = q.of_element WHERE q.key = '2H'"
    ).fetchone()
    assert row == ("2H", "isotope", "H", 2, pytest.approx(2.01410178e-3), "D")
    kinds = dict(
        conn.execute(
            "SELECT key, kind::text FROM tk.conserved_quantity WHERE key IN ('alk', 'Nit', 'LIGAND', 'H')"
        ).fetchall()
    )
    assert kinds == {
        "alk": "alkalinity",
        "Nit": "decoupled_inventory",
        "LIGAND": "moiety",
        "H": "element",
    }
    elements = dict(
        conn.execute(
            "SELECT key, of_element IS NOT NULL FROM tk.conserved_quantity "
            "WHERE key IN ('alk', 'Nit', 'LIGAND', '2H', 'H')"
        ).fetchall()
    )
    assert elements == {"alk": False, "Nit": True, "LIGAND": False, "2H": True, "H": False}


def test_the_heavy_water_exchange_reaction_is_balanced_in_hydrogen_and_deuterium(
    world: World, conn: psycopg.Connection
) -> None:
    for target in (
        "reaction.conserves_declared_quantities",
        "system_reaction.conserves_system_quantities",
    ):
        assert run_check(conn, CHECKS[target]).violations == 0


def test_the_writer_refuses_an_isotope_with_no_element_or_no_positive_mass_number(
    world: World,
) -> None:
    w = fresh_writer(world)
    hydrogen = world.ids["hydrogen"]
    with pytest.raises(ValidationError, match="mass_number_positive"):
        w.kind(
            "isotope",
            {"key": "0H", "kind": "isotope", "of_element": hydrogen, "mass_number": 0},
            origins=at("bad"),
        )
    with pytest.raises(ValidationError, match="mass_number"):
        w.kind(
            "isotope", {"key": "xH", "kind": "isotope", "of_element": hydrogen}, origins=at("bad")
        )


# -- 20: temperature scales ------------------------------------------------------------------


def test_convention_sets_state_the_provisional_low_temperature_scale_and_the_scale_of_1927(
    world: World, conn: psycopg.Connection
) -> None:
    scales = dict(
        conn.execute(
            "SELECT key, temperature_scale::text FROM tk.convention_set WHERE key IN ('ept', 'its27')"
        ).fetchall()
    )
    assert scales == {"ept": "ept_76", "its27": "its_27"}


# -- 21: group identity and structure ---------------------------------------------------------


def test_a_scheme_holds_two_groups_with_one_label_and_one_carrier_two_decompositions(
    world: World, conn: psycopg.Connection
) -> None:
    rows = conn.execute(
        'SELECT code, label FROM tk."group" WHERE scheme = %s ORDER BY code',
        (world.ids["groups_scheme"],),
    ).fetchall()
    assert rows == [("1", "CH3"), ("10", "CHO"), ("9", "CHO")]
    assert conn.execute(
        "SELECT occurrence FROM tk.group_assignment WHERE entity = %s ORDER BY occurrence",
        (world.ids["groups_species"],),
    ).fetchall() == [(1,), (2,)]


def test_a_bond_between_two_groups_is_one_unordered_row_and_may_join_a_group_to_its_kind(
    world: World, conn: psycopg.Connection
) -> None:
    rows = conn.execute(
        "SELECT g1.code, g2.code, b.value FROM tk.group_bond_count b "
        'JOIN tk."group" g1 ON g1.id = b.first JOIN tk."group" g2 ON g2.id = b.second '
        "WHERE b.assignment = %s ORDER BY 1, 2",
        (world.ids["assignment_first"],),
    ).fetchall()
    assert sorted((frozenset((first, second)), value) for first, second, value in rows) == sorted(
        [(frozenset(("1",)), 0.0), (frozenset(("9", "1")), 1.0)]
    )


def test_the_writer_stores_a_bond_in_the_canonical_orientation_whichever_way_it_is_given(
    world: World,
) -> None:
    w = fresh_writer(world)
    scheme = w.kind(
        "group_scheme", {"key": "s", "revision": "1", "role": "activity"}, origins=at("s")
    )
    first, second = (
        w.kind("group", {"scheme": scheme, "code": c, "label": c, "role": "group"}, origins=at(c))
        for c in ("a", "b")
    )
    assignment = w.kind(
        "group_assignment",
        {"entity": first, "scheme": scheme, "asserted_by": CARRIER, "origin": "published"},
        origins=at("x"),
    )
    forward = w.relation(
        "group_bond_count",
        {"assignment": assignment, "first": first, "second": second},
        {"value": 1},
        at="a.json#/1",
    )
    backward = w.relation(
        "group_bond_count",
        {"assignment": assignment, "first": second, "second": first},
        {"value": 1},
        at="a.json#/1",
    )
    assert forward == backward, "the pair is unordered: the same row"
    w.relation(
        "group_bond_count",
        {"assignment": assignment, "first": first, "second": first},
        {"value": 2},
        at="a.json#/2",
    )
    with pytest.raises(ValidationError, match="occurrence"):
        w.kind(
            "group_assignment",
            {
                "entity": first,
                "scheme": scheme,
                "asserted_by": CARRIER,
                "origin": "published",
                "occurrence": 0,
            },
            origins=at("y"),
        )


# -- 22: energy datum with stated values -------------------------------------------------------


def test_the_iir_datum_states_enthalpy_and_entropy_of_the_saturated_liquid(
    world: World, conn: psycopg.Connection
) -> None:
    row = conn.execute(
        "SELECT enthalpy::text, entropy::text, datum_energy::text, temperature, state, "
        "specific_energy_value, specific_entropy_value, energy_value, entropy_value, a.name "
        "FROM tk.energy_reference r JOIN tk.aggregation a ON a.id = r.aggregation WHERE r.key = 'iir'"
    ).fetchone()
    assert row == (
        "at_state",
        "at_state",
        "enthalpy",
        pytest.approx(273.15),
        "saturated liquid",
        pytest.approx(2.0e5),
        pytest.approx(1.0e3),
        None,
        None,
        "liquid",
    )


def test_iapws95_fixes_the_internal_energy_and_the_entropy_of_the_liquid_at_the_triple_point(
    world: World, conn: psycopg.Connection
) -> None:
    row = conn.execute(
        "SELECT enthalpy::text, datum_energy::text, energy_value, specific_energy_value, entropy_value, "
        "temperature, pressure FROM tk.energy_reference WHERE key = 'iapws95'"
    ).fetchone()
    assert row == (
        "at_state",
        "internal_energy",
        0.0,
        None,
        0.0,
        pytest.approx(273.16),
        pytest.approx(611.654771),
    ), "a stated zero is a stated value"


def test_no_datum_member_is_called_zero_at_state(world: World, conn: psycopg.Connection) -> None:
    members = conn.execute(
        "SELECT enum, name FROM meta.enum_member WHERE enum IN ('enthalpy_datum', 'entropy_datum') "
        "AND name IN ('at_state', 'zero_at_state') ORDER BY enum"
    ).fetchall()
    assert members == [("enthalpy_datum", "at_state"), ("entropy_datum", "at_state")]
    assert conn.execute(
        "SELECT enum, member FROM meta.enum_member_facet WHERE facet = 'stated_value' ORDER BY enum"
    ).fetchall() == [("enthalpy_datum", "at_state"), ("entropy_datum", "at_state")]


def test_the_datum_energy_is_one_of_enthalpy_and_internal_energy(
    world: World, conn: psycopg.Connection
) -> None:
    assert conn.execute(
        "SELECT name FROM meta.enum_member WHERE enum = 'datum_energy' ORDER BY name"
    ).fetchall() == [("enthalpy",), ("internal_energy",)]


# -- 23: COSMO-SAC dispersion class ---------------------------------------------------------------


def test_a_slot_group_declares_a_slot_of_the_dispersion_class_and_a_set_loads_with_it(
    world: World, conn: psycopg.Connection
) -> None:
    rows = dict(
        conn.execute(
            "SELECT s.label, d.dispersion_class::text FROM param.cosmo_sac_dsp_fixture__pure d "
            "JOIN tk.material_entity s ON s.id = d.i"
        ).fetchall()
    )
    assert rows == {"water": "water", "ethanol": "hb_donor_acceptor"}
    assert conn.execute(
        "SELECT name FROM meta.enum_member WHERE enum = 'dispersion_class' ORDER BY name"
    ).fetchall() == [("cooh",), ("hb_acceptor",), ("hb_donor_acceptor",), ("nhb",), ("water",)]


def test_the_writer_refuses_a_dispersion_class_that_is_not_a_member(world: World) -> None:
    w = fresh_writer(world)
    parameterization = w.kind(
        "parameterization",
        {"key": "p", "revision": "1", "title": "p", "coherence": "independent_records"},
        origins=at("p"),
    )
    species = w.kind("species", {"canonical_key": "x", "label": "x"}, origins=at("x"))
    with pytest.raises(ValidationError, match="dispersion_class"):
        w.parameter_set(
            parameterization=parameterization,
            slot_group="cosmo_sac_dsp_fixture.pure",
            subjects=[species],
            slots={"dispersion_class": "OH", "energy_over_k": Quantity(1.0, "K")},
            origins=at("p"),
        )


# -- 24: site-class-qualified evidence --------------------------------------------------------------


def test_a_column_reports_on_a_site_class_of_the_phase_definition_its_phase_names(
    world: World, conn: psycopg.Connection
) -> None:
    row = conn.execute(
        "SELECT pd.key, sc.index FROM ev.dataset_column c "
        "JOIN ev.dataset_phase p ON p.id = c.phase JOIN tk.phase_definition pd ON pd.id = p.phase_definition "
        "JOIN tk.site_class sc ON sc.id = c.site_class WHERE c.id = %s",
        (world.ids["column_site"],),
    ).fetchone()
    assert row == ("BCC_B2", 1)
    assert run_check(conn, CHECKS["dataset_column.site_class_of_the_phase"]).violations == 0


# -- 25: order-disorder pairing -----------------------------------------------------------------------


def test_an_ordered_phase_names_its_disordered_partner(
    world: World, conn: psycopg.Connection
) -> None:
    row = conn.execute(
        "SELECT o.key, d.key, p.never_disorder FROM tk.disordered_partner p "
        "JOIN tk.phase_definition o ON o.id = p.ordered JOIN tk.phase_definition d ON d.id = p.disordered"
    ).fetchone()
    assert row == ("BCC_B2", "BCC_A2", False)


def test_the_writer_refuses_a_phase_as_its_own_partner(world: World) -> None:
    w = fresh_writer(world)
    system = w.kind("chemical_system", {"key": "s", "revision": "1"}, origins=at("s"))
    phase = w.kind(
        "phase_definition",
        {
            "system": system,
            "key": "BCC_A2",
            "aggregation": entity_id(world.decl, "aggregation", "crystalline"),
            "structure": "sublattice",
        },
        origins=at("phase"),
    )
    with pytest.raises(ValidationError, match="diagonal"):
        w.relation(
            "disordered_partner",
            {"ordered": phase, "disordered": phase},
            {"never_disorder": True},
            at="a.json#/p",
        )


# -- 26 and 27: accuracy statements and uncertainty kinds -----------------------------------------------


def test_iapws95_states_a_relative_density_bound_over_a_region_of_temperature_and_pressure(
    world: World, conn: psycopg.Connection
) -> None:
    row = conn.execute(
        "SELECT o.key, a.kind::text, a.statistic::text, a.relative_magnitude, a.magnitude, a.sample_size, "
        "a.region, a.against FROM tk.accuracy_statement a JOIN tk.observable o ON o.id = a.observable "
        "WHERE a.id = %s",
        (world.ids["accuracy_density"],),
    ).fetchone()
    assert row is not None
    assert row[:6] == ("molar_density", "relative", "bound", pytest.approx(1e-5), None, 120)
    assert row[7] is None
    clauses = conn.execute(
        "SELECT o.key, c.lower, c.upper FROM tk.region_clause c JOIN tk.observable o ON o.id = c.observable "
        "WHERE c.region = %s ORDER BY c.ordinal",
        (row[6],),
    ).fetchall()
    assert clauses == [
        ("temperature", pytest.approx(273.16), pytest.approx(350.0)),
        ("pressure", pytest.approx(1e5), pytest.approx(1e8)),
    ]


def test_if97_states_a_25_millikelvin_consistency_bound_against_a_second_record(
    world: World, conn: psycopg.Connection
) -> None:
    row = conn.execute(
        "SELECT a.magnitude, a.against, a.kind::text, a.region FROM tk.accuracy_statement a WHERE a.id = %s",
        (world.ids["accuracy_consistency"],),
    ).fetchone()
    assert row == (pytest.approx(0.025), world.ids["iapws95"], "interval", None)


def test_a_condition_is_a_region_with_a_clause_of_one_point(
    world: World, conn: psycopg.Connection
) -> None:
    lower, upper = conn.execute(
        "SELECT c.lower, c.upper FROM tk.accuracy_statement a JOIN tk.region_clause c ON c.region = a.region "
        "WHERE a.id = %s",
        (world.ids["accuracy_at_a_condition"],),
    ).fetchone()  # type: ignore[misc]
    assert lower == upper == pytest.approx(298.15)


def test_a_multiplicative_factor_and_an_unspecified_magnitude_are_held(
    world: World, conn: psycopg.Connection
) -> None:
    rows = dict(
        conn.execute(
            "SELECT kind::text, coalesce(relative_magnitude, magnitude) FROM tk.accuracy_statement "
            "WHERE id IN (%s, %s)",
            (world.ids["accuracy_factor"], world.ids["accuracy_unspecified"]),
        ).fetchall()
    )
    assert rows == {"multiplicative_factor": 2.0, "unspecified": 4.0}
    rows = dict(
        conn.execute(
            "SELECT a.kind::text, coalesce(u.relative_minus, u.minus) FROM ev.datum_uncertainty u "
            "JOIN ev.uncertainty_assessment a ON a.id = u.assessment "
            "WHERE a.kind IN ('unspecified', 'multiplicative_factor')"
        ).fetchall()
    )
    assert rows == {"unspecified": pytest.approx(1e-8), "multiplicative_factor": 1.1}


def test_the_facet_factor_marks_the_multiplicative_kind_only(
    world: World, conn: psycopg.Connection
) -> None:
    assert conn.execute(
        "SELECT member FROM meta.enum_member_facet WHERE enum = 'uncertainty_kind' AND facet = 'factor'"
    ).fetchall() == [("multiplicative_factor",)]
    members = {
        name
        for (name,) in conn.execute(
            "SELECT name FROM meta.enum_member WHERE enum = 'uncertainty_kind'"
        )
    }
    assert {"unspecified", "multiplicative_factor"} <= members
    assert ("unspecified",) not in conn.execute(
        "SELECT member FROM meta.enum_member_facet WHERE enum = 'uncertainty_kind'"
    ).fetchall()


def test_the_writer_refuses_a_statement_with_a_position_of_zero_or_no_sample(world: World) -> None:
    w = fresh_writer(world)
    record = w.kind(
        "parameterization",
        {"key": "p", "revision": "1", "title": "p", "coherence": "independent_records"},
        origins=at("p"),
    )
    temperature = world.decl.observable_entity("temperature").id  # type: ignore[union-attr]
    common = {"kind": "exact", "statistic": "bound"}
    with pytest.raises(ValidationError, match="ordinal_from_one"):
        w.relation(
            "accuracy_statement",
            {"record": record, "observable": temperature, "ordinal": 0},
            common,
            at="a.json#/1",
        )
    with pytest.raises(ValidationError, match="sample_size_positive"):
        w.relation(
            "accuracy_statement",
            {"record": record, "observable": temperature, "ordinal": 1},
            {**common, "sample_size": 0},
            at="a.json#/1",
        )


# -- 28: convention constants -------------------------------------------------------------------------


def test_the_constants_are_quantity_types_and_convention_attributes(
    world: World, conn: psycopg.Connection
) -> None:
    assert conn.execute(
        "SELECT name, scale::text FROM meta.quantity_type "
        "WHERE name IN ('BoltzmannConstant', 'AvogadroConstant') ORDER BY name"
    ).fetchall() == [("AvogadroConstant", "absolute"), ("BoltzmannConstant", "absolute")]
    types = world.decl.quantity_types
    assert same_unit(types["BoltzmannConstant"].unit, "J/K")
    assert same_unit(types["AvogadroConstant"].unit, "1/mol")
    assert conn.execute(
        "SELECT form, name FROM meta.form_convention WHERE form = 'additions_constants_sum' ORDER BY name"
    ).fetchall() == [
        ("additions_constants_sum", "avogadro_constant"),
        ("additions_constants_sum", "boltzmann_constant"),
    ]


def constants_source(world: World, conn: psycopg.Connection, *keys: str) -> DatabaseSource:
    return DatabaseSource(conn, world.decl, [world.ids[f"p_{key}"] for key in keys])


def weighted(world: World, found: DatabaseSource, names: list[str]) -> np.ndarray:
    members = [str(world.ids[f"species_{name}"]) for name in names]
    bound = bind(world.decl, "additions_constants_sum", source=found, sets={"components": members})
    return bound.evaluate("y", T=np.array([300.0]))


def test_a_form_reads_the_boltzmann_and_avogadro_constants_of_the_convention_set(
    world: World, conn: psycopg.Connection
) -> None:
    value = weighted(world, constants_source(world, conn, "kb_both"), ["water", "oxygen"])
    np.testing.assert_allclose(value, 1.380649e-23 * 6.02214076e23 * 5.0, rtol=1e-14)


def test_a_parameterisation_that_omits_a_constant_the_form_reads_is_refused(
    world: World, conn: psycopg.Connection
) -> None:
    for key, missing in (
        ("kb_only_boltzmann", "avogadro_constant"),
        ("kb_only_avogadro", "boltzmann_constant"),
        ("kb_with_gas", "boltzmann_constant"),
    ):
        with pytest.raises(EvaluationRefusal, match=rf"`p-{key}@1`.*`{missing}`"):
            weighted(world, constants_source(world, conn, key), ["water", "oxygen"])


def test_the_check_flags_the_parameterisations_that_state_no_constant_a_form_reads(
    world: World, conn: psycopg.Connection
) -> None:
    flagged = run_check(conn, CHECKS["parameterization_has_conventions"])
    reasons = {row[0]: row[-1] for row in flagged.rows}
    assert (
        reasons[str(world.ids["p_kb_only_boltzmann"])]
        == "its convention set states no avogadro_constant"
    )
    assert (
        reasons[str(world.ids["p_kb_only_avogadro"])]
        == "its convention set states no boltzmann_constant"
    )


def test_sets_drawn_from_parameterisations_with_different_avogadro_constants_are_refused(
    world: World, conn: psycopg.Connection
) -> None:
    # `kb_same` holds only oxygen: water comes from the second parameterisation
    with pytest.raises(EvaluationRefusal, match="`avogadro_constant` differs"):
        weighted(
            world,
            constants_source(world, conn, "kb_same", "kb_other_avogadro"),
            ["water", "oxygen"],
        )
    value = weighted(
        world, constants_source(world, conn, "kb_same", "kb_both"), ["water", "oxygen"]
    )
    np.testing.assert_allclose(value, 1.380649e-23 * 6.02214076e23 * 5.0, rtol=1e-14)


# -- 29: level of theory -------------------------------------------------------------------------------


def test_a_computation_names_a_composite_level_of_theory_of_a_frequency_and_an_energy_level(
    world: World, conn: psycopg.Connection
) -> None:
    row = conn.execute(
        "SELECT d.kind::text, l.method, f.method, f.basis, e.method, e.auxiliary_basis, e.cabs, e.solvent, "
        "e.solvation_method, e.arguments::text, r.version "
        "FROM prov.derivation d JOIN prov.level_of_theory l ON l.id = d.level_of_theory "
        "JOIN prov.level_of_theory f ON f.id = l.frequency_level JOIN prov.level_of_theory e ON e.id = l.energy_level "
        "JOIN prov.software_release r ON r.id = e.software WHERE d.id = %s",
        (world.ids["derivation_computed"],),
    ).fetchone()
    assert row is not None
    assert row[:9] == (
        "computation",
        "composite",
        "B3LYP",
        "6-31G(d)",
        "CCSD(T)-F12",
        "cc-pVTZ-F12/JKFIT",
        "cc-pVTZ-F12/OPTRI",
        "water",
        "COSMO",
    )
    assert "2 GB" in str(row[9]) and row[10] == "2022.1"


# -- 30: sample description -------------------------------------------------------------------------------


def test_a_sample_states_its_source_and_status_and_several_purity_statements(
    world: World, conn: psycopg.Connection
) -> None:
    assert conn.execute(
        "SELECT local_key, source::text, status::text, supplier FROM tk.sample ORDER BY local_key"
    ).fetchall() == [
        ("1", "standard_reference_material", "described", "NIST"),
        ("2", "not_stated", "described_previously", None),
    ]
    rows = conn.execute(
        "SELECT p.ordinal, p.basis::text, p.value, p.digits, m.key, p.method_text, e.canonical_key "
        "FROM tk.purity_statement p LEFT JOIN tk.analysis_method m ON m.id = p.method "
        "LEFT JOIN tk.material_entity e ON e.id = p.impurity WHERE p.sample = %s ORDER BY p.ordinal",
        (world.ids["sample"],),
    ).fetchall()
    assert rows == [
        (1, "mole", pytest.approx(0.9995), 4, "gas_chromatography", None, None),
        (2, "mass", pytest.approx(0.0003), 1, None, "Karl Fischer titration", "water"),
    ]


def test_a_sample_has_a_stepwise_purification_history(
    world: World, conn: psycopg.Connection
) -> None:
    rows = conn.execute(
        "SELECT s.step, m.key, s.method_text FROM tk.purification_step s "
        "LEFT JOIN tk.purification_method m ON m.id = s.method WHERE s.sample = %s ORDER BY s.step",
        (world.ids["sample"],),
    ).fetchall()
    assert rows == [(1, "fractional_distillation", None), (2, None, "dried over molecular sieves")]


def test_a_sample_has_no_purity_attribute_of_its_own(
    world: World, conn: psycopg.Connection
) -> None:
    columns = {
        name
        for (name,) in conn.execute(
            "SELECT column_name FROM information_schema.columns WHERE table_schema = 'tk' AND table_name = 'sample'"
        )
    }
    assert {"purity", "purity_basis"}.isdisjoint(columns)
    assert {"source", "status", "supplier"} <= columns


def test_the_writer_refuses_a_step_with_no_method_or_two_and_a_position_of_zero(
    world: World,
) -> None:
    w = fresh_writer(world)
    species = w.kind("species", {"canonical_key": "x", "label": "x"}, origins=at("x"))
    sample = w.kind(
        "sample",
        {
            "carrier": CARRIER,
            "local_key": "s",
            "entity": species,
            "source": "commercial",
            "status": "described",
        },
        origins=at("s"),
    )
    method = entity_id(world.decl, "purification_method", "fractional_distillation")
    for values, step, message in (
        ({}, 1, "one_method"),
        ({"method": method, "method_text": "distilled"}, 1, "one_method"),
        ({"method_text": "distilled"}, 0, "step_from_one"),
    ):
        with pytest.raises(ValidationError, match=message):
            w.relation(
                "purification_step", {"sample": sample, "step": step}, values, at="a.json#/s"
            )
    with pytest.raises(ValidationError, match="ordinal_from_one"):
        w.relation(
            "purity_statement", {"sample": sample, "ordinal": 0}, {"basis": "mole"}, at="a.json#/s"
        )


# -- 31: component description --------------------------------------------------------------------------------


def test_components_state_their_function_and_speciation_and_a_mixture_its_multicomponent_kind(
    world: World, conn: psycopg.Connection
) -> None:
    assert conn.execute(
        "SELECT function::text, speciation::text FROM ev.dataset_component ORDER BY ordinal"
    ).fetchall() == [
        ("buffer", "equilibrium_mixture"),
        ("inert", "single_species"),
        ("cofactor", "not_stated"),
    ]
    assert conn.execute("SELECT multicomponent_kind::text FROM tk.defined_mixture").fetchall() == [
        ("alloy",)
    ]
    assert conn.execute(
        "SELECT name FROM meta.enum_member WHERE enum = 'speciation_state' ORDER BY name"
    ).fetchall() == [("equilibrium_mixture",), ("not_stated",), ("single_species",)]
    assert conn.execute(
        "SELECT name FROM meta.enum_member WHERE enum = 'multicomponent_kind' ORDER BY name"
    ).fetchall() == [("alloy",), ("clathrate",), ("complex",), ("crystal",), ("solution",)]


def test_a_component_that_states_no_speciation_does_not_state_a_single_species(
    world: World,
) -> None:
    w = fresh_writer(world)
    species = w.kind("species", {"canonical_key": "x", "label": "x"}, origins=at("x"))
    dataset = w.kind(
        "dataset",
        {"carrier": CARRIER, "local_key": "d", "kind": "measured"},
        origins=[origin("a.json#/d", "measured")],
    )
    component = w.kind(
        "dataset_component",
        {"dataset": dataset, "ordinal": 1, "entity": species, "function": "component"},
        at="a.json#/d",
    )
    assert component is not None
    rows = w.tables()["ev.dataset_component"].to_pylist()
    assert rows[0]["speciation"] == "not_stated"


# -- 32 and 33: transformed quantities and unstated conventions ----------------------------------------------------


def test_a_transformed_quantity_and_standard_state_are_members(
    world: World, conn: psycopg.Connection
) -> None:
    assert conn.execute(
        "SELECT relation::text, path::text, basis::text, diffusion::text, key FROM tk.observable "
        "WHERE key LIKE 'fixture\\_%' ORDER BY key"
    ).fetchall() == [
        ("absolute", "none", "none", "not_stated", "fixture_binary_diffusion"),
        (
            "increment",
            "none",
            "per_mole_of_solute",
            "not_applicable",
            "fixture_enthalpy_of_solution",
        ),
        ("absolute", "isenthalpic", "none", "not_applicable", "fixture_joule_thomson_coefficient"),
        ("absolute", "none", "none", "not_applicable", "fixture_kinematic_viscosity"),
        ("absolute", "none", "none", "not_applicable", "fixture_quadrupole_moment"),
        ("transformed", "none", "molar", "not_applicable", "fixture_transformed_gibbs_energy"),
    ]
    assert scalar(conn, "SELECT kind::text FROM tk.standard_state WHERE key = 'transformed'") == (
        "transformed_biochemical"
    )


def test_a_standard_state_may_state_no_pressure_and_no_scale_for_infinite_dilution(
    world: World, conn: psycopg.Connection
) -> None:
    rows = conn.execute(
        "SELECT s.key, s.kind::text, s.pressure_rule::text, b.name FROM tk.standard_state s "
        "LEFT JOIN tk.composition_basis b ON b.id = s.scale WHERE s.key IN ('unstated', 'dilute_without_scale') "
        "ORDER BY s.key"
    ).fetchall()
    assert rows == [
        ("dilute_without_scale", "infinite_dilution", "not_stated", "not_stated"),
        ("unstated", "not_stated", "not_stated", None),
    ]


def test_an_infinite_dilution_standard_state_still_needs_a_scale_and_a_solvent(
    world: World,
) -> None:
    w = fresh_writer(world)
    with pytest.raises(ValidationError, match="names a solvent and a composition scale"):
        w.kind(
            "standard_state",
            {"key": "bare", "kind": "infinite_dilution", "pressure_rule": "not_stated"},
            origins=at("bare"),
        )


# -- 34: quantity types -------------------------------------------------------------------------------------------


def test_the_new_quantity_types_have_the_storage_units_and_scales_declared(
    world: World, conn: psycopg.Connection
) -> None:
    declared = {
        "ElectricPotential": ("V", "difference"),
        "ExcessLoading": ("mol/kg", "difference"),
        "Fluidity": ("1/(Pa*s)", "absolute"),
        "KinematicViscosity": ("m^2/s", "absolute"),
        "Loading": ("mol/kg", "absolute"),
        "MomentOfInertia": ("kg*m^2", "absolute"),
        "PolarizabilityVolume": ("m^3", "absolute"),
        "QuadrupoleMoment": ("C*m^2", "difference"),
        "ThermalDiffusivity": ("m^2/s", "absolute"),
        "Wavenumber": ("1/m", "absolute"),
    }
    for name, (unit, scale) in declared.items():
        found = world.decl.quantity_types[name]
        assert same_unit(found.unit, unit) and found.scale == scale, name
    assert dict(
        conn.execute(
            "SELECT name, scale::text FROM meta.quantity_type WHERE name = ANY(%s)",
            (list(declared),),
        ).fetchall()
    ) == {name: scale for name, (_, scale) in declared.items()}


def test_a_surface_excess_can_be_negative_and_a_quadrupole_moment_carries_its_sign(
    world: World, conn: psycopg.Connection
) -> None:
    rows = dict(
        conn.execute(
            "SELECT o.key, d.value FROM ev.datum d JOIN ev.dataset_column c ON c.id = d.column "
            "JOIN tk.observable o ON o.id = c.observable WHERE c.dataset = %s",
            (world.ids["evidence_dataset"],),
        ).fetchall()
    )
    assert rows["adsorbed_amount_excess"] == pytest.approx(-0.25)
    assert rows["fixture_quadrupole_moment"] == pytest.approx(-1.0e-39)
    assert rows["fixture_kinematic_viscosity"] == pytest.approx(1.0e-6)


# -- 36: attribution locator ------------------------------------------------------------------------------------------


def test_an_attribution_says_where_in_the_primary_work_the_content_is(
    world: World, conn: psycopg.Connection
) -> None:
    assert conn.execute(
        'SELECT page, "table", equation, figure FROM prov.attribution WHERE id = %s',
        (world.ids["attribution"],),
    ).fetchall() == [("217", "3", "(12)", "4b")]


# -- 10: size and reference volume ------------------------------------------------------------------------------------------


def test_a_flory_huggins_parameterisation_states_its_composition_basis_and_reference_volume(
    world: World, conn: psycopg.Connection
) -> None:
    row = conn.execute(
        "SELECT b.name, p.reference_volume FROM tk.parameterization p "
        "JOIN tk.composition_basis b ON b.id = p.composition_basis WHERE p.id = %s",
        (world.ids["flory"],),
    ).fetchone()
    assert row == ("volume_fraction", pytest.approx(1.8e-5))


def test_abstract_pseudo_components_have_a_size_slot_and_an_interaction_slot(
    world: World, conn: psycopg.Connection
) -> None:
    sizes = dict(
        conn.execute(
            "SELECT e.label, s.size FROM param.flory_huggins_fixture__pure s "
            "JOIN tk.material_entity e ON e.id = s.i"
        ).fetchall()
    )
    assert sizes == {"fh-solvent": 1.0, "fh-polymer": 250.0}
    chi = conn.execute("SELECT chi FROM param.flory_huggins_fixture__pair").fetchall()
    assert chi == [(0.45,)]
    kinds = dict(
        conn.execute(
            "SELECT e.label, p.kind::text FROM tk.pseudo_component p JOIN tk.material_entity e ON e.id = p.id"
        ).fetchall()
    )
    assert kinds == {"fh-solvent": "abstract_component", "fh-polymer": "abstract_component"}
