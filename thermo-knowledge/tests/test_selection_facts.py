# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Selection facts a carrier asserts (plan 24, packet TK2g, alignment items 5 and 7).

Occurrence in a set's identity keeps every repeated row of a source (item 5, first half). Which of
them the carrier's own code uses, and which of several competing tables it chooses by default,
are `selection_policy` facts: scoped to a slot group or an observable, asserted by the carrier
with the locator of the code or the documentation, and saying whether the behaviour is what the
carrier documents.
"""

from __future__ import annotations

import uuid
from collections.abc import Iterator
from dataclasses import dataclass

import psycopg
import pytest

from build_support import fingerprint, inputs_of, write_source
from mapping_support import carrier, origin, writer
from mechanisms_support import entity_id, mechanism_declaration
from thermo_knowledge import config, db, identity
from thermo_knowledge.build import build_database
from thermo_knowledge.canonical.values import Quantity
from thermo_knowledge.canonical.writer import CanonicalWriter, ValidationError
from thermo_knowledge.declaration import Declaration
from thermo_knowledge.qualify.source import AmbiguousOccurrence, DatabaseSource
from thermo_knowledge.testing import TestDatabase
from thermo_knowledge.verify.checks import VERIFY_DIR, load_checks
from thermo_knowledge.verify.run import run_check, run_checks

CARRIER = identity.identifier("source", [carrier("src", "a.json").key])
CHECKS = {c.target: c for c in load_checks(config.TREE_DIR / VERIFY_DIR)[0]}
KIJ = "kij_fixture.pair"
TABLE = "critical_table_fixture.pure"
CHECK = "selection_policy.carrier_policy_is_a_scoped_assessed_fact"


@dataclass
class World:
    decl: Declaration
    database: TestDatabase
    ids: dict[str, uuid.UUID]


def asserted(locator: str) -> list:  # noqa: ANN401
    """The origin of a fact about a carrier's behaviour: the place in the carrier's code or
    documentation that shows it."""
    return [origin(locator, "published")]


def carrier_policy(w: CanonicalWriter, key: str, locator: str, **attributes: object) -> uuid.UUID:
    return w.kind(
        "selection_policy",
        {"key": key, "revision": "1", "unasserted": "refuse", "asserted_by": CARRIER, **attributes},
        origins=asserted(locator),
    )


def parameterization(w: CanonicalWriter, key: str) -> uuid.UUID:
    return w.kind(
        "parameterization",
        {"key": key, "revision": "1", "title": key, "coherence": "independent_records"},
        origins=asserted(f"a.json#/p-{key}"),
    )


def write_world(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    critical = entity_id(decl, "observable", "critical_temperature")
    a, b = (
        w.kind(
            "species", {"canonical_key": key, "label": key}, origins=asserted(f"a.json#/s-{key}")
        )
        for key in ("a", "b")
    )
    ids.update(a=a, b=b)
    # item 5: a kij table that lists one pair twice, and the carrier's own rule for which row wins
    pair_table = parameterization(w, "kij-table")
    ids["kij_table"] = pair_table
    for occurrence, value in ((1, 0.10), (2, 0.12)):
        w.parameter_set(
            parameterization=pair_table,
            slot_group=KIJ,
            subjects=[a, b],
            slots={"k_ij": value},
            occurrence=occurrence,
            origins=asserted(f"a.json#/kij/{occurrence}"),
        )
    ids["policy_last_wins"] = carrier_policy(
        w,
        "kij-last-wins",
        "a.json#/code/kij-read-loop",
        scope_slot_group=KIJ,
        repeated_rows="last_wins",
        as_documented=False,  # the code overwrites: the documentation says the first row is used
    )
    # item 7: competing tables of critical temperatures, chosen by insertion order for one observable
    tables = {key: parameterization(w, key) for key in ("tc-ihs", "tc-yaws", "tc-poling")}
    ids.update({f"table_{key}": value for key, value in tables.items()})
    for key, value in tables.items():
        w.parameter_set(
            parameterization=value,
            slot_group=TABLE,
            subjects=[a],
            slots={"T_c": Quantity(500.0, "K")},
            origins=asserted(f"a.json#/tc/{key}"),
        )
    ids["policy_insertion_order"] = carrier_policy(
        w,
        "tc-insertion-order",
        "a.json#/code/critical-methods-dict",
        scope_observable=critical,
        table_choice="listing_order",
        as_documented=True,
    )
    for rank, key in enumerate(("tc-ihs", "tc-yaws", "tc-poling"), start=1):
        w.relation(
            "policy_precedence",
            {"policy": ids["policy_insertion_order"], "parameterization": tables[key]},
            {"value": rank},
            at="a.json#/code/critical-methods-dict",
        )
    # the other ways the surveys found a default is chosen
    for member in ("tagged_default", "ranked", "first_only"):
        ids[f"policy_{member}"] = carrier_policy(
            w,
            f"tc-{member}",
            f"a.json#/code/{member}",
            scope_slot_group=TABLE,
            table_choice=member,
            as_documented=member != "tagged_default",
        )
    # one policy that states both facts for one scope
    ids["policy_both"] = carrier_policy(
        w,
        "both",
        "a.json#/code/both",
        scope_slot_group=KIJ,
        scope_observable=critical,
        repeated_rows="first_wins",
        table_choice="listing_order",
        as_documented=True,
    )


@pytest.fixture(scope="module")
def decl(tmp_path_factory: pytest.TempPathFactory) -> Declaration:
    return mechanism_declaration(tmp_path_factory.mktemp("selection-declaration"))


@pytest.fixture(scope="module")
def world(decl: Declaration, tmp_path_factory: pytest.TempPathFactory) -> Iterator[World]:
    canonical = tmp_path_factory.mktemp("selection-canonical")
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


def policy_row(conn: psycopg.Connection, identifier: uuid.UUID) -> dict[str, object]:
    cursor = conn.execute(
        "SELECT p.key, p.repeated_rows::text, p.table_choice::text, p.as_documented, "
        "p.scope_slot_group, o.key AS scope_observable "
        "FROM tk.selection_policy p LEFT JOIN tk.observable o ON o.id = p.scope_observable "
        "WHERE p.id = %s",
        (identifier,),
    )
    row = cursor.fetchone()
    assert row is not None
    return dict(zip([c.name for c in cursor.description or []], row, strict=True))


def test_every_fixture_satisfies_every_invariant(world: World, conn: psycopg.Connection) -> None:
    results = run_checks(conn, list(CHECKS.values()))
    assert [(r.check.target, r.violations, r.error) for r in results if not r.passed] == []


# -- item 5: the rows a carrier repeats and the one its code uses --------------------------------


def test_both_rows_of_a_repeated_pair_are_held_under_their_occurrence(
    world: World, conn: psycopg.Connection
) -> None:
    rows = conn.execute(
        'SELECT ps.occurrence, g.k_ij FROM param."kij_fixture__pair" g '
        "JOIN tk.parameter_set ps ON ps.id = g.id ORDER BY ps.occurrence"
    ).fetchall()
    assert rows == [(1, 0.10), (2, 0.12)]


def test_a_source_that_holds_repeated_rows_is_never_read_without_naming_the_one_to_use(
    world: World, conn: psycopg.Connection
) -> None:
    subjects = (str(world.ids["a"]), str(world.ids["b"]))
    with pytest.raises(AmbiguousOccurrence):
        DatabaseSource(conn, world.decl, [world.ids["kij_table"]]).slot_values(KIJ, subjects)


def test_the_carriers_rule_decides_which_occurrence_to_read(
    world: World, conn: psycopg.Connection
) -> None:
    """The recorded rule is `last_wins`: the occurrence it names is the highest one, and reading
    it gives the row the carrier's own code would use."""
    rule = policy_row(conn, world.ids["policy_last_wins"])["repeated_rows"]
    (present,) = conn.execute(
        "SELECT array_agg(ps.occurrence ORDER BY ps.occurrence) FROM tk.parameter_set ps "
        "WHERE ps.parameterization = %s",
        (world.ids["kij_table"],),
    ).fetchone()  # type: ignore[misc]
    winner = {"first_wins": present[0], "last_wins": present[-1]}[rule]
    subjects = (str(world.ids["a"]), str(world.ids["b"]))
    found = DatabaseSource(
        conn, world.decl, [world.ids["kij_table"]], occurrences={world.ids["kij_table"]: winner}
    ).slot_values(KIJ, subjects)
    assert found == {"k_ij": 0.12}


def test_the_policy_is_asserted_by_the_carrier_with_the_locator_of_its_code(
    world: World, conn: psycopg.Connection
) -> None:
    row = conn.execute(
        "SELECT p.asserted_by, p.as_documented, i.locator FROM tk.selection_policy p "
        "JOIN prov.record_origin o ON o.record = p.id JOIN prov.import_record i ON i.id = o.import_record "
        "WHERE p.id = %s",
        (world.ids["policy_last_wins"],),
    ).fetchone()
    assert row == (CARRIER, False, "a.json#/code/kij-read-loop")


def test_the_rule_is_scoped_to_a_slot_group(world: World, conn: psycopg.Connection) -> None:
    found = policy_row(conn, world.ids["policy_last_wins"])
    slot_group = conn.execute(
        "SELECT qualified_name FROM meta.slot_group WHERE id = %s", (found["scope_slot_group"],)
    ).fetchone()
    assert slot_group == (KIJ,)
    assert (found["repeated_rows"], found["scope_observable"]) == ("last_wins", None)


# -- item 7: the default among competing tables --------------------------------------------------


def test_a_default_chosen_by_listing_order_for_one_observable_is_a_scoped_fact(
    world: World, conn: psycopg.Connection
) -> None:
    found = policy_row(conn, world.ids["policy_insertion_order"])
    assert (found["table_choice"], found["scope_observable"], found["as_documented"]) == (
        "listing_order",
        "critical_temperature",
        True,
    )
    assert found["scope_slot_group"] is None


def test_the_order_the_carrier_yields_is_the_precedence_of_its_policy(
    world: World, conn: psycopg.Connection
) -> None:
    rows = conn.execute(
        "SELECT pz.key, pp.value FROM tk.policy_precedence pp "
        "JOIN tk.parameterization pz ON pz.id = pp.parameterization WHERE pp.policy = %s ORDER BY pp.value",
        (world.ids["policy_insertion_order"],),
    ).fetchall()
    assert rows == [("tc-ihs", 1), ("tc-yaws", 2), ("tc-poling", 3)]


@pytest.mark.parametrize("member", ["tagged_default", "ranked", "first_only"])
def test_each_way_a_carrier_chooses_a_default_is_a_member(
    world: World, conn: psycopg.Connection, member: str
) -> None:
    found = policy_row(conn, world.ids[f"policy_{member}"])
    assert found["table_choice"] == member
    assert found["as_documented"] is (member != "tagged_default")


def test_a_policy_may_state_both_facts_for_a_scope_of_a_slot_group_and_an_observable(
    world: World, conn: psycopg.Connection
) -> None:
    found = policy_row(conn, world.ids["policy_both"])
    assert (found["repeated_rows"], found["table_choice"]) == ("first_wins", "listing_order")
    assert (
        found["scope_observable"] == "critical_temperature"
        and found["scope_slot_group"] is not None
    )


# -- the requirement ----------------------------------------------------------------------------


@pytest.fixture(scope="module")
def violations(
    decl: Declaration, tmp_path_factory: pytest.TempPathFactory
) -> Iterator[tuple[psycopg.Connection, dict[str, uuid.UUID]]]:
    ids: dict[str, uuid.UUID] = {}

    def fill(w: CanonicalWriter) -> None:
        critical = entity_id(decl, "observable", "critical_temperature")
        ids["sound"] = carrier_policy(
            w,
            "sound",
            "a.json#/c/1",
            scope_observable=critical,
            table_choice="ranked",
            as_documented=True,
        )
        ids["unscoped"] = carrier_policy(
            w, "unscoped", "a.json#/c/2", table_choice="ranked", as_documented=True
        )
        ids["no_rule"] = carrier_policy(
            w, "no-rule", "a.json#/c/3", scope_observable=critical, as_documented=True
        )
        ids["unassessed"] = carrier_policy(
            w, "unassessed", "a.json#/c/4", scope_observable=critical, repeated_rows="last_wins"
        )
        ids["authored"] = w.kind(
            "selection_policy",
            {
                "key": "authored",
                "revision": "1",
                "unasserted": "refuse",
                "repeated_rows": "last_wins",
            },
            origins=asserted("a.json#/c/5"),
        )
        ids["authored_sound"] = w.kind(
            "selection_policy",
            {"key": "authored-sound", "revision": "1", "unasserted": "refuse"},
            origins=asserted("a.json#/c/6"),
        )

    canonical = tmp_path_factory.mktemp("selection-violations")
    write_source(canonical, "src", fill, decl=decl, declaration=fingerprint(decl))
    with TestDatabase() as database:
        build_database(database.url, decl, inputs_of(canonical))
        with db.connect(database.url) as conn:
            yield conn, ids


def test_a_carriers_policy_needs_a_scope_a_rule_and_an_assessment_and_an_authored_policy_has_none(
    violations: tuple[psycopg.Connection, dict[str, uuid.UUID]],
) -> None:
    conn, ids = violations
    found = run_check(conn, CHECKS[CHECK])
    reasons = {row[0]: row[-1] for row in found.rows}
    assert reasons == {
        str(ids["unscoped"]): "the carrier's policy has no scope",
        str(ids["no_rule"]): "the carrier's policy states no rule",
        str(ids["unassessed"]): "the carrier's policy does not say whether it is as documented",
        str(ids["authored"]): "a policy no carrier asserts states a carrier's fact",
    }


def test_a_repeated_row_rule_is_one_of_the_declared_members(decl: Declaration) -> None:
    w = writer(decl)
    with pytest.raises(ValidationError, match="not a member of enum `repeated_row_rule`"):
        carrier_policy(
            w,
            "bad",
            "a.json#/x",
            scope_slot_group=KIJ,
            repeated_rows="newest_wins",
            as_documented=True,
        )
