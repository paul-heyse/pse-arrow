# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""`tk map`: map staged tables to canonical Parquet.

`tk map <id> --phase identity` runs phase 1; `tk map <id>` runs phase 2 (after `tk resolve`).
With no ids, every source that has a mapping. Exit codes: 0 success; 1 a refusal or failure (a
stale or missing input, an invalid mapping, a refused row set); 2 a usage error.
"""

from __future__ import annotations

from typing import Annotated

import typer

from thermo_knowledge import config
from thermo_knowledge.acquire.errors import AcquireError, ManifestError
from thermo_knowledge.canonical.environment import Environment
from thermo_knowledge.canonical.store import CanonicalError
from thermo_knowledge.canonical.writer import WriteError
from thermo_knowledge.declaration import DeclarationError
from thermo_knowledge.mapping import runner
from thermo_knowledge.mapping.spec import SpecError
from thermo_knowledge.mapping.staged import MappingError
from thermo_knowledge.resolve.engine import ResolveError

MAP_HELP = "Map source-faithful tables to canonical Parquet."


def map_command(
    ids: Annotated[
        list[str] | None,
        typer.Argument(help="Source ids; every source with a mapping when none is named."),
    ] = None,
    phase: Annotated[
        str,
        typer.Option(
            "--phase",
            help="`identity` emits source entities and identity assertions (before `tk resolve`); "
            "`records` (the default) emits everything else (after it).",
        ),
    ] = "records",
    force: Annotated[
        bool, typer.Option("--force", help="Map again although the inputs are unchanged.")
    ] = False,
) -> None:
    """Map staged tables to canonical Parquet under .store/canonical/<id>/."""
    if phase not in ("identity", "records"):
        typer.echo(f"error: --phase is `identity` or `records`, not `{phase}`", err=True)
        raise typer.Exit(code=2)
    env = Environment()
    targets = ids or runner.mapping_sources(env)
    if not targets:
        typer.echo("nothing to map: no source has a mapping under mappings/")
        return
    failed: list[str] = []
    for source_id in targets:
        try:
            outcome = (
                runner.run_identity(env, source_id, force=force)
                if phase == "identity"
                else runner.run_records(env, source_id, force=force)
            )
        except SpecError as error:
            for problem in error.problems:
                typer.echo(f"error: {problem}", err=True)
            failed.append(source_id)
        except (
            CanonicalError,
            WriteError,
            MappingError,
            ResolveError,
            DeclarationError,
            ManifestError,
            AcquireError,
            config.ConfigError,
        ) as error:
            typer.echo(f"{'refused':<12} {source_id}  {error}", err=True)
            failed.append(source_id)
        else:
            typer.echo(f"{outcome.status:<12} {source_id}  phase {outcome.phase}")
            width = max((len(name) for name in outcome.tables), default=0)
            for name, rows in outcome.tables.items():
                typer.echo(f"  {name:<{width}}  {rows}")
            if outcome.coverage is not None:
                typer.echo("")
                for line in runner.coverage_lines(outcome.coverage):
                    typer.echo(line)
    if failed:
        typer.echo(
            f"error: {len(failed)} of {len(targets)} source(s) not mapped: {', '.join(failed)}",
            err=True,
        )
        raise typer.Exit(code=1)
