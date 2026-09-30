# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Structure of the JANAF mapping (rules, units and constants are in mapping.toml).

A table file is split into segments, one for each aggregation its rows cover: a table of one phase
is one segment; a table of several (cr,l, l,g, an element's reference form) changes aggregation
at a transition marker that names two different phases, the marker row being the last row of the
lower segment and the row after it the first of the upper. Polymorph changes inside one aggregation
(ALPHA <--> BETA) do not split a segment. A table whose aggregation neither its designator nor its
markers give holds its block.

Phase 1: each segment is one source entity, a species form, that asserts the names of its table
and the Hill formula of its composition, with the charge the formula's sign states.

Phase 2: each segment is one evaluated dataset (the segment's species form as its component, one
phase, the eight typed columns) with a point for each of its rows and a datum for each cell that
holds a number, and each form's composition is asserted. A row whose cells run together is held
in a block of its own and is not a point.
"""

from __future__ import annotations

import re
from collections import defaultdict

from thermo_knowledge import pipeline_contract as pc
from thermo_knowledge.mapping.context import (
    IdentityContext,
    RecordContext,
    RowHeld,
    RunContext,
    ORIGIN,
    first,
    numbered,
)
from thermo_knowledge.mapping.formula import CHARGE, hill
from thermo_knowledge.mapping.spec import declared_entity
from thermo_knowledge.mapping.staged import SourceRow

VARIABLE = "variable"
"""The role of the column that holds the temperature."""
DATUM_VALUE = "datum.value"
"""The target of the rule of each column of a table that becomes a column of a dataset."""
FORMULA = re.compile(r"(?:(?P<symbol>[A-Z][a-z]?)(?P<count>\d+(?:\.\d+)?))+")
TERM = re.compile(r"(?P<symbol>[A-Z][a-z]?)(?P<count>\d+(?:\.\d+)?)")


class Segment:
    """One aggregation's run of rows of a table, numbered from one."""

    def __init__(self, number: int, aggregation: str, rows: list[SourceRow]) -> None:
        self.number = number
        self.aggregation = aggregation
        self.rows = rows


def formula_of(ctx: RunContext[object], table: SourceRow) -> tuple[dict[str, float], int]:
    """The composition and charge of a JANAF formula (`Al1Cl1F1+`: every count written, a charge as
    a trailing sign, which the decoding `charge_of_formula` reads); an element the declaration
    does not hold holds the block."""
    decoded = ctx.decoded(table, "derived:charge")
    assert decoded is not None  # a decoding without `required` is held, not `None`
    charge = int(decoded.to)
    text = str(table["janaf_formula"]).rstrip("+-")
    if not FORMULA.fullmatch(text):
        raise RowHeld("pattern_mismatch", f"{table.locator}: `{text}` is not a JANAF formula")
    counts: defaultdict[str, float] = defaultdict(float)
    for term in TERM.finditer(text):
        symbol = term["symbol"]
        if declared_entity(ctx.decl, "conserved_quantity", symbol) is None:
            raise RowHeld(
                "unknown_subject", f"{table.locator}: `{symbol}` is not a declared conserved quantity"
            )
        counts[symbol] += float(term["count"])
    return counts, charge


def segments_of(ctx: RunContext[object], table: SourceRow, rows: list[SourceRow]) -> list[Segment]:
    """The segments of a table (see the module's text)."""
    designator = ctx.decoded(table, "phase_designator", required=False)
    if designator is not None:
        return [Segment(ORIGIN, designator.to, rows)]
    cuts: dict[int, tuple[str, str]] = {}
    for position, row in enumerate(rows):
        change = ctx.decoded(row, "marker", required=False)
        if change is None:
            continue
        lower = ctx.decoded(table, "derived:phase_name", using={"phase_name": change.groups["lower"]})
        upper = ctx.decoded(table, "derived:phase_name", using={"phase_name": change.groups["upper"]})
        assert lower is not None and upper is not None
        if lower.to != upper.to:
            cuts[position] = (lower.to, upper.to)
    if not cuts:
        raise RowHeld(
            "missing_convention",
            f"{table.locator}: the designator names no single aggregation and no marker of "
            "the table changes it",
        )
    found: list[Segment] = []
    current: list[SourceRow] = []
    aggregation = first(next(iter(cuts.values())))
    for position, row in enumerate(rows):
        current.append(row)
        if position not in cuts:
            continue
        lower, upper = cuts[position]
        if lower != aggregation:
            raise RowHeld(
                "pattern_mismatch",
                f"{table.locator}: a transition leaves {lower} from a phase that is {aggregation}",
            )
        found.append(Segment(ORIGIN + len(found), aggregation, current))
        current, aggregation = [], upper
    found.append(Segment(ORIGIN + len(found), aggregation, current))
    return found


def _lines(ctx: RunContext[object], tables: list[SourceRow]) -> dict[str, list[SourceRow]]:
    """The data lines of each table file, in file order."""
    found: dict[str, list[SourceRow]] = {}
    for row in ctx.rows("rows", "data"):  # type: ignore[attr-defined]
        found.setdefault(row.artifact, []).append(row)
    return found


def _split(
    rows: list[SourceRow],
) -> tuple[list[SourceRow], list[SourceRow]]:
    """The lines that can be points and those whose cells run together (held, each in a block of
    its own)."""
    return (
        [row for row in rows if row["parse_note"] is None],
        [row for row in rows if row["parse_note"] is not None],
    )


def identities(ctx: IdentityContext) -> None:
    tables = list(ctx.rows("tables"))
    lines = _lines(ctx, tables)
    for table in tables:
        rows, run_together = _split(lines.get(table.artifact, []))
        for row in run_together:
            with ctx.emit(row) as emit:
                emit.check()
        with ctx.emit(table, *rows) as emit:
            counts, charge = formula_of(ctx, table)
            for segment in segments_of(ctx, table, rows):
                entity = emit.source_entity(
                    "tables",
                    aggregation=segment.aggregation,
                    stated_charge=charge,
                    part=str(segment.number),
                )
                for column in ("code", "substance_name", "name_formula"):
                    emit.assertion(entity, column)
                emit.assertion(entity, "janaf_formula", hill(counts))


def records(ctx: RecordContext) -> None:
    tables = list(ctx.rows("tables"))
    lines = _lines(ctx, tables)
    charge_entity = ctx.declared("conserved_quantity", CHARGE)
    for table in tables:
        rows, _ = _split(lines.get(table.artifact, []))
        with ctx.emit(table, *rows) as emit:
            counts, charge = formula_of(ctx, table)
            segments = segments_of(ctx, table, rows)
            convention = emit.convention_set("janaf")
            forms = [ctx.subject("tables", str(table["code"]), part=str(s.number)) for s in segments]
            for form in forms:
                for symbol, count in sorted(counts.items()):
                    emit.relation(
                        "composition",
                        {"entity": form, "quantity": ctx.declared("conserved_quantity", symbol)},
                        {"value": count},
                    )
                if charge:
                    emit.relation(
                        "composition",
                        {"entity": form, "quantity": charge_entity},
                        {"value": float(charge)},
                    )
            phases: dict[int, object] = {}
            datasets: list[tuple[Segment, object]] = []
            for segment, form in zip(segments, forms, strict=True):
                first_row = first(segment.rows)
                dataset = emit.kind(
                    "dataset",
                    {
                        **ctx.attributes(first_row, "dataset"),
                        "carrier": emit.carrier,
                        "local_key": f"{table['code']}#{segment.number}",
                        "convention_set": convention,
                    },
                )
                component = emit.kind(
                    "dataset_component",
                    {**ctx.constants(first_row, "dataset_component"), "dataset": dataset, "entity": form},
                )
                phase = emit.kind(
                    "dataset_phase",
                    {
                        **ctx.constants(first_row, "dataset_phase"),
                        "dataset": dataset,
                        "aggregation": ctx.declared("aggregation", segment.aggregation),
                    },
                )
                phases[segment.number] = phase
                standard = ctx.decoded(
                    first_row, "derived:standard_state", using={"aggregation": segment.aggregation}
                )
                assert standard is not None
                state = emit.kind(
                    "standard_state",
                    {
                        **ctx.constants(first_row, "standard_state"),
                        "key": standard.key,
                        "kind": standard.to,
                    },
                )
                datasets.append((segment, (dataset, component, phase, state)))
            for segment, (dataset, component, phase, state) in datasets:
                first_row = first(segment.rows)
                names = ctx.value_columns(first_row, DATUM_VALUE)
                temperature = next(
                    name for name in names if ctx.column_attributes(first_row, name)["role"] == VARIABLE
                )
                columns = {}
                for ordinal, name in numbered(names):
                    attributes = ctx.column_attributes(first_row, name)
                    attributes.update(dataset=dataset, ordinal=ordinal)
                    if attributes["role"] == "property":
                        attributes.update(component=component, phase=phase)
                        if ctx.states_standard_state(first_row, name):
                            attributes["standard_state"] = state
                    if "reference_temperature" in attributes:
                        attributes["reference_phase"] = _reference_phase(
                            ctx, segments, phases, attributes["reference_temperature"], temperature
                        )
                    columns[name] = emit.kind("dataset_column", attributes)
                constants = ctx.constants(first_row, "datum")
                for index, row in numbered(segment.rows):
                    point = emit.kind("data_point", {"dataset": dataset, "index": index})
                    for name in names:
                        found = ctx.quantity(row, name)
                        if found is None:
                            continue
                        emit.relation(
                            "datum",
                            {pc.DATUM.point: point, pc.DATUM.column: columns[name]},
                            {
                                **constants,
                                "value": found,
                                "digits": ctx.digits(row, name),
                            },
                        )


def _reference_phase(
    ctx: RecordContext,
    segments: list[Segment],
    phases: dict[int, object],
    reference_temperature: object,
    column: str,
) -> object | None:
    """The phase of the segment that has a row at the reference temperature of a column: the
    phase whose enthalpy the column's increment is measured from."""
    wanted = reference_temperature.value  # type: ignore[attr-defined]
    for segment in segments:
        for row in segment.rows:
            found = ctx.quantity(row, column)
            if found is not None and found.value == wanted:
                return phases[segment.number]
    return None
