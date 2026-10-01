# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The CoolProp mapping against the real staged data: fluid identities and the saturation-pressure
ancillaries."""

from __future__ import annotations

import json
import re
import tomllib
from collections import Counter
from collections.abc import Iterator
from dataclasses import dataclass
from pathlib import Path

import pyarrow.parquet as pq
import pytest
from rdkit import Chem
from rdkit.Chem import inchi as rd_inchi

from mapping_support import real_declaration, rows
from thermo_knowledge import config
from thermo_knowledge.acquire.lock import read_lock
from thermo_knowledge.acquire.manifest import default_lock_path
from thermo_knowledge.build import build_database, discover
from thermo_knowledge.canonical.environment import Environment
from thermo_knowledge.mapping import claims, runner
from thermo_knowledge.resolve.command import resolve_all
from thermo_knowledge.testing import TestDatabase

SATURATION = "vapor_pressure_exp_series_tau__pure"
PSEUDO_PURE_FILE = re.compile(r".+\.[Pp][Pp][Ff]")
"""How the source marks a pseudo-pure fluid: its INFO/CAS names a REFPROP blend file."""


def _staged_directory() -> Path | None:
    entry = read_lock(default_lock_path()).get("coolprop")
    if entry is None or entry.pin is None:
        return None
    directory = config.staged_dir() / "coolprop" / entry.pin
    raw = config.raw_dir() / "coolprop" / entry.pin
    return directory if (directory / "manifest.json").is_file() and raw.is_dir() else None


STAGED = _staged_directory()
pytestmark = pytest.mark.skipif(
    STAGED is None,
    reason="the CoolProp source is not acquired and staged here (run `just tk-acquire coolprop` "
    "and `just tk-read coolprop`)",
)


def staged(table: str) -> list[dict[str, object]]:
    assert STAGED is not None
    return rows(STAGED / f"{table}.parquet")


@dataclass(frozen=True)
class Run:
    env: Environment
    coverage: runner.coverage.Coverage
    report: dict[str, object]

    def table(self, name: str, *, resolution: bool = False) -> list[dict[str, object]]:
        directory = self.env.resolution_dir if resolution else self.env.canonical_dir / "coolprop"
        return rows(directory / f"{name}.parquet")


@pytest.fixture(scope="module")
def run(tmp_path_factory: pytest.TempPathFactory) -> Iterator[Run]:
    env = Environment(canonical_dir=tmp_path_factory.mktemp("canonical"))
    decl = real_declaration()
    assert runner.run_identity(env, "coolprop", decl=decl).status == "mapped"
    resolve_all(env, decl=decl)
    outcome = runner.run_records(env, "coolprop", decl=decl)
    assert outcome.coverage is not None
    report = json.loads((env.resolution_dir / "report.json").read_text())
    yield Run(env, outcome.coverage, report)


# -- fluid identities --------------------------------------------------------------------------

CAS = re.compile(r"^[0-9]{2,7}-[0-9]{2}-[0-9]$")
PLACEHOLDERS = {"REFPROP_NAME": "N/A", "SMILES": "?"}


def expected_assertions(row: dict[str, object]) -> set[tuple[str, str]]:
    """What a fluid row asserts, computed from the staged table and the documented placeholders."""
    found: set[tuple[str, str]] = {("name", str(row["NAME"]))}
    cas = str(row["CAS"])
    found.add(("cas" if CAS.match(cas) else "source_local", cas))
    for column, scheme in (
        ("REFPROP_NAME", "name"),
        ("FORMULA", "formula"),
        ("INCHI_STRING", "inchi"),
        ("INCHI_KEY", "inchikey"),
        ("SMILES", "smiles"),
    ):
        value = row[column]
        if value is not None and value != PLACEHOLDERS.get(column):
            found.add((scheme, str(value)))
    found.update(("name", str(alias)) for alias in row["ALIASES"] or [])  # type: ignore[attr-defined]
    return found


def test_there_is_one_source_entity_per_fluid(run: Run) -> None:
    fluids = staged("fluids")
    entities = run.table("tk.source_entity", resolution=True)
    assert len(entities) == len(fluids) == 136
    assert {r["local_key"] for r in entities} == {r["NAME"] for r in fluids}
    assert {r["scope"] for r in entities} == {"fluids"}
    assert run.report["totals"]["source_entities"] == 136  # type: ignore[index]


def test_every_identifier_is_asserted_and_no_placeholder_is(run: Run) -> None:
    fluids = staged("fluids")
    assert (
        Counter(r["REFPROP_NAME"] for r in fluids)["N/A"] > 0
        and Counter(r["SMILES"] for r in fluids)["?"] > 0
    ), "the staged table carries the placeholders this test is about"
    directory = run.env.canonical_dir / "coolprop" / claims.IDENTITY_DIR
    found = {(c.local_key, c.scheme, c.value) for c in claims.read_assertion_claims(directory)}
    expected = {
        (str(r["NAME"]), scheme, value) for r in fluids for scheme, value in expected_assertions(r)
    }
    assert found == expected
    for _, scheme, value in found:
        assert not (scheme == "smiles" and value == "?") and value != "N/A"
    assert not any(scheme == "cas" and not CAS.match(value) for _, scheme, value in found)
    assert len(run.table("tk.identity_assertion", resolution=True)) == len(expected)


def test_identifier_kinds_without_a_naming_scheme_are_declared_not_invented(run: Run) -> None:
    decl = real_declaration()
    schemes = {e.name for e in decl.entities if e.kind == "naming_scheme"}
    assert "chemspider" not in schemes
    rules = {(r["source_table"], r["source_field"]): r for r in run.table("qual.mapping_rule")}
    assert rules[("fluids", "CHEMSPIDER_ID")]["disposition"] == "deferred"


def test_resolution_agrees_with_an_independent_structure_computation(run: Run) -> None:
    def keys(row: dict[str, object]) -> set[str]:
        found: set[str] = set()
        if row["INCHI_KEY"]:
            found.add(str(row["INCHI_KEY"]))
        if row["INCHI_STRING"]:
            found.add(rd_inchi.InchiToInchiKey(str(row["INCHI_STRING"])))
        if row["SMILES"] and row["SMILES"] != "?":
            found.add(Chem.MolToInchiKey(Chem.MolFromSmiles(str(row["SMILES"]))))
        return found

    fluids = [r for r in staged("fluids") if not PSEUDO_PURE_FILE.fullmatch(str(r["CAS"]))]
    structural = {r["NAME"]: keys(r) for r in fluids}
    status = {(r["local_key"]): r["status"] for r in run.table("tk.source_entity", resolution=True)}
    for name, found in structural.items():
        expected = "unresolved" if not found else ("unique" if len(found) == 1 else "ambiguous")
        assert status[name] == expected, name
    counts = Counter(status.values())
    # 130 pure fluids (the five pseudo-pure fluids with a composition are unique mixtures and
    # SES36, which has none, is unresolved: see the tests on the pseudo-pure fluids below)
    assert sum(counts.values()) == 136 and counts["ambiguous"] == 7
    assert counts["unresolved"] == 5 and counts["unique"] == 124
    totals = run.report["totals"]
    assert totals["status"] == dict(sorted(counts.items()))  # type: ignore[index]
    assert {i["entity"]["key"] for i in run.report["ambiguous"]} == {  # type: ignore[attr-defined]
        n for n, s in status.items() if s == "ambiguous"
    }
    species = {r["id"]: r for r in run.table("tk.species", resolution=True)}
    material = {r["id"]: r for r in run.table("tk.material_entity", resolution=True)}
    by_name = {r["local_key"]: r for r in run.table("tk.source_entity", resolution=True)}
    for row in fluids:
        entity = by_name[row["NAME"]]
        target = material[entity["target"]]
        if entity["status"] == "unique":
            assert target["canonical_key"] == next(iter(structural[row["NAME"]]))
            assert species[entity["target"]]["inchikey"] == target["canonical_key"]
        else:
            assert target["provisional"] is True


def component_names() -> dict[str, str]:
    """The fluid a name in the predefined-mixture table denotes, by name or alias without regard
    to case, from the staged fluids alone."""
    names: dict[str, set[str]] = {}
    for row in staged("fluids"):
        for name in (row["NAME"], *(row["ALIASES"] or [])):  # type: ignore[misc]
            names.setdefault(str(name).casefold(), set()).add(str(row["NAME"]))
    return {name: next(iter(found)) for name, found in names.items() if len(found) == 1}


def test_the_pseudo_pure_fluids_are_defined_mixtures_of_resolved_species(run: Run) -> None:
    fluids = {str(r["NAME"]): r for r in staged("fluids")}
    pseudo = sorted(n for n, r in fluids.items() if PSEUDO_PURE_FILE.fullmatch(str(r["CAS"])))
    assert pseudo == ["Air", "R404A", "R407C", "R410A", "R507A", "SES36"]
    entities = {r["local_key"]: r for r in run.table("tk.source_entity", resolution=True)}
    material = {r["id"]: r for r in run.table("tk.material_entity", resolution=True)}
    mixtures = {r["id"]: r for r in run.table("tk.defined_mixture", resolution=True)}
    species = {r["id"] for r in run.table("tk.species", resolution=True)}
    assert {n for n, e in entities.items() if e["entity_class"] == "defined_mixture"} == set(pseudo)
    # no provisional species stands for a pseudo-pure fluid, and no pure fluid is a mixture
    for name in pseudo:
        assert entities[name]["target"] in mixtures and entities[name]["target"] not in species
    assert not any(e["target"] in mixtures for n, e in entities.items() if n not in set(pseudo))
    # the compositions the predefined-mixture table gives: the resolved species of its components
    table = {
        (str(r["name"]), int(r["component_index"])): r
        for r in staged("predefined_mixture_components")
    }  # type: ignore[call-overload]
    lookup = component_names()
    links: dict[object, dict[object, float]] = {}
    for row in run.table("tk.mixture_component", resolution=True):
        links.setdefault(row["mixture"], {})[row["component"]] = row["value"]
    given = 0
    for name in pseudo:
        entity = entities[name]
        parts = [r for (mixture, _), r in sorted(table.items()) if mixture == name]
        if not parts:
            assert name == "SES36"
            assert (entity["status"], entity["rule"]) == ("unresolved", "provisional")
            assert material[entity["target"]]["provisional"] is True
            assert entity["target"] not in links
            continue
        given += 1
        assert (entity["status"], entity["rule"]) == ("unique", "composition"), name
        assert material[entity["target"]]["provisional"] is False
        expected = {}
        for part in parts:
            component = entities[lookup[str(part["fluids"]).casefold()]]
            assert component["status"] == "unique" and component["target"] in species
            expected[component["target"]] = part["mole_fractions"]
        assert links[entity["target"]] == pytest.approx(expected, abs=0, rel=1e-15)
        assert sum(expected.values()) == pytest.approx(1.0, abs=1e-6)
        assert (
            mixtures[entity["target"]]["definition"],
            mixtures[entity["target"]]["mole_basis"],
        ) == (
            "by_definition",
            True,
        )
    assert given == 5
    totals = run.report["totals"]
    assert totals["defined_mixtures"] == 5  # type: ignore[index]
    assert totals["provisional"]["defined_mixture"] == 1  # type: ignore[index]
    for name in pseudo:
        assert entities[name]["entity_class"] == "defined_mixture"


def test_the_compositions_of_the_pseudo_pure_fluids_are_claimed_in_phase_one(run: Run) -> None:
    directory = run.env.canonical_dir / "coolprop" / claims.IDENTITY_DIR
    pseudo = {"Air", "R404A", "R407C", "R410A", "R507A"}
    lookup = component_names()
    expected = {
        (
            str(r["name"]),
            lookup[str(r["fluids"]).casefold()],
            claims.decimal_text(r["mole_fractions"]),
        )  # type: ignore[arg-type]
        for r in staged("predefined_mixture_components")
        if r["name"] in pseudo
    }
    found = claims.read_component_claims(directory)
    assert {(c.local_key, c.component_key, c.fraction) for c in found} == expected
    assert len(found) == len(expected) == 13
    assert {(c.scope, c.component_scope, c.origin_role) for c in found} == {
        ("fluids", "fluids", "published")
    }
    assert ("Air", "Nitrogen", "0.7812") in expected and (
        "R410A",
        "R32",
        "0.697614699375863",
    ) in expected
    ledger = {
        (r["table"], r["state"])
        for r in claims.read_ledger(directory)
        if r["table"].startswith("predefined")
    }
    assert ledger == {
        ("predefined_mixtures", "emitted"),
        ("predefined_mixture_components", "emitted"),
    }
    manifest = run.table("qual.mapping_coverage")
    states = {
        (r["source_table"], r["state"]): r["rows"]
        for r in manifest
        if r["source_table"].startswith("predefined")
    }
    assert states == {
        ("predefined_mixtures", "mapped_with_loss"): 5,
        ("predefined_mixtures", "deferred"): 149,
        ("predefined_mixture_components", "mapped_with_loss"): 13,
        ("predefined_mixture_components", "deferred"): 505,
    }


def test_air_resolves_to_a_defined_mixture_of_nitrogen_argon_and_oxygen(run: Run) -> None:
    entities = {r["local_key"]: r for r in run.table("tk.source_entity", resolution=True)}
    material = {
        r["id"]: r["canonical_key"] for r in run.table("tk.material_entity", resolution=True)
    }
    air = entities["Air"]
    assert (air["status"], air["rule"], air["entity_class"]) == (
        "unique",
        "composition",
        "defined_mixture",
    )
    parts = {
        material[r["component"]]: r["value"]
        for r in run.table("tk.mixture_component", resolution=True)
        if r["mixture"] == air["target"]
    }
    assert parts == {
        material[entities["Nitrogen"]["target"]]: 0.7812,
        material[entities["Argon"]["target"]]: 0.0092,
        material[entities["Oxygen"]["target"]]: 0.2096,
    }
    in_key_order = sorted(parts.items())
    assert material[air["target"]] == "mixture:" + json.dumps(
        ["mole", *(part for key, fraction in in_key_order for part in (key, str(fraction)))],
        separators=(",", ":"),
    )


def test_resolution_is_deterministic(run: Run) -> None:
    first = {p.name: p.read_bytes() for p in run.env.resolution_dir.iterdir()}
    resolve_all(run.env, decl=real_declaration(), force=True)
    assert first == {p.name: p.read_bytes() for p in run.env.resolution_dir.iterdir()}


# -- saturation-pressure ancillaries -----------------------------------------------------------


def test_every_usable_curve_is_one_parameter_set_with_its_terms_and_its_values(run: Run) -> None:
    curves = [
        r for r in staged("ancillary_equations") if r["ancillary"] == "pS" and r["using_tau_r"]
    ]
    coefficients: dict[str, list[dict[str, object]]] = {}
    for row in staged("ancillary_equation_rows"):
        if row["ancillary"] == "pS":
            coefficients.setdefault(str(row["fluid"]), []).append(row)
    entities = {r["local_key"]: r for r in run.table("tk.source_entity", resolution=True)}
    sets = {r["i"]: r for r in run.table(f"param.{SATURATION}")}
    terms: dict[object, list[dict[str, object]]] = {}
    for row in run.table(f"param.{SATURATION}__term"):
        terms.setdefault(row["set_id"], []).append(row)
    regions = {r["record"]: r for r in run.table("tk.validity_region")}
    clauses = {r["region"]: r for r in run.table("tk.region_clause")}
    coverage = {(r["record"], r["kind"]): r["value"] for r in run.table("tk.validity_coverage")}
    temperature = next(
        e.id
        for e in real_declaration().entities
        if e.kind == "observable" and e.name == "temperature"
    )
    held_fluids: set[str] = set()
    assert len(curves) == 130
    for curve in curves:
        name = str(curve["fluid"])
        entity = entities[name]
        if entity["status"] == "ambiguous":
            held_fluids.add(name)
            assert entity["target"] not in sets
            continue
        made = sets[entity["target"]]
        assert made["T_r"] == curve["T_r"] and made["p_r"] == curve["reducing_value"]
        expected = sorted(coefficients[name], key=lambda r: r["row_index"])  # type: ignore[arg-type,return-value]
        got = sorted(terms[made["id"]], key=lambda r: r["k"])  # type: ignore[arg-type,return-value]
        assert len(got) == len(expected) > 0
        assert [(r["k"], r["n"], r["t"]) for r in got] == [
            (int(e["row_index"]) + 1, e["n"], e["t"])
            for e in expected  # type: ignore[call-overload]
        ]
        region = regions[made["id"]]
        assert region["kind"] == "fitted_range" and region["ordinal"] == 1
        clause = clauses[region["id"]]
        assert (clause["lower"], clause["upper"]) == (curve["Tmin"], curve["Tmax"])
        assert clause["observable"] == temperature and clause["ordinal"] == 1
        assert clause["component"] is None and clause["aggregation"] is None
        assert coverage[(made["id"], "fitted_range")] == "stated"
    assert len(sets) == len(curves) - len(held_fluids)
    assert len(regions) == len(clauses) == len(coverage) == len(sets)
    assert len(run.table(f"param.{SATURATION}__term")) == sum(
        len(coefficients[str(c["fluid"])]) for c in curves if c["fluid"] not in held_fluids
    )
    # the role of a set is that of the origins of its record
    set_ids = {r["id"] for r in run.table("tk.parameter_set")}
    roles = {r["role"] for r in run.table("prov.record_origin") if r["record"] in set_ids}
    assert roles == {"fitted"}
    assert len(run.table("tk.parameterization")) == 1
    (parameterization,) = run.table("tk.parameterization")
    assert (
        parameterization["key"] == "coolprop_saturation_ancillaries"
        and parameterization["convention_set"] is None
    )
    # the sets are fitted; the carrier's collection of them is presented as published
    assert {
        r["role"] for r in run.table("prov.record_origin") if r["record"] == parameterization["id"]
    } == {"published"}
    assert not (run.env.canonical_dir / "coolprop" / "tk.auxiliary_of.parquet").exists(), (
        "the equation-of-state parameter sets the curves approximate do not exist yet"
    )
    held = {r["locator"] for r in run.table("qual.held_row")}
    assert {c["_locator"] for c in curves if c["fluid"] in held_fluids} <= held


def test_every_curve_has_one_fit_that_produced_it_and_no_inputs_yet(run: Run) -> None:
    sets = {r["id"] for r in run.table(f"param.{SATURATION}")}
    derivations = {r["id"]: r for r in run.table("prov.derivation")}
    fits = run.table("prov.fit")
    assert len(sets) == len(derivations) == len(fits) == 123
    assert {r["outcome"] for r in fits} == {"not_stated"} and {
        r["kind"] for r in derivations.values()
    } == {"fit"}
    (method,) = {r["method"] for r in derivations.values()}
    assert "fit_ancillary_ODRPACK.py" in method and "replace_ancillaries.py" in method
    assert "does not record the version of the software" in method
    declared = tomllib.loads(
        (config.TREE_DIR / "mappings" / "coolprop" / "mapping.toml").read_text()
    )
    assert (
        method == declared["tables"]["ancillary_equations"]["partitions"][0]["derivation"]["method"]
    )
    outputs = run.table("prov.derivation_output")
    regions = {r["id"] for r in run.table("tk.validity_region")}
    assert len(regions) == 123
    assert {r["record"] for r in outputs} == sets | regions and len(outputs) == 246, (
        "the fit produced each set and the fitted range stated for it"
    )
    assert {r["derivation"] for r in outputs} == set(derivations)
    assert not (run.env.canonical_dir / "coolprop" / "prov.derivation_input.parquet").exists(), (
        "the equation-of-state sets the fits consumed are not mapped yet"
    )
    roles = {r["role"] for r in run.table("prov.record_origin") if r["record"] in derivations}
    assert roles == {"fitted"}


def test_the_source_states_no_units_so_the_assumptions_are_recorded_as_loss(run: Run) -> None:
    rules = {(r["source_table"], r["source_field"]): r for r in run.table("qual.mapping_rule")}
    for column, unit in (("T_r", "K"), ("reducing_value", "Pa"), ("Tmin", "K"), ("Tmax", "K")):
        rule = rules[("ancillary_equations", column)]
        assert rule["source_unit"] == unit and "assumed" in rule["loss"]  # type: ignore[operator]
        # the unit of a clause bound is fixed by the observable of its clause, not by its type
        assert rule["factor"] == (1.0 if column in ("T_r", "reducing_value") else None)
    declared = rules[("ancillary_equations", "partition:saturation_pressure")]["loss"]
    assert "no input" in declared and "auxiliary_of" in declared  # type: ignore[operator]
    assert rules[("ancillary_equations", "derivation:saturation_pressure")]["target"] == "fit"


def test_the_ancillary_equation_fields_are_all_accounted_for(run: Run) -> None:
    rules = {(r["source_table"], r["source_field"]): r for r in run.table("qual.mapping_rule")}
    schema = pq.read_schema(STAGED / "ancillary_equations.parquet")  # type: ignore[operator]
    for field in schema.names:
        if not field.startswith("_"):
            assert ("ancillary_equations", field) in rules, field


# -- coverage ----------------------------------------------------------------------------------


def test_every_row_of_every_table_is_in_exactly_one_state(run: Run) -> None:
    manifest = json.loads((STAGED / "manifest.json").read_text())  # type: ignore[operator]
    assert set(run.coverage.counts) == set(manifest["tables"]) and len(manifest["tables"]) == 37
    for table, counts in run.coverage.counts.items():
        assert sum(counts.values()) == manifest["tables"][table]["rows"], table
    assert run.coverage.counts["fluids"] == {"mapped": 1, "mapped_with_loss": 135}
    curves = run.coverage.counts["ancillary_equations"]
    assert (
        curves["mapped_with_loss"] == 123
        and curves["held"] == 7
        and curves["deferred"] == 854 - 130
    )
    assert run.coverage.counts["ancillary_equation_rows"]["mapped_with_loss"] == 712
    assert run.coverage.counts["text_lines"] == {"out_of_scope": 20540}
    assert run.coverage.total("unmapped") == 0
    written = {
        (r["source_table"], r["state"]): r["rows"] for r in run.table("qual.mapping_coverage")
    }
    assert written == {(t, s): n for t, c in run.coverage.counts.items() for s, n in c.items()}


# -- the canonical Parquet loads ---------------------------------------------------------------


def test_the_canonical_output_builds_with_every_constraint_satisfied_at_commit(run: Run) -> None:
    """The resolution result and the mapped records are the inputs of one build; identical rows
    both emit (the carrier, its artifacts, the licence) are one row."""
    decl = real_declaration()
    with TestDatabase() as database:
        result = build_database(database.url, decl, discover(run.env.canonical_dir))
    counts = result.tables
    sets = len(run.table(f"param.{SATURATION}"))
    assert counts[f"param.{SATURATION}"] == sets == 123
    assert counts["tk.source_entity"] == 136 and counts["prov.carrier"] == 1
    assert counts["tk.identity_assertion"] == len(
        run.table("tk.identity_assertion", resolution=True)
    )
    assert [source.manifest_id for source in result.sources] == ["coolprop"]
