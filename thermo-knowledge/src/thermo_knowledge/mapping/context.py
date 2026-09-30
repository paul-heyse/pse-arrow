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

from thermo_knowledge import identity
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
    Decoding,
    DerivationSpec,
    FieldRule,
    MappingSpec,
    ParameterizationSpec,
    Scalar,
    Target,
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


def _satisfies(
    where: Mapping[str, object], values: Mapping[str, object]
) -> dict[str, str] | None:
    """The named groups of the patterns of `where` when `values` satisfy every condition (the
    empty mapping for conditions with no group), else `None`."""
    groups: dict[str, str] = {}
    for column, wanted in where.items():
        found = values.get(column)
        if isinstance(wanted, dict):
            if not isinstance(found, str):
                return None
            match = re.fullmatch(wanted["pattern"], found)
            if match is None:
                return None
            groups.update({k: v for k, v in match.groupdict().items() if v is not None})
        elif isinstance(wanted, list):
            if found not in wanted:
                return None
        elif found != wanted or isinstance(found, bool) != isinstance(wanted, bool):
            return None
    return groups


def _decoding_columns(decoding: Decoding) -> list[str]:
    return sorted({column for line in decoding.rules for column in line.where})


STANDARD_STATE = "standard_state"
"""The key of a rule's `column` table that says the column is of the standard state."""
def numbered[T](items: Sequence[T]) -> Iterator[tuple[int, T]]:
    """The items with their numbers, counting from one, as a declaration numbers ordinals, indices
    and pieces."""
    return enumerate(items, start=ORIGIN)


ORIGIN = 1
"""The number of the first item of anything a declaration numbers."""
SINGLE = 1
"""How many of a thing there is when there is one of it."""


def first[T](items: Sequence[T]) -> T:
    """The first of `items`."""
    return items[ORIGIN - 1]


def last[T](items: Sequence[T]) -> T:
    """The last of `items`."""
    return items[-ORIGIN]

PLAIN_NUMBER = re.compile(r"[+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?")
NUMBER_WITH_UNIT = re.compile(r"\s*([+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?)\s+(\S.*?)\s*")


@dataclass(frozen=True)
class Decoded:
    """What a decoding made of a row: the value it decodes to (a declared entity or an enum
    member, by name), the polymorph its rule states, if any, and the rule's reason."""

    to: str
    polymorph: str | None
    reason: str
    key: str | None = None
    groups: Mapping[str, str] = field(default_factory=dict)
    """The named groups of the patterns the row satisfied."""


def _structure_held(ctx: "RunContext[object]", rows: Sequence[SourceRow]) -> None:
    """Hold the block when a row's structure is not the one the mapping relies on: a column whose
    rule states the exact value (`equals`) holds another."""
    for row in rows:
        for column, rule in ctx.fields(row).items():
            if rule.equals is None and not rule.holds:
                continue
            found = row[column]
            if isinstance(found, list):
                found = list(found)
            if rule.holds and found is not None and found != "":
                raise RowHeld(
                    "missing_convention",
                    f"{row.locator}: `{column}` states {found!r}, which the mapping does not "
                    f"map yet ({rule.reason})",
                )
            if rule.equals is not None and found != rule.equals:
                raise RowHeld(
                    "pattern_mismatch",
                    f"{row.locator}: `{column}` is {found!r}, the mapping relies on "
                    f"{rule.equals!r}",
                )


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
        self._resolved: dict[str, Target] = {}

    # -- reading ------------------------------------------------------------------------------

    def rows(self, table: str, partition: str | None = None) -> Iterator[SourceRow]:
        """The rows of `table` that the mapping maps (those of one partition when named)."""
        self._require_table(table)
        return self.classifier.mapped(table, partition)

    def source_rows(self, table: str) -> list[SourceRow]:
        """Every row of `table`, whatever the mapping does with it: a read to look a fact up (the
        phases that include a species), never a source of records. A row read this way keeps the
        state its disposition gives it."""
        self._require_table(table)
        return self.tables.rows(table)

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

    def _partition(self, row: SourceRow) -> str | None:
        return self.classifier.classify(row.table)[row.locator].partition

    def fields(self, row: SourceRow) -> dict[str, FieldRule]:
        """The rules the row's columns follow: the table's, overridden by its partition's."""
        return self.spec.tables[row.table].fields_of(self._partition(row))

    def rule(self, row: SourceRow, column: str) -> FieldRule:
        found = self.fields(row).get(column)
        if found is None:
            raise MappingError(f"table {row.table}: column `{column}` has no field rule")
        return found

    def _label(self, row: SourceRow, column: str) -> str:
        """How rule use names the rule of `column`: the column, or, for a rule the row's partition
        gives, `partition:<name>:<column>`."""
        name = self._partition(row)
        if name is not None:
            partition = next(p for p in self.spec.tables[row.table].partitions if p.name == name)
            if column in partition.fields:
                return f"partition:{name}:{column}"
        return column

    def _use(self, row: SourceRow, column: str, rule: FieldRule) -> None:
        """Note that `rule`, the rule of `column`, produced a value for `row` in the open block."""
        if self._block is None:
            return
        self._block.applied.add((row.table, self._label(row, column), row.locator))
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

    def _target(self, rule: FieldRule) -> Target:
        assert rule.target is not None
        found = self._resolved.get(rule.target)
        if found is None:
            resolved = resolve_target(self.decl, rule.target)
            assert not isinstance(resolved, str)  # validated
            found = self._resolved[rule.target] = resolved
        return found

    def _raw(self, row: SourceRow, column: str, rule: FieldRule) -> object | None:
        """The source value of `column`, the rule's documented default where the source has
        none, or `None` (absent, or written as a marker the rule declares absent)."""
        raw = row[column]
        if self._is_absent(row.table, column, rule, raw):
            return rule.default
        return raw

    def value(self, row: SourceRow, column: str) -> Scalar | uuid.UUID | None:
        """The value of `column`, or `None` when the source has none or writes a marker
        `mapping.toml` declares absent. A number is multiplied by the rule's `factor`; text that
        names a declared entity (a rule whose target is a reference to a kind whose instances the
        declaration holds) is that entity's identifier, after the rule's `case`."""
        rule = self.rule(row, column)
        if rule.target is None:
            raise MappingError(f"table {row.table}: column `{column}` has no target")
        raw = self._raw(row, column, rule)
        if raw is None:
            return None
        self._use(row, column, rule)
        type_ = self._target(rule).field.type
        if type_.element_kind == "kind" and isinstance(raw, str):
            return self._entity(row, column, rule, type_.element, raw)
        if rule.factor is not None and isinstance(raw, (int, float)) and not isinstance(raw, bool):
            return raw * rule.factor
        return raw  # type: ignore[return-value]

    def _entity(
        self, row: SourceRow, column: str, rule: FieldRule, kind: str, name: str
    ) -> uuid.UUID:
        if rule.case is not None:
            name = getattr(name, rule.case)()
        entity = declared_entity(self.decl, kind, name)
        if entity is None:
            raise RowHeld(
                "unknown_subject",
                f"{row.table}.{column}: `{name}` is not a declared {kind}",
            )
        return entity.id

    def name(self, row: SourceRow, column: str) -> str:
        """The declared entity the text of `column` names, as its declared name (after the
        rule's `case`): the element an element symbol of the source stands for. A text that names
        none holds the block (`unknown_subject`)."""
        rule = self.rule(row, column)
        raw = row[column]
        if not isinstance(raw, str) or self._is_absent(row.table, column, rule, raw):
            raise RowHeld("unusable_key", f"{row.table}.{column}: {raw!r} names nothing")
        kind = self._target(rule).field.type.element
        self._entity(row, column, rule, kind, raw)
        self._use(row, column, rule)
        return getattr(raw, rule.case)() if rule.case is not None else raw

    def quantity(self, row: SourceRow, column: str) -> Quantity | None:
        """The value of `column` with the unit its rule declares (or, for a text column that
        states its unit, the unit the text states), or `None` when absent."""
        rule = self.rule(row, column)
        if rule.unit is None and not rule.text_unit:
            raise MappingError(f"table {row.table}: column `{column}` declares no unit")
        raw = self._raw(row, column, rule)
        if raw is None:
            return None
        if rule.text_unit and not isinstance(raw, str):
            assert rule.unit is not None  # validated: a default has its unit
            self._use(row, column, rule)
            return Quantity(float(raw), rule.unit)
        if rule.text_unit and isinstance(raw, str):
            found = self._quantity_text(row, column, rule, raw)
            self._use(row, column, rule)
            return found
        if rule.text_number is not None and isinstance(raw, str):
            digits = re.fullmatch(rule.text_number, raw)
            if digits is None:
                raise RowHeld(
                    "pattern_mismatch",
                    f"{row.table}.{column}: {raw!r} does not match `{rule.text_number}`",
                )
            try:
                number = float(digits.group(1))
            except ValueError as error:
                raise RowHeld("not_a_number", f"{row.table}.{column}: {raw!r}") from error
            self._use(row, column, rule)
            assert rule.unit is not None
            return Quantity(number, rule.unit)
        if isinstance(raw, bool) or not isinstance(raw, (int, float)):
            raise RowHeld("not_a_number", f"{row.table}.{column}: {raw!r} is not a number")
        self._use(row, column, rule)
        assert rule.unit is not None
        return Quantity(float(raw) * (rule.factor if rule.factor is not None else 1.0), rule.unit)

    def _quantity_text(self, row: SourceRow, column: str, rule: FieldRule, text: str) -> Quantity:
        found = NUMBER_WITH_UNIT.fullmatch(text)
        if found is None:
            raise RowHeld(
                "unit_not_parseable",
                f"{row.table}.{column}: {text!r} is not a number followed by its unit",
            )
        unit = found.group(2)
        storage = self._target(rule).unit
        try:
            if storage is not None:
                convert(1.0, unit, storage)
        except ValueRefused as error:
            raise RowHeld("unit_not_parseable", f"{row.table}.{column}: {error}") from error
        return Quantity(float(found.group(1)), unit)

    def integer(self, row: SourceRow, column: str) -> int | None:
        """The value of an integer `column` with the offset its rule declares added (a family
        index that the source counts from zero) and its `factor` applied (a count the source
        states with the opposite sign), or `None` when absent."""
        rule = self.rule(row, column)
        raw = self._raw(row, column, rule)
        if raw is None:
            return None
        if isinstance(raw, bool) or not isinstance(raw, int):
            if isinstance(raw, float) and raw == int(raw) and rule.factor is not None:
                raw = int(raw)
            else:
                raise RowHeld("not_an_integer", f"{row.table}.{column}: {raw!r} is not an integer")
        self._use(row, column, rule)
        factor = 1 if rule.factor is None else int(rule.factor)
        return raw * factor + (rule.offset or 0)

    def digits(self, row: SourceRow, column: str) -> int | None:
        """The number of significant digits the source wrote for the value of `column`, from the
        text its rule names (`text`): `None` when the text is not a plain number (a marker,
        `INFINITE`, a cell the source leaves empty). The digits run from the first nonzero digit
        to the last one written; for zero they are the digits written after the decimal point (at
        least one)."""
        rule = self.rule(row, column)
        if rule.text is None:
            raise MappingError(f"table {row.table}: column `{column}` names no `text` column")
        raw = row[rule.text]
        if not isinstance(raw, str) or not PLAIN_NUMBER.fullmatch(raw.strip()):
            return None
        body = raw.strip().lstrip("+-").split("e")[0].split("E")[0]
        whole, _, fraction = body.partition(".")
        digits = (whole + fraction).lstrip("0")
        return len(digits) or len(fraction) or ORIGIN

    def value_columns(self, row: SourceRow, target: str) -> list[str]:
        """The columns of `row`'s table whose rules have the target `target`, in the order of the
        rules: how a mapping finds the columns of a dataset without naming them."""
        return [c for c, rule in self.fields(row).items() if rule.target == target]

    def decoded(
        self,
        row: SourceRow,
        column: str,
        *,
        using: Mapping[str, object] | None = None,
        required: bool = True,
    ) -> Decoded | None:
        """What the decoding named by the rule of `column` makes of `row`: the first rule whose
        conditions the row satisfies decides (the conditions are tested against `using` where the
        fact decoded is that of another row, such as the phase that includes a species). A row
        no rule covers is held (`missing_convention`): the source does not state what the
        mapping needs of it; with `required` false it is `None` instead, for a fact the mapping
        can do without."""
        rule = self.rule(row, column)
        if rule.decode is None:
            raise MappingError(f"table {row.table}: column `{column}` names no decoding")
        decoding = self.spec.decodings[rule.decode]
        values = row.values if using is None else using
        for line in decoding.rules:
            groups = _satisfies(line.where, values)
            if groups is None:
                continue
            self._use(row, column, rule)
            polymorph = None if line.polymorph is None else line.polymorph.format(**groups)
            key = None if line.key is None else line.key.format(**groups)
            return Decoded(line.to.format(**groups), polymorph, line.reason, key, groups)
        if not required:
            return None
        shown = ", ".join(f"{c}={values.get(c)!r}" for c in _decoding_columns(decoding))
        raise RowHeld(
            "missing_convention",
            f"{row.locator}: decoding `{rule.decode}` has no rule for {shown}",
        )

    def decoded_by(
        self,
        row: SourceRow,
        decoding: str,
        *,
        using: Mapping[str, object] | None = None,
        required: bool = True,
    ) -> Decoded | None:
        """What the decoding `decoding` makes of `row` (see `decoded`), through the rule of
        the row's table that names it."""
        columns = [c for c, rule in self.fields(row).items() if rule.decode == decoding]
        if len(columns) != ORIGIN:
            raise MappingError(
                f"table {row.table}: {len(columns)} field rules name the decoding `{decoding}`, "
                "needs one"
            )
        return self.decoded(row, first(columns), using=using, required=required)

    @staticmethod
    def family_key(family: str) -> str:
        """The name of a family inside its slot group (`piece` for `nasa7.pure.piece`), which is
        how a set's `families` are keyed."""
        *_, name = family.split(".")
        return name

    def family_of(self, group: str) -> str:
        """The `form.group.family` of the one family of the slot group `group` (`form.group`)."""
        found = [f.qualified for f in self.decl.families if f.qualified.rpartition(".")[0] == group]
        if len(found) != ORIGIN:
            raise MappingError(f"slot group `{group}` has {len(found)} families, needs one")
        return first(found)

    def _column_value(self, row: SourceRow, column: str, rule: FieldRule) -> object | None:
        if rule.unit is not None or rule.text_unit:
            return self.quantity(row, column)
        if rule.offset is not None:
            return self.integer(row, column)
        return self.value(row, column)

    def _targets(self, row: SourceRow, prefix: str) -> Iterator[tuple[str, str, FieldRule]]:
        """(column, name, rule) of every column of `row`'s table that maps to `prefix.name`."""
        for column, rule in self.fields(row).items():
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
        values.update(self.constants(row, kind))
        return values

    def constants(self, row: SourceRow, kind: str) -> dict[str, object]:
        """The constants of `kind` declared for the row's table or partition, by attribute
        name: the values the mapping gives that no column holds."""
        return {
            text.rpartition(".")[2]: self._constant(text, raw)
            for text, raw in self._constants(row).items()
            if text.rpartition(".")[0] == kind
        }

    def declared(self, kind: str, name: str) -> uuid.UUID:
        """The identifier of the declared entity `name` of `kind` (the aggregation a decoding
        gave, by its name)."""
        found = declared_entity(self.decl, kind, name)
        if found is None:
            raise RowHeld("unknown_subject", f"`{name}` is not a declared {kind}")
        return found.id

    def states_standard_state(self, row: SourceRow, column: str) -> bool:
        """Whether the values of `column` are of the standard state of the segment they are in
        (`standard_state = true` in the rule's `column` table)."""
        return self.rule(row, column).column_standard_state

    def column_attributes(self, row: SourceRow, column: str) -> dict[str, object]:
        """The attributes of the `dataset_column` the values of `column` fill, as its rule's
        `column` table states them (an enum member or a declared entity by name, text, a number
        or a dimensioned value)."""
        rule = self.rule(row, column)
        if rule.column is None:
            raise MappingError(f"table {row.table}: column `{column}` states no `column`")
        return {
            name: self._constant(f"{pc.DATASET_COLUMN.declared}.{name}", raw)
            for name, raw in rule.column.items()
            if name != STANDARD_STATE
        }

    def _constants(self, row: SourceRow) -> dict[str, Scalar | dict[str, Scalar]]:
        rule = self.spec.tables[row.table]
        constants = dict(rule.constants)
        name = self._partition(row)
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


PART_SEPARATOR = "#"


def keyed(key: str, part: str | None) -> str:
    """The local key of the source entity that is one `part` of what the row's key names (a
    segment of a table that changes phase): the key, `#` and the part."""
    return key if part is None else f"{key}{PART_SEPARATOR}{part}"


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

    def entity(self, scope: str, key: str, part: str | None = None) -> EntityHandle:
        """The source entity (`scope`, `key`) an earlier block of this run declared, to cite it (as
        a component of a mixture). An entity not declared holds the rows of the block."""
        key = keyed(key, part)
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
        self._checked = False

    def _ready(self) -> None:
        if not self._checked:
            self._checked = True
            _structure_held(self._ctx, self._block.rows)  # type: ignore[arg-type]

    def check(self) -> None:
        """Apply the structure rules of the block's rows now (`equals`, `holds`): a block that
        emits nothing but is to be held for what its rows state calls this."""
        self._ready()

    def source_entity(
        self,
        scope: str,
        *,
        aggregation: str | None = None,
        polymorph: str | None = None,
        stated_charge: int | None = None,
        part: str | None = None,
    ) -> EntityHandle:
        """Declare the source entity the block's first row is, in `scope` (the scope names the
        table and the key column in `mapping.toml`). `part` names one part of what the row's key
        names, when the row holds several entities (`keyed`)."""
        self._ready()
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
        key = keyed(key, part)
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
        self._ready()
        ctx = self._ctx
        declared = ctx._declared.get((mixture.scope, mixture.local_key))
        if declared is None or declared.entity_class != pc.ENTITY_CLASS.member("defined_mixture"):
            raise MappingError(
                f"({mixture.scope}, {mixture.local_key!r}) is not a declared defined mixture"
            )
        rule = ctx.rule(row, column)
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

    def assertion(
        self,
        entity: EntityHandle,
        column: str,
        value: object | None = None,
        *,
        row: SourceRow | None = None,
    ) -> bool:
        """Assert the value of `column` (or `value`, when a column holds several, such as a list
        of aliases, or the identifier is made from several values) as an identifier of `entity`,
        under the scheme its rule declares. `row` is the row of the block whose `column` it is
        (the block's first row when not given). Returns whether an assertion was made: a value
        the source does not have, or writes as a declared absent marker, yields none."""
        self._ready()
        ctx = self._ctx
        row = self._row if row is None else row
        rule = ctx.rule(row, column)
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

    def subject(self, scope: str, key: str, part: str | None = None) -> uuid.UUID:
        """The material entity the source entity (`scope`, `key`) of this carrier resolved to
        (`part`: see `keyed`).

        An unknown or ambiguous source entity holds the rows of the block."""
        key = keyed(key, part)
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
        self._checked = False

    def _ready(self) -> None:
        """Hold the block if a row's structure is not the one the mapping relies on (once, at the
        first record of the block)."""
        if not self._checked:
            self._checked = True
            _structure_held(self._ctx, self._block.rows)  # type: ignore[arg-type]

    def check(self) -> None:
        """Apply the structure rules of the block's rows now (`equals`, `holds`)."""
        self._ready()

    @property
    def locator(self) -> str:
        return self._origins[0].ref.locator

    @property
    def carrier(self) -> uuid.UUID:
        """The identifier of the carrier the rows come from (the `carrier` of a dataset)."""
        return identity.identifier(pc.SOURCE.declared, [self._ctx.carrier.key])

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
        self._ready()
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
        self._ready()
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
        self._ready()
        writer = self._ctx.writer
        relation = writer.decl.relations.get(name)
        own = relation is not None and relation.provenance.mode == "own"
        found = writer.relation(
            name, keys, values, origins=self._origins if own else (), at=self.locator
        )
        self._block.count += 1
        return found

    def validity(
        self,
        record: uuid.UUID,
        region: Mapping[str, object],
        clauses: Sequence[Mapping[str, object]],
        *,
        ordinal: int = 1,
    ) -> uuid.UUID:
        """One region of the validity of `record` with its clauses, and the coverage row that says
        the source states regions of that kind for it, in one call. `region` holds the attributes
        of `validity_region` the mapping declares (its `kind`, `ctx.attributes(row,
        "validity_region")`) and each of `clauses` those of one `region_clause`
        (`ctx.attributes(row, "region_clause")`); the region takes the block's origins. Several
        regions of one kind are alternatives, each with its own `ordinal`."""
        found = self._ctx.writer.validity_region(
            record, region, clauses, origins=self._origins, ordinal=ordinal, at=self.locator
        )
        self._block.count += 1
        return found

    def validity_not_stated(self, record: uuid.UUID, region: Mapping[str, object]) -> uuid.UUID:
        """Record that the source gives no region of the kind `region` names for `record`."""
        self._ready()
        kind = region.get(pc.VALIDITY_REGION.kind)
        if kind is None:
            raise MappingError("a `not_stated` validity names the `kind` of region the source omits")
        found = self._ctx.writer.validity_not_stated(record, kind, at=self.locator)
        self._block.count += 1
        return found

    def derivation(self, outputs: Sequence[uuid.UUID]) -> uuid.UUID:
        """The derivation declared for the block's first row (a `fit` when it is one), with a
        `derivation_output` row for each record in `outputs`. It takes no input: the records a
        derivation consumed are linked only when the mapping can name them."""
        self._ready()
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

    def convention_set(self, name: str, origins: tuple[Origin, ...] | None = None) -> uuid.UUID:
        """The convention set `name` of `mapping.toml` (with the energy reference it names), its
        records taking `origins` (the block's rows when not given)."""
        self._ready()
        ctx = self._ctx
        origins = self._origins if origins is None else origins
        table = ctx.spec.convention_sets.get(name)
        if table is None:
            raise MappingError(f"`{name}` is not a declared convention set")
        attributes: dict[str, object] = {}
        for attribute, raw in table.items():
            if attribute == pc.CONVENTION_SET.energy_reference:
                attributes[attribute] = self._energy_reference(str(raw), origins)
            else:
                attributes[attribute] = _stated(raw, ctx.carrier.pin)
        return self._kind(pc.CONVENTION_SET.declared, attributes, origins)

    def _energy_reference(self, name: str, origins: tuple[Origin, ...]) -> uuid.UUID:
        ctx = self._ctx
        table = ctx.spec.energy_references[name]
        return self._kind(
            pc.ENERGY_REFERENCE.declared,
            {a: _stated(raw, ctx.carrier.pin) for a, raw in table.items()},
            origins,
        )

    def parameterization(self, name: str, *, artifact: str | None = None) -> uuid.UUID:
        """The parameterization `name` of `mapping.toml` (and its convention set), with this
        block's rows among its origins, in the origin role `mapping.toml` gives the
        parameterization and not the role of the block that happens to emit it. A
        parameterization that is `per_artifact` is the one of the file `artifact`."""
        self._ready()
        ctx = self._ctx
        spec = ctx.spec.parameterizations.get(name)
        if spec is None:
            raise MappingError(f"`{name}` is not a declared parameterization")
        if spec.per_artifact != (artifact is not None):
            raise MappingError(
                f"parameterization `{name}` "
                + ("is per artifact and needs the `artifact`" if spec.per_artifact else "names no artifact")
            )
        origins = tuple(Origin(origin.ref, spec.origin_role) for origin in self._origins)
        convention: uuid.UUID | None = None
        if spec.convention_set is not None:
            convention = self.convention_set(spec.convention_set, origins)
        pz = pc.PARAMETERIZATION
        found = self._kind(
            pz.declared,
            {
                pz.key: spec.key.format(artifact=artifact, pin=ctx.carrier.pin),
                pz.revision: spec.revision.format(pin=ctx.carrier.pin),
                pz.title: spec.title.format(artifact=artifact, pin=ctx.carrier.pin),
                pz.coherence: spec.coherence,
                pz.convention_set: convention,
            },
            origins,
        )
        ctx.parameterizations[found] = spec
        return found


def _stated(raw: Scalar | dict[str, Scalar], pin: str) -> object:
    """A fact `mapping.toml` states: `{ value, unit }` as a quantity, text with the carrier's
    resolved pin for `{pin}`, anything else as itself."""
    if isinstance(raw, dict):
        return Quantity(float(raw["value"]), str(raw["unit"]))  # type: ignore[arg-type]
    if isinstance(raw, str):
        return raw.replace("{pin}", pin)
    return raw
