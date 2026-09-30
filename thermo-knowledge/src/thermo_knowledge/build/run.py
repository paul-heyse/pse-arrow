# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""`build_database`: the single path that replaces the canonical schemas of a database."""

from __future__ import annotations

import tempfile
from collections.abc import Sequence
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path

from thermo_knowledge import config
from thermo_knowledge.build import database
from thermo_knowledge.build.inputs import SourceInput
from thermo_knowledge.build.plan import BuildPlan, plan_build
from thermo_knowledge.build.record import BuiltSource
from thermo_knowledge.declaration.model import Declaration


@dataclass(frozen=True)
class BuildResult:
    fingerprint: str
    tables: dict[str, int]
    """Rows the database holds per table after the build (`schema.table`)."""
    merged: dict[str, int]
    """Identical duplicate rows collapsed by the union, for each table that had any."""
    sources: tuple[BuiltSource, ...]
    meta_rows: int
    """Rows of the reified declaration, in the `meta` tables the declaration defines."""
    entity_rows: int
    """Rows inserted for the declared entities (`prov.record` rows included)."""


def _result(plan: BuildPlan, built_at: datetime) -> BuildResult:
    sources = tuple(
        BuiltSource(
            item.source_id,
            item.carrier.pin,
            item.carrier.tree_hash,
            item.manifest.reuse_key,
            built_at,
        )
        for item in plan.inputs
        if item.carrier is not None
    )
    return BuildResult(
        fingerprint=plan.fingerprint,
        tables=plan.counts(),
        merged=plan.merged(),
        sources=sources,
        meta_rows=sum(len(b.rows) for b in plan.batches if b.schema == "meta"),
        entity_rows=sum(len(b.rows) for b in plan.batches if b.schema != "meta"),
    )


def build_database(
    url: str,
    decl: Declaration,
    inputs: Sequence[SourceInput] = (),
    *,
    tree: Path | None = None,
) -> BuildResult:
    """Replace the canonical schemas of the database `url` names with `decl` and `inputs`.

    With no inputs the result is the empty schema: the generated DDL, the reified declaration
    and the declared entities. Everything happens in one transaction; any refusal or failure,
    including one the commit's constraint validation raises, leaves the database as it was.
    """
    config.refuse_production(config.database_name(url))
    with tempfile.TemporaryDirectory(prefix="tk-build-") as work:
        plan = plan_build(decl, inputs, tree=tree, work=Path(work))
        built_at = datetime.now(UTC)
        database.apply(url, plan, built_at)
    return _result(plan, built_at)


def dry_run(
    decl: Declaration, inputs: Sequence[SourceInput] = (), *, tree: Path | None = None
) -> BuildResult:
    """What a build would hold, from the union and every consistency check, without touching
    any database."""
    plan = plan_build(decl, inputs, tree=tree)
    return _result(plan, datetime.now(UTC))
