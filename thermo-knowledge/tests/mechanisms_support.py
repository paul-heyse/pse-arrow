# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Helpers for the tests of the TK2g model mechanisms: the committed declaration plus the fixture
forms and vocabulary of `fixtures/mechanisms`, and a way to break one file of it. Contains no
tests."""

from __future__ import annotations

import shutil
import uuid
from collections.abc import Callable, Mapping
from pathlib import Path

from thermo_knowledge import config
from thermo_knowledge.declaration import Declaration, LoadResult, load_declaration

FIXTURES = Path(__file__).parent / "fixtures" / "mechanisms"


def mechanism_directories(
    destination: Path, edits: Mapping[str, Callable[[str], str]] | None = None
) -> tuple[Path, Path]:
    """Copies of the committed `model/` and `forms/` with the fixture files added. `edits` maps
    a path relative to the copy (`model/reactions.toml`, `forms/rate_forms.toml`) to a function
    that rewrites its text."""
    shutil.copytree(config.TREE_DIR / "model", destination / "model")
    shutil.copytree(config.TREE_DIR / "forms", destination / "forms")
    for part in ("model", "forms"):
        for path in (FIXTURES / part).glob("*.toml"):
            shutil.copy(path, destination / part / path.name)
    for relative, edit in (edits or {}).items():
        target = destination / relative
        text = target.read_text()
        edited = edit(text)
        assert edited != text, f"the edit of {relative} changed nothing"
        target.write_text(edited)
    return destination / "model", destination / "forms"


def mechanism_declaration(destination: Path) -> Declaration:
    """The committed declaration plus every fixture file, which must load cleanly."""
    model, forms = mechanism_directories(destination)
    return load_declaration(model, forms).require()


def broken(destination: Path, edits: Mapping[str, Callable[[str], str]]) -> LoadResult:
    """The same declaration with `edits` applied, loaded without raising."""
    model, forms = mechanism_directories(destination, edits)
    return load_declaration(model, forms)


def replace(old: str, new: str) -> Callable[[str], str]:
    """An edit that replaces the one occurrence of `old`."""

    def edit(text: str) -> str:
        assert text.count(old) == 1, f"{old!r} occurs {text.count(old)} times"
        return text.replace(old, new)

    return edit


def entity_id(decl: Declaration, kind: str, name: str) -> uuid.UUID:
    return next(e.id for e in decl.entities if e.kind == kind and e.name == name)
