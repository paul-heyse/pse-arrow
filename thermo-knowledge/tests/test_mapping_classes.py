# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""A mapping states the class of each source entity: one class for a scope, or a column and a
value-to-class table; a scope that has defined mixtures states their definition and basis."""

from __future__ import annotations

import json
import shutil
import tomllib
from collections.abc import Callable
from pathlib import Path

import msgspec
import pytest

from mapping_support import FAKE_MAPPINGS, fake_environment, real_declaration, rows
from thermo_knowledge.canonical.environment import Environment
from thermo_knowledge.mapping import claims, runner
from thermo_knowledge.mapping.spec import MappingSpec, validate
from thermo_knowledge.mapping.staged import MappingError
from thermo_knowledge.resolve.command import resolve_all

BY_COLUMN = """
[scopes.species.class_by]
column = "cas"
default = "species"

[[scopes.species.class_by.rules]]
pattern = '.+\\.[Pp][Pp][Ff]'
class = "defined_mixture"
reason = "A name ending .ppf is a blend file."

[scopes.species.mixture]
definition = "by_definition"
basis = "mole"
reason = "The fixture's blends are fixed by definition, in mole fractions."
"""


def with_classes(tmp_path: Path, edit: Callable[[str], str]) -> tuple[Environment, Path]:
    mappings = tmp_path / "mappings"
    shutil.copytree(FAKE_MAPPINGS, mappings)
    path = mappings / "fake" / "mapping.toml"
    path.write_text(edit(path.read_text()))
    env, _ = fake_environment(tmp_path / "work", mappings=mappings)
    return env, path


def by_column(text: str) -> str:
    return text.replace('class = "species"\n', "", 1).replace(
        "[parameterizations.curves]", BY_COLUMN.strip() + "\n\n[parameterizations.curves]", 1
    )


def test_a_value_to_class_table_gives_each_entity_its_class(tmp_path: Path) -> None:
    env, _ = with_classes(tmp_path, by_column)
    runner.run_identity(env, "fake")
    directory = env.canonical_dir / "fake" / claims.IDENTITY_DIR
    found = {c.local_key: c for c in claims.read_entity_claims(directory)}
    assert {k: c.entity_class for k, c in found.items()} == {
        "Ethanol": "species",
        "Water": "species",
        "Mystery": "defined_mixture",
        "Conflicted": "species",
        "Plain": "species",
        "Lonely": "species",
    }
    mystery = found["Mystery"]
    assert (mystery.mixture_definition, mystery.mole_basis) == ("by_definition", True)
    assert all(
        c.mixture_definition is None and c.mole_basis is None
        for k, c in found.items()
        if k != "Mystery"
    )
    resolve_all(env, decl=real_declaration())
    resolution = {r["local_key"]: r for r in rows(env.resolution_dir / "tk.source_entity.parquet")}
    mixtures = {r["id"] for r in rows(env.resolution_dir / "tk.defined_mixture.parquet")}
    assert resolution["Mystery"]["target"] in mixtures
    assert (resolution["Mystery"]["status"], resolution["Mystery"]["rule"]) == (
        "unresolved",
        "provisional",
    ), "the source states no composition, so the blend is a provisional defined mixture"
    species = {r["id"] for r in rows(env.resolution_dir / "tk.species.parquet")}
    assert resolution["Mystery"]["target"] not in species
    report = json.loads((env.resolution_dir / "report.json").read_text())
    assert report["totals"]["class"] == {"defined_mixture": 1, "species": 5}


def test_a_scope_of_one_class_states_it_and_the_fixture_mapping_is_valid(tmp_path: Path) -> None:
    env, _ = fake_environment(tmp_path)
    data = tomllib.loads((env.mappings_dir / "fake" / "mapping.toml").read_text())
    assert data["scopes"]["species"]["class"] == "species"
    runner.run_identity(env, "fake")
    classes = {
        c.entity_class
        for c in claims.read_entity_claims(env.canonical_dir / "fake" / claims.IDENTITY_DIR)
    }
    assert classes == {"species"}


def problems_of(env: Environment, mutate: Callable[[dict[str, object]], None]) -> list[str]:
    data = tomllib.loads((env.mappings_dir / "fake" / "mapping.toml").read_text())
    mutate(data)
    prepared = runner.prepare(env, "fake")
    return validate(
        msgspec.convert(data, MappingSpec),
        real_declaration(),
        prepared.tables.schemas,
        source="fake",
    )


def scope(data: dict[str, object]) -> dict[str, object]:
    return data["scopes"]["species"]  # type: ignore[index,return-value]


def by_rule(data: dict[str, object], **changes: object) -> None:
    scope(data).pop("class")
    scope(data)["class_by"] = {
        "column": "cas",
        "default": "species",
        "rules": [
            {"pattern": "x", "class": "defined_mixture", "reason": "why", **changes},
        ],
    }
    scope(data)["mixture"] = {"definition": "by_definition", "basis": "mole", "reason": "why"}


@pytest.mark.parametrize(
    ("mutate", "problem"),
    [
        (lambda d: scope(d).pop("class"), "state exactly one of `class`"),
        (
            lambda d: scope(d).update(
                class_by={"column": "cas", "default": "species", "rules": []}
            ),
            "state exactly one of `class`",
        ),
        (lambda d: scope(d).update({"class": "mineral"}), "`mineral` is not an entity class"),
        (
            lambda d: scope(d).update({"class": "pseudo_component"}),
            "class `pseudo_component` has no resolution rules yet",
        ),
        (
            lambda d: scope(d).update({"class": "defined_mixture"}),
            "the scope has defined mixtures, so it states `mixture`",
        ),
        (
            lambda d: scope(d).update(
                mixture={"definition": "by_definition", "basis": "mole", "reason": "x"}
            ),
            "`mixture` belongs to a scope that has defined mixtures",
        ),
        (
            lambda d: (
                by_rule(d),
                d["scopes"]["species"]["mixture"].update(definition="by_magic"),  # type: ignore[index]
            ),
            "mixture definition `by_magic` is not a `mixture_definition`",
        ),
        (
            lambda d: by_rule(d, pattern=None, values=[]),
            "state exactly one of `values` and `pattern`",
        ),
        (lambda d: by_rule(d, pattern="("), "`pattern` is not a regular expression"),
        (lambda d: by_rule(d, reason=" "), "states its `reason`"),
        (lambda d: by_rule(d, **{"class": "pseudo_component"}), "has no resolution rules yet"),
        (
            lambda d: (by_rule(d), scope(d)["class_by"].update(column="nothing")),  # type: ignore[attr-defined]
            "class_by column `nothing` is not a column of the table",
        ),
    ],
)
def test_a_scope_class_declaration_the_resolver_cannot_use_is_refused(
    tmp_path: Path, mutate: Callable[[dict[str, object]], object], problem: str
) -> None:
    env, _ = fake_environment(tmp_path)
    assert any(problem in p for p in problems_of(env, mutate)), problem


def test_only_an_entity_of_class_species_form_states_an_aggregation(tmp_path: Path) -> None:
    env, _ = fake_environment(tmp_path / "a")
    path = env.mappings_dir / "fake" / "mapping.py"
    path.write_text(
        path.read_text().replace(
            'entity = emit.source_entity("species")',
            'entity = emit.source_entity("species", aggregation="gas")',
        )
    )
    with pytest.raises(
        MappingError, match="only an entity of class `species_form` states an aggregation"
    ):
        runner.run_identity(env, "fake")

    forms, _ = with_classes(
        tmp_path / "b", lambda text: text.replace('class = "species"', 'class = "species_form"', 1)
    )
    with pytest.raises(MappingError, match="is of class `species_form`"):
        runner.run_identity(forms, "fake")
    path = forms.mappings_dir / "fake" / "mapping.py"
    path.write_text(
        path.read_text().replace(
            'entity = emit.source_entity("species")',
            'entity = emit.source_entity("species", aggregation="gas")',
        )
    )
    assert runner.run_identity(forms, "fake", force=True).status == "mapped"
    assert {
        c.aggregation
        for c in claims.read_entity_claims(forms.canonical_dir / "fake" / claims.IDENTITY_DIR)
    } == {"gas"}
