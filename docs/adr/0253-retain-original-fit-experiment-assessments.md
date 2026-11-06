---
id: ADR-0253
title: Retain original fit experiment assessments across partial failure
status: proposed
date: 2026-11-06
deciders: [paul-heyse]
level: decision
principles: [AP-04, DP-01, DP-05, DP-19, PS-11, PS-12]
blueprint: [§19.2, §19.4, §19.8]
review: "not-required: bounded observational relation and profile extension within authorized Plan66 A02/J; existing native qualification and scientific permission remain unchanged"
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: A consumer requires experiment/member scientific use permission or a physical-time/source/mesh scope beyond the retained original assessment observations.
verification: Fresh multi-experiment fits retain completed and partially failed original checks/reports, distinguish failed from unattempted experiments after later failure or cancellation, preserve exact declared InstanceId and physical row values, refuse global qualification after incomplete assessment, and expose the same observations through actual diagnostic, profile, trajectory and Arrow result consumers.
---

# ADR-0253: Retain original fit experiment assessments across partial failure

## Context

`PreparedFit::assess_candidate` returns flattened checks/reports or an error, discarding
earlier experiments' evidence after a later failure. Profile pins consume the same lossy
result. Original trajectories already borrow the exact joined producer and report.

## Scope

Extend the fitting observations under blueprint §19.2/§19.4/§19.8. This record introduces
passive assessment evidence, not experiment/trajectory use permission, a native control
format or a change to global fit estimate qualification. Plan66a A02 owns the remaining
governing-scope work.

## Drivers

Preserve original evidence and attribution during failure, cancellation and partial
assessment. Keep one owned check/report population, exact declared experiment identities,
and existing physical time/target/source facts. Inspection and export must not repeat a solve.

## Options

Keeping the current fail-fast tuple loses useful sibling evidence. Independently owned
flat and member values create competing facts. Select one native assessment with immutable
flat check/report buffers and private disjoint per-experiment ranges, explicit completed,
failed and unattempted states, and original structured failures. Derived views expose those
same values. A relation per profile scope or synthetic point/time hashes adds unnecessary
identity machinery; existing profile point keys already identify their original pins.

## Outcome

Add `runtime.fit_experiment_assessments`, keyed by original `(run_id, experiment_id)`.
Its original declared `InstanceId`, assessment state, original check/report ranges and
structured failure are observational. The native math catalog owns this field shape once;
the final-estimate relation and a typed per-experiment list in `profile_points.original_assessment`
consume it. That optional named record also derives assessment completeness and retains an
assessment-wide interruption independently of member failures.
The profile relation retains its existing parameter/end/original point ordinal key and
increments its interpretation version. Historical absent assessment metadata remains
unavailable rather than being inferred from pin acceptance or whole-fit permission.

Only the joined fitting producer selects original assessment evidence. Later member failure
retains completed siblings and any partial rows from the failing member. Cancellation stops
new work; remaining declarations stay explicitly unattempted without invented failures.
All distinct failures remain inspectable even when strategy/profile APIs require a derived
primary cause. Existing `modeling_checks` and `modeling_reports` export the sole retained
row population in original order; range coordinates describe that population, not scientific
point identity or use permission.

### Consequences

Native final-fit and profile consumers migrate together, replacing independently mutable
flat completeness/error fields. Original diagnostics, known owned heap and result exports
include the retained member evidence. Global fit and profile-statistic qualification still
require their existing complete original assessment, numerical and statistical conditions.

This unit preserves observations actually returned by a scientific member owner. The
steady `assess_observations` owner still returns an all-or-error tuple and can discard
its own internal partial buffer; addressing that producer remains separate A02 scientific
work. No unreturned row is recovered or invented by this fitting assessment.

### Compensating controls

Validate candidate extent and declared/source inventories before indexed reads. Preserve
raw failed trajectories under their actual joined borrow. Challenge later failure, partial
member rows, unattempted siblings, cancellation, foreign/duplicate identity, nonzero physical
time, profile refusal and independent Arrow readback. No local member grant is synthesized.

### Confirmation

The current producer, profile calls, canonical row exporters and joined trajectory owner
were inspected before this decision. Functional evidence belongs to the implementation
commit and Plan66 acceptance; this proposed record does not claim qualification.

## Pros and cons

One native owner preserves partial evidence and supports attributable inspection without
repeating work. It adds a bounded per-experiment inventory and a new observational relation;
the larger permission/physical-scope problem remains explicit rather than being approximated.

## More information

- [Plan66a](../plans/66a-scientific-operations-and-product-semantics.md)
- [Fitting producer](../../crates/pse-runtime/src/workflow/fitting/modeling.rs)
- [Profile producer](../../crates/pse-runtime/src/workflow/fitting/profile.rs)
- [Native math catalog](../../crates/pse-schema/src/catalog/native_math.rs)
- [Joined trajectory owner](../../crates/pse-runtime/src/workflow/fitting/trajectory.rs)

## Status history

- 2026-11-06 — Proposed before the observational catalog/export extension in authorized Plan66 execution.
