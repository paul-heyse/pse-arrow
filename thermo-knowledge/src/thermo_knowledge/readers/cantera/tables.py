# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The declared tables of the Cantera reader. Column names follow Cantera's keys (hyphens become
underscores; the key is kept in the column's `source_name` metadata)."""

from __future__ import annotations

import pyarrow as pa

from thermo_knowledge.readers.cantera.common import (
    NOT_APPLICABLE,
    UNIT_NOTE,
    VALUE_FIELDS,
    col,
    flag,
    floats,
    integer,
    number,
    quantity,
    strings,
    text,
)
from thermo_knowledge.staging.schema import INT64, STRING, table_schema

SECTION = text("section", "name of the top-level key that holds the list")
INDEX = integer("index", "position in the list (0-based)")
SPECIES_LOCATOR = text("species_locator", "_locator of the species row")


def _entry(*columns: pa.Field) -> pa.Schema:
    return table_schema(SECTION, INDEX, *columns)


SCHEMAS: dict[str, pa.Schema] = {
    "yaml_files": table_schema(
        text("description", "description"),
        text("generator", "generator"),
        text("cantera_version", "cantera-version"),
        text("git_commit", "git-commit"),
        text("date", "date"),
        text("source", "source"),
        strings("input_files", "input-files"),
        col(
            "top_level_keys",
            pa.list_(STRING),
            "the file's top-level keys, in order",
            NOT_APPLICABLE,
        ),
    ),
    "yaml_duplicate_keys": table_schema(
        col("key", STRING, "the repeated key", NOT_APPLICABLE),
        integer("line", "line of the discarded occurrence", "line"),
        integer("replaced_at_line", "line of the later occurrence that replaced it", "line"),
        col(
            "discarded_value_json",
            STRING,
            "value of the discarded occurrence, as JSON",
            NOT_APPLICABLE,
        ),
    ),
    "file_sections": table_schema(
        text("section", "top-level key"),
        integer("position", "position among the file's top-level keys (0-based)"),
        col("shape", STRING, "YAML shape of the value: list, mapping or scalar", NOT_APPLICABLE),
        integer("entry_count", "number of list entries or mapping keys"),
        col(
            "entry_kinds",
            pa.list_(STRING),
            "kinds of the list entries, in order of first appearance: units, phase, species, "
            "reaction, element, collision or other",
            NOT_APPLICABLE,
            "a section's entry kind is decided by its name (phases, species, reactions, elements) "
            "or, for a section with another name, by the keys of each entry: equation, symbol, "
            "composition, target",
        ),
    ),
    "units_entries": table_schema(
        col(
            "scope",
            STRING,
            "where the units mapping stands: file, section (first item of a list) or entry",
            NOT_APPLICABLE,
        ),
        col(
            "scope_path",
            STRING,
            "JSON pointer of the file (empty), section or entry the mapping applies to",
            NOT_APPLICABLE,
        ),
        text("dimension", "key of the units mapping (length, time, quantity, ...)"),
        *VALUE_FIELDS,
    ),
    "elements": _entry(
        text("symbol", "symbol"),
        number("atomic_weight", "atomic-weight"),
        number("atomic_number", "atomic-number"),
        number("entropy298", "entropy298"),
        text("source", "source"),
        text("note", "note"),
    ),
    "species": _entry(
        text("name", "name"),
        text("note", "note"),
        number("charge", "charge"),
        number("sites", "sites"),
        text("descriptive_name", "descriptive-name"),
        number("molecular_weight", "molecular-weight"),
    ),
    "species_composition": table_schema(
        SPECIES_LOCATOR,
        text("element", "composition key"),
        number("atoms", "composition value"),
    ),
    "species_thermo": table_schema(
        SPECIES_LOCATOR,
        text("model", "model"),
        *quantity("reference_pressure", "reference-pressure"),
        text("note", "note"),
        floats(
            "temperature_ranges",
            "temperature-ranges",
            "the boundaries as written; a NASA-7 or NASA-9 piece i spans boundary i to i+1",
        ),
        integer("piece_count", "number of coefficient rows in data"),
        *quantity("T0", "T0"),
        *quantity("h0", "h0"),
        *quantity("s0", "s0"),
        *quantity("cp0", "cp0"),
        number("T_min", "T-min"),
        number("T_max", "T-max"),
        flag("dimensionless", "dimensionless"),
    ),
    "thermo_pieces": table_schema(
        text("thermo_locator", "_locator of the species_thermo row"),
        integer("piece_index", "row of data (0-based)"),
        number("temperature_low", "temperature-ranges[i]"),
        number("temperature_high", "temperature-ranges[i+1]"),
        floats(
            "coefficients",
            "data[i]",
            "the coefficient row as written (7 for NASA-7 and Shomate, 9 for NASA-9); " + UNIT_NOTE,
        ),
        integer("coefficient_count", "length of data[i]"),
    ),
    "species_transport": table_schema(
        SPECIES_LOCATOR,
        text("model", "model"),
        text("geometry", "geometry"),
        number("diameter", "diameter"),
        number("well_depth", "well-depth"),
        number("dipole", "dipole"),
        number("polarizability", "polarizability"),
        number("rotational_relaxation", "rotational-relaxation"),
        number("acentric_factor", "acentric-factor"),
        number("dispersion_coefficient", "dispersion-coefficient"),
        number("quadrupole_polarizability", "quadrupole-polarizability"),
        text("note", "note"),
    ),
    "species_critical_parameters": table_schema(
        SPECIES_LOCATOR,
        number("critical_temperature", "critical-temperature"),
        *quantity("critical_pressure", "critical-pressure"),
        number("acentric_factor", "acentric-factor"),
        number("critical_molar_volume", "critical-molar-volume"),
        number("critical_compressibility", "critical-compressibility"),
        text("source", "source"),
    ),
    "species_equation_of_state": table_schema(
        SPECIES_LOCATOR,
        text("model", "model"),
    ),
    "phases": _entry(
        text("name", "name"),
        text("thermo", "thermo"),
        text("kinetics", "kinetics"),
        text("transport", "transport"),
        *quantity("site_density", "site-density"),
        *quantity("density", "density"),
        text("standard_concentration_basis", "standard-concentration-basis"),
        text("pure_fluid_name", "pure-fluid-name"),
        flag("skip_undeclared_elements", "skip-undeclared-elements"),
        flag("skip_undeclared_third_bodies", "skip-undeclared-third-bodies"),
        text("note", "note"),
        strings("adjacent_phases", "adjacent-phases"),
    ),
    "phase_references": table_schema(
        text("phase_locator", "_locator of the phases row"),
        col(
            "reference",
            STRING,
            "which phase key: elements, species or reactions",
            NOT_APPLICABLE,
        ),
        integer("position", "position in the list (null when the key's value is a string)"),
        col(
            "form",
            STRING,
            "string (the whole key is a word such as all), name (a list item naming one "
            "entry) or section (a single-key map of a section to its names)",
            NOT_APPLICABLE,
        ),
        text("name", "the list item"),
        text("section", "the map key: a section of this file, file/section, or default"),
        text("selection", "the word all (or another string) in place of a list of names"),
        strings("names", "the map value"),
    ),
    "reactions": _entry(
        text("equation", "equation"),
        col(
            "arrow",
            STRING,
            "the arrow token of the equation: <=>, => or =",
            NOT_APPLICABLE,
        ),
        flag("equation_parsed", "whether reaction_terms holds the equation's terms"),
        text("type", "type"),
        flag("duplicate", "duplicate"),
        text("id", "id"),
        text("note", "note"),
        flag("negative_A", "negative-A"),
        flag("negative_orders", "negative-orders"),
        flag("nonreactant_orders", "nonreactant-orders"),
        number("default_efficiency", "default-efficiency"),
        text("sticking_species", "sticking-species"),
    ),
    "reaction_terms": table_schema(
        text("reaction_locator", "_locator of the reactions row"),
        col("side", STRING, "reactant or product", NOT_APPLICABLE),
        integer("position", "term position within the side (0-based)"),
        col(
            "role",
            STRING,
            "participant, or falloff_collider for a trailing (+M) or (+ M)",
            NOT_APPLICABLE,
        ),
        number(
            "coefficient",
            "stoichiometric coefficient token",
            "null when the equation writes none (Cantera then uses 1)",
        ),
        text("species", "species token"),
    ),
    "collisions": _entry(
        text("target", "target"),
        text("product", "product"),
        text("kind", "kind"),
        number("threshold", "threshold"),
        text("note", "note"),
    ),
    "other_entries": _entry(
        col("shape", STRING, "mapping, list or scalar", NOT_APPLICABLE),
    ),
    "entry_parameters": table_schema(
        col(
            "entry_table",
            STRING,
            "the table that holds the owning row",
            NOT_APPLICABLE,
        ),
        text("entry_locator", "_locator of the owning row"),
        text(
            "path",
            "JSON pointer of the leaf relative to the owning row",
        ),
        text("block", "first key of the path: the source key this leaf belongs to"),
        *VALUE_FIELDS,
    ),
    "text_files": table_schema(
        col("encoding", STRING, "decoding applied: utf-8, or latin-1", NOT_APPLICABLE),
        integer("line_count", "number of lines"),
        col("byte_count", INT64, "size of the file", "bytes"),
    ),
    "text_lines": table_schema(
        integer("line", "line number"),
        text("text", "line content without its \\n"),
    ),
}
