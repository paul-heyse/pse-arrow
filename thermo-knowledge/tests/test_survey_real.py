# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The committed survey records and dispositions conform to docs/survey.md."""

from __future__ import annotations

from thermo_knowledge import config
from thermo_knowledge.survey_index.command import evaluate, load_tree_declaration
from thermo_knowledge.survey_index.loader import survey_files


def test_the_real_survey_directory_loads_with_no_diagnostics() -> None:
    declaration, declaration_diagnostics = load_tree_declaration(config.TREE_DIR)
    outcome = evaluate(config.TREE_DIR, declaration, declaration_diagnostics)
    assert [str(item) for item in outcome.diagnostics] == []
    assert sorted(outcome.surveys.surveys) == [path.stem for path in survey_files(config.TREE_DIR)]
