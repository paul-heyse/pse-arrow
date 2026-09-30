# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""`tk survey`: validate the survey records and their dispositions, and write the residue report.

Exit codes: 0 success; 1 a diagnostic, or (with `--strict`) a construct without a disposition.
"""

from __future__ import annotations

import os
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Annotated

import typer

from thermo_knowledge import config
from thermo_knowledge.declaration import Declaration, load_declaration
from thermo_knowledge.survey_index.diagnostics import Code, SurveyDiagnostic
from thermo_knowledge.survey_index.dispositions import (
    DispositionSet,
    build_ref_index,
    load_dispositions,
)
from thermo_knowledge.survey_index.loader import SurveySet, load_surveys, survey_files
from thermo_knowledge.survey_index.report import (
    REPORT_RELATIVE,
    SourceResidue,
    render_report,
    residue,
    summary_lines,
    undispositioned_total,
)

HELP = "Validate the survey records and dispositions; write the residue report."

CALCULATION_KIND = "calculation"
"""The declared kind whose entities are the calculation vocabulary of a capability."""


@dataclass(frozen=True)
class Outcome:
    """Everything one run established."""

    surveys: SurveySet
    dispositions: DispositionSet
    rows: list[SourceResidue]
    diagnostics: tuple[SurveyDiagnostic, ...]


def calculation_vocabulary(declaration: Declaration | None) -> frozenset[str] | None:
    """The names of the declared entities of kind `calculation` (or a refinement of it);
    `None` when there is no declaration or it declares no such kind."""
    if declaration is None or CALCULATION_KIND not in declaration.kinds:
        return None
    return frozenset(
        entity.name
        for entity in declaration.entities
        if entity.kind in declaration.kinds and declaration.is_a(entity.kind, CALCULATION_KIND)
    )


def evaluate(
    tree: Path,
    declaration: Declaration | None,
    declaration_diagnostics: tuple[SurveyDiagnostic, ...] = (),
) -> Outcome:
    """Load and validate the surveys and dispositions of `tree` against `declaration`, and count
    what has no disposition. `declaration_diagnostics` are carried into the outcome."""
    surveys = load_surveys(tree, calculations=calculation_vocabulary(declaration))
    index, input_diagnostics = build_ref_index(tree, declaration)
    dispositions = load_dispositions(tree, surveys.surveys, index)
    diagnostics = (
        *declaration_diagnostics,
        *input_diagnostics,
        *surveys.diagnostics,
        *dispositions.diagnostics,
    )
    return Outcome(
        surveys=surveys,
        dispositions=dispositions,
        rows=residue(surveys.surveys, dispositions.by_source),
        diagnostics=diagnostics,
    )


def report_text(tree: Path, outcome: Outcome) -> str:
    """The residue report of `outcome`."""
    return render_report(
        outcome.rows,
        survey_files=len(survey_files(tree)),
        disposition_files=len(outcome.dispositions.files),
    )


def write_report(tree: Path, text: str) -> Path:
    """Write the report atomically: a temporary file in the same directory, then a rename."""
    path = tree / REPORT_RELATIVE
    path.parent.mkdir(parents=True, exist_ok=True)
    handle, temporary = tempfile.mkstemp(dir=path.parent, prefix=".residue-report.", suffix=".tmp")
    try:
        with os.fdopen(handle, "w", encoding="utf-8", newline="\n") as stream:
            stream.write(text)
        os.chmod(temporary, 0o644)  # mkstemp creates the file private to its owner
        os.replace(temporary, path)
    except BaseException:
        Path(temporary).unlink(missing_ok=True)
        raise
    return path


def load_tree_declaration(tree: Path) -> tuple[Declaration | None, tuple[SurveyDiagnostic, ...]]:
    """The declaration of `tree`, or a diagnostic saying why it is unavailable."""
    result = load_declaration(tree / "model", tree / "forms")
    if result.declaration is not None and not result.diagnostics:
        return result.declaration, ()
    shown = "; ".join(str(item) for item in result.diagnostics[:3])
    return None, (
        SurveyDiagnostic(
            file="model/",
            code=Code.DECLARATION_UNAVAILABLE,
            message=(
                f"the declaration was refused ({len(result.diagnostics)} diagnostics; first: "
                f"{shown}); calculation keys and model refs cannot be checked"
            ),
        ),
    )


def survey_command(
    check: Annotated[
        bool,
        typer.Option(
            "--check",
            help="Validate the surveys and dispositions; exit 1 on any diagnostic. Always done, "
            "so the flag only names the mode when --report is absent.",
        ),
    ] = False,
    report: Annotated[
        bool,
        typer.Option(
            "--report",
            help="Write survey/residue-report.md and print a summary; needs no diagnostics.",
        ),
    ] = False,
    strict: Annotated[
        bool,
        typer.Option("--strict", help="Exit 1 while any construct has no disposition."),
    ] = False,
    tree: Annotated[
        Path,
        typer.Option("--tree", help="The tree whose survey/ is read.", file_okay=False),
    ] = config.TREE_DIR,
) -> None:
    """Validate the survey records and their dispositions; with --report write the residue
    report, which shows every construct that needs a disposition and has none."""
    declaration, declaration_diagnostics = load_tree_declaration(tree)
    outcome = evaluate(tree, declaration, declaration_diagnostics)
    for diagnostic in outcome.diagnostics:
        typer.echo(str(diagnostic), err=True)
    if outcome.diagnostics:
        typer.echo(f"error: {len(outcome.diagnostics)} diagnostics", err=True)
        raise typer.Exit(code=1)
    if report:
        path = write_report(tree, report_text(tree, outcome))
        typer.echo(f"wrote {path.relative_to(tree).as_posix()}")
    else:
        constructs = sum(row.total for row in outcome.rows)
        typer.echo(
            f"surveys and dispositions are valid: {len(outcome.rows)} sources, "
            f"{constructs} constructs"
        )
    for line in summary_lines(outcome.rows):
        typer.echo(line)
    if strict and undispositioned_total(outcome.rows):
        typer.echo(
            f"error: {undispositioned_total(outcome.rows)} constructs have no disposition",
            err=True,
        )
        raise typer.Exit(code=1)
