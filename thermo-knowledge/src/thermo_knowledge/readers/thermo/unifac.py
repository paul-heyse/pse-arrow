# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The UNIFAC-family data of the thermo payload: thirteen interaction-parameter tables, the DDBST
group-assignment table and the subgroup and main-group definitions written in `thermo/unifac.py`.

| Payload | Table |
|---|---|
| `Phase Change/*interaction parameters*.tsv` (13 files) | `unifac_interaction_parameters` |
| `Phase Change/DDBST UNIFAC assignments.tsv` | `ddbst_unifac_assignments`, `ddbst_unifac_assignment_pairs` |
| `unifac.py`: `UNIFAC_subgroup(...)` assignments | `unifac_subgroups` |
| `unifac.py`: main-group tuples and dictionaries | `unifac_main_groups` |

The interaction files have no headings and no statement of variant other than the file name, so
`_artifact` names the variant. Their width is 3 (`m`, `n`, `a`), 5 (plus `b`, `c`) or 7 (plus `Tmin`,
`Tmax`); a column the file does not have is null, and the width is constant within a file. `m` and `n`
are main-group numbers. The files state no units.

The DDBST file is `InChIKey`, three validity flags (`'1 1 1'`), then three cells of `'subgroup count'`
pairs. The cells are kept verbatim (`UNIFAC_cell`, `modified_cell`, `PSRK_cell`, trailing space
included) and each pair is also a row of `ddbst_unifac_assignment_pairs`; which scheme a cell
belongs to (original UNIFAC, modified Dortmund UNIFAC, PSRK) is the order the loader documents.
A cell with its flag 0 is kept: the file fills every cell.

`unifac.py` is Python source. Only its data constructs are read, by the `ast` module and without
importing it: assignments `XXSG[n] = UNIFAC_subgroup(group_id, group, main_group_id, main_group, R,
Q, smarts=, atoms=, bonds=, priority=, hydrogen_from_smarts=)` into `unifac_subgroups` and the main
group maps (`XXMG[n] = ("name", [subgroup ids])` and the dictionary literals of the same form) into
`unifac_main_groups` (a third tuple item, a description, is `description`); `set_name` is the module variable (`UFSG`, `DOUFSG`, ...), and `_locator` is
the source line. `R` and `Q` are the constructor's literals. For `smarts`, `atoms` and `bonds` the
column `<name>_expr` is the argument's source text; `smarts` (a list of strings) and `atoms_json`
are filled only where the argument is a literal, and null where it refers to another set's entry
(`UFSG[1].atoms`). Nothing in the module is evaluated.
"""

from __future__ import annotations

import ast
import json
import re
from pathlib import Path

import pyarrow as pa

from thermo_knowledge.staging import tabular
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.tabular import (
    Col,
    flag,
    index,
    integer,
    number,
    text,
    texts,
)
from thermo_knowledge.staging.writer import Writer

PHASE_CHANGE = "thermo/Phase Change"
UNIFAC_SOURCE = "thermo/unifac.py"
DDBST_FILE = f"{PHASE_CHANGE}/DDBST UNIFAC assignments.tsv"
INTERACTION_FILES = (
    f"{PHASE_CHANGE}/NIST KT 2011 interaction parameters.tsv",
    f"{PHASE_CHANGE}/PSRK interaction parameters.tsv",
    f"{PHASE_CHANGE}/UNIFAC 2.0 Dortmund interaction parameters.tsv",
    f"{PHASE_CHANGE}/UNIFAC 2.0 interaction parameters.tsv",
    f"{PHASE_CHANGE}/UNIFAC LLE interaction parameters.tsv",
    f"{PHASE_CHANGE}/UNIFAC Lyngby interaction parameters.tsv",
    f"{PHASE_CHANGE}/UNIFAC modified Dortmund interaction parameters 2006.tsv",
    f"{PHASE_CHANGE}/UNIFAC modified Dortmund interaction parameters.tsv",
    f"{PHASE_CHANGE}/UNIFAC modified NIST 2015 interaction parameters.tsv",
    f"{PHASE_CHANGE}/UNIFAC original interaction parameters.tsv",
    f"{PHASE_CHANGE}/VTPR 2012 interaction parameters.tsv",
    f"{PHASE_CHANGE}/VTPR 2014 interaction parameters.tsv",
    f"{PHASE_CHANGE}/VTPR 2016 interaction parameters.tsv",
)
_WIDTHS = (3, 5, 7)
_COLUMN_NOTE = "a column the file does not have is null; the file states no unit"
_INTERACTIONS = (
    integer("m", "column 1 (no heading): main group m"),
    integer("n", "column 2 (no heading): main group n"),
    number("a", "column 3 (no heading): a_mn", note=_COLUMN_NOTE),
    number("b", "column 4 (no heading), 5-column and 7-column files", note=_COLUMN_NOTE),
    number("c", "column 5 (no heading), 5-column and 7-column files", note=_COLUMN_NOTE),
    number("Tmin", "column 6 (no heading), 7-column files", note=_COLUMN_NOTE),
    number("Tmax", "column 7 (no heading), 7-column files", note=_COLUMN_NOTE),
)
_ASSIGNMENTS = (
    text("InChIKey", "column 1 (no heading): InChIKey"),
    integer("UNIFAC_flag", "column 2 (no heading), first flag", note="0 or 1"),
    integer("modified_flag", "column 2 (no heading), second flag", note="0 or 1"),
    integer("PSRK_flag", "column 2 (no heading), third flag", note="0 or 1"),
    text("UNIFAC_cell", "column 3 (no heading): original UNIFAC pairs", note="verbatim cell"),
    text("modified_cell", "column 4 (no heading): modified Dortmund pairs", note="verbatim cell"),
    text("PSRK_cell", "column 5 (no heading): PSRK pairs", note="verbatim cell"),
)
_SCHEMES = (
    ("UNIFAC", "UNIFAC_cell"),
    ("modified", "modified_cell"),
    ("PSRK", "PSRK_cell"),
)
_PAIRS = (
    text("InChIKey", "column 1 (no heading): InChIKey"),
    text("scheme", "which of the three cells: UNIFAC, modified, PSRK"),
    index("pair_index", "position of the pair within its cell"),
    integer("subgroup_id", "first number of the pair", note="-1 occurs in cells whose flag is 0"),
    integer("count", "second number of the pair"),
)
_SUBGROUPS = (
    text("set_name", "module variable holding the subgroup table"),
    integer("group_id", "first argument: group_id"),
    text("group", "second argument: group"),
    integer("main_group_id", "third argument: main_group_id"),
    text("main_group", "fourth argument: main_group"),
    number("R", "fifth argument: R"),
    number("Q", "sixth argument: Q"),
    text("smarts_expr", "smarts argument, source text"),
    texts(
        "smarts",
        "smarts argument",
        note="the literal string or list of strings; null for a reference",
    ),
    text("atoms_expr", "atoms argument, source text"),
    text(
        "atoms_json",
        "atoms argument",
        note="a literal dictionary as JSON text; null for a reference",
    ),
    text(
        "bonds_expr",
        "bonds argument, source text",
        note="bond kinds are names imported by the module",
    ),
    integer("priority", "priority argument"),
    flag("hydrogen_from_smarts", "hydrogen_from_smarts argument"),
)
_MAIN_GROUPS = (
    text("set_name", "module variable holding the main-group table"),
    integer("main_group_id", "key"),
    text("name", "first tuple item"),
    Col("subgroup_ids", "N", "second tuple item", note="the subgroup numbers of the main group"),
    text("description", "third tuple item", note="only some tables give one; null otherwise"),
)

SCHEMAS: dict[str, pa.Schema] = {
    "unifac_interaction_parameters": tabular.schema(*_INTERACTIONS),
    "ddbst_unifac_assignments": tabular.schema(*_ASSIGNMENTS),
    "ddbst_unifac_assignment_pairs": tabular.schema(*_PAIRS),
    "unifac_subgroups": tabular.schema(*_SUBGROUPS),
    "unifac_main_groups": tabular.schema(*_MAIN_GROUPS),
}


def read_interactions(tree: Path, artifact: str, writer: Writer) -> None:
    lines = tabular.split_lines(tabular.read_text(tree, artifact))
    rows = []
    width = None
    for number_, cells in tabular.cells_of(artifact, lines):
        place = tabular.where(artifact, number_)
        if len(cells) not in _WIDTHS:
            raise StagingError(f"{place}: {len(cells)} cells, expected one of {_WIDTHS}")
        if width is None:
            width = len(cells)
        elif len(cells) != width:
            raise StagingError(f"{place}: {len(cells)} cells, the file's first row has {width}")
        row: dict[str, object] = {
            "_artifact": artifact,
            "_locator": place,
            "m": tabular.value_of("i", cells[0], place, "m"),
            "n": tabular.value_of("i", cells[1], place, "n"),
        }
        for name, cell in zip(("a", "b", "c", "Tmin", "Tmax"), cells[2:], strict=False):
            row[name] = tabular.value_of("f", cell, place, name)
        rows.append(row)
    writer.rows("unifac_interaction_parameters", rows)
    writer.opened(artifact)


def _pairs(place: str, cell: str) -> list[tuple[int, int]]:
    tokens = cell.split()
    if len(tokens) % 2:
        raise StagingError(f"{place}: a cell of {len(tokens)} numbers is not made of pairs")
    numbers = []
    for token in tokens:
        value = tabular.value_of("i", token, place, "pair")
        assert isinstance(value, int)
        numbers.append(value)
    return list(zip(numbers[0::2], numbers[1::2], strict=True))


def read_ddbst(tree: Path, artifact: str, writer: Writer) -> None:
    lines = tabular.split_lines(tabular.read_text(tree, artifact))
    rows = []
    pair_rows = []
    for number_, cells in tabular.cells_of(artifact, lines):
        place = tabular.where(artifact, number_)
        if len(cells) != 5:
            raise StagingError(f"{place}: {len(cells)} cells, expected 5")
        flags = cells[1].split(" ")
        if len(flags) != 3 or any(item not in ("0", "1") for item in flags):
            raise StagingError(f"{place}: the flags {cells[1]!r} are not three of 0 or 1")
        rows.append(
            {
                "_artifact": artifact,
                "_locator": place,
                "InChIKey": cells[0],
                "UNIFAC_flag": int(flags[0]),
                "modified_flag": int(flags[1]),
                "PSRK_flag": int(flags[2]),
                "UNIFAC_cell": cells[2],
                "modified_cell": cells[3],
                "PSRK_cell": cells[4],
            }
        )
        for (scheme, _), cell in zip(_SCHEMES, cells[2:], strict=True):
            for position, (subgroup, count) in enumerate(_pairs(place, cell)):
                pair_rows.append(
                    {
                        "_artifact": artifact,
                        "_locator": f"{place}/{scheme}[{position}]",
                        "InChIKey": cells[0],
                        "scheme": scheme,
                        "pair_index": position,
                        "subgroup_id": subgroup,
                        "count": count,
                    }
                )
    writer.rows("ddbst_unifac_assignments", rows)
    writer.rows("ddbst_unifac_assignment_pairs", pair_rows)
    writer.opened(artifact)


# -- unifac.py -------------------------------------------------------------------------------------

SUBGROUP_SETS = (
    "UFSG", "DOUFSG", "VTPRSG", "NISTUFSG", "PSRKSG", "LLEUFSG", "LUFSG", "NISTKTUFSG",
)  # fmt: skip
MAIN_GROUP_SETS = (
    "UFMG", "DOUFMG", "VTPRMG", "NISTUFMG", "PSRKMG", "LLEMG", "LUFMG", "NISTKTUFMG",
)  # fmt: skip
_SET_NAME = re.compile(r"^[A-Z0-9]+(SG|MG)$")
_SUBGROUP_KEYWORDS = ("smarts", "atoms", "bonds", "priority", "hydrogen_from_smarts")
PARTLY_READ = (
    "Python source: only the UNIFAC_subgroup constructor assignments and the main-group tables "
    "are decomposed; the class, the functions, the lazy loaders and the docstrings are code"
)


def _literal(node: ast.AST, place: str) -> object:
    try:
        return ast.literal_eval(node)
    except (ValueError, SyntaxError) as error:
        raise StagingError(f"{place}: an argument that is not a literal: {error}") from error


def _optional_literal(node: ast.AST | None) -> object:
    if node is None:
        return None
    try:
        return ast.literal_eval(node)
    except ValueError, SyntaxError:
        return None


def _float(value: object, place: str) -> float:
    if isinstance(value, bool) or not isinstance(value, int | float):
        raise StagingError(f"{place}: expected a number, found {value!r}")
    return float(value)


def _subgroup_row(artifact: str, source: str, set_name: str, call: ast.Call) -> dict[str, object]:
    place = f"{artifact}#L{call.lineno}"
    if len(call.args) != 6:
        raise StagingError(f"{place}: UNIFAC_subgroup takes 6 positional arguments here")
    group_id, group, main_group_id, main_group, r_value, q_value = (
        _literal(arg, place) for arg in call.args
    )
    if not isinstance(group_id, int) or not isinstance(main_group_id, int):
        raise StagingError(f"{place}: group numbers must be integers")
    if not isinstance(group, str) or not isinstance(main_group, str):
        raise StagingError(f"{place}: group names must be text")
    keywords = {kw.arg: kw.value for kw in call.keywords}
    extra = sorted(set(keywords) - set(_SUBGROUP_KEYWORDS), key=str)
    if extra:
        raise StagingError(f"{place}: keyword arguments no column declares: {extra}")

    def expression(name: str) -> str | None:
        node = keywords.get(name)
        return None if node is None else ast.get_source_segment(source, node)

    smarts = _optional_literal(keywords.get("smarts"))
    if isinstance(smarts, str):
        smarts = [smarts]
    if smarts is not None and not (
        isinstance(smarts, list) and all(isinstance(item, str) for item in smarts)
    ):
        raise StagingError(f"{place}: smarts is neither a string nor a list of strings")
    atoms = _optional_literal(keywords.get("atoms"))
    if atoms is not None and not isinstance(atoms, dict):
        raise StagingError(f"{place}: atoms is not a dictionary")
    priority = _optional_literal(keywords.get("priority"))
    if priority is not None and (isinstance(priority, bool) or not isinstance(priority, int)):
        raise StagingError(f"{place}: priority is not an integer")
    hydrogen = _optional_literal(keywords.get("hydrogen_from_smarts"))
    if hydrogen is not None and not isinstance(hydrogen, bool):
        raise StagingError(f"{place}: hydrogen_from_smarts is not a boolean")
    return {
        "_artifact": artifact,
        "_locator": place,
        "set_name": set_name,
        "group_id": group_id,
        "group": group,
        "main_group_id": main_group_id,
        "main_group": main_group,
        "R": _float(r_value, place),
        "Q": _float(q_value, place),
        "smarts_expr": expression("smarts"),
        "smarts": smarts,
        "atoms_expr": expression("atoms"),
        "atoms_json": None if atoms is None else json.dumps(atoms, separators=(",", ":")),
        "bonds_expr": expression("bonds"),
        "priority": priority,
        "hydrogen_from_smarts": hydrogen,
    }


def _main_group_row(
    artifact: str, set_name: str, key: object, value: ast.AST, line: int
) -> dict[str, object]:
    place = f"{artifact}#L{line}"
    item = _literal(value, place)
    if (
        not isinstance(key, int)
        or not isinstance(item, tuple)
        or len(item) not in (2, 3)
        or not isinstance(item[0], str)
        or not isinstance(item[1], list)
        or not all(isinstance(entry, int) for entry in item[1])
        or (len(item) == 3 and not isinstance(item[2], str))
    ):
        raise StagingError(
            f"{place}: a main-group entry is (integer key) -> (name, [integers][, description])"
        )
    return {
        "_artifact": artifact,
        "_locator": place,
        "set_name": set_name,
        "main_group_id": key,
        "name": item[0],
        "subgroup_ids": list(item[1]),
        "description": item[2] if len(item) == 3 else None,
    }


def read_unifac_source(tree: Path, artifact: str, writer: Writer) -> None:
    source = tabular.read_text(tree, artifact)
    try:
        module = ast.parse(source, filename=artifact)
    except SyntaxError as error:
        raise StagingError(f"{artifact}: cannot be parsed as Python: {error}") from error
    subgroups: list[dict[str, object]] = []
    main_groups: list[dict[str, object]] = []
    for node in module.body:
        if not isinstance(node, ast.Assign) or len(node.targets) != 1:
            continue
        target = node.targets[0]
        if isinstance(target, ast.Subscript) and isinstance(target.value, ast.Name):
            name = target.value.id
            if name in SUBGROUP_SETS:
                if not isinstance(node.value, ast.Call):
                    raise StagingError(f"{artifact}#L{node.lineno}: {name}[..] is not a call")
                row = _subgroup_row(artifact, source, name, node.value)
                if _literal(target.slice, f"{artifact}#L{node.lineno}") != row["group_id"]:
                    raise StagingError(
                        f"{artifact}#L{node.lineno}: the key of {name}[..] differs from group_id"
                    )
                subgroups.append(row)
            elif name in MAIN_GROUP_SETS:
                key = _literal(target.slice, f"{artifact}#L{node.lineno}")
                main_groups.append(_main_group_row(artifact, name, key, node.value, node.lineno))
            elif _SET_NAME.match(name):
                raise StagingError(
                    f"{artifact}#L{node.lineno}: {name}[..] looks like a group table the reader "
                    "does not know; add it and bump READER_VERSION"
                )
        elif isinstance(target, ast.Name) and target.id in MAIN_GROUP_SETS:
            if isinstance(node.value, ast.Dict):
                for key_node, value_node in zip(node.value.keys, node.value.values, strict=True):
                    if key_node is None:
                        raise StagingError(f"{artifact}#L{node.lineno}: dictionary unpacking")
                    key = _literal(key_node, f"{artifact}#L{key_node.lineno}")
                    main_groups.append(
                        _main_group_row(artifact, target.id, key, value_node, key_node.lineno)
                    )
            else:
                raise StagingError(f"{artifact}#L{node.lineno}: {target.id} is not a dictionary")
    writer.rows("unifac_subgroups", subgroups)
    writer.rows("unifac_main_groups", main_groups)
    writer.partly_read(artifact, PARTLY_READ)


HANDLERS: dict[str, tabular.Handler] = {
    **{path: read_interactions for path in INTERACTION_FILES},
    DDBST_FILE: read_ddbst,
    UNIFAC_SOURCE: read_unifac_source,
}
