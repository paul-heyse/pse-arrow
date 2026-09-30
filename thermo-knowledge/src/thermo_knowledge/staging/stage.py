# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""`tk read`: turn an acquired source into source-faithful staged Parquet.

```
<staged>/<id>/<resolved-pin>/manifest.json
<staged>/<id>/<resolved-pin>/<table>.parquet
```

The staged directory is built in a temporary sibling and renamed into place when complete; a
failed run leaves nothing that looks staged. Reading refuses a source that is not in the lock,
whose store does not verify against the lock (the acquisition stage's own check), that declares
no payload or no reader, whose reader emits a table without a declared schema, a batch that
does not match its schema, a duplicate `_locator` or a payload file nothing accounts for.
"""

from __future__ import annotations

import shutil
import tempfile
from dataclasses import dataclass, field
from enum import StrEnum
from pathlib import Path

from thermo_knowledge import config
from thermo_knowledge.acquire import runner, store
from thermo_knowledge.acquire.lock import LockEntry
from thermo_knowledge.acquire.manifest import Manifest
from thermo_knowledge.acquire.runtime import Context
from thermo_knowledge.staging import manifest as staged_manifest
from thermo_knowledge.staging import payload, reader, side
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.manifest import ReaderRecord, StagedManifest
from thermo_knowledge.staging.reader import ResolvedReader, Resolver
from thermo_knowledge.staging.writer import Writer, WriterResult


class StageState(StrEnum):
    NOT_STAGED = "not staged"
    CURRENT = "current"
    STALE = "stale"


@dataclass(frozen=True)
class StageContext:
    """Where a read run finds its inputs and puts its outputs, and how it finds readers."""

    acquire: Context = field(default_factory=Context)
    staged_dir: Path = field(default_factory=config.staged_dir)
    resolver: Resolver = reader.default_resolver
    side_command: side.Command = side.tk_env_command


@dataclass(frozen=True)
class ReadOutcome:
    source_id: str
    status: str  # "read" | "current"
    message: str


def staged_path(ctx: StageContext, source_id: str, pin: str) -> Path:
    return ctx.staged_dir / source_id / pin


def tree_path(ctx: StageContext, manifest: Manifest, entry: LockEntry) -> Path:
    """The directory of the acquired files."""
    if entry.kind == "local":
        if entry.path is None:
            raise StagingError(f"{manifest.id}: the lock entry of a local source has no path")
        return ctx.acquire.repo_root / entry.path
    if entry.pin is None:
        raise StagingError(f"{manifest.id}: the lock entry has no pin")
    return store.pin_dir(ctx.acquire.raw_dir, manifest.id, entry.pin) / store.TREE_DIR_NAME


def tree_identity(manifest: Manifest, entry: LockEntry) -> str:
    """What identifies the acquired content: the lock's tree hash (for a local checkout, its
    resolved revision)."""
    identity = entry.tree_hash or entry.resolved or entry.pin
    if identity is None:
        raise StagingError(f"{manifest.id}: the lock records neither a tree hash nor a revision")
    return identity


def _acquired_entry(manifest: Manifest, entries: dict[str, LockEntry]) -> LockEntry:
    entry = entries.get(manifest.id)
    if entry is None:
        raise StagingError(
            f"{manifest.id}: not in sources.lock; run `tk acquire {manifest.id}` first"
        )
    if manifest.kind == "none" or entry.kind == "none" or entry.pin is None:
        raise StagingError(f"{manifest.id}: nothing was acquired ({entry.reason or manifest.kind})")
    return entry


def stage_state(
    ctx: StageContext,
    manifest: Manifest,
    entry: LockEntry,
    resolved: ResolvedReader,
) -> tuple[StageState, str]:
    """Whether the staged directory for the lock's pin was made from the current inputs."""
    assert entry.pin is not None
    directory = staged_path(ctx, manifest.id, entry.pin)
    if not (directory / staged_manifest.MANIFEST_NAME).is_file():
        return StageState.NOT_STAGED, ""
    key, _ = reader.reuse_key(
        source_id=manifest.id, tree_identity=tree_identity(manifest, entry), reader=resolved
    )
    try:
        recorded = staged_manifest.read(directory)
    except StagingError as error:
        return StageState.STALE, str(error)
    if recorded.reuse_key == key:
        return StageState.CURRENT, ""
    return StageState.STALE, "the recorded reuse key differs from the current inputs"


def require_reader(ctx: StageContext, manifest: Manifest) -> ResolvedReader:
    if manifest.payload.reader is None:
        raise StagingError(f"{manifest.id}: the manifest declares no [payload] reader")
    resolved = ctx.resolver(manifest)
    if resolved is None:
        raise StagingError(
            f"{manifest.id}: no reader {manifest.payload.environment}:{manifest.payload.reader} "
            "exists"
        )
    return resolved


def read_source(
    ctx: StageContext,
    manifest: Manifest,
    entries: dict[str, LockEntry],
    *,
    force: bool = False,
) -> ReadOutcome:
    """Stage one source; raises `StagingError` for every refusal."""
    entry = _acquired_entry(manifest, entries)
    resolved = require_reader(ctx, manifest)
    if not manifest.payload.include:
        raise StagingError(f"{manifest.id}: the manifest declares no [payload] include globs")
    assert entry.pin is not None
    key, source_digest = reader.reuse_key(
        source_id=manifest.id, tree_identity=tree_identity(manifest, entry), reader=resolved
    )
    destination = staged_path(ctx, manifest.id, entry.pin)
    if not force:
        state, _ = stage_state(ctx, manifest, entry, resolved)
        if state is StageState.CURRENT:
            return ReadOutcome(manifest.id, "current", f"reuse key {key[:12]} matches")

    verification = runner.check_sources(ctx.acquire, [manifest], entries)[0]
    if not verification.ok:
        problems = "\n  ".join(verification.problems)
        raise StagingError(
            f"{manifest.id}: the store does not verify against sources.lock:\n  {problems}"
        )
    tree = tree_path(ctx, manifest, entry)
    files = payload.payload_files(tree, manifest.payload.include, manifest.payload.exclude)
    if not files:
        raise StagingError(f"{manifest.id}: the payload globs match no file under {tree}")

    source_root = destination.parent
    source_root.mkdir(parents=True, exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix=".partial-", dir=source_root))
    try:
        result = _run_reader(ctx, resolved, tree, work, files)
        staged_manifest.write(
            work,
            StagedManifest(
                schema=staged_manifest.MANIFEST_SCHEMA,
                source_id=manifest.id,
                pin=entry.pin,
                reader=ReaderRecord(
                    name=resolved.name,
                    version=resolved.version,
                    environment=resolved.environment,
                    source_hash=source_digest,
                ),
                reuse_key=key,
                tree_hash=tree_identity(manifest, entry),
                tables=result.tables,
                payload=result.payload,
            ),
        )
        _install(work, destination)
    except BaseException:
        shutil.rmtree(work, ignore_errors=True)
        raise
    rows = sum(table.rows for table in result.tables.values())
    return ReadOutcome(
        manifest.id,
        "read",
        f"{len(result.tables)} tables, {rows} rows, {len(result.payload)} payload files",
    )


def _run_reader(
    ctx: StageContext, resolved: ResolvedReader, tree: Path, work: Path, files: list[str]
) -> WriterResult:
    if resolved.side:
        with tempfile.TemporaryDirectory(prefix="tk-side-", dir=work.parent) as scratch:
            out = Path(scratch)
            described = side.run_side(resolved, tree, out, command=ctx.side_command)
            schemas = side.declared_schemas(described)
            writer = Writer(work, schemas, files)
            try:
                side.feed(described, out, writer, schemas)
                return writer.finish()
            except BaseException:
                writer.abort()
                raise
    schemas = reader.module_tables(resolved)
    read = reader.module_read(resolved)
    writer = Writer(work, schemas, files)
    try:
        read(tree, writer)
        return writer.finish()
    except BaseException:
        writer.abort()
        raise


def _install(work: Path, destination: Path) -> None:
    """Rename the completed `work` into place, replacing an earlier staging of the same pin."""
    if destination.exists():
        retired = destination.with_name(f".retired-{work.name}")
        destination.rename(retired)
        try:
            work.rename(destination)
        except BaseException:
            retired.rename(destination)
            raise
        shutil.rmtree(retired, ignore_errors=True)
    else:
        work.rename(destination)


def readable_sources(
    ctx: StageContext, manifests: list[Manifest], entries: dict[str, LockEntry]
) -> list[Manifest]:
    """Sources with a registered reader that are present in the store."""
    chosen = []
    for manifest in manifests:
        state, _ = runner.store_state(ctx.acquire, manifest, entries)
        if state is not runner.State.PRESENT or manifest.payload.reader is None:
            continue
        if ctx.resolver(manifest) is not None:
            chosen.append(manifest)
    return chosen
