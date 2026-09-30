# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""`tk acquire`: acquire declared sources, check the store against the lock, list states.

Exit codes: 0 success; 1 a refusal, failure or difference (a failed acquisition, a manifest
error, a `--check` difference); 2 a usage error (an unknown source id, `--check` with `--list`).
"""

from __future__ import annotations

from pathlib import Path
from typing import Annotated

import typer

from thermo_knowledge.acquire import runner
from thermo_knowledge.acquire.errors import AcquireError, ManifestError, UsageError
from thermo_knowledge.acquire.lock import read_lock
from thermo_knowledge.acquire.manifest import default_lock_path, default_sources_dir, load_sources
from thermo_knowledge.acquire.runtime import Context

HELP = "Acquire declared sources into the raw store and record sources.lock."


def _error(message: str, code: int = 1) -> typer.Exit:
    typer.echo(f"error: {message}", err=True)
    return typer.Exit(code=code)


def acquire_command(
    ids: Annotated[
        list[str] | None,
        typer.Argument(help="Source ids; all declared sources when none is named."),
    ] = None,
    check: Annotated[
        bool,
        typer.Option(
            "--check",
            help="Offline: recompute each tree hash from the store and compare it with the lock.",
        ),
    ] = False,
    list_: Annotated[
        bool,
        typer.Option("--list", help="One line per source: tier, kind, pin, store state."),
    ] = False,
    sources: Annotated[
        Path | None, typer.Option("--sources", help="Sources directory (default: the tree's).")
    ] = None,
    lock: Annotated[
        Path | None, typer.Option("--lock", help="Lock file (default: the tree's sources.lock).")
    ] = None,
) -> None:
    """Acquire the named sources (all when none is named) not already in the store at the
    declared pin, and update the lock; `--check` and `--list` are offline."""
    execute(ids=ids or [], check=check, list_=list_, sources=sources, lock=lock, ctx=Context())


def execute(
    *,
    ids: list[str],
    check: bool,
    list_: bool,
    sources: Path | None,
    lock: Path | None,
    ctx: Context,
) -> None:
    """Run one form of the command with explicit effects; raises `typer.Exit` on failure."""
    if check and list_:
        raise _error("--check and --list cannot be combined", 2)
    lock_path = lock or default_lock_path()
    try:
        manifests = runner.select(load_sources(sources or default_sources_dir()), ids)
        if list_:
            entries = read_lock(lock_path)
            for row in runner.list_sources(ctx, manifests, entries):
                typer.echo(row.format())
            return
        if check:
            entries = read_lock(lock_path)
            results = runner.check_sources(ctx, manifests, entries)
            for result in results:
                for line in result.lines():
                    typer.echo(line)
            failed = [result.source_id for result in results if not result.ok]
            if failed:
                raise _error(
                    f"{len(failed)} of {len(results)} source(s) differ: {', '.join(failed)}"
                )
            return
        acquirer = runner.Acquirer(ctx, lock_path)
        outcomes = acquirer.run(manifests)
    except ManifestError as error:
        for problem in error.problems:
            typer.echo(f"error: {problem}", err=True)
        raise typer.Exit(code=1) from error
    except UsageError as error:
        raise _error(str(error), 2) from error
    except AcquireError as error:
        raise _error(str(error)) from error
    for outcome in outcomes:
        typer.echo(outcome.format(), err=not outcome.ok)
    failed = [outcome.source_id for outcome in outcomes if not outcome.ok]
    if failed:
        raise _error(
            f"{len(failed)} of {len(outcomes)} source(s) not acquired: {', '.join(failed)}"
        )
