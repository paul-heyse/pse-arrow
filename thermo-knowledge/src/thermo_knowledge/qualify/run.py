# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Run one qualification case (pipeline section 5): select the subjects and the grid, evaluate
the form from the database through the reference evaluator, ask the harness, compare point by
point, and record the run.

A run ends `passed` (every compared point agrees), `failed`, or `blocked` (it could not be carried
out; the reason is in the note). Points the library answers with NaN or an error are counted and
listed, never dropped: the case declares whether they are excluded, with the reason, or fail the run.
"""

from __future__ import annotations

import hashlib
import itertools
import json
import math
import random
import uuid
from collections.abc import Callable, Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path

import numpy as np
import psycopg
from psycopg import sql

from thermo_knowledge import config, reuse
from thermo_knowledge import pipeline_contract as pc
from thermo_knowledge.build import currency
from thermo_knowledge.canonical import store
from thermo_knowledge.canonical.store import ReadRecords
from thermo_knowledge.canonical.values import convert, storage_unit
from thermo_knowledge.declaration import model as m
from thermo_knowledge.expression.canonical import evaluation_hash
from thermo_knowledge.expression.compiled import CompileCache
from thermo_knowledge.expression.evaluate import EvaluationRefusal, PiecePolicy, bind
from thermo_knowledge.expression.validity import Membership, counts
from thermo_knowledge.generate.fingerprint import declaration_fingerprint
from thermo_knowledge.qualify import harness, persist
from thermo_knowledge.qualify.case import (
    ArgumentSpec,
    Case,
    CaseError,
    ParameterizationSpec,
    validate,
)
from thermo_knowledge.qualify.source import (
    AmbiguousOccurrence,
    DatabaseSource,
    SubformBinding,
    check_database,
    choose_occurrence,
    table_identifier,
)
from thermo_knowledge.schema_build import read_physical

PASSED = pc.RUN_OUTCOME.member("passed")
FAILED = pc.RUN_OUTCOME.member("failed")
BLOCKED = pc.RUN_OUTCOME.member("blocked")

REPORT_SCHEMA = 1
_SHOWN_INVALID = 10


class Blocked(Exception):
    """The run cannot be carried out: `reason` is the `blocked_reason` member the run records and
    the message is recorded as its note."""

    def __init__(self, reason: str, detail: str) -> None:
        if reason not in pc.BLOCKED_REASON.members:
            raise ValueError(
                f"`{reason}` is not a blocked reason: {', '.join(pc.BLOCKED_REASON.members)}"
            )
        self.reason = reason
        super().__init__(detail)


@dataclass(frozen=True)
class Context:
    """The locations and the declaration of one `tk qualify`, so a test can point each at a
    fixture."""

    decl: m.Declaration
    url: str
    canonical: Path
    tree: Path = config.TREE_DIR
    oracles: Path | None = None
    """Where the harnesses are (default: `oracles/` of the tree)."""
    force: bool = False

    @property
    def oracles_dir(self) -> Path:
        return self.oracles if self.oracles is not None else self.tree / harness.ORACLES_DIR


@dataclass(frozen=True)
class Subject:
    set_id: uuid.UUID
    ids: tuple[str, ...]
    """The subject entities, in the order of the slot group's roles."""
    keys: tuple[str, ...]
    """The library's own key of each."""


@dataclass
class SubjectOutcome:
    subject: Subject
    points: int
    passed: int = 0
    failed: int = 0
    invalid: int = 0
    worst: float | None = None
    worst_at: dict[str, float] | None = None
    refused: str | None = None


@dataclass(frozen=True)
class Evaluation:
    """What evaluating the form for the subjects gave: the answer at each subject's points (or
    the refusal that stopped it), the points that were evaluated (those a case excludes for
    lying outside its validity regions are not among them) and, for a case that names a validity
    kind, where every point of the grid lies (`Membership` codes; `None` for a subject the form
    was refused for)."""

    answers: dict[uuid.UUID, np.ndarray | str]
    points: dict[uuid.UUID, dict[str, list[float]]]
    membership: dict[uuid.UUID, np.ndarray | None]

    def counts(self) -> dict[Membership, int]:
        """How many grid points are in each membership, over the subjects that have one."""
        total = {member: 0 for member in Membership}
        for codes in self.membership.values():
            if codes is not None:
                for member, number in counts(codes).items():
                    total[member] += number
        return total


@dataclass
class _State:
    library: str
    version: str | None = None
    """The library's version, once the probe has reported it; `None` until then."""
    sets: tuple[uuid.UUID, ...] = ()


@dataclass(frozen=True)
class CaseOutcome:
    """What `run_case` did: the report (see `persist.REPORT_NAME`), whether the evaluation was
    carried out or found current, and where the output is."""

    case: str
    status: str
    """`ran` or `current`."""
    report: dict[str, object]
    directory: Path

    @property
    def outcome(self) -> str:
        return str(self.report["outcome"])

    @property
    def blocked_reason(self) -> str | None:
        found = self.report.get("blocked_reason")
        return None if found is None else str(found)


# -- selection ---------------------------------------------------------------------------------


def _revision(conn: psycopg.Connection, spec: ParameterizationSpec) -> str:
    if "{pin}" not in spec.revision:
        return spec.revision
    assert spec.carrier is not None
    carrier = pc.CARRIER
    rows = conn.execute(
        sql.SQL("SELECT {resolved_pin} FROM {table} WHERE {manifest_id} = %s").format(
            resolved_pin=sql.Identifier(carrier.resolved_pin),
            table=table_identifier(carrier.table),
            manifest_id=sql.Identifier(carrier.manifest_id),
        ),
        (spec.carrier,),
    ).fetchall()
    if len(rows) != 1:
        raise Blocked(
            "carrier_not_unique",
            f"the carrier `{spec.carrier}` has {len(rows)} acquisitions in the database; "
            "`{pin}` needs exactly one",
        )
    return spec.revision.replace("{pin}", rows[0][0])


def _parameterization(
    conn: psycopg.Connection, spec: ParameterizationSpec
) -> tuple[uuid.UUID, str]:
    revision = _revision(conn, spec)
    pz = pc.PARAMETERIZATION
    row = conn.execute(
        sql.SQL("SELECT id FROM {table} WHERE {key} = %s AND {revision} = %s").format(
            table=table_identifier(pz.table),
            key=sql.Identifier(pz.key),
            revision=sql.Identifier(pz.revision),
        ),
        (spec.key, revision),
    ).fetchone()
    if row is None:
        raise Blocked(
            "parameterization_missing",
            f"the parameterization {spec.key}@{revision} is not in the database",
        )
    return row[0], revision


def _subject_sets(
    conn: psycopg.Connection,
    group: m.SlotGroup,
    parameterization: uuid.UUID,
    occurrence: int | None,
) -> list[tuple[uuid.UUID, tuple[str, ...]]]:
    """The set of `group` for each subject of the parameterization: the stated occurrence, or the
    only one (`AmbiguousOccurrence` when a subject has several and none is stated)."""
    ps = pc.PARAMETER_SET
    columns = [sql.SQL("g.id"), sql.SQL("ps.{}").format(sql.Identifier(ps.occurrence))]
    columns += [sql.SQL("g.{}").format(sql.Identifier(s.name)) for s in group.subjects]
    query = sql.SQL(
        "SELECT {columns} FROM {table} g JOIN {sets} ps ON ps.id = g.id "
        "WHERE ps.{parameterization} = %s AND ps.{parent} IS NULL"
    ).format(
        columns=sql.SQL(", ").join(columns),
        table=sql.Identifier(m.PARAM_SCHEMA, group.id),
        sets=sql.Identifier(*ps.table.split(".")),
        parameterization=sql.Identifier(ps.parameterization),
        parent=sql.Identifier(ps.parent),
    )
    held: dict[tuple[str, ...], dict[int, uuid.UUID]] = {}
    for row in conn.execute(query, (parameterization,)).fetchall():
        held.setdefault(tuple(str(part) for part in row[2:]), {})[int(row[1])] = row[0]
    found: list[tuple[uuid.UUID, tuple[str, ...]]] = []
    for ids in sorted(held):
        chosen = choose_occurrence(group.qualified, ids, held[ids], occurrence)
        if chosen is not None:
            found.append((chosen, ids))
    return found


def _library_keys(
    conn: psycopg.Connection, case: Case, found: Sequence[tuple[uuid.UUID, tuple[str, ...]]]
) -> dict[str, list[str]]:
    wanted = sorted({uuid.UUID(part) for _, ids in found for part in ids})
    se, carrier = pc.SOURCE_ENTITY, pc.CARRIER
    rows = conn.execute(
        sql.SQL(
            "SELECT se.{target}, se.{local_key} FROM {se_table} se "
            "JOIN {carrier_table} c ON c.id = se.{se_carrier} "
            "WHERE c.{manifest_id} = %s AND se.{scope} = %s AND se.{target} = ANY(%s)"
        ).format(
            target=sql.Identifier(se.target),
            local_key=sql.Identifier(se.local_key),
            se_table=table_identifier(se.table),
            carrier_table=table_identifier(carrier.table),
            se_carrier=sql.Identifier(se.carrier),
            manifest_id=sql.Identifier(carrier.manifest_id),
            scope=sql.Identifier(se.scope),
        ),
        (case.spec.subjects.library_key.carrier, case.spec.subjects.library_key.scope, wanted),
    ).fetchall()
    keys: dict[str, list[str]] = {}
    for target, local_key in rows:
        keys.setdefault(str(target), []).append(local_key)
    return {target: sorted(values) for target, values in keys.items()}


def select_subjects(
    conn: psycopg.Connection, case: Case, group: m.SlotGroup, parameterization: uuid.UUID
) -> list[Subject]:
    """The subjects of `case`: the sets of its slot group in the parameterization, each with the
    library's key, selected as the case declares, in the order of the library keys."""
    found = _subject_sets(conn, group, parameterization, case.spec.parameterization.occurrence)
    keys = _library_keys(conn, case, found)
    spec = case.spec.subjects
    subjects: list[Subject] = []
    problems: list[str] = []
    for set_id, ids in found:
        names: list[str] = []
        for part in ids:
            options = keys.get(part, [])
            if len(options) != 1:
                where = f"{spec.library_key.carrier}/{spec.library_key.scope}"
                problems.append(
                    f"subject {part} has {len(options)} source entities in {where}, needs one"
                )
            else:
                names.append(options[0])
        if len(names) == len(ids):
            subjects.append(Subject(set_id, ids, tuple(names)))
    if problems:
        raise Blocked(
            "subject_unidentified",
            "; ".join(problems[:5])
            + (f" (and {len(problems) - 5} more)" if len(problems) > 5 else ""),
        )
    subjects.sort(key=lambda subject: subject.keys)
    if spec.select == "list":
        by_key = {subject.keys: subject for subject in subjects}
        unknown = [key for key in spec.keys if (key,) not in by_key]
        if unknown:
            raise Blocked(
                "subjects_not_found",
                f"no set in the parameterization for subject(s) {', '.join(unknown)}",
            )
        chosen = [by_key[(key,)] for key in spec.keys]
        return sorted(chosen, key=lambda subject: subject.keys)
    if spec.select == "sample":
        assert spec.size is not None and spec.seed is not None
        if spec.size > len(subjects):
            raise Blocked(
                "sample_too_large",
                f"a sample of {spec.size} subjects was asked for, {len(subjects)} exist",
            )
        return sorted(random.Random(spec.seed).sample(subjects, spec.size), key=lambda s: s.keys)
    return subjects


# -- grids -------------------------------------------------------------------------------------


@dataclass(frozen=True)
class _Clause:
    observable: uuid.UUID
    component: uuid.UUID | None
    aggregation: uuid.UUID | None
    lower: float | None
    upper: float | None


@dataclass(frozen=True)
class _Region:
    kind: str
    ordinal: int
    clauses: tuple[_Clause, ...]


def _regions(conn: psycopg.Connection, sets: Sequence[uuid.UUID]) -> dict[uuid.UUID, list[_Region]]:
    """The validity regions of each of `sets` (of its own record), with their clauses."""
    region, clause = pc.VALIDITY_REGION, pc.REGION_CLAUSE
    rows = conn.execute(
        sql.SQL(
            "SELECT r.{r_record}, r.id, r.{r_kind}::text, r.{r_ordinal}, "
            "c.{observable}, c.{component}, c.{aggregation}, c.{lower}, c.{upper} "
            "FROM {r_table} r JOIN {c_table} c ON c.{c_region} = r.id "
            "WHERE r.{r_record} = ANY(%s) "
            "ORDER BY r.{r_record}, r.{r_kind}::text, r.{r_ordinal}, c.{c_ordinal}"
        ).format(
            r_record=sql.Identifier(region.record),
            r_kind=sql.Identifier(region.kind),
            r_ordinal=sql.Identifier(region.ordinal),
            observable=sql.Identifier(clause.observable),
            component=sql.Identifier(clause.component),
            aggregation=sql.Identifier(clause.aggregation),
            lower=sql.Identifier(clause.lower),
            upper=sql.Identifier(clause.upper),
            r_table=table_identifier(region.table),
            c_table=table_identifier(clause.table),
            c_region=sql.Identifier(clause.region),
            c_ordinal=sql.Identifier(clause.ordinal),
        ),
        (list(sets),),
    ).fetchall()
    held: dict[uuid.UUID, dict[uuid.UUID, tuple[str, int, list[_Clause]]]] = {}
    for record, identifier, kind, ordinal, observable, component, aggregation, lower, upper in rows:
        entry = held.setdefault(record, {}).setdefault(identifier, (kind, int(ordinal), []))
        entry[2].append(
            _Clause(
                observable,
                component,
                aggregation,
                None if lower is None else float(lower),
                None if upper is None else float(upper),
            )
        )
    return {
        record: [
            _Region(kind, ordinal, tuple(clauses)) for kind, ordinal, clauses in by_id.values()
        ]
        for record, by_id in held.items()
    }


def _spread(grid: ArgumentSpec, low: float, high: float) -> np.ndarray:
    assert grid.points is not None
    low, high = low + grid.inset * (high - low), high - grid.inset * (high - low)
    if grid.points == 1:
        return np.array([(low + high) / 2])
    return np.linspace(low, high, grid.points)


def grids(
    conn: psycopg.Connection,
    case: Case,
    decl: m.Declaration,
    contract: m.Contract,
    subjects: Sequence[Subject],
) -> dict[uuid.UUID, dict[str, list[float]]]:
    """The argument columns of each subject's points (aligned, one entry per point; several
    arguments give their Cartesian product, the last varying fastest), keyed by set."""
    per_argument: dict[str, dict[uuid.UUID, np.ndarray]] = {}
    problems: list[str] = []
    regions: dict[uuid.UUID, list[_Region]] = {}
    if any(case.spec.arguments[a.name].points is not None for a in contract.arguments):
        regions = _regions(conn, [s.set_id for s in subjects])
    for argument in contract.arguments:
        grid = case.spec.arguments[argument.name]
        if grid.values is not None:
            assert grid.unit is not None
            target = storage_unit(decl, argument.type)
            values = [convert(v, grid.unit, target) if target else v for v in grid.values]
            per_argument[argument.name] = {s.set_id: np.array(values) for s in subjects}
            continue
        assert argument.observable is not None  # validated: a grid within a region needs it
        entity = decl.observable_entity(argument.observable)
        assert entity is not None
        table: dict[uuid.UUID, np.ndarray] = {}
        for subject in subjects:
            label = "/".join(subject.keys)
            kind = f" of kind `{grid.envelope_kind}`" if grid.envelope_kind else ""
            found = [
                region
                for region in regions.get(subject.set_id, [])
                if grid.envelope_kind is None or region.kind == grid.envelope_kind
            ]
            if len(found) != 1:
                problems.append(
                    f"{label}: {len(found)} validity regions{kind}, a grid within a region "
                    "needs exactly one (give explicit `values` for this argument)"
                )
                continue
            along = [
                c
                for c in found[0].clauses
                if c.observable == entity.id and c.component is None and c.aggregation is None
            ]
            if len(along) != 1:
                problems.append(
                    f"{label}: the validity region{kind} has {len(along)} clauses on "
                    f"`{argument.observable}` that are about the whole record, a grid follows "
                    "exactly one (give explicit `values` for this argument)"
                )
                continue
            if along[0].lower is None or along[0].upper is None:
                problems.append(
                    f"{label}: the clause of the validity region{kind} on "
                    f"`{argument.observable}` is open at one end"
                )
                continue
            table[subject.set_id] = _spread(grid, along[0].lower, along[0].upper)
        per_argument[argument.name] = table
    if problems:
        shown = "; ".join(problems[:5])
        raise Blocked("no_grid", f"no grid for {len(problems)} subject(s): {shown}")
    names = [a.name for a in contract.arguments]
    result: dict[uuid.UUID, dict[str, list[float]]] = {}
    for subject in subjects:
        product = itertools.product(
            *(per_argument[name][subject.set_id].tolist() for name in names)
        )
        rows = list(product)
        result[subject.set_id] = {name: [row[i] for row in rows] for i, name in enumerate(names)}
    return result


# -- keys and digests --------------------------------------------------------------------------


def _key(
    inputs: Mapping[str, object], *, files: Mapping[str, Sequence[Path]] | None = None
) -> reuse.Key:
    """The reuse key of a run: the shared builder over the run's inputs, the framework files,
    the installed libraries and, when the run got far enough to have them, the harness script and
    the lock of the environment it runs in."""
    try:
        return reuse.stage_key("qualify", inputs, files=files)
    except reuse.ReuseError as error:
        raise Blocked("harness_failed", f"the run cannot be keyed: {error}") from error


def _digest(value: object) -> str:
    text = json.dumps(value, sort_keys=True, separators=(",", ":"), default=str)
    return hashlib.sha256(text.encode()).hexdigest()


def records_read(
    conn: psycopg.Connection,
    decl: m.Declaration,
    sets: Sequence[uuid.UUID],
    parameterizations: Sequence[uuid.UUID] = (),
) -> ReadRecords:
    """The content hash of every record the database holds that a run evaluating `sets` reads
    (`build.currency` says which), and every set of the `parameterizations` (those a sub-form
    choice reads). `tk build` computes the same hashes from the canonical Parquet it loads, so a
    rebuilt database with other values under the same identifiers gives another result."""
    layout = currency.Layout(decl)
    found = currency.read_records(
        layout,
        currency.DatabaseRows(conn, layout),
        sets=sets,
        parameterizations=parameterizations,
    )
    return ReadRecords(
        sets=sorted(str(identifier) for identifier in sets),
        parameterizations=sorted(str(identifier) for identifier in parameterizations),
        records=found,
    )


# -- evaluation and comparison -----------------------------------------------------------------


def _roles(group: m.SlotGroup, subject: Subject) -> dict[str, str]:
    by_role = {
        subject_field.name: part
        for subject_field, part in zip(group.subjects, subject.ids, strict=True)
    }
    return {binding.target: by_role[binding.role] for binding in group.bindings if binding.target}


def evaluate(
    source: DatabaseSource,
    decl: m.Declaration,
    case: Case,
    group: m.SlotGroup,
    subjects: Sequence[Subject],
    points: Mapping[uuid.UUID, dict[str, list[float]]],
) -> Evaluation:
    """The form's output at each subject's points from the database, or the refusal that stopped
    it (a string). The subjects share one compile cache, so subjects whose expansion has the same
    structure (the same term count, say) compile the form once.

    A case that names a validity kind has every point classified against the regions of that kind
    of the records the evaluation read (`BoundForm.validity`), and leaves out the points outside
    them when its policy is to `exclude`: they are neither evaluated nor compared, and the
    classification still counts them."""
    source.prefetch(group.qualified, [subject.ids for subject in subjects])
    cache = CompileCache()
    validity = case.spec.validity
    answers: dict[uuid.UUID, np.ndarray | str] = {}
    kept: dict[uuid.UUID, dict[str, list[float]]] = {}
    membership: dict[uuid.UUID, np.ndarray | None] = {}
    for subject in subjects:
        columns = points[subject.set_id]
        arguments = {name: np.array(values) for name, values in columns.items()}
        kept[subject.set_id] = columns
        try:
            bound = bind(
                decl,
                case.spec.form,
                source=source,
                roles=_roles(group, subject),
                cache=cache,
                pieces=PiecePolicy(case.spec.pieces.boundary, case.spec.pieces.outside),
            )
            if validity is not None:
                codes = bound.validity(
                    case.spec.output, validity.kind, references=None, **arguments
                )
                membership[subject.set_id] = codes
                if validity.outside == "exclude":
                    keep = codes != Membership.OUTSIDE
                    arguments = {name: values[keep] for name, values in arguments.items()}
                    kept[subject.set_id] = {
                        name: values.tolist() for name, values in arguments.items()
                    }
            # A non-finite answer is compared (and fails the point); NumPy's warning adds nothing.
            with np.errstate(all="ignore"):
                answers[subject.set_id] = bound.evaluate(case.spec.output, **arguments)
        except EvaluationRefusal as refusal:
            answers[subject.set_id] = str(refusal)
            membership[subject.set_id] = None  # a subject the form is refused for is not classified
    return Evaluation(answers, kept, membership)


def _deviation(form_value: float, library_value: float) -> float:
    if not math.isfinite(form_value):
        return math.inf
    difference = abs(form_value - library_value)
    if library_value == 0:
        return 0.0 if difference == 0 else math.inf
    return difference / abs(library_value)


def compare(
    case: Case,
    decl: m.Declaration,
    contract: m.Contract,
    subjects: Sequence[Subject],
    points: Mapping[uuid.UUID, dict[str, list[float]]],
    answers: Mapping[uuid.UUID, np.ndarray | str],
    result: harness.Result,
) -> tuple[list[SubjectOutcome], list[dict[str, object]]]:
    """Each subject's outcome, and the invalid points (NaN or error answers) of the library."""
    comparison = case.spec.comparison
    absolute: float | None = None
    if comparison.absolute_tolerance is not None:
        output = next(o for o in contract.outputs if o.name == case.spec.output)
        unit = comparison.absolute_tolerance
        target = storage_unit(decl, output.type)
        absolute = convert(unit.value, unit.unit, target) if target else unit.value
    outcomes: list[SubjectOutcome] = []
    invalid: list[dict[str, object]] = []
    for subject, library_points in zip(subjects, result.subjects, strict=True):
        columns = points[subject.set_id]
        outcome = SubjectOutcome(subject, len(library_points))
        outcomes.append(outcome)
        answer = answers[subject.set_id]
        if isinstance(answer, str):
            outcome.refused = answer
            outcome.failed = sum(1 for p in library_points if p.status == "ok")
            outcome.invalid = len(library_points) - outcome.failed
        for position, point in enumerate(library_points):
            at = {name: values[position] for name, values in columns.items()}
            if point.status != "ok":
                if not isinstance(answer, str):
                    outcome.invalid += 1
                invalid.append(
                    {
                        "subject": list(subject.keys),
                        "at": at,
                        "status": point.status,
                        "message": point.message,
                    }
                )
                continue
            if isinstance(answer, str):
                continue
            assert point.value is not None
            value = float(answer[position])
            deviation = _deviation(value, point.value)
            agrees = deviation <= comparison.relative_tolerance or (
                absolute is not None and abs(value - point.value) <= absolute
            )
            if outcome.worst is None or deviation > outcome.worst:
                outcome.worst, outcome.worst_at = deviation, at
            if agrees:
                outcome.passed += 1
            else:
                outcome.failed += 1
    return outcomes, invalid


# -- the run -----------------------------------------------------------------------------------


def _number(value: float | None) -> float | str | None:
    """A float for JSON: infinity and NaN are written as text, which JSON has no number for."""
    if value is None or math.isfinite(value):
        return value
    return "inf" if value > 0 else "-inf" if value < 0 else "nan"


def _report(
    case: Case,
    key: str,
    revision: str,
    state: _State,
    outcome: str,
    blocked_reason: str | None,
    note: str | None,
    outcomes: Sequence[SubjectOutcome],
    invalid: Sequence[dict[str, object]],
    sets: Sequence[uuid.UUID],
    totals: Mapping[Membership, int] | None,
) -> dict[str, object]:
    comparison = case.spec.comparison
    exclude = comparison.invalid_points == "exclude"
    worst = max(
        (o for o in outcomes if o.worst is not None), key=lambda o: o.worst or 0.0, default=None
    )
    return {
        "schema": REPORT_SCHEMA,
        "case": case.name,
        "run_key": key,
        "form": case.spec.form,
        "output": case.spec.output,
        "basis": case.spec.basis,
        "outcome": outcome,
        "blocked_reason": blocked_reason,
        "note": note,
        "library": state.library,
        "library_version": state.version,
        "parameterization": {"key": case.spec.parameterization.key, "revision": revision},
        "relative_tolerance": comparison.relative_tolerance,
        "absolute_tolerance": None
        if comparison.absolute_tolerance is None
        else {
            "value": comparison.absolute_tolerance.value,
            "unit": comparison.absolute_tolerance.unit,
        },
        "invalid_point_policy": comparison.invalid_points,
        "invalid_point_reason": comparison.invalid_reason,
        "subjects": len(outcomes),
        "points": sum(o.passed + o.failed + (0 if exclude else o.invalid) for o in outcomes),
        "passed": sum(o.passed for o in outcomes),
        "failed": sum(o.failed for o in outcomes),
        "invalid": len(invalid),
        "worst": None
        if worst is None
        else {
            "subject": list(worst.subject.keys),
            "relative_deviation": _number(worst.worst),
            "at": worst.worst_at,
        },
        "per_subject": [
            {
                "key": list(o.subject.keys),
                "set": str(o.subject.set_id),
                "points": o.points,
                "passed": o.passed,
                "failed": o.failed,
                "invalid": o.invalid,
                "worst_relative_deviation": _number(o.worst),
                "worst_at": o.worst_at,
                "refused": o.refused,
            }
            for o in outcomes
        ],
        "invalid_points": list(invalid),
        "sets": [str(s) for s in sets],
        "validity": _validity_report(case, totals),
    }


def _validity_report(
    case: Case, totals: Mapping[Membership, int] | None
) -> dict[str, object] | None:
    """Where the grid lies with respect to the validity regions the case names: how many points
    are inside, outside, undetermined, and in none of these because no region of the kind is
    stated; `excluded` is the outside points the case left out of the comparison. `None` for a
    case that names no validity kind."""
    validity = case.spec.validity
    if validity is None:
        return None
    return {
        "kind": validity.kind,
        "outside_policy": validity.outside,
        "inside": None if totals is None else totals[Membership.INSIDE],
        "outside": None if totals is None else totals[Membership.OUTSIDE],
        "undetermined": None if totals is None else totals[Membership.UNDETERMINED],
        "not_stated": None if totals is None else totals[Membership.NOT_STATED],
        "excluded": (
            None
            if totals is None
            else totals[Membership.OUTSIDE]
            if validity.outside == "exclude"
            else 0
        ),
    }


def table_lines(report: Mapping[str, object]) -> list[str]:
    """The table `tk qualify` prints for one case."""
    worst = report["worst"]
    where = "-"
    deviation = "-"
    if isinstance(worst, dict):
        at = ", ".join(f"{name}={value:.6g}" for name, value in (worst["at"] or {}).items())
        deviation = (
            f"{worst['relative_deviation']:.3e}"
            if isinstance(worst["relative_deviation"], float)
            else str(worst["relative_deviation"])
        )
        where = f"{'/'.join(worst['subject'])} at {at}"
    release = " ".join(
        str(part) for part in (report["library"], report["library_version"]) if part is not None
    )
    lines = [
        f"{report['case']}: {report['outcome']} ({release}, "
        f"basis {report['basis']}, relative tolerance {report['relative_tolerance']:g})",
        f"  {'subjects':>8} {'points':>8} {'passed':>8} {'failed':>8} {'invalid':>8}  worst relative deviation",
        f"  {report['subjects']:>8} {report['points']:>8} {report['passed']:>8} {report['failed']:>8} "
        f"{report['invalid']:>8}  {deviation}  ({where})",
    ]
    region = report.get("validity")
    if isinstance(region, dict) and region["inside"] is not None:
        lines.append(
            f"  against the `{region['kind']}` regions: {region['inside']} inside, "
            f"{region['outside']} outside ({region['outside_policy']}"
            + (f", {region['excluded']} left out" if region["excluded"] else "")
            + f"), {region['undetermined']} undetermined, {region['not_stated']} with no region stated"
        )
    if report["blocked_reason"]:
        lines.append(f"  blocked ({report['blocked_reason']}): {report['note']}")
    elif report["note"]:
        lines.append(f"  note: {report['note']}")
    refused = [o for o in report["per_subject"] if o["refused"]]  # type: ignore[union-attr]
    for entry in refused[:_SHOWN_INVALID]:
        lines.append(f"  refused: {'/'.join(entry['key'])}: {entry['refused']}")
    listed: list[dict[str, object]] = report["invalid_points"]  # type: ignore[assignment]
    for point in listed[:_SHOWN_INVALID]:
        at = ", ".join(f"{name}={value:.6g}" for name, value in point["at"].items())  # type: ignore[union-attr]
        lines.append(
            f"  invalid ({point['status']}): {'/'.join(point['subject'])} at {at}: {point['message']}"  # type: ignore[arg-type]
        )
    if len(listed) > _SHOWN_INVALID:
        lines.append(
            f"  ... and {len(listed) - _SHOWN_INVALID} more invalid points (see the report)"
        )
    failing = [o for o in report["per_subject"] if o["failed"]]  # type: ignore[union-attr]
    for entry in failing[:_SHOWN_INVALID]:
        lines.append(
            f"  failed: {'/'.join(entry['key'])}: {entry['failed']} point(s), worst "
            f"{entry['worst_relative_deviation']} at {entry['worst_at']}"
        )
    return lines


def run_case(ctx: Context, conn: psycopg.Connection, case: Case) -> CaseOutcome:
    """Carry out `case` against the database, record the output and load it into the database.

    Raises `CaseError` when the declaration refuses the case (nothing is recorded), and
    `SourceError` when the database was built from another declaration."""
    decl = ctx.decl
    problems = validate(case, decl)
    if problems:
        raise CaseError(case.path, problems)
    check_database(conn, decl, ctx.tree)
    spec = case.spec
    form = decl.forms[spec.form]
    contract = decl.contracts[form.implements]
    group = next(g for g in form.slot_groups if g.qualified == spec.subjects.group)
    argument_units = {
        a.name: storage_unit(decl, a.type) or "dimensionless" for a in contract.arguments
    }
    output = next(o for o in contract.outputs if o.name == spec.output)
    output_unit = storage_unit(decl, output.type) or "dimensionless"
    fingerprint = declaration_fingerprint(decl, read_physical(ctx.tree))
    expression = evaluation_hash(form, spec.output)
    state = _State(spec.harness.library)
    revision = spec.parameterization.revision
    outcomes: list[SubjectOutcome] = []
    invalid: list[dict[str, object]] = []
    outcome, blocked_reason, note = BLOCKED, None, None
    key: reuse.Key | None = None
    read: ReadRecords | None = None
    evaluation: Evaluation | None = None

    def run_key_of() -> str:
        return f"{case.name}/{spec.form}/{spec.parameterization.key}@{revision}/{fingerprint[:16]}"

    try:
        parameterization, revision = _parameterization(conn, spec.parameterization)
        stated = spec.parameterization.occurrence
        source = DatabaseSource(
            conn,
            decl,
            [parameterization],
            occurrences=None if stated is None else {parameterization: stated},
            subforms=_subforms(conn, case),
            tree=ctx.tree,
        )
        subjects = select_subjects(conn, case, group, parameterization)
        if not subjects:
            raise Blocked(
                "no_subjects", "the parameterization holds no set of the slot group for any subject"
            )
        state.sets = tuple(s.set_id for s in subjects)
        points = grids(conn, case, decl, contract, subjects)
        probed = _harness(
            lambda: harness.probe(
                spec.harness.library,
                spec.harness.call,
                spec.harness.environment,
                argument_units,
                output_unit,
                oracles=ctx.oracles_dir,
                tree=ctx.tree,
            )
        )
        state.library, state.version = probed.library, probed.version
        extra = [choice for choices in _subform_ids(conn, case).values() for choice in choices]
        read = records_read(conn, decl, state.sets, extra)
        script = harness.harness_script(spec.harness.library, ctx.oracles_dir)
        key = _key(
            {
                "declaration": fingerprint,
                "form": expression,
                "case": case.content_hash,
                "library": f"{probed.library} {probed.version}",
                "subjects": _digest([[list(s.keys), str(s.set_id)] for s in subjects]),
                "read": currency.read_digest(read.records),
                "format": persist.FORMAT,
            },
            files={
                "harness": [script],
                f"environment:{spec.harness.environment}": list(
                    reuse.environment_locks(spec.harness.environment, ctx.tree)
                ),
            },
        )
        stored = persist.read_output(ctx.canonical, case.name)
        if (
            not ctx.force
            and stored is not None
            and stored.manifest.reuse_key == key.digest
            and stored.report.get("outcome") != BLOCKED
        ):
            persist.load_live(ctx.url, stored.directory, stored.manifest, f"{case.name}/")
            return CaseOutcome(case.name, "current", stored.report, stored.directory)
        evaluation = evaluate(source, decl, case, group, subjects, points)
        request = harness.Request(
            spec.harness.library,
            spec.harness.call,
            argument_units,
            output_unit,
            tuple(_subject_request(s, evaluation.points[s.set_id]) for s in subjects),
        )
        result = _harness(
            lambda: harness.run(
                request, spec.harness.environment, oracles=ctx.oracles_dir, tree=ctx.tree
            )
        )
        state.library, state.version = result.library, result.version
        outcomes, invalid = compare(
            case, decl, contract, subjects, evaluation.points, evaluation.answers, result
        )
        outcome, blocked_reason, note = _verdict(case, outcomes, invalid, evaluation.counts())
    except Blocked as reason:
        blocked_reason, note = reason.reason, str(reason)
    except AmbiguousOccurrence as reason:
        blocked_reason, note = pc.BLOCKED_REASON.member("ambiguous_occurrence"), str(reason)
    run_key = run_key_of()
    worst = max((o.worst for o in outcomes if o.worst is not None), default=None)
    totals = None if evaluation is None else evaluation.counts()
    record = persist.RunRecord(
        key=run_key,
        form=spec.form,
        expression_hash=expression,
        basis=spec.basis,
        library=state.library,
        version=state.version,
        outcome=outcome,
        points=sum(o.passed + o.failed for o in outcomes)
        + (0 if spec.comparison.invalid_points == "exclude" else len(invalid)),
        relative_tolerance=spec.comparison.relative_tolerance,
        absolute_tolerance=None
        if spec.comparison.absolute_tolerance is None
        else (spec.comparison.absolute_tolerance.value, spec.comparison.absolute_tolerance.unit),
        observable=output.observable,
        worst_relative_deviation=worst if worst is not None and math.isfinite(worst) else None,
        blocked_reason=blocked_reason,
        note=note,
        sets=state.sets if outcomes else (),
        validity_kind=None if spec.validity is None else spec.validity.kind,
        inside_points=None
        if totals is None or spec.validity is None
        else totals[Membership.INSIDE],
        outside_points=None
        if totals is None or spec.validity is None
        else totals[Membership.OUTSIDE],
        undetermined_points=None
        if totals is None or spec.validity is None
        else totals[Membership.UNDETERMINED],
    )
    report = _report(
        case,
        run_key,
        revision,
        state,
        outcome,
        blocked_reason,
        note,
        outcomes,
        invalid,
        record.sets,
        totals if spec.validity is not None else None,
    )
    if key is None:
        key = _key(
            {
                "declaration": fingerprint,
                "case": case.content_hash,
                "blocked": f"{blocked_reason}: {note}",
                "format": persist.FORMAT,
            }
        )
    directory = persist.write_output(ctx.canonical, case.name, decl, record, key, report, read)
    persist.load_live(ctx.url, directory, store.read_manifest(directory), f"{case.name}/")
    return CaseOutcome(case.name, "ran", report, directory)


def _verdict(
    case: Case,
    outcomes: Sequence[SubjectOutcome],
    invalid: Sequence[dict[str, object]],
    totals: Mapping[Membership, int],
) -> tuple[str, str | None, str | None]:
    """The outcome, the blocked reason (only for a blocked outcome) and the note."""
    comparison = case.spec.comparison
    compared = sum(o.passed + o.failed for o in outcomes)
    excluded = comparison.invalid_points == "exclude"
    notes: list[str] = []
    validity = case.spec.validity
    if validity is not None and validity.outside == "exclude" and totals[Membership.OUTSIDE]:
        notes.append(
            f"{totals[Membership.OUTSIDE]} point(s) outside the `{validity.kind}` regions excluded "
            "from the comparison"
        )
    if invalid:
        if excluded:
            notes.append(
                f"{len(invalid)} point(s) the library answered NaN or error excluded: {comparison.invalid_reason}"
            )
        else:
            notes.append(
                f"{len(invalid)} point(s) the library answered NaN or error, which fail the run"
            )
    refused = [o for o in outcomes if o.refused]
    if refused:
        notes.append(f"the form was refused for {len(refused)} subject(s)")
    failed = sum(o.failed for o in outcomes)
    if failed:
        bad = sum(1 for o in outcomes if o.failed)
        notes.append(f"{failed} point(s) of {bad} subject(s) outside the tolerance")
    if compared == 0 and not refused:
        return (
            BLOCKED,
            pc.BLOCKED_REASON.member("nothing_compared"),
            "no point could be compared: " + (notes[0] if notes else "the grid is empty"),
        )
    fails = bool(failed or refused or (invalid and not excluded))
    return (FAILED if fails else PASSED), None, ("; ".join(notes) or None)


def _harness(call: Callable[[], harness.Result]) -> harness.Result:
    try:
        return call()
    except harness.HarnessUnavailable as error:
        raise Blocked(error.reason, f"harness unavailable: {error}") from error


def _subject_request(subject: Subject, columns: dict[str, list[float]]) -> harness.SubjectRequest:
    count = len(next(iter(columns.values()), []))
    return harness.SubjectRequest(subject.keys, columns, count)


def _subform_ids(conn: psycopg.Connection, case: Case) -> dict[str, list[uuid.UUID]]:
    return {
        slot: [_parameterization(conn, choice.parameterization)[0] for choice in choices]
        for slot, choices in case.spec.subforms.items()
    }


def _subforms(conn: psycopg.Connection, case: Case) -> dict[str, list[SubformBinding]]:
    bindings: dict[str, list[SubformBinding]] = {}
    for slot, choices in case.spec.subforms.items():
        bindings[slot] = []
        for choice in choices:
            identifier, _ = _parameterization(conn, choice.parameterization)
            stated = choice.parameterization.occurrence
            bindings[slot].append(
                SubformBinding(
                    choice.form, (identifier,), {} if stated is None else {identifier: stated}
                )
            )
    return bindings


__all__ = [
    "Blocked",
    "CaseOutcome",
    "Context",
    "Subject",
    "Evaluation",
    "SubjectOutcome",
    "compare",
    "evaluate",
    "grids",
    "run_case",
    "select_subjects",
    "records_read",
    "table_lines",
]
