---
id: ADR-0121
title: Establish convexity and cone recognition as compiler facts
status: accepted
date: 2026-09-28
deciders: [paul-heyse]
level: decision
principles: [PS-09, AP-04, DP-15, DP-13, DP-16]
blueprint: [§7.5, §18.6, §18.7, §18.9, §18.10]
review: docs/design_review/reviews/design_review_solver-scope-decisions_2026-09-28.md#adr-0121
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A maintained Rust library for curvature analysis or conic recognition over an expression DAG appears; curvature_sound_against_scip_oracle finds a row that the pass calls convex and SCIP proves nonconvex; or the C5 census shows that exact certificates exceed the preparation budget on the reference cases.
verification: Plan 22 C4 tests clarabel_lp_matches_highs, clarabel_qp_farkas_certificate, farkas_certificate_verified_in_original_coordinates, almost_infeasible_is_not_certified, clarabel_mkl_pardiso_matches_qdldl, clarabel_chordal_matches_undecomposed and explicit_only_classes_never_automatic, with the C4 image loading check. C5 tests gram_certificate_yields_soc, exact_ldlt_certifies_nondiagonal_psd, recognized_exp_cone_routes_to_clarabel, unrecognized_problem_not_routed, convexity_fact_rebinds_with_values, numerical_psd_only_under_explicit_policy and curvature_sound_against_scip_oracle, with the C5 census rerun for coverage and preparation time.
standard: core-3.1/process-simulator-1.1
scenarios: [https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/22-solver-capabilities-architecture.md#s18, docs/design_review/reviews/design_review_solver-scope-decisions_2026-09-28.md#s05]
---

# ADR-0121: Establish convexity and cone recognition as compiler facts

## Context

**Convexity is decided at run time.** Convexity of a quadratic is decided per run in
`pse-runtime` `math/solves.rs`:
- a floating-point LDLᵀ without pivoting proposes Gram factors;
- Symbolica verifies them exactly (`GramCertificate`);
- `Requirements.convex` carries the verdict into routing.

This proves diagonal matrices and some nondiagonal ones, but not every positive
semidefinite matrix.

**Routing cannot recognize cones.** Cones are never inferred from algebraic facts, so
Clarabel is reached only through an explicit cone request (blueprint §18.7). Routing gives
each adapter one automatic rank, which makes an adapter automatic either for all of its
classes or for none. Plan 22 C4 and C5 add Clarabel for explicit LP and convex QP, and
automatic cone recognition as a compiler fact (capability review L-C5, L-C6, L-C7).

**Clarabel's faer backend and its certificates.**
- Clarabel 0.11.1's `faer-sparse` feature requires faer 0.21.x, beside the workspace's pinned faer 0.24.4.
- Infeasibility certificates are reported as string-keyed `certificate.*` metrics and a `Certificate.kind: String`.

The solver scope packet adopts improvements I10 and I11.

## Scope

**Within earlier decisions.** Within [ADR-0102](0102-discrete-and-global-design-target.md)
(routing follows compiler facts), [ADR-0105](0105-scip-factorable-backend.md)
(`FactorableProgram`) and [ADR-0106](0106-execution-vocabulary-discrete-and-global.md)
(exact, numerical and inconclusive convexity evidence). None is superseded.

**Governed sections.** The review checked which sections own the contract; §18.9 alone
does not:
- §7.5 owns `ProblemFacts`;
- §18.6 owns certificate outcomes;
- §18.7 owns selection;
- §18.9 publishes the capability records;
- §18.10 owns the HiGHS and Clarabel adapters.

**Image changes.** A change to the solver image for MKL Pardiso would be ADR-0122, written
only if the C4 loading check needs it. Implementation is Plan 22 C4 and C5.

## Drivers

- **PS-09.** Solver selection follows the problem class and declared capability. The class comes from facts, not from a per-run side computation.
- **AP-04 and DP-01.** Convexity has one authority: a compiler fact with a value identity.
- **DP-15.** Library boundaries are qualified, and each type-sharing family keeps one resolved version.
- **DP-13 and DP-16.** Library first. Bespoke curvature analysis is chosen only because no library fits (below), and SCIP's analysis serves as an independent oracle.
- **PS-10.** A certificate is typed and verified in original coordinates before it counts.

## Options

| Option | Assessment | Selection |
|---|---|---|
| Keep the run-time exact decision | Not a fact, so it is recomputed per run and invisible to preparation. Floating candidate factors miss many PSD matrices, and cones are never recognized | Replaced |
| SCIP curvature detection during preparation | Needs a native SCIP instance per preparation, from an optional feature. The compiler sits below the backends and must not depend on one | Rejected for production; kept as a differential test oracle |
| Numerical PSD (faer eigenanalysis) as a fact | Depends on tolerance and policy, so it is not a fact | Rejected as a fact; stays an explicit run-time policy |
| **A DCP pass over `FactorableProgram` with exact rational LDLᵀ Gram certificates, recorded in `ProblemFacts.convexity`** | Exact, established at preparation, and rebound with values | **Selected** |

**Library consideration (core §F), the reason for a bespoke pass.**
- *Capability:* prove the curvature of each row and of the objective, and recognize cone-representable atoms, exactly, over the factorable DAG that preparation already builds.
- *Candidates:*
  - No maintained Rust curvature-analysis library exists; the planning survey found none.
  - CVXPY implements DCP, but it is Python and works on its own expression trees.
  - SCIP detects curvature, but needs a native instance per preparation, and the compiler sits below the backends.
- *Integration owner:* `pse-math` (`curvature.rs`). The exact arithmetic is Symbolica's rationals.
- *Decision:* build, bounded to DCP composition rules over the existing `FactorableProgram` node vocabulary and the atoms listed in item 1.
- *Revisit:* the trigger in `revisit:`.

## Outcome

1. **Curvature pass.** A DCP pass over the `FactorableProgram` that preparation already builds, with signs from the FBBT box, proves curvature and maps atoms to cones:
   - exp, log, entropy and relative entropy map to the exponential cone;
   - pow maps to the power cone;
   - abs and norms map to second-order cones;
   - affine rows stay affine.

   Only rows and objectives of `Exact` fidelity count. A `Relaxed` or `Unavailable` one is never convex by recognition.
2. **Exact Gram certificates.** `convexity.rs` computes an exact rational LDLᵀ with symmetric pivoting (`MathGramV2`, factors kept). Within its resource limits this decides positive semidefiniteness of a rational symmetric matrix, and a binary64 matrix is rational. An exhausted limit is `inconclusive`, never convex.
3. **The fact.** `ProblemFacts.convexity` records the result.
   - `ValueProducts` carries it, so it rebinds with values and carries the identity of the values it consumed (§7.5).
   - Routing emits `convex_quadratic` and `continuous_cone` from it.
   - The run-time exact decision in `math/solves.rs` and `Requirements.convex` are deleted.
4. **Numerical policy stays run time** (review F05).
   - `ConvexityPolicy::Numerical` remains an explicit qualification of one request. Under it, that request's residual-qualified PSD assessment may add `convex_quadratic` for that request only, recorded as numerical evidence with its tolerances (ADR-0106 Outcome 5).
   - It is never stored as a fact, never rebinds and never serves another request.
   - Without the policy, only the fact counts.
5. **SCIP as oracle only.** `curvature_sound_against_scip_oracle` compares the pass with SCIP's curvature detection on test models. No production path consults SCIP for curvature.
6. **Automatic ownership per class.**
   - `Capability.automatic_classes` lists the classes an adapter may be selected for without an explicit request, and `runtime.solver_capabilities` publishes it.
   - Within one class, the existing rank orders the eligible automatic owners. `routing::problem_classes` orders classes from most specific to least.
   - Clarabel's classes become `linear`, `convex_quadratic` and `continuous_cone`, and its only automatic class is `continuous_cone`. Clarabel owns LP and convex QP explicitly, while HiGHS stays their automatic owner, so a recognized cone problem routes automatically to Clarabel.
   - Every other adapter's current automatic behaviour is restated as its automatic classes, unchanged. POUNCE-convex has none (ADR-0109).
   - The coefficients runner lowers to cone form for a conic adapter and re-evaluates the original model at the candidate (PS-10).
7. **Clarabel's direct solver.**
   - `faer-sparse` is excluded. It pulls faer 0.21.9 beside the pinned 0.24.4, which breaks one version per family, and it duplicates the threaded direct solve.
   - `Settings.direct ∈ {qdldl, mkl_pardiso}`, and QDLDL with more than one thread is refused.
   - MKL Pardiso (Cargo feature `clarabel/pardiso-mkl`) is adopted subject to the C4 loading check against the solver image.
8. **One typed, verified infeasibility certificate.**
   - `InfeasibilityCertificate{kind, accuracy, ray, verification}` is verified in original coordinates, for Clarabel and HiGHS rays alike, and published in `runtime.infeasibility_certificates`.
   - An almost-infeasible status is never certified.
   - It replaces the `certificate.*` metrics and `Certificate.kind: String`.

### Consequences

- Preparation pays for exact certificates and the curvature pass. The C5 census measures that cost.
- The registry gains `automatic_classes`, the convexity fact and the certificate relation, followed by `just codegen`.
- Routing tests asserting that Clarabel refuses LP are deleted.
- A value rebind can change a problem's class, and so its route, between study points.

### Compensating controls

- Refusal of explicit-only classes in automatic selection (`explicit_only_classes_never_automatic`).
- The SCIP differential oracle.
- Re-evaluation of the original model at every lowered candidate.
- Negative controls: `unrecognized_problem_not_routed` and `almost_infeasible_is_not_certified`.

### Confirmation

The tests and checks named in `verification:`, run in Plan 22 C4 and C5. The
[change-tier review](../design_review/reviews/design_review_solver-scope-decisions_2026-09-28.md#adr-0121)
interface-checked Clarabel 0.11.1's manifest: `faer-sparse` depends on faer 0.21.9, and
`pardiso-mkl` enables `pardiso-wrapper/mkl`. The decision is *Proposed*.

## Pros and cons

Convexity becomes a fact that preparation, routing and publication all see, and cones become
reachable without an explicit request. The cost is a bespoke DCP pass, bounded to
composition rules over a DAG that already exists, and exact arithmetic during preparation.

## More information

- Solver scope packet [I10 and I11](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/plans/22-solver-scope-execution.md#improvements-over-the-target-design) and packets C4 and C5, which own progress.
- [Change-tier review](../design_review/reviews/design_review_solver-scope-decisions_2026-09-28.md#adr-0121), finding F05.
- Architecture companion [§8](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/22-solver-capabilities-architecture.md#8-coefficient-and-conic-extensions); capability review L-C5–L-C7 and [§8.4](../design_review/reviews/design_review_solver-capabilities_2026-09-27.md#84-coefficient-and-conic-highs-clarabel).
- Related: ADR-0102, ADR-0105, ADR-0106, ADR-0108, ADR-0109.

## Status history

- 2026-09-28 — proposed and accepted under the maintainer's authorization of Plan 22's full scope, and of the solver scope packet's improvements (2026-09-28), after the [change-tier review](../design_review/reviews/design_review_solver-scope-decisions_2026-09-28.md#adr-0121) returned Accept (author review, Proposed evidence level). Finding F05 was corrected in this record before acceptance.
