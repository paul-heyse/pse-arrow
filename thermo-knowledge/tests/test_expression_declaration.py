# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""What the declaration gained for expressions: contract roles and sets, indexed arguments,
subject bindings, locals and outputs, their reified `meta` rows and the fingerprint."""

from __future__ import annotations

import hashlib
import shutil
from pathlib import Path

import psycopg
import pytest
from psycopg import errors

from declaration_support import NO_PHYSICAL, full_declaration
from expression_support import VALID, load, scenario
from thermo_knowledge.build import build_database
from thermo_knowledge.declaration import load_declaration
from thermo_knowledge.expression.canonical import content_hash, residual_hash, serialise
from thermo_knowledge.expression.parser import parse
from thermo_knowledge.generate import declaration_fingerprint
from thermo_knowledge.generate.meta_rows import meta_rows
from thermo_knowledge.testing import TestDatabase


def test_contracts_declare_roles_sets_and_indexed_arguments() -> None:
    contract = scenario("cubic").contracts["eos_pressure"]
    assert [(s.name, s.type.element) for s in contract.sets] == [("components", "material_entity")]
    assert contract.roles == ()
    x = next(a for a in contract.arguments if a.name == "x")
    assert (x.over, x.basis) == (("components",), "mole_fraction")
    T = next(a for a in contract.arguments if a.name == "T")
    assert (T.over, T.basis) == ((), None)
    alpha = scenario("cubic").contracts["alpha_function"]
    assert [(r.name, r.type.element) for r in alpha.roles] == [("i", "species")]


def test_a_subject_role_binds_to_the_contract_role_or_set_of_the_same_name_by_default() -> None:
    antoine = scenario("antoine").forms["antoine"].slot_groups[0]
    assert [(b.role, b.kind, b.target) for b in antoine.bindings] == [("i", "role", "i")]
    kij = full_declaration().forms["kij"].slot_groups[0]
    assert [(b.role, b.kind, b.target) for b in kij.bindings] == [
        ("i", "role", "i"),
        ("j", "role", "j"),
    ]


def test_bind_names_the_role_or_set_when_the_names_differ() -> None:
    groups = {g.name: g for g in scenario("cubic").forms["cubic_srk"].slot_groups}
    assert [(b.role, b.kind, b.target) for b in groups["pair"].bindings] == [
        ("i", "set", "components"),
        ("j", "set", "components"),
    ]
    assert groups["core"].bindings == ()


def test_a_catalogued_form_may_leave_a_subject_role_unbound() -> None:
    (binding,) = full_declaration().forms["antoine"].slot_groups[0].bindings
    assert (binding.role, binding.kind, binding.target) == ("i", None, None)


def test_the_real_model_still_loads_and_generates() -> None:
    assert load_declaration().diagnostics == ()


# -- meta rows ---------------------------------------------------------------------------------


def rows_of(decl, table: str) -> list[dict[str, object]]:  # noqa: ANN001
    from thermo_knowledge.generate.meta_tables import META_TABLES

    columns = [c.name for c in META_TABLES[table].columns]
    return [dict(zip(columns, row, strict=True)) for row in meta_rows(decl)[table]]


def test_contract_roles_sets_and_argument_indices_are_reified() -> None:
    decl = scenario("cubic")
    sets = rows_of(decl, "contract_set")
    assert [(r["contract"], r["name"], r["position"], r["kind"]) for r in sets] == [
        ("eos_pressure", "components", 1, "material_entity")
    ]
    assert [(r["contract"], r["name"], r["kind"]) for r in rows_of(decl, "contract_role")] == [
        ("alpha_function", "i", "species")
    ]
    assert [
        (r["contract"], r["argument"], r["position"], r["set_name"])
        for r in rows_of(decl, "contract_argument_set")
    ] == [("eos_pressure", "x", 1, "components")]
    arguments = {(r["contract"], r["name"]): r["basis"] for r in rows_of(decl, "contract_argument")}
    # a basis is a reference to the declared entity, not its name
    (mole_fraction,) = [
        e.id for e in decl.entities if e.kind == "composition_basis" and e.name == "mole_fraction"
    ]
    assert arguments[("eos_pressure", "x")] == mole_fraction
    assert arguments[("eos_pressure", "T")] is None


def test_subject_bindings_are_reified() -> None:
    rows = rows_of(scenario("cubic"), "slot_group_subject")
    by_key = {(r["slot_group"], r["name"]): (r["binds_to_type"], r["binds_to"]) for r in rows}
    assert by_key[("cubic_srk.pair", "j")] == ("set", "components")
    assert by_key[("cubic_srk.pure", "i")] == ("set", "components")
    assert by_key[("alpha_fitted.pure", "i")] == ("role", "i")
    unbound = {
        (r["slot_group"], r["name"]): (r["binds_to_type"], r["binds_to"])
        for r in rows_of(full_declaration(), "slot_group_subject")
    }
    assert unbound[("antoine.pure", "i")] == (None, None)


def test_locals_and_outputs_are_reified_as_text_and_the_hash_of_the_canonical_tree() -> None:
    decl = scenario("nasa7")
    locals_ = rows_of(decl, "form_local")
    assert [(r["form"], r["name"], r["position"]) for r in locals_[:3]] == [
        ("nasa7", "n", 1),
        ("nasa7", "R", 2),
        ("nasa7", "a1", 3),
    ]
    assert locals_[0]["expression"] == "at(pure.piece(i), T)"
    assert locals_[0]["content_hash"] == hashlib.sha256(b"at(pure.piece(i), T)").hexdigest()
    outputs = rows_of(decl, "form_output")
    assert [(r["form"], r["contract"], r["name"], r["position"]) for r in outputs] == [
        ("nasa7", "standard_state", "cp", 1),
        ("nasa7", "standard_state", "h", 2),
        ("nasa7", "standard_state", "s", 3),
    ]
    for row in outputs:
        assert row["content_hash"] == content_hash(str(row["expression"]))
        assert len(str(row["content_hash"])) == 64


def test_the_content_hash_is_that_of_the_canonical_tree_not_of_the_spelling() -> None:
    original = 'exp(pure.A[i] - pure.B[i] / theta) * unit("Pa")'
    assert content_hash(original) == content_hash("exp( pure.A[i]-pure.B[i]/(theta) ) *unit('Pa')")
    assert serialise(parse(original)) == "exp(pure.A[i] - pure.B[i] / theta) * unit('Pa')"


def test_implicit_blocks_are_reified_with_unknowns_bounds_residuals_and_selection() -> None:
    decl = scenario("implicit_association")
    (block,) = rows_of(decl, "form_implicit")
    assert (block["form"], block["name"], block["position"], block["select_rule"]) == (
        "site_fractions",
        "fractions",
        1,
        "unique",
    )
    assert block["select_expression"] is None and block["select_hash"] is None
    (unknown,) = rows_of(decl, "form_unknown")
    assert (unknown["block"], unknown["name"], unknown["element_kind"], unknown["element"]) == (
        "fractions",
        "X",
        "quantity",
        "Fraction",
    )
    (over,) = rows_of(decl, "form_unknown_set")
    assert (over["contract"], over["unknown"], over["position"], over["set_name"]) == (
        "association",
        "X",
        1,
        "sites",
    )
    bounds = {r["bound"]: r for r in rows_of(decl, "form_unknown_bound")}
    assert set(bounds) == {"lower", "upper", "start"}
    assert (bounds["lower"]["number"], bounds["lower"]["expression"]) == (0.0, None)
    assert (bounds["start"]["number"], bounds["start"]["content_hash"]) == (0.5, None)
    (residual,) = rows_of(decl, "form_residual")
    text = (
        "X[a] * (1 + rho * sum(x[b] * X[b] * pair.delta[a, b] for b in sites)) - 1 for a in sites"
    )
    assert residual["expression"] == text
    assert residual["content_hash"] == residual_hash(text)


def test_a_selection_by_an_expression_and_expression_bounds_are_reified_with_their_hashes() -> None:
    decl = scenario("implicit_cubic")
    by = {r["form"]: r for r in rows_of(decl, "form_implicit")}
    assert (
        by["srk_smallest"]["select_rule"] == "smallest"
        and by["srk_smallest"]["select_expression"] is None
    )
    stable = by["srk_stable"]
    assert stable["select_rule"] == "by"
    assert str(stable["select_expression"]).startswith("core.R * T * (p * v")
    assert stable["select_hash"] == content_hash(str(stable["select_expression"]))
    bounds = {(r["form"], r["bound"]): r for r in rows_of(decl, "form_unknown_bound")}
    lower = bounds[("srk_stable", "lower")]
    assert (lower["number"], lower["expression"]) == (None, "b")
    assert lower["content_hash"] == content_hash("b")
    assert ("srk_stable", "start") not in bounds


def edited(tmp_path: Path, scenario_name: str, old: str, new: str) -> Path:
    shutil.copytree(VALID / scenario_name, tmp_path / scenario_name)
    path = tmp_path / scenario_name / "forms.toml"
    text = path.read_text()
    assert old in text
    path.write_text(text.replace(old, new))
    return tmp_path / scenario_name


def test_the_fingerprint_covers_expressions(tmp_path: Path) -> None:
    before = declaration_fingerprint(scenario("antoine"), b"")
    changed = load(
        edited(tmp_path, "antoine", 'theta = "T + pure.C[i]"', 'theta = "T + 2 * pure.C[i]"')
    ).require()
    assert declaration_fingerprint(changed, b"") != before


def test_the_evaluation_hash_covers_what_evaluating_an_output_reads(tmp_path: Path) -> None:
    from thermo_knowledge.expression.canonical import evaluation_hash

    def hash_of(folder: str, old: str, new: str, form: str = "antoine") -> str:
        where = tmp_path / folder
        where.mkdir()
        changed = load(edited(where, "antoine", old, new)).require()
        return evaluation_hash(changed.forms[form], "p_sat")

    before = evaluation_hash(scenario("antoine").forms["antoine"], "p_sat")
    assert len(before) == 64 and before == evaluation_hash(
        scenario("antoine").forms["antoine"], "p_sat"
    )
    # a local and the output are what it reads
    assert hash_of("local", 'theta = "T + pure.C[i]"', 'theta = "T + 2 * pure.C[i]"') != before
    assert hash_of("output", "- pure.B[i] / theta)", "- pure.B[i] / theta) * 1") != before
    # spelling is not content: whitespace and redundant parentheses serialise alike
    assert hash_of("spacing", 'theta = "T + pure.C[i]"', 'theta = "(T   +  pure.C[i])"') == before
    # another form's text is not read
    other = evaluation_hash(scenario("antoine").forms["antoine_bar"], "p_sat")
    assert other != before


def test_the_evaluation_hash_is_per_output_and_reified_in_meta() -> None:
    from thermo_knowledge.expression.canonical import evaluation_hash
    from thermo_knowledge.generate.meta_rows import meta_rows

    decl = scenario("cubic")
    rows = {
        (form, name): hash_
        for form, _contract, name, _position, _text, _content, hash_ in meta_rows(decl)[
            "form_output"
        ]
    }
    assert rows
    for (form, name), hash_ in rows.items():
        assert hash_ == evaluation_hash(decl.forms[form], name)
    outputs = [name for (form, name) in rows if form == "cubic_srk"]
    assert len(outputs) == len(set(outputs))
    if len(outputs) > 1:
        assert len({rows[("cubic_srk", name)] for name in outputs}) == len(outputs)


def test_the_fingerprint_covers_implicit_blocks(tmp_path: Path) -> None:
    before = declaration_fingerprint(scenario("implicit_association"), b"")
    cases = [
        ("- 1 for a in sites", "- 1.5 for a in sites"),
        ("lower = 0,", "lower = 0.1,"),
        ("start = 0.5", "start = 0.6"),
    ]
    for n, (old, new) in enumerate(cases):
        folder = tmp_path / str(n)
        folder.mkdir()
        changed = load(edited(folder, "implicit_association", old, new)).require()
        assert declaration_fingerprint(changed, b"") != before, old
    selection = tmp_path / "select"
    selection.mkdir()
    cubic = declaration_fingerprint(scenario("implicit_cubic"), b"")
    changed = load(
        edited(selection, "implicit_cubic", 'select = "largest"', 'select = "smallest"')
    ).require()
    assert declaration_fingerprint(changed, b"") != cubic


def test_the_fingerprint_covers_roles_sets_and_bindings(tmp_path: Path) -> None:
    before = declaration_fingerprint(scenario("cubic"), b"")
    rebased = load(
        edited(tmp_path, "cubic", 'basis = "mole_fraction"', 'basis = "mass_fraction"')
    ).require()
    assert declaration_fingerprint(rebased, b"") != before
    other = tmp_path / "other"
    other.mkdir()
    rebound = load(
        edited(
            other,
            "cubic",
            'bind = { i = "components", j = "components" }',
            'bind = { j = "components", i = "components" }',
        )
    ).require()
    # the same binding written in another order is the same declaration
    assert declaration_fingerprint(rebound, b"") == before


# -- in a database -----------------------------------------------------------------------------


def test_implicit_blocks_build_in_a_database_and_the_constraints_hold() -> None:
    decl = scenario("implicit_cubic")
    with TestDatabase() as database:
        build_database(database.url, decl, tree=NO_PHYSICAL)
        with psycopg.connect(database.url) as conn:
            assert conn.execute("SELECT count(*) FROM meta.form_implicit").fetchone() == (4,)
            assert conn.execute("SELECT count(*) FROM meta.form_residual").fetchone() == (4,)
            assert conn.execute(
                "SELECT select_rule FROM meta.form_implicit WHERE form = 'srk_stable'"
            ).fetchone() == ("by",)
            # `by` needs its expression, and another rule must not have one
            with pytest.raises(errors.CheckViolation), conn.transaction():
                conn.execute(
                    "UPDATE meta.form_implicit SET select_rule = 'by' WHERE form = 'srk_unique'"
                )
            with pytest.raises(errors.CheckViolation), conn.transaction():
                conn.execute(
                    "UPDATE meta.form_implicit SET select_rule = 'unique' WHERE form = 'srk_stable'"
                )
            with pytest.raises(errors.CheckViolation), conn.transaction():
                conn.execute(
                    "UPDATE meta.form_implicit SET select_rule = 'median' WHERE form = 'srk_unique'"
                )
            # a bound is a number or an expression, not both and not neither
            with pytest.raises(errors.CheckViolation), conn.transaction():
                conn.execute(
                    "INSERT INTO meta.form_unknown_bound (form, block, unknown, bound, number, expression, content_hash) "
                    "VALUES ('srk_unique', 'volume', 'v', 'start', 1.0, 'b', %s)",
                    ("0" * 64,),
                )
            with pytest.raises(errors.CheckViolation), conn.transaction():
                conn.execute(
                    "INSERT INTO meta.form_unknown_bound (form, block, unknown, bound) "
                    "VALUES ('srk_unique', 'volume', 'v', 'start')"
                )
            with pytest.raises(errors.CheckViolation), conn.transaction():
                conn.execute(
                    "INSERT INTO meta.form_unknown_bound (form, block, unknown, bound, expression, content_hash) "
                    "VALUES ('srk_unique', 'volume', 'v', 'start', 'b', 'short')"
                )
            with pytest.raises(errors.CheckViolation), conn.transaction():
                conn.execute(
                    "INSERT INTO meta.form_unknown_bound (form, block, unknown, bound, number) "
                    "VALUES ('srk_unique', 'volume', 'v', 'middle', 1.0)"
                )
            # a residual belongs to a block of the form
            with pytest.raises(errors.ForeignKeyViolation), conn.transaction():
                conn.execute(
                    "INSERT INTO meta.form_residual (form, block, position, expression, content_hash) "
                    "VALUES ('srk_unique', 'nowhere', 9, 'x', %s)",
                    ("0" * 64,),
                )
                conn.execute("SET CONSTRAINTS ALL IMMEDIATE")
    association = scenario("implicit_association")
    with TestDatabase() as database:
        build_database(database.url, association, tree=NO_PHYSICAL)
        with psycopg.connect(database.url) as conn:
            assert conn.execute(
                "SELECT contract, set_name FROM meta.form_unknown_set WHERE unknown = 'X'"
            ).fetchone() == ("association", "sites")
            # an unknown ranges over a set of the contract the form implements
            with pytest.raises(errors.ForeignKeyViolation), conn.transaction():
                conn.execute(
                    "INSERT INTO meta.form_unknown_set (form, contract, block, unknown, position, set_name) "
                    "VALUES ('site_fractions', 'association', 'fractions', 'X', 2, 'nowhere')"
                )
                conn.execute("SET CONSTRAINTS ALL IMMEDIATE")


def test_the_new_meta_rows_build_in_a_database_and_the_constraints_hold() -> None:
    decl = scenario("cubic")
    with TestDatabase() as database:
        build_database(database.url, decl, tree=NO_PHYSICAL)
        with psycopg.connect(database.url) as conn:
            assert conn.execute("SELECT count(*) FROM meta.form_output").fetchone() == (3,)
            assert conn.execute("SELECT count(*) FROM meta.form_local").fetchone() == (3,)
            assert conn.execute(
                "SELECT set_name FROM meta.contract_argument_set WHERE argument = 'x'"
            ).fetchone() == ("components",)
            assert conn.execute(
                "SELECT binds_to_type, binds_to FROM meta.slot_group_subject "
                "WHERE slot_group = 'cubic_srk.pair' AND name = 'i'"
            ).fetchone() == ("set", "components")
            # an output row must name an output of the contract
            with pytest.raises(errors.ForeignKeyViolation), conn.transaction():
                conn.execute(
                    "INSERT INTO meta.form_output "
                    "(form, contract, name, position, expression, content_hash, evaluation_hash) "
                    "VALUES ('cubic_srk', 'eos_pressure', 'nothing', 9, 'T', %s, %s)",
                    ("0" * 64, "0" * 64),
                )
                conn.execute("SET CONSTRAINTS ALL IMMEDIATE")
            with pytest.raises(errors.ForeignKeyViolation), conn.transaction():
                conn.execute(
                    "INSERT INTO meta.form_output "
                    "(form, contract, name, position, expression, content_hash, evaluation_hash) "
                    "VALUES ('cubic_srk', 'alpha_function', 'value', 9, 'T', %s, %s)",
                    ("0" * 64, "0" * 64),
                )
                conn.execute("SET CONSTRAINTS ALL IMMEDIATE")
            with pytest.raises(errors.CheckViolation), conn.transaction():
                conn.execute(
                    "INSERT INTO meta.form_output "
                    "(form, contract, name, position, expression, content_hash, evaluation_hash) "
                    "VALUES ('cubic_srk', 'eos_pressure', 'p', 9, 'T', 'short', %s)",
                    ("0" * 64,),
                )
            with pytest.raises(errors.CheckViolation), conn.transaction():
                conn.execute(
                    "INSERT INTO meta.form_output "
                    "(form, contract, name, position, expression, content_hash, evaluation_hash) "
                    "VALUES ('cubic_srk', 'eos_pressure', 'p', 9, 'T', %s, 'short')",
                    ("0" * 64,),
                )
            with pytest.raises(errors.CheckViolation), conn.transaction():
                conn.execute(
                    "UPDATE meta.slot_group_subject SET binds_to = NULL "
                    "WHERE slot_group = 'cubic_srk.pair' AND name = 'i'"
                )
