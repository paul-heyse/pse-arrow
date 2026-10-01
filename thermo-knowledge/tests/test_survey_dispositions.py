# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Dispositions: the need rule, the format and the validation of each entry."""

from __future__ import annotations

from pathlib import Path

import pytest

from declaration_support import full_declaration
from survey_support import (
    ALPHA_NEEDING,
    BETA_NEEDING,
    CALCULATIONS,
    DECLARED,
    FIXTURES,
    REFS,
    codes,
    complete_dispositions,
    entry,
    make_tree,
    write_dispositions,
)
from thermo_knowledge.survey_index import (
    Code,
    RefIndex,
    load_dispositions,
    load_surveys,
    needs_disposition,
)
from thermo_knowledge.survey_index.dispositions import (
    DeclaredNames,
    build_ref_index,
    parse_alignment_items,
    parse_review_findings,
)
from thermo_knowledge.survey_index.structs import Construct


def construct(precision: str, loss: str) -> Construct:
    return Construct(
        name="c",
        locator="l",
        meaning="m",
        subject="s",
        fields=[],
        origin="authored",
        conventions="",
        absence="",
        validity="",
        provenance="",
        count="1",
        example="e",
        candidate="x",
        precision=precision,
        loss=loss,
    )


@pytest.mark.parametrize(
    ("precision", "loss", "needs"),
    [
        ("exact", "", False),
        ("exact", "   ", False),
        ("exact", "none", False),
        ("exact", " None. ", False),
        ("exact", "NONE", False),
        ("exact", "none of the units are stated", True),
        ("exact", "The unit is assumed.", True),
        ("narrower", "", True),
        ("broader", "", True),
        ("close", "", True),
        ("unmapped", "", True),
        ("close", "none", True),
    ],
)
def test_need_for_a_disposition(precision: str, loss: str, needs: bool) -> None:
    assert needs_disposition(construct(precision, loss)) is needs


def load(tree: Path, refs: RefIndex = REFS) -> tuple[dict, tuple]:
    surveys = load_surveys(tree, calculations=CALCULATIONS)
    assert surveys.diagnostics == ()
    result = load_dispositions(tree, surveys.surveys, refs)
    return result.by_source, result.diagnostics


def test_the_fixture_needs_exactly_the_documented_constructs(tmp_path: Path) -> None:
    surveys = load_surveys(make_tree(tmp_path)).surveys
    needing = {
        source: tuple(item.name for item in survey.construct if needs_disposition(item))
        for source, survey in surveys.items()
    }
    assert needing == {"alpha": ALPHA_NEEDING, "beta": BETA_NEEDING}


def test_complete_dispositions_load(tmp_path: Path) -> None:
    tree = make_tree(tmp_path)
    complete_dispositions(tree)
    by_source, diagnostics = load(tree)
    assert diagnostics == ()
    assert sorted(by_source["alpha"]) == sorted(ALPHA_NEEDING)
    assert by_source["alpha"]["unmapped record"].ref == ""
    assert by_source["beta"]["broad table"].disposition == "model_change"


def test_no_dispositions_is_valid(tmp_path: Path) -> None:
    by_source, diagnostics = load(make_tree(tmp_path))
    assert by_source == {} and diagnostics == ()


def diagnose(tmp_path: Path, text: str, source: str = "alpha") -> list:
    tree = make_tree(tmp_path)
    write_dispositions(tree, source, text)
    return codes(load(tree)[1])


def test_unknown_construct(tmp_path: Path) -> None:
    found = diagnose(tmp_path, entry("no such construct", "out_of_scope", ""))
    assert found == [("disposition", "no such construct", "construct", Code.UNKNOWN_CONSTRUCT)]


def test_a_construct_name_is_matched_exactly(tmp_path: Path) -> None:
    found = diagnose(tmp_path, entry("Close record", "held_by_model", "kind:species"))
    assert [item[3] for item in found] == [Code.UNKNOWN_CONSTRUCT]


def test_the_construct_of_another_source_is_unknown(tmp_path: Path) -> None:
    found = diagnose(tmp_path, entry("broad table", "out_of_scope", ""), source="alpha")
    assert [item[3] for item in found] == [Code.UNKNOWN_CONSTRUCT]


def test_duplicate_disposition(tmp_path: Path) -> None:
    text = entry("close record", "mapping_loss", "mechanism:x") + entry(
        "close record", "model_change", "alignment-notes #1"
    )
    assert diagnose(tmp_path, text) == [
        ("disposition", "close record", "construct", Code.DUPLICATE_DISPOSITION)
    ]


def test_a_disposition_for_a_construct_that_needs_none_is_refused(tmp_path: Path) -> None:
    text = entry("plain record", "held_by_model", "kind:species") + entry(
        "exact, loss none", "held_by_model", "kind:species"
    )
    assert [item[3] for item in diagnose(tmp_path, text)] == [Code.NOT_NEEDED, Code.NOT_NEEDED]


@pytest.mark.parametrize("disposition", ["model_change", "mapping_loss", "held_by_model"])
def test_ref_is_required_except_for_out_of_scope(tmp_path: Path, disposition: str) -> None:
    found = diagnose(tmp_path, entry("close record", disposition, ""))
    assert found == [("disposition", "close record", "ref", Code.MISSING_REF)]


def test_out_of_scope_may_omit_or_give_a_ref(tmp_path: Path) -> None:
    text = (
        '[[disposition]]\nconstruct = "close record"\ndisposition = "out_of_scope"\n'
        'reason = "Outside."\n'
    ) + entry("narrower record", "out_of_scope", "mechanism:elsewhere")
    assert diagnose(tmp_path, text) == []


@pytest.mark.parametrize(
    "ref",
    [
        "kind:species",
        "relation:uses",
        "form:nasa7",
        "enum:basis",
        "mechanism:mapping rule for units",
        "alignment-notes #3",
        "review F14",
    ],
)
def test_valid_refs(tmp_path: Path, ref: str) -> None:
    assert diagnose(tmp_path, entry("close record", "model_change", ref)) == []


@pytest.mark.parametrize(
    "ref",
    ["kind:nothing", "relation:species", "form:species", "enum:uses", "kind:"],
)
def test_a_model_ref_that_the_declaration_lacks_is_refused(tmp_path: Path, ref: str) -> None:
    found = diagnose(tmp_path, entry("close record", "held_by_model", ref))
    assert found[0][2] == "ref" and found[0][3] in (Code.UNKNOWN_REF_TARGET, Code.BAD_REF)


def test_an_alignment_item_that_does_not_exist_is_refused(tmp_path: Path) -> None:
    found = diagnose(tmp_path, entry("close record", "model_change", "alignment-notes #9"))
    assert found == [("disposition", "close record", "ref", Code.UNKNOWN_REF_TARGET)]


def test_a_review_finding_that_does_not_exist_is_refused(tmp_path: Path) -> None:
    found = diagnose(tmp_path, entry("close record", "model_change", "review F09"))
    assert found == [("disposition", "close record", "ref", Code.UNKNOWN_REF_TARGET)]


@pytest.mark.parametrize(
    "ref",
    ["alignment-notes 2", "alignment-notes #x", "F14", "review 14", "somewhere", "Kind:species"],
)
def test_an_unrecognised_ref_is_refused(tmp_path: Path, ref: str) -> None:
    found = diagnose(tmp_path, entry("close record", "model_change", ref))
    assert found == [("disposition", "close record", "ref", Code.BAD_REF)]


def test_unscheduled_is_only_for_model_change(tmp_path: Path) -> None:
    assert diagnose(tmp_path / "a", entry("close record", "model_change", "unscheduled")) == []
    found = diagnose(tmp_path / "b", entry("close record", "held_by_model", "unscheduled"))
    assert found == [("disposition", "close record", "ref", Code.BAD_REF)]


def test_kind_and_reason_and_keys_are_validated(tmp_path: Path) -> None:
    text = (
        '[[disposition]]\nconstruct = "close record"\ndisposition = "maybe"\nref = "kind:species"\n'
        'reason = ""\nextra = 1\n'
    )
    found = diagnose(tmp_path, text)
    assert ("disposition", "close record", "disposition", Code.BAD_VALUE) in found
    assert ("disposition", "close record", "reason", Code.BAD_VALUE) in found
    assert ("disposition", "close record", "extra", Code.UNKNOWN_KEY) in found


def test_a_missing_reason_is_refused(tmp_path: Path) -> None:
    text = (
        '[[disposition]]\nconstruct = "close record"\ndisposition = "held_by_model"\n'
        'ref = "kind:species"\n'
    )
    assert diagnose(tmp_path, text) == [("disposition", "close record", "reason", Code.MISSING_KEY)]


def test_a_file_for_no_survey_and_a_stray_key_are_refused(tmp_path: Path) -> None:
    tree = make_tree(tmp_path)
    write_dispositions(tree, "gamma", entry("close record", "out_of_scope", ""))
    write_dispositions(tree, "beta", 'note = "x"\n')
    found = load(tree)[1]
    assert {(d.file, d.code) for d in found} == {
        ("survey/dispositions/gamma.toml", Code.UNKNOWN_SOURCE),
        ("survey/dispositions/beta.toml", Code.UNKNOWN_KEY),
    }


def test_a_syntax_error_in_a_disposition_file_is_reported(tmp_path: Path) -> None:
    tree = make_tree(tmp_path)
    write_dispositions(tree, "alpha", "[[disposition\n")
    found = load(tree)[1]
    assert [d.code for d in found] == [Code.TOML_SYNTAX]


def test_every_deviation_of_a_file_is_reported(tmp_path: Path) -> None:
    text = (
        entry("close record", "model_change", "alignment-notes #9")
        + entry("narrower record", "mapping_loss", "")
        + entry("nothing", "out_of_scope", "")
    )
    assert [item[3] for item in diagnose(tmp_path, text)] == [
        Code.UNKNOWN_REF_TARGET,
        Code.MISSING_REF,
        Code.UNKNOWN_CONSTRUCT,
    ]


def test_a_declaration_that_is_unavailable_leaves_model_refs_unchecked(tmp_path: Path) -> None:
    tree = make_tree(tmp_path)
    write_dispositions(tree, "alpha", entry("close record", "held_by_model", "kind:nothing"))
    refs = RefIndex(None, REFS.alignment_items, REFS.review_findings)
    assert load(tree, refs)[1] == ()


# -- the documents a ref cites ----------------------------------------------------------------


def test_alignment_items_are_the_numbered_rows() -> None:
    text = (FIXTURES / "tree" / "docs" / "alignment-notes.md").read_text()
    assert parse_alignment_items(text) == {1, 2, 3}


def test_review_findings_are_the_headings() -> None:
    text = next((FIXTURES / "docs").rglob("*.md")).read_text()
    assert parse_review_findings(text) == {"F01", "F14"}


def test_the_ref_index_resolves_its_documents_from_the_tree(tmp_path: Path) -> None:
    tree = make_tree(tmp_path)
    index, diagnostics = build_ref_index(tree, full_declaration())
    assert diagnostics == []
    assert index.alignment_items == {1, 2, 3}
    assert index.review_findings == {"F01", "F14"}
    assert index.declared is not None and index.declared.kinds
    assert index.declared == DeclaredNames.of(full_declaration())


def test_a_missing_document_is_a_diagnostic(tmp_path: Path) -> None:
    tree = make_tree(tmp_path)
    (tree / "docs" / "alignment-notes.md").unlink()
    (tmp_path / "docs" / "design_review" / "reviews").joinpath(
        "design_review_thermo-knowledge-core_2026-09-30.md"
    ).unlink()
    index, diagnostics = build_ref_index(tree, None)
    assert [d.code for d in diagnostics] == [Code.INPUT_UNAVAILABLE, Code.INPUT_UNAVAILABLE]
    assert [d.file for d in diagnostics] == [
        "docs/alignment-notes.md",
        "docs/design_review/reviews/design_review_thermo-knowledge-core_2026-09-30.md",
    ]
    assert index.alignment_items == frozenset() and index.declared is None
    assert DECLARED.names("kind")
