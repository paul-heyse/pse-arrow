# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Helpers for the staging tests: a workspace holding the tiny fixture source as an acquired
store, reader modules and resolvers. Contains no tests."""

from __future__ import annotations

import importlib.util
import shutil
import sys
from dataclasses import dataclass
from pathlib import Path
from types import ModuleType

from test_acquire_support import RIGHTS

from thermo_knowledge.acquire import store
from thermo_knowledge.acquire.lock import LockEntry, read_lock, write_lock
from thermo_knowledge.acquire.manifest import Manifest, load_sources
from thermo_knowledge.acquire.runtime import Context
from thermo_knowledge.staging import reader
from thermo_knowledge.staging.reader import ResolvedReader, Resolver
from thermo_knowledge.staging.stage import StageContext

FIXTURES = Path(__file__).parent / "fixtures" / "readers"
TINY_TREE = FIXTURES / "tiny" / "tree"
TINY_READER = FIXTURES / "tiny_reader.py"
FAKE_ENVS = FIXTURES / "envs"
COMMIT = "0123456789abcdef0123456789abcdef01234567"
PIN = COMMIT[:12]
INCLUDE = ["data/**", "notes.txt", "blob/**"]


def load_module(path: Path, name: str) -> ModuleType:
    """Import the file `path` as module `name` (not registered in `sys.modules`)."""
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def tiny_module(path: Path = TINY_READER) -> ModuleType:
    return load_module(path, "tiny_reader_under_test")


def module_resolver(module: ModuleType, name: str = "tiny") -> Resolver:
    def resolve(manifest: Manifest) -> ResolvedReader | None:
        return ResolvedReader.from_module(name, module)

    return resolve


def side_resolver() -> Resolver:
    def resolve(manifest: Manifest) -> ResolvedReader | None:
        return reader.resolve(manifest, package="thermo_knowledge.readers", envs=FAKE_ENVS)

    return resolve


def fake_side_command(environment: str, script: Path, tree: Path, out: Path) -> list[str]:
    """The side-reader invocation with the core interpreter in place of `tk-env.sh run`."""
    return [sys.executable, str(script), "--tree", str(tree), "--out", str(out)]


@dataclass
class Workspace:
    """An acquired source `tiny` in a temporary store, with its manifest and lock."""

    root: Path
    source_id: str = "tiny"

    @property
    def sources(self) -> Path:
        return self.root / "sources"

    @property
    def lock(self) -> Path:
        return self.root / "sources.lock"

    @property
    def raw(self) -> Path:
        return self.root / "store" / "raw"

    @property
    def staged(self) -> Path:
        return self.root / "store" / "staged"

    @property
    def tree(self) -> Path:
        return store.pin_dir(self.raw, self.source_id, PIN) / store.TREE_DIR_NAME

    @property
    def staged_pin(self) -> Path:
        return self.staged / self.source_id / PIN

    def manifest(self) -> Manifest:
        return load_sources(self.sources)[self.source_id]

    def entries(self) -> dict[str, LockEntry]:
        return read_lock(self.lock)

    def context(self, resolver: Resolver, **overrides: object) -> StageContext:
        return StageContext(
            acquire=Context(raw_dir=self.raw, repo_root=self.root),
            staged_dir=self.staged,
            resolver=resolver,
            **overrides,  # type: ignore[arg-type]
        )


def build_workspace(
    root: Path,
    *,
    reader_name: str = "tiny",
    environment: str = "core",
    include: list[str] | None = None,
    exclude: list[str] | None = None,
    source_id: str = "tiny",
) -> Workspace:
    """Declare `source_id`, copy the tiny tree into a raw store and record the lock."""
    workspace = Workspace(root, source_id)
    workspace.sources.mkdir(parents=True)
    include_text = ", ".join(f'"{pattern}"' for pattern in (include or INCLUDE))
    exclude_text = ", ".join(f'"{pattern}"' for pattern in (exclude or []))
    (workspace.sources / f"{source_id}.toml").write_text(
        f'id = "{source_id}"\ntitle = "{source_id}"\ntier = "A"\n\n'
        f'[acquire]\nkind = "git"\nurl = "https://example.invalid/{source_id}.git"\n'
        f'commit = "{COMMIT}"\n\n'
        f'[payload]\nreader = "{reader_name}"\nenvironment = "{environment}"\n'
        f"include = [{include_text}]\nexclude = [{exclude_text}]\n{RIGHTS}"
    )
    pin_dir = store.pin_dir(workspace.raw, source_id, PIN)
    shutil.copytree(TINY_TREE, pin_dir / store.TREE_DIR_NAME)
    entries = store.scan_tree(pin_dir / store.TREE_DIR_NAME)
    acquisition = store.build_acquisition(
        source_id=source_id,
        kind="git",
        pin=PIN,
        resolved=COMMIT,
        retrieved="2026-09-30T00:00:00Z",
        tool_versions={},
        urls=[],
        details={},
        entries=entries,
    )
    store.write_acquisition(pin_dir, acquisition)
    write_lock(
        workspace.lock,
        {
            source_id: LockEntry(
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
    return workspace
