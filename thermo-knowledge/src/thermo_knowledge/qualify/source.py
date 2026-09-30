# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""A `ParameterSource` over the built database (pipeline section 5).

The source answers the reference evaluator from canonical records: slot values in storage units
from the `param.<form>__<group>` tables, the rows of a family from its table, a nested or
referenced set by following the slot's foreign key to the set it holds, the forms chosen for a
sub-form slot (from an explicit choice given by the caller, else for a slot chosen per subject
from the `subject_subform_choice` relation), the convention facts of the parameterizations that
supplied the sets an evaluation read and the validity regions of those sets and parameterizations. It reads one or several parameterizations in order:
the first that holds a set for a slot group and subjects answers. It holds no defaults
(`default_slot_values` is `None`, so a missing set is reported as missing and the evaluator
refuses) unless the `policy` the source is given applies stated defaults to unasserted subjects.

Subjects are stored in the canonical orientation, with the `arrangement` the values were asserted
for where the group's rule acts on values. The source finds a set by the canonical orientation of
the subjects asked for and returns its values for the order asked: the stored numbers unchanged
when that is the order asserted, otherwise the rule applied once (`thermo_knowledge.transposition`).
Reads are bounded: one query for the row of a slot group (all its slots at once) and one for each
family the evaluation touches, or, after `prefetch`, one query for the sets of many subjects and
one per family for all of them. A database built from another declaration than the one given is
refused.

A slot a set leaves at its stated default (value state `stated_default`) holds no value: given a
`policy`, the source supplies the default that policy states for the slot (`policy_default`; a
default whose state is `not_applicable` supplies none), and without one, or where the policy
states none, the slot has no value and the evaluator refuses. A policy whose `unasserted` is
`stated_default` also supplies its defaults for a subject that has no set in a slot group it covers,
once every slot of the group has a known default (`default_slot_values`): an association matrix
read as complete where the source lists only the bonding pairs.

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

from thermo_knowledge import config, identity, transposition
from thermo_knowledge import pipeline_contract as pc
from thermo_knowledge.declaration import model as m
from thermo_knowledge.expression.parameters import (
    ConventionFact,
    FamilyRows,
    FormChoice,
    RecordValidity,
    RegionClause,
    SetRead,
    SlotValues,
    Subject,
    ValidityRegion,
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
    """The subjects in the canonical orientation, as stored."""
    parameterization: uuid.UUID
    occurrence: int = 1
    arrangement: int = 0
    """For which order of the subjects the values were asserted (0 when the group records none)."""
    slots: dict[str, float] = field(default_factory=dict)
    at_default: tuple[str, ...] = ()
    """The slots the set leaves at their stated default: they hold no value of their own."""
    nested: dict[str, uuid.UUID] = field(default_factory=dict)
    """The set each set-valued slot outside a family holds: nested in this set or referenced."""
    redirects: dict[str, uuid.UUID] = field(default_factory=dict)
    families: dict[str, dict[tuple[int, ...], dict[str, float]]] = field(default_factory=dict)
    family_sets: dict[str, dict[tuple[int, ...], uuid.UUID]] = field(default_factory=dict)
    """The set each set-valued slot of a family holds, by `family.slot` and the row's index."""


class _Backend:
    """What every source over one connection shares: the declaration's index of slot groups and
    the sets read so far."""

    def __init__(
        self, conn: psycopg.Connection, decl: m.Declaration, policy: uuid.UUID | None = None
    ) -> None:
        self.conn = conn
        self.decl = decl
        self.policy = policy
        self._policy_defaults: dict[str, float] | None = None
        self.groups = {group.qualified: group for group in decl.slot_groups}
        self.by_id: dict[uuid.UUID, _Set] = {}
        self.by_key: dict[tuple[str, tuple[Subject, ...], Scope], _Set | None] = {}
        ids = decl.meta_ids()
        self.group_of_marker: dict[uuid.UUID, str] = {
            marker: name for name, marker in ids["slot_group"].items()
        }
        self.form_of_id: dict[uuid.UUID, str] = {
            marker: name for name, marker in ids["form"].items()
        }
        self.subform_slots = {
            sub.qualified: sub for form in decl.forms.values() for sub in form.subforms
        }
        self.subform_ids = ids["subform_slot"]
        self.facts: dict[tuple[uuid.UUID, str], ConventionFact] = {}

    def policy_defaults(self) -> dict[str, float]:
        """The value each slot (`form.group.slot`) takes under the policy in force where a set
        leaves it at its stated default; a default that is `not_applicable` states no value."""
        if self._policy_defaults is None:
            found: dict[str, float] = {}
            if self.policy is not None:
                default = pc.POLICY_DEFAULT
                for qualified, value in self.conn.execute(
                    f"SELECT sl.qualified_name, d.{default.value} FROM {default.table} d "  # noqa: S608
                    f"JOIN meta.slot sl ON sl.id = d.{default.slot} "
                    f"WHERE d.{default.policy} = %s AND d.{default.value} IS NOT NULL",
                    (self.policy,),
                ).fetchall():
                    found[qualified] = float(value)
            self._policy_defaults = found
        return self._policy_defaults

    def unasserted_defaults(self, group: m.SlotGroup) -> dict[str, float] | None:
        """The values a subject that has no set in `group` takes under the policy in force, or
        `None`: the policy must apply stated defaults to unasserted subjects
        (`unasserted = stated_default`), cover the group (it has no scope, or this one) and state a
        known default for every slot of the group, since a set is all of its slots."""
        if self.policy is None:
            return None
        policy = pc.SELECTION_POLICY
        row = self.conn.execute(
            f"SELECT p.{policy.unasserted}::text, p.{policy.scope_slot_group} FROM {policy.table} p "  # noqa: S608
            "WHERE p.id = %s",
            (self.policy,),
        ).fetchone()
        if row is None or row[0] != pc.UNASSERTED_POLICY.member("stated_default"):
            return None
        scope = self.group_of_marker.get(row[1]) if row[1] is not None else None
        if row[1] is not None and scope != group.qualified:
            return None
        defaults = self.policy_defaults()
        values: dict[str, float] = {}
        for slot in group.slots:
            held = defaults.get(f"{group.qualified}.{slot.name}")
            if slot.shape != "quantity" or held is None:
                return None
            values[slot.name] = held
        return values

    def with_defaults(self, group: m.SlotGroup, record: _Set) -> dict[str, float]:
        """The slot values of `record`, with the policy's default for each slot it leaves at its
        stated default."""
        values = dict(record.slots)
        if record.at_default:
            defaults = self.policy_defaults()
            for name in record.at_default:
                held = defaults.get(f"{group.qualified}.{name}")
                if held is not None:
                    values[name] = held
        return values

    def group(self, qualified: str) -> m.SlotGroup:
        found = self.groups.get(qualified)
        if found is None:
            raise SourceError(f"`{qualified}` is not a slot group (`form.group`)")
        return found

    def canonical(self, group: m.SlotGroup, subjects: tuple[Subject, ...]) -> tuple[Subject, ...]:
        """The orientation `subjects` is stored in."""
        return transposition.canonical_orientation(group, subjects)[0]

    def asserted(self, group: m.SlotGroup, record: _Set) -> tuple[Subject, ...]:
        """The order of its subjects the values of `record` were asserted for."""
        if transposition.stores_arrangement(group):
            return transposition.asserted_order(group, record.subjects, record.arrangement)
        return record.subjects

    # -- queries ---------------------------------------------------------------------------

    def _columns(self, group: m.SlotGroup) -> list[sql.Composable]:
        columns: list[sql.Composable] = [sql.SQL("g.id")]
        columns.extend(sql.SQL("g.{}").format(sql.Identifier(s.name)) for s in group.subjects)
        if transposition.stores_arrangement(group):
            columns.append(sql.SQL("g.{}").format(sql.Identifier(m.ARRANGEMENT)))
        for slot in group.slots:
            columns.append(sql.SQL("g.{}").format(sql.Identifier(slot.name)))
            if slot.presence == "stateful":
                columns.append(sql.SQL("g.{}").format(sql.Identifier(f"{slot.name}__state")))
                columns.append(sql.SQL("g.{}").format(sql.Identifier(f"{slot.name}__redirect")))
        ps = pc.PARAMETER_SET
        columns.append(sql.SQL("ps.{}").format(sql.Identifier(ps.parameterization)))
        columns.append(sql.SQL("ps.{}").format(sql.Identifier(ps.occurrence)))
        return columns

    def _select(
        self, group: m.SlotGroup, where: sql.Composable, params: Sequence[object]
    ) -> list[_Set]:
        """The sets `where` selects; `where` may use the group's row as `g` and its parameter-set
        row as `ps`."""
        ps = pc.PARAMETER_SET
        query = sql.SQL("SELECT {columns} FROM {table} g JOIN {sets} ps ON ps.id = g.id {where}").format(
            columns=sql.SQL(", ").join(self._columns(group)),
            table=sql.Identifier(PARAM_SCHEMA, group.id),
            sets=sql.Identifier(*ps.table.split(".")),
            where=where,
        )
        found: list[_Set] = []
        for row in self.conn.execute(query, params).fetchall():
            values = iter(row)
            identifier = next(values)
            subjects = tuple(str(next(values)) for _ in group.subjects)
            arrangement = int(next(values)) if transposition.stores_arrangement(group) else 0
            stored: dict[str, tuple[object, object, object]] = {}
            for slot in group.slots:
                value = next(values)
                state = redirect = None
                if slot.presence == "stateful":
                    state, redirect = next(values), next(values)
                stored[slot.name] = (value, state, redirect)
            parameterization, occurrence = next(values), int(next(values))
            record = _Set(
                identifier,
                group.qualified,
                subjects,
                parameterization,
                occurrence,
                arrangement,
            )
            record.at_default = tuple(
                slot.name
                for slot in group.slots
                if slot.shape == "quantity" and stored[slot.name][1] == "stated_default"
            )
            for slot in group.slots:
                value, state, redirect = stored[slot.name]
                if slot.shape in ("nested_set", "set_reference"):
                    if value is not None:
                        record.nested[slot.name] = value  # type: ignore[assignment]
                elif slot.shape == "quantity":
                    if state == "redirect" and redirect is not None:
                        record.redirects[slot.name] = redirect  # type: ignore[assignment]
                    elif value is not None and state in (None, "known"):
                        record.slots[slot.name] = float(value)  # type: ignore[arg-type]
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
        """Read the top-level sets of `group` for every tuple of `subjects` (each in the canonical
        orientation) in the parameterizations of `scope` (one query), remembering the answer,
        found or not. For each subject the first parameterization that holds a set answers, with
        the occurrence `scope` states for it."""
        wanted = [key for key in dict.fromkeys(self.canonical(group, key) for key in subjects)]
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
            sql.SQL("WHERE ps.{parameterization} = ANY(%s) AND ps.{parent} IS NULL AND ").format(
                parameterization=sql.Identifier(ps.parameterization),
                parent=sql.Identifier(ps.parent),
            )
            + match
        )
        rows = self._select(group, where, [list(scope.parameterizations), *params])
        held: dict[tuple[Subject, ...], dict[uuid.UUID, dict[int, _Set]]] = {}
        for row in rows:
            by_parameterization = held.setdefault(row.subjects, {})
            by_parameterization.setdefault(row.parameterization, {})[row.occurrence] = row
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
        """The set of `group` for `subjects` in any order its transposition makes equivalent."""
        key = (group.qualified, self.canonical(group, subjects), scope)
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
            held = [s for s in family.slots if s.shape in ("nested_set", "set_reference")]
            columns = [sql.Identifier("set_id")]
            columns += [sql.Identifier(index.name) for index in family.indices]
            columns += [sql.Identifier(s.name) for s in (*numeric, *held)]
            query = sql.SQL("SELECT {} FROM {} WHERE set_id = ANY(%s)").format(
                sql.SQL(", ").join(columns), sql.Identifier(PARAM_SCHEMA, family.id)
            )
            rows: dict[uuid.UUID, dict[tuple[int, ...], dict[str, float]]] = {i: {} for i in ids}
            sets: dict[uuid.UUID, dict[str, dict[tuple[int, ...], uuid.UUID]]] = {i: {} for i in ids}
            for row in self.conn.execute(query, (ids,)).fetchall():
                identifier, rest = row[0], row[1:]
                key = tuple(int(part) for part in rest[: len(family.indices)])
                values = rest[len(family.indices) :]
                rows[identifier][key] = {
                    slot.name: float(value)
                    for slot, value in zip(numeric, values)
                    if value is not None
                }
                for slot, value in zip(held, values[len(numeric) :]):
                    if value is not None:
                        sets[identifier].setdefault(f"{family.name}.{slot.name}", {})[key] = value
            for record in pending:
                record.families[family.name] = rows[record.id]
                record.family_sets.update(sets[record.id])

    # -- parameterizations ----------------------------------------------------------------

    def label(self, parameterization: uuid.UUID) -> str:
        """`key@revision`, how a message names a parameterization."""
        pz = pc.PARAMETERIZATION
        row = self.conn.execute(
            f"SELECT {pz.key}, {pz.revision} FROM {pz.table} WHERE id = %s",  # noqa: S608
            (parameterization,),
        ).fetchone()
        return f"{row[0]}@{row[1]}" if row is not None else str(parameterization)

    def convention_fact(self, parameterization: uuid.UUID, name: str) -> ConventionFact:
        """The convention fact `name` of the convention set of `parameterization`."""
        kind = self.decl.framework.get(m.CONVENTION_SET_ROLE)
        if kind is None or name not in {a.name for a in self.decl.attributes_of(kind)}:
            raise SourceError(f"`{name}` is not an attribute of the convention-set kind")
        known = self.facts.get((parameterization, name))
        if known is not None:
            return known
        pz = pc.PARAMETERIZATION
        row = self.conn.execute(
            sql.SQL(
                "SELECT p.{link}, c.{fact} FROM {parameterizations} p "
                "LEFT JOIN {conventions} c ON c.id = p.{link} WHERE p.id = %s"
            ).format(
                link=sql.Identifier(pz.convention_set),
                fact=sql.Identifier(name),
                parameterizations=sql.Identifier(*pz.table.split(".")),
                conventions=sql.Identifier(*pc.CONVENTION_SET.table.split(".")),
            ),
            (parameterization,),
        ).fetchone()
        label = self.label(parameterization)
        if row is None or row[0] is None:
            fact = ConventionFact(label, None, "has no convention set")
        elif row[1] is None:
            fact = ConventionFact(label, None, f"has a convention set that states no `{name}`")
        else:
            fact = ConventionFact(label, float(row[1]))
        self.facts[(parameterization, name)] = fact
        return fact

    def _names(self, kind: str) -> dict[uuid.UUID, str]:
        """The declared entities of `kind` by identifier, as their names."""
        return {
            entity.id: entity.name
            for entity in self.decl.entities
            if self.decl.is_a(entity.kind, kind)
        }

    def validities(
        self, kind: str, holders: Sequence[tuple[uuid.UUID, str]]
    ) -> list[RecordValidity]:
        """What each of `holders` (a record's identifier and how a message names it) states about
        its validity for regions of `kind`: its coverage row and its regions with their clauses.
        A record with neither has no coverage and no region."""
        if not holders:
            return []
        coverage_table, region_table = pc.VALIDITY_COVERAGE, pc.VALIDITY_REGION
        clause_table = pc.REGION_CLAUSE
        identifiers = [holder for holder, _ in holders]
        states = dict(
            self.conn.execute(
                f"SELECT {coverage_table.record}, {coverage_table.value}::text "  # noqa: S608
                f"FROM {coverage_table.table} "
                f"WHERE {coverage_table.kind}::text = %s AND {coverage_table.record} = ANY(%s)",
                (kind, identifiers),
            ).fetchall()
        )
        regions = self.conn.execute(
            f"SELECT id, {region_table.record} FROM {region_table.table} "  # noqa: S608
            f"WHERE {region_table.kind}::text = %s AND {region_table.record} = ANY(%s) "
            f"ORDER BY {region_table.record}, {region_table.ordinal}",
            (kind, identifiers),
        ).fetchall()
        clauses: dict[uuid.UUID, list[RegionClause]] = {}
        if regions:
            observables = self._names(self.decl.framework[m.OBSERVABLE_ROLE])
            aggregations = self._names(pc.AGGREGATION.declared)
            rows = self.conn.execute(
                f"SELECT {clause_table.region}, {clause_table.observable}, "  # noqa: S608
                f"{clause_table.component}, {clause_table.aggregation}, "
                f"{clause_table.lower}, {clause_table.upper}, "
                f"{clause_table.lower_relative_to}, {clause_table.upper_relative_to} "
                f"FROM {clause_table.table} "
                f"WHERE {clause_table.region} = ANY(%s) ORDER BY {clause_table.region}, "
                f"{clause_table.ordinal}",
                ([identifier for identifier, _ in regions],),
            ).fetchall()
            for (
                region,
                observable,
                component,
                aggregation,
                lower,
                upper,
                lower_reference,
                upper_reference,
            ) in rows:
                clauses.setdefault(region, []).append(
                    RegionClause(
                        observables[observable],
                        None if lower is None else float(lower),
                        None if upper is None else float(upper),
                        None if component is None else str(component),
                        None if aggregation is None else aggregations[aggregation],
                        None if lower_reference is None else observables[lower_reference],
                        None if upper_reference is None else observables[upper_reference],
                    )
                )
        held: dict[uuid.UUID, list[ValidityRegion]] = {}
        for identifier, record in regions:
            held.setdefault(record, []).append(ValidityRegion(tuple(clauses.get(identifier, ()))))
        return [
            RecordValidity(label, kind, states.get(holder), tuple(held.get(holder, ())))
            for holder, label in holders
        ]

    def positions(self, array: Subject, member: Subject) -> tuple[int, ...]:
        """The positions the constituent array gives the species among the members it places on a
        site class, from `constituent_array_member`, ascending."""
        try:
            array_id, member_id = uuid.UUID(array), uuid.UUID(member)
        except ValueError:
            return ()
        placed = pc.CONSTITUENT_ARRAY_MEMBER
        rows = self.conn.execute(
            f'SELECT "{placed.position}" FROM {placed.table} '  # noqa: S608
            f'WHERE "{placed.array}" = %s AND "{placed.species}" = %s ORDER BY "{placed.position}"',
            (array_id, member_id),
        ).fetchall()
        return tuple(int(row[0]) for row in rows)

    def choices(
        self, slot: str, subjects: tuple[Subject, ...], parameterizations: Sequence[uuid.UUID]
    ) -> list[tuple[str, uuid.UUID]]:
        """The forms `subject_subform_choice` gives the sub-form slot for `subjects`, in ordinal
        order, each with the parameterization that holds the chosen form's sets: those of the
        first parameterization (in order) that states any choice."""
        try:
            key = identity.canonical_encoding([uuid.UUID(part) for part in subjects])
        except ValueError:
            return []
        choice = pc.SUBJECT_SUBFORM_CHOICE
        rows = self.conn.execute(
            f"SELECT {choice.parameterization}, {choice.form}, {choice.source_parameterization} "  # noqa: S608
            f"FROM {choice.table} WHERE {choice.slot} = %s AND {choice.subject_key} = %s "
            f"AND {choice.parameterization} = ANY(%s) ORDER BY {choice.ordinal}",
            (self.subform_ids[slot], key, list(parameterizations)),
        ).fetchall()
        for parameterization in parameterizations:
            mine = [row for row in rows if row[0] == parameterization]
            if mine:
                return [(self.form_of_id[row[1]], row[2] or row[0]) for row in mine]
        return []


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
    parameterizations its sets are read from. A slot chosen per subject that the map does not
    mention takes its choices from the relation `subject_subform_choice` in the parameterizations
    read; with neither, it has no choices. The constructor refuses a database whose recorded
    fingerprint differs from `decl`'s. `policy` is the selection policy whose stated defaults
    fill the slots that sets leave at their stated default.
    """

    def __init__(
        self,
        conn: psycopg.Connection,
        decl: m.Declaration,
        parameterizations: Sequence[uuid.UUID],
        *,
        occurrences: Mapping[uuid.UUID, int] | None = None,
        subforms: Mapping[str, Sequence[SubformBinding]] | None = None,
        policy: uuid.UUID | None = None,
        tree: Path | None = None,
    ) -> None:
        check_database(conn, decl, tree)
        self._init(
            _Backend(conn, decl, policy), Scope.of(parameterizations, occurrences), subforms or {}, None
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
            if (
                record := self._backend.by_key[
                    (group, self._backend.canonical(slot_group, key), self._scope)
                ]
            )
            is not None
        ]
        for record in found:
            self._backend.resolve_redirects(record)
        self._backend.families(slot_group, found)

    def _set(self, group: str, subjects: tuple[Subject, ...]) -> _Set | None:
        slot_group = self._backend.group(group)
        if self._pinned is not None:
            if (
                self._pinned.group == group
                and self._pinned.subjects == self._backend.canonical(slot_group, subjects)
            ):
                return self._pinned
            return None
        record = self._backend.by_key_lookup(slot_group, subjects, self._scope)
        if record is not None:
            self._backend.resolve_redirects(record)
        return record

    def set_id(self, group: str, subjects: tuple[Subject, ...]) -> uuid.UUID | None:
        """The identifier of the set this source reads for `group` and `subjects`."""
        record = self._set(group, subjects)
        return None if record is None else record.id

    def slot_values(self, group: str, subjects: tuple[Subject, ...]) -> SlotValues | None:
        record = self._set(group, subjects)
        if record is None:
            return None
        owner = self._backend.group(group)
        return transposition.read_slots(
            owner,
            self._backend.with_defaults(owner, record),
            self._backend.asserted(owner, record),
            subjects,
        )

    def default_slot_values(self, group: str, subjects: tuple[Subject, ...]) -> SlotValues | None:
        return self._backend.unasserted_defaults(self._backend.group(group))

    def family_rows(
        self, group: str, family: str, subjects: tuple[Subject, ...]
    ) -> FamilyRows | None:
        record = self._set(group, subjects)
        if record is None:
            return None
        owner = self._backend.group(group)
        if family not in record.families:
            self._backend.families(owner, [record])
        if family not in record.families:
            raise SourceError(f"`{family}` is not a family of `{group}`")
        declared = next(f for f in owner.families if f.name == family)
        return transposition.read_rows(
            owner,
            declared,
            record.families[family],
            self._backend.asserted(owner, record),
            subjects,
        )

    def nested_set(
        self, group: str, slot: str, subjects: tuple[Subject, ...], index: tuple[int, ...] = ()
    ) -> FormChoice | None:
        record = self._set(group, subjects)
        if record is None:
            return None
        if "." in slot:
            family = slot.split(".", 1)[0]
            if family not in record.families:
                self._backend.families(self._backend.group(group), [record])
            target = record.family_sets.get(slot, {}).get(index)
        else:
            target = record.nested.get(slot)
        if target is None:
            return None
        child = self._backend.set_by_id(target)
        self._backend.resolve_redirects(child)
        form = self._backend.group(child.group).form
        # A nested set is read where its holder is; a referenced set is top-level, so what it
        # reads (sub-form choices, conventions) is of the parameterization that holds it.
        scope = (
            self._scope
            if child.parameterization == record.parameterization
            else Scope.of((child.parameterization,))
        )
        return FormChoice(
            form,
            self._derived(self._backend, scope, child, self._subforms),
            child.group,
            child.subjects,
        )

    def subform_choices(self, slot: str, subjects: tuple[Subject, ...]) -> tuple[FormChoice, ...]:
        explicit = self._subforms.get(slot)
        if explicit is not None:
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
                for binding in explicit
            )
        declared = self._backend.subform_slots.get(slot)
        if declared is None or declared.per != "subject":
            return ()
        parameterizations = (
            (self._pinned.parameterization,)
            if self._pinned is not None
            else self._scope.parameterizations
        )
        return tuple(
            FormChoice(
                form,
                self._derived(self._backend, Scope.of((source,)), None, self._subforms),
            )
            for form, source in self._backend.choices(slot, subjects, parameterizations)
        )

    def convention_facts(self, name: str, reads: tuple[SetRead, ...]) -> tuple[ConventionFact, ...]:
        """The convention fact `name` of each parameterization that supplied one of the `reads`
        (this source's pinned set, for a nested or referenced one); with no read, of the first
        parameterization this source reads."""
        if self._pinned is not None:
            supplying = [self._pinned.parameterization]
        else:
            found: dict[uuid.UUID, None] = {}
            for group, subjects in reads:
                record = self._set(group, subjects)
                if record is not None:
                    found.setdefault(record.parameterization)
            supplying = list(found) or list(self._scope.parameterizations[:1])
        return tuple(self._backend.convention_fact(p, name) for p in supplying)

    def member_positions(self, array: Subject, member: Subject) -> tuple[int, ...]:
        """The positions the constituent array gives the species, as `constituent_array_member`
        holds them."""
        return self._backend.positions(array, member)

    def validity(self, kind: str, reads: tuple[SetRead, ...]) -> tuple[RecordValidity, ...]:
        """The validity of `kind` that each set of the `reads` and the parameterization it belongs
        to state (this source's pinned set, for a nested or referenced one)."""
        sets: dict[uuid.UUID, _Set] = {}
        for group, subjects in reads:
            record = self._set(group, subjects)
            if record is not None:
                sets.setdefault(record.id, record)
        holders: list[tuple[uuid.UUID, str]] = [
            (record.id, f"{record.group}({', '.join(record.subjects)})") for record in sets.values()
        ]
        for parameterization in dict.fromkeys(record.parameterization for record in sets.values()):
            holders.append((parameterization, self._backend.label(parameterization)))
        return tuple(self._backend.validities(kind, holders))
