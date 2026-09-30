# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""`tk resolve`: resolve the identity claims of every source that has phase-1 output.

Exit codes: 0 success; 1 a refusal or failure (a stale or missing input, contradicting claims,
an invalid decisions file).
"""

from __future__ import annotations

import hashlib
import json
import shutil
from dataclasses import dataclass
from pathlib import Path
from typing import Annotated

import typer

from thermo_knowledge import config, reuse
from thermo_knowledge.canonical import store
from thermo_knowledge.canonical.environment import Environment
from thermo_knowledge.canonical.provenance import Carriers
from thermo_knowledge.canonical.store import CanonicalError, CanonicalManifest
from thermo_knowledge.declaration import Declaration, DeclarationError
from thermo_knowledge.mapping.claims import IDENTITY_DIR
from thermo_knowledge.resolve import decisions as decisions_module
from thermo_knowledge.resolve import engine, output, structure
from thermo_knowledge.resolve.decisions import DecisionError
from thermo_knowledge.resolve.engine import ResolveError

FORMAT = 2
"""Bump when the rows or rules of resolution change in a way the inputs do not show."""
REPORT_NAME = "report.json"
RESOLUTION_ID = "_resolution"

RESOLVE_HELP = "Resolve identity assertions to canonical entities."


@dataclass(frozen=True)
class ResolveOutcome:
    status: str  # "resolved" | "current"
    lines: list[str]
    tables: dict[str, int]


def phase1_sources(env: Environment) -> dict[str, tuple[Path, CanonicalManifest]]:
    """Every source whose phase-1 output exists (manifest id to directory and manifest)."""
    found: dict[str, tuple[Path, CanonicalManifest]] = {}
    if not env.canonical_dir.is_dir():
        return found
    for entry in sorted(env.canonical_dir.iterdir()):
        directory = entry / IDENTITY_DIR
        if entry.name.startswith((".", "_")) or not (directory / store.MANIFEST_NAME).is_file():
            continue
        manifest = store.read_manifest(directory)
        store.verify_directory(directory, manifest)
        found[entry.name] = (directory, manifest)
    return found


def resolve_all(
    env: Environment, *, force: bool = False, decl: Declaration | None = None
) -> ResolveOutcome:
    """Resolve every source with phase-1 output and write `<canonical>/_resolution/`."""
    decl = decl if decl is not None else env.declaration()
    sources = phase1_sources(env)
    if not sources:
        raise CanonicalError(
            f"no source has phase-1 output under {env.canonical_dir}; "
            "run `tk map <id> --phase identity` first"
        )
    decided, decision_bytes = decisions_module.load(env.decisions_path)
    try:
        key = reuse.stage_key(
            "resolve",
            {
                **{
                    f"identity:{i}": store.manifest_hash(d) for i, (d, _) in sorted(sources.items())
                },
                "decisions": hashlib.sha256(decision_bytes).hexdigest(),
                "declaration": env.fingerprint(decl),
                "format": FORMAT,
            },
        )
    except reuse.ReuseError as error:
        raise ResolveError(str(error)) from error
    destination = env.resolution_dir
    if not force and (destination / store.MANIFEST_NAME).is_file():
        try:
            recorded = store.read_manifest(destination)
        except CanonicalError:
            recorded = None
        if recorded is not None and recorded.reuse_key == key.digest:
            data = json.loads((destination / REPORT_NAME).read_text())
            return ResolveOutcome("current", output.report_table(data), _rows(recorded))

    read = output.read_claims(sources)
    _check_structural_schemes(decl)
    resolution = engine.resolve(
        read.entities,
        decided,
        read.formula_scopes,
        structure.structure_key,
        engine.scheme_kinds(decl),
    )
    carriers = Carriers()
    for info in read.carriers.values():
        carriers.add(info)
    writer = output.write_rows(decl, read, resolution, carriers)
    data = output.report(read, resolution)

    work = store.new_work_directory(env.canonical_dir)
    try:
        records = store.write_tables(work, writer.tables())
        (work / REPORT_NAME).write_text(
            json.dumps(data, indent=2, sort_keys=True, ensure_ascii=False) + "\n"
        )
        manifest = CanonicalManifest(
            schema=store.MANIFEST_SCHEMA,
            source_id=RESOLUTION_ID,
            phase="resolution",
            reuse_key=key.digest,
            inputs=key.inputs,
            tables=records,
            summary={
                "source_entities": len(resolution.entities),
                "species": len(resolution.species),
            },
        )
        store.write_manifest(work, manifest)
        store.install_directory(work, destination)
    except BaseException:
        shutil.rmtree(work, ignore_errors=True)
        raise
    return ResolveOutcome("resolved", output.report_table(data), _rows(manifest))


def _rows(manifest: CanonicalManifest) -> dict[str, int]:
    return {name: record.rows for name, record in sorted(manifest.tables.items())}


def _check_structural_schemes(decl: Declaration) -> None:
    """A declared structural naming scheme the resolver cannot compute a key from is refused:
    its assertions would silently not take part in rule 2."""
    missing = sorted(engine.scheme_kinds(decl).structural - set(structure.STRUCTURAL_SCHEMES))
    if missing:
        raise ResolveError(
            f"the declaration marks {', '.join(missing)} as structural naming schemes and "
            "`resolve/structure.py` cannot compute an InChIKey from them"
        )


def resolve_command(
    force: Annotated[
        bool, typer.Option("--force", help="Resolve again although the inputs are unchanged.")
    ] = False,
) -> None:
    """Resolve the identity claims of every source with phase-1 output, under
    .store/canonical/_resolution/."""
    try:
        outcome = resolve_all(Environment(), force=force)
    except (
        CanonicalError,
        ResolveError,
        DecisionError,
        DeclarationError,
        config.ConfigError,
    ) as error:
        typer.echo(f"error: {error}", err=True)
        raise typer.Exit(code=1) from error
    typer.echo(f"{outcome.status}: {Environment().resolution_dir}")
    for line in outcome.lines:
        typer.echo(line)
    typer.echo("")
    width = max(len(name) for name in outcome.tables) if outcome.tables else 0
    for name, rows in outcome.tables.items():
        typer.echo(f"{name:<{width}}  {rows}")
