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
from collections.abc import Mapping
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
DERIVED = "derived:"
ELEMENT_KEY = re.compile(r"(.+)\[(\d+)\]")
_STRICT = {"forbid_unknown_fields": True}


class ClassRule(Struct, kw_only=True, **_STRICT):
    """One line of a value-to-class table: a row whose column value is one of `values` (exactly)
    or matches `pattern` (as a whole) is of class `class`, for the stated `reason`."""

    reason: str
    entity_class: str = msgspec.field(name="class")
    values: list[str] = []
    pattern: str | None = None

    def applies(self, value: object) -> bool:
        if not isinstance(value, str):
            return False
        if self.pattern is not None:
            return re.fullmatch(self.pattern, value) is not None
        return value in self.values


class ClassBy(Struct, **_STRICT):
    """The class of each source entity of a scope from a column of its row: the first rule the
    value satisfies decides, and a value no rule covers (or an absent one) is of class `default`."""

    column: str
    default: str
    rules: list[ClassRule]


class MixtureSpec(Struct, **_STRICT):
    """What a scope states about its defined mixtures: whether their composition is fixed by
    definition or by measurement (`definition`, a `mixture_definition` member), and whether the
    fractions the source gives are mole or mass fractions (`basis`: `mole` or `mass`)."""

    definition: str
    basis: Literal["mole", "mass"]
    reason: str


class ScopeSpec(Struct, **_STRICT):
    """A scope of source entities: each row of `table` is one, keyed by column `key`, of the class
    `class` (an `entity_class` member) or of the class `class_by` gives from a column of its row.
    A scope that has defined mixtures states `mixture`."""

    table: str
    key: str
    doc: str
    entity_class: str | None = msgspec.field(name="class", default=None)
    class_by: ClassBy | None = None
    mixture: MixtureSpec | None = None

    def classes(self) -> set[str]:
        """Every class an entity of the scope can have."""
        if self.class_by is not None:
            return {self.class_by.default, *(rule.entity_class for rule in self.class_by.rules)}
        return {self.entity_class} if self.entity_class is not None else set()

    def class_of(self, value: object) -> str:
        """The class of an entity whose `class_by` column holds `value` (the scope's class when it
        has a single one)."""
        if self.class_by is None:
            assert self.entity_class is not None  # validated
            return self.entity_class
        for rule in self.class_by.rules:
            if rule.applies(value):
                return rule.entity_class
        return self.class_by.default


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


class DecodeRule(Struct, kw_only=True, **_STRICT):
    """One line of a decoding: a row that satisfies every condition of `where` decodes to `to`
    (a declared entity or an enum member, by name), for the stated `reason`.

    A condition is `column = value` (exactly), `column = [values]` (one of them) or `column =
    { pattern = "..." }` (the whole text of the value matches). Named groups of the patterns are
    available to `to`, `polymorph` and `key`, text templates (`"{group}"`): `polymorph` is the
    decoded polymorph of a species form and `key` the key of a record the decoding names. A row
    no rule covers is undecoded, and the mapping holds it (or goes on without, where the fact is
    optional)."""

    where: dict[str, Scalar | list[Scalar] | dict[str, str]]
    to: str
    reason: str
    polymorph: str | None = None
    key: str | None = None


class Decoding(Struct, **_STRICT):
    """A table from what a source states to what the mapping makes of it: the aggregation a phase
    flag stands for, the phase a transition marker names. The first rule a row satisfies decides."""

    doc: str
    rules: list[DecodeRule]


class Join(Struct, **_STRICT):
    """A related table whose columns a partition may test (`<join>.<column>` in `where`): each
    row of this table is joined to the row of `table` whose columns equal the values of this
    table's columns in `on` (this table's column to the related table's column). A row with no
    related row never satisfies a condition on it; more than one related row is refused."""

    table: str
    on: dict[str, str]


class FieldRule(Struct, **_STRICT):
    """What one column of a table becomes.

    Exactly one of: `target` (a value that becomes a canonical value), `role = "structure"` (the
    column keys, selects or indexes rows and carries no value of its own) or `disposition`
    `out_of_scope` / `deferred` (the column is not mapped, with a reason).

    A column holding a list is ruled by one rule for each element it has, named `column[index]`
    (an index from zero); the column itself then has no rule.

    A value rule must be applied to at least one mapped row of its table, or the run is refused;
    `optional = true` declares a field the source fills only sometimes, whose rule may be applied
    to none (the run then reports it).

    - `factor` multiplies a numeric value (a count the source states with the opposite sign);
    - `default` is the value an absent source value takes, in the rule's unit, the source's own
      documented default, which the rule's `loss` states;
    - `case` (`capitalize`, `upper`, `lower`) is applied to the text that names a declared entity;
    - `text` names the column that holds the number of this rule as the source wrote it: the
      significant digits of that text are the digits the source reports for the value;
    - `decode` names a `[decodings]` table: the row's decoded value is the rule's value;
    - `equals` states the exact value a column holds in every row that maps, a structure that the
      mapping relies on; a row with another value is held (`pattern_mismatch`);
    - `text_unit` marks a text column whose value is a number followed by its unit, which the
      source states (`0.0 kJ/mol`); the rule's `unit`, which it needs only with a `default`, is
      the unit of that default;
    - `text_number` is a regular expression with one group that captures the number a text
      column holds (the temperature a key path ends in), which the rule's `unit` is the unit of;
    - `holds` marks a column the mapping does not map (`out_of_scope` or `deferred`) and cannot
      drop either: a row that states it is held (`missing_convention`), so that the loss is not
      silent;
    - a rule keyed `derived:<name>` is the rule of a value the mapping makes from several
      columns (the formula of a species from its composition rows) and belongs to no column;
    - `column` states the attributes of the `dataset_column` the values of this rule fill, for a
      rule whose target is `datum.value`: the observable, the role and what the column is
      presented against; `standard_state = true` in it says the values are of the standard state
      of the segment the column is in, which the mapping attaches.
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
    optional: bool = False
    role: Literal["structure"] | None = None
    disposition: Literal["out_of_scope", "deferred"] | None = None
    reason: str | None = None
    wave: int | None = None
    factor: float | None = None
    default: float | None = None
    case: Literal["capitalize", "upper", "lower"] | None = None
    text: str | None = None
    decode: str | None = None
    equals: Scalar | list[Scalar] | None = None
    text_unit: bool = False
    text_number: str | None = None
    holds: bool = False
    column: dict[str, Scalar | dict[str, Scalar]] | None = None

    @property
    def column_standard_state(self) -> bool:
        return bool(self.column and self.column.get("standard_state") is True)


class Partition(Struct, **_STRICT):
    """The rows of a table matching `where` (every column equals its value, is one of its listed
    values, or, as `{ not = value-or-list }`, is none of them) and what the mapping does with
    them, in place of the table's disposition.
    `derivation` states how the records made from these rows were produced.

    `constants` are canonical values the rows' records take that no column holds, by target
    (`kind.attribute`): a reference to a declared entity by name, an enum member, text, a number,
    or `{ value = ..., unit = "..." }` for a dimensioned value.

    `where` may test a column of a related table, written `<join>.<column>` (the joins of the
    table). `fields` are the rules of the partition's columns: where a column has a rule in both
    the partition and the table, the partition's rule is the one its rows follow."""

    name: str
    where: dict[str, Scalar | list[Scalar] | dict[str, Scalar | list[Scalar]]]
    disposition: Disposition
    reason: str | None = None
    wave: int | None = None
    loss: str | None = None
    origin_role: str | None = None
    constants: dict[str, Scalar | dict[str, Scalar]] = {}
    derivation: DerivationSpec | None = None
    fields: dict[str, FieldRule] = {}


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
    joins: dict[str, Join] = {}

    def fields_of(self, partition: str | None) -> dict[str, FieldRule]:
        """The rules the rows of `partition` follow (the table's own for a row in none): the
        table's, where the partition rules a column (by a rule of its own or of its elements) in
        place of the table's rules of that column."""
        if partition is None:
            return self.fields
        found = next(p for p in self.partitions if p.name == partition)
        replaced = {element_of(key)[0] for key in found.fields}
        kept = {k: r for k, r in self.fields.items() if element_of(k)[0] not in replaced}
        return {**kept, **found.fields}


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
    per_artifact: bool = False
    """One parameterization for each file the sets come from: `{artifact}` in the key and the
    title is the path of the file, and a block names the file (`emit.parameterization(name,
    artifact=...)`)."""


class MappingSpec(Struct, **_STRICT):
    source: str
    doc: str
    tables: dict[str, TableRule]
    scopes: dict[str, ScopeSpec] = {}
    formula_scope: list[FormulaScope] = []
    parameterizations: dict[str, ParameterizationSpec] = {}
    convention_sets: dict[str, dict[str, Scalar | dict[str, Scalar]]] = {}
    energy_references: dict[str, dict[str, Scalar | dict[str, Scalar]]] = {}
    decodings: dict[str, Decoding] = {}


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
            _table(problems, spec, decl, table, rule, staged, roles, schemes)
    for name, scope in spec.scopes.items():
        schema = staged.get(scope.table)
        if schema is None:
            problems.append(f"scope {name}: table `{scope.table}` is not staged")
        elif scope.key not in schema.names:
            problems.append(f"scope {name}: `{scope.key}` is not a column of `{scope.table}`")
        _scope_class(problems, decl, name, scope, schema)
    for item in spec.formula_scope:
        if item.scope not in spec.scopes:
            problems.append(f"formula_scope: `{item.scope}` is not a declared scope")
    for name, attributes in spec.energy_references.items():
        _attributes(
            problems, decl, f"energy_references.{name}", pc.ENERGY_REFERENCE.declared, attributes
        )
    for name, attributes in spec.convention_sets.items():
        _convention_set(problems, decl, spec, name, attributes)
    for name, decoding in spec.decodings.items():
        _decoding(problems, name, decoding)
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
        except KeyError, IndexError, ValueError:
            problems.append(
                f"parameterization {name}: `revision` may use only the placeholder {{pin}}"
            )
        for text, label in ((item.key, "key"), (item.title, "title")):
            try:
                text.format(artifact="", pin="")
            except KeyError, IndexError, ValueError:
                problems.append(
                    f"parameterization {name}: `{label}` may use only the placeholders "
                    "{artifact} and {pin}"
                )
        if item.per_artifact and "{artifact}" not in item.key:
            problems.append(
                f"parameterization {name}: `per_artifact` needs `{{artifact}}` in `key`"
            )
        if not item.per_artifact and "{artifact}" in item.key:
            problems.append(f"parameterization {name}: `{{artifact}}` belongs to `per_artifact`")
    return problems


UNSUPPORTED_CLASSES = ("pseudo_component",)
"""Classes that have no resolution rules yet: a pseudo-component's provisional entity needs the
`kind` that says why it has no formula, which no source statement carries so far; its class rules
come with the wave that needs them."""


def _scope_class(
    problems: list[str], decl: m.Declaration, name: str, scope: ScopeSpec, schema: pa.Schema | None
) -> None:
    """The class declaration of a scope: exactly one of `class` and `class_by`, each class an
    `entity_class` member the resolver has rules for, and `mixture` exactly for a scope that can
    have defined mixtures."""
    where = f"scope {name}"
    members = {member.name for member in decl.enums[pc.ENTITY_CLASS.declared].members}
    if (scope.entity_class is None) == (scope.class_by is None):
        problems.append(
            f"{where}: state exactly one of `class` (the class of every entity) and `class_by` "
            "(a column and a value-to-class table)"
        )
        return
    if scope.class_by is not None:
        by = scope.class_by
        if schema is not None and by.column not in schema.names:
            problems.append(f"{where}: class_by column `{by.column}` is not a column of the table")
        for index, rule in enumerate(by.rules):
            label = f"{where} class_by rule {index + 1}"
            if (rule.pattern is None) == (not rule.values):
                problems.append(f"{label}: state exactly one of `values` and `pattern`")
            if not rule.reason.strip():
                problems.append(f"{label}: states its `reason`")
            if rule.pattern is not None:
                try:
                    re.compile(rule.pattern)
                except re.error as error:
                    problems.append(f"{label}: `pattern` is not a regular expression: {error}")
    for entity_class in sorted(scope.classes()):
        if entity_class not in members:
            problems.append(f"{where}: `{entity_class}` is not an entity class")
        elif entity_class in UNSUPPORTED_CLASSES:
            problems.append(f"{where}: class `{entity_class}` has no resolution rules yet")
    mixtures = pc.ENTITY_CLASS.member("defined_mixture") in scope.classes()
    if mixtures and scope.mixture is None:
        problems.append(
            f"{where}: the scope has defined mixtures, so it states `mixture` (their `definition` "
            "and fraction `basis`)"
        )
    elif not mixtures and scope.mixture is not None:
        problems.append(f"{where}: `mixture` belongs to a scope that has defined mixtures")
    if scope.mixture is not None:
        definitions = {member.name for member in decl.enums[pc.MIXTURE_DEFINITION.declared].members}
        if scope.mixture.definition not in definitions:
            problems.append(
                f"{where}: mixture definition `{scope.mixture.definition}` is not a "
                "`mixture_definition`"
            )
        if not scope.mixture.reason.strip():
            problems.append(f"{where}: `mixture` states its `reason`")


def _decoding(problems: list[str], name: str, decoding: Decoding) -> None:
    """A decoding's rules state their conditions and their reason, and its patterns compile."""
    if not decoding.doc.strip():
        problems.append(f"decodings.{name}: states its `doc`")
    if not decoding.rules:
        problems.append(f"decodings.{name}: has at least one rule")
    for index, line in enumerate(decoding.rules, start=1):
        label = f"decodings.{name} rule {index}"
        if not line.where:
            problems.append(f"{label}: `where` names at least one condition")
        if not line.reason.strip():
            problems.append(f"{label}: states its `reason`")
        for column, wanted in line.where.items():
            if isinstance(wanted, dict):
                if set(wanted) != {"pattern"}:
                    problems.append(
                        f"{label}: `{column}` is a value, a list or `{{ pattern = ... }}`"
                    )
                    continue
                try:
                    re.compile(wanted["pattern"])
                except re.error as error:
                    problems.append(
                        f"{label}: `{column}` pattern is not a regular expression: {error}"
                    )


def _convention_set(
    problems: list[str],
    decl: m.Declaration,
    spec: MappingSpec,
    name: str,
    attributes: dict[str, Scalar | dict[str, Scalar]],
) -> None:
    """One `[convention_sets.<name>]` table states attributes of the `convention_set` kind as
    value rules do: a dimensioned fact (the gas constant) as `{ value, unit }` with a unit that
    converts to its storage unit, an enum member by name, text or a number as itself, and the
    energy reference by the name of an `[energy_references]` entry."""
    kind = pc.CONVENTION_SET.declared
    for attribute, raw in attributes.items():
        where = f"convention_sets.{name}.{attribute}"
        target = resolve_target(decl, f"{kind}.{attribute}")
        if isinstance(target, str):
            problems.append(f"{where}: {target}")
            continue
        type_ = target.field.type
        if attribute == pc.CONVENTION_SET.energy_reference:
            if not isinstance(raw, str) or raw not in spec.energy_references:
                problems.append(f"{where}: `{raw}` is not an [energy_references] entry")
            continue
        if target.dimensioned:
            if not isinstance(raw, dict) or set(raw) != {"value", "unit"}:
                problems.append(
                    f'{where}: state `{{ value = ..., unit = "..." }}`, a dimensioned fact'
                )
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


def _covered(column: str, fields: Mapping[str, FieldRule]) -> bool:
    """Whether `fields` rules `column`: by a rule of its own or by rules of its elements."""
    return column in fields or any(key.startswith(f"{column}[") for key in fields)


def element_of(key: str) -> tuple[str, int | None]:
    """The column and the element index a field-rule key names: `coefficients[3]` is element 3 of
    `coefficients`; a key that is a column name has no index."""
    match = ELEMENT_KEY.fullmatch(key)
    if match is None:
        return key, None
    return match.group(1), int(match.group(2))


def _table(
    problems: list[str],
    spec: MappingSpec,
    decl: m.Declaration,
    table: str,
    rule: TableRule,
    staged: dict[str, pa.Schema],
    roles: set[str],
    schemes: set[str],
) -> None:
    schema = staged[table]
    where = f"table {table}"
    _disposition(problems, where, rule.disposition, rule.reason, rule.wave)
    if rule.disposition == claims.MAPPED and rule.origin_role is None:
        problems.append(f"{where}: a mapped table names the `origin_role` of its records")
    if rule.origin_role is not None and rule.origin_role not in roles:
        problems.append(f"{where}: `{rule.origin_role}` is not an origin role")
    names = set(schema.names)
    _constants(problems, decl, where, rule.constants, rule.fields)
    _derivation(problems, decl, where, rule.derivation)
    joined: dict[str, set[str]] = {}
    for name, join in rule.joins.items():
        label = f"{where} join {name}"
        related = staged.get(join.table)
        if related is None:
            problems.append(f"{label}: table `{join.table}` is not staged")
            continue
        joined[name] = set(related.names)
        if not join.on:
            problems.append(f"{label}: `on` names at least one pair of columns")
        for own, theirs in join.on.items():
            if own not in names:
                problems.append(f"{label}: `{own}` is not a column of `{table}`")
            if theirs not in related.names:
                problems.append(f"{label}: `{theirs}` is not a column of `{join.table}`")
    partition_names: set[str] = set()
    for partition in rule.partitions:
        label = f"{where} partition {partition.name}"
        if partition.name in partition_names:
            problems.append(f"{label}: the name is repeated")
        partition_names.add(partition.name)
        _disposition(problems, label, partition.disposition, partition.reason, partition.wave)
        _constants(problems, decl, label, partition.constants, rule.fields_of(partition.name))
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
        for column, wanted in partition.where.items():
            if isinstance(wanted, dict) and set(wanted) != {"not"}:
                problems.append(f"{label}: `{column}` is a value, a list or `{{ not = ... }}`")
        for column in partition.where:
            join_name, dot, joined_column = column.partition(".")
            if dot:
                if join_name not in joined:
                    problems.append(f"{label}: `{column}` names no join of the table")
                elif joined_column not in joined[join_name]:
                    problems.append(
                        f"{label}: `{joined_column}` is not a column of the table of join "
                        f"`{join_name}`"
                    )
            elif column not in names:
                problems.append(f"{label}: `{column}` is not a column of the table")
        if partition.fields and partition.disposition != claims.MAPPED:
            problems.append(f"{label}: no row is mapped, so it declares no field rules")
    if not maps_rows(rule):
        if rule.fields:
            problems.append(f"{where}: no row is mapped, so it declares no field rules")
        return
    effective: list[tuple[str, dict[str, FieldRule]]] = []
    if rule.disposition == claims.MAPPED:
        effective.append((where, rule.fields))
    effective.extend(
        (f"{where} partition {p.name}", rule.fields_of(p.name))
        for p in rule.partitions
        if p.disposition == claims.MAPPED
    )
    for label, fields in effective:
        for column in sorted(names - set(PROVENANCE_COLUMNS)):
            if not _covered(column, fields):
                problems.append(
                    f"{label}: column `{column}` has neither a rule nor a declared skip"
                )
    checked = [(f"{where} column", rule.fields)] + [
        (f"{where} partition {p.name} column", p.fields) for p in rule.partitions
    ]
    for label, fields in checked:
        for key, field_rule in fields.items():
            if key.startswith(DERIVED):
                _field(
                    problems,
                    decl,
                    spec,
                    f"{label} {key}",
                    rule_table=table,
                    rule=field_rule,
                    field=None,
                    schemes=schemes,
                )
                continue
            column, index = element_of(key)
            if column not in names:
                problems.append(f"{label} {key}: field rule names no column")
                continue
            field = schema.field(column)
            if index is not None and not pa.types.is_list(field.type):
                problems.append(f"{label} {key}: `{column}` holds no list, so it has no elements")
                continue
            if index is None and _covered_by_elements(column, fields):
                problems.append(
                    f"{label} {key}: the column is ruled by its elements, so it has no rule of its own"
                )
            _field(
                problems,
                decl,
                spec,
                f"{label} {key}",
                rule_table=table,
                rule=field_rule,
                field=field,
                schemes=schemes,
            )


def _covered_by_elements(column: str, fields: Mapping[str, FieldRule]) -> bool:
    return any(key.startswith(f"{column}[") for key in fields)


def _constants(
    problems: list[str],
    decl: m.Declaration,
    where: str,
    constants: dict[str, Scalar | dict[str, Scalar]],
    fields: Mapping[str, FieldRule],
) -> None:
    mapped = {f.target for f in fields.values() if f.target is not None}
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
    spec: MappingSpec,
    where: str,
    *,
    rule_table: str,
    rule: FieldRule,
    field: pa.Field | None,
    schemes: set[str],
) -> None:
    kinds = [rule.target is not None, rule.role is not None, rule.disposition is not None]
    if sum(kinds) != 1:
        problems.append(f"{where}: give exactly one of `target`, `role` and `disposition`")
        return
    if rule.optional and rule.target is None:
        problems.append(f"{where}: `optional` belongs to a value rule (one with a `target`)")
    if rule.equals is not None and rule.role is None:
        problems.append(f'{where}: `equals` belongs to a structure column (`role = "structure"`)')
    if rule.role is not None or rule.disposition is not None:
        if rule.disposition is not None:
            _disposition(problems, where, rule.disposition, rule.reason, rule.wave)
        elif not rule.reason:
            problems.append(f"{where}: a structure column states its `reason`")
        for option in ("factor", "default", "case", "decode", "column", "text_number"):
            if getattr(rule, option) is not None:
                problems.append(f"{where}: `{option}` belongs to a value rule")
        if rule.holds and rule.role is not None:
            problems.append(f"{where}: `holds` belongs to a column the mapping does not map")
        return
    if rule.holds:
        problems.append(f"{where}: `holds` belongs to a column the mapping does not map")
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
    _value_options(problems, decl, spec, where, rule_table, rule, target)
    if rule.decode is not None:
        if rule.unit is not None:
            problems.append(
                f"{where}: a decoded value has no unit of its own, so the rule states none"
            )
        return
    if target.dimensioned:
        if rule.text_unit:
            if (rule.unit is None) != (rule.default is None):
                problems.append(
                    f"{where}: the unit is stated in the text (`text_unit`); the rule states a "
                    "`unit` exactly when it states a `default`, which is in that unit"
                )
            elif rule.unit is not None:
                message = _unit_parses(rule.unit)
                if message:
                    problems.append(f"{where}: {message}")
            if field is None or not pa.types.is_string(field.type):
                problems.append(f"{where}: `text_unit` belongs to a text column")
            return
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
        stated = None if field is None else stated_unit(field)
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
    elif rule.text_unit:
        problems.append(f"{where}: `text_unit` belongs to a dimensioned target")
    if rule.offset is not None and target.field.type.text != "Integer":
        problems.append(f"{where}: `offset` applies to an Integer target")


def _value_options(
    problems: list[str],
    decl: m.Declaration,
    spec: MappingSpec,
    where: str,
    table: str,
    rule: FieldRule,
    target: Target,
) -> None:
    """The options of a value rule beyond its target and unit: `default`, `case`, `decode` and
    `column`."""
    type_ = target.field.type
    if rule.default is not None and not rule.loss:
        problems.append(
            f"{where}: a `default` is an assumption of the mapping, so `loss` states it"
        )
    if rule.case is not None and type_.element_kind != "kind":
        problems.append(f"{where}: `case` belongs to a target that names a declared entity")
    if rule.decode is not None:
        found = spec.decodings.get(rule.decode)
        if found is None:
            problems.append(f"{where}: `{rule.decode}` is not a [decodings] entry")
            return
        if type_.element_kind == "enum":
            members = {member.name for member in decl.enums[type_.element].members}
            for line in found.rules:
                if "{" not in line.to and line.to not in members:
                    problems.append(
                        f"{where}: decoding `{rule.decode}` decodes to `{line.to}`, which is not "
                        f"a member of `{type_.element}`"
                    )
        elif type_.element_kind == "kind":
            for line in found.rules:
                if "{" not in line.to and declared_entity(decl, type_.element, line.to) is None:
                    problems.append(
                        f"{where}: decoding `{rule.decode}` decodes to `{line.to}`, which is not "
                        f"a declared entity of kind `{type_.element}`"
                    )
    if rule.text is not None and rule.target != "datum.value":
        problems.append(f"{where}: `text` belongs to a rule whose target is `datum.value`")
    if rule.text_number is not None:
        try:
            if re.compile(rule.text_number).groups != 1:
                problems.append(f"{where}: `text_number` has exactly one group")
        except re.error as error:
            problems.append(f"{where}: `text_number` is not a regular expression: {error}")
        if not target.dimensioned or rule.text_unit:
            problems.append(f"{where}: `text_number` belongs to a dimensioned target with a `unit`")
    if rule.column is not None:
        if rule.target != "datum.value":
            problems.append(f"{where}: `column` belongs to a rule whose target is `datum.value`")
        else:
            _attributes(
                problems,
                decl,
                f"{where} column",
                "dataset_column",
                {k: v for k, v in rule.column.items() if k != "standard_state"},
            )
            if rule.column.get("standard_state", True) is not True:
                problems.append(f"{where} column: `standard_state` is `true` or left out")


def _attributes(
    problems: list[str],
    decl: m.Declaration,
    where: str,
    kind: str,
    attributes: Mapping[str, Scalar | dict[str, Scalar]],
) -> None:
    """Attributes of `kind` a table of the mapping states: an enum member or a declared entity by
    name, text or a number as itself, a dimensioned fact as `{ value, unit }`."""
    for attribute, raw in attributes.items():
        label = f"{where} {attribute}"
        found = resolve_target(decl, f"{kind}.{attribute}")
        if isinstance(found, str):
            problems.append(f"{label}: {found}")
            continue
        type_ = found.field.type
        if isinstance(raw, dict):
            if set(raw) != {"value", "unit"} or not found.dimensioned:
                problems.append(f"{label}: give `{{ value, unit }}` for a dimensioned attribute")
            elif found.unit is not None:
                try:
                    convert(1.0, str(raw["unit"]), found.unit)
                except ValueRefused as error:
                    problems.append(f"{label}: {error}")
            else:
                message = _unit_parses(str(raw["unit"]))
                if message:
                    problems.append(f"{label}: {message}")
        elif found.dimensioned:
            problems.append(f'{label}: state `{{ value = ..., unit = "..." }}`, a dimensioned fact')
        elif type_.element_kind == "enum":
            members = {member.name for member in decl.enums[type_.element].members}
            if raw not in members:
                problems.append(f"{label}: `{raw}` is not a member of `{type_.element}`")
        elif type_.element_kind == "kind":
            if not isinstance(raw, str) or declared_entity(decl, type_.element, raw) is None:
                problems.append(
                    f"{label}: `{raw}` is not a declared entity of kind `{type_.element}`"
                )
