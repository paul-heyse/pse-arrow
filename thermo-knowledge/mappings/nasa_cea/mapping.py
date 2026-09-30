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

from thermo_knowledge.mapping.context import IdentityContext, RecordContext
from thermo_knowledge.mapping.formula import composition, hill

NASA9 = "nasa9.pure"
PIECE = "nasa9.pure.piece"
ASSIGNED = "assigned_enthalpy.pure"


def identities(ctx: IdentityContext) -> None:
    pairs = ctx.group("species_formula_pairs", "species_locator")
    for record in ctx.rows("species_records"):
        rows = pairs.get((record.locator,), [])
        with ctx.emit(record, *rows) as emit:
            decoded = ctx.decoded(record, "phase_flag")
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
    occurrences: dict[str, int] = {}
    for record in ctx.rows("species_records"):
        name = record["name"]
        subject = lambda: ctx.subject("species", name)  # noqa: E731
        with ctx.emit(record, *pairs.get((record.locator,), [])) as emit:
            form = subject()
            for pair in pairs.get((record.locator,), []):
                found = ctx.attributes(pair, "composition")
                emit.relation(
                    "composition",
                    {"entity": form, "quantity": found["quantity"]},
                    {"value": found["value"]},
                )
        rows = sorted(
            intervals.get((record.locator,), []),
            key=lambda row: row["interval_index"],  # type: ignore[arg-type,return-value]
        )
        occurrence = occurrences[name] = occurrences.get(name, 0) + 1  # type: ignore[arg-type]
        with ctx.emit(record, *rows) as emit:
            if record["interval_count"] == 0:
                (line,) = rows
                emit.parameter_set(
                    parameterization=emit.parameterization("thermo_inp"),
                    slot_group=ASSIGNED,
                    subjects=[subject()],
                    slots={**ctx.slot_values(record, ASSIGNED), **ctx.slot_values(line, ASSIGNED)},
                    occurrence=occurrence,
                )
                continue
            pieces = [ctx.family_row(row, PIECE) for row in rows]
            parameter_set = emit.parameter_set(
                parameterization=emit.parameterization("thermo_inp"),
                slot_group=NASA9,
                subjects=[subject()],
                slots={},
                families={"piece": pieces},
                occurrence=occurrence,
            )
            clause = {
                **ctx.attributes(record, "region_clause"),
                "lower": pieces[0].values["T_low"],
                "upper": pieces[-1].values["T_high"],
            }
            region = emit.validity(
                parameter_set, ctx.attributes(record, "validity_region"), [clause]
            )
            emit.derivation([parameter_set, region])
