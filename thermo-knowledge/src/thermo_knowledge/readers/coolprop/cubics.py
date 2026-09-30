# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Cubic pure-fluid records (`dev/cubics/all_cubic_fluids.json`), PC-SAFT fluids and binary pairs
(`dev/pcsaft/*.json`)."""

from __future__ import annotations

from pathlib import Path

from thermo_knowledge.readers.coolprop import fluids
from thermo_knowledge.readers.coolprop.common import (
    Fields,
    ObjectSpec,
    Sink,
    index,
    integer,
    load_list,
    locator,
    merge_schemas,
    num,
    pointer,
    text,
    texts,
)

CUBIC_NAME = text("name", source_name="name")
CUBIC_FLUIDS = ObjectSpec(
    "cubic_fluids",
    keys=(CUBIC_NAME,),
    scalars=(
        text("CAS"),
        num("Tc", "K", unit_from="Tc_units"),
        text("Tc_units"),
        num("acentric"),
        num("molemass", "kg/mol", unit_from="molemass_units"),
        text("molemass_units"),
        num("pc", "Pa", unit_from="pc_units"),
        text("pc_units"),
        num("rhomolarc", "mol/m^3", unit_from="rhomolarc_units"),
        text("rhomolarc_units"),
        texts("aliases"),
    ),
    lift=("name",),
    lengths=False,
)
CUBIC_ALPHA0 = fluids.alpha0_spec(
    "cubic_alpha0_terms",
    "cubic_alpha0_term_rows",
    (CUBIC_NAME, index("term_index", "position in alpha0"), fluids.TYPE),
)

PCSAFT_NAME = text("name", source_name="name")
PCSAFT_FLUIDS = ObjectSpec(
    "pcsaft_fluids",
    keys=(PCSAFT_NAME,),
    scalars=(
        text("BibTeX"),
        text("CAS"),
        integer("charge"),
        num("dipm", "Debye", unit_from="dipm_units"),
        text("dipm_units"),
        integer("dipnum"),
        num("m"),
        num("molemass", "kg/mol", unit_from="molemass_units"),
        text("molemass_units"),
        num("sigma", "Angstrom", unit_from="sigma_units"),
        text("sigma_units"),
        num("u", "K", unit_from="u_units"),
        text("u_units"),
        num("uAB", "K", unit_from="uAB_units"),
        text("uAB_units"),
        num("volA", "Angstrom^3", unit_from="volA_units"),
        text("volA_units"),
        texts("aliases"),
        texts("assocScheme"),
    ),
    lift=("name",),
    lengths=False,
)
PCSAFT_PAIRS = ObjectSpec(
    "pcsaft_binary_pairs",
    keys=(index("pair_index", "position in the top-level array"),),
    scalars=(
        text("BibTeX"),
        text("CAS1"),
        text("CAS2"),
        text("Name1"),
        text("Name2"),
        num("kij"),
        num("kijT"),
    ),
    lengths=False,
)

SCHEMAS = merge_schemas(
    CUBIC_FLUIDS.schemas(),
    CUBIC_ALPHA0.schemas(),
    PCSAFT_FLUIDS.schemas(),
    PCSAFT_PAIRS.schemas(),
)

CUBIC_FILE = "dev/cubics/all_cubic_fluids.json"
PCSAFT_FILE = "dev/pcsaft/all_pcsaft_fluids.json"
PCSAFT_PAIR_FILE = "dev/pcsaft/mixture_binary_pairs_pcsaft.json"


def read_cubic_fluids(tree: Path, artifact: str, sink: Sink) -> None:
    for position, record in enumerate(load_list(tree, artifact)):
        body = Fields(record, locator(artifact, pointer(position)))
        alpha0 = body.take_list("alpha0") or []
        name = body.remaining().get("name")
        head = dict(body.remaining())
        sink.add_object(CUBIC_FLUIDS, artifact, pointer(position), {}, head)
        for term_index, term in enumerate(alpha0):
            sink.add_object(
                CUBIC_ALPHA0,
                artifact,
                pointer(position, "alpha0", term_index),
                {"name": name, "term_index": term_index},
                term,
            )


def read_pcsaft_fluids(tree: Path, artifact: str, sink: Sink) -> None:
    for position, record in enumerate(load_list(tree, artifact)):
        sink.add_object(PCSAFT_FLUIDS, artifact, pointer(position), {}, record)


def read_pcsaft_pairs(tree: Path, artifact: str, sink: Sink) -> None:
    for position, pair in enumerate(load_list(tree, artifact)):
        sink.add_object(PCSAFT_PAIRS, artifact, pointer(position), {"pair_index": position}, pair)
