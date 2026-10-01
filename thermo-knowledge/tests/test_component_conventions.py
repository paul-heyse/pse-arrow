# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""A convention fact read per component (plan 24, packet TK2g, proof fixture 4; the teqp multifluid
mechanism the dispositions assume).

A mixture gas constant R = sum x_i R_i averages the gas constants of its components, whose
pure-fluid parameterisations were built with different editions of the constant. A form that reads
`convention.gas_constant` refuses sets from parameterisations that differ; a form that declares
the fact in `component_conventions` reads `convention.gas_constant[i]`, the fact of the
parameterisation that supplied the set of its slot group for component `i`. Only where a form
declares it is the difference allowed.
"""

from __future__ import annotations

import uuid
from collections.abc import Iterator
from dataclasses import dataclass
from pathlib import Path

import numpy as np
import psycopg
import pytest

from build_support import fingerprint, inputs_of, write_source
from mapping_support import origin
from mechanisms_support import broken, mechanism_declaration, replace
from thermo_knowledge import config, db
from thermo_knowledge.build import build_database
from thermo_knowledge.canonical.values import Quantity
from thermo_knowledge.canonical.writer import CanonicalWriter
from thermo_knowledge.declaration import Code, Declaration
from thermo_knowledge.expression.canonical import evaluation_hash
from thermo_knowledge.expression.evaluate import EvaluationRefusal, bind
from thermo_knowledge.qualify.source import DatabaseSource
from thermo_knowledge.testing import TestDatabase
from thermo_knowledge.verify.checks import VERIFY_DIR, load_checks
from thermo_knowledge.verify.run import run_check, run_checks

CHECKS = {c.target: c for c in load_checks(config.TREE_DIR / VERIFY_DIR)[0]}
AVERAGED, SINGLE = "averaged_gas_constant_fixture", "single_gas_constant_fixture"
R_OLD, R_NEW = 8.314472, 8.314462618  # CODATA 2006 and 2018


@dataclass
class World:
    decl: Declaration
    database: TestDatabase
    ids: dict[str, uuid.UUID]


def at(tag: str) -> list:  # noqa: ANN401
    return [origin(f"a.json#/{tag}", "published")]


def write_world(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    species = {
        name: w.kind("species", {"canonical_key": name, "label": name}, origins=at(f"s-{name}"))
        for name in ("a", "b", "c", "d")
    }
    ids.update({f"species_{name}": value for name, value in species.items()})
    editions = {
        "old": {"gas_constant": Quantity(R_OLD, "J/(mol*K)")},
        "new": {"gas_constant": Quantity(R_NEW, "J/(mol*K)")},
        "silent": {"boltzmann_constant": Quantity(1.380649e-23, "J/K")},
    }
    for edition, facts in editions.items():
        conventions = w.kind(
            "convention_set",
            {"key": f"codata-{edition}", "revision": "1", "temperature_scale": "its_90", **facts},
            origins=at(f"conventions-{edition}"),
        )
        p = w.kind(
            "parameterization",
            {
                "key": f"pure-{edition}",
                "revision": "1",
                "title": edition,
                "coherence": "independent_records",
                "convention_set": conventions,
            },
            origins=at(f"p-{edition}"),
        )
        ids[f"p_{edition}"] = p
        members = {"old": ("a", "c"), "new": ("b", "d"), "silent": ("c",)}[edition]
        for form in (AVERAGED, SINGLE):
            for name in members:
                w.parameter_set(
                    parameterization=p,
                    slot_group=f"{form}.pure",
                    subjects=[species[name]],
                    slots={"T_c": Quantity(300.0, "K")},
                    origins=at(f"{form}-{edition}-{name}"),
                )


@pytest.fixture(scope="module")
def decl(tmp_path_factory: pytest.TempPathFactory) -> Declaration:
    return mechanism_declaration(tmp_path_factory.mktemp("component-declaration"))


@pytest.fixture(scope="module")
def world(decl: Declaration, tmp_path_factory: pytest.TempPathFactory) -> Iterator[World]:
    canonical = tmp_path_factory.mktemp("component-canonical")
    ids: dict[str, uuid.UUID] = {}
    write_source(
        canonical,
        "src",
        lambda w: write_world(w, decl, ids),
        decl=decl,
        declaration=fingerprint(decl),
    )
    with TestDatabase() as database:
        build_database(database.url, decl, inputs_of(canonical))
        yield World(decl, database, ids)


@pytest.fixture
def conn(world: World) -> Iterator[psycopg.Connection]:
    connection = db.connect(world.database.url)
    connection.execute("SELECT 1")
    try:
        yield connection
    finally:
        connection.rollback()
        connection.close()


def mixture(
    world: World,
    conn: psycopg.Connection,
    form: str,
    editions: list[str],
    composition: dict[str, float],
    output: str = "R",
) -> float:
    source = DatabaseSource(conn, world.decl, [world.ids[f"p_{edition}"] for edition in editions])
    ids = {name: str(world.ids[f"species_{name}"]) for name in composition}
    bound = bind(world.decl, form, source=source, sets={"components": list(ids.values())})
    found = bound.evaluate(output, x={ids[name]: np.array([x]) for name, x in composition.items()})
    return float(np.asarray(found).reshape(-1)[0])


def test_the_fixture_loads_and_only_the_silent_parameterisation_fails_the_convention_check(
    world: World, conn: psycopg.Connection
) -> None:
    results = run_checks(conn, list(CHECKS.values()))
    failing = [
        (r.check.target, r.error)
        for r in results
        if not r.passed and r.check.target != "parameterization_has_conventions"
    ]
    assert failing == []
    flagged = run_check(conn, CHECKS["parameterization_has_conventions"])
    assert {row[0] for row in flagged.rows} == {str(world.ids["p_silent"])}


def test_a_fact_read_per_component_is_reified_with_its_group_and_set(
    world: World, conn: psycopg.Connection
) -> None:
    rows = conn.execute(
        "SELECT form, name, slot_group, over FROM meta.form_convention ORDER BY form"
    ).fetchall()
    assert (
        "averaged_gas_constant_fixture",
        "gas_constant",
        "averaged_gas_constant_fixture.pure",
        "components",
    ) in rows
    assert ("single_gas_constant_fixture", "gas_constant", None, None) in rows


def test_the_mixture_gas_constant_averages_the_constants_of_its_components(
    world: World, conn: psycopg.Connection
) -> None:
    found = mixture(world, conn, AVERAGED, ["old", "new"], {"a": 0.3, "b": 0.7})
    assert found == pytest.approx(0.3 * R_OLD + 0.7 * R_NEW, rel=1e-15)


def test_each_component_takes_its_own_parameterisation_whatever_the_order(
    world: World, conn: psycopg.Connection
) -> None:
    expected = 0.25 * R_OLD + 0.35 * R_NEW + 0.4 * R_OLD
    for editions in (["old", "new"], ["new", "old"]):
        found = mixture(world, conn, AVERAGED, editions, {"a": 0.25, "b": 0.35, "c": 0.4})
        assert found == pytest.approx(expected, rel=1e-15)


def test_components_of_one_edition_give_that_edition(
    world: World, conn: psycopg.Connection
) -> None:
    assert mixture(world, conn, AVERAGED, ["new"], {"b": 0.5, "d": 0.5}) == pytest.approx(
        R_NEW, rel=1e-15
    )


def test_the_form_that_reads_the_fact_of_the_parameterisation_still_refuses_different_editions(
    world: World, conn: psycopg.Connection
) -> None:
    with pytest.raises(
        EvaluationRefusal, match="`gas_constant` differs between the parameterizations"
    ):
        mixture(world, conn, SINGLE, ["old", "new"], {"a": 0.3, "b": 0.7}, "RT")
    found = mixture(world, conn, SINGLE, ["old"], {"a": 0.3, "c": 0.7}, "RT")
    assert found == pytest.approx(R_OLD * 300.0, rel=1e-14)


def test_the_form_that_declares_the_fact_per_component_reads_the_critical_temperatures_too(
    world: World, conn: psycopg.Connection
) -> None:
    found = mixture(world, conn, AVERAGED, ["old", "new"], {"a": 0.3, "b": 0.7}, "RT")
    assert found == pytest.approx((0.3 * R_OLD + 0.7 * R_NEW) * 300.0, rel=1e-14)


def test_a_component_whose_parameterisation_states_no_gas_constant_is_refused(
    world: World, conn: psycopg.Connection
) -> None:
    with pytest.raises(EvaluationRefusal, match=r"states no `gas_constant`.*component"):
        mixture(world, conn, AVERAGED, ["silent", "new"], {"c": 0.5, "b": 0.5})


def test_a_component_with_no_set_in_any_parameterisation_is_refused(
    world: World, conn: psycopg.Connection
) -> None:
    with pytest.raises(EvaluationRefusal, match=r"averaged_gas_constant_fixture\.pure"):
        mixture(world, conn, AVERAGED, ["old"], {"a": 0.5, "b": 0.5})


def test_the_evaluation_hash_states_which_group_names_the_convention_set_of_a_component(
    decl: Declaration,
) -> None:
    averaged, single = decl.forms[AVERAGED], decl.forms[SINGLE]
    assert evaluation_hash(averaged, "R") != evaluation_hash(single, "R")
    (fact,) = averaged.conventions
    assert (fact.group, fact.over) == ("averaged_gas_constant_fixture.pure", "components")


# -- the refusals of the declaration ------------------------------------------------------------


def refused(tmp_path: Path, edit: object) -> list:
    result = broken(tmp_path, {"forms/component_convention_forms.toml": edit})  # type: ignore[dict-item]
    assert result.declaration is None
    return list(result.diagnostics)


def test_a_fact_read_per_component_cannot_be_read_as_a_fact_of_the_parameterisation(
    tmp_path: Path,
) -> None:
    found = refused(
        tmp_path,
        replace(
            'R = "sum(x[i] * convention.gas_constant[i] for i in components)"',
            'R = "convention.gas_constant * sum(x[i] for i in components)"',
        ),
    )
    assert {d.code for d in found} == {Code.BAD_CONVENTION}
    assert "read per component of `components`" in found[0].message


def test_a_fact_of_the_parameterisation_cannot_be_read_per_component(tmp_path: Path) -> None:
    found = refused(
        tmp_path,
        replace(
            'R = "sum(x[i] * convention.gas_constant for i in components)"',
            'R = "sum(x[i] * convention.gas_constant[i] for i in components)"',
        ),
    )
    assert {d.code for d in found} == {Code.BAD_CONVENTION}
    assert "not of a component" in found[0].message


def test_a_per_component_fact_undeclared_is_refused(tmp_path: Path) -> None:
    found = refused(
        tmp_path,
        replace(
            'R = "sum(x[i] * convention.gas_constant[i] for i in components)"',
            "R = \"sum(x[i] * convention.boltzmann_constant[i] * unit('mol') for i in components)\"",
        ),
    )
    assert Code.UNKNOWN_NAME in {d.code for d in found}


def test_the_group_that_names_the_convention_set_is_a_group_of_the_form(tmp_path: Path) -> None:
    found = refused(
        tmp_path,
        replace(
            'component_conventions = { gas_constant = "pure" }',
            'component_conventions = { gas_constant = "no_such_group" }',
        ),
    )
    assert {d.code for d in found} == {Code.BAD_CONVENTION}
    assert "is not a slot group of form" in found[0].message


def test_the_group_needs_one_subject_bound_to_a_set(tmp_path: Path) -> None:
    found = refused(
        tmp_path,
        replace(
            'component_conventions = { gas_constant = "pure" }\n\n[forms.averaged_gas_constant_fixture.slot_groups.pure]\ndoc = "The pure-fluid parameters of one component."\nsubject.i = { type = "species", doc = "The component." }\nbind = { i = "components" }',
            'component_conventions = { gas_constant = "pure" }\n\n[forms.averaged_gas_constant_fixture.slot_groups.pure]\ndoc = "The pure-fluid parameters of one component."\nsubject.i = { type = "species", doc = "The component." }\nsubject.j = { type = "species", doc = "Another." }\nbind = { i = "components", j = "components" }',
        ),
    )
    assert Code.BAD_CONVENTION in {d.code for d in found}
    assert any("one subject role" in d.message for d in found)


def test_a_fact_is_not_both_a_fact_of_the_parameterisation_and_of_its_components(
    tmp_path: Path,
) -> None:
    found = refused(
        tmp_path,
        replace(
            'component_conventions = { gas_constant = "pure" }',
            'component_conventions = { gas_constant = "pure" }\nconventions = ["gas_constant"]',
        ),
    )
    assert Code.BAD_CONVENTION in {d.code for d in found}
    assert any("named twice" in d.message for d in found)
