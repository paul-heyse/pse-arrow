# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Helpers shared by the expression tests: the fixture declarations under
`fixtures/declarations/expressions/`, each loaded against the valid fixture's model plus the
quantity types of `expressions/model/`."""

from __future__ import annotations

import atexit
import shutil
import tempfile
from functools import cache
from pathlib import Path

from declaration_support import FULL, FIXTURES

from thermo_knowledge.declaration import Declaration, LoadResult, load_declaration

EXPRESSIONS = FIXTURES / "expressions"
VALID = EXPRESSIONS / "valid"
BROKEN = EXPRESSIONS / "broken"


@cache
def model_dir() -> Path:
    """The model the expression fixtures use: the valid fixture's, plus extra quantity types.
    Built once per test run in a temporary directory."""
    root = Path(tempfile.mkdtemp(prefix="tk-expression-model-"))
    atexit.register(shutil.rmtree, root, ignore_errors=True)
    shutil.copytree(FULL / "model", root / "model")
    for extra in (EXPRESSIONS / "model").glob("*.toml"):
        shutil.copy(extra, root / "model" / extra.name)
    return root / "model"


def load(forms_dir: Path) -> LoadResult:
    return load_declaration(model_dir(), forms_dir, contract=None)


@cache
def scenario(name: str) -> Declaration:
    """The declaration of `valid/<name>`, loaded once."""
    return load(VALID / name).require()
