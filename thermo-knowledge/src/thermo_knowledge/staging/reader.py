# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Finding the reader a source declares, and the reuse key of a stage.

A reader is either

- an in-process module `thermo_knowledge.readers.<reader>` (source `environment = "core"`), with

  ```python
  READER_VERSION: str                         # bump when its output changes
  TABLES: Mapping[str, pyarrow.Schema]        # an explicit schema for every table it emits
  def read(tree: Path, writer: Writer) -> None
  ```

  `read` receives the path of the acquired tree and the writer; it emits rows or batches through
  the writer and lists the payload files it does not decompose. It may be a package; or

- a script `envs/<environment>/readers/<reader>.py` run in that side environment through
  `envs/tk-env.sh` (see `side.py`), for readers that need a library the core environment cannot
  hold and cannot import `thermo_knowledge`. It declares `READER_VERSION = "..."` as a top-level
  string literal, which the core reads without running the script.

`<reader>` is the source manifest's `payload.reader`.

The reuse key of a stage is the hash of its complete inputs: the source's tree hash from the lock,
the reader's name, environment and version, the hash of the reader's source files and the staging
format (the layout this package writes). A staged directory whose recorded key equals the current
key is reused; `--force` reads again.
"""

from __future__ import annotations

import ast
import hashlib
import importlib
import importlib.util
import json
import re
from collections.abc import Callable, Mapping
from dataclasses import dataclass
from pathlib import Path
from types import ModuleType

import pyarrow as pa

from thermo_knowledge import config
from thermo_knowledge.acquire.manifest import Manifest
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.writer import Writer

READERS_PACKAGE = "thermo_knowledge.readers"
STAGING_FORMAT = 1
"""Bump when the layout this package writes (manifest, Parquet options) changes."""

CORE = "core"
_READER_NAME = re.compile(r"^[a-z][a-z0-9_]*$")
_ENVIRONMENT_NAME = re.compile(r"^[a-z][a-z0-9_-]*$")


def envs_dir() -> Path:
    """`envs/` of the tree."""
    return config.TREE_DIR / "envs"


@dataclass(frozen=True)
class ResolvedReader:
    """The reader a source declares."""

    name: str
    environment: str
    version: str
    source_files: tuple[Path, ...]
    module: ModuleType | None = None
    script: Path | None = None

    @property
    def side(self) -> bool:
        return self.script is not None

    def describe(self) -> str:
        return f"{self.environment}:{self.name}"

    @classmethod
    def from_module(cls, name: str, module: ModuleType, environment: str = CORE) -> ResolvedReader:
        """An in-process reader from an imported module (also the way tests supply one)."""
        version = getattr(module, "READER_VERSION", None)
        if not isinstance(version, str) or not version:
            raise StagingError(f"reader {name}: READER_VERSION must be a non-empty string")
        return cls(name, environment, version, _module_sources(module), module=module)


Resolver = Callable[[Manifest], "ResolvedReader | None"]


def _module_sources(module: ModuleType) -> tuple[Path, ...]:
    spec = getattr(module, "__spec__", None)
    origin = getattr(spec, "origin", None) or getattr(module, "__file__", None)
    if origin is None:
        raise StagingError(f"reader module {module.__name__} has no source file")
    path = Path(origin)
    if spec is not None and spec.submodule_search_locations:
        root = path.parent
        files = sorted(file for file in root.rglob("*.py") if "__pycache__" not in file.parts)
        return tuple(files)
    return (path,)


def default_resolver(manifest: Manifest) -> ResolvedReader | None:
    """The reader of `manifest.payload`, or `None` when it declares none or the reader does not
    exist."""
    return resolve(manifest, package=READERS_PACKAGE, envs=envs_dir())


def resolve(manifest: Manifest, *, package: str, envs: Path) -> ResolvedReader | None:
    name = manifest.payload.reader
    if name is None:
        return None
    if not _READER_NAME.match(name):
        raise StagingError(f"{manifest.id}: reader name {name!r} must match ^[a-z][a-z0-9_]*$")
    environment = manifest.payload.environment
    if environment == CORE:
        try:
            spec = importlib.util.find_spec(f"{package}.{name}")
        except ModuleNotFoundError:
            return None
        if spec is None:
            return None
        try:
            module = importlib.import_module(f"{package}.{name}")
        except ImportError as error:
            raise StagingError(
                f"{manifest.id}: reader {name} cannot be imported: {error}"
            ) from error
        return ResolvedReader.from_module(name, module)
    if not _ENVIRONMENT_NAME.match(environment):
        raise StagingError(f"{manifest.id}: environment {environment!r} is not a valid name")
    script = envs / environment / "readers" / f"{name}.py"
    if not script.is_file():
        return None
    return ResolvedReader(name, environment, script_version(script), (script,), script=script)


def script_version(script: Path) -> str:
    """`READER_VERSION` of a side-reader script, read from its syntax tree without running it."""
    try:
        tree = ast.parse(script.read_text(encoding="utf-8"), filename=str(script))
    except (SyntaxError, UnicodeDecodeError) as error:
        raise StagingError(f"{script}: cannot be parsed: {error}") from error
    for node in tree.body:
        if isinstance(node, ast.Assign):
            for target in node.targets:
                if isinstance(target, ast.Name) and target.id == "READER_VERSION":
                    if isinstance(node.value, ast.Constant) and isinstance(node.value.value, str):
                        if node.value.value:
                            return node.value.value
    raise StagingError(f'{script}: declare READER_VERSION = "..." as a top-level string literal')


def module_tables(reader: ResolvedReader) -> Mapping[str, pa.Schema]:
    """The declared schemas of an in-process reader."""
    module = reader.module
    assert module is not None
    tables = getattr(module, "TABLES", None)
    if not isinstance(tables, Mapping) or not all(
        isinstance(schema, pa.Schema) for schema in tables.values()
    ):
        raise StagingError(f"reader {reader.name}: TABLES must map table names to pyarrow schemas")
    return tables


def module_read(reader: ResolvedReader) -> Callable[[Path, Writer], None]:
    read = getattr(reader.module, "read", None)
    if not callable(read):
        raise StagingError(f"reader {reader.name}: a function read(tree, writer) is required")
    return read  # type: ignore[no-any-return]


def source_hash(files: tuple[Path, ...]) -> str:
    """SHA-256 over the names and contents of a reader's source files."""
    digest = hashlib.sha256()
    base = _common_parent(files)
    for file in sorted(files):
        digest.update(file.relative_to(base).as_posix().encode())
        digest.update(b"\0")
        digest.update(hashlib.sha256(file.read_bytes()).digest())
    return digest.hexdigest()


def _common_parent(files: tuple[Path, ...]) -> Path:
    parents = [file.parent for file in files]
    base = parents[0]
    for parent in parents[1:]:
        while base not in (parent, *parent.parents):
            base = base.parent
    return base


def reuse_key(*, source_id: str, tree_identity: str, reader: ResolvedReader) -> tuple[str, str]:
    """`(key, reader_source_hash)` for a stage's complete inputs."""
    hashed = source_hash(reader.source_files)
    description = {
        "staging_format": STAGING_FORMAT,
        "source": source_id,
        "tree": tree_identity,
        "reader": reader.name,
        "environment": reader.environment,
        "reader_version": reader.version,
        "reader_source": hashed,
    }
    text = json.dumps(description, sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(text.encode()).hexdigest(), hashed
