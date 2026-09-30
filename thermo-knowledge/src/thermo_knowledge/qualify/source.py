# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""A `ParameterSource` over the built database (pipeline section 5).

The source answers the reference evaluator from canonical records: slot values in storage units
from the `param.<form>__<group>` tables, the rows of a family from its table, a nested set by
following the slot's foreign key to the set it holds, and the forms chosen for a sub-form slot
from an explicit choice given by the caller (model assemblies come later). It reads one or
several parameterizations in order: the first that holds a set for a slot group and subjects
answers. It holds no defaults (`default_slot_values` is always `None`), so a missing set is
reported as missing and the evaluator refuses.

Subjects are stored in the canonical orientation, and the source looks a subject tuple up exactly
as asked; the evaluator tries the other orientations itself. Reads are bounded: one query for the
row of a slot group (all its slots at once) and one for each family the evaluation touches, or,
after `prefetch`, one query for the sets of many subjects and one per family for all of them.
A database built from another declaration than the one given is refused.

One parameterization may hold several sets for one subject (repeated assertions of a source,
distinguished by their `occurrence`). The source never picks one silently: for each
parameterization the caller states the occurrence to read, and a parameterization that holds
several occurrences for a subject it is asked about, with none stated, is refused with a message
naming the subject and the occurrences present (`AmbiguousOccurrence`).
"""

from __future__ import annotations

import uuid
from collections.abc import Mapping, Sequence
from dataclasses import dataclass, field
from pathlib import Path

import psycopg
from psycopg import sql

from thermo_knowledge import config
from thermo_knowledge import pipeline_contract as pc
from thermo_knowledge.declaration import model as m
from thermo_knowledge.expression.parameters import (
    FamilyRows,
    FormChoice,
    SlotValues,
    Subject,
)
from thermo_knowledge.generate.fingerprint import declaration_fingerprint
from thermo_knowledge.schema_build import read_physical, recorded_in

PARAM_SCHEMA = m.PARAM_SCHEMA
_MAX_REDIRECTS = 8


class SourceError(Exception):
    """The source cannot read the database: it was built from another declaration, or a
    record is not what its slot group declares."""


class AmbiguousOccurrence(SourceError):
    """A parameterization holds several sets for one subject and no occurrence was chosen."""


@dataclass(frozen=True)
class SubformBinding:
    """The form chosen for a sub-form slot, the parameterizations (in order) its own sets are
    read from and the occurrence to read in each parameterization that states one."""

    form: str
    parameterizations: tuple[uuid.UUID, ...]
    occurrences: Mapping[uuid.UUID, int] = field(default_factory=dict)


@dataclass(frozen=True)
class Scope:
    """The parameterizations a source reads, in order, and the occurrence to read in each one
    that states one (a parameterization without an entry has one occurrence per subject or is
    refused)."""

    parameterizations: tuple[uuid.UUID, ...]
    occurrences: tuple[tuple[uuid.UUID, int], ...] = ()

    @staticmethod
    def of(
        parameterizations: Sequence[uuid.UUID], occurrences: Mapping[uuid.UUID, int] | None = None
    ) -> Scope:
        stated = occurrences or {}
        return Scope(
            tuple(parameterizations), tuple(sorted(stated.items(), key=lambda kv: str(kv[0])))
        )

    def occurrence(self, parameterization: uuid.UUID) -> int | None:
        return dict(self.occurrences).get(parameterization)


def choose_occurrence[T](
    group: str,
    subjects: Sequence[str],
    candidates: Mapping[int, T],
    wanted: int | None,
) -> T | None:
    """The one set of a parameterization for a subject: the stated occurrence (`None` when the
    parameterization holds no such occurrence), or the only one there is. Several occurrences
    with none stated are refused, naming the subject and the occurrences present."""
    if wanted is not None:
        return candidates.get(wanted)
    if len(candidates) > 1:
        present = ", ".join(str(n) for n in sorted(candidates))
        raise AmbiguousOccurrence(
            f"`{group}` holds {len(candidates)} sets for the subject ({', '.join(subjects)}) in one "
            f"parameterization, with occurrences {present}: state `occurrence = <n>` for the "
            "parameterization to choose one"
        )
    return next(iter(candidates.values()), None)


@dataclass
class _Set:
    """One parameter set of a slot group, as stored."""

    id: uuid.UUID
    group: str
    subjects: tuple[Subject, ...]
    parameterization: uuid.UUID | None = None
    """Known for a set read by subject (the join that selects it carries it), else `None`."""
    occurrence: int = 1
    """Known with the parameterization."""
    slots: dict[str, float] = field(default_factory=dict)
    nested: dict[str, uuid.UUID] = field(default_factory=dict)
    redirects: dict[str, uuid.UUID] = field(default_factory=dict)
    families: dict[str, dict[tuple[int, ...], dict[str, float]]] = field(default_factory=dict)


class _Backend:
    """What every source over one connection shares: the declaration's index of slot groups and
    the sets read so far."""

    def __init__(self, conn: psycopg.Connection, decl: m.Declaration) -> None:
        self.conn = conn
        self.decl = decl
        self.groups = {group.qualified: group for group in decl.slot_groups}
        self.by_id: dict[uuid.UUID, _Set] = {}
        self.by_key: dict[tuple[str, tuple[Subject, ...], Scope], _Set | None] = {}
        self.group_of_marker: dict[uuid.UUID, str] = {
            marker: name for name, marker in decl.meta_ids()["slot_group"].items()
        }

    def group(self, qualified: str) -> m.SlotGroup:
        found = self.groups.get(qualified)
        if found is None:
            raise SourceError(f"`{qualified}` is not a slot group (`form.group`)")
        return found

    # -- queries ---------------------------------------------------------------------------

    def _columns(self, group: m.SlotGroup) -> list[sql.Composable]:
        columns: list[sql.Composable] = [sql.SQL("g.id")]
        columns.extend(sql.SQL("g.{}").format(sql.Identifier(s.name)) for s in group.subjects)
        for slot in group.slots:
            columns.append(sql.SQL("g.{}").format(sql.Identifier(slot.name)))
            if slot.presence == "stateful":
                columns.append(sql.SQL("g.{}").format(sql.Identifier(f"{slot.name}__state")))
                columns.append(sql.SQL("g.{}").format(sql.Identifier(f"{slot.name}__redirect")))
        return columns

    def _select(
        self,
        group: m.SlotGroup,
        where: sql.Composable,
        params: Sequence[object],
        *,
        owner: bool = False,
    ) -> list[_Set]:
        """The sets `where` selects. With `owner`, `where` joins the parameter sets as `ps` and
        each set carries its parameterization and occurrence."""
        columns = self._columns(group)
        if owner:
            columns.append(sql.SQL("ps.{}").format(sql.Identifier(pc.PARAMETER_SET.parameterization)))
            columns.append(sql.SQL("ps.{}").format(sql.Identifier(pc.PARAMETER_SET.occurrence)))
        query = sql.SQL("SELECT {columns} FROM {table} g {where}").format(
            columns=sql.SQL(", ").join(columns),
            table=sql.Identifier(PARAM_SCHEMA, group.id),
            where=where,
        )
        found: list[_Set] = []
        for row in self.conn.execute(query, params).fetchall():
            values = iter(row)
            identifier = next(values)
            subjects = tuple(str(next(values)) for _ in group.subjects)
            record = _Set(identifier, group.qualified, subjects)
            for slot in group.slots:
                value = next(values)
                state = redirect = None
                if slot.presence == "stateful":
                    state, redirect = next(values), next(values)
                if slot.shape == "nested_set":
                    if value is not None:
                        record.nested[slot.name] = value
                elif slot.shape == "quantity":
                    if state == "redirect" and redirect is not None:
                        record.redirects[slot.name] = redirect
                    elif value is not None and state in (None, "known"):
                        record.slots[slot.name] = float(value)
            if owner:
                record.parameterization = next(values)
                record.occurrence = int(next(values))
            found.append(record)
        return found

    def _remember(self, sets: Sequence[_Set]) -> None:
        for record in sets:
            self.by_id.setdefault(record.id, record)

    def load_by_keys(
        self,
        group: m.SlotGroup,
        subjects: Sequence[tuple[Subject, ...]],
        scope: Scope,
    ) -> None:
        """Read the top-level sets of `group` for every tuple of `subjects` in the parameterizations
        of `scope` (one query), remembering the answer, found or not. For each subject the first
        parameterization that holds a set answers, with the occurrence `scope` states for it."""
        wanted = [key for key in dict.fromkeys(subjects)]
        valid: list[tuple[uuid.UUID, ...]] = []
        for key in wanted:
            try:
                valid.append(tuple(uuid.UUID(part) for part in key))
            except ValueError:
                self.by_key[(group.qualified, key, scope)] = None
        if not valid:
            return
        if group.subjects:
            arrays = [[key[position] for key in valid] for position in range(len(group.subjects))]
            match = sql.SQL("({cols}) IN (SELECT * FROM unnest({arrays}))").format(
                cols=sql.SQL(", ").join(
                    sql.SQL("g.{}").format(sql.Identifier(s.name)) for s in group.subjects
                ),
                arrays=sql.SQL(", ").join(sql.SQL("%s::uuid[]") for _ in group.subjects),
            )
            params: list[object] = [*arrays]
        else:
            match = sql.SQL("true")
            params = []
        ps = pc.PARAMETER_SET
        where = (
            sql.SQL(
                "JOIN {sets} ps ON ps.id = g.id "
                "WHERE ps.{parameterization} = ANY(%s) AND ps.{parent} IS NULL AND "
            ).format(
                sets=sql.Identifier(*ps.table.split(".")),
                parameterization=sql.Identifier(ps.parameterization),
                parent=sql.Identifier(ps.parent),
            )
            + match
        )
        rows = self._select(group, where, [list(scope.parameterizations), *params], owner=True)
        held: dict[tuple[Subject, ...], dict[uuid.UUID, dict[int, _Set]]] = {}
        for row in rows:
            by_parameterization = held.setdefault(row.subjects, {})
            by_parameterization.setdefault(row.parameterization, {})[row.occurrence] = row  # type: ignore[index]
        chosen: list[_Set] = []
        for key in wanted:
            if (group.qualified, key, scope) in self.by_key:
                continue
            found: _Set | None = None
            for parameterization in scope.parameterizations:
                candidates = held.get(key, {}).get(parameterization)
                if not candidates:
                    continue
                found = choose_occurrence(
                    group.qualified, key, candidates, scope.occurrence(parameterization)
                )
                if found is not None:
                    break
            self.by_key[(group.qualified, key, scope)] = found
            if found is not None:
                chosen.append(found)
        self._remember(chosen)

    def by_key_lookup(
        self,
        group: m.SlotGroup,
        subjects: tuple[Subject, ...],
        scope: Scope,
    ) -> _Set | None:
        key = (group.qualified, subjects, scope)
        if key not in self.by_key:
            self.load_by_keys(group, [subjects], scope)
        return self.by_key[key]

    def set_by_id(self, identifier: uuid.UUID) -> _Set:
        """The set with this identifier (a nested set, or a redirect's target)."""
        known = self.by_id.get(identifier)
        if known is not None:
            return known
        row = self.conn.execute(
            f"SELECT {pc.PARAMETER_SET.slot_group} FROM {pc.PARAMETER_SET.table} WHERE id = %s",  # noqa: S608
            (identifier,),
        ).fetchone()
        if row is None:
            raise SourceError(f"the parameter set {identifier} is not in the database")
        name = self.group_of_marker.get(row[0])
        if name is None:
            raise SourceError(
                f"the parameter set {identifier} has a slot group the declaration lacks"
            )
        group = self.group(name)
        found = self._select(group, sql.SQL("WHERE g.id = %s"), [identifier])
        if not found:
            raise SourceError(f"the parameter set {identifier} has no row in {group.id}")
        self._remember(found)
        return self.by_id[identifier]

    def resolve_redirects(self, record: _Set) -> None:
        """Replace each redirected slot by the value of the same slot of the set it names."""
        for slot, target in list(record.redirects.items()):
            seen = {record.id}
            current = target
            for _ in range(_MAX_REDIRECTS):
                if current in seen:
                    break
                seen.add(current)
                other = self.set_by_id(current)
                if other.group != record.group:
                    break
                if slot in other.slots:
                    record.slots[slot] = other.slots[slot]
                    break
                if slot not in other.redirects:
                    break
                current = other.redirects[slot]
            del record.redirects[slot]

    def families(self, group: m.SlotGroup, records: Sequence[_Set]) -> None:
        """Read the rows of every family of `group` for `records` (one query per family)."""
        pending = [r for r in records if not set(f.name for f in group.families) <= set(r.families)]
        if not pending or not group.families:
            return
        ids = [r.id for r in pending]
        for family in group.families:
            numeric = [s for s in family.slots if s.shape == "quantity"]
            columns = [sql.Identifier("set_id")]
            columns += [sql.Identifier(index.name) for index in family.indices]
            columns += [sql.Identifier(s.name) for s in numeric]
            query = sql.SQL("SELECT {} FROM {} WHERE set_id = ANY(%s)").format(
                sql.SQL(", ").join(columns), sql.Identifier(PARAM_SCHEMA, family.id)
            )
            rows: dict[uuid.UUID, dict[tuple[int, ...], dict[str, float]]] = {i: {} for i in ids}
            for row in self.conn.execute(query, (ids,)).fetchall():
                identifier, rest = row[0], row[1:]
                key = tuple(int(part) for part in rest[: len(family.indices)])
                values = rest[len(family.indices) :]
                rows[identifier][key] = {
                    slot.name: float(value)
                    for slot, value in zip(numeric, values)
                    if value is not None
                }
            for record in pending:
                record.families[family.name] = rows[record.id]


def check_database(conn: psycopg.Connection, decl: m.Declaration, tree: Path | None = None) -> None:
    """Refuse a database whose recorded schema fingerprint is not the one `decl` produces."""
    expected = declaration_fingerprint(decl, read_physical(tree))
    recorded = recorded_in(conn)
    if recorded != expected:
        seen = (
            "records no schema fingerprint"
            if recorded is None
            else f"was built from {recorded[:12]}"
        )
        raise SourceError(
            f"the database {seen}, the declaration is {expected[:12]}: rebuild it with `tk build`"
        )


class DatabaseSource:
    """A `ParameterSource` over the built database for an ordered list of parameterizations.

    `occurrences` maps a parameterization to the occurrence to read in it, where it holds
    repeated assertions for a subject; a parameterization that holds several occurrences for a
    subject it is asked about and has no entry is refused (`AmbiguousOccurrence`).
    `subforms` maps a sub-form slot (`form.slot`) to the forms chosen for it, each with the
    parameterizations its sets are read from; a slot with none chosen has no choices. The
    constructor refuses a database whose recorded fingerprint differs from `decl`'s.
    """

    def __init__(
        self,
        conn: psycopg.Connection,
        decl: m.Declaration,
        parameterizations: Sequence[uuid.UUID],
        *,
        occurrences: Mapping[uuid.UUID, int] | None = None,
        subforms: Mapping[str, Sequence[SubformBinding]] | None = None,
        tree: Path | None = None,
    ) -> None:
        check_database(conn, decl, tree)
        self._init(
            _Backend(conn, decl), Scope.of(parameterizations, occurrences), subforms or {}, None
        )

    def _init(
        self,
        backend: _Backend,
        scope: Scope,
        subforms: Mapping[str, Sequence[SubformBinding]],
        pinned: _Set | None,
    ) -> None:
        self._backend = backend
        self._scope = scope
        self._subforms = subforms
        self._pinned = pinned

    @classmethod
    def _derived(
        cls,
        backend: _Backend,
        scope: Scope,
        pinned: _Set | None,
        subforms: Mapping[str, Sequence[SubformBinding]] | None = None,
    ) -> DatabaseSource:
        source = cls.__new__(cls)
        source._init(backend, scope, subforms or {}, pinned)
        return source

    # -- reading ---------------------------------------------------------------------------

    def prefetch(self, group: str, subjects: Sequence[tuple[Subject, ...]]) -> None:
        """Read the sets of `group` for all of `subjects`, and their family rows, in a number of
        queries that does not grow with the subjects."""
        slot_group = self._backend.group(group)
        if self._pinned is not None:
            return
        self._backend.load_by_keys(slot_group, list(subjects), self._scope)
        found = [
            record
            for key in subjects
            if (record := self._backend.by_key[(group, key, self._scope)]) is not None
        ]
        for record in found:
            self._backend.resolve_redirects(record)
        self._backend.families(slot_group, found)

    def _set(self, group: str, subjects: tuple[Subject, ...]) -> _Set | None:
        slot_group = self._backend.group(group)
        if self._pinned is not None:
            if self._pinned.group == group and self._pinned.subjects == subjects:
                return self._pinned
            return None
        record = self._backend.by_key_lookup(slot_group, subjects, self._scope)
        if record is not None:
            self._backend.resolve_redirects(record)
        return record

    def set_id(self, group: str, subjects: tuple[Subject, ...]) -> uuid.UUID | None:
        """The identifier of the set this source reads for `group` and exactly `subjects`."""
        record = self._set(group, subjects)
        return None if record is None else record.id

    def slot_values(self, group: str, subjects: tuple[Subject, ...]) -> SlotValues | None:
        record = self._set(group, subjects)
        return None if record is None else dict(record.slots)

    def default_slot_values(self, group: str, subjects: tuple[Subject, ...]) -> SlotValues | None:
        return None

    def family_rows(
        self, group: str, family: str, subjects: tuple[Subject, ...]
    ) -> FamilyRows | None:
        record = self._set(group, subjects)
        if record is None:
            return None
        if family not in record.families:
            self._backend.families(self._backend.group(group), [record])
        if family not in record.families:
            raise SourceError(f"`{family}` is not a family of `{group}`")
        rows = record.families[family]
        return {key: dict(values) for key, values in rows.items()}

    def nested_set(self, group: str, slot: str, subjects: tuple[Subject, ...]) -> FormChoice | None:
        record = self._set(group, subjects)
        if record is None or slot not in record.nested:
            return None
        child = self._backend.set_by_id(record.nested[slot])
        self._backend.resolve_redirects(child)
        form = self._backend.group(child.group).form
        return FormChoice(form, self._derived(self._backend, self._scope, child, self._subforms))

    def subform_choices(self, slot: str, subjects: tuple[Subject, ...]) -> tuple[FormChoice, ...]:
        return tuple(
            FormChoice(
                binding.form,
                self._derived(
                    self._backend,
                    Scope.of(binding.parameterizations, binding.occurrences),
                    None,
                    self._subforms,
                ),
            )
            for binding in self._subforms.get(slot, ())
        )
