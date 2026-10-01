# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Run the checks against a built database and report (pipeline section 4).

Each check runs in a transaction that is rolled back, so a setup script's helpers vanish and
nothing is written. The violations are counted in the database and only the first few are
fetched, so a check over millions of offending rows costs a count, not their transfer.
"""

from __future__ import annotations

import json
from collections.abc import Sequence
from dataclasses import dataclass, field
from pathlib import Path

import psycopg

from thermo_knowledge.verify.checks import Check

SHOWN = 10
"""Violating rows a result keeps (ordered by `id`)."""
_IDS_IN_TABLE = 3
REPORT_SCHEMA = 1
REPORT_NAME = "verify-report.json"


@dataclass(frozen=True)
class CheckResult:
    check: Check
    violations: int
    columns: tuple[str, ...] = ()
    rows: tuple[tuple[str, ...], ...] = ()
    error: str | None = None

    @property
    def passed(self) -> bool:
        return self.error is None and self.violations == 0

    @property
    def ids(self) -> list[str]:
        """The `id` of each kept row."""
        if "id" not in self.columns:
            return []
        position = self.columns.index("id")
        return [row[position] for row in self.rows]


@dataclass(frozen=True)
class Report:
    database: str
    fingerprint: str | None
    results: tuple[CheckResult, ...]
    problems: tuple[str, ...] = field(default=())
    """Declared invariants with no check, check files naming no invariant, unreadable files."""

    @property
    def passed(self) -> bool:
        return not self.problems and all(result.passed for result in self.results)

    def as_json(self) -> dict[str, object]:
        return {
            "schema": REPORT_SCHEMA,
            "database": self.database,
            "fingerprint": self.fingerprint,
            "passed": self.passed,
            "problems": list(self.problems),
            "checks": [
                {
                    "name": result.check.name,
                    "kind": result.check.kind,
                    "target": result.check.target,
                    "description": result.check.description,
                    "violations": result.violations,
                    "error": result.error,
                    "columns": list(result.columns),
                    "first_rows": [list(row) for row in result.rows],
                }
                for result in self.results
            ],
        }


def _text(value: object) -> str:
    return "" if value is None else str(value)


def _over(check: Check, before: str, after: str) -> bytes:
    """The check's query between `before` and `after`: its text comes from a declared check file,
    not from a value, so it is passed as encoded text (a `psycopg` query given as bytes)."""
    return f"{before}{check.query}\n{after}".encode()


def run_check(conn: psycopg.Connection, check: Check, *, shown: int = SHOWN) -> CheckResult:
    """Run one check on `conn` in a rolled-back transaction (a savepoint when `conn` already
    has one, so uncommitted rows are visible)."""
    try:
        with conn.transaction(force_rollback=True):
            if check.setup:
                conn.execute(check.setup.encode())
            described = conn.execute(_over(check, "SELECT * FROM (", ") AS v LIMIT 0"))
            columns = tuple(column.name for column in described.description or ())
            if "id" not in columns:
                return CheckResult(check, 0, columns, error="the query returns no `id` column")
            counted = conn.execute(_over(check, "SELECT count(*) FROM (", ") AS v")).fetchone()
            assert counted is not None
            rows = conn.execute(
                _over(check, "SELECT * FROM (", f') AS v ORDER BY "id" LIMIT {shown}')
            ).fetchall()
    except psycopg.Error as error:
        return CheckResult(check, 0, error=str(error).strip())
    return CheckResult(
        check,
        int(counted[0]),
        columns,
        tuple(tuple(_text(value) for value in row) for row in rows),
    )


def run_checks(
    conn: psycopg.Connection, checks: Sequence[Check], *, shown: int = SHOWN
) -> list[CheckResult]:
    """Every check, in the order given; a check that fails to run is a result, not an
    exception, and the others still run."""
    return [run_check(conn, check, shown=shown) for check in checks]


def table_lines(report: Report) -> list[str]:
    """The report as a table: check, kind, violations and the first offending identifiers."""
    header = ("check", "kind", "violations", "first offending ids")
    body: list[tuple[str, str, str, str]] = []
    for result in report.results:
        if result.error is not None:
            count, ids = "error", result.error.splitlines()[0]
        else:
            count = str(result.violations)
            ids = ", ".join(result.ids[:_IDS_IN_TABLE])
            if result.violations > _IDS_IN_TABLE:
                ids += ", ..."
        body.append((result.check.target, result.check.kind, count, ids))
    widths = [max(len(row[i]) for row in [header, *body]) for i in range(3)]
    lines = [
        "  ".join(cell.ljust(widths[i]) for i, cell in enumerate(header[:3])) + "  " + header[3]
    ]
    lines.extend(
        "  ".join(cell.ljust(widths[i]) for i, cell in enumerate(row[:3])) + "  " + row[3]
        for row in body
    )
    return [line.rstrip() for line in lines]


def write_report(report: Report, path: Path) -> None:
    """The report as JSON with sorted keys, beside the canonical store."""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(report.as_json(), indent=2, sort_keys=True, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
