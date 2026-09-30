# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""`tk qualify`: run the qualification cases of `qualification/` against the built database.

For each case: select the subjects, evaluate the form from the database through the reference
evaluator, run the oracle harness, compare point by point, record one `qualification_run` and
print a table. The Parquet output lives under `<store>/canonical/_qualification/<case>/` and is
loaded into the live database at once. Exit codes: 0 every case passed (or was current); 1 a case
failed or was blocked, a case the declaration refuses, or a database built from another
declaration; 2 a usage error.
"""

from __future__ import annotations

import json
from collections.abc import Sequence
from pathlib import Path
from typing import Annotated

import psycopg
import typer

from thermo_knowledge import config, db
from thermo_knowledge.build.database import DatabaseRefusedError
from thermo_knowledge.canonical.store import CanonicalError
from thermo_knowledge.declaration import DeclarationError
from thermo_knowledge.generate.command import TREE_OPTION, load_tree
from thermo_knowledge.qualify.case import CASES_DIR, CaseError, case_names, load_case
from thermo_knowledge.qualify.run import PASSED, CaseOutcome, Context, run_case, table_lines
from thermo_knowledge.qualify.source import SourceError

QUALIFY_HELP = "Qualify forms against oracle harnesses."
REPORT_NAME = "qualify-report.json"


def blocked_by_reason(outcomes: Sequence[CaseOutcome]) -> dict[str, list[str]]:
    """The blocked cases grouped by their typed reason, reasons in name order."""
    grouped: dict[str, list[str]] = {}
    for done in outcomes:
        if done.blocked_reason is not None:
            grouped.setdefault(done.blocked_reason, []).append(done.case)
    return {reason: sorted(names) for reason, names in sorted(grouped.items())}


def qualify_command(
    cases: Annotated[
        list[str] | None,
        typer.Argument(
            help="Case names (file stems under qualification/); every case when none is named."
        ),
    ] = None,
    force: Annotated[
        bool,
        typer.Option("--force", help="Evaluate again although the case's reuse key is unchanged."),
    ] = False,
    report: Annotated[
        Path | None,
        typer.Option(
            "--report", help=f"Where to write the JSON report (default: <store>/{REPORT_NAME})."
        ),
    ] = None,
    tree: Path = TREE_OPTION,
) -> None:
    """Evaluate each case, compare it with its harness and record the run."""
    decl = load_tree(tree)
    directory = tree / CASES_DIR
    available = case_names(directory)
    wanted = cases or available
    unknown = sorted(set(wanted) - set(available))
    if unknown:
        typer.echo(f"error: no such case: {', '.join(unknown)} (under {directory})", err=True)
        raise typer.Exit(code=2)
    if not wanted:
        typer.echo(f"nothing to qualify: no case under {directory}")
        return
    outcomes: list[CaseOutcome] = []
    problems = 0
    try:
        url = config.database_url()
        context = Context(decl, url, config.canonical_dir(), tree=tree, force=force)
        with db.connect(url) as conn:
            for name in wanted:
                try:
                    done = run_case(context, conn, load_case(directory / f"{name}.toml"))
                except CaseError as error:
                    for problem in error.problems:
                        typer.echo(f"error: {error.path}: {problem}", err=True)
                    problems += 1
                    continue
                outcomes.append(done)
                typer.echo(f"{done.status:<8} {name}")
                for line in table_lines(done.report):
                    typer.echo(line)
    except (
        config.ConfigError,
        DeclarationError,
        SourceError,
        CanonicalError,
        DatabaseRefusedError,
    ) as error:
        typer.echo(f"error: {error}", err=True)
        raise typer.Exit(code=1) from error
    except psycopg.Error as error:
        typer.echo(f"error: cannot qualify against the database: {error}", err=True)
        raise typer.Exit(code=1) from error
    target = report if report is not None else config.store_root() / REPORT_NAME
    target.parent.mkdir(parents=True, exist_ok=True)
    blocked = blocked_by_reason(outcomes)
    target.write_text(
        json.dumps(
            {"cases": [o.report for o in outcomes], "blocked_by_reason": blocked},
            indent=2,
            sort_keys=True,
            allow_nan=False,
        )
        + "\n",
        encoding="utf-8",
    )
    for reason, names in blocked.items():
        typer.echo(f"blocked ({reason}): {len(names)} case(s): {', '.join(names)}")
    bad = [o for o in outcomes if o.outcome != PASSED]
    typer.echo(
        f"{len(outcomes)} case(s), {len(bad)} not passed, {problems} refused; report: {target}"
    )
    if bad or problems:
        raise typer.Exit(code=1)
