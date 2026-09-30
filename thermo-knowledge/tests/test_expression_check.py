# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Name resolution and dimensional checking: each refusal of expressions.md section 4 is one small
declaration, refused with its stable code, construct path and position in the expression text."""

from __future__ import annotations

import tomllib

import pytest
from expression_support import BROKEN, VALID, load

from thermo_knowledge.declaration import Diagnostic

CASES: dict[str, list[dict[str, object]]] = {
    name: body["expect"]
    for name, body in tomllib.loads((BROKEN / "cases.toml").read_text()).items()
}


def test_every_case_directory_has_expectations_and_a_declaration() -> None:
    directories = {path.name for path in BROKEN.iterdir() if path.is_dir()}
    assert directories == set(CASES)
    assert all((BROKEN / name / "forms.toml").is_file() for name in CASES)


@pytest.mark.parametrize("case", sorted(CASES))
def test_broken_expression_is_refused_with_its_code_construct_and_position(case: str) -> None:
    result = load(BROKEN / case)
    assert result.declaration is None
    listing = "\n".join(f"{d} @ {d.line}:{d.column}" for d in result.diagnostics)
    for expected in CASES[case]:
        matching = [
            d
            for d in result.diagnostics
            if d.code == expected["code"]
            and d.construct == expected.get("construct", d.construct)
            and d.line == expected.get("line", d.line)
            and d.column == expected.get("column", d.column)
        ]
        assert matching, f"{case}: no {expected} among\n{listing}"
        assert all(d.message for d in matching)
        assert all(
            d.document == f"{BROKEN.name}/{case}/forms.toml" or d.document.endswith("forms.toml")
            for d in matching
        )


@pytest.mark.parametrize("case", sorted(CASES))
def test_a_refusal_inside_expression_text_carries_a_position(case: str) -> None:
    result = load(BROKEN / case)
    in_text = [
        d
        for d in result.diagnostics
        if (".outputs." in d.construct or ".let." in d.construct)
        and d.code not in ("bad-name", "unknown-output")
    ]
    assert all(d.line >= 1 and d.column >= 1 for d in in_text), [str(d) for d in in_text]


def refusals(case: str, code: str) -> list[Diagnostic]:
    return [d for d in load(BROKEN / case).diagnostics if d.code == code]


def test_a_dimension_mismatch_states_both_dimensions() -> None:
    (refusal,) = refusals("dimension_mismatch", "dimension-mismatch")
    assert "[temperature]" in refusal.message and "dimensionless" in refusal.message


def test_an_output_of_the_wrong_dimension_states_both() -> None:
    (refusal,) = refusals("output_of_the_wrong_dimension", "output-dimension")
    assert "Pressure" in refusal.message and "[temperature]" in refusal.message
    assert "[mass]" in refusal.message


def test_a_non_integer_power_names_the_base_dimension() -> None:
    (refusal,) = refusals("non_integer_power_of_a_dimensioned_base", "bad-power")
    assert "[temperature]" in refusal.message and "integer" in refusal.message


def test_a_missing_output_names_the_form_and_the_output() -> None:
    (refusal,) = refusals("expressed_form_missing_one_output", "missing-output")
    assert "`q`" in refusal.message and "`f`" in refusal.message and "expressed" in refusal.message


def test_a_refusal_prints_its_position() -> None:
    (refusal,) = refusals("position_on_second_line", "unknown-name")
    assert str(refusal).endswith("(line 2, column 5)")


def test_a_cycle_among_sub_form_contracts_is_a_contract_cycle() -> None:
    (refusal,) = refusals("subform_contracts_form_a_cycle", "contract-cycle")
    assert "outer" in refusal.message and "inner" in refusal.message


def test_one_expression_reports_every_problem_it_has() -> None:
    result = load(BROKEN / "local_refers_to_a_later_local")
    assert [d.code for d in result.diagnostics] == ["local-order"]


@pytest.mark.parametrize("scenario", sorted(path.name for path in VALID.iterdir() if path.is_dir()))
def test_every_valid_scenario_loads_without_diagnostics(scenario: str) -> None:
    result = load(VALID / scenario)
    assert result.diagnostics == ()
    assert result.declaration is not None


def test_a_catalogued_form_may_declare_an_implicit_block_and_has_no_expression() -> None:
    decl = load(VALID / "implicit_catalogued").require()
    form = decl.forms["association"]
    assert form.status == "catalogued"
    assert [block.name for block in form.implicit] == ["site_fractions"]
    assert form.outputs == () and form.locals == ()


def test_an_implicit_block_is_resolved_into_unknowns_bounds_residuals_and_a_selection() -> None:
    decl = load(VALID / "implicit_cubic").require()
    block = decl.forms["srk_stable"].implicit[0]
    assert (block.name, block.select) == ("volume", "by")
    assert block.select_by is not None and block.select_by.startswith("core.R * T * (p * v")
    (unknown,) = block.unknowns
    assert (unknown.name, unknown.type.text, unknown.over) == ("v", "MolarVolume", ())
    assert (unknown.lower.text, unknown.lower.number) == ("b", None)
    assert unknown.upper.text == "core.R * T / p + b" and unknown.start is None
    assert [r.text for r in block.residuals] == [
        "p * (v - b) * v * (v + b) - core.R * T * v * (v + b) + a * (v - b)"
    ]
    assert block.residuals[0].construct == "forms.srk_stable.implicit.volume.residuals[0]"
    association = load(VALID / "implicit_association").require().forms["site_fractions"]
    (fractions,) = association.implicit[0].unknowns
    assert fractions.over == ("sites",)
    assert (fractions.lower.number, fractions.upper.number, fractions.start.number) == (
        0.0,
        1.0,
        0.5,
    )


def test_an_unknown_and_the_locals_that_depend_on_it_are_found() -> None:
    from thermo_knowledge.expression.scope import FormScope

    decl = load(VALID / "implicit_association").require()
    scope = FormScope.of(decl, decl.forms["site_fractions"])
    assert set(scope.unknowns) == {"X"} and scope.dependent == {"unbonded"}
    cubic = load(VALID / "implicit_cubic").require()
    assert FormScope.of(cubic, cubic.forms["srk_unique"]).dependent == frozenset()


def test_locals_and_outputs_are_kept_in_declared_order() -> None:
    decl = load(VALID / "nasa7").require()
    form = decl.forms["nasa7"]
    assert [local.name for local in form.locals] == ["n", "R", *(f"a{k}" for k in range(1, 8))]
    assert [output.name for output in form.outputs] == ["cp", "h", "s"]
    assert form.outputs[0].construct == "forms.nasa7.outputs.cp"
    assert form.locals[0].text == "at(pure.piece(i), T)"
