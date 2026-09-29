---
id: ADR-0105
title: Integrate SCIP 10.0.2 through a factorable projection and a pse-owned scip-sys adapter
status: accepted
date: 2026-09-27
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-06, DP-13, DP-15, DP-20, PS-09, PS-10]
blueprint: [§3.3, §18.7, §18.9]
review: git:f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc:docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#t07
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: SCIP or scip-sys changes major version, the runtime ABI check fails against the image, or the Q1 gap-closure measurement on the certify fixtures misses its recorded budget.
verification: Review scenario S01 and architecture scenarios S11, S12, S16 and S18, settled by the Plan 22 G1–G8 tests scip_abi_matches_image, projection_exact_rows_match_evaluator, relaxed_rows_enclose_evaluator, pcsaft_valid_guard_projects_exactly, scip_status_map_exhaustive, scip_interrupt_via_event_handler, scip_export_readback_equivalent, certify_known_global_optimum, relaxed_export_bound_only, nonlinear_iis_irreducible_flag and killed_worker_attempt_goes_stale_and_resumes_from_incumbent, plus the skill runtime receipt recorded in G1.
standard: core-3.0/process-simulator-1.1
scenarios: [docs/design_review/reviews/design_review_solver-capabilities_2026-09-27.md#s01, docs/plans/22-solver-capabilities-architecture.md#s11, docs/plans/22-solver-capabilities-architecture.md#s12, docs/plans/22-solver-capabilities-architecture.md#s16, docs/plans/22-solver-capabilities-architecture.md#s18]
---

# ADR-0105: Integrate SCIP 10.0.2 through a factorable projection and a pse-owned scip-sys adapter

## Context

ADR-0102 makes SCIP the owner of MIQP, MINLP, global certification, global infeasibility and
the nonlinear IIS. ADR-0083's revisit trigger — a concrete problem class needs an additional
native interface — fires. Solvers see callbacks only: Symbolica atoms stay inside `pse-math`
and FBBT tapes cover rows but no objective, so no representation SCIP can consume exists
([capability review §8.1](../design_review/reviews/design_review_solver-capabilities_2026-09-27.md#81-scip-the-maintainers-hypothesis)).
The P1 census found 92.9% of the rows of the 36 steady reference cases exactly representable;
the remainder is provider outputs, nested implicit realizations and a validity guard lowered
into a branch stage.

## Scope

Adds a native interface within ADR-0083, whose outcome — distinct class-specific interfaces
under one lifecycle — stays true, so it is not superseded. Binds the representation, the
binding, the adapter lifecycle, reserved options, status mapping, capabilities, routing and
the SCIP components of the solver image. The image contract itself (one Ipopt, one oneMKL,
one OpenMP runtime, checksummed sources) is ADR-0108; the outcome vocabulary is ADR-0106.

## Drivers

- **PS-09 and DP-13.** SCIP owns spatial branch-and-bound, cutting, presolve and MINLP search.
- **PS-10.** Every SCIP status maps to a typed outcome; the candidate is re-qualified in
  original coordinates independently of SCIP's claim.
- **AP-02.** No SCIP type crosses the adapter; `FactorableProgram` is library-neutral.
- **AP-06.** The projection is testable without SCIP; the adapter is testable with a fixture.
- **DP-15.** The binding's ABI and version are checked at run time against the image.
- **DP-20.** Native threads (SCIP concurrency, SCIP's nested Ipopt) are admitted, not ambient.

## Options

| Option | Assessment | Selection |
|---|---|---|
| `russcip` safe wrapper | Panicking `From` conversions on statuses; `Rc` handles add nothing the adapter needs | Rejected |
| `scip-sys` `bundled` or `from-source` profiles | Unverified prebuilt download carrying a second Ipopt; or `IPOPT=OFF`, no PaPILO, no exact mode | Rejected |
| A user expression handler wrapping opaque evaluators | Needs rigorous interval and estimator callbacks, not derivatives; would forfeit global guarantees | Rejected |
| Raw `scip-sys` against the image's `SCIPOPTDIR`, fed by a neutral `FactorableProgram` with per-row fidelity | Exact where the model is factorable, sound relaxation elsewhere, testable without SCIP | **Selected** |

## Outcome

1. **`FactorableProgram` in `pse-math`.** A library-neutral DAG projected from Symbolica atoms
   beside the FBBT projection, never replacing the evaluator.
   - Nodes: Var; Const (rational or f64); n-ary Sum and Product; Pow with a rational exponent;
     Exp, Log, Abs, Sin, Cos; Min and Max through the absolute-value identity; `Aux`, a free
     auxiliary standing for an opaque output.
   - Every row and the objective carry fidelity `Exact`, `Relaxed` or `Unavailable`.
   - Obligations (`Require`, `Domain`) become closed bounds or constraints, never expression
     dependencies, so a validity guard cannot poison projection.
   - Implicit blocks export their residual equations and declared bounds, whatever their
     evaluation realization.
   - Providers become `Aux` within enforced envelopes (`Relaxed`). Branches with proven
     continuity become disjunctions (exact, mixed-integer) or `Aux` (relaxed) by a declared
     policy.
2. **Relaxation soundness.**
   - A `Relaxed` export supports a dual bound and a global infeasibility conclusion, never an
     optimality or solution claim.
   - Every candidate from any export is re-qualified in original coordinates by `quality.rs`.
   - An `Unavailable` objective yields no bound, and `Certify` is refused with that reason.
   - For a mixed-integer program with relaxed rows, SCIP's incumbent is an assignment
     proposal. The candidate comes from the fixed-assignment continuous re-solve through the
     one NLP runner (ADR-0103, Plan 22 M2). A gap may combine the relaxation's dual bound with
     that original-qualified candidate; both sources are recorded.
3. **Binding.** Raw `scip-sys` built against the image's `SCIPOPTDIR`. At run time the adapter
   checks `SCIPmajorVersion`, `SCIPminorVersion`, `SCIPtechVersion`, `SCIP_APIVERSION`,
   `SCIP_Real = f64` and the index width; a mismatch is a typed `Unsupported` refusal.
4. **Lifecycle.** One `SCIP*` per attempt, created, used and freed on the owning worker (§18.8).
5. **Reserved options.**
   - `misc/catchctrlc = FALSE`, mandatory under PyO3;
   - `limits/time` from the deadline and `limits/memory` from the foreign allowance;
   - random seeds recorded;
   - concurrent solving only in deterministic mode, with the thread count equal to the
     admitted permits;
   - SCIP's nested Ipopt: `nlpi/ipopt/linear_solver` is set from the typed Ipopt linear-solver
     setting (default `mumps`) and `nlpi/ipopt/hsllib` and `nlpi/ipopt/pardisolib` are refused;
     the nested Ipopt runs serial unless threads are admitted for it.
6. **Cancellation.** An event handler polls the attempt flag and calls `SCIPinterruptSolve` on
   the owning thread.
7. **Status mapping.** Raw `SCIPgetStatus` maps exhaustively to `NativeTermination`; no
   wildcard arm.
8. **Capabilities.** Incumbent injection (`SCIPaddSolFree`) from Ipopt or POUNCE;
   export-equivalence readback; native indicator, SOS and logic handlers (ADR-0104); a ranked
   solution pool; reoptimization for MIP sequences; exact rational MILP
   (`SCIPenableExactSolving` before problem creation, incompatible with reoptimization); IIS
   on the true problem (`SCIPgenerateIIS`) with its irreducible flag; an incumbent and bound
   event stream to the operational store (ADR-0112).
9. **Routing** through the backend-execution adapter table (Plan 22 A2) and one capability
   record: automatic for `mixed_integer_quadratic` and `mixed_integer_nonlinear`; `Certify`
   for `smooth_nlp` and `nonconvex_quadratic`; explicit for `linear`, `mixed_linear` (including
   exact mode) and `convex_quadratic`. SCIP is not used for DegeneracyHunter or tear selection.
10. **Solver-image components** (under ADR-0108): checksummed scipoptsuite 10.0.2; SoPlex LP
    (`LPS=spx`); `IPOPT=ON` against the image's single Ipopt; `PAPILO=ON`; GMP, MPFR and Boost
    for exact mode; `THREADSAFE=ON`; `TPI=tny`, so SCIP's concurrency does not share the OpenMP
    runtime settings SPRAL requires.

### Consequences

A long solver-image build and a larger image. A new representation in `pse-math` with its own
tests. Rigorous-thermodynamics cases certify only as relaxations until implicit residuals are
exported. Global certification is small-scale.

### Compensating controls

The runtime ABI check; export readback; exhaustive status map; projection tests with a
negative control; skill runtime receipt before adoption (Plan 22 G1).

### Confirmation

The tests named in `verification:`. *Interface-checked*: the scip-sys 0.1.28 / SCIP 10.0.2
entry points named here were read in the `native-solver-libraries` corpus by the capability
review (§8.1), and `SCIPenableExactSolving`'s precondition (before problem creation, not with
reoptimization) was checked in the SCIP documentation. Nothing was built; value is *Proposed*.

## Pros and cons

The neutral projection makes SCIP additive and testable, and it also feeds cone recognition
(Plan 22 C5). The raw binding carries more FFI code than `russcip`, in exchange for exhaustive
status mapping and no panicking conversions.

## More information

- Architecture companion [§5](../plans/22-solver-capabilities-architecture.md#5-factorable-projection-and-scip).
- Target review [T07](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#t07) (relaxed-export semantics) and [T08](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#t08) (nested Ipopt settings).
- Capability review [§8.1](../design_review/reviews/design_review_solver-capabilities_2026-09-27.md#81-scip-the-maintainers-hypothesis) and qualification steps 1–7 (slot 10).
- Related: ADR-0083 (revisit trigger fired), ADR-0102, ADR-0104, ADR-0106, ADR-0108, ADR-0112. Plan 22 packets G1–G8, C5.

## Status history

- 2026-09-27 — proposed (Plan 22 D0).
- 2026-09-27 — accepted under the maintainer's authorization of the full Plan 22 scope (2026-09-27), after the [Plan 22 target review](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#decision) returned Accept (author review, Proposed evidence level). Findings T07, T08 were corrected in this record before acceptance.
