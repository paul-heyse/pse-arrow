# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Identifier files of the chemicals payload: the PubChem-derived and ion and inorganic
identifier rows (headerless, positional, variable width), the preference lists, the library-minted
CAS registry, the mixture compositions and the DIPPR 2014 compound list.

| File | Table |
|---|---|
| `Identifiers/chemical identifiers *.tsv`, `Cation db`, `Anion db`, `Inorganic db` | `identifiers_chemical` |
| `Identifiers/*_preferences.json` | `identifiers_preferences` |
| `Identifiers/Fake CAS Registry.tsv` | `identifiers_fake_cas` |
| `Identifiers/Mixtures Compositions.tsv` | `identifiers_mixtures`, `identifiers_mixture_components` |
| `Identifiers/dippr_2014.csv` | `identifiers_dippr_2014` |

The positional columns of an identifier row are named as the library names them
(pubchemid, CAS, formula, MW, smiles, InChI, InChI_key, iupac_name, common_name); every further
cell is a synonym and goes to the list column `synonyms`, a blank cell in it staying null. The
files hold no quoting, so a quote is an ordinary character. A missing value is a blank cell and
becomes null: in particular `pubchemid` -1 is kept as -1, because that is what the file writes.
"""

from __future__ import annotations

from collections.abc import Iterator
from pathlib import Path

import pyarrow as pa

from thermo_knowledge.staging import tabular
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.tabular import index, integer, number, text, texts
from thermo_knowledge.staging.writer import Writer

IDENTIFIER_FILES = (
    "chemicals/Identifiers/Anion db.tsv",
    "chemicals/Identifiers/Cation db.tsv",
    "chemicals/Identifiers/Inorganic db.tsv",
    "chemicals/Identifiers/chemical identifiers example user db.tsv",
    "chemicals/Identifiers/chemical identifiers pubchem large.tsv",
    "chemicals/Identifiers/chemical identifiers pubchem small.tsv",
)
PREFERENCE_FILES = (
    "chemicals/Identifiers/anion_preferences.json",
    "chemicals/Identifiers/cation_preferences.json",
    "chemicals/Identifiers/inorganic_preferences.json",
    "chemicals/Identifiers/organic_preferences.json",
)
FAKE_CAS_FILE = "chemicals/Identifiers/Fake CAS Registry.tsv"
MIXTURE_FILE = "chemicals/Identifiers/Mixtures Compositions.tsv"
DIPPR_FILE = "chemicals/Identifiers/dippr_2014.csv"

POSITIONAL = 9
_POSITIONAL_COLUMNS = (
    ("pubchemid", "i"),
    ("CAS", "s"),
    ("formula", "s"),
    ("MW", "f"),
    ("smiles", "s"),
    ("InChI", "s"),
    ("InChI_key", "s"),
    ("iupac_name", "s"),
    ("common_name", "s"),
)

_CHEMICAL = (
    integer("pubchemid", "column 1 (no heading): pubchemid"),
    text("CAS", "column 2 (no heading): CAS"),
    text("formula", "column 3 (no heading): formula"),
    number("MW", "column 4 (no heading): MW", note="a blank cell is null"),
    text("smiles", "column 5 (no heading): smiles"),
    text("InChI", "column 6 (no heading): InChI"),
    text("InChI_key", "column 7 (no heading): InChI_key"),
    text("iupac_name", "column 8 (no heading): iupac_name"),
    text("common_name", "column 9 (no heading): common_name"),
    texts(
        "synonyms",
        "columns 10 onward (no heading): synonyms",
        note="every cell after the ninth, in file order; a blank cell is null",
    ),
)
_PREFERENCES = (
    text("list_name", "object key: preferred_cas or unpreferred_cas"),
    index("list_index", "position within the array"),
    text("CAS", "array item"),
)
_FAKE = (
    text("CAS", "column 1 (no heading): the library-minted CAS-format number"),
    text("key_1", "column 2 (no heading): key of the first key/value pair, or blank"),
    text("value_1", "column 3 (no heading): value of the first pair"),
    text("key_2", "column 4 (no heading): key of the second pair"),
    text("value_2", "column 5 (no heading): value of the second pair"),
)
_MIXTURES = (
    text("primary_name", "Primary Name"),
    text("source", "Source"),
    integer(
        "n_components", "N components", note="the count N that sizes the four per-component blocks"
    ),
    texts(
        "synonyms",
        "Synonyms",
        note="the cells after the four per-component blocks, without the blank padding that ends "
        "the line",
    ),
)
_COMPONENTS = (
    integer("mixture_line", "line of the mixture row", note="the physical line in the file"),
    index("component_index", "position within the N components"),
    text("CASRN", "CASRNs*N"),
    text("name", "Names*N"),
    number("mass_fraction", "mass fracs*N"),
    number("mole_fraction", "mole fracs*N"),
)
_DIPPR = (text("CAS", "column 1 (no heading): CAS"),)

SCHEMAS: dict[str, pa.Schema] = {
    "identifiers_chemical": tabular.schema(*_CHEMICAL),
    "identifiers_preferences": tabular.schema(*_PREFERENCES),
    "identifiers_fake_cas": tabular.schema(*_FAKE),
    "identifiers_mixtures": tabular.schema(*_MIXTURES),
    "identifiers_mixture_components": tabular.schema(*_COMPONENTS),
    "identifiers_dippr_2014": tabular.schema(*_DIPPR),
}


def _identifier_rows(artifact: str, content: str) -> Iterator[dict[str, object]]:
    for number_, cells in tabular.cells_of(artifact, tabular.split_lines(content)):
        place = tabular.where(artifact, number_)
        if len(cells) < POSITIONAL:
            raise StagingError(
                f"{place}: {len(cells)} cells, an identifier row has at least {POSITIONAL}"
            )
        row: dict[str, object] = {"_artifact": artifact, "_locator": place}
        for (name, kind), cell in zip(_POSITIONAL_COLUMNS, cells, strict=False):
            row[name] = tabular.value_of(kind, cell, place, name)
        row["synonyms"] = [cell or None for cell in cells[POSITIONAL:]]
        yield row


def read_identifiers(tree: Path, artifact: str, writer: Writer) -> None:
    writer.rows(
        "identifiers_chemical", _identifier_rows(artifact, tabular.read_text(tree, artifact))
    )
    writer.opened(artifact)


def read_preferences(tree: Path, artifact: str, writer: Writer) -> None:
    document = tabular.as_object(tabular.load_json(tree, artifact), artifact)
    tabular.only_keys(document, ("preferred_cas", "unpreferred_cas"), artifact)
    rows: list[dict[str, object]] = []
    for name in ("preferred_cas", "unpreferred_cas"):
        if name not in document:
            raise StagingError(f"{artifact}: the key {name} is missing")
        place = tabular.pointer(name)
        for position, item in enumerate(tabular.as_list(document[name], f"{artifact}#{place}")):
            item_place = tabular.pointer(name, position)
            rows.append(
                {
                    "_artifact": artifact,
                    "_locator": tabular.locator(artifact, item_place),
                    "list_name": name,
                    "list_index": position,
                    "CAS": tabular.json_text(item, f"{artifact}#{item_place}"),
                }
            )
    writer.rows("identifiers_preferences", rows)
    writer.opened(artifact)


def read_fake_cas(tree: Path, artifact: str, writer: Writer) -> None:
    rows: list[dict[str, object]] = []
    names = ("CAS", "key_1", "value_1", "key_2", "value_2")
    for number_, cells in tabular.cells_of(
        artifact, tabular.split_lines(tabular.read_text(tree, artifact))
    ):
        place = tabular.where(artifact, number_)
        if len(cells) != len(names):
            raise StagingError(f"{place}: {len(cells)} cells, the table declares {len(names)}")
        row: dict[str, object] = {"_artifact": artifact, "_locator": place}
        for name, cell in zip(names, cells, strict=True):
            row[name] = cell or None
        rows.append(row)
    writer.rows("identifiers_fake_cas", rows)
    writer.opened(artifact)


_MIXTURE_HEAD = (
    "Primary Name",
    "Source",
    "N components",
    "CASRNs*N",
    "Names*N",
    "mass fracs*N",
    "mole fracs*N",
    "Synonyms",
)


def read_mixtures(tree: Path, artifact: str, writer: Writer) -> None:
    lines = tabular.split_lines(tabular.read_text(tree, artifact))
    mixtures: list[dict[str, object]] = []
    components: list[dict[str, object]] = []
    for number_, cells in tabular.cells_of(artifact, lines):
        place = tabular.where(artifact, number_)
        if number_ == 1:
            if tuple(cells[: len(_MIXTURE_HEAD)]) != _MIXTURE_HEAD or any(
                cells[len(_MIXTURE_HEAD) :]
            ):
                raise StagingError(f"{place}: the headings differ from {_MIXTURE_HEAD!r}")
            continue
        if len(cells) < 3:
            raise StagingError(f"{place}: a mixture row has at least 3 cells")
        count = tabular.value_of("i", cells[2], place, "N components")
        if not isinstance(count, int) or count < 1:
            raise StagingError(f"{place}: N components must be a positive integer")
        block_end = 3 + 4 * count
        if len(cells) < block_end:
            raise StagingError(f"{place}: {len(cells)} cells, {count} components need {block_end}")
        blocks = [cells[3 + k * count : 3 + (k + 1) * count] for k in range(4)]
        rest = list(cells[block_end:])
        while rest and rest[-1] == "":
            rest.pop()
        mixtures.append(
            {
                "_artifact": artifact,
                "_locator": place,
                "primary_name": cells[0] or None,
                "source": cells[1] or None,
                "n_components": count,
                "synonyms": [cell or None for cell in rest],
            }
        )
        for k in range(count):
            components.append(
                {
                    "_artifact": artifact,
                    "_locator": f"{place}[{k}]",
                    "mixture_line": number_,
                    "component_index": k,
                    "CASRN": blocks[0][k] or None,
                    "name": blocks[1][k] or None,
                    "mass_fraction": tabular.value_of("f", blocks[2][k], place, "mass fracs*N"),
                    "mole_fraction": tabular.value_of("f", blocks[3][k], place, "mole fracs*N"),
                }
            )
    writer.rows("identifiers_mixtures", mixtures)
    writer.rows("identifiers_mixture_components", components)
    writer.opened(artifact)


def read_dippr(tree: Path, artifact: str, writer: Writer) -> None:
    rows = []
    for number_, cells in tabular.cells_of(
        artifact, tabular.split_lines(tabular.read_text(tree, artifact))
    ):
        place = tabular.where(artifact, number_)
        if len(cells) != 1:
            raise StagingError(f"{place}: {len(cells)} cells, the list has one")
        rows.append({"_artifact": artifact, "_locator": place, "CAS": cells[0] or None})
    writer.rows("identifiers_dippr_2014", rows)
    writer.opened(artifact)


HANDLERS: dict[str, tabular.Handler] = {
    **{path: read_identifiers for path in IDENTIFIER_FILES},
    **{path: read_preferences for path in PREFERENCE_FILES},
    FAKE_CAS_FILE: read_fake_cas,
    MIXTURE_FILE: read_mixtures,
    DIPPR_FILE: read_dippr,
}
