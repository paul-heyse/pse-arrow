# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Where `tk map` and `tk resolve` find their inputs and put their outputs, in one value, so a
test can point every location at a fixture."""

from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path

from thermo_knowledge import config
from thermo_knowledge.declaration import Declaration, load_declaration
from thermo_knowledge.generate.fingerprint import declaration_fingerprint
from thermo_knowledge.schema_build import read_physical
from thermo_knowledge.staging.stage import StageContext

RESOLUTION_DIR = "_resolution"
"""The resolution result's directory under the canonical store."""


@dataclass(frozen=True)
class Environment:
    """The locations of one run."""

    canonical_dir: Path = field(default_factory=config.canonical_dir)
    tree: Path = config.TREE_DIR
    stage: StageContext = field(default_factory=StageContext)
    model_dir: Path | None = None
    forms_dir: Path | None = None
    sources_dir: Path | None = None
    lock_path: Path | None = None
    decisions_path: Path | None = None

    @property
    def mappings_dir(self) -> Path:
        return self.tree / "mappings"

    @property
    def resolution_dir(self) -> Path:
        return self.canonical_dir / RESOLUTION_DIR

    def source_dir(self, source_id: str) -> Path:
        return self.canonical_dir / source_id

    def declaration(self) -> Declaration:
        return load_declaration(self.model_dir, self.forms_dir).require()

    def fingerprint(self, decl: Declaration) -> str:
        """The schema fingerprint of `decl`: what a change of the declaration changes."""
        return declaration_fingerprint(decl, read_physical(self.tree))
