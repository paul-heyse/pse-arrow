# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The declaration meta-model: typed structs, the loader and the loaded model."""

from __future__ import annotations

from thermo_knowledge.declaration.diagnostics import Code, DeclarationError, Diagnostic
from thermo_knowledge.declaration.loader import (
    LoadResult,
    default_forms_dir,
    default_model_dir,
    load_declaration,
)
from thermo_knowledge.declaration.model import Declaration

__all__ = [
    "Code",
    "Declaration",
    "DeclarationError",
    "Diagnostic",
    "LoadResult",
    "default_forms_dir",
    "default_model_dir",
    "load_declaration",
]
