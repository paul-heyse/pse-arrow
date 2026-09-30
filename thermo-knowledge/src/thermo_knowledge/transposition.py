# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""What swapping the subject roles of a slot group means (meta-model section 4.3).

One place answers four questions for the reference evaluator (which orientations may hold a
value), the canonical writer (which orientation is stored, and what the swap does to the values)
and the loader (what a `linear` rule must satisfy):

* `orientations`: the orders in which a parameter set for some subjects may be held;
* `is_diagonal`: whether a transposable group's roles name one instance twice;
* `canonical_orientation`: the stored orientation of a subject tuple and whether the tuple given
  had to be swapped to reach it;
* `swapped_values`: the values of a set after its subjects are swapped: the `reciprocal` slots
  inverted and the `linear` slots multiplied by the declared matrix. Applying it twice gives the
  values back, so it turns the values given in one orientation into those of the other.

The canonical orientation is the one whose first role has the smaller identifier (for a
`permutation_group`, the smallest arrangement over the group). The generated DDL enforces the
same order: `ROW(roles) <= ROW(image)`.
"""

from __future__ import annotations

import math
from collections.abc import Mapping, Sequence
from fractions import Fraction

from thermo_knowledge.declaration import model as m

type Orderable = str | int | bytes
"""A subject compared for orientation: an identifier as text, or a UUID's integer or bytes."""


type Owner = m.SlotGroup | m.Relation
"""What has transposable roles: a slot group (its subject roles) or a relation (its keys)."""


def _roles(owner: Owner) -> list[str]:
    if isinstance(owner, m.SlotGroup):
        return [subject.name for subject in owner.subjects]
    return [key.name for key in owner.keys]


def _positions(owner: Owner) -> dict[str, int]:
    return {name: index for index, name in enumerate(_roles(owner))}


def orientations[T](group: Owner, subjects: tuple[T, ...]) -> list[tuple[tuple[T, ...], bool]]:
    """The orders in which a parameter set of `group` for `subjects` may be held: the one asked
    for (not swapped), then those the group's transposition makes equivalent (swapped)."""
    transposition = group.transposition
    found: list[tuple[tuple[T, ...], bool]] = [(subjects, False)]
    if transposition is None or transposition.rule == "ordered":
        return found
    names = _roles(group)
    if transposition.rule == "permutation_group":
        for arrangement in transposition.permutations:
            moved = list(subjects)
            for position, role in enumerate(transposition.roles):
                moved[names.index(role)] = subjects[names.index(arrangement[position])]
            candidate = tuple(moved)
            if all(candidate != other for other, _ in found):
                found.append((candidate, True))
        return found
    first, second = (names.index(role) for role in transposition.roles)
    swapped = list(subjects)
    swapped[first], swapped[second] = subjects[second], subjects[first]
    if tuple(swapped) != subjects:
        found.append((tuple(swapped), True))
    return found


def is_diagonal[T](group: Owner, subjects: tuple[T, ...]) -> bool:
    """Whether the group forbids the diagonal and its transposed roles name one instance twice."""
    transposition = group.transposition
    if transposition is None or transposition.diagonal != "forbidden":
        return False
    positions = _positions(group)
    named = [subjects[positions[role]] for role in transposition.roles]
    return len(set(named)) != len(named)


def arrangements(
    roles: Sequence[str], generators: Sequence[Sequence[str]]
) -> list[tuple[str, ...]]:
    """Every arrangement of `roles` in the group the `generators` produce, identity first."""
    position = {role: index for index, role in enumerate(roles)}
    identity = tuple(roles)
    seen = {identity}
    pending = [identity]
    while pending:
        current = pending.pop()
        for generator in generators:
            image = tuple(current[position[role]] for role in generator)
            if image not in seen:
                seen.add(image)
                pending.append(image)
    return [identity, *sorted(seen - {identity})]


def canonical_orientation[T: Orderable](
    group: Owner, subjects: tuple[T, ...]
) -> tuple[tuple[T, ...], bool]:
    """The stored orientation of `subjects` and whether it differs from the tuple given."""
    transposition = group.transposition
    if transposition is None or transposition.rule == "ordered":
        return subjects, False
    positions = _positions(group)
    if transposition.rule == "permutation_group":
        roles = transposition.roles
        values = {role: subjects[positions[role]] for role in roles}
        orbit = [
            tuple(values[role] for role in arrangement)
            for arrangement in arrangements(roles, transposition.permutations)
        ]
        best = min(orbit)
        moved = list(subjects)
        for role, value in zip(roles, best, strict=True):
            moved[positions[role]] = value
        canonical = tuple(moved)
        return canonical, canonical != subjects
    first, second = (positions[role] for role in transposition.roles)
    if subjects[first] <= subjects[second]:
        return subjects, False
    swapped = list(subjects)
    swapped[first], swapped[second] = subjects[second], subjects[first]
    return tuple(swapped), True


type Matrix = Sequence[Sequence[int | float]]
"""A `linear` rule's matrix, row by row, as declared."""


def _exact(matrix: Matrix) -> list[list[Fraction]]:
    """The declared numbers as exact rationals: `0.1` is one tenth, not the nearest double."""
    return [[Fraction(str(entry)) for entry in row] for row in matrix]


def is_involution(matrix: Matrix) -> bool:
    """Whether the square `matrix` applied twice gives the identity, in exact rational arithmetic
    on the declared numbers. A swap applied twice must return the values it started from."""
    size = len(matrix)
    if any(len(row) != size for row in matrix) or not all(
        math.isfinite(entry) for row in matrix for entry in row
    ):
        return False
    exact = _exact(matrix)
    return all(
        sum(exact[i][k] * exact[k][j] for k in range(size)) == (1 if i == j else 0)
        for i in range(size)
        for j in range(size)
    )


def swapped_slots(owner: Owner) -> tuple[str, ...]:
    """The slots whose value changes when the subjects are swapped: those a `reciprocal` or a
    `linear` rule names."""
    transposition = owner.transposition
    if transposition is not None and transposition.rule in ("reciprocal", "linear"):
        return transposition.slots
    return ()


def swapped_values(owner: Owner, values: Mapping[str, float]) -> dict[str, float]:
    """`values` (slot name to number, in storage units) of a set held in one orientation, as the
    other orientation holds them. A `reciprocal` slot becomes its reciprocal (an exact zero gives
    a signed infinity; a slot not in `values` is left out); the `linear` slots become the product
    of the declared matrix and the vector of their values, all of which `values` holds; every
    other entry is unchanged."""
    transposition = owner.transposition
    result = dict(values)
    if transposition is None:
        return result
    if transposition.rule == "reciprocal":
        for slot in transposition.slots:
            if slot in values:
                number = float(values[slot])
                result[slot] = math.copysign(math.inf, number) if number == 0 else 1.0 / number
    elif transposition.rule == "linear":
        vector = [float(values[slot]) for slot in transposition.slots]
        for slot, row in zip(transposition.slots, transposition.matrix, strict=True):
            result[slot] = math.fsum(
                entry * number for entry, number in zip(row, vector, strict=True)
            )
    return result


def parity_index(group: Owner, family: m.Family) -> int | None:
    """The position in `family`'s index of the index whose odd values change sign when the
    subjects are swapped (`parity`), or `None` when the group's rule is not parity or names an
    index of another family."""
    transposition = group.transposition
    if transposition is None or transposition.rule != "parity":
        return None
    names = [index.name for index in family.indices]
    if transposition.by in names:
        return names.index(transposition.by)
    return None
