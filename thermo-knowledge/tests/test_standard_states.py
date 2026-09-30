# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Standard states by the role a species form plays in a chemical system (meta-model section 3):
one convention set gives the solvent and the solute of one liquid their own standard states, and a
reaction whose equilibrium constant is stated needs one for each participant."""

from __future__ import annotations

import uuid
from collections.abc import Iterator
from pathlib import Path

import psycopg
import pytest
from build_support import fingerprint, inputs_of, write_source
from mapping_support import extended_declaration, origin, real_declaration
from psycopg import sql

from thermo_knowledge import config
from thermo_knowledge.build import build_database
from thermo_knowledge.canonical.values import Quantity
from thermo_knowledge.canonical.writer import CanonicalWriter
from thermo_knowledge.declaration import Declaration
from thermo_knowledge.testing import TestDatabase
from thermo_knowledge.verify.checks import VERIFY_DIR, load_checks
from thermo_knowledge.verify.run import run_check

CHECK = {c.target: c for c in load_checks(config.TREE_DIR / VERIFY_DIR)[0]}[
    "reaction.standard_states_for_equilibrium_constant"
]
ROLES = (
    "gas",
    "pure_liquid",
    "pure_solid",
    "solvent",
    "solute",
    "solution_constituent",
    "surface_species",
    "exchange_species",
)


def entity(decl: Declaration, kind: str, name: str) -> uuid.UUID:
    return next(e.id for e in decl.entities if e.kind == kind and e.name == name)


def test_the_member_roles_are_declared_entities_with_a_doc() -> None:
    decl = real_declaration()
    declared = {e.name: e for e in decl.entities if e.kind == "member_role"}
    assert tuple(declared) == ROLES or set(declared) == set(ROLES)
    assert all(e.doc.endswith(".") and e.doc.count(". ") == 0 for e in declared.values())
    kind = decl.kinds["member_role"]
    assert kind.identity == ("name",) and kind.provenance.mode == "declaration"


def test_a_convention_set_gives_the_standard_state_of_each_member_role() -> None:
    decl = real_declaration()
    relation = decl.relations["convention_standard_state"]
    assert [k.name for k in relation.keys] == ["convention_set", "member_role"]
    assert relation.keys[1].type.text == "member_role"
    member = next(c for c in decl.relations["system_member"].values if c.name == "member_role")
    assert member.optional and member.type.text == "member_role"


class Water:
    """The canonical rows of a chemical system with water and two ions, and a reaction over them
    whose equilibrium constant a parameter set states."""

    def __init__(self, tmp_path: Path) -> None:
        self.decl = extended_declaration(tmp_path / "decl")
        self.ids: dict[str, uuid.UUID] = {}
        self.canonical = tmp_path / "canonical"
        write_source(self.canonical, "src", self.fill, decl=self.decl, declaration=fingerprint(self.decl))

    def fill(self, w: CanonicalWriter) -> None:
        decl, ids = self.decl, self.ids
        liquid = entity(decl, "aggregation", "liquid")
        forms: dict[str, uuid.UUID] = {}
        for position, (name, charge) in enumerate((("water", 0), ("sodium", 1), ("chloride", -1))):
            species = w.kind(
                "species",
                {"canonical_key": name, "label": name, "charge": charge},
                origins=[origin(f"a.json#/{position}")],
            )
            forms[name] = w.kind(
                "species_form",
                {
                    "canonical_key": f"{name} liquid",
                    "label": name,
                    "species": species,
                    "aggregation": liquid,
                },
                origins=[origin(f"a.json#/{position}")],
            )
        ids.update({f"form_{name}": found for name, found in forms.items()})
        ids["water_state"] = w.kind(
            "standard_state",
            {"key": "water", "kind": "pure_real", "pressure_rule": "fixed", "pressure": Quantity(1.0, "bar")},
            origins=[origin("a.json#/3")],
        )
        ids["solute_state"] = w.kind(
            "standard_state",
            {
                "key": "one molal",
                "kind": "infinite_dilution",
                "scale": entity(decl, "composition_basis", "molality"),
                "solvent": forms["water"],
                "pressure_rule": "fixed",
                "pressure": Quantity(1.0, "bar"),
            },
            origins=[origin("a.json#/3")],
        )
        ids["convention"] = w.kind(
            "convention_set",
            {"key": "aqueous", "revision": "1", "temperature_scale": "its_90"},
            origins=[origin("a.json#/4")],
        )
        for role, state in (("solvent", "water_state"), ("solute", "solute_state")):
            w.relation(
                "convention_standard_state",
                {"convention_set": ids["convention"], "member_role": entity(decl, "member_role", role)},
                {"value": ids[state]},
            )
        ids["system"] = w.kind(
            "chemical_system", {"key": "brine", "revision": "1"}, origins=[origin("a.json#/5")]
        )
        for name, role in (("water", "solvent"), ("sodium", "solute"), ("chloride", "solute")):
            w.relation(
                "system_member",
                {"system": ids["system"], "form": forms[name]},
                {"role": "member", "member_role": entity(decl, "member_role", role)},
            )
        ids["parameterization"] = w.kind(
            "parameterization",
            {
                "key": "constants",
                "revision": "1",
                "title": "Equilibrium constants",
                "coherence": "independent_records",
                "convention_set": ids["convention"],
                "chemical_system": ids["system"],
            },
            origins=[origin("a.json#/6", "published")],
        )
        ids["reaction"] = w.kind(
            "reaction",
            {"canonical_key": "dissolution", "extent": "as_written"},
            origins=[origin("a.json#/7")],
        )
        for name, coefficient in (("water", -1.0), ("sodium", 1.0), ("chloride", 1.0)):
            w.relation(
                "reaction_participant",
                {"reaction": ids["reaction"], "form": forms[name]},
                {"coefficient": coefficient},
            )
        w.parameter_set(
            parameterization=ids["parameterization"],
            slot_group="fixture_log_k.pure",
            subjects=[ids["reaction"]],
            slots={"log_k": 1.5},
            origins=[origin("a.json#/8", "published")],
        )


@pytest.fixture(scope="module")
def water(tmp_path_factory: pytest.TempPathFactory) -> Iterator[tuple[Water, str]]:
    made = Water(tmp_path_factory.mktemp("water"))
    with TestDatabase() as database:
        build_database(database.url, made.decl, inputs_of(made.canonical))
        yield made, database.url


@pytest.fixture
def conn(water: tuple[Water, str]) -> Iterator[psycopg.Connection]:
    connection = psycopg.connect(water[1])
    connection.execute("SELECT 1")
    try:
        yield connection
    finally:
        connection.rollback()
        connection.close()


def violations(conn: psycopg.Connection) -> list[dict[str, str]]:
    result = run_check(conn, CHECK)
    assert result.error is None, result.error
    return [dict(zip(result.columns, row)) for row in result.rows]


def test_solvent_and_solute_take_their_states_from_one_convention_set(
    water: tuple[Water, str], conn: psycopg.Connection
) -> None:
    made, _ = water
    # no participant states a standard state of its own
    (states,) = conn.execute(
        "SELECT count(standard_state) FROM tk.reaction_participant"
    ).fetchone()  # type: ignore[misc]
    assert states == 0
    # both roles are read, through the roles the system gives the members
    rows = conn.execute(
        "SELECT sm.member_role, c.value FROM tk.system_member sm "
        "JOIN tk.parameterization pz ON pz.chemical_system = sm.system "
        "JOIN tk.convention_standard_state c ON c.convention_set = pz.convention_set "
        "AND c.member_role = sm.member_role ORDER BY sm.form"
    ).fetchall()
    assert {state for _, state in rows} == {made.ids["water_state"], made.ids["solute_state"]}
    assert violations(conn) == []


def test_a_convention_set_without_the_solute_entry_fails_the_check(
    water: tuple[Water, str], conn: psycopg.Connection
) -> None:
    made, _ = water
    conn.execute(
        "DELETE FROM tk.convention_standard_state WHERE member_role = %s",
        (entity(made.decl, "member_role", "solute"),),
    )
    found = violations(conn)
    assert {row["participant"] for row in found} == {
        str(made.ids["form_sodium"]),
        str(made.ids["form_chloride"]),
    }
    assert {row["id"] for row in found} == {str(made.ids["reaction"])}
    assert {row["locator"] for row in found} == {"a.json#/7"}
    assert {row["member_role"] for row in found} == {str(entity(made.decl, "member_role", "solute"))}


def test_a_participant_may_state_its_own_standard_state(
    water: tuple[Water, str], conn: psycopg.Connection
) -> None:
    made, _ = water
    conn.execute("DELETE FROM tk.convention_standard_state")
    assert len(violations(conn)) == 3
    conn.execute(
        "UPDATE tk.reaction_participant SET standard_state = %s", (made.ids["water_state"],)
    )
    assert violations(conn) == []


def test_a_participant_that_is_no_member_of_the_system_has_no_role_to_map(
    water: tuple[Water, str], conn: psycopg.Connection
) -> None:
    made, _ = water
    conn.execute(
        sql.SQL("DELETE FROM tk.system_member WHERE form = %s"), (made.ids["form_chloride"],)
    )
    found = violations(conn)
    assert [row["participant"] for row in found] == [str(made.ids["form_chloride"])]
    assert found[0]["member_role"] == ""


def test_a_reaction_with_no_equilibrium_constant_needs_no_standard_states(
    water: tuple[Water, str], conn: psycopg.Connection
) -> None:
    conn.execute("DELETE FROM tk.convention_standard_state")
    conn.execute("DELETE FROM param.fixture_log_k__pure")
    assert violations(conn) == []
