# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""A rate constant's dimension follows its reaction (plan 24, packet TK2g, alignment item 18).

`RateConstant` is a dependent quantity type: the dimension of a slot or contract output of that
type is derived from the subject reaction (the rate per volume, or per area when a participant is
on a surface, over the concentration of each species to its forward order, and one more bulk
concentration per extra order). A declaration sees one opaque symbol, so the Arrhenius expression
closes; the concrete unit is checked per set when the canonical writer writes it, against the
participants, explicit orders and phases of the reaction.
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
from mapping_support import origin, writer
from mechanisms_support import broken, entity_id, mechanism_declaration, replace
from thermo_knowledge import config, db
from thermo_knowledge.build import build_database
from thermo_knowledge.canonical.values import Quantity
from thermo_knowledge.canonical.writer import CanonicalWriter, FamilyRow, ValidationError
from thermo_knowledge.declaration import Code, Declaration
from thermo_knowledge.expression.evaluate import bind
from thermo_knowledge.expression.units import describe, type_dimension
from thermo_knowledge.qualify.source import DatabaseSource
from thermo_knowledge.testing import TestDatabase
from thermo_knowledge.verify.checks import VERIFY_DIR, load_checks
from thermo_knowledge.verify.run import run_checks

PUBLISHED = [origin("a.json#/rate", "published")]
CHECKS = {c.target: c for c in load_checks(config.TREE_DIR / VERIFY_DIR)[0]}
ARRHENIUS = "arrhenius_fixture.rate"
LINDEMANN = "lindemann_fixture.limits"
TABLE = "rate_table_fixture.plog"


@dataclass
class World:
    decl: Declaration
    database: TestDatabase
    ids: dict[str, uuid.UUID]


@pytest.fixture(scope="module")
def decl(tmp_path_factory: pytest.TempPathFactory) -> Declaration:
    return mechanism_declaration(tmp_path_factory.mktemp("rate-declaration"))


def at(tag: str) -> list:  # noqa: ANN401
    return [origin(f"a.json#/{tag}", "published")]


def species_form(w: CanonicalWriter, decl: Declaration, name: str, aggregation: str) -> uuid.UUID:
    species = w.kind("species", {"canonical_key": name, "label": name}, origins=at(f"s-{name}"))
    return w.kind(
        "species_form",
        {
            "canonical_key": f"{name} {aggregation}",
            "label": name,
            "species": species,
            "aggregation": entity_id(decl, "aggregation", aggregation),
        },
        origins=at(f"f-{name}"),
    )


def reaction(w: CanonicalWriter, key: str, participants: dict[uuid.UUID, int]) -> uuid.UUID:
    written = w.kind(
        "reaction", {"canonical_key": key, "extent": "as_written"}, origins=at(f"r-{key}")
    )
    for form, coefficient in participants.items():
        w.relation(
            "reaction_participant",
            {"reaction": written, "form": form},
            {"coefficient": coefficient},
            at="a.json#/p",
        )
    return written


def parameterization(w: CanonicalWriter, key: str, **extra: object) -> uuid.UUID:
    return w.kind(
        "parameterization",
        {"key": key, "revision": "1", "title": key, "coherence": "independent_records", **extra},
        origins=at(key),
    )


def write_reactions(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    """One reaction of each kind the dimension distinguishes, each with a parameter set in
    the unit its source states."""
    gas = {name: species_form(w, decl, name, "gas") for name in ("A", "B", "C", "M2", "R", "R2")}
    site = species_form(w, decl, "site", "surface")
    bound = species_form(w, decl, "bound", "surface")
    ids.update({f"form_{name}": form for name, form in gas.items()})
    ids.update(form_site=site, form_bound=bound)
    conventions = w.kind(
        "convention_set",
        {
            "key": "rates",
            "revision": "1",
            "temperature_scale": "its_90",
            "gas_constant": Quantity(8.314462618, "J/(mol*K)"),
        },
        origins=at("conventions"),
    )
    p = parameterization(w, "rates", convention_set=conventions)
    ids["parameterization"] = p
    reactions = {
        # A + B = C
        "bimolecular": reaction(w, "A + B = C", {gas["A"]: -1, gas["B"]: -1, gas["C"]: 1}),
        # A = B
        "unimolecular": reaction(w, "A = B", {gas["A"]: -1, gas["B"]: 1}),
        # 2 R (+ M) = R2
        "recombination": reaction(w, "2 R (+M) = R2", {gas["R"]: -2, gas["R2"]: 1}),
        # A + site* = bound*
        "adsorption": reaction(w, "A + * = A*", {gas["A"]: -1, site: -1, bound: 1}),
        # A + B = C with fractional orders stated
        "fractional": reaction(
            w, "A + B => C, orders 1.5 and 1", {gas["A"]: -1, gas["B"]: -1, gas["C"]: 1}
        ),
        # A = B with a non-reactant in the rate law
        "inhibited": reaction(w, "A = B, inhibited by C", {gas["A"]: -1, gas["B"]: 1}),
    }
    ids.update({f"reaction_{name}": value for name, value in reactions.items()})
    for form, order in ((gas["A"], 1.5), (gas["B"], 1.0)):
        w.relation(
            "reaction_order",
            {"reaction": reactions["fractional"], "form": form},
            {"value": order},
            at="a.json#/o",
        )
    w.relation(  # a species that is not a reactant, and a stated zero for a reactant
        "reaction_order",
        {"reaction": reactions["inhibited"], "form": gas["C"]},
        {"value": -1.0},
        at="a.json#/o",
    )
    w.relation(
        "reaction_order",
        {"reaction": reactions["inhibited"], "form": gas["A"]},
        {"value": 0.0},
        at="a.json#/o",
    )
    arrhenius = {
        "bimolecular": Quantity(3.0e12, "cm^3/(mol*s)"),
        "unimolecular": Quantity(2.0e13, "1/s"),
        "adsorption": Quantity(4.0e9, "cm^3/(mol*s)"),
        "fractional": Quantity(1.0e9, "(cm^3/mol)^1.5/s"),
        "inhibited": Quantity(7.0e2, "mol^2/(m^6*s)"),
    }
    for name, factor in arrhenius.items():
        ids[f"set_{name}"] = w.parameter_set(
            parameterization=p,
            slot_group=ARRHENIUS,
            subjects=[reactions[name]],
            slots={"A": factor, "b": 0.5, "Ea": Quantity(40.0, "kJ/mol")},
            origins=at(f"a-{name}"),
        )
    ids["set_falloff"] = w.parameter_set(
        parameterization=p,
        slot_group=LINDEMANN,
        subjects=[reactions["recombination"]],
        slots={
            "k_inf": Quantity(1.0e13, "cm^3/(mol*s)"),
            "k_0": Quantity(5.0e19, "cm^6/(mol^2*s)"),
        },
        origins=at("falloff"),
    )
    ids["set_falloff_unimolecular"] = w.parameter_set(
        parameterization=p,
        slot_group=LINDEMANN,
        subjects=[reactions["unimolecular"]],
        slots={"k_inf": Quantity(1.0e6, "1/s"), "k_0": Quantity(2.0e12, "cm^3/(mol*s)")},
        origins=at("falloff-uni"),
    )
    ids["set_table"] = w.parameter_set(
        parameterization=p,
        slot_group=TABLE,
        subjects=[reactions["adsorption"]],
        slots={},
        families={
            "row": [
                FamilyRow(
                    {"n": 1}, {"P": Quantity(1.0, "atm"), "k": Quantity(1.0e9, "cm^3/(mol*s)")}
                ),
                FamilyRow(
                    {"n": 2}, {"P": Quantity(10.0, "atm"), "k": Quantity(2.0e9, "cm^3/(mol*s)")}
                ),
            ]
        },
        origins=at("table"),
    )


@pytest.fixture(scope="module")
def world(decl: Declaration, tmp_path_factory: pytest.TempPathFactory) -> Iterator[World]:
    canonical = tmp_path_factory.mktemp("rate-canonical")
    ids: dict[str, uuid.UUID] = {}
    write_source(
        canonical,
        "src",
        lambda w: write_reactions(w, decl, ids),
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


def stored(conn: psycopg.Connection, table: str, column: str, where: uuid.UUID) -> float:
    row = conn.execute(f'SELECT "{column}" FROM param."{table}" WHERE id = %s', (where,)).fetchone()  # type: ignore[arg-type]
    assert row is not None
    return row[0]


def fresh(decl: Declaration) -> CanonicalWriter:
    return writer(decl)


# -- the declaration ----------------------------------------------------------------------------


def test_the_fixture_declaration_with_both_forms_loads_and_its_expressions_close(
    decl: Declaration,
) -> None:
    """The Arrhenius expression, whose factor has the dimension of the reaction, closes, and so
    does the Lindemann expression, whose low-pressure limit has one more concentration power."""
    assert decl.forms["arrhenius_fixture"].status == "expressed"
    assert decl.forms["lindemann_fixture"].status == "expressed"
    rule = decl.quantity_types["RateConstant"].dependent
    assert rule is not None and rule.on == "reaction"


def test_a_declaration_sees_the_dependent_dimension_as_one_opaque_symbol(decl: Declaration) -> None:
    group = next(g for g in decl.slot_groups if g.qualified == LINDEMANN)
    slots = {s.name: s for s in group.slots}
    plain = type_dimension(decl, slots["k_inf"].type, slots["k_inf"].extra_order)
    low = type_dimension(decl, slots["k_0"].type, slots["k_0"].extra_order)
    assert plain is not None and low is not None
    assert describe(plain) == "[RateConstant]"
    assert low == plain * decl_concentration(decl) ** -1


def decl_concentration(decl: Declaration):  # noqa: ANN201
    from thermo_knowledge.expression.units import from_info

    rule = decl.quantity_types["RateConstant"].dependent
    assert rule is not None
    return from_info(decl.units[rule.concentration])


def test_the_aggregation_vocabulary_says_which_states_are_counted_per_area(
    decl: Declaration,
) -> None:
    surface = next(e for e in decl.entities if e.kind == "aggregation" and e.name == "surface")
    gas = next(e for e in decl.entities if e.kind == "aggregation" and e.name == "gas")
    per_area = decl.enum_members_with("concentration_domain", "per_area")
    assert surface.values["concentration_domain"] in per_area
    assert gas.values["concentration_domain"] not in per_area


# -- the concrete unit, per set, at load --------------------------------------------------------


def test_a_bimolecular_gas_reaction_takes_a_rate_constant_in_cubic_metres_per_mole_and_second(
    world: World, conn: psycopg.Connection
) -> None:
    value = stored(conn, "arrhenius_fixture__rate", "A", world.ids["set_bimolecular"])
    assert value == pytest.approx(3.0e12 * 1e-6)  # cm^3/(mol s) to m^3/(mol s)


def test_a_unimolecular_reaction_takes_a_rate_constant_in_reciprocal_seconds(
    world: World, conn: psycopg.Connection
) -> None:
    assert stored(
        conn, "arrhenius_fixture__rate", "A", world.ids["set_unimolecular"]
    ) == pytest.approx(2.0e13)


def test_a_falloff_low_pressure_limit_has_one_more_concentration_power(
    world: World, conn: psycopg.Connection
) -> None:
    """2 R (+M) = R2: the high-pressure limit is second order (m^3 mol^-1 s^-1) and the low-pressure
    limit m^6 mol^-2 s^-1. For a unimolecular falloff the limits are s^-1 and m^3 mol^-1 s^-1."""
    table = "lindemann_fixture__limits"
    assert stored(conn, table, "k_inf", world.ids["set_falloff"]) == pytest.approx(1.0e13 * 1e-6)
    assert stored(conn, table, "k_0", world.ids["set_falloff"]) == pytest.approx(5.0e19 * 1e-12)
    assert stored(conn, table, "k_inf", world.ids["set_falloff_unimolecular"]) == pytest.approx(
        1.0e6
    )
    assert stored(conn, table, "k_0", world.ids["set_falloff_unimolecular"]) == pytest.approx(
        2.0e12 * 1e-6
    )


def test_a_surface_reaction_with_a_bulk_and_a_surface_reactant_is_a_rate_per_area(
    world: World, conn: psycopg.Connection
) -> None:
    """mol m^-2 s^-1 over (mol m^-3)(mol m^-2) is m^3 mol^-1 s^-1 (the same as a bimolecular gas
    reaction, reached through the per-area rate and the per-area concentration)."""
    assert stored(
        conn, "arrhenius_fixture__rate", "A", world.ids["set_adsorption"]
    ) == pytest.approx(4.0e9 * 1e-6)


def test_an_explicit_fractional_order_fixes_the_dimension(
    world: World, conn: psycopg.Connection
) -> None:
    """Orders 1.5 and 1 give m^4.5 mol^-1.5 s^-1: a value in (cm^3/mol)^1.5 per second converts
    by a factor of 1e-9."""
    assert stored(
        conn, "arrhenius_fixture__rate", "A", world.ids["set_fractional"]
    ) == pytest.approx(1.0e9 * 1e-9)


def test_explicit_orders_replace_the_stoichiometric_default_and_may_name_a_non_reactant(
    world: World, conn: psycopg.Connection
) -> None:
    """A = B with the order of A stated as zero and of the product C as minus one: the rate
    constant is a rate per volume times one concentration, mol^2 m^-6 s^-1. The stoichiometric
    order of A (one) is replaced by the stated zero, and C, which is no reactant, takes part."""
    assert stored(
        conn, "arrhenius_fixture__rate", "A", world.ids["set_inhibited"]
    ) == pytest.approx(7.0e2)


def test_a_family_slot_of_a_dependent_type_takes_the_dimension_of_the_reaction(
    world: World, conn: psycopg.Connection
) -> None:
    rows = conn.execute(
        'SELECT "k" FROM param."rate_table_fixture__plog__row" WHERE set_id = %s ORDER BY "n"',
        (world.ids["set_table"],),
    ).fetchall()
    assert [r[0] for r in rows] == pytest.approx([1.0e9 * 1e-6, 2.0e9 * 1e-6])


def test_every_fixture_satisfies_every_invariant(world: World, conn: psycopg.Connection) -> None:
    results = run_checks(conn, list(CHECKS.values()))
    assert [(r.check.target, r.violations, r.error) for r in results if not r.passed] == []


# -- the refusals of the writer -----------------------------------------------------------------


def fresh_reaction(decl: Declaration) -> tuple[CanonicalWriter, uuid.UUID, uuid.UUID]:
    """A writer with a bimolecular gas reaction and a parameterisation, and nothing else."""
    w = fresh(decl)
    a, b, c = (species_form(w, decl, name, "gas") for name in ("A", "B", "C"))
    written = reaction(w, "A + B = C", {a: -1, b: -1, c: 1})
    conventions = w.kind(
        "convention_set",
        {
            "key": "rates",
            "revision": "1",
            "temperature_scale": "its_90",
            "gas_constant": Quantity(8.314462618, "J/(mol*K)"),
        },
        origins=at("conventions"),
    )
    return w, written, parameterization(w, "rates", convention_set=conventions)


def write_arrhenius(
    w: CanonicalWriter, p: uuid.UUID, subject: uuid.UUID, factor: object
) -> uuid.UUID:
    return w.parameter_set(
        parameterization=p,
        slot_group=ARRHENIUS,
        subjects=[subject],
        slots={"A": factor, "b": 0.0, "Ea": Quantity(0.0, "J/mol")},
        origins=PUBLISHED,
    )


def test_a_set_whose_stated_unit_does_not_match_its_reaction_is_refused(decl: Declaration) -> None:
    w, written, p = fresh_reaction(decl)
    with pytest.raises(
        ValidationError, match=r"A:.*cannot be converted to `meter \*\* 3 / mole / second`"
    ):
        write_arrhenius(
            w, p, written, Quantity(1.0e6, "1/s")
        )  # a first-order unit for a second-order reaction
    with pytest.raises(ValidationError, match="cannot be converted"):
        write_arrhenius(w, p, written, Quantity(1.0, "cm^6/(mol^2*s)"))
    assert w.rows("param.arrhenius_fixture__rate") == 0


def test_a_set_with_the_right_unit_for_the_reaction_is_written(decl: Declaration) -> None:
    w, written, p = fresh_reaction(decl)
    write_arrhenius(w, p, written, Quantity(1.0e6, "cm^3/(mol*s)"))
    assert w.rows("param.arrhenius_fixture__rate") == 1


def test_a_bare_number_has_no_unit_for_a_dimensioned_reaction(decl: Declaration) -> None:
    w, written, p = fresh_reaction(decl)
    with pytest.raises(ValidationError, match="has no unit"):
        write_arrhenius(w, p, written, 1.0e6)


def test_a_stated_order_changes_what_unit_is_accepted(decl: Declaration) -> None:
    """The same reaction with orders 2 and 1 needs m^6 mol^-2 s^-1: the bimolecular unit is now
    refused."""
    w = fresh(decl)
    a, b, c = (species_form(w, decl, name, "gas") for name in ("A", "B", "C"))
    written = reaction(w, "A + B = C", {a: -1, b: -1, c: 1})
    w.relation("reaction_order", {"reaction": written, "form": a}, {"value": 2.0}, at="a.json#/o")
    conventions = w.kind(
        "convention_set",
        {
            "key": "rates",
            "revision": "1",
            "temperature_scale": "its_90",
            "gas_constant": Quantity(8.314462618, "J/(mol*K)"),
        },
        origins=at("conventions"),
    )
    p = parameterization(w, "rates", convention_set=conventions)
    with pytest.raises(ValidationError, match="cannot be converted"):
        write_arrhenius(w, p, written, Quantity(1.0, "m^3/(mol*s)"))
    write_arrhenius(w, p, written, Quantity(1.0, "m^6/(mol^2*s)"))


def test_a_reaction_with_no_participant_written_yet_is_refused_not_guessed(
    decl: Declaration,
) -> None:
    w, _, p = fresh_reaction(decl)
    bare = w.kind("reaction", {"canonical_key": "bare", "extent": "as_written"}, origins=at("bare"))
    with pytest.raises(ValidationError, match="has no participant written before this set"):
        write_arrhenius(w, p, bare, Quantity(1.0, "1/s"))


def test_a_participant_whose_phase_is_unknown_is_refused(decl: Declaration) -> None:
    """A participant that is neither written by this writer nor resolved has no known phase, so
    the dimension cannot be derived."""
    w = fresh(decl)
    unknown = uuid.uuid4()
    written = reaction(w, "X = Y", {unknown: -1})
    p = parameterization(w, "rates")
    with pytest.raises(ValidationError, match="phase of species form .* is not known"):
        write_arrhenius(w, p, written, Quantity(1.0, "1/s"))


def test_a_resolved_species_form_gives_the_phase_a_mapping_run_does_not_write(
    decl: Declaration,
) -> None:
    """A mapping run writes no species form: the aggregation of each form resolution wrote is
    given to the writer, and a surface participant makes the rate per area."""
    from mapping_support import carrier as make_carrier
    from thermo_knowledge.canonical.provenance import Carriers

    gas_form, surface_form = uuid.uuid4(), uuid.uuid4()
    registry = Carriers()
    registry.add(make_carrier("src", "a.json", "b.json"))
    w = CanonicalWriter(
        decl,
        registry,
        form_aggregations={
            gas_form: entity_id(decl, "aggregation", "gas"),
            surface_form: entity_id(decl, "aggregation", "surface"),
        },
    )
    written = reaction(w, "G = S*", {gas_form: -1, surface_form: 1})
    p = parameterization(w, "rates")
    with pytest.raises(ValidationError, match="cannot be converted"):
        write_arrhenius(w, p, written, Quantity(1.0, "1/s"))
    write_arrhenius(w, p, written, Quantity(1.0, "m/s"))  # mol m^-2 s^-1 over mol m^-3


# -- the refusals of the declaration ------------------------------------------------------------


def diagnostics(tmp_path: Path, edits: dict) -> list:
    result = broken(tmp_path, edits)
    assert result.declaration is None
    return list(result.diagnostics)


def codes(found: list) -> set[str]:
    return {d.code for d in found}


def test_a_dependent_type_is_refused_where_no_subject_gives_it_a_dimension(tmp_path: Path) -> None:
    found = diagnostics(
        tmp_path,
        {
            "model/reactions.toml": replace(
                'equation = { type = "SourceText", optional = true,',
                'rate = { type = "RateConstant", optional = true, doc = "A rate." }\nequation = { type = "SourceText", optional = true,',
            )
        },
    )
    assert codes(found) == {Code.BAD_DEPENDENT}
    assert "only a slot or a contract output has it" in found[0].message


def test_a_slot_of_a_dependent_type_needs_exactly_one_subject_of_the_reaction_kind(
    tmp_path: Path,
) -> None:
    found = diagnostics(
        tmp_path,
        {
            "forms/rate_forms.toml": replace(
                'subject.r = { type = "reaction", doc = "The reaction." }\n\n[forms.arrhenius_fixture.slot_groups.rate.slots]',
                'subject.r = { type = "reaction", doc = "The reaction." }\nsubject.s = { type = "reaction", doc = "Another reaction." }\n\n[forms.arrhenius_fixture.slot_groups.rate.slots]',
            )
        },
    )
    assert Code.BAD_DEPENDENT in codes(found)
    assert any("exactly one such role (it has 2)" in d.message for d in found)


def test_a_slot_of_a_dependent_type_in_a_group_about_no_reaction_is_refused(tmp_path: Path) -> None:
    found = diagnostics(
        tmp_path,
        {
            "forms/rate_forms.toml": replace(
                '[forms.arrhenius_fixture.slot_groups.rate]\ndoc = "The three coefficients of one reaction."\nsubject.r = { type = "reaction", doc = "The reaction." }',
                '[forms.arrhenius_fixture.slot_groups.rate]\ndoc = "The three coefficients of one reaction."\nsubject.r = { type = "species", doc = "Not a reaction." }',
            )
        },
    )
    assert Code.BAD_DEPENDENT in codes(found)
    assert any(
        "from the subject of kind `reaction`" in d.message and "(it has 0)" in d.message
        for d in found
    )


def test_an_extra_order_belongs_to_a_dependent_type_and_is_not_negative(tmp_path: Path) -> None:
    found = diagnostics(
        tmp_path,
        {
            "forms/rate_forms.toml": replace(
                'b = { type = "Scalar", doc = "Temperature exponent." }',
                'b = { type = "Scalar", extra_order = 1, doc = "Temperature exponent." }',
            )
        },
    )
    assert codes(found) == {Code.BAD_DEPENDENT}
    assert "is not one" in found[0].message
    found = diagnostics(
        tmp_path / "negative",
        {
            "forms/rate_forms.toml": replace(
                'k_0 = { type = "RateConstant", extra_order = 1,',
                'k_0 = { type = "RateConstant", extra_order = -1,',
            )
        },
    )
    assert codes(found) == {Code.BAD_DEPENDENT}
    assert "not below zero" in found[0].message


def test_an_output_of_a_dependent_type_needs_a_role_of_the_reaction_kind(tmp_path: Path) -> None:
    found = diagnostics(
        tmp_path,
        {
            "forms/rate_forms.toml": replace(
                'roles.r = { type = "reaction", doc = "The reaction." }\narguments.T = { type = "Temperature", observable = "temperature", doc = "Temperature." }\noutputs.k = { type = "RateConstant", doc = "The rate constant, of the dimension the reaction gives." }',
                'roles.i = { type = "material_entity", doc = "An entity." }\narguments.T = { type = "Temperature", observable = "temperature", doc = "Temperature." }\noutputs.k = { type = "RateConstant", doc = "The rate constant." }',
            )
        },
    )
    assert Code.BAD_DEPENDENT in codes(found)


def test_an_expression_whose_extra_order_differs_from_the_output_is_refused(tmp_path: Path) -> None:
    """The Lindemann output with one more factor of the low-pressure limit has the wrong
    dimension: the opaque symbol appears twice."""
    found = diagnostics(
        tmp_path,
        {
            "forms/rate_forms.toml": replace(
                'k = "limits.k_inf[r] * Pr / (1 + Pr)"', 'k = "limits.k_0[r] * Pr / (1 + Pr)"'
            )
        },
    )
    assert codes(found) == {Code.OUTPUT_DIMENSION}


def test_a_per_area_unit_that_is_not_the_per_volume_unit_per_length_is_refused(
    tmp_path: Path,
) -> None:
    found = diagnostics(
        tmp_path,
        {
            "model/reactions.toml": replace(
                'surface_unit = "mol/(m^2*s)"', 'surface_unit = "mol/(m^3*s)"'
            )
        },
    )
    assert codes(found) == {Code.BAD_DEPENDENT}
    assert "per-area form" in found[0].message


def test_a_dependent_type_names_a_declared_kind(tmp_path: Path) -> None:
    found = diagnostics(
        tmp_path,
        {
            "model/reactions.toml": replace(
                'dependent = { on = "reaction"', 'dependent = { on = "no_such_kind"'
            )
        },
    )
    assert codes(found) == {Code.UNKNOWN_NAME}


def test_a_dependent_type_is_not_a_unit_of_a_quantity_expression(tmp_path: Path) -> None:
    found = diagnostics(
        tmp_path,
        {
            "model/reactions.toml": replace(
                'equation = { type = "SourceText", optional = true,',
                'rate = { type = "RateConstant * Temperature", optional = true, doc = "A rate." }\nequation = { type = "SourceText", optional = true,',
            )
        },
    )
    assert Code.BAD_DEPENDENT in codes(found)


# -- evaluation ---------------------------------------------------------------------------------


def test_the_arrhenius_form_evaluates_in_the_stored_coherent_units(
    world: World, conn: psycopg.Connection
) -> None:
    source = DatabaseSource(conn, world.decl, [world.ids["parameterization"]])
    bound = bind(
        world.decl,
        "arrhenius_fixture",
        source=source,
        roles={"r": str(world.ids["reaction_bimolecular"])},
    )
    temperature = np.array([300.0, 600.0, 1200.0])
    expected = (
        3.0e12 * 1e-6 * (temperature) ** 0.5 * np.exp(-40_000.0 / (8.314462618 * temperature))
    )
    np.testing.assert_allclose(bound.evaluate("k", T=temperature), expected, rtol=1e-12)


def test_the_lindemann_form_reaches_its_limits(world: World, conn: psycopg.Connection) -> None:
    source = DatabaseSource(conn, world.decl, [world.ids["parameterization"]])
    bound = bind(
        world.decl,
        "lindemann_fixture",
        source=source,
        roles={"r": str(world.ids["reaction_recombination"])},
    )
    k_inf, k_0 = 1.0e13 * 1e-6, 5.0e19 * 1e-12
    concentration = np.array([1e-6, 1.0, 1e6])  # mol/m^3: far below, near and far above k_inf/k_0
    pr = k_0 * concentration / k_inf
    np.testing.assert_allclose(
        bound.evaluate("k", T=np.full(3, 300.0), M=concentration), k_inf * pr / (1 + pr), rtol=1e-12
    )
    assert bound.evaluate("k", T=np.array([300.0]), M=np.array([1e9]))[0] == pytest.approx(
        k_inf, rel=1e-3
    )
