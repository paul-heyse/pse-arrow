# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Helpers for the canonical writer, mapping and resolution tests. Contains no tests."""

from __future__ import annotations

import shutil
from functools import cache
from pathlib import Path

import pyarrow.parquet as pq

from readers_support import Workspace

from thermo_knowledge import config
from thermo_knowledge.acquire.lock import LockEntry
from thermo_knowledge.canonical.environment import Environment
from thermo_knowledge.canonical.provenance import (
    ArtifactInfo,
    CarrierInfo,
    Carriers,
    Origin,
    RightsInfo,
    SourceRef,
)
from thermo_knowledge.canonical.writer import CanonicalWriter
from thermo_knowledge.declaration import Declaration, load_declaration

FIXTURES = Path(__file__).parent / "fixtures" / "mapping"
MODEL = config.TREE_DIR / "model"
FORMS = config.TREE_DIR / "forms"
SATURATION = "vapor_pressure_exp_series_tau.pure"
TERM = "vapor_pressure_exp_series_tau.pure.term"


@cache
def real_declaration() -> Declaration:
    """The committed declaration, loaded once."""
    return load_declaration().require()


def extended_directories(destination: Path) -> tuple[Path, Path]:
    """Copies of the committed `model/` and `forms/` with the fixture forms added."""
    shutil.copytree(MODEL, destination / "model")
    shutil.copytree(FORMS, destination / "forms")
    for form in (FIXTURES / "forms").glob("*.toml"):
        shutil.copy(form, destination / "forms" / form.name)
    return destination / "model", destination / "forms"


def extended_declaration(destination: Path) -> Declaration:
    """The committed declaration plus slot groups with every transposition rule."""
    model, forms = extended_directories(destination)
    return load_declaration(model, forms).require()


def carrier(manifest_id: str = "src", *artifacts: str, spdx: str | None = "MIT") -> CarrierInfo:
    """A carrier whose named artifacts have a recorded hash, so no tree is needed."""
    return CarrierInfo(
        manifest_id=manifest_id,
        title=f"The source {manifest_id}",
        pin="0123456789ab",
        tree_hash="ab" * 32,
        retrieved="2026-09-30T09:32:09Z",
        reader="fake",
        reader_version="1",
        rights=[
            RightsInfo(
                scope="data",
                basis="licence_grant" if spdx else "not_stated",
                statement="A statement.",
                store="yes",
                redistribute="not_stated",
                commercial="not_stated",
                attribution=True,
                share_alike=False,
                observed="a test",
                spdx=spdx,
            )
        ],
        artifacts={
            name: ArtifactInfo(sha256=f"{index:02x}" * 32, size=100 + index)
            for index, name in enumerate(artifacts, 1)
        },
    )


def writer(decl: Declaration, *carriers: CarrierInfo) -> CanonicalWriter:
    registry = Carriers()
    for info in carriers or (carrier("src", "a.json", "b.json"),):
        registry.add(info)
    return CanonicalWriter(decl, registry)


def origin(
    locator: str = "a.json#/0", role: str = "published", *, carrier_id: str = "src"
) -> Origin:
    artifact = locator.split("#", 1)[0]
    return Origin(SourceRef(carrier_id, artifact, locator), role)


def rows(table_path: Path) -> list[dict[str, object]]:
    return pq.read_table(table_path).to_pylist()


# -- phase-1 output of fake carriers ---------------------------------------------------------

ETHANOL = "LFQSCWFLJHTTHZ-UHFFFAOYSA-N"
ETHANOL_INCHI = "InChI=1S/C2H6O/c1-2-3/h3H,2H2,1H3"
ETHANOL_SMILES = "CCO"
WATER = "XLYOFNOQVPJJNP-UHFFFAOYSA-N"
WATER_INCHI = "InChI=1S/H2O/h1H2"
METHANE = "VNWKTOKETHGBQD-UHFFFAOYSA-N"


def write_identity(
    canonical: Path,
    manifest_id: str,
    entities: list[dict[str, object]],
    *,
    formula_scopes: tuple[tuple[str, str | None], ...] = (),
    role: str = "published",
) -> Path:
    """Write the phase-1 output of a fake carrier.

    Each entity is `{"scope", "key", "assertions": [(scheme, value), ...]}` with optional
    `aggregation`, `polymorph`, `charge` and `locator`."""
    from thermo_knowledge.canonical import store
    from thermo_knowledge.canonical.store import CanonicalManifest, FormulaScopeRecord
    from thermo_knowledge.mapping import claims
    from thermo_knowledge.staging import schema as staging_schema
    from thermo_knowledge.staging.writer import sha256_file

    artifacts = sorted({f"data/{e['scope']}.json" for e in entities})
    info = carrier(manifest_id, *artifacts)
    claim_entities: list[claims.EntityClaim] = []
    claim_assertions: list[claims.AssertionClaim] = []
    for entity in entities:
        artifact = f"data/{entity['scope']}.json"
        locator = str(entity.get("locator", f"{artifact}#/{entity['key']}"))
        claim_entities.append(
            claims.EntityClaim(
                manifest_id,
                str(entity["scope"]),
                str(entity["key"]),
                entity.get("aggregation"),  # type: ignore[arg-type]
                entity.get("polymorph"),  # type: ignore[arg-type]
                entity.get("charge"),  # type: ignore[arg-type]
                role,
                artifact,
                locator,
            )
        )
        for scheme, value in entity["assertions"]:  # type: ignore[misc]
            claim_assertions.append(
                claims.AssertionClaim(
                    manifest_id,
                    str(entity["scope"]),
                    str(entity["key"]),
                    scheme,
                    value,
                    scheme,
                    artifact,
                    locator,
                )
            )
    directory = canonical / manifest_id / claims.IDENTITY_DIR
    directory.mkdir(parents=True)
    summary = claims.write_claims(directory, claim_entities, claim_assertions, [])
    records = {}
    for file in (claims.SOURCE_ENTITY_FILE, claims.ASSERTION_FILE, claims.LEDGER_FILE):
        parquet = pq.ParquetFile(directory / file)
        records[file.removesuffix(".parquet")] = store.TableRecord(
            file,
            parquet.metadata.num_rows,
            staging_schema.fingerprint(parquet.schema_arrow),
            sha256_file(directory / file),
        )
    store.write_manifest(
        directory,
        CanonicalManifest(
            schema=store.MANIFEST_SCHEMA,
            source_id=manifest_id,
            phase="identity",
            reuse_key=manifest_id,
            inputs={},
            tables=records,
            summary=summary,
            carrier=info,
            formula_scopes=[FormulaScopeRecord(scope, disc) for scope, disc in formula_scopes],
        ),
    )
    return directory


# -- a fake source, staged, with a mapping -----------------------------------------------------

FAKE_TREE = FIXTURES / "fake" / "tree"
FAKE_READER = FIXTURES / "fake_reader.py"
FAKE_MAPPINGS = FIXTURES / "mappings"


def fake_environment(
    root: Path, *, mappings: Path = FAKE_MAPPINGS, tree: Path = FAKE_TREE
) -> tuple[Environment, Workspace]:
    """A temporary acquired source `fake` (raw store, lock, manifest), its staged tables and the
    fixture mapping, with every location of `Environment` inside `root`."""
    import importlib.util

    from readers_support import COMMIT, PIN, module_resolver
    from test_acquire_support import RIGHTS

    from thermo_knowledge.acquire import store as raw_store
    from thermo_knowledge.acquire.lock import write_lock
    from thermo_knowledge.staging import stage

    workspace = Workspace(root, "fake")
    workspace.sources.mkdir(parents=True)
    (workspace.sources / "fake.toml").write_text(
        'id = "fake"\ntitle = "The fake source"\ntier = "A"\n\n'
        f'[acquire]\nkind = "git"\nurl = "https://example.invalid/fake.git"\ncommit = "{COMMIT}"\n\n'
        '[payload]\nreader = "fake"\nenvironment = "core"\n'
        'include = ["data/**", "notes.txt"]\nexclude = []\n' + RIGHTS
    )
    pin_dir = raw_store.pin_dir(workspace.raw, "fake", PIN)
    shutil.copytree(tree, pin_dir / raw_store.TREE_DIR_NAME)
    entries = raw_store.scan_tree(pin_dir / raw_store.TREE_DIR_NAME)
    acquisition = raw_store.build_acquisition(
        source_id="fake",
        kind="git",
        pin=PIN,
        resolved=COMMIT,
        retrieved="2026-09-30T00:00:00Z",
        tool_versions={},
        urls=[],
        details={},
        entries=entries,
    )
    raw_store.write_acquisition(pin_dir, acquisition)
    write_lock(
        workspace.lock,
        {
            "fake": LockEntry(
                kind="git",
                pin=PIN,
                retrieved="2026-09-30T00:00:00Z",
                file_count=acquisition.file_count,
                total_bytes=acquisition.total_bytes,
                tree_hash=acquisition.tree_hash,
                resolved=COMMIT,
            )
        },
    )
    spec = importlib.util.spec_from_file_location("fake_reader_under_test", FAKE_READER)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    context = workspace.context(module_resolver(module, "fake"))
    stage.read_source(context, workspace.manifest(), workspace.entries())
    home = root / "tree"
    shutil.copytree(mappings, home / "mappings")
    environment = Environment(
        canonical_dir=root / "canonical",
        tree=home,
        stage=context,
        sources_dir=workspace.sources,
        lock_path=workspace.lock,
        decisions_path=root / "decisions.toml",
    )
    return environment, workspace
