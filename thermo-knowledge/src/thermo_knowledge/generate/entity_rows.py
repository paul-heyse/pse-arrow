# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Rows of the declared entities, one batch per table they populate, in table-name order.

An entity of a kind with `provenance = "own"` also registers its `prov.record` row."""

from __future__ import annotations

from dataclasses import dataclass, field

from thermo_knowledge.declaration import model as m


@dataclass
class TableRows:
    schema: str
    table: str
    columns: tuple[str, ...]
    rows: list[tuple[object, ...]] = field(default_factory=list)


def entity_rows(decl: m.Declaration) -> list[TableRows]:
    """The rows that insert `decl`'s declared entities: one row per kind table along each
    entity's refinement chain, and one per member of each `Set<K>` attribute."""
    batches: dict[tuple[str, str], TableRows] = {}

    def batch(schema: str, table: str, columns: tuple[str, ...]) -> TableRows:
        return batches.setdefault((schema, table), TableRows(schema, table, columns))

    for entity in decl.entities:
        if decl.kinds[entity.kind].provenance.mode == "own":
            # an instance of an `own` kind is registered like any other; it has no import record
            batch("prov", "record", ("id", "kind")).rows.append((entity.id, entity.kind))
        for kind in decl.chain(entity.kind):
            own = [a for a in kind.attributes if a.type.container != "set"]
            columns = ("id", *(a.name for a in own))
            row = (entity.id, *(_value(entity.values.get(a.name), a) for a in own))
            batch(kind.schema, kind.name, columns).rows.append(row)
            for attribute in kind.attributes:
                if attribute.type.container == "set":
                    child = batch(
                        kind.schema, f"{kind.name}__{attribute.name}", ("owner", "member")
                    )
                    for member in entity.values.get(attribute.name, ()):  # type: ignore[attr-defined]
                        child.rows.append((entity.id, member))
    return [batches[key] for key in sorted(batches)]


def _value(value: object, attribute: m.Field) -> object:
    if value is None:
        return None
    if attribute.type.container == "range":
        lower, upper = value  # type: ignore[misc]
        return f"[{lower!r},{upper!r}]"
    if attribute.type.container == "array":
        return list(value)  # type: ignore[call-overload]
    return value
