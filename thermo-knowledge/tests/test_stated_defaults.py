# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Stated defaults of a selection policy and the value state that uses them (plan 24, packet TK2g,
alignment items 35 and 8).

`policy_default` states, per (policy, slot), a value state and a value in the slot's unit. A slot
the source leaves at its stated default is in the value state `stated_default`: the set stores no
value and the default is the selecting policy's. A PC-SAFT binary record with no fields is a pair
set whose slots are all in that state; a parameterless association site is a site set whose
self-association slots are `not_applicable`. A set with a slot at its default is selectable only
under a policy that states a default for the slot: the structural check
`set_default_needs_policy_default` holds the declared policies to it (selection itself is a later
packet).
"""

from __future__ import annotations

import uuid
from collections.abc import Iterator
from dataclasses import dataclass

import numpy as np
import psycopg
import pytest
from build_support import fingerprint, inputs_of, write_source
from mapping_support import origin, writer
from mechanisms_support import mechanism_declaration

from thermo_knowledge import config, db
from thermo_knowledge.build import build_database
from thermo_knowledge.canonical.values import NotApplicable, Quantity, StatedDefault
from thermo_knowledge.canonical.writer import CanonicalWriter, ValidationError
from thermo_knowledge.declaration import Declaration
from thermo_knowledge.expression.evaluate import EvaluationRefusal, bind
from thermo_knowledge.qualify.source import DatabaseSource
from thermo_knowledge.testing import TestDatabase
from thermo_knowledge.verify.checks import VERIFY_DIR, load_checks
from thermo_knowledge.verify.run import run_check, run_checks

CHECKS = {c.target: c for c in load_checks(config.TREE_DIR / VERIFY_DIR)[0]}
NRTL = "nrtl_fixture.pair"
BINARY = "pcsaft_binary_fixture.pair"
SITE = "association_site_fixture.site"
STRUCTURAL = "set_default_needs_policy_default"


@dataclass
class World:
    decl: Declaration
    database: TestDatabase
    ids: dict[str, uuid.UUID]


def at(tag: str) -> list:  # noqa: ANN401
    return [origin(f"a.json#/{tag}", "published")]


def parameterization(w: CanonicalWriter, key: str) -> uuid.UUID:
    return w.kind(
        "parameterization",
        {"key": key, "revision": "1", "title": key, "coherence": "independent_records"},
        origins=at(f"p-{key}"),
    )


def policy(
    w: CanonicalWriter,
    key: str,
    *,
    over: list[uuid.UUID],
    defaults: dict[str, tuple[str, object]] | None = None,
    unasserted: str = "refuse",
    **attributes: object,
) -> uuid.UUID:
    """A policy that may select from the parameterizations `over` and states `defaults` by slot
    (`form.group.slot`) as (state, value)."""
    made = w.kind(
        "selection_policy",
        {"key": key, "revision": "1", "unasserted": unasserted, **attributes},
        origins=at(f"policy-{key}"),
    )
    for rank, parameterization_id in enumerate(over, start=1):
        w.relation(
            "policy_precedence",
            {"policy": made, "parameterization": parameterization_id},
            {"value": rank},
            at="a.json#/rank",
        )
    for slot, (state, value) in (defaults or {}).items():
        w.relation(
            "policy_default",
            {"policy": made, "slot": slot},
            {"state": state, **({} if value is None else {"value": value})},
            at="a.json#/default",
        )
    return made


def write_world(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    water, ethanol = (
        w.kind("species", {"canonical_key": key, "label": key}, origins=at(f"s-{key}"))
        for key in ("water", "ethanol")
    )
    ids.update(water=water, ethanol=ethanol)
    # NRTL: a set at the default alpha with no temperature coefficient, and a set that states both
    nrtl = parameterization(w, "nrtl")
    ids["nrtl"] = nrtl
    ids["set_defaulted"] = w.parameter_set(
        parameterization=nrtl,
        slot_group=NRTL,
        subjects=[ethanol, water],
        slots={"a": 1.5, "b": StatedDefault(), "alpha": StatedDefault()},
        origins=at("nrtl-defaulted"),
    )
    ids["set_explicit"] = w.parameter_set(
        parameterization=nrtl,
        slot_group=NRTL,
        subjects=[water, ethanol],
        slots={"a": 1.5, "b": Quantity(300.0, "K"), "alpha": 0.2},
        origins=at("nrtl-explicit"),
    )
    ids["policy_nrtl"] = policy(
        w,
        "nrtl-defaults",
        over=[nrtl],
        defaults={
            f"{NRTL}.alpha": ("known", 0.3),
            f"{NRTL}.b": ("known", Quantity(0.0, "K")),
        },
    )
    ids["policy_nrtl_other"] = policy(  # a policy that states other defaults for the same sets
        w, "nrtl-other", over=[nrtl], defaults={f"{NRTL}.alpha": ("known", 0.4), f"{NRTL}.b": ("known", Quantity(10.0, "K"))}
    )
    # PC-SAFT: a binary record with no fields
    feos = parameterization(w, "feos")
    ids["feos"] = feos
    ids["set_empty_record"] = w.parameter_set(
        parameterization=feos,
        slot_group=BINARY,
        subjects=[water, ethanol],
        slots={"k_ij": StatedDefault(), "l_ij": StatedDefault()},
        origins=at("empty-record"),
    )
    ids["policy_feos"] = policy(
        w,
        "feos-defaults",
        over=[feos],
        defaults={f"{BINARY}.k_ij": ("known", 0.0), f"{BINARY}.l_ij": ("known", 0.0)},
    )
    # a parameterless association site: induced association only
    assoc = parameterization(w, "assoc")
    ids["assoc"] = assoc
    scheme = w.kind("site_scheme", {"key": "2B"}, origins=at("scheme"))
    site = w.kind(
        "association_site",
        {
            "scheme": scheme,
            "label": "H",
            "on_entity": water,
            "multiplicity": 2,
        },
        origins=at("site"),
    )
    ids["site"] = site
    ids["set_parameterless_site"] = w.parameter_set(
        parameterization=assoc,
        slot_group=SITE,
        subjects=[site],
        slots={"kappa_AB": NotApplicable(), "epsilon_AB_over_k": NotApplicable()},
        origins=at("parameterless-site"),
    )


@pytest.fixture(scope="module")
def decl(tmp_path_factory: pytest.TempPathFactory) -> Declaration:
    return mechanism_declaration(tmp_path_factory.mktemp("default-declaration"))


@pytest.fixture(scope="module")
def world(decl: Declaration, tmp_path_factory: pytest.TempPathFactory) -> Iterator[World]:
    canonical = tmp_path_factory.mktemp("default-canonical")
    ids: dict[str, uuid.UUID] = {}
    write_source(canonical, "src", lambda w: write_world(w, decl, ids), decl=decl, declaration=fingerprint(decl))
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


def source(world: World, conn: psycopg.Connection, key: str, policy_key: str | None = None) -> DatabaseSource:
    return DatabaseSource(
        conn,
        world.decl,
        [world.ids[key]],
        policy=None if policy_key is None else world.ids[policy_key],
    )


# -- the state and the stated defaults ----------------------------------------------------------


def test_every_fixture_satisfies_every_invariant(world: World, conn: psycopg.Connection) -> None:
    results = run_checks(conn, list(CHECKS.values()))
    assert [(r.check.target, r.violations, r.error) for r in results if not r.passed] == []


def test_the_value_state_has_a_member_for_a_slot_left_at_its_default() -> None:
    from mapping_support import real_declaration

    decl = real_declaration()
    members = {m.name: m for m in decl.enums["value_state"].members}
    assert "stated_default" in members
    assert decl.enum_members_with("value_state", "policy_supplied") == ("stated_default",)
    assert decl.enum_members_with("value_state", "may_be_policy_default") == ("known", "not_applicable")


def test_the_free_text_rule_of_a_policy_is_gone() -> None:
    from mapping_support import real_declaration

    names = {a.name for a in real_declaration().kinds["selection_policy"].attributes}
    assert "rule" not in names and "rule_form" in names


def test_a_set_that_leaves_a_slot_at_its_default_stores_no_value(world: World, conn: psycopg.Connection) -> None:
    row = conn.execute(
        'SELECT a, b, b__state::text, alpha, alpha__state::text FROM param."nrtl_fixture__pair" WHERE id = %s',
        (world.ids["set_defaulted"],),
    ).fetchone()
    assert row == (1.5, None, "stated_default", None, "stated_default")


def test_the_empty_feos_binary_record_is_a_set_whose_slots_are_all_at_their_default(
    world: World, conn: psycopg.Connection
) -> None:
    row = conn.execute(
        'SELECT k_ij, k_ij__state::text, l_ij, l_ij__state::text FROM param."pcsaft_binary_fixture__pair" WHERE id = %s',
        (world.ids["set_empty_record"],),
    ).fetchone()
    assert row == (None, "stated_default", None, "stated_default")


def test_the_parameterless_site_states_its_self_association_slots_not_applicable(
    world: World, conn: psycopg.Connection
) -> None:
    row = conn.execute(
        'SELECT "kappa_AB__state"::text, "epsilon_AB_over_k__state"::text FROM param."association_site_fixture__site" WHERE id = %s',
        (world.ids["set_parameterless_site"],),
    ).fetchone()
    assert row == ("not_applicable", "not_applicable")


# -- evaluation under a policy ------------------------------------------------------------------


def nrtl(world: World, found: DatabaseSource, first: str, second: str, temperature: np.ndarray) -> np.ndarray:
    bound = bind(
        world.decl,
        "nrtl_fixture",
        source=found,
        roles={"i": str(world.ids[first]), "j": str(world.ids[second])},
    )
    return bound.evaluate("G", T=temperature)


def test_nrtl_alpha_takes_the_default_the_policy_states(world: World, conn: psycopg.Connection) -> None:
    """alpha is 0.3 and the absent temperature coefficient b is read as zero, both stated by the
    policy: G = exp(-0.3 * 1.5)."""
    temperature = np.array([280.0, 350.0])
    found = nrtl(world, source(world, conn, "nrtl", "policy_nrtl"), "ethanol", "water", temperature)
    np.testing.assert_allclose(found, np.exp(-0.3 * (1.5 + 0.0 / temperature)), rtol=1e-14)


def test_another_policy_gives_the_same_set_its_own_defaults(world: World, conn: psycopg.Connection) -> None:
    temperature = np.array([300.0])
    found = nrtl(world, source(world, conn, "nrtl", "policy_nrtl_other"), "ethanol", "water", temperature)
    np.testing.assert_allclose(found, np.exp(-0.4 * (1.5 + 10.0 / temperature)), rtol=1e-14)


def test_a_set_that_states_its_values_ignores_the_policy_defaults(world: World, conn: psycopg.Connection) -> None:
    temperature = np.array([300.0])
    found = nrtl(world, source(world, conn, "nrtl", "policy_nrtl"), "water", "ethanol", temperature)
    np.testing.assert_allclose(found, np.exp(-0.2 * (1.5 + 300.0 / temperature)), rtol=1e-14)


def test_without_a_policy_a_slot_at_its_default_has_no_value_and_the_evaluator_refuses(
    world: World, conn: psycopg.Connection
) -> None:
    with pytest.raises(EvaluationRefusal, match="holds no value for slot `alpha`"):
        nrtl(world, source(world, conn, "nrtl"), "ethanol", "water", np.array([300.0]))


def test_a_policy_that_states_no_default_for_the_slot_leaves_it_without_a_value(
    world: World, conn: psycopg.Connection
) -> None:
    with pytest.raises(EvaluationRefusal, match="holds no value for slot"):
        nrtl(world, source(world, conn, "nrtl", "policy_feos"), "ethanol", "water", np.array([300.0]))


def test_the_empty_binary_record_reads_the_defaults_of_its_policy(world: World, conn: psycopg.Connection) -> None:
    subjects = (str(world.ids["water"]), str(world.ids["ethanol"]))
    assert source(world, conn, "feos", "policy_feos").slot_values(BINARY, subjects) == {"k_ij": 0.0, "l_ij": 0.0}
    assert source(world, conn, "feos").slot_values(BINARY, subjects) == {}


def test_the_parameterless_site_holds_no_value_for_a_not_applicable_slot(
    world: World, conn: psycopg.Connection
) -> None:
    """Even under a policy: a default that is `not_applicable` supplies no value, and the set
    states none of its own."""
    subjects = (str(world.ids["site"]),)
    assert source(world, conn, "assoc").slot_values(SITE, subjects) == {}


# -- the requirements ---------------------------------------------------------------------------


def fresh(decl: Declaration) -> CanonicalWriter:
    return writer(decl)


def test_a_named_rule_policy_names_its_form_and_no_other_policy_does(decl: Declaration) -> None:
    w = fresh(decl)
    with pytest.raises(ValidationError, match="rule_form"):
        policy(w, "no-form", over=[], unasserted="named_rule")
    with pytest.raises(ValidationError, match="rule_form"):
        policy(w, "form-without-rule", over=[], unasserted="refuse", rule_form="nrtl_fixture")
    policy(w, "with-form", over=[], unasserted="named_rule", rule_form="nrtl_fixture")


def test_a_default_has_a_value_exactly_when_it_is_known(decl: Declaration) -> None:
    w = fresh(decl)
    p = policy(w, "p", over=[])
    with pytest.raises(ValidationError, match="value_matches_state"):
        w.relation("policy_default", {"policy": p, "slot": f"{NRTL}.alpha"}, {"state": "known"}, at="a.json#/d")
    with pytest.raises(ValidationError, match="value_matches_state"):
        w.relation(
            "policy_default",
            {"policy": p, "slot": f"{NRTL}.alpha"},
            {"state": "not_applicable", "value": 0.3},
            at="a.json#/d",
        )
    w.relation("policy_default", {"policy": p, "slot": f"{NRTL}.alpha"}, {"state": "not_applicable"}, at="a.json#/d")


def test_a_default_value_is_in_the_unit_of_its_slot(decl: Declaration) -> None:
    """The default of a temperature coefficient is stated with a unit and converted to the slot's
    storage unit; a bare number has no unit for a slot in kelvin."""
    w = fresh(decl)
    p = policy(w, "p", over=[])
    w.relation(
        "policy_default",
        {"policy": p, "slot": f"{NRTL}.b"},
        {"state": "known", "value": Quantity(1.0, "mK")},
        at="a.json#/d",
    )
    with pytest.raises(ValidationError, match="has no unit"):
        w.relation("policy_default", {"policy": p, "slot": f"{NRTL}.b"}, {"state": "known", "value": 1.0}, at="a.json#/d")


@pytest.fixture(scope="module")
def violations(
    decl: Declaration, tmp_path_factory: pytest.TempPathFactory
) -> Iterator[tuple[psycopg.Connection, dict[str, uuid.UUID]]]:
    ids: dict[str, uuid.UUID] = {}

    def fill(w: CanonicalWriter) -> None:
        a, b = (
            w.kind("species", {"canonical_key": key, "label": key}, origins=at(f"s-{key}"))
            for key in ("a", "b")
        )
        covered, bare, scoped = (parameterization(w, key) for key in ("covered", "bare", "scoped"))
        ids.update(covered=covered, bare=bare)
        for p, key in ((covered, "covered"), (bare, "bare")):
            ids[f"set_{key}"] = w.parameter_set(
                parameterization=p,
                slot_group=NRTL,
                subjects=[a, b],
                slots={"a": 1.0, "b": Quantity(1.0, "K"), "alpha": StatedDefault()},
                origins=at(f"set-{key}"),
            )
        # a set is selectable under a policy that ranks its parameterisation or overrides to it
        ids["policy_covered"] = policy(w, "covered", over=[covered], defaults={f"{NRTL}.alpha": ("known", 0.3)})
        ids["policy_silent"] = policy(w, "silent", over=[bare], defaults={f"{NRTL}.b": ("known", Quantity(0.0, "K"))})
        ids["policy_override"] = policy(w, "override", over=[], defaults={f"{NRTL}.alpha": ("known", 0.3)})
        w.relation(
            "policy_override",
            {"policy": ids["policy_override"], "parameter_set": ids["set_bare"]},
            at="a.json#/o",
        )
        ids["policy_override_silent"] = policy(w, "override-silent", over=[])
        w.relation(
            "policy_override",
            {"policy": ids["policy_override_silent"], "parameter_set": ids["set_covered"]},
            at="a.json#/o",
        )
        # a policy that supplies stated defaults and states none; a default that is a default,
        # and one for a slot outside the policy's scope
        ids["policy_empty"] = policy(w, "empty", over=[], unasserted="stated_default")
        ids["policy_default_of_default"] = policy(
            w, "default-of-default", over=[], defaults={f"{NRTL}.alpha": ("stated_default", None)}
        )
        ids["policy_redirect"] = policy(
            w, "redirect-default", over=[], defaults={f"{NRTL}.b": ("withheld", None)}
        )
        ids["policy_scoped"] = policy(
            w,
            "scoped",
            over=[scoped],
            defaults={f"{BINARY}.k_ij": ("known", 0.0), f"{NRTL}.alpha": ("known", 0.3)},
            scope_slot_group=BINARY,
        )

    canonical = tmp_path_factory.mktemp("default-violations")
    write_source(canonical, "src", fill, decl=decl, declaration=fingerprint(decl))
    with TestDatabase() as database:
        build_database(database.url, decl, inputs_of(canonical))
        with db.connect(database.url) as conn:
            yield conn, ids


def test_a_set_at_its_default_under_a_policy_that_states_none_is_flagged_by_the_structural_check(
    violations: tuple[psycopg.Connection, dict[str, uuid.UUID]],
) -> None:
    conn, ids = violations
    found = run_check(conn, CHECKS[STRUCTURAL])
    flagged = {(row[0], str(row[-1])) for row in found.rows}
    assert flagged == {
        (str(ids["set_bare"]), str(ids["policy_silent"])),  # ranks the parameterisation, no alpha default
        (str(ids["set_covered"]), str(ids["policy_override_silent"])),  # overrides to the set, no alpha default
    }


def test_a_policy_that_supplies_stated_defaults_states_some(
    violations: tuple[psycopg.Connection, dict[str, uuid.UUID]],
) -> None:
    conn, ids = violations
    found = run_check(conn, CHECKS["selection_policy.stated_default_has_defaults"])
    assert {row[0] for row in found.rows} == {str(ids["policy_empty"])}


def test_a_default_is_a_known_value_or_not_applicable(
    violations: tuple[psycopg.Connection, dict[str, uuid.UUID]],
) -> None:
    conn, ids = violations
    found = run_check(conn, CHECKS["policy_default.state_may_be_a_default"])
    assert {(str(row[2]), row[-1]) for row in found.rows} == {
        (str(ids["policy_default_of_default"]), "stated_default"),
        (str(ids["policy_redirect"]), "withheld"),
    }


def test_a_default_is_for_a_slot_of_the_group_the_policy_is_scoped_to(
    violations: tuple[psycopg.Connection, dict[str, uuid.UUID]],
) -> None:
    conn, ids = violations
    found = run_check(conn, CHECKS["policy_default.slot_in_policy_scope"])
    assert [(str(row[2]), row[3]) for row in found.rows] == [(str(ids["policy_scoped"]), f"{NRTL}.alpha")]


def test_the_writer_refuses_a_value_for_a_slot_that_is_not_stateful(decl: Declaration) -> None:
    w = fresh(decl)
    a, b = (w.kind("species", {"canonical_key": key, "label": key}, origins=at(f"s-{key}")) for key in ("a", "b"))
    p = parameterization(w, "p")
    with pytest.raises(ValidationError, match="a: is not a stateful slot"):
        w.parameter_set(
            parameterization=p,
            slot_group=NRTL,
            subjects=[a, b],
            slots={"a": StatedDefault(), "b": StatedDefault(), "alpha": StatedDefault()},
            origins=at("bad"),
        )
