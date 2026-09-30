# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The generated tree: a pure function from a declaration, a writer and a comparison."""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

from thermo_knowledge.declaration.diagnostics import DeclarationError
from thermo_knowledge.declaration.model import Declaration
from thermo_knowledge.generate.plan import build_plan, projection_diagnostics, render_schema

SCHEMA_PATH = "sql/generated/schema.sql"
GENERATED_ROOTS: tuple[str, ...] = ("sql/generated",)
"""Directories (relative to the tree) whose files are all generated: a file there that the
generator does not produce is an extra difference."""

type Tree = Mapping[str, bytes]


@dataclass(frozen=True, order=True)
class Difference:
    """One way the files on disk differ from the generated tree."""

    path: str
    kind: Literal["missing", "extra", "changed"]

    def __str__(self) -> str:
        return f"{self.kind}: {self.path}"


def generate(decl: Declaration) -> dict[str, bytes]:
    """The generated tree: relative path to bytes. Pure; identical input, identical bytes.

    Raises `DeclarationError` when the declaration projects an identifier PostgreSQL would
    truncate or collide.
    """
    problems = projection_diagnostics(decl)
    if problems:
        raise DeclarationError(problems)
    return {SCHEMA_PATH: render_schema(build_plan(decl)).encode("utf-8")}


def compare_tree(tree: Tree, root: Path) -> list[Difference]:
    """Every difference between `tree` and the files under `root`, in both directions."""
    differences: list[Difference] = []
    for path, content in tree.items():
        target = root / path
        if not target.is_file():
            differences.append(Difference(path, "missing"))
        elif target.read_bytes() != content:
            differences.append(Difference(path, "changed"))
    for generated_root in GENERATED_ROOTS:
        base = root / generated_root
        if not base.is_dir():
            continue
        for file in base.rglob("*"):
            relative = file.relative_to(root).as_posix()
            if file.is_file() and relative not in tree:
                differences.append(Difference(relative, "extra"))
    return sorted(differences)


def write_tree(tree: Tree, root: Path) -> list[Difference]:
    """Make the files under `root` equal `tree`: write, and delete extra generated files.

    Returns the differences that were resolved.
    """
    differences = compare_tree(tree, root)
    for difference in differences:
        target = root / difference.path
        if difference.kind == "extra":
            target.unlink()
        else:
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(tree[difference.path])
    return differences
