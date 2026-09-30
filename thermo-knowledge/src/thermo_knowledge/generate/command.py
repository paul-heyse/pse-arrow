# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The `tk generate` command, and the tree option and declaration loader the stage commands
share."""

from __future__ import annotations

from pathlib import Path

import typer

from thermo_knowledge import config
from thermo_knowledge.declaration import Declaration, load_declaration
from thermo_knowledge.generate.tree import compare_tree, generate, write_tree

TREE_OPTION = typer.Option(
    config.TREE_DIR,
    "--tree",
    help="The tree whose model/, forms/ and sql/ are read and written.",
    file_okay=False,
)


def _fail(message: str) -> typer.Exit:
    typer.echo(f"error: {message}", err=True)
    return typer.Exit(code=1)


def load_tree(tree: Path) -> Declaration:
    result = load_declaration(tree / "model", tree / "forms")
    if result.declaration is None or result.diagnostics:
        for diagnostic in result.diagnostics:
            typer.echo(str(diagnostic), err=True)
        raise _fail(f"the declaration was refused ({len(result.diagnostics)} diagnostics)")
    return result.declaration


def generate_command(
    check: bool = typer.Option(
        False, "--check", help="Compare with the files on disk and exit 1 on any difference."
    ),
    tree: Path = TREE_OPTION,
) -> None:
    """Generate the DDL from the declaration, or with --check compare it."""
    files = generate(load_tree(tree))
    if check:
        differences = compare_tree(files, tree)
        for difference in differences:
            typer.echo(str(difference))
        if differences:
            raise _fail(f"{len(differences)} generated files differ; run `tk generate`")
        typer.echo(f"generated tree is current ({len(files)} files)")
        return
    for difference in write_tree(files, tree):
        typer.echo(str(difference))
    typer.echo(f"generated {len(files)} files")
