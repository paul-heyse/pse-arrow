# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Helpers shared by the survey tests: the invented fixture tree and small file edits."""

from __future__ import annotations

import shutil
from pathlib import Path

from thermo_knowledge.survey_index import Code, RefIndex, SurveyDiagnostic
from thermo_knowledge.survey_index.dispositions import DeclaredNames

FIXTURES = Path(__file__).parent / "fixtures" / "survey"
CALCULATIONS = frozenset({"flash_tp"})
DECLARED = DeclaredNames(
    kinds=frozenset({"species", "parameter_set"}),
    relations=frozenset({"uses"}),
    forms=frozenset({"nasa7"}),
    enums=frozenset({"basis"}),
)
REFS = RefIndex(
    declared=DECLARED,
    alignment_items=frozenset({1, 2, 3}),
    review_findings=frozenset({"F01", "F14"}),
)

# The constructs of the fixture that need a disposition (section 4 of docs/survey.md).
ALPHA_NEEDING = ("exact but lossy", "close record", "narrower record", "unmapped record")
BETA_NEEDING = ("broad table",)


def make_tree(root: Path) -> Path:
    """A copy of the fixture tree under `root`, with its design review beside it (as the real tree
    has it under the repository root); returns the tree."""
    shutil.copytree(FIXTURES / "tree", root / "tree")
    shutil.copytree(FIXTURES / "docs", root / "docs")
    return root / "tree"


def edit(tree: Path, name: str, old: str, new: str) -> None:
    """Replace `old` by `new` in `survey/<name>.toml`; `old` must occur exactly once."""
    path = tree / "survey" / f"{name}.toml"
    text = path.read_text(encoding="utf-8")
    assert text.count(old) == 1, (old, text.count(old))
    path.write_text(text.replace(old, new), encoding="utf-8")


def write_survey(tree: Path, name: str, text: str) -> None:
    (tree / "survey" / f"{name}.toml").write_text(text, encoding="utf-8")


def write_dispositions(tree: Path, source: str, text: str) -> None:
    directory = tree / "survey" / "dispositions"
    directory.mkdir(parents=True, exist_ok=True)
    (directory / f"{source}.toml").write_text(text, encoding="utf-8")


def entry(construct: str, disposition: str, ref: str, reason: str = "A reason.") -> str:
    return (
        "[[disposition]]\n"
        f'construct = "{construct}"\n'
        f'disposition = "{disposition}"\n'
        f'ref = "{ref}"\n'
        f'reason = "{reason}"\n'
    )


def codes(diagnostics: tuple[SurveyDiagnostic, ...]) -> list[tuple[str, str, str, Code]]:
    """(table, record, key, code) of each diagnostic."""
    return [(d.table, d.record, d.key, d.code) for d in diagnostics]


def complete_dispositions(tree: Path) -> None:
    """Dispositions for every construct of the fixture that needs one."""
    write_dispositions(
        tree,
        "alpha",
        entry("exact but lossy", "mapping_loss", "mechanism:unit assumption")
        + entry("close record", "model_change", "alignment-notes #2")
        + entry("narrower record", "held_by_model", "kind:parameter_set")
        + entry("unmapped record", "out_of_scope", ""),
    )
    write_dispositions(tree, "beta", entry("broad table", "model_change", "unscheduled"))
