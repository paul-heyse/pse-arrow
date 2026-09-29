# Design review: native solver pipeline and unused solver capability

## 1. Scope, drivers and coverage

| Field | Content |
|---|---|
| Subject and boundary | The **native solver pipeline** and the library capability it leaves unused. The boundary is `pse-backend-native` (routing, the Ipopt, POUNCE, KINSOL, HiGHS, Clarabel, Diffsol and IDAS adapters, presolve, tears, recycle, implicit, quality, diagnostics). It also covers the representation that pipeline consumes from `pse-math` and `pse-compiler` (`ProblemFacts`, `presolve::Facts`, `PreparedBody` expressions, variable domains), and the runtime and Python solve-settings surface (`pse-runtime::math::solves`, `workflow::fitting`, `pse-py` settings). The supply side is the `native-solver-libraries` skill: 11 routes and 33 capabilities over Ipopt 3.14.20, POUNCE 0.12, KINSOL/IDA/IDAS (SUNDIALS 7.1.1), diffsol-nl, Diffsol 0.16.2, HiGHS, Clarabel 0.11.1, SCIP 10.0.2 (russcip 0.10.0 / scip-sys 0.1.28), FERAL 0.18 and MUMPS 5.9.1. The API index and corpus behind it were read as well. |
| Standard | Core **3.0** (AP-01–AP-06, DP-01–DP-24, G1–G9); process-simulator profile **1.1** (PS-01–PS-13, PS-G1–PS-G3); binding `pse-arrow` ([standard.toml](../design_principles/standard.toml)). |
| Tier / purpose | **Design tier, target purpose** (binding default). Authority text that blocks a best-in-class capability is recorded in slot 11 as a required change, not treated as a constraint. |
| Reviewer / date | Agent review in the maintainer's session, 2026-09-27. Four read-only `library-leverage-reviewer` passes (NLP and sparse linear algebra; roots and dynamics; coefficient and conic; SCIP) supplied slot-8 ledgers. The reviewer verified the defects it adopts against source. This is not an independent review of any design the same agent authored. |
| Decisions | Current implementation: behavioural adequacy **not adequate for three PS-G3 items** (F03, F04, F06); architectural fitness **G9 fail on narrow, owner-local items** (F01, F02, F09). Capability additions: **Accept-scoped at the Proposed level**, as ranked in slot 12. **SCIP: not beneficial for the classes the pipeline solves today (C); beneficial as a new certification and MINLP capability once its triggers fire (B).** Overall: **Revise-scoped**; see slot 12. |
| Disposition owner | [Plan 22](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/plans/22-solver-capabilities.md#finding-dispositions) (draft), created after this review at the maintainer's request. It owns the status of every finding here and in the addendum; scope items need the ADR routes in slot 11. |

**Question.** The maintainer asked whether solvers not yet integrated, SCIP in particular,
would benefit the current pipeline and the problem types it solves. They also asked what
other solver functionality could be put to beneficial use.

**Functional target (profile).** Square simulation, optimization, dynamics and parameter
estimation from one model, with truthful and physically scoped results. Also: edit→re-solve
without rebuilding structure, studies and sensitivities, recycles, dynamics, and extension by
a new solver. The IDAES 2.13 capability target (ADR-0097, proposed; Plan 20 A1–A10) adds
covariance and confidence intervals, NLP parametric sensitivity, dynamic optimization and
NMPC/MHE, multiperiod and price-taker MILP, DegeneracyHunter and complementarity.

**Analysis modes and workloads examined.** All four analysis modes; the solve, sweep,
recycle, dynamic-start and infeasible-problem journeys; and a representative library
replacement or upgrade.

**Not examined:**
- property-package physics and basis conventions (PS-01/PS-02), beyond the solver-side quality recomputation;
- the Delta/Arrow publication of solver results;
- wheel and distribution builds;
- runtime behaviour of any library not already linked.

**Evidence base.**
- **Source** at the seams listed in slot 2. The working tree has substantial uncommitted Plan 21 work; this review reads the tree as it stands on 2026-09-27.
- **The skill's evidence layers:**
  - routes and capabilities (`python3 scripts/reference.py show|compare|features|matrix`);
  - the rustdoc API index (26 crates);
  - the checksummed native and crate corpus;
  - executed receipts: HiGHS, Clarabel, POUNCE, FERAL, KINSOL and IDA fixtures on rustc 1.98.1. **SCIP, Ipopt 3.14.20, IDAS sensitivities and MUMPS are not runtime-qualified by the skill.**
- **Consumer pins** from `Cargo.toml` and `Cargo.lock`.
- **One executed probe, P1,** a factorability census over all 36 steady reference cases (slot 10).
- **Not built:** by maintainer choice, SCIP was not built. Every SCIP claim is at most *Interface-checked*; value and performance are *Proposed*.
- **Context7** was unavailable to the delegated passes. Upstream behaviour comes from the pinned release sources, which is stronger evidence for these exact versions.

**Version skew noted.** The workspace pins `highs-sys =1.14.3` (HiGHS 1.14.0), while the
skill's HiGHS runtime receipts and API index are for 1.15.0, so none of the skill's HiGHS
runtime evidence transfers. `pounce-sensitivity` and `pounce-hsl` 0.12.0 are absent from both
the skill corpus and the local registry, so their API is *Proposed*.

**Variation axes.**
- A new native backend or class (PSE-S02).
- A library upgrade within its contract (PSE-S06).
- A new analysis workflow over existing preparation (PSE-S03): covariance, sensitivity, certification.
- A new modeling concept: discrete domains and disjunctions.

## 2. Decomposition, ownership and dependencies

| Component / responsibility | Decision or invariant hidden | Contract consumed / exposed | Dependency direction and reason | State/effect owner | Local test setup |
|---|---|---|---|---|---|
| `pse-math::facts`, `presolve` | Class facts (domains, bound shapes, affine and degree-two proofs, derivative order); per-row FBBT tapes and completeness | `CasePlan` → `ProblemFacts`, `presolve::Facts` (pounce `FbbtTape`) | pse-math → pounce-nlp tape type only | Immutable, Salsa-tracked | Pure; unit tests in `presolve.rs` |
| `pse-math::execution::PreparedBody` | Symbolica atoms per output; `expression()` is `None` when a provider or branch prevents exact projection | Compiled evaluators behind `NlpOracle`/`NleOracle` | pse-math owns Symbolica; no solver type crosses | Immutable | Pure |
| `routing::Requirements` | Five-fact selection (requested, available, admitted, eligible, selected); every refusal reason; automatic order | `ProblemFacts`, `SolveIntent`, `Controls` → `Route` | Backend-neutral; one policy owner | None | Pure (`routing.rs` tests) |
| Algebraic adapters: `ipopt.rs`, `pounce.rs`+`tnlp.rs`, `kinsol.rs`, `highs.rs`, `conic.rs` | Native lifecycle, option reservation, status → `NativeTermination`, warm starts, reuse keys | `NlpOracle`/`NleOracle`/`CoefficientProblem`/`ConicProblem` → `SolveReport` | Each adapter depends only on its library; FFI confined to one file per library | Worker-local native handles (`!Send`) | Unit tests per adapter; native link required |
| `presolve::Pipeline` | pounce-presolve passes, certified infeasibility, back-map of duals and columns | `NlpOracle` + `Facts` → transformed TNLP | Consumes the pounce-presolve library | Per solve | `presolve/tests.rs` |
| `quality.rs` | Independent post-solve qualification in original units (`Unqualified` < `Feasible` < `Stationary`, `OptimalWithinTolerance`, `GapQualified`) | `SolveReport` → `Qualification` | Backend-neutral | None | Pure |
| `dynamics` (`integrator.rs`, `idas.rs`) | Method choice (Diffsol BDF or IDAS), events, resets, forward sensitivities, quadratures | Compiled dynamic functions → trajectories | Diffsol option types are exposed in `Profile` (F09) | Worker-local | `dynamics/tests.rs` |
| `tears.rs`, `jacobian_diagnostics.rs`, `highs/diagnostics.rs` | Internal MILP and LP formulations; IIS, rays, ranging, relaxation | Graph or Jacobian → HiGHS | Consumers of the HiGHS adapter | Per call | Unit tests |
| `implicit.rs` (`InnerSolver`) | Nested square solves during evaluation | `pse_math::implicit::InnerSolver` | Injected into pse-math, keeping the dependency direction pse-backend-native → pse-math | New KINSOL session per call | Unit tests |
| `pse-runtime::math::solves`, `workflow::fitting` | `SolverProfile`, `BackendSettings`, sequence reuse, fitting oracle, start receipts | Runtime ↔ backend | Runtime → backend | `MathService` jobs (§18.8) | Workflow tests |
| `pse-py` settings | Python `SolveSettings` → `SolverProfile` | Always `BackendSettings::Default` (`settings.rs:151`) | py → runtime | None | Python tests |

The composition root is `pse-runtime::math::solves`, which builds the route and a per-backend
session on the owning worker. Solver selection varies independently of model preparation.
Backend-specific policy stays in the adapters, and routing sees only capability facts.

**Correction (addendum, same day).** The original text called this separation the design's
strongest property and said it makes a new backend an additive change. The principles sweep
([F27](#f27)) disproves that. A new backend edits about a dozen places across three crates, and
the NLP orchestration is copied four times ([F28](#f28)). The intent is right; the realization is not.

## 3. Contracts, authority and constraints

| Meaning / contract | Authoritative owner and update path | Consumer obligations / invariant | Enforcement and failure | Derived representations / evolution |
|---|---|---|---|---|
| Supported problem classes | §18.9 capability matrix; §25 scope; ADR-0083 | Unsupported integrality, convexity, representation, derivative or root-domain combinations are refused before execution | `Requirements::eligibility` lists every reason | `NativeProblemClass` registry enum (ADR-0090). No MINLP, nonconvex or global class exists |
| Solve outcome and qualification | §16.6, §18.6; `quality.rs` | Native status is never read as success; quality is recomputed in original units | Typed `NativeTermination` and `Qualification` | `GapQualified` only for HiGHS discrete models after upload readback (`quality.rs:514-535`) |
| Native options | §16.6, §18.6 ("queried native defaults") | Reserved keys are refused; tolerances come from one numerical policy | Per-adapter `reject_reserved` | Recorded in `report.options`. **Not the effective native state after session reuse** (F02); POUNCE effective options are not snapshotted (F03) |
| Threading | §18.8 | "A job acquires … CPU permits for its admitted cores and a pool reservation covering stacks" | POUNCE `with_threads` scoped pool | **FERAL builds its own pool** (F01) |
| Warm starts | §17.6, §18.9 | A start is an input with provenance (PS-08, PS-11) | `StartReceipt` | The SQP working set is claimed but unreachable (F07) |
| Variable domains | `pse-math::binding::VariableDomain` (continuous, integer, binary, semi) | Integer domains are never relaxed (`routing.rs` test) | Compiler emits `Continuous` only (`grouped.rs:298-304`) | There is no authored domain declaration, so the MILP class is internal-only (F08) |
| Structural representation at the solver boundary | `presolve_facts()` (rows only); `PreparedBody::expression` (Symbolica, inside pse-math) | Tapes are loss-aware and never an evaluator | `complete[r]` flag | No objective tape, no exported factorable DAG: the deciding fact for SCIP |

**Physical-semantics table (solver-facing quantities).**

| Quantity or model element | Dimension and unit | Basis | Reference state / convention | Validity envelope | Authority |
|---|---|---|---|---|---|
| Residual and bound violation in qualification | Physical row and variable units | Original (un-normalized) coordinates | Normalization is a positive diagonal transform (§16.1) | Tolerances derived from the resolved numerical policy | §16.6, `quality.rs` |
| Ipopt progress metrics `stationarity.unscaled`, `iterate.unscaled.*` | **Normalized coordinates, despite the name** (F10) | Normalized | — | — | `ipopt.rs:349, 370` |
| Duals, shadow prices, ranging | Objective unit per row unit | Original | HiGHS LP only; MILP duals withheld unless valid | LP optimal; no QP rays or ranging | §15.5 |
| Covariance, confidence intervals, parametric sensitivity (target) | Parameter unit² / output unit per parameter unit | Original, requiring the normalization back-map H_phys = S_f·S⁻¹·H_norm·S⁻¹ | SOSC, LICQ, strict complementarity, stable active set, statistical model (PS-12) | Withheld when conditions fail | Plan 20 §6 (target); not implemented |
| Global gap (prospective SCIP) | Objective unit | Original | SCIP's feasibility tolerance and declared box | Exact export or a sound relaxation only | Proposed (slot 8) |

**Well-posedness statement.**
- The class is derived from compiler facts, never from author hints (§18.1, PS-09). Structural analysis and DoF run before routing, and eligibility refuses unsupported combinations with reasons.
- KINSOL is refused for general boxes before solve. Ipopt or POUNCE feasibility then takes over, and the agent reading of KINSOL's sign-only constraints confirms that design is correct.
- A structurally singular square system is diagnosed before any solver runs.
- What cannot yet be stated:
  - finite boxes on nonlinear variables are not required (`grouped.rs:303` defaults `upper: None`), which any spatial branch-and-bound needs;
  - no integer declaration exists to be well-posed about.

## 4. Change scenarios and composition

| Scenario / stimulus and conditions | Expected response and change boundary | Edit/composition path | Observed or predicted impact | Acceptance and evidence |
|---|---|---|---|---|
| <a id="s01"></a>S01 (PSE-S02) Add SCIP as an explicitly selected continuous "certify" backend | New `Backend::Scip`, a factorable projection and qualification vocabulary; routing gains reasons; no consumer re-authors policy | pse-math neutral factorable DAG (beside `presolve.rs`) → `pse-backend-native::scip` (feature) → `Backend`/`ProblemClass`/`Assurance` registry values (ADR-0090, codegen) → routing reasons → `quality.rs` global assurance | Additive. The vocabulary change is a legitimate new core concept (global assurance). The existing five-fact selection absorbs it | Predicted; *Proposed* |
| <a id="s02"></a>S02 (PSE-S03) Covariance and parametric sensitivity after a qualified fit | A sensitivity entry beside `pounce::Session`; fitting and studies consume `SensResult`; validity flags per PS-12 | `pounce-rs` feature `sensitivity`; the fitting workflow consumes the result. Presolve Off, or kept parameter columns plus the normalization back-map | Additive in the backend and fitting owners. Plan 20 §6 would instead build a faer KKT LU, which gives no inertia on a symmetric indefinite KKT (G8) | *Interface-checked* (`pounce-sens-core`); *Proposed* (`pounce-sensitivity` API) |
| <a id="s03"></a>S03 (PSE-S06) Upgrade highs-sys 1.14.3 → 1.15.0 | Absorbed in `highs.rs` | Pin bump; set `qp_allow_hot_start=true` | 1.15 makes QP hot start opt-in, so an unchanged adapter would silently lose QP warm starts (PS-11) | *Interface-checked* |
| <a id="s04"></a>S04 (PSE-S06) Upgrade Clarabel | Should be absorbed in `conic.rs` | Pin bump | **Leaks.** Clarabel's serde encoding is the Python conic wire format and feeds request identity (`strategies.rs:29-54, 90-96`), and `conic.rs` exposes `CscMatrix`/`SupportedConeT`/`DefaultSettings` (F09) | *Interface-checked* |
| <a id="s05"></a>S05 (PSE-S06) Upgrade Diffsol with a new option field | Absorbed in `dynamics` | Pin bump | The serde remote fails to compile (good), but the hand-written `settings_identity` silently omits the new field (F09) | *Interface-checked* |
| <a id="s06"></a>S06 Authored binary decision (W5 price-taker, linear operations) | Kernel concept: a domain facet → compiler → `CoefficientProblem` → HiGHS MILP | Grammar (`parser.rs`), registry, `grouped.rs`, routing | A genuinely new core concept. **It needs no SCIP**: HiGHS serves the linear MILP. SCIP becomes necessary only with nonlinear operation models | *Proposed* |
| <a id="s07"></a>S07 Edit→re-solve with different native options on a reused session | New options take effect, old ones do not persist, the report states what ran | `SolveSequence` reuse (`solves.rs:1065-1126`) | **Options persist** in reused Ipopt and POUNCE handles; the report misstates them (F02) | *Interface-checked* |
| <a id="s08"></a>S08 (PSE-S05) Test routing and admission locally | No native startup | `Requirements::eligibility` over facts | Satisfied | Routing tests (not run here) |
| <a id="s09"></a>S09 Dynamic start with piecewise inputs and recoverable trials (IDAES PETSc PID parity) | IDAS with `IDAReInit`/`IDASensReInit`/`IDACalcIC` at change times | `idas.rs` | Refused today by both routes (`dynamics.rs:245-249, 268-274`) | *Interface-checked* |

## 5. Mechanisms and execution

| Stage / owner | Formulation policy | Derivative source and order | Scaling | Problem class · solver capability used | Status → outcome mapping | Tolerances · post-solve check |
|---|---|---|---|---|---|---|
| Presolve (`presolve::Pipeline`) | Guards retained as obligations; smoothing is authored | Symbolica; FBBT tapes (rows only) | Normalization first | pounce-presolve FBBT, affine elimination, certified infeasibility | Certified infeasibility re-confirmed; fallback to the original problem on structural mismatch | Automatic presolve can return multipliers that fail original complementarity (§25) |
| NLP: Ipopt (`ipopt.rs`) | Smooth; complementarity by ε-smoothing | Exact Hessian, or limited memory | §16.1 normalization; gradient-based on top (`ipopt.rs:701-713`); native `Scaling` path has no caller (F11) | 3.14.20 C API, MUMPS sequential **without METIS**, `linear_solver` locked | Every status mapped (`ipopt.rs:388-497`) | Original-space quality; options persist across reuse (F02) |
| NLP: POUNCE (`pounce.rs`) | As Ipopt | As Ipopt | As Ipopt | Interior point and active-set SQP; FERAL; restoration | Mapped; unknown statuses wildcard to `Numerical` (F10) | **Internal μ-fallback retry unrecorded** (F03); FERAL threads unadmitted (F01) |
| Roots (`kinsol.rs`) | Sign constraints only | Analytic sparse Jacobian (KLU); dense; SPGMR JVP | Normalization | Line search, FP, Picard, Anderson | `KIN_STEP_LT_STPTOL` → Acceptable; roots top out at Feasible | Original residual re-validated |
| Coefficient (`highs.rs`) | Affine/degree-two proofs; convexity by exact Gram certificate or opt-in PSD | Exact coefficients | HiGHS scaling (off only for simplex LP) | LP/MILP/convex QP; IIS, rays, ranging, relaxation | Full model readback before any claim transfers | Explicit method on a QP ignored or `ModelError` (F04); QP regularisation gap (§25) |
| Conic (`conic.rs`) | Explicit cones only | — | Clarabel equilibration from policy | Zero/NN/SOC/exp/pow/genpow/PSD; certificates | Mapped | Serial QDLDL |
| Dynamics (`integrator.rs`, `idas.rs`) | ODE / semi-explicit index-1; events and resets on Diffsol only | Analytic; forward sensitivities (Diffsol `bdf_sens`, IDAS simultaneous) | State and residual scales | Diffsol BDF (FaerSparseLU); IDAS KLU + `IDACalcIC` | **IDAS native failures → `Contract`/InvalidModel** (F06) | IDAS sensitivity assembly O(n²·np) (F12) |
| Fitting (`workflow::fitting`) | Declared statistical model | Gram + constraint Hessian (steady); limited memory (transient) | Normalization | NLP over the steady or dynamic oracle | Qualified estimate needs convergence, feasibility and response rank | No covariance (§25) |

## 6. Architectural assessment and gates

| Foundation | Scenario and evidence / scope reason | Verdict | Required action |
|---|---|---|---|
| AP-01 Separation of concerns | One file per library; routing is the only policy owner; runtime composition separated (slot 2) | **satisfied** | — |
| AP-02 Stable contracts | S04/S05: Clarabel's serde forms are the Python wire and identity contract; `conic.rs` exposes Clarabel types; `dynamics::Profile` exposes Diffsol option types, and their identity is hand-written | **violated** (F09) | Project pse-owned types at the boundary, or record the coupling as a declared contract with an upgrade check |
| AP-03 Composition | Five-fact selection, class-specific sessions and `InnerSolver` injection are sound. However, a new backend needs coordinated switches across `solves.rs`, `solve.rs`, routing, initialization and fitting ([F27](#f27)). The NLP orchestration is copied four times ([F28](#f28)), and there are two initialization and two multi-case engines ([F29](#f29)) | **violated** (corrected by the addendum) | One backend-execution adapter; one NLP runner; one staged-sequence primitive |
| AP-04 Authoritative meaning | S07: recorded `report.options` and the effective native option table diverge after reuse; ADR-0028 consequences and `docker/solvers/README.md` describe HSL selection the code forbids | **violated** (F02, F11) | Reset or compare option state on reuse; correct the stale texts |
| AP-05 Explicit structure and constraints | §18.8 admission is bypassed by FERAL's own pool; POUNCE's default-on μ-fallback retry is an implicit second attempt | **violated** (F01, F03) | Serial FERAL until pool injection exists; pin and record retries |
| AP-06 Local reasoning / testability | S08: routing and projections are pure; SCIP projection could be tested without SCIP | **satisfied** | — |

| Gate | Result | Evidence or scope reason | Required action |
|---|---|---|---|
| G1 Authority | **fail** | F02 (effective vs recorded options); F11 (stale HSL texts) | Slot 12 P1, P3 |
| G2 Semantic fidelity | **fail (minor)** | F10: metrics named "unscaled" are normalized | Rename or convert |
| G3 Validity | pass | Class refusal before solve with reasons; integer domains never relaxed | — |
| G4 Hidden behaviour | **fail** | F03 (unrecorded POUNCE retry); F04 (HiGHS silently ignores an explicit QP method) | Slot 12 P1 |
| G5 Consistency and recovery | pass | Outcomes truthful; POUNCE 0.12 restores the first attempt when the retry fails; partial trajectories published as such | — |
| G6 Transformation and reuse | **fail** | F02 (option leakage on reuse); F09 (Diffsol identity can omit fields) | Slot 12 P1, P2 |
| G7 Truthful capability claims | **fail** | F01 (§18.8 admission claim), F07 (SQP working-set start claimed), F08 (MILP class without an authored route; KINSOL Dense/SPGMR/Picard advertised without solve evidence; PDLP listed as interruptible) | Slot 12 P1, P2 |
| G8 Library leverage | **pass (current) / fail (target, Proposed)** | Current code reimplements no Newton, line search, factorization or time stepping (roots and dynamics pass; only the sanctioned homotopy controller is bespoke). The target Plan 20 §6 proposes a bespoke faer KKT sensitivity where POUNCE sensitivity exists, and a bespoke whole-model elastic policy where `pounce-l1penalty` exists. F12 (IDAS dense loops) is minor | Amend Plan 20 §6 (slot 11) |
| G9 Architectural fitness | **fail** | AP-02, AP-04, AP-05 violated. Each correction is local to one owner; no cross-owner redesign is needed | Slot 12 |
| PS-G1 Physical consistency | not assessed | Property physics and basis out of scope; solver quality is recomputed in original units | — |
| PS-G2 Well-posedness | pass | Class, DoF and structural analysis before solve; KINSOL box refusal before solve | — |
| PS-G3 Numerical integrity | **fail** | F06 (IDAS failures misclassified as invalid model); F04 (explicit QP method); F03 (retry outside the declared iteration budget) | Slot 12 P1 |

## 7. Findings

Current defects (F01–F12) are separated from proposed capability additions (slot 8).
The correctness items F01–F07 were adopted after the reviewer checked the cited source. Each
remains *Interface-checked*: nothing was executed.

| ID | Finding | Principles / gate / scenario | Evidence or gap | Consequence | Correction | Verification |
|---|---|---|---|---|---|---|
| <a id="f01"></a>F01 | FERAL factorizes on its own rayon pool sized from `RAYON_NUM_THREADS` or `available_parallelism()`, outside the admitted scoped pool | AP-05, DP-20 · G7 | `pounce.rs:377` sets `feral.parallel = Some(threads > 1)`. FERAL `numeric/solver.rs:658-712` builds `ThreadPoolBuilder::new().num_threads(pool_num_threads())`. The scoped pool at `pounce.rs:106-126` does not bound it | With `threads > 1`, a job uses unadmitted cores and unreserved stacks, contrary to §18.8; concurrent jobs oversubscribe | Run FERAL serial, or admit all cores when parallel. Ask upstream for pool injection | A test with `threads = 2` observes the FERAL worker count |
| <a id="f02"></a>F02 | Native options persist across reused Ipopt and POUNCE sessions, so `report.options` can misstate what ran | AP-04, PS-11 · G1, G6 · S07 | Ipopt reuse is keyed on layout, bounds and the Jacobian/Hessian pattern (`ipopt.rs:600-613`); POUNCE on layout only (`pounce.rs:348-350`). Neither re-creates the option table. KINSOL does check `matches_settings` (`solves.rs:1785`) | A later step without option K inherits K, so results depend on sequence history that is not recorded (PS-11 warm-start and reuse dependency) | Include the option set in the reuse key, or reset options on reuse | A two-step sequence test with differing options |
| <a id="f03"></a>F03 | POUNCE's default-on `mu_strategy_fallback` can run a second solve beyond `controls.iterations`. `l1_fallback_on_restoration_failure` and `l1_exact_penalty_barrier` are reachable through untyped options. Effective POUNCE options are not snapshotted | AP-05, §18 "no fallback", PS-10 · G4, PS-G3 | `pounce-algorithm` `upstream_options.rs:2460-2463`; `pounce.rs:260-293` does not reserve them; HiGHS by contrast snapshots options (`highs.rs:759-761`) | An iteration-limited result can come from an undeclared second attempt, and the progress trace concatenates both attempts | Pin `mu_strategy_fallback=no` and reserve the l1 fallbacks, unless a typed policy selects them. Snapshot the effective options | Adapter unit test of reserved keys; option snapshot present in the report |
| <a id="f04"></a>F04 | An explicit HiGHS `Simplex`/`Pdlp` method on a continuous QP is silently ignored (active set runs; the snapshot records the method). `Ipm` gives `ModelError` because HiPO is not built | PS-10 · G4, G7, PS-G3 | `highs.rs:594-598` refuses non-`Choose` only for discrete models. HiGHS `HighsOptions.cpp:1157-1176`; HIPO off (`CMakeLists.txt:162`) | Misrecorded provenance, or an `Invalid` termination for a valid request (Rust `BackendSettings::Highs` only) | Refuse a non-`Choose` method when the Hessian is nonzero | Adapter unit test |
| <a id="f05"></a>F05 | The HiGHS callback writes `user_interrupt` for every callback kind, including MIP kinds where HiGHS asserts that no action is taken | PS-10 robustness | `highs.rs:1124-1127`; `HighsCallback::callbackAction` assertions | Harmless under `highs_release` (NDEBUG). A debug or `discover` build would abort the process with a C++ assert that `catch_unwind` cannot contain | Set the flag only for kinds 1, 2 and 6 | Unit test over callback kinds |
| <a id="f06"></a>F06 | IDAS native failure flags and terminal-policy failures become `ProblemError::Contract`, which maps to `Class::InvalidModel`. Diffsol native errors are untyped strings | PS-10 · PS-G3 | `idas.rs:698-700, 729-737`; `callback.rs:59`; `pse-runtime/src/workflow/diagnostics.rs:175`; `integrator.rs:397-399` | Convergence and error-test failures are reported as an invalid model, and the source identity is lost | Map IDA flags to typed terminations, as `kinsol::termination` does; keep the typed `ProblemError` | Unit test forcing `IDA_CONV_FAIL` |
| <a id="f07"></a>F07 | The POUNCE SQP working-set warm start is claimed (§17.6, §18.9 "Primal/dual, working set") but the runtime cannot reach it | PS-11 · G7 | `presolve/pipeline.rs:156` refuses non-`Nlp` seeds; every POUNCE route goes through the pipeline (`solves.rs:1713`, `initialization.rs:608`, `fitting/oracle.rs:476`) | NMPC, sweeps and edit→re-solve cannot use the active-set restart the design advertises | Key native seeds by `report.transformation` so the pipeline can pass them through, or narrow the claim | Runtime test of an SQP working-set restart |
| <a id="f08"></a>F08 | Capability claims run ahead of routes: (a) MILP is listed as a supported class (§18.9, §25) but authored models compile only to `Continuous`; (b) KINSOL Dense, SPGMR and Picard are advertised (§18.1, §18.10) with no solve test; (c) PDLP is covered by the interrupt claim but never polls the callback | G7, DP-22 | (a) `grouped.rs:298-304`; (b) no solve uses `Linear::Dense`, `Linear::Spgmr` or `Function::Picard`; (c) HiGHS `pdlp/*Wrapper.cpp` | Readers infer authored MILP support and untested solver paths | Scope the claims (internal MILP routes; time-limit-only PDLP), or add the domain facet (S06) and solve tests | Documentation or tests |
| <a id="f09"></a>F09 | Library types and encodings form public contracts: Clarabel serde forms are the Python conic wire format and identity input; `conic.rs` exposes Clarabel types; Diffsol options are in `dynamics::Profile`, with a hand-written identity | AP-02, PS-11 · G6 · S04, S05 | `strategies.rs:29-54, 90-96`; `conic.rs:13-23`; `dynamics.rs:219-226, 552-554` | A library upgrade silently changes a Python contract and request identity, or omits a new option from reuse identity | Project pse-owned setting types, or derive the identity from the serde remote and add an upgrade check | Identity test over all option fields |
| <a id="f10"></a>F10 | Minor semantic and robustness items: Ipopt `*.unscaled` metrics are in normalized coordinates (`ipopt.rs:349, 370`); POUNCE status name uses `Debug`, and a wildcard maps unknown statuses to `Numerical` (`pounce.rs:605-609`); the shared iteration control becomes `mip_max_nodes` (default 3000) for internal MILPs | PS-07, PS-10 · G2 | As cited | Misread diagnostics; an upgrade could hide a new status; internal MILPs truncated with `SolutionLimit` | Rename or convert metrics; make the status match exhaustive; give internal MILPs their own node budget | Unit tests |
| <a id="f11"></a>F11 | Stale or dead authority: ADR-0028's consequences (`fallback_linear_solver`, HSL selection) and `docker/solvers/README.md:159-185` contradict `ipopt.rs:679, 700`, which forces MUMPS. The native Ipopt/POUNCE `Scaling` input has no production caller | AP-04, DP-16 · G1 | `solves.rs:1649, 1718`, `initialization.rs:612, 633` and `fitting/oracle.rs:480` all pass `None` | Readers and agents follow texts that the code refuses; dead scaling code could compound normalization if revived | Correct the README; supersede the ADR-0028 consequence with the HSL decision (slot 8, L-N2). Delete the dead `Scaling` path | Review |
| <a id="f12"></a>F12 | The IDAS sensitivity residual and output sensitivities loop over every (row, column) pair with sparse `get` | G8, DP-10 | `idas.rs:230-246, 576-585`; the Diffsol path uses faer sparse×dense (`integrator.rs:195-202`) | O(n²·np) cost on large dynamic flowsheets (unmeasured; no IDAS benchmark) | Use the faer sparse product | Benchmark before and after |

**Strengths that bear on the argument.**
- Status mapping is exhaustive and pessimistic: Ipopt statuses carry no assurance.
- Quality is recomputed independently in original units.
- HiGHS uploads are read back in full before any claim transfers, and MILP duals are withheld when invalid.
- Presolve infeasibility proofs are re-confirmed.
- The KINSOL box-bound refusal routes to Ipopt or POUNCE correctly.
- No bespoke Newton, line search, factorization or integrator exists (PS-09).

## 8. Library fit and ownership cost

The verdict key applies throughout:

| Verdict | Meaning |
|---|---|
| **A** | Adopt within accepted decisions (an ordinary PR, or a short ADR for a new binding under ADR-0083) |
| **B** | Needs a scope change: ADR plus design review |
| **C** | Not beneficial now; the observable trigger is named |
| **D** | Reject |

Evidence is *Interface-checked* unless marked *Proposed*.

### 8.1 SCIP (the maintainer's hypothesis)

**Deciding facts.**

1. **No discrete demand reaches a solver.** The compiler emits only continuous variables (`grouped.rs:298-304`). The only integer producers are the internal MILPs in `tears.rs` and `jacobian_diagnostics.rs`, which HiGHS already solves well.
2. **SCIP is not a better solver for any class the pipeline solves today.**
   - It delegates local NLP to Ipopt (`nlpi_ipopt`) and has no Newton root solver.
   - DegeneracyHunter is a small MILP. IDAES's `solver="scip"` default (`degeneracy_hunter.py:69`) is incidental, and HiGHS loses nothing.
3. **Its value is new assurance classes:**
   - global dual bounds and gap for nonconvex problems;
   - global infeasibility proofs, and an IIS over the true problem (`SCIPgenerateIIS`);
   - MINLP/GDP.

   SCIP 10 guarantees these only for **bounded** problems, within its numerical tolerance (`doc/xternal.c:450-456`).
4. **Representation.**
   - Solvers see callbacks only. Symbolica atoms (`PreparedBody::expression`) stay in pse-math, and FBBT tapes cover rows only (no objective).
   - A user expression handler wrapped around an opaque provider cannot preserve global guarantees: it needs rigorous `INTEVAL` and `ESTIMATE` callbacks, not derivatives (`xternal.c:4354-4370`).
   - Opaque parts can therefore only be exported as free auxiliaries: a sound relaxation that yields weaker bounds.
5. **Bounds.** Nonlinear variables have no finite upper bounds by default (`grouped.rs:303`), and spatial branch-and-bound needs them.
6. **Census (P1, slot 10).** Across all 36 steady reference cases, 92.9% of rows are SCIP-exact representable. The rest (7.1%, 131 rows) sit in exactly the cases where global assurance matters:
   - PR cubic provider outputs in the BT-PR heater, heat exchanger and flash;
   - nested implicit realizations;
   - branch-stage rows in all three PC-SAFT cases.

   **The only authored optimization, `heater_optimization`, has 2 of 7 rows non-projectable and an unestablished objective degree.** SCIP could certify it today only as a relaxation.

| Capability / contract | Integration owner / exposed types | Candidate or current mechanism | Fit and limits | Coupling, lifecycle, test, upgrade/replacement cost | Recommendation |
|---|---|---|---|---|---|
| **Global bound and gap certification** of small bounded factorable NLPs: single-unit design optimization, small kinetic or thermodynamic regressions | pse-math neutral factorable DAG (n-ary sum/product, rational pow, exp, log, abs, sin, cos; per-row and objective fidelity `Exact`/`Relaxed`/`Unavailable`; obligations as bounds) → `pse-backend-native::scip` → `GapQualified` with box, tolerances and fidelity recorded | Today: Ipopt/POUNCE `Stationary` only. Candidate: raw `scip-sys` `SCIPcreateExpr*`, `SCIPcreateConsBasicNonlinear` (epigraph objective), `SCIPaddSolFree` with the Ipopt incumbent, `SCIPgetDualbound`/`SCIPgetGap` | Sound as exact or relaxed export. Closes gaps only at small scale; upstream calls non-quadratic support "not yet as robust" (`faqtext.txt:204-220`) | High: solver-image build, FFI lifecycle on the owning worker (russcip `Rc` is `!Send`, matching §18.8), export readback, new assurance vocabulary (ADR-0090). The projection is testable without SCIP | **B**. Explicit selection only, never automatic. *Proposed* for value |
| **Global infeasibility proof and nonlinear IIS** | Same owner; a new native-proof assurance, distinct from Ipopt's `(Infeasible, None)` | Today: an elastic deletion filter with local solves, explicitly uncertified (`diagnostic_nonlinear.rs:3, 40-41`). Candidate: `SCIPgenerateIIS` (greedy deletion with global sub-solves, honest `irreducible` flag) | A relaxation export keeps infeasibility proofs sound. Cost is bounded by `iis/time` | As above | **B** |
| **Global phase-stability (tangent-plane distance) check** | A package `@check` consumer → SCIP route | Today: "unproved global stability" is a refused claim (§25, §9) | PR (implicit residual exported exactly) and PC-SAFT (authored, no association) are factorable once the branch-stage rows (census) are resolved; n+1 variables; density box declared | Consumer and oracles exist (teqp) | **B**. Strongest in-scope value |
| **MINLP, GDP, explicit discrete design modes** | Domain facet (S06) + K11 disjunction lowering (generalizing `implicit … regime … eligible`) + SCIP adapter | Refused today. Candidates: indicator, SOS1/2, disjunction, bounddisjunction handlers | The only credible open-source nonconvex MINLP route; globally optimal only at small scale; not for phase appearance, where smoothing and complementarity are standard (PS-06) | Largest: authoring, registry, transformation, qualification | **B** (outside §25 today) |
| Nonconvex QP / MIQP | `CoefficientProblem` → russcip `add_cons_quadratic` | Nonconvex QP → local NLP; MIQP refused | Good fit, but MIQP is unreachable without integer authoring, and nonconvex QP with affine rows is rare | Cheapest SCIP consumer | **C**: trigger is a price-taker study with quadratic cost and declared commitment |
| MIP IIS on the true MIP | HiGHS diagnostics | HiGHS relaxation IIS vs `SCIPgenerateIIS` | No user MILP; internal MILPs never need an IIS | — | **C** |
| Solution pool; reoptimization | — | `cons_countsols`; `SCIPenableReoptimization` (MIP sequences; binary-only tree compression) | No declared demand | — | **C** |
| Exact rational mode | — | `SCIPenableExactSolving` (MILP only; GMP, MPFR, Boost) | Answers a different question from tolerance-based degeneracy; exact rank is available through Symbolica/faer | — | **D** |
| DegeneracyHunter, tears, local NLP, roots | HiGHS, Ipopt, POUNCE, KINSOL | — | No advantage | — | **D** |
| Build route | `docker/solvers/` recipe; `scip-sys` via `SCIPOPTDIR` | `bundled`: unverified download of a prebuilt binary carrying a second Ipopt (3.14.19). `from-source`: `IPOPT=OFF`, no PaPILO, no exact mode | Only a solver-image build is acceptable: checksummed scipoptsuite 10.0.2, `IPOPT=ON` against `/opt/pse-solvers`, `LPS=spx` (the HiGHS LPI is experimental), `PAPILO=ON`, `THREADSAFE=ON`. Reserve `misc/catchctrlc=FALSE`; cancel via an event handler calling `SCIPinterruptSolve` on the owning thread; map raw `SCIPgetStatus`, never russcip's panicking `From` | ADR amending ADR-0028; libclang on `scip` builds | Precondition for any B above |

**SCIP verdict.** The hypothesis is right about value and wrong about placement.
- SCIP adds no benefit to the problem classes the pipeline solves today: square simulation, local NLP, LP/MILP/convex QP, cones, dynamics and fitting.
- It is the right library for three capabilities the pipeline does not have: global certification, global infeasibility proof, and MINLP/GDP.

All three are outside the current §25 target. They become beneficial when one of these triggers fires:
1. an authored discrete decision needs nonlinear operation models;
2. the maintainer lifts §25's "unproved global stability" or global-estimation limits;
3. factorable coverage is high **and** measured gap closure on a small fixture set is acceptable. The fixtures: a PR/PC-SAFT binary TPD, a small regression, and `heater_optimization` once its branch rows are projectable.

The minimal first step when triggered is an **explicitly selected, serial, continuous certify route**:
- the neutral factorable projection in pse-math (relaxation-sound);
- raw `scip-sys` export;
- the Ipopt incumbent injected as a start;
- `GapQualified` plus a global-infeasibility assurance;
- TPD `@check` as the first consumer.

Do not start with russcip MIQP or MIP IIS: neither has a reachable consumer.

### 8.2 NLP and sparse linear algebra (Ipopt, POUNCE, FERAL, MUMPS)

| ID | Capability | Candidate mechanism | Fit and limits | Cost | Recommendation |
|---|---|---|---|---|---|
| <a id="l-n1"></a>L-N1 | **Covariance/CI and NLP parametric sensitivity** (Plan 20 §6; IDAES parmest and `sens.py`) | `pounce-rs` feature `sensitivity`: `SensSolve::new(pins).with_deltas(Δp).with_reduced_hessian[_eigen]()` → `SensResult{dx, reduced_hessian}`; `classify_activity`. On the Ipopt route: `pounce-sens-core` `SensApplication` + `parametric_step` over a barrier-replica KKT with a FERAL LDLᵀ backsolver | sIPOPT semantics. Supplies PS-12 validity inputs (activity class, ambiguous kinks, reduced-Hessian eigenvalues for SOSC, Schur failure for LICQ). Limits: exact Hessian only (steady fits); presolve Off or parameter columns kept; normalization back-map; mind the −H_R/+H_R sign convention (gh#937). Transient fits: Gauss–Newton covariance from the existing response SVD (`fitting/oracle.rs:739-909`) | A feature flag under the existing =0.12.0 pins. **Replaces Plan 20's faer sparse-LU KKT**, which gives no inertia on a symmetric indefinite KKT | **B** (§25 refuses covariance today; ADR-0097 target), with an A-type binding. *Interface-checked* (`pounce-sens-core`), *Proposed* (`pounce-sensitivity` API) |
| <a id="l-n2"></a>L-N2 | **HSL MA27/MA57 for Ipopt** | `linear_solver` plus `hsllib` through the linear-solver loader already enabled in the image; the library digest recorded in the profile key; refusal (never fallback) when unloadable | MA57 is the IDAES default. It also enables equilibration-based scaling. Speed benefit is *Proposed*: current benchmarks are small (≤192 variables) | `probe_host` is a stub; local-only qualification | **A**, as a short ADR superseding ADR-0028's `fallback_linear_solver` (which conflicts with §18's no-fallback rule). MA86/MA97 (OpenMP threads) → **B** (§18.8). METIS in MUMPS → **C** (trigger: a factor-dominated case with n_KKT ≳ 10⁴) |
| <a id="l-n3"></a>L-N3 | Typed NLP warm-restart profile (NMPC, sweeps, edit→re-solve) | Ipopt `mu_init`, `warm_start_bound_push`/`mult_bound_push`/`slack_bound_push` from the final barrier value (observable at `ipopt.rs:320`), recorded in `StartReceipt` | With defaults, a dual-seeded interior-point restart re-centres and loses most of the benefit | Small | **A** (§17.6) |
| <a id="l-n4"></a>L-N4 | SQP working-set restart | Fix F07; then expose POUNCE `method`/`linear` to Python | The corrector in the sensitivity-predictor + SQP pattern (advanced-step NMPC) | Small | **A** |
| <a id="l-n5"></a>L-N5 | ℓ1 exact penalty-barrier | `pounce-l1penalty` (already linked through `pounce-algorithm`); Thierry–Biegler, the same method as IDAES `ipopt_l1` | Replaces the whole-model part of Plan 20's bespoke elastic policy. Its own README calls it not the recommended MPCC route | Typed `Method` value | Explicit route **A** (short ADR). For MPCC **C** (trigger: ε-smoothing continuation fails where ℓ1 qualifies in original space) |
| <a id="l-n6"></a>L-N6 | KKT and Jacobian conditioning and certified inertia (Plan 20 diagnostics; PS-12 second-order checks) | FERAL `Solver::estimate_condition_1norm`, `Solver::inertia`; `lu::condition_estimate_1` | Sparse condition estimates and certified inertia on both routes | A direct `feral =0.18.0` dependency (same resolved version) | **A** when the Plan 20 diagnostics are built |
| L-N7 | `pounce-convex` (LP/QP IPM, SOS polynomial bounds, QP sensitivity) | `pounce-rs` feature `convex` | Overlaps HiGHS and Clarabel. The SOS bound is a floating-point SDP, not rigorous, and process models contain exp and log | Adds a crate | **C**: trigger is admitted demand for polynomial lower bounds or parametric QP sensitivity |
| L-N8 | sIPOPT via Ipopt | contrib/sIPOPT (C++ only; nothing in the C API) | Would need a C++ bridge crate | — | **D** (possibly a parity oracle via `ipopt_sens`, unverified) |
| L-N9 | Ipopt native user scaling (H11) | `SetIpoptProblemScaling` | §16.1 normalization is the same diagonal transform; tolerances are defined in normalized space | — | **Not a gap**. Delete the dead path (F11) |

### 8.3 Roots and dynamics (KINSOL, diffsol-nl, Diffsol, IDA/IDAS)

| ID | Capability | Candidate mechanism | Fit and limits | Cost | Recommendation |
|---|---|---|---|---|---|
| <a id="l-d1"></a>L-D1 | IDAS scheduled input changes with recoverable trials (IDAES PETSc PID and antiwindup parity) | `IDASetStopTime` (used), `IDAReInit`, `IDASensReInit`, `IDACalcIC`, `IDAGetSensConsistentIC` at change times | Library-owned; no bespoke maths. This is the one dynamics capability IDAES PETSc parity needs (PETSc retries SNES failures over piecewise elements) | Small | **B (small)**: ADR-0093 revisit + §13.6. Trigger: the first piecewise-input parity case needing recoverable trials |
| <a id="l-d2"></a>L-D2 | Adjoint sensitivities for transient fits with many parameters | Diffsol `bdf_solver_adjoint`, `solve_with_checkpointing`, `solve_adjoint_backwards_pass` (mass matrices supported; reset and terminal-root adjoints reject mass matrices). IDAS `IDAAdjInit`…`IDASolveB` as the alternative | The fit contract needs the full response Jacobian for rank and covariance, so forward sensitivity stays the default. Diffsol is the better owner | Needs `NonLinearOpAdjoint`, `SensAdjoint` and related operator impls | **C** now, **B** when adopted. Trigger: forward-sensitivity cost dominating measured fit time as np grows, with a gradient-only estimator accepted |
| L-D3 | Gauss–Newton Hessian for transient fits | JᵀWJ from forward sensitivities already computed | Cheaper than forward-over-adjoint (IDAS-only, bespoke-heavy) | A new Hessian-mode contract | **B**. Trigger: measured L-BFGS iteration counts or failures on transient fits |
| L-D4 | Shooting-based dynamic optimization, NMPC, MHE | Diffsol forward sensitivity (multiple shooting) or adjoint (single shooting) | The accepted target is simultaneous collocation | New NLP-over-integration route | **B**, only on trigger: an NMPC/MHE case whose simultaneous NLP misses its sampling budget |
| L-D5 | IDAS events without sensitivities; `IDASetConstraints`; staggered sensitivities; Krylov + preconditioners; Diffsol SDIRK (`tr_bdf2`, `esdirk34`); Diffsol KLU backend | As named | All available at these pins. IDAS reset sensitivities would be bespoke (no library saltation), so that part is D | Small to moderate | **C** each. Triggers: sign-violation-dominated trial counts; KLU limits on large flowsheets; BDF restarts dominating change-heavy profiles; measured LU cost |
| L-D6 | KINSOL `KINSetMaxNewtonStep`, eta forms, preconditioners, other Krylov solvers, Anderson orthogonalization/delay | As named | Defaults are sound; KLU remains right | Small | **C**. Triggers: `KIN_MXNEWT_5X_EXCEEDED`; FP+AA stalls on a recycle |
| L-D7 | diffsol-nl as the nested `InnerSolver` | `NewtonNonlinearSolver` | Infallible function, no sign constraints, step-norm stopping, no cancellation. KINSOL's recoverable-trial backtracking would be lost | — | **D**. Consider a per-worker KINSOL session cache instead (C; measure setup overhead) |
| L-D8 | CVODES, ARKODE, `tsit45` | — | Not built at these features; no capability beyond IDAS + Diffsol | — | **D** |

### 8.4 Coefficient and conic (HiGHS, Clarabel)

| ID | Capability | Candidate mechanism | Fit and limits | Cost | Recommendation |
|---|---|---|---|---|---|
| <a id="l-c1"></a>L-C1 | Python projection of backend settings | HiGHS `Method`, diagnostics and `sparse_start`; Clarabel `Mode` and `DefaultSettings` via Clarabel's own serde (`serde(default)`); POUNCE `method`/`linear` after F07 | Python always sends `BackendSettings::Default` (`settings.rs:151`), so Python conic reuse always rebuilds | Additive typed projection; regenerate stubs | **A** (B only if the binding treats new `SolveSettings` fields as a boundary-contract change) |
| L-C2 | highs-sys 1.14.3 → 1.15.0 | Pin bump | IIS and semi-variable fixes, QP non-convexity detection. **1.15 makes QP hot start opt-in** (`qp_allow_hot_start=false`) | Rerun HiGHS unit tests | **A** as an explicit maintainer choice, with `qp_allow_hot_start=true` |
| L-C3 | Fixed-integer LP duals (MILP marginal prices; price-taker) | `Highs_getFixedLp` | Its `kWarning` on fractional values becomes a free validity gate. Duals are conditional on the fixed commitment (PS-12) | About 40 lines of FFI | **C** now (no reachable MILP has physical duals); **A** under §15.5 once S06 lands |
| L-C4 | DegeneracyHunter session reuse | One `Session` per LP/MILP family via `Session::update`, with a structural layout stamp | Avoids up to 2×rows process-wide scheduler resets under the exclusive lifecycle lock | Small | **A** (*Proposed* benefit) |
| L-C5 | Explicit Clarabel eligibility for continuous LP / certified convex QP | `conic.rs::data()` already lowers bounds to cone rows | A second convex-QP owner that avoids the §25 regularisation gap, plus Farkas certificates for QP | Amends §18.9, §18.10 | **B** (short ADR), *Proposed* |
| L-C6 | Automatic cone recognition | The exact Gram certificate already yields the SOC factor | §18.9 excludes it; new compiler lowering | — | **C** now, **B** if pursued. Trigger: convex quadratic families failing on the local route or needing certificates |
| L-C7 | Clarabel faer/Pardiso backend, threads | Feature `faer-sparse` (pulls faer 0.21.x alongside 0.24.4) | The current conic benchmark is 1×1 and cannot show value | Compile time; §18.8 serial admission | **C** |
| L-C8 | HiGHS multi-objective; presolve/postsolve and basis-inverse APIs; cut pool; incumbent vectors from callbacks | As named | No demand class | — | **C** (multi-objective becomes B: it needs an authored multi-objective contract) |
| L-C9 | HiGHS lazy constraints; user-solution callback as a tear seed; automatic Clarabel routing | — | Kind 8 is never invoked in 1.14 or 1.15; seeds are already passed before the run; routing forbids silent fallback | — | **D** |

## 9. Alternatives and tradeoffs

**Global certification and MINLP (the SCIP question).**

| Alternative | Scenarios served / change locality | Contracts, composition and test isolation | Meaning or machinery carried | Correctness / operational cost | Selection and revisit condition |
|---|---|---|---|---|---|
| Current baseline | Local optimality only; discrete decisions and global claims refused | Unchanged | — | Honest (§25 states the limits) | Keep until a §8.1 trigger fires |
| Proposed: SCIP certify route, then MINLP | S01 additive; S06 prerequisite for MINLP | Neutral factorable DAG testable without SCIP; adapter behind a feature | New assurance vocabulary; relaxation-soundness rule | Solver-image build; small-scale only | Adopt at trigger; first consumer TPD `@check` |
| Library-owned alternative: Couenne/Bonmin | Same classes | Couenne needs NL/ASL or C++ expressions (conflicts with D12); Bonmin has no C API and is heuristic on nonconvex problems | — | Low maintenance upstream | **D** |
| Simplest viable: POUNCE `convex` SOS bounds | Polynomial problems only | Already a library feature | Not rigorous (floating-point SDP); monomial basis grows as C(n+d,d); no exp/log | Low | **C** |
| Bespoke outer approximation over HiGHS + Ipopt | MINLP (local) | Bespoke algorithm | Reimplements solver machinery | — | **D** (DP-13, G8) |

**Covariance and parametric sensitivity.**

| Alternative | Assessment | Selection |
|---|---|---|
| Plan 20 §6: faer sparse LU on the KKT | No inertia on a symmetric indefinite KKT, so SOSC cannot be checked; bespoke assembly | **Replace** |
| POUNCE `sensitivity` + `pounce-sens-core` with a FERAL backsolver | Library-owned sIPOPT semantics with PS-12 validity inputs; covers both routes; bespoke part limited to KKT assembly for the Ipopt route | **Select** (L-N1) |
| sIPOPT through Ipopt | C++ only | **D** |
| Gauss–Newton from the response SVD | Transient and limited-memory fits only; drops residual curvature | Complement (L-N1) |

**Reference practice** (behaviour only).
- gPROMS offers local MINLP (OA/ER/AP); Aspen Plus flowsheet optimization is NLP only.
- IDAES users reach GDP and MINLP through Pyomo (GDPopt, MindtPy, SCIP, BARON), but no IDAES 2.13 model or example uses GDP, bonmin, couenne or SCIP beyond a toy solver test and DegeneracyHunter's default.
- idaes-ext deliberately removes SCIP from its build.
- A best-in-class simulator would distinguish itself here, but IDAES parity does not require it.

## 10. Verification

**P1: factorability census (executed).**
- **What:** a scratch harness (session scratchpad, never committed) loads `packages/reference` (seed-data, process, thermodynamics, methods, physical) through the public `ModelingPackage` path. For each `run steady` case it runs `declared_analysis` + `prepare_analysis` and reads `PreparedCase::presolve` (`complete`, `affine`, `objective_degree`). It classifies each opaque row by walking `PreparedBody::expression` with the same opacity rules as `presolve.rs`.
- **Build:** debug profile in an isolated target directory (`target/tmp/p1-census`), first with `pse-runtime/solver-pounce`. The four nested-realization cases need the KINSOL capability to prepare, so they were rerun with `solver-pounce` + `solver-kinsol` under `scripts/native-math-env.sh`.
- **Run:** 2 m 48 s for the 32 cases, on the local workstation.
- **Scope:** all 36 `run steady` cases found by `rg` in `packages/reference`.

| Group | Cases | Rows | FBBT-complete | Opaque cause | SCIP-exact representable |
|---|---|---|---|---|---|
| Fully factorable | 20: saponification heater, mixer, separators ×3, CSTR, pressure changer, pump; volume ×4; PFR backward (622 rows) and Radau (382); compressor; turbine; feed_product; liquid_state; PR `density_inline`; heat_meter (0 rows) | 1,310 | all | — | all |
| Real exponents only | 5: BT-Ideal `flash_368`, `ftpx_oracle`, `fphx_oracle`, `flash_bt_ideal`, `separator_phaseFlow` | 74 | 50 | 24 rows with non-integer powers (FBBT-opaque) | all (SCIP `pow`) |
| PR provider outputs | 4: `density_accelerated`, BT-PR `ftpx_368`, `heater_bt`, `heat_exchanger_bt_cocurrent` | 394 | 288 | 106 provider-output rows (accelerated or nested cubic realization) | Relaxation only, unless the implicit residual is exported instead (the residual atom exists, `implicit_cubic.rs:17`) |
| Branch-stage rows | 3: PC-SAFT `pcsaft_flash`, `heater_recycle`, `heater_optimization` | 24 | 15 | 9 rows depend on a branch stage. The likely source is the `if … then … else` inside the `pcsaft.potential` `valid(...)` guard, not the physics; not resolved here | Relaxation only until resolved |
| Nested implicit realization | 4: PR `density_nested`; ideal-Raoult `nested_liquid`, `nested_two_phase`, `nested_vapor` | 43 | 21 | 16 nested-realization outputs; 6 real-exponent rows | 27 exact; the nested outputs are relaxation-only unless the implicit residual is exported |
| **Total (36 cases)** | | **1,845** | **1,684 (91.3%)** | 161 | **1,714 (92.9%)** |

Reading: the process models are overwhelmingly factorable. The non-factorable remainder is
concentrated in rigorous thermodynamics, and for PC-SAFT it is probably an artefact of how a
validity guard is lowered. It also includes the only authored optimization. Two things are
therefore needed before SCIP could certify the current fixtures exactly: resolving branch-stage
projection, and exporting implicit residuals rather than realizations.

**Correction (G2, 2026-09-27).** The table and the reading above keep the original
observation. Their attribution of the 9 PC-SAFT rows in the "Branch-stage rows" group is
wrong. The cause was the flattening limit of `analyze()`: 16,384 operations or a 1 MB
substitution. PC-SAFT's Helmholtz derivatives share many intermediates, so flattening them
exceeded that limit, and their outputs lost the flattened expression that this census walked.
The `valid(...)` guard was not the cause. Plan 22 G2's `FactorableProgram` projects the
demanded stage program as a shared DAG instead. With implicit residual definitions supplied,
1,830 of the 1,845 steady rows are Exact and 15 are Relaxed: the `nested-equilibrium` regime
outputs. No row is Unavailable. Without implicit definitions, only 1,723 rows are exact.
FBBT-complete rows went from 1,684 to 1,693. Per case, as Exact/Relaxed/Unavailable:

- `pcsaft_flash` 11/0/0;
- `heater_optimization` 7/0/0, with the objective Exact;
- `heater_recycle` 6/0/0;
- `heat_exchanger_bt_cocurrent` 228/0/0, with 32 implicit blocks;
- `heater_bt` 112/0/0;
- `ftpx_368` 53/0/0;
- each density case 1/0/0;
- each of the three nested cases 9/5/0.

Current packet status is owned by Plan 22 execution packet E11.

**Evidence labels for the main claims.**

| Claim / scenario / risk | Evidence label | Reasoning, test or measurement | Conditions and expected result | Result or gap |
|---|---|---|---|---|
| SCIP adds no value to current classes | Interface-checked | Routing, domain emission, SCIP NLPI delegation | — | Consistent across four passes |
| SCIP certify-route value on small fixtures | Proposed | Upstream documentation; census | Gap closure on TPD, a small regression and `heater_optimization` | Unmeasured; qualification step 7 below |
| Factorable coverage | Measured (P1) | Census as above | All 36 steady cases, debug build, this tree | 91.3% FBBT-complete; 92.9% SCIP-exact |
| F01–F07 defects | Interface-checked | Source read by the reviewer | — | No executed reproduction |
| POUNCE sensitivity API | Proposed | `pounce-rs` facade source; `pounce-sensitivity` not in corpus | — | Fetch and verify before adoption |

**Qualification before any SCIP adoption packet:**
1. An ADR under ADR-0083's revisit trigger plus the §3.3, §16.6, §18.9 and §25 amendments, and an ADR-0090 vocabulary decision.
2. The solver-image recipe amendment to ADR-0028, with checksums.
3. A skill runtime receipt for SCIP 10.0.2: link, ABI/version check, factorable constraint, bound/gap/status mapping, event-handler interrupt, `catchctrlc=FALSE`, memory limit.
4. A pse-math projection unit test (exact and relaxed rows against evaluator values).
5. An export-equivalence readback test.
6. A negative control where a relaxation must give a valid bound or a sound infeasibility proof.
7. Measured gap closure on named fixtures under recorded conditions.

## 11. Authority changes, exceptions and disposition

| Authority text | Conflict | Required change | Route |
|---|---|---|---|
| Plan 20 §6 ("NLP parametric sensitivity from the factorized KKT system … faer sparse LU") | G8: bespoke KKT where POUNCE sensitivity exists; LU gives no inertia | Name POUNCE `sensitivity` / `pounce-sens-core` with FERAL as the mechanism | Edit the living plan |
| Plan 20 elastic/whole-model policy | G8 against `pounce-l1penalty` | Use the explicit ℓ1 route for the whole-model part | Edit the living plan; short ADR for the route |
| ADR-0028 consequence `fallback_linear_solver`; `docker/solvers/README.md` HSL text | Conflicts with §18's no-fallback rule and `ipopt.rs` | Supersede with the HSL decision (L-N2); correct the README | Short ADR (`just adr-supersede`) |
| §18.8 threading claim | F01 | Code correction (no text change needed) | Ordinary PR |
| §17.6 / §18.9 SQP working-set start | F07 | Fix, or narrow the claim | Ordinary PR, or `design:` PR |
| §18.9 / §25 MILP class; §18.1 / §18.10 KINSOL paths; §18.8 PDLP interrupt | F08 | Scope the claims, or add S06 and solve tests | `design:` PR (`PSE_DESIGN_EDIT=1`) |
| §25 "Covariance…" refused; §19 fitting | Blocks L-N1 (a target item under ADR-0097) | Admit covariance and sensitivity with PS-12 validity conditions | Plan 22; ADR + design review |
| §25 "global MINLP … not part of the design target"; §18.9 "Outside the matrix: global MINLP, disjunctive programs"; §19.7; §3.3 "general global MINLP are not admitted"; §25 and §9 "unproved global stability" | Blocks the SCIP B items | Admit an explicitly selected global-certification class first; MINLP/GDP only with S06 and K11 | ADR + design review; ADR-0090 vocabulary |
| ADR-0093 / §13.6 | Blocks L-D1 (and L-D2 when triggered) | Admit IDAS scheduled changes with recoverable trials | Short ADR (revisit trigger) |

**Register.** None of the C items above needs a register row unless the maintainer wants its
trigger tracked. The SCIP triggers (slot 8.1) are the ones worth recording.

**Disposition.** The owner is [Plan 22](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/plans/22-solver-capabilities.md#finding-dispositions).
It records the status of F01–F32 and the coverage of every slot-8 capability. This review keeps
only its observations.

## 12. Decision

**Behavioural adequacy.**
- Adequate for the declared classes, with three PS-G3 defects: F03, F04 and F06.
- The pipeline never reads a solver status as success. It recomputes quality in original units and refuses unsupported classes before solving.

**Architectural fitness.**
- G9 fails on F01, F02 and F09, and, after the addendum, on AP-03 (F27–F29).
- The decomposition intent is right for adding capability, including SCIP: class-specific adapters, one routing owner and pure admission (S08).
- Its realization is not yet additive (F27), so a backend-execution adapter precedes any new backend.

**Overall.** Revise-scoped for the current implementation. The corrections are local to their adapters. The capability recommendations below are accepted at the *Proposed* level.

**On the SCIP hypothesis.**
- SCIP would not improve the problem types the pipeline solves today.
- It is the right future library for global certification and MINLP. Those are outside the current target, so they wait for a trigger, and a certify route comes first.
- The larger and nearer benefits are elsewhere:
  - POUNCE's sensitivity machinery, which replaces a planned bespoke KKT solver;
  - HSL for Ipopt;
  - the SQP and interior-point warm restarts the design already promises;
  - the correctness items F01–F07.

| Priority | Change | Findings / scenarios | Acceptance evidence | Disposition owner |
|---|---|---|---|---|
| P1 | Correct PS-G3 and G4/G6/G7 defects: FERAL serial or fully admitted; options in the reuse key; POUNCE retries pinned and options snapshotted; HiGHS QP method refusal and callback kinds; IDAS typed terminations; SQP working-set pass-through | F01–F07 · S07 | Named unit tests (slot 7) | Plan 22 |
| P2 | Scope the capability claims and fix contract leaks: MILP/KINSOL/PDLP claims; Clarabel and Diffsol boundary types and identity; stale HSL texts; dead `Scaling` path; metric names | F08–F11 · S04, S05 | Review plus identity test | Plan 22 / `design:` PR |
| P3 | NLP leverage within accepted decisions: HSL MA27/MA57 (short ADR); typed warm-restart profile; Python backend-settings projection; FERAL conditioning and inertia for diagnostics; DegeneracyHunter session reuse; highs-sys 1.15 with QP hot start | L-N2, L-N3, L-N4, L-N6, L-C1, L-C2, L-C4 | Unit tests; a measured KKT-size benchmark for HSL | Plan 22 |
| P4 | Covariance and parametric sensitivity through POUNCE `sensitivity` / `pounce-sens-core`; Gauss–Newton covariance for transient fits; amend Plan 20 §6 | L-N1 · S02 | PS-12 flags on analytic NLPs; comparison with `ipopt_sens` as parity oracle | Plan 22; ADR + design review |
| P5 | Discrete domain facet for authored MILP (W5 price-taker) on HiGHS; fixed-LP duals after it | S06, L-C3 | Authored MILP fixture | Plan 22 (kernel concept) |
| P6 | IDAS scheduled inputs with recoverable trials, for PETSc parity | L-D1 · S09 | IDAS change-time test | Plan 22; short ADR |
| P7 | SCIP certify route (global bound, global infeasibility/IIS, TPD `@check`); then MINLP/GDP after P5 and K11 | §8.1 · S01 | Qualification steps 1–7 (slot 10) | Plan 22; ADR + design review |
| — | C items with triggers (adjoint, SDIRK, KINSOL options, Clarabel backends, `pounce-convex`, METIS, multi-objective, MIQP) | slot 8 | Named triggers | Plan 22 (all adopted at the maintainer's direction) |

**What would falsify these conclusions.**
- Measured large-scale gap closure by SCIP on process flowsheets, which would raise its priority.
- A `pounce-sensitivity` API that cannot compose with the FERAL factory or presolve back-map, which would reopen the faer alternative with an inertia-capable factorization.
- A measured benefit of HSL over MUMPS failing to materialize on n_KKT ≳ 10⁴.

## Addendum (2026-09-27): design-principles sweep of solver-adjacent code

The maintainer asked for the solver work to be planned together with alignment to the core
principles. A second read-only pass covered:
- `pse-runtime` (`math/*`, `workflow/*`);
- `pse-backend-native`;
- the `pse-py` workflow bindings.

It checked them against AP-01–AP-06, DP-01–DP-24 and PS-01–PS-13. It found 20 misalignments
not covered by F01–F12. F13, F14, F15 and F17 were re-read in source by the reviewer; the rest
are *Interface-checked* from the pass. Status belongs to
[Plan 22](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/plans/22-solver-capabilities.md#finding-dispositions), not to this addendum.

| ID | Finding | Principles / gate | Evidence | Consequence | Correction |
|---|---|---|---|---|---|
| <a id="f13"></a>F13 | "Accepted candidate" has at least five disagreeing definitions | DP-01, PS-10 · G1, PS-G3 | `quality.rs:486-499` grants Feasible before checking termination; `solves.rs:274-284` (`accepts_feasible_candidate`) ignores termination; `workflow/modeling/results.rs:266-284` has an inline copy; `workflow/numerics.rs:74-81` requires Success/Acceptable/FeasibleOnly; `initialization.rs:745-772` requires Success/Acceptable; `diagnostic_nonlinear.rs:293` has its own rule | An iteration-limited feasible iterate is `accepted`, seeds the next step and advances homotopy, while its published candidate assessment says `Unusable` | One typed `CandidateUse` decision with one owner, consumed by every workflow |
| <a id="f14"></a>F14 | Dynamic initial conditions are chosen by parsing a provenance string | DP-02, PS-08 · G2 | `workflow/modeling/cases.rs:168-297` writes `"annotation:{id}"` and similar; `workflow/modeling/dynamics.rs:733` branches on `starts_with("annotation:")`, and `starts[id]` can panic | A label change silently freezes expression-evaluated initial states | Typed `StartSource` |
| <a id="f15"></a>F15 | A timed-out transient fit is reported as an evaluation failure classified InvalidModel | PS-10, DP-19 · PS-G3 | `fitting/oracle.rs:167-187` turns a non-Completed trajectory into `Err`; the Diffsol deadline is `Contract("dynamic deadline")`; `callback.rs:128-136` latches `Evaluation` | Wrong category for retry logic and users | Carry the typed trajectory termination through; keep TimeLimit |
| <a id="f16"></a>F16 | Independent qualification reads untyped, string-keyed adapter metrics | DP-02, PS-10 | `quality.rs:488-553` reads `"upload.equivalent"`, `"mip_gap"`, `dual_solution_status == Integer(2)`; `callback.rs:177-184`; `solves.rs:1139-1140` | A renamed key or an upgraded native info name silently downgrades qualification or disables retry | Typed evidence structs from adapters; metrics for observation only |
| <a id="f17"></a>F17 | Typed failures are flattened into `Contract(String)` and classified InvalidModel | DP-21 · G3 | `workflow/diagnostics.rs:175`; about 350 `ProblemError::Contract(` sites; `ProviderError` converted with `to_string()` at about 10 sites; `implicit.rs:143-146` drops structural row and column ids; routing's "adapter not linked" becomes InvalidModel | Unsupported, numerical, infrastructure and internal failures are indistinguishable (the IDAS instance is F06) | Typed Provider, Unsupported, Internal, Numerical and Limit variants, with causes preserved |
| <a id="f18"></a>F18 | Validation failure is stored twice, and fit prose becomes a rule code | DP-21, DP-01 | `solve.rs:811-872` has both `validation_failure` (typed, crate-private setter) and `validation_error` (string); `workflow/diagnostics.rs:335-358` passes `FitReport.diagnostic` prose as `rule` | The typed cause is lost; diagnostics carry prose where a stable code belongs | One typed field; strings derived from it |
| <a id="f19"></a>F19 | The diagnostic vocabulary cannot express severity, numerical failure or inconclusive | DP-21 | `workflow/modeling/diagnostics.rs:137-157` classifies scaling warnings as `InvalidModel`; `NativeBoundaryClass` has no such classes | Tools cannot separate warnings from model errors | Severity and the DP-21 categories in the registry vocabulary |
| <a id="f20"></a>F20 | `Controls.accuracy` is both a required-default input and a derived output | DP-02, DP-05, DP-01 | Five checks for `accuracy == Accuracy::default()` (`solves.rs:601, 790`, `initialization.rs:83`, `fitting/preparation.rs:144`, `strategies/conditional.rs:227`); `solve_declared_root` accepts any value | Explicit defaults are indistinguishable from none, and unguarded paths qualify with raw defaults | Separate user controls from a `ResolvedAccuracy` |
| <a id="f21"></a>F21 | The published capability table is not what routing uses | DP-01 · G1, G7 | `solve.rs:33-124` (published) vs hard-coded rules in `routing.rs:84-159`; serial set again at `solves.rs:406-414`; exact-Hessian predicate differs (`solves.rs:737-745` vs `routing.rs:145-149`) | Published capability and actual eligibility can disagree | Derive eligibility from one capability record |
| <a id="f22"></a>F22 | KINSOL settings policy is written five times | DP-01, DP-11 | `solves.rs:621-638, 1746-1767`, `initialization.rs:563-582`, `strategies/conditional.rs:515-529`, `implicit.rs:92-136` (absolute 1e-12, ignores numerical policy) | Valid settings refused elsewhere; nested tolerances ignore policy | `kinsol::Settings::from_policy` |
| <a id="f23"></a>F23 | The coefficient-class rule is written three times | DP-01 | `pse-runtime/src/math.rs:310-321`, compiler `executable/solve.rs:193-204`, `pse-math/src/coefficients.rs:87-94` | Divergence fails preparation or silently removes HiGHS eligibility | One `Facts::coefficient_eligible()` |
| <a id="f24"></a>F24 | One layout hash serves both warm-start compatibility and profile identity | DP-04, DP-09 | `solves.rs:507-558` mixes native options into `layout`; `solve.rs:772-781` needs an exact match; `solves.rs:1083-1099` rejects | Changing a native option rejects the sequence's own seed | Separate a coordinate-compatibility stamp from the profile stamp |
| <a id="f25"></a>F25 | Start and lineage provenance is textual, partly wrong, and missing from result identity | DP-21, PS-08, PS-11 (MUST) | `solves.rs:1120-1122` says "admitted native presolve" for every route; `initialization.rs:674-685` labels initial points as warm starts; `workflow/completion.rs:115-119, 189-191, 250-251` leaves seeds and reused native state out of request identity | Results influenced by warm starts or reuse are not traceable to them | Typed transformation records; seeds and reuse state in lineage identity |
| <a id="f26"></a>F26 | Initialization admission rules exist only in the Python adapter | DP-14, DP-03 · G4 | `pse-py/src/workflow/modeling.rs:836-844` vs `workflow/modeling.rs:492-505` and `initialization.rs:53-124`; execution silently forces `Policy::Off` and POUNCE InteriorPoint (`initialization.rs:611, 643`) | Rust callers bypass the rules; settings are silently overridden | Checks in `validate_profile`; route-typed linear settings |
| <a id="f27"></a>F27 | Adding a backend is not an additive change | AP-01, AP-03, §E · G9 | New backends edit `BackendSettings`, `admit_profile`, `hash_controls`, `with_primal_start`, `run_step` (691 lines, including a HiGHS-only re-check at `:1414-1507`), `Retained`, `ALGEBRAIC_BACKENDS`, `WarmPayload`, routing lists, `workflow/mod.rs:132`, and the initialization and fitting matches | Every new capability in this plan (SCIP, POUNCE-convex, HSL) multiplies edits | A backend-execution adapter owned by `pse-backend-native` |
| <a id="f28"></a>F28 | The NLP orchestration is copied four times, and the copies already differ | PSE-S03, DP-01 | `solves.rs:1036-1052, 1630-1672, 1699-1741`; `initialization.rs:444-455, 604-671`; `fitting/oracle.rs:441-552` | Fixes such as F03, F07 and F11 need four edits; behaviour differs by workflow | One NLP runner |
| <a id="f29"></a>F29 | Two initialization engines and two multi-case engines | AP-03, PS-11, DP-10 | `initialization.rs:383-742` vs `workflow/modeling/engines.rs:338-625`; `study` (`engines.rs:251-335`, re-prepares per point) vs `start_modeling` (`workflow/run.rs:269-314`); `presolve.matches` forces re-preparation (`solves.rs:593-599`) | Divergent stage semantics; value-only steps rebuild structure | One staged-sequence primitive; value-only rebind |
| <a id="f30"></a>F30 | The Python boundary exposes Rust `Debug` output and prose as its contract | DP-24, DP-02, DP-14 | `pse-py/src/workflow.rs:563`, `strategies.rs:33-105`; hand-written string-to-enum tables in `settings.rs:51-107`; eligibility as prose; `results.rs:108-115` | Rust refactors change a Python contract | Names from registry `as_str`; typed eligibility rows |
| <a id="f31"></a>F31 | Cancellation and memory-budget gaps inside jobs | DP-19, DP-20 | `strategies/conditional.rs:174` and `sequence.rs:50` use `Registration::worker()` without the attempt's cancel flag; extra workers created beyond the single reservation (`solves.rs:1012-1016, 1431`) | Uncancellable nested evaluation; memory over-commitment | `worker_scoped` with the attempt flag; reservation per worker |
| <a id="f32"></a>F32 | Very large functions mix reasons for change | AP-01 | `prepare_simulation_mode` 791 lines, `run_step` 691, `prepare_fit_problem` 673, `diagnose_case` 558, `Pipeline::new` 437, HiGHS `solve` 390, `resolve_case` 363 | Changes need wide context; hard to test locally | Split along the responsibilities the F27–F29 corrections create; no split for its own sake |

**Effect on the decision.**
- AP-03 is now **violated** (F27–F29).
- G1 (F13, F21), G2 (F14), G3 (F17) and PS-G3 (F15) gain further failing evidence.
- The overall decision remains **Revise-scoped**. The disposition owner is
  [Plan 22](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/plans/22-solver-capabilities.md).
