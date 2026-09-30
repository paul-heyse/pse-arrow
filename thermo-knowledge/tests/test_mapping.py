# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The mapping framework on a fake source: rules, row states, phases, provenance and the CLI."""

from __future__ import annotations

import ast
import json
import re
import shutil
import tomllib
from collections.abc import Callable
from pathlib import Path

import msgspec
import pytest
from mapping_support import (
    FAKE_MAPPINGS,
    FAKE_TREE,
    Workspace,
    fake_environment,
    real_declaration,
    rows,
)
from typer.testing import CliRunner

from thermo_knowledge import config
from thermo_knowledge.canonical import store
from thermo_knowledge.canonical.environment import Environment
from thermo_knowledge.canonical.writer import CompetingAssertion
from thermo_knowledge.cli import app
from thermo_knowledge.declaration.types import registry
from thermo_knowledge.mapping import claims, runner
from thermo_knowledge.mapping.spec import MappingSpec, load_spec, validate
from thermo_knowledge.mapping.runner import MapError
from thermo_knowledge.resolve.command import resolve_all

cli = CliRunner()
REAL_MAPPINGS = config.TREE_DIR / "mappings"


@pytest.fixture
def fake(tmp_path: Path) -> tuple[Environment, Workspace]:
    return fake_environment(tmp_path)


def run_all(env: Environment) -> runner.MapOutcome:
    runner.run_identity(env, "fake")
    resolve_all(env, decl=real_declaration())
    return runner.run_records(env, "fake")


def output(env: Environment, name: str, *, source: str = "fake") -> list[dict[str, object]]:
    return rows(env.canonical_dir / source / f"{name}.parquet")


# -- mapping.toml against the declaration and the staged tables -------------------------------


def spec_problems(env: Environment, mutate: Callable[[dict[str, object]], None]) -> list[str]:
    data = tomllib.loads((env.mappings_dir / "fake" / "mapping.toml").read_text())
    mutate(data)
    prepared = runner.prepare(env, "fake")
    return validate(
        msgspec.convert(data, MappingSpec),
        real_declaration(),
        prepared.tables.schemas,
        source="fake",
    )


def tables(data: dict[str, object]) -> dict[str, dict[str, object]]:
    return data["tables"]  # type: ignore[return-value]


def fields(data: dict[str, object], table: str) -> dict[str, dict[str, object]]:
    return tables(data)[table]["fields"]  # type: ignore[return-value]


def test_the_fixture_mapping_is_valid(fake: tuple[Environment, Workspace]) -> None:
    assert spec_problems(fake[0], lambda data: None) == []


@pytest.mark.parametrize(
    ("mutate", "message"),
    [
        (lambda d: tables(d).pop("notes"), "table notes: has no declared disposition"),
        (
            lambda d: tables(d).update(extra={"disposition": "mapped", "origin_role": "published"}),
            "table extra: is declared but not staged",
        ),
        (
            lambda d: fields(d, "curves").pop("p_bar"),
            "column `p_bar` has neither a rule nor a declared skip",
        ),
        (lambda d: fields(d, "curves")["T_r"].update(unit="Pa"), "cannot be converted to `K`"),
        (
            lambda d: fields(d, "curves")["p_bar"].update(unit="Pa"),
            "the source states `bar` and the rule says `Pa`",
        ),
        (
            lambda d: fields(d, "curves")["T_r"].pop("loss"),
            "the source states no unit, so the rule records its assumption",
        ),
        (
            lambda d: fields(d, "curves")["T_r"].pop("unit"),
            "the target is dimensioned, so the rule states its `unit`",
        ),
        (
            lambda d: fields(d, "curves")["T_r"].update(
                target="vapor_pressure_exp_series_tau.pure.nothing"
            ),
            "is not a slot of",
        ),
        (
            lambda d: fields(d, "curves")["T_r"].update(target="nothing.value"),
            "is neither a kind nor a relation",
        ),
        (
            lambda d: fields(d, "species")["cas"].update(scheme="nonsense"),
            "`nonsense` is not a declared naming scheme",
        ),
        (
            lambda d: fields(d, "species")["cas"].pop("scheme"),
            "an identity assertion names its `scheme`",
        ),
        (lambda d: fields(d, "species")["cas"].update(pattern="("), "not a regular expression"),
        (
            lambda d: fields(d, "species")["name"].pop("precision"),
            "a mapped column states its `precision`",
        ),
        (lambda d: fields(d, "species")["name"].update(unit="K"), "the target has no unit"),
        (lambda d: tables(d)["notes"].pop("reason"), "`out_of_scope` needs a `reason`"),
        (lambda d: tables(d)["curves"].pop("wave"), "`deferred` names the `wave`"),
        (
            lambda d: tables(d)["species"].pop("origin_role"),
            "names the `origin_role` of its records",
        ),
        (
            lambda d: tables(d)["species"].update(origin_role="invented"),
            "`invented` is not an origin role",
        ),
        (
            lambda d: tables(d)["curves"]["partitions"][0].update(where={"nothing": 1}),
            "`nothing` is not a column",
        ),
        (
            lambda d: tables(d)["curves"]["partitions"][0]["constants"].update(
                {"validity_region.kind": "nonsense"}
            ),
            "`nonsense` is not a member of `envelope_kind`",
        ),
        (
            lambda d: tables(d)["curves"]["partitions"][0]["constants"].update(
                {"region_clause.observable": "no_such_observable"}
            ),
            "is not a declared entity of kind `observable`",
        ),
        (
            lambda d: tables(d)["curves"]["partitions"][0]["constants"].update(
                {"region_clause.missing": 1}
            ),
            "is not an attribute of kind `region_clause`",
        ),
        (
            lambda d: d["scopes"]["species"].update(key="nothing"),
            "`nothing` is not a column of `species`",
        ),  # type: ignore[index]
        (
            lambda d: d["parameterizations"]["curves"].update(coherence="sometimes"),
            "`sometimes` is not a coherence",
        ),  # type: ignore[index]
        (
            lambda d: d["parameterizations"]["curves"].update(origin_role="invented"),
            "`invented` is not an origin role",
        ),  # type: ignore[index]
        (
            lambda d: d["parameterizations"]["curves"].pop("no_convention_set"),
            "state exactly one of `convention_set`",
        ),  # type: ignore[index]
        (
            lambda d: d["parameterizations"]["curves"].update(revision="{version}"),
            "may use only the placeholder {pin}",
        ),  # type: ignore[index]
        (lambda d: d.update(source="other"), "is not the mapping's directory name"),
        (
            lambda d: tables(d)["curves"]["partitions"][0]["derivation"].update(kind="invented"),
            "derivation kind `invented` is not a `derivation_kind`",
        ),
        (
            lambda d: tables(d)["curves"]["partitions"][0]["derivation"].pop("outcome"),
            "a fit states its `outcome`",
        ),
        (
            lambda d: tables(d)["curves"]["partitions"][0]["derivation"].update(kind="estimation"),
            "`outcome` belongs to a fit",
        ),
        (
            lambda d: tables(d)["curves"]["partitions"][0]["derivation"].update(method=" "),
            "a derivation states its `method`",
        ),
        (
            lambda d: d.update(formula_scope=[{"scope": "nothing", "reason": "r"}]),
            "`nothing` is not a declared scope",
        ),
    ],
)
def test_a_mapping_the_declaration_cannot_hold_is_refused(
    fake: tuple[Environment, Workspace], mutate: Callable[[dict[str, object]], None], message: str
) -> None:
    assert any(message in problem for problem in spec_problems(fake[0], mutate))


def test_an_offset_belongs_to_an_integer_target(fake: tuple[Environment, Workspace]) -> None:
    problems = spec_problems(fake[0], lambda d: fields(d, "curves")["T_r"].update(offset=1))
    assert any("`offset` applies to an Integer target" in problem for problem in problems)


def test_an_unknown_key_in_mapping_toml_is_refused(tmp_path: Path) -> None:
    path = tmp_path / "mapping.toml"
    path.write_text('source = "x"\ndoc = "d"\nsurprise = 1\n[tables]\n')
    with pytest.raises(Exception, match="surprise"):
        load_spec(path)


# -- the phases and their order ---------------------------------------------------------------


def test_phase_two_needs_phase_one_and_a_resolution_made_from_it(
    fake: tuple[Environment, Workspace],
) -> None:
    env, _ = fake
    with pytest.raises(MapError, match="no phase-1 output; run `tk map fake --phase identity`"):
        runner.run_records(env, "fake")
    assert runner.run_identity(env, "fake").status == "mapped"
    with pytest.raises(MapError, match="there is no resolution result; run `tk resolve`"):
        runner.run_records(env, "fake")
    resolve_all(env, decl=real_declaration())
    assert runner.run_records(env, "fake").status == "mapped"

    # the mapping changes: phase 1 is older than the mapping
    path = env.mappings_dir / "fake" / "mapping.toml"
    path.write_text(path.read_text() + "\n# a later edit\n")
    with pytest.raises(MapError, match="phase-1 output is older than the mapping"):
        runner.run_records(env, "fake")
    # phase 1 again: the resolution result is now older than the phase-1 output
    assert runner.run_identity(env, "fake").status == "mapped"
    with pytest.raises(
        MapError, match="resolution result is older than the phase-1 output of fake"
    ):
        runner.run_records(env, "fake")
    resolve_all(env, decl=real_declaration())
    assert runner.run_records(env, "fake").status == "mapped"


def test_each_phase_is_skipped_when_its_inputs_are_unchanged(
    fake: tuple[Environment, Workspace],
) -> None:
    env, _ = fake
    assert runner.run_identity(env, "fake").status == "mapped"
    assert runner.run_identity(env, "fake").status == "current"
    resolve_all(env, decl=real_declaration())
    first = runner.run_records(env, "fake")
    assert first.status == "mapped" and runner.run_records(env, "fake").status == "current"
    before = {p.name: p.read_bytes() for p in (env.canonical_dir / "fake").glob("*.parquet")}
    assert runner.run_records(env, "fake", force=True).status == "mapped"
    assert before == {
        p.name: p.read_bytes() for p in (env.canonical_dir / "fake").glob("*.parquet")
    }
    assert (env.canonical_dir / "fake" / claims.IDENTITY_DIR / store.MANIFEST_NAME).is_file(), (
        "a new phase-2 output keeps the phase-1 output"
    )


def test_phase_two_output_is_written_atomically(fake: tuple[Environment, Workspace]) -> None:
    env, _ = fake
    run_all(env)
    leftovers = [p.name for p in env.canonical_dir.iterdir() if p.name.startswith(".")]
    assert leftovers == []
    manifest = store.read_manifest(env.canonical_dir / "fake")
    assert manifest.phase == "records" and set(manifest.inputs) >= {
        "staged",
        "mapping",
        "declaration",
        "identity",
        "resolution",
    }
    store.verify_directory(env.canonical_dir / "fake", manifest)


# -- phase 1: identities -----------------------------------------------------------------------


def test_identity_claims_carry_every_identifier_but_no_placeholder(
    fake: tuple[Environment, Workspace],
) -> None:
    env, _ = fake
    runner.run_identity(env, "fake")
    directory = env.canonical_dir / "fake" / claims.IDENTITY_DIR
    entities, assertions = (
        claims.read_entity_claims(directory),
        claims.read_assertion_claims(directory),
    )
    assert sorted(c.local_key for c in entities) == [
        "Conflicted",
        "Ethanol",
        "Lonely",
        "Mystery",
        "Plain",
        "Water",
    ]
    ethanol = {(a.scheme, a.value) for a in assertions if a.local_key == "Ethanol"}
    assert ethanol == {
        ("name", "Ethanol"),
        ("name", "EtOH"),
        ("name", "alcohol"),
        ("cas", "64-17-5"),
        ("inchikey", "LFQSCWFLJHTTHZ-UHFFFAOYSA-N"),
    }
    assert not any(a.value in ("N/A", "?") for a in assertions), (
        "a declared absent marker is never asserted"
    )
    mystery = {(a.scheme, a.value) for a in assertions if a.local_key == "Mystery"}
    assert ("source_local", "MYSTERY.PPF") in mystery and not any(s == "cas" for s, _ in mystery)
    ledger = claims.read_ledger(directory)
    assert {r["state"] for r in ledger} == {"emitted"} and len(ledger) == 6


def all_optional(path: Path) -> None:
    """Declare every value rule of a mapping.toml optional: test scaffolding for a mapping.py
    that emits less than the fixture's rules declare."""
    text = path.read_text().replace("optional = true\n", "")
    path.write_text(re.sub(r"(?m)^(target = .*)$", r"\1\noptional = true", text))


def test_a_value_that_fits_no_scheme_holds_the_row(
    fake: tuple[Environment, Workspace], tmp_path: Path
) -> None:
    env, _ = fake_environment(tmp_path / "strict")
    path = env.mappings_dir / "fake" / "mapping.toml"
    path.write_text(path.read_text().replace('otherwise_scheme = "source_local"\n', ""))
    runner.run_identity(env, "fake")
    ledger = {
        r["locator"]: r
        for r in claims.read_ledger(env.canonical_dir / "fake" / claims.IDENTITY_DIR)
    }
    held = ledger["data/species.json#/2"]
    assert held["state"] == "held" and held["reason"] == "pattern_mismatch"
    assert "does not match" in held["detail"]  # type: ignore[operator]
    assert len(claims.read_entity_claims(env.canonical_dir / "fake" / claims.IDENTITY_DIR)) == 5


def test_a_declared_formula_scope_reaches_resolution(tmp_path: Path) -> None:
    """The scopes a mapping declares formula-identified travel in the phase-1 manifest, which
    is all resolution reads of a mapping."""
    mappings = tmp_path / "mappings"
    shutil.copytree(FAKE_MAPPINGS, mappings)
    path = mappings / "fake" / "mapping.toml"
    path.write_text(
        path.read_text().replace(
            "[tables.species]\n",
            '[[formula_scope]]\nscope = "species"\nreason = "The fixture names ions by formula."\n'
            'discriminator = "aqueous"\n\n[tables.species]\n',
            1,
        )
    )
    env, _ = fake_environment(tmp_path / "work", mappings=mappings)
    runner.run_identity(env, "fake")
    manifest = store.read_manifest(env.canonical_dir / "fake" / claims.IDENTITY_DIR)
    assert [(s.scope, s.discriminator) for s in manifest.formula_scopes] == [("species", "aqueous")]
    assert manifest.carrier is not None and manifest.carrier.key == "fake@0123456789ab"
    assert set(manifest.carrier.artifacts) == {"data/species.json"}


# -- phase 2: records and coverage -------------------------------------------------------------


def test_every_row_ends_in_exactly_one_state_and_the_counts_are_the_report(
    fake: tuple[Environment, Workspace],
) -> None:
    env, _ = fake
    outcome = run_all(env)
    assert outcome.coverage is not None
    counts = outcome.coverage.counts
    assert counts == {
        "species": {"mapped": 6},
        "curves": {"mapped_with_loss": 3, "held": 3, "unmapped": 1, "deferred": 1},
        "coefficients": {"mapped_with_loss": 6, "held": 3, "deferred": 1},
        "notes": {"out_of_scope": 2},
    }
    totals = {"species": 6, "curves": 8, "coefficients": 10, "notes": 2}
    assert {t: sum(c.values()) for t, c in counts.items()} == totals
    written = {
        (r["source_table"], r["state"]): r["rows"] for r in output(env, "qual.mapping_coverage")
    }
    assert written == {(t, s): n for t, c in counts.items() for s, n in c.items()}
    held = {(r["source_table"], r["locator"]): r for r in output(env, "qual.held_row")}
    unmapped = held[("curves", "data/curves.json#/6")]
    assert unmapped["state"] == "unmapped" and unmapped["reason"] == "unmapped_by_mapping"
    assert "emitted nothing" in unmapped["detail"]  # type: ignore[operator]
    ambiguous = held[("curves", "data/curves.json#/3")]
    assert ambiguous["state"] == "held" and ambiguous["reason"] == "ambiguous_subject"
    assert "ambiguous" in ambiguous["detail"]  # type: ignore[operator]
    invalid = held[("curves", "data/curves.json#/4")]
    assert invalid["reason"] == "validation_failed"
    assert invalid["detail"].startswith("data/curves.json#/4: ")  # type: ignore[union-attr]
    assert "T_r: -5.0 is negative" in invalid["detail"]  # type: ignore[operator]
    unknown = held[("curves", "data/curves.json#/5")]
    assert unknown["reason"] == "unknown_subject"
    assert "no source entity (species, 'Nobody')" in unknown["detail"]  # type: ignore[operator]
    assert len(held) == 7, "only held and unmapped rows are listed"
    by_reason = {reason: sum(1 for r in held.values() if r["reason"] == reason) for reason in
                 {r["reason"] for r in held.values()}}
    assert outcome.coverage.by_reason() == dict(sorted(by_reason.items()))
    assert "rows not loaded, by reason:" in "\n".join(runner.coverage_lines(outcome.coverage))


def test_a_mapping_that_reads_rows_and_emits_nothing_covers_nothing(tmp_path: Path) -> None:
    """Silence is never coverage: every row of a mapped table or partition that produced no
    record and was not held is `unmapped`."""
    mappings = tmp_path / "mappings"
    shutil.copytree(FAKE_MAPPINGS, mappings)
    (mappings / "fake" / "mapping.py").write_text(
        "def identities(ctx):\n    for row in ctx.rows('species'):\n        pass\n\n\n"
        "def records(ctx):\n    for row in ctx.rows('curves'):\n        pass\n"
    )
    # a mapping that emits nothing applies no rule; declaring every value rule optional keeps
    # this test about coverage (the refusal of an unused rule is tested in test_rule_use.py)
    all_optional(mappings / "fake" / "mapping.toml")
    env, _ = fake_environment(tmp_path / "work", mappings=mappings)
    outcome = run_all(env)
    assert outcome.coverage is not None
    assert outcome.coverage.counts == {
        "species": {"unmapped": 6},
        "curves": {"unmapped": 7, "deferred": 1},
        "coefficients": {"unmapped": 9, "deferred": 1},
        "notes": {"out_of_scope": 2},
    }
    assert len(output(env, "qual.held_row")) == 6 + 7 + 9
    assert not any(
        name.startswith("param.") for name in store.read_manifest(env.canonical_dir / "fake").tables
    )


def test_parameter_sets_carry_converted_values_and_every_origin(
    fake: tuple[Environment, Workspace],
) -> None:
    env, _ = fake
    run_all(env)
    sets = output(env, "param.vapor_pressure_exp_series_tau__pure")
    assert len(sets) == 3
    by_temperature = {round(float(r["T_r"]), 1): r for r in sets}  # type: ignore[arg-type]
    ethanol = by_temperature[513.9]
    assert ethanol["p_r"] == pytest.approx(6.14e6), "61.4 bar in pascal"
    terms = [
        r
        for r in output(env, "param.vapor_pressure_exp_series_tau__pure__term")
        if r["set_id"] == ethanol["id"]
    ]
    assert sorted((r["k"], r["n"], r["t"]) for r in terms) == [
        (1, -8.5, 1.0),
        (2, 1.2, 1.5),
        (3, -3.1, 3.0),
    ]
    origins = [r for r in output(env, "prov.record_origin") if r["record"] == ethanol["id"]]
    assert len(origins) == 4 and {r["role"] for r in origins} == {"fitted"}
    records = {r["id"]: r["kind"] for r in output(env, "prov.record")}
    assert records[ethanol["id"]] == "vapor_pressure_exp_series_tau__pure"
    (region,) = [r for r in output(env, "tk.validity_region") if r["record"] == ethanol["id"]]
    assert region["kind"] == "fitted_range" and region["ordinal"] == 1
    (clause,) = [r for r in output(env, "tk.region_clause") if r["region"] == region["id"]]
    assert clause["lower"] == 250.0 and clause["upper"] is None and clause["ordinal"] == 1
    (coverage,) = [r for r in output(env, "tk.validity_coverage") if r["record"] == ethanol["id"]]
    assert (coverage["kind"], coverage["value"]) == ("fitted_range", "stated")
    (parameterization,) = output(env, "tk.parameterization")
    assert (
        parameterization["revision"] == "0123456789ab"
        and parameterization["convention_set"] is None
    )
    # the subject is the resolved entity: Ethanol's species
    species = {r["id"]: r for r in rows(env.resolution_dir / "tk.material_entity.parquet")}
    assert species[ethanol["i"]]["canonical_key"] == "LFQSCWFLJHTTHZ-UHFFFAOYSA-N"


def test_a_parameterization_takes_its_own_origin_role_not_the_role_of_the_block_that_emits_it(
    fake: tuple[Environment, Workspace],
) -> None:
    """The sets are fitted; the collection they belong to is presented as published, whichever
    block emits it first, and so is the convention set it names."""
    env, _ = fake
    path = env.mappings_dir / "fake" / "mapping.toml"
    text = path.read_text()
    text = text.replace(
        'no_convention_set = "The fixture states no conventions."',
        'convention_set = "fixture"\n\n[convention_sets.fixture]\nkey = "fixture_conventions"\nrevision = "1"\ntemperature_scale = "its_90"',
    )
    path.write_text(text)
    run_all(env)
    records = {r["id"]: r["kind"] for r in output(env, "prov.record")}
    roles: dict[str, set[object]] = {}
    for origin in output(env, "prov.record_origin"):
        roles.setdefault(records[origin["record"]], set()).add(origin["role"])  # type: ignore[index]
    assert roles["vapor_pressure_exp_series_tau__pure"] == {"fitted"}
    assert roles["parameterization"] == {"published"}
    assert roles["convention_set"] == {"published"}
    (parameterization,) = output(env, "tk.parameterization")
    assert parameterization["convention_set"] is not None


def test_each_parameter_set_has_the_fit_derivation_that_produced_it(
    fake: tuple[Environment, Workspace],
) -> None:
    env, _ = fake
    run_all(env)
    sets = {r["id"] for r in output(env, "param.vapor_pressure_exp_series_tau__pure")}
    fits = output(env, "prov.fit")
    derivations = {r["id"]: r for r in output(env, "prov.derivation")}
    assert len(fits) == len(derivations) == len(sets) == 3
    assert {r["outcome"] for r in fits} == {"converged"}
    assert {(r["kind"], r["method"]) for r in derivations.values()} == {
        ("fit", "A fit declared by the fixture mapping.")
    }
    outputs = output(env, "prov.derivation_output")
    regions = {r["id"] for r in output(env, "tk.validity_region")}
    assert len(regions) == 3
    assert {r["record"] for r in outputs} == sets | regions
    assert {r["derivation"] for r in outputs} == set(derivations)
    assert len(outputs) == 6, "one producing derivation per record: the set and its fitted region"
    assert not (env.canonical_dir / "fake" / "prov.derivation_input.parquet").exists()
    records = {r["id"]: r["kind"] for r in output(env, "prov.record")}
    assert {records[i] for i in derivations} == {"fit"}
    origins = {r["record"]: r for r in output(env, "prov.record_origin")}
    assert all(i in origins for i in derivations), "a derivation is a record with origins"
    rules = {(r["source_table"], r["source_field"]): r for r in output(env, "qual.mapping_rule")}
    assert (
        rules[("curves", "derivation:pressure")]["loss"] == "A fit declared by the fixture mapping."
    )


def test_the_rules_of_a_mapping_are_written_with_their_unit_factor_and_loss(
    fake: tuple[Environment, Workspace],
) -> None:
    env, _ = fake
    run_all(env)
    rules = {(r["source_table"], r["source_field"]): r for r in output(env, "qual.mapping_rule")}
    bar = rules[("curves", "p_bar")]
    assert bar["target"] == "vapor_pressure_exp_series_tau.pure.p_r" and bar["source_unit"] == "bar"
    assert bar["factor"] == pytest.approx(1e5) and bar["precision"] == "exact"
    kelvin = rules[("curves", "T_r")]
    assert kelvin["factor"] == 1.0 and "kelvin is assumed" in kelvin["loss"]  # type: ignore[operator]
    assert (
        rules[("curves", "")]["disposition"] == "deferred"
        and "wave 2" in rules[("curves", "")]["loss"]
    )  # type: ignore[operator]
    assert rules[("curves", "partition:pressure")]["disposition"] == "mapped"
    assert rules[("curves", "constant:validity_region.kind")]["disposition"] == "mapped"
    assert rules[("notes", "")]["disposition"] == "out_of_scope"
    assert (
        rules[("curves", "species")]["disposition"] == "mapped"
        and rules[("curves", "species")]["target"] is None
    )
    assert "declared absent: 'N/A', '?'" in rules[("species", "placeholder")]["loss"]  # type: ignore[operator]


def test_two_rows_that_make_one_record_with_different_content_refuse_the_run(
    tmp_path: Path,
) -> None:
    tree = tmp_path / "tree"
    shutil.copytree(FAKE_TREE, tree)
    curves = json.loads((tree / "data" / "curves.json").read_text())
    curves.append({"species": "Ethanol", "curve": "pressure", "T_r": 999.0, "p_bar": 61.4})
    (tree / "data" / "curves.json").write_text(json.dumps(curves))
    env, _ = fake_environment(tmp_path / "work", tree=tree)
    runner.run_identity(env, "fake")
    resolve_all(env, decl=real_declaration())
    with pytest.raises(CompetingAssertion) as caught:
        runner.run_records(env, "fake")
    assert "curves.json#/0" in str(caught.value) and "curves.json#/8" in str(caught.value)
    assert not (env.canonical_dir / "fake" / "manifest.json").exists(), (
        "a refused run writes nothing"
    )


def test_a_mapping_numbers_repeated_assertions_and_each_becomes_a_set(tmp_path: Path) -> None:
    """The source lists the pair twice with different values: the mapping passes `occurrence`, from
    one in source order, and both sets are kept (a mapping that does not pass it gets one)."""
    tree = tmp_path / "tree"
    shutil.copytree(FAKE_TREE, tree)
    curves = json.loads((tree / "data" / "curves.json").read_text())
    curves.append({"species": "Ethanol", "curve": "pressure", "T_r": 999.0, "p_bar": 61.4})
    (tree / "data" / "curves.json").write_text(json.dumps(curves))
    mappings = tmp_path / "mappings"
    shutil.copytree(FAKE_MAPPINGS, mappings)
    source = (mappings / "fake" / "mapping.py").read_text()
    assert '    for curve in ctx.rows("curves", "pressure"):\n' in source
    source = source.replace(
        '    for curve in ctx.rows("curves", "pressure"):\n',
        '    seen: dict[object, int] = {}\n    for curve in ctx.rows("curves", "pressure"):\n'
        '        seen[curve["species"]] = seen.get(curve["species"], 0) + 1\n',
    ).replace(
        '                families={"term"',
        '                occurrence=seen[curve["species"]],\n                families={"term"',
    )
    (mappings / "fake" / "mapping.py").write_text(source)
    env, _ = fake_environment(tmp_path / "work", tree=tree, mappings=mappings)
    runner.run_identity(env, "fake")
    resolve_all(env, decl=real_declaration())
    runner.run_records(env, "fake")
    sets = {r["id"]: r for r in output(env, "tk.parameter_set")}
    assert sorted(r["occurrence"] for r in sets.values()) == [1, 1, 1, 2]
    (second,) = [r for r in sets.values() if r["occurrence"] == 2]
    (first,) = [
        r
        for r in sets.values()
        if r["occurrence"] == 1 and r["subject_key"] == second["subject_key"]
    ]
    rows = {r["id"]: r for r in output(env, "param.vapor_pressure_exp_series_tau__pure")}
    assert rows[first["id"]]["T_r"] == pytest.approx(513.9) and rows[second["id"]][
        "T_r"
    ] == pytest.approx(999.0)


# -- the convention sets a mapping states -------------------------------------------------------

CONVENTIONS = """convention_set = "fixture"

[convention_sets.fixture]
key = "fixture_conventions"
revision = "1"
temperature_scale = "its_90"
{facts}"""

NASA_RECORDS = """
PIECE = {
    "T_low": Quantity(200.0, "K"),
    "T_high": Quantity(1000.0, "K"),
    "a1": 3.5,
    "a2": Quantity(0.0, "1/K"),
    "a3": Quantity(0.0, "1/K**2"),
    "a4": Quantity(0.0, "1/K**3"),
    "a5": Quantity(0.0, "1/K**4"),
    "a6": Quantity(0.0, "K"),
    "a7": 1.0,
}


def records(ctx: RecordContext) -> None:
    for curve in ctx.rows("curves", "pressure"):
        with ctx.emit(curve) as emit:
            emit.parameter_set(
                parameterization=emit.parameterization("curves"),
                slot_group="nasa7.pure",
                subjects=[ctx.subject("species", curve["species"])],
                slots={},
                families={"piece": [FamilyRow({"n": 1}, PIECE)]},
            )
"""


def nasa_mapping(env: Environment, convention: str | None) -> None:
    """The fixture mapping with its records replaced by sets of `nasa7`, a form that reads the gas
    constant (test scaffolding: a real mapping.py holds no number or unit), and its
    parameterization given `convention`, a [convention_sets] body, or none."""
    directory = env.mappings_dir / "fake"
    text = (directory / "mapping.py").read_text()
    kept = text[: text.index("def records")].replace(
        "from thermo_knowledge.mapping.context",
        "from thermo_knowledge.canonical.values import Quantity\n"
        "from thermo_knowledge.canonical.writer import FamilyRow\n"
        "from thermo_knowledge.mapping.context",
        1,
    )
    (directory / "mapping.py").write_text(kept + NASA_RECORDS)
    all_optional(directory / "mapping.toml")
    if convention is not None:
        path = directory / "mapping.toml"
        path.write_text(
            path.read_text().replace(
                'no_convention_set = "The fixture states no conventions."',
                CONVENTIONS.format(facts=convention),
            )
        )


def test_a_convention_set_states_the_gas_constant_with_a_source_unit(
    fake: tuple[Environment, Workspace],
) -> None:
    env, _ = fake
    nasa_mapping(env, 'gas_constant = { value = 1.98720425864083, unit = "cal/(mol*K)" }\n')
    run_all(env)
    (convention,) = output(env, "tk.convention_set")
    # 1.98720425864083 thermochemical calories per mole and kelvin, in J/(mol K)
    assert convention["gas_constant"] == pytest.approx(1.98720425864083 * 4.184, rel=1e-14)
    assert "standard_pressure" not in convention
    assert output(env, "param.nasa7__pure"), "the sets of the form that reads the constant load"


@pytest.mark.parametrize(
    ("convention", "why"),
    [
        ("", "does not state it"),
        (None, "names no convention set"),
    ],
    ids=["a convention set without the fact", "no convention set"],
)
def test_a_mapping_that_emits_sets_of_a_form_reading_conventions_without_stating_them_is_refused(
    fake: tuple[Environment, Workspace], convention: str | None, why: str
) -> None:
    from thermo_knowledge.mapping.staged import MappingError

    env, _ = fake
    nasa_mapping(env, convention)
    runner.run_identity(env, "fake")
    resolve_all(env, decl=real_declaration())
    with pytest.raises(MappingError) as refused:
        runner.run_records(env, "fake")
    message = str(refused.value)
    assert "form `nasa7`" in message and "`gas_constant`" in message and why in message
    assert not (env.canonical_dir / "fake" / "tk.parameter_set.parquet").exists(), (
        "the run is refused before anything is written"
    )


@pytest.mark.parametrize(
    ("facts", "problem"),
    [
        ({"gas_constant": {"value": 8.3, "unit": "m"}}, "cannot be converted"),
        ({"gas_constant": 8.3}, "state `{ value"),
        ({"gas_constant": {"value": 8.3, "unit": "J/(mol*K)"}, "nothing": 1}, "not an attribute"),
        ({"standard_pressure": {"value": 1.0, "unit": "bar"}}, "not an attribute"),
        ({"temperature_scale": "nonsense"}, "not a member of `temperature_scale`"),
        ({"temperature_scale": {"value": 1.0, "unit": "K"}}, "no unit"),
    ],
)
def test_a_convention_fact_is_checked_like_any_other_value_rule(
    fake: tuple[Environment, Workspace], facts: dict[str, object], problem: str
) -> None:
    def mutate(data: dict[str, object]) -> None:
        data["parameterizations"]["curves"].pop("no_convention_set")  # type: ignore[index]
        data["parameterizations"]["curves"]["convention_set"] = "fixture"  # type: ignore[index]
        data["convention_sets"] = {  # type: ignore[assignment]
            "fixture": {"key": "k", "revision": "1", "temperature_scale": "its_90", **facts}
        }

    assert any(problem in found for found in spec_problems(fake[0], mutate))


# -- mapping.py holds structure only -----------------------------------------------------------


@pytest.mark.parametrize(
    "path", sorted(REAL_MAPPINGS.glob("*/mapping.py")) + [FAKE_MAPPINGS / "fake" / "mapping.py"]
)
def test_a_mapping_py_has_no_unit_number_or_default(path: Path) -> None:
    """Units, constants and defaults belong in mapping.toml (pipeline section 2)."""
    tree = ast.parse(path.read_text())
    ureg = registry()
    problems: list[str] = []
    for node in ast.walk(tree):
        if isinstance(node, ast.Constant) and not isinstance(node.value, bool):
            if isinstance(node.value, (int, float)):
                problems.append(f"line {node.lineno}: the number {node.value!r}")
            elif isinstance(node.value, str) and node.value and not node.value.isspace():
                try:
                    unit = ureg.parse_units(node.value)
                except Exception:
                    continue
                if not unit.dimensionless or node.value in ("dimensionless",):
                    problems.append(f"line {node.lineno}: the unit {node.value!r}")
    assert problems == []


# -- the CLI -----------------------------------------------------------------------------------


def test_the_commands_run_the_stages_in_order(
    fake: tuple[Environment, Workspace], monkeypatch: pytest.MonkeyPatch
) -> None:
    from thermo_knowledge.mapping import command as map_command
    from thermo_knowledge.resolve import command as resolve_command

    env, _ = fake
    monkeypatch.setattr(map_command, "Environment", lambda: env)
    monkeypatch.setattr(resolve_command, "Environment", lambda: env)

    refused = cli.invoke(app, ["map", "fake"])
    assert refused.exit_code == 1 and "no phase-1 output" in refused.output
    bad = cli.invoke(app, ["map", "--phase", "both"])
    assert bad.exit_code == 2
    identity = cli.invoke(app, ["map", "fake", "--phase", "identity"])
    assert identity.exit_code == 0, identity.output
    assert "claims.source_entity" in identity.output and "6" in identity.output
    early = cli.invoke(app, ["map", "fake"])
    assert early.exit_code == 1 and "no resolution result" in early.output
    resolved = cli.invoke(app, ["resolve"])
    assert resolved.exit_code == 0, resolved.output
    assert (
        "unique/structural: 4" in resolved.output
        and "ambiguous  fake/species/Conflicted" in resolved.output
    )
    mapped = cli.invoke(app, ["map"])
    assert mapped.exit_code == 0, mapped.output
    assert "mapped_with_loss" in mapped.output and "unmapped" in mapped.output
    assert cli.invoke(app, ["map", "fake"]).output.startswith("current")
