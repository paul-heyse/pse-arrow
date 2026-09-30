# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""What `mapping.py` is given: the context of a run and the emitters of its blocks.

A mapping reads mapped rows from the context and emits inside `with ctx.emit(row, *also) as
emit:` blocks. A block is one unit of work: the records it emits are written together with the
rows `row` and `also` as their origins, and if anything in it is refused (validation, an
ambiguous subject, a value that fits no declared scheme) nothing of it is written and those
rows end `held` with a typed reason (a `held_reason` member) and its detail. The context tracks
each row's outcome, which the coverage report is computed from, and counts how many mapped rows
each value rule was applied to (rows that produced a value in a block that wrote records), which
the run checks after phase 2; it never decides a state itself.

Units, constants and defaults are not given to a mapping: `quantity` returns a value with the
unit `mapping.toml` declares for its column, and `value` a value with the rule's absent
markers removed.

Phase 1 (`IdentityContext`) emits source entities and identity assertions; phase 2
(`RecordContext`) emits everything else, referring to subjects by source entity.
"""

from __future__ import annotations

import re
import uuid
from collections.abc import Iterator, Mapping, Sequence
from contextlib import AbstractContextManager, contextmanager, nullcontext
from dataclasses import dataclass, field

from thermo_knowledge import pipeline_contract as pc
from thermo_knowledge.canonical.provenance import CarrierInfo, Origin, SourceRef
from thermo_knowledge.canonical.values import Quantity, ValueRefused, convert
from thermo_knowledge.canonical.writer import (
    CanonicalWriter,
    FamilyRow,
    NestedSet,
    ValidationError,
)
from thermo_knowledge.declaration import model as m
from thermo_knowledge.mapping import claims
from thermo_knowledge.mapping.spec import (
    DerivationSpec,
    FieldRule,
    MappingSpec,
    ParameterizationSpec,
    Scalar,
    declared_entity,
    resolve_target,
)
from thermo_knowledge.mapping.staged import (
    Classifier,
    MappingError,
    SourceRow,
    StagedTables,
)

MAPPED = claims.MAPPED


class RowHeld(Exception):
    """The rows of the block cannot be loaded: `reason` is the `held_reason` member recorded for
    them and the message its detail."""

    def __init__(self, reason: str, detail: str) -> None:
        if reason not in pc.HELD_REASON.members:
            raise ValueError(f"`{reason}` is not a held reason: {', '.join(pc.HELD_REASON.members)}")
        self.reason = reason
        super().__init__(detail)


class SubjectHeld(RowHeld):
    """A subject is unknown or ambiguous."""


VALIDATION_FAILED = pc.HELD_REASON.member("validation_failed")


@dataclass
class Outcome:
    """How one source row fared: emitted (with a loss or not) or held, with the held reason and
    its detail."""

    state: str
    lossy: bool
    reason: str | None = None
    detail: str | None = None


@dataclass
class _Block:
    rows: tuple[SourceRow, ...]
    lossy: set[str] = field(default_factory=set)
    applied: set[tuple[str, str, str]] = field(default_factory=set)
    """(table, column, locator) of each value rule application of the block."""
    count: int = 0


class RunContext[E]:
    """The state both phases share: the rules, the staged tables and the row outcomes."""

    def __init__(
        self,
        *,
        source_id: str,
        spec: MappingSpec,
        decl: m.Declaration,
        tables: StagedTables,
        carrier: CarrierInfo,
    ) -> None:
        self.source_id = source_id
        self.spec = spec
        self.decl = decl
        self.tables = tables
        self.carrier = carrier
        self.classifier = Classifier(tables, spec.tables)
        self.outcomes: dict[tuple[str, str], Outcome] = {}
        self.absent: dict[tuple[str, str, str], int] = {}
        self._applied: dict[tuple[str, str], set[str]] = {}
        self._block: _Block | None = None

    # -- reading ------------------------------------------------------------------------------

    def rows(self, table: str, partition: str | None = None) -> Iterator[SourceRow]:
        """The rows of `table` that the mapping maps (those of one partition when named)."""
        self._require_table(table)
        return self.classifier.mapped(table, partition)

    def group(
        self, table: str, *columns: str, partition: str | None = None
    ) -> dict[tuple[object, ...], list[SourceRow]]:
        """Mapped rows of `table` grouped by the values of `columns`, in table order."""
        groups: dict[tuple[object, ...], list[SourceRow]] = {}
        for row in self.rows(table, partition):
            groups.setdefault(tuple(row[c] for c in columns), []).append(row)
        return groups

    def _require_table(self, table: str) -> None:
        if table not in self.spec.tables:
            raise MappingError(f"`{table}` is not a staged table of {self.source_id}")

    def rule(self, table: str, column: str) -> FieldRule:
        found = self.spec.tables[table].fields.get(column)
        if found is None:
            raise MappingError(f"table {table}: column `{column}` has no field rule")
        return found

    def _use(self, row: SourceRow, column: str, rule: FieldRule) -> None:
        """Note that `rule`, the rule of `column`, produced a value for `row` in the open block."""
        if self._block is None:
            return
        self._block.applied.add((row.table, column, row.locator))
        if rule.loss:
            self._block.lossy.add(row.locator)

    def rule_use(self) -> dict[tuple[str, str], int]:
        """The mapped rows each value rule was applied to, by (table, column): rows for which it
        produced a value in a block that wrote records."""
        return {key: len(rows) for key, rows in sorted(self._applied.items())}

    def _is_absent(self, table: str, column: str, rule: FieldRule, raw: object) -> bool:
        if raw is None or raw == "":
            return True
        if any(
            raw == marker and isinstance(raw, bool) == isinstance(marker, bool)
            for marker in rule.absent
        ):
            key = (table, column, str(raw))
            self.absent[key] = self.absent.get(key, 0) + 1
            return True
        return False

    def value(self, row: SourceRow, column: str) -> Scalar | None:
        """The value of `column`, or `None` when the source has none or writes a marker
        `mapping.toml` declares absent."""
        rule = self.rule(row.table, column)
        if rule.target is None:
            raise MappingError(f"table {row.table}: column `{column}` has no target")
        raw = row[column]
        if self._is_absent(row.table, column, rule, raw):
            return None
        self._use(row, column, rule)
        return raw  # type: ignore[return-value]

    def quantity(self, row: SourceRow, column: str) -> Quantity | None:
        """The value of `column` with the unit its rule declares, or `None` when absent."""
        rule = self.rule(row.table, column)
        if rule.unit is None:
            raise MappingError(f"table {row.table}: column `{column}` declares no unit")
        raw = row[column]
        if self._is_absent(row.table, column, rule, raw):
            return None
        if isinstance(raw, bool) or not isinstance(raw, (int, float)):
            raise RowHeld("not_a_number", f"{row.table}.{column}: {raw!r} is not a number")
        self._use(row, column, rule)
        return Quantity(float(raw), rule.unit)

    def integer(self, row: SourceRow, column: str) -> int | None:
        """The value of an integer `column` with the offset its rule declares added (a family
        index that the source counts from zero), or `None` when absent."""
        rule = self.rule(row.table, column)
        raw = row[column]
        if self._is_absent(row.table, column, rule, raw):
            return None
        if isinstance(raw, bool) or not isinstance(raw, int):
            raise RowHeld("not_an_integer", f"{row.table}.{column}: {raw!r} is not an integer")
        self._use(row, column, rule)
        return raw + (rule.offset or 0)

    def _column_value(self, row: SourceRow, column: str, rule: FieldRule) -> object | None:
        if rule.unit is not None:
            return self.quantity(row, column)
        if rule.offset is not None:
            return self.integer(row, column)
        return self.value(row, column)

    def _targets(self, row: SourceRow, prefix: str) -> Iterator[tuple[str, str, FieldRule]]:
        """(column, name, rule) of every column of `row`'s table that maps to `prefix.name`."""
        for column, rule in self.spec.tables[row.table].fields.items():
            if rule.target is not None and rule.target.rpartition(".")[0] == prefix:
                yield column, rule.target.rpartition(".")[2], rule

    def slot_values(self, row: SourceRow, group: str) -> dict[str, object]:
        """The slot values of `group` (`form.group`) that `row`'s columns hold, by slot name.
        A slot whose column is absent in the row is left out."""
        values: dict[str, object] = {}
        for column, name, rule in self._targets(row, group):
            found = self._column_value(row, column, rule)
            if found is not None:
                values[name] = found
        return values

    def family_row(self, row: SourceRow, family: str) -> FamilyRow:
        """One row of `family` (`form.group.family`) from `row`'s columns: their indices and
        slot values. A value absent in the row is left out."""
        declared = next((f for f in self.decl.families if f.qualified == family), None)
        if declared is None:
            raise MappingError(f"`{family}` is not a family (`form.group.family`)")
        indices = {index.name for index in declared.indices}
        index: dict[str, object] = {}
        values: dict[str, object] = {}
        for column, name, rule in self._targets(row, family):
            found = self._column_value(row, column, rule)
            if found is not None:
                (index if name in indices else values)[name] = found
        return FamilyRow(index, values)

    def attributes(self, row: SourceRow, kind: str) -> dict[str, object]:
        """The attributes of `kind` that `row`'s columns hold, and the constants declared for
        the row's table or partition, by attribute name."""
        values: dict[str, object] = {}
        for column, name, rule in self._targets(row, kind):
            found = self._column_value(row, column, rule)
            if found is not None:
                values[name] = found
        for text, raw in self._constants(row).items():
            owner, _, name = text.rpartition(".")
            if owner == kind:
                values[name] = self._constant(text, raw)
        return values

    def _constants(self, row: SourceRow) -> dict[str, Scalar | dict[str, Scalar]]:
        rule = self.spec.tables[row.table]
        constants = dict(rule.constants)
        name = self.classifier.classify(row.table)[row.locator].partition
        for partition in rule.partitions:
            if partition.name == name:
                constants.update(partition.constants)
        return constants

    def _constant(self, target: str, raw: Scalar | dict[str, Scalar]) -> object:
        if isinstance(raw, dict):
            return Quantity(float(raw["value"]), str(raw["unit"]))  # type: ignore[arg-type]
        resolved = resolve_target(self.decl, target)
        assert not isinstance(resolved, str)
        type_ = resolved.field.type
        if type_.element_kind == "kind":
            entity = declared_entity(self.decl, type_.element, str(raw))
            assert entity is not None  # validated
            return entity.id
        return raw

    # -- blocks -------------------------------------------------------------------------------

    def _unit(self) -> AbstractContextManager[None]:
        return nullcontext()

    def _emitter(self, block: _Block) -> E:
        raise NotImplementedError

    @contextmanager
    def emit(self, row: SourceRow, *also: SourceRow) -> Iterator[E]:
        """One unit of work whose records come from `row` (and `also`)."""
        if self._block is not None:
            raise MappingError("emit blocks do not nest")
        block = _Block((row, *also))
        for item in block.rows:
            disposition = self.classifier.classify(item.table)[item.locator]
            if disposition.disposition != MAPPED:
                raise MappingError(
                    f"{item.locator} is {disposition.disposition}, not mapped: a mapping emits "
                    "only from rows it maps"
                )
        self._block = block
        try:
            with self._unit():
                yield self._emitter(block)
        except RowHeld as held:
            self._hold(block, held.reason, str(held))
        except ValidationError as held:
            self._hold(block, VALIDATION_FAILED, str(held))
        else:
            if block.count:
                self._commit(block)
        finally:
            self._block = None

    def _hold(self, block: _Block, reason: str, detail: str) -> None:
        for row in block.rows:
            key = (row.table, row.locator)
            existing = self.outcomes.get(key)
            if existing is None or existing.state != claims.HELD:
                self.outcomes[key] = Outcome(claims.HELD, False, reason, detail)

    def _commit(self, block: _Block) -> None:
        for table, column, locator in block.applied:
            self._applied.setdefault((table, column), set()).add(locator)
        for row in block.rows:
            key = (row.table, row.locator)
            declared = self.classifier.classify(row.table)[row.locator].loss is not None
            lossy = declared or row.locator in block.lossy
            existing = self.outcomes.get(key)
            if existing is None:
                self.outcomes[key] = Outcome(claims.EMITTED, lossy)
            elif existing.state == claims.EMITTED:
                existing.lossy = existing.lossy or lossy

    def derivation_of(self, row: SourceRow) -> DerivationSpec | None:
        """The derivation declared for the partition or table `row` is in."""
        rule = self.spec.tables[row.table]
        name = self.classifier.classify(row.table)[row.locator].partition
        for partition in rule.partitions:
            if partition.name == name and partition.derivation is not None:
                return partition.derivation
        return rule.derivation

    def role_of(self, row: SourceRow) -> str:
        """The origin role of the records produced from `row`."""
        role = self.classifier.classify(row.table)[row.locator].origin_role
        assert role is not None  # validated: a mapped table or partition names it
        return role

    def origins(self, block: _Block) -> tuple[Origin, ...]:
        return tuple(
            Origin(SourceRef(self.source_id, row.artifact, row.locator), self.role_of(row))
            for row in block.rows
        )


# -- phase 1 -----------------------------------------------------------------------------------


@dataclass(frozen=True)
class EntityHandle:
    """A source entity declared in the current block."""

    scope: str
    local_key: str


class IdentityContext(RunContext["IdentityEmitter"]):
    """Phase 1: source entities and identity assertions."""

    def __init__(self, **kwargs: object) -> None:
        super().__init__(**kwargs)  # type: ignore[arg-type]
        self.entities: list[claims.EntityClaim] = []
        self.assertions: list[claims.AssertionClaim] = []
        self.components: list[claims.ComponentClaim] = []
        self._seen: set[tuple[object, ...]] = set()
        self._declared: dict[tuple[str, str], claims.EntityClaim] = {}

    @contextmanager
    def _unit(self) -> Iterator[None]:
        marks = (
            len(self.entities),
            len(self.assertions),
            len(self.components),
            set(self._seen),
            dict(self._declared),
        )
        try:
            yield
        except BaseException:
            del self.entities[marks[0] :]
            del self.assertions[marks[1] :]
            del self.components[marks[2] :]
            self._seen = marks[3]
            self._declared = marks[4]
            raise

    def entity(self, scope: str, key: str) -> EntityHandle:
        """The source entity (`scope`, `key`) an earlier block of this run declared, to cite it (as
        a component of a mixture). An entity not declared holds the rows of the block."""
        if (scope, key) not in self._declared:
            raise SubjectHeld(
                "unknown_subject",
                f"the source entity ({scope}, {key!r}) of {self.source_id} is not declared by "
                "this run",
            )
        return EntityHandle(scope, key)

    def _emitter(self, block: _Block) -> IdentityEmitter:
        return IdentityEmitter(self, block)


class IdentityEmitter:
    """What a phase-1 block emits: one source entity and its assertions."""

    def __init__(self, ctx: IdentityContext, block: _Block) -> None:
        self._ctx = ctx
        self._block = block
        self._row = block.rows[0]

    def source_entity(
        self,
        scope: str,
        *,
        aggregation: str | None = None,
        polymorph: str | None = None,
        stated_charge: int | None = None,
    ) -> EntityHandle:
        """Declare the source entity the block's first row is, in `scope` (the scope names the
        table and the key column in `mapping.toml`)."""
        ctx = self._ctx
        spec = ctx.spec.scopes.get(scope)
        if spec is None:
            raise MappingError(f"`{scope}` is not a declared scope")
        if spec.table != self._row.table:
            raise MappingError(
                f"scope `{scope}` is keyed by table `{spec.table}`, not `{self._row.table}`"
            )
        key = self._row[spec.key]
        if not isinstance(key, str) or key == "":
            raise RowHeld("unusable_key", f"{self._row.table}.{spec.key}: {key!r} is not a usable key")
        entity_class = spec.class_of(
            None if spec.class_by is None else self._row[spec.class_by.column]
        )
        form = pc.ENTITY_CLASS.member("species_form")
        if (entity_class == form) != (aggregation is not None):
            raise MappingError(
                f"scope `{scope}`: the entity {key!r} is of class `{entity_class}`, and only an "
                "entity of class `species_form` states an aggregation (and one of that class "
                "states it)"
            )
        if polymorph is not None and entity_class != form:
            raise MappingError(f"scope `{scope}`: only a species form states a polymorph")
        mixture = spec.mixture if entity_class == pc.ENTITY_CLASS.member("defined_mixture") else None
        claim = claims.EntityClaim(
            ctx.source_id,
            scope,
            key,
            entity_class,
            aggregation,
            polymorph,
            stated_charge,
            None if mixture is None else mixture.definition,
            None if mixture is None else mixture.basis == "mole",
            ctx.role_of(self._row),
            self._row.artifact,
            self._row.locator,
        )
        marker = ("entity", claim.key, claim.locator)
        if marker not in ctx._seen:
            ctx._seen.add(marker)
            ctx.entities.append(claim)
        ctx._declared.setdefault((scope, key), claim)
        self._block.count += 1
        return EntityHandle(scope, key)

    def component(
        self, mixture: EntityHandle, component: EntityHandle, row: SourceRow, column: str
    ) -> None:
        """Claim that `component` is a component of the defined mixture `mixture`, with the
        fraction `column` of `row` holds on the basis the scope states, converted from the rule's
        unit and written as exact decimal text. Both entities are declared by this run; a
        mixture is of class `defined_mixture` or the block is refused with a `MappingError`."""
        ctx = self._ctx
        declared = ctx._declared.get((mixture.scope, mixture.local_key))
        if declared is None or declared.entity_class != pc.ENTITY_CLASS.member("defined_mixture"):
            raise MappingError(
                f"({mixture.scope}, {mixture.local_key!r}) is not a declared defined mixture"
            )
        rule = ctx.rule(row.table, column)
        target = pc.MIXTURE_COMPONENT
        if rule.target != f"{target.declared}.{target.value}" or rule.unit is None:
            raise MappingError(
                f"table {row.table}: column `{column}` is not a mixture fraction with a unit"
            )
        raw = row[column]
        if ctx._is_absent(row.table, column, rule, raw):
            raise RowHeld("not_a_number", f"{row.table}.{column}: the component has no fraction")
        if isinstance(raw, bool) or not isinstance(raw, (int, float)):
            raise RowHeld("not_a_number", f"{row.table}.{column}: {raw!r} is not a number")
        ctx._use(row, column, rule)
        try:
            fraction = claims.decimal_text(convert(float(raw), rule.unit, m.DIMENSIONLESS))
        except ValueRefused as error:
            raise RowHeld("unit_not_parseable", f"{row.table}.{column}: {error}") from error
        claim = claims.ComponentClaim(
            ctx.source_id,
            mixture.scope,
            mixture.local_key,
            component.scope,
            component.local_key,
            fraction,
            ctx.role_of(row),
            row.artifact,
            row.locator,
        )
        marker = ("component", claim.mixture, claim.component, fraction, row.locator)
        if marker not in ctx._seen:
            ctx._seen.add(marker)
            ctx.components.append(claim)
        self._block.count += 1

    def assertion(self, entity: EntityHandle, column: str, value: object | None = None) -> bool:
        """Assert the value of `column` (or `value`, when a column holds several, such as a list
        of aliases) as an identifier of `entity`, under the scheme its rule declares. Returns
        whether an assertion was made: a value the source does not have, or writes as a
        declared absent marker, yields none."""
        ctx, row = self._ctx, self._row
        rule = ctx.rule(row.table, column)
        assertion = pc.IDENTITY_ASSERTION
        if rule.target != f"{assertion.declared}.{assertion.value}":
            raise MappingError(
                f"table {row.table}: column `{column}` does not map to an identity assertion"
            )
        raw = row[column] if value is None else value
        if ctx._is_absent(row.table, column, rule, raw):
            return False
        text = str(raw)
        scheme = rule.scheme
        assert scheme is not None  # validated
        if rule.pattern is not None and not re.fullmatch(rule.pattern, text):
            if rule.otherwise_scheme is None:
                raise RowHeld(
                    "pattern_mismatch",
                    f"{row.table}.{column}: {text!r} does not match `{rule.pattern}`",
                )
            scheme = rule.otherwise_scheme
        ctx._use(row, column, rule)
        claim = claims.AssertionClaim(
            ctx.source_id,
            entity.scope,
            entity.local_key,
            scheme,
            text,
            column,
            row.artifact,
            row.locator,
        )
        marker = ("assertion", claim.entity, scheme, text, row.locator)
        if marker not in ctx._seen:
            ctx._seen.add(marker)
            ctx.assertions.append(claim)
        self._block.count += 1
        return True


# -- phase 2 -----------------------------------------------------------------------------------


@dataclass(frozen=True)
class SubjectInfo:
    """What resolution decided about one source entity."""

    target: uuid.UUID
    status: str
    candidates: tuple[str, ...] = ()


class RecordContext(RunContext["RecordEmitter"]):
    """Phase 2: every record but source entities and identity assertions."""

    def __init__(
        self,
        *,
        writer: CanonicalWriter,
        subjects: Mapping[tuple[str, str], SubjectInfo],
        **kwargs: object,
    ) -> None:
        super().__init__(**kwargs)  # type: ignore[arg-type]
        self.writer = writer
        self._subjects = subjects
        self.parameterizations: dict[uuid.UUID, ParameterizationSpec] = {}
        """The parameterizations this run created, by identifier, with their declared spec."""

    def subject(self, scope: str, key: str) -> uuid.UUID:
        """The material entity the source entity (`scope`, `key`) of this carrier resolved to.

        An unknown or ambiguous source entity holds the rows of the block."""
        info = self._subjects.get((scope, key))
        if info is None:
            raise SubjectHeld(
                "unknown_subject",
                f"no source entity ({scope}, {key!r}) of {self.source_id} was resolved",
            )
        if info.status == pc.RESOLUTION_STATUS.member("ambiguous"):
            raise SubjectHeld(
                "ambiguous_subject",
                f"the source entity ({scope}, {key!r}) is ambiguous; candidates: "
                f"{', '.join(info.candidates) or '(none)'}",
            )
        return info.target

    def _unit(self) -> AbstractContextManager[None]:
        return self.writer.transaction()

    def _emitter(self, block: _Block) -> RecordEmitter:
        return RecordEmitter(self, block)


class RecordEmitter:
    """What a phase-2 block emits: records whose origins are the block's rows."""

    def __init__(self, ctx: RecordContext, block: _Block) -> None:
        self._ctx = ctx
        self._block = block
        self._origins = ctx.origins(block)
        self.role = self._origins[0].role

    @property
    def locator(self) -> str:
        return self._origins[0].ref.locator

    def parameter_set(
        self,
        *,
        parameterization: uuid.UUID,
        slot_group: str,
        subjects: Sequence[uuid.UUID],
        slots: Mapping[str, object],
        families: Mapping[str, Sequence[FamilyRow]] | None = None,
        occurrence: int | None = None,
    ) -> uuid.UUID:
        """One parameter set of `slot_group` (`form.group`) for `subjects`, with the block's rows
        as its origins (and so their role). `occurrence` numbers repeated assertions for one
        subject within a parameterization, from one in source order; one when not given.

        A form that declares the convention facts it reads needs the parameterization's
        convention set to state them: a mapping that emits such a set without stating them is
        refused (`MappingError`, which ends the run, so nothing is written)."""
        self._require_conventions(parameterization, slot_group, slots, families)
        found = self._ctx.writer.parameter_set(
            parameterization=parameterization,
            slot_group=slot_group,
            subjects=subjects,
            slots=slots,
            families=families,
            origins=self._origins,
            occurrence=occurrence,
        )
        self._block.count += 1
        return found

    def _require_conventions(
        self,
        parameterization: uuid.UUID,
        slot_group: str,
        slots: Mapping[str, object],
        families: Mapping[str, Sequence[FamilyRow]] | None,
    ) -> None:
        """Refuse a set of a form that reads a convention fact (itself, or a nested set it holds)
        when the parameterization's convention set in `mapping.toml` does not state it."""
        ctx = self._ctx
        spec = ctx.parameterizations.get(parameterization)
        stated: set[str] = set()
        if spec is not None and spec.convention_set is not None:
            stated = set(ctx.spec.convention_sets[spec.convention_set])
        groups = {group.qualified: group for group in ctx.decl.slot_groups}
        pending: list[tuple[str, Mapping[str, object], Mapping[str, Sequence[FamilyRow]]]] = [
            (slot_group, slots, families or {})
        ]
        while pending:
            name, held, rows = pending.pop()
            group = groups.get(name)
            if group is None:
                continue  # the writer refuses an unknown slot group, naming it
            for convention in ctx.decl.forms[group.form].conventions:
                if convention.name in stated:
                    continue
                if spec is None:
                    where = "the parameterization is not one of mapping.toml's"
                elif spec.convention_set is None:
                    where = f"parameterization `{spec.key}` names no convention set"
                else:
                    where = (
                        f"the convention set `{spec.convention_set}` of parameterization "
                        f"`{spec.key}` does not state it"
                    )
                raise MappingError(
                    f"form `{group.form}` reads the convention fact `{convention.name}`, and "
                    f"{where}: state `{convention.name}` in a [convention_sets] entry of "
                    "mapping.toml"
                )
            values = [*held.values()]
            values += [
                value for family_rows in rows.values() for row in family_rows for value in row.values.values()
            ]
            pending.extend(
                (value.slot_group, value.slots, value.families or {})
                for value in values
                if isinstance(value, NestedSet)
            )

    def kind(self, name: str, values: Mapping[str, object]) -> uuid.UUID:
        """One instance of kind `name`; a kind with its own record takes the block's origins."""
        return self._kind(name, values, self._origins)

    def _kind(
        self, name: str, values: Mapping[str, object], origins: tuple[Origin, ...]
    ) -> uuid.UUID:
        writer = self._ctx.writer
        own = (
            name in writer.decl.kinds
            and writer.decl.kinds[writer.decl.kinds[name].root].provenance.mode == "own"
        )
        found = writer.kind(name, values, origins=origins if own else (), at=self.locator)
        self._block.count += 1
        return found

    def relation(
        self, name: str, keys: Mapping[str, object], values: Mapping[str, object] | None = None
    ) -> uuid.UUID:
        """One row of relation `name`; a relation with its own record takes the block's origins."""
        writer = self._ctx.writer
        relation = writer.decl.relations.get(name)
        own = relation is not None and relation.provenance.mode == "own"
        found = writer.relation(
            name, keys, values, origins=self._origins if own else (), at=self.locator
        )
        self._block.count += 1
        return found

    def derivation(self, outputs: Sequence[uuid.UUID]) -> uuid.UUID:
        """The derivation declared for the block's first row (a `fit` when it is one), with a
        `derivation_output` row for each record in `outputs`. It takes no input: the records a
        derivation consumed are linked only when the mapping can name them."""
        ctx = self._ctx
        row = self._block.rows[0]
        spec = ctx.derivation_of(row)
        if spec is None:
            raise MappingError(f"the rows of {row.table} declare no derivation in mapping.toml")
        if not outputs:
            raise MappingError("a derivation produces at least one record")
        values: dict[str, object] = {
            pc.DERIVATION.key: f"{ctx.source_id}:{outputs[0]}",
            pc.DERIVATION.kind: spec.kind,
            pc.DERIVATION.method: spec.method,
        }
        name = pc.DERIVATION.declared
        if spec.kind == pc.DERIVATION_KIND.member("fit"):
            name, values[pc.FIT.outcome] = pc.FIT.declared, spec.outcome
        found = self.kind(name, values)
        for record in outputs:
            output = pc.DERIVATION_OUTPUT
            self.relation(output.declared, {output.derivation: found, output.record: record})
        return found

    def parameterization(self, name: str) -> uuid.UUID:
        """The parameterization `name` of `mapping.toml` (and its convention set), with this
        block's rows among its origins, in the origin role `mapping.toml` gives the
        parameterization and not the role of the block that happens to emit it."""
        ctx = self._ctx
        spec = ctx.spec.parameterizations.get(name)
        if spec is None:
            raise MappingError(f"`{name}` is not a declared parameterization")
        origins = tuple(Origin(origin.ref, spec.origin_role) for origin in self._origins)
        convention: uuid.UUID | None = None
        if spec.convention_set is not None:
            attributes = {
                attribute: (
                    Quantity(float(raw["value"]), str(raw["unit"]))  # type: ignore[arg-type]
                    if isinstance(raw, dict)
                    else raw
                )
                for attribute, raw in ctx.spec.convention_sets[spec.convention_set].items()
            }
            convention = self._kind(pc.CONVENTION_SET.declared, attributes, origins)
        pz = pc.PARAMETERIZATION
        found = self._kind(
            pz.declared,
            {
                pz.key: spec.key,
                pz.revision: spec.revision.format(pin=ctx.carrier.pin),
                pz.title: spec.title,
                pz.coherence: spec.coherence,
                pz.convention_set: convention,
            },
            origins,
        )
        ctx.parameterizations[found] = spec
        return found
