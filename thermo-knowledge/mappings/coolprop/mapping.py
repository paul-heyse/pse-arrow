# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Structure of the CoolProp mapping (rules, units and constants are in mapping.toml).

Phase 1: each row of `fluids` is one source entity with an identity assertion for every
identifier the row carries (a list-valued column asserts each of its values).

Phase 2: each saturation-pressure curve (partition `saturation_pressure` of
`ancillary_equations`) becomes one parameter set of `vapor_pressure_exp_series_tau.pure` for its
fluid, with one `term` row per coefficient row, one envelope on the set and the `fit` derivation
that produced it.
"""

from __future__ import annotations

from thermo_knowledge.mapping.context import IdentityContext, RecordContext

IDENTIFIER_COLUMNS = (
    "NAME",
    "CAS",
    "REFPROP_NAME",
    "FORMULA",
    "INCHI_STRING",
    "INCHI_KEY",
    "SMILES",
)
LIST_COLUMNS = ("ALIASES",)


def identities(ctx: IdentityContext) -> None:
    for row in ctx.rows("fluids"):
        with ctx.emit(row) as emit:
            entity = emit.source_entity("fluids")
            for column in IDENTIFIER_COLUMNS:
                emit.assertion(entity, column)
            for column in LIST_COLUMNS:
                for alias in row[column] or []:  # type: ignore[attr-defined]
                    emit.assertion(entity, column, alias)


def records(ctx: RecordContext) -> None:
    coefficients = ctx.group(
        "ancillary_equation_rows", "fluid", "ancillary", partition="saturation_pressure"
    )
    for curve in ctx.rows("ancillary_equations", "saturation_pressure"):
        rows = sorted(
            coefficients.get((curve["fluid"], curve["ancillary"]), []),
            key=lambda row: row["row_index"],  # type: ignore[arg-type,return-value]
        )
        with ctx.emit(curve, *rows) as emit:
            parameter_set = emit.parameter_set(
                parameterization=emit.parameterization("saturation_ancillaries"),
                slot_group="vapor_pressure_exp_series_tau.pure",
                subjects=[ctx.subject("fluids", curve["fluid"])],  # type: ignore[arg-type]
                slots=ctx.slot_values(curve, "vapor_pressure_exp_series_tau.pure"),
                families={
                    "term": [
                        ctx.family_row(row, "vapor_pressure_exp_series_tau.pure.term")
                        for row in rows
                    ]
                },
            )
            emit.kind(
                "envelope", {**ctx.attributes(curve, "envelope"), "parameter_set": parameter_set}
            )
            emit.derivation([parameter_set])
