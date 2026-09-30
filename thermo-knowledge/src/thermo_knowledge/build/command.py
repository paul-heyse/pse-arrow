# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""`tk build`, and the build state `tk db status` reports.

`tk build` replaces the canonical schemas of the configured database from the declaration and
the canonical Parquet under `<store>/canonical/` (the resolution result and every source's
records) in one transaction. With no canonical Parquet it builds the empty schema. Exit codes:
0 success; 1 a refusal or failure (stale or conflicting inputs, a constraint the commit
violates), after which the database is as it was; 2 a usage error.
"""

from __future__ import annotations

from pathlib import Path
from typing import Annotated

import adbc_driver_manager
import psycopg
import typer

from thermo_knowledge import config
from thermo_knowledge.build.inputs import discover, discover_qualification
from thermo_knowledge.build.record import read_state
from thermo_knowledge.build.run import BuildResult, build_database, dry_run
from thermo_knowledge.canonical.store import CanonicalError
from thermo_knowledge.declaration import DeclarationError, load_declaration
from thermo_knowledge.generate.command import TREE_OPTION, load_tree
from thermo_knowledge.generate.fingerprint import declaration_fingerprint
from thermo_knowledge.schema_build import read_physical

BUILD_HELP = "Build the database from the generated DDL and canonical Parquet."
_SHORT = 12


def _ids(values: list[str] | None) -> list[str]:
    """Source ids from repeated `--sources`, each possibly a comma-separated list."""
    return [item for value in values or [] for item in value.split(",") if item]


def _report(result: BuildResult) -> list[str]:
    lines = [f"fingerprint {result.fingerprint}"]
    for source in result.sources:
        lines.append(
            f"source      {source.manifest_id}  pin {source.resolved_pin}  "
            f"tree {source.tree_hash[:_SHORT]}  mapping {source.reuse_key[:_SHORT]}"
        )
    width = max((len(name) for name in result.tables), default=0)
    for name, rows in result.tables.items():
        merged = result.merged.get(name)
        note = f"  ({merged} identical duplicates merged)" if merged else ""
        lines.append(f"{name:<{width}}  {rows}{note}")
    return lines


def build_command(
    sources: Annotated[
        list[str] | None,
        typer.Option(
            "--sources",
            help="A source to build (repeat the option, or comma-separate ids); every source "
            "with canonical records when none is named. The resolution result is always loaded.",
        ),
    ] = None,
    dry_run_only: Annotated[
        bool,
        typer.Option(
            "--dry-run",
            help="Take the union and run every consistency check, report the rows per table, "
            "and touch no database.",
        ),
    ] = False,
    tree: Path = TREE_OPTION,
) -> None:
    """Replace the canonical schemas of the database from the declaration and every source's
    canonical Parquet, in one transaction."""
    decl = load_tree(tree)
    try:
        inputs = discover(config.canonical_dir(), _ids(sources))
        fingerprint = declaration_fingerprint(decl, read_physical(tree))
        outputs, skipped = discover_qualification(config.canonical_dir(), inputs, fingerprint)
        inputs = [*inputs, *outputs]
        for item in skipped:
            typer.echo(f"skipped qualification output {item.case}: {item.reason}")
        if outputs or skipped:
            typer.echo(f"qualification outputs: {len(outputs)} loaded, {len(skipped)} skipped")
        if dry_run_only:
            result = dry_run(decl, inputs, tree=tree)
            typer.echo("dry run: no database was touched")
            for line in _report(result):
                typer.echo(line)
            return
        url = config.database_url()
        result = build_database(url, decl, inputs, tree=tree)
    except (CanonicalError, DeclarationError, config.ConfigError) as error:
        typer.echo(f"error: {error}", err=True)
        raise typer.Exit(code=1) from error
    except (psycopg.Error, adbc_driver_manager.Error) as error:
        typer.echo(f"error: cannot build the database: {error}", err=True)
        raise typer.Exit(code=1) from error
    typer.echo(f"built {len(result.sources)} source(s) into {config.database_name(url)}")
    for line in _report(result):
        typer.echo(line)


def status_lines(url: str | None = None) -> list[str]:
    """The fingerprint the database records, whether it matches the current declaration, and
    the sources built: the lines `tk db status` adds."""
    url = url if url is not None else config.database_url()
    state = read_state(url)
    lines = [f"fingerprint: {state.fingerprint or '(none recorded)'}"]
    result = load_declaration()
    if result.declaration is None or result.diagnostics:
        lines.append(f"declaration: refused ({len(result.diagnostics)} diagnostics)")
    else:
        current = declaration_fingerprint(result.declaration, read_physical())
        if state.fingerprint is None:
            verdict = "the database records none"
        elif state.fingerprint == current:
            verdict = "matches the database"
        else:
            verdict = "differs from the database: rebuild with `tk build`"
        lines.append(f"declaration: {current} ({verdict})")
    if state.sources:
        lines.append("sources:")
        lines.extend(
            f"  {s.manifest_id}  pin {s.resolved_pin}  tree {s.tree_hash[:_SHORT]}  "
            f"mapping {s.reuse_key[:_SHORT]}  built {s.built_at:%Y-%m-%d %H:%M:%S%z}"
            for s in state.sources
        )
    else:
        lines.append("sources:     (none built)")
    return lines
