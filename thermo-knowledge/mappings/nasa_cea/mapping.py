# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Structure of the NASA CEA mapping (rules, units and constants are in mapping.toml).

Phase 1: each record of `species_records` is one source entity, a species form whose aggregation
and polymorph the decoding of its phase flag and name gives, whose charge is that of the
pseudo-element E among its formula pairs, and which asserts its name and the Hill formula of the
pairs. A record that names an element the declaration does not hold, or whose phase the decoding
does not cover, holds its block.

Phase 2: each pair becomes a `composition` row of the record's form; a fitted record becomes one
`nasa9.pure` set of its form with a piece for each interval, the fitted range as its validity
region and the `fit` derivation that produced both; a record without coefficients becomes an
`assigned_enthalpy.pure` set. Records that repeat a name are repeated assertions, numbered in file
order.
"""

from __future__ import annotations

from collections import Counter

from thermo_knowledge.mapping.context import IdentityContext, RecordContext, first, last
from thermo_knowledge.mapping.formula import composition, hill
from thermo_knowledge.mapping.staged import SourceRow

NASA9 = "nasa9.pure"
ASSIGNED = "assigned_enthalpy.pure"


def identities(ctx: IdentityContext) -> None:
    pairs = ctx.group("species_formula_pairs", "species_locator")
    for record in ctx.rows("species_records"):
        rows = pairs.get((record.locator,), [])
        with ctx.emit(record, *rows) as emit:
            decoded = ctx.decoded_by(record, "aggregation_of_record")
            counts, charge = composition(ctx, rows, symbol="symbol")
            entity = emit.source_entity(
                "species",
                aggregation=decoded.to,
                polymorph=decoded.polymorph,
                stated_charge=charge,
            )
            emit.assertion(entity, "name")
            emit.assertion(entity, "derived:formula", hill(counts))


def records(ctx: RecordContext) -> None:
    pairs = ctx.group("species_formula_pairs", "species_locator")
    intervals = ctx.group("thermo_intervals", "species_locator")
    piece = ctx.family_of(NASA9)
    occurrences: Counter[object] = Counter()

    def lines_of(record: SourceRow) -> list[SourceRow]:
        return sorted(
            intervals.get((record.locator,), []),
            key=lambda row: row["interval_index"],  # type: ignore[arg-type,return-value]
        )

    for record in ctx.rows("species_records"):
        rows = pairs.get((record.locator,), [])
        with ctx.emit(record, *rows) as emit:
            form = ctx.subject("species", str(record["name"]))
            for pair in rows:
                found = ctx.attributes(pair, "composition")
                emit.relation(
                    "composition",
                    {"entity": form, "quantity": found["quantity"]},
                    {"value": found["value"]},
                )
        occurrences.update([record["name"]])
    occurrences.clear()
    for record in ctx.rows("species_records", "reactant_only"):
        lines = lines_of(record)
        occurrences.update([record["name"]])
        with ctx.emit(record, *lines) as emit:
            emit.parameter_set(
                parameterization=emit.parameterization("thermo_inp"),
                slot_group=ASSIGNED,
                subjects=[ctx.subject("species", str(record["name"]))],
                slots={
                    **ctx.slot_values(record, ASSIGNED),
                    **ctx.slot_values(first(lines), ASSIGNED),
                },
                occurrence=occurrences[record["name"]],
            )
    occurrences.clear()
    for record in ctx.rows("species_records", "fitted"):
        lines = lines_of(record)
        occurrences.update([record["name"]])
        with ctx.emit(record, *lines) as emit:
            pieces = [ctx.family_row(row, piece) for row in lines]
            parameter_set = emit.parameter_set(
                parameterization=emit.parameterization("thermo_inp"),
                slot_group=NASA9,
                subjects=[ctx.subject("species", str(record["name"]))],
                slots={},
                families={ctx.family_key(piece): pieces},
                occurrence=occurrences[record["name"]],
            )
            clause = {
                **ctx.attributes(record, "region_clause"),
                "lower": first(pieces).values["T_low"],
                "upper": last(pieces).values["T_high"],
            }
            region = emit.validity(
                parameter_set, ctx.attributes(record, "validity_region"), [clause]
            )
            emit.derivation([parameter_set, region])
