# Design review: Plan 22 solver-scope decisions (ADR-0118–ADR-0121)

## 1. Scope, drivers and coverage

| Field | Content |
|---|---|
| Subject and boundary | Four decision records written before the dependent packets of Plan 22's remaining solver scope start: [ADR-0118](../../adr/0118-one-kkt-point-analysis.md) (one KKT-point analysis; supersedes ADR-0107), [ADR-0119](../../adr/0119-fixture-analysis-selections.md) (analysis selections in kernel fixtures), [ADR-0120](../../adr/0120-provider-envelope-contract.md) (provider envelopes) and [ADR-0121](../../adr/0121-convexity-compiler-facts.md) (convexity and cones as compiler facts). Their content comes from the [solver scope packet](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/plans/22-solver-scope-execution.md#improvements-over-the-target-design) (I1–I4, I6, I7, I10, I11, its rejected list and its decision table). **Neighbours read:** ADR-0103, ADR-0104, ADR-0105, ADR-0106, ADR-0107, ADR-0109, ADR-0110 and ADR-0111; blueprint §7.5, §9.4, §13.5, §13.6, §15.5, §18.6, §18.7, §18.9, §18.10, §19.2, §19.4, §19.8 and §25. |
| Standard | Core **3.1** (AP-01–AP-06, DP-01–DP-24, G1–G9); process-simulator profile **1.1** (PS-01–PS-13, PS-G1–PS-G3); binding `pse-arrow` ([standard.toml](../design_principles/standard.toml)). |
| Tier / purpose | **Change tier, conformance purpose** (the binding's default for a change within an approved plan). Each record is judged against the core and profile standard and against the accepted decisions it lies within or supersedes. |
| Reviewer / date | Agent T-D1 (Plan 22 W7), 2026-09-28. **Author review, not an independent review.** |
| Decisions | Behavioural adequacy **passes** after the corrections below; architectural fitness **passes**. Overall: **Accept** for each record. ADR-0118, ADR-0119 and ADR-0121 are accepted at the *Proposed* level. ADR-0120 is accepted at its tested level for the as-built contract, with the F04 corrections *Proposed*. See [slot 12](#decision). |
| Disposition owner | The [solver scope packet](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/plans/22-solver-scope-execution.md), through the packets named in slot 11. |

**Functional target.** A process simulator whose derived local quantities, analysis
selections, global relaxations and convexity claims each have one owner, a typed validity
statement and no hidden fallback. It follows the
[Plan 22 architecture](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/22-solver-capabilities-architecture.md#1-target-drivers-and-scenarios);
these records add no capability beyond the plan.

**Inspected.**
- **ADR-0118:** the second-order KKT assembly (`pse-backend-native` `conditioning.rs`, `solve.rs` `Evidence::second_order`).
- **ADR-0119:** the runtime mode and event inputs (`workflow/modeling/dynamics_events.rs`); `ModelingFixturePolicy` (`workflow/modeling/conformance.rs`); `IdasSettings.constraints` and `InputChange` (`pse-backend-native` `dynamics.rs`).
- **ADR-0120:** the kernel envelope contract and its test (`pse-kernels` `lib.rs`, `envelope_tests.rs`); the runtime collection of provider envelopes (`pse-runtime` `math/solves.rs`); the factorable projection's use of envelopes (`pse-math` `factorable.rs`, `factorable_tests.rs`); the G4 commit `7d0f8bf1`.
- **ADR-0121:** the run-time convexity decision (`math/solves.rs`, `pse-math` `convexity.rs`).
- **Libraries:** the pinned sources of `pounce-sens-core` 0.12.0, FERAL 0.18.0 and Clarabel 0.11.1 in the registry cache; `Cargo.lock` and the registry cache, searched for `pounce-sensitivity`.

**Not examined.** No code was built or run for this review. Implementation behaviour is left
to the packets' tests. The statistical adequacy of profile-likelihood intervals beyond the
packet's tests was not examined.

## 4. Change scenarios

Scenario definitions that already exist are linked rather than restated.

| ID | Scenario and conditions | Expected response and change boundary | Records |
|---|---|---|---|
| <a id="s01"></a>S01 | Covariance, intervals and propagated uncertainty after a steady fit ([architecture S13](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/22-solver-capabilities-architecture.md#s13), [capability review S02](design_review_solver-capabilities_2026-09-27.md#s02)), on any NLP backend and with presolve on | One analysis at the qualified candidate; a validity row for every requested quantity, withheld or not; no route-specific code in fitting | ADR-0118 |
| <a id="s02"></a>S02 | Add a backend or a QP route whose results need sensitivities, conditional duals or an advanced step ([architecture S18](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/22-solver-capabilities-architecture.md#s18), [S14](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/22-solver-capabilities-architecture.md#s14)) | The backend supplies an original-coordinate candidate with multipliers; nothing in the analysis changes | ADR-0118 |
| <a id="s03"></a>S03 | Author a certified tangent-plane stability check and a PID start with scheduled inputs as fixtures ([architecture S12](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/22-solver-capabilities-architecture.md#s12), [capability review S09](design_review_solver-capabilities_2026-09-27.md#s09)) | Package data only; no runtime- or Python-only inputs; the check reads a bound the step already has | ADR-0119 |
| <a id="s04"></a>S04 | Certify a model whose provider output enters a nonlinear term | The provider declares an envelope; the export is `Relaxed`; bounds are sound because the envelope is enforced | ADR-0120 |
| <a id="s05"></a>S05 | A convex model with exponential-cone atoms, then a value rebind that makes a quadratic indefinite | Preparation proves the cone form, and routing sends the model to Clarabel automatically; after the rebind the fact changes and routing follows | ADR-0121 |

## Per-record assessment

### <a id="adr-0118"></a>ADR-0118 — One KKT-point analysis serves every NLP and QP route

**Drivers.**
- PS-12: validity is stated and a withheld quantity keeps its reason.
- PS-10: analysis only at a qualified candidate.
- AP-02 and DP-01: one analysis owner, and no route-specific KKT.
- DP-13 and DP-16: FERAL and `pounce-sens-core` own the numerics.
- PS-07: exact second derivatives from the model's programs.

**Scenarios.** S01 and S02. ADR-0107's two routes would have made S02 a new KKT assembly for
every backend, and served neither the SCIP re-solve nor the QP routes. The selected design
makes S02 a matter of supplying original-coordinate multipliers.

**Foundations.**
- **AP-01 (satisfied).** The assembly, which depends on the model, is separated from the algebra, which is the library's.
- **AP-02 (satisfied).** Consumers see `KktPoint` and the `runtime.*` relations, never a backend or FERAL type. The factor never enters `SolveReport`.
- **AP-03 (satisfied).** `Pinned` composes the SCIP re-solve, parameter pins and profile pins from one transformation.
- **AP-04 (satisfied).** There is one validity record and one `Commitment`.
- **AP-05 (satisfied).** Withheld reasons, approximations and interval methods are registry enums.
- **AP-06 (satisfied).** The assembly and backsolve are testable against a dense reference without a solver (S0 tests).

**Gates.**
- **G3 and PS-G3:** pass after [F01](#f01).
- **G7 and PS-12:** pass after [F02](#f02).
- **G8:** pass. The bespoke assembly is justified by the §F note in the record. `pounce-sensitivity` was confirmed absent from both the registry cache and `Cargo.lock`, and `pounce-sens-core`'s `SensBacksolver` is generic in its dimension, so an active-set layout fits.
- **G9:** pass.

**Library fit (interface-checked).**
- `pounce-sens-core` 0.12.0 exports `SensBacksolver` (`dim`, `solve`, with optional natural-units and bound-row hooks), `SensApplication::parametric_step`, `compute_reduced_hessian_eigen` and `IndexSchurData`.
- FERAL 0.18.0 exports `Solver::solve_refined` and `Solver::inertia`.

**Verdict.** Accept at the Proposed level, with F01 and F02 corrected in the record.

### <a id="adr-0119"></a>ADR-0119 — Kernel fixtures declare analysis selections

**Drivers.**
- PS-11: analysis modes are declared over one model, including event handling and integration policy.
- AP-04: each selection has one authority.
- AP-05 and DP-02: selections are typed structure.
- PS-12: a check states its basis.

**Scenario.** S03. Today a stability check cannot select `certify`. Schedules switch
sensitivities off, and events and modes can be supplied only by programs.

**Foundations.**
- **AP-04 (satisfied after [F03](#f03)).** The intent conflict is refused rather than resolved by precedence.
- **AP-05 (satisfied).** Intent, schedules, events, modes and check basis become typed fixture declarations. The registry enum `EventDirection` replaces `Crossing`.
- **AP-01 (satisfied).** A check stays free of effects. The rejected alternative, a check that starts a solve, would have moved analysis selection into a check.
- **AP-02, AP-03 and AP-06: not affected.** The integrator and backend contracts of ADR-0110 are unchanged.

**Gates.**
- **G1:** pass after F03.
- **G4:** pass; no hidden solve.
- **G7 and PS-12:** pass. A `point` check result never states a global property, and a `global_bound` result is scoped to the step's box and tolerances.
- **G9:** pass.

**Verdict.** Accept at the Proposed level, with F03 corrected in the record.

### <a id="adr-0120"></a>ADR-0120 — The provider envelope contract

**Drivers.**
- PS-02 and DP-03: an envelope is a validity claim that needs an enforcement point.
- PS-10 and ADR-0105 item 2: relaxation soundness.
- DP-08: provider-to-auxiliary is a named relaxation.
- AP-02: the kernel host owns the contract.

**Scenario.** S04.

**As built** (G4, `7d0f8bf1`):
- the declaration defaults to none;
- `Registration::envelope` checks arity, NaN and order;
- the runtime collects checked envelopes only for a factorable route;
- the projection bounds an unconditional call's auxiliaries and marks dependent rows `Relaxed`;
- every interval's bits enter the program key and so the preparation identity.

No production factory declares an envelope.

**Foundations.**
- **AP-02 (satisfied).** The projection consumes checked intervals, never provider internals.
- **AP-04 (satisfied after [F04](#f04)).** The envelope's identity moves to the host.
- **AP-05 (satisfied after F04).** The promise gets an enforcement point.
- **AP-01, AP-03 and AP-06: not affected**, beyond the local tests F04 adds.

**Gates.**
- **G3, PS-G1 and G7:** these failed as built, because the soundness-critical promise was only documentation and an empty interval passed the check. They pass after F04 at the decision level.
- **G6:** pass. The relaxation is declared and never claimed exact.
- **G9:** pass.

**Verdict.** Accept. Items 1–3 are Tested, item 4 is Implemented, and the F04 corrections
(items 5–7) are Proposed. Implementation is owned as in slot 11.

### <a id="adr-0121"></a>ADR-0121 — Convexity and cone recognition are compiler facts

**Drivers.**
- PS-09: selection follows class and capability.
- AP-04 and DP-01: one convexity authority.
- DP-15: one version per family.
- DP-13 and DP-16: bespoke code only where no library fits.
- PS-10: verified certificates.

**Scenario.** S05. Today convexity is a run-time side computation that routing reads through
`Requirements.convex`, and cones are never recognized.

**Foundations.**
- **AP-04 (satisfied after [F05](#f05)).** `ProblemFacts.convexity` is the one exact authority, with a value identity. Numerical evidence stays a per-request policy input.
- **AP-02 (satisfied).** Routing reads facts and capability records. SCIP's curvature analysis stays a test oracle, so the compiler does not depend on a backend.
- **AP-05 (satisfied).** Automatic ownership becomes a declared per-class list instead of a side effect of a rank.
- **AP-06 (satisfied).** The curvature pass and exact LDLᵀ are pure functions over the factorable DAG and the matrix.
- **AP-01 and AP-03: not affected.**

**Gates.**
- **G1 and G2:** pass after F05.
- **G7:** pass. An almost-infeasible status is never certified, and certificates are verified in original coordinates.
- **G8:** pass. The §F reasoning is in the record. The Clarabel 0.11.1 manifest confirms `faer-sparse` → faer 0.21.9 beside the pinned 0.24.4, and `pardiso-mkl` → `pardiso-wrapper/mkl`.
- **G9:** pass.

**Governed sections.** The requested owner, §18.9, alone does not own the contract. The
review found the owners to be §7.5 (`ProblemFacts`), §18.6 (certificates), §18.7 (selection),
§18.9 (the published records) and §18.10 (the HiGHS and Clarabel adapters). The record governs
all five.

**Verdict.** Accept at the Proposed level, with F05 corrected in the record.

## 7. Findings

| ID | Finding | Principles / gate / scenario | Evidence or gap | Consequence | Correction | Verification |
|---|---|---|---|---|---|---|
| <a id="f01"></a>F01 | The draft removed ADR-0107's presolve restriction unconditionally, although the analysis needs a correct multiplier for every original row | PS-10, PS-12; G3, PS-G3; S01 | Blueprint §25 records that pinned bound tightening under automatic presolve can return a multiplier that fails complementarity against the original bound. The packet lists multiplier recovery only as an S1 packet-start check | A Lagrangian Hessian or an activity classification built from a wrong multiplier gives a sensitivity or reduced Hessian that is reported as valid | ADR-0118 items 2 and 7: the analysis runs only at a candidate whose original-coordinate KKT evidence is `Stationary` or better, with every original row's multiplier. Otherwise the quantity is withheld with a typed reason, and presolve is never switched off as a fallback | `sensitivity_survives_presolve` should include a fixture whose recovered multiplier fails complementarity, and assert that the quantity is withheld |
| <a id="f02"></a>F02 | The statistical model behind covariance left the loss's importance weights undefined | PS-12; G7; S01 | The loss in §19.4 is `0.5·Σ importance·((prediction − observation)/σ)²`. ADR-0107 Outcome 4 names weighted least squares with declared σ but not importance, and the S3 packet-start check leaves the question open | With importance ≠ 1, the loss Hessian is not the estimator's information matrix, so a covariance and interval computed from it would be wrong, yet reported as valid | ADR-0118 item 2: the declared model requires unit importance for every included observation. Otherwise covariance is withheld with a typed reason. This answers the S3 packet-start check; admitting non-unit weights (a sandwich estimator) would need a new decision | An S3 test beside `covariance_withheld_without_declared_sigma` in which a non-unit importance withholds covariance |
| <a id="f03"></a>F03 | Two sources for IDAS sign constraints | DP-01, PS-12; G1; S03 | `IdasSettings.constraints: Vec<StateSign>` is "one declared sign per state in state order" (`pse-backend-native` `dynamics.rs`), and the packet adds constraints derived from authored bounds without deleting the profile source | An authored bound and a profile vector can disagree with no stated precedence. A state-order vector also exposes solver indices to authors | ADR-0119 item 4: the authored annotations are the only source. The native vector is derived, and is no longer a runtime or Python setting | `idas_sign_constraints_from_authored_bounds`, and the settings surface no longer accepts a sign vector |
| <a id="f04"></a>F04 | The provider-envelope promise that relaxation soundness depends on has no enforcement point, and the interval check admits empty intervals | DP-03, PS-02, PS-10; G3, PS-G1, G7; S04 | `Registration::envelope` checks arity, NaN and `lower > upper` only, so (+∞, +∞) and (−∞, −∞) pass. `worker_scoped` returns the factory's worker unchecked. Including the envelope in `configuration_key` is a trait-documentation obligation that nothing checks | A factory whose outputs leave the declared envelope, or which declares an empty one, turns a `Relaxed` export into something that is not a relaxation. A `global_bound` or `proven_infeasible` would then be false. The risk is latent, since no production factory declares an envelope | ADR-0120 items 5–7, in `pse-kernels` only: each interval contains a real number; the host checks every successful value evaluation against the envelope, with a violation a typed `Contract` error; `Registration::configuration_key` frames the checked envelope | Extend `provider_envelope_is_checked_against_the_contract` with the two infinite intervals, and add a worker returning an out-of-envelope value that fails as `Contract` |
| <a id="f05"></a>F05 | The draft deleted `Requirements.convex` without saying how an explicit `ConvexityPolicy::Numerical` still reaches routing | DP-01, PS-09; G2; S05 | Blueprint §18.10 admits HiGHS convex QP on numerical PSD evidence under the explicit policy. The draft said only that the policy "never becomes a fact" | Two readings: either the explicit policy silently stops working, or numerical verdicts reach routing through a side channel that bypasses the fact | ADR-0121 item 4: routing reads the fact. Under the explicit numerical policy, that request's qualification may add `convex_quadratic` for that request only. It is recorded as numerical evidence, never stored, rebound or reused | `numerical_psd_only_under_explicit_policy` |

**Strengths that affect the argument.**
- ADR-0118 deletes a planned mechanism (S2) and a planned library route before either exists.
- ADR-0119 deletes three programmatic inputs.
- ADR-0121 removes a per-run computation from the run path.

Each record reduces the number of places that state one meaning.

## 11. Disposition

The solver scope packet owns the status of every finding. Proposed owners:

| Finding | Scenario | Disposition | Work owner | Evidence or trigger |
|---|---|---|---|---|
| F01 | S01 | Resolved in ADR-0118 (decision); implementation open | S1 | The verification row above |
| F02 | S01 | Resolved in ADR-0118 (decision); implementation open | S3 | The verification row above |
| F03 | S03 | Resolved in ADR-0119 (decision); implementation open | Y0d | `idas_sign_constraints_from_authored_bounds` |
| F04 | S04 | Resolved in ADR-0120 (decision); implementation open. The packet names no step for items 5–7 yet | The G6 kernel gaps, in `pse-kernels` (proposed: T-L with the G6r kernel) | The verification row above |
| F05 | S05 | Resolved in ADR-0121 (decision); implementation open | C5 | `numerical_psd_only_under_explicit_policy` |

No authority text blocks these records. ADR-0107 is superseded through `just adr-supersede`,
and the blueprint carries revision 69 with decision markers at each governed section.

## <a id="decision"></a>12. Decision

**Behavioural and semantic adequacy: pass**, with F01–F05 corrected in the records before
acceptance. **Architectural fitness (G9): pass.** Every affected foundation is satisfied in
the scenarios above.

**Overall: Accept** ADR-0118, ADR-0119, ADR-0120 and ADR-0121.

| Record | Accepted at | Findings corrected |
|---|---|---|
| ADR-0118 | Proposed | F01, F02 |
| ADR-0119 | Proposed | F03 |
| ADR-0120 | Tested (items 1–3), Implemented (item 4), Proposed (items 5–7) | F04 |
| ADR-0121 | Proposed | F05 |

**Strongest evidence.**
- The source reads of the as-built envelope contract and the convexity path.
- The interface checks of `pounce-sens-core`, FERAL and Clarabel at their pinned versions.
- The confirmed absence of `pounce-sensitivity`.

**Main uncertainty.**
- Whether the active-set analysis agrees with `ipopt_sens` within a tolerance that covers the barrier term (ADR-0118's revisit trigger).
- The preparation cost of exact certificates (the C5 census).

Acceptance does not implement or qualify any of this. The packets' tests settle it.
