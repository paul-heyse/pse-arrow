# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The curated decisions of the thermochemical sources: the file rejects exactly the entities
whose unit lists several differently named species under one formula (`identity_support`), so a
change to a mapping that moves an entity into or out of such a group makes this test fail until
the file is regenerated."""

from __future__ import annotations

from pathlib import Path

import pytest
from identity_support import entries, rejections

from thermo_knowledge import config
from thermo_knowledge.mapping import runner
from thermo_knowledge.canonical.environment import Environment
from thermo_knowledge.resolve import decisions
from mapping_support import real_declaration

CARRIERS = ("cantera", "nasa_cea", "janaf")


def _staged(carrier: str) -> bool:
    return (config.staged_dir() / carrier).is_dir()


pytestmark = pytest.mark.skipif(
    not all(_staged(carrier) for carrier in CARRIERS),
    reason="Cantera, NASA CEA and JANAF are not acquired and staged here",
)


@pytest.fixture(scope="module")
def found(tmp_path_factory: pytest.TempPathFactory):  # noqa: ANN201
    env = Environment(canonical_dir=tmp_path_factory.mktemp("canonical"))
    decl = real_declaration()
    for carrier in CARRIERS:
        assert runner.run_identity(env, carrier, decl=decl).status == "mapped"
    return entries(env.canonical_dir, CARRIERS)


def test_every_entity_a_formula_does_not_identify_is_rejected_and_no_other(found) -> None:  # noqa: ANN001
    wanted = {key for key in rejections(found)}
    parsed, _ = decisions.load()
    rejected = {
        key for key in parsed.reject if key[0] in CARRIERS and key in {e for e in wanted} | set(parsed.reject)
    }
    assert wanted == rejected, (
        f"{len(wanted - rejected)} entities missing a decision, {len(rejected - wanted)} decisions "
        "without a group: regenerate identity/decisions.toml from tests/identity_support.py"
    )


def test_every_rejection_states_the_names_that_are_its_evidence(found) -> None:  # noqa: ANN001
    parsed, _ = decisions.load()
    groups = rejections(found)
    for key, reason in parsed.reject.items():
        if key in groups:
            _, names = groups[key]
            assert f"{len(names)} differently named species" in reason


def test_the_groups_have_several_species_and_share_a_form(found) -> None:  # noqa: ANN001
    for entry, names in rejections(found).values():
        assert len(names) > 1
        assert entry.species in names


def test_a_unit_of_one_name_is_never_rejected(found) -> None:  # noqa: ANN001
    groups = rejections(found)
    by_unit: dict[tuple[object, ...], set[str]] = {}
    for entry in found:
        by_unit.setdefault((entry.unit, entry.form), set()).add(entry.species)
    for entry in found:
        if len(by_unit[(entry.unit, entry.form)]) == 1:
            assert (entry.carrier, entry.scope, entry.key) not in groups
