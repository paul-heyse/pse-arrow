# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Name resolution and validation of decoded modules (meta-model sections 1 to 6).

The resolver collects every refusal it can find and reports them together. A construct whose
own declaration is refused is left out of later steps so one mistake does not cascade.
"""

from __future__ import annotations

import dataclasses
import graphlib
import math
import re
import uuid
from collections.abc import Iterable, Mapping
from dataclasses import dataclass, field
from datetime import date, datetime
from typing import Literal

import pint
from pint.util import UnitsContainer

from thermo_knowledge import identity
from thermo_knowledge import transposition as transposition_module
from thermo_knowledge.declaration import model as m
from thermo_knowledge.declaration import schema as s
from thermo_knowledge.declaration.diagnostics import Code, Diagnostic
from thermo_knowledge.expression.units import describe, from_info, type_dimension
from thermo_knowledge.declaration.types import (
    META_CONSTRUCTS,
    PRIMITIVES,
    ElementKind,
    TypeRef,
    UnitError,
    UnitInfo,
    evaluate_expression,
    expression_info,
    is_bare_name,
    parse_storage_unit,
    split_type,
)

MANIFEST_DOCUMENT = "model/manifest.toml"

SNAKE = re.compile(r"^[a-z][a-z0-9]*(_[a-z0-9]+)*$")
CAMEL = re.compile(r"^[A-Z][A-Za-z0-9]*$")
SYMBOL = re.compile(r"^[A-Za-z][A-Za-z0-9_]*$")
HEX_HASH = re.compile(r"^[0-9a-fA-F]{64}$")
RESERVED_TYPE_NAMES = frozenset(
    {*PRIMITIVES, "Record", "Real", "SourceText", "Id", "Range", "Array", "Set", "Meta"}
)
RESERVED_COLUMNS = frozenset({"id"})

_SECTION_CATEGORY = {
    "enums": "enum",
    "identifier_schemes": "scheme",
    "quantity_types": "quantity_type",
    "kinds": "kind",
    "relations": "relation",
    "contracts": "contract",
    "forms": "form",
}


@dataclass
class ModuleDoc:
    """One decoded module document."""

    document: str
    header: s.ModuleHeader
    sections: dict[str, dict[str, s.Construct]]
    entities: dict[str, dict[str, tuple[s.EntityDecl, dict[str, object]]]]


@dataclass
class _Named:
    category: str
    module: str
    construct: str


@dataclass
class _RawEntity:
    kind: str
    name: str
    decl: s.EntityDecl
    values: dict[str, object]
    module: str
    construct: str


CHECK_FORMS = (
    "nonempty",
    "positive",
    "nonnegative",
    "ordered",
    "ordered_same_reference",
    "one_of_present",
    "present_iff",
    "within",
)
"""The forms of a `check` on a `ddl` invariant (meta-model section 3.4)."""


@dataclass(frozen=True)
class _Check:
    """A validated `ddl` check: its form, the columns it is about and its parameters."""

    rule: str | None
    attributes: tuple[str, ...]
    members: tuple[str, ...] = ()
    lower: float | None = None
    upper: float | None = None


def _contract_of(slot: m.Field) -> str | None:
    """The contract a slot's set value (nested or referenced) implements, else `None`."""
    return slot.accepts or slot.references


class _Refused(Exception):
    """Raised inside value coercion to carry one diagnostic out of a helper."""

    def __init__(self, code: str, message: str) -> None:
        super().__init__(message)
        self.code = code


@dataclass
class Resolver:
    manifest: s.ManifestDecl
    docs: list[ModuleDoc]
    diagnostics: list[Diagnostic] = field(default_factory=list)

    def __post_init__(self) -> None:
        self.doc_of: dict[str, str] = {}
        self.headers: dict[str, s.ModuleHeader] = {}
        self.visible: dict[str, frozenset[str]] = {}
        self.names: dict[str, _Named] = {}
        self.unit_objects: dict[str, pint.Unit] = {}
        self.unit_infos: dict[str, UnitInfo] = {}
        self.expressions: dict[str, str] = {}
        self.modules: dict[str, m.Module] = {}
        self.enums: dict[str, m.Enum] = {}
        self.schemes: dict[str, m.Scheme] = {}
        self.quantity_types: dict[str, m.QuantityType] = {}
        self.kinds: dict[str, m.Kind] = {}
        self.relations: dict[str, m.Relation] = {}
        self.contracts: dict[str, m.Contract] = {}
        self.forms: dict[str, m.Form] = {}
        self.framework: dict[str, str] = {}

    # -- diagnostics ---------------------------------------------------------------------

    def err(self, module: str | None, construct: str, code: Code, message: str) -> None:
        document = self.doc_of.get(module, MANIFEST_DOCUMENT) if module else MANIFEST_DOCUMENT
        self.diagnostics.append(
            Diagnostic(document=document, construct=construct, code=code, message=message)
        )

    # -- run -----------------------------------------------------------------------------

    def run(self) -> m.Declaration | None:
        self._modules()
        self._names()
        self._quantity_types()
        self._schemes_and_enums()
        self._kinds()
        self._relations()
        self._contracts()
        self._forms_and_framework()
        self._observables_need_a_role()
        self._bases_need_a_role()
        self._unit_sources()
        if self.diagnostics:
            return None
        declaration = self._declaration(entities=())
        resolved = self._entities(declaration)
        if resolved is None:
            return None
        self._observables_name_entities(resolved)
        self._bases_name_entities(resolved)
        return None if self.diagnostics else resolved

    # -- modules ---------------------------------------------------------------------------

    def _modules(self) -> None:
        graph: dict[str, list[str]] = {}
        for doc in self.docs:
            header = doc.header
            name = header.module
            if name in self.headers:
                self.diagnostics.append(
                    Diagnostic(
                        document=doc.document,
                        construct="module",
                        code=Code.DUPLICATE_MODULE,
                        message=f"module `{name}` is also declared in {self.doc_of[name]}",
                    )
                )
                continue
            self.doc_of[name] = doc.document
            self.headers[name] = header
            if not SNAKE.match(name):
                self.err(name, "module", Code.BAD_NAME, f"`{name}` is not lowercase snake_case")
            if header.schema is not None and header.schema not in m.SCHEMAS:
                self.err(
                    name,
                    "schema",
                    Code.BAD_SCHEMA,
                    f"schema `{header.schema}` is not one of {', '.join(m.SCHEMAS)}",
                )
            holds_tables = bool(doc.sections.get("kinds") or doc.sections.get("relations"))
            if holds_tables and header.schema is None:
                self.err(
                    name,
                    "schema",
                    Code.MISSING_SCHEMA,
                    "a module that declares kinds or relations names its `schema`",
                )
            graph[name] = list(header.uses)
        for name, header in self.headers.items():
            for used in header.uses:
                if used not in self.headers:
                    self.err(name, "uses", Code.UNKNOWN_NAME, f"module `{used}` is not declared")
        known = {name: [u for u in uses if u in self.headers] for name, uses in graph.items()}
        try:
            graphlib.TopologicalSorter(known).prepare()
        except graphlib.CycleError as error:
            cycle = error.args[1]
            self.err(
                cycle[0],
                "uses",
                Code.USES_CYCLE,
                "modules use each other in a cycle: " + " -> ".join(cycle),
            )
        for name in self.headers:
            seen: set[str] = set()
            pending = [name]
            while pending:
                current = pending.pop()
                if current in seen:
                    continue
                seen.add(current)
                pending.extend(known.get(current, []))
            self.visible[name] = frozenset(seen)
        for doc in self.docs:
            header = doc.header
            if self.doc_of.get(header.module) == doc.document:
                self.modules[header.module] = m.Module(
                    name=header.module,
                    doc=header.doc,
                    schema=header.schema,
                    uses=tuple(header.uses),
                    document=doc.document,
                )

    # -- names -----------------------------------------------------------------------------

    def _names(self) -> None:
        for doc in self.docs:
            module = doc.header.module
            if self.doc_of.get(module) != doc.document:
                continue
            for section, constructs in doc.sections.items():
                category = _SECTION_CATEGORY[section]
                for name in constructs:
                    construct = f"{section}.{name}"
                    if category == "quantity_type":
                        if not CAMEL.match(name) or name in RESERVED_TYPE_NAMES:
                            self.err(
                                module,
                                construct,
                                Code.BAD_NAME,
                                f"quantity type `{name}` must be CamelCase and not a built-in "
                                "type name",
                            )
                    elif not SNAKE.match(name):
                        self.err(
                            module,
                            construct,
                            Code.BAD_NAME,
                            f"`{name}` is not lowercase snake_case",
                        )
                    if name in self.names:
                        other = self.names[name]
                        self.err(
                            module,
                            construct,
                            Code.DUPLICATE_NAME,
                            f"`{name}` is already declared as a {other.category} at "
                            f"{self.doc_of[other.module]}: {other.construct}",
                        )
                        continue
                    self.names[name] = _Named(category, module, construct)

    def lookup(
        self,
        name: str,
        categories: tuple[str, ...],
        module: str,
        construct: str,
        what: str = "name",
    ) -> str | None:
        """The category of declared `name` if it is reachable from `module` and of one of
        `categories`; otherwise report why not and return `None`."""
        named = self.names.get(name)
        if named is None:
            self.err(module, construct, Code.UNKNOWN_NAME, f"{what} `{name}` is not declared")
            return None
        if named.module not in self.visible.get(module, frozenset({module})):
            self.err(
                module,
                construct,
                Code.UNREACHABLE_NAME,
                f"`{name}` is declared in module `{named.module}`, which `{module}` does not "
                "use (directly or transitively)",
            )
            return None
        if named.category not in categories:
            self.err(
                module,
                construct,
                Code.BAD_TYPE,
                f"`{name}` is a {named.category}, expected {' or '.join(categories)}",
            )
            return None
        return named.category

    # -- quantity types --------------------------------------------------------------------

    def _quantity_types(self) -> None:
        for doc in self.docs:
            module = doc.header.module
            for name, raw in doc.sections.get("quantity_types", {}).items():
                assert isinstance(raw, s.QuantityTypeDecl)
                construct = f"quantity_types.{name}"
                if self.names.get(name) is None or self.names[name].construct != construct:
                    continue
                if self.names[name].module != module:
                    continue
                try:
                    unit, info = parse_storage_unit(raw.unit)
                except UnitError as error:
                    self.err(module, f"{construct}.unit", Code(error.code), str(error))
                    continue
                dependent = None
                if raw.dependent is not None:
                    dependent = self._dependent(module, construct, info, raw.dependent)
                    if dependent is None:
                        continue
                self.unit_objects[name] = unit
                self.unit_infos[info.unit] = info
                self.quantity_types[name] = m.QuantityType(
                    name=name,
                    module=module,
                    doc=raw.doc,
                    unit=info.unit,
                    scale=raw.scale,
                    production=raw.production,
                    construct=construct,
                    dependent=dependent,
                    traces=raw.traces,
                    pse=raw.pse,
                )

    def _dependent(
        self, module: str, construct: str, base: UnitInfo, raw: s.DependentDecl
    ) -> m.Dependent | None:
        """The rule that derives the dimension of a dependent quantity type from its subject
        reaction: every unit parses as a coherent storage unit, and each per-area unit has the
        length dimension the per-volume unit lacks."""
        where = f"{construct}.dependent"
        if self.lookup(raw.on, ("kind",), module, f"{where}.on", "kind") is None:
            return None
        infos: dict[str, UnitInfo] = {}
        for label, text in (
            ("surface_unit", raw.surface_unit),
            ("concentration", raw.concentration),
            ("surface_concentration", raw.surface_concentration),
        ):
            try:
                infos[label] = parse_storage_unit(text)[1]
            except UnitError as error:
                self.err(module, f"{where}.{label}", Code(error.code), str(error))
                return None
        length = UnitsContainer({"[length]": 1})
        for surface, volume, label in (
            (infos["surface_unit"], base, "surface_unit"),
            (infos["surface_concentration"], infos["concentration"], "surface_concentration"),
        ):
            if from_info(surface) != from_info(volume) * length:
                self.err(
                    module,
                    f"{where}.{label}",
                    Code.BAD_DEPENDENT,
                    f"`{surface.unit}` is not the per-area form of `{volume.unit}`: a per-area "
                    "unit has one more length dimension than its per-volume unit",
                )
                return None
        for info in infos.values():
            self.unit_infos[info.unit] = info
        return m.Dependent(
            on=raw.on,
            surface_unit=infos["surface_unit"].unit,
            concentration=infos["concentration"].unit,
            surface_concentration=infos["surface_concentration"].unit,
        )

    def dependent_of(self, type_: TypeRef) -> m.Dependent | None:
        """The dependence of a scalar quantity type, `None` for any other type."""
        if type_.container != "scalar" or type_.element_kind != "quantity":
            return None
        quantity = self.quantity_types.get(type_.element)  # absent when its own declaration failed
        return None if quantity is None else quantity.dependent

    def _dependent_subject(
        self,
        module: str,
        construct: str,
        field_: m.Field,
        subjects: Iterable[m.Field],
        owner: str,
    ) -> None:
        """A field of a dependent quantity type has exactly one subject of the kind its dimension
        follows; a field of any other type declares no extra order."""
        if field_.type.element_kind == "quantity" and field_.type.element not in self.quantity_types:
            return  # the type's own declaration was refused and reported
        dependent = self.dependent_of(field_.type)
        if dependent is None:
            if field_.extra_order != 0:
                self.err(
                    module,
                    f"{construct}.extra_order",
                    Code.BAD_DEPENDENT,
                    f"`extra_order` applies to a dependent quantity type, and {field_.type.text} "
                    "is not one",
                )
            return
        if field_.extra_order < 0:
            self.err(
                module,
                f"{construct}.extra_order",
                Code.BAD_DEPENDENT,
                "the extra order is a whole number of concentration powers, not below zero",
            )
        found = [s_.name for s_ in subjects if self._is_a(s_.type.element, dependent.on)]
        if len(found) != 1:
            self.err(
                module,
                construct,
                Code.BAD_DEPENDENT,
                f"{field_.type.text} takes its dimension from the subject of kind `{dependent.on}` "
                f"of the {owner}, which must have exactly one such role (it has "
                f"{len(found)})",
            )

    def _schemes_and_enums(self) -> None:
        for doc in self.docs:
            module = doc.header.module
            for name, raw in doc.sections.get("identifier_schemes", {}).items():
                assert isinstance(raw, s.SchemeDecl)
                construct = f"identifier_schemes.{name}"
                if self._owns(name, construct, module):
                    self.schemes[name] = m.Scheme(
                        name=name,
                        module=module,
                        doc=raw.doc,
                        construct=construct,
                        traces=raw.traces,
                        pse=raw.pse,
                    )
            for name, raw in doc.sections.get("enums", {}).items():
                assert isinstance(raw, s.EnumDecl)
                construct = f"enums.{name}"
                if not self._owns(name, construct, module):
                    continue
                if not raw.members:
                    self.err(module, construct, Code.INVALID_VALUE, "an enum declares members")
                facets: list[m.Facet] = []
                for facet_name, facet in raw.facets.items():
                    where = f"{construct}.facets.{facet_name}"
                    if not SNAKE.match(facet_name):
                        self.err(
                            module, where, Code.BAD_NAME, f"`{facet_name}` is not lowercase snake_case"
                        )
                    facets.append(
                        m.Facet(
                            name=facet_name,
                            doc=facet.doc,
                            construct=where,
                            traces=facet.traces,
                            pse=facet.pse,
                        )
                    )
                members: list[m.Member] = []
                for member_name, member in raw.members.items():
                    where = f"{construct}.members.{member_name}"
                    if not SYMBOL.match(member_name):
                        self.err(module, where, Code.BAD_NAME, f"`{member_name}` is not a name")
                    for facet_name in member.facets:
                        if facet_name not in raw.facets:
                            self.err(
                                module,
                                f"{where}.facets",
                                Code.UNKNOWN_NAME,
                                f"`{facet_name}` is not a facet of enum `{name}` "
                                f"(facets: {', '.join(raw.facets) or 'it declares none'})",
                            )
                    if len(set(member.facets)) != len(member.facets):
                        self.err(
                            module, f"{where}.facets", Code.BAD_ATTRIBUTE, "a facet is named twice"
                        )
                    members.append(
                        m.Member(
                            name=member_name,
                            doc=member.doc,
                            construct=where,
                            facets=tuple(member.facets),
                            traces=member.traces,
                            pse=member.pse,
                        )
                    )
                self.enums[name] = m.Enum(
                    name=name,
                    module=module,
                    doc=raw.doc,
                    members=tuple(members),
                    construct=construct,
                    facets=tuple(facets),
                    traces=raw.traces,
                    pse=raw.pse,
                )

    def _owns(self, name: str, construct: str, module: str) -> bool:
        named = self.names.get(name)
        return named is not None and named.construct == construct and named.module == module

    # -- types -----------------------------------------------------------------------------

    def _unit_of(self, name: str, module: str, construct: str) -> pint.Unit | None:
        if self.lookup(name, ("quantity_type",), module, construct, "quantity type") is None:
            return None
        if self._refuses_dependent(name, module, construct, allowed=False):
            return None
        return self.unit_objects.get(name)

    def resolve_type(
        self, text: str, module: str, construct: str, *, allow_dependent: bool = False
    ) -> TypeRef | None:
        """Resolve a type expression (section 2); report and return `None` when refused.

        A dependent quantity type (section 3.3) is accepted only where `allow_dependent` says so:
        the type of a slot or of a contract output."""
        shape = split_type(text)
        inner = shape.inner
        if shape.head == "Id":
            if self.lookup(inner, ("scheme",), module, construct, "identifier scheme"):
                return TypeRef(
                    container="scalar",
                    element_kind="identifier",
                    element=inner,
                    text=f"Id<{inner}>",
                )
            return None
        if shape.head == "Range":
            if self.lookup(inner, ("quantity_type",), module, construct, "quantity type"):
                if self._refuses_dependent(inner, module, construct, allowed=False):
                    return None
                return TypeRef(
                    container="range",
                    element_kind="quantity",
                    element=inner,
                    text=f"Range<{inner}>",
                )
            return None
        if shape.head == "Set":
            if self.lookup(inner, ("kind",), module, construct, "kind"):
                return TypeRef(
                    container="set", element_kind="kind", element=inner, text=f"Set<{inner}>"
                )
            return None
        if shape.head == "Meta":
            if inner in META_CONSTRUCTS:
                return TypeRef(
                    container="scalar", element_kind="meta", element=inner, text=f"Meta<{inner}>"
                )
            self.err(
                module,
                construct,
                Code.BAD_TYPE,
                f"`Meta<{inner}>`: the construct is one of {', '.join(META_CONSTRUCTS)}",
            )
            return None
        if shape.head == "Array":
            element = self._scalar_type(inner, module, construct)
            if element is None:
                return None
            if element.container != "scalar" or element.element_kind not in (
                "primitive",
                "real",
                "quantity",
                "expression",
                "enum",
            ):
                self.err(
                    module,
                    construct,
                    Code.BAD_TYPE,
                    "`Array<T>` holds a primitive (`Real` included), quantity or enum type",
                )
                return None
            return TypeRef(
                container="array",
                element_kind=element.element_kind,
                element=element.element,
                text=f"Array<{element.text}>",
            )
        return self._scalar_type(inner, module, construct, allow_dependent=allow_dependent)

    def _refuses_dependent(self, name: str, module: str, construct: str, *, allowed: bool) -> bool:
        """Whether `name` is a dependent quantity type where none is allowed (reported)."""
        quantity = self.quantity_types.get(name)
        if allowed or quantity is None or quantity.dependent is None:
            return False
        self.err(
            module,
            construct,
            Code.BAD_DEPENDENT,
            f"`{name}` is a dependent quantity type: its dimension follows a subject, so only a "
            "slot or a contract output has it",
        )
        return True

    def _scalar_type(
        self, text: str, module: str, construct: str, *, allow_dependent: bool = False
    ) -> TypeRef | None:
        if text in PRIMITIVES:
            return TypeRef(container="scalar", element_kind="primitive", element=text, text=text)
        simple: dict[str, ElementKind] = {
            "Record": "record",
            "Real": "real",
            "SourceText": "source_text",
        }
        if text in simple:
            return TypeRef(
                container="scalar", element_kind=simple[text], element=text, text=text
            )
        if is_bare_name(text):
            category = self.lookup(
                text, ("quantity_type", "enum", "kind"), module, construct, "type"
            )
            if category is None:
                return None
            if category == "quantity_type" and self._refuses_dependent(
                text, module, construct, allowed=allow_dependent
            ):
                return None
            kinds: dict[str, ElementKind] = {
                "quantity_type": "quantity",
                "enum": "enum",
                "kind": "kind",
            }
            return TypeRef(container="scalar", element_kind=kinds[category], element=text, text=text)
        try:
            normalised, unit = evaluate_expression(
                text, lambda name: self._unit_of(name, module, construct)
            )
        except UnitError as error:
            if error.code != "unknown-name":  # unknown names are reported by the lookup
                self.err(module, construct, Code(error.code), str(error))
            return None
        info = expression_info(unit)
        self.unit_infos[info.unit] = info
        self.expressions[normalised] = info.unit
        return TypeRef(
            container="scalar", element_kind="expression", element=normalised, text=normalised
        )

    # -- values ----------------------------------------------------------------------------

    def scalar_value(self, type_: TypeRef, raw: object, *, absolute_check: bool = True) -> m.Scalar:
        """Check a scalar `raw` against a scalar type and return its normalised value."""
        kind = type_.element_kind
        if kind == "primitive":
            name = type_.element
            if name == "Boolean" and isinstance(raw, bool):
                return raw
            if name == "Integer" and isinstance(raw, int) and not isinstance(raw, bool):
                return raw
            if name == "Text" and isinstance(raw, str):
                return raw
            if name == "Date" and isinstance(raw, date) and not isinstance(raw, datetime):
                return raw
            if name == "Timestamp" and isinstance(raw, datetime) and raw.tzinfo is not None:
                return raw
            if name == "Hash" and isinstance(raw, str) and HEX_HASH.match(raw):
                return bytes.fromhex(raw)
            raise _Refused("bad-default", f"{raw!r} is not a {name}")
        if kind == "identifier":
            if isinstance(raw, str) and raw:
                return raw
            raise _Refused("bad-default", f"{raw!r} is not a non-empty identifier")
        if kind in ("quantity", "expression", "real"):
            if isinstance(raw, bool) or not isinstance(raw, (int, float)):
                raise _Refused("bad-default", f"{raw!r} is not a number")
            number = float(raw)
            if not math.isfinite(number):
                raise _Refused("bad-default", f"{raw!r} is not finite")
            if (
                absolute_check
                and kind == "quantity"
                and self.quantity_types[type_.element].scale == "absolute"
                and number < 0
            ):
                raise _Refused(
                    "bad-default", f"{raw!r} is negative, as an absolute quantity must not be"
                )
            return number
        if kind == "enum":
            members = {member.name for member in self.enums[type_.element].members}
            if isinstance(raw, str) and raw in members:
                return raw
            raise _Refused("bad-enum-member", f"{raw!r} is not a member of enum `{type_.element}`")
        if kind == "source_text" and isinstance(raw, str):
            return raw
        raise _Refused("bad-default", f"a value of type {type_.text} cannot be written here")

    # -- fields ----------------------------------------------------------------------------

    def symbol(self, module: str, construct: str, name: str, what: str) -> bool:
        if not SYMBOL.match(name):
            self.err(module, construct, Code.BAD_NAME, f"{what} `{name}` is not a name")
            return False
        if name in RESERVED_COLUMNS and not what.startswith("contract"):
            self.err(
                module, construct, Code.BAD_NAME, f"`{name}` is reserved for a generated column"
            )
            return False
        return True

    def _checked_default(
        self, module: str, construct: str, type_: TypeRef, raw: object
    ) -> tuple[bool, m.Scalar | None]:
        if raw is None:
            return True, None
        if type_.container != "scalar" or type_.element_kind in ("kind", "record", "meta"):
            self.err(
                module,
                f"{construct}.default",
                Code.BAD_DEFAULT,
                f"a default cannot be given for type {type_.text}",
            )
            return False, None
        try:
            value = self.scalar_value(type_, raw)
        except _Refused as refused:
            self.err(module, f"{construct}.default", Code(refused.code), str(refused))
            return False, None
        return True, value

    def _attribute(
        self,
        module: str,
        construct: str,
        name: str,
        raw: s.AttributeDecl,
    ) -> m.Field | None:
        if not self.symbol(module, construct, name, "attribute"):
            return None
        type_ = self.resolve_type(raw.type, module, f"{construct}.type")
        if type_ is None:
            return None
        ok = True
        if type_.container == "set" and (raw.optional or raw.unique):
            self.err(
                module,
                construct,
                Code.BAD_ATTRIBUTE,
                "a `Set<K>` attribute is a child table: it is neither optional nor unique",
            )
            ok = False
        valid, default = self._checked_default(module, construct, type_, raw.default)
        if not (ok and valid):
            return None
        return m.Field(
            name=name,
            type=type_,
            doc=raw.doc,
            construct=construct,
            optional=raw.optional,
            unique=raw.unique,
            default=default,
            unit_from=raw.unit_from,
            traces=raw.traces,
            pse=raw.pse,
        )

    # -- provenance ------------------------------------------------------------------------

    def _provenance(self, module: str, construct: str, text: str | None) -> m.Provenance | None:
        if text is None:
            self.err(
                module,
                f"{construct}.provenance",
                Code.BAD_PROVENANCE,
                "declare `provenance` as own, inherit:<attribute>, declaration or none",
            )
            return None
        if text in ("own", "declaration", "none"):
            return m.Provenance(mode=text)
        head, _, attribute = text.partition(":")
        if head == "inherit" and SYMBOL.match(attribute):
            return m.Provenance(mode="inherit", attribute=attribute)
        self.err(
            module,
            f"{construct}.provenance",
            Code.BAD_PROVENANCE,
            f"`{text}` is not own, inherit:<attribute>, declaration or none",
        )
        return None

    def _check_inherit(
        self,
        module: str,
        construct: str,
        provenance: m.Provenance,
        fields: Mapping[str, m.Field],
    ) -> bool:
        if provenance.mode != "inherit":
            return True
        assert provenance.attribute is not None
        target = fields.get(provenance.attribute)
        if (
            target is None
            or target.optional
            or target.type.container != "scalar"
            or target.type.element_kind not in ("kind", "record")
        ):
            self.err(
                module,
                f"{construct}.provenance",
                Code.BAD_PROVENANCE,
                f"`inherit:{provenance.attribute}` needs a required reference-typed "
                f"{'attribute' if construct.startswith('kinds') else 'key'} of that name",
            )
            return False
        return True

    # -- kinds -----------------------------------------------------------------------------

    def _kinds(self) -> None:
        decls: dict[str, tuple[str, s.KindDecl]] = {}
        for doc in self.docs:
            module = doc.header.module
            for name, raw in doc.sections.get("kinds", {}).items():
                assert isinstance(raw, s.KindDecl)
                if self._owns(name, f"kinds.{name}", module):
                    decls[name] = (module, raw)
        parents: dict[str, str | None] = {}
        for name, (module, raw) in decls.items():
            parent = None
            if raw.extends is not None:
                if self.lookup(raw.extends, ("kind",), module, f"kinds.{name}.extends", "kind"):
                    parent = raw.extends
                else:
                    parents[name] = None
                    continue
            parents[name] = parent
        # Cycles in `extends`.
        in_cycle: set[str] = set()
        for name in parents:
            seen = [name]
            current = parents.get(name)
            while current is not None:
                if current in seen:
                    cycle = seen[seen.index(current) :]
                    in_cycle.update(cycle)
                    break
                seen.append(current)
                current = parents.get(current)
        for name in sorted(in_cycle):
            module, _ = decls[name]
            self.err(
                module,
                f"kinds.{name}.extends",
                Code.EXTENDS_CYCLE,
                f"`{name}` is part of a cycle of refinements",
            )
        unresolved = {
            name
            for name, (_, raw) in decls.items()
            if name in in_cycle or (raw.extends is not None and parents.get(name) is None)
        }

        def depth(name: str) -> int:
            count = 0
            current = parents.get(name)
            while current is not None:
                count += 1
                current = parents.get(current)
            return count

        ordered = sorted(
            (name for name in decls if name not in unresolved),
            key=lambda name: (depth(name), name),
        )
        failed: set[str] = set(unresolved)
        for name in ordered:
            module, raw = decls[name]
            parent = parents[name]
            if parent is not None and parent in failed:
                failed.add(name)
                continue
            built = self._kind(name, module, raw, parent)
            if built is None:
                failed.add(name)
            else:
                self.kinds[name] = built

    def _kind(self, name: str, module: str, raw: s.KindDecl, parent: str | None) -> m.Kind | None:
        construct = f"kinds.{name}"
        ok = True
        inherited: dict[str, m.Field] = {}
        root = name
        root_provenance: m.Provenance | None = None
        if parent is not None:
            for ancestor in self._chain(parent):
                for attribute in ancestor.attributes:
                    inherited[attribute.name] = attribute
            root_kind = self.kinds[self._chain(parent)[0].name]
            root = root_kind.name
            root_provenance = root_kind.provenance
        attributes: list[m.Field] = []
        for attr_name, decl in raw.attributes.items():
            where = f"{construct}.attributes.{attr_name}"
            built = self._attribute(module, where, attr_name, decl)
            if built is None:
                ok = False
                continue
            if attr_name in inherited:
                self.err(
                    module,
                    where,
                    Code.BAD_ATTRIBUTE,
                    f"`{attr_name}` is already an attribute of `{inherited[attr_name].construct}`",
                )
                ok = False
                continue
            attributes.append(built)
        own = {attribute.name: attribute for attribute in attributes}
        everything = {**inherited, **own}
        # Identity.
        if parent is None:
            identity_names = raw.identity
            if not identity_names:
                self.err(
                    module,
                    f"{construct}.identity",
                    Code.MISSING_IDENTITY,
                    "a root kind declares the `identity` attributes that make instances the same",
                )
                ok = False
            for attr_name in identity_names:
                attribute = own.get(attr_name)
                if attribute is None:
                    if attr_name not in raw.attributes:
                        self.err(
                            module,
                            f"{construct}.identity",
                            Code.BAD_IDENTITY_ATTRIBUTE,
                            f"identity attribute `{attr_name}` is not declared",
                        )
                        ok = False
                    continue
                if attribute.optional or not attribute.type.identity_eligible:
                    reason = (
                        "optional"
                        if attribute.optional
                        else f"of type {attribute.type.text}, which is floating point or not "
                        "identity-eligible"
                    )
                    self.err(
                        module,
                        f"{construct}.identity",
                        Code.BAD_IDENTITY_ATTRIBUTE,
                        f"identity attribute `{attr_name}` is {reason}",
                    )
                    ok = False
            if len(set(identity_names)) != len(identity_names):
                self.err(
                    module,
                    f"{construct}.identity",
                    Code.BAD_IDENTITY_ATTRIBUTE,
                    "an identity attribute is listed twice",
                )
                ok = False
        elif raw.identity:
            self.err(
                module,
                f"{construct}.identity",
                Code.REFINEMENT_IDENTITY,
                "identity is declared on the root of a refinement chain; a refinement adds "
                "content, not identity",
            )
            ok = False
        # Provenance.
        provenance = (
            self._provenance(module, construct, raw.provenance)
            if (parent is None or raw.provenance is not None)
            else root_provenance
        )
        if provenance is None:
            ok = False
        elif parent is not None and root_provenance is not None and provenance != root_provenance:
            self.err(
                module,
                f"{construct}.provenance",
                Code.BAD_PROVENANCE,
                "a refinement shares the provenance of its root kind; omit it or repeat it",
            )
            ok = False
        elif not self._check_inherit(module, construct, provenance, everything):
            ok = False
        requires = self._requires(module, construct, raw.requires, own, owner="kind")
        uniques = self._uniques(module, construct, raw, attributes, own)
        if requires is None or uniques is None or provenance is None or not ok:
            return None
        return m.Kind(
            name=name,
            module=module,
            schema=self.headers[module].schema or "tk",
            doc=raw.doc,
            extends=parent,
            root=root,
            abstract=raw.abstract,
            identity=tuple(raw.identity) if parent is None else (),
            provenance=provenance,
            attributes=tuple(attributes),
            requires=requires,
            uniques=uniques,
            construct=construct,
            traces=raw.traces,
            pse=raw.pse,
        )

    def _chain(self, name: str) -> list[m.Kind]:
        chain: list[m.Kind] = []
        current: str | None = name
        while current is not None:
            kind = self.kinds[current]
            chain.append(kind)
            current = kind.extends
        return list(reversed(chain))

    def _requires(
        self,
        module: str,
        construct: str,
        declared: tuple[s.RequireDecl, ...],
        own: Mapping[str, m.Field],
        *,
        owner: str,
    ) -> tuple[m.Requirement, ...] | None:
        """The invariants of a kind (`own` are its own attributes) or of a relation (`own` are
        its keys and value columns), each with its enforcement point."""
        result: list[m.Requirement] = []
        ok = True
        seen: set[str] = set()
        for index, item in enumerate(declared):
            where = f"{construct}.requires[{index}]"
            if not SNAKE.match(item.name):
                self.err(module, where, Code.BAD_NAME, f"`{item.name}` is not lowercase snake_case")
                ok = False
            if item.name in seen:
                self.err(module, where, Code.DUPLICATE_NAME, f"invariant `{item.name}` is repeated")
                ok = False
            seen.add(item.name)
            checked = _Check(None, ())
            if item.enforced == "ddl":
                found = self._check_rule(module, where, item.check, own, owner=owner)
                if found is None:
                    ok = False
                else:
                    checked = found
            elif item.check is not None:
                self.err(
                    module,
                    f"{where}.check",
                    Code.BAD_CHECK,
                    "`check` belongs to invariants enforced by `ddl`",
                )
                ok = False
            result.append(
                m.Requirement(
                    name=item.name,
                    doc=item.doc,
                    enforced=item.enforced,
                    rule=checked.rule,
                    attributes=checked.attributes,
                    construct=where,
                    members=checked.members,
                    lower=checked.lower,
                    upper=checked.upper,
                    traces=item.traces,
                    pse=item.pse,
                )
            )
        return tuple(result) if ok else None

    def _check_rule(
        self,
        module: str,
        where: str,
        check: s.CheckDecl | None,
        own: Mapping[str, m.Field],
        *,
        owner: str,
    ) -> _Check | None:
        if check is None:
            self.err(
                module,
                f"{where}.check",
                Code.BAD_CHECK,
                "an invariant enforced by `ddl` declares a `check`",
            )
            return None
        given = {
            key: getattr(check, key) for key in CHECK_FORMS if getattr(check, key) is not None
        }
        if len(given) != 1:
            self.err(
                module,
                f"{where}.check",
                Code.BAD_CHECK,
                f"a `check` has exactly one of {', '.join(CHECK_FORMS)}",
            )
            return None
        ((rule, value),) = given.items()
        members: tuple[str, ...] = ()
        lower: float | None = None
        upper: float | None = None
        if isinstance(value, s.PresentIffDecl):
            attributes = (value.column, value.when)
            members = tuple(value.members)
        elif isinstance(value, s.WithinDecl):
            attributes = (value.column,)
            lower = None if value.lower is None else float(value.lower)
            upper = None if value.upper is None else float(value.upper)
        else:
            attributes = (value,) if isinstance(value, str) else tuple(value)
        problems: list[str] = []
        fields: list[m.Field] = []
        for attr_name in attributes:
            attribute = own.get(attr_name)
            if attribute is None:
                problems.append(f"`{attr_name}` is not a column of this {owner}'s own table")
            elif attribute.type.container != "scalar":
                problems.append(f"`{attr_name}` is not a scalar column")
            else:
                fields.append(attribute)
        if not problems:
            numeric = [
                f.type.element_kind in ("quantity", "expression", "real")
                or f.type.text == "Integer"
                for f in fields
            ]
            if (
                rule == "nonempty"
                and fields[0].type.text not in ("Text",)
                and (fields[0].type.element_kind != "identifier")
            ):
                problems.append("`nonempty` applies to Text or identifier attributes")
            elif rule in ("positive", "nonnegative") and not numeric[0]:
                problems.append(f"`{rule}` applies to numeric attributes")
            elif rule == "ordered":
                if len(fields) != 2:
                    problems.append("`ordered` names exactly two attributes")
                elif not (
                    all(numeric)
                    or (
                        fields[0].type == fields[1].type
                        and fields[0].type.text in ("Date", "Timestamp")
                    )
                ):
                    problems.append("`ordered` compares two numeric or two date attributes")
            elif rule == "ordered_same_reference":
                if len(fields) != 4:
                    problems.append(
                        "`ordered_same_reference` names four attributes: two bounds, then the "
                        "reference each is stated against"
                    )
                elif not all(numeric[:2]):
                    problems.append("`ordered_same_reference` compares two numeric bounds")
                elif fields[2].type != fields[3].type or fields[2].type.element_kind != "kind":
                    problems.append(
                        "`ordered_same_reference` takes two references of one kind as the "
                        "references of the bounds"
                    )
            elif rule == "one_of_present":
                if len(fields) < 2 or not all(f.optional for f in fields):
                    problems.append("`one_of_present` names two or more optional attributes")
            elif rule == "present_iff":
                governed, state = fields
                if not governed.optional:
                    problems.append(
                        f"`present_iff` governs an optional column, and `{governed.name}` is "
                        "always present"
                    )
                if state.type.element_kind != "enum":
                    problems.append(f"`when` names an enum column, and `{state.name}` is not one")
                else:
                    known = {member.name for member in self.enums[state.type.element].members}
                    if not members:
                        problems.append("`in` lists at least one member")
                    for member_name in members:
                        if member_name not in known:
                            problems.append(
                                f"`{member_name}` is not a member of enum `{state.type.element}`"
                            )
                    if len(set(members)) != len(members):
                        problems.append("`in` names a member twice")
            elif rule == "within":
                if not numeric[0]:
                    problems.append("`within` applies to numeric attributes")
                if lower is None and upper is None:
                    problems.append("`within` states `lower`, `upper` or both")
                for bound in (lower, upper):
                    if bound is not None and not math.isfinite(bound):
                        problems.append("`within` bounds are finite numbers")
                if lower is not None and upper is not None and lower > upper:
                    problems.append("`within` has `lower` above `upper`")
        if problems:
            self.err(module, f"{where}.check", Code.BAD_CHECK, "; ".join(problems))
            return None
        return _Check(rule, attributes, members, lower, upper)

    def _uniques(
        self,
        module: str,
        construct: str,
        raw: s.KindDecl,
        attributes: list[m.Field],
        own: dict[str, m.Field],
    ) -> tuple[m.Uniqueness, ...] | None:
        result: list[m.Uniqueness] = []
        ok = True
        names: set[str] = set()
        for index, item in enumerate(raw.unique):
            where = f"{construct}.unique[{index}]"
            problems = [
                f"`{a}` is not a scalar attribute of this kind"
                for a in item.attributes
                if a not in own or own[a].type.container == "set"
            ]
            if not item.attributes:
                problems.append("name at least one attribute")
            name = item.name or "_".join(item.attributes)
            if name in names:
                problems.append(f"uniqueness `{name}` is repeated")
            names.add(name)
            if problems:
                self.err(module, where, Code.BAD_UNIQUE, "; ".join(problems))
                ok = False
                continue
            result.append(
                m.Uniqueness(name=name, attributes=tuple(item.attributes), construct=where)
            )
        return tuple(result) if ok else None

    # -- transposition ---------------------------------------------------------------------

    def _transposition(
        self,
        module: str,
        construct: str,
        raw: s.TranspositionDecl,
        roles: Mapping[str, m.Field],
        value_names: Mapping[str, m.Field],
        by_names: Iterable[str],
        *,
        owner: str,
    ) -> m.Transposition | None:
        where = f"{construct}.transposition"
        problems: list[str] = []
        minimum = 2
        if raw.rule != "permutation_group" and len(raw.roles) != 2:
            problems.append(f"rule `{raw.rule}` names exactly two roles")
        elif len(raw.roles) < minimum:
            problems.append("a transposition names at least two roles")
        if len(set(raw.roles)) != len(raw.roles):
            problems.append("a role is named twice")
        for role in raw.roles:
            if role not in roles:
                problems.append(f"`{role}` is not a {owner}'s key or subject role")
        if problems:
            self.err(module, where, Code.BAD_TRANSPOSITION, "; ".join(problems))
            return None
        types = {roles[role].type.text for role in raw.roles}
        if len(types) != 1:
            self.err(
                module,
                where,
                Code.TRANSPOSITION_KINDS,
                f"roles {', '.join(raw.roles)} have different types ({', '.join(sorted(types))}); "
                "a transposition swaps roles of one kind",
            )
            return None
        if raw.rule == "parity":
            if raw.by is None or raw.by not in set(by_names):
                problems.append(
                    "`parity` names with `by` an index of the group or a key of the relation"
                )
        elif raw.by is not None:
            problems.append("`by` belongs to `parity`")
        if raw.rule == "reciprocal":
            if not raw.slots:
                problems.append("`reciprocal` names the slots that invert with `slots`")
            for slot in raw.slots:
                target = value_names.get(slot)
                if target is None or target.type.element_kind not in (
                    "quantity",
                    "expression",
                    "real",
                ):
                    problems.append(f"`{slot}` is not a numeric slot or value column")
        elif raw.rule == "linear":
            problems.extend(self._linear(raw, value_names))
        elif raw.slots:
            problems.append("`slots` belongs to `reciprocal` and `linear`")
        if raw.rule != "linear" and raw.matrix:
            problems.append("`matrix` belongs to `linear`")
        if raw.rule == "permutation_group":
            if not raw.permutations:
                problems.append("`permutation_group` lists its `permutations`")
            for permutation in raw.permutations:
                if sorted(permutation) != sorted(raw.roles):
                    problems.append(f"{list(permutation)} is not a permutation of the roles")
        elif raw.permutations:
            problems.append("`permutations` belongs to `permutation_group`")
        if problems:
            self.err(module, where, Code.BAD_TRANSPOSITION, "; ".join(problems))
            return None
        return m.Transposition(
            rule=raw.rule,
            roles=tuple(raw.roles),
            diagonal=raw.diagonal,
            by=raw.by,
            slots=tuple(raw.slots),
            permutations=tuple(tuple(p) for p in raw.permutations),
            matrix=tuple(tuple(row) for row in raw.matrix),
        )

    def _arrangement(
        self, owner: m.SlotGroup | m.Relation, what: str
    ) -> tuple[m.Field, m.Requirement]:
        """The `arrangement` column of a slot group or relation whose rule acts on values, and
        the `ddl` requirement that keeps it in range (meta-model section 4.3)."""
        transposition = owner.transposition
        assert transposition is not None
        construct = f"{owner.construct}.transposition"
        if transposition.rule == "permutation_group":
            meaning = (
                f"0 when the values were asserted for the canonical order of the {what}; otherwise "
                "the number, from 1, of the arrangement of the group that takes the asserted order "
                "to the canonical one (the declared permutations in declared order, then the rest "
                "of the group)."
            )
        else:
            meaning = (
                f"0 when the values were asserted for the canonical order of the {what}, 1 when "
                "they were asserted for the swapped order."
            )
        field = m.Field(
            name=m.ARRANGEMENT,
            type=TypeRef(
                container="scalar", element_kind="primitive", element="Integer", text="Integer"
            ),
            doc=f"For which order of the {what} the values were asserted: {meaning}",
            construct=construct,
        )
        requirement = m.Requirement(
            name="arrangement_range",
            doc=f"The arrangement is one the rule `{transposition.rule}` admits.",
            enforced="ddl",
            rule="within",
            attributes=(m.ARRANGEMENT,),
            construct=construct,
            lower=0,
            upper=transposition_module.arrangement_count(owner) - 1,
        )
        return field, requirement

    def _dimension(self, type_: TypeRef) -> tuple[tuple[str, int], ...] | None:
        """The dimension of a quantity or quantity-expression type as base-dimension exponents,
        or `None` for any other type."""
        if type_.container != "scalar":
            return None
        if type_.element_kind == "quantity":
            return self.unit_infos[self.quantity_types[type_.element].unit].dimensions
        if type_.element_kind == "expression":
            return self.unit_infos[self.expressions[type_.element]].dimensions
        return None

    def _linear(self, raw: s.TranspositionDecl, value_names: Mapping[str, m.Field]) -> list[str]:
        """What a `linear` rule must satisfy: slots that exist, are distinct, required and
        dimensioned, a square matrix over them that combines only slots of the dimension of the
        slot it gives, and that is an involution, so a swap applied twice returns the values."""
        problems: list[str] = []
        if not raw.slots:
            return ["`linear` names the slots its matrix multiplies with `slots`"]
        if len(set(raw.slots)) != len(raw.slots):
            problems.append("a slot is named twice")
        dimensions: dict[str, tuple[tuple[str, int], ...]] = {}
        for slot in raw.slots:
            target = value_names.get(slot)
            if target is None:
                problems.append(f"`{slot}` is not a slot or value column")
                continue
            dimension = self._dimension(target.type)
            if dimension is None:
                problems.append(
                    f"`{slot}` is {target.type.text}, not a quantity a linear combination can take"
                )
            elif target.presence != "required":
                problems.append(f"`{slot}` is stateful: a linear rule takes required slots only")
            else:
                dimensions[slot] = dimension
        size = len(raw.slots)
        if len(raw.matrix) != size or any(len(row) != size for row in raw.matrix):
            shape = " x ".join(str(n) for n in (len(raw.matrix), *{len(row) for row in raw.matrix}))
            problems.append(
                f"`matrix` is square over the {size} slots named in `slots`"
                + (f", found {shape}" if raw.matrix else ", and is missing")
            )
            return problems
        if not all(math.isfinite(entry) for row in raw.matrix for entry in row):
            problems.append("`matrix` holds finite numbers")
            return problems
        if len(dimensions) == size:
            for row, slot in zip(raw.matrix, raw.slots, strict=True):
                for entry, other in zip(row, raw.slots, strict=True):
                    if entry != 0 and dimensions[other] != dimensions[slot]:
                        problems.append(
                            f"the row for `{slot}` combines `{other}`, which has another dimension: "
                            "a row combines slots of equal dimension"
                        )
        if not transposition_module.is_involution(raw.matrix):
            problems.append(
                "`matrix` is not an involution: applying it twice does not give the identity, so "
                "a swap applied twice would not return the values"
            )
        return problems

    # -- relations -------------------------------------------------------------------------

    def _relations(self) -> None:
        for doc in self.docs:
            module = doc.header.module
            for name, raw in doc.sections.get("relations", {}).items():
                assert isinstance(raw, s.RelationDecl)
                if self._owns(name, f"relations.{name}", module):
                    built = self._relation(name, module, raw)
                    if built is not None:
                        self.relations[name] = built

    def _relation(self, name: str, module: str, raw: s.RelationDecl) -> m.Relation | None:
        construct = f"relations.{name}"
        ok = True
        if not raw.keys:
            self.err(
                module, f"{construct}.keys", Code.BAD_RELATION, "a relation has at least one key"
            )
            ok = False
        keys: dict[str, m.Field] = {}
        for key_name, key in raw.keys.items():
            where = f"{construct}.keys.{key_name}"
            if not self.symbol(module, where, key_name, "key"):
                ok = False
                continue
            type_ = self.resolve_type(key.type, module, f"{where}.type")
            if type_ is None:
                ok = False
                continue
            if not type_.identity_eligible:
                self.err(
                    module,
                    f"{where}.type",
                    Code.BAD_RELATION,
                    f"a key cannot be of type {type_.text}: keys are kinds, `Record`, "
                    "`Meta<...>`, Integer, enums, Text, Id<scheme> or other non-floating "
                    "primitives",
                )
                ok = False
                continue
            keys[key_name] = m.Field(
                name=key_name,
                type=type_,
                doc=key.doc,
                construct=where,
                traces=key.traces,
                pse=key.pse,
            )
        if raw.value is not None and raw.columns:
            self.err(
                module,
                construct,
                Code.BAD_RELATION,
                "declare either a single `value` or `columns`, not both",
            )
            ok = False
        values: dict[str, m.Field] = {}
        single = raw.value is not None
        declared_values: list[tuple[str, s.ValueDecl, str]] = []
        if raw.value is not None:
            declared_values.append(("value", raw.value, f"{construct}.value"))
        declared_values.extend(
            (col, decl, f"{construct}.columns.{col}") for col, decl in raw.columns.items()
        )
        for value_name, decl, where in declared_values:
            if not self.symbol(module, where, value_name, "value column"):
                ok = False
                continue
            if value_name in keys:
                self.err(module, where, Code.BAD_RELATION, f"`{value_name}` is also a key")
                ok = False
                continue
            type_ = self.resolve_type(decl.type, module, f"{where}.type")
            if type_ is None:
                ok = False
                continue
            if type_.container == "set":
                self.err(module, f"{where}.type", Code.BAD_TYPE, "a relation value is not a set")
                ok = False
                continue
            valid, default = self._checked_default(module, where, type_, decl.default)
            if not valid:
                ok = False
                continue
            values[value_name] = m.Field(
                name=value_name,
                type=type_,
                doc=decl.doc,
                construct=where,
                optional=decl.optional,
                default=default,
                unit_from=decl.unit_from,
                traces=decl.traces,
                pse=decl.pse,
            )
        provenance = self._provenance(module, construct, raw.provenance)
        if provenance is None:
            ok = False
        elif not self._check_inherit(module, construct, provenance, keys):
            ok = False
        absence: Literal["required", "optional", "default"] = "required"
        absence_default: m.Scalar | None = None
        if raw.absence is None:
            self.err(
                module,
                f"{construct}.absence",
                Code.BAD_ABSENCE,
                "declare `absence`: required, optional or { default = value }",
            )
            ok = False
        elif isinstance(raw.absence, s.AbsenceDefault):
            absence = "default"
            only = next(iter(values.values())) if single and len(values) == 1 else None
            if only is None:
                self.err(
                    module,
                    f"{construct}.absence",
                    Code.BAD_ABSENCE,
                    "a default applies to a relation with a single `value`",
                )
                ok = False
            else:
                try:
                    absence_default = self.scalar_value(only.type, raw.absence.default)
                except _Refused as refused:
                    self.err(module, f"{construct}.absence", Code.BAD_ABSENCE, str(refused))
                    ok = False
        else:
            absence = raw.absence
        requires = self._requires(
            module, construct, raw.requires, {**keys, **values}, owner="relation"
        )
        if requires is None:
            ok = False
        transposition: m.Transposition | None = None
        if raw.transposition is not None and keys:
            transposition = self._transposition(
                module,
                construct,
                raw.transposition,
                keys,
                values,
                [k for k, f in keys.items() if f.type.text == "Integer"],
                owner="relation",
            )
            if transposition is None:
                ok = False
        if not ok or provenance is None:
            return None
        relation = m.Relation(
            name=name,
            module=module,
            schema=self.headers[module].schema or "tk",
            doc=raw.doc,
            provenance=provenance,
            absence=absence,
            absence_default=absence_default,
            keys=tuple(keys.values()),
            values=tuple(values.values()),
            single_value=single,
            transposition=transposition,
            construct=construct,
            requires=requires or (),
            traces=raw.traces,
            pse=raw.pse,
        )
        if transposition is None or not transposition_module.stores_arrangement(relation):
            return relation
        if m.ARRANGEMENT in (*keys, *values):
            self.err(
                module,
                f"{construct}.transposition",
                Code.BAD_NAME,
                f"`{m.ARRANGEMENT}` is reserved: a relation whose rule acts on values records the "
                "arrangement of its keys in it",
            )
            return None
        arrangement, in_range = self._arrangement(relation, "keys")
        return dataclasses.replace(
            relation,
            values=(*relation.values, arrangement),
            requires=(*relation.requires, in_range),
        )

    # -- contracts -------------------------------------------------------------------------

    def _contracts(self) -> None:
        for doc in self.docs:
            module = doc.header.module
            for name, raw in doc.sections.get("contracts", {}).items():
                assert isinstance(raw, s.ContractDecl)
                construct = f"contracts.{name}"
                if not self._owns(name, construct, module):
                    continue
                ok = True
                arguments: list[m.Field] = []
                outputs: list[m.Field] = []
                roles: list[m.Field] = []
                sets: list[m.Field] = []
                for label, subject_table, subject_target in (
                    ("role", raw.roles, roles),
                    ("set", raw.sets, sets),
                ):
                    for field_name, subject in subject_table.items():
                        where = f"{construct}.{label}s.{field_name}"
                        if not self.symbol(module, where, field_name, f"contract {label}"):
                            ok = False
                            continue
                        type_ = self.resolve_type(subject.type, module, f"{where}.type")
                        if type_ is None:
                            ok = False
                            continue
                        if type_.container != "scalar" or type_.element_kind != "kind":
                            self.err(
                                module,
                                f"{where}.type",
                                Code.BAD_TYPE,
                                f"a contract {label} has a kind as its type, not {type_.text}",
                            )
                            ok = False
                            continue
                        subject_target.append(
                            m.Field(
                                name=field_name,
                                type=type_,
                                doc=subject.doc,
                                construct=where,
                                traces=subject.traces,
                                pse=subject.pse,
                            )
                        )
                set_names = {f.name for f in sets}
                for role_field in roles:
                    if role_field.name in set_names:
                        self.err(
                            module,
                            role_field.construct,
                            Code.DUPLICATE_NAME,
                            f"`{role_field.name}` is both a role and a set of this contract",
                        )
                        ok = False
                for label, table, target in (
                    ("argument", raw.arguments, arguments),
                    ("output", raw.outputs, outputs),
                ):
                    for field_name, decl in table.items():
                        where = f"{construct}.{label}s.{field_name}"
                        if not self.symbol(module, where, field_name, f"contract {label}"):
                            ok = False
                            continue
                        type_ = self.resolve_type(
                            decl.type,
                            module,
                            f"{where}.type",
                            allow_dependent=isinstance(decl, s.OutputDecl),
                        )
                        if type_ is None:
                            continue
                        if type_.container != "scalar":
                            self.err(
                                module,
                                f"{where}.type",
                                Code.BAD_TYPE,
                                "a contract argument or output is a scalar",
                            )
                            ok = False
                            continue
                        over: tuple[str, ...] = ()
                        basis: str | None = None
                        if isinstance(decl, s.ArgumentDecl):
                            over, basis = tuple(decl.over), decl.basis
                            clash = [f.name for f in (*roles, *sets) if f.name == field_name]
                            if clash:
                                self.err(
                                    module,
                                    where,
                                    Code.DUPLICATE_NAME,
                                    f"`{field_name}` is also a role or set of this contract",
                                )
                                ok = False
                            for set_name in over:
                                if set_name not in set_names:
                                    self.err(
                                        module,
                                        f"{where}.over",
                                        Code.UNKNOWN_NAME,
                                        f"`{set_name}` is not a set of this contract",
                                    )
                                    ok = False
                            if len(set(over)) != len(over):
                                self.err(
                                    module,
                                    f"{where}.over",
                                    Code.BAD_ATTRIBUTE,
                                    "an argument ranges over each set at most once",
                                )
                                ok = False
                            if basis is not None and not SNAKE.match(basis):
                                self.err(
                                    module,
                                    f"{where}.basis",
                                    Code.BAD_NAME,
                                    f"`{basis}` is not lowercase snake_case",
                                )
                                ok = False
                        if (
                            isinstance(decl, s.OutputDecl)
                            and decl.observable_from_set
                            and decl.observable is not None
                        ):
                            self.err(
                                module,
                                f"{where}.observable_from_set",
                                Code.BAD_OUTPUT_OBSERVABLE,
                                "an output names its `observable` or takes it from a set "
                                "(`observable_from_set`), not both",
                            )
                            ok = False
                        target.append(
                            m.Field(
                                name=field_name,
                                type=type_,
                                doc=decl.doc,
                                construct=where,
                                observable=getattr(decl, "observable", None),
                                observable_from_set=getattr(decl, "observable_from_set", False),
                                over=over,
                                basis=basis,
                                extra_order=getattr(decl, "extra_order", 0),
                                traces=decl.traces,
                                pse=decl.pse,
                            )
                        )
                        if isinstance(decl, s.OutputDecl):
                            self._dependent_subject(
                                module, where, target[-1], roles, f"contract `{name}`"
                            )
                if not raw.outputs:
                    self.err(
                        module,
                        f"{construct}.outputs",
                        Code.INVALID_VALUE,
                        "a contract declares at least one output",
                    )
                    ok = False
                if len(arguments) != len(raw.arguments) or len(outputs) != len(raw.outputs):
                    ok = False
                if ok:
                    self.contracts[name] = m.Contract(
                        name=name,
                        module=module,
                        doc=raw.doc,
                        arguments=tuple(arguments),
                        outputs=tuple(outputs),
                        roles=tuple(roles),
                        sets=tuple(sets),
                        construct=construct,
                        traces=raw.traces,
                        pse=raw.pse,
                    )

    # -- forms and the framework -----------------------------------------------------------

    def _framework(self, has_forms: bool) -> bool:
        framework = self.manifest.framework
        ok = True
        for role in (*m.FRAMEWORK_ROLES, *m.OPTIONAL_ROLES):
            bound = getattr(framework, role)
            construct = f"framework.{role}"
            if bound is None:
                if has_forms and role not in m.OPTIONAL_ROLES:
                    self.err(
                        None,
                        construct,
                        Code.FRAMEWORK_ROLE,
                        f"the declaration has forms, so the manifest binds `{role}` to a kind",
                    )
                    ok = False
                continue
            if bound not in self.kinds:
                named = self.names.get(bound)
                if named is None:
                    self.err(None, construct, Code.UNKNOWN_NAME, f"kind `{bound}` is not declared")
                elif named.category != "kind":
                    self.err(
                        None,
                        construct,
                        Code.BAD_TYPE,
                        f"`{bound}` is a {named.category}, expected a kind",
                    )
                ok = False
                continue
            self.framework[role] = bound
        if ok and has_forms:
            parameter_set = self.kinds[self.framework["parameter_set"]]
            own = {attribute.name: attribute for attribute in parameter_set.attributes}
            marker = own.get("slot_group")
            if marker is None or marker.optional or marker.type.text != "Meta<slot_group>":
                self.err(
                    None,
                    "framework.parameter_set",
                    Code.FRAMEWORK_ROLE,
                    f"kind `{parameter_set.name}` declares its own required attribute "
                    "`slot_group` of type `Meta<slot_group>`",
                )
                ok = False
        return ok

    def _forms_and_framework(self) -> None:
        pending: list[tuple[str, str, s.FormDecl]] = []
        for doc in self.docs:
            module = doc.header.module
            for name, raw in doc.sections.get("forms", {}).items():
                assert isinstance(raw, s.FormDecl)
                if self._owns(name, f"forms.{name}", module):
                    pending.append((module, name, raw))
        framework_ok = self._framework(bool(pending))
        for module, name, raw in pending:
            if not framework_ok:
                break
            form = self._form(module, name, raw)
            if form is not None:
                self.forms[name] = form
        if framework_ok and self.forms:
            self._contract_cycles()
            self._value_state()
            self._expand_slot_groups()

    def _form(self, module: str, name: str, raw: s.FormDecl) -> m.Form | None:
        construct = f"forms.{name}"
        ok = True
        if (
            self.lookup(
                raw.implements, ("contract",), module, f"{construct}.implements", "contract"
            )
            is None
        ):
            ok = False
        contract = self.contracts.get(raw.implements)
        if raw.status == "qualified":
            self.err(
                module,
                f"{construct}.status",
                Code.INVALID_VALUE,
                "a form is not declared qualified: its status is `catalogued` or `expressed`, and "
                "whether it is qualified is a fact about recorded runs, which the view "
                "`qual.form_qualification` lists against the form's current expression",
            )
            ok = False
        elif raw.status not in ("catalogued", "expressed"):
            self.err(
                module,
                f"{construct}.status",
                Code.INVALID_VALUE,
                f"`status` is `catalogued` or `expressed`, found {raw.status!r}",
            )
            ok = False
        groups: list[m.SlotGroup] = []
        for group_name, group in raw.slot_groups.items():
            built = self._slot_group(module, name, group_name, group, contract, raw.status)
            if built is None:
                ok = False
            else:
                groups.append(built)
        subforms: list[m.SubformSlot] = []
        for sub_name, sub in raw.subforms.items():
            where = f"{construct}.subforms.{sub_name}"
            if not SNAKE.match(sub_name):
                self.err(module, where, Code.BAD_NAME, f"`{sub_name}` is not lowercase snake_case")
                ok = False
            if (
                self.lookup(sub.accepts, ("contract",), module, f"{where}.accepts", "contract")
                is None
            ):
                ok = False
                continue
            subforms.append(
                m.SubformSlot(
                    name=sub_name,
                    qualified=f"{name}.{sub_name}",
                    form=name,
                    doc=sub.doc,
                    accepts=sub.accepts,
                    multiplicity=sub.multiplicity,
                    per=sub.per,
                    construct=where,
                    traces=sub.traces,
                    pse=sub.pse,
                )
            )
        output_observables: list[m.OutputObservable] | None = []
        if len(groups) == len(raw.slot_groups):
            output_observables = self._output_observables(module, name, raw, contract, groups)
            if output_observables is None:
                ok = False
        conventions = self._conventions(module, name, raw, groups)
        if conventions is None:
            ok = False
        defined: dict[str, list[m.ExpressionDef]] = {"let": [], "outputs": []}
        for label, table in (("let", raw.let), ("outputs", raw.outputs)):
            for item_name, text in table.items():
                where = f"{construct}.{label}.{item_name}"
                what = "local" if label == "let" else "output"
                if not self.symbol(module, where, item_name, what):
                    ok = False
                    continue
                defined[label].append(m.ExpressionDef(name=item_name, text=text, construct=where))
        blocks: list[m.ImplicitBlock] = []
        for block_name, block in raw.implicit.items():
            built = self._implicit_block(module, name, block_name, block, contract)
            if built is None:
                ok = False
            else:
                blocks.append(built)
        if not ok:
            return None
        return m.Form(
            name=name,
            module=module,
            doc=raw.doc,
            implements=raw.implements,
            completeness=raw.completeness,
            status=raw.status,
            citations=tuple(raw.citations),
            slot_groups=tuple(groups),
            subforms=tuple(subforms),
            construct=construct,
            locals=tuple(defined["let"]),
            outputs=tuple(defined["outputs"]),
            implicit=tuple(blocks),
            output_observables=tuple(output_observables or ()),
            conventions=tuple(conventions or ()),
            traces=raw.traces,
            pse=raw.pse,
        )

    def _conventions(
        self, module: str, form: str, raw: s.FormDecl, groups: list[m.SlotGroup]
    ) -> list[m.FormConvention] | None:
        """The convention facts a form reads: each names a quantity-typed attribute of the kind
        bound to the framework role `convention_set`. A fact of `component_conventions` is read
        per component: it names the slot group of the form, with one subject bound to a contract
        set, whose set for a component says which parameterization's convention set the
        component's fact is taken from."""
        construct = f"forms.{form}.conventions"
        if not raw.conventions and not raw.component_conventions:
            return []
        kind = self.framework.get(m.CONVENTION_SET_ROLE)
        if kind is None:
            self.err(
                module,
                construct,
                Code.FRAMEWORK_ROLE,
                f"the form declares `conventions`, so the manifest binds `{m.CONVENTION_SET_ROLE}` "
                "to a kind",
            )
            return None
        attributes = {a.name: a for link in self._chain(kind) for a in link.attributes}
        found: list[m.FormConvention] = []
        ok = True

        def attribute_of(name: str, where: str) -> m.Field | None:
            attribute = attributes.get(name)
            if name in (f.name for f in found):
                self.err(module, where, Code.BAD_CONVENTION, f"`{name}` is named twice")
            elif attribute is None:
                self.err(
                    module,
                    where,
                    Code.BAD_CONVENTION,
                    f"`{name}` is not an attribute of kind `{kind}`, the convention set "
                    f"(its attributes: {', '.join(attributes)})",
                )
            elif attribute.type.container != "scalar" or attribute.type.element_kind not in (
                "quantity",
                "expression",
            ):
                self.err(
                    module,
                    where,
                    Code.BAD_CONVENTION,
                    f"`{name}` is {attribute.type.text}, not a quantity: an expression reads "
                    "only quantity-typed convention facts",
                )
            else:
                return attribute
            return None

        for name in raw.conventions:
            attribute = attribute_of(name, construct)
            if attribute is None:
                ok = False
                continue
            found.append(
                m.FormConvention(
                    name=name, type=attribute.type, doc=attribute.doc, construct=construct
                )
            )
        by_name = {g.name: g for g in groups}
        for name, group_name in raw.component_conventions.items():
            where = f"forms.{form}.component_conventions.{name}"
            attribute = attribute_of(name, where)
            group = by_name.get(group_name)
            if group is None:
                self.err(
                    module,
                    where,
                    Code.BAD_CONVENTION,
                    f"`{group_name}` is not a slot group of form `{form}`",
                )
                ok = False
                continue
            bound = [b for b in group.bindings if b.kind == "set"]
            if len(group.subjects) != 1 or len(bound) != 1:
                self.err(
                    module,
                    where,
                    Code.BAD_CONVENTION,
                    f"slot group `{group.qualified}` names the convention set of a component, so "
                    "it has one subject role and it is bound to a set of the contract",
                )
                ok = False
                continue
            if attribute is None:
                ok = False
                continue
            assert bound[0].target is not None
            found.append(
                m.FormConvention(
                    name=name,
                    type=attribute.type,
                    doc=attribute.doc,
                    construct=where,
                    group=group.qualified,
                    over=bound[0].target,
                )
            )
        return found if ok else None

    def _output_observables(
        self,
        module: str,
        form: str,
        raw: s.FormDecl,
        contract: m.Contract | None,
        groups: list[m.SlotGroup],
    ) -> list[m.OutputObservable] | None:
        """Which slot supplies the observable of each output the contract declares
        `observable_from_set`: a form implementing such a contract declares one for each of those
        outputs and none for any other, and the slot is a required observable reference on a slot
        group of the form, not inside a family."""
        construct = f"forms.{form}.output_observables"
        if contract is None:
            return [] if not raw.output_observables else None
        from_set = {f.name for f in contract.outputs if f.observable_from_set}
        ok = True
        for name in raw.output_observables:
            if name not in from_set:
                self.err(
                    module,
                    f"{construct}.{name}",
                    Code.BAD_OUTPUT_OBSERVABLE,
                    f"`{name}` is not an output of contract `{contract.name}` that takes its "
                    "observable from a set"
                    + (f" (those are {', '.join(sorted(from_set))})" if from_set else ""),
                )
                ok = False
        observable_kind = self.framework.get(m.OBSERVABLE_ROLE)
        result: list[m.OutputObservable] = []
        for name in sorted(from_set):
            where = f"{construct}.{name}"
            path = raw.output_observables.get(name)
            if path is None:
                self.err(
                    module,
                    where,
                    Code.BAD_OUTPUT_OBSERVABLE,
                    f"output `{name}` of contract `{contract.name}` takes its observable from a "
                    "set: the form names the slot that supplies it in `output_observables`",
                )
                ok = False
                continue
            problem = self._observable_slot(form, path, groups, observable_kind)
            if isinstance(problem, str):
                self.err(module, where, Code.BAD_OUTPUT_OBSERVABLE, f"`{path}`: {problem}")
                ok = False
                continue
            group, slot = problem
            result.append(
                m.OutputObservable(
                    output=name,
                    slot_group=group.qualified,
                    slot=slot.name,
                    qualified=f"{group.qualified}.{slot.name}",
                    construct=where,
                )
            )
        return result if ok else None

    def _observable_slot(
        self,
        form: str,
        path: str,
        groups: list[m.SlotGroup],
        observable_kind: str | None,
    ) -> str | tuple[m.SlotGroup, m.Field]:
        """The slot `path` (`group.slot`) names, or why it cannot supply an observable."""
        if observable_kind is None:
            return "the manifest's [framework] binds no `observable` role"
        parts = path.split(".")
        by_name = {group.name: group for group in groups}
        if len(parts) == 3 and parts[0] in by_name:
            family = next((f for f in by_name[parts[0]].families if f.name == parts[1]), None)
            if family is not None:
                return "a slot inside a family cannot supply an observable: name a slot of the slot group"
        if len(parts) != 2:
            return "name a slot as `<slot group>.<slot>`"
        group = by_name.get(parts[0])
        if group is None:
            return f"`{parts[0]}` is not a slot group of form `{form}`"
        slot = next((f for f in group.slots if f.name == parts[1]), None)
        if slot is None:
            return f"`{parts[1]}` is not a slot of `{group.qualified}`"
        if slot.presence != "required":
            return "the slot is stateful: a required slot supplies the observable"
        if slot.shape != "reference" or not self._is_a(slot.type.element, observable_kind):
            return f"the slot is {slot.type.text}, not a reference to an observable (kind `{observable_kind}`)"
        return group, slot

    def _implicit_block(
        self,
        module: str,
        form: str,
        name: str,
        raw: s.ImplicitDecl,
        contract: m.Contract | None,
    ) -> m.ImplicitBlock | None:
        where = f"forms.{form}.implicit.{name}"
        ok = True
        if not SNAKE.match(name):
            self.err(module, where, Code.BAD_NAME, f"`{name}` is not lowercase snake_case")
            ok = False
        if not raw.unknowns:
            self.err(
                module,
                f"{where}.unknowns",
                Code.INVALID_VALUE,
                "an implicit block declares at least one unknown",
            )
            ok = False
        if not raw.residuals:
            self.err(
                module,
                f"{where}.residuals",
                Code.INVALID_VALUE,
                "an implicit block declares at least one residual",
            )
            ok = False
        select_by: str | None = None
        select = raw.select
        if select.startswith("by:"):
            select_by = select[3:].strip()
            select = "by"
            if not select_by:
                self.err(
                    module,
                    f"{where}.select",
                    Code.INVALID_VALUE,
                    "`by:` is followed by the expression to minimise",
                )
                ok = False
        elif select not in ("unique", "smallest", "largest"):
            self.err(
                module,
                f"{where}.select",
                Code.INVALID_VALUE,
                f"`select` is `unique`, `smallest`, `largest` or `by: <expression>`, found {raw.select!r}",
            )
            ok = False
        set_names = {f.name for f in contract.sets} if contract is not None else None
        unknowns: list[m.Unknown] = []
        for unknown_name, unknown in raw.unknowns.items():
            at = f"{where}.unknowns.{unknown_name}"
            if not self.symbol(module, at, unknown_name, "unknown"):
                ok = False
                continue
            type_ = self.resolve_type(unknown.type, module, f"{at}.type")
            if type_ is None:
                ok = False
                continue
            if type_.container != "scalar" or type_.element_kind not in ("quantity", "expression"):
                self.err(
                    module,
                    f"{at}.type",
                    Code.BAD_TYPE,
                    f"an unknown is a quantity, found type {type_.text}",
                )
                ok = False
                continue
            for set_name in unknown.over:
                if set_names is not None and set_name not in set_names:
                    self.err(
                        module,
                        f"{at}.over",
                        Code.UNKNOWN_NAME,
                        f"`{set_name}` is not a set of the contract the form implements",
                    )
                    ok = False
            if len(set(unknown.over)) != len(unknown.over):
                self.err(
                    module,
                    f"{at}.over",
                    Code.BAD_ATTRIBUTE,
                    "an unknown ranges over each set at most once",
                )
                ok = False
            bounds: dict[str, m.Bound | None] = {}
            for label in ("lower", "upper", "start"):
                value = getattr(unknown, label)
                if value is None:
                    bounds[label] = None
                elif isinstance(value, str):
                    bounds[label] = m.Bound(text=value)
                else:
                    bounds[label] = m.Bound(number=float(value))
            unknowns.append(
                m.Unknown(
                    name=unknown_name,
                    type=type_,
                    over=tuple(unknown.over),
                    lower=bounds["lower"],
                    upper=bounds["upper"],
                    start=bounds["start"],
                    construct=at,
                    traces=unknown.traces,
                    pse=unknown.pse,
                )
            )
        if not ok or len(unknowns) != len(raw.unknowns):
            return None
        return m.ImplicitBlock(
            name=name,
            form=form,
            doc=raw.doc,
            unknowns=tuple(unknowns),
            residuals=tuple(
                m.ExpressionDef(
                    name=str(position), text=text, construct=f"{where}.residuals[{position - 1}]"
                )
                for position, text in enumerate(raw.residuals, start=1)
            ),
            select=select,
            select_by=select_by,
            construct=where,
            traces=raw.traces,
            pse=raw.pse,
        )

    def _slot(
        self, module: str, construct: str, name: str, raw: s.SlotDecl, *, family: bool
    ) -> m.Field | None:
        if not self.symbol(module, construct, name, "slot"):
            return None
        if sum(item is not None for item in (raw.type, raw.accepts, raw.references)) != 1:
            self.err(
                module,
                construct,
                Code.SLOT_SHAPE,
                "a slot declares exactly one of `type`, `accepts` and `references`",
            )
            return None
        if family and raw.presence != "required":
            self.err(
                module,
                f"{construct}.presence",
                Code.BAD_FAMILY,
                "a family slot is `required`: `stateful` states belong to a slot group",
            )
            return None
        parameter_set = self.framework["parameter_set"]
        if raw.accepts is not None or raw.references is not None:
            label, named = ("accepts", raw.accepts) if raw.accepts else ("references", raw.references)
            assert named is not None
            if self.lookup(named, ("contract",), module, f"{construct}.{label}", "contract") is None:
                return None
            type_ = TypeRef(
                container="scalar", element_kind="kind", element=parameter_set, text=parameter_set
            )
            shape = "nested_set" if raw.accepts is not None else "set_reference"
        else:
            assert raw.type is not None
            resolved = self.resolve_type(
                raw.type, module, f"{construct}.type", allow_dependent=True
            )
            if resolved is None:
                return None
            if resolved.container != "scalar" or resolved.element_kind not in (
                "quantity",
                "expression",
                "enum",
                "kind",
            ):
                self.err(
                    module,
                    f"{construct}.type",
                    Code.SLOT_SHAPE,
                    "a slot holds a quantity, an enum member, a reference to an entity, a "
                    "tabulated function (`accepts` names a nested set)",
                )
                return None
            type_ = resolved
            if resolved.element_kind in ("quantity", "expression"):
                shape = "quantity"
            elif resolved.element_kind == "enum":
                shape = "enum"
            else:
                tabulated = self.framework["tabulated_function"]
                shape = (
                    "tabulated_function" if self._is_a(resolved.element, tabulated) else "reference"
                )
        return m.Field(
            name=name,
            type=type_,
            doc=raw.doc,
            construct=construct,
            presence=raw.presence,
            shape=shape,
            accepts=raw.accepts,
            references=raw.references,
            observable=raw.observable,
            extra_order=raw.extra_order,
            traces=raw.traces,
            pse=raw.pse,
        )

    def _is_a(self, kind: str, ancestor: str) -> bool:
        current: str | None = kind
        while current is not None:
            if current == ancestor:
                return True
            current = self.kinds[current].extends if current in self.kinds else None
        return False

    def _bindings(
        self,
        module: str,
        construct: str,
        raw: s.SlotGroupDecl,
        subjects: Mapping[str, m.Field],
        contract: m.Contract | None,
        status: str,
    ) -> tuple[list[m.SubjectBinding], bool]:
        """Bind each subject role of a slot group to a role or set of the form's contract: by
        the same name unless `bind` says otherwise. A `catalogued` form may leave a role
        unbound; any other form may not."""
        ok = True
        for key in raw.bind:
            if key not in subjects:
                self.err(
                    module,
                    f"{construct}.bind.{key}",
                    Code.BAD_BINDING,
                    f"`{key}` is not a subject role of this slot group",
                )
                ok = False
        bindings: list[m.SubjectBinding] = []
        roles = {f.name: f for f in contract.roles} if contract else {}
        sets = {f.name: f for f in contract.sets} if contract else {}
        for role, subject in subjects.items():
            target = raw.bind.get(role, role)
            found = roles.get(target) or sets.get(target)
            if found is None:
                bindings.append(m.SubjectBinding(role=role, kind=None, target=None))
                if contract is None or (role not in raw.bind and status == "catalogued"):
                    continue
                self.err(
                    module,
                    f"{construct}.bind.{role}"
                    if role in raw.bind
                    else f"{construct}.subject.{role}",
                    Code.BAD_BINDING,
                    f"subject role `{role}` binds to `{target}`, which is neither a role nor a "
                    f"set of contract `{contract.name}`"
                    + ("" if role in raw.bind else "; name one with `bind`"),
                )
                ok = False
                continue
            if not (
                self._is_a(subject.type.element, found.type.element)
                or self._is_a(found.type.element, subject.type.element)
            ):
                self.err(
                    module,
                    f"{construct}.subject.{role}",
                    Code.BAD_BINDING,
                    f"subject role `{role}` is a `{subject.type.element}` but `{target}` of "
                    f"contract `{contract.name if contract else ''}` is a "
                    f"`{found.type.element}`: the kinds are unrelated",
                )
                ok = False
            bindings.append(
                m.SubjectBinding(
                    role=role, kind="role" if target in roles else "set", target=target
                )
            )
        return bindings, ok

    def _slot_group(
        self,
        module: str,
        form: str,
        name: str,
        raw: s.SlotGroupDecl,
        contract: m.Contract | None,
        status: str,
    ) -> m.SlotGroup | None:
        construct = f"forms.{form}.slot_groups.{name}"
        ok = True
        if not SNAKE.match(name):
            self.err(module, construct, Code.BAD_NAME, f"`{name}` is not lowercase snake_case")
            ok = False
        subjects: dict[str, m.Field] = {}
        for role, decl in raw.subject.items():
            where = f"{construct}.subject.{role}"
            if not self.symbol(module, where, role, "subject role"):
                ok = False
                continue
            resolved = self.resolve_type(decl.type, module, f"{where}.type")
            if resolved is None:
                ok = False
                continue
            if resolved.container != "scalar" or resolved.element_kind != "kind":
                self.err(
                    module,
                    f"{where}.type",
                    Code.SUBJECT_NOT_KIND,
                    f"subject role `{role}` has type {resolved.text}, which is not a kind",
                )
                ok = False
                continue
            subjects[role] = m.Field(
                name=role,
                type=resolved,
                doc=decl.doc,
                construct=where,
                traces=decl.traces,
                pse=decl.pse,
            )
        slots: dict[str, m.Field] = {}
        for slot_name, decl in raw.slots.items():
            where = f"{construct}.slots.{slot_name}"
            built = self._slot(module, where, slot_name, decl, family=False)
            if built is None:
                ok = False
            elif slot_name in subjects:
                self.err(module, where, Code.BAD_NAME, f"`{slot_name}` is also a subject role")
                ok = False
            else:
                slots[slot_name] = built
        families: list[m.Family] = []
        for family_name, family in raw.families.items():
            built_family = self._family(module, form, name, family_name, family)
            if built_family is None:
                ok = False
            else:
                families.append(built_family)
        owner = f"slot group `{form}.{name}`"
        for slot_field in slots.values():
            self._dependent_subject(
                module, slot_field.construct, slot_field, subjects.values(), owner
            )
        for built_family in families:
            for slot_field in built_family.slots:
                self._dependent_subject(
                    module, slot_field.construct, slot_field, subjects.values(), owner
                )
        transposition = None
        if raw.transposition is not None:
            by_names = [index.name for fam in families for index in fam.indices]
            transposition = self._transposition(
                module, construct, raw.transposition, subjects, slots, by_names, owner="slot group"
            )
            if transposition is None:
                ok = False
            elif transposition.rule in transposition_module.VALUE_RULES and m.ARRANGEMENT in (
                *subjects,
                *slots,
            ):
                self.err(
                    module,
                    f"{construct}.transposition",
                    Code.BAD_NAME,
                    f"`{m.ARRANGEMENT}` is reserved: a slot group whose rule acts on values "
                    "records the arrangement of its subjects in it",
                )
                ok = False
        bindings, bound_ok = self._bindings(module, construct, raw, subjects, contract, status)
        if not (ok and bound_ok):
            return None
        return m.SlotGroup(
            name=name,
            id=f"{form}__{name}",
            qualified=f"{form}.{name}",
            form=form,
            doc=raw.doc,
            subjects=tuple(subjects.values()),
            slots=tuple(slots.values()),
            families=tuple(families),
            transposition=transposition,
            construct=construct,
            bindings=tuple(bindings),
            traces=raw.traces,
            pse=raw.pse,
        )

    def _family(
        self, module: str, form: str, group: str, name: str, raw: s.FamilyDecl
    ) -> m.Family | None:
        construct = f"forms.{form}.slot_groups.{group}.families.{name}"
        ok = True
        if not SNAKE.match(name):
            self.err(module, construct, Code.BAD_NAME, f"`{name}` is not lowercase snake_case")
            ok = False
        if not raw.index:
            self.err(module, f"{construct}.index", Code.BAD_FAMILY, "a family declares an index")
            ok = False
        indices: list[m.Field] = []
        for index_name, decl in raw.index.items():
            where = f"{construct}.index.{index_name}"
            if not self.symbol(module, where, index_name, "index"):
                ok = False
                continue
            resolved = self.resolve_type(decl.type, module, f"{where}.type")
            if resolved is None:
                ok = False
                continue
            if not resolved.identity_eligible:
                self.err(
                    module,
                    f"{where}.type",
                    Code.BAD_FAMILY,
                    f"an index cannot be of type {resolved.text}",
                )
                ok = False
                continue
            if decl.min is not None and resolved.text != "Integer":
                self.err(
                    module, f"{where}.min", Code.BAD_FAMILY, "`min` applies to Integer indices"
                )
                ok = False
                continue
            indices.append(
                m.Field(
                    name=index_name,
                    type=resolved,
                    doc=decl.doc,
                    construct=where,
                    minimum=decl.min,
                    traces=decl.traces,
                    pse=decl.pse,
                )
            )
        slots: dict[str, m.Field] = {}
        for slot_name, decl in raw.slots.items():
            where = f"{construct}.slots.{slot_name}"
            built = self._slot(module, where, slot_name, decl, family=True)
            if built is None:
                ok = False
            else:
                slots[slot_name] = built
        interval: tuple[str, str] | None = None
        if raw.interval is not None:
            bounds = (raw.interval.lower, raw.interval.upper)
            lower, upper = (slots.get(b) for b in bounds)
            if lower is None or upper is None or bounds[0] == bounds[1]:
                self.err(
                    module,
                    f"{construct}.interval",
                    Code.BAD_FAMILY,
                    "an interval names two different slots of the family",
                )
                ok = False
            elif not (lower.type.element_kind == "quantity" and upper.type.element_kind == "quantity"):
                self.err(
                    module,
                    f"{construct}.interval",
                    Code.BAD_FAMILY,
                    "interval bounds are slots of one quantity type",
                )
                ok = False
            elif lower.type != upper.type:
                self.err(
                    module,
                    f"{construct}.interval",
                    Code.BAD_FAMILY,
                    "the interval bounds are slots of different quantity types",
                )
                ok = False
            else:
                interval = bounds
        if not ok:
            return None
        return m.Family(
            name=name,
            id=f"{form}__{group}__{name}",
            qualified=f"{form}.{group}.{name}",
            slot_group=f"{form}.{group}",
            doc=raw.doc,
            indices=tuple(indices),
            slots=tuple(slots.values()),
            interval=interval,
            construct=construct,
            traces=raw.traces,
            pse=raw.pse,
        )

    def _contract_cycles(self) -> None:
        graph: dict[str, set[str]] = {name: set() for name in self.contracts}
        for form in self.forms.values():
            edges = graph.setdefault(form.implements, set())
            for group in form.slot_groups:
                edges.update(c for slot in group.slots if (c := _contract_of(slot)))
                for family in group.families:
                    edges.update(c for s in family.slots if (c := _contract_of(s)))
            edges.update(sub.accepts for sub in form.subforms)
        try:
            graphlib.TopologicalSorter(graph).prepare()
        except graphlib.CycleError as error:
            cycle = error.args[1]
            first = self.contracts.get(cycle[0])
            self.err(
                first.module if first else None,
                f"contracts.{cycle[0]}",
                Code.CONTRACT_CYCLE,
                "nested-set and referenced-set contracts form a cycle: " + " -> ".join(cycle),
            )

    def _value_state(self) -> None:
        stateful = [
            slot
            for group in (g for f in self.forms.values() for g in f.slot_groups)
            for slot in group.slots
            if slot.presence == "stateful"
        ]
        if not stateful:
            return
        members = ("known", "not_applicable", "redirect", "withheld", "stated_default")
        enum = self.enums.get("value_state")
        if enum is None or tuple(sorted(x.name for x in enum.members)) != tuple(sorted(members)):
            self.err(
                None,
                "enums.value_state",
                Code.FRAMEWORK_ROLE,
                "stateful slots need an enum `value_state` with exactly the members "
                + ", ".join(members),
            )

    def _expand_slot_groups(self) -> None:
        parameter_set = self.kinds[self.framework["parameter_set"]]
        for form in self.forms.values():
            for group in form.slot_groups:
                arrangement: tuple[m.Field, ...] = ()
                requires: tuple[m.Requirement, ...] = ()
                if transposition_module.stores_arrangement(group):
                    field, in_range = self._arrangement(group, "subjects")
                    arrangement, requires = (field,), (in_range,)
                self.kinds[group.id] = m.Kind(
                    name=group.id,
                    module=form.module,
                    schema=m.PARAM_SCHEMA,
                    doc=group.doc,
                    extends=parameter_set.name,
                    root=parameter_set.root,
                    abstract=False,
                    identity=(),
                    provenance=parameter_set.provenance,
                    attributes=(*group.subjects, *arrangement, *group.slots),
                    requires=requires,
                    uniques=(),
                    construct=group.construct,
                    origin="slot_group",
                    slot_group=group.id,
                    traces=group.traces,
                    pse=group.pse,
                )

    # -- units of Real values --------------------------------------------------------------

    def _unit_sources(self) -> None:
        """Every `Real` and `Array<Real>` attribute or relation column states `unit_from`, and
        nothing else does; the path it gives resolves."""
        observable = self.framework.get(m.OBSERVABLE_ROLE)
        for kind in self.kinds.values():
            if kind.origin != "declared":
                continue
            fields = {a.name: a for link in self._chain(kind.name) for a in link.attributes}
            for attribute in kind.attributes:
                self._unit_source(kind.module, attribute, fields, observable)
        for relation in self.relations.values():
            fields = {f.name: f for f in (*relation.keys, *relation.values)}
            for value in relation.values:
                self._unit_source(relation.module, value, fields, observable)

    def _unit_source(
        self, module: str, field: m.Field, fields: Mapping[str, m.Field], observable: str | None
    ) -> None:
        type_ = field.type
        is_real = type_.element_kind == "real" and type_.container in ("scalar", "array")
        if not is_real:
            if field.unit_from is not None:
                self.err(
                    module,
                    f"{field.construct}.unit_from",
                    Code.BAD_UNIT_FROM,
                    "`unit_from` belongs to `Real` and `Array<Real>` attributes and columns",
                )
            return
        if field.unit_from is None:
            self.err(
                module,
                field.construct,
                Code.MISSING_UNIT_FROM,
                f"a {type_.text} states where its unit comes from: `unit_from` names an attribute "
                "or column (or a dotted path of references ending in one) of type Meta<quantity_type>, "
                'Meta<slot> or a reference to an observable, or is "dimensionless"',
            )
            return
        if field.unit_from == m.DIMENSIONLESS:
            return
        problem = self._walk_unit_path(field.unit_from.split("."), fields, observable)
        if problem is not None:
            self.err(
                module,
                f"{field.construct}.unit_from",
                Code.BAD_UNIT_FROM,
                f"`{field.unit_from}`: {problem}",
            )

    def _walk_unit_path(
        self, segments: list[str], fields: Mapping[str, m.Field], observable: str | None
    ) -> str | None:
        """Why the path of references does not end in something that fixes a unit, or `None`."""
        current = fields
        for position, segment in enumerate(segments):
            found = current.get(segment)
            if found is None:
                return f"`{segment}` is not an attribute or column of the record it is read from"
            type_ = found.type
            if type_.container != "scalar":
                return f"`{segment}` is {type_.text}, not a single reference"
            if position < len(segments) - 1:
                if type_.element_kind != "kind":
                    return f"`{segment}` is {type_.text}, not a reference to a kind, so the path cannot continue"
                current = {
                    a.name: a for link in self._chain(type_.element) for a in link.attributes
                }
                continue
            if type_.element_kind == "meta" and type_.element in ("quantity_type", "slot"):
                return None
            if (
                type_.element_kind == "kind"
                and observable is not None
                and self._is_a(type_.element, observable)
            ):
                quantities = [
                    a
                    for link in self._chain(type_.element)
                    for a in link.attributes
                    if a.type.container == "scalar"
                    and a.type.element_kind == "meta"
                    and a.type.element == "quantity_type"
                ]
                if len(quantities) != 1:
                    return (
                        f"the observable kind `{type_.element}` has {len(quantities)} attributes "
                        "of type Meta<quantity_type>; exactly one fixes the unit"
                    )
                return None
            return (
                f"`{segment}` is {type_.text}; the path ends in Meta<quantity_type>, Meta<slot> "
                "or a reference to an observable"
            )
        return "the path is empty"

    # -- observables -----------------------------------------------------------------------

    def _observable_fields(self) -> list[tuple[str, m.Field]]:
        """Every contract argument, contract output and slot that names an observable, with its
        module."""
        found: list[tuple[str, m.Field]] = []
        for contract in self.contracts.values():
            found.extend((contract.module, f) for f in contract.arguments if f.observable)
            found.extend((contract.module, f) for f in contract.outputs if f.observable)
        for form in self.forms.values():
            for group in form.slot_groups:
                found.extend((form.module, f) for f in group.slots if f.observable)
                for family in group.families:
                    found.extend((form.module, f) for f in family.slots if f.observable)
        return found

    def _observables_need_a_role(self) -> None:
        if m.OBSERVABLE_ROLE in self.framework:
            return
        for contract in self.contracts.values():
            for output in contract.outputs:
                if output.observable_from_set:
                    self.err(
                        contract.module,
                        f"{output.construct}.observable_from_set",
                        Code.FRAMEWORK_ROLE,
                        "`observable_from_set` is used but the manifest's [framework] binds no "
                        "`observable` role; bind it to the kind whose declared entities are the "
                        "observables",
                    )
        for module, field_ in self._observable_fields():
            self.err(
                module,
                f"{field_.construct}.observable",
                Code.FRAMEWORK_ROLE,
                "`observable` is used but the manifest's [framework] binds no `observable` "
                "role; bind it to the kind whose declared entities are the observables",
            )

    def _observables_name_entities(self, declaration: m.Declaration) -> None:
        if m.OBSERVABLE_ROLE not in self.framework:
            return
        kind = self.framework[m.OBSERVABLE_ROLE]
        quantity_names = {
            identifier: name for name, identifier in declaration.meta_ids()["quantity_type"].items()
        }
        quantity_attributes = [
            a.name
            for a in declaration.attributes_of(kind)
            if a.type.container == "scalar"
            and a.type.element_kind == "meta"
            and a.type.element == "quantity_type"
        ]
        for module, field_ in self._observable_fields():
            entity = declaration.observable_entity(field_.observable or "")
            if entity is None:
                self.err(
                    module,
                    f"{field_.construct}.observable",
                    Code.UNKNOWN_NAME,
                    f"`{field_.observable}` is not a declared entity of kind `{kind}`",
                )
                continue
            if field_.construct.startswith("forms.") or len(quantity_attributes) != 1:
                continue  # a slot holds a reference to an observable; the writer checks its set
            identifier = entity.values.get(quantity_attributes[0])
            named = quantity_names.get(identifier) if isinstance(identifier, uuid.UUID) else None
            if named is None:
                continue
            expected = type_dimension(
                declaration,
                TypeRef(container="scalar", element_kind="quantity", element=named, text=named),
            )
            assert expected is not None  # a quantity type has a dimension
            found = type_dimension(declaration, field_.type, field_.extra_order)
            if found is not None and expected != found:
                self.err(
                    module,
                    f"{field_.construct}.observable",
                    Code.OBSERVABLE_DIMENSION,
                    f"`{field_.observable}` is a `{named}` ({describe(expected)}) and "
                    f"`{field_.name}` is a {field_.type.text} ({describe(found)}): an argument "
                    "or output that names an observable has its quantity type's dimension",
                )

    # -- composition bases -----------------------------------------------------------------

    def _basis_arguments(self) -> list[tuple[str, m.Field]]:
        """Every contract argument that names a composition basis, with its module."""
        return [
            (contract.module, argument)
            for contract in self.contracts.values()
            for argument in contract.arguments
            if argument.basis is not None
        ]

    def _bases_need_a_role(self) -> None:
        if m.COMPOSITION_BASIS_ROLE in self.framework:
            return
        for module, argument in self._basis_arguments():
            self.err(
                module,
                f"{argument.construct}.basis",
                Code.FRAMEWORK_ROLE,
                "`basis` is used but the manifest's [framework] binds no `composition_basis` "
                "role; bind it to the kind whose declared entities are the composition bases",
            )

    def _bases_name_entities(self, declaration: m.Declaration) -> None:
        if m.COMPOSITION_BASIS_ROLE not in self.framework:
            return
        kind = self.framework[m.COMPOSITION_BASIS_ROLE]
        for module, argument in self._basis_arguments():
            if declaration.composition_basis_entity(argument.basis or "") is None:
                self.err(
                    module,
                    f"{argument.construct}.basis",
                    Code.UNKNOWN_NAME,
                    f"`{argument.basis}` is not a declared entity of kind `{kind}`",
                )

    # -- assembling ------------------------------------------------------------------------

    def _declaration(self, entities: tuple[m.Entity, ...]) -> m.Declaration:
        def ordered[T](items: Mapping[str, T]) -> dict[str, T]:
            return {key: items[key] for key in sorted(items)}

        return m.Declaration(
            manifest_doc=self.manifest.doc,
            framework=dict(self.framework),
            modules=ordered(self.modules),
            enums=ordered(self.enums),
            schemes=ordered(self.schemes),
            quantity_types=ordered(self.quantity_types),
            units=ordered(self.unit_infos),
            expressions=ordered(self.expressions),
            kinds=ordered(self.kinds),
            relations=ordered(self.relations),
            contracts=ordered(self.contracts),
            forms=ordered(self.forms),
            entities=entities,
        )

    # -- entities --------------------------------------------------------------------------

    def _entities(self, declaration: m.Declaration) -> m.Declaration | None:
        raws: dict[tuple[str, str], _RawEntity] = {}
        for doc in self.docs:
            module = doc.header.module
            if self.doc_of.get(module) != doc.document:
                continue
            for kind_name, table in doc.entities.items():
                where_kind = f"entities.{kind_name}"
                if self.lookup(kind_name, ("kind",), module, where_kind, "kind") is None:
                    continue
                kind = declaration.kinds.get(kind_name)
                if kind is None:
                    continue
                for name, (decl, values) in table.items():
                    construct = f"{where_kind}.{name}"
                    if kind.abstract:
                        self.err(
                            module,
                            construct,
                            Code.ABSTRACT_ENTITY,
                            f"kind `{kind_name}` is abstract and has no direct instances",
                        )
                        continue
                    if kind.provenance.mode not in ("declaration", "own"):
                        self.err(
                            module,
                            construct,
                            Code.BAD_ENTITY,
                            f"kind `{kind_name}` has provenance `{kind.provenance.mode}`; "
                            "declared entities need `declaration` or `own`",
                        )
                        continue
                    if (kind_name, name) in raws:
                        self.err(
                            module, construct, Code.DUPLICATE_NAME, f"entity `{name}` is repeated"
                        )
                        continue
                    raws[(kind_name, name)] = _RawEntity(
                        kind=kind_name,
                        name=name,
                        decl=decl,
                        values=values,
                        module=module,
                        construct=construct,
                    )
        if self.diagnostics:
            return None
        return _EntityBuilder(self, declaration, raws).build()


@dataclass
class _EntityBuilder:
    resolver: Resolver
    declaration: m.Declaration
    raws: dict[tuple[str, str], _RawEntity]

    def __post_init__(self) -> None:
        self.meta = self.declaration.meta_ids()
        self.ids: dict[tuple[str, str], uuid.UUID | None] = {}
        self.in_progress: set[tuple[str, str]] = set()

    def build(self) -> m.Declaration | None:
        resolver = self.resolver
        entities: list[m.Entity] = []
        for key in sorted(self.raws):
            raw = self.raws[key]
            entity_id = self.id_of(key)
            values = self.values_of(raw)
            if entity_id is None or values is None:
                continue
            entities.append(
                m.Entity(
                    kind=raw.kind,
                    name=raw.name,
                    doc=raw.decl.doc,
                    id=entity_id,
                    values=values,
                    construct=raw.construct,
                    traces=raw.decl.traces,
                    pse=raw.decl.pse,
                )
            )
        if resolver.diagnostics:
            return None
        return dataclasses.replace(self.declaration, entities=tuple(entities))

    def _err(self, raw: _RawEntity, suffix: str, code: str, message: str) -> None:
        self.resolver.err(
            raw.module,
            f"{raw.construct}{suffix}",
            Code(code),
            message,
        )

    def _raw_value(self, raw: _RawEntity, attribute: m.Field) -> object | None:
        if attribute.name in raw.values:
            return raw.values[attribute.name]
        if attribute.name == "name":
            return raw.name
        return None

    def _coerce(self, raw: _RawEntity, attribute: m.Field, value: object) -> object:
        type_ = attribute.type
        if type_.container == "array":
            if not isinstance(value, list):
                raise _Refused("bad-entity", f"expected a list for type {type_.text}")
            element = TypeRef(
                container="scalar",
                element_kind=type_.element_kind,
                element=type_.element,
                text=type_.element,
            )
            return tuple(self.resolver.scalar_value(element, item) for item in value)
        if type_.container == "range":
            if (
                not isinstance(value, list)
                or len(value) != 2
                or any(isinstance(v, bool) or not isinstance(v, (int, float)) for v in value)
            ):
                raise _Refused("bad-entity", f"expected [lower, upper] for type {type_.text}")
            lower, upper = float(value[0]), float(value[1])
            scale = self.resolver.quantity_types[type_.element].scale
            if not (math.isfinite(lower) and math.isfinite(upper)) or lower > upper:
                raise _Refused("bad-entity", f"{value!r} is not a closed finite interval")
            if scale == "absolute" and lower < 0:
                raise _Refused("bad-entity", "an absolute interval is not negative")
            return (lower, upper)
        if type_.container == "set":
            if not isinstance(value, list) or not all(isinstance(v, str) for v in value):
                raise _Refused("bad-entity", "expected a list of entity names")
            found = {self._reference(raw, attribute, type_.element, name) for name in value}
            return tuple(sorted(found, key=str))
        kind = type_.element_kind
        if kind == "kind":
            if not isinstance(value, str):
                raise _Refused("bad-entity", "expected the name of a declared entity")
            return self._reference(raw, attribute, type_.element, value)
        if kind == "meta":
            if not isinstance(value, str):
                raise _Refused("bad-entity", f"expected the name of a {type_.element}")
            found_id = self.meta[type_.element].get(value)
            if found_id is None:
                raise _Refused(
                    "bad-entity", f"`{value}` is not a declared {type_.element} ({type_.text})"
                )
            return found_id
        if kind == "record":
            raise _Refused("bad-entity", "a declared entity cannot reference a record")
        try:
            return self.resolver.scalar_value(type_, value)
        except _Refused as refused:
            code = "bad-enum-member" if refused.code == "bad-enum-member" else "bad-entity"
            raise _Refused(code, str(refused)) from refused

    def _reference(self, raw: _RawEntity, attribute: m.Field, kind: str, name: str) -> uuid.UUID:
        candidates = [
            key for key in self.raws if key[1] == name and self.declaration.is_a(key[0], kind)
        ]
        if not candidates:
            raise _Refused("bad-entity", f"no declared entity `{name}` of kind `{kind}`")
        if len(candidates) > 1:
            raise _Refused(
                "bad-entity",
                f"`{name}` names entities of several kinds ({', '.join(k[0] for k in candidates)})",
            )
        found = self.id_of(candidates[0])
        if found is None:
            raise _Refused("bad-entity", f"entity `{name}` has no identifier")
        return found

    def id_of(self, key: tuple[str, str]) -> uuid.UUID | None:
        if key in self.ids:
            return self.ids[key]
        raw = self.raws[key]
        if key in self.in_progress:
            self._err(
                raw,
                "",
                "bad-entity",
                "the identity of this entity depends on itself through references",
            )
            return None
        self.in_progress.add(key)
        try:
            kind = self.declaration.kinds[raw.kind]
            chain_attributes = {a.name: a for a in self.declaration.attributes_of(raw.kind)}
            root = self.declaration.kinds[kind.root]
            values: list[identity.IdentityValue] = []
            ok = True
            for attr_name in root.identity:
                attribute = chain_attributes[attr_name]
                given = self._raw_value(raw, attribute)
                if given is None and attribute.default is None:
                    self._err(
                        raw,
                        "",
                        "bad-entity",
                        f"identity attribute `{attr_name}` has no value",
                    )
                    ok = False
                    continue
                try:
                    values.append(
                        attribute.default
                        if given is None
                        else self._coerce(raw, attribute, given)
                    )
                except _Refused as refused:
                    self._err(raw, f".{attr_name}", refused.code, str(refused))
                    ok = False
            result = identity.identifier(root.name, values) if ok else None
        finally:
            self.in_progress.discard(key)
        self.ids[key] = result
        return result

    def values_of(self, raw: _RawEntity) -> dict[str, m.EntityValue] | None:
        attributes = {a.name: a for a in self.declaration.attributes_of(raw.kind)}
        ok = True
        for key in raw.values:
            if key not in attributes:
                self._err(
                    raw, f".{key}", "bad-entity", f"`{key}` is not an attribute of `{raw.kind}`"
                )
                ok = False
        values: dict[str, m.EntityValue] = {}
        for name, attribute in attributes.items():
            given = self._raw_value(raw, attribute)
            if given is None:
                if attribute.default is not None:
                    values[name] = attribute.default
                elif attribute.type.container == "set":
                    values[name] = ()
                elif not attribute.optional:
                    self._err(raw, f".{name}", "bad-entity", f"attribute `{name}` needs a value")
                    ok = False
                continue
            try:
                values[name] = self._coerce(raw, attribute, given)
            except _Refused as refused:
                self._err(raw, f".{name}", refused.code, str(refused))
                ok = False
        return values if ok else None
