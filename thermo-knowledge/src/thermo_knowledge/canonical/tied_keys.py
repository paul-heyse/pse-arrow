# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Text keys that restate references, computed by the writer (meta-model section 5).

An identity is one tuple of values, so where a kind is hosted by one of two references (a site
class by a phase definition or a material, an association site by an entity or a group) the
identity holds one text column that restates whichever reference is present. A mapping supplies
the reference and never the key: the writer computes the key as the identifier text of the
reference (lowercase hyphenated UUID, the form of `identity.canonical_encoding`), and a verify
invariant of the kind (`host_key_matches_host`, `carrier_key_matches_carrier`) holds the two
together in the database.
"""

from __future__ import annotations

import uuid
from collections.abc import Mapping
from dataclasses import dataclass

from thermo_knowledge import pipeline_contract as contract
from thermo_knowledge.canonical.values import Problems
from thermo_knowledge.declaration import model as m


@dataclass(frozen=True)
class TiedKey:
    """A text attribute of a kind that is the identifier text of the one reference present among
    `references`."""

    kind: str
    key: str
    references: tuple[str, ...]


TIED: tuple[TiedKey, ...] = (
    TiedKey(
        contract.SITE_CLASS.declared,
        contract.SITE_CLASS.host_key,
        (contract.SITE_CLASS.phase, contract.SITE_CLASS.material),
    ),
    TiedKey(
        contract.ASSOCIATION_SITE.declared,
        contract.ASSOCIATION_SITE.carrier_key,
        (contract.ASSOCIATION_SITE.on_entity, contract.ASSOCIATION_SITE.on_group),
    ),
)


def complete(
    decl: m.Declaration, kind: str, values: Mapping[str, object], problems: Problems
) -> dict[str, object]:
    """`values` of an instance of `kind` with its tied keys computed. A key the mapping gave, or
    a reference count other than one, is a problem (the latter is also the declared check of
    the kind, which names it in its own words once the key exists)."""
    completed = dict(values)
    chain = {link.name for link in decl.chain(kind)}
    for tie in TIED:
        if tie.kind not in chain:
            continue
        if values.get(tie.key) is not None:
            problems.add(
                tie.key,
                f"is computed by the writer from {' or '.join(tie.references)}: give the reference, "
                "not the key",
            )
            continue
        present = [values[name] for name in tie.references if values.get(name) is not None]
        if len(present) == 1 and isinstance(present[0], uuid.UUID):
            completed[tie.key] = str(present[0])
        elif len(present) != 1:
            problems.add(
                tie.key,
                f"is the identifier of the one of {', '.join(tie.references)} that is present, "
                f"and {len(present)} are",
            )
    return completed
