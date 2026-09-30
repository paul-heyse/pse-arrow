from __future__ import annotations

from thermo_knowledge.mapping.context import IdentityContext, RecordContext

GROUP = "vapor_pressure_exp_series_tau.pure"


def identities(ctx: IdentityContext) -> None:
    for row in ctx.rows("species"):
        with ctx.emit(row) as emit:
            entity = emit.source_entity("species")
            for column in ("name", "cas", "inchikey", "inchi", "placeholder"):
                emit.assertion(entity, column)
            for alias in row["aliases"] or []:  # type: ignore[attr-defined]
                emit.assertion(entity, "aliases", alias)


def records(ctx: RecordContext) -> None:
    coefficients = ctx.group("coefficients", "species", "curve")
    for curve in ctx.rows("curves", "pressure"):
        rows = sorted(
            coefficients.get((curve["species"], curve["curve"]), []),
            key=lambda row: row["pos"],  # type: ignore[arg-type,return-value]
        )
        if not rows:
            continue
        with ctx.emit(curve, *rows) as emit:
            parameter_set = emit.parameter_set(
                parameterization=emit.parameterization("curves"),
                slot_group=GROUP,
                subjects=[ctx.subject("species", curve["species"])],  # type: ignore[arg-type]
                slots=ctx.slot_values(curve, GROUP),
                families={"term": [ctx.family_row(row, f"{GROUP}.term") for row in rows]},
            )
            region = emit.validity(
                parameter_set,
                ctx.attributes(curve, "validity_region"),
                [ctx.attributes(curve, "region_clause")],
            )
            emit.derivation([parameter_set, region])
