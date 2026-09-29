---
id: ADR-0109
title: Expose POUNCE l1 exact penalty and POUNCE-convex as explicit native methods
status: accepted
date: 2026-09-27
deciders: [paul-heyse]
level: decision
principles: [DP-13, DP-15, PS-09, PS-10, PS-12]
blueprint: [§15.5, §18.9]
review: git:f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc:docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#t14
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: A POUNCE release changes the l1 or convex method surface, or an l1 or POUNCE-convex candidate fails original-space qualification where the declared semantics say it should qualify.
verification: Plan 22 N3 and N5 tests l1_route_returns_labelled_least_infeasible_point, l1_never_automatic, pounce_convex_qp_matches_highs, pounce_convex_batched_study and sos_bound_labelled_nonrigorous; M5 test flash_phase_disappearance_agrees_across_realizations; A3 test pounce_retry_options_reserved_and_snapshotted.
standard: core-3.0/process-simulator-1.1
scenarios: [docs/plans/22-solver-capabilities-architecture.md#s18]
---

# ADR-0109: Expose POUNCE l1 exact penalty and POUNCE-convex as explicit native methods

## Context

`pounce-l1penalty` (the Thierry–Biegler ℓ1 exact penalty-barrier, the same method as IDAES
`ipopt_l1`) is already linked through `pounce-algorithm` at `=0.12.0`, but it is reachable only
through untyped options and POUNCE's own restoration fallback
([F03](../design_review/reviews/design_review_solver-capabilities_2026-09-27.md#f03)). Plan 20
§5 proposed a bespoke whole-model elastic formulation where this route exists
([L-N5](../design_review/reviews/design_review_solver-capabilities_2026-09-27.md#l-n5)).
`pounce-rs` feature `convex` offers an interior-point method for LP, QP and cones with warm
start, batched QP solves, QP parametric sensitivity and SOS polynomial bounds (L-N7).

## Scope

New backend bindings within ADR-0083; nothing is superseded. Binds how these methods are
selected, what they may claim and where the runtime uses them.

## Drivers

- **DP-13.** Use the linked ℓ1 method rather than a bespoke elastic formulation for the whole model.
- **DP-15.** No fallback: a method runs only when typed and selected.
- **PS-09.** Selection by declared capability and explicit policy, recorded.
- **PS-10 and PS-12.** A least-infeasible point and a floating-point SOS bound are labelled for
  what they are.

## Options

| Option | Assessment | Selection |
|---|---|---|
| Leave ℓ1 reachable through raw options and POUNCE's internal fallback | Hidden second attempts (F03, G4) | Rejected |
| Bespoke whole-model elastic formulation in the runtime | Reimplements a linked library method (G8) | Rejected |
| ℓ1 as an automatic retry after restoration failure | A fallback, which §18 forbids | Rejected |
| Typed explicit methods: `L1ExactPenalty` and a POUNCE-convex backend | Explicit, recorded, library-owned | **Selected** |

## Outcome

1. **`L1ExactPenalty`.** The POUNCE method enum gains `L1ExactPenalty`. It is never selected
   automatically and never used as a retry. POUNCE's `l1_fallback_on_restoration_failure` and
   `l1_exact_penalty_barrier` options are reserved, so the typed method is the only way in.
2. **Uses.** (a) Whole-model infeasibility explanation, replacing the runtime's bespoke
   whole-model elastic formulation; authored elastic overlays on selected constraints remain
   under ADR-0101. (b) The authored `penalty(l1)` complementarity realization (ADR-0104), where
   the authored realization is the explicit selection. (c) Explicit user selection.
3. **Outcome semantics.** The route returns either a candidate that `quality.rs` qualifies in
   original coordinates — the penalty is exact at a nondegenerate solution with a sufficient
   penalty parameter — or a labelled least-infeasible point, recorded as `diagnostic_only`
   (ADR-0106) with its violated constraints named.
4. **POUNCE-convex backend** (`pounce-rs` feature `convex`) through the backend-execution
   adapter (Plan 22 A2): interior point for LP, QP and cones with warm start; batched parallel
   QP solves for studies, with threads admitted under §18.8; QP parametric sensitivity under
   ADR-0107's validity record; SOS polynomial lower bounds labelled `sos_bound_nonrigorous`.
   It is an explicit alternative for `linear`, `convex_quadratic` and `continuous_cone`; HiGHS
   and Clarabel remain the automatic owners.

### Consequences

The whole-model part of Plan 20 §5's elastic policy is replaced (Plan 20 amended in D0). The
runtime deletes its bespoke whole-model elastic formulation when N3 lands. A new backend enters
through the adapter table with its capability record.

### Compensating controls

Reserved POUNCE options with an effective-option snapshot (Plan 22 A3); the `l1_never_automatic`
test; labelled non-rigorous bounds that never combine into a gap claim.

### Confirmation

The tests named in `verification:`. *Interface-checked*: `pounce-l1penalty` is linked in the
current lockfile and the `convex` feature surface was read by the capability review (L-N5,
L-N7); neither is exercised yet.

## Pros and cons

Both methods are library-owned and cheap to bind. POUNCE-convex overlaps HiGHS and Clarabel;
its distinct value (batched QP, QP sensitivity, SOS bounds) justifies an explicit-only route,
not a second automatic owner.

## More information

- Architecture companion [§2.5, §6.3, §8](../plans/22-solver-capabilities-architecture.md#6-sensitivity-covariance-and-uncertainty).
- Target review [T14](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#t14).
- Capability review F03, L-N5, L-N7.
- Related: ADR-0083, ADR-0104, ADR-0106, ADR-0107. Plan 22 packets N3, N5, A3.

## Status history

- 2026-09-27 — proposed (Plan 22 D0).
- 2026-09-27 — accepted under the maintainer's authorization of the full Plan 22 scope (2026-09-27), after the [Plan 22 target review](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#decision) returned Accept (author review, Proposed evidence level). Findings T14 were corrected in this record before acceptance.
