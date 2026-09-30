# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""`qualification/<case>.toml`: one qualification case (pipeline section 5.1).

The file is decoded into the structs below (an unknown key is refused) and checked against the
declaration: the form, its contract output and the comparison basis exist, the subject slot
group binds every role of the contract, every argument has a grid, units convert and the
sub-form choices name forms that implement what their slot accepts. `validate` returns every
problem found; a case that has any is refused before anything is evaluated.
"""

from __future__ import annotations

import hashlib
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

import msgspec
from msgspec import Struct

from thermo_knowledge import pipeline_contract as pc
from thermo_knowledge.canonical.values import ValueRefused, convert, storage_unit
from thermo_knowledge.declaration import model as m

CASES_DIR = "qualification"
"""Where the cases are, relative to the tree."""
_STRICT = {"forbid_unknown_fields": True}


class CaseError(Exception):
    """A case file that cannot be read or that the declaration refuses."""

    def __init__(self, path: Path | str, problems: list[str]) -> None:
        self.path = str(path)
        self.problems = tuple(problems)
        super().__init__("\n".join(f"{path}: {problem}" for problem in problems))


class ParameterizationSpec(Struct, **_STRICT):
    """A parameterization by key and revision. `{pin}` in the revision stands for the resolved
    pin of the carrier named by `carrier`. `occurrence` chooses, for a subject that the
    parameterization holds several sets for (repeated assertions of the source), the set of that
    occurrence; without it such a subject is refused."""

    key: str
    revision: str
    carrier: str | None = None
    occurrence: int | None = None


class LibraryKey(Struct, **_STRICT):
    """How the library's own key for a subject is obtained: the local key of the source entity
    of this carrier and scope whose target is the subject."""

    carrier: str
    scope: str


class SubjectsSpec(Struct, **_STRICT):
    """The subjects of a case: the sets of one slot group in the parameterization.

    `select` is `all`, a declared `list` (`keys` are the library's own keys) or a `sample` of
    `size` subjects drawn with the fixed `seed`."""

    group: str
    library_key: LibraryKey
    select: Literal["all", "list", "sample"] = "all"
    keys: list[str] = []
    size: int | None = None
    seed: int | None = None


class ArgumentSpec(Struct, **_STRICT):
    """The grid of one argument: explicit `values` (in `unit`), or `points` evenly spread
    within each set's envelope on `observable`, leaving `inset` (a fraction of the range) at
    each end."""

    values: list[float] | None = None
    unit: str | None = None
    points: int | None = None
    within: Literal["envelope"] | None = None
    observable: str | None = None
    envelope_kind: str | None = None
    inset: float = 0.0


class AbsoluteTolerance(Struct, **_STRICT):
    value: float
    unit: str


class ComparisonSpec(Struct, **_STRICT):
    """How points are compared: a point agrees when its relative deviation is within
    `relative_tolerance`, or its absolute deviation within `absolute_tolerance` when given.
    `invalid_points` says what a point the library answers with NaN or an error does: make the
    run `fail`, or be `exclude`d for the `invalid_reason` stated."""

    relative_tolerance: float
    absolute_tolerance: AbsoluteTolerance | None = None
    invalid_points: Literal["fail", "exclude"] = "fail"
    invalid_reason: str | None = None


class HarnessSpec(Struct, **_STRICT):
    """The oracle harness: `oracles/<library>.py`, run in `environment` (`core`, or a side
    environment of `envs/`), evaluating its `call`."""

    library: str
    call: str
    environment: str = "core"


class SubformChoiceSpec(Struct, **_STRICT):
    form: str
    parameterization: ParameterizationSpec


class CaseSpec(Struct, **_STRICT):
    doc: str
    form: str
    output: str
    basis: str
    parameterization: ParameterizationSpec
    subjects: SubjectsSpec
    arguments: dict[str, ArgumentSpec]
    comparison: ComparisonSpec
    harness: HarnessSpec
    subforms: dict[str, list[SubformChoiceSpec]] = {}


@dataclass(frozen=True)
class Case:
    name: str
    path: Path
    spec: CaseSpec
    content_hash: str
    """SHA-256 of the file's bytes."""


def case_names(directory: Path) -> list[str]:
    """The cases under `directory`, by file stem."""
    return sorted(path.stem for path in directory.glob("*.toml")) if directory.is_dir() else []


def load_case(path: Path) -> Case:
    """Decode one case file; raises `CaseError` for unreadable or malformed text."""
    try:
        data = path.read_bytes()
    except OSError as error:
        raise CaseError(path, [f"cannot be read: {error}"]) from error
    try:
        document = tomllib.loads(data.decode("utf-8"))
        spec = msgspec.convert(document, CaseSpec)
    except (tomllib.TOMLDecodeError, UnicodeDecodeError) as error:
        raise CaseError(path, [f"is not valid TOML: {error}"]) from error
    except msgspec.ValidationError as error:
        raise CaseError(path, [str(error)]) from error
    return Case(path.stem, path, spec, hashlib.sha256(data).hexdigest())


def _unit(decl: m.Declaration, field: m.Field) -> str | None:
    return storage_unit(decl, field.type)


def validate(case: Case, decl: m.Declaration) -> list[str]:
    """Every way `case` disagrees with `decl`; empty when it is well formed."""
    spec = case.spec
    problems: list[str] = []
    form = decl.forms.get(spec.form)
    if form is None:
        return [f"`{spec.form}` is not a declared form"]
    contract = decl.contracts[form.implements]
    if not any(o.name == spec.output for o in contract.outputs):
        problems.append(
            f"`{spec.output}` is not an output of contract `{contract.name}` "
            f"({', '.join(o.name for o in contract.outputs)})"
        )
    if spec.basis not in {member.name for member in decl.enums[pc.COMPARISON_BASIS.declared].members}:
        problems.append(f"`{spec.basis}` is not a comparison basis")
    if contract.sets:
        problems.append(
            f"contract `{contract.name}` has index sets ({', '.join(s.name for s in contract.sets)}): "
            "a case evaluates a form for single subjects; model assemblies come later"
        )
    group = next((g for g in form.slot_groups if g.qualified == spec.subjects.group), None)
    if group is None:
        problems.append(f"`{spec.subjects.group}` is not a slot group of form `{form.name}`")
    else:
        bound = {b.target for b in group.bindings if b.kind == "role"}
        for role in contract.roles:
            if role.name not in bound:
                problems.append(
                    f"slot group `{group.qualified}` binds no subject to role `{role.name}` of "
                    f"contract `{contract.name}`"
                )
        if any(b.kind != "role" for b in group.bindings):
            problems.append(f"slot group `{group.qualified}` has a subject bound to a set")
    _validate_arguments(case, decl, contract, problems)
    _validate_comparison(case, decl, contract, problems)
    if spec.subjects.select == "list" and not spec.subjects.keys:
        problems.append('subjects: `select = "list"` needs `keys`')
    if spec.subjects.select == "sample" and (
        spec.subjects.size is None or spec.subjects.size < 1 or spec.subjects.seed is None
    ):
        problems.append('subjects: `select = "sample"` needs `size` (at least 1) and `seed`')
    if spec.subjects.select != "list" and spec.subjects.keys:
        problems.append('subjects: `keys` is for `select = "list"`')
    if spec.subjects.select != "sample" and (
        spec.subjects.size is not None or spec.subjects.seed is not None
    ):
        problems.append('subjects: `size` and `seed` are for `select = "sample"`')
    _validate_pins(case, problems)
    for slot_name, choices in spec.subforms.items():
        slot = next((s for s in form.subforms if s.qualified == slot_name), None)
        if slot is None:
            problems.append(f"subforms: `{slot_name}` is not a sub-form slot of form `{form.name}`")
            continue
        for choice in choices:
            chosen = decl.forms.get(choice.form)
            if chosen is None:
                problems.append(f"subforms.{slot_name}: `{choice.form}` is not a declared form")
            elif chosen.implements != slot.accepts:
                problems.append(
                    f"subforms.{slot_name}: form `{chosen.name}` implements `{chosen.implements}`, "
                    f"the slot accepts `{slot.accepts}`"
                )
    return problems


def _validate_pins(case: Case, problems: list[str]) -> None:
    specs = [("parameterization", case.spec.parameterization)]
    specs += [
        (f"subforms.{slot}", choice.parameterization)
        for slot, choices in case.spec.subforms.items()
        for choice in choices
    ]
    for where, item in specs:
        if item.occurrence is not None and item.occurrence < 1:
            problems.append(f"{where}: `occurrence` counts from 1")
        if "{pin}" in item.revision and item.carrier is None:
            problems.append(
                f"{where}: a revision with `{{pin}}` names the `carrier` whose pin it is"
            )
        if item.carrier is not None and "{pin}" not in item.revision:
            problems.append(f"{where}: `carrier` is for a revision that uses `{{pin}}`")
        leftover = item.revision.replace("{pin}", "")
        if "{" in leftover or "}" in leftover:
            problems.append(f"{where}: a revision may use only the placeholder {{pin}}")


def _validate_arguments(
    case: Case, decl: m.Declaration, contract: m.Contract, problems: list[str]
) -> None:
    declared = {a.name: a for a in contract.arguments}
    for name in declared:
        if name not in case.spec.arguments:
            problems.append(f"arguments: `{name}` of contract `{contract.name}` has no grid")
    for name, grid in case.spec.arguments.items():
        argument = declared.get(name)
        if argument is None:
            problems.append(f"arguments.{name}: not an argument of contract `{contract.name}`")
            continue
        if argument.over:
            problems.append(f"arguments.{name}: an argument indexed over sets is not supported")
            continue
        if (grid.values is None) == (grid.points is None):
            problems.append(f"arguments.{name}: give `values` or `points`, not both or neither")
            continue
        if grid.values is not None:
            if not grid.values:
                problems.append(f"arguments.{name}: `values` is empty")
            target = _unit(decl, argument)
            if grid.unit is None:
                problems.append(f"arguments.{name}: `values` need their `unit`")
            elif target is not None:
                try:
                    convert(1.0, grid.unit, target)
                except ValueRefused as error:
                    problems.append(f"arguments.{name}: {error}")
            if grid.within or grid.observable or grid.envelope_kind or grid.inset:
                problems.append(
                    f"arguments.{name}: `within`, `observable`, `envelope_kind` and `inset` go with `points`"
                )
            continue
        if grid.points is None or grid.points < 1:
            problems.append(f"arguments.{name}: `points` is at least 1")
        if grid.within != "envelope":
            problems.append(f'arguments.{name}: `points` are spread `within = "envelope"`')
        if grid.observable is None:
            problems.append(f"arguments.{name}: `points` within an envelope name the `observable`")
        elif decl.observable_entity(grid.observable) is None:
            problems.append(f"arguments.{name}: `{grid.observable}` is not a declared observable")
        if not 0 <= grid.inset < 0.5:
            problems.append(f"arguments.{name}: `inset` is a fraction of the range in [0, 0.5)")
        if grid.unit is not None:
            problems.append(f"arguments.{name}: `unit` goes with `values`")
        kinds = {member.name for member in decl.enums[pc.ENVELOPE_KIND.declared].members}
        if grid.envelope_kind is not None and grid.envelope_kind not in kinds:
            problems.append(f"arguments.{name}: `{grid.envelope_kind}` is not an envelope kind")


def _validate_comparison(
    case: Case, decl: m.Declaration, contract: m.Contract, problems: list[str]
) -> None:
    comparison = case.spec.comparison
    if not comparison.relative_tolerance > 0:
        problems.append("comparison: `relative_tolerance` is positive")
    output = next((o for o in contract.outputs if o.name == case.spec.output), None)
    absolute = comparison.absolute_tolerance
    if absolute is not None:
        if absolute.value < 0:
            problems.append("comparison: `absolute_tolerance` is not negative")
        if output is None or output.observable is None:
            problems.append(
                "comparison: an absolute tolerance is stated for an output that denotes an "
                "observable, and this one does not"
            )
        else:
            target = _unit(decl, output)
            try:
                if target is not None:
                    convert(1.0, absolute.unit, target)
            except ValueRefused as error:
                problems.append(f"comparison.absolute_tolerance: {error}")
    if comparison.invalid_points == "exclude" and not comparison.invalid_reason:
        problems.append("comparison: excluding invalid points needs the `invalid_reason`")
    if comparison.invalid_points == "fail" and comparison.invalid_reason:
        problems.append('comparison: `invalid_reason` goes with `invalid_points = "exclude"`')
