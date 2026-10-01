# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Structure of the Cantera mapping (rules, units and constants are in mapping.toml).

Phase 1: each species entry that states a thermo block of a mapped model is one source entity, a
species form whose aggregation the decoding of the thermo model of the phases that include it
gives, whose charge is that of the pseudo-element E among its composition rows, and which asserts
its name and the Hill formula of its composition. An entry that no phase includes, that phases
of different aggregations include, or that names an element the declaration does not hold holds
its block.

Phase 2: each composition row becomes a `composition` row of the entry's form; each thermo block
becomes one parameter set of its form's slot group under the parameterization of its file, with
the pieces of the block as the rows of the family, the fitted (or recommended) range as its
validity region and, for a fitted set, the `fit` derivation that produced both. Blocks of one
subject in one file are repeated assertions, numbered in file order.
"""

from __future__ import annotations

import posixpath
from collections import Counter

from thermo_knowledge.canonical.writer import FamilyRow
from thermo_knowledge.mapping.context import (
    SINGLE,
    IdentityContext,
    RecordContext,
    RowHeld,
    first,
    last,
    numbered,
)
from thermo_knowledge.mapping.formula import composition, hill
from thermo_knowledge.mapping.staged import SourceRow

POLYNOMIALS = {"nasa7": "nasa7.pure", "nasa9": "nasa9.pure", "shomate": "shomate.pure"}
"""The partition of a polynomial model of `species_thermo`, and the slot group of its form."""
CONSTANT_CP = "constant_cp.pure"
PIECEWISE = "piecewise_gibbs.pure"
ALL = {"all", "declared-species"}
"""What a phase's `selection` says when it includes every species of a section."""


def _sections(ctx: IdentityContext) -> dict[tuple[str, str], list[SourceRow]]:
    """The species entries of each (file, section)."""
    found: dict[tuple[str, str], list[SourceRow]] = {}
    for row in ctx.source_rows("species"):
        fields = row.fields
        found.setdefault((str(fields._artifact), str(fields.section)), []).append(row)
    return found


def _target(
    artifact: str, section: str, sections: dict[tuple[str, str], list[SourceRow]]
) -> tuple[str, str] | None:
    """The (file, section) a phase names as a source of species: a section of its own file, or
    `file/section` of another, found in the directory of the file first and then in the data
    directory, as Cantera searches."""
    if "/" not in section:
        return (artifact, section)
    name, _, tail = section.rpartition("/")
    here = posixpath.dirname(artifact)
    for candidate in (posixpath.join(here, name), posixpath.join("data", name)):
        if (posixpath.normpath(candidate), tail) in sections:
            return (posixpath.normpath(candidate), tail)
    found = [
        (file, section)
        for file, section in sections
        if section == tail and file.endswith("/" + posixpath.basename(name))
    ]
    return first(found) if len(found) == SINGLE else None


def _including(ctx: IdentityContext) -> dict[str, list[SourceRow]]:
    """For each species entry (by locator), the phases of the tree whose `species` entry names
    it, in file and position order."""
    sections = _sections(ctx)
    references: dict[str, list[SourceRow]] = {}
    for reference in ctx.source_rows("phase_references"):
        fields = reference.fields
        if fields.reference == "species":
            references.setdefault(str(fields.phase_locator), []).append(reference)
    found: dict[str, list[SourceRow]] = {}
    for phase in ctx.source_rows("phases"):
        artifact = str(phase.fields._artifact)
        for reference in references.get(phase.locator, []):
            fields = reference.fields
            wanted: set[str] | None
            if fields.form == "name":
                keys, wanted = [(artifact, "species")], {str(fields.name)}
            elif fields.form == "string":
                keys, wanted = (
                    ([(artifact, "species")], None) if fields.selection != "none" else ([], set())
                )
            else:
                target = _target(artifact, str(fields.section), sections)
                keys = [] if target is None else [target]
                if fields.names:
                    wanted = {str(n) for n in fields.names}
                else:
                    wanted = None if fields.selection in ALL else set()
            for key in keys:
                for entry in sections.get(key, []):
                    if wanted is None or entry.fields.name in wanted:
                        phases_of = found.setdefault(entry.locator, [])
                        if phase not in phases_of:
                            phases_of.append(phase)
    return found


def _aggregation(ctx: IdentityContext, species: SourceRow, phases: list[SourceRow]) -> str:
    if not phases:
        raise RowHeld(
            "missing_convention",
            f"{species.locator}: no phase of the tree includes the entry, so nothing states its "
            "aggregation",
        )
    found = set()
    for phase in phases:
        decoded = ctx.decoded_by(species, "aggregation_of_phase_thermo", using=phase.values)
        assert decoded is not None  # an undecodable phase holds the block
        found.add(decoded.to)
    if len(found) > SINGLE:
        raise RowHeld(
            "missing_convention",
            f"{species.locator}: phases of different aggregations include the entry "
            f"({', '.join(sorted(found))})",
        )
    return found.pop()


def identities(ctx: IdentityContext) -> None:
    compositions = ctx.group("species_composition", "species_locator")
    including = _including(ctx)
    for species in ctx.rows("species", "with_thermo"):
        rows = compositions.get((species.locator,), [])
        with ctx.emit(species, *rows) as emit:
            aggregation = _aggregation(ctx, species, including.get(species.locator, []))
            counts, charge = composition(ctx, rows, symbol="element")
            entity = emit.source_entity("species", aggregation=aggregation, stated_charge=charge)
            emit.assertion(entity, "name")
            emit.assertion(entity, "descriptive_name")
            emit.assertion(entity, "derived:formula", hill(counts))


def records(ctx: RecordContext) -> None:
    compositions = ctx.group("species_composition", "species_locator")
    pieces = ctx.group("thermo_pieces", "thermo_locator")
    points = ctx.group("entry_parameters", "entry_locator", partition="piecewise_gibbs_points")
    species_by_locator = {row.locator: row for row in ctx.rows("species", "with_thermo")}
    for species in species_by_locator.values():
        rows = compositions.get((species.locator,), [])
        with ctx.emit(species, *rows) as emit:
            form = ctx.subject("species", species.locator)
            for pair in rows:
                found = ctx.attributes(pair, "composition")
                emit.relation(
                    "composition",
                    {"entity": form, "quantity": found["quantity"]},
                    {"value": found["value"]},
                )
    occurrences: Counter[tuple[str, str, object]] = Counter()

    def occurrence_of(artifact: str, group: str, subject: object) -> int:
        key = (artifact, group, subject)
        occurrences.update([key])
        return occurrences[key]

    for partition, group in POLYNOMIALS.items():
        piece = ctx.family_of(group)
        for block in ctx.rows("species_thermo", partition):
            species = species_by_locator[str(block["species_locator"])]
            piece_rows = sorted(
                pieces.get((block.locator,), []),
                key=lambda row: row["piece_index"],  # type: ignore[arg-type,return-value]
            )
            with ctx.emit(block, species, *piece_rows) as emit:
                subject = ctx.subject("species", species.locator)
                family = [ctx.family_row(row, piece) for row in piece_rows]
                parameter_set = emit.parameter_set(
                    parameterization=emit.parameterization("file", artifact=species.artifact),
                    slot_group=group,
                    subjects=[subject],
                    slots={},
                    families={ctx.family_key(piece): family},
                    occurrence=occurrence_of(species.artifact, group, subject),
                )
                clause = {
                    **ctx.attributes(block, "region_clause"),
                    "lower": first(family).values["T_low"],
                    "upper": last(family).values["T_high"],
                }
                region = emit.validity(
                    parameter_set, ctx.attributes(block, "validity_region"), [clause]
                )
                emit.derivation([parameter_set, region])
    for block in ctx.rows("species_thermo", "constant_cp"):
        species = species_by_locator[str(block["species_locator"])]
        with ctx.emit(block, species) as emit:
            subject = ctx.subject("species", species.locator)
            parameter_set = emit.parameter_set(
                parameterization=emit.parameterization("file", artifact=species.artifact),
                slot_group=CONSTANT_CP,
                subjects=[subject],
                slots=ctx.slot_values(block, CONSTANT_CP),
                occurrence=occurrence_of(species.artifact, CONSTANT_CP, subject),
            )
            limits = ctx.attributes(block, "region_clause")
            region = ctx.attributes(block, "validity_region")
            if "lower" in limits or "upper" in limits:
                emit.validity(parameter_set, region, [limits])
            else:
                emit.validity_not_stated(parameter_set, region)
    point = ctx.family_of(PIECEWISE)
    for block in ctx.rows("species_thermo", "piecewise_gibbs"):
        species = species_by_locator[str(block["species_locator"])]
        point_rows = sorted(
            points.get((block.locator,), []),
            key=lambda row: ctx.quantity(row, "path").value,  # type: ignore[union-attr]
        )
        with ctx.emit(block, species, *point_rows) as emit:
            subject = ctx.subject("species", species.locator)
            family = []
            for number, row in numbered(point_rows):
                found = ctx.family_row(row, point)
                family.append(FamilyRow({"n": number}, found.values))
            emit.parameter_set(
                parameterization=emit.parameterization("file", artifact=species.artifact),
                slot_group=PIECEWISE,
                subjects=[subject],
                slots=ctx.slot_values(block, PIECEWISE),
                families={ctx.family_key(point): family},
                occurrence=occurrence_of(species.artifact, PIECEWISE, subject),
            )
