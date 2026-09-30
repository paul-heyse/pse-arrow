# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Helpers shared by the declaration and generator tests."""

from __future__ import annotations

import shutil
from functools import cache
from pathlib import Path

from thermo_knowledge.declaration import Declaration, load_declaration

FIXTURES = Path(__file__).parent / "fixtures" / "declarations"
FULL = FIXTURES / "full"
BROKEN = FIXTURES / "broken"
NO_PHYSICAL = FIXTURES
"""A tree without `sql/physical.sql`. The committed physical layer belongs to the committed model
(its view reads `qual.qualification_run`), so a test that builds another declaration gives
`tree=NO_PHYSICAL`."""


@cache
def full_declaration() -> Declaration:
    """The valid fixture declaration, loaded once."""
    return load_declaration(FULL / "model", FULL / "forms", contract=None).require()


def copy_full(destination: Path) -> Path:
    """Copy the valid fixture (`model/` and `forms/`) under `destination`."""
    shutil.copytree(FULL / "model", destination / "model")
    shutil.copytree(FULL / "forms", destination / "forms")
    return destination


def overlay(case: str, destination: Path) -> Path:
    """The valid fixture with a broken case's files written over it."""
    copy_full(destination)
    shutil.copytree(BROKEN / case, destination, dirs_exist_ok=True)
    return destination


EMPTY_MANIFEST = 'doc = "An empty declaration."\n\n[framework]\n'


def empty_declaration(directory: Path) -> Declaration:
    """A declaration with a manifest and nothing else: the base schemas, `meta` and
    `prov.record` are all it projects to."""
    (directory / "model").mkdir(parents=True)
    (directory / "model" / "manifest.toml").write_text(EMPTY_MANIFEST)
    return load_declaration(directory / "model", directory / "forms", contract=None).require()
