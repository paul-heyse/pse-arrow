# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Where the evaluator reads parameter values (expressions.md section 5).

A `ParameterSource` answers for a slot group, by its qualified name `form.group`, and an ordered
tuple of subjects: the slot values (in storage units), the rows of a family, a nested set, and,
for a sub-form slot, the form chosen together with that form's own source. A subject is the
opaque identifier of an entity, as text. The source returns `None` for what it does not hold;
the evaluator refuses, it never substitutes a zero.

`InMemorySource` is the implementation for tests. A database-backed one is a later packet.
"""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass, field
from typing import Protocol

type Subject = str
"""The identifier of one entity (a component, a reaction), as text; never parsed."""
type SlotValues = Mapping[str, float]
type FamilyRows = Mapping[tuple[int, ...], SlotValues]
"""The rows of a family: its index tuple to the slot values of that row."""


@dataclass(frozen=True)
class FormChoice:
    """A form chosen for a sub-form slot or held by a nested set, and where its own parameter
    sets are read."""

    form: str
    source: ParameterSource


class ParameterSource(Protocol):
    def slot_values(self, group: str, subjects: tuple[Subject, ...]) -> SlotValues | None:
        """The slot values of the parameter set of slot group `group` for exactly `subjects`
        (in that orientation), or `None` when there is none."""
        ...

    def default_slot_values(self, group: str, subjects: tuple[Subject, ...]) -> SlotValues | None:
        """The values a declared default rule gives for `subjects` when no set is held in any
        orientation, or `None` when no rule applies: the subjects are then required."""
        ...

    def family_rows(
        self, group: str, family: str, subjects: tuple[Subject, ...]
    ) -> FamilyRows | None:
        """The rows of `family` of `group` for exactly `subjects`, or `None`."""
        ...

    def nested_set(self, group: str, slot: str, subjects: tuple[Subject, ...]) -> FormChoice | None:
        """The form and source of the nested set held by `slot` of `group` for exactly
        `subjects`, or `None`."""
        ...

    def subform_choices(self, slot: str, subjects: tuple[Subject, ...]) -> tuple[FormChoice, ...]:
        """The forms chosen for sub-form slot `slot` (`form.slot`) for `subjects`: the subject of
        each role of the accepted contract when the slot is chosen per subject, none when per
        model. One for `one`, none or one for `optional`, any number for `many`."""
        ...


@dataclass
class InMemorySource:
    """A `ParameterSource` over dictionaries.

    `slots` is keyed by (group, subjects), `families` by (group, family, subjects), `nested` by
    (group, slot, subjects) and `subforms` by (slot, subjects). `defaults` is the declared
    default rule, given explicitly: the slot values used for any subject tuple of a group that
    has no set in any orientation.
    """

    slots: Mapping[tuple[str, tuple[Subject, ...]], SlotValues] = field(default_factory=dict)
    families: Mapping[tuple[str, str, tuple[Subject, ...]], FamilyRows] = field(
        default_factory=dict
    )
    nested: Mapping[tuple[str, str, tuple[Subject, ...]], FormChoice] = field(default_factory=dict)
    subforms: Mapping[tuple[str, tuple[Subject, ...]], tuple[FormChoice, ...]] = field(
        default_factory=dict
    )
    defaults: Mapping[str, SlotValues] = field(default_factory=dict)

    def slot_values(self, group: str, subjects: tuple[Subject, ...]) -> SlotValues | None:
        return self.slots.get((group, subjects))

    def default_slot_values(self, group: str, subjects: tuple[Subject, ...]) -> SlotValues | None:
        return self.defaults.get(group)

    def family_rows(
        self, group: str, family: str, subjects: tuple[Subject, ...]
    ) -> FamilyRows | None:
        return self.families.get((group, family, subjects))

    def nested_set(self, group: str, slot: str, subjects: tuple[Subject, ...]) -> FormChoice | None:
        return self.nested.get((group, slot, subjects))

    def subform_choices(self, slot: str, subjects: tuple[Subject, ...]) -> tuple[FormChoice, ...]:
        return self.subforms.get((slot, subjects), ())
