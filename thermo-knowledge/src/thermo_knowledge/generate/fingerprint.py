# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The schema fingerprint of meta-model section 7."""

from __future__ import annotations

import hashlib
import re

from thermo_knowledge.declaration.model import Declaration

FINGERPRINT_PREFIX = "tk-schema-fingerprint sha256:"
"""Precedes the digest in the comment on schema `tk`."""

_RECORDED = re.compile(rf"{re.escape(FINGERPRINT_PREFIX)}([0-9a-f]{{64}})")


def schema_fingerprint(ddl: bytes, physical: bytes, reified: bytes) -> str:
    """SHA-256 over the generated DDL, `sql/physical.sql` (empty when there is none) and the
    canonical serialisation of the reified rows and declared entities.

    The parts are length-prefixed so that no split of the same bytes gives the same digest.
    """
    digest = hashlib.sha256()
    for part in (ddl, physical, reified):
        digest.update(len(part).to_bytes(8, "big"))
        digest.update(part)
    return digest.hexdigest()


def declaration_fingerprint(decl: Declaration, physical: bytes) -> str:
    """The fingerprint a database built from `decl` and `physical` records."""
    # Imported here: `tree` and `reified` build on the declaration, like this module.
    from thermo_knowledge.generate.reified import insert_batches, serialise
    from thermo_knowledge.generate.tree import SCHEMA_PATH, generate

    return schema_fingerprint(
        generate(decl)[SCHEMA_PATH], physical, serialise(insert_batches(decl))
    )


def fingerprint_comment(schema_doc: str, fingerprint: str) -> str:
    """The comment on schema `tk`: its description, then the fingerprint."""
    return f"{schema_doc}\n{FINGERPRINT_PREFIX}{fingerprint}"


def recorded_fingerprint(comment: str | None) -> str | None:
    """The fingerprint a schema comment records, or `None`."""
    if comment is None:
        return None
    found = _RECORDED.search(comment)
    return found.group(1) if found else None
