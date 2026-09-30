# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""`mappings/<id>/mapping.toml`: the declared rules of one source's mapping (pipeline section 2).

The file is decoded into the structs below (an unknown key is refused) and then checked against
the declaration and the staged tables: every staged table has a disposition, every column of a
table that maps rows has a rule, every target exists and every unit is compatible with the
target's storage unit. `validate` returns every problem found, each naming the table and column.
"""

from __future__ import annotations

import re
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

import msgspec
import pyarrow as pa
from msgspec import Struct

from thermo_knowledge import pipeline_contract as pc
from thermo_knowledge.canonical.values import ValueRefused, convert, storage_unit
from thermo_knowledge.declaration import model as m
from thermo_knowledge.declaration.types import registry
from thermo_knowledge.mapping import claims
from thermo_knowledge.staging import schema as staging_schema

type Scalar = str | int | float | bool
Disposition = Literal["mapped", "out_of_scope", "deferred"]
Precision = Literal["exact", "narrower", "broader", "close"]

PROVENANCE_COLUMNS = (staging_schema.ARTIFACT, staging_schema.LOCATOR)
_STRICT = {"forbid_unknown_fields": True}


class ScopeSpec(Struct, **_STRICT):
    """A scope of source entities: each row of `table` is one, keyed by column `key`."""

    table: str
    key: str
    doc: str


class FormulaScope(Struct, **_STRICT):
    """A scope whose source entities a formula and charge identify (resolution rule 4)."""

    scope: str
    reason: str
    discriminator: str | None = None


class DerivationSpec(Struct, **_STRICT):
    """The derivation that produced the records of a table or partition: one `derivation` (a
    `fit` when `kind = "fit"`, with its `outcome`) per group of rows that make a record, with a
    `derivation_output` row for each record. `method` is text the mapping declares, not the
    source's."""

    kind: str
    method: str
    outcome: str | None = None


class Partition(Struct, **_STRICT):
    """The rows of a table matching `where` (every column equals its value, or is one of its
    listed values) and what the mapping does with them, in place of the table's disposition.
    `derivation` states how the records made from these rows were produced.

    `constants` are canonical values the rows' records take that no column holds, by target
    (`kind.attribute`): a reference to a declared entity by name, an enum member, text, a number,
    or `{ value = ..., unit = "..." }` for a dimensioned value."""

    name: str
    where: dict[str, Scalar | list[Scalar]]
    disposition: Disposition
    reason: str | None = None
    wave: int | None = None
    loss: str | None = None
    origin_role: str | None = None
    constants: dict[str, Scalar | dict[str, Scalar]] = {}
    derivation: DerivationSpec | None = None


class FieldRule(Struct, **_STRICT):
    """What one column of a table becomes.

    Exactly one of: `target` (a value that becomes a canonical value), `role = "structure"` (the
    column keys, selects or indexes rows and carries no value of its own) or `disposition`
    `out_of_scope` / `deferred` (the column is not mapped, with a reason).
    """

    target: str | None = None
    scheme: str | None = None
    unit: str | None = None
    precision: Precision | None = None
    loss: str | None = None
    absent: list[Scalar] = []
    pattern: str | None = None
    otherwise_scheme: str | None = None
    offset: int | None = None
    role: Literal["structure"] | None = None
    disposition: Literal["out_of_scope", "deferred"] | None = None
    reason: str | None = None
    wave: int | None = None


class TableRule(Struct, **_STRICT):
    """What the mapping does with one staged table; `constants` apply to every mapped row that
    its partitions' own constants do not override."""

    disposition: Disposition
    reason: str | None = None
    wave: int | None = None
    loss: str | None = None
    origin_role: str | None = None
    constants: dict[str, Scalar | dict[str, Scalar]] = {}
    derivation: DerivationSpec | None = None
    partitions: list[Partition] = []
    fields: dict[str, FieldRule] = {}


class ParameterizationSpec(Struct, **_STRICT):
    """A parameterization the mapping creates: the collection its sets belong to, as the carrier
    presents it. `origin_role` is how the carrier presents the collection, whichever table's rows
    emit it; the role of a set is that of its own rows. A convention set the parameterization
    names takes the same role."""

    key: str
    revision: str
    title: str
    coherence: str
    origin_role: str
    convention_set: str | None = None
    no_convention_set: str | None = None


class MappingSpec(Struct, **_STRICT):
    source: str
    doc: str
    tables: dict[str, TableRule]
    scopes: dict[str, ScopeSpec] = {}
    formula_scope: list[FormulaScope] = []
    parameterizations: dict[str, ParameterizationSpec] = {}
    convention_sets: dict[str, dict[str, Scalar | dict[str, Scalar]]] = {}


class SpecError(Exception):
    """A mapping.toml that is unreadable or inconsistent; `problems` lists each."""

    def __init__(self, problems: list[str]) -> None:
        self.problems = problems
        super().__init__("\n".join(problems))


def load_spec(path: Path) -> MappingSpec:
    """Decode `mapping.toml`; raises `SpecError` naming the file and the key."""
    try:
        data = tomllib.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as error:
        raise SpecError([f"{path}: missing"]) from error
    except tomllib.TOMLDecodeError as error:
        raise SpecError([f"{path}: invalid TOML: {error}"]) from error
    try:
        return msgspec.convert(data, MappingSpec, strict=True)
    except msgspec.ValidationError as error:
        raise SpecError([f"{path}: {error}"]) from error


# -- targets ----------------------------------------------------------------------------------


@dataclass(frozen=True)
class Target:
    """A resolved canonical target: `kind.attribute`, `relation.column`, `form.group.slot` or
    `form.group.family.slot` (or `.index`)."""

    text: str
    field: m.Field
    unit: str | None
    """The storage unit when the field is a quantity, or a `Real` that is always dimensionless;
    `None` for a `Real` whose unit its record fixes (`unit_from`)."""

    @property
    def dimensioned(self) -> bool:
        return self.unit is not None or self.field.type.element_kind == "real"


def resolve_target(decl: m.Declaration, text: str) -> Target | str:
    """The target `text` names, or a description of why it names none."""
    parts = text.split(".")
    found: m.Field | None = None
    if len(parts) == 2:
        owner, name = parts
        kind = decl.kinds.get(owner)
        relation = decl.relations.get(owner)
        if kind is not None and kind.origin == "declared":
            found = next((a for a in decl.attributes_of(owner) if a.name == name), None)
            if found is None:
                return f"`{name}` is not an attribute of kind `{owner}`"
        elif relation is not None:
            found = next((f for f in (*relation.keys, *relation.values) if f.name == name), None)
            if found is None:
                return f"`{name}` is not a key or value column of relation `{owner}`"
        else:
            return f"`{owner}` is neither a kind nor a relation"
    elif len(parts) == 3:
        group = next((g for g in decl.slot_groups if g.qualified == ".".join(parts[:2])), None)
        if group is None:
            return f"`{'.'.join(parts[:2])}` is not a slot group (`form.group`)"
        found = next((s for s in group.slots if s.name == parts[2]), None)
        if found is None:
            return f"`{parts[2]}` is not a slot of `{group.qualified}`"
    elif len(parts) == 4:
        family = next((f for f in decl.families if f.qualified == ".".join(parts[:3])), None)
        if family is None:
            return f"`{'.'.join(parts[:3])}` is not a family (`form.group.family`)"
        found = next((f for f in (*family.indices, *family.slots) if f.name == parts[3]), None)
        if found is None:
            return f"`{parts[3]}` is not an index or slot of `{family.qualified}`"
    else:
        return "a target is `kind.attribute`, `relation.column`, `form.group.slot` or `form.group.family.slot`"
    unit = storage_unit(decl, found.type)
    if unit is None and found.unit_from == m.DIMENSIONLESS:
        unit = m.DIMENSIONLESS
    return Target(text=text, field=found, unit=unit)


# -- validation -------------------------------------------------------------------------------


def _unit_parses(unit: str) -> str | None:
    try:
        registry().parse_units(unit)
    except Exception as error:  # pint raises many error types for bad unit text
        return f"`{unit}` is not a unit pint can parse: {error}"
    return None


def stated_unit(field: pa.Field) -> str | None:
    """The unit a staged column states, or `None` when it states none."""
    unit = staging_schema.field_metadata(field).get(staging_schema.UNIT, staging_schema.NOT_STATED)
    if unit in (staging_schema.NOT_STATED, "not applicable") or unit.startswith("as stated"):
        return None
    return unit


def naming_schemes(decl: m.Declaration) -> set[str]:
    """The names of the declared naming schemes."""
    return {entity.name for entity in decl.entities if entity.kind == pc.NAMING_SCHEME.declared}


def maps_rows(rule: TableRule) -> bool:
    """Whether any row of the table may be mapped."""
    return rule.disposition == claims.MAPPED or any(
        p.disposition == claims.MAPPED for p in rule.partitions
    )


def validate(
    spec: MappingSpec, decl: m.Declaration, staged: dict[str, pa.Schema], *, source: str
) -> list[str]:
    """Every problem of `spec` against the declaration and the staged tables."""
    problems: list[str] = []
    if spec.source != source:
        problems.append(f"source: `{spec.source}` is not the mapping's directory name `{source}`")
    roles = {member.name for member in decl.enums[pc.ORIGIN_ROLE.declared].members}
    schemes = naming_schemes(decl)
    for table in sorted(set(staged) - set(spec.tables)):
        problems.append(f"table {table}: has no declared disposition")
    for table in sorted(set(spec.tables) - set(staged)):
        problems.append(f"table {table}: is declared but not staged")
    for table, rule in spec.tables.items():
        if table in staged:
            _table(problems, spec, decl, table, rule, staged[table], roles, schemes)
    for name, scope in spec.scopes.items():
        schema = staged.get(scope.table)
        if schema is None:
            problems.append(f"scope {name}: table `{scope.table}` is not staged")
        elif scope.key not in schema.names:
            problems.append(f"scope {name}: `{scope.key}` is not a column of `{scope.table}`")
    for item in spec.formula_scope:
        if item.scope not in spec.scopes:
            problems.append(f"formula_scope: `{item.scope}` is not a declared scope")
    for name, attributes in spec.convention_sets.items():
        _convention_set(problems, decl, name, attributes)
    coherence = {member.name for member in decl.enums[pc.COHERENCE.declared].members}
    for name, item in spec.parameterizations.items():
        if item.coherence not in coherence:
            problems.append(f"parameterization {name}: `{item.coherence}` is not a coherence")
        if item.origin_role not in roles:
            problems.append(f"parameterization {name}: `{item.origin_role}` is not an origin role")
        if (item.convention_set is None) == (item.no_convention_set is None):
            problems.append(
                f"parameterization {name}: state exactly one of `convention_set` (the name of a "
                "[convention_sets] entry) and `no_convention_set` (why it has none)"
            )
        elif item.convention_set is not None and item.convention_set not in spec.convention_sets:
            problems.append(
                f"parameterization {name}: `{item.convention_set}` is not a [convention_sets] entry"
            )
        try:
            item.revision.format(pin="")
        except (KeyError, IndexError, ValueError):
            problems.append(
                f"parameterization {name}: `revision` may use only the placeholder {{pin}}"
            )
    return problems


def _convention_set(
    problems: list[str],
    decl: m.Declaration,
    name: str,
    attributes: dict[str, Scalar | dict[str, Scalar]],
) -> None:
    """One `[convention_sets.<name>]` table states attributes of the `convention_set` kind as
    value rules do: a dimensioned fact (the gas constant) as `{ value, unit }` with a unit that
    converts to its storage unit, an enum member by name, text or a number as itself."""
    kind = pc.CONVENTION_SET.declared
    for attribute, raw in attributes.items():
        where = f"convention_sets.{name}.{attribute}"
        target = resolve_target(decl, f"{kind}.{attribute}")
        if isinstance(target, str):
            problems.append(f"{where}: {target}")
            continue
        type_ = target.field.type
        if target.dimensioned:
            if not isinstance(raw, dict) or set(raw) != {"value", "unit"}:
                problems.append(f"{where}: state `{{ value = ..., unit = \"...\" }}`, a dimensioned fact")
            elif target.unit is not None:
                try:
                    convert(1.0, str(raw["unit"]), target.unit)
                except ValueRefused as error:
                    problems.append(f"{where}: {error}")
            else:
                message = _unit_parses(str(raw["unit"]))
                if message:
                    problems.append(f"{where}: {message}")
        elif isinstance(raw, dict):
            problems.append(f"{where}: the attribute has no unit, so the fact states none")
        elif type_.element_kind == "enum":
            members = {member.name for member in decl.enums[type_.element].members}
            if raw not in members:
                problems.append(f"{where}: `{raw}` is not a member of `{type_.element}`")
        elif type_.element_kind == "kind":
            problems.append(f"{where}: a reference to another record cannot be stated here")


def _disposition(
    problems: list[str],
    where: str,
    disposition: str,
    reason: str | None,
    wave: int | None,
) -> None:
    if disposition != claims.MAPPED and not reason:
        problems.append(f"{where}: `{disposition}` needs a `reason`")
    if disposition == claims.DEFERRED and wave is None:
        problems.append(f"{where}: `deferred` names the `wave` that will map it")
    if disposition != claims.DEFERRED and wave is not None:
        problems.append(f"{where}: `wave` belongs to `deferred`")


def _table(
    problems: list[str],
    spec: MappingSpec,
    decl: m.Declaration,
    table: str,
    rule: TableRule,
    schema: pa.Schema,
    roles: set[str],
    schemes: set[str],
) -> None:
    where = f"table {table}"
    _disposition(problems, where, rule.disposition, rule.reason, rule.wave)
    if rule.disposition == claims.MAPPED and rule.origin_role is None:
        problems.append(f"{where}: a mapped table names the `origin_role` of its records")
    if rule.origin_role is not None and rule.origin_role not in roles:
        problems.append(f"{where}: `{rule.origin_role}` is not an origin role")
    names = set(schema.names)
    _constants(problems, decl, where, rule.constants, rule)
    _derivation(problems, decl, where, rule.derivation)
    partition_names: set[str] = set()
    for partition in rule.partitions:
        label = f"{where} partition {partition.name}"
        if partition.name in partition_names:
            problems.append(f"{label}: the name is repeated")
        partition_names.add(partition.name)
        _disposition(problems, label, partition.disposition, partition.reason, partition.wave)
        _constants(problems, decl, label, partition.constants, rule)
        _derivation(problems, decl, label, partition.derivation)
        if (
            partition.disposition == claims.MAPPED
            and partition.origin_role is None
            and rule.origin_role is None
        ):
            problems.append(f"{label}: a mapped partition names the `origin_role` of its records")
        if partition.origin_role is not None and partition.origin_role not in roles:
            problems.append(f"{label}: `{partition.origin_role}` is not an origin role")
        if not partition.where:
            problems.append(f"{label}: `where` names at least one column")
        for column in partition.where:
            if column not in names:
                problems.append(f"{label}: `{column}` is not a column of the table")
    if not maps_rows(rule):
        if rule.fields:
            problems.append(f"{where}: no row is mapped, so it declares no field rules")
        return
    for column in sorted(names - set(rule.fields) - set(PROVENANCE_COLUMNS)):
        problems.append(f"{where}: column `{column}` has neither a rule nor a declared skip")
    for column, field_rule in rule.fields.items():
        if column not in names:
            problems.append(f"{where}: field rule `{column}` names no column")
            continue
        _field(
            problems, decl, f"{where} column {column}", field_rule, schema.field(column), schemes
        )


def _constants(
    problems: list[str],
    decl: m.Declaration,
    where: str,
    constants: dict[str, Scalar | dict[str, Scalar]],
    rule: TableRule,
) -> None:
    mapped = {f.target for f in rule.fields.values() if f.target is not None}
    for text, value in constants.items():
        target = resolve_target(decl, text)
        if isinstance(target, str):
            problems.append(f"{where}: constant `{text}`: {target}")
            continue
        if text in mapped:
            problems.append(f"{where}: constant `{text}` is also the target of a column")
        type_ = target.field.type
        if isinstance(value, dict):
            if set(value) != {"value", "unit"} or not target.dimensioned:
                problems.append(
                    f"{where}: constant `{text}`: give `{{ value, unit }}` for a dimensioned target"
                )
        elif type_.element_kind == "enum":
            members = {member.name for member in decl.enums[type_.element].members}
            if value not in members:
                problems.append(
                    f"{where}: constant `{text}`: `{value}` is not a member of `{type_.element}`"
                )
        elif type_.element_kind == "kind":
            if not isinstance(value, str) or declared_entity(decl, type_.element, value) is None:
                problems.append(
                    f"{where}: constant `{text}`: `{value}` is not a declared entity of kind `{type_.element}`"
                )


def _derivation(
    problems: list[str], decl: m.Declaration, where: str, spec: DerivationSpec | None
) -> None:
    if spec is None:
        return
    kinds = {member.name for member in decl.enums[pc.DERIVATION_KIND.declared].members}
    if spec.kind not in kinds:
        problems.append(f"{where}: derivation kind `{spec.kind}` is not a `derivation_kind`")
    if not spec.method.strip():
        problems.append(f"{where}: a derivation states its `method`")
    outcomes = {member.name for member in decl.enums[pc.FIT_OUTCOME.declared].members}
    if spec.kind == pc.DERIVATION_KIND.member("fit"):
        if spec.outcome not in outcomes:
            problems.append(
                f"{where}: a fit states its `outcome` (one of {', '.join(sorted(outcomes))})"
            )
    elif spec.outcome is not None:
        problems.append(f"{where}: `outcome` belongs to a fit")


def declared_entity(decl: m.Declaration, kind: str, name: str) -> m.Entity | None:
    """The declared entity `name` of `kind` (or of a refinement of it), if there is exactly one."""
    found = [e for e in decl.entities if e.name == name and decl.is_a(e.kind, kind)]
    return found[0] if len(found) == 1 else None


def _field(
    problems: list[str],
    decl: m.Declaration,
    where: str,
    rule: FieldRule,
    field: pa.Field,
    schemes: set[str],
) -> None:
    kinds = [rule.target is not None, rule.role is not None, rule.disposition is not None]
    if sum(kinds) != 1:
        problems.append(f"{where}: give exactly one of `target`, `role` and `disposition`")
        return
    if rule.role is not None or rule.disposition is not None:
        if rule.disposition is not None:
            _disposition(problems, where, rule.disposition, rule.reason, rule.wave)
        elif not rule.reason:
            problems.append(f"{where}: a structure column states its `reason`")
        return
    assert rule.target is not None
    if rule.precision is None:
        problems.append(f"{where}: a mapped column states its `precision`")
    for pattern in (rule.pattern,):
        if pattern is not None:
            try:
                re.compile(pattern)
            except re.error as error:
                problems.append(f"{where}: `pattern` is not a regular expression: {error}")
    if rule.otherwise_scheme is not None and rule.pattern is None:
        problems.append(f"{where}: `otherwise_scheme` needs a `pattern`")
    target = resolve_target(decl, rule.target)
    if isinstance(target, str):
        problems.append(f"{where}: target `{rule.target}`: {target}")
        return
    is_assertion = rule.target == "identity_assertion.value"
    if is_assertion:
        for scheme in (rule.scheme, rule.otherwise_scheme):
            if scheme is not None and scheme not in schemes:
                problems.append(f"{where}: `{scheme}` is not a declared naming scheme")
        if rule.scheme is None:
            problems.append(f"{where}: an identity assertion names its `scheme`")
    elif rule.scheme is not None or rule.otherwise_scheme is not None:
        problems.append(f"{where}: `scheme` belongs to the target `identity_assertion.value`")
    if target.dimensioned:
        if rule.unit is None:
            problems.append(f"{where}: the target is dimensioned, so the rule states its `unit`")
            return
        if target.unit is not None:
            try:
                convert(1.0, rule.unit, target.unit)
            except ValueRefused as error:
                problems.append(f"{where}: {error}")
                return
        else:
            message = _unit_parses(rule.unit)
            if message:
                problems.append(f"{where}: {message}")
        stated = stated_unit(field)
        if stated is None:
            if not rule.loss:
                problems.append(
                    f"{where}: the source states no unit, so the rule records its assumption "
                    f"of `{rule.unit}` as `loss`"
                )
        elif stated != rule.unit:
            problems.append(
                f"{where}: the source states `{stated}` and the rule says `{rule.unit}`"
            )
    elif rule.unit is not None:
        problems.append(f"{where}: the target has no unit, so the rule states none")
    if rule.offset is not None and target.field.type.text != "Integer":
        problems.append(f"{where}: `offset` applies to an Integer target")
