---
id: ADR-0106
title: Extend the shared execution vocabulary for discrete, global and certified outcomes
status: accepted
date: 2026-09-27
deciders: [paul-heyse]
level: decision
principles: [AP-04, DP-01, DP-02, DP-21, PS-10, PS-12]
blueprint: [§4.2, §16.6, §18.6, §18.7, §21.1, §23.2]
review: git:f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc:docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#t01
evidence: Proposed
supersedes: [ADR-0090]
superseded-by: null
revisit: A new execution or storage path cannot preserve these distinctions, or a backend reports an outcome that none of the tags can state without loss.
verification: Vocabulary codec round trips (Rust, Arrow, Python) after just codegen; the Plan 22 A1 tests candidate_use_iteration_limited_feasible_is_seed_only_everywhere, adapter_not_linked_is_unsupported, structural_failure_keeps_rows_and_columns, idas_conv_fail_is_numerical and quality_reads_typed_evidence_only; G4/G5 tests certify_known_global_optimum, relaxed_export_bound_only and global_infeasibility_proof; N5 test sos_bound_labelled_nonrigorous.
standard: core-3.0/process-simulator-1.1
scenarios: [https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/22-solver-capabilities-architecture.md#s12, docs/design_review/reviews/design_review_solver-capabilities_2026-09-27.md#s01]
---

# ADR-0106: Extend the shared execution vocabulary for discrete, global and certified outcomes

## Context

ADR-0090 made the schema registry the owner of the shared execution vocabulary. ADR-0102
admits mixed-integer and global classes, an explicit certification intent and new kinds of
assurance, none of which the vocabulary can state: `NativeProblemClass` has no nonconvex,
MIQP or MINLP value; `NativeAssurance` stops at `certificate`; `SolveIntent` is a
backend-private Rust enum. The capability review's addendum found five disagreeing
definitions of an "accepted candidate"
([F13](../design_review/reviews/design_review_solver-capabilities_2026-09-27.md#f13)),
typed failures flattened into `Contract(String)` ([F17](../design_review/reviews/design_review_solver-capabilities_2026-09-27.md#f17))
and a diagnostic vocabulary without severity, numerical failure or inconclusive
([F19](../design_review/reviews/design_review_solver-capabilities_2026-09-27.md#f19)).

## Scope

Supersedes ADR-0090 and restates its whole outcome, which remains in force, together with the
additions. Binds vocabulary and ownership, not adapter mechanics. Registry changes are
followed by `just codegen`.

## Drivers

- **DP-01 and AP-04.** One owner per tag and one owner of the candidate-use decision.
- **DP-02.** Outcomes, assurances and failure categories are types, not prose.
- **PS-10.** Native termination, numerical qualification, physical closure and final
  usability remain independent facts.
- **PS-12.** An assurance states the conditions under which it holds.
- **DP-21.** Diagnostics distinguish invalid model, unsupported, numerical, infrastructure and
  inconclusive, and carry severity.

## Options

| Option | Assessment | Selection |
|---|---|---|
| Amend ADR-0090 in place | Accepted records are immutable | Not permitted |
| Add backend-specific assurance fields in the SCIP adapter | Consumers would branch on backend objects (AP-02) | Rejected |
| A second, SCIP-only outcome vocabulary | Two authorities for one meaning (DP-01) | Rejected |
| Supersede ADR-0090 with one extended registry vocabulary | Single owner, codec round trips, generated consumers | **Selected** |

## Outcome

**Retained from ADR-0090, unchanged.**

1. The schema registry owns durable semantic tags, projected into library-neutral `pse-model`
   values, Arrow codecs and Python contracts. Backend identity, native termination, candidate
   kind, qualification, unavailable-evidence reasons and metric alternatives stay distinct.
   Heat input and output are distinct from work; an absolute time origin is distinct from an
   elapsed duration.
2. Adapters own backend-native behaviour and translate statuses into shared tags. Static
   capability inventory is distinct from contextual eligibility and from the selected execution.
3. Boundary diagnostics classify invalid model, unsupported capability, resource exhaustion,
   trial rejection, nonfinite evaluation, infrastructure failure, cancellation, conflict,
   incompatibility and internal failure, and retain source identities, stage and
   observations. Missing evidence uses typed reasons, never invented NaN observations.
4. Completion records the actual request, preparation, effective backend settings, submitted
   start, provider data and environment lineage. Arrow and Python consume that immutable
   product; reading it cannot evaluate or reclassify the model. Diagnostic capture is bounded
   and optional and cannot change a scientific or publication outcome.
5. Physical vocabulary derives from `pse-quantity`, shared validation from the resolved
   registry obligations. Resolved numerical requirements, source-attributed presolve evidence,
   exact/numerical/inconclusive convexity evidence and the immutable candidate assessment are
   part of the vocabulary; native termination, numerical acceptance, physical closure and
   final usability remain separate facts.
6. The authored scalar-function vocabulary and exhaustive handler tags live in
   `pse-quantity::functions`; `pse-math` re-exports and executes them.

**Added.**

7. **Problem classes.** `NativeProblemClass` gains `nonconvex_quadratic`,
   `mixed_integer_quadratic` (with convexity recorded as a fact) and `mixed_integer_nonlinear`.
8. **Intent.** `SolveIntent` becomes a registry enum — `optimize`, `root`, `feasible_point`,
   `initialize`, `certify` — because request identity, lineage and Python consume it. `certify`
   is only ever selected explicitly.
9. **Assurances.** `NativeAssurance` gains four values, each with its conditions:

   | Value | Meaning | Rigour |
   |---|---|---|
   | `global_bound` | A dual bound on the exported program over the declared box | Valid within the backend's recorded feasibility and optimality tolerances and the export fidelity; not interval-rigorous |
   | `proven_infeasible` | The backend's global infeasibility conclusion for the exported program | As `global_bound`; a relaxed export keeps it sound; not interval-rigorous |
   | `exact_certificate` | Optimality or infeasibility established in rational arithmetic (exact MILP) | The only rigorous assurance |
   | `sos_bound_nonrigorous` | A floating-point SOS polynomial lower bound | Never a certificate; never combined into a gap claim |

10. **Gap qualification.** `GapQualified` extends to SCIP and records the box, tolerances,
    export fidelity, the dual-bound source and the primal-candidate source.
11. **Candidate use.** `CandidateUse` remains the single typed decision, owned by the workflow
    completion owner (`pse-runtime/src/workflow/numerics.rs`), and is extended rather than
    replaced:

    | Value | Meaning |
    |---|---|
    | `usable` | Every required original-coordinate check passed |
    | `qualified_unclosed` | Numerically qualified; closure failed under explicit `AllowUnclosed` |
    | `seed_only` | Feasible in original coordinates, but the native stop forbids use as a result (for example an iteration limit); it may seed a later step and is never published as a solution |
    | `diagnostic_only` | A relaxed-export point or a least-infeasible point; an observation only, never a seed or a result |
    | `unusable` | Anything else |

    Every workflow — solve, initialization, homotopy, study, fitting, diagnostics and
    publication — consumes this value. No workflow re-derives acceptance.
12. **Failures.** `ProblemError` gains `Provider`, `Unsupported`, `Numerical`, `Limit`,
    `Cancelled` and `Internal` variants, each keeping its cause and structural identities; the
    `Contract(String)` flattening is removed. The boundary vocabulary gains `numerical`
    (algorithmic failure: convergence or error-test failure, a singular factorization without
    a model cause) and `inconclusive`, and every diagnostic carries a severity (`error`,
    `warning`, `info`). A scaling warning is a warning, not an invalid model.

### Consequences

Registry, generated code and Python contracts change together. Every workflow that decides
acceptance today is rewired to `CandidateUse` (Plan 22 A1). Adapters must return typed evidence
structs instead of string-keyed metrics for qualification (F16).

### Compensating controls

Codec round trips; exhaustive matches without wildcards; the A1 test that one iteration-limited
feasible candidate is `seed_only` in every workflow and in the published assessment.

### Confirmation

The tests named in `verification:`. The review's slot 3 records the assurance conditions.
Whole-system acceptance is not inferred from them.

## Pros and cons

One vocabulary keeps outcome meaning singular across Rust, Arrow and Python. The cost is a
coordinated registry change touching every consumer of `CandidateUse`, which is exactly the
duplication F13 exposed.

## More information

- Architecture companion [§3–§4](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/22-solver-capabilities-architecture.md#3-problem-classes-routing-and-assurance-vocabulary).
- Target review [T01](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#t01) (candidate-use vocabulary) and [T10](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#t10) (assurance rigour).
- Capability review findings F13, F16, F17, F19.
- Supersedes ADR-0090. Related: ADR-0102, ADR-0105, ADR-0109. Plan 22 packets A1, G4, G5, N5.

## Status history

- 2026-09-27 — proposed (Plan 22 D0). Supersedes ADR-0090: its outcome is restated in full
  above; the additions are the classes, intent, assurances, candidate-use values and failure
  categories.
- 2026-09-27 — accepted under the maintainer's authorization of the full Plan 22 scope (2026-09-27), after the [Plan 22 target review](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#decision) returned Accept (author review, Proposed evidence level). Findings T01, T10 were corrected in this record before acceptance.
