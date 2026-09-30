# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""`tk verify`: run every check of `sql/verify/` against the built database.

Prints a table (check, kind, violations, the first offending identifiers), writes a JSON report
beside the canonical store and exits 1 when any check has violations or cannot run, when a
declared verify invariant has no check file, or when a check file names an invariant the
declaration does not declare as `verify`. A database built from another declaration is refused:
rebuild it first.
"""

from __future__ import annotations

from pathlib import Path
from typing import Annotated

import psycopg
import typer

from thermo_knowledge import config, db
from thermo_knowledge.declaration import DeclarationError
from thermo_knowledge.generate.command import TREE_OPTION, load_tree
from thermo_knowledge.schema_build import compare_fingerprint
from thermo_knowledge.verify.checks import VERIFY_DIR, agreement_problems, load_checks
from thermo_knowledge.verify.run import REPORT_NAME, Report, run_checks, table_lines, write_report

VERIFY_HELP = "Check constraints, invariants and cross-source agreement."


def verify_command(
    report: Annotated[
        Path | None,
        typer.Option(
            "--report", help=f"Where to write the JSON report (default: <store>/{REPORT_NAME})."
        ),
    ] = None,
    tree: Path = TREE_OPTION,
) -> None:
    """Run every check of sql/verify/ against the built database and report the violations."""
    decl = load_tree(tree)
    try:
        url = config.database_url()
        comparison = compare_fingerprint(url, decl, tree=tree)
        if not comparison.matches:
            typer.echo(f"error: {comparison.message}", err=True)
            raise typer.Exit(code=1)
        checks, problems = load_checks(tree / VERIFY_DIR)
        problems.extend(agreement_problems(decl, checks))
        with db.connect(url) as conn:
            results = run_checks(conn, checks)
    except (config.ConfigError, DeclarationError) as error:
        typer.echo(f"error: {error}", err=True)
        raise typer.Exit(code=1) from error
    except psycopg.Error as error:
        typer.echo(f"error: cannot verify the database: {error}", err=True)
        raise typer.Exit(code=1) from error
    outcome = Report(
        config.database_name(url), comparison.recorded, tuple(results), tuple(problems)
    )
    for line in table_lines(outcome):
        typer.echo(line)
    for problem in outcome.problems:
        typer.echo(f"problem: {problem}", err=True)
    target = report if report is not None else config.store_root() / REPORT_NAME
    write_report(outcome, target)
    failing = sum(1 for result in outcome.results if not result.passed)
    typer.echo(
        f"{len(outcome.results)} checks, {failing} failing, {len(outcome.problems)} problem(s); "
        f"report: {target}"
    )
    if not outcome.passed:
        raise typer.Exit(code=1)
