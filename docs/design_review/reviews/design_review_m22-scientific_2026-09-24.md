---
title: M22 scientific design and qualification review
date: 2026-09-24
reviewer: independent-scientific-review-agent
scope: Plan 14 M22 scientific contracts and acceptance witnesses
decision: Accept
evidence: Tested; authenticated local functional receipts and native witnesses; Measured case workload evidence inspected
---

# M22 scientific design and qualification review

## 1. Scope, purpose and coverage

| Item | Assessment |
|---|---|
| Subject | Current Q01–Q05, Q07–Q11 and Q14–Q15 scientific contracts, native adapters and exact acceptance witnesses; the M22 execution packet and proposed ADR-0087 |
| Standard | Core 2.0, DP-01–DP-24; process-simulator profile 1.0, PS-01–PS-13; pse-arrow binding |
| Tier · purpose | Design · conformance to the approved Plan 14 target |
| Reviewer | Independent scientific review agent; no implementation edits authored |
| Source observation | Working tree based on `8e9c3ce8264010e6cc2824161e7abb1b791d98c6`; scientific source and retained execution checked against `m22-functional-cost-input` and `m22-costs`; final document reconciliation and review digest binding follow separately |
| Decision | **Accept G2, G3 and PS-G1–PS-G3 for the declared local Linux design-stage scope. Open mandatory findings: zero.** |

The supported analysis modes are square simulation, continuous optimization, eligible
linear/quadratic/conic execution, fixed-mass ODE/index-1 dynamics and local parameter
fitting. The review covers physical meaning, structural admission, derivatives,
native outcomes, original-space recovery, conservation and the assertions that
distinguish successful execution from failure.

The inspected baseline already delegates numerical methods to libraries. The material
M22 change is actual execution evidence and stronger assertions, including a corrected
KINSOL CSC refill and named dynamic structural refusals. This reviewer inspected the
source directly; implementation-agent reports were used only to locate evidence.

Excluded from this review: publication crash consistency, runtime resource ownership,
library-wide capability claims and end-to-end performance verdicts assigned to the
other independent reviewers. Release, coverage, exhaustive feature/platform campaigns
and the known upstream `proc-macro-error2` warning are outside the maintainer's local
design-stage boundary. No scientific requirement is waived by those exclusions.
No builds, tests, benchmark or reference regeneration were run by this reviewer.

## 2. Authority and identity

| Fact | Authority and revision boundary | Derived form |
|---|---|---|
| Physical quantities, components, reference states and model equations | Authored declarations, admitted physical inventory and immutable model revision | Typed bodies, case layouts and native coordinates |
| Property interpretation | `FeosPorts`, provider specification, coefficient data and declared envelope | Attempt-owned FeOS/num-dual workers and one-trial cached outputs |
| Solver policy and numerical tolerances | Explicit solver/initialization/simulation/fitting profile | Native options, callback arrays and post-solve observations |
| Balance meaning | Declared signed contributions and accumulation/impulse contract | Compiled balance rows and independent physical closure observations |
| Numerical outcome | Native report plus original-space validation | Physical result relations retaining native termination, assurance and diagnostic state |

### Physical semantics

| Quantity | Unit and basis | Reference/convention | Envelope and authority |
|---|---|---|---|
| Provider temperature/density | K and mol/m³ | Absolute SI state ports; point quantities | Positive finite state and declared temperature/density window |
| Composition | Methane/ethane mole fractions; propane complement | Ordered three-component molar system, strict interior | All three fractions checked against the declared envelope |
| Pressure | Pa | Absolute, unshifted provider port | Checked even when pressure is not a requested output |
| Enthalpy/entropy | J/mol and J/(mol·K) | Explicit custom 298.15 K caloric integration convention, no formation enthalpy | Physical registry and fixed PC-SAFT/DIPPR coefficient identity |
| Vessel inventory/energy | Amount and energy in the authored units | Signed inflow/outflow/heat/work and declared event impulses | Independent differential/integral closure with declared tolerances |

`crates/pse-kernels/src/feos.rs:95` rejects conflicting caloric conventions;
its port admission checks molar and mole-fraction basis. The envelope is an enforced
operating window, **not empirical accuracy certification**. Explicit-density evaluation
is homogeneous-state evaluation. NPT initialization chooses a density starting branch;
it does not guarantee phase selection. Stability is an explicit optional calculation.

## 3. Contracts and invariants

| Contract | Enforcement and refusal | Evidence |
|---|---|---|
| Physical meaning precedes algebraic normalization | Typed quantity/basis/reference inference; incompatible operations and conversions reject | `typed_tests`, `typed_math::tests`, Q01 |
| Erased or inactive algebra cannot bypass domain obligations | Guarded schedule captures obligations before CAS; derivative order has separate admission | `guarded_tests`, Q02; `derivative_tests`, Q03 |
| Structural matching is not numerical rank | Complete semantic incidence and class-specific admission; native numerical failure remains distinct | `incidence_tests`, `structural::admit`; staged singular control |
| Recoverable trials do not become terminal latches | Typed callback classification; only fatal/cancel/panic outcomes latch | `callback.rs`; actual both-backend controls in `native_outcomes.rs` |
| Solver success does not substitute for original-space quality | Fresh residual/bound/domain observation and independent balance closure | `quality.rs:124`, `workflow/balances.rs:236`, `workflow/balances.rs:271` |
| Failed initialization cannot commit its candidate | Native success, finite primal, identity and physical-quality conditions at block commit | `math/initialization.rs:430` |
| Local sensitivity rank is not parameter certainty | Response Jacobian, weighted/scaled SVD and explicit cutoff; unavailable response diagnostics remain distinct | `workflow/fitting/oracle.rs:799`; rank-deficient physical fit |

Variables retain explicit fixed/free roles. Roots require complete square equality
structure; NLP admission retains legitimate optimization freedom. Dynamic preparation
analyzes the mass-zero algebraic partition before integration. After the review repair,
it returns the existing `StructuralAnalysis` and uses `structural::admit(Roots)`, preserving
deficient row/state identities rather than reducing failure to a generic string.

Equivalence is real-algebra equivalence on the admitted domain, with declared numerical
tolerances, not bitwise equivalence to a retired evaluator. Missing, failed, limited,
cancelled, feasible-only and locally stationary results retain different meanings.

## 4. Derivation and numerical execution

| Stage | Mechanism and retained meaning | Guards / derivative source | Scaling, outcome and validation |
|---|---|---|---|
| Typed preparation | Authored definitions to guarded Symbolica/Numerica bodies; explicit aliases and row contributions | Authored domain obligations; exact arithmetic/provider first and second derivatives | Value changes bind separately from structural preparation; analytic derivatives and alias/repeated-row controls |
| Property evaluation | FeOS PC-SAFT/DIPPR with num-dual and explicit composition coordinates | Input/output-envelope checks; requested raw partial order; no finite-difference production fallback | Fresh finite complete outputs; failures clear the one-trial cache |
| Root initialization | Existing block plan to native KINSOL with KLU/dense/SPGMR ownership | Root smoothness and square admission; exact Jacobian mapping | Residual scales from declared tolerances; only validated successful block results commit |
| NLP | Shared original oracle through Ipopt and POUNCE | Exact or explicitly selected limited-memory Hessian; typed recoverable/fatal callbacks | Native termination and assurance remain separate from original physical quality |
| Presolve/postsolve | Qualified pounce-presolve wrappers and original-space recovery | Explicit tape/affine eligibility; incompatible warm starts reject | Both native backends tested by authored controls with Off/Auto, nontrivial scaling, eliminated coordinates and original dual stationarity |
| Coefficients/cones | Eligible HiGHS coefficient representation or explicit Clarabel cone data | Current convexity certificate and class refusal | Analytic optima, native-optimal assurance, original row quality and unsupported-class negatives |
| Dynamics/fitting | Diffsol fixed-mass integration and native fitting over the same definitions | Index-1 structural partition, event/reset policy, smooth forward derivatives; hybrid sensitivities refused | Typed completed/partial outcomes; independent conservation and local-rank observations |

KINSOL's sparse callback now copies CSC column pointers and row indices as well as
values (`crates/pse-backend-native/src/kinsol.rs:317`). The arrays are derived from the
admitted canonical sparsity and sized with the allocated matrix. This restores the
structure cleared by native matrix zeroing. It is a boundary repair, not a second
factorization or iteration implementation.

## 5. Scientific journeys examined

- **Physical solve/optimization:** heater/recycle and flash compare physical values to
  analytic or frozen independent references. Heater optimization now requires
  locally stationary native assurance and the expected temperature, rather than
  accepting feasible-only status as optimization success.
- **Failure and retry inside a solve:** the actual native quadratic changes its trial
  point before injected recovery, fatal failure, panic or cancellation. Both NLP
  backends and both Hessian modes exercise the analytic optimum; iteration limits
  retain no success assurance.
- **Presolve and warm start:** native solves recover `(2, 2)`, objective 8, original
  equality multiplier −4 and original-space stationarity. Auto eliminates a variable;
  Off does not. Compatible starts execute; incompatible identity is rejected.
- **Initialization:** coupled equations start from `(1000, −100)` and reach the analytic
  root through two declared stages. A full-matching rank-deficient inconsistent block
  cannot commit. The old physical-type-error surrogate has been removed.
- **Dynamic start/event:** vessel samples preserve inventory and energy; consistent
  algebraic initialization, scheduled input changes and valve mode/reset behavior are
  covered. A higher-index declaration is rejected before native start with model IDs.
- **Estimation:** steady, transient and mixed observations recover the known heat
  parameter; zero inflow makes a second inlet-enthalpy parameter unidentifiable while
  an optimum remains possible. The result reports rank one for two parameters.

## 6. Gates

Verdicts below cover the inspected scientific design and authenticated local execution.
Whole-M22 closure also requires the separately assigned gates and final review binding.

| Gate | Verdict | Evidence or scope reason | Required action |
|---|---|---|---|
| G1 Authority | n.a. | Whole-system authority review assigned separately; no competing scientific authority found | Separate architecture verdict |
| G2 Semantic fidelity | pass | Explicit physical, status, derivative, rank and coordinate contracts preserve distinctions; authenticated native and Python witnesses pass | None in scientific scope |
| G3 Validity | pass | Typed admission, original-space checks and named dynamic deficiency; repaired refusal witness passed in the 127-test native selection | None in scientific scope |
| G4 Hidden behaviour | n.a. | System effect review assigned separately | Separate architecture verdict |
| G5 Consistency and recovery | n.a. | Publication/resource lifecycle beyond this scientific review | Separate architecture verdict |
| G6 Transformation and reuse | n.a. | Full reuse/performance review assigned separately; mathematical recovery inspected here | Separate architecture verdict |
| G7 Truthful capability claims | n.a. | Whole supported-capability review assigned separately | Separate capability verdict |
| G8 Library leverage | n.a. | Full bespoke-capability inventory assigned separately; scientific adapters remain library-owned | Separate library verdict |
| PS-G1 Physical consistency | pass | Physical admission, property envelope, independent reference values and balance closure pass; empirical envelope certification remains unclaimed | None in scientific scope |
| PS-G2 Well-posedness | pass | Complete class-aware structural admission, numerical singularity distinction and named dynamic refusal are exercised | None in scientific scope |
| PS-G3 Numerical integrity | pass | Actual native outcome controls, analytic derivatives, presolve/original-space recovery, dynamic/fitting checks and declared perturbed-start workload pass | None in scientific scope |

## 7. Findings and principle verdicts

**Open mandatory scientific findings: zero.** Baseline: zero.

| ID | Finding | Principles · gate | Evidence | Consequence | Correction | Verification |
|---|---|---|---|---|---|---|
| F01 — resolved and Tested | Dynamic structural refusal discarded deficient row/state identities | DP-21, PS-04 · G3, PS-G2 | The former boolean result is replaced by `Arc<StructuralAnalysis>` at `workspace.rs:466`; `math/functions.rs:57` calls existing class admission | An engineer can now locate the rejected algebraic equation/state rather than receiving only an index-1 string | Implemented by execution owner; reviewed directly; no parallel diagnostic authority | `higher_index_vessel_is_refused_before_native_start` checks the closure-row identity and passed in the authenticated `m22-functional-ready` native origin |
| O01 — evidence boundary resolved for this review | Functional execution and declared case measurements are available | DP-22, DP-23 | `m22-functional-cost-input/checks.json`: all 40 gates and Q01–Q17 pass; `m22-costs/checks.json`: measurement passes, reviews not yet run | Scientific acceptance has executed support; this document alone is not whole-plan closure | Preserve retained origins and finish independent review binding | Native/Python artifacts and all 232 measurement artifacts authenticated; no digest mismatch |
| O02 — disclosed local convergence limit | Flash at 1.15 times both reference density guesses did not converge to a feasible state; the final workload uses 1.02 | PS-08, PS-10, PS-13 · PS-G3 | Failed process log and public probe retain `Infeasible_Problem_Detected`, assurance `None`, infeasible candidate and rejected closure about −0.0784; final 1.02 campaign passes | Local convergence and cost for the declared start are supported; arbitrary-start robustness and global infeasibility are not | Exact multiplier, unchanged remaining coordinates, post-failure selection and unchanged references/tolerances disclosed in the M22 packet | Fresh 23-case campaign binds the 1.02 benchmark source; no failed or earlier sample is relabeled |

| Applicable principle | Verdict and reason |
|---|---|
| DP-02 | Satisfied: physical/status/rank distinctions remain typed or checked at boundaries |
| DP-03 | Satisfied: physical, structural and numerical assumptions have observable rejection paths |
| DP-07 | Satisfied in scope: equation incidence, flow graph and initialization order remain separate |
| DP-08 | Satisfied in scope: guards survive normalization and original-space postsolve is explicit |
| DP-11 | Satisfied in scope: derivative order, scaling, numeric tolerances and rank cutoff are explicit |
| DP-15 | Satisfied in inspected scope: selected native representations and unavailable classes reject explicitly |
| DP-21 | Satisfied after F01 correction: scientific structural failure retains model identities |
| DP-22 | Satisfied for this review: implementation, authenticated retained tests, case measurements and convergence limits are distinguished |
| DP-23 | Satisfied in scope: independent analytic/reference and negative controls address the observed risks and are executed in authenticated local profiles |
| DP-24 | Satisfied in scope: generated declaration boundaries reject retired spellings rather than silently reinterpret them |
| PS-01, PS-02, PS-03 | Satisfied: explicit physical semantics, enforced envelope and independently observed conservation |
| PS-04, PS-05 | Satisfied after F01: complete class-aware matching and distinct topology/solve structures |
| PS-06, PS-07 | Satisfied: guarded admitted domain, library derivative chain and explicit scaling |
| PS-08, PS-09, PS-10 | Satisfied: declared initialization/native strategy, class selection and truthful outcomes with independent quality |
| PS-11 | Satisfied in the inspected science paths: shared definitions serve dynamic/steady/fitting modes; full cache correctness reviewed separately |
| PS-12, PS-13 | Satisfied in scope: physical observations and local rank are honestly scoped; shared checks and independent references exist |

Other core principles are outside the assigned source-review depth; this table is not
a substitute for the architecture/runtime and capability/library reviews.

## 8. Library-leverage ledger

| Capability | Existing mechanism | Fit and boundary | Recommendation |
|---|---|---|---|
| Arithmetic/derivatives | Symbolica/Numerica, FeOS/num-dual | Typed guarded composition and raw-partial adapters own domain meaning | Retain; do not add a custom evaluator or finite-difference fallback |
| Structural decomposition | pounce-presolve matching/DM/BTF plus existing semantic mapping | Complete inventory and class admission; existing result now also supplies dynamic diagnostics | Retain the shared result, not a second diagnostic/matching implementation |
| Native iteration and factors | Ipopt, POUNCE, KINSOL, HiGHS, Clarabel, Diffsol | Explicit representation and profile contracts | Keep boundary corrections local; no replacement solver loops warranted |
| Local sensitivity/rank | faer LU/SVD | Bounded local analysis with cutoff and backward-error checks | Retain; do not present local numerical rank as statistical certainty |

## 9. Alternatives

| Alternative | Meaning and machinery | Risk | Decision |
|---|---|---|---|
| Earlier unit-only acceptance | Shared library design, but fabricated native finalizations and weak status assertions | Could accept an integration failure or feasible non-optimum | Superseded by actual native controls |
| Current implementation | Same domain authority and libraries, actual executions and independent checks | Declared local/profile limits remain; final whole-plan review binding is separate | Selected |
| Library-owned correction | Refill native CSC arrays and reuse existing structural analysis/error type | Preserves one structural authority and native solver ownership | Selected; also the simplest viable correction |
| Add a custom numerical/diagnostic subsystem | Duplicated structure/solver policy | New semantics and verification burden without an unmet capability | Rejected |

## 10. Verification evidence

| Claim | Label | Command / conditions | Inspected result and boundary |
|---|---|---|---|
| Selected native controls execute | Tested | `just plan14-native --profile ci --success-output final --config-file build/plan14/m22-functional-ready/plan14-native-nextest.toml`; `native-force-validate`, `rust-native`, pinned local libraries and one nested native thread | Retained origin `build/plan14/m22-functional-ready`: 127 passed; zero failures, errors, skipped or not-run selected tests; baseline 0. All four origin artifacts match their receipt digests |
| Public Python journeys execute | Tested | `just plan14-python build/plan14/m22-functional-tested`; linked native extension, `python-native` | Retained origin: 4 passed; zero failures, errors, skipped or not-run selected tests; baseline 0. All four origin artifacts match their receipt digests |
| Final local functional scope | Tested | `just architecture-acceptance build/plan14/m22-functional-cost-input --phase functional`, with explicit continuation/affected-gate arguments retained in the receipt | `complete: true`, `source_unchanged: true`, no provenance errors; all 40 declared checks and Q01–Q17 pass. Native, unit/component/integration Python and tooling origins remain distinguished |
| Case performance and memory | Measured | `just architecture-acceptance build/plan14/m22-costs --phase performance --functional-from build/plan14/m22-functional-cost-input --stop-after plan14-measure`; collector command `just plan14-measure build/plan14/m22-costs`; cached Cargo dev profile, native force validation | All 23 workload processes pass, each with ten Criterion samples and native nested threads one; all 232 recorded artifacts match. `compilation_timed: false`; measurement source digest `43a97300e00d83aa7e456f9b5a05fc65a4aaac2455833afe6e303892f3589aaf`. Performance receipt correctly remains incomplete until independent reviews run |
| F01 named structural refusal | Tested | `acceptance::vessel_fit::higher_index_vessel_is_refused_before_native_start` in the selected native command above | JUnit records its pass; current source checks structural deficiency and the named closure-row identity |

The reviewer directly parsed the receipts and JUnit, verified retained artifact hashes,
and matched inspected scientific source and benchmark source against receipt identities.
Retained executions are not relabeled as fresh final-byte runs. The packet records the
authorized impact review of intervening mechanical edits and affected-gate reruns.
Filtered tests outside a selected scope are not passing witnesses; an omitted or skipped
required acceptance test cannot qualify.
Independent thermodynamic references are frozen teqp PC-SAFT and Decimal caloric
integrals under the same coefficient/convention inputs. No reference was regenerated
or altered by this reviewer. No numerical IDAES-equivalence claim is made.

The flash workload's 1.02 density multiplier is a documented selection after the 1.15
failure, with all other starting coordinates at reference and all references and
tolerances unchanged. The independent public probe reaches acceptable local stationarity
in 15 iterations, with feasible output and balance closures 0 and −2.7756e−17; the fresh
benchmark also passes its physical checks. This is a fixed perturbed-start observation,
not a measured basin of convergence. The failed run remains at
`build/plan14/m22-measurements/process-cost/flash-difficult-1/process.log`.
The [M22 packet](../../plans/14-m22-execution.md#verification) supplies the complete
workload table and conditions. Pool reservations and process RSS remain separate;
no general speedup, arbitrary-start robustness or release-performance claim is made.

## 11. Authority and exceptions

Followed the binding's K1 resolution: Plan 14 D02–D04 and ADR-0082–0084 supply the target
scientific ownership contract while formal decision/blueprint reconciliation proceeds.
This review does not edit or authorize itself to amend the blueprint or accepted records.
Proposed ADR-0087 records the user-directed local design-stage boundary; it does not waive
physical/numerical validity or turn incomplete Q01–Q18 evidence into acceptance.

No new scientific SHOULD exception is requested. General implicit/high-index DAE,
hybrid sensitivities, JIT/SIMD, global MINLP, empirical certification of the full property
window and unexecuted platform/release performance remain outside supported claims.

## 12. Decision

**Accept the scientific design and local qualification for G2, G3 and PS-G1–PS-G3.**
Open mandatory scientific findings: **zero**, against baseline zero. The named dynamic
diagnostic is corrected and tested. Native/Python witnesses, independent references,
negative controls and declared case measurements support the scoped scientific claims.
The disclosed flash limitation does not support a broader convergence claim.

| Priority | Remaining action | Findings | Acceptance evidence |
|---|---|---|---|
| Correctness | None in this scientific scope | F01 resolved; O01 evidence supplied | Authenticated 127-test native and four-journey Python origins, complete functional aggregate |
| Library ownership | Retain the existing analysis and solver owners | None open | Other independent reviewers' assigned gates |
| Final integration | Bind independent reviews after authority/status documentation settles | No scientific finding open | Final source-bound review records and whole-plan collector; owned by execution coordinator |

Final source-bound gate records follow the coordinator's stable digest; this review does
not claim those integration steps have already run. Reviews remain evidence, not authority.
