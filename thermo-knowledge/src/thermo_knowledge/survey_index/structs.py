# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The survey record as `docs/survey.md` specifies it, and the vocabularies its keys use.

The structs are the schema: the loader validates a record key by key against their fields and
their resolved annotations, so a deviation is reported at the key that has it.
"""

from __future__ import annotations

import msgspec

NOT_RECORDED = "not recorded"
"""The value of a key the record predates (docs/survey.md section 2)."""

PROPOSED_SUFFIX = " (proposed)"

PRECISIONS = ("exact", "narrower", "broader", "close", "unmapped")
SHAPES = ("scalar", "enum", "reference", "function", "list", "table")
FIELD_ROLES = ("input", "computed", "unread", "metadata", "not established", NOT_RECORDED)
ORIGIN_KEYWORDS = (
    "authored",
    "transcribed",
    "generated",
    "computed",
    "other",
    "not stated",
    NOT_RECORDED,
)
READ_BY_SOURCE = ("all", "partly", "none", NOT_RECORDED)
CLASSES = (
    "explicit_correlation",
    "helmholtz_potential",
    "residual_or_excess_contribution",
    "group_contribution",
    "association",
    "standard_state_species",
    "reaction_property",
    "electrolyte_interaction",
    "sublattice_gibbs",
    "pseudo_component_characterisation",
    "implicit_constitutive",
    "spatial_functional",
    "adsorption",
    "transport",
    "regional_piecewise",
    "model_component",
    "wrapper",
    "regression",
    "kinetics",
    "other",
)
CLOSED_FORM_KEYWORDS = ("yes", "implicit", "procedural", "not stated")
OFFERED = ("yes", "partial", "no", NOT_RECORDED)
EVIDENCE = ("documentation", "source_inspected")


class Struct(msgspec.Struct, frozen=True, kw_only=True, forbid_unknown_fields=True):
    """Base of the record structs."""


class Document(Struct, kw_only=True):
    """One document of a payload made of documents (`[[payload]].documents`)."""

    slug: str
    identifier: str
    title: str
    pages: int
    bytes: int


class Payload(Struct, kw_only=True):
    paths: list[str]
    format: str
    files: int
    bytes: int
    records: str
    reader: str
    read_by_source: str
    notes: str
    documents: list[Document] = []


class Field(Struct, kw_only=True):
    name: str
    meaning: str
    unit: str
    shape: str
    role: str


class Construct(Struct, kw_only=True):
    name: str
    locator: str
    meaning: str
    subject: str
    fields: list[Field]
    origin: str
    conventions: str
    absence: str
    validity: str
    provenance: str
    count: str
    example: str
    candidate: str
    precision: str
    loss: str
    values: list[str] | None = None


class Compose(Struct, kw_only=True):
    slot: str
    accepts: str
    default: str


class Variant(Struct, kw_only=True):
    selector: str
    selects: str


class ModelFamily(Struct, kw_only=True):
    name: str
    locator: str
    # `class` is a Python keyword; the TOML key is `class`.
    class_: str = msgspec.field(name="class")
    inputs: str
    parameters: str
    composes: list[Compose]
    variants: list[Variant] | None = None
    equation_source: str
    closed_form: str
    data: str


class Convention(Struct, kw_only=True):
    name: str
    locator: str
    statement: str
    scope: str


class Selection(Struct, kw_only=True):
    name: str
    locator: str
    rule: str
    scope: str
    documented: str


class Discrepancy(Struct, kw_only=True):
    locator: str
    documented: str
    actual: str
    consequence: str


class Capability(Struct, kw_only=True):
    calculation: str
    offered: str
    scope: str
    locator: str
    evidence: str


class Question(Struct, kw_only=True):
    about: str
    question: str
    why_it_matters: str


class Survey(Struct, kw_only=True):
    """One survey file: a source and everything recorded about it."""

    source: str
    pin: str
    additional_pins: dict[str, str] = {}
    surveyed: str
    summary: str
    payload: list[Payload] = []
    construct: list[Construct] = []
    model_family: list[ModelFamily] = []
    convention: list[Convention] = []
    selection: list[Selection] = []
    discrepancy: list[Discrepancy] = []
    capability: list[Capability] = []
    question: list[Question] = []


TABLES: tuple[str, ...] = (
    "payload",
    "construct",
    "model_family",
    "convention",
    "selection",
    "discrepancy",
    "capability",
    "question",
)
"""The record tables of a survey file, in the order the specification lists them."""
