# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Invariants declared `enforced = "load"` that one row can decide, and the `ddl` checks, of
kinds and of relations.

The declaration names a `load` invariant and documents it; it cannot state it. The evaluators
live here, keyed by kind and invariant name. `missing_evaluators` lists every declared `load`
invariant without one, and the writer refuses to start while there is any: an invariant with no
enforcement point is not allowed to exist (meta-model section 3.4).

A `ddl` check is evaluated from the declared `check` (`Requirement.rule` and `attributes`) over
the row, with SQL's `CHECK` semantics: a null operand satisfies the check.
"""

from __future__ import annotations

from collections.abc import Callable, Mapping

from thermo_knowledge import pipeline_contract as pc
from thermo_knowledge.canonical.values import Converted
from thermo_knowledge.declaration import model as m

type Row = Mapping[str, Converted]
type Evaluator = Callable[[Row], str | None]
"""Returns a description of the violation, or `None` when the row satisfies the invariant."""


def _nested_has_both(row: Row) -> str | None:
    parent, slot = row.get(pc.PARAMETER_SET.parent), row.get(pc.PARAMETER_SET.parent_slot)
    if (parent is None) != (slot is None):
        return "a nested set names both its parent and the parent's slot; a top-level set names neither"
    return None


def _dilution_needs_solvent_and_scale(row: Row) -> str | None:
    state = pc.STANDARD_STATE
    if row.get(state.kind) == "infinite_dilution" and (
        row.get(state.solvent) is None or row.get(state.scale) is None
    ):
        return "an infinite-dilution standard state names a solvent and a composition scale"
    return None


def _constraint_has_constant(row: Row) -> str | None:
    column = pc.DATASET_COLUMN
    has_constant = row.get(column.constant) is not None
    if (row.get(column.role) == "constraint") != has_constant:
        return "a constraint column states its constant; variable and property columns do not"
    return None


def _uncertainty_needs_slot(row: Row) -> str | None:
    parameter = pc.FIT_FREE_PARAMETER
    if row.get(parameter.standard_uncertainty) is not None and row.get(parameter.slot) is None:
        return "a standard uncertainty is stated only for a parameter that names its slot"
    return None


def _key_follows_doi(row: Row) -> str | None:
    """A publication's key is `doi:<doi>` when it has a DOI, else `carrier:<manifest id>:<key>`."""
    key, doi = row.get(pc.SOURCE.key), row.get(pc.PUBLICATION.doi)
    if doi is not None:
        if key != f"doi:{doi}":
            return f"a publication with the DOI {doi!r} has the key `doi:{doi}`, not {key!r}"
        if doi != str(doi).lower():
            return f"the DOI {doi!r} is stated in lower case"
        return None
    parts = str(key).split(":", 2)
    if len(parts) != 3 or parts[0] != "carrier" or not parts[1] or not parts[2]:
        return (
            f"a publication without a DOI has the key `carrier:<manifest id>:<citation key>`, "
            f"not {key!r}"
        )
    return None


LOAD_INVARIANTS: dict[tuple[str, str], Evaluator] = {
    (pc.PUBLICATION.declared, "key_follows_doi"): _key_follows_doi,
    (pc.PARAMETER_SET.declared, "nested_has_both"): _nested_has_both,
    (
        pc.STANDARD_STATE.declared,
        "dilution_needs_solvent_and_scale",
    ): _dilution_needs_solvent_and_scale,
    (pc.DATASET_COLUMN.declared, "constraint_has_constant"): _constraint_has_constant,
    (pc.FIT_FREE_PARAMETER.declared, "uncertainty_needs_slot"): _uncertainty_needs_slot,
}


def missing_evaluators(decl: m.Declaration) -> list[tuple[str, str]]:
    """Every (kind or relation, invariant) the declaration enforces at load for which no
    evaluator exists."""
    owners: list[tuple[str, tuple[m.Requirement, ...]]] = [
        *((kind.name, kind.requires) for kind in decl.kinds.values()),
        *((relation.name, relation.requires) for relation in decl.relations.values()),
    ]
    return sorted(
        (owner, requirement.name)
        for owner, requires in owners
        for requirement in requires
        if requirement.enforced == "load" and (owner, requirement.name) not in LOAD_INVARIANTS
    )


def violation(owner: str, requirement: m.Requirement, row: Row) -> str | None:
    """Why `row` of the kind or relation `owner` fails `requirement` where the writer enforces it
    (`ddl` and `load`), or `None`. A `verify` requirement is enforced by the verify stage."""
    if requirement.enforced == "ddl":
        return ddl_violation(requirement, row)
    if requirement.enforced == "load":
        return LOAD_INVARIANTS[(owner, requirement.name)](row)
    return None


def ddl_violation(requirement: m.Requirement, row: Row) -> str | None:
    """Why `row` fails the `ddl` check of `requirement`, or `None`."""
    values = [row.get(name) for name in requirement.attributes]
    rule = requirement.rule
    if rule == "one_of_present":
        present = sum(value is not None for value in values)
        if present != 1:
            return (
                f"`{requirement.name}`: exactly one of {', '.join(requirement.attributes)} "
                f"is present, found {present}"
            )
        return None
    if rule == "ordered":
        lower, upper = values
        if lower is not None and upper is not None and lower > upper:  # type: ignore[operator]
            return f"`{requirement.name}`: {requirement.attributes[0]} exceeds {requirement.attributes[1]}"
        return None
    if rule == "present_iff":
        value, state = values
        column, when = requirement.attributes
        if state is None:
            return None  # a null condition satisfies the check, as SQL's CHECK does
        wanted = state in requirement.members
        if (value is not None) != wanted:
            listed = ", ".join(requirement.members)
            return (
                f"`{requirement.name}`: {column} is "
                f"{'absent' if wanted else 'present'} but {when} is `{state}` "
                f"({column} is present exactly when {when} is one of {listed})"
            )
        return None
    (value,) = values
    if value is None:
        return None
    column = requirement.attributes[0]
    if rule == "nonempty" and value == "":
        return f"`{requirement.name}`: {column} is empty"
    if rule == "positive" and not value > 0:  # type: ignore[operator]
        return f"`{requirement.name}`: {column} is not positive"
    if rule == "nonnegative" and value < 0:  # type: ignore[operator]
        return f"`{requirement.name}`: {column} is negative"
    if rule == "within":
        if requirement.lower is not None and value < requirement.lower:  # type: ignore[operator]
            return f"`{requirement.name}`: {column} {value!r} is below {requirement.lower!r}"
        if requirement.upper is not None and value > requirement.upper:  # type: ignore[operator]
            return f"`{requirement.name}`: {column} {value!r} is above {requirement.upper!r}"
    return None
