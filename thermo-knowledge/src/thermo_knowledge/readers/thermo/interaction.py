# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The interaction-parameter and scalar-parameter JSON files under `thermo/Interaction
Parameters/` and `thermo/Scalar Parameters/`.

Both families are `{"metadata": {...}, "data": {key: {parameters}}}` (the eppr78 file has no
metadata). `parameter_files` holds one row per file with its metadata as the file spells it
(`T dependent` becomes `T_dependent`, `necessary keys` becomes `necessary_keys`, `missing` is kept as
JSON text); the data rows go to one table per parameter family:

| Files | Table | Key |
|---|---|---|
| `ChemSep/henry.json`, `Sander_henry_const.json`, `Sander_henry_T_dep.json`, `PRTranslated_best_henry_T_dep.json` | `henry_pairs` | `'CAS1 CAS2'` |
| `ChemSep/nrtl.json`, `uniquac.json`, `wilson.json`, `pr.json` | `nrtl_pairs`, `uniquac_pairs`, `wilson_pairs`, `pr_kij_pairs` | `'CAS1 CAS2'` |
| `eppr78_common.json` | `eppr78_kij_pairs` | `'CAS1 CAS2'` |
| `Scalar Parameters/*.json` | `scalar_<file>` (eight tables) | `CAS` |

A pair key is kept as written (`pair`) and split on its single space into `CAS1` and `CAS2`, in the
order written: the tables do not say which orientation a row means beyond that order, and most
store both orientations as separate rows. Henry rows carry no unit in the file; `name` is only in
the ChemSep file, null elsewhere. The files state no units for any parameter (the JSON carries the
temperature-dependence flag, not the unit).
"""

from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path

import pyarrow as pa

from thermo_knowledge.readers.chemicals import tabular
from thermo_knowledge.readers.chemicals.tabular import Col, flag, integer, number, text, texts
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.writer import Writer

DIRECTORY = "thermo/Interaction Parameters"
SCALAR_DIRECTORY = "thermo/Scalar Parameters"
_METADATA_KEYS = (
    "symmetric",
    "source",
    "components",
    "necessary keys",
    "P dependent",
    "T dependent",
    "missing",
    "type",
)

_FILES = (
    text("type", "metadata/type"),
    text("source", "metadata/source"),
    flag("symmetric", "metadata/symmetric"),
    integer("components", "metadata/components"),
    flag("T_dependent", "metadata/T dependent"),
    flag("P_dependent", "metadata/P dependent"),
    texts("necessary_keys", "metadata/necessary keys"),
    text(
        "missing",
        "metadata/missing",
        note="the object that gives the value of an absent parameter, as JSON text",
    ),
)


@dataclass(frozen=True)
class Field:
    """One parameter: the staged column, the JSON key and its kind (`f` number or `s` text)."""

    column: str
    key: str
    kind: str = "f"


@dataclass(frozen=True)
class Family:
    """A data table: the files that hold it, how rows are keyed and which parameters they carry."""

    table: str
    files: tuple[str, ...]
    fields: tuple[Field, ...]
    pairs: bool = True
    lists: tuple[str, ...] = ()
    """Keys whose values are arrays of text (kept as list columns)."""

    def columns(self) -> tuple[Col, ...]:
        head: tuple[Col, ...]
        if self.pairs:
            head = (
                text("pair", "data key", note="'CAS1 CAS2', one space, as written"),
                text("CAS1", "data key, first token"),
                text("CAS2", "data key, second token"),
            )
        else:
            head = (text("CAS", "data key"),)
        body = tuple(
            texts(f.column, f.key)
            if f.key in self.lists
            else (text(f.column, f.key) if f.kind == "s" else number(f.column, f.key))
            for f in self.fields
        )
        return (*head, *body)


_HENRY = tuple(Field(name, name) for name in "ABCDEF")
FAMILIES: tuple[Family, ...] = (
    Family(
        "henry_pairs",
        (
            f"{DIRECTORY}/ChemSep/henry.json",
            f"{DIRECTORY}/Sander_henry_const.json",
            f"{DIRECTORY}/Sander_henry_T_dep.json",
            f"{DIRECTORY}/PRTranslated_best_henry_T_dep.json",
        ),
        (Field("name", "name", "s"), *_HENRY),
    ),
    Family(
        "nrtl_pairs",
        (f"{DIRECTORY}/ChemSep/nrtl.json",),
        (Field("name", "name", "s"), Field("bij", "bij"), Field("alphaij", "alphaij")),
    ),
    Family(
        "uniquac_pairs",
        (f"{DIRECTORY}/ChemSep/uniquac.json",),
        (Field("name", "name", "s"), Field("bij", "bij")),
    ),
    Family(
        "wilson_pairs",
        (f"{DIRECTORY}/ChemSep/wilson.json",),
        (Field("name", "name", "s"), Field("aij", "aij"), Field("bij", "bij")),
    ),
    Family(
        "pr_kij_pairs",
        (f"{DIRECTORY}/ChemSep/pr.json",),
        (
            Field("kij", "kij"),
            Field("Tmin", "Tmin"),
            Field("T", "T"),
            Field("Pmin", "Pmin"),
            Field("Pmax", "Pmax"),
            Field("P", "P"),
            Field("page", "page", "s"),
        ),
        lists=("page",),
    ),
    Family(
        "eppr78_kij_pairs",
        (f"{DIRECTORY}/eppr78_common.json",),
        (Field("kij", "kij"),),
    ),
    Family(
        "scalar_pr_twu_pina_martinez",
        (f"{SCALAR_DIRECTORY}/PRTwu_PinaMartinez.json",),
        (
            Field("name", "name", "s"),
            Field("TwuPRL", "TwuPRL"),
            Field("TwuPRM", "TwuPRM"),
            Field("TwuPRN", "TwuPRN"),
            Field("TwuPRc", "TwuPRc"),
        ),
        pairs=False,
    ),
    Family(
        "scalar_pr_twu_ibell_2018",
        (f"{SCALAR_DIRECTORY}/PRTwu_ibell_2018.json",),
        (
            Field("name", "name", "s"),
            Field("TwuPRL", "TwuPRL"),
            Field("TwuPRM", "TwuPRM"),
            Field("TwuPRN", "TwuPRN"),
        ),
        pairs=False,
    ),
    Family(
        "scalar_pr_volume_translation_pina_martinez",
        (f"{SCALAR_DIRECTORY}/PRVolumeTranslation_PinaMartinez.json",),
        (Field("name", "name", "s"), Field("PRc", "PRc")),
        pairs=False,
    ),
    Family(
        "scalar_srk_twu_pina_martinez",
        (f"{SCALAR_DIRECTORY}/SRKTwu_PinaMartinez.json",),
        (
            Field("name", "name", "s"),
            Field("TwuSRKL", "TwuSRKL"),
            Field("TwuSRKM", "TwuSRKM"),
            Field("TwuSRKN", "TwuSRKN"),
            Field("TwuSRKc", "TwuSRKc"),
        ),
        pairs=False,
    ),
    Family(
        "scalar_srk_volume_translation_pina_martinez",
        (f"{SCALAR_DIRECTORY}/SRKVolumeTranslation_PinaMartinez.json",),
        (Field("name", "name", "s"), Field("SRKc", "SRKc")),
        pairs=False,
    ),
    Family(
        "scalar_chemsep_apisrk",
        (f"{SCALAR_DIRECTORY}/chemsep_APISRK.json",),
        (
            Field("name", "name", "s"),
            Field("APISRKS1", "APISRKS1"),
            Field("APISRKS2", "APISRKS2"),
        ),
        pairs=False,
    ),
    Family(
        "scalar_chemsep_psrk_matthias_copeman",
        (f"{SCALAR_DIRECTORY}/chemsep_PSRK_matthias_copeman.json",),
        (
            Field("name", "name", "s"),
            Field("MCSRKC1", "MCSRKC1"),
            Field("MCSRKC2", "MCSRKC2"),
            Field("MCSRKC3", "MCSRKC3"),
        ),
        pairs=False,
    ),
    Family(
        "scalar_chemsep_regular_solution",
        (f"{SCALAR_DIRECTORY}/chemsep_regular_solution.json",),
        (
            Field("name", "name", "s"),
            Field("RegularSolutionSP", "RegularSolutionSP"),
            Field("RegularSolutionV", "RegularSolutionV"),
        ),
        pairs=False,
    ),
)

SCHEMAS: dict[str, pa.Schema] = {
    "parameter_files": tabular.schema(*_FILES),
    **{family.table: tabular.schema(*family.columns()) for family in FAMILIES},
}
_BY_FILE = {path: family for family in FAMILIES for path in family.files}


def _metadata_row(artifact: str, metadata: dict[str, object]) -> dict[str, object]:
    place = f"{artifact}#/metadata"
    tabular.only_keys(metadata, _METADATA_KEYS, place)
    necessary = metadata.get("necessary keys")
    return {
        "_artifact": artifact,
        "_locator": place,
        "type": tabular.json_text(metadata.get("type"), f"{place}/type"),
        "source": tabular.json_text(metadata.get("source"), f"{place}/source"),
        "symmetric": tabular.json_flag(metadata.get("symmetric"), f"{place}/symmetric"),
        "components": tabular.json_int(metadata.get("components"), f"{place}/components"),
        "T_dependent": tabular.json_flag(metadata.get("T dependent"), f"{place}/T dependent"),
        "P_dependent": tabular.json_flag(metadata.get("P dependent"), f"{place}/P dependent"),
        "necessary_keys": None
        if necessary is None
        else [
            tabular.json_text(item, f"{place}/necessary keys")
            for item in tabular.as_list(necessary, f"{place}/necessary keys")
        ],
        "missing": None
        if "missing" not in metadata
        else json.dumps(metadata["missing"], separators=(",", ":")),
    }


def _value(field: Field, family: Family, body: dict[str, object], place: str) -> object:
    raw = body.get(field.key)
    where_ = f"{place}/{field.key}"
    if field.key in family.lists:
        if raw is None:
            return None
        return [tabular.json_text(item, where_) for item in tabular.as_list(raw, where_)]
    if field.kind == "s":
        return tabular.json_text(raw, where_)
    return tabular.json_float(raw, where_)


def read_family(tree: Path, artifact: str, writer: Writer) -> None:
    family = _BY_FILE[artifact]
    document = tabular.as_object(tabular.load_json(tree, artifact), artifact)
    tabular.only_keys(document, ("metadata", "data"), artifact)
    if "data" not in document:
        raise StagingError(f"{artifact}: the key data is missing")
    if "metadata" in document:
        writer.rows(
            "parameter_files",
            [_metadata_row(artifact, tabular.as_object(document["metadata"], artifact))],
        )
    allowed = [f.key for f in family.fields]
    rows = []
    for key, value in tabular.as_object(document["data"], artifact).items():
        place = f"{artifact}#{tabular.pointer('data', key)}"
        body = tabular.as_object(value, place)
        tabular.only_keys(body, allowed, place)
        row: dict[str, object] = {"_artifact": artifact, "_locator": place}
        if family.pairs:
            tokens = key.split(" ")
            if len(tokens) != 2 or not all(tokens):
                raise StagingError(f"{place}: the key {key!r} is not 'CAS1 CAS2'")
            row.update({"pair": key, "CAS1": tokens[0], "CAS2": tokens[1]})
        else:
            row["CAS"] = key
        for field in family.fields:
            row[field.column] = _value(field, family, body, place)
        rows.append(row)
    writer.rows(family.table, rows)
    writer.opened(artifact)


HANDLERS: dict[str, tabular.Handler] = {path: read_family for path in _BY_FILE}
