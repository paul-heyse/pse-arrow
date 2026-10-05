---
title: "27c: Selective refinement and completion"
status: in-progress
date: 2026-10-05
adrs: [ADR-0163]
review_sources: ["../design_review/reviews/design_review_contextual-accuracy-policy_2026-10-05.md"]
scenario_sources: ["../design_review/reviews/design_review_contextual-accuracy-policy_2026-10-05.md#scenarios-that-distinguish-the-alternatives"]
---

# 27c: Selective refinement and completion

## Purpose and owners

Make execution respond to actual engineering evidence, then retain one immutable result
assessment. This companion owns C0–C5 and CA-F04 execution work. Consume
[27a's admitted context/goals](27a-accuracy-intent-and-contextual-policy.md) and
[27b's evidence/allocation](27b-error-allocation-and-numerical-evidence.md).
[25k](25k-integrated-qualification-and-closure.md#contextual-accuracy-review-dispositions)
owns finding disposition, full qualification and measurements.

Current strategy declarations already grant finite mechanisms and transitions with one
task scope and actual charge ownership. `workflow/numerics.rs` composes native qualification,
model checks, closure, applicability and coverage into candidate use. Reuse these owners;
do not add goal loops in workflows or make inspection/publication effectful. Generic Boolean
checks remain valid under their existing contract, without automatic threshold extraction.

## Pure goal assessment and candidate use

For a physical value `q` with an admitted error estimate/bound `e`, form its envelope using
the evidence's actual meaning. Certified uses conservative endpoints; Estimated uses an
explicitly estimated envelope. Actual objective intervals may be asymmetric. Assessment
requires matching target, location, point, context, normalization/units, branch/validity and
evidence at least as strong as requested. Missing or invalid evidence is Unresolved.

Resolution and criterion outcomes remain independent:

| Fact | Assessment |
|---|---|
| Value resolution requested | Met when admissible output error is at most the explicit physical resolution; otherwise Unmet or Unavailable |
| No value resolution | NotRequested; a criterion-only goal does not add an output precision demand |
| Criterion requested | Satisfied when the whole admitted envelope is inside its allowed set; Violated when disjoint; otherwise Unresolved |
| No criterion | NotRequested; a value-only goal makes no specification assertion |

Upper/lower limits and bands are inclusive. For `q<=U`, `[U-e,U]` satisfies while
`[U,U+e]` is unresolved for positive `e`. A singleton exactly at the boundary satisfies
only when actual admissible evidence establishes it. Numerical resolution never enlarges
the allowed set. Requesting Estimated evidence does not weaken a separate branch certificate.

A value-only goal resolves when its resolution is Met. A criterion-only goal resolves once
its decision is Satisfied or Violated at the requested evidence strength. A combined goal
needs both resolution Met and a resolved decision. Retain the two facts even when only one
blocks completion; do not collapse them into a generic failed check.

| Use policy | Resolved Satisfied / value-only Met | Resolved Violated | Unresolved required goal |
|---|---|---|---|
| Assess (default) | May permit result | May permit a result reporting violation | Refuse requested goal-qualified result; retain diagnostic point/evidence |
| RequireSatisfied | May permit result | Refuse result | Refuse result |

Every permission remains conditional on all existing independent candidate obligations. No
goal upgrades native failure, cancellation/panic, missing original validation, relaxed-only
incumbents, absent coverage or forbidden domain/selection evidence. Seed eligibility remains
the existing separate composed permission; a refinement proposal may consume a fresh point
only under the granted start/branch rules. No goals preserves ordinary candidate use and
retains NotRequested, never an implied output guarantee.

## Bounded execution operation

1. Prepare admitted goals/context and execute the existing automatic or explicit strategy.
2. Apply independent original assessment. A forbidden candidate does not launch extra goal
   work; retain missing assessment reasons with its actual failure. For a permissible candidate,
   obtain only declared goal-relevant evidence under the remaining task grant.
3. Classify goals with the pure assessor. Stop on resolved goals; a resolved specification
   violation is not an excuse to search for a passing point.
4. For a refinement-permitted unresolved goal, derive the required output allowance, ask its
   mathematical owner for contributing production demands, and admit that operation through
   the existing strategy. Rebuild incompatible native layouts using the tighter work identity;
   leave original goals, acceptance and scientific problem unchanged.
5. Reassess the new actual candidate and evidence, including original obligations and branch
   validity. Finish with a resolved assessment or a typed unresolved reason.

For a criterion with point value inside an allowed band, use distance to the nearest boundary;
outside, use distance to the nearest allowed boundary. For a one-sided limit, use absolute
distance to that boundary. Where current evidence overlaps it, target the largest representable
positive output allowance strictly smaller than that distance. Combine with an explicit value
resolution by taking the tighter comparable allowance. Zero distance does not create a blind
zero-error retry: only an already supported exact/certified operation can settle that boundary;
otherwise return BoundaryUnresolved. Recompute distances at the new point; never move the
specification to track the candidate.

Producer allocation comes from B2; local controls and any comparator method belong to their
native/dynamic owners. The strategy owner does not implement Newton, an integrator controller
or repeated multiplication of all tolerances by an arbitrary factor. A zero remaining refinement
grant permits final assessment only. Explicit strategies and backend choices retain their
constraints; Auto uses only admitted capabilities and never performs a solver tournament.

### Progress, identity and resource lifetime

Compare successive valid observations in the same physical goal meaning. Progress means a
strictly smaller finite output error estimate/bound, a stronger admissible evidence class, or
a newly resolved required outcome. Changed branches/contexts are not comparable error progress.
After one completed requested refinement produces no progress, stop with Nonprogress;
remaining budget is permission, not a demand to spend it. Overlapping-but-improving evidence
can continue until the existing finite grant ends.

Typed unresolved reasons include EvidenceUnavailable, InvalidValidity, BoundaryUnresolved,
Nonprogress, PrecisionLimit, RefinementDisabled and BudgetExhausted, with the actual producer
or grant cause retained. Cancellation, resource exhaustion, panic and mandatory mathematical
domain/branch failures keep their existing terminal precedence. Do not wrap them as a successful
unresolved goal result. A locally unavailable optional action remains distinct from a failed
required execution effect.

Use the existing task deadline, memory scopes, attempts and observed evaluation/iteration/factor/
proof counters. Each evidence operation and refinement has a distinct occurrence identity and
one actual charging owner. Shared producer work for several goals is charged once; unavailable
counters remain unknown. Additional work is admitted before allocating native state. Release
superseded scratch/workers before replacement; preserve the complete admitted owner through the
operation, then partition completed retained reports through existing completion ownership.

Reuse is producer-owned. Structural preparation can survive unchanged science; point factors,
error evidence and candidate assessments require their actual consumed point, parameters,
normalization, branch, demand and profile identities. A changed frozen goal/context changes
request/preparation identity. Changed work precision changes attempt/product identity without
mutating the acceptance key. Do not accept a stale certificate solely because its allowance
looks smaller, or couple unrelated goals to a changed policy when they consume none of it.

## Interfaces, migration and decision route

C0 inventories actual Rust request owners, authored documents, registry versions, Python
generated contracts, job/study payloads, operational storage and result transports before
changing them. The inventory settles concrete codec/version changes from consumed meaning,
not from a blanket bump. Use existing version-first readmission and explicit migration
capabilities. Preserve old stored bytes/results; unsupported historic readmission refuses
truthfully. A supported old no-goal request maps to NotRequested, never inferred goals.

Add registry-owned frozen engineering interpretation and goal assessment records. Retain
goal identity, observation location, value or typed absence, resolution outcome, decision
outcome, error or interval, method/strength/validity, source/branch, contributing products,
refinement history summary and unresolved cause. Extend composed candidate reasons/qualifiers
to include required goal outcomes and Estimated/empirical or canonical-fallback limitations.
Use the existing result/export route; workflows and Python project the retained assessment.
No separately computed usability flag, duplicate relation schema or sidecar report engine.

Derived/refinement work records link actual attempts and demand identities to final assessment.
Keep bounded evidence required to explain permission, not every callback/iterate or an unbounded
copy of each trajectory. Restart and publication consume completed evidence; neither retries
the solve. Durable jobs resume numerical work only through their existing granted execution
transition, accounting for prior completed work.

Authored/request goals and observation binding migrate steady simulation, optimization,
initialization/recycles where those public analyses consume goals, nested/reduced execution,
integrated/discretized dynamics, fitting response and study member dispatch. Each consumes the
shared policy/assessment appropriate to its actual numerical scope. Statistical study stopping,
controller policy, fitting covariance and physical events retain their own meanings; do not
turn them into engineering tolerances. A block receives only projected goals it can evaluate;
full-output goals remain at their full-analysis consumer.

New relation meaning needs an ADR; changed hashing/Python boundary contracts require ADR and
design review before production changes. C0 records those actual changes and required amended
architecture owners from the coordinator. This document supplies the target, not accepted ADR
status or implicit authority to edit protected architecture. Follow the repository's existing
decision/design route without introducing another approval system.

### C0 consumed-version inventory

ADR-0163 and the current-standard contracts review govern this migration.

| Owner | Selected change and preservation |
|---|---|
| NumericalPolicy | Required standalone Version1; omitted numerical policy at a current enclosing leaf still uses its typed defaults |
| SolveSettings | Version3 to4, version-first Python/native read; unsupported historical execution requests require explicit readmission |
| SimulationProfile | Required Version1 for previously unversioned execution profile; version-first Python JSON read |
| StudyOperation | Version4 to5 because omitted simulation profiles select revised authored defaults |
| numerical_requirements / resolved_numerics | Relation3 / relation2 for inherited selectors / frozen engineering interpretation |
| Goals, scales, shared rules, goal assessments | New relation1; independent resolution and criterion outcomes retained |
| Completion | Optional default-empty retained goal assessments preserve exact historical no-goal JSON without adding accuracy claims |
| StudyRequest4, StudyDefinition6, JobPayload8, StoredPointOutcome1, publication ticket2, artifact descriptor3 | Their owned meaning remains; revised nested execution leaves gate interpretation |

Existing outer identities already frame consumed nested inputs; numerical policy V3 and
resolved-policy V2 identify the changed meaning. Recorded historical Arrow resolved_numerics1
remains readable under its recorded contract, without fabricating the new context. Current
context consumption requires explicit migration/readmission. Retained metric profile JSON
is historical provenance and is never fed through the new execution decoder. Stored bytes
are preserved. Final consumer checks and migration qualification remain pending.

## Packages and qualification handoff

| Package | Prerequisite | Delivery, integration owner and acceptance | Progress |
|---|---|---|---|
| C0 — Decisions and boundary inventory | Settled A0/shared target | ADR-0163, current-standard contracts review and consumed-version inventory above; root owns generated/request/completion migration | done |
| C1 — Pure assessment/completion | Working A3/B1 and C0 route | Completion owner implements physical envelope classification and composed use truth table; migrates immediate assessment consumers. Delete goal permission re-derivation. Targeted `engineering_goal_assessment` controls | in progress |
| C2 — Finite strategy refinement | Working C1/B2 and appropriate B producer | Strategy owner admits evidence/refinement operations, work demands, progress and typed stops under one task grant. Delete workflow-local loops and borrowed tolerances once replaced. Targeted `goal_refinement_strategy` controls | in progress |
| C3 — Retained and durable contracts | Working C1/C2 with C0 versions | Schema/operations/Python integration owner migrates retained reports, jobs and generated boundary surfaces together; deletes replaced interpretations/codecs/callers, preserving old data through the selected supported transition. Targeted `accuracy_completion_transport` controls | in progress |
| C4 — Complete workflow adoption | Working A/B/C1–C3 | Runtime integration owner migrates all actual consumers listed above and reference shared policy use; exercises real successful, violated, unavailable and finite-refusal journeys. No hidden old production path. Targeted `accuracy_workflow_adoption` controls | in progress |
| C5 — Stable handoff | All functional packages and immediate deletions complete | Coordinator hands final tree and accuracy acceptance/measurement scopes to 25k K3/K4/K5; scheduled independent review assesses actual integrated behavior and architecture | planned |

**Proposed targeted controls:** independently specified envelopes on both sides/touching a
threshold; combined resolution and decision outcomes; Estimated versus Certified evidence;
value-only/decision-only/no-goal and Assess versus RequireSatisfied; invalid validity and
unavailable observations; accepted violation without retry; a far-boundary decision without
extra output digits; true near-boundary refinement; exact boundary, nonprogress, precision
limits and zero/exhausted grants. Inject actual finite execution effects, not a fabricated
solver-success report. Check original obligations and terminal failure precedence through all
transitions, unique shared-work charging, no duplicate owner and bounded report retention.

Transport tests compare independently specified semantic rows, supported historical no-goal
readmission, unsupported version refusal and preservation/restart. Report/export inspection
must leave solve/evidence-operation counters unchanged. Real native/dynamic journeys consume
the same classifier and result contract; tests do not maintain alternative acceptance logic.

After functional scope, run scope-end hygiene, relevant integration/component/native/Python/
reference/parity journeys and manual checks through existing 25k recipes. Select commands from
`just --list` and the qualification guide at execution time; do not invent long ad hoc native
commands. Heavy native/parity/Python work remains serialized and memory-capped. Fix actual
failures against the zero baseline and rerun affected checks; record composite evidence honestly.

The accuracy measurement supplement compares no-goal overhead, already-separated decision
assessment and active refinement including total work/retained memory. Use the same scientific
conditions and declared requirements; retain branch/output differences and censored failures.
No speedup is promised. The existing 25k required K4 campaign remains owned there, with overlap
executed once rather than a second independent qualification.

## Verification and checkpoint

**Implemented / Interface-checked:** existing task accounting, product identities, composed
candidate use and immutable completion inspected at the coordinator baseline.
C0 is complete. C1–C3 assessment, finite strategy refinement and retained/generated
contracts are implemented with focused controls; C4 adoption is being integrated with
the static and dynamic producers. Optional evidence failures retain their typed cause
and the original candidate, while cancellation, task limits and contract failures keep
terminal precedence. Refinement recognizes newly resolved goals and changed work
precision without changing frozen acceptance. Native journeys and linked Python
transport validation precede C5's stable handoff. The generated Python codec now checks
schema-required version headers on explicitly supplied documents before constructor defaults
apply, including nested numerical policies and typed containers. Current constructors and lawful
omission of an owner-defaulted policy remain supported; retained no-goal completions retain their
historical interpretation. Focused generator, transport and native-boundary controls pass in the
regenerated tree. The final dynamic journey still precedes handoff. No performance measurements have yet
been produced for this series. A local C package pass does not qualify the unexercised
enclosing system. Final CA-F04 resolution and series closure require linked
corrective/assembled evidence at 25k, while this package table owns execution progress.

## Outcome (recorded after implementation)

Record delivery, a mistake corrected, deliberate deviations, scoped local/assembled evidence
and measurement limits here; link the final 25k finding/qualification decision.
