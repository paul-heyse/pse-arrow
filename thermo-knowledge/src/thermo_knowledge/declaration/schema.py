# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The TOML shape of a declaration as typed `msgspec` structs (meta-model sections 1 to 4).

Every struct forbids unknown keys. `_children` names the fields that hold a table of child
constructs and `_lists` the fields that hold an array of tables; `decode` converts each child
on its own so a refusal names the exact construct.
"""

from __future__ import annotations

import re
from typing import ClassVar, Literal

import msgspec

from thermo_knowledge.declaration.diagnostics import Code, Diagnostic


class Construct(msgspec.Struct, frozen=True, kw_only=True, forbid_unknown_fields=True):
    """Base of every declared construct."""

    _children: ClassVar[dict[str, type[Construct]]] = {}
    _lists: ClassVar[dict[str, type[Construct]]] = {}


class Marked(Construct, frozen=True, kw_only=True, forbid_unknown_fields=True):
    """A construct with the `traces` and `pse` marks of section 1."""

    traces: tuple[str, ...] = ()
    pse: str | None = None


class FacetDecl(Marked, frozen=True, kw_only=True, forbid_unknown_fields=True):
    """A facet an enum declares: a property some of its members have."""

    doc: str


class MemberDecl(Marked, frozen=True, kw_only=True, forbid_unknown_fields=True):
    doc: str
    facets: tuple[str, ...] = ()


class EnumDecl(Marked, frozen=True, kw_only=True, forbid_unknown_fields=True):
    _children: ClassVar[dict[str, type[Construct]]] = {
        "members": MemberDecl,
        "facets": FacetDecl,
    }
    doc: str
    members: dict[str, MemberDecl] = {}
    facets: dict[str, FacetDecl] = {}


class SchemeDecl(Marked, frozen=True, kw_only=True, forbid_unknown_fields=True):
    doc: str


class DependentDecl(Construct, frozen=True, kw_only=True, forbid_unknown_fields=True):
    """A dimension derived from a reaction: the quantity type's own `unit` is the rate per volume
    of a homogeneous reaction of order zero, and `surface_unit` the rate per area; `concentration`
    and `surface_concentration` are the units of a participant's concentration in a bulk phase and
    on a surface. `on` is the kind of the subject the dimension follows."""

    on: str
    surface_unit: str
    concentration: str
    surface_concentration: str


class QuantityTypeDecl(Marked, frozen=True, kw_only=True, forbid_unknown_fields=True):
    doc: str
    unit: str
    scale: Literal["absolute", "difference", "ratio", "dimensionless", "count"]
    production: str | None = None
    dependent: DependentDecl | None = None


class AttributeDecl(Marked, frozen=True, kw_only=True, forbid_unknown_fields=True):
    type: str
    doc: str
    optional: bool = False
    unique: bool = False
    default: object = None
    unit_from: str | None = None


class PresentIffDecl(Construct, frozen=True, kw_only=True, forbid_unknown_fields=True):
    """`present_iff`: `column` is non-null exactly when the enum column `when` holds one of the
    members in `in`."""

    column: str
    when: str
    members: tuple[str, ...] = msgspec.field(name="in")


class WithinDecl(Construct, frozen=True, kw_only=True, forbid_unknown_fields=True):
    """`within`: `column` lies in the closed interval from `lower` to `upper`; either bound may
    be omitted."""

    column: str
    lower: int | float | None = None
    upper: int | float | None = None


class CheckDecl(Construct, frozen=True, kw_only=True, forbid_unknown_fields=True):
    nonempty: str | None = None
    positive: str | None = None
    nonnegative: str | None = None
    ordered: tuple[str, ...] | None = None
    ordered_same_reference: tuple[str, ...] | None = None
    one_of_present: tuple[str, ...] | None = None
    present_iff: PresentIffDecl | None = None
    within: WithinDecl | None = None


class RequireDecl(Marked, frozen=True, kw_only=True, forbid_unknown_fields=True):
    name: str
    doc: str
    enforced: Literal["ddl", "load", "verify"]
    check: CheckDecl | None = None


class UniqueDecl(Marked, frozen=True, kw_only=True, forbid_unknown_fields=True):
    attributes: tuple[str, ...]
    name: str | None = None
    doc: str | None = None


class KindDecl(Marked, frozen=True, kw_only=True, forbid_unknown_fields=True):
    _children: ClassVar[dict[str, type[Construct]]] = {"attributes": AttributeDecl}
    _lists: ClassVar[dict[str, type[Construct]]] = {
        "requires": RequireDecl,
        "unique": UniqueDecl,
    }
    doc: str
    extends: str | None = None
    abstract: bool = False
    identity: tuple[str, ...] = ()
    provenance: str | None = None
    attributes: dict[str, AttributeDecl] = {}
    requires: tuple[RequireDecl, ...] = ()
    unique: tuple[UniqueDecl, ...] = ()


class KeyDecl(Marked, frozen=True, kw_only=True, forbid_unknown_fields=True):
    type: str
    doc: str


class ValueDecl(Marked, frozen=True, kw_only=True, forbid_unknown_fields=True):
    type: str
    doc: str
    optional: bool = False
    default: object = None
    unit_from: str | None = None


class TranspositionDecl(Construct, frozen=True, kw_only=True, forbid_unknown_fields=True):
    rule: Literal["ordered", "symmetric", "parity", "reciprocal", "linear", "permutation_group"]
    roles: tuple[str, ...]
    diagonal: Literal["forbidden", "allowed"] = "forbidden"
    by: str | None = None
    slots: tuple[str, ...] = ()
    permutations: tuple[tuple[str, ...], ...] = ()
    matrix: tuple[tuple[int | float, ...], ...] = ()


class AbsenceDefault(Construct, frozen=True, kw_only=True, forbid_unknown_fields=True):
    default: object


class RelationDecl(Marked, frozen=True, kw_only=True, forbid_unknown_fields=True):
    _children: ClassVar[dict[str, type[Construct]]] = {"keys": KeyDecl, "columns": ValueDecl}
    _lists: ClassVar[dict[str, type[Construct]]] = {"requires": RequireDecl}
    doc: str
    provenance: str | None = None
    absence: Literal["required", "optional"] | AbsenceDefault | None = None
    keys: dict[str, KeyDecl] = {}
    value: ValueDecl | None = None
    columns: dict[str, ValueDecl] = {}
    transposition: TranspositionDecl | None = None
    requires: tuple[RequireDecl, ...] = ()


class ArgumentDecl(Marked, frozen=True, kw_only=True, forbid_unknown_fields=True):
    type: str
    doc: str
    over: tuple[str, ...] = ()
    basis: str | None = None
    observable: str | None = None


class RoleDecl(Marked, frozen=True, kw_only=True, forbid_unknown_fields=True):
    """A single subject of a contract, or (as a set) the kind of the elements it ranges over."""

    type: str
    doc: str


class OutputDecl(Marked, frozen=True, kw_only=True, forbid_unknown_fields=True):
    type: str
    doc: str
    observable: str | None = None
    observable_from_set: bool = False
    extra_order: int = 0


class ContractDecl(Marked, frozen=True, kw_only=True, forbid_unknown_fields=True):
    _children: ClassVar[dict[str, type[Construct]]] = {
        "roles": RoleDecl,
        "sets": RoleDecl,
        "arguments": ArgumentDecl,
        "outputs": OutputDecl,
    }
    doc: str
    roles: dict[str, RoleDecl] = {}
    sets: dict[str, RoleDecl] = {}
    arguments: dict[str, ArgumentDecl] = {}
    outputs: dict[str, OutputDecl] = {}


class SlotDecl(Marked, frozen=True, kw_only=True, forbid_unknown_fields=True):
    doc: str
    type: str | None = None
    accepts: str | None = None
    references: str | None = None
    presence: Literal["required", "stateful"] = "required"
    observable: str | None = None
    extra_order: int = 0


class IndexDecl(Marked, frozen=True, kw_only=True, forbid_unknown_fields=True):
    type: str
    doc: str
    min: int | None = None


class IntervalDecl(Construct, frozen=True, kw_only=True, forbid_unknown_fields=True):
    lower: str
    upper: str


class FamilyDecl(Marked, frozen=True, kw_only=True, forbid_unknown_fields=True):
    _children: ClassVar[dict[str, type[Construct]]] = {"index": IndexDecl, "slots": SlotDecl}
    doc: str
    index: dict[str, IndexDecl] = {}
    interval: IntervalDecl | None = None
    slots: dict[str, SlotDecl] = {}


class SlotGroupDecl(Marked, frozen=True, kw_only=True, forbid_unknown_fields=True):
    _children: ClassVar[dict[str, type[Construct]]] = {
        "subject": KeyDecl,
        "slots": SlotDecl,
        "families": FamilyDecl,
    }
    doc: str
    subject: dict[str, KeyDecl] = {}
    bind: dict[str, str] = {}
    slots: dict[str, SlotDecl] = {}
    families: dict[str, FamilyDecl] = {}
    transposition: TranspositionDecl | None = None


class SubformDecl(Marked, frozen=True, kw_only=True, forbid_unknown_fields=True):
    doc: str
    accepts: str
    multiplicity: Literal["one", "optional", "many"]
    per: Literal["model", "subject"]


class UnknownDecl(Marked, frozen=True, kw_only=True, forbid_unknown_fields=True):
    """An unknown of an implicit block: a quantity, optionally over sets, with optional bounds
    and start, each a number (in storage units) or an expression in text."""

    type: str
    over: tuple[str, ...] = ()
    lower: float | str | None = None
    upper: float | str | None = None
    start: float | str | None = None


class ImplicitDecl(Marked, frozen=True, kw_only=True, forbid_unknown_fields=True):
    """An implicit block (expressions.md section 3): quantities defined by equations. `select`
    is `unique`, `smallest`, `largest` or `by: <expression>`; the text is read by the checker."""

    _children: ClassVar[dict[str, type[Construct]]] = {"unknowns": UnknownDecl}
    doc: str
    unknowns: dict[str, UnknownDecl] = {}
    residuals: tuple[str, ...] = ()
    select: str = "unique"


class FormDecl(Marked, frozen=True, kw_only=True, forbid_unknown_fields=True):
    _children: ClassVar[dict[str, type[Construct]]] = {
        "slot_groups": SlotGroupDecl,
        "subforms": SubformDecl,
        "implicit": ImplicitDecl,
    }
    doc: str
    implements: str
    completeness: Literal["fully_declared", "structure_declared_equation_external", "opaque_bundle"]
    status: str
    citations: tuple[str, ...] = ()
    slot_groups: dict[str, SlotGroupDecl] = {}
    subforms: dict[str, SubformDecl] = {}
    let: dict[str, str] = {}
    outputs: dict[str, str] = {}
    output_observables: dict[str, str] = {}
    conventions: tuple[str, ...] = ()
    component_conventions: dict[str, str] = {}
    implicit: dict[str, ImplicitDecl] = {}


class EntityDecl(Marked, frozen=True, kw_only=True, forbid_unknown_fields=True):
    """The `doc` and marks of an entity; every other key is an attribute value."""

    doc: str


class FrameworkDecl(Construct, frozen=True, kw_only=True, forbid_unknown_fields=True):
    parameter_set: str | None = None
    parameterization: str | None = None
    tabulated_function: str | None = None
    slot_uncertainty: str | None = None
    observable: str | None = None
    composition_basis: str | None = None
    convention_set: str | None = None


class ManifestDecl(Construct, frozen=True, kw_only=True, forbid_unknown_fields=True):
    doc: str
    framework: FrameworkDecl = FrameworkDecl()


class ModuleHeader(Construct, frozen=True, kw_only=True, forbid_unknown_fields=True):
    module: str
    doc: str
    schema: str | None = None
    uses: tuple[str, ...] = ()


# Sections of a module document and the construct each section holds.
SECTIONS: dict[str, type[Construct]] = {
    "enums": EnumDecl,
    "identifier_schemes": SchemeDecl,
    "quantity_types": QuantityTypeDecl,
    "kinds": KindDecl,
    "relations": RelationDecl,
    "contracts": ContractDecl,
    "forms": FormDecl,
}
HEADER_KEYS = frozenset({"module", "doc", "schema", "uses"})
ENTITY_MARKS = frozenset({"doc", "traces", "pse"})

_UNKNOWN = re.compile(r"^Object contains unknown field `(?P<name>[^`]+)`")
_MISSING = re.compile(r"^Object missing required field `(?P<name>[^`]+)`")
_AT = re.compile(r"^(?P<text>.*) - at `\$(?P<path>[^`]*)`$", re.DOTALL)
_PSE_MARK = re.compile(r"^gap:[a-z][a-z0-9_]*$")


def diagnostic_from(error: msgspec.ValidationError, document: str, construct: str) -> Diagnostic:
    """Translate a `msgspec` refusal of one construct into a structured diagnostic."""
    text = str(error)
    path = ""
    located = _AT.match(text)
    if located:
        text, path = located["text"], located["path"]
    where = construct + path.replace("[...]", "")
    if match := _UNKNOWN.match(text):
        code = Code.UNKNOWN_CHECK if where.endswith(".check") else Code.UNKNOWN_KEY
        return Diagnostic(
            document=document,
            construct=f"{where}.{match['name']}",
            code=code,
            message=f"unknown key `{match['name']}`",
        )
    if match := _MISSING.match(text):
        name = match["name"]
        code = {"doc": Code.MISSING_DOC, "enforced": Code.MISSING_ENFORCEMENT}.get(
            name, Code.MISSING_FIELD
        )
        return Diagnostic(
            document=document, construct=where, code=code, message=f"missing required key `{name}`"
        )
    return Diagnostic(document=document, construct=where, code=Code.INVALID_VALUE, message=text)


def decode[T: Construct](
    cls: type[T],
    data: object,
    construct: str,
    document: str,
    diagnostics: list[Diagnostic],
) -> T | None:
    """Convert one TOML table into `cls`, converting each child construct on its own.

    Returns `None` and appends diagnostics when anything in it is refused.
    """
    if not isinstance(data, dict):
        diagnostics.append(
            Diagnostic(
                document=document,
                construct=construct,
                code=Code.INVALID_VALUE,
                message=f"expected a table, found {type(data).__name__}",
            )
        )
        return None
    failed = False
    replacements: dict[str, object] = {}
    rest = dict(data)
    for field, child_cls in cls._children.items():
        if field not in rest:
            continue
        table = rest.pop(field)
        if not isinstance(table, dict):
            diagnostics.append(
                Diagnostic(
                    document=document,
                    construct=f"{construct}.{field}",
                    code=Code.INVALID_VALUE,
                    message=f"expected a table, found {type(table).__name__}",
                )
            )
            failed = True
            continue
        children: dict[str, Construct] = {}
        for name, body in table.items():
            child = decode(child_cls, body, f"{construct}.{field}.{name}", document, diagnostics)
            if child is None:
                failed = True
            else:
                children[name] = child
        replacements[field] = children
    for field, child_cls in cls._lists.items():
        if field not in rest:
            continue
        rows = rest.pop(field)
        if not isinstance(rows, list):
            diagnostics.append(
                Diagnostic(
                    document=document,
                    construct=f"{construct}.{field}",
                    code=Code.INVALID_VALUE,
                    message="expected an array of tables",
                )
            )
            failed = True
            continue
        items: list[Construct] = []
        for index, body in enumerate(rows):
            item = decode(child_cls, body, f"{construct}.{field}[{index}]", document, diagnostics)
            if item is None:
                failed = True
            else:
                items.append(item)
        replacements[field] = tuple(items)
    try:
        built = msgspec.convert(rest, cls, strict=True)
    except msgspec.ValidationError as error:
        diagnostics.append(diagnostic_from(error, document, construct))
        return None
    if failed:
        return None
    if isinstance(built, Marked) and built.pse is not None and not _PSE_MARK.match(built.pse):
        diagnostics.append(
            Diagnostic(
                document=document,
                construct=construct,
                code=Code.BAD_PSE_MARK,
                message=f"`pse` must read `gap:<short name>`, found {built.pse!r}",
            )
        )
        return None
    return msgspec.structs.replace(built, **replacements) if replacements else built
