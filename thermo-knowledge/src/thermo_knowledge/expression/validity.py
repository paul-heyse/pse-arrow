# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Where an evaluation point lies with respect to the validity regions of the records it read
(expressions.md section 5).

A record states its validity as regions (`RecordValidity`): each a conjunction of clauses, several
alternatives of one kind a union. A clause is decided from the arguments of the evaluation only
when it limits an observable that exactly one argument of the contract names and is about no
component and no aggregation; every other clause cannot be decided from the arguments.

For one region, a point is

* `INSIDE` when every clause of the region is decidable and holds;
* `OUTSIDE` when every clause is decidable and one fails;
* `UNDETERMINED` when the region has a clause that cannot be decided.

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


def _holds(clause: RegionClause, value: np.ndarray) -> np.ndarray:
    held = np.ones(np.shape(value), dtype=bool)
    if clause.lower is not None:
        held &= value >= clause.lower
    if clause.upper is not None:
        held &= value <= clause.upper
    return held


def _region(
    region: ValidityRegion, observed: Mapping[str, np.ndarray], shape: tuple[int, ...]
) -> np.ndarray | None:
    """Whether each point is inside the region, or `None` when a clause cannot be decided."""
    inside = np.ones(shape, dtype=bool)
    for clause in region.clauses:
        if (
            clause.component is not None
            or clause.aggregation is not None
            or clause.observable not in observed
        ):
            return None
        inside &= _holds(clause, observed[clause.observable])
    return inside


def membership(
    records: Sequence[RecordValidity],
    observed: Mapping[str, np.ndarray],
    shape: tuple[int, ...],
) -> np.ndarray:
    """The `Membership` of each point, as an array of `shape` of integer codes.

    `observed` gives, for each observable that exactly one argument of the contract names, that
    argument's values (arrays broadcastable to `shape`)."""
    stated = [record for record in records if record.regions]
    if not stated:
        return np.full(shape, Membership.NOT_STATED, dtype=np.int8)
    outside = np.zeros(shape, dtype=bool)
    undetermined = np.zeros(shape, dtype=bool)
    for record in stated:
        found = [_region(region, observed, shape) for region in record.regions]
        inside_any = np.zeros(shape, dtype=bool)
        for inside in found:
            if inside is not None:
                inside_any |= inside
        undecided = any(inside is None for inside in found)
        outside |= ~inside_any & (not undecided)
        undetermined |= ~inside_any & undecided
    codes = np.full(shape, Membership.INSIDE, dtype=np.int8)
    codes[undetermined] = Membership.UNDETERMINED
    codes[outside] = Membership.OUTSIDE
    return codes


def counts(codes: np.ndarray) -> dict[Membership, int]:
    """How many points are in each membership."""
    return {member: int(np.count_nonzero(codes == member)) for member in Membership}
