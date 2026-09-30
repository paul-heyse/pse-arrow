# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""FeOS parameter records: pure-component, segment, binary and binary segment records, their
association sites and ePC-SAFT permittivity records.

The files hold JSON arrays. A record with an `identifier` object is a pure-component record, one
whose `identifier` is a plain string is a segment record (group-contribution files), one with
`id1` and `id2` objects is a binary record and one with `id1` and `id2` strings a binary segment
record. One row is written per record, with the identifier object's six keys as columns (null for
a key the record lacks) and the model fields under the source's own names; a model field a record
lacks is null, and `association_site_count` is null when the record has no `association_sites`
key. Units are those of the FeOS parameter structs' comments (`crates/feos/src/*/parameters.rs`);
the JSON files state none.
"""

from __future__ import annotations

from pathlib import Path

import pyarrow as pa

from thermo_knowledge.readers.feos.common import (
    NOT_APPLICABLE,
    Fields,
    JsonValue,
    Sink,
    identifier_columns,
    identifier_row,
    integer,
    is_number,
    load_array,
    locator,
    num,
    position,
    schema_of,
    text,
)
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.schema import column

SIGMA_NOTE = "FeOS parameter struct comment: segment diameter in units of Angstrom"
EPSILON_NOTE = "FeOS parameter struct comment: energetic parameter in units of Kelvin"

MODEL_DIRECTORY = text(
    "model_directory",
    source_name="second path segment of the file (parameters/<model>/)",
    unit=NOT_APPLICABLE,
    nullable=False,
)
RECORD_INDEX = position("record_index", "position of the record in the file's JSON array")


def _model_fields() -> list[pa.Field]:
    """The model fields of a pure or segment record, shared by both tables."""
    return [
        num(
            "molarweight",
            "g/mol",
            note="FeOS reads it as g/mol (molarweight converted with GRAM/MOL)",
        ),
        num("m", note="segment number"),
        num("sigma", "Angstrom", note=SIGMA_NOTE),
        num("epsilon_k", "K", note=EPSILON_NOTE),
        num("mu", "Debye", note="FeOS parameter struct comment: dipole moment in units of Debye"),
        num(
            "q",
            "Debye * Angstrom",
            note="FeOS parameter struct comment: quadrupole moment in units of Debye * Angstrom",
        ),
        num("z", note="ion charge number (ePC-SAFT); written as a JSON integer"),
        column(
            "viscosity",
            pa.list_(pa.float64()),
            source_name="viscosity",
            note="entropy scaling coefficients for the viscosity (four numbers)",
        ),
        num("lr", note="repulsive Mie exponent (SAFT-VR Mie, SAFT-VRQ Mie)"),
        num("la", note="attractive Mie exponent (SAFT-VR Mie, SAFT-VRQ Mie)"),
        integer("fh", source_name="fh", unit="not stated"),
        integer("association_site_count", source_name="association_sites (length of the array)"),
        text("permittivity_variant", source_name="permittivity_record (the one key of the object)"),
    ]


_MODEL_NUMBERS = ("molarweight", "m", "sigma", "epsilon_k", "mu", "q", "z", "lr", "la")

PURE_RECORDS = schema_of(
    MODEL_DIRECTORY,
    RECORD_INDEX,
    *identifier_columns("identifier", "identifier"),
    *_model_fields(),
)

SEGMENT_RECORDS = schema_of(
    MODEL_DIRECTORY,
    RECORD_INDEX,
    text("segment", source_name="identifier (a plain string: the segment name)"),
    *_model_fields(),
)

BINARY_RECORDS = schema_of(
    MODEL_DIRECTORY,
    RECORD_INDEX,
    *identifier_columns("id1", "id1"),
    *identifier_columns("id2", "id2"),
    num("k_ij", note="binary dispersion interaction parameter"),
    num("l_ij", note="SAFT-VRQ Mie binary correction to the diameter"),
    integer("association_site_count", source_name="association_sites (length of the array)"),
)

BINARY_SEGMENT_RECORDS = schema_of(
    MODEL_DIRECTORY,
    RECORD_INDEX,
    text("segment1", source_name="id1 (a plain string: a segment name)"),
    text("segment2", source_name="id2 (a plain string: a segment name)"),
    num("k_ij", note="binary dispersion interaction parameter"),
    integer("association_site_count", source_name="association_sites (length of the array)"),
)

ASSOCIATION_SITES = schema_of(
    text(
        "owner_table",
        source_name="the staged table of the record that holds the site",
        unit=NOT_APPLICABLE,
        nullable=False,
    ),
    RECORD_INDEX,
    position("site_index", "position in the association_sites array"),
    num("na", note="number of association sites of type A"),
    num("nb", note="number of association sites of type B"),
    num("kappa_ab", note="association volume parameter"),
    num("epsilon_k_ab", "K", note="association energy parameter in units of Kelvin"),
    num("rc_ab", note="association radius parameter (SAFT-VR Mie)"),
)

PERMITTIVITY_RECORDS = schema_of(
    RECORD_INDEX,
    text("variant", source_name="the one key of permittivity_record", nullable=False),
    integer("point_count", source_name="ExperimentalData/data (length of the array)"),
    num("dipole_scaling", source_name="PerturbationTheory/dipole_scaling"),
    num("polarizability_scaling", source_name="PerturbationTheory/polarizability_scaling"),
    num(
        "correlation_integral_parameter",
        source_name="PerturbationTheory/correlation_integral_parameter",
    ),
)

PERMITTIVITY_DATA_POINTS = schema_of(
    RECORD_INDEX,
    position("point_index", "position in ExperimentalData/data"),
    num("temperature", "K", source_name="data[i][0]", note="FeOS states temperature in K"),
    num("permittivity", source_name="data[i][1]"),
)

SCHEMAS = {
    "pure_records": PURE_RECORDS,
    "segment_records": SEGMENT_RECORDS,
    "binary_records": BINARY_RECORDS,
    "binary_segment_records": BINARY_SEGMENT_RECORDS,
    "association_sites": ASSOCIATION_SITES,
    "permittivity_records": PERMITTIVITY_RECORDS,
    "permittivity_data_points": PERMITTIVITY_DATA_POINTS,
}

_SITE_NUMBERS = ("na", "nb", "kappa_ab", "epsilon_k_ab", "rc_ab")


def model_directory(artifact: str) -> str:
    parts = artifact.split("/")
    if len(parts) < 3:
        raise StagingError(f"{artifact}: expected parameters/<model>/<file>.json")
    return parts[1]


def read_records(tree: Path, artifact: str, sink: Sink) -> None:
    """Every record of a parameter file whose records are pure, segment or binary records."""
    document = load_array(tree, artifact)
    directory = model_directory(artifact)
    for index, record in enumerate(document):
        where = locator(artifact, f"/{index}")
        if not isinstance(record, dict):
            raise StagingError(f"{where}: expected a JSON object, found {type(record).__name__}")
        if "id1" in record or "id2" in record:
            _binary(artifact, directory, index, record, where, sink)
        elif "identifier" in record:
            _pure_or_segment(artifact, directory, index, record, where, sink)
        else:
            raise StagingError(f"{where}: a record with neither identifier nor id1/id2")


def _sites(
    fields: Fields,
    artifact: str,
    owner: str,
    index: int,
    where: str,
    sink: Sink,
) -> int | None:
    sites = fields.array("association_sites")
    if sites is None:
        return None
    for site_index, raw in enumerate(sites):
        site_where = f"{where}/association_sites/{site_index}"
        site = Fields(raw, site_where)
        row: dict[str, JsonValue] = {
            "_artifact": artifact,
            "_locator": site_where,
            "owner_table": owner,
            "record_index": index,
            "site_index": site_index,
        }
        for name in _SITE_NUMBERS:
            row[name] = site.number(name)
        site.done()
        sink.add("association_sites", row)
    return len(sites)


def _permittivity(
    fields: Fields, artifact: str, index: int, where: str, sink: Sink
) -> str | None:
    block = fields.object("permittivity_record")
    if block is None:
        return None
    if len(block) != 1:
        raise StagingError(f"{where}/permittivity_record: expected an object with one variant key")
    ((variant, body),) = block.items()
    here = f"{where}/permittivity_record"
    row: dict[str, JsonValue] = {
        "_artifact": artifact,
        "_locator": here,
        "record_index": index,
        "variant": variant,
    }
    body_fields = Fields(body, f"{here}/{variant}")
    if variant == "ExperimentalData":
        data = body_fields.array("data")
        if data is None:
            raise StagingError(f"{here}/{variant}: no data array")
        row["point_count"] = len(data)
        for point_index, pair in enumerate(data):
            point_where = f"{here}/{variant}/data/{point_index}"
            if (
                not isinstance(pair, list)
                or len(pair) != 2
                or not all(is_number(item) for item in pair)
            ):
                raise StagingError(f"{point_where}: expected a [temperature, permittivity] pair")
            sink.add(
                "permittivity_data_points",
                {
                    "_artifact": artifact,
                    "_locator": point_where,
                    "record_index": index,
                    "point_index": point_index,
                    "temperature": float(pair[0]),
                    "permittivity": float(pair[1]),
                },
            )
    elif variant == "PerturbationTheory":
        for name in ("dipole_scaling", "polarizability_scaling", "correlation_integral_parameter"):
            row[name] = body_fields.number(name)
    else:
        raise StagingError(f"{here}: unknown permittivity variant {variant!r}")
    body_fields.done()
    sink.add("permittivity_records", row)
    return variant


def _pure_or_segment(
    artifact: str,
    directory: str,
    index: int,
    record: dict[str, JsonValue],
    where: str,
    sink: Sink,
) -> None:
    fields = Fields(record, where)
    identifier = fields.take("identifier")
    row: dict[str, JsonValue] = {
        "_artifact": artifact,
        "_locator": where,
        "model_directory": directory,
        "record_index": index,
    }
    if isinstance(identifier, dict):
        table = "pure_records"
        row.update(identifier_row("identifier", identifier, f"{where}/identifier"))
    elif isinstance(identifier, str):
        table = "segment_records"
        row["segment"] = identifier
    else:
        raise StagingError(f"{where}/identifier: expected an object or a string")
    for name in _MODEL_NUMBERS:
        row[name] = fields.number(name)
    row["viscosity"] = fields.number_list("viscosity")
    row["fh"] = fields.integer("fh")
    row["permittivity_variant"] = _permittivity(fields, artifact, index, where, sink)
    row["association_site_count"] = _sites(fields, artifact, table, index, where, sink)
    fields.done()
    sink.add(table, row)


def _binary(
    artifact: str,
    directory: str,
    index: int,
    record: dict[str, JsonValue],
    where: str,
    sink: Sink,
) -> None:
    fields = Fields(record, where)
    first, second = fields.take("id1"), fields.take("id2")
    row: dict[str, JsonValue] = {
        "_artifact": artifact,
        "_locator": where,
        "model_directory": directory,
        "record_index": index,
    }
    if isinstance(first, dict) and isinstance(second, dict):
        table = "binary_records"
        row.update(identifier_row("id1", first, f"{where}/id1"))
        row.update(identifier_row("id2", second, f"{where}/id2"))
        row["l_ij"] = fields.number("l_ij")
    elif isinstance(first, str) and isinstance(second, str):
        table = "binary_segment_records"
        row["segment1"], row["segment2"] = first, second
    else:
        raise StagingError(f"{where}: id1 and id2 must both be objects or both be strings")
    row["k_ij"] = fields.number("k_ij")
    row["association_site_count"] = _sites(fields, artifact, table, index, where, sink)
    fields.done()
    sink.add(table, row)
