# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""A declared value rule must be applied to at least one mapped row: the context counts the
application, the run refuses a rule nothing consumed (unless it is declared optional), and
`qual.mapping_rule` records the count."""

from __future__ import annotations

import shutil
import tomllib
from pathlib import Path

import msgspec
import pytest
from mapping_support import FAKE_MAPPINGS, fake_environment, real_declaration, rows

from thermo_knowledge.canonical.environment import Environment
from thermo_knowledge.mapping import coverage, runner
from thermo_knowledge.mapping.runner import MapError
from thermo_knowledge.mapping.spec import MappingSpec, validate
from thermo_knowledge.resolve.command import resolve_all


def run_all(env: Environment) -> runner.MapOutcome:
    runner.run_identity(env, "fake")
    resolve_all(env, decl=real_declaration())
    return runner.run_records(env, "fake")


def mapping_copy(tmp_path: Path) -> Path:
    mappings = tmp_path / "mappings"
    shutil.copytree(FAKE_MAPPINGS, mappings)
    return mappings / "fake"


def test_the_rules_carry_the_number_of_mapped_rows_they_were_applied_to(tmp_path: Path) -> None:
    env, _ = fake_environment(tmp_path)
    outcome = run_all(env)
    rules = {
        (r["source_table"], r["source_field"]): r
        for r in rows(env.canonical_dir / "fake" / "qual.mapping_rule.parquet")
    }
    # six species rows; the placeholder column is always absent, so its rule is applied to none
    assert rules[("species", "name")]["applied_rows"] == 6
    assert rules[("species", "aliases")]["applied_rows"] >= 1
    assert rules[("species", "placeholder")]["applied_rows"] == 0
    # only the curves whose blocks wrote records count: the held ones and the one nothing was
    # emitted for do not
    mapped = outcome.coverage.counts["curves"]  # type: ignore[union-attr]
    assert rules[("curves", "T_r")]["applied_rows"] == mapped["mapped_with_loss"] == 3
    assert rules[("curves", "p_bar")]["applied_rows"] == 3
    assert rules[("coefficients", "n")]["applied_rows"] == 6
    # a rule that is not a value rule states no count
    for key in (
        ("species", ""),
        ("curves", "partition:pressure"),
        ("curves", "derivation:pressure"),
        ("curves", "constant:validity_region.kind"),
        ("curves", "species"),
    ):
        assert rules[key]["applied_rows"] is None, key


def test_a_mapping_that_stops_emitting_a_declared_target_is_refused_naming_the_rule(
    tmp_path: Path,
) -> None:
    directory = mapping_copy(tmp_path)
    source = (directory / "mapping.py").read_text()
    stopped = source.replace(
        "            for alias in row[\"aliases\"] or []:  # type: ignore[attr-defined]\n"
        "                emit.assertion(entity, \"aliases\", alias)\n",
        "",
    )
    assert stopped != source
    (directory / "mapping.py").write_text(stopped)
    env, _ = fake_environment(tmp_path / "work", mappings=directory.parent)
    runner.run_identity(env, "fake")
    resolve_all(env, decl=real_declaration())
    with pytest.raises(MapError) as refused:
        runner.run_records(env, "fake")
    message = str(refused.value)
    assert "1 declared value rule(s) were applied to no row" in message
    assert (
        "table species column aliases: the value rule for `identity_assertion.value` was applied "
        "to none of the 6 mapped row(s) of the table" in message
    )
    assert "optional = true" in message
    assert not (env.canonical_dir / "fake" / "manifest.json").exists(), (
        "the run is refused before its records are installed"
    )


def test_an_optional_rule_applied_to_no_row_is_reported_and_the_run_completes(
    tmp_path: Path,
) -> None:
    env, _ = fake_environment(tmp_path)
    outcome = run_all(env)
    assert outcome.unused_optional == (
        "table species column placeholder: the value rule for `identity_assertion.value` was "
        "applied to none of the 6 mapped row(s) of the table",
    )
    assert runner.coverage_lines(outcome.coverage)  # type: ignore[arg-type]


def test_the_same_rule_not_declared_optional_refuses_the_run(tmp_path: Path) -> None:
    directory = mapping_copy(tmp_path)
    path = directory / "mapping.toml"
    text = path.read_text()
    assert text.count("optional = true\n") == 1
    path.write_text(text.replace("optional = true\n", ""))
    env, _ = fake_environment(tmp_path / "work", mappings=directory.parent)
    runner.run_identity(env, "fake")
    resolve_all(env, decl=real_declaration())
    with pytest.raises(MapError, match="table species column placeholder: the value rule"):
        runner.run_records(env, "fake")


def test_a_table_with_no_mapped_row_has_no_rule_to_apply(tmp_path: Path) -> None:
    env, _ = fake_environment(tmp_path)
    outcome = run_all(env)
    spec = runner.prepare(env, "fake").spec
    result = outcome.coverage
    assert result is not None
    none_applied: dict[tuple[str, str], int] = {}
    refused, reported = coverage.check_rule_use(spec, result, none_applied)
    assert any("table curves column T_r" in line for line in refused)
    # with every row of the curves table out of scope there is no mapped row, so no rule is owed
    emptied = {**result.counts, "curves": {"deferred": 8}, "coefficients": {"deferred": 10}}
    result.counts = emptied
    refused, _ = coverage.check_rule_use(spec, result, none_applied)
    assert not any("table curves" in line or "table coefficients" in line for line in refused)


def test_optional_belongs_to_a_value_rule(tmp_path: Path) -> None:
    env, _ = fake_environment(tmp_path)
    data = tomllib.loads((env.mappings_dir / "fake" / "mapping.toml").read_text())
    data["tables"]["curves"]["fields"]["species"]["optional"] = True
    prepared = runner.prepare(env, "fake")
    problems = validate(
        msgspec.convert(data, MappingSpec),
        real_declaration(),
        prepared.tables.schemas,
        source="fake",
    )
    assert any(
        "curves column species: `optional` belongs to a value rule" in problem
        for problem in problems
    )
