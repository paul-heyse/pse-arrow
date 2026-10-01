# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""`tk build`: replace the canonical schemas of the database from the declaration and every
source's canonical Parquet, in one transaction (pipeline section 3)."""

from __future__ import annotations

from thermo_knowledge.build.inputs import RESOLUTION_ID, SourceInput, discover
from thermo_knowledge.build.record import BuildState, BuiltSource, read_state
from thermo_knowledge.build.run import BuildResult, build_database, dry_run
from thermo_knowledge.build.union import UnionConflictError

__all__ = [
    "RESOLUTION_ID",
    "BuildResult",
    "BuildState",
    "BuiltSource",
    "SourceInput",
    "UnionConflictError",
    "build_database",
    "discover",
    "dry_run",
    "read_state",
]
