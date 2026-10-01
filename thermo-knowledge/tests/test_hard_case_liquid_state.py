# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Hard case (l), plan 24 packet TK2: density functionals and integral-equation inputs.

The inputs of a PRISM-style integral-equation theory: the components are site types, abstract
pseudo-components (kind `abstract_component`) with no formula and no molecule, whose composition is a
number of sites per volume (the observable `site_number_density`, on the composition basis of the
same name). The potential between two site types is a pair set of a potential form (Lennard-Jones, or
the purely repulsive Weeks-Chandler-Andersen cut of it), stored once for an unordered pair with the
diagonal allowed; the potential and the closure (Percus-Yevick, hypernetted chain) of each pair are
chosen per pair by the choices of the parameterisation.

A SurfPack-style functional is a form whose structure is declared and whose equation is in code: a SAFT
functional with sub-form slots for its hard-sphere, dispersion, chain and association contributions,
each a catalogued form whose parameters are slots of its components and pairs, with the completeness
`structure_declared_equation_external`. An assembly names the form of each slot.

The numbers are synthetic. The only evaluation is the pair potential and, pointwise, the closure
applied to it, written in numpy.
"""

from __future__ import annotations

import uuid
from collections.abc import Iterator
from dataclasses import dataclass
from pathlib import Path

import numpy as np
import psycopg
import pytest

from hard_case_support import CHECKS, at, build, entity_id, failing, observable_id
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
NM = 1.0e-9

# -- the synthetic system: two site types and their pair potentials -------------------------------

SITES = ("A", "B")
DENSITY_PER_NM3 = {"A": 6.0, "B": 3.5}
# (first, second) as the source asserts the pair: (epsilon over k in K, sigma in nm, form)
PAIRS = {
    ("A", "A"): (250.0, 0.40, "lennard_jones_site_potential"),
    ("B", "A"): (180.0, 0.43, "lennard_jones_site_potential"),  # asserted with B first
    ("B", "B"): (120.0, 0.46, "wca_site_potential"),
}
CLOSURES = {
    ("A", "A"): "percus_yevick_closure",
    ("A", "B"): "hypernetted_chain_closure",
    ("B", "B"): "hypernetted_chain_closure",
}
POTENTIAL_OF = {
    ("A", "A"): "lennard_jones_site_potential",
    ("A", "B"): "lennard_jones_site_potential",
    ("B", "B"): "wca_site_potential",
}
R = np.array([0.30, 0.38, 0.44, 0.50, 0.80, 1.20]) * NM
TEMPERATURES = (300.0, 450.0)
GAMMA = np.array([-0.4, -0.1, 0.0, 0.2, 0.5, 1.1])

# -- the independent calculation ---------------------------------------------------------------------


def lennard_jones(r: np.ndarray, T: float, eps: float, sigma: float) -> np.ndarray:
    x = sigma * NM / r
    return 4.0 * eps / T * (x**12 - x**6)


def wca(r: np.ndarray, T: float, eps: float, sigma: float) -> np.ndarray:
    return np.where(
        r < 2.0 ** (1.0 / 6.0) * sigma * NM, lennard_jones(r, T, eps, sigma) + eps / T, 0.0
    )


def potential(pair: tuple[str, str], r: np.ndarray, T: float) -> np.ndarray:
    key = pair if pair in PAIRS else (pair[1], pair[0])
    eps, sigma, form = PAIRS[key]
    return (lennard_jones if form == "lennard_jones_site_potential" else wca)(r, T, eps, sigma)


def percus_yevick(u: np.ndarray, gamma: np.ndarray) -> np.ndarray:
    return (np.exp(-u) - 1.0) * (1.0 + gamma)


def hypernetted_chain(u: np.ndarray, gamma: np.ndarray) -> np.ndarray:
    return np.exp(-u + gamma) - 1.0 - gamma


CLOSURE_FUNCTION = {
    "percus_yevick_closure": percus_yevick,
    "hypernetted_chain_closure": hypernetted_chain,
}

# -- the functional -----------------------------------------------------------------------------------

FLUIDS = ("fluid-a", "fluid-b")
SAFT = {  # component: (m, sigma in angstrom, epsilon over k in K)
    "fluid-a": (1.0, 3.7, 150.0),
    "fluid-b": (2.4, 3.3, 230.0),
}
K_AB = 0.03
ASSOCIATION = {"fluid-b": (2500.0, 0.05)}  # only fluid-b associates
CONTRIBUTIONS = {  # path: (slot, form)
    "hard_sphere": ("saft_functional.hard_sphere", "white_bear_hard_sphere_functional"),
    "dispersion": ("saft_functional.dispersion", "saft_dispersion_functional"),
    "chain": ("saft_functional.chain", "pc_saft_chain_functional"),
    "association": ("saft_functional.association", "pc_saft_association_functional"),
}

# -- the fixture -----------------------------------------------------------------------------------------


@dataclass
class World:
    decl: Declaration
    database: TestDatabase
    ids: dict[str, uuid.UUID]


def parameterization(w: CanonicalWriter, key: str, **extra: object) -> uuid.UUID:
    return w.kind(
        "parameterization",
        {"key": key, "revision": "1", "title": key, "coherence": "independent_records", **extra},
        origins=at(f"parameterization-{key}"),
    )


def write_sites(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    for name in SITES:
        ids[f"site_{name}"] = w.kind(
            "pseudo_component",
            {
                "canonical_key": f"site-{name}",
                "label": f"site type {name}",
                "kind": "abstract_component",
            },
            origins=at(f"site-{name}", "conventional"),
        )
    # the state of the system: the number of sites of each type per volume, a composition on the basis of site densities
    dataset = w.kind(
        "dataset",
        {
            "carrier": CARRIER,
            "local_key": "system",
            "kind": "computed",
            "title": "A polymer liquid: sites of each type per volume",
        },
        origins=at("system", "computed"),
    )
    ids["system"] = dataset
    basis = entity_id(decl, "composition_basis", "site_number_density")
    w.kind(
        "dataset_column",
        {
            "dataset": dataset,
            "ordinal": 1,
            "role": "constraint",
            "observable": observable_id(decl, "temperature"),
            "constant": Quantity(450.0, "K"),
        },
        at="a.json#/columns",
    )
    for ordinal, name in enumerate(SITES, start=2):
        component = w.kind(
            "dataset_component",
            {
                "dataset": dataset,
                "ordinal": ordinal - 1,
                "entity": ids[f"site_{name}"],
                "function": "component",
            },
            at="a.json#/components",
        )
        ids[f"density_{name}"] = w.kind(
            "dataset_column",
            {
                "dataset": dataset,
                "ordinal": ordinal,
                "role": "constraint",
                "observable": observable_id(decl, "site_number_density"),
                "component": component,
                "composition_basis": basis,
                "constant": Quantity(DENSITY_PER_NM3[name], "1/nm**3"),
            },
            at="a.json#/columns",
        )


def write_pairs(w: CanonicalWriter, ids: dict[str, uuid.UUID]) -> None:
    sets = {}
    for form in ("lennard_jones_site_potential", "wca_site_potential"):
        sets[form] = parameterization(w, form.replace("_site_potential", "-sites"))
        ids[f"parameterization_{form}"] = sets[form]
    for (first, second), (eps, sigma, form) in PAIRS.items():
        ids[f"pair_{first}{second}"] = w.parameter_set(
            parameterization=sets[form],
            slot_group=f"{form}.pair",
            subjects=[ids[f"site_{first}"], ids[f"site_{second}"]],
            slots={"epsilon_over_k": Quantity(eps, "K"), "sigma": Quantity(sigma, "nm")},
            origins=at(f"pair-{first}{second}"),
        )
    system = parameterization(w, "prism-system")
    ids["prism_system"] = system
    for (first, second), closure in CLOSURES.items():
        subjects = [ids[f"site_{first}"], ids[f"site_{second}"]]
        form = POTENTIAL_OF[(first, second)]
        w.subform_choice(
            parameterization=system,
            slot="prism_pair_model.potential",
            subjects=subjects,
            form=form,
            source_parameterization=sets[form],
            at="a.json#/choice",
        )
        w.subform_choice(
            parameterization=system,
            slot="prism_pair_model.closure",
            subjects=subjects,
            form=closure,
            at="a.json#/choice",
        )


def write_functional(w: CanonicalWriter, ids: dict[str, uuid.UUID]) -> None:
    for name in FLUIDS:
        ids[name] = w.kind("species", {"canonical_key": name, "label": name}, origins=at(name))
    p = parameterization(w, "saft-functional")
    ids["saft_functional_parameterization"] = p
    for name, (m, sigma, eps) in SAFT.items():
        w.parameter_set(
            parameterization=p,
            slot_group="white_bear_hard_sphere_functional.component",
            subjects=[ids[name]],
            slots={"sigma": Quantity(sigma, "angstrom")},
            origins=at(f"hs-{name}"),
        )
        w.parameter_set(
            parameterization=p,
            slot_group="saft_dispersion_functional.component",
            subjects=[ids[name]],
            slots={
                "m": m,
                "sigma": Quantity(sigma, "angstrom"),
                "epsilon_over_k": Quantity(eps, "K"),
            },
            origins=at(f"disp-{name}"),
        )
        w.parameter_set(
            parameterization=p,
            slot_group="pc_saft_chain_functional.component",
            subjects=[ids[name]],
            slots={"m": m, "sigma": Quantity(sigma, "angstrom")},
            origins=at(f"chain-{name}"),
        )
    for name, (eps, kappa) in ASSOCIATION.items():
        w.parameter_set(
            parameterization=p,
            slot_group="pc_saft_association_functional.component",
            subjects=[ids[name]],
            slots={"epsilon_hb_over_k": Quantity(eps, "K"), "kappa_hb": kappa},
            origins=at(f"assoc-{name}"),
        )
    w.parameter_set(
        parameterization=p,
        slot_group="saft_dispersion_functional.pair",
        subjects=[ids[FLUIDS[1]], ids[FLUIDS[0]]],
        slots={"k_ij": K_AB},
        origins=at("k-ab"),
    )
    assembly = w.kind(
        "model_assembly",
        {
            "key": "saft-functional",
            "revision": "1",
            "title": "A SAFT functional",
            "root": "saft_functional",
        },
        origins=at("assembly"),
    )
    ids["assembly"] = assembly
    for path, (slot, form) in CONTRIBUTIONS.items():
        w.kind(
            "assembly_choice",
            {"assembly": assembly, "path": path, "ordinal": 1, "slot": slot, "form": form},
            at="a.json#/choice",
        )


def write_world(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    write_sites(w, decl, ids)
    write_pairs(w, ids)
    write_functional(w, ids)


@pytest.fixture(scope="module")
def decl() -> Declaration:
    return real_declaration()


@pytest.fixture(scope="module")
def world(decl: Declaration, tmp_path_factory: pytest.TempPathFactory) -> Iterator[World]:
    ids: dict[str, uuid.UUID] = {}
    database = build(
        tmp_path_factory.mktemp("liquid-state"), lambda w: write_world(w, decl, ids), decl
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


def evaluate(bound, output: str, **arguments: np.ndarray) -> np.ndarray:  # noqa: ANN001
    return np.asarray(bound.evaluate(output, **arguments), dtype=float).reshape(-1)


def test_the_fixture_satisfies_every_invariant(conn: psycopg.Connection) -> None:
    assert failing(conn) == {}


# -- site types -------------------------------------------------------------------------------------------


def test_site_types_are_abstract_components_with_no_formula_no_molecule_and_no_producer(
    world: World, conn: psycopg.Connection
) -> None:
    for name in SITES:
        row = conn.execute(
            "SELECT p.kind::text, p.produced_by, e.provisional FROM tk.pseudo_component p JOIN tk.material_entity e ON e.id = p.id WHERE p.id = %s",
            (world.ids[f"site_{name}"],),
        ).fetchone()
        assert row == ("abstract_component", None, False)
        assert (
            scalar(
                conn,
                "SELECT count(*) FROM tk.composition WHERE entity = %s",
                world.ids[f"site_{name}"],
            )
            == 0
        )
    # the grouping of site types into molecules is no record: nothing here is a species or a mixture
    assert (
        scalar(
            conn,
            "SELECT count(*) FROM tk.species WHERE id = ANY(%s)",
            [world.ids[f"site_{n}"] for n in SITES],
        )
        == 0
    )
    assert scalar(conn, "SELECT count(*) FROM tk.defined_mixture") == 0


def test_the_density_of_each_site_type_is_a_number_per_volume_on_the_site_density_basis(
    world: World, conn: psycopg.Connection
) -> None:
    for name in SITES:
        row = conn.execute(
            "SELECT o.key, b.name, c.constant, e.canonical_key, q.name "
            "FROM ev.dataset_column c JOIN tk.observable o ON o.id = c.observable "
            "JOIN tk.composition_basis b ON b.id = c.composition_basis JOIN meta.quantity_type q ON q.id = o.quantity "
            "JOIN ev.dataset_component k ON k.id = c.component JOIN tk.material_entity e ON e.id = k.entity WHERE c.id = %s",
            (world.ids[f"density_{name}"],),
        ).fetchone()
        assert row[:2] == ("site_number_density", "site_number_density")
        assert row[2] == pytest.approx(
            DENSITY_PER_NM3[name] * 1e27, rel=1e-12
        )  # sites per cubic metre
        assert row[3:] == (f"site-{name}", "NumberDensity")


def test_a_site_density_is_not_a_molar_density_or_a_mole_fraction(decl: Declaration) -> None:
    w = writer(decl)
    dataset = w.kind(
        "dataset",
        {"carrier": CARRIER, "local_key": "d", "kind": "computed"},
        origins=at("d", "computed"),
    )
    with pytest.raises(ValidationError, match="cannot be converted"):
        w.kind(
            "dataset_column",
            {
                "dataset": dataset,
                "ordinal": 1,
                "role": "constraint",
                "observable": observable_id(decl, "site_number_density"),
                "constant": Quantity(1.0, "mol/m**3"),
            },
            at="a.json#/columns",
        )


# -- pair potentials as pair sets --------------------------------------------------------------------------


def test_the_pair_of_site_types_is_one_unordered_set_with_the_diagonal_allowed(
    world: World, conn: psycopg.Connection
) -> None:
    for form in ("lennard_jones_site_potential", "wca_site_potential"):
        (group,) = world.decl.forms[form].slot_groups
        assert [(s.name, s.type.text) for s in group.subjects] == [
            ("a", "pseudo_component"),
            ("b", "pseudo_component"),
        ]
        assert (group.transposition.rule, group.transposition.diagonal) == ("symmetric", "allowed")
    rows = conn.execute(
        'SELECT a, b, epsilon_over_k, sigma FROM param."lennard_jones_site_potential__pair"'
    ).fetchall()
    assert (
        len(rows) == 2
    )  # (A, A) and the pair asserted as (B, A), stored once in the canonical orientation
    by_pair = {frozenset((r[0], r[1])): r[2:] for r in rows}
    a, b = world.ids["site_A"], world.ids["site_B"]
    assert by_pair[frozenset((a,))] == pytest.approx((250.0, 0.40 * NM), rel=1e-12)
    assert by_pair[frozenset((a, b))] == pytest.approx((180.0, 0.43 * NM), rel=1e-12)
    assert all(str(r[0]) <= str(r[1]) for r in rows)


@pytest.mark.parametrize("pair", [("A", "A"), ("A", "B"), ("B", "A")])
def test_the_lennard_jones_potential_matches_numpy_in_either_order_of_the_pair(
    world: World, conn: psycopg.Connection, pair: tuple[str, str]
) -> None:
    bound = bind(
        world.decl,
        "lennard_jones_site_potential",
        source=DatabaseSource(
            conn, world.decl, [world.ids["parameterization_lennard_jones_site_potential"]]
        ),
        roles={"a": str(world.ids[f"site_{pair[0]}"]), "b": str(world.ids[f"site_{pair[1]}"])},
        cache=CACHE,
    )
    for T in TEMPERATURES:
        np.testing.assert_allclose(
            evaluate(bound, "u", r=R, T=np.full(R.shape, T)), potential(pair, R, T), rtol=1e-12
        )


def test_the_wca_potential_is_repulsive_continuous_and_matches_numpy(
    world: World, conn: psycopg.Connection
) -> None:
    bound = bind(
        world.decl,
        "wca_site_potential",
        source=DatabaseSource(conn, world.decl, [world.ids["parameterization_wca_site_potential"]]),
        roles={"a": str(world.ids["site_B"]), "b": str(world.ids["site_B"])},
        cache=CACHE,
    )
    r = np.array([0.36, 0.42, 0.50, 0.5163, 0.516, 0.52, 0.9]) * NM
    found = evaluate(bound, "u", r=r, T=np.full(r.shape, 450.0))
    np.testing.assert_allclose(
        found, wca(r, 450.0, 120.0, 0.46), rtol=1e-12, atol=1e-13
    )  # near the cut the terms cancel
    assert np.all(found >= 0.0) and found[-1] == 0.0
    cut = 2.0 ** (1.0 / 6.0) * 0.46  # nm: the minimum, where the shifted potential reaches zero
    edge = evaluate(bound, "u", r=np.array([cut * NM * (1 - 1e-9)]), T=np.array([450.0]))
    assert abs(edge[0]) < 1e-6


# -- the closure chosen per pair --------------------------------------------------------------------------------


def test_the_potential_and_the_closure_of_each_pair_are_choices_of_the_parameterisation(
    world: World, conn: psycopg.Connection
) -> None:
    rows = conn.execute(
        "SELECT c.subject_key, sl.qualified_name, f.name, c.source_parameterization FROM tk.subject_subform_choice c "
        "JOIN meta.subform_slot sl ON sl.id = c.slot JOIN meta.form f ON f.id = c.form WHERE c.parameterization = %s",
        (world.ids["prism_system"],),
    ).fetchall()
    assert len(rows) == 6
    found = {(key, slot): (form, source) for key, slot, form, source in rows}
    for (first, second), closure in CLOSURES.items():
        key = identity.canonical_encoding([world.ids[f"site_{first}"], world.ids[f"site_{second}"]])
        potential_form = POTENTIAL_OF[(first, second)]
        assert found[(key, "prism_pair_model.closure")] == (closure, None)
        assert found[(key, "prism_pair_model.potential")] == (
            potential_form,
            world.ids[f"parameterization_{potential_form}"],
        )
    # the closure is no parameter of a species: no slot of any species or site form holds one
    assert {
        c.name
        for group in world.decl.forms["lennard_jones_site_potential"].slot_groups
        for c in group.slots
    } == {"epsilon_over_k", "sigma"}


@pytest.mark.parametrize("pair", [("A", "A"), ("A", "B"), ("B", "B")])
def test_the_pair_model_applies_the_chosen_closure_to_the_chosen_potential(
    world: World, conn: psycopg.Connection, pair: tuple[str, str]
) -> None:
    bound = bind(
        world.decl,
        "prism_pair_model",
        source=DatabaseSource(conn, world.decl, [world.ids["prism_system"]]),
        roles={"a": str(world.ids[f"site_{pair[0]}"]), "b": str(world.ids[f"site_{pair[1]}"])},
        cache=CACHE,
    )
    for T in TEMPERATURES:
        u = potential(pair, R, T)
        found = evaluate(bound, "c", r=R, T=np.full(R.shape, T), gamma=GAMMA)
        np.testing.assert_allclose(found, CLOSURE_FUNCTION[CLOSURES[pair]](u, GAMMA), rtol=1e-11)


def test_the_two_closures_differ_for_one_potential_and_a_pair_without_a_choice_is_refused(
    world: World, conn: psycopg.Connection
) -> None:
    u = potential(("A", "B"), R, 450.0)
    assert np.max(np.abs(percus_yevick(u, GAMMA) - hypernetted_chain(u, GAMMA))) > 0.5
    stranger = uuid.uuid4()
    bound = bind(
        world.decl,
        "prism_pair_model",
        source=DatabaseSource(conn, world.decl, [world.ids["prism_system"]]),
        roles={"a": str(stranger), "b": str(stranger)},
        cache=CACHE,
    )
    with pytest.raises(EvaluationRefusal, match=r"prism_pair_model\.(potential|closure)"):
        bound.evaluate("c", r=R, T=np.full(R.shape, 450.0), gamma=GAMMA)


# -- the functional ----------------------------------------------------------------------------------------------


def test_the_functional_declares_its_structure_and_its_equation_is_external(world: World) -> None:
    decl = world.decl
    for name in ("saft_functional", *(form for _, form in CONTRIBUTIONS.values())):
        form = decl.forms[name]
        assert (form.status, form.completeness) == (
            "catalogued",
            "structure_declared_equation_external",
        )
    root = decl.forms["saft_functional"]
    assert {(s.name, s.multiplicity, s.per) for s in root.subforms} == {
        ("hard_sphere", "one", "model"),
        ("dispersion", "one", "model"),
        ("chain", "optional", "model"),
        ("association", "optional", "model"),
    }
    assert root.implements == "helmholtz_functional"
    assert {decl.forms[form].implements for _, form in CONTRIBUTIONS.values()} == {
        "helmholtz_functional_contribution"
    }


def test_a_catalogued_functional_has_no_expression_to_evaluate(
    world: World, conn: psycopg.Connection
) -> None:
    bound = bind(
        world.decl,
        "white_bear_hard_sphere_functional",
        source=DatabaseSource(conn, world.decl, [world.ids["saft_functional_parameterization"]]),
        sets={"components": [str(world.ids[n]) for n in FLUIDS]},
        cache=CACHE,
    )
    with pytest.raises(EvaluationRefusal):
        bound.evaluate(
            "f", T=np.array([300.0]), rho={str(world.ids[n]): np.array([1000.0]) for n in FLUIDS}
        )


def test_the_parameters_of_each_contribution_are_sets_of_its_components_and_pairs(
    world: World, conn: psycopg.Connection
) -> None:
    disp = {
        r[0]: r[1:]
        for r in conn.execute(
            'SELECT e.canonical_key, d.m, d.sigma, d.epsilon_over_k FROM param."saft_dispersion_functional__component" d JOIN tk.material_entity e ON e.id = d.i'
        ).fetchall()
    }
    for name, (m, sigma, eps) in SAFT.items():
        assert disp[name] == pytest.approx((m, sigma * 1e-10, eps), rel=1e-12)
    # only the associating component has an association set: the absence of a set is not a zero
    assert [
        r[0]
        for r in conn.execute(
            'SELECT e.canonical_key FROM param."pc_saft_association_functional__component" a JOIN tk.material_entity e ON e.id = a.i'
        ).fetchall()
    ] == ["fluid-b"]
    pair = conn.execute(
        'SELECT i, j, k_ij FROM param."saft_dispersion_functional__pair"'
    ).fetchall()
    assert (
        len(pair) == 1 and pair[0][2] == K_AB and str(pair[0][0]) < str(pair[0][1])
    )  # asserted (b, a), stored canonical


def test_the_assembly_names_the_form_of_each_contribution_slot(
    world: World, conn: psycopg.Connection
) -> None:
    rows = conn.execute(
        "SELECT c.path, sl.qualified_name, f.name FROM tk.assembly_choice c JOIN meta.subform_slot sl ON sl.id = c.slot "
        "JOIN meta.form f ON f.id = c.form WHERE c.assembly = %s ORDER BY c.path",
        (world.ids["assembly"],),
    ).fetchall()
    assert rows == sorted((path, slot, form) for path, (slot, form) in CONTRIBUTIONS.items())


# -- what the model refuses ----------------------------------------------------------------------------------------


def test_a_species_is_not_a_site_type_and_a_diameter_needs_a_length(decl: Declaration) -> None:
    w = writer(decl)
    species = w.kind("species", {"canonical_key": "s", "label": "s"}, origins=at("s"))
    site = w.kind(
        "pseudo_component",
        {"canonical_key": "x", "label": "x", "kind": "abstract_component"},
        origins=at("x", "conventional"),
    )
    p = parameterization(w, "p")
    slots = {"epsilon_over_k": Quantity(100.0, "K"), "sigma": Quantity(0.4, "nm")}
    with pytest.raises(
        ValidationError,
        match="is a species, and the role of `lennard_jones_site_potential.pair` is of kind `pseudo_component`",
    ):
        w.parameter_set(
            parameterization=p,
            slot_group="lennard_jones_site_potential.pair",
            subjects=[species, site],
            slots=slots,
            origins=at("a"),
        )
    with pytest.raises(ValidationError, match="cannot be converted"):
        w.parameter_set(
            parameterization=p,
            slot_group="lennard_jones_site_potential.pair",
            subjects=[site, site],
            slots={**slots, "sigma": Quantity(0.4, "K")},
            origins=at("b"),
        )
    with pytest.raises(ValidationError, match="not a member of enum `pseudo_component_kind`"):
        w.kind(
            "pseudo_component",
            {"canonical_key": "y", "label": "y", "kind": "site"},
            origins=at("y"),
        )


def test_a_closure_choice_needs_a_form_of_the_closure_contract_and_a_slot_chosen_per_pair(
    decl: Declaration,
) -> None:
    w = writer(decl)
    a = w.kind(
        "pseudo_component",
        {"canonical_key": "a", "label": "a", "kind": "abstract_component"},
        origins=at("a", "conventional"),
    )
    p = parameterization(w, "p")
    with pytest.raises(
        ValidationError,
        match=r"slot `prism_pair_model.closure` accepts contract `closure_relation`, and form `lennard_jones_site_potential` implements `site_pair_potential`",
    ):
        w.subform_choice(
            parameterization=p,
            slot="prism_pair_model.closure",
            subjects=[a, a],
            form="lennard_jones_site_potential",
        )
    with pytest.raises(ValidationError, match="is chosen per model"):
        w.subform_choice(
            parameterization=p,
            slot="saft_functional.hard_sphere",
            subjects=[a],
            form="white_bear_hard_sphere_functional",
        )


def test_the_verify_checks_flag_an_assembly_that_leaves_a_required_slot_undecided_or_chooses_the_wrong_contract(
    decl: Declaration, tmp_path: Path
) -> None:
    ids: dict[str, uuid.UUID] = {}

    def emit(w: CanonicalWriter) -> None:
        for key, choices in (
            ("no-dispersion", {k: v for k, v in CONTRIBUTIONS.items() if k != "dispersion"}),
            (
                "wrong-hard-sphere",
                {
                    **CONTRIBUTIONS,
                    "hard_sphere": ("saft_functional.hard_sphere", "prism_pair_model"),
                },
            ),
            (
                "optional-left-out",
                {k: v for k, v in CONTRIBUTIONS.items() if k not in ("chain", "association")},
            ),
        ):
            ids[key] = assembly = w.kind(
                "model_assembly",
                {"key": key, "revision": "1", "title": key, "root": "saft_functional"},
                origins=at(key),
            )
            for path, (slot, form) in choices.items():
                w.kind(
                    "assembly_choice",
                    {"assembly": assembly, "path": path, "ordinal": 1, "slot": slot, "form": form},
                    at="a.json#/choice",
                )

    database = build(tmp_path, emit, decl)
    try:
        with psycopg.connect(database.url, autocommit=True) as connection:
            found = failing(connection)
            assert found == {
                "model_assembly.single_slots_decided": 1,
                "assembly_choice.form_implements_slot_contract": 1,
            }
            flagged = run_check(connection, CHECKS["model_assembly.single_slots_decided"], shown=10)
            assert flagged.ids == [str(ids["no-dispersion"])]
    finally:
        database.remove()
