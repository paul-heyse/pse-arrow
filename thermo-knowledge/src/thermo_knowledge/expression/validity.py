# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Where an evaluation point lies with respect to the validity regions of the records it read
(expressions.md section 5).

A record states its validity as regions (`RecordValidity`): each a conjunction of clauses, several
alternatives of one kind a union. A clause is decided from the arguments of the evaluation only
when it limits an observable that exactly one argument of the contract names and is about no
component and no aggregation; every other clause cannot be decided from the arguments. A bound
stated as an offset from another observable of the subject (`lower_relative_to`,
`upper_relative_to`) is decided only when the evaluation has a value of that observable
(`references`); without one the clause cannot be decided.

For one region, a point is

* `OUTSIDE` when a decidable clause fails, whatever the clauses that cannot be decided say (a
  conjunction with a false conjunct is false);
* `UNDETERMINED` when no decidable clause fails and some clause cannot be decided;
* `INSIDE` when every clause is decidable and holds.

For one record, which states regions of the kind, a point is inside when it is inside some region,
otherwise undetermined when some region is, otherwise outside. For an evaluation, which reads
several records, it is outside when it is outside a record that states regions, else undetermined
when undetermined for one, else inside. When no record read states a region of the kind the answer
is `NOT_STATED`. A membership never refuses an evaluation: it is independent of permission to
extrapolate, and what to do outside is the caller's decision.
"""

from __future__ import annotations

import enum
from collections.abc import Mapping, Sequence

import numpy as np

from thermo_knowledge.expression.parameters import RecordValidity, RegionClause, ValidityRegion


class Membership(enum.IntEnum):
    """Where a point lies with respect to the validity regions of one kind."""

    INSIDE = 0
    OUTSIDE = 1
    UNDETERMINED = 2
    NOT_STATED = 3


def _holds(
    clause: RegionClause, value: np.ndarray, references: Mapping[str, np.ndarray]
) -> np.ndarray | None:
    """Where `value` lies in the closed interval of the clause, or `None` when a bound is an
    offset from an observable `references` has no value of."""
    held = np.ones(np.shape(value), dtype=bool)
    for bound, reference, below in (
        (clause.lower, clause.lower_relative_to, True),
        (clause.upper, clause.upper_relative_to, False),
    ):
        if bound is None:
            continue
        if reference is not None:
            if reference not in references:
                return None
            bound = references[reference] + bound
        held &= (value >= bound) if below else (value <= bound)
    return held


def _region(
    region: ValidityRegion,
    observed: Mapping[str, np.ndarray],
    references: Mapping[str, np.ndarray],
    shape: tuple[int, ...],
) -> tuple[np.ndarray, bool]:
    """For each point whether a decidable clause of the region fails, and whether the region has a
    clause that cannot be decided."""
    fails = np.zeros(shape, dtype=bool)
    undecided = False
    for clause in region.clauses:
        if (
            clause.component is not None
            or clause.aggregation is not None
            or clause.observable not in observed
        ):
            undecided = True
            continue
        held = _holds(clause, observed[clause.observable], references)
        if held is None:
            undecided = True
            continue
        fails |= ~held
    return fails, undecided


def membership(
    records: Sequence[RecordValidity],
    observed: Mapping[str, np.ndarray],
    shape: tuple[int, ...],
    references: Mapping[str, np.ndarray] | None = None,
) -> np.ndarray:
    """The `Membership` of each point, as an array of `shape` of integer codes.

    `observed` gives, for each observable that exactly one argument of the contract names, that
    argument's values (arrays broadcastable to `shape`). `references` gives the subject's value
    of each observable a bound may be relative to."""
    known = references or {}
    stated = [record for record in records if record.regions]
    if not stated:
        return np.full(shape, Membership.NOT_STATED, dtype=np.int8)
    outside = np.zeros(shape, dtype=bool)
    undetermined = np.zeros(shape, dtype=bool)
    for record in stated:
        inside_any = np.zeros(shape, dtype=bool)
        undetermined_any = np.zeros(shape, dtype=bool)
        for fails, undecided in (
            _region(region, observed, known, shape) for region in record.regions
        ):
            inside_any |= ~fails & (not undecided)
            undetermined_any |= ~fails & undecided
        undetermined |= ~inside_any & undetermined_any
        outside |= ~inside_any & ~undetermined_any
    codes = np.full(shape, Membership.INSIDE, dtype=np.int8)
    codes[undetermined] = Membership.UNDETERMINED
    codes[outside] = Membership.OUTSIDE
    return codes


def counts(codes: np.ndarray) -> dict[Membership, int]:
    """How many points are in each membership."""
    return {member: int(np.count_nonzero(codes == member)) for member in Membership}
