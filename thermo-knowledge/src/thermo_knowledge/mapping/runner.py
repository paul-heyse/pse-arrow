# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The two phases of `tk map` (pipeline section 2).

Phase 1 (`run_identity`) runs the mapping's `identities(ctx)` over the staged rows and writes the
claims to `<canonical>/<id>/_identity/`. Phase 2 (`run_records`) refuses to run unless the
phase-1 output is current for the mapping and the resolution result exists and was made from
exactly that phase-1 output; it then runs `records(ctx)`, computes the coverage and writes
`<canonical>/<id>/`. Each phase is skipped when its reuse key is unchanged.

The reuse key of phase 1 is built by the shared key builder (`thermo_knowledge.reuse`) from the
staged manifest's key, the mapping version (the hash of `mapping.toml`, `mapping.py` and the
framework's format number), the declaration fingerprint, the framework files the stage executes
and the installed versions of its libraries; phase 2 adds the hash of the phase-1 manifest and of
the resolution manifest. A manifest records every input of its key.
"""

from __future__ import annotations

import hashlib
import importlib.util
import shutil
import uuid
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path
from types import ModuleType

import pyarrow.parquet as pq

from thermo_knowledge import identity, reuse
from thermo_knowledge import pipeline_contract as pc
from thermo_knowledge.acquire.lock import read_lock
from thermo_knowledge.acquire.manifest import (
    Manifest,
    default_lock_path,
    default_sources_dir,
    load_sources,
)
from thermo_knowledge.canonical import provenance, store
from thermo_knowledge.canonical.environment import Environment
from thermo_knowledge.canonical.provenance import CarrierInfo, Carriers
from thermo_knowledge.canonical.store import (
    CanonicalError,
    CanonicalManifest,
    FormulaScopeRecord,
)
from thermo_knowledge.canonical.writer import CanonicalWriter
from thermo_knowledge.declaration import Declaration
from thermo_knowledge.mapping import claims, coverage
from thermo_knowledge.mapping.context import IdentityContext, RecordContext, SubjectInfo
from thermo_knowledge.mapping.spec import MappingSpec, SpecError, load_spec, validate
from thermo_knowledge.mapping.staged import StagedTables
from thermo_knowledge.staging import load, stage
from thermo_knowledge.staging import schema as staging_schema
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.stage import StageState
from thermo_knowledge.staging.writer import sha256_file

FORMAT = 2
"""Bump when the framework's output changes in a way the mapping's files do not show."""
IDENTITIES = "identities"
RECORDS = "records"


class MapError(CanonicalError):
    """A refusal of `tk map`: the mapping or its inputs are not in a state to run."""


@dataclass(frozen=True)
class Prepared:
    """Everything a phase needs, checked."""

    source_id: str
    manifest: Manifest
    carrier: CarrierInfo
    tree: Path
    tables: StagedTables
    staged_key: str
    spec: MappingSpec
    module_path: Path
    version: str
    decl: Declaration
    fingerprint: str


@dataclass(frozen=True)
class MapOutcome:
    source_id: str
    phase: str
    status: str  # "mapped" | "current"
    tables: dict[str, int]
    coverage: coverage.Coverage | None = None
    unused_optional: tuple[str, ...] = ()
    """Value rules declared `optional` that were applied to no row: reported, not refused."""


def mapping_sources(env: Environment) -> list[str]:
    """The ids of every source that has a `mappings/<id>/mapping.toml`."""
    if not env.mappings_dir.is_dir():
        return []
    return sorted(
        entry.name for entry in env.mappings_dir.iterdir() if (entry / "mapping.toml").is_file()
    )


def prepare(env: Environment, source_id: str, decl: Declaration | None = None) -> Prepared:
    """Load and check the source, its staged tables and its mapping."""
    decl = decl if decl is not None else env.declaration()
    mapping_dir = env.mappings_dir / source_id
    toml_path, module_path = mapping_dir / "mapping.toml", mapping_dir / "mapping.py"
    if not toml_path.is_file() or not module_path.is_file():
        raise MapError(f"{source_id}: {mapping_dir} needs mapping.toml and mapping.py")
    manifests = load_sources(env.sources_dir or default_sources_dir())
    manifest = manifests.get(source_id)
    if manifest is None:
        raise MapError(f"{source_id}: no source manifest of that id")
    entries = read_lock(env.lock_path or default_lock_path())
    entry = entries.get(source_id)
    if entry is None or entry.pin is None:
        raise MapError(f"{source_id}: not acquired (see sources.lock)")
    try:
        resolved = stage.require_reader(env.stage, manifest)
        state, why = stage.stage_state(env.stage, manifest, entry, resolved)
        if state is StageState.NOT_STAGED:
            raise MapError(f"{source_id}: not staged; run `tk read {source_id}`")
        if state is StageState.STALE:
            raise MapError(
                f"{source_id}: the staged data is stale ({why}); run `tk read {source_id}`"
            )
        directory = stage.staged_path(env.stage, source_id, entry.pin)
        staged_manifest, schemas = load.verify_staged(directory)
        tree = stage.tree_path(env.stage, manifest, entry)
    except StagingError as error:
        raise MapError(str(error)) from error
    spec = load_spec(toml_path)
    problems = validate(spec, decl, schemas, source=source_id)
    if problems:
        raise SpecError([f"{toml_path}: {problem}" for problem in problems])
    digest = hashlib.sha256()
    for part in (toml_path.read_bytes(), module_path.read_bytes(), str(FORMAT).encode()):
        digest.update(len(part).to_bytes(8, "big"))
        digest.update(part)
    return Prepared(
        source_id=source_id,
        manifest=manifest,
        carrier=provenance.from_source(manifest, entry, staged_manifest),
        tree=tree,
        tables=StagedTables(directory, staged_manifest, schemas),
        staged_key=staged_manifest.reuse_key,
        spec=spec,
        module_path=module_path,
        version=digest.hexdigest(),
        decl=decl,
        fingerprint=env.fingerprint(decl),
    )


def load_module(prepared: Prepared) -> ModuleType:
    spec = importlib.util.spec_from_file_location(
        f"thermo_knowledge_mapping_{prepared.source_id}", prepared.module_path
    )
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _entry_point(module: ModuleType, name: str, source_id: str) -> Callable[..., None]:
    found = getattr(module, name, None)
    if not callable(found):
        raise MapError(f"{source_id}: mapping.py defines no function `{name}(ctx)`")
    return found  # type: ignore[no-any-return]


def _key(prepared: Prepared, phase: str, **upstream: str) -> reuse.Key:
    """The reuse key of one phase: the mapping's own inputs, then what the phase reads."""
    try:
        return reuse.stage_key(
            "map",
            {
                "phase": phase,
                "staged": prepared.staged_key,
                "mapping": prepared.version,
                "declaration": prepared.fingerprint,
                "format": FORMAT,
                **upstream,
            },
        )
    except reuse.ReuseError as error:
        raise MapError(f"{prepared.source_id}: {error}") from error


def _identity_key(prepared: Prepared) -> reuse.Key:
    return _key(prepared, "identity")


def _recorded(directory: Path) -> CanonicalManifest | None:
    if not (directory / store.MANIFEST_NAME).is_file():
        return None
    try:
        return store.read_manifest(directory)
    except CanonicalError:
        return None


def _claim_records(directory: Path) -> dict[str, store.TableRecord]:
    records: dict[str, store.TableRecord] = {}
    for file in (
        claims.SOURCE_ENTITY_FILE,
        claims.ASSERTION_FILE,
        claims.COMPONENT_FILE,
        claims.LEDGER_FILE,
        claims.RULES_FILE,
    ):
        path = directory / file
        parquet = pq.ParquetFile(path)
        records[file.removesuffix(".parquet")] = store.TableRecord(
            file=file,
            rows=parquet.metadata.num_rows,
            schema_fingerprint=staging_schema.fingerprint(parquet.schema_arrow),
            content_hash=sha256_file(path),
        )
    return records


def run_identity(
    env: Environment, source_id: str, *, force: bool = False, decl: Declaration | None = None
) -> MapOutcome:
    """Phase 1: emit the source entities and identity assertions of `source_id`."""
    prepared = prepare(env, source_id, decl)
    key = _identity_key(prepared)
    destination = env.source_dir(source_id) / claims.IDENTITY_DIR
    recorded = _recorded(destination)
    if not force and recorded is not None and recorded.reuse_key == key.digest:
        return MapOutcome(source_id, "identity", "current", _counts(recorded))
    run = _entry_point(load_module(prepared), IDENTITIES, source_id)
    ctx = IdentityContext(
        source_id=source_id,
        spec=prepared.spec,
        decl=prepared.decl,
        tables=prepared.tables,
        carrier=prepared.carrier,
    )
    run(ctx)
    carriers = Carriers()
    carriers.add(prepared.carrier, prepared.tree)
    for artifact in sorted(
        {c.artifact for c in ctx.entities}
        | {c.artifact for c in ctx.assertions}
        | {c.artifact for c in ctx.components}
    ):
        carriers.artifact(source_id, artifact)
    parent = env.source_dir(source_id)
    work = store.new_work_directory(parent)
    try:
        summary = claims.write_claims(
            work,
            ctx.entities,
            ctx.assertions,
            ctx.components,
            coverage.ledger_rows(ctx.outcomes),
            ctx.rule_use(),
        )
        manifest = CanonicalManifest(
            schema=store.MANIFEST_SCHEMA,
            source_id=source_id,
            phase="identity",
            reuse_key=key.digest,
            inputs=key.inputs,
            tables=_claim_records(work),
            summary=summary,
            carrier=prepared.carrier,
            formula_scopes=[
                FormulaScopeRecord(item.scope, item.discriminator)
                for item in prepared.spec.formula_scope
            ],
        )
        store.write_manifest(work, manifest)
        store.install_directory(work, destination)
    except BaseException:
        shutil.rmtree(work, ignore_errors=True)
        raise
    return MapOutcome(source_id, "identity", "mapped", _counts(manifest))


def _counts(manifest: CanonicalManifest) -> dict[str, int]:
    return {name: record.rows for name, record in sorted(manifest.tables.items())}


def load_subjects(resolution_dir: Path, carrier: CarrierInfo) -> dict[tuple[str, str], SubjectInfo]:
    """What resolution decided for each source entity of `carrier`: (scope, key) to target,
    status and, for an ambiguous entity, its candidates' canonical keys."""
    entity, candidate, material = (
        pc.SOURCE_ENTITY,
        pc.RESOLUTION_CANDIDATE,
        pc.MATERIAL_ENTITY,
    )
    carrier_id = identity.identifier(pc.SOURCE.declared, [carrier.key])
    path = resolution_dir / store.file_name(entity.table)
    if not path.is_file():  # a resolution of no entities writes no table
        return {}
    entities = pq.read_table(path).to_pylist()
    keys: dict[uuid.UUID, str] = {}
    candidates: dict[uuid.UUID, list[str]] = {}
    candidate_path = resolution_dir / store.file_name(candidate.table)
    if candidate_path.is_file():
        for row in pq.read_table(resolution_dir / store.file_name(material.table)).to_pylist():
            keys[row["id"]] = row[material.canonical_key]
        for row in pq.read_table(candidate_path).to_pylist():
            candidates.setdefault(row[candidate.source_entity], []).append(
                keys[row[candidate.candidate]]
            )
    return {
        (row[entity.scope], row[entity.local_key]): SubjectInfo(
            row[entity.target],
            row[entity.status],
            tuple(sorted(candidates.get(row["id"], ()))),
        )
        for row in entities
        if row[entity.carrier] == carrier_id
    }


def load_form_aggregations(resolution_dir: Path) -> dict[uuid.UUID, uuid.UUID]:
    """The aggregation of each species form resolution wrote (species form to aggregation): the
    phase of a reaction's participants, which fixes the dimension of its rate constants."""
    form = pc.SPECIES_FORM
    path = resolution_dir / store.file_name(form.table)
    if not path.is_file():  # a resolution of no species forms writes no table
        return {}
    return {
        row["id"]: row[form.aggregation] for row in pq.read_table(path).to_pylist()
    }


def _check_identity(env: Environment, prepared: Prepared) -> tuple[Path, CanonicalManifest]:
    directory = env.source_dir(prepared.source_id) / claims.IDENTITY_DIR
    recorded = _recorded(directory)
    if recorded is None:
        raise MapError(
            f"{prepared.source_id}: no phase-1 output; run "
            f"`tk map {prepared.source_id} --phase identity`"
        )
    if recorded.reuse_key != _identity_key(prepared).digest:
        raise MapError(
            f"{prepared.source_id}: the phase-1 output is older than the mapping, the staged "
            f"data or the declaration; run `tk map {prepared.source_id} --phase identity`"
        )
    store.verify_directory(directory, recorded)
    return directory, recorded


def _check_resolution(env: Environment, prepared: Prepared, identity_hash: str) -> str:
    directory = env.resolution_dir
    recorded = _recorded(directory)
    if recorded is None:
        raise MapError("there is no resolution result; run `tk resolve`")
    if recorded.inputs.get(f"identity:{prepared.source_id}") != identity_hash:
        raise MapError(
            f"the resolution result is older than the phase-1 output of {prepared.source_id} "
            "(or does not include it); run `tk resolve`"
        )
    if recorded.inputs.get("declaration") != prepared.fingerprint:
        raise MapError("the resolution result was made from another declaration; run `tk resolve`")
    store.verify_directory(directory, recorded)
    return store.manifest_hash(directory)


def run_records(
    env: Environment, source_id: str, *, force: bool = False, decl: Declaration | None = None
) -> MapOutcome:
    """Phase 2: emit every record but source entities and identity assertions."""
    prepared = prepare(env, source_id, decl)
    identity_dir, _ = _check_identity(env, prepared)
    identity_hash = store.manifest_hash(identity_dir)
    resolution_hash = _check_resolution(env, prepared, identity_hash)
    key = _key(prepared, "records", identity=identity_hash, resolution=resolution_hash)
    destination = env.source_dir(source_id)
    recorded = _recorded(destination)
    if (
        not force
        and recorded is not None
        and recorded.reuse_key == key.digest
        and recorded.phase == "records"
    ):
        return MapOutcome(source_id, "records", "current", _counts(recorded))
    run = _entry_point(load_module(prepared), RECORDS, source_id)
    carriers = Carriers()
    carriers.add(prepared.carrier, prepared.tree)
    writer = CanonicalWriter(
        prepared.decl,
        carriers,
        form_aggregations=load_form_aggregations(env.resolution_dir),
    )
    ctx = RecordContext(
        writer=writer,
        subjects=load_subjects(env.resolution_dir, prepared.carrier),
        source_id=source_id,
        spec=prepared.spec,
        decl=prepared.decl,
        tables=prepared.tables,
        carrier=prepared.carrier,
    )
    run(ctx)
    outcomes = coverage.merge_outcomes(claims.read_ledger(identity_dir), ctx.outcomes)
    result = coverage.compute(prepared.tables, ctx.classifier, prepared.spec.tables, outcomes)
    applied = dict(claims.read_rule_use(identity_dir))
    for rule, rows in ctx.rule_use().items():
        applied[rule] = applied.get(rule, 0) + rows
    refused, reported = coverage.check_rule_use(prepared.spec, result, applied)
    if refused:
        raise MapError(
            f"{source_id}: {len(refused)} declared value rule(s) were applied to no row:\n  "
            + "\n  ".join(refused)
        )
    coverage.write_rows(
        writer,
        source_id,
        result,
        coverage.rule_rows(prepared.spec, prepared.decl, prepared.tables.schemas, applied),
    )
    work = store.new_work_directory(env.canonical_dir)
    try:
        records = store.write_tables(work, writer.tables())
        manifest = CanonicalManifest(
            schema=store.MANIFEST_SCHEMA,
            source_id=source_id,
            phase="records",
            reuse_key=key.digest,
            inputs=key.inputs,
            tables=records,
            summary={state: result.total(state) for state in coverage.STATES},
        )
        store.write_manifest(work, manifest)
        shutil.copytree(identity_dir, work / claims.IDENTITY_DIR)
        store.install_directory(work, destination)
    except BaseException:
        shutil.rmtree(work, ignore_errors=True)
        raise
    return MapOutcome(
        source_id, "records", "mapped", _counts(manifest), result, tuple(reported)
    )


def coverage_lines(result: coverage.Coverage) -> list[str]:
    """The coverage table: rows per staged table and state."""
    width = max((len(table) for table in result.counts), default=5)
    header = f"{'table':<{width}}  " + "  ".join(f"{state:>16}" for state in coverage.STATES)
    lines = [header]
    for table, counts in result.counts.items():
        lines.append(
            f"{table:<{width}}  "
            + "  ".join(f"{counts.get(state, 0):>16}" for state in coverage.STATES)
        )
    lines.append(
        f"{'total':<{width}}  "
        + "  ".join(f"{result.total(state):>16}" for state in coverage.STATES)
    )
    reasons = result.by_reason()
    if reasons:
        lines.append("")
        lines.append("rows not loaded, by reason:")
        lines.extend(f"  {reason}: {rows}" for reason, rows in reasons.items())
    return lines
