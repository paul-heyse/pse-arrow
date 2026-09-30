# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""`tk read` and `tk load-src`.

Exit codes: 0 success; 1 a refusal or failure (a manifest error, an unverified store, a reader
or loader refusal); 2 a usage error (an unknown source id, `--list` with `--force`). One failing
source does not stop the others.
"""

from __future__ import annotations

from pathlib import Path
from typing import Annotated

import psycopg
import typer

from thermo_knowledge import config
from thermo_knowledge.acquire import runner
from thermo_knowledge.acquire.errors import AcquireError, ManifestError
from thermo_knowledge.acquire.errors import UsageError as AcquireUsageError
from thermo_knowledge.acquire.lock import LockEntry, read_lock
from thermo_knowledge.acquire.manifest import (
    Manifest,
    default_lock_path,
    default_sources_dir,
    load_sources,
)
from thermo_knowledge.staging import load, stage
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.stage import StageContext, StageState

READ_HELP = "Read raw sources into source-faithful staged Parquet."
LOAD_HELP = "Load staged Parquet into the src_<id> schemas."


def _error(message: str, code: int = 1) -> typer.Exit:
    typer.echo(f"error: {message}", err=True)
    return typer.Exit(code=code)


def _select(
    sources: Path | None, lock: Path | None, ids: list[str]
) -> tuple[dict[str, Manifest], list[Manifest], dict[str, LockEntry]]:
    manifests = load_sources(sources or default_sources_dir())
    entries = read_lock(lock or default_lock_path())
    return manifests, runner.select(manifests, ids), entries


def read_command(
    ids: Annotated[
        list[str] | None,
        typer.Argument(
            help="Source ids; every source with a reader that is in the store when none is named."
        ),
    ] = None,
    force: Annotated[
        bool, typer.Option("--force", help="Read again although the stage is current.")
    ] = False,
    list_: Annotated[
        bool,
        typer.Option(
            "--list",
            help="Per source: its reader and the stage state (not staged, current, stale).",
        ),
    ] = False,
    sources: Annotated[
        Path | None, typer.Option("--sources", help="Sources directory (default: the tree's).")
    ] = None,
    lock: Annotated[
        Path | None, typer.Option("--lock", help="Lock file (default: the tree's sources.lock).")
    ] = None,
) -> None:
    """Turn acquired sources into source-faithful Parquet under .store/staged/<id>/<pin>/."""
    execute_read(
        ids=ids or [], force=force, list_=list_, sources=sources, lock=lock, ctx=StageContext()
    )


def execute_read(
    *,
    ids: list[str],
    force: bool,
    list_: bool,
    sources: Path | None,
    lock: Path | None,
    ctx: StageContext,
) -> None:
    """Run `tk read` with explicit effects; raises `typer.Exit` on failure."""
    if list_ and force:
        raise _error("--list and --force cannot be combined", 2)
    try:
        _, selected, entries = _select(sources, lock, ids)
        if list_:
            for manifest in selected:
                typer.echo(_list_row(ctx, manifest, entries))
            return
        targets = selected if ids else stage.readable_sources(ctx, selected, entries)
    except ManifestError as error:
        for problem in error.problems:
            typer.echo(f"error: {problem}", err=True)
        raise typer.Exit(code=1) from error
    except AcquireUsageError as error:
        raise _error(str(error), 2) from error
    except (AcquireError, StagingError) as error:
        raise _error(str(error)) from error
    if not targets:
        typer.echo("nothing to read: no source with a reader is in the store")
        return
    failed: list[str] = []
    for manifest in targets:
        try:
            outcome = stage.read_source(ctx, manifest, entries, force=force)
        except (StagingError, AcquireError) as error:
            typer.echo(f"{'refused':<12} {manifest.id}  {error}", err=True)
            failed.append(manifest.id)
        else:
            typer.echo(f"{outcome.status:<12} {outcome.source_id}  {outcome.message}")
    if failed:
        raise _error(f"{len(failed)} of {len(targets)} source(s) not read: {', '.join(failed)}")


def _list_row(ctx: StageContext, manifest: Manifest, entries: dict[str, LockEntry]) -> str:
    try:
        resolved = ctx.resolver(manifest)
    except StagingError as error:
        return f"{manifest.id:<24} {'error':<24} {error}"
    reader_text = resolved.describe() if resolved else "no reader"
    state, detail = runner.store_state(ctx.acquire, manifest, entries)
    if state is not runner.State.PRESENT:
        text = f"{state} ({detail})" if detail else str(state)
        return f"{manifest.id:<24} {reader_text:<24} {text}"
    if resolved is None:
        return f"{manifest.id:<24} {reader_text:<24} not staged"
    stage_result, why = stage.stage_state(ctx, manifest, entries[manifest.id], resolved)
    text = f"{stage_result} ({why})" if why else str(stage_result)
    return f"{manifest.id:<24} {reader_text:<24} {text}"


def load_src_command(
    ids: Annotated[
        list[str] | None,
        typer.Argument(
            help="Source ids; every source with current staged data when none is named."
        ),
    ] = None,
    sources: Annotated[
        Path | None, typer.Option("--sources", help="Sources directory (default: the tree's).")
    ] = None,
    lock: Annotated[
        Path | None, typer.Option("--lock", help="Lock file (default: the tree's sources.lock).")
    ] = None,
) -> None:
    """Load staged Parquet into schema src_<id> of the configured database."""
    execute_load(ids=ids or [], sources=sources, lock=lock, ctx=StageContext())


def execute_load(
    *,
    ids: list[str],
    sources: Path | None,
    lock: Path | None,
    ctx: StageContext,
    url: str | None = None,
) -> None:
    """Run `tk load-src` with explicit effects; raises `typer.Exit` on failure."""
    try:
        target = url if url is not None else config.database_url()
        _, selected, entries = _select(sources, lock, ids)
        if not ids:
            selected = [m for m in selected if _has_current_stage(ctx, m, entries)]
    except ManifestError as error:
        for problem in error.problems:
            typer.echo(f"error: {problem}", err=True)
        raise typer.Exit(code=1) from error
    except AcquireUsageError as error:
        raise _error(str(error), 2) from error
    except (AcquireError, StagingError, config.ConfigError) as error:
        raise _error(str(error)) from error
    if not selected:
        typer.echo("nothing to load: no source has current staged data")
        return
    failed: list[str] = []
    for manifest in selected:
        try:
            outcome = _load_one(ctx, manifest, entries, target)
        except (StagingError, AcquireError, config.ConfigError, psycopg.Error, OSError) as error:
            typer.echo(f"{'refused':<12} {manifest.id}  {error}", err=True)
            failed.append(manifest.id)
        else:
            rows = sum(outcome.tables.values())
            typer.echo(
                f"{'loaded':<12} {manifest.id}  schema {outcome.schema}: "
                f"{len(outcome.tables)} tables, {rows} rows"
            )
    if failed:
        raise _error(f"{len(failed)} of {len(selected)} source(s) not loaded: {', '.join(failed)}")


def _has_current_stage(
    ctx: StageContext, manifest: Manifest, entries: dict[str, LockEntry]
) -> bool:
    entry = entries.get(manifest.id)
    if entry is None or entry.pin is None or manifest.payload.reader is None:
        return False
    try:
        resolved = ctx.resolver(manifest)
        if resolved is None:
            return False
        state, _ = stage.stage_state(ctx, manifest, entry, resolved)
    except StagingError:
        return False
    return state is StageState.CURRENT


def _load_one(
    ctx: StageContext, manifest: Manifest, entries: dict[str, LockEntry], url: str
) -> load.LoadOutcome:
    entry = entries.get(manifest.id)
    if entry is None or entry.pin is None:
        raise StagingError(f"{manifest.id}: not in sources.lock")
    resolved = stage.require_reader(ctx, manifest)
    state, why = stage.stage_state(ctx, manifest, entry, resolved)
    if state is StageState.NOT_STAGED:
        raise StagingError(f"{manifest.id}: not staged; run `tk read {manifest.id}`")
    if state is StageState.STALE:
        raise StagingError(
            f"{manifest.id}: the staged data is stale ({why}); run `tk read {manifest.id}`"
        )
    directory = stage.staged_path(ctx, manifest.id, entry.pin)
    return load.load_staged(url, directory, f"src_{manifest.id}")
