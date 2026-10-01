# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""What swapping the subject roles of a slot group means (meta-model section 4.3).

One place answers the questions of the reference evaluator's parameter sources, the canonical
writer and the loader:

* `orientations`: the orders in which a parameter set for some subjects may be held;
* `is_diagonal`: whether a transposable group's roles name one instance twice;
* `canonical_orientation`: the stored orientation of a subject tuple and whether the tuple given
  had to be swapped to reach it (the subject columns of a row, its uniqueness and its
  `subject_key` depend on it and on nothing else);
* `stores_arrangement`, `arrangement_count`, `arrangement_of` and `asserted_order`: the integer
  `arrangement` a row of a group or relation whose rule changes values records, which says for
  which order of the subjects the stored values were asserted;
* `read_slots` and `read_rows`: the values of a stored set for the order a reader asks for. They
  are the only place a rule acts on a number: the writer never transforms a value, and a
  source returns the stored numbers unchanged when the order asked for is the order asserted.

The canonical orientation is the one whose first role has the smaller identifier (for a
`permutation_group`, the smallest arrangement over the group). The generated DDL enforces the
same order: `ROW(roles) <= ROW(image)`.
"""

from __future__ import annotations

import math
from collections.abc import Mapping, Sequence
from fractions import Fraction
from typing import Protocol, Self

from thermo_knowledge.declaration import model as m

class Orderable(Protocol):
    """A subject compared for orientation: an identifier as text, a UUID, an integer or bytes."""

    def __lt__(self, other: Self, /) -> bool: ...

    def __le__(self, other: Self, /) -> bool: ...


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
        for arrangement in arrangements(transposition.roles, transposition.permutations)[1:]:
            candidate = _arranged(names, transposition.roles, arrangement, subjects)
            if all(candidate != other for other, _ in found):
                found.append((candidate, True))
        return found
    first, second = (names.index(role) for role in transposition.roles)
    swapped = list(subjects)
    swapped[first], swapped[second] = subjects[second], subjects[first]
    if tuple(swapped) != subjects:
        found.append((tuple(swapped), True))
    return found


def _arranged[T](
    names: Sequence[str],
    roles: Sequence[str],
    arrangement: Sequence[str],
    subjects: tuple[T, ...],
) -> tuple[T, ...]:
    """`subjects` with the subjects of `roles` read in `arrangement`: the subject now in the
    k-th of `roles` is the one the k-th role of `arrangement` held."""
    moved = list(subjects)
    for role, source in zip(roles, arrangement, strict=True):
        moved[names.index(role)] = subjects[names.index(source)]
    return tuple(moved)


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


class TranspositionError(ValueError):
    """A stored set cannot be read in the other order: a slot its rule needs is not held."""


def swapped_values(owner: Owner, values: Mapping[str, float]) -> dict[str, float]:
    """`values` (slot name to number, in storage units) of a set asserted in one order, as the
    other order holds them. A `reciprocal` slot becomes its reciprocal (an exact zero gives a
    signed infinity; a slot not in `values` is left out); the `linear` slots become the product
    of the declared matrix and the vector of their values, all of which `values` must hold;
    every other entry is unchanged."""
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
        missing = [slot for slot in transposition.slots if slot not in values]
        if missing:
            raise TranspositionError(
                f"holds no value for slot `{missing[0]}`, which the swap of its subjects needs "
                f"for `{transposition.slots[0]}`"
            )
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


# -- the arrangement a row records ----------------------------------------------------------------

VALUE_RULES = ("reciprocal", "parity", "linear", "permutation_group")
"""The rules whose stored values depend on the order the source asserted the subjects in."""


def stores_arrangement(owner: Owner) -> bool:
    """Whether a row of `owner` records the `arrangement` of its subjects: its rule is one that
    acts on values (`reciprocal`, `parity`, `linear`) or keeps the asserted order of a
    `permutation_group`. `symmetric` and `ordered` have no such column."""
    transposition = owner.transposition
    return transposition is not None and transposition.rule in VALUE_RULES


def group_arrangements(transposition: m.Transposition) -> list[tuple[str, ...]]:
    """The non-identity arrangements of a `permutation_group`, numbered from one: the declared
    permutations in declared order, then the other arrangements of the group they generate, in
    sorted order (a declared list need not be closed)."""
    declared = [tuple(permutation) for permutation in transposition.permutations]
    identity = tuple(transposition.roles)
    ordered = [item for item in dict.fromkeys(declared) if item != identity]
    rest = [
        item
        for item in arrangements(transposition.roles, transposition.permutations)[1:]
        if item not in ordered
    ]
    return [*ordered, *rest]


def arrangement_count(owner: Owner) -> int:
    """How many arrangements a row of `owner` can record, `0` (the canonical order) included:
    the range of its `arrangement` column is `0` to this number less one."""
    transposition = owner.transposition
    assert transposition is not None and transposition.rule in VALUE_RULES
    if transposition.rule == "permutation_group":
        return 1 + len(group_arrangements(transposition))
    return 2


def arrangement_of[T](owner: Owner, asserted: tuple[T, ...], canonical: tuple[T, ...]) -> int:
    """The arrangement of a set asserted for the subjects `asserted` and stored for `canonical`:
    `0` when they are the same order; for a two-role rule `1`, the swapped order; for a
    `permutation_group` the 1-based number of the arrangement (see `group_arrangements`) that
    takes the asserted order to the canonical one."""
    if asserted == canonical:
        return 0
    transposition = owner.transposition
    assert transposition is not None and transposition.rule in VALUE_RULES
    if transposition.rule != "permutation_group":
        return 1
    names = _roles(owner)
    for number, arrangement in enumerate(group_arrangements(transposition), start=1):
        if _arranged(names, transposition.roles, arrangement, asserted) == canonical:
            return number
    raise ValueError("the asserted subjects are not an arrangement of the canonical ones")


def asserted_order[T](owner: Owner, canonical: tuple[T, ...], arrangement: int) -> tuple[T, ...]:
    """The order of the subjects a row stored as `canonical` with `arrangement` was asserted
    for: the inverse of `arrangement_of`."""
    if arrangement == 0:
        return canonical
    transposition = owner.transposition
    assert transposition is not None and transposition.rule in VALUE_RULES
    names = _roles(owner)
    if transposition.rule != "permutation_group":
        first, second = (names.index(role) for role in transposition.roles)
        swapped = list(canonical)
        swapped[first], swapped[second] = canonical[second], canonical[first]
        return tuple(swapped)
    chosen = group_arrangements(transposition)[arrangement - 1]
    moved = list(canonical)
    for role, source in zip(transposition.roles, chosen, strict=True):
        moved[names.index(source)] = canonical[names.index(role)]
    return tuple(moved)


# -- reading a stored set in the order a reader asks for --------------------------------------------


def read_slots[T](
    owner: Owner, values: Mapping[str, float], asserted: tuple[T, ...], asked: tuple[T, ...]
) -> dict[str, float]:
    """The slot values of a set stored with `values` for a set asserted for `asserted`, as a
    reader asking for `asked` reads them. The order asked for is the order asserted: the stored
    numbers, unchanged. Otherwise the rule of `owner` acts once (`swapped_values`)."""
    if asked == asserted:
        return dict(values)
    return swapped_values(owner, values)


def read_rows[T](
    owner: Owner,
    family: m.Family,
    rows: Mapping[tuple[int, ...], Mapping[str, float]],
    asserted: tuple[T, ...],
    asked: tuple[T, ...],
) -> dict[tuple[int, ...], dict[str, float]]:
    """The rows of `family` of a set asserted for `asserted`, as a reader asking for `asked` reads
    them: unchanged for the order asserted; for another order the rows whose index is odd under
    `parity` change sign."""
    by = None if asked == asserted else parity_index(owner, family)
    result: dict[tuple[int, ...], dict[str, float]] = {}
    for key, row in rows.items():
        negate = by is not None and bool(key[by] % 2)
        result[key] = {name: -float(number) if negate else number for name, number in row.items()}
    return result
