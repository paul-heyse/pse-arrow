#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Deferred-decision register checker (standard library only, Python >= 3.11).

    python3 scripts/check_register.py --lint   validate every row
    python3 scripts/check_register.py --due    report the rows that are due

`docs/adr/register.md` carries one row per item that was deliberately deferred
with a stated trigger. `--lint` runs in `governance / adr-lint`; `--due` runs
monthly in `register-review.yml`, which opens or updates one
`Register review YYYY-MM` issue from its output.

A `check` cell that starts with `$ ` is a shell command. `--due` runs it and
prints what it said. A command that fails is reported, never fatal: the point of
the monthly pass is to surface what changed, not to break CI on a network blip.
"""

from __future__ import annotations

import argparse
import datetime
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REGISTER = ROOT / "docs" / "adr" / "register.md"
ADR_DIR = ROOT / "docs" / "adr"

ROW_RE = re.compile(r"^\|\s*(R-\d{2})\s*\|")
DATE_RE = re.compile(r"^\d{4}-\d{2}-\d{2}$")
ADR_REF_RE = re.compile(r"ADR-(\d{4})")
STATUSES = {"open", "watch", "closed"}
COLUMNS = (
    "id",
    "item",
    "adr",
    "trigger",
    "check",
    "owner",
    "last-checked",
    "next-check",
    "status",
)


def rows() -> list[dict[str, str]]:
    parsed: list[dict[str, str]] = []
    for lineno, line in enumerate(REGISTER.read_text(encoding="utf-8").splitlines(), 1):
        if not ROW_RE.match(line):
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) != len(COLUMNS):
            parsed.append(
                {
                    "id": cells[0],
                    "_lineno": str(lineno),
                    "_error": f"{len(cells)} cells, expected {len(COLUMNS)}",
                }
            )
            continue
        row = dict(zip(COLUMNS, cells, strict=True))
        row["_lineno"] = str(lineno)
        parsed.append(row)
    return parsed


def known_adrs() -> set[str]:
    return {f"ADR-{p.name[:4]}" for p in ADR_DIR.glob("[0-9][0-9][0-9][0-9]-*.md")}


def lint() -> int:
    today = datetime.date.today()
    errors: list[str] = []
    seen: set[str] = set()
    adrs = known_adrs()
    parsed = rows()
    if not parsed:
        print("error: docs/adr/register.md has no R-NN rows", file=sys.stderr)
        return 1

    for row in parsed:
        where = f"register.md:{row['_lineno']} {row['id']}"
        if "_error" in row:
            errors.append(f"{where}: {row['_error']}")
            continue
        if row["id"] in seen:
            errors.append(f"{where}: duplicate row id")
        seen.add(row["id"])
        if row["status"] not in STATUSES:
            errors.append(
                f"{where}: status {row['status']!r} not in {sorted(STATUSES)}"
            )
        for key in ("last-checked", "next-check"):
            if not DATE_RE.match(row[key]):
                errors.append(f"{where}: {key} {row[key]!r} must be YYYY-MM-DD")
        if not row["owner"]:
            errors.append(f"{where}: owner is empty")
        if not row["trigger"]:
            errors.append(f"{where}: trigger is empty")
        for ref in ADR_REF_RE.findall(row["adr"]):
            if f"ADR-{ref}" not in adrs:
                errors.append(f"{where}: references unknown ADR-{ref}")
        if DATE_RE.match(row["next-check"]) and row["status"] != "closed":
            due = datetime.date.fromisoformat(row["next-check"])
            if due < today:
                errors.append(
                    f"{where}: next-check {row['next-check']} is in the past and the row is not closed"
                )
        if (
            DATE_RE.match(row["last-checked"])
            and DATE_RE.match(row["next-check"])
            and datetime.date.fromisoformat(row["next-check"])
            < datetime.date.fromisoformat(row["last-checked"])
        ):
            errors.append(f"{where}: next-check precedes last-checked")

    for err in errors:
        print(f"error: {err}", file=sys.stderr)
    if errors:
        print(
            f"register lint: {len(errors)} problem(s) in {len(parsed)} row(s)",
            file=sys.stderr,
        )
        return 1
    print(f"register lint: {len(parsed)} row(s) OK")
    return 0


def run_check(command: str) -> str:
    try:
        proc = subprocess.run(
            command,
            shell=True,
            cwd=ROOT,
            capture_output=True,
            text=True,
            timeout=300,
            check=False,
        )
    except (
        OSError,
        subprocess.SubprocessError,
    ) as exc:  # pragma: no cover - environment dependent
        return f"    check could not be run: {exc}"
    out = (proc.stdout + proc.stderr).strip() or "(no output)"
    lines = ["    " + line for line in out.splitlines()[:20]]
    if proc.returncode != 0:
        lines.append(f"    (command exited {proc.returncode}; reported, not fatal)")
    return "\n".join(lines)


def due() -> int:
    today = datetime.date.today()
    pending = [
        row
        for row in rows()
        if "_error" not in row
        and DATE_RE.match(row["next-check"])
        and datetime.date.fromisoformat(row["next-check"]) <= today
        and row["status"] != "closed"
    ]
    print(f"# Register review {today:%Y-%m}")
    print()
    if not pending:
        print(f"No register rows are due on {today.isoformat()}.")
        return 0
    print(f"{len(pending)} row(s) due on {today.isoformat()}:")
    print()
    for row in pending:
        print(f"## {row['id']} — {row['item']}")
        print(f"- Decision record: {row['adr'] or '—'}")
        print(f"- Trigger: {row['trigger']}")
        print(
            f"- Owner: {row['owner']}  ·  last checked {row['last-checked']}  ·  due {row['next-check']}"
        )
        command = row["check"]
        if command.startswith("$ "):
            shell = command[2:].strip().strip("`")
            print(f"- Check: `{shell}`")
            print(run_check(shell))
        else:
            print(f"- Check (manual): {command}")
        print()
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="check_register.py",
        description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("--lint", action="store_true", help="validate every row")
    parser.add_argument(
        "--due", action="store_true", help="report and run the rows that are due"
    )
    args = parser.parse_args(argv)
    if not (args.lint or args.due):
        parser.error("choose --lint or --due")
    status = 0
    if args.lint:
        status |= lint()
    if args.due:
        status |= due()
    return status


if __name__ == "__main__":
    sys.exit(main())
