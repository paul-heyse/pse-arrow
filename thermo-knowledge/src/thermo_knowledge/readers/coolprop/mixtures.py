# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Mixture files: binary interaction pairs (as written, in whatever orientation the source used),
departure functions and predefined mixtures."""

from __future__ import annotations

from pathlib import Path

from thermo_knowledge.readers.coolprop.common import (
    ObjectSpec,
    Sink,
    index,
    integer,
    load_list,
    merge_schemas,
    num,
    pointer,
    text,
    texts,
)

PAIR_INDEX = index("pair_index", "position in the top-level array")

BINARY_PAIRS = ObjectSpec(
    "binary_pairs",
    keys=(PAIR_INDEX,),
    scalars=(
        text("BibTeX"),
        text("CAS1"),
        text("CAS2"),
        text("Name1"),
        text("Name2"),
        num("betaT"),
        num("betaV"),
        num("gammaT"),
        num("gammaV"),
        num("F"),
        text("function"),
        num("xi"),
        num("zeta"),
    ),
    lengths=False,
)

DEPARTURE_FUNCTIONS = ObjectSpec(
    "departure_functions",
    keys=(
        index("function_index", "position in the top-level array"),
        text("Name", source_name="Name", nullable=False),
        text("type", source_name="type"),
    ),
    scalars=(
        text("BibTeX"),
        integer("Npower"),
        texts("aliases"),
    ),
    rows_table="departure_function_rows",
    arrays=tuple(num(name) for name in ("beta", "d", "epsilon", "eta", "gamma", "l", "n", "t")),
    lift=("Name", "type"),
)

PREDEFINED_MIXTURES = ObjectSpec(
    "predefined_mixtures",
    keys=(
        index("mixture_index", "position in the top-level array"),
        text("name", source_name="name"),
    ),
    scalars=(),
    rows_table="predefined_mixture_components",
    arrays=(text("fluids", source_name="fluids"), num("mole_fractions")),
    lift=("name",),
    row_index="component_index",
)

SCHEMAS = merge_schemas(
    BINARY_PAIRS.schemas(), DEPARTURE_FUNCTIONS.schemas(), PREDEFINED_MIXTURES.schemas()
)

BINARY_PAIR_FILES = ("dev/mixtures/mixture_binary_pairs.json", "dev/mixtures/old_BIP.json")
DEPARTURE_FILE = "dev/mixtures/mixture_departure_functions.json"
PREDEFINED_FILE = "dev/mixtures/predefined_mixtures.json"


def read_binary_pairs(tree: Path, artifact: str, sink: Sink) -> None:
    for position, pair in enumerate(load_list(tree, artifact)):
        sink.add_object(BINARY_PAIRS, artifact, pointer(position), {"pair_index": position}, pair)


def read_departure_functions(tree: Path, artifact: str, sink: Sink) -> None:
    for position, function in enumerate(load_list(tree, artifact)):
        sink.add_object(
            DEPARTURE_FUNCTIONS, artifact, pointer(position), {"function_index": position}, function
        )


def read_predefined_mixtures(tree: Path, artifact: str, sink: Sink) -> None:
    for position, mixture in enumerate(load_list(tree, artifact)):
        sink.add_object(
            PREDEFINED_MIXTURES, artifact, pointer(position), {"mixture_index": position}, mixture
        )
