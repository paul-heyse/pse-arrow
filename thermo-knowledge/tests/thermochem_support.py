# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Helpers for the mapping tests of the thermochemical sources (Cantera, NASA CEA, JANAF), which
run each mapping over its real staged data. Contains no tests."""

from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path

import pyarrow.parquet as pq
from mapping_support import real_declaration, rows

from thermo_knowledge import config
from thermo_knowledge.acquire.lock import read_lock
from thermo_knowledge.acquire.manifest import default_lock_path
from thermo_knowledge.canonical.environment import Environment
from thermo_knowledge.mapping import claims, runner
from thermo_knowledge.resolve.command import resolve_all


def staged_directory(source: str) -> Path | None:
    entry = read_lock(default_lock_path()).get(source)
    if entry is None or entry.pin is None:
        return None
    directory = config.staged_dir() / source / entry.pin
    raw = config.raw_dir() / source / entry.pin
    return directory if (directory / "manifest.json").is_file() and raw.is_dir() else None


def staged(source: str, table: str) -> list[dict[str, object]]:
    directory = staged_directory(source)
    assert directory is not None
    return rows(directory / f"{table}.parquet")


@dataclass(frozen=True)
class Run:
    source: str
    env: Environment
    coverage: runner.coverage.Coverage
    report: dict[str, object]

    def table(self, name: str, *, resolution: bool = False) -> list[dict[str, object]]:
        directory = self.env.resolution_dir if resolution else self.env.canonical_dir / self.source
        return rows(directory / f"{name}.parquet")

    def claims(self) -> tuple[list[claims.EntityClaim], list[claims.AssertionClaim]]:
        directory = self.env.canonical_dir / self.source / claims.IDENTITY_DIR
        return claims.read_entity_claims(directory), claims.read_assertion_claims(directory)

    def held(self) -> list[dict[str, object]]:
        return self.table("qual.held_row")


def run_source(directory: Path, source: str) -> Run:
    """Phase 1, resolution and phase 2 of one source over its real staged data."""
    env = Environment(canonical_dir=directory)
    decl = real_declaration()
    assert runner.run_identity(env, source, decl=decl).status == "mapped"
    resolve_all(env, decl=decl)
    outcome = runner.run_records(env, source, decl=decl)
    assert outcome.coverage is not None
    report = json.loads((env.resolution_dir / "report.json").read_text())
    return Run(source, env, outcome.coverage, report)


def parquet(path: Path) -> list[dict[str, object]]:
    return pq.read_table(path).to_pylist()
