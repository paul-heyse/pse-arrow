# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""One broken declaration per refusal of section 6, each asserting its code and construct."""

from __future__ import annotations

import tomllib
from pathlib import Path

import pytest
from declaration_support import BROKEN, FULL, copy_full, overlay

from thermo_knowledge.declaration import Code, Diagnostic, load_declaration

CASES: dict[str, list[dict[str, str]]] = {
    name: body["expect"]
    for name, body in tomllib.loads((BROKEN / "cases.toml").read_text()).items()
}


def test_every_case_directory_has_expectations() -> None:
    directories = {path.name for path in BROKEN.iterdir() if path.is_dir()}
    assert directories == set(CASES)


@pytest.mark.parametrize("case", sorted(CASES))
def test_broken_declaration_is_refused_with_its_code_and_construct(
    case: str, tmp_path: Path
) -> None:
    tree = overlay(case, tmp_path)
    result = load_declaration(tree / "model", tree / "forms", contract=None)
    assert result.declaration is None
    for expected in CASES[case]:
        matching = [
            d
            for d in result.diagnostics
            if d.code == expected["code"]
            and d.construct == expected.get("construct", d.construct)
            and d.document == expected.get("document", d.document)
        ]
        assert matching, f"{case}: no {expected} among\n" + "\n".join(map(str, result.diagnostics))
        assert all(d.message for d in matching)


def test_valid_fixture_has_no_diagnostics_so_cases_isolate_their_refusal(tmp_path: Path) -> None:
    copy_full(tmp_path)
    assert load_declaration(tmp_path / "model", tmp_path / "forms", contract=None).diagnostics == ()


def test_a_refusal_outside_the_case_would_show(tmp_path: Path) -> None:
    # every case's diagnostics all come from its own overlay or the thing it breaks
    for case in ("unknown_key", "bad_name_kind", "default_mismatch", "set_optional"):
        tree = overlay(case, tmp_path / case)
        result = load_declaration(tree / "model", tree / "forms", contract=None)
        assert {d.document for d in result.diagnostics} == {"model/zz_case.toml"}


def test_loading_collects_all_diagnostics_not_the_first(tmp_path: Path) -> None:
    tree = overlay("collects_all", tmp_path)
    result = load_declaration(tree / "model", tree / "forms", contract=None)
    codes = {d.code for d in result.diagnostics}
    assert {"unknown-name", "bad-default"} <= codes
    assert len(result.diagnostics) >= 2


def test_structural_refusals_across_documents_are_collected(tmp_path: Path) -> None:
    tree = overlay("unknown_key", tmp_path)
    (tree / "forms" / "zz_other.toml").write_text('module = "other"\ndoc = "d"\nbogus = 1\n')
    result = load_declaration(tree / "model", tree / "forms", contract=None)
    assert {(d.document, d.code) for d in result.diagnostics} == {
        ("model/zz_case.toml", "unknown-key"),
        ("forms/zz_other.toml", "unknown-key"),
    }


def test_missing_manifest_is_refused(tmp_path: Path) -> None:
    (tmp_path / "model").mkdir()
    result = load_declaration(tmp_path / "model", tmp_path / "forms", contract=None)
    assert [(d.code, d.document) for d in result.diagnostics] == [
        ("missing-manifest", "model/manifest.toml")
    ]


def test_unknown_key_in_the_manifest_is_refused(tmp_path: Path) -> None:
    copy_full(tmp_path)
    (tmp_path / "model" / "manifest.toml").write_text(
        'doc = "d"\nbogus = 1\n[framework]\nsurprise = "x"\n'
    )
    result = load_declaration(tmp_path / "model", tmp_path / "forms", contract=None)
    assert [(d.document, d.construct, d.code) for d in result.diagnostics] == [
        ("model/manifest.toml", "manifest.bogus", "unknown-key")
    ]


def test_diagnostics_are_structured_and_printable() -> None:
    diagnostic = Diagnostic(
        document="model/x.toml", construct="kinds.k", code=Code.MISSING_DOC, message="oops"
    )
    assert str(diagnostic) == "model/x.toml: kinds.k: [missing-doc] oops"
    assert Code.UNKNOWN_KEY == "unknown-key"


def test_fixture_constant_paths_exist() -> None:
    assert (FULL / "model" / "manifest.toml").is_file()
