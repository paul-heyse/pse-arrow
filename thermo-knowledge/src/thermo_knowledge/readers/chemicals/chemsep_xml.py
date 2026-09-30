# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The ChemSep v8.3 pure-component library (`Misc/ChemSep8.32.xml`).

The root `compounds` element holds one `compound` element per component. Every child of a
compound is one property element, and the file uses three shapes of it:

| Shape | Element | Table |
|---|---|---|
| a value | `<Tag name=".." units=".." value=".."/>` | `chemsep_scalars` |
| a temperature-dependent equation | `<Tag name=".." units="..">` holding `eqno`, `A` to `E`, `Tmin`, `Tmax` | `chemsep_equations` |
| a group list | `<Tag name="..">` holding `group id=".." value=".."` | `chemsep_groups` |

Tag names, `name` and `units` attributes are the file's own words. A compound is identified by its
zero-based position in the file (`compound_index`); its own keys (`LibraryIndex`, `CompoundID`,
`CAS`) are scalar rows like any other. `value` is the attribute text exactly as written, and
`value_number` is the same text parsed as a decimal number where it is one (null otherwise); the
unit `_` in the file marks a dimensionless value and is kept as written. An equation's coefficients
are float64 columns named as the file names them (A to E), null when the element has no such child;
`Tmin` and `Tmax` carry their own `units` attribute in `Tmin_units` and `Tmax_units`. Any attribute,
child or shape not listed here is an error, so a new construct is a visible failure.
"""

from __future__ import annotations

import re
import xml.etree.ElementTree as ET
from pathlib import Path

import pyarrow as pa

from thermo_knowledge.readers.chemicals import tabular
from thermo_knowledge.readers.chemicals.tabular import index, integer, number, text
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.writer import Writer

ARTIFACT = "chemicals/Misc/ChemSep8.32.xml"
_NUMBER = re.compile(r"^[+-]?(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?$")
_COEFFICIENTS = ("A", "B", "C", "D", "E")
_LIBRARY_ATTRIBUTES = ("library", "file", "create-date", "create-time", "create-user")

_LIBRARY = (
    text("library", "compounds/@library"),
    text("file", "compounds/@file"),
    text("create_date", "compounds/@create-date"),
    text("create_time", "compounds/@create-time"),
    text("create_user", "compounds/@create-user"),
)
_SCALARS = (
    index("compound_index", "position of the compound element"),
    text("element", "element name"),
    text("name", "@name"),
    text("units", "@units", note="absent on identifiers and names; `_` means dimensionless"),
    text("value", "@value", note="the attribute text exactly as written"),
    number("value_number", "@value", note="the same text parsed as a decimal number; null if it is not one"),
)
_EQUATIONS = (
    index("compound_index", "position of the compound element"),
    text("element", "element name"),
    text("name", "@name"),
    text("units", "@units"),
    integer("eqno", "eqno/@value", note="the equation number ChemSep assigns to the form"),
    *(number(name, f"{name}/@value") for name in _COEFFICIENTS),
    number("Tmin", "Tmin/@value", note="unit in Tmin_units"),
    text("Tmin_units", "Tmin/@units"),
    number("Tmax", "Tmax/@value", note="unit in Tmax_units"),
    text("Tmax_units", "Tmax/@units"),
)
_GROUPS = (
    index("compound_index", "position of the compound element"),
    text("element", "element name"),
    text("name", "@name"),
    index("group_position", "position of the group element within its list"),
    integer("group_id", "group/@id"),
    integer("count", "group/@value", note="the number of occurrences of the group"),
)

SCHEMAS: dict[str, pa.Schema] = {
    "chemsep_library": tabular.schema(*_LIBRARY),
    "chemsep_scalars": tabular.schema(*_SCALARS),
    "chemsep_equations": tabular.schema(*_EQUATIONS),
    "chemsep_groups": tabular.schema(*_GROUPS),
}


def _attributes(element: ET.Element, allowed: tuple[str, ...], place: str) -> None:
    extra = sorted(set(element.attrib) - set(allowed))
    if extra:
        raise StagingError(f"{place}: attributes no column declares: {', '.join(extra)}")


def _decimal(text_value: str, place: str) -> float:
    if not _NUMBER.match(text_value):
        raise StagingError(f"{place}: {text_value!r} is not a decimal number")
    return float(text_value)


def _whole(text_value: str, place: str) -> int:
    if not re.match(r"^[+-]?\d+$", text_value):
        raise StagingError(f"{place}: {text_value!r} is not an integer")
    return int(text_value)


def _equation(compound: int, child: ET.Element, place: str) -> dict[str, object]:
    _attributes(child, ("name", "units"), place)
    row: dict[str, object] = {
        "compound_index": compound,
        "element": child.tag,
        "name": child.attrib.get("name"),
        "units": child.attrib.get("units"),
    }
    seen: set[str] = set()
    for part in child:
        if part.tag in seen:
            raise StagingError(f"{place}/{part.tag}: repeated element")
        seen.add(part.tag)
        part_place = f"{place}/{part.tag}"
        if len(part):
            raise StagingError(f"{part_place}: unexpected nested elements")
        if part.tag == "eqno":
            _attributes(part, ("value",), part_place)
            row["eqno"] = _whole(part.attrib["value"], part_place)
        elif part.tag in _COEFFICIENTS:
            _attributes(part, ("value",), part_place)
            row[part.tag] = _decimal(part.attrib["value"], part_place)
        elif part.tag in ("Tmin", "Tmax"):
            _attributes(part, ("units", "value"), part_place)
            row[part.tag] = _decimal(part.attrib["value"], part_place)
            row[f"{part.tag}_units"] = part.attrib.get("units")
        else:
            raise StagingError(f"{part_place}: an equation part no column declares")
    return row


def read_chemsep(tree: Path, artifact: str, writer: Writer) -> None:
    try:
        root = ET.parse(tree / artifact).getroot()
    except (OSError, ET.ParseError) as error:
        raise StagingError(f"{artifact}: cannot be read as XML: {error}") from error
    if root.tag != "compounds":
        raise StagingError(f"{artifact}: the root element is {root.tag!r}, expected 'compounds'")
    _attributes(root, _LIBRARY_ATTRIBUTES, f"{artifact}#/compounds")
    writer.rows(
        "chemsep_library",
        [
            {
                "_artifact": artifact,
                "_locator": f"{artifact}#/compounds",
                "library": root.attrib.get("library"),
                "file": root.attrib.get("file"),
                "create_date": root.attrib.get("create-date"),
                "create_time": root.attrib.get("create-time"),
                "create_user": root.attrib.get("create-user"),
            }
        ],
    )
    scalars: list[dict[str, object]] = []
    equations: list[dict[str, object]] = []
    groups: list[dict[str, object]] = []
    for compound, element in enumerate(root):
        if element.tag != "compound":
            raise StagingError(f"{artifact}#/compounds/{element.tag}: expected 'compound'")
        _attributes(element, (), f"{artifact}#/compounds/compound[{compound}]")
        for child in element:
            place = f"{artifact}#/compounds/compound[{compound}]/{child.tag}"
            if "value" in child.attrib:
                if len(child):
                    raise StagingError(f"{place}: a value element with children")
                _attributes(child, ("name", "units", "value"), place)
                value = child.attrib["value"]
                scalars.append(
                    {
                        "_artifact": artifact,
                        "_locator": place,
                        "compound_index": compound,
                        "element": child.tag,
                        "name": child.attrib.get("name"),
                        "units": child.attrib.get("units"),
                        "value": value,
                        "value_number": float(value) if _NUMBER.match(value) else None,
                    }
                )
            elif child.find("eqno") is not None:
                row = _equation(compound, child, place)
                equations.append({"_artifact": artifact, "_locator": place, **row})
            elif len(child) and all(part.tag == "group" for part in child):
                _attributes(child, ("name",), place)
                for position, part in enumerate(child):
                    part_place = f"{place}/group[{position}]"
                    _attributes(part, ("id", "value"), part_place)
                    groups.append(
                        {
                            "_artifact": artifact,
                            "_locator": part_place,
                            "compound_index": compound,
                            "element": child.tag,
                            "name": child.attrib.get("name"),
                            "group_position": position,
                            "group_id": _whole(part.attrib["id"], part_place),
                            "count": _whole(part.attrib["value"], part_place),
                        }
                    )
            else:
                raise StagingError(f"{place}: an element of a shape no table declares")
    writer.rows("chemsep_scalars", scalars)
    writer.rows("chemsep_equations", equations)
    writer.rows("chemsep_groups", groups)
    writer.opened(artifact)


HANDLERS: dict[str, tabular.Handler] = {ARTIFACT: read_chemsep}
