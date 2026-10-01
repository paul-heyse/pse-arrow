# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Which formula-identified entities share a formula and are not one species (the evidence of
`identity/decisions.toml` for the thermochemical sources). Contains no tests.

A formula and a charge do not identify a species of a thermochemical source when that source
itself lists several species under them: isomers (n-butanol and diethyl ether), the electronic states
of a radical, the isomeric radicals of a combustion mechanism. The evidence is the source's own
listing: two entities of one unit (a Cantera file, the NASA CEA file, the JANAF tables) that have
the same formula, charge, aggregation and polymorph and different names are different species, so
neither is identified by the formula. The name is the source's own name of the entity, compared
without regard to case; a JANAF table name is cut at its first comma (`Titanium Oxide, Rutile` and
`Titanium Oxide, Anatase` are one substance in two polymorphs, which the JANAF tables do not
state).
"""

from __future__ import annotations

from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path

from thermo_knowledge.mapping import claims

NAME_COLUMN = {"cantera": "name", "nasa_cea": "name", "janaf": "substance_name"}
"""The column of each source that holds the name of an entity."""


@dataclass(frozen=True)
class Entry:
    carrier: str
    scope: str
    key: str
    formula: str
    charge: int
    aggregation: str
    polymorph: str | None
    name: str

    @property
    def unit(self) -> tuple[str, str]:
        """The listing that distinguishes species: a Cantera file, else the whole source."""
        return (self.carrier, self.key.split("#")[0] if self.carrier == "cantera" else "")

    @property
    def form(self) -> tuple[str, int, str, str | None]:
        return (self.formula, self.charge, self.aggregation, self.polymorph)

    @property
    def species(self) -> str:
        found = self.name.split(",")[0] if self.carrier == "janaf" else self.name
        return found.casefold().strip()


def entries(canonical: Path, carriers: tuple[str, ...]) -> list[Entry]:
    """The formula-identified entities of the phase-1 output of `carriers` under `canonical`."""
    found: list[Entry] = []
    for carrier in carriers:
        directory = canonical / carrier / claims.IDENTITY_DIR
        assertions: dict[tuple[str, str], dict[str, list[str]]] = defaultdict(
            lambda: defaultdict(list)
        )
        for claim in claims.read_assertion_claims(directory):
            assertions[(claim.scope, claim.local_key)][claim.column].append(claim.value)
            if claim.scheme == "formula":
                assertions[(claim.scope, claim.local_key)]["formula"].append(claim.value)
        seen: set[tuple[str, str]] = set()
        for entity in claims.read_entity_claims(directory):
            key = (entity.scope, entity.local_key)
            if key in seen:
                continue
            seen.add(key)
            values = assertions[key]
            formulas = set(values.get("formula", []))
            if len(formulas) != 1:
                continue  # no formula, or several: the formula rule does not identify it
            (name,) = set(values[NAME_COLUMN[carrier]]) or {entity.local_key}
            found.append(
                Entry(
                    carrier,
                    entity.scope,
                    entity.local_key,
                    formulas.pop(),
                    entity.stated_charge or 0,
                    entity.aggregation or "",
                    entity.polymorph,
                    name,
                )
            )
    return found


def rejections(found: list[Entry]) -> dict[tuple[str, str, str], tuple[Entry, list[str]]]:
    """The entities the formula does not identify, with the distinct species names their unit
    lists under their formula, charge, aggregation and polymorph."""
    groups: dict[tuple[object, ...], list[Entry]] = defaultdict(list)
    for entry in found:
        groups[(entry.unit, entry.form)].append(entry)
    result: dict[tuple[str, str, str], tuple[Entry, list[str]]] = {}
    for members in groups.values():
        distinct = sorted({member.species for member in members})
        if len(distinct) > 1:
            for member in members:
                result[(member.carrier, member.scope, member.key)] = (member, distinct)
    return result
