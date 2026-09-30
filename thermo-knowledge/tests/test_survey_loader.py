# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The survey loader: it accepts a conforming record and reports each kind of deviation."""

from __future__ import annotations

from pathlib import Path

import pytest
from survey_support import CALCULATIONS, codes, edit, make_tree, write_survey

from thermo_knowledge.survey_index import Code, load_surveys


def load(tree: Path) -> tuple[dict, tuple]:
    result = load_surveys(tree, calculations=CALCULATIONS)
    return result.surveys, result.diagnostics


def test_conforming_fixture_loads_without_diagnostics(tmp_path: Path) -> None:
    surveys, diagnostics = load(make_tree(tmp_path))
    assert diagnostics == ()
    assert sorted(surveys) == ["alpha", "beta"]
    alpha = surveys["alpha"]
    assert alpha.additional_pins == {"alpha_code": "ba9876543210"}
    assert [item.name for item in alpha.construct][:2] == ["plain record", "exact but lossy"]
    assert alpha.construct[3].values == ["a", "b"]
    assert alpha.construct[0].values is None
    assert alpha.model_family[0].class_ == "other (an invented equation)"
    assert alpha.model_family[0].composes[1].slot == ""
    assert alpha.model_family[1].variants is None
    assert alpha.payload[1].documents[0].slug == "rel-1"
    assert surveys["beta"].additional_pins == {}


def test_a_missing_survey_directory_is_an_empty_set(tmp_path: Path) -> None:
    result = load_surveys(tmp_path)
    assert result.surveys == {} and result.diagnostics == ()


# (old text, new text, expected (table, record, key, code)) on alpha.toml
DEVIATIONS = [
    ('precision = "close"', 'precision = "nearly"',
     ("construct", "close record", "precision", Code.BAD_VALUE)),
    ('unit = "K"', "unit = 3", ("construct", "plain record", "fields[0].unit", Code.WRONG_TYPE)),
    ('role = "input" },\n]\norigin = "authored', 'role = "sometimes" },\n]\norigin = "authored',
     ("construct", "plain record", "fields[0].role", Code.BAD_VALUE)),
    ('shape = "scalar", role = "input" },\n]\norigin = "authored',
     'shape = "blob", role = "input" },\n]\norigin = "authored',
     ("construct", "plain record", "fields[0].shape", Code.BAD_VALUE)),
    ('origin = "other (hand edited)"', 'origin = "digitised"',
     ("construct", "narrower record", "origin", Code.BAD_VALUE)),
    ('origin = "not stated"', 'origin = "not statedly"',
     ("construct", "unmapped record", "origin", Code.BAD_VALUE)),
    ('offered = "yes"', "offered = true", ("capability", "flash_tp", "offered", Code.WRONG_TYPE)),
    ('offered = "yes"', 'offered = "maybe"', ("capability", "flash_tp", "offered", Code.BAD_VALUE)),
    ('evidence = "documentation"', 'evidence = "hearsay"',
     ("capability", "virial_sweep (proposed)", "evidence", Code.BAD_VALUE)),
    ('calculation = "virial_sweep (proposed)"', 'calculation = "virial_sweep"',
     ("capability", "virial_sweep", "calculation", Code.BAD_VALUE)),
    ('calculation = "virial_sweep (proposed)"', 'calculation = "proposed:virial_sweep"',
     ("capability", "proposed:virial_sweep", "calculation", Code.BAD_VALUE)),
    ('calculation = "virial_sweep (proposed)"',
     'calculation = "virial_sweep (proposed new key: x)"',
     ("capability", "virial_sweep (proposed new key: x)", "calculation", Code.BAD_VALUE)),
    ('class = "wrapper"', 'class = "wrapping"',
     ("model_family", "Wrapper", "class", Code.BAD_VALUE)),
    ('class = "wrapper"', 'class = "other:x"',
     ("model_family", "Wrapper", "class", Code.BAD_VALUE)),
    ('closed_form = "procedural"', 'closed_form = "sometimes"',
     ("model_family", "Wrapper", "closed_form", Code.BAD_VALUE)),
    ('composes = []', 'composes = "none"',
     ("model_family", "Wrapper", "composes", Code.WRONG_TYPE)),
    ('composes = []', 'composes = ["a mixing rule"]',
     ("model_family", "Wrapper", "composes[0]", Code.NOT_A_TABLE)),
    ('{ slot = "", accepts = "a mixing rule", default = "" }',
     '{ slot = "", accepts = "a mixing rule" }',
     ("model_family", "AlphaEOS", "composes[1].default", Code.MISSING_KEY)),
    ('{ selector = "version=2", selects', '{ flag = "version=2", selects',
     ("model_family", "AlphaEOS", "variants[0].flag", Code.UNKNOWN_KEY)),
    ('variants = [\n  { selector = "version=2", selects = "a different default table" },\n]',
     'variants = "a different default table"',
     ("model_family", "AlphaEOS", "variants", Code.WRONG_TYPE)),
    ('read_by_source = "partly"', 'read_by_source = "yes"',
     ("payload", "data/*.json", "read_by_source", Code.BAD_VALUE)),
    ("files = 3", 'files = "3"', ("payload", "data/*.json", "files", Code.WRONG_TYPE)),
    ("bytes = 1200", "bytes = 1200.5", ("payload", "data/*.json", "bytes", Code.WRONG_TYPE)),
    ('records = "14 objects, counted by parsing"', "records = 14",
     ("payload", "data/*.json", "records", Code.WRONG_TYPE)),
    ('count = "3 records"\nexample = "data/plain.json#/0"',
     'count = 3\nexample = "data/plain.json#/0"',
     ("construct", "plain record", "count", Code.WRONG_TYPE)),
    ('values = ["a", "b"]', 'values = "a, b"',
     ("construct", "close record", "values", Code.WRONG_TYPE)),
    ('surveyed = "2026-01-02"', 'surveyed = "yesterday"', ("", "", "surveyed", Code.BAD_VALUE)),
    ('source = "alpha"', 'source = "gamma"', ("", "", "source", Code.SOURCE_MISMATCH)),
    ('pin = "0123456789ab"\n', "", ("", "", "pin", Code.MISSING_KEY)),
    ('additional_pins = { alpha_code = "ba9876543210" }',
     "additional_pins = { alpha_code = 7 }", ("", "", "additional_pins", Code.WRONG_TYPE)),
    ('why_it_matters = "The mapping factor depends on it."', 'why = "x"',
     ("question", "units", "why", Code.UNKNOWN_KEY)),
    ('name = "units"\n', "", ("convention", "alpha/units.py", "name", Code.MISSING_KEY)),
    ('name = "plain record"', 'name = ""', ("construct", "[0]", "name", Code.BAD_VALUE)),
    ('name = "exact, loss none"', 'name = "plain record"',
     ("construct", "plain record", "name", Code.DUPLICATE_NAME)),
]


@pytest.mark.parametrize(("old", "new", "expected"), DEVIATIONS)
def test_each_kind_of_deviation_is_reported_at_its_key(
    tmp_path: Path, old: str, new: str, expected: tuple[str, str, str, Code]
) -> None:
    tree = make_tree(tmp_path)
    edit(tree, "alpha", old, new)
    _, diagnostics = load(tree)
    assert expected in codes(diagnostics)
    hit = next(d for d in diagnostics if (d.table, d.record, d.key, d.code) == expected)
    assert hit.file == "survey/alpha.toml" and hit.message


def test_an_unknown_top_level_key_and_a_non_table_are_reported(tmp_path: Path) -> None:
    tree = make_tree(tmp_path)
    edit(tree, "beta", 'summary = "A second', 'colour = "red"\nsummary = "A second')
    edit(tree, "beta", "[[payload]]", "question = 3\n\n[[payload]]")
    _, diagnostics = load(tree)
    found = codes(diagnostics)
    assert ("", "", "colour", Code.UNKNOWN_KEY) in found
    assert ("question", "", "", Code.NOT_A_TABLE) in found


def test_toml_syntax_is_a_diagnostic_and_other_files_still_load(tmp_path: Path) -> None:
    tree = make_tree(tmp_path)
    write_survey(tree, "broken", 'source = "broken"\n[[construct\n')
    surveys, diagnostics = load(tree)
    assert [d.code for d in diagnostics] == [Code.TOML_SYNTAX]
    assert diagnostics[0].file == "survey/broken.toml"
    assert sorted(surveys) == ["alpha", "beta"]


def test_every_deviation_is_reported_not_only_the_first(tmp_path: Path) -> None:
    tree = make_tree(tmp_path)
    edit(tree, "alpha", 'precision = "close"', 'precision = "nearly"')
    edit(tree, "alpha", 'unit = "K"', "unit = 3")
    edit(tree, "alpha", 'offered = "yes"', "offered = true")
    edit(tree, "alpha", 'closed_form = "procedural"', 'closed_form = "sometimes"')
    edit(tree, "beta", 'origin = "authored (by hand)"', 'origin = "by hand"')
    _, diagnostics = load(tree)
    assert len(diagnostics) == 5
    assert {d.file for d in diagnostics} == {"survey/alpha.toml", "survey/beta.toml"}


def test_a_record_with_a_deviation_is_left_out_of_the_loaded_survey(tmp_path: Path) -> None:
    tree = make_tree(tmp_path)
    edit(tree, "alpha", 'precision = "close"', 'precision = "nearly"')
    surveys, diagnostics = load(tree)
    assert len(diagnostics) == 1
    names = [item.name for item in surveys["alpha"].construct]
    assert "close record" not in names and len(names) == 5


def test_a_proposed_key_needs_no_vocabulary_but_a_plain_key_does(tmp_path: Path) -> None:
    tree = make_tree(tmp_path)
    edit(tree, "alpha", 'calculation = "flash_tp"', 'calculation = "flash_xy"')
    assert codes(load(tree)[1]) == [
        ("capability", "flash_xy", "calculation", Code.BAD_VALUE)
    ]
    # Without a vocabulary only the form of the key is checked.
    assert load_surveys(tree).diagnostics == ()


@pytest.mark.parametrize(
    ("key", "accepted"),
    [
        ('origin = "authored"', True),
        ('origin = "authored and transcribed"', True),
        ('origin = "authored; no source column"', True),
        ('origin = "not stated (the file is silent)"', True),
        ('origin = "not recorded"', True),
        ('origin = "other (digitised)"', True),
        ('origin = "authorship"', False),
        ('origin = "digitised"', False),
        ("origin = \"\"", False),
    ],
)
def test_origin_starts_with_a_keyword_as_a_whole_word(
    tmp_path: Path, key: str, accepted: bool
) -> None:
    tree = make_tree(tmp_path)
    edit(tree, "beta", 'origin = "authored (by hand)"', key)
    _, diagnostics = load(tree)
    assert (diagnostics == ()) is accepted
