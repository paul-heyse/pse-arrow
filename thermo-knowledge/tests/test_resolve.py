# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Identity resolution: each rule on a fixture corpus of fake carriers, determinism and order
independence, the decisions file and the output."""

from __future__ import annotations

import itertools
import json
import random
from pathlib import Path

import pytest
from mapping_support import (
    ETHANOL,
    ETHANOL_INCHI,
    ETHANOL_SMILES,
    METHANE,
    WATER,
    WATER_INCHI,
    real_declaration,
    rows,
    write_identity,
)

from thermo_knowledge.canonical import store
from thermo_knowledge.canonical.environment import Environment
from thermo_knowledge.canonical.store import CanonicalError
from thermo_knowledge.resolve import decisions as decision_module
from thermo_knowledge.resolve import engine, output, structure
from thermo_knowledge.resolve.command import phase1_sources, resolve_all
from thermo_knowledge.resolve.decisions import DecisionError
from thermo_knowledge.resolve.engine import ResolveError
from thermo_knowledge.build import build_database, discover
from thermo_knowledge.testing import TestDatabase


def entity(
    key: str, *assertions: tuple[str, str], scope: str = "species", **extra: object
) -> dict[str, object]:
    return {"scope": scope, "key": key, "assertions": list(assertions), **extra}


def corpus(canonical: Path) -> None:
    """Three fake carriers describing ethanol, water and a few things nobody can identify."""
    write_identity(
        canonical,
        "alpha",
        [
            entity("ethanol-a", ("inchikey", ETHANOL), ("cas", "64-17-5"), ("name", "Ethanol")),
            entity("water-a", ("inchi", WATER_INCHI), ("cas", "7732-18-5")),
            entity("mystery-a", ("name", "mystery")),
        ],
    )
    write_identity(
        canonical,
        "beta",
        [
            entity("EtOH", ("smiles", ETHANOL_SMILES), ("name", "ethyl alcohol")),
            entity("H2O", ("cas", "7732-18-5")),
            entity("ethanol-by-cas", ("cas", "64-17-5")),
        ],
    )
    write_identity(
        canonical, "gamma", [entity("ethanol", ("inchikey", ETHANOL), ("name", "Ethanol"))]
    )


def env(tmp_path: Path, decisions: str | None = None) -> Environment:
    path = None
    if decisions is not None:
        path = tmp_path / "decisions.toml"
        path.write_text(decisions)
    return Environment(canonical_dir=tmp_path / "canonical", tree=tmp_path, decisions_path=path)


def resolved(tmp_path: Path, decisions: str | None = None) -> dict[str, list[dict[str, object]]]:
    """Resolve and return every output table's rows by name."""
    environment = env(tmp_path, decisions)
    resolve_all(environment, decl=real_declaration())
    manifest = store.read_manifest(environment.resolution_dir)
    return {
        name: rows(environment.resolution_dir / record.file)
        for name, record in manifest.tables.items()
    }


def by_entity(
    tables: dict[str, list[dict[str, object]]],
) -> dict[tuple[str, str], dict[str, object]]:
    carriers = {r["id"]: r["manifest_id"] for r in tables["prov.carrier"]}
    return {(carriers[r["carrier"]], r["local_key"]): r for r in tables["tk.source_entity"]}  # type: ignore[index]


def species_of(tables: dict[str, list[dict[str, object]]]) -> dict[object, dict[str, object]]:
    material = {r["id"]: r for r in tables["tk.material_entity"]}
    return {r["id"]: {**material[r["id"]], **r} for r in tables["tk.species"]}


# -- structure identifiers ---------------------------------------------------------------------


def test_structure_keys_come_from_rdkit_and_are_standard_only() -> None:
    assert structure.structure_key("inchikey", ETHANOL).inchikey == ETHANOL
    assert structure.structure_key("inchi", ETHANOL_INCHI) == structure.StructureKey(ETHANOL, 0)
    assert structure.structure_key("smiles", ETHANOL_SMILES).inchikey == ETHANOL
    assert structure.structure_key("smiles", "[NH4+]").charge == 1
    assert structure.structure_key("inchikey", "LFQSCWFLJHTTHZ-UHFFFAOYNA-N").inchikey is None
    assert "not a standard InChI" in (
        structure.structure_key("inchi", "InChI=1/CH4/h1H4").problem or ""
    )
    assert structure.structure_key("smiles", "?").problem == "RDKit cannot parse the SMILES"
    assert structure.structure_key("smiles", "?").inchikey is None


# -- the rules ---------------------------------------------------------------------------------


def test_agreeing_inchikeys_merge_whatever_notation_carries_them(tmp_path: Path) -> None:
    corpus(tmp_path / "canonical")
    tables = resolved(tmp_path)
    found = by_entity(tables)
    species = species_of(tables)
    targets = {
        found[key]["target"]
        for key in (("alpha", "ethanol-a"), ("beta", "EtOH"), ("gamma", "ethanol"))
    }
    assert len(targets) == 1
    ethanol = species[targets.pop()]
    assert ethanol["canonical_key"] == ETHANOL and ethanol["inchikey"] == ETHANOL
    assert ethanol["provisional"] is False and ethanol["charge"] == 0.0
    for key in (("alpha", "ethanol-a"), ("beta", "EtOH"), ("gamma", "ethanol")):
        assert (found[key]["status"], found[key]["rule"]) == ("unique", "structural")
    assert species[found[("alpha", "water-a")]["target"]]["canonical_key"] == WATER


def test_a_registry_number_the_corpus_maps_to_one_structure_resolves(tmp_path: Path) -> None:
    corpus(tmp_path / "canonical")
    tables = resolved(tmp_path)
    found, species = by_entity(tables), species_of(tables)
    for key, structure_key in ((("beta", "H2O"), WATER), (("beta", "ethanol-by-cas"), ETHANOL)):
        assert (found[key]["status"], found[key]["rule"]) == ("unique", "registry")
        assert species[found[key]["target"]]["canonical_key"] == structure_key


def test_a_cas_the_corpus_maps_to_two_structures_is_ambiguous_and_provisional(
    tmp_path: Path,
) -> None:
    canonical = tmp_path / "canonical"
    write_identity(canonical, "one", [entity("a", ("cas", "50-00-0"), ("inchikey", ETHANOL))])
    write_identity(canonical, "two", [entity("b", ("cas", "50-00-0"), ("inchikey", WATER))])
    write_identity(canonical, "three", [entity("c", ("cas", "50-00-0"))])
    write_identity(canonical, "four", [entity("d", ("cas", "50-00-0"), ("name", "four"))])
    tables = resolved(tmp_path)
    found, species = by_entity(tables), species_of(tables)
    assert found[("one", "a")]["status"] == found[("two", "b")]["status"] == "unique"
    for key in (("three", "c"), ("four", "d")):
        assert (found[key]["status"], found[key]["rule"]) == ("ambiguous", "registry")
        assert species[found[key]["target"]]["provisional"] is True
    candidates = {
        (r["source_entity"], species[r["candidate"]]["canonical_key"])
        for r in tables["tk.resolution_candidate"]
    }
    assert candidates == {
        (found[key]["id"], structure_key)
        for key in (("three", "c"), ("four", "d"))
        for structure_key in (ETHANOL, WATER)
    }
    report = json.loads((env(tmp_path).resolution_dir / "report.json").read_text())
    assert report["registry_conflicts"] == [
        {"scheme": "cas", "value": "50-00-0", "structures": sorted([ETHANOL, WATER])}
    ]
    assert {item["entity"]["key"] for item in report["ambiguous"]} == {"c", "d"}
    assert report["totals"]["status"] == {"ambiguous": 2, "unique": 2}


def test_conflicting_structures_in_one_entity_are_ambiguous(tmp_path: Path) -> None:
    write_identity(
        tmp_path / "canonical",
        "alpha",
        [entity("x", ("inchikey", ETHANOL), ("inchi", WATER_INCHI))],
    )
    tables = resolved(tmp_path)
    (row,) = tables["tk.source_entity"]
    assert (row["status"], row["rule"]) == ("ambiguous", "structural")
    assert len(tables["tk.resolution_candidate"]) == 2


def test_a_curated_decision_overrides_the_rules(tmp_path: Path) -> None:
    corpus(tmp_path / "canonical")
    decisions = f"""
[[decision]]
action = "identify"
class = "species"
entity = {{ carrier = "gamma", scope = "species", key = "ethanol" }}
canonical_key = "{METHANE}"
reason = "The source's InChIKey is a known error."

[[decision]]
action = "identify"
class = "species"
entity = {{ carrier = "alpha", scope = "species", key = "mystery-a" }}
canonical_key = "made-up-key"
charge = -1
reason = "Curated by hand."

[[decision]]
action = "reject"
entity = {{ carrier = "beta", scope = "species", key = "H2O" }}
reason = "The CAS number is a typo in this source."
"""
    tables = resolved(tmp_path, decisions)
    found, species = by_entity(tables), species_of(tables)
    gamma = found[("gamma", "ethanol")]
    assert (gamma["status"], gamma["rule"]) == ("unique", "curated")
    assert species[gamma["target"]]["canonical_key"] == METHANE
    mystery = found[("alpha", "mystery-a")]
    assert species[mystery["target"]]["canonical_key"] == "made-up-key"
    assert (
        species[mystery["target"]]["charge"] == -1.0
        and species[mystery["target"]]["inchikey"] is None
    )
    rejected = found[("beta", "H2O")]
    assert (rejected["status"], rejected["rule"]) == ("rejected", "curated")
    assert species[rejected["target"]]["provisional"] is True
    assert found[("alpha", "ethanol-a")]["status"] == "unique"


def test_a_distinct_decision_unmerges_what_a_rule_would_merge(tmp_path: Path) -> None:
    corpus(tmp_path / "canonical")
    decisions = """
[[decision]]
action = "distinct"
entities = [
  { carrier = "gamma", scope = "species", key = "ethanol" },
  { carrier = "alpha", scope = "species", key = "ethanol-a" },
]
reason = "These are different samples."
"""
    tables = resolved(tmp_path, decisions)
    found = by_entity(tables)
    assert (
        found[("gamma", "ethanol")]["status"]
        == found[("alpha", "ethanol-a")]["status"]
        == "ambiguous"
    )
    assert found[("beta", "EtOH")]["status"] == "unique"
    report = json.loads((env(tmp_path).resolution_dir / "report.json").read_text())
    assert (
        "distinct"
        in next(i for i in report["ambiguous"] if i["entity"]["key"] == "ethanol")["reason"]
    )


def test_a_decision_about_an_unknown_entity_is_reported_not_applied(tmp_path: Path) -> None:
    corpus(tmp_path / "canonical")
    resolved(
        tmp_path,
        """
[[decision]]
action = "reject"
entity = { carrier = "alpha", scope = "species", key = "no-such-entity" }
reason = "A typo."
""",
    )
    report = json.loads((env(tmp_path).resolution_dir / "report.json").read_text())
    assert report["unmatched_decisions"] == [
        {"carrier": "alpha", "scope": "species", "key": "no-such-entity"}
    ]


def test_a_formula_identified_scope_resolves_by_formula_and_charge(tmp_path: Path) -> None:
    canonical = tmp_path / "canonical"
    write_identity(
        canonical,
        "alpha",
        [
            entity("water", ("formula", "H2O"), scope="ions", charge=0),
            entity("hydroxide", ("formula", "OH"), scope="ions", charge=-1),
            entity("free", ("formula", "H2O"), scope="other", charge=0),
        ],
        formula_scopes=(("ions", None),),
    )
    write_identity(
        canonical,
        "beta",
        [
            entity("H2O(aq)", ("formula", "H2O"), ("name", "water"), scope="ions", charge=0),
            entity("OH-", ("formula", "OH"), scope="ions", charge=0),
        ],
        formula_scopes=(("ions", "aqueous"),),
    )
    tables = resolved(tmp_path)
    found, species = by_entity(tables), species_of(tables)
    water_a, water_b = found[("alpha", "water")], found[("beta", "H2O(aq)")]
    assert (water_a["status"], water_a["rule"]) == ("unique", "formula_scope")
    assert species[water_a["target"]]["canonical_key"] == "formula:H2O:+0"
    assert species[water_b["target"]]["canonical_key"] == "formula:H2O:+0:aqueous", (
        "a discriminator is part of the key"
    )
    assert species[found[("alpha", "hydroxide")]["target"]]["canonical_key"] == "formula:OH:-1"
    assert species[found[("alpha", "hydroxide")]["target"]]["charge"] == -1.0
    assert species[found[("beta", "OH-")]["target"]]["canonical_key"] == "formula:OH:+0:aqueous"
    free = found[("alpha", "free")]
    assert (free["status"], free["rule"]) == ("unresolved", "provisional"), (
        "a formula alone identifies nothing outside a declared scope"
    )


def test_everything_else_is_provisional_and_joins_nothing(tmp_path: Path) -> None:
    corpus(tmp_path / "canonical")
    tables = resolved(tmp_path)
    found, species = by_entity(tables), species_of(tables)
    mystery = found[("alpha", "mystery-a")]
    assert (mystery["status"], mystery["rule"]) == ("unresolved", "provisional")
    provisional = species[mystery["target"]]
    assert provisional["provisional"] is True and provisional["label"] == "mystery-a"
    assert provisional["canonical_key"].startswith("provisional:")
    assert (
        "alpha@0123456789ab" in provisional["canonical_key"]
        and "mystery-a" in provisional["canonical_key"]
    )
    assert {r["target"] for r in found.values() if r["status"] == "unresolved"} == {
        mystery["target"]
    }


def test_an_entity_with_an_aggregation_resolves_to_a_species_form(tmp_path: Path) -> None:
    write_identity(
        tmp_path / "canonical",
        "alpha",
        [
            entity("ethanol(g)", ("inchikey", ETHANOL), aggregation="gas"),
            entity("ethanol(l)", ("inchikey", ETHANOL), aggregation="liquid"),
            entity("ethanol", ("inchikey", ETHANOL)),
        ],
    )
    tables = resolved(tmp_path)
    found, species = by_entity(tables), species_of(tables)
    assert len(species) == 1
    forms = {r["id"]: r for r in tables["tk.species_form"]}
    assert len(forms) == 2
    gas = found[("alpha", "ethanol(g)")]
    assert gas["target"] in forms and forms[gas["target"]]["species"] in species
    assert found[("alpha", "ethanol")]["target"] in species
    form_keys = {r["id"]: r["canonical_key"] for r in tables["tk.material_entity"]}
    assert form_keys[gas["target"]] == json.dumps([ETHANOL, "gas"], separators=(",", ":"))


def test_the_label_is_chosen_deterministically_and_is_never_identity(tmp_path: Path) -> None:
    corpus(tmp_path / "canonical")
    tables = resolved(tmp_path)
    found, species = by_entity(tables), species_of(tables)
    assert species[found[("alpha", "ethanol-a")]["target"]]["label"] == "Ethanol", (
        "the name most entities assert"
    )
    assert species[found[("alpha", "water-a")]["target"]]["label"] == "H2O", (
        "no name: the smallest local key"
    )

    def rec(key: str, *names: str) -> engine.SourceEntityRec:
        return engine.SourceEntityRec(
            ("c", "s", key),
            "c@p",
            None,
            None,
            None,
            (),
            tuple(engine.Assertion("name", n, "name") for n in names)
            + (engine.Assertion("formula", "Z", "f"),),
        )

    assert engine.label([rec("a", "beta", "alpha"), rec("b", "gamma")]) == "alpha"
    assert engine.label([rec("a", "beta"), rec("b", "beta", "zeta")]) == "beta"
    assert engine.label([rec("zeta", "zeta", "alpha")]) == "zeta", (
        "a local key outranks code-point order"
    )
    assert engine.label([rec("a")]) == "Z"


# -- determinism and order ---------------------------------------------------------------------


def test_resolution_twice_is_byte_identical(tmp_path: Path) -> None:
    corpus(tmp_path / "canonical")
    environment = env(tmp_path)
    resolve_all(environment, decl=real_declaration())
    first = {p.name: p.read_bytes() for p in environment.resolution_dir.iterdir()}
    outcome = resolve_all(environment, decl=real_declaration())
    assert outcome.status == "current"
    resolve_all(environment, decl=real_declaration(), force=True)
    second = {p.name: p.read_bytes() for p in environment.resolution_dir.iterdir()}
    assert first == second


def test_nothing_resolves_by_first_match(tmp_path: Path) -> None:
    """Every order of the entities, and so of the carriers, gives the same resolution."""
    canonical = tmp_path / "canonical"
    write_identity(canonical, "one", [entity("a", ("cas", "50-00-0"), ("inchikey", ETHANOL))])
    write_identity(canonical, "two", [entity("b", ("cas", "50-00-0"), ("inchikey", WATER))])
    write_identity(
        canonical, "three", [entity("c", ("cas", "50-00-0")), entity("d", ("cas", "7-7-7"))]
    )
    write_identity(canonical, "four", [entity("e", ("cas", "50-00-0"), ("inchi", WATER_INCHI))])
    environment = env(tmp_path)
    read = output.read_claims(phase1_sources(environment))
    schemes = engine.scheme_kinds(real_declaration())
    expected = engine.resolve(
        read.entities, decision_module.EMPTY, read.formula_scopes, structure.structure_key, schemes
    )
    orders = [
        list(reversed(read.entities)),
        *(random.Random(n).sample(read.entities, len(read.entities)) for n in range(6)),
    ]
    orders.extend(itertools.islice(itertools.permutations(read.entities), 6))
    for order in orders:
        got = engine.resolve(
            order, decision_module.EMPTY, read.formula_scopes, structure.structure_key, schemes
        )
        assert got.entities == expected.entities and got.species == expected.species
        assert got.registry_conflicts == expected.registry_conflicts
    statuses = {key[2]: r.status for key, r in expected.entities.items()}
    assert statuses == {
        "a": "unique",
        "b": "unique",
        "c": "ambiguous",
        "d": "unresolved",
        "e": "unique",
    }


# -- claims, decisions and output --------------------------------------------------------------


def test_contradicting_claims_about_one_entity_are_refused(tmp_path: Path) -> None:
    write_identity(
        tmp_path / "canonical",
        "alpha",
        [
            entity("x", ("name", "x"), charge=1),
            entity("x", ("name", "x"), charge=-1, locator="data/species.json#/second"),
        ],
    )
    with pytest.raises(
        ResolveError, match="competing claims for the source entity .*stated_charge"
    ):
        resolve_all(env(tmp_path), decl=real_declaration())


def test_resolution_needs_phase_one_output(tmp_path: Path) -> None:
    with pytest.raises(CanonicalError, match="tk map <id> --phase identity"):
        resolve_all(env(tmp_path), decl=real_declaration())


def test_decisions_file_format() -> None:
    assert decision_module.parse("# only a comment\n", file="x") == decision_module.EMPTY
    parsed = decision_module.parse(
        """
[[decision]]
action = "identify"
class = "species"
entity = { carrier = "a", scope = "s", key = "k" }
canonical_key = "K"
reason = "r"
""",
        file="x",
    )
    assert parsed.identify[("a", "s", "k")] == decision_module.Identify("K", "species", 0, "r")
    for text, message in (
        (
            '[[decision]]\naction = "reject"\nentity = { carrier = "a", scope = "s", key = "k" }\nreason = " "\n',
            "carries a reason",
        ),
        (
            '[[decision]]\naction = "identify"\nclass = "species"\nentity = { carrier = "a", scope = "s", key = "k" }\nreason = "r"\n',
            "states the `canonical_key`",
        ),
        (
            '[[decision]]\naction = "identify"\nentity = { carrier = "a", scope = "s", key = "k" }\ncanonical_key = "K"\nreason = "r"\n',
            "states the `class` of the entity it names",
        ),
        (
            '[[decision]]\naction = "identify"\nclass = "pseudo_component"\nentity = { carrier = "a", scope = "s", key = "k" }\ncanonical_key = "K"\nreason = "r"\n',
            "states the `class` of the entity it names",
        ),
        (
            '[[decision]]\naction = "identify"\nclass = "material"\ncharge = 1\nentity = { carrier = "a", scope = "s", key = "k" }\ncanonical_key = "K"\nreason = "r"\n',
            "`charge` belongs to a species",
        ),
        (
            '[[decision]]\naction = "reject"\nclass = "species"\nentity = { carrier = "a", scope = "s", key = "k" }\nreason = "r"\n',
            "belong to `identify`",
        ),
        (
            '[[decision]]\naction = "distinct"\nentities = [{ carrier = "a", scope = "s", key = "k" }]\nreason = "r"\n',
            "exactly two",
        ),
        ('[[decision]]\naction = "maybe"\nreason = "r"\n', "action"),
        (
            '[[decision]]\naction = "reject"\nreason = "r"\nentity = { carrier = "a", scope = "s", key = "k" }\nextra = 1\n',
            "extra",
        ),
    ):
        with pytest.raises(DecisionError, match=message):
            decision_module.parse(text, file="x")
    twice = (
        '[[decision]]\naction = "reject"\nentity = { carrier = "a", scope = "s", key = "k" }\nreason = "r"\n'
    ) * 2
    with pytest.raises(DecisionError, match="already identifies or rejects"):
        decision_module.parse(twice, file="x")


def test_the_committed_decisions_file_has_no_decisions() -> None:
    decided, data = decision_module.load()
    assert decided == decision_module.EMPTY
    assert data.startswith(b"# SPDX-License-Identifier")


def test_the_resolution_output_loads_with_every_constraint_satisfied(tmp_path: Path) -> None:
    corpus(tmp_path / "canonical")
    write_identity(
        tmp_path / "canonical",
        "delta",
        [
            entity("g", ("inchikey", ETHANOL), aggregation="gas"),
            entity("amb", ("inchikey", ETHANOL), ("inchi", WATER_INCHI)),
        ],
    )
    environment = env(tmp_path)
    resolve_all(environment, decl=real_declaration())
    with TestDatabase() as database:
        counts = build_database(
            database.url,
            real_declaration(),
            discover(environment.canonical_dir),
            tree=environment.tree,
        ).tables
    assert counts["tk.source_entity"] == 9 and counts["tk.species_form"] == 1
    assert counts["tk.resolution_candidate"] == 2


def test_the_schemes_that_are_structural_or_registries_come_from_the_declaration(
    tmp_path: Path,
) -> None:
    kinds = engine.scheme_kinds(real_declaration())
    assert kinds.structural == {"inchikey", "inchi", "smiles"} and kinds.registry == {
        "cas",
        "pubchem_cid",
    }
    canonical = tmp_path / "canonical"
    write_identity(canonical, "alpha", [entity("a", ("cas", "64-17-5"), ("inchikey", ETHANOL))])
    write_identity(canonical, "beta", [entity("b", ("cas", "64-17-5"))])
    read = output.read_claims(phase1_sources(env(tmp_path)))
    run = lambda schemes: engine.resolve(  # noqa: E731
        read.entities, decision_module.EMPTY, read.formula_scopes, structure.structure_key, schemes
    ).entities[("beta", "species", "b")]
    assert (run(kinds).status, run(kinds).rule) == ("unique", "registry")
    assert run(engine.SchemeKinds(kinds.structural, frozenset())).status == "unresolved", (
        "a scheme is a registry because the declaration says so, not because of its name"
    )
    assert run(engine.SchemeKinds(frozenset(), kinds.registry)).status == "unresolved"


def test_the_target_is_provisional_exactly_when_the_status_is_ambiguous_unresolved_or_rejected(
    tmp_path: Path,
) -> None:
    decl = real_declaration()
    (requirement,) = [
        r
        for r in decl.kinds["source_entity"].requires
        if r.name == "provisional_target_matches_status"
    ]
    assert "ambiguous, unresolved or rejected" in requirement.doc
    canonical = tmp_path / "canonical"
    write_identity(
        canonical,
        "alpha",
        [
            entity("unique", ("inchikey", ETHANOL)),
            entity("ambiguous", ("inchikey", ETHANOL), ("inchi", WATER_INCHI)),
            entity("unresolved", ("name", "nothing known")),
            entity("rejected", ("inchikey", WATER)),
        ],
    )
    tables = resolved(
        tmp_path,
        """
[[decision]]
action = "reject"
entity = { carrier = "alpha", scope = "species", key = "rejected" }
reason = "The source's InChIKey is wrong."
""",
    )
    found, species = by_entity(tables), species_of(tables)
    statuses = {key[1]: row["status"] for key, row in found.items()}
    assert statuses == {
        "unique": "unique",
        "ambiguous": "ambiguous",
        "unresolved": "unresolved",
        "rejected": "rejected",
    }
    for key, row in found.items():
        provisional = species[row["target"]]["provisional"]
        assert provisional is (row["status"] in ("ambiguous", "unresolved", "rejected")), key
    rejected = found[("alpha", "rejected")]
    assert rejected["rule"] == "curated" and species[rejected["target"]][
        "canonical_key"
    ].startswith("provisional:")
