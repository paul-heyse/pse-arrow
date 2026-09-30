# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Where the evaluator reads parameter values (expressions.md section 5).

A `ParameterSource` answers for a slot group, by its qualified name `form.group`, and an ordered
tuple of subjects: the slot values (in storage units), the rows of a family, a nested or
referenced set, the convention facts of the parameterizations that supplied the sets read, and,
for a sub-form slot, the form chosen together with that form's own source. A subject is the
opaque identifier of an entity, as text. The source returns `None` for what it does not hold; the
evaluator refuses, it never substitutes a zero.

**Orientation.** A source returns the values for the order of the subjects it is asked for. A
set is held for the order its source asserted; asked for another order of a transposable group,
the source applies the group's rule once (`thermo_knowledge.transposition`), and asked for the
order asserted it returns the stored numbers unchanged. The evaluator applies no rule itself.

`InMemorySource` is the implementation for tests; `qualify.source.DatabaseSource` reads the built
database.
"""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass, field
from typing import Protocol

from thermo_knowledge import transposition
from thermo_knowledge.declaration import model as m

type Subject = str
"""The identifier of one entity (a component, a reaction), as text; never parsed."""
type SlotValues = Mapping[str, float]
type FamilyRows = Mapping[tuple[int, ...], SlotValues]
"""The rows of a family: its index tuple to the slot values of that row."""
type SetRead = tuple[str, tuple[Subject, ...]]
"""A set an evaluation read: its slot group (`form.group`) and the subjects as asked for."""


@dataclass(frozen=True)
class FormChoice:
    """A form chosen for a sub-form slot or held by a nested or referenced set, and where its own
    parameter sets are read.

    A set held by a slot also names its slot group (`form.group`) and its `subjects`, in the
    role order of the group. A call through a referenced set does not name the subjects of the
    set it reaches: the roles of the called contract that it does not give are those subjects,
    bound as the group binds them."""

    form: str
    source: ParameterSource
    group: str | None = None
    subjects: tuple[Subject, ...] = ()


@dataclass(frozen=True)
class ConventionFact:
    """One convention fact of one parameterization: its value in storage units, or `None` with
    `absent` saying why the parameterization has none (it has no convention set, or its
    convention set states no such fact)."""

    parameterization: str
    value: float | None
    absent: str = ""


class ParameterSource(Protocol):
    def slot_values(self, group: str, subjects: tuple[Subject, ...]) -> SlotValues | None:
        """The slot values of the parameter set of slot group `group` for `subjects`, as held for
        that order of them, or `None` when there is none in any order the group's transposition
        makes equivalent."""
        ...

    def default_slot_values(self, group: str, subjects: tuple[Subject, ...]) -> SlotValues | None:
        """The values a declared default rule gives for `subjects` when no set is held in any
        orientation, or `None` when no rule applies: the subjects are then required."""
        ...

    def family_rows(
        self, group: str, family: str, subjects: tuple[Subject, ...]
    ) -> FamilyRows | None:
        """The rows of `family` of `group` for `subjects`, as held for that order of them, or
        `None`."""
        ...

    def nested_set(
        self, group: str, slot: str, subjects: tuple[Subject, ...], index: tuple[int, ...] = ()
    ) -> FormChoice | None:
        """The form and source of the set that `slot` of `group` holds for `subjects`: a set
        nested in the holder's or an independently identified set it references. A slot of a
        family is named `family.slot` and `index` is the index of the row that holds it. `None`
        when there is none."""
        ...

    def subform_choices(self, slot: str, subjects: tuple[Subject, ...]) -> tuple[FormChoice, ...]:
        """The forms chosen for sub-form slot `slot` (`form.slot`) for `subjects`: the subject of
        each role of the accepted contract when the slot is chosen per subject, none when per
        model. One for `one`, none or one for `optional`, any number for `many`."""
        ...

    def convention_facts(self, name: str, reads: tuple[SetRead, ...]) -> tuple[ConventionFact, ...]:
        """The convention fact `name` of each parameterization that supplied one of the sets
        `reads` (slot group and subjects as asked for) to this source, once each; with no read,
        of the parameterization the source is in force for."""
        ...


@dataclass
class InMemorySource:
    """A `ParameterSource` over dictionaries.

    `slots` is keyed by (group, subjects), `families` by (group, family, subjects), `nested` by
    (group, slot, subjects) and, for a slot of a family, by (group, `family.slot`, subjects,
    index); `subforms` by (slot, subjects). A set is held for the subjects as written in its key
    (the order its source asserted). Given the `declaration`, a set asked for in another order a
    group's transposition makes equivalent is read through the rule; without it only the order
    written is found. `defaults` is the declared default rule, given explicitly: the slot values
    used for any subject tuple of a group that has no set in any orientation. `conventions` are
    the convention facts of the parameterization the source stands for, named `parameterization`.
    """

    slots: Mapping[tuple[str, tuple[Subject, ...]], SlotValues] = field(default_factory=dict)
    families: Mapping[tuple[str, str, tuple[Subject, ...]], FamilyRows] = field(
        default_factory=dict
    )
    nested: Mapping[
        tuple[str, str, tuple[Subject, ...]] | tuple[str, str, tuple[Subject, ...], tuple[int, ...]],
        FormChoice,
    ] = field(default_factory=dict)
    subforms: Mapping[tuple[str, tuple[Subject, ...]], tuple[FormChoice, ...]] = field(
        default_factory=dict
    )
    defaults: Mapping[str, SlotValues] = field(default_factory=dict)
    declaration: m.Declaration | None = None
    conventions: Mapping[str, float] = field(default_factory=dict)
    parameterization: str = "(in memory)"

    def _group(self, group: str) -> m.SlotGroup | None:
        if self.declaration is None:
            return None
        return next((g for g in self.declaration.slot_groups if g.qualified == group), None)

    def _orders(self, group: str, subjects: tuple[Subject, ...]) -> list[tuple[Subject, ...]]:
        """The orders a set for `subjects` may be held in: as asked, then the equivalent ones."""
        found = self._group(group)
        if found is None:
            return [subjects]
        return [candidate for candidate, _ in transposition.orientations(found, subjects)]

    def slot_values(self, group: str, subjects: tuple[Subject, ...]) -> SlotValues | None:
        for asserted in self._orders(group, subjects):
            held = self.slots.get((group, asserted))
            if held is not None:
                owner = self._group(group)
                return held if owner is None else transposition.read_slots(
                    owner, held, asserted, subjects
                )
        return None

    def default_slot_values(self, group: str, subjects: tuple[Subject, ...]) -> SlotValues | None:
        return self.defaults.get(group)

    def family_rows(
        self, group: str, family: str, subjects: tuple[Subject, ...]
    ) -> FamilyRows | None:
        for asserted in self._orders(group, subjects):
            held = self.families.get((group, family, asserted))
            if held is not None:
                owner = self._group(group)
                declared = None if owner is None else next(
                    (f for f in owner.families if f.name == family), None
                )
                if owner is None or declared is None:
                    return held
                return transposition.read_rows(owner, declared, held, asserted, subjects)
        return None

    def nested_set(
        self, group: str, slot: str, subjects: tuple[Subject, ...], index: tuple[int, ...] = ()
    ) -> FormChoice | None:
        for asserted in self._orders(group, subjects):
            found = self.nested.get((group, slot, asserted, index) if index else (group, slot, asserted))
            if found is not None:
                return found
        return None

    def subform_choices(self, slot: str, subjects: tuple[Subject, ...]) -> tuple[FormChoice, ...]:
        return self.subforms.get((slot, subjects), ())

    def convention_facts(self, name: str, reads: tuple[SetRead, ...]) -> tuple[ConventionFact, ...]:
        value = self.conventions.get(name)
        if value is None:
            return (
                ConventionFact(
                    self.parameterization, None, f"states no convention fact `{name}`"
                ),
            )
        return (ConventionFact(self.parameterization, float(value)),)
