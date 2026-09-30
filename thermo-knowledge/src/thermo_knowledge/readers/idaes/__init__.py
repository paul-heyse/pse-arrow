# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Reader of the IDAES-PSE property-package modules (`sources/idaes.toml`, tag 2.13.0).

Clean-room and values only. The modules are parsed with `ast` (never imported) and only literal
parameter values are read, keyed by module, scope and target, with the file and position as
locator; no code, docstring or comment of the source is copied into a row.

| Payload | Tables |
|---|---|
| literal dict, list and tuple assignments (configuration dictionaries with their `parameter_data`, stoichiometries, correlation coefficient tables) and, in the data-definition modules, numeric assignments | `literal_entries` |
| `self.<name> = Param(...)` / `Var(...)` of the data-definition modules | `parameter_declarations` |

Every non-test module of `idaes/models/properties` is parsed; one that holds no literal data is
accounted as read without rows. The test modules are skipped: they exercise the packages and
their configurations are fixtures, not package data.
"""

from __future__ import annotations

import ast
from collections.abc import Callable
from pathlib import Path

import pyarrow as pa

from thermo_knowledge.readers.idaes import declarations, literals
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.writer import Writer

READER_VERSION = "1"

TABLES: dict[str, pa.Schema] = {**literals.SCHEMAS, **declarations.SCHEMAS}

TESTS = "test modules exercise the property packages; their configurations are test fixtures, not package data"
BATCH = 20_000
Visit = Callable[[ast.stmt, str, str], None]


def is_test(artifact: str) -> bool:
    return "/tests/" in artifact or Path(artifact).name.startswith("test_")


def _bodies(statement: ast.stmt) -> list[list[ast.stmt]]:
    """The nested statement lists of a compound statement (not class and function bodies)."""
    lists: list[list[ast.stmt]] = []
    for name in ("body", "orelse", "finalbody"):
        value = getattr(statement, name, None)
        if isinstance(value, list):
            lists.append(value)
    for handler in getattr(statement, "handlers", []):
        lists.append(handler.body)
    for case in getattr(statement, "cases", []):
        lists.append(case.body)
    return lists


def walk(body: list[ast.stmt], kind: str, scope: str, visit: Visit) -> None:
    for statement in body:
        visit(statement, kind, scope)
        if isinstance(statement, ast.ClassDef):
            walk(statement.body, "class", _join(scope, statement.name), visit)
        elif isinstance(statement, ast.FunctionDef | ast.AsyncFunctionDef):
            walk(statement.body, "function", _join(scope, statement.name), visit)
        else:
            for nested in _bodies(statement):
                walk(nested, kind, scope, visit)


def _join(scope: str, name: str) -> str:
    return name if not scope else f"{scope}.{name}"


def read(tree: Path, writer: Writer) -> None:
    """Parse every non-test module and emit its literal parameter data."""
    buffers: dict[str, list[dict[str, object]]] = {name: [] for name in TABLES}

    def flush(table: str) -> None:
        if buffers[table]:
            writer.rows(table, buffers[table])
            buffers[table] = []

    for artifact in writer.payload_files:
        if is_test(artifact):
            writer.skipped(artifact, TESTS)
            continue
        try:
            source = (tree / artifact).read_text(encoding="utf-8")
            module = ast.parse(source, filename=artifact)
        except (OSError, UnicodeDecodeError) as error:
            raise StagingError(f"{artifact}: cannot be read: {error}") from error
        except SyntaxError as error:
            raise StagingError(
                f"{artifact}#L{error.lineno}: cannot be parsed as Python: {error.msg}"
            ) from error
        data = artifact.startswith(literals.DATA_MODULES)

        def visit(
            statement: ast.stmt,
            kind: str,
            scope: str,
            artifact: str = artifact,
            data: bool = data,
            source: str = source,
        ) -> None:
            buffers["literal_entries"].extend(
                literals.literal_rows(artifact, statement, kind, scope, data, source)
            )
            if data:
                row = declarations.declaration_row(artifact, statement, kind, scope)
                if row is not None:
                    buffers["parameter_declarations"].append(row)

        walk(module.body, "module", "", visit)
        writer.opened(artifact)
        for table in buffers:
            if len(buffers[table]) >= BATCH:
                flush(table)
    for table in buffers:
        flush(table)
