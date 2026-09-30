# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The one reuse-key builder of every stage (pipeline section 6).

A stage's output is reused when its recorded key equals the key of the present run. Every stage
builds that key here, and the key always covers, besides the inputs the stage states:

- a digest of the framework source files the stage executes, from the list this module declares
  for the stage (`STAGES`): the stage's own package and the shared packages it runs. The list is
  declared, not derived from imports, so what a stage depends on is stated in one reviewable
  place; a listed path that does not exist is an error, never an empty digest;
- the resolved versions of the third-party libraries the stage declares as result-affecting,
  taken from the installed distribution metadata (`installed_version`);
- the digest of each file the stage names in addition (an oracle harness script, the lock of the
  environment a side reader or a harness runs in).

The format numbers the stages keep mark only deliberate contract changes, that is, changes of
what a stage writes that none of the above shows. They are ordinary inputs here.
"""

from __future__ import annotations

import hashlib
import json
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from importlib import metadata
from pathlib import Path

from thermo_knowledge import config

PACKAGE_ROOT = Path(__file__).resolve().parent
"""The directory the declared framework paths are relative to: `src/thermo_knowledge`."""
CORE = "core"
_SUFFIXES = frozenset({".py", ".toml"})
_ENVIRONMENT_LOCKS = ("uv.lock", "environment.lock.txt", "pip.lock.txt")
"""The lock files a side environment's directory may hold."""


class ReuseError(Exception):
    """A stage's dependencies cannot be stated: a declared path or library is missing."""


@dataclass(frozen=True)
class StageDeclaration:
    """What a stage executes and which installed libraries decide its result."""

    sources: tuple[str, ...]
    """Packages (directories) and modules (files), relative to `PACKAGE_ROOT`."""
    libraries: tuple[str, ...]
    """Distribution names whose installed versions enter the key."""


_DECLARATION_AND_SCHEMA = (
    "declaration",
    "generate",
    "canonical",
    "acquire",
    "config.py",
    "identity.py",
    "transposition.py",
    "schema_build.py",
    "pipeline_contract.py",
    "pipeline_contract.toml",
)
"""The packages and modules that turn the declaration into canonical rows, which every stage
after reading runs: the loader, the generator's table descriptions, the canonical writer with its
provenance rows, and the identity and orientation rules."""

STAGES: Mapping[str, StageDeclaration] = {
    "read": StageDeclaration(
        sources=("staging", "acquire", "config.py"),
        libraries=("pyarrow",),
    ),
    "map": StageDeclaration(
        sources=(*_DECLARATION_AND_SCHEMA, "mapping", "staging"),
        libraries=("pint", "pyarrow"),
    ),
    "resolve": StageDeclaration(
        sources=(*_DECLARATION_AND_SCHEMA, "resolve", "mapping/claims.py"),
        libraries=("rdkit", "pyarrow"),
    ),
    "qualify": StageDeclaration(
        sources=(*_DECLARATION_AND_SCHEMA, "qualify", "expression", "build"),
        libraries=("sympy", "numpy", "scipy", "pint", "pyarrow"),
    ),
}
"""Every stage that keeps a reuse key. `tk load-src` has none of its own: it loads only staged
data whose `read` key is current. `tk build` keeps none: it rebuilds."""


def installed_version(distribution: str) -> str:
    """The installed version of a distribution, from its metadata; the one accessor the keys
    read versions through."""
    try:
        return metadata.version(distribution)
    except metadata.PackageNotFoundError as error:
        raise ReuseError(
            f"the library `{distribution}` is declared result-affecting and is not installed"
        ) from error


def _files(entry: str, root: Path) -> list[Path]:
    path = root / entry
    if path.is_file():
        return [path]
    if path.is_dir():
        return sorted(
            file
            for file in path.rglob("*")
            if file.is_file() and file.suffix in _SUFFIXES and "__pycache__" not in file.parts
        )
    raise ReuseError(f"the declared framework path `{entry}` does not exist under {root}")


def _digest_files(files: Sequence[tuple[str, Path]]) -> str:
    digest = hashlib.sha256()
    for name, path in sorted(files):
        digest.update(name.encode())
        digest.update(b"\0")
        digest.update(hashlib.sha256(path.read_bytes()).digest())
    return digest.hexdigest()


def framework_digest(stage: str) -> str:
    """The digest of the framework source files `stage` executes: each file's path relative to
    `PACKAGE_ROOT` and its content."""
    declared = STAGES.get(stage)
    if declared is None:
        raise ReuseError(f"`{stage}` is not a stage that keeps a reuse key")
    found: dict[str, Path] = {}
    for entry in declared.sources:
        for file in _files(entry, PACKAGE_ROOT):
            found[file.relative_to(PACKAGE_ROOT).as_posix()] = file
    return _digest_files(list(found.items()))


def file_digest(paths: Sequence[Path]) -> str:
    """The digest of the named files' contents (and names), for an extra input."""
    missing = [str(path) for path in paths if not path.is_file()]
    if missing:
        raise ReuseError(f"the file(s) a reuse key covers do not exist: {', '.join(missing)}")
    return _digest_files([(path.name, path) for path in paths])


def core_lock() -> Path:
    """The lock of the core environment this process runs in."""
    return config.TREE_DIR / "uv.lock"


def directory_locks(directory: Path) -> tuple[Path, ...]:
    """The lock files of the environment whose directory is `directory` (`envs/<name>/`); an
    environment with none cannot be keyed."""
    found = tuple(directory / name for name in _ENVIRONMENT_LOCKS if (directory / name).is_file())
    if not found:
        raise ReuseError(f"the environment directory {directory} has no lock file")
    return found


def environment_locks(environment: str, tree: Path) -> tuple[Path, ...]:
    """The lock files of an environment: the tree's `uv.lock` for `core`, else the lock files
    in `envs/<environment>/` of `tree`."""
    if environment == CORE:
        return (core_lock(),)
    return directory_locks(tree / "envs" / environment)


@dataclass(frozen=True)
class Key:
    """A reuse key and what it was computed from, so a manifest can record both."""

    digest: str
    inputs: dict[str, str]
    """The stated inputs, then `framework`, each `library:<name>` and each `file:<label>`."""


def stage_key(
    stage: str,
    inputs: Mapping[str, object],
    *,
    libraries: Sequence[str] = (),
    files: Mapping[str, Sequence[Path]] | None = None,
) -> Key:
    """The key of one run of `stage`.

    `inputs` are what the stage states (upstream hashes, the declaration fingerprint, the format
    number); `libraries` names further distributions for this run (the library under test);
    `files` maps a label to files whose content enters the key (a harness script, environment
    locks)."""
    declared = STAGES.get(stage)
    if declared is None:
        raise ReuseError(f"`{stage}` is not a stage that keeps a reuse key")
    recorded: dict[str, str] = {name: str(value) for name, value in inputs.items()}
    recorded["framework"] = framework_digest(stage)
    for library in sorted({*declared.libraries, *libraries}):
        recorded[f"library:{library}"] = installed_version(library)
    for label, paths in sorted((files or {}).items()):
        recorded[f"file:{label}"] = file_digest(paths)
    text = json.dumps({"stage": stage, **recorded}, sort_keys=True, separators=(",", ":"))
    return Key(hashlib.sha256(text.encode()).hexdigest(), recorded)
