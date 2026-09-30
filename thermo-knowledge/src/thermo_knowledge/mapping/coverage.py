# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Row states, the coverage report and the rows that record a mapping's rules.

Every source-faithful row of every staged table ends in exactly one state (pipeline section 2):

| State | The row |
|---|---|
| `mapped` | produced records under rules that declare no loss |
| `mapped_with_loss` | produced records under a rule, partition or table that declares a loss or an assumption |
| `out_of_scope`, `deferred` | belongs to a table or partition with that disposition |
| `held` | was refused: its subject is ambiguous or unknown, or it failed validation |
| `unmapped` | a row of a mapped table or partition that produced nothing and was not held |

`compute` derives the states from the dispositions `mapping.toml` declares and the outcomes the
run recorded; it never reads a mapping's intent. The counts per table and state are written to
`qual.mapping_coverage`, each held or unmapped row to `qual.held_row` with its typed reason
(a `held_reason` member) and the detail in words, and each rule to `qual.mapping_rule`, a value
rule with the number of mapped rows it was applied to. `check_rule_use` states which value rules
were applied to no row of a table that has mapped rows.
"""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass, field

import pyarrow as pa

from thermo_knowledge import pipeline_contract as pc
from thermo_knowledge.canonical.values import conversion_factor
from thermo_knowledge.canonical.writer import CanonicalWriter
from thermo_knowledge.declaration import model as m
from thermo_knowledge.mapping import claims
from thermo_knowledge.mapping.context import Outcome
from thermo_knowledge.mapping.spec import MappingSpec, TableRule, maps_rows, resolve_target
from thermo_knowledge.mapping.staged import Classifier, StagedTables

RULE = pc.MAPPING_RULE
COVERAGE = pc.MAPPING_COVERAGE
HELD_ROW = pc.HELD_ROW
MAPPED = claims.MAPPED
FIT_KIND = pc.DERIVATION_KIND.member("fit")
STATES = pc.ROW_STATE.members
UNMAPPED = pc.ROW_STATE.member("unmapped")
MAPPED_WITH_LOSS = pc.ROW_STATE.member("mapped_with_loss")
ROW_MAPPED = pc.ROW_STATE.member("mapped")
UNMAPPED_BY_MAPPING = pc.HELD_REASON.member("unmapped_by_mapping")
UNMAPPED_DETAIL = "read by the mapping, which emitted nothing for it"


@dataclass(frozen=True)
class HeldRow:
    """A row that was not loaded: where it is, its state (`held` or `unmapped`), why as a
    `held_reason` member and the detail in words."""

    table: str
    locator: str
    state: str
    reason: str
    detail: str


@dataclass
class Coverage:
    """Rows per staged table and state, and the rows that were not loaded."""

    counts: dict[str, dict[str, int]] = field(default_factory=dict)
    held: list[HeldRow] = field(default_factory=list)
    """Every held and unmapped row."""

    def total(self, state: str) -> int:
        return sum(row.get(state, 0) for row in self.counts.values())

    def by_reason(self) -> dict[str, int]:
        """The held and unmapped rows grouped by their typed reason, in reason-name order."""
        counts: dict[str, int] = {}
        for row in self.held:
            counts[row.reason] = counts.get(row.reason, 0) + 1
        return dict(sorted(counts.items()))


def merge_outcomes(
    ledger: list[dict[str, object]], outcomes: Mapping[tuple[str, str], Outcome]
) -> dict[tuple[str, str], Outcome]:
    """The phase-1 ledger and the phase-2 outcomes as one outcome per row: a row is held when
    either phase held it, lossy when either phase was."""
    merged: dict[tuple[str, str], Outcome] = {}
    for row in ledger:
        merged[(str(row["table"]), str(row["locator"]))] = Outcome(
            str(row["state"]),
            bool(row["lossy"]),
            row["reason"] if isinstance(row["reason"], str) else None,
            row["detail"] if isinstance(row["detail"], str) else None,
        )
    for key, outcome in outcomes.items():
        existing = merged.get(key)
        if existing is None:
            merged[key] = Outcome(outcome.state, outcome.lossy, outcome.reason, outcome.detail)
        elif outcome.state == claims.HELD:
            kept = existing if existing.state == claims.HELD else outcome
            merged[key] = Outcome(claims.HELD, False, kept.reason, kept.detail)
        elif existing.state == claims.EMITTED:
            existing.lossy = existing.lossy or outcome.lossy
    return merged


def ledger_rows(outcomes: Mapping[tuple[str, str], Outcome]) -> list[dict[str, object]]:
    """The ledger of the outcomes: one row per source row."""
    return [
        {
            "table": table,
            "locator": locator,
            "state": o.state,
            "lossy": o.lossy,
            "reason": o.reason,
            "detail": o.detail,
        }
        for (table, locator), o in outcomes.items()
    ]


def compute(
    tables: StagedTables,
    classifier: Classifier,
    rules: Mapping[str, TableRule],
    outcomes: Mapping[tuple[str, str], Outcome],
) -> Coverage:
    """The state of every row of every staged table."""
    result = Coverage()
    for table in tables.names:
        rule = rules[table]
        counts = result.counts.setdefault(table, {})
        if not rule.partitions and rule.disposition != MAPPED:
            counts[rule.disposition] = tables.row_count(table)
            continue
        for locator, disposition in classifier.classify(table).items():
            if disposition.disposition != MAPPED:
                state = disposition.disposition
            else:
                outcome = outcomes.get((table, locator))
                if outcome is None:
                    state = UNMAPPED
                    result.held.append(
                        HeldRow(table, locator, state, UNMAPPED_BY_MAPPING, UNMAPPED_DETAIL)
                    )
                elif outcome.state == claims.HELD:
                    assert outcome.reason is not None  # a held row always states its reason
                    state = claims.HELD
                    result.held.append(
                        HeldRow(
                            table, locator, state, outcome.reason, outcome.detail or outcome.reason
                        )
                    )
                else:
                    state = MAPPED_WITH_LOSS if outcome.lossy else ROW_MAPPED
            counts[state] = counts.get(state, 0) + 1
        if sum(counts.values()) != tables.row_count(table):
            raise AssertionError(
                f"{table}: {sum(counts.values())} row states for {tables.row_count(table)} rows"
            )
    return result


def _reason(disposition: str, reason: str | None, wave: int | None, loss: str | None) -> str | None:
    if disposition == claims.DEFERRED:
        return f"deferred to wave {wave}: {reason}"
    if disposition == claims.OUT_OF_SCOPE:
        return reason
    return loss


def check_rule_use(
    spec: MappingSpec, result: Coverage, applied: Mapping[tuple[str, str], int]
) -> tuple[list[str], list[str]]:
    """The value rules that were applied to no row although their table has mapped rows:
    `(refused, reported)`, each naming the rule. A rule the mapping declares `optional` (a field
    the source fills only sometimes) is reported; any other is refused, since a declared rule the
    mapping never consumed is a mapping that stopped emitting what it declares."""
    refused: list[str] = []
    reported: list[str] = []
    for table, rule in spec.tables.items():
        counts = result.counts.get(table, {})
        mapped = sum(
            rows
            for state, rows in counts.items()
            if state not in (claims.OUT_OF_SCOPE, claims.DEFERRED)
        )
        if not maps_rows(rule) or mapped == 0:
            continue
        for column, field_rule in rule.fields.items():
            if field_rule.target is None or applied.get((table, column), 0) > 0:
                continue
            text = (
                f"table {table} column {column}: the value rule for `{field_rule.target}` was "
                f"applied to none of the {mapped} mapped row(s) of the table"
            )
            if field_rule.optional:
                reported.append(text)
            else:
                refused.append(
                    f"{text}; the mapping no longer emits what the rule declares (declare "
                    "`optional = true` for a field the source fills only sometimes)"
                )
    return refused, reported


def rule_rows(
    spec: MappingSpec,
    decl: m.Declaration,
    schemas: Mapping[str, pa.Schema],
    applied: Mapping[tuple[str, str], int],
) -> list[dict[str, object]]:
    """The `qual.mapping_rule` rows of a mapping: one per table, partition and column; a value
    rule states the number of mapped rows of its table it was applied to (`applied`)."""
    rows: list[dict[str, object]] = []
    manifest_id = spec.source
    for table in schemas:
        rule = spec.tables[table]
        rows.append(
            {
                RULE.manifest_id: manifest_id,
                RULE.source_table: table,
                RULE.source_field: "",
                RULE.disposition: rule.disposition,
                RULE.loss: _reason(rule.disposition, rule.reason, rule.wave, rule.loss),
            }
        )
        for partition in rule.partitions:
            rows.append(
                {
                    RULE.manifest_id: manifest_id,
                    RULE.source_table: table,
                    RULE.source_field: f"partition:{partition.name}",
                    RULE.disposition: partition.disposition,
                    RULE.loss: _reason(
                        partition.disposition, partition.reason, partition.wave, partition.loss
                    ),
                }
            )
        for owner, derivation in [("", rule.derivation)] + [
            (p.name, p.derivation) for p in rule.partitions
        ]:
            if derivation is not None:
                rows.append(
                    {
                        RULE.manifest_id: manifest_id,
                        RULE.source_table: table,
                        RULE.source_field: f"derivation:{owner}" if owner else "derivation",
                        RULE.disposition: MAPPED,
                        RULE.target: pc.FIT.declared
                        if derivation.kind == FIT_KIND
                        else pc.DERIVATION.declared,
                        RULE.loss: derivation.method,
                    }
                )
        constants = dict(rule.constants)
        for partition in rule.partitions:
            constants.update({f"{k}": v for k, v in partition.constants.items()})
        for target, value in sorted(constants.items()):
            rows.append(
                {
                    RULE.manifest_id: manifest_id,
                    RULE.source_table: table,
                    RULE.source_field: f"constant:{target}",
                    RULE.disposition: MAPPED,
                    RULE.target: target,
                    RULE.loss: f"a constant of the mapping: {value!r}",
                }
            )
        for column, field_rule in rule.fields.items():
            row: dict[str, object] = {
                RULE.manifest_id: manifest_id,
                RULE.source_table: table,
                RULE.source_field: column,
                RULE.disposition: field_rule.disposition or MAPPED,
            }
            if field_rule.disposition is not None:
                row[RULE.loss] = _reason(
                    field_rule.disposition, field_rule.reason, field_rule.wave, None
                )
            elif field_rule.role is not None:
                row[RULE.loss] = f"structure: {field_rule.reason}"
            else:
                assert field_rule.target is not None
                target = resolve_target(decl, field_rule.target)
                assert not isinstance(target, str)
                row[RULE.target] = (
                    f"{field_rule.target} ({field_rule.scheme})"
                    if field_rule.scheme
                    else field_rule.target
                )
                row[RULE.precision] = field_rule.precision
                row[RULE.applied_rows] = applied.get((table, column), 0)
                if field_rule.unit is not None:
                    row[RULE.source_unit] = field_rule.unit
                    if target.unit is not None:
                        row[RULE.factor] = conversion_factor(field_rule.unit, target.unit)
                notes = [field_rule.loss] if field_rule.loss else []
                if field_rule.absent:
                    notes.append(
                        "declared absent: " + ", ".join(repr(a) for a in field_rule.absent)
                    )
                if field_rule.otherwise_scheme:
                    notes.append(
                        f"a value not matching `{field_rule.pattern}` is asserted under "
                        f"`{field_rule.otherwise_scheme}`"
                    )
                row[RULE.loss] = "; ".join(notes) or None
            rows.append(row)
    return rows


def write_rows(
    writer: CanonicalWriter,
    manifest_id: str,
    coverage: Coverage,
    rules: list[dict[str, object]],
) -> None:
    """Write the rule, coverage and held-row rows."""
    for row in rules:
        writer.kind(
            RULE.declared,
            row,
            at=f"mapping rule {row[RULE.source_table]}.{row[RULE.source_field]}",
        )
    for table, counts in coverage.counts.items():
        for state in STATES:
            if counts.get(state):
                writer.kind(
                    COVERAGE.declared,
                    {
                        COVERAGE.manifest_id: manifest_id,
                        COVERAGE.source_table: table,
                        COVERAGE.state: state,
                        COVERAGE.rows: counts[state],
                    },
                    at=f"coverage {table}",
                )
    for held in coverage.held:
        writer.kind(
            HELD_ROW.declared,
            {
                HELD_ROW.manifest_id: manifest_id,
                HELD_ROW.source_table: held.table,
                HELD_ROW.locator: held.locator,
                HELD_ROW.state: held.state,
                HELD_ROW.reason: held.reason,
                HELD_ROW.detail: held.detail,
            },
            at=held.locator,
        )
