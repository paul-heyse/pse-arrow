# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The check files of `sql/verify/` and their agreement with the declaration (pipeline
section 4).

A check is one file, `sql/verify/<name>.sql`:

```sql
-- invariant: species.charge_matches_composition
-- One line saying what a violation is.
SELECT s.id, ...            -- the violating rows; zero rows is a pass
```

The first line names what the check enforces: `-- invariant: <kind or relation>.<requirement>`
for a requirement of a kind or a relation that the declaration marks `enforced = "verify"`, or `-- structural: <name>` for one of
the structural checks. The second is the description. The rest is the query, which returns the
violating rows with an `id` column that locates the record, a `locator` column where the record
has an origin, and whatever else describes the violation.

A check that needs a helper (a function over the reified declaration, since a table name the
declaration decides cannot be written in a static query) puts a line `-- query` before the
query; everything above it is a setup script that runs first, in the transaction the check runs
in and rolls back, so a helper created in `pg_temp` leaves nothing behind.

A declared verify requirement with no check file, and a check file that names a requirement the
declaration does not declare as `verify`, are refusals of their own (`problems`).
"""

from __future__ import annotations

import re
from collections.abc import Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

from thermo_knowledge.declaration import model as m

VERIFY_DIR = "sql/verify"
"""Check files, relative to the tree."""

_HEADER = re.compile(r"^--\s*(?P<kind>invariant|structural):\s*(?P<target>\S+)\s*$")
_QUERY_MARK = re.compile(r"^--\s*query\s*$", re.MULTILINE)

type Kind = Literal["invariant", "structural"]


@dataclass(frozen=True)
class Check:
    """One check file."""

    name: str
    """The file's name without `.sql`."""
    kind: Kind
    target: str
    """The `<kind or relation>.<requirement>` an invariant check enforces, or a structural check's name."""
    description: str
    setup: str
    query: str
    path: Path


def parse(path: Path) -> Check | str:
    """The check a file declares, or the problem with it."""
    lines = path.read_text(encoding="utf-8").splitlines()
    found = _HEADER.match(lines[0].strip()) if lines else None
    if found is None:
        return (
            f"{path.name}: the first line must be `-- invariant: <kind or relation>.<requirement>` or "
            "`-- structural: <name>`"
        )
    if len(lines) < 2 or not lines[1].startswith("--") or not lines[1].lstrip("- ").strip():
        return f"{path.name}: the second line must be a `--` comment describing a violation"
    description = lines[1].lstrip("- ").strip()
    body = "\n".join(lines[2:])
    marker = _QUERY_MARK.search(body)
    setup, query = (body[: marker.start()], body[marker.end() :]) if marker else ("", body)
    query = query.strip().rstrip(";").strip()
    if not query or all(
        line.lstrip().startswith("--") or not line.strip() for line in query.splitlines()
    ):
        return f"{path.name}: the file has no query"
    kind: Kind = "invariant" if found.group("kind") == "invariant" else "structural"
    return Check(path.stem, kind, found.group("target"), description, setup.strip(), query, path)


def load_checks(directory: Path) -> tuple[list[Check], list[str]]:
    """Every check file under `directory` (in name order) and the problems with the files."""
    checks: list[Check] = []
    problems: list[str] = []
    for path in sorted(directory.glob("*.sql")) if directory.is_dir() else []:
        parsed = parse(path)
        if isinstance(parsed, str):
            problems.append(parsed)
        else:
            checks.append(parsed)
    seen: dict[tuple[str, str], str] = {}
    for check in checks:
        key = (check.kind, check.target)
        if key in seen:
            problems.append(
                f"{check.name}.sql and {seen[key]}.sql both enforce the {check.kind} "
                f"`{check.target}`"
            )
        else:
            seen[key] = check.name
    return checks, problems


def verify_invariants(decl: m.Declaration) -> dict[str, m.Requirement]:
    """The requirements of kinds and relations the declaration marks `enforced = "verify"`, by
    `<kind or relation>.<name>`."""
    owners: list[tuple[str, tuple[m.Requirement, ...]]] = [
        *((kind.name, kind.requires) for kind in decl.kinds.values()),
        *((relation.name, relation.requires) for relation in decl.relations.values()),
    ]
    return {
        f"{owner}.{requirement.name}": requirement
        for owner, requires in owners
        for requirement in requires
        if requirement.enforced == "verify"
    }


def agreement_problems(decl: m.Declaration, checks: Sequence[Check]) -> list[str]:
    """A verify invariant with no check file, and a check file naming an invariant that is not
    declared as `verify`."""
    declared = verify_invariants(decl)
    named = {check.target for check in checks if check.kind == "invariant"}
    problems = [
        f"the declared verify invariant `{name}` has no check file in {VERIFY_DIR}/"
        for name in sorted(set(declared) - named)
    ]
    for check in checks:
        if check.kind == "invariant" and check.target not in declared:
            problems.append(
                f"{check.name}.sql names the invariant `{check.target}`, which the declaration "
                'does not declare with `enforced = "verify"`'
            )
    return problems
