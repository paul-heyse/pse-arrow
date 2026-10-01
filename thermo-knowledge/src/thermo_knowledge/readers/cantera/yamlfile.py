# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Decomposition of one Cantera YAML file into rows.

Every row is located by the JSON pointer of the YAML node it comes from. A dictionary is taken
key by key into typed columns; a key whose value has another type than its column, and every
key no column names, is kept as leaf rows of `entry_parameters`, so nothing the file states is
dropped and nothing is interpreted: a value stays as written (a string with a unit stays a
string).
"""

from __future__ import annotations

import json
import re
from collections.abc import Mapping
from pathlib import Path

import yaml

from thermo_knowledge.readers.cantera import yaml12
from thermo_knowledge.readers.cantera.common import (
    Fields,
    Sink,
    Value,
    as_float,
    as_floats,
    leaf_columns,
    locator,
    pointer,
    walk,
)
from thermo_knowledge.staging.errors import StagingError

PHASES = "phases"
SPECIES = "species"
REACTIONS = "reactions"
ELEMENTS = "elements"
ARROWS = ("<=>", "=>", "=")
_NUMBER = re.compile(r"^[-+]?(?:[0-9]+\.?[0-9]*|\.[0-9]+)(?:[eE][-+]?[0-9]+)?$")
_COLLIDER = re.compile(r"^\(\+\S*\)$")
_DEFAULT_KINDS = {
    PHASES: "phase",
    SPECIES: "species",
    REACTIONS: "reaction",
    ELEMENTS: "element",
}


def load_document(
    tree: Path, artifact: str
) -> tuple[dict[object, Value], list[yaml12.DuplicateKey]]:
    """The parsed file, which must be a YAML mapping; errors name the file and line."""
    try:
        content = (tree / artifact).read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError) as error:
        raise StagingError(f"{artifact}: cannot be read: {error}") from error
    try:
        document, duplicates = yaml12.load(content)
    except yaml.YAMLError as error:
        mark = getattr(error, "problem_mark", None)
        where = f"line {mark.line + 1}, column {mark.column + 1}: " if mark is not None else ""
        problem = getattr(error, "problem", None) or str(error)
        raise StagingError(f"{artifact}: cannot be read as YAML: {where}{problem}") from error
    if not isinstance(document, dict):
        raise StagingError(
            f"{artifact}: the top level must be a mapping of sections, "
            f"found {type(document).__name__}"
        )
    return document, duplicates


def classify(section: str, index: int, entry: Value) -> str:
    """The kind of list entry `entry`: units, phase, species, reaction, element, collision or
    other."""
    if index == 0 and isinstance(entry, dict) and set(entry) == {"units"}:
        if isinstance(entry["units"], dict):
            return "units"
    if section in _DEFAULT_KINDS:
        return _DEFAULT_KINDS[section] if isinstance(entry, dict) else "other"
    if isinstance(entry, dict):
        if "equation" in entry:
            return "reaction"
        if "symbol" in entry:
            return "element"
        if "composition" in entry:
            return "species"
        if "target" in entry and ("kind" in entry or "cross-sections" in entry):
            return "collision"
    return "other"


def first_token(relative: str) -> str:
    """The first key of a relative JSON pointer, unescaped (empty for the empty pointer)."""
    if not relative:
        return ""
    return relative.split("/")[1].replace("~1", "/").replace("~0", "~")


def shape(value: Value) -> str:
    if isinstance(value, dict):
        return "mapping"
    if isinstance(value, list):
        return "list"
    return "scalar"


class FileReader:
    """Rows of one file, in a `Sink`."""

    def __init__(self, artifact: str, sink: Sink) -> None:
        self.artifact = artifact
        self.sink = sink

    # -- helpers -----------------------------------------------------------------------------

    def row(self, table: str, at: str, /, **columns: Value) -> str:
        where = locator(self.artifact, at)
        self.sink.add(table, {"_artifact": self.artifact, "_locator": where, **columns})
        return where

    def units(
        self, scope: str, scope_path: str, units_pointer: str, mapping: Mapping[object, Value]
    ) -> None:
        if not mapping:
            self.row(
                "units_entries",
                units_pointer,
                scope=scope,
                scope_path=scope_path,
                **leaf_columns(self.artifact, units_pointer, mapping),
            )
            return
        for kind, where, relative, leaf in walk(dict(mapping), units_pointer):
            if kind == "units" and isinstance(leaf, dict):
                # a units mapping nested in a units mapping: keep it as its own block
                self.units("entry", relative, where, leaf)
            else:
                self.row(
                    "units_entries",
                    where,
                    scope=scope,
                    scope_path=scope_path,
                    dimension=relative.lstrip("/").replace("/", "."),
                    **leaf_columns(self.artifact, where, leaf),
                )

    def leaves_of(self, table: str, owner_pointer: str, value: Value, at: str, rel: str) -> None:
        """Leaf rows for `value`, which stands at pointer `at` and `rel` inside the owning row."""
        owner = locator(self.artifact, owner_pointer)
        block = first_token(rel)
        for kind, where, relative, leaf in walk(value, at, rel):
            if kind == "units" and isinstance(leaf, dict):
                self.units("entry", relative, where, leaf)
                continue
            self.row(
                "entry_parameters",
                where,
                entry_table=table,
                entry_locator=owner,
                path=relative,
                block=block,
                **leaf_columns(self.artifact, where, leaf),
            )

    def parameters(self, table: str, owner_pointer: str, rest: Mapping[object, Value]) -> None:
        """The keys nobody took, as leaf rows; units mappings go to `units_entries`."""
        for key, value in rest.items():
            at = owner_pointer + pointer(key)
            if key == "units" and isinstance(value, dict):
                self.units("entry", owner_pointer, at, value)
            else:
                self.leaves_of(table, owner_pointer, value, at, pointer(key))

    # -- the file ----------------------------------------------------------------------------

    def read(self, document: dict[object, Value], duplicates: list[yaml12.DuplicateKey]) -> None:
        for duplicate in duplicates:
            self.row(
                "yaml_duplicate_keys",
                f"L{duplicate.line}",
                key=str(duplicate.key),
                line=duplicate.line,
                replaced_at_line=duplicate.replaced_at,
                discarded_value_json=json.dumps(duplicate.value, default=str),
            )
        top = Fields(document)
        meta = {
            "description": top.string("description"),
            "generator": top.string("generator"),
            "cantera_version": top.string("cantera-version"),
            "git_commit": top.string("git-commit"),
            "date": top.string("date"),
            "source": top.string("source"),
            "input_files": top.strings("input-files"),
        }
        self.row(
            "yaml_files",
            "",
            **meta,
            top_level_keys=[str(key) for key in document],
        )
        for position, (key, value) in enumerate(document.items()):
            section = str(key)
            kinds: list[str] = []
            if isinstance(value, list):
                for index, entry in enumerate(value):
                    kind = classify(section, index, entry)
                    if kind not in kinds:
                        kinds.append(kind)
            count = len(value) if isinstance(value, list | dict) else None
            self.row(
                "file_sections",
                pointer(key),
                section=section,
                position=position,
                shape=shape(value),
                entry_count=count,
                entry_kinds=kinds,
            )
            if key not in top.rest:
                continue  # a metadata key taken into yaml_files
            if key == "units" and isinstance(value, dict):
                self.units("file", "", pointer(key), value)
            elif isinstance(value, list):
                self.section(section, value)
            else:
                self.leaves_of("file_sections", pointer(key), value, pointer(key), "")

    def section(self, section: str, entries: list[Value]) -> None:
        for index, entry in enumerate(entries):
            where = pointer(section, index)
            kind = classify(section, index, entry)
            if kind == "units":
                assert isinstance(entry, dict)
                self.units("section", pointer(section), where + pointer("units"), entry["units"])
                continue
            if kind == "other":
                self.row(
                    "other_entries",
                    where,
                    section=section,
                    index=index,
                    shape=shape(entry),
                )
                self.other(where, entry)
                continue
            assert isinstance(entry, dict)
            fields = Fields(entry)
            if kind == "phase":
                self.phase(section, index, where, fields)
            elif kind == "species":
                self.species(section, index, where, fields)
            elif kind == "reaction":
                self.reaction(section, index, where, fields)
            elif kind == "element":
                self.element(section, index, where, fields)
            else:
                self.collision(section, index, where, fields)

    def other(self, where: str, entry: Value) -> None:
        if isinstance(entry, dict):
            self.parameters("other_entries", where, entry)
        else:
            self.leaves_of("other_entries", where, entry, where, "")

    # -- entries -----------------------------------------------------------------------------

    def element(self, section: str, index: int, where: str, fields: Fields) -> None:
        self.row(
            "elements",
            where,
            section=section,
            index=index,
            symbol=fields.string("symbol"),
            atomic_weight=fields.number("atomic-weight"),
            atomic_number=fields.number("atomic-number"),
            entropy298=fields.number("entropy298"),
            source=fields.string("source"),
            note=fields.string("note"),
        )
        self.parameters("elements", where, fields.rest)

    def collision(self, section: str, index: int, where: str, fields: Fields) -> None:
        self.row(
            "collisions",
            where,
            section=section,
            index=index,
            target=fields.string("target"),
            product=fields.string("product"),
            kind=fields.string("kind"),
            threshold=fields.number("threshold"),
            note=fields.string("note"),
        )
        self.parameters("collisions", where, fields.rest)

    def phase(self, section: str, index: int, where: str, fields: Fields) -> None:
        site_density, site_density_text = fields.quantity("site-density")
        density, density_text = fields.quantity("density")
        row_locator = self.row(
            "phases",
            where,
            section=section,
            index=index,
            name=fields.string("name"),
            thermo=fields.string("thermo"),
            kinetics=fields.string("kinetics"),
            transport=fields.string("transport"),
            site_density=site_density,
            site_density_text=site_density_text,
            density=density,
            density_text=density_text,
            standard_concentration_basis=fields.string("standard-concentration-basis"),
            pure_fluid_name=fields.string("pure-fluid-name"),
            skip_undeclared_elements=fields.boolean("skip-undeclared-elements"),
            skip_undeclared_third_bodies=fields.boolean("skip-undeclared-third-bodies"),
            note=fields.string("note"),
            adjacent_phases=fields.strings("adjacent-phases"),
        )
        for reference in (ELEMENTS, SPECIES, REACTIONS):
            if reference in fields.rest:
                self.references(row_locator, where, reference, fields.rest.pop(reference))
        self.parameters("phases", where, fields.rest)

    def references(
        self, phase_locator: str, phase_pointer: str, reference: str, value: Value
    ) -> None:
        base = phase_pointer + pointer(reference)
        leftovers: list[tuple[str, Value]] = []

        def emit(position: int | None, at: str, **columns: Value) -> None:
            self.row(
                "phase_references",
                at,
                phase_locator=phase_locator,
                reference=reference,
                position=position,
                **columns,
            )

        if isinstance(value, str):
            emit(None, base, form="string", selection=value)
        elif isinstance(value, list) and value:
            for position, item in enumerate(value):
                at = base + pointer(position)
                if isinstance(item, str):
                    emit(position, at, form="name", name=item)
                elif isinstance(item, dict) and item:
                    for key, names in item.items():
                        entry_at = at + pointer(key)
                        if not isinstance(key, str):
                            leftovers.append((pointer(reference, position, key), names))
                        elif isinstance(names, str):
                            emit(position, entry_at, form="section", section=key, selection=names)
                        elif isinstance(names, list) and all(isinstance(n, str) for n in names):
                            emit(position, entry_at, form="section", section=key, names=list(names))
                        else:
                            leftovers.append((pointer(reference, position, key), names))
                else:
                    leftovers.append((pointer(reference, position), item))
        else:
            leftovers.append((pointer(reference), value))
        for relative, leftover in leftovers:
            self.leaves_of("phases", phase_pointer, leftover, phase_pointer + relative, relative)

    def species(self, section: str, index: int, where: str, fields: Fields) -> None:
        row_locator = self.row(
            "species",
            where,
            section=section,
            index=index,
            name=fields.string("name"),
            note=fields.string("note"),
            charge=fields.number("charge"),
            sites=fields.number("sites"),
            descriptive_name=fields.string("descriptive-name"),
            molecular_weight=fields.number("molecular-weight"),
        )
        composition = fields.rest.get("composition")
        if isinstance(composition, dict) and composition:
            rest: dict[object, Value] = {}
            for element, atoms in composition.items():
                count = as_float(atoms) if isinstance(element, str) else None
                if count is None:
                    rest[element] = atoms
                else:
                    self.row(
                        "species_composition",
                        where + pointer("composition", element),
                        species_locator=row_locator,
                        element=element,
                        atoms=count,
                    )
            del fields.rest["composition"]
            if rest:
                fields.rest["composition"] = rest
        thermo = fields.mapping("thermo")
        if thermo is not None:
            self.thermo(row_locator, where + pointer("thermo"), thermo)
        transport = fields.mapping("transport")
        if transport is not None:
            self.transport(row_locator, where + pointer("transport"), transport)
        critical = fields.mapping("critical-parameters")
        if critical is not None:
            self.critical(row_locator, where + pointer("critical-parameters"), critical)
        eos = fields.mapping("equation-of-state")
        if eos is not None:
            body = Fields(eos)
            at = where + pointer("equation-of-state")
            self.row(
                "species_equation_of_state",
                at,
                species_locator=row_locator,
                model=body.string("model"),
            )
            self.parameters("species_equation_of_state", at, body.rest)
        self.parameters("species", where, fields.rest)

    def thermo(self, species_locator: str, where: str, thermo: dict[object, Value]) -> None:
        body = Fields(thermo)
        ranges = body.numbers("temperature-ranges")
        data = body.rest.get("data")
        pieces: list[list[float]] | None = None
        if isinstance(data, list):
            converted = [as_floats(row) for row in data]
            if all(row is not None for row in converted):
                pieces = [row for row in converted if row is not None]
                del body.rest["data"]
        reference_pressure, reference_pressure_text = body.quantity("reference-pressure")
        t0, t0_text = body.quantity("T0")
        h0, h0_text = body.quantity("h0")
        s0, s0_text = body.quantity("s0")
        cp0, cp0_text = body.quantity("cp0")
        row_locator = self.row(
            "species_thermo",
            where,
            species_locator=species_locator,
            model=body.string("model"),
            reference_pressure=reference_pressure,
            reference_pressure_text=reference_pressure_text,
            note=body.string("note"),
            temperature_ranges=ranges,
            piece_count=len(pieces) if pieces is not None else None,
            T0=t0,
            T0_text=t0_text,
            h0=h0,
            h0_text=h0_text,
            s0=s0,
            s0_text=s0_text,
            cp0=cp0,
            cp0_text=cp0_text,
            T_min=body.number("T-min"),
            T_max=body.number("T-max"),
            dimensionless=body.boolean("dimensionless"),
        )
        if pieces is not None:
            aligned = ranges is not None and len(ranges) == len(pieces) + 1
            for piece_index, coefficients in enumerate(pieces):
                self.row(
                    "thermo_pieces",
                    where + pointer("data", piece_index),
                    thermo_locator=row_locator,
                    piece_index=piece_index,
                    temperature_low=ranges[piece_index] if aligned and ranges else None,
                    temperature_high=ranges[piece_index + 1] if aligned and ranges else None,
                    coefficients=coefficients,
                    coefficient_count=len(coefficients),
                )
        self.parameters("species_thermo", where, body.rest)

    def transport(self, species_locator: str, where: str, transport: dict[object, Value]) -> None:
        body = Fields(transport)
        self.row(
            "species_transport",
            where,
            species_locator=species_locator,
            model=body.string("model"),
            geometry=body.string("geometry"),
            diameter=body.number("diameter"),
            well_depth=body.number("well-depth"),
            dipole=body.number("dipole"),
            polarizability=body.number("polarizability"),
            rotational_relaxation=body.number("rotational-relaxation"),
            acentric_factor=body.number("acentric-factor"),
            dispersion_coefficient=body.number("dispersion-coefficient"),
            quadrupole_polarizability=body.number("quadrupole-polarizability"),
            note=body.string("note"),
        )
        self.parameters("species_transport", where, body.rest)

    def critical(self, species_locator: str, where: str, critical: dict[object, Value]) -> None:
        body = Fields(critical)
        pressure, pressure_text = body.quantity("critical-pressure")
        self.row(
            "species_critical_parameters",
            where,
            species_locator=species_locator,
            critical_temperature=body.number("critical-temperature"),
            critical_pressure=pressure,
            critical_pressure_text=pressure_text,
            acentric_factor=body.number("acentric-factor"),
            critical_molar_volume=body.number("critical-molar-volume"),
            critical_compressibility=body.number("critical-compressibility"),
            source=body.string("source"),
        )
        self.parameters("species_critical_parameters", where, body.rest)

    def reaction(self, section: str, index: int, where: str, fields: Fields) -> None:
        equation = fields.string("equation")
        terms = parse_equation(equation) if equation is not None else None
        row_locator = self.row(
            "reactions",
            where,
            section=section,
            index=index,
            equation=equation,
            arrow=terms[0] if terms else None,
            equation_parsed=terms is not None,
            type=fields.string("type"),
            duplicate=fields.boolean("duplicate"),
            id=fields.string("id"),
            note=fields.string("note"),
            negative_A=fields.boolean("negative-A"),
            negative_orders=fields.boolean("negative-orders"),
            nonreactant_orders=fields.boolean("nonreactant-orders"),
            default_efficiency=fields.number("default-efficiency"),
            sticking_species=fields.string("sticking-species"),
        )
        if terms is not None:
            for side, position, role, coefficient, species in terms[1]:
                self.row(
                    "reaction_terms",
                    where + pointer("equation", side, position),
                    reaction_locator=row_locator,
                    side=side,
                    position=position,
                    role=role,
                    coefficient=coefficient,
                    species=species,
                )
        self.parameters("reactions", where, fields.rest)


Term = tuple[str, int, str, "float | None", str]


def parse_equation(equation: str) -> tuple[str, list[Term]] | None:
    """The arrow and terms of a reaction equation, or `None` when it does not follow the
    documented grammar (every token, `+` and the arrow separated by spaces; a term is a species,
    optionally preceded by a coefficient; a side may end in `(+M)` or `(+ M)`)."""
    tokens = equation.split()
    arrows = [i for i, token in enumerate(tokens) if token in ARROWS]
    if len(arrows) != 1:
        return None
    arrow = tokens[arrows[0]]
    terms: list[Term] = []
    for side, side_tokens in (
        ("reactant", tokens[: arrows[0]]),
        ("product", tokens[arrows[0] + 1 :]),
    ):
        collider: str | None = None
        body = side_tokens
        if body and _COLLIDER.match(body[-1]):
            collider, body = body[-1][2:-1], body[:-1]
        elif len(body) >= 2 and body[-2] == "(+" and body[-1].endswith(")"):
            collider, body = body[-1][:-1], body[:-2]
        if collider == "":
            return None
        groups: list[list[str]] = [[]]
        for token in body:
            if token == "+":
                groups.append([])
            else:
                groups[-1].append(token)
        position = 0
        for group in groups:
            if len(group) == 1 and not _NUMBER.match(group[0]):
                terms.append((side, position, "participant", None, group[0]))
            elif len(group) == 2 and _NUMBER.match(group[0]) and not _NUMBER.match(group[1]):
                terms.append((side, position, "participant", float(group[0]), group[1]))
            else:
                return None
            position += 1
        if collider is not None:
            terms.append((side, position, "falloff_collider", None, collider))
    return arrow, terms
