# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Projection of a declaration to PostgreSQL: generated DDL, `meta` rows, declared entities."""

from __future__ import annotations

from thermo_knowledge.generate.fingerprint import declaration_fingerprint, schema_fingerprint
from thermo_knowledge.generate.tree import (
    GENERATED_ROOTS,
    SCHEMA_PATH,
    Difference,
    compare_tree,
    generate,
    write_tree,
)

__all__ = [
    "GENERATED_ROOTS",
    "SCHEMA_PATH",
    "Difference",
    "compare_tree",
    "declaration_fingerprint",
    "generate",
    "schema_fingerprint",
    "write_tree",
]
