# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The JSON files of the chemicals payload (objects keyed by CAS), one or two tables each.

| File | Tables |
|---|---|
| `Heat Capacity/JANAF_1998_{gas,liq,solid}_Cp.json` | `heat_capacity_janaf_{gas,liquid,solid}_cp`: one row per point |
| `Heat Capacity/Perrys Table 2-151.json` | `heat_capacity_perry_2_151_json`: one row per phase entry |
| `Heat Capacity/psi4_{adjusted,unadjusted}_characteristic_temperatures.json` | `heat_capacity_psi4_{adjusted,unadjusted}_temperatures`: one row per list item |
| `Heat Capacity/webbook_shomate_coefficients.json` | `heat_capacity_webbook_shomate`: one row per piece |
| `Misc/VDI Saturation Compounds Data.json` | `misc_vdi_saturation_compounds`, `misc_vdi_saturation_points` |
| `Safety/Ontario Exposure Limits.json` | `safety_ontario_exposure_limits_json` |

Column names come from the files' own keys where the file has any. The JANAF point tables and the
Shomate pieces are positional arrays in the file; their column names (`temperature`, `cp`, `slot`,
`Tmin` ... `E`) follow the library that reads them and each note says so. The Shomate file holds,
for every CAS, three slots (solid, liquid, gas), each `null` or a list of pieces `[Tmin, Tmax, A, B,
C, D, E]`; `slot` is the zero-based position and a `null` slot has no row. The Ontario file states
units inside its keys (`TWA (ppm)`, `STEL (mg/m^3)`), and the column units repeat them. A JSON
`null` is a null cell; JSON numbers are stored as float64.
"""

from __future__ import annotations

from collections.abc import Iterator
from pathlib import Path

import pyarrow as pa

from thermo_knowledge.readers.chemicals import tabular
from thermo_knowledge.readers.chemicals.tabular import flag, index, number, text
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.writer import Writer

POSITIONAL_NOTE = (
    "the file holds a positional array; the name and meaning follow the library that reads it "
    "(chemicals.heat_capacity); the file states no unit"
)

MIXED_NOTE = (
    "the value when the file writes it as a JSON string (for example an empty string or a "
    "suffixed grade such as 2a); the numeric column holds it when the file writes a number"
)

_JANAF_POINT = (
    text("CAS", "object key"),
    index("point_index", "position within the two parallel arrays"),
    number("temperature", "first array", note=POSITIONAL_NOTE),
    number("cp", "second array", note=POSITIONAL_NOTE),
)
_PERRY_KEYS = ("Formula", "Phase", "Subphase", "Const", "Lin", "Quadinv", "Quad", "Tmin", "Tmax", "Error")
_PERRY = (
    text("CAS", "object key"),
    text("phase_key", "key of the phase object (c, l, g, gls)"),
    text("Formula", "Formula"),
    text("Phase", "Phase"),
    text("Subphase", "Subphase"),
    number("Const", "Const"),
    number("Lin", "Lin"),
    number("Quadinv", "Quadinv"),
    number("Quad", "Quad"),
    number("Tmin", "Tmin"),
    text("Tmin_text", "Tmin", note=MIXED_NOTE),
    number("Tmax", "Tmax"),
    text("Tmax_text", "Tmax", note=MIXED_NOTE),
    number("Error", "Error"),
    text("Error_text", "Error", note=MIXED_NOTE),
)
_PSI4 = (
    text("CAS", "object key"),
    index("item_index", "position within the array"),
    number("value", "array item"),
)
_SHOMATE_NAMES = ("Tmin", "Tmax", "A", "B", "C", "D", "E")
_SHOMATE = (
    text("CAS", "object key"),
    index("slot", "position of the phase slot (chemicals: solid, liquid, gas)"),
    index("piece_index", "position of the piece within its slot"),
    *(number(name, f"piece item {i}", note=POSITIONAL_NOTE) for i, name in enumerate(_SHOMATE_NAMES)),
)
_VDI_KEYS = (
    "Name", "MW", "Tc", "T", "P", "Density (l)", "Density (g)", "Hvap", "Cp (l)", "Cp (g)",
    "Mu (l)", "Mu (g)", "K (l)", "K (g)", "Pr (l)", "Pr (g)", "sigma", "Beta", "Volume (l)",
    "Volume (g)",
)  # fmt: skip
_VDI_ARRAYS = (
    ("T", "T"), ("P", "P"), ("Density_l", "Density (l)"), ("Density_g", "Density (g)"),
    ("Hvap", "Hvap"), ("Cp_l", "Cp (l)"), ("Cp_g", "Cp (g)"), ("Mu_l", "Mu (l)"),
    ("Mu_g", "Mu (g)"), ("K_l", "K (l)"), ("K_g", "K (g)"), ("Pr_l", "Pr (l)"),
    ("Pr_g", "Pr (g)"), ("sigma", "sigma"), ("Beta", "Beta"), ("Volume_l", "Volume (l)"),
    ("Volume_g", "Volume (g)"),
)  # fmt: skip
_VDI_COMPOUND = (
    text("CAS", "object key"),
    text("Name", "Name"),
    number("MW", "MW"),
    number("Tc", "Tc"),
)
_VDI_POINT = (
    text("CAS", "object key"),
    index("point_index", "position within the parallel arrays"),
    *(number(name, key) for name, key in _VDI_ARRAYS),
)
_ONTARIO_KEYS = (
    "Name", "TWA (ppm)", "TWA (mg/m^3)", "STEL (ppm)", "STEL (mg/m^3)", "Ceiling (ppm)",
    "Ceiling (mg/m^3)", "Skin", "MW",
)  # fmt: skip
_ONTARIO = (
    text("CAS", "object key"),
    text("Name", "Name"),
    number("TWA_ppm", "TWA (ppm)", unit="ppm"),
    number("TWA_mg_m3", "TWA (mg/m^3)", unit="mg/m^3"),
    number("STEL_ppm", "STEL (ppm)", unit="ppm"),
    number("STEL_mg_m3", "STEL (mg/m^3)", unit="mg/m^3"),
    number("Ceiling_ppm", "Ceiling (ppm)", unit="ppm"),
    number("Ceiling_mg_m3", "Ceiling (mg/m^3)", unit="mg/m^3"),
    flag("Skin", "Skin"),
    number("MW", "MW"),
)

JANAF = {
    "chemicals/Heat Capacity/JANAF_1998_gas_Cp.json": "heat_capacity_janaf_gas_cp",
    "chemicals/Heat Capacity/JANAF_1998_liq_Cp.json": "heat_capacity_janaf_liquid_cp",
    "chemicals/Heat Capacity/JANAF_1998_solid_Cp.json": "heat_capacity_janaf_solid_cp",
}
PERRY_FILE = "chemicals/Heat Capacity/Perrys Table 2-151.json"
PSI4 = {
    "chemicals/Heat Capacity/psi4_adjusted_characteristic_temperatures.json": (
        "heat_capacity_psi4_adjusted_temperatures"
    ),
    "chemicals/Heat Capacity/psi4_unadjusted_characteristic_temperatures.json": (
        "heat_capacity_psi4_unadjusted_temperatures"
    ),
}
SHOMATE_FILE = "chemicals/Heat Capacity/webbook_shomate_coefficients.json"
VDI_FILE = "chemicals/Misc/VDI Saturation Compounds Data.json"
ONTARIO_FILE = "chemicals/Safety/Ontario Exposure Limits.json"

SCHEMAS: dict[str, pa.Schema] = {
    **{table: tabular.schema(*_JANAF_POINT) for table in JANAF.values()},
    "heat_capacity_perry_2_151_json": tabular.schema(*_PERRY),
    **{table: tabular.schema(*_PSI4) for table in PSI4.values()},
    "heat_capacity_webbook_shomate": tabular.schema(*_SHOMATE),
    "misc_vdi_saturation_compounds": tabular.schema(*_VDI_COMPOUND),
    "misc_vdi_saturation_points": tabular.schema(*_VDI_POINT),
    "safety_ontario_exposure_limits_json": tabular.schema(*_ONTARIO),
}


def _cas_object(tree: Path, artifact: str) -> dict[str, object]:
    return tabular.as_object(tabular.load_json(tree, artifact), artifact)


def _row(artifact: str, *tokens: object) -> dict[str, object]:
    return {"_artifact": artifact, "_locator": tabular.locator(artifact, tabular.pointer(*tokens))}


def _place(artifact: str, *tokens: object) -> str:
    return tabular.locator(artifact, tabular.pointer(*tokens))


def _parallel(artifact: str, cas: str, arrays: dict[str, object]) -> dict[str, list[float | None]]:
    parsed = {
        name: tabular.json_floats(value, _place(artifact, cas, name))
        for name, value in arrays.items()
    }
    lengths = {len(values) for values in parsed.values()}
    if len(lengths) > 1:
        raise StagingError(
            f"{_place(artifact, cas)}: parallel arrays of different lengths "
            f"{ {name: len(v) for name, v in parsed.items()} }"
        )
    return parsed


def read_janaf(table: str) -> tabular.Handler:
    def read(tree: Path, artifact: str, writer: Writer) -> None:
        def rows() -> Iterator[dict[str, object]]:
            for cas, value in _cas_object(tree, artifact).items():
                pair = tabular.as_list(value, _place(artifact, cas))
                if len(pair) != 2:
                    raise StagingError(f"{_place(artifact, cas)}: expected [temperatures, values]")
                arrays = _parallel(artifact, cas, {"temperature": pair[0], "cp": pair[1]})
                for position, (t, cp) in enumerate(
                    zip(arrays["temperature"], arrays["cp"], strict=True)
                ):
                    yield {
                        **_row(artifact, cas, position),
                        "CAS": cas,
                        "point_index": position,
                        "temperature": t,
                        "cp": cp,
                    }

        writer.rows(table, rows())
        writer.opened(artifact)

    return read


def _number_or_text(body: dict[str, object], keys: tuple[str, ...], place: str) -> dict[str, object]:
    """A field that the file writes as a number in most entries and as a string in a few: the
    number goes to the column, the string to `<key>_text`."""
    out: dict[str, object] = {}
    for key in keys:
        value = body.get(key)
        if isinstance(value, str):
            out[key] = None
            out[f"{key}_text"] = value
        else:
            out[key] = tabular.json_float(value, f"{place}/{key}")
            out[f"{key}_text"] = None
    return out


def read_perry(tree: Path, artifact: str, writer: Writer) -> None:
    rows: list[dict[str, object]] = []
    for cas, value in _cas_object(tree, artifact).items():
        phases = tabular.as_object(value, _place(artifact, cas))
        for key, entry in phases.items():
            place = _place(artifact, cas, key)
            body = tabular.as_object(entry, place)
            tabular.only_keys(body, _PERRY_KEYS, place)
            rows.append(
                {
                    **_row(artifact, cas, key),
                    "CAS": cas,
                    "phase_key": key,
                    "Formula": tabular.json_text(body.get("Formula"), place),
                    "Phase": tabular.json_text(body.get("Phase"), place),
                    "Subphase": tabular.json_text(body.get("Subphase"), place),
                    **{
                        name: tabular.json_float(body.get(name), f"{place}/{name}")
                        for name in ("Const", "Lin", "Quadinv", "Quad")
                    },
                    **_number_or_text(body, ("Tmin", "Tmax", "Error"), place),
                }
            )
    writer.rows("heat_capacity_perry_2_151_json", rows)
    writer.opened(artifact)


def read_psi4(table: str) -> tabular.Handler:
    def read(tree: Path, artifact: str, writer: Writer) -> None:
        rows: list[dict[str, object]] = []
        for cas, value in _cas_object(tree, artifact).items():
            values = tabular.json_floats(value, _place(artifact, cas))
            for position, item in enumerate(values):
                rows.append(
                    {**_row(artifact, cas, position), "CAS": cas, "item_index": position, "value": item}
                )
        writer.rows(table, rows)
        writer.opened(artifact)

    return read


def read_shomate(tree: Path, artifact: str, writer: Writer) -> None:
    rows: list[dict[str, object]] = []
    for cas, value in _cas_object(tree, artifact).items():
        slots = tabular.as_list(value, _place(artifact, cas))
        if len(slots) != 3:
            raise StagingError(f"{_place(artifact, cas)}: expected three phase slots")
        for slot, pieces in enumerate(slots):
            if pieces is None:
                continue
            for piece_index, piece in enumerate(tabular.as_list(pieces, _place(artifact, cas, slot))):
                place = _place(artifact, cas, slot, piece_index)
                items = tabular.json_floats(piece, place)
                if len(items) != len(_SHOMATE_NAMES):
                    raise StagingError(f"{place}: expected {len(_SHOMATE_NAMES)} values")
                rows.append(
                    {
                        **_row(artifact, cas, slot, piece_index),
                        "CAS": cas,
                        "slot": slot,
                        "piece_index": piece_index,
                        **dict(zip(_SHOMATE_NAMES, items, strict=True)),
                    }
                )
    writer.rows("heat_capacity_webbook_shomate", rows)
    writer.opened(artifact)


def read_vdi(tree: Path, artifact: str, writer: Writer) -> None:
    compounds: list[dict[str, object]] = []
    points: list[dict[str, object]] = []
    for cas, value in _cas_object(tree, artifact).items():
        place = _place(artifact, cas)
        body = tabular.as_object(value, place)
        tabular.only_keys(body, _VDI_KEYS, place)
        compounds.append(
            {
                **_row(artifact, cas),
                "CAS": cas,
                "Name": tabular.json_text(body.get("Name"), place),
                "MW": tabular.json_float(body.get("MW"), place),
                "Tc": tabular.json_float(body.get("Tc"), place),
            }
        )
        arrays = _parallel(artifact, cas, {key: body[key] for _, key in _VDI_ARRAYS if key in body})
        width = max((len(v) for v in arrays.values()), default=0)
        for position in range(width):
            row: dict[str, object] = {
                **_row(artifact, cas, position),
                "CAS": cas,
                "point_index": position,
            }
            for name, key in _VDI_ARRAYS:
                row[name] = arrays[key][position] if key in arrays else None
            points.append(row)
    writer.rows("misc_vdi_saturation_compounds", compounds)
    writer.rows("misc_vdi_saturation_points", points)
    writer.opened(artifact)


def read_ontario(tree: Path, artifact: str, writer: Writer) -> None:
    rows: list[dict[str, object]] = []
    for cas, value in _cas_object(tree, artifact).items():
        place = _place(artifact, cas)
        body = tabular.as_object(value, place)
        tabular.only_keys(body, _ONTARIO_KEYS, place)
        row: dict[str, object] = {
            **_row(artifact, cas),
            "CAS": cas,
            "Name": tabular.json_text(body.get("Name"), place),
            "Skin": tabular.json_flag(body.get("Skin"), place),
            "MW": tabular.json_float(body.get("MW"), place),
        }
        for name, key in (
            ("TWA_ppm", "TWA (ppm)"),
            ("TWA_mg_m3", "TWA (mg/m^3)"),
            ("STEL_ppm", "STEL (ppm)"),
            ("STEL_mg_m3", "STEL (mg/m^3)"),
            ("Ceiling_ppm", "Ceiling (ppm)"),
            ("Ceiling_mg_m3", "Ceiling (mg/m^3)"),
        ):
            row[name] = tabular.json_float(body.get(key), f"{place}/{key}")
        rows.append(row)
    writer.rows("safety_ontario_exposure_limits_json", rows)
    writer.opened(artifact)


HANDLERS: dict[str, tabular.Handler] = {
    **{path: read_janaf(table) for path, table in JANAF.items()},
    PERRY_FILE: read_perry,
    **{path: read_psi4(table) for path, table in PSI4.items()},
    SHOMATE_FILE: read_shomate,
    VDI_FILE: read_vdi,
    ONTARIO_FILE: read_ontario,
}
