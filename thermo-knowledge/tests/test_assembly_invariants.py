# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""What a model assembly must satisfy (plan 24, packet TK2, hard case (b), the invariants TK2g left).

An assembly is a tree of form choices, one row for each sub-form slot it fills, keyed by the path of
slot names from the root. Three rules hold it to the declared forms, each a verify check written
from `meta`:

- the chosen form implements the contract the slot accepts (`assembly_choice.form_implements_slot_contract`);
- the last segment of the path is the slot's name, and the slot belongs to the form that stands at
  the rest of the path: the root for a path of one segment, else a form chosen at the parent path
  (`assembly_choice.slot_belongs_to_form_at_path`);
- every single-valued slot a model decides is decided, at the root and below every chosen form
  (`model_assembly.single_slots_decided`).

The fixture is the assembled cubic of `test_hard_case_cubic`: one complete assembly and, for each
rule, variants that break it.
"""

from __future__ import annotations

import uuid
from collections.abc import Iterator
from dataclasses import dataclass

import psycopg
import pytest
from hard_case_support import CHECKS, at, build, failing
from mapping_support import real_declaration

from thermo_knowledge import db
from thermo_knowledge.canonical.writer import CanonicalWriter
from thermo_knowledge.declaration import Declaration
from thermo_knowledge.testing import TestDatabase
from thermo_knowledge.verify.run import run_check

ROOT = "peng_robinson_core"
COMPLETE = [
    ("alpha", "peng_robinson_core.alpha", "twu_alpha"),
    ("mixing", "peng_robinson_core.mixing", "huron_vidal_pr_mixing"),
    ("mixing/excess", "huron_vidal_pr_mixing.excess", "nrtl_excess_gibbs"),
    ("translation", "peng_robinson_core.translation", "peneloux_translation"),
]
ALPHA, MIXING, EXCESS, TRANSLATION = COMPLETE
IMPLEMENTS = "assembly_choice.form_implements_slot_contract"
BELONGS = "assembly_choice.slot_belongs_to_form_at_path"
DECIDED = "model_assembly.single_slots_decided"


@dataclass
class World:
    database: TestDatabase
    assemblies: dict[str, uuid.UUID]
    choices: dict[str, uuid.UUID]  # the choice each variant breaks the rule with


def assemble(
    w: CanonicalWriter, world: World, key: str, choices: list[tuple[str, str, str]], *, break_with: int | None = None
) -> None:
    """One assembly of `choices` (path, slot, form). The choice at position `break_with` is the one
    the variant breaks a rule with; its identifier is kept."""
    assembly = w.kind(
        "model_assembly",
        {"key": key, "revision": "1", "title": key, "root": ROOT},
        origins=at(f"assembly-{key}"),
    )
    world.assemblies[key] = assembly
    for position, (path, slot, form) in enumerate(choices):
        chosen = w.kind(
            "assembly_choice",
            {"assembly": assembly, "path": path, "ordinal": 1, "slot": slot, "form": form},
            at="a.json#/choice",
        )
        if position == break_with:
            world.choices[key] = chosen


def write_world(w: CanonicalWriter, world: World) -> None:
    assemble(w, world, "complete", COMPLETE)
    # a form of another contract in the slot of the alpha function
    assemble(w, world, "wrong-contract", [(ALPHA[0], ALPHA[1], "peneloux_translation"), *COMPLETE[1:]], break_with=0)
    # a path whose last segment is another slot than the one the choice names
    assemble(w, world, "path-names-another-slot", [("translation", ALPHA[1], "twu_alpha"), *COMPLETE[1:3]], break_with=0)
    # a slot of a form that is not the root, at the root
    assemble(w, world, "slot-of-another-form-at-the-root", [*COMPLETE, ("excess", EXCESS[1], EXCESS[2])], break_with=4)
    # a slot of the mixing rule below a path whose form is the alpha function
    assemble(w, world, "slot-below-the-wrong-form", [*COMPLETE, ("alpha/excess", EXCESS[1], EXCESS[2])], break_with=4)
    # a slot below a path at which nothing is chosen
    assemble(w, world, "slot-below-nothing", [*COMPLETE[:2], COMPLETE[3], ("unchosen/excess", EXCESS[1], EXCESS[2])], break_with=3)
    # a root slot left open
    assemble(w, world, "root-slot-open", [ALPHA, MIXING, EXCESS])
    # the slot of the chosen mixing rule left open
    assemble(w, world, "nested-slot-open", [ALPHA, MIXING, TRANSLATION])
    # nothing decided at all
    assemble(w, world, "nothing-decided", [])


@pytest.fixture(scope="module")
def decl() -> Declaration:
    return real_declaration()


@pytest.fixture(scope="module")
def world(decl: Declaration, tmp_path_factory: pytest.TempPathFactory) -> Iterator[World]:
    holder = World(None, {}, {})  # type: ignore[arg-type]
    database = build(tmp_path_factory.mktemp("assembly-invariants"), lambda w: write_world(w, holder), decl)
    holder.database = database
    try:
        yield holder
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


def flagged(conn: psycopg.Connection, target: str) -> list[str]:
    found = run_check(conn, CHECKS[target], shown=100)
    assert found.error is None, found.error
    return sorted(found.ids)


def ids(*values: uuid.UUID) -> list[str]:
    return sorted(str(v) for v in values)


# -- the rules ----------------------------------------------------------------------------------


def test_the_three_rules_are_declared_invariants_with_a_check_each(decl: Declaration) -> None:
    assert {IMPLEMENTS, BELONGS, DECIDED} <= set(CHECKS)
    declared = {
        (owner, r.name, r.enforced)
        for owner, requires in (
            ("assembly_choice", decl.kinds["assembly_choice"].requires),
            ("model_assembly", decl.kinds["model_assembly"].requires),
        )
        for r in requires
    }
    assert {
        ("assembly_choice", "form_implements_slot_contract", "verify"),
        ("assembly_choice", "slot_belongs_to_form_at_path", "verify"),
        ("model_assembly", "single_slots_decided", "verify"),
    } <= declared


def test_a_form_of_another_contract_than_the_slot_accepts_is_flagged(world: World, conn: psycopg.Connection) -> None:
    assert flagged(conn, IMPLEMENTS) == ids(world.choices["wrong-contract"])


def test_a_path_that_does_not_end_in_the_slots_name_or_a_slot_of_another_form_is_flagged(
    world: World, conn: psycopg.Connection
) -> None:
    assert flagged(conn, BELONGS) == ids(
        world.choices["path-names-another-slot"],
        world.choices["slot-of-another-form-at-the-root"],
        world.choices["slot-below-the-wrong-form"],
        world.choices["slot-below-nothing"],
    )


def test_a_single_valued_slot_left_open_is_flagged_at_the_root_and_below_a_chosen_form(
    world: World, conn: psycopg.Connection
) -> None:
    found = run_check(conn, CHECKS[DECIDED], shown=100)
    assert found.error is None
    open_slots = {(row[found.columns.index("id")], row[found.columns.index("open_slot")]) for row in found.rows}
    a = world.assemblies
    assert open_slots == {
        # the wrong-contract and path variants decide fewer slots than the complete assembly does
        (str(a["path-names-another-slot"]), "peng_robinson_core.alpha"),
        (str(a["path-names-another-slot"]), "peng_robinson_core.translation"),
        (str(a["slot-below-nothing"]), "huron_vidal_pr_mixing.excess"),
        (str(a["root-slot-open"]), "peng_robinson_core.translation"),
        (str(a["nested-slot-open"]), "huron_vidal_pr_mixing.excess"),
        (str(a["nothing-decided"]), "peng_robinson_core.alpha"),
        (str(a["nothing-decided"]), "peng_robinson_core.mixing"),
        (str(a["nothing-decided"]), "peng_robinson_core.translation"),
    }
    assert found.violations == len(open_slots)


def test_the_complete_assembly_breaks_none_of_the_three_rules(world: World, conn: psycopg.Connection) -> None:
    complete = world.assemblies["complete"]
    row = conn.execute("SELECT count(*) FROM tk.assembly_choice WHERE assembly = %s", (complete,)).fetchone()
    assert row == (4,)
    for target in (IMPLEMENTS, BELONGS):
        owners = {
            str(conn.execute("SELECT assembly FROM tk.assembly_choice WHERE id = %s", (flagged_id,)).fetchone()[0])  # type: ignore[index]
            for flagged_id in flagged(conn, target)
        }
        assert str(complete) not in owners
    assert str(complete) not in run_check(conn, CHECKS[DECIDED], shown=100).ids


def test_only_the_three_checks_of_the_rules_fail_in_the_database_of_variants(conn: psycopg.Connection) -> None:
    found = failing(conn)
    assert set(found) == {IMPLEMENTS, BELONGS, DECIDED}
    assert found[IMPLEMENTS] == 1 and found[BELONGS] == 4
