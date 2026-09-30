# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""`tk verify`: every check against a database built from the real declaration, with a violation
and a conforming case hand-inserted in a transaction that is rolled back (constraints are
deferrable, so a row need not have its referents), and the framework's own refusals."""

from __future__ import annotations

import json
import re
import shutil
import uuid
from collections.abc import Iterator
from pathlib import Path

import psycopg
import pytest
from mapping_support import extended_directories, real_declaration
from psycopg import sql
from typer.testing import CliRunner

from thermo_knowledge import config
from thermo_knowledge.build import build_database
from thermo_knowledge.cli import app
from thermo_knowledge.declaration import load_declaration
from thermo_knowledge.testing import TestDatabase
from thermo_knowledge.verify.checks import (
    VERIFY_DIR,
    Check,
    agreement_problems,
    load_checks,
    parse,
    verify_invariants,
)
from thermo_knowledge.verify.run import CheckResult, Report, run_check, run_checks, table_lines

runner = CliRunner()
SATURATION = "vapor_pressure_exp_series_tau.pure"
SATURATION_TABLE = "vapor_pressure_exp_series_tau__pure"
REAL_CHECKS, REAL_PROBLEMS = load_checks(config.TREE_DIR / VERIFY_DIR)
CHECKS = {check.target: check for check in REAL_CHECKS}

STRUCTURAL = {
    "record_has_origin",
    "origin_carrier_has_rights",
    "abstract_kind_has_one_refinement",
    "nested_set_implements_contract",
    "family_indices_contiguous",
    "producing_derivation",
    "producing_derivation_is_fit",
    "subject_key_matches_subjects",
    "referenced_set_implements_contract",
    "parameterization_has_conventions",
}
"""Section 4's structural checks. "Reactions conserve every conserved quantity with a
composition entry on a participant" is the declared invariant
`reaction.conserves_declared_quantities`, so one check file covers both."""


@pytest.fixture(scope="module")
def built() -> Iterator[TestDatabase]:
    with TestDatabase() as database:
        build_database(database.url, real_declaration())
        yield database


@pytest.fixture
def conn(built: TestDatabase) -> Iterator[psycopg.Connection]:
    connection = psycopg.connect(built.url)
    connection.execute("SELECT 1")  # an open transaction: what a test inserts is rolled back
    try:
        yield connection
    finally:
        connection.rollback()
        connection.close()


def new() -> uuid.UUID:
    return uuid.uuid4()


def insert(conn: psycopg.Connection, table: str, **values: object) -> None:
    schema, name = table.split(".")
    statement = sql.SQL("INSERT INTO {} ({}) VALUES ({})").format(
        sql.Identifier(schema, name),
        sql.SQL(", ").join(sql.Identifier(column) for column in values),
        sql.SQL(", ").join(sql.Placeholder() for _ in values),
    )
    conn.execute(statement, list(values.values()))


def lookup(conn: psycopg.Connection, query: str, *params: object) -> uuid.UUID:
    row = conn.execute(query, params).fetchone()  # type: ignore[arg-type]
    assert row is not None
    return row[0]  # type: ignore[no-any-return]


def run(conn: psycopg.Connection, target: str) -> CheckResult:
    result = run_check(conn, CHECKS[target], shown=100)
    assert result.error is None, result.error
    return result


def violating(result: CheckResult) -> list[str]:
    return sorted(result.ids)


def ids(*values: uuid.UUID) -> list[str]:
    return sorted(str(value) for value in values)


def provenance(
    conn: psycopg.Connection, record: uuid.UUID, locator: str, role: str = "published"
) -> None:
    """One import record and origin for `record`, so the check can report where it came from."""
    imported = new()
    insert(
        conn,
        "prov.import_record",
        id=imported,
        artifact=new(),
        locator=locator,
        reader="r",
        reader_version="1",
    )
    insert(conn, "prov.record_origin", id=new(), record=record, import_record=imported, role=role)


# -- the committed checks agree with the declaration -----------------------------------------


def test_every_declared_verify_invariant_has_a_check_and_every_check_names_one() -> None:
    assert REAL_PROBLEMS == []
    assert agreement_problems(real_declaration(), REAL_CHECKS) == []
    declared = set(verify_invariants(real_declaration()))
    assert declared == {check.target for check in REAL_CHECKS if check.kind == "invariant"}
    assert declared  # the declaration does declare some
    assert {check.target for check in REAL_CHECKS if check.kind == "structural"} == STRUCTURAL
    for check in REAL_CHECKS:
        assert check.description and check.query


def test_a_verify_invariant_of_a_relation_needs_its_check_file_like_one_of_a_kind() -> None:
    declared = verify_invariants(real_declaration())
    relations = {name for name in declared if name.split(".")[0] in real_declaration().relations}
    assert {
        "derivation_output.one_producer_per_record",
        "mixture_component.fractions_close",
        "supersedes.acyclic",
        "site_equivalence.same_host",
        "group_count.group_in_assignment_scheme",
        "datum.column_in_point_dataset",
        "subject_subform_choice.form_implements_slot_contract",
        "subject_subform_choice.ordinals_contiguous",
        "dependency.acyclic",
        "derivation_input.lineage_acyclic",
        "snapshot_selection.no_failed_fit",
        "datum.column_is_variable_or_property",
        "datum_uncertainty.assessment_in_point_dataset",
        "datum_uncertainty.magnitude_matches_kind",
        "column_uncertainty.column_is_constraint",
        "column_uncertainty.magnitude_matches_kind",
        "validity_coverage.state_matches_regions",
        "system_reaction.conserves_system_quantities",
    } <= relations
    without = [check for check in REAL_CHECKS if check.target != "supersedes.acyclic"]
    assert agreement_problems(real_declaration(), without) == [
        "the declared verify invariant `supersedes.acyclic` has no check file in sql/verify/"
    ]


def test_the_database_as_built_has_no_violations(conn: psycopg.Connection) -> None:
    results = run_checks(conn, REAL_CHECKS)
    assert [(r.check.target, r.violations, r.error) for r in results if not r.passed] == []


# -- the declared invariants -----------------------------------------------------------------


def test_species_charge_matches_composition(conn: psycopg.Connection) -> None:
    charge = lookup(conn, "SELECT id FROM tk.conserved_quantity WHERE key = 'charge'")
    good, bad, unstated = new(), new(), new()
    for species, value, stated in ((good, -1.0, -1.0), (bad, 2.0, 1.0), (unstated, 3.0, None)):
        insert(conn, "tk.species", id=species, charge=value)
        if stated is not None:
            insert(conn, "tk.composition", id=new(), entity=species, quantity=charge, value=stated)
    provenance(conn, bad, "a.json#/9")
    result = run(conn, "species.charge_matches_composition")
    assert violating(result) == ids(bad) and result.violations == 1
    row = dict(zip(result.columns, result.rows[0]))
    assert (
        row["locator"] == "a.json#/9"
        and row["charge"] == "2.0"
        and row["composition_charge"] == "1.0"
    )


def test_polymorph_only_for_crystalline(conn: psycopg.Connection) -> None:
    crystalline = lookup(conn, "SELECT id FROM tk.aggregation WHERE name = 'crystalline'")
    gas = lookup(conn, "SELECT id FROM tk.aggregation WHERE name = 'gas'")
    species, bad, crystal, plain = new(), new(), new(), new()
    insert(conn, "tk.species_form", id=bad, species=species, aggregation=gas, polymorph="alpha")
    insert(
        conn,
        "tk.species_form",
        id=crystal,
        species=species,
        aggregation=crystalline,
        polymorph="alpha",
    )
    insert(conn, "tk.species_form", id=plain, species=species, aggregation=gas)
    assert violating(run(conn, "species_form.polymorph_only_for_crystalline")) == ids(bad)


def test_provisional_target_matches_status(conn: psycopg.Connection) -> None:
    canonical, provisional = new(), new()
    insert(
        conn, "tk.material_entity", id=canonical, canonical_key="C", label="C", provisional=False
    )
    insert(
        conn, "tk.material_entity", id=provisional, canonical_key="P", label="P", provisional=True
    )
    cases = {
        "unique_canonical": ("unique", canonical, True),
        "unique_provisional": ("unique", provisional, False),
        "ambiguous_provisional": ("ambiguous", provisional, True),
        "unresolved_canonical": ("unresolved", canonical, False),
        "rejected_provisional": ("rejected", provisional, True),
    }
    bad: list[uuid.UUID] = []
    for key, (status, target, conforming) in cases.items():
        entity = new()
        insert(
            conn,
            "tk.source_entity",
            id=entity,
            carrier=new(),
            scope="s",
            local_key=key,
            entity_class="species",
            status=status,
            rule="structural",
            target=target,
        )
        if not conforming:
            bad.append(entity)
    assert violating(run(conn, "source_entity.provisional_target_matches_status")) == ids(*bad)


def test_constant_site_ratio_present(conn: psycopg.Connection) -> None:
    cases = {
        "constant_stated": ("constant", 2.0, True),
        "constant_missing": ("constant", None, False),
        "constant_zero": ("constant", 0.0, False),
        "dependent_absent": ("composition_dependent", None, True),
        "dependent_stated": ("composition_dependent", 1.5, False),
    }
    bad: list[uuid.UUID] = []
    for position, (host, (kind, ratio, conforming)) in enumerate(cases.items(), 1):
        site = new()
        insert(
            conn,
            "tk.site_class",
            id=site,
            host_key=host,
            index=position,
            phase=new(),
            ratio_kind=kind,
            ratio=ratio,
        )
        if not conforming:
            bad.append(site)
    assert violating(run(conn, "site_class.constant_ratio_present")) == ids(*bad)


def test_group_parent_is_acyclic_and_in_the_same_scheme(conn: psycopg.Connection) -> None:
    scheme, other = new(), new()
    root, child, foreign_child, foreign_class, classed = (new() for _ in range(5))
    loop_a, loop_b, itself = new(), new(), new()

    def group(identifier: uuid.UUID, in_scheme: uuid.UUID, label: str, **links: uuid.UUID) -> None:
        insert(
            conn, "tk.group", id=identifier, scheme=in_scheme, label=label, role="group", **links
        )

    group(root, scheme, "root")
    group(child, scheme, "child", parent=root)
    group(classed, scheme, "classed", partition_class=root)
    group(foreign_child, other, "foreign child", parent=root)
    group(foreign_class, other, "foreign class", partition_class=root)
    group(loop_a, scheme, "a", parent=loop_b)
    group(loop_b, scheme, "b", parent=loop_a)
    group(itself, scheme, "self", parent=itself)
    result = run(conn, "group.parent_acyclic_same_scheme")
    assert violating(result) == ids(foreign_child, foreign_class, loop_a, loop_b, itself)
    reasons = {row[0]: row[result.columns.index("reason")] for row in result.rows}
    assert reasons[str(foreign_child)] == "the parent is in another scheme"
    assert reasons[str(foreign_class)] == "the partition class is in another scheme"
    assert reasons[str(loop_a)] == reasons[str(itself)] == "the parent chain contains a cycle"


def test_reactions_conserve_every_quantity_with_a_composition_entry(
    conn: psycopg.Connection,
) -> None:
    hydrogen, oxygen = (
        lookup(conn, "SELECT id FROM tk.conserved_quantity WHERE key = %s", symbol)
        for symbol in "HO"
    )
    gas = lookup(conn, "SELECT id FROM tk.aggregation WHERE name = 'gas'")
    formulas = {"H2": {hydrogen: 2.0}, "O2": {oxygen: 2.0}, "H2O": {hydrogen: 2.0, oxygen: 1.0}}
    form: dict[str, uuid.UUID] = {}
    for name, composition in formulas.items():
        species = new()
        form[name] = new()
        insert(conn, "tk.species_form", id=form[name], species=species, aggregation=gas)
        for quantity, value in composition.items():
            insert(conn, "tk.composition", id=new(), entity=species, quantity=quantity, value=value)
    # an entry on the form itself wins over the species' and is not counted twice
    insert(conn, "tk.composition", id=new(), entity=form["H2"], quantity=hydrogen, value=2.0)

    def reaction(key: str, coefficients: dict[str, float]) -> uuid.UUID:
        identifier = new()
        insert(conn, "tk.reaction", id=identifier, canonical_key=key, extent="as_written")
        for name, coefficient in coefficients.items():
            insert(
                conn,
                "tk.reaction_participant",
                id=new(),
                reaction=identifier,
                form=form[name],
                coefficient=coefficient,
            )
        return identifier

    reaction("balanced", {"H2": -2.0, "O2": -1.0, "H2O": 2.0})
    unbalanced = reaction("unbalanced", {"H2": -1.0, "O2": -1.0, "H2O": 1.0})
    result = run(conn, "reaction.conserves_declared_quantities")
    assert violating(result) == ids(unbalanced)
    row = dict(zip(result.columns, result.rows[0]))
    assert row["quantity"] == "O" and float(row["net"]) == -1.0


def test_an_absolute_tolerance_needs_an_observable(conn: psycopg.Connection) -> None:
    basis, outcome = (
        lookup(
            conn,
            "SELECT enumlabel::text FROM pg_enum WHERE enumtypid = %s::regtype ORDER BY enumsortorder LIMIT 1",
            t,
        )
        for t in ("meta.comparison_basis", "meta.run_outcome")
    )
    cases = {
        "absolute_with_observable": (new(), 1e-9, True),
        "absolute_without": (None, 1e-9, False),
        "relative_only": (None, None, True),
        "observable_only": (new(), None, True),
    }
    bad: list[uuid.UUID] = []
    for key, (observable, absolute, conforming) in cases.items():
        run_id = new()
        insert(
            conn,
            "qual.qualification_run",
            id=run_id,
            key=key,
            form=new(),
            expression_hash=b"\x00" * 32,
            basis=basis,
            reference=new(),
            outcome=outcome,
            points=1,
            relative_tolerance=1e-6,
            observable=observable,
            absolute_tolerance=absolute,
        )
        if not conforming:
            bad.append(run_id)
    result = run(conn, "qualification_run.absolute_tolerance_needs_observable")
    assert violating(result) == ids(*bad)
    assert result.rows[0][result.columns.index("key")] == "absolute_without"


def test_an_uncertainty_magnitude_matches_its_kind(conn: psycopg.Connection) -> None:
    # kind: (magnitude, relative magnitude) that conforms, then one that does not
    rules = {
        "relative": ((None, 0.1), (1.0, 0.1)),
        "relative_curve_deviation": ((None, 0.1), (None, None)),
        "standard": ((1.0, None), (None, None)),
        "expanded": ((1.0, None), (None, 0.1)),
        "combined_standard": ((1.0, None), (1.0, 0.1)),
        "combined_expanded": ((1.0, None), (None, None)),
        "interval": ((1.0, None), (1.0, 0.1)),
        "repeatability_single_biased": ((1.0, None), (None, 0.1)),
        "repeatability_single_unbiased": ((1.0, None), (None, None)),
        "repeatability_of_mean": ((1.0, None), (1.0, 0.1)),
        "repeatability_other": ((1.0, None), (None, 0.1)),
        "device_specification": ((1.0, None), (None, None)),
        "curve_deviation": ((1.0, None), (None, 0.1)),
        "exact": ((None, None), (1.0, None)),
        "not_stated": ((None, None), (None, 0.1)),
    }
    bad: list[uuid.UUID] = []
    parameter_set, slot = new(), new()
    for kind, (good, wrong) in rules.items():
        for label, (magnitude, relative) in (("good", good), ("wrong", wrong)):
            identifier = new()
            insert(
                conn,
                "tk.slot_uncertainty",
                id=identifier,
                parameter_set=parameter_set,
                slot=slot,
                index_key=f"{kind}-{label}",
                kind=kind,
                magnitude=magnitude,
                relative_magnitude=relative,
            )
            if label == "wrong":
                bad.append(identifier)
    result = run(conn, "slot_uncertainty.magnitude_matches_kind")
    assert violating(result) == ids(*bad) and result.violations == len(rules)


def test_tabulated_axis_points_are_strictly_ascending(conn: psycopg.Connection) -> None:
    function = new()
    insert(conn, "tk.tabulated_function", id=function, key="f", interpolation="linear")
    provenance(conn, function, "a.json#/4")
    quantity_type = lookup(conn, "SELECT id FROM meta.quantity_type LIMIT 1")
    axes = {
        "ascending": [1.0, 2.0, 3.5],
        "single": [4.0],
        "repeated": [1.0, 2.0, 2.0],
        "descending": [3.0, 2.0, 1.0],
        "dip": [1.0, 3.0, 2.0, 5.0],
    }
    identifiers: dict[str, uuid.UUID] = {}
    for ordinal, (name, points) in enumerate(axes.items(), start=1):
        identifiers[name] = new()
        insert(
            conn,
            "tk.tabulated_axis",
            id=identifiers[name],
            function=function,
            ordinal=ordinal,
            axis_type=quantity_type,
            points=points,
        )
    result = run(conn, "tabulated_axis.points_ascending")
    assert violating(result) == ids(
        *(identifiers[name] for name in ("repeated", "descending", "dip"))
    )
    found = {r[0]: dict(zip(result.columns, r)) for r in result.rows}
    assert found[str(identifiers["repeated"])]["first_out_of_order"] == "3"
    assert found[str(identifiers["dip"])]["first_out_of_order"] == "3"
    assert found[str(identifiers["dip"])]["locator"] == "a.json#/4"


def test_tabulated_series_values_cover_the_grid_of_the_axes(conn: psycopg.Connection) -> None:
    quantity_type = lookup(conn, "SELECT id FROM meta.quantity_type LIMIT 1")
    functions = {name: new() for name in ("grid", "line", "no_axis", "short")}
    for name, function in functions.items():
        insert(conn, "tk.tabulated_function", id=function, key=name, interpolation="linear")
    provenance(conn, functions["short"], "a.json#/6")
    for name, lengths in (("grid", (2, 3)), ("line", (4,)), ("short", (2, 2))):
        for ordinal, length in enumerate(lengths, start=1):
            insert(
                conn,
                "tk.tabulated_axis",
                id=new(),
                function=functions[name],
                ordinal=ordinal,
                axis_type=quantity_type,
                points=[float(n) for n in range(length)],
            )
    series: dict[str, uuid.UUID] = {}
    for name, function, count in (
        ("grid_full", "grid", 6),
        ("grid_one_axis_only", "grid", 3),
        ("grid_second", "grid", 6),
        ("line_full", "line", 4),
        ("line_empty", "line", 0),
        ("no_axis_one", "no_axis", 1),
        ("no_axis_many", "no_axis", 2),
        ("short_full", "short", 4),
        ("short_three", "short", 3),
    ):
        series[name] = new()
        insert(
            conn,
            "tk.tabulated_series",
            id=series[name],
            function=functions[function],
            name=name,
            value_type=quantity_type,
            values=[1.0] * count,
        )
    result = run(conn, "tabulated_series.values_cover_grid")
    assert violating(result) == ids(
        *(series[n] for n in ("grid_one_axis_only", "line_empty", "no_axis_many", "short_three"))
    )
    found = {r[0]: dict(zip(result.columns, r)) for r in result.rows}
    row = found[str(series["short_three"])]
    assert (row["values"], row["grid_points"], row["shape"], row["locator"]) == (
        "3",
        "4",
        "2 x 2",
        "a.json#/6",
    )
    assert found[str(series["grid_one_axis_only"])]["grid_points"] == "6"
    assert found[str(series["no_axis_many"])]["shape"] == "no axes"


# -- the structural checks -------------------------------------------------------------------


def test_every_record_has_an_origin_unless_it_is_a_declared_entity(
    conn: psycopg.Connection,
) -> None:
    declared = lookup(conn, "SELECT id FROM meta.entity e JOIN prov.record r USING (id) LIMIT 1")
    assert declared  # a declared entity's record has no origin and conforms
    origin, orphan = new(), new()
    insert(conn, "prov.record", id=origin, kind="species")
    insert(conn, "prov.record", id=orphan, kind="species")
    provenance(conn, origin, "a.json#/1")
    result = run(conn, "record_has_origin")
    assert violating(result) == ids(orphan)
    assert result.rows[0][result.columns.index("kind")] == "species"


def test_every_origin_carrier_has_a_rights_determination(conn: psycopg.Connection) -> None:
    carriers = {name: new() for name in ("stated", "silent", "unused")}
    for name, carrier in carriers.items():
        insert(
            conn,
            "prov.carrier",
            id=carrier,
            manifest_id=name,
            resolved_pin="p",
            tree_hash=b"\x01" * 32,
            retrieved="2026-09-30T00:00:00Z",
        )
    insert(
        conn,
        "prov.rights_determination",
        id=new(),
        carrier=carriers["stated"],
        scope="data",
        ordinal=1,
        basis="not_stated",
        statement="The terms are silent.",
        store="not_stated",
        redistribute="not_stated",
        commercial="not_stated",
        attribution_required=False,
        share_alike=False,
        observed="a test",
    )
    for name in ("stated", "silent"):
        artifact, imported, record = new(), new(), new()
        insert(
            conn,
            "prov.artifact",
            id=artifact,
            carrier=carriers[name],
            path=f"{name}.json",
            sha256=b"\x02" * 32,
            size=1,
        )
        insert(
            conn,
            "prov.import_record",
            id=imported,
            artifact=artifact,
            locator=f"{name}.json#/0",
            reader="r",
            reader_version="1",
        )
        insert(
            conn,
            "prov.record_origin",
            id=new(),
            record=record,
            import_record=imported,
            role="published",
        )
    result = run(conn, "origin_carrier_has_rights")
    assert violating(result) == ids(carriers["silent"])
    row = dict(zip(result.columns, result.rows[0]))
    assert row["manifest_id"] == "silent" and row["locator"] == "silent.json#/0"


def test_an_abstract_row_has_exactly_one_concrete_refinement(conn: psycopg.Connection) -> None:
    aggregation = lookup(conn, "SELECT id FROM tk.aggregation WHERE name = 'gas'")
    one, none, two = new(), new(), new()
    for entity in (one, none, two):
        insert(conn, "tk.material_entity", id=entity, canonical_key=str(entity), label="x")
    insert(conn, "tk.species", id=one)
    insert(conn, "tk.species", id=two)
    insert(conn, "tk.species_form", id=two, species=one, aggregation=aggregation)
    # `parameter_set`'s refinements are the slot-group tables the declaration generates
    parameterization, slot_group = new(), lookup(conn, "SELECT id FROM meta.slot_group LIMIT 1")
    described, bare = new(), new()
    for subject, identifier in (("described", described), ("bare", bare)):
        insert(
            conn,
            "tk.parameter_set",
            id=identifier,
            parameterization=parameterization,
            slot_group=slot_group,
            subject_key=subject,
        )
    insert(
        conn, "param.vapor_pressure_exp_series_tau__pure", id=described, i=new(), T_r=300.0, p_r=1e5
    )
    # a source is refined by carrier, publication and software release
    carrier, unrefined = new(), new()
    for source in (carrier, unrefined):
        insert(conn, "prov.source", id=source, key=str(source), title="t")
    insert(
        conn,
        "prov.carrier",
        id=carrier,
        manifest_id="m",
        resolved_pin="p",
        tree_hash=b"\x03" * 32,
        retrieved="2026-09-30T00:00:00Z",
    )
    result = run(conn, "abstract_kind_has_one_refinement")
    assert violating(result) == ids(none, two, bare, unrefined)
    found = {
        row[0]: (row[result.columns.index("kind")], row[result.columns.index("refinements")])
        for row in result.rows
    }
    assert found[str(none)] == ("material_entity", "0") and found[str(two)] == (
        "material_entity",
        "2",
    )
    assert found[str(bare)] == ("parameter_set", "0") and found[str(unrefined)] == ("source", "0")


def test_a_nested_set_implements_the_contract_its_slot_accepts(conn: psycopg.Connection) -> None:
    def form(name: str, implements: str) -> None:
        insert(
            conn,
            "meta.form",
            id=new(),
            name=name,
            module="m",
            implements=implements,
            completeness="fully_declared",
            status="catalogued",
            doc="d",
        )
        insert(
            conn,
            "meta.slot_group",
            id=groups[name],
            qualified_name=f"{name}.pure",
            form=name,
            name="pure",
            kind=name,
            table_name=name,
            doc="d",
        )

    groups = {name: new() for name in ("parent_form", "child_form", "other_form")}
    form("parent_form", "parent_contract")
    form("child_form", "child_contract")
    form("other_form", "other_contract")
    slot = new()
    insert(
        conn,
        "meta.slot",
        id=slot,
        qualified_name="parent_form.pure.f",
        slot_group="parent_form.pure",
        name="f",
        position=1,
        shape="nested_set",
        accepts="child_contract",
        presence="required",
        doc="d",
    )
    parameterization, parent, good, wrong_contract, no_slot = new(), new(), new(), new(), new()

    def parameter_set(identifier: uuid.UUID, group: str, **links: object) -> None:
        insert(
            conn,
            "tk.parameter_set",
            id=identifier,
            parameterization=parameterization,
            slot_group=groups[group],
            subject_key=str(identifier),
            **links,
        )

    parameter_set(parent, "parent_form")
    parameter_set(good, "child_form", parent=parent, parent_slot=slot)
    parameter_set(wrong_contract, "other_form", parent=parent, parent_slot=slot)
    parameter_set(no_slot, "child_form", parent=parent)
    result = run(conn, "nested_set_implements_contract")
    assert violating(result) == ids(wrong_contract, no_slot)
    row = {r[0]: dict(zip(result.columns, r)) for r in result.rows}[str(wrong_contract)]
    assert (
        row["accepted_contract"] == "child_contract"
        and row["implemented_contract"] == "other_contract"
    )


def test_family_indices_are_contiguous_from_their_minimum(conn: psycopg.Connection) -> None:
    sets = {name: new() for name in ("contiguous", "gap", "late_start")}
    for name, indices in (("contiguous", (1, 2, 3)), ("gap", (1, 2, 4)), ("late_start", (2, 3, 4))):
        for index in indices:
            insert(
                conn,
                "param.vapor_pressure_exp_series_tau__pure__term",
                set_id=sets[name],
                k=index,
                n=1.0,
                t=1.0,
            )
    result = run(conn, "family_indices_contiguous")
    assert violating(result) == ids(sets["gap"], sets["late_start"])
    by_set = {r[0]: dict(zip(result.columns, r)) for r in result.rows}
    assert (
        by_set[str(sets["gap"])]["highest"] == "4"
        and by_set[str(sets["gap"])]["distinct_values"] == "3"
    )
    assert by_set[str(sets["late_start"])]["lowest"] == "2"
    assert by_set[str(sets["gap"])]["family"] == "vapor_pressure_exp_series_tau.pure.term"


def test_derivation_lineage_is_acyclic(conn: psycopg.Connection) -> None:
    records = {name: new() for name in "ABCDEFG"}

    def derive(key: str, sources: str, products: str, *, excluded: bool = False) -> None:
        derivation = new()
        insert(conn, "prov.derivation", id=derivation, key=key, kind="computation")
        for source in sources:
            insert(
                conn,
                "prov.derivation_input",
                id=new(),
                derivation=derivation,
                record=records[source],
                excluded=excluded,
            )
        for product in products:
            insert(
                conn,
                "prov.derivation_output",
                id=new(),
                derivation=derivation,
                record=records[product],
            )

    derive("ab", "A", "B")  # A -> B -> C: a chain
    derive("bc", "B", "C")
    derive("de", "D", "E")  # D -> E -> D: a cycle
    derive("ed", "E", "D")
    derive("fg", "F", "G")  # G -> F through an excluded input is no lineage
    derive("gf", "G", "F", excluded=True)
    provenance(conn, records["D"], "a.json#/5")
    result = run(conn, "derivation_input.lineage_acyclic")
    assert violating(result) == ids(records["D"], records["E"])
    row = {r[0]: dict(zip(result.columns, r)) for r in result.rows}[str(records["D"])]
    assert row["locator"] == "a.json#/5"


def test_dependencies_are_acyclic(conn: psycopg.Connection) -> None:
    dependent, prerequisite, loop_x, loop_y, chain = (new() for _ in range(5))
    for subject, required in (
        (dependent, prerequisite),  # a chain: prerequisite <- dependent <- chain
        (chain, dependent),
        (loop_x, loop_y),  # a cycle
        (loop_y, loop_x),
    ):
        insert(
            conn,
            "tk.dependency",
            id=new(),
            dependent=subject,
            prerequisite=required,
            kind="fitted_given",
        )
    provenance(conn, loop_x, "a.json#/6")
    result = run(conn, "dependency.acyclic")
    assert violating(result) == ids(loop_x, loop_y)
    row = {r[0]: dict(zip(result.columns, r)) for r in result.rows}[str(loop_x)]
    assert row["locator"] == "a.json#/6"


def test_a_record_is_the_output_of_at_most_one_derivation(conn: psycopg.Connection) -> None:
    once, twice = new(), new()
    first, second, third = new(), new(), new()
    for identifier in (first, second, third):
        insert(conn, "prov.derivation", id=identifier, key=str(identifier), kind="computation")
    insert(conn, "prov.derivation_output", id=new(), derivation=first, record=once)
    insert(conn, "prov.derivation_output", id=new(), derivation=second, record=twice)
    insert(conn, "prov.derivation_output", id=new(), derivation=third, record=twice)
    provenance(conn, twice, "a.json#/2")
    result = run(conn, "derivation_output.one_producer_per_record")
    assert violating(result) == ids(twice)
    row = dict(zip(result.columns, result.rows[0]))
    assert row["producers"] == "2" and row["locator"] == "a.json#/2"


def test_the_fractions_of_a_defined_mixture_close_to_one(conn: psycopg.Connection) -> None:
    mixtures = {
        "exact": (0.5, 0.5),
        "within_tolerance": (0.5, 0.5 + 5e-7),
        "short": (0.5, 0.4),
        "over": (0.6, 0.5),
        "single": (1.0,),
        "off_by_more_than_tolerance": (0.5, 0.5 + 2e-6),
    }
    made: dict[str, uuid.UUID] = {}
    for name, fractions in mixtures.items():
        made[name] = new()
        for fraction in fractions:
            insert(
                conn,
                "tk.mixture_component",
                id=new(),
                mixture=made[name],
                component=new(),
                value=fraction,
            )
    provenance(conn, made["short"], "a.json#/3")
    result = run(conn, "mixture_component.fractions_close")
    assert violating(result) == ids(made["short"], made["over"], made["off_by_more_than_tolerance"])
    row = {r[0]: dict(zip(result.columns, r)) for r in result.rows}[str(made["short"])]
    assert row["locator"] == "a.json#/3" and float(row["total"]) == pytest.approx(0.9)


def test_supersession_is_acyclic(conn: psycopg.Connection) -> None:
    oldest, middle, newest = new(), new(), new()
    loop_a, loop_b, itself = new(), new(), new()
    for newer, older in (
        (newest, middle),
        (middle, oldest),
        (loop_a, loop_b),
        (loop_b, loop_a),
        (itself, itself),
    ):
        insert(conn, "tk.supersedes", id=new(), newer=newer, older=older)
    provenance(conn, loop_a, "a.json#/4")
    result = run(conn, "supersedes.acyclic")
    assert violating(result) == ids(loop_a, loop_b, itself)
    assert {r[0]: r[result.columns.index("locator")] for r in result.rows}[str(loop_a)] == (
        "a.json#/4"
    )


def site_class(
    conn: psycopg.Connection, host_key: str, index: int = 1, **host: uuid.UUID
) -> uuid.UUID:
    identifier = new()
    insert(
        conn,
        "tk.site_class",
        id=identifier,
        host_key=host_key,
        index=index,
        ratio_kind="composition_dependent",
        **host,
    )
    return identifier


def test_a_site_class_key_is_the_identifier_of_its_host(conn: psycopg.Connection) -> None:
    phase, material = new(), new()
    by_phase = site_class(conn, str(phase), phase=phase)
    by_material = site_class(conn, str(material), 2, material=material)
    wrong = site_class(conn, "a host", 3, phase=phase)
    other = site_class(conn, str(material), 4, phase=phase)
    provenance(conn, wrong, "a.json#/5")
    result = run(conn, "site_class.host_key_matches_host")
    assert violating(result) == ids(wrong, other)
    assert by_phase not in violating(result) and by_material not in violating(result)
    row = {r[0]: dict(zip(result.columns, r)) for r in result.rows}[str(wrong)]
    assert row["locator"] == "a.json#/5" and row["host"] == str(phase)


def test_equivalent_site_classes_have_one_host(conn: psycopg.Connection) -> None:
    host, other = new(), new()
    a = site_class(conn, str(host), 1, phase=host)
    b = site_class(conn, str(host), 2, phase=host)
    foreign = site_class(conn, str(other), 1, phase=other)
    good, bad = new(), new()
    # the stored orientation has the smaller identifier first
    first, second = sorted((a, b), key=str)
    insert(conn, "tk.site_equivalence", id=good, a=first, b=second)
    first, second = sorted((a, foreign), key=str)
    insert(conn, "tk.site_equivalence", id=bad, a=first, b=second)
    result = run(conn, "site_equivalence.same_host")
    assert violating(result) == ids(bad)
    row = dict(zip(result.columns, result.rows[0]))
    assert {row["host_a"], row["host_b"]} == {str(host), str(other)}


def test_an_association_site_key_is_the_identifier_of_its_carrier(
    conn: psycopg.Connection,
) -> None:
    entity, group = new(), new()
    scheme = new()

    def site(label: str, carrier_key: str, **carrier: uuid.UUID) -> uuid.UUID:
        identifier = new()
        insert(
            conn,
            "tk.association_site",
            id=identifier,
            scheme=scheme,
            carrier_key=carrier_key,
            label=label,
            multiplicity=1.0,
            **carrier,
        )
        return identifier

    good_entity = site("H", str(entity), on_entity=entity)
    good_group = site("e", str(group), on_group=group)
    wrong = site("x", "by name", on_entity=entity)
    swapped = site("y", str(group), on_entity=entity)
    result = run(conn, "association_site.carrier_key_matches_carrier")
    assert violating(result) == ids(wrong, swapped)
    assert good_entity not in violating(result) and good_group not in violating(result)


def test_a_group_count_names_a_group_of_the_assignments_scheme(
    conn: psycopg.Connection,
) -> None:
    scheme, other = new(), new()
    member, foreign = new(), new()
    for identifier, in_scheme, label in ((member, scheme, "m"), (foreign, other, "f")):
        insert(
            conn, "tk.group", id=identifier, scheme=in_scheme, label=label, role="group"
        )
    assignment = new()
    insert(
        conn,
        "tk.group_assignment",
        id=assignment,
        entity=new(),
        scheme=scheme,
        asserted_by=new(),
        origin="published",
    )
    good, bad = new(), new()
    insert(conn, "tk.group_count", id=good, assignment=assignment, group=member, value=1.0)
    insert(conn, "tk.group_count", id=bad, assignment=assignment, group=foreign, value=2.0)
    provenance(conn, assignment, "a.json#/6")
    result = run(conn, "group_count.group_in_assignment_scheme")
    assert violating(result) == ids(bad)
    row = dict(zip(result.columns, result.rows[0]))
    assert row["locator"] == "a.json#/6"
    assert row["assignment_scheme"] == str(scheme) and row["group_scheme"] == str(other)


def test_a_datums_column_belongs_to_the_dataset_of_its_point(conn: psycopg.Connection) -> None:
    dataset, other = new(), new()
    observable = lookup(conn, "SELECT id FROM tk.observable LIMIT 1")
    point, column, foreign = new(), new(), new()
    insert(conn, "ev.data_point", id=point, dataset=dataset, index=1)
    for identifier, owner, ordinal in ((column, dataset, 1), (foreign, other, 1)):
        insert(
            conn,
            "ev.dataset_column",
            id=identifier,
            dataset=owner,
            ordinal=ordinal,
            role="property",
            observable=observable,
        )
    good, bad = new(), new()
    for identifier, target in ((good, column), (bad, foreign)):
        insert(
            conn,
            "ev.datum",
            id=identifier,
            point=point,
            column=target,
            state="not_measured",
        )
    provenance(conn, dataset, "a.json#/7")
    result = run(conn, "datum.column_in_point_dataset")
    assert violating(result) == ids(bad)
    row = dict(zip(result.columns, result.rows[0]))
    assert row["locator"] == "a.json#/7" and row["column_dataset"] == str(other)


def evidence_column(
    conn: psycopg.Connection, dataset: uuid.UUID, ordinal: int, role: str = "property", **extra: object
) -> uuid.UUID:
    identifier = new()
    observable = lookup(conn, "SELECT id FROM tk.observable LIMIT 1")
    insert(
        conn,
        "ev.dataset_column",
        id=identifier,
        dataset=dataset,
        ordinal=ordinal,
        role=role,
        observable=observable,
        **extra,
    )
    return identifier


def test_a_datums_column_is_a_variable_or_a_property(conn: psycopg.Connection) -> None:
    dataset = new()
    point = new()
    insert(conn, "ev.data_point", id=point, dataset=dataset, index=1)
    columns = {
        role: evidence_column(conn, dataset, ordinal, role, **(
            {"constant": 1.0} if role == "constraint" else {}
        ))
        for ordinal, role in enumerate(("variable", "property", "constraint"), 1)
    }
    made = {}
    for role, column in columns.items():
        made[role] = new()
        insert(
            conn, "ev.datum", id=made[role], point=point, column=column, state="not_measured"
        )
    provenance(conn, dataset, "a.json#/8")
    result = run(conn, "datum.column_is_variable_or_property")
    assert violating(result) == ids(made["constraint"])
    row = dict(zip(result.columns, result.rows[0]))
    assert row["role"] == "constraint" and row["locator"] == "a.json#/8"


def test_a_column_presented_against_a_reference_state_states_its_kind(
    conn: psycopg.Connection,
) -> None:
    dataset = new()
    cases = {
        # presentation: (reference state kind, conforms)
        "direct": (None, True),
        "difference_between_temperatures": (None, True),
        "mean_between_temperatures": (None, True),
        "difference_from_reference": ("ideal_gas_same_density", True),
        "ratio_to_reference": (None, False),
        "relative_difference_from_reference": (None, False),
    }
    bad: list[uuid.UUID] = []
    for ordinal, (presentation, (kind, conforms)) in enumerate(cases.items(), 1):
        column = evidence_column(
            conn, dataset, ordinal, presentation=presentation, reference_state_kind=kind
        )
        if not conforms:
            bad.append(column)
    provenance(conn, dataset, "a.json#/9")
    result = run(conn, "dataset_column.reference_state_kind_when_relative")
    assert violating(result) == ids(*bad)
    assert {dict(zip(result.columns, r))["presentation"] for r in result.rows} == {
        "ratio_to_reference",
        "relative_difference_from_reference",
    }


def assessment(
    conn: psycopg.Connection, column: uuid.UUID, ordinal: int, kind: str, **extra: object
) -> uuid.UUID:
    identifier = new()
    insert(
        conn,
        "ev.uncertainty_assessment",
        id=identifier,
        column=column,
        ordinal=ordinal,
        kind=kind,
        **extra,
    )
    return identifier


def test_an_uncertainty_is_stated_for_a_point_of_the_dataset_of_its_assessment(
    conn: psycopg.Connection,
) -> None:
    dataset, other = new(), new()
    point = new()
    insert(conn, "ev.data_point", id=point, dataset=dataset, index=1)
    mine = assessment(conn, evidence_column(conn, dataset, 1), 1, "standard")
    foreign = assessment(conn, evidence_column(conn, other, 1), 1, "standard")
    good, bad = new(), new()
    for identifier, target in ((good, mine), (bad, foreign)):
        insert(
            conn,
            "ev.datum_uncertainty",
            id=identifier,
            point=point,
            assessment=target,
            minus=0.1,
            plus=0.1,
        )
    provenance(conn, dataset, "a.json#/10")
    result = run(conn, "datum_uncertainty.assessment_in_point_dataset")
    assert violating(result) == ids(bad)
    row = dict(zip(result.columns, result.rows[0]))
    assert row["locator"] == "a.json#/10" and row["column_dataset"] == str(other)


MAGNITUDES = {
    # kind: (minus, plus, relative minus, relative plus) that conforms, then some that do not
    "standard": ((0.1, 0.1, None, None), [(0.1, None, None, None), (None, None, 0.1, 0.1)]),
    "expanded": ((0.1, 0.3, None, None), [(None, None, None, None), (0.1, 0.1, 0.1, 0.1)]),
    "combined_standard": ((0.2, 0.2, None, None), [(0.2, None, None, None)]),
    "relative": ((None, None, 0.1, 0.2), [(0.1, 0.1, None, None), (None, None, 0.1, None)]),
    "relative_curve_deviation": ((None, None, 0.1, 0.1), [(None, None, None, None)]),
    "device_specification": ((0.5, 0.5, None, None), [(None, None, 0.5, 0.5)]),
    "not_stated": ((None, None, None, None), [(0.1, 0.1, None, None)]),
    "exact": ((None, None, None, None), [(None, None, 0.1, 0.1)]),
}


@pytest.mark.parametrize("relation", ["datum_uncertainty", "column_uncertainty"])
def test_the_pair_of_magnitudes_present_is_the_one_the_kind_calls_for(
    conn: psycopg.Connection, relation: str
) -> None:
    dataset = new()
    point = new()
    insert(conn, "ev.data_point", id=point, dataset=dataset, index=1)
    column = evidence_column(conn, dataset, 1, "constraint", constant=1.0)
    bad: list[uuid.UUID] = []
    ordinal = 0
    for kind, (good, wrong) in MAGNITUDES.items():
        for label, (minus, plus, relative_minus, relative_plus) in [
            ("good", good),
            *(("wrong", found) for found in wrong),
        ]:
            ordinal += 1
            made = assessment(conn, column, ordinal, kind)
            identifier = new()
            keys = {"point": point} if relation == "datum_uncertainty" else {}
            insert(
                conn,
                f"ev.{relation}",
                id=identifier,
                assessment=made,
                minus=minus,
                plus=plus,
                relative_minus=relative_minus,
                relative_plus=relative_plus,
                **keys,
            )
            if label == "wrong":
                bad.append(identifier)
    provenance(conn, dataset, "a.json#/11")
    result = run(conn, f"{relation}.magnitude_matches_kind")
    assert violating(result) == ids(*bad)
    assert {dict(zip(result.columns, r))["locator"] for r in result.rows} == {"a.json#/11"}


def test_an_uncertainty_of_a_constant_is_of_a_constraint_column(conn: psycopg.Connection) -> None:
    dataset = new()
    constraint = evidence_column(conn, dataset, 1, "constraint", constant=1.0)
    variable = evidence_column(conn, dataset, 2, "variable")
    good, bad = new(), new()
    for identifier, column in ((good, constraint), (bad, variable)):
        insert(
            conn,
            "ev.column_uncertainty",
            id=identifier,
            assessment=assessment(conn, column, 1, "standard"),
            minus=0.1,
            plus=0.1,
        )
    result = run(conn, "column_uncertainty.column_is_constraint")
    assert violating(result) == ids(bad)
    assert dict(zip(result.columns, result.rows[0]))["role"] == "variable"


def test_a_member_with_a_facet_is_checked_for_its_magnitudes_without_editing_the_check(
    tmp_path: Path,
) -> None:
    """A new `uncertainty_kind` member that carries the facet `relative` is held to a relative
    magnitude by the checks that read the facets, with no SQL change."""
    model, forms = extended_directories(tmp_path / "decl")
    module = model / "observables.toml"
    text = module.read_text()
    marker = 'members.interval = { doc = "A bound the true value is stated to lie within." }'
    assert marker in text
    module.write_text(
        text.replace(
            marker,
            'members.fractional_bound = { doc = "A bound as a fraction of the value.", '
            'facets = ["relative"] }\n' + marker,
            1,
        )
    )
    decl = load_declaration(model, forms).require()
    with TestDatabase() as database:
        build_database(database.url, decl)
        with psycopg.connect(database.url) as connection:
            connection.execute("SELECT 1")
            good, bad = new(), new()
            for identifier, magnitude, relative in ((good, None, 0.1), (bad, 1.0, None)):
                insert(
                    connection,
                    "tk.slot_uncertainty",
                    id=identifier,
                    parameter_set=new(),
                    slot=new(),
                    index_key="",
                    kind="fractional_bound",
                    magnitude=magnitude,
                    relative_magnitude=relative,
                )
            found = run_check(connection, CHECKS["slot_uncertainty.magnitude_matches_kind"])
            connection.rollback()
    assert found.error is None and violating(found) == ids(bad)


def test_a_clause_of_a_region_states_a_bound(conn: psycopg.Connection) -> None:
    region = new()
    observable = lookup(conn, "SELECT id FROM tk.observable LIMIT 1")
    lower, upper, both, neither = new(), new(), new(), new()
    for identifier, ordinal, low, high in (
        (lower, 1, 250.0, None),
        (upper, 2, None, 300.0),
        (both, 3, 250.0, 300.0),
        (neither, 4, None, None),
    ):
        insert(
            conn,
            "tk.region_clause",
            id=identifier,
            region=region,
            ordinal=ordinal,
            observable=observable,
            lower=low,
            upper=high,
        )
    provenance(conn, region, "a.json#/12")
    result = run(conn, "region_clause.has_a_bound")
    assert violating(result) == ids(neither)
    assert dict(zip(result.columns, result.rows[0]))["locator"] == "a.json#/12"


def test_a_coverage_row_says_stated_exactly_when_the_record_has_a_region_of_the_kind(
    conn: psycopg.Connection,
) -> None:
    names = (
        "stated_with_region",
        "stated_without_region",
        "not_stated_without_region",
        "not_stated_with_region",
        "region_without_row",
        "other_kind",
    )
    record = {name: new() for name in names}
    coverage = {name: new() for name in names}
    for name, kind, state in (
        ("stated_with_region", "fitted_range", "stated"),
        ("stated_without_region", "fitted_range", "stated"),
        ("not_stated_without_region", "fitted_range", "not_stated"),
        ("not_stated_with_region", "fitted_range", "not_stated"),
        ("other_kind", "fitted_range", "stated"),
    ):
        insert(
            conn,
            "tk.validity_coverage",
            id=coverage[name],
            record=record[name],
            kind=kind,
            value=state,
        )
    # the regions of one kind do not make another kind `stated`: no `recommended_range` is stated
    insert(
        conn,
        "tk.validity_coverage",
        id=new(),
        record=record["other_kind"],
        kind="recommended_range",
        value="not_stated",
    )
    for name in ("stated_with_region", "not_stated_with_region", "region_without_row", "other_kind"):
        insert(
            conn,
            "tk.validity_region",
            id=new(),
            record=record[name],
            kind="fitted_range",
            ordinal=1,
        )
    provenance(conn, record["stated_without_region"], "a.json#/13")
    result = run(conn, "validity_coverage.state_matches_regions")
    assert violating(result) == ids(
        coverage["stated_without_region"],
        coverage["not_stated_with_region"],
        record["region_without_row"],
    )
    by_id = {r[0]: dict(zip(result.columns, r)) for r in result.rows}
    assert by_id[str(coverage["stated_without_region"])]["locator"] == "a.json#/13"
    assert by_id[str(record["region_without_row"])]["state"] == "", "no row: no state"


def test_a_derivation_of_kind_estimation_states_its_method(conn: psycopg.Connection) -> None:
    made = {}
    for key, kind, method in (
        ("named", "estimation", "Joback"),
        ("missing", "estimation", None),
        ("blank", "estimation", "  "),
        ("fit_without", "computation", None),
    ):
        made[key] = new()
        insert(conn, "prov.derivation", id=made[key], key=key, kind=kind, method=method)
    provenance(conn, made["missing"], "a.json#/14")
    result = run(conn, "derivation.estimation_names_method")
    assert violating(result) == ids(made["missing"], made["blank"])
    assert {dict(zip(result.columns, r))["locator"] for r in result.rows} == {"a.json#/14", ""}


def test_a_snapshot_selects_no_set_made_by_a_failed_fit(conn: psycopg.Connection) -> None:
    snapshot = new()
    selected = {name: new() for name in ("failed", "converged", "not_stated", "estimated", "bare")}
    for name, outcome in (("failed", "failed"), ("converged", "converged"), ("not_stated", "not_stated")):
        derivation = new()
        insert(conn, "prov.derivation", id=derivation, key=name, kind="fit")
        insert(conn, "prov.fit", id=derivation, outcome=outcome)
        insert(conn, "prov.derivation_output", id=new(), derivation=derivation, record=selected[name])
    estimation = new()
    insert(conn, "prov.derivation", id=estimation, key="e", kind="estimation", method="m")
    insert(conn, "prov.derivation_output", id=new(), derivation=estimation, record=selected["estimated"])
    chosen = {}
    for name, record in selected.items():
        chosen[name] = new()
        insert(conn, "tk.snapshot_selection", id=chosen[name], snapshot=snapshot, parameter_set=record)
    provenance(conn, selected["failed"], "a.json#/15")
    result = run(conn, "snapshot_selection.no_failed_fit")
    assert violating(result) == ids(chosen["failed"])
    assert dict(zip(result.columns, result.rows[0]))["locator"] == "a.json#/15"


def test_a_licence_is_named_only_for_a_licence_grant(conn: psycopg.Connection) -> None:
    licence = new()
    insert(conn, "prov.licence", id=licence, key="MIT", title="MIT")
    made = {}
    for ordinal, (name, basis, named) in enumerate(
        (
            ("grant_with_licence", "licence_grant", licence),
            ("grant_without", "licence_grant", None),
            ("terms_without", "terms_of_use", None),
            ("terms_with_licence", "terms_of_use", licence),
            ("public_domain_with_licence", "public_domain", licence),
        ),
        1,
    ):
        made[name] = new()
        insert(
            conn,
            "prov.rights_determination",
            id=made[name],
            carrier=new(),
            scope="data",
            ordinal=ordinal,
            basis=basis,
            licence=named,
            statement="s",
            store="yes",
            redistribute="yes",
            commercial="yes",
            attribution_required=False,
            share_alike=False,
            observed="o",
        )
    result = run(conn, "rights_determination.licence_only_for_licence_grant")
    assert violating(result) == ids(made["terms_with_licence"], made["public_domain_with_licence"])


def test_a_row_that_was_not_loaded_is_held_or_unmapped(conn: psycopg.Connection) -> None:
    made = {}
    for state in ("held", "unmapped", "mapped", "deferred"):
        made[state] = new()
        insert(
            conn,
            "qual.held_row",
            id=made[state],
            manifest_id="m",
            source_table="t",
            locator=f"t#{state}",
            state=state,
            reason="unmapped_by_mapping",
            detail="d",
        )
    result = run(conn, "held_row.state_is_held_or_unmapped")
    assert violating(result) == ids(made["mapped"], made["deferred"])


def test_a_reaction_of_a_system_conserves_the_quantities_the_system_declares(
    conn: psycopg.Connection,
) -> None:
    hydrogen, oxygen = (
        lookup(conn, "SELECT id FROM tk.conserved_quantity WHERE key = %s", symbol)
        for symbol in "HO"
    )
    gas = lookup(conn, "SELECT id FROM tk.aggregation WHERE name = 'gas'")
    form: dict[str, uuid.UUID] = {}
    for name, composition in {
        "H2": {hydrogen: 2.0},
        "O2": {oxygen: 2.0},
        "H2O": {hydrogen: 2.0, oxygen: 1.0},
    }.items():
        species = new()
        form[name] = new()
        insert(conn, "tk.species_form", id=form[name], species=species, aggregation=gas)
        for quantity, value in composition.items():
            insert(conn, "tk.composition", id=new(), entity=species, quantity=quantity, value=value)

    def reaction(key: str, coefficients: dict[str, float]) -> uuid.UUID:
        identifier = new()
        insert(conn, "tk.reaction", id=identifier, canonical_key=key, extent="as_written")
        for name, coefficient in coefficients.items():
            insert(
                conn,
                "tk.reaction_participant",
                id=new(),
                reaction=identifier,
                form=form[name],
                coefficient=coefficient,
            )
        return identifier

    balanced = reaction("balanced", {"H2": -2.0, "O2": -1.0, "H2O": 2.0})
    unbalanced = reaction("unbalanced", {"H2": -1.0, "O2": -1.0, "H2O": 1.0})
    system, other = new(), new()
    # the system declares only hydrogen: the oxygen imbalance of `unbalanced` is not its concern
    insert(conn, "tk.system_conserves", id=new(), system=system, quantity=hydrogen)
    conserving = {}
    for name, owner, target in (
        ("balanced", system, balanced),
        ("unbalanced_in_hydrogen", system, unbalanced),
    ):
        conserving[name] = new()
        insert(conn, "tk.system_reaction", id=conserving[name], system=owner, reaction=target)
    # declared oxygen makes `unbalanced` a violation of the system that declares it
    insert(conn, "tk.system_conserves", id=new(), system=other, quantity=oxygen)
    conserving["unbalanced_in_oxygen"] = new()
    insert(conn, "tk.system_reaction", id=conserving["unbalanced_in_oxygen"], system=other, reaction=unbalanced)
    provenance(conn, unbalanced, "a.json#/16")
    result = run(conn, "system_reaction.conserves_system_quantities")
    assert violating(result) == ids(conserving["unbalanced_in_oxygen"])
    row = dict(zip(result.columns, result.rows[0]))
    assert row["quantity"] == "O" and float(row["net"]) == -1.0 and row["locator"] == "a.json#/16"


def test_the_target_of_a_source_entity_is_of_the_refinement_its_class_names(
    conn: psycopg.Connection,
) -> None:
    species, mixture, unclassified = new(), new(), new()
    for table, identifier, extra in (
        ("tk.species", species, {"charge": 0}),
        ("tk.defined_mixture", mixture, {"definition": "by_definition", "mole_basis": True}),
        ("tk.unclassified_entity", unclassified, {}),
    ):
        insert(
            conn,
            "tk.material_entity",
            id=identifier,
            canonical_key=str(identifier),
            label="x",
            provisional=False,
        )
        insert(conn, table, id=identifier, **extra)
    cases = {
        # key: (class, target, conforms)
        "species_to_species": ("species", species, True),
        "mixture_to_mixture": ("defined_mixture", mixture, True),
        "undetermined_to_unclassified": ("undetermined", unclassified, True),
        "species_to_mixture": ("species", mixture, False),
        "mixture_to_species": ("defined_mixture", species, False),
        "undetermined_to_species": ("undetermined", species, False),
        "species_to_unclassified": ("species", unclassified, False),
        "species_to_nothing": ("species", new(), False),
    }
    bad: list[uuid.UUID] = []
    for key, (entity_class, target, conforms) in cases.items():
        entity = new()
        insert(
            conn,
            "tk.source_entity",
            id=entity,
            carrier=new(),
            scope="s",
            local_key=key,
            entity_class=entity_class,
            status="unique",
            rule="structural",
            target=target,
        )
        if not conforms:
            bad.append(entity)
    result = run(conn, "source_entity.class_matches_target")
    assert violating(result) == ids(*bad)


def test_the_database_enforces_that_a_conversion_is_named_exactly_for_the_level_that_needs_one(
    conn: psycopg.Connection,
) -> None:
    conversion = new()
    kinds = {"exact": None, "equal_under_conversion": conversion}
    for level, named in kinds.items():
        first, second = sorted((new(), new()), key=str)
        insert(conn, "prov.equivalence_assessment", id=new(), a=first, b=second, level=level, conversion=named)
    for level, named in (("exact", conversion), ("equal_under_conversion", None)):
        first, second = sorted((new(), new()), key=str)
        with pytest.raises(psycopg.errors.CheckViolation, match="conversion_iff_under_conversion"):
            with conn.transaction():
                insert(
                    conn,
                    "prov.equivalence_assessment",
                    id=new(),
                    a=first,
                    b=second,
                    level=level,
                    conversion=named,
                )


def test_a_validity_region_has_a_clause(conn: psycopg.Connection) -> None:
    observable = lookup(conn, "SELECT id FROM tk.observable LIMIT 1")
    with_clause, empty = new(), new()
    record = new()
    for ordinal, identifier in enumerate((with_clause, empty), 1):
        insert(
            conn, "tk.validity_region", id=identifier, record=record, kind="fitted_range", ordinal=ordinal
        )
    insert(
        conn,
        "tk.region_clause",
        id=new(),
        region=with_clause,
        ordinal=1,
        observable=observable,
        lower=1.0,
    )
    provenance(conn, empty, "a.json#/17")
    result = run(conn, "validity_region.has_a_clause")
    assert violating(result) == ids(empty)
    assert dict(zip(result.columns, result.rows[0]))["locator"] == "a.json#/17"


def test_an_uncertainty_qualifies_a_value_that_exists(conn: psycopg.Connection) -> None:
    dataset = new()
    point = new()
    insert(conn, "ev.data_point", id=point, dataset=dataset, index=1)
    cases = {
        # name: (datum state and value or None for no datum, conforms)
        "known": (("known", 1.0), True),
        "censored_below": (("censored_below", 1.0), True),
        "censored_above": (("censored_above", 1.0), True),
        "not_measured": (("not_measured", None), False),
        "no_datum": (None, False),
    }
    bad: list[uuid.UUID] = []
    for ordinal, (name, (datum, conforms)) in enumerate(cases.items(), 1):
        column = evidence_column(conn, dataset, ordinal)
        if datum is not None:
            insert(
                conn,
                "ev.datum",
                id=new(),
                point=point,
                column=column,
                state=datum[0],
                value=datum[1],
            )
        identifier = new()
        insert(
            conn,
            "ev.datum_uncertainty",
            id=identifier,
            point=point,
            assessment=assessment(conn, column, 1, "standard"),
            minus=0.1,
            plus=0.1,
        )
        if not conforms:
            bad.append(identifier)
    provenance(conn, dataset, "a.json#/18")
    result = run(conn, "datum_uncertainty.datum_exists")
    assert violating(result) == ids(*bad)
    assert {dict(zip(result.columns, r))["locator"] for r in result.rows} == {"a.json#/18"}


FROM_ONE = (
    ("tk.parameter_set", "occurrence", "occurrence_from_one"),
    ("ev.data_point", "index", "index_from_one"),
    ("ev.dataset_column", "ordinal", "ordinal_from_one"),
    ("ev.dataset_component", "ordinal", "ordinal_from_one"),
    ("ev.dataset_phase", "ordinal", "ordinal_from_one"),
    ("ev.uncertainty_assessment", "ordinal", "ordinal_from_one"),
    ("tk.tabulated_axis", "ordinal", "ordinal_from_one"),
    ("tk.assembly_choice", "ordinal", "ordinal_from_one"),
    ("tk.group", "position", "position_from_one"),
    ("tk.validity_region", "ordinal", "ordinal_from_one"),
    ("tk.region_clause", "ordinal", "ordinal_from_one"),
    ("tk.constituent_array_member", "position", "position_from_one"),
    ("prov.fit_free_parameter", "ordinal", "ordinal_from_one"),
    ("prov.fit_correlation", "a", "a_from_one"),
    ("prov.fit_correlation", "b", "b_from_one"),
)


@pytest.mark.parametrize(("table", "column", "requirement"), FROM_ONE)
def test_a_position_documented_as_starting_at_one_is_checked_positive(
    conn: psycopg.Connection, table: str, column: str, requirement: str
) -> None:
    schema, name = table.split(".")
    owner = real_declaration().kinds.get(name) or real_declaration().relations[name]
    (declared,) = [r for r in owner.requires if r.name == requirement]
    assert (declared.enforced, declared.rule, declared.attributes) == ("ddl", "positive", (column,))
    found = conn.execute(
        "SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = %s",
        (f"{name}__ck__{requirement}",),
    ).fetchone()
    assert found is not None and re.search(rf'"?{column}"? > 0', found[0]), found


def test_a_position_of_zero_is_refused_by_the_database(conn: psycopg.Connection) -> None:
    with pytest.raises(psycopg.errors.CheckViolation, match="index_from_one"):
        with conn.transaction():
            insert(conn, "ev.data_point", id=new(), dataset=new(), index=0)
    with pytest.raises(psycopg.errors.CheckViolation, match="a_from_one"):
        with conn.transaction():
            insert(
                conn,
                "prov.fit_correlation",
                id=new(),
                fit=new(),
                a=0,
                b=1,
                coefficient=0.5,
            )


def test_the_subject_key_of_a_set_encodes_its_subjects(conn: psycopg.Connection) -> None:
    slot_group = lookup(conn, "SELECT id FROM meta.slot_group WHERE qualified_name = %s", SATURATION)
    parameterization, good, wrong_subject, unsorted_key = new(), new(), new(), new()
    subjects = {good: new(), wrong_subject: new(), unsorted_key: new()}
    keys = {
        good: f'["{subjects[good]}"]',
        wrong_subject: f'["{new()}"]',
        unsorted_key: str(subjects[unsorted_key]),
    }
    for identifier, subject in subjects.items():
        insert(
            conn,
            "tk.parameter_set",
            id=identifier,
            parameterization=parameterization,
            slot_group=slot_group,
            subject_key=keys[identifier],
        )
        insert(conn, f"param.{SATURATION_TABLE}", id=identifier, i=subject, T_r=300.0, p_r=1e5)
    provenance(conn, wrong_subject, "a.json#/8")
    result = run(conn, "subject_key_matches_subjects")
    assert violating(result) == ids(wrong_subject, unsorted_key)
    row = {r[0]: dict(zip(result.columns, r)) for r in result.rows}[str(wrong_subject)]
    assert row["locator"] == "a.json#/8" and row["expected"] == f'["{subjects[wrong_subject]}"]'


def test_a_record_in_a_role_that_requires_a_derivation_has_one(conn: psycopg.Connection) -> None:
    # the roles are those `origin_role` marks with the facet `requires_derivation`
    produced, fitted, estimated, derived, published, computed = (new() for _ in range(6))
    derivation = new()
    insert(conn, "prov.derivation", id=derivation, key="fit", kind="fit")
    insert(conn, "prov.derivation_output", id=new(), derivation=derivation, record=produced)
    provenance(conn, produced, "a.json#/0", role="fitted")
    provenance(conn, derivation, "a.json#/5", role="fitted")  # a derivation is not produced by one
    for record, role in (
        (fitted, "fitted"),
        (estimated, "estimated"),
        (derived, "derived"),
        (published, "published"),
        (computed, "computed"),
    ):
        provenance(conn, record, f"a.json#/{role}", role=role)
    result = run(conn, "producing_derivation")
    assert violating(result) == ids(fitted, estimated, derived)
    assert {r[0]: r[result.columns.index("role")] for r in result.rows} == {
        str(fitted): "fitted",
        str(estimated): "estimated",
        str(derived): "derived",
    }


def test_the_producing_derivation_of_a_fitted_record_is_a_fit(conn: psycopg.Connection) -> None:
    by_fit, by_estimation, unproduced, published_by_estimation = (new() for _ in range(4))
    fit, estimation = new(), new()
    insert(conn, "prov.derivation", id=fit, key="f", kind="fit")
    insert(conn, "prov.fit", id=fit, outcome="converged")
    insert(conn, "prov.derivation", id=estimation, key="e", kind="estimation")
    for record, producer in (
        (by_fit, fit),
        (by_estimation, estimation),
        (published_by_estimation, estimation),
    ):
        insert(conn, "prov.derivation_output", id=new(), derivation=producer, record=record)
    provenance(conn, by_fit, "a.json#/1", role="fitted")
    provenance(conn, by_estimation, "a.json#/2", role="fitted")
    provenance(conn, unproduced, "a.json#/3", role="fitted")  # the finding of `producing_derivation`
    provenance(conn, published_by_estimation, "a.json#/4", role="published")
    result = run(conn, "producing_derivation_is_fit")
    assert violating(result) == ids(by_estimation)
    row = dict(zip(result.columns, result.rows[0]))
    assert row["locator"] == "a.json#/2" and row["derivation"] == str(estimation)


def test_a_member_with_a_facet_is_covered_by_the_checks_without_editing_them(
    tmp_path: Path,
) -> None:
    """Declaring the facet on a new `origin_role` member is all it takes: the check reads
    the facets from `meta`."""
    model, forms = extended_directories(tmp_path / "decl")
    provenance_module = model / "provenance.toml"
    text = provenance_module.read_text()
    marker = 'members.computed = { doc = "The output of a calculation'
    assert marker in text
    provenance_module.write_text(
        text.replace(
            marker,
            'members.simulated = { doc = "A simulated value.", '
            'facets = ["requires_derivation", "requires_fit"] }\n' + marker,
            1,
        )
    )
    decl = load_declaration(model, forms).require()
    check = CHECKS["producing_derivation"]
    needing_fit = CHECKS["producing_derivation_is_fit"]
    with TestDatabase() as database:
        build_database(database.url, decl)
        with psycopg.connect(database.url) as connection:
            connection.execute("SELECT 1")
            bare, by_estimation = new(), new()
            estimation = new()
            insert(connection, "prov.derivation", id=estimation, key="e", kind="estimation")
            insert(
                connection, "prov.derivation_output", id=new(), derivation=estimation, record=by_estimation
            )
            provenance(connection, bare, "a.json#/0", role="simulated")
            provenance(connection, by_estimation, "a.json#/1", role="simulated")
            first = run_check(connection, check)
            second = run_check(connection, needing_fit)
            connection.rollback()
    assert first.error is None and second.error is None
    assert violating(first) == ids(bare) and violating(second) == ids(by_estimation)


# -- the framework ---------------------------------------------------------------------------


def write_check(directory: Path, name: str, text: str) -> Path:
    directory.mkdir(parents=True, exist_ok=True)
    path = directory / f"{name}.sql"
    path.write_text(text)
    return path


def test_check_files_declare_their_invariant_and_description(tmp_path: Path) -> None:
    good = write_check(
        tmp_path,
        "good",
        "-- invariant: species.charge_matches_composition\n-- What.\nSELECT 1 AS id;\n",
    )
    parsed = parse(good)
    assert isinstance(parsed, Check)
    assert (parsed.kind, parsed.target, parsed.description, parsed.query) == (
        "invariant",
        "species.charge_matches_composition",
        "What.",
        "SELECT 1 AS id",
    )
    structural = parse(
        write_check(
            tmp_path, "s", "-- structural: some_check\n-- What.\n-- query\nSELECT 1 AS id\n"
        )
    )
    assert (
        isinstance(structural, Check) and structural.kind == "structural" and structural.setup == ""
    )
    setup = parse(
        write_check(
            tmp_path,
            "setup",
            "-- structural: x\n-- What.\nCREATE TEMP TABLE t (id int);\n-- query\nSELECT id FROM t\n",
        )
    )
    assert (
        isinstance(setup, Check)
        and setup.setup.startswith("CREATE TEMP TABLE")
        and setup.query == "SELECT id FROM t"
    )
    for name, text, expected in (
        ("headerless", "SELECT 1 AS id\n", "the first line must be"),
        ("unknown_kind", "-- rule: x\n-- What.\nSELECT 1 AS id\n", "the first line must be"),
        ("no_description", "-- invariant: a.b\nSELECT 1 AS id\n", "the second line must be"),
        ("no_query", "-- invariant: a.b\n-- What.\n-- nothing follows\n", "has no query"),
    ):
        problem = parse(write_check(tmp_path, name, text))
        assert isinstance(problem, str) and expected in problem and name in problem


def test_two_files_enforcing_one_invariant_are_a_problem(tmp_path: Path) -> None:
    text = "-- invariant: species.charge_matches_composition\n-- What.\nSELECT 1 AS id\n"
    write_check(tmp_path, "first", text)
    write_check(tmp_path, "second", text)
    checks, problems = load_checks(tmp_path)
    assert len(checks) == 2
    assert problems == [
        "second.sql and first.sql both enforce the invariant `species.charge_matches_composition`"
    ]


def test_a_declared_invariant_without_a_check_file_fails(tmp_path: Path) -> None:
    for check in REAL_CHECKS:
        if check.target != "species.charge_matches_composition":
            shutil.copy(check.path, tmp_path / check.path.name)
    checks, problems = load_checks(tmp_path)
    assert problems == []
    assert agreement_problems(real_declaration(), checks) == [
        "the declared verify invariant `species.charge_matches_composition` has no check file in sql/verify/"
    ]


def test_a_check_file_naming_an_undeclared_invariant_fails(tmp_path: Path) -> None:
    for check in REAL_CHECKS:
        shutil.copy(check.path, tmp_path / check.path.name)
    write_check(
        tmp_path, "nothing", "-- invariant: species.no_such_requirement\n-- What.\nSELECT 1 AS id\n"
    )
    # a requirement the declaration enforces elsewhere than in verify is not a verify invariant
    write_check(
        tmp_path, "ddl", "-- invariant: element.atomic_number_positive\n-- What.\nSELECT 1 AS id\n"
    )
    checks, problems = load_checks(tmp_path)
    assert problems == []
    found = agreement_problems(real_declaration(), checks)
    assert len(found) == 2
    assert "ddl.sql names the invariant `element.atomic_number_positive`" in found[0]
    assert "nothing.sql names the invariant `species.no_such_requirement`" in found[1]


def test_a_check_that_cannot_run_is_a_result_and_the_others_still_run(
    conn: psycopg.Connection, tmp_path: Path
) -> None:
    write_check(
        tmp_path, "a_broken", "-- structural: broken\n-- What.\nSELECT id FROM no.such_table\n"
    )
    write_check(tmp_path, "b_no_id", "-- structural: no_id\n-- What.\nSELECT 1 AS other\n")
    write_check(tmp_path, "c_fine", "-- structural: fine\n-- What.\nSELECT 1 AS id WHERE false\n")
    checks, _ = load_checks(tmp_path)
    broken, no_id, fine = run_checks(conn, checks)
    assert broken.error is not None and "no.such_table" in broken.error and not broken.passed
    assert no_id.error == "the query returns no `id` column"
    assert fine.passed and fine.violations == 0 and fine.rows == ()
    assert conn.execute("SELECT 1").fetchone() == (1,)  # the connection is still usable


def test_a_check_runs_in_a_transaction_that_is_rolled_back(
    conn: psycopg.Connection, tmp_path: Path
) -> None:
    write_check(
        tmp_path,
        "writes",
        "-- structural: writes\n-- What.\n"
        "CREATE FUNCTION pg_temp.helper() RETURNS int LANGUAGE sql AS 'SELECT 7';\n"
        "INSERT INTO prov.record (id, kind) VALUES ('00000000-0000-0000-0000-000000000001', 'x');\n"
        "-- query\nSELECT '00000000-0000-0000-0000-000000000002'::uuid AS id WHERE pg_temp.helper() = 7\n",
    )
    (check,) = load_checks(tmp_path)[0]
    seen = run_check(conn, check)  # the helper and the insert are visible to the check itself
    assert seen.error is None and seen.violations == 1
    assert conn.execute("SELECT count(*) FROM prov.record WHERE kind = 'x'").fetchone() == (0,)
    assert conn.execute("SELECT count(*) FROM pg_proc WHERE proname = 'helper'").fetchone() == (0,)


def test_violations_are_counted_in_the_database_and_only_the_first_are_kept(
    conn: psycopg.Connection, tmp_path: Path
) -> None:
    write_check(
        tmp_path,
        "many",
        "-- structural: many\n-- What.\nSELECT g::text::uuid AS id FROM (SELECT md5(g::text) AS g FROM generate_series(1, 1000) g) s\n",
    )
    (check,) = load_checks(tmp_path)[0]
    result = run_check(conn, check, shown=4)
    assert result.violations == 1000 and len(result.rows) == 4 and result.ids == sorted(result.ids)


def test_the_table_lists_check_kind_violations_and_the_first_offending_ids() -> None:
    check = CHECKS["species.charge_matches_composition"]
    other = CHECKS["record_has_origin"]
    report = Report(
        "db",
        "abc",
        (
            CheckResult(check, 5, ("id", "locator"), tuple((f"id-{n}", "") for n in range(5))),
            CheckResult(other, 0),
            CheckResult(CHECKS["dependency.acyclic"], 0, error="boom\nsecond line"),
        ),
    )
    lines = table_lines(report)
    assert lines[0].split() == ["check", "kind", "violations", "first", "offending", "ids"]
    assert lines[1].split(None, 3) == [
        "species.charge_matches_composition",
        "invariant",
        "5",
        "id-0, id-1, id-2, ...",
    ]
    assert lines[2].split() == ["record_has_origin", "structural", "0"]
    assert lines[3].split(None, 3)[2:] == ["error", "boom"]
    assert not report.passed


# -- the command -----------------------------------------------------------------------------


@pytest.fixture
def tree(tmp_path: Path) -> Path:
    """The real declaration and checks, copied, so a test may change them."""
    root = tmp_path / "tree"
    shutil.copytree(config.TREE_DIR / "model", root / "model")
    shutil.copytree(config.TREE_DIR / "forms", root / "forms")
    shutil.copytree(config.TREE_DIR / VERIFY_DIR, root / VERIFY_DIR)
    return root


@pytest.fixture
def cli(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> Iterator[TestDatabase]:
    monkeypatch.setenv(config.STORE_ENV, str(tmp_path / "store"))
    with TestDatabase() as database:
        monkeypatch.setenv(config.DATABASE_URL_ENV, database.url)
        yield database


def test_cli_passes_on_a_clean_database_and_writes_the_report(
    cli: TestDatabase, tree: Path, tmp_path: Path
) -> None:
    build_database(cli.url, real_declaration(), tree=tree)
    result = runner.invoke(app, ["verify", "--tree", str(tree)])
    assert result.exit_code == 0, result.output
    assert "species.charge_matches_composition" in result.output
    assert f"{len(REAL_CHECKS)} checks, 0 failing, 0 problem(s)" in result.output
    report = json.loads((tmp_path / "store" / "verify-report.json").read_text())
    assert report["schema"] == 1 and report["passed"] is True and report["problems"] == []
    assert report["database"] == cli.name and len(report["fingerprint"]) == 64
    assert [c["target"] for c in report["checks"]] == [c.target for c in REAL_CHECKS]
    for entry in report["checks"]:
        assert set(entry) == {
            "name",
            "kind",
            "target",
            "description",
            "violations",
            "error",
            "columns",
            "first_rows",
        }
        assert entry["violations"] == 0 and entry["error"] is None and entry["first_rows"] == []
        assert "id" in entry["columns"]
    elsewhere = tmp_path / "elsewhere" / "report.json"
    again = runner.invoke(app, ["verify", "--tree", str(tree), "--report", str(elsewhere)])
    assert again.exit_code == 0 and json.loads(elsewhere.read_text())["passed"] is True


def test_cli_fails_on_violations_and_reports_the_rows(
    cli: TestDatabase, tree: Path, tmp_path: Path
) -> None:
    build_database(cli.url, real_declaration(), tree=tree)
    orphan = new()
    with psycopg.connect(cli.url) as conn:  # a record nothing is the origin of
        insert(conn, "prov.record", id=orphan, kind="species")
    result = runner.invoke(app, ["verify", "--tree", str(tree)])
    assert result.exit_code == 1
    row = next(line for line in result.output.splitlines() if line.startswith("record_has_origin"))
    assert row.split() == ["record_has_origin", "structural", "1", str(orphan)]
    assert "1 failing" in result.output
    report = json.loads((tmp_path / "store" / "verify-report.json").read_text())
    failing = [c for c in report["checks"] if c["violations"]]
    assert report["passed"] is False and len(failing) == 1
    columns, first = failing[0]["columns"], failing[0]["first_rows"][0]
    assert (
        dict(zip(columns, first))["id"] == str(orphan)
        and dict(zip(columns, first))["kind"] == "species"
    )


def test_cli_fails_when_a_declared_invariant_has_no_check_or_a_check_names_none(
    cli: TestDatabase, tree: Path, tmp_path: Path
) -> None:
    build_database(cli.url, real_declaration(), tree=tree)
    (tree / VERIFY_DIR / "species.charge_matches_composition.sql").unlink()
    write_check(
        tree / VERIFY_DIR,
        "orphan",
        "-- invariant: species.no_such_requirement\n-- What.\nSELECT 1 AS id WHERE false\n",
    )
    result = runner.invoke(app, ["verify", "--tree", str(tree)])
    assert result.exit_code == 1
    assert "species.charge_matches_composition` has no check file" in result.output
    assert "orphan.sql names the invariant `species.no_such_requirement`" in result.output
    report = json.loads((tmp_path / "store" / "verify-report.json").read_text())
    assert report["passed"] is False and len(report["problems"]) == 2
    # every other check still ran and passed
    assert all(c["violations"] == 0 and c["error"] is None for c in report["checks"])


def test_cli_refuses_a_database_built_from_another_declaration(
    cli: TestDatabase, tree: Path
) -> None:
    unbuilt = runner.invoke(app, ["verify", "--tree", str(tree)])
    assert unbuilt.exit_code == 1 and "no schema fingerprint" in unbuilt.output
    build_database(cli.url, real_declaration(), tree=tree)
    (tree / "model" / "extra.toml").write_text(
        'module = "extra"\nschema = "tk"\ndoc = "d"\n\n[kinds.extra_kind]\ndoc = "d"\n'
        'identity = ["a"]\nprovenance = "none"\n\n[kinds.extra_kind.attributes]\na = { type = "Text", doc = "d" }\n'
    )
    changed = runner.invoke(app, ["verify", "--tree", str(tree)])
    assert changed.exit_code == 1 and "rebuild it with `tk build`" in changed.output
