# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The loaded declaration: resolved, validated and ready to project.

Every collection is ordered by name so that the projection does not depend on which file
declares what. Attribute, member and slot order is the declared order.
"""

from __future__ import annotations

import uuid
from dataclasses import dataclass, field
from datetime import date, datetime
from typing import Literal

from thermo_knowledge import identity
from thermo_knowledge.declaration.types import TypeRef, UnitInfo

type Scalar = str | int | float | bool | date | datetime | bytes
type EntityValue = (
    str
    | int
    | float
    | bool
    | date
    | datetime
    | bytes
    | uuid.UUID
    | tuple[uuid.UUID, ...]
    | tuple[str | int | float | bool | date | datetime | bytes, ...]
    | tuple[float, float]
)

FRAMEWORK_ROLES = ("parameter_set", "parameterization", "tabulated_function", "slot_uncertainty")
"""Roles a declaration with forms must bind."""
DIMENSIONLESS = "dimensionless"
"""The `unit_from` of a `Real` whose unit is always the dimensionless one."""
OBSERVABLE_ROLE = "observable"
"""Optional role: the kind whose declared entities are the observables a contract output or a
slot may denote."""
COMPOSITION_BASIS_ROLE = "composition_basis"
"""Optional role: the kind whose declared entities are the composition bases a contract argument
may name, and an expression may assert for a vector it builds."""
OPTIONAL_ROLES = (OBSERVABLE_ROLE, COMPOSITION_BASIS_ROLE)
"""The roles a declaration may leave unbound; using what they enable without binding one is
refused."""
SCHEMAS = ("tk", "prov", "ev", "qual")
"""The schemas a module may name; slot-group tables go to `param`, reified rows to `meta`."""
PARAM_SCHEMA = "param"
META_SCHEMA = "meta"


@dataclass(frozen=True, kw_only=True)
class Module:
    name: str
    doc: str
    schema: str | None
    uses: tuple[str, ...]
    document: str


@dataclass(frozen=True, kw_only=True)
class Field:
    """An attribute, key, value column, slot, subject role, contract argument or index."""

    name: str
    type: TypeRef
    doc: str
    construct: str
    optional: bool = False
    unique: bool = False
    default: Scalar | None = None
    unit_from: str | None = None
    presence: Literal["required", "stateful"] = "required"
    shape: str | None = None
    accepts: str | None = None
    observable: str | None = None
    observable_from_set: bool = False
    minimum: int | None = None
    over: tuple[str, ...] = ()
    basis: str | None = None
    traces: tuple[str, ...] = ()
    pse: str | None = None


@dataclass(frozen=True, kw_only=True)
class Provenance:
    mode: Literal["own", "inherit", "declaration", "none"]
    attribute: str | None = None


@dataclass(frozen=True, kw_only=True)
class Requirement:
    """An invariant of a kind or a relation and where it is enforced.

    For a `ddl` requirement, `rule` is the declared check and `attributes` the columns it is
    about (the keys and value columns of a relation): one for `nonempty`, `positive`,
    `nonnegative` and `within`, two for `ordered`, two or more for `one_of_present`, and for
    `present_iff` the governed column, then the enum column it depends on. `members` are the
    enum members of `present_iff`; `lower` and `upper` the bounds of `within` (`None` when
    omitted)."""

    name: str
    doc: str
    enforced: Literal["ddl", "load", "verify"]
    rule: str | None
    attributes: tuple[str, ...]
    construct: str
    members: tuple[str, ...] = ()
    lower: float | None = None
    upper: float | None = None
    traces: tuple[str, ...] = ()
    pse: str | None = None


@dataclass(frozen=True, kw_only=True)
class Uniqueness:
    name: str
    attributes: tuple[str, ...]
    construct: str


@dataclass(frozen=True, kw_only=True)
class Transposition:
    rule: Literal["ordered", "symmetric", "parity", "reciprocal", "linear", "permutation_group"]
    roles: tuple[str, ...]
    diagonal: Literal["forbidden", "allowed"]
    by: str | None = None
    slots: tuple[str, ...] = ()
    permutations: tuple[tuple[str, ...], ...] = ()
    matrix: tuple[tuple[int | float, ...], ...] = ()
    """For `linear`: the matrix, row by row, as declared; it multiplies the vector of the values of
    `slots` when the roles are swapped."""


@dataclass(frozen=True, kw_only=True)
class Facet:
    """A property an enum declares that some of its members have."""

    name: str
    doc: str
    construct: str
    traces: tuple[str, ...] = ()
    pse: str | None = None


@dataclass(frozen=True, kw_only=True)
class Member:
    name: str
    doc: str
    construct: str
    facets: tuple[str, ...] = ()
    traces: tuple[str, ...] = ()
    pse: str | None = None


@dataclass(frozen=True, kw_only=True)
class Enum:
    name: str
    module: str
    doc: str
    members: tuple[Member, ...]
    construct: str
    facets: tuple[Facet, ...] = ()
    traces: tuple[str, ...] = ()
    pse: str | None = None


@dataclass(frozen=True, kw_only=True)
class Scheme:
    name: str
    module: str
    doc: str
    construct: str
    traces: tuple[str, ...] = ()
    pse: str | None = None


@dataclass(frozen=True, kw_only=True)
class QuantityType:
    name: str
    module: str
    doc: str
    unit: str
    scale: str
    production: str | None
    construct: str
    traces: tuple[str, ...] = ()
    pse: str | None = None


@dataclass(frozen=True, kw_only=True)
class Kind:
    name: str
    module: str
    schema: str
    doc: str
    extends: str | None
    root: str
    abstract: bool
    identity: tuple[str, ...]
    provenance: Provenance
    attributes: tuple[Field, ...]
    requires: tuple[Requirement, ...]
    uniques: tuple[Uniqueness, ...]
    construct: str
    origin: Literal["declared", "slot_group"] = "declared"
    slot_group: str | None = None
    traces: tuple[str, ...] = ()
    pse: str | None = None


@dataclass(frozen=True, kw_only=True)
class Relation:
    name: str
    module: str
    schema: str
    doc: str
    provenance: Provenance
    absence: Literal["required", "optional", "default"]
    absence_default: Scalar | None
    keys: tuple[Field, ...]
    values: tuple[Field, ...]
    single_value: bool
    transposition: Transposition | None
    construct: str
    requires: tuple[Requirement, ...] = ()
    traces: tuple[str, ...] = ()
    pse: str | None = None


@dataclass(frozen=True, kw_only=True)
class Contract:
    name: str
    module: str
    doc: str
    arguments: tuple[Field, ...]
    outputs: tuple[Field, ...]
    construct: str
    roles: tuple[Field, ...] = ()  # single subjects: `type` is a kind
    sets: tuple[Field, ...] = ()  # index sets of entities: `type` is the element kind
    traces: tuple[str, ...] = ()
    pse: str | None = None


@dataclass(frozen=True, kw_only=True)
class Family:
    name: str
    id: str  # `<form>__<group>__<family>`: the table name
    qualified: str  # `<form>.<group>.<family>`
    slot_group: str  # the owning group's `qualified`
    doc: str
    indices: tuple[Field, ...]
    slots: tuple[Field, ...]
    interval: tuple[str, str] | None
    construct: str
    traces: tuple[str, ...] = ()
    pse: str | None = None


@dataclass(frozen=True, kw_only=True)
class SubjectBinding:
    """What one subject role of a slot group is bound to in the form's contract: a contract
    role, a contract set, or (`kind` and `target` are `None`) nothing, which only a
    `catalogued` form may leave."""

    role: str
    kind: Literal["role", "set"] | None
    target: str | None


@dataclass(frozen=True, kw_only=True)
class SlotGroup:
    name: str
    id: str  # `<form>__<group>`: the table and expanded kind name
    qualified: str  # `<form>.<group>`
    form: str
    doc: str
    subjects: tuple[Field, ...]
    slots: tuple[Field, ...]
    families: tuple[Family, ...]
    transposition: Transposition | None
    construct: str
    bindings: tuple[SubjectBinding, ...] = ()  # one per subject, in subject order
    traces: tuple[str, ...] = ()
    pse: str | None = None


@dataclass(frozen=True, kw_only=True)
class SubformSlot:
    name: str
    qualified: str  # `<form>.<subform>`
    form: str
    doc: str
    accepts: str
    multiplicity: str
    per: str
    construct: str
    traces: tuple[str, ...] = ()
    pse: str | None = None


@dataclass(frozen=True, kw_only=True)
class ExpressionDef:
    """A local or an output of a form: its name and the expression text as declared."""

    name: str
    text: str
    construct: str


@dataclass(frozen=True, kw_only=True)
class Bound:
    """A bound or start of an unknown: a number in storage units, or expression text."""

    number: float | None = None
    text: str | None = None


@dataclass(frozen=True, kw_only=True)
class Unknown:
    """An unknown of an implicit block: a quantity over zero or more sets of the contract."""

    name: str
    type: TypeRef
    over: tuple[str, ...]
    lower: Bound | None
    upper: Bound | None
    start: Bound | None
    construct: str
    traces: tuple[str, ...] = ()
    pse: str | None = None


@dataclass(frozen=True, kw_only=True)
class ImplicitBlock:
    """An implicit block of a form: unknowns and the residuals that determine them.

    `select` is `unique`, `smallest`, `largest` or `by`, and `select_by` is the expression
    of `by` (else `None`). `residuals` are expression texts, each an expression or an expression
    followed by `for` clauses."""

    name: str
    form: str
    doc: str
    unknowns: tuple[Unknown, ...]
    residuals: tuple[ExpressionDef, ...]
    select: str
    select_by: str | None
    construct: str
    traces: tuple[str, ...] = ()
    pse: str | None = None


@dataclass(frozen=True, kw_only=True)
class OutputObservable:
    """The slot that supplies the observable of a contract output declared
    `observable_from_set`: a required slot of a slot group of the form, whose value is an
    observable."""

    output: str
    slot_group: str  # the group's `qualified`
    slot: str
    qualified: str  # `<form>.<group>.<slot>`
    construct: str


@dataclass(frozen=True, kw_only=True)
class Form:
    name: str
    module: str
    doc: str
    implements: str
    completeness: str
    status: str
    citations: tuple[str, ...]
    slot_groups: tuple[SlotGroup, ...]
    subforms: tuple[SubformSlot, ...]
    construct: str
    locals: tuple[ExpressionDef, ...] = ()  # in declared (evaluation) order
    outputs: tuple[ExpressionDef, ...] = ()
    implicit: tuple[ImplicitBlock, ...] = ()
    output_observables: tuple[OutputObservable, ...] = ()
    traces: tuple[str, ...] = ()
    pse: str | None = None


@dataclass(frozen=True, kw_only=True)
class Entity:
    kind: str
    name: str
    doc: str
    id: uuid.UUID
    values: dict[str, EntityValue]
    construct: str
    traces: tuple[str, ...] = ()
    pse: str | None = None


@dataclass(frozen=True, kw_only=True)
class Declaration:
    """A complete, validated declaration."""

    manifest_doc: str
    framework: dict[str, str]
    modules: dict[str, Module]
    enums: dict[str, Enum]
    schemes: dict[str, Scheme]
    quantity_types: dict[str, QuantityType]
    units: dict[str, UnitInfo]
    expressions: dict[str, str]  # normalised quantity expression -> unit key
    kinds: dict[str, Kind]  # declared kinds and expanded slot groups
    relations: dict[str, Relation]
    contracts: dict[str, Contract]
    forms: dict[str, Form]
    entities: tuple[Entity, ...] = field(default=())

    @property
    def slot_groups(self) -> tuple[SlotGroup, ...]:
        return tuple(group for form in self.forms.values() for group in form.slot_groups)

    @property
    def families(self) -> tuple[Family, ...]:
        return tuple(family for group in self.slot_groups for family in group.families)

    def chain(self, kind: str) -> tuple[Kind, ...]:
        """The refinement chain of `kind`, root first, `kind` last."""
        chain: list[Kind] = []
        current: str | None = kind
        while current is not None:
            item = self.kinds[current]
            chain.append(item)
            current = item.extends
        return tuple(reversed(chain))

    def attributes_of(self, kind: str) -> tuple[Field, ...]:
        """Every attribute of `kind`, inherited ones first."""
        return tuple(attribute for item in self.chain(kind) for attribute in item.attributes)

    def is_a(self, kind: str, ancestor: str) -> bool:
        """Whether `kind` is `ancestor` or a refinement of it."""
        return any(item.name == ancestor for item in self.chain(kind))

    def role_entity(self, role: str, name: str) -> Entity | None:
        """The declared entity named `name` of the kind bound to framework `role` (or of one of
        its refinements), or `None` when the role is unbound or there is no such entity."""
        kind = self.framework.get(role)
        if kind is None:
            return None
        for entity in self.entities:
            if entity.name == name and self.is_a(entity.kind, kind):
                return entity
        return None

    def observable_entity(self, name: str) -> Entity | None:
        """The declared entity named `name` of the `observable` framework kind."""
        return self.role_entity(OBSERVABLE_ROLE, name)

    def composition_basis_entity(self, name: str) -> Entity | None:
        """The declared entity named `name` of the `composition_basis` framework kind."""
        return self.role_entity(COMPOSITION_BASIS_ROLE, name)

    def enum_members_with(self, enum: str, facet: str) -> tuple[str, ...]:
        """The members of `enum` that carry `facet`, in declared order."""
        return tuple(member.name for member in self.enums[enum].members if facet in member.facets)

    def meta_ids(self) -> dict[str, dict[str, uuid.UUID]]:
        """Identifiers of the reified rows `Meta<construct>` refers to, by qualified name."""
        names: dict[str, list[str]] = {
            "quantity_type": list(self.quantity_types),
            "kind": list(self.kinds),
            "contract": list(self.contracts),
            "form": list(self.forms),
            "slot_group": [group.qualified for group in self.slot_groups],
            "slot": [],
            "family": [family.qualified for family in self.families],
            "subform_slot": [
                subform.qualified for form in self.forms.values() for subform in form.subforms
            ],
        }
        for group in self.slot_groups:
            names["slot"].extend(f"{group.qualified}.{slot.name}" for slot in group.slots)
            for family in group.families:
                names["slot"].extend(f"{family.qualified}.{slot.name}" for slot in family.slots)
        return {
            construct: {name: identity.meta_identifier(construct, name) for name in qualified}
            for construct, qualified in names.items()
        }
