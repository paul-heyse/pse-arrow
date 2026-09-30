# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Convention facts have one home, the convention set, and a form reads them from it.

A form declares the facts it reads (`conventions = ["gas_constant"]`, `convention.gas_constant`
in its expressions). A parameter source gives the facts of the parameterizations that supplied the
sets an evaluation read; an evaluation that combines parameterizations whose facts differ, or
that reads a fact a parameterization does not state, is refused; the structural check
`parameterization_has_conventions` states the second rule over the database.
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
from qualify_support import fixture_declaration

from thermo_knowledge import config, db
from thermo_knowledge.build import build_database
from thermo_knowledge.canonical.values import Quantity
from thermo_knowledge.canonical.writer import CanonicalWriter, FamilyRow
from thermo_knowledge.declaration import Declaration
from thermo_knowledge.expression.evaluate import EvaluationRefusal, bind
from thermo_knowledge.expression.parameters import FormChoice, InMemorySource
from thermo_knowledge.qualify.source import DatabaseSource, SubformBinding
from thermo_knowledge.testing import TestDatabase
from thermo_knowledge.verify.checks import VERIFY_DIR, load_checks
from thermo_knowledge.verify.run import run_check

CHECK = {c.target: c for c in load_checks(config.TREE_DIR / VERIFY_DIR)[0]}[
    "parameterization_has_conventions"
]
R = 8.314462618
R_OTHER = 8.3145
T = np.array([250.0, 300.0, 444.4, 900.0])
WATER = dict(
    a1=3.386,
    a2=3.47e-3,
    a3=-6.35e-6,
    a4=6.97e-9,
    a5=-2.50e-12,
    a6=-30209.0,
    a7=2.59,
)
OXYGEN = dict(a1=3.2, a2=1.1e-3, a3=-2.1e-6, a4=2.0e-9, a5=-7.0e-13, a6=-1000.0, a7=5.0)


def nasa7(gas_constant: float, a: dict[str, float], temperature: np.ndarray) -> np.ndarray:
    """cp, h and s of one NASA-7 piece, from the textbook polynomials."""
    t = temperature
    cp = gas_constant * (a["a1"] + a["a2"] * t + a["a3"] * t**2 + a["a4"] * t**3 + a["a5"] * t**4)
    h = (
        gas_constant
        * t
        * (
            a["a1"]
            + a["a2"] * t / 2
            + a["a3"] * t**2 / 3
            + a["a4"] * t**3 / 4
            + a["a5"] * t**4 / 5
            + a["a6"] / t
        )
    )
    s = gas_constant * (
        a["a1"] * np.log(t)
        + a["a2"] * t
        + a["a3"] * t**2 / 2
        + a["a4"] * t**3 / 3
        + a["a5"] * t**4 / 4
        + a["a7"]
    )
    return np.array([cp, h, s])


def piece(a: dict[str, float]) -> FamilyRow:
    return FamilyRow(
        {"n": 1},
        {
            "T_low": Quantity(200.0, "K"),
            "T_high": Quantity(1000.0, "K"),
            "a1": a["a1"],
            "a2": Quantity(a["a2"], "1/K"),
            "a3": Quantity(a["a3"], "1/K**2"),
            "a4": Quantity(a["a4"], "1/K**3"),
            "a5": Quantity(a["a5"], "1/K**4"),
            "a6": Quantity(a["a6"], "K"),
            "a7": a["a7"],
        },
    )


@dataclass
class World:
    decl: Declaration
    database: TestDatabase
    ids: dict[str, uuid.UUID]


def fill(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    published = origin("a.json#/9", "published")
    gas = next(e.id for e in decl.entities if e.kind == "aggregation" and e.name == "gas")
    for n, name in enumerate(("water", "oxygen")):
        species = w.kind(
            "species", {"canonical_key": name, "label": name}, origins=[origin(f"a.json#/{n}")]
        )
        ids[f"species_{name}"] = species
        ids[f"form_{name}"] = w.kind(
            "species_form",
            {
                "canonical_key": f"{name} gas",
                "label": name,
                "species": species,
                "aggregation": gas,
            },
            origins=[origin(f"a.json#/{n}")],
        )

    def parameterization(key: str, convention: uuid.UUID | None) -> uuid.UUID:
        found = w.kind(
            "parameterization",
            {
                "key": key,
                "revision": "1",
                "title": key,
                "coherence": "independent_records",
                "convention_set": convention,
            },
            origins=[published],
        )
        ids[key] = found
        return found

    def conventions(key: str, **facts: Quantity) -> uuid.UUID:
        return w.kind(
            "convention_set",
            {"key": key, "revision": "1", "temperature_scale": "its_90", **facts},
            origins=[published],
        )

    base = conventions("base", gas_constant=Quantity(R, "J/(mol*K)"))
    other = conventions("other", gas_constant=Quantity(R_OTHER, "J/(mol*K)"))
    same = conventions("same", gas_constant=Quantity(R, "J/(mol*K)"))
    silent = conventions("silent")
    nasa = {
        "p1": parameterization("p1", base),
        "p2": parameterization("p2", other),
        "p3": parameterization("p3", same),
        "p4": parameterization("p4", None),
        "p5": parameterization("p5", silent),
    }
    for key, param in nasa.items():
        for name, coefficients in (("water", WATER), ("oxygen", OXYGEN)):
            w.parameter_set(
                parameterization=param,
                slot_group="nasa7.pure",
                subjects=[ids[f"form_{name}"]],
                slots={},
                families={"piece": [piece(coefficients)]},
                origins=[published],
            )
    # a form over a set of components, with weights held by three parameterizations: the first two
    # hold both components, the third only oxygen
    sums = {
        "g1": parameterization("g1", base),
        "g2": parameterization("g2", other),
        "g3": parameterization("g3", same),
    }
    for key, param in sums.items():
        for name, weight in (("water", 2.0), ("oxygen", 3.0)):
            if key == "g3" and name == "water":
                continue
            w.parameter_set(
                parameterization=param,
                slot_group="qfix_gas_sum.pure",
                subjects=[ids[f"species_{name}"]],
                slots={"w": weight},
                origins=[published],
            )


@pytest.fixture(scope="module")
def world(tmp_path_factory: pytest.TempPathFactory) -> Iterator[World]:
    decl = fixture_declaration(tmp_path_factory.mktemp("conventions-declaration"))
    canonical = tmp_path_factory.mktemp("conventions-canonical")
    ids: dict[str, uuid.UUID] = {}
    write_source(
        canonical, "src", lambda w: fill(w, decl, ids), decl=decl, declaration=fingerprint(decl)
    )
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


def source(world: World, conn: psycopg.Connection, *keys: str, **options: object) -> DatabaseSource:
    return DatabaseSource(conn, world.decl, [world.ids[key] for key in keys], **options)  # type: ignore[arg-type]


def evaluate_nasa(world: World, found: DatabaseSource | InMemorySource, name: str) -> np.ndarray:
    bound = bind(world.decl, "nasa7", source=found, roles={"i": str(world.ids[f"form_{name}"])})
    return np.array([bound.evaluate(output, T=T) for output in ("cp", "h", "s")])


# -- the gas constant comes from the convention set ---------------------------------------------------


def test_the_nasa_forms_hold_no_gas_constant_of_their_own() -> None:
    decl = _declaration()
    for name in ("nasa7", "nasa9"):
        form = decl.forms[name]
        assert [group.name for group in form.slot_groups] == ["pure"]
        assert [convention.name for convention in form.conventions] == ["gas_constant"]
    assert "standard_pressure" not in {a.name for a in decl.kinds["convention_set"].attributes}


def test_nasa7_takes_the_gas_constant_from_the_convention_set(
    world: World, conn: psycopg.Connection
) -> None:
    got = evaluate_nasa(world, source(world, conn, "p1"), "water")
    np.testing.assert_allclose(got, nasa7(R, WATER, T), rtol=1e-13, atol=0.0)
    # the other parameterization built its coefficients with another gas constant: what the
    # evaluation gives follows the convention set, not an ambient value
    other = evaluate_nasa(world, source(world, conn, "p2"), "water")
    np.testing.assert_allclose(other, nasa7(R_OTHER, WATER, T), rtol=1e-13, atol=0.0)
    np.testing.assert_allclose(other / got, R_OTHER / R, rtol=1e-13)


def test_the_in_memory_source_gives_the_convention_facts_it_is_given() -> None:
    decl = _declaration()
    rows = {(1,): {
        "T_low": 200.0, "T_high": 1000.0, "a1": WATER["a1"], "a2": WATER["a2"],
        "a3": WATER["a3"], "a4": WATER["a4"], "a5": WATER["a5"], "a6": WATER["a6"],
        "a7": WATER["a7"],
    }}
    held = InMemorySource(families={("nasa7.pure", "piece", ("water",)): rows}, conventions={"gas_constant": R})
    bound = bind(decl, "nasa7", source=held, roles={"i": "water"})
    got = np.array([bound.evaluate(output, T=T) for output in ("cp", "h", "s")])
    np.testing.assert_allclose(got, nasa7(R, WATER, T), rtol=1e-13, atol=0.0)
    assert sorted(bound.parameters.values()).count(R) == 1, "the fact is one stored value"


def test_the_fact_is_one_symbol_passed_as_an_argument() -> None:
    decl = _declaration()
    rows = {(1,): {
        "T_low": 200.0, "T_high": 1000.0, "a1": 1.0, "a2": 0.0, "a3": 0.0, "a4": 0.0, "a5": 0.0,
        "a6": 0.0, "a7": 0.0,
    }}
    first = bind(
        decl, "nasa7",
        source=InMemorySource(families={("nasa7.pure", "piece", ("x",)): rows}, conventions={"gas_constant": 8.0}),
        roles={"i": "x"},
    )
    second = bind(
        decl, "nasa7",
        source=InMemorySource(families={("nasa7.pure", "piece", ("x",)): rows}, conventions={"gas_constant": 9.0}),
        roles={"i": "x"},
        cache=first.cache,
    )
    np.testing.assert_allclose(first.evaluate("cp", T=T), 8.0)
    compiled = first.cache.compilations
    np.testing.assert_allclose(second.evaluate("cp", T=T), 9.0)
    assert first.cache.compilations == compiled, "values are not part of the structure"


# -- a parameterization must state what a form reads ----------------------------------------------------


def test_the_check_flags_the_parameterizations_that_state_no_gas_constant(
    world: World, conn: psycopg.Connection
) -> None:
    result = run_check(conn, CHECK)
    assert result.error is None
    assert sorted(result.ids) == sorted(str(world.ids[key]) for key in ("p4", "p5"))
    reasons = {row[0]: row[-1] for row in result.rows}
    assert reasons[str(world.ids["p4"])] == "has no convention set"
    assert reasons[str(world.ids["p5"])] == "its convention set states no gas_constant"


def test_the_evaluator_refuses_a_parameterization_that_states_no_gas_constant(
    world: World, conn: psycopg.Connection
) -> None:
    for key, reason in (("p4", "has no convention set"), ("p5", "states no `gas_constant`")):
        found = source(world, conn, key)
        bound = bind(
            world.decl, "nasa7", source=found, roles={"i": str(world.ids["form_water"])}
        )
        with pytest.raises(EvaluationRefusal, match=rf"`{key}@1`.*{reason}.*`gas_constant`"):
            bound.evaluate("cp", T=T)


# -- parameterizations are not combined when the facts differ ----------------------------------------------


def gas_sum(world: World, found: DatabaseSource, names: list[str]) -> np.ndarray:
    members = [str(world.ids[f"species_{name}"]) for name in names]
    bound = bind(world.decl, "qfix_gas_sum", source=found, sets={"components": members})
    return bound.evaluate("y", T=T)


def test_sets_drawn_from_two_parameterizations_with_different_gas_constants_are_refused(
    world: World, conn: psycopg.Connection
) -> None:
    # g3 holds no weight for water, so water comes from g2 and oxygen from g3
    found = source(world, conn, "g3", "g2")
    with pytest.raises(EvaluationRefusal) as refused:
        gas_sum(world, found, ["water", "oxygen"])
    message = str(refused.value)
    assert "`gas_constant` differs" in message
    assert "`g2@1` has 8.3145" in message and "`g3@1` has 8.314462618" in message


def test_sets_drawn_from_two_parameterizations_with_equal_gas_constants_are_accepted(
    world: World, conn: psycopg.Connection
) -> None:
    found = source(world, conn, "g3", "g1")
    # oxygen comes from g3 and water from g1: R is equal in both convention sets
    np.testing.assert_allclose(
        gas_sum(world, found, ["water", "oxygen"]), np.full_like(T, R * (2.0 + 3.0)), rtol=1e-14
    )


def test_a_parameterization_that_supplied_no_set_is_not_compared(
    world: World, conn: psycopg.Connection
) -> None:
    # g2 is listed but every set comes from g1: only the parameterizations that supplied a set count
    found = source(world, conn, "g1", "g2")
    np.testing.assert_allclose(
        gas_sum(world, found, ["water", "oxygen"]), np.full_like(T, R * 5.0), rtol=1e-14
    )


def heat_pair(world: World, conn: psycopg.Connection, first: str, second: str) -> np.ndarray:
    found = source(
        world,
        conn,
        "p1",
        subforms={
            "qfix_heat_pair_form.first": [SubformBinding("nasa7", (world.ids[first],))],
            "qfix_heat_pair_form.second": [SubformBinding("nasa7", (world.ids[second],))],
        },
    )
    roles = {"i": str(world.ids["form_water"]), "j": str(world.ids["form_oxygen"])}
    return bind(world.decl, "qfix_heat_pair_form", source=found, roles=roles).evaluate("cp", T=T)


def test_forms_called_through_different_parameterizations_must_agree_on_the_gas_constant(
    world: World, conn: psycopg.Connection
) -> None:
    with pytest.raises(EvaluationRefusal, match=r"`gas_constant` differs.*`p1@1`.*`p2@1`"):
        heat_pair(world, conn, "p1", "p2")
    agreed = heat_pair(world, conn, "p1", "p3")
    want = nasa7(R, WATER, T)[0] + nasa7(R, OXYGEN, T)[0]
    np.testing.assert_allclose(agreed, want, rtol=1e-13, atol=0.0)


def test_in_memory_sources_with_different_facts_are_refused_and_with_equal_facts_accepted() -> None:
    decl = _declaration()

    def rows(a: dict[str, float]) -> dict[tuple[int, ...], dict[str, float]]:
        return {(1,): {"T_low": 200.0, "T_high": 1000.0, **a}}

    def nasa(name: str, a: dict[str, float], fact: float) -> FormChoice:
        return FormChoice(
            "nasa7",
            InMemorySource(
                families={("nasa7.pure", "piece", (name,)): rows(a)},
                conventions={"gas_constant": fact},
                parameterization=f"{name} set",
            ),
        )

    def evaluate(second_fact: float) -> np.ndarray:
        held = InMemorySource(
            subforms={
                ("qfix_heat_pair_form.first", ()): (nasa("water", WATER, R),),
                ("qfix_heat_pair_form.second", ()): (nasa("oxygen", OXYGEN, second_fact),),
            }
        )
        bound = bind(decl, "qfix_heat_pair_form", source=held, roles={"i": "water", "j": "oxygen"})
        return bound.evaluate("cp", T=T)

    with pytest.raises(EvaluationRefusal, match=r"`water set` has 8\.314462618.*`oxygen set` has 8\.3145"):
        evaluate(R_OTHER)
    np.testing.assert_allclose(
        evaluate(R), nasa7(R, WATER, T)[0] + nasa7(R, OXYGEN, T)[0], rtol=1e-13, atol=0.0
    )


# -- the convention facts are reified ---------------------------------------------------------------


def test_the_facts_a_form_reads_are_reified_in_meta(world: World, conn: psycopg.Connection) -> None:
    assert conn.execute(
        "SELECT form, name, kind FROM meta.form_convention WHERE name = 'gas_constant' ORDER BY form"
    ).fetchall() == [
        (form, "gas_constant", "convention_set")
        for form in (
            "cef_magnetic_ihj",
            "flory_huggins_excess_gibbs",
            "helmholtz_pure_fluid",
            "if97_gibbs_region1",
            "if97_gibbs_region2",
            "if97_gibbs_region5",
            "if97_helmholtz_region3",
            "log_k_van_t_hoff",
            "nasa7",
            "nasa9",
            "nrtl_excess_gibbs",
            "peng_robinson_core",
            "qfix_gas_sum",
        )
    ]
    assert conn.execute(
        "SELECT form, name, kind FROM meta.form_convention WHERE name = 'avogadro_constant' ORDER BY form"
    ).fetchall() == [
        ("chapman_enskog_transport", "avogadro_constant", "convention_set"),
        ("pcsaft_association", "avogadro_constant", "convention_set"),
    ]
    assert conn.execute(
        "SELECT kind FROM meta.framework_role WHERE role = 'convention_set'"
    ).fetchone() == ("convention_set",)


def _declaration() -> Declaration:
    import tempfile

    if not _DECLARATION:
        _DECLARATION.append(
            fixture_declaration(Path(tempfile.mkdtemp(prefix="tk-conventions-declaration-")))
        )
    return _DECLARATION[0]


_DECLARATION: list[Declaration] = []
