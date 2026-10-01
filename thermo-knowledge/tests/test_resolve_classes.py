# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Resolution by entity class (pipeline section 1): each class's rule on a fixture corpus, curated
decisions that state a class, and determinism and order independence of the whole."""

from __future__ import annotations

import itertools
import json
import random
from pathlib import Path

import pytest

from mapping_support import (
    ETHANOL,
    METHANE,
    WATER,
    WATER_INCHI,
    real_declaration,
    write_identity,
)
from test_resolve import by_entity, entity, env, resolved, species_of
from thermo_knowledge.build import build_database, discover
from thermo_knowledge.canonical.environment import Environment
from thermo_knowledge.mapping import claims
from thermo_knowledge.resolve import decisions as decision_module
from thermo_knowledge.resolve import engine, output, structure
from thermo_knowledge.resolve.command import phase1_sources, resolve_all
from thermo_knowledge.resolve.engine import ResolveError, mixture_key
from thermo_knowledge.testing import TestDatabase

type Tables = dict[str, list[dict[str, object]]]


def blend(
    key: str,
    *components: tuple[str, str],
    basis: str = "mole",
    definition: str = "by_definition",
    name: str | None = None,
    **extra: object,
) -> dict[str, object]:
    """A defined mixture of the species `components` (key, fraction text) of the same carrier."""
    return entity(
        key,
        ("name", name or key),
        scope="blends",
        **{
            "class": "defined_mixture",
            "basis": basis,
            "definition": definition,
            "components": [("species", component, fraction) for component, fraction in components],
            **extra,
        },
    )


def pure(carrier: str, canonical: Path, *extra: dict[str, object]) -> None:
    """A carrier with ethanol, water and methane as species and the given extra entities."""
    write_identity(
        canonical,
        carrier,
        [
            entity("eth", ("inchikey", ETHANOL)),
            entity("wat", ("inchikey", WATER)),
            entity("met", ("inchikey", METHANE)),
            *extra,
        ],
    )


def keys(tables: Tables) -> dict[object, str]:
    return {r["id"]: str(r["canonical_key"]) for r in tables["tk.material_entity"]}


def provisional_of(tables: Tables) -> dict[object, bool]:
    """Whether each material entity (of any refinement) is provisional."""
    return {r["id"]: bool(r["provisional"]) for r in tables["tk.material_entity"]}


def components_of(tables: Tables) -> dict[str, dict[str, float]]:
    """Each mixture's components by canonical key: fraction."""
    names = keys(tables)
    found: dict[str, dict[str, float]] = {}
    for row in tables.get("tk.mixture_component", []):
        found.setdefault(names[row["mixture"]], {})[names[row["component"]]] = float(row["value"])  # type: ignore[arg-type]
    return found


# -- defined mixtures --------------------------------------------------------------------------


def test_a_defined_mixture_of_resolved_species_resolves_by_composition(tmp_path: Path) -> None:
    pure("alpha", tmp_path / "canonical", blend("hydrous", ("eth", "0.6"), ("wat", "0.4")))
    tables = resolved(tmp_path)
    found = by_entity(tables)
    mixture = found[("alpha", "hydrous")]
    assert (mixture["status"], mixture["rule"]) == ("unique", "composition")
    assert mixture["entity_class"] == "defined_mixture"
    (row,) = tables["tk.defined_mixture"]
    assert row["id"] == mixture["target"]
    assert (row["definition"], row["mole_basis"], provisional_of(tables)[row["id"]]) == (
        "by_definition",
        True,
        False,
    )
    expected = mixture_key(True, [(ETHANOL, "0.6"), (WATER, "0.4")])
    assert keys(tables)[row["id"]] == expected
    assert expected == f'mixture:["mole","{ETHANOL}","0.6","{WATER}","0.4"]'
    assert components_of(tables) == {expected: {ETHANOL: 0.6, WATER: 0.4}}
    # the components are the resolved species themselves, and no species stands for the blend
    assert {r["canonical_key"] for r in species_of(tables).values()} == {ETHANOL, WATER, METHANE}
    assert not any(r["provisional"] for r in species_of(tables).values())
    assert not any(provisional_of(tables).values())


def test_the_same_composition_from_two_carriers_is_one_mixture_in_any_component_order(
    tmp_path: Path,
) -> None:
    canonical = tmp_path / "canonical"
    pure("alpha", canonical, blend("one", ("eth", "0.6"), ("wat", "0.4"), name="mixture one"))
    pure("beta", canonical, blend("other", ("wat", "0.4"), ("eth", "0.6"), name="the other"))
    tables = resolved(tmp_path)
    found = by_entity(tables)
    assert found[("alpha", "one")]["target"] == found[("beta", "other")]["target"]
    assert len(tables["tk.defined_mixture"]) == 1
    assert len(tables["tk.mixture_component"]) == 2, "the identical rows of both carriers are one"


def test_another_fraction_or_another_basis_is_another_mixture(tmp_path: Path) -> None:
    canonical = tmp_path / "canonical"
    pure(
        "alpha",
        canonical,
        blend("base", ("eth", "0.6"), ("wat", "0.4")),
        blend("shifted", ("eth", "0.61"), ("wat", "0.39")),
        blend("by_mass", ("eth", "0.6"), ("wat", "0.4"), basis="mass"),
    )
    tables = resolved(tmp_path)
    found = by_entity(tables)
    targets = {found[("alpha", key)]["target"] for key in ("base", "shifted", "by_mass")}
    assert len(targets) == 3
    names = keys(tables)
    assert names[found[("alpha", "by_mass")]["target"]] == mixture_key(
        False, [(ETHANOL, "0.6"), (WATER, "0.4")]
    )
    assert {r["mole_basis"] for r in tables["tk.defined_mixture"]} == {True, False}


def test_a_fraction_is_keyed_as_exact_decimal_text() -> None:
    assert claims.decimal_text(0.7812) == "0.7812"
    assert claims.decimal_text(0.5) == claims.decimal_text(0.50) == "0.5"
    assert claims.decimal_text(1e-05) == "0.00001"
    assert claims.decimal_text(0.357816784026318) == "0.357816784026318"
    assert claims.decimal_text(1) == claims.decimal_text(1.0) == "1"
    assert claims.decimal_text(-0.0) == "0"
    with pytest.raises(ValueError, match="not a finite number"):
        claims.decimal_text(float("nan"))
    assert mixture_key(True, [("b", "0.5"), ("a", "0.5")]) == mixture_key(
        True, [("a", "0.5"), ("b", "0.5")]
    )


def test_a_mixture_is_unique_only_when_every_component_is(tmp_path: Path) -> None:
    canonical = tmp_path / "canonical"
    write_identity(
        canonical,
        "alpha",
        [
            entity("eth", ("inchikey", ETHANOL)),
            entity("unknown", ("name", "unknown stuff")),
            entity("clash", ("inchikey", ETHANOL), ("inchi", WATER_INCHI)),
            blend("with_unknown", ("eth", "0.5"), ("unknown", "0.5")),
            blend("with_clash", ("eth", "0.5"), ("clash", "0.5")),
            blend("nothing_said"),
            blend("of_a_blend", ("eth", "0.5"), ("with_unknown", "0.5")),
        ],
    )
    tables = resolved(tmp_path)
    found, names = by_entity(tables), keys(tables)
    for key in ("with_unknown", "with_clash", "nothing_said", "of_a_blend"):
        row = found[("alpha", key)]
        assert (row["status"], row["rule"]) == ("unresolved", "provisional"), key
        assert names[row["target"]].startswith("provisional:")
    flags = provisional_of(tables)
    assert len(tables["tk.defined_mixture"]) == 4
    assert all(flags[r["id"]] for r in tables["tk.defined_mixture"])
    assert "tk.mixture_component" not in tables, "a provisional mixture has no components"
    assert {str(r["canonical_key"]) for r in species_of(tables).values() if r["provisional"]} == {
        names[found[("alpha", "unknown")]["target"]],
        names[found[("alpha", "clash")]["target"]],
    }, "only entities of class species have provisional species"
    report = json.loads((env(tmp_path).resolution_dir / "report.json").read_text())
    assert report["totals"]["provisional"]["defined_mixture"] == 4


def test_carriers_that_disagree_on_how_a_composition_is_fixed_leave_it_ambiguous(
    tmp_path: Path,
) -> None:
    canonical = tmp_path / "canonical"
    pure("alpha", canonical, blend("air", ("eth", "0.6"), ("wat", "0.4")))
    pure(
        "beta",
        canonical,
        blend("air", ("eth", "0.6"), ("wat", "0.4"), definition="by_measurement"),
    )
    tables = resolved(tmp_path)
    found = by_entity(tables)
    for carrier in ("alpha", "beta"):
        row = found[(carrier, "air")]
        assert (row["status"], row["rule"]) == ("ambiguous", "composition")
    candidates = {r["candidate"] for r in tables["tk.resolution_candidate"]}
    assert len(candidates) == 1
    (candidate,) = candidates
    assert keys(tables)[candidate] == mixture_key(True, [(ETHANOL, "0.6"), (WATER, "0.4")])
    assert found[("alpha", "air")]["target"] != candidate, "the target stays provisional"


def test_a_component_listed_twice_leaves_the_mixture_provisional() -> None:
    species = engine.SourceEntityRec(
        key=("a", "species", "eth"),
        carrier_key="a@pin",
        aggregation=None,
        polymorph=None,
        stated_charge=None,
        origins=(),
        assertions=(engine.Assertion("inchikey", ETHANOL, "inchikey"),),
    )
    twice = engine.SourceEntityRec(
        key=("a", "blends", "x"),
        carrier_key="a@pin",
        aggregation=None,
        polymorph=None,
        stated_charge=None,
        origins=(),
        assertions=(),
        entity_class="defined_mixture",
        mixture_definition="by_definition",
        mole_basis=True,
        components=((("a", "species", "eth"), "0.5"), (("a", "species", "eth"), "0.5")),
    )
    resolution = engine.resolve(
        [species, twice],
        decision_module.EMPTY,
        engine.FormulaScopes({}),
        structure.structure_key,
        engine.scheme_kinds(real_declaration()),
    )
    outcome = resolution.entities[("a", "blends", "x")]
    assert (outcome.status, outcome.reason) == ("unresolved", "a component is listed twice")


def test_two_fractions_for_one_component_are_competing_claims(tmp_path: Path) -> None:
    pure(
        "alpha",
        tmp_path / "canonical",
        blend("air", ("eth", "0.5"), ("eth", "0.6"), ("wat", "0.4")),
    )
    with pytest.raises(ResolveError, match="competing claims for the source entity .*fraction"):
        resolve_all(env(tmp_path), decl=real_declaration())


def test_a_mixture_needs_its_definition_and_basis() -> None:
    bare = engine.SourceEntityRec(
        key=("a", "blends", "x"),
        carrier_key="a@pin",
        aggregation=None,
        polymorph=None,
        stated_charge=None,
        origins=(),
        assertions=(),
        entity_class="defined_mixture",
    )
    with pytest.raises(ResolveError, match="states no definition and basis"):
        engine.resolve(
            [bare],
            decision_module.EMPTY,
            engine.FormulaScopes({}),
            structure.structure_key,
            engine.scheme_kinds(real_declaration()),
        )


# -- materials, polymer types and entities of undetermined class -------------------------------


def test_a_material_resolves_by_a_registry_identifier_and_otherwise_stays_provisional(
    tmp_path: Path,
) -> None:
    canonical = tmp_path / "canonical"
    write_identity(
        canonical,
        "alpha",
        [
            entity("zeolite", ("cas", "1318-02-1"), ("name", "zeolite"), **{"class": "material"}),
            entity("named_only", ("name", "a sorbent"), **{"class": "material"}),
            entity(
                "two_registries",
                ("cas", "1318-02-1"),
                ("pubchem_cid", "14793"),
                **{"class": "material"},
            ),
        ],
    )
    write_identity(
        canonical,
        "beta",
        [entity("Z", ("cas", "1318-02-1"), scope="hosts", **{"class": "material"})],
    )
    tables = resolved(tmp_path)
    found, names = by_entity(tables), keys(tables)
    wanted = engine.material_key("cas", "1318-02-1")
    assert found[("alpha", "zeolite")]["target"] == found[("beta", "Z")]["target"]
    for key in (("alpha", "zeolite"), ("beta", "Z")):
        assert (found[key]["status"], found[key]["rule"]) == ("unique", "registry")
        assert names[found[key]["target"]] == wanted
    row = found[("alpha", "named_only")]
    assert (row["status"], row["rule"]) == ("unresolved", "provisional")
    assert names[row["target"]].startswith("provisional:")
    ambiguous = found[("alpha", "two_registries")]
    assert (ambiguous["status"], ambiguous["rule"]) == ("ambiguous", "registry")
    materials = {r["id"]: r for r in tables["tk.material"]}
    assert materials[found[("beta", "Z")]["target"]]["registry_key"] == "cas:1318-02-1"
    assert provisional_of(tables)[row["target"]] is True
    assert "tk.species" not in tables, "no material is a species"


def test_a_polymer_type_is_provisional_unless_a_curated_decision_identifies_it(
    tmp_path: Path,
) -> None:
    canonical = tmp_path / "canonical"
    write_identity(
        canonical,
        "alpha",
        [
            entity("pe", ("name", "polyethylene"), **{"class": "polymer_type"}),
            entity("pp", ("name", "polypropylene"), **{"class": "polymer_type"}),
        ],
    )
    decisions = """
[[decision]]
action = "identify"
class = "polymer_type"
entity = { carrier = "alpha", scope = "species", key = "pp" }
canonical_key = "polymer:polypropylene"
reason = "Curated for the test."
"""
    tables = resolved(tmp_path, decisions)
    found, names = by_entity(tables), keys(tables)
    pe, pp = found[("alpha", "pe")], found[("alpha", "pp")]
    assert (pe["status"], pe["rule"]) == ("unresolved", "provisional")
    assert names[pe["target"]].startswith("provisional:")
    assert (pp["status"], pp["rule"]) == ("unique", "curated")
    assert names[pp["target"]] == "polymer:polypropylene"
    assert {r["id"]: provisional_of(tables)[r["id"]] for r in tables["tk.polymer_type"]} == {
        pe["target"]: True,
        pp["target"]: False,
    }
    assert "tk.species" not in tables


def test_an_undetermined_entity_gets_an_unclassified_entity_never_a_species(
    tmp_path: Path,
) -> None:
    canonical = tmp_path / "canonical"
    write_identity(
        canonical,
        "alpha",
        [
            entity("what", ("inchikey", ETHANOL), **{"class": "undetermined"}),
            entity("settled", ("name", "by hand"), **{"class": "undetermined"}),
        ],
    )
    decisions = """
[[decision]]
action = "identify"
class = "species"
entity = { carrier = "alpha", scope = "species", key = "settled" }
canonical_key = "made-up-key"
reason = "A curator states the class the source does not."
"""
    tables = resolved(tmp_path, decisions)
    found, names = by_entity(tables), keys(tables)
    what = found[("alpha", "what")]
    assert (what["status"], what["rule"]) == ("unresolved", "provisional")
    assert what["entity_class"] == "undetermined"
    assert names[what["target"]].startswith("provisional:")
    (unclassified,) = tables["tk.unclassified_entity"]
    assert unclassified["id"] == what["target"]
    assert provisional_of(tables)[unclassified["id"]] is True
    settled = found[("alpha", "settled")]
    assert (settled["status"], settled["rule"]) == ("unique", "curated")
    assert names[settled["target"]] == "made-up-key"
    assert [r["canonical_key"] for r in species_of(tables).values()] == ["made-up-key"]


def test_a_source_entity_records_the_class_its_source_states(tmp_path: Path) -> None:
    canonical = tmp_path / "canonical"
    pure("alpha", canonical, blend("air", ("eth", "0.5"), ("wat", "0.5")))
    tables = resolved(tmp_path)
    classes = {key[1]: row["entity_class"] for key, row in by_entity(tables).items()}
    assert classes == {
        "eth": "species",
        "wat": "species",
        "met": "species",
        "air": "defined_mixture",
    }
    report = json.loads((env(tmp_path).resolution_dir / "report.json").read_text())
    assert report["totals"]["class"] == {"defined_mixture": 1, "species": 3}
    assert report["totals"]["class_status_rule"] == {
        "defined_mixture/unique/composition": 1,
        "species/unique/structural": 3,
    }


# -- curated decisions state a class ------------------------------------------------------------


def test_a_decision_must_agree_with_the_class_the_source_states(tmp_path: Path) -> None:
    pure("alpha", tmp_path / "canonical", blend("air", ("eth", "0.5"), ("wat", "0.5")))
    decisions = """
[[decision]]
action = "identify"
class = "species"
entity = { carrier = "alpha", scope = "blends", key = "air" }
canonical_key = "an-air-species"
reason = "Wrongly states a species for a defined mixture."
"""
    with pytest.raises(ResolveError, match="names a `species`, and its source states the class"):
        resolved(tmp_path, decisions)


def test_a_curated_mixture_takes_the_components_its_entity_claims(tmp_path: Path) -> None:
    pure("alpha", tmp_path / "canonical", blend("air", ("eth", "0.5"), ("wat", "0.5")))
    decisions = """
[[decision]]
action = "identify"
class = "defined_mixture"
entity = { carrier = "alpha", scope = "blends", key = "air" }
canonical_key = "mixture:curated"
reason = "Named by hand."
"""
    tables = resolved(tmp_path, decisions)
    row = by_entity(tables)[("alpha", "air")]
    assert (row["status"], row["rule"]) == ("unique", "curated")
    assert keys(tables)[row["target"]] == "mixture:curated"
    assert components_of(tables) == {"mixture:curated": {ETHANOL: 0.5, WATER: 0.5}}


# -- order independence and the database ---------------------------------------------------------


def classified_corpus(canonical: Path) -> None:
    pure("alpha", canonical, blend("air", ("eth", "0.6"), ("wat", "0.4"), name="Air"))
    write_identity(
        canonical,
        "beta",
        [
            entity("e", ("inchikey", ETHANOL)),
            entity("w", ("name", "unidentified")),
            blend("mix", ("w", "0.5"), ("e", "0.5")),
            entity("host", ("cas", "1318-02-1"), **{"class": "material"}),
            entity("p", ("name", "a polymer"), **{"class": "polymer_type"}),
            entity("u", ("name", "who knows"), **{"class": "undetermined"}),
        ],
    )
    write_identity(
        canonical,
        "gamma",
        [
            entity("e", ("inchikey", ETHANOL)),
            entity("w", ("inchikey", WATER)),
            blend("same_air", ("e", "0.6"), ("w", "0.4"), name="the same air"),
        ],
    )


def test_every_order_of_the_entities_gives_one_resolution(tmp_path: Path) -> None:
    classified_corpus(tmp_path / "canonical")
    environment = env(tmp_path)
    read = output.read_claims(phase1_sources(environment))
    schemes = engine.scheme_kinds(real_declaration())

    def run(order: list[engine.SourceEntityRec]) -> engine.Resolution:
        return engine.resolve(
            order, decision_module.EMPTY, read.formula_scopes, structure.structure_key, schemes
        )

    expected = run(read.entities)
    orders = [
        list(reversed(read.entities)),
        *(random.Random(n).sample(read.entities, len(read.entities)) for n in range(6)),
        *itertools.islice(itertools.permutations(read.entities), 6),
    ]
    for order in orders:
        got = run(order)
        assert got.entities == expected.entities
        assert got.species == expected.species and got.forms == expected.forms
        assert got.mixtures == expected.mixtures and got.materials == expected.materials
        assert got.others == expected.others
    # the two carriers' air is one mixture; beta's blend has an unidentified component
    found = expected.entities
    assert (
        found[("alpha", "blends", "air")].target_key
        == found[("gamma", "blends", "same_air")].target_key
    )
    assert found[("beta", "blends", "mix")].status == "unresolved"
    # no provisional species except for entities of class species
    species_classes = {
        key: outcome.entity_class
        for key, outcome in found.items()
        if outcome.target_key in {k for k, s in expected.species.items() if s.provisional}
    }
    assert set(species_classes.values()) <= {"species", "species_form"}


def test_resolution_of_every_class_is_byte_identical_and_loads_with_every_constraint(
    tmp_path: Path,
) -> None:
    classified_corpus(tmp_path / "canonical")
    environment = env(tmp_path)
    resolve_all(environment, decl=real_declaration())
    first = {p.name: p.read_bytes() for p in environment.resolution_dir.iterdir()}
    resolve_all(environment, decl=real_declaration(), force=True)
    assert first == {p.name: p.read_bytes() for p in environment.resolution_dir.iterdir()}
    with TestDatabase() as database:
        counts = build_database(
            database.url,
            real_declaration(),
            discover(environment.canonical_dir),
            tree=environment.tree,
        ).tables
    assert counts["tk.defined_mixture"] == 2  # the air both carriers give, and beta's blend
    assert counts["tk.mixture_component"] == 2
    assert counts["tk.material"] == 1 and counts["tk.polymer_type"] == 1
    assert counts["tk.unclassified_entity"] == 1
    assert counts["tk.source_entity"] == 13


def test_the_resolution_report_counts_by_class_status_and_rule(tmp_path: Path) -> None:
    classified_corpus(tmp_path / "canonical")
    resolve_all(env(tmp_path), decl=real_declaration())
    report = json.loads((env(tmp_path).resolution_dir / "report.json").read_text())
    beta = report["carriers"]["beta"]
    assert beta["class"] == {
        "defined_mixture": 1,
        "material": 1,
        "polymer_type": 1,
        "species": 2,
        "undetermined": 1,
    }
    assert beta["class_status_rule"]["defined_mixture/unresolved/provisional"] == 1
    assert beta["class_status_rule"]["material/unique/registry"] == 1
    totals = report["totals"]
    assert totals["defined_mixtures"] == 1 and totals["materials"] == 1
    assert totals["provisional"] == {
        "defined_mixture": 1,
        "polymer_type": 1,
        "species": 1,
        "unclassified_entity": 1,
    }
    lines = output.report_table(report)
    assert any("defined_mixture/unique/composition" in line for line in lines)
    assert Environment  # the report is also what `tk resolve` prints
