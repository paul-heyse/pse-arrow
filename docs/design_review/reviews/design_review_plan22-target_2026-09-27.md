# Design review: Plan 22 target — solver capabilities, discrete decisions and the operational store

## 1. Scope, drivers and coverage

| Field | Content |
|---|---|
| Subject and boundary | The **Plan 22 target design**: the [architecture companion](../../plans/22-solver-capabilities-architecture.md) together with the twelve decision records [ADR-0102](../../adr/0102-discrete-and-global-design-target.md)–[ADR-0113](../../adr/0113-python-solve-settings-and-eligibility.md) (Plan 22 D22-01…D22-12). The boundary is the whole solve path the target touches: authoring and the registry (domains, constraint forms, objectives), compiler lowering, `pse-math` projection, routing and the backend-execution adapters in `pse-backend-native`, the solver image, the runtime workflows (NLP runner, staged sequences, fitting, studies), the new operational store and publication catalog, Delta publication, and the Python boundary. Suppliers read: the pinned library surfaces in the `native-solver-libraries` corpus, PostgreSQL 18 and sqlx 0.9 documentation, crates.io metadata (2026-09-27), idaes-ext 3.4.2, and the maintainer's [external review of Rust PostgreSQL options](../../external-review-postgresl-options.md) with the coordinator's assessment of it. Consumers: the authoritative sections the ADRs amend (§3.3, §6.8, §9.5, §13.6, §15.5, §16.6, §18, §19, §20, §21, §25, §26, D10, D13). |
| Standard | Core **3.0** (AP-01–AP-06, DP-01–DP-24, G1–G9); process-simulator profile **1.1** (PS-01–PS-13, PS-G1–PS-G3); binding `pse-arrow` ([standard.toml](../design_principles/standard.toml)). |
| Tier / purpose | **Design tier, target purpose** (binding default). Authority text that blocks the target is recorded in slot 11 as a required change with its route. |
| Reviewer / date | Agent review in the maintainer's session (Plan 22 D0 track), 2026-09-27. The same agent wrote ADR-0102–ADR-0113 and corrected the architecture companion for the findings below. **This is an author review, not an independent review.** |
| Decisions | Behavioural and semantic adequacy: **adequate at the Proposed level** after the corrections recorded for T01–T16. Architectural fitness: **G9 pass**; all six foundations satisfied. Overall: **Accept** — the target design at the *Proposed* evidence level, within the scope stated below. See [slot 12](#decision). |
| Disposition owner | [Plan 22](../../plans/22-solver-capabilities.md#finding-dispositions). The corrections for T01–T16 are design-text changes made in D0 before acceptance (slot 11 records where); implementation of every packet remains open under the plan. |

**Functional target (profile).** A process simulator that solves every problem class an
engineer can author from one model — square simulation, local and global optimization,
MILP/MIQP/MINLP, generalized disjunctive programs, conic programs, dynamics and dynamic
optimization, estimation with covariance, and parametric sensitivity — with truthful,
physically scoped and durable outcomes, and long or many-case work that runs across processes
without losing state ([architecture §1](../../plans/22-solver-capabilities-architecture.md#1-target-drivers-and-scenarios)).

**Drivers.** PS-09 and DP-13 (libraries own iteration, search and factorization); PS-10 and
PS-12 (typed outcomes; derived quantities carry validity); DP-01 and AP-04 (one authority per
fact, including across two stores); §E and AP-03 (a backend or capability is additive); DP-19
and DP-20 (explicit, durable lifecycle; coordinated resources); DP-24 (versioned durable
contracts); DP-11 (determinism stated for multithreaded native libraries).

**Baseline.** The current system as the [solver capability review](design_review_solver-capabilities_2026-09-27.md)
found it: class-specific adapters behind one routing owner, but a new backend edits about a
dozen places (F27), four NLP orchestration copies (F28), five candidate-acceptance rules (F13),
continuous-only authoring (F08a), a single-local-writer Delta publication, and in-memory runs.

**Analysis modes and workloads examined.** All four analysis modes. The profile's workloads:
edit → re-solve (S14), studies (S15), recycles (unchanged; tear selection stays on HiGHS),
dynamics (S09, S14), and extension by a solver (S18). Failure and recovery journeys: crash of a
long solve (S16), concurrent and lost-acknowledgement publication (S17).

**Not examined.**
- Property-package physics, bases and reference states beyond the solver-facing quantities in
  slot 3 (PS-G1 is settled only for those quantities).
- Runtime behaviour of any library not yet linked: SCIP, SPRAL, oneMKL, METIS, the POUNCE
  `sensitivity` and `convex` features. Their facts are *Interface-checked* at most.
- Performance. Every speed or scale benefit (SPRAL and Pardiso on large KKT systems, SCIP gap
  closure, event throughput) is *Proposed* until Plan 22 Q1 measures it.
- Distribution, wheels and licences for distributed artifacts (register R-09, R-31).

**Evidence base.**
- The architecture companion and the twelve ADRs as written in this session.
- The capability review's slot 8 ledgers, census P1 and addendum findings F13–F32 (reused, not
  re-derived).
- Source-level checks made for this review, all *Interface-checked*: Ipopt 3.14.20
  `doc/install.dox` (Pardiso from MKL requires MKL as Ipopt's BLAS/LAPACK; SPRAL requires
  `OMP_CANCELLATION=TRUE` and `OMP_PROC_BIND=TRUE`), `IpAlgBuilder.cpp` (default
  `linear_solver` precedence: MUMPS ahead of `pardisomkl` and `spral`),
  `IpMumpsSolverInterface.cpp` (`mumps_pivot_order` default 7, automatic), SCIP 10
  `nlpi_ipopt.cpp` parameters (`nlpi/ipopt/linear_solver`, `hsllib`, `pardisolib`); idaes-ext
  3.4.2 `scripts/compile_solvers.sh` (builds ThirdParty/Metis; HSL when present); PostgreSQL 18
  documentation (`uuidv7()`, session- versus transaction-scoped advisory locks, `LISTEN` taking
  effect at commit with the listen-then-inspect rule); sqlx `#[sqlx::test]` (a fresh
  database per test, `DATABASE_URL` required) and `PgListener::try_recv` (`Ok(None)` on a lost
  connection, with notifications in the gap lost); crates.io metadata for sqlx 0.9.0,
  `datafusion-table-providers` 0.13.1 (requires datafusion ^54, arrow ^58) and the ADBC crates
  0.24 (arrow-array ≥58 <60); oneMKL conditional numerical reproducibility
  (a constant thread count is required); SCIP `SCIPenableExactSolving` (before problem creation,
  incompatible with reoptimization).
- Current source read for authority questions: generated `CandidateUse`, `NativeAssurance`,
  `NativeProblemClass` and boundary-class enums; `SolveIntent` in `pse-backend-native`;
  attempt and publication identities minted through UUIDv7 `SemanticId` values.

**Variation axes.** A new native backend or class (PSE-S02, S18); a library upgrade within its
contract (PSE-S06); a new analysis workflow over existing preparation (PSE-S03: certification,
sensitivity, multi-objective, NMPC); a new modeling concept (discrete domains, disjunctions);
a change of result storage or transport (PSE-S04: the catalog move); independent policy tests
(PSE-S05).

## 2. Decomposition, ownership and dependencies

| Component / responsibility | Decision or invariant hidden | Contract consumed / exposed | Dependency direction and reason | State/effect owner | Local test setup |
|---|---|---|---|---|---|
| Authoring and registry (`pse-authoring`, `pse-schema`, generated `pse-model`) | Domain facet syntax; declaration families for indicator, SOS, cardinality, piecewise, logic, disjunction, complementarity; objective priority/weight | Registry enums (`ModelingVariableDomain`, `SolveIntent`, assurances, `CandidateUse`, reason codes) → generated Rust/Arrow/Python | Foundations below everything (unchanged direction) | None | Parse/render and codec round trips |
| Kernel checking and lowering (`pse-modeling`, `pse-compiler`) | Per-mode discrete semantics; lowering choice and equivalence; FBBT completeness for derived big-M | Checked revision → lowered rows/variables with lineage to alternatives | Uses `pounce-presolve` FBBT (library) | Pure, Salsa-tracked | Synthetic fixtures; no solver |
| `FactorableProgram` (`pse-math`) | Node vocabulary; per-row fidelity; obligations as bounds; provider `Aux` envelopes | Symbolica atoms → neutral DAG | No solver type; beside the FBBT projection | Immutable | Projection versus evaluator values, with a relaxed-enclosure negative control |
| Routing (`pse-backend-native::routing`) | Five-fact selection; class derivation; explicit versus automatic owners; refusal reasons | `ProblemFacts`, intent, controls, capability records → route | Backend-neutral; reads capability records only | None | Pure (S08) |
| Backend-execution adapters (`pse-backend-native`, Plan 22 A2) | Native lifecycle, reserved options, status maps, pse-owned `Settings`, typed evidence, warm payloads | `BackendExecution` trait; static adapter table | One adapter per library; library types stay inside | Worker-local native handles | Per adapter; a test-only stub adapter proves S18 |
| SCIP adapter (`scip`, feature) | ABI check, reserved options including nested-Ipopt settings, event-handler cancellation, exhaustive status map, relaxed-export semantics | `FactorableProgram` → SCIP; `SolveReport` | `scip-sys` against the image | One `SCIP*` per attempt on the owning worker | Fixture solves; ABI check against the image |
| Ipopt linear algebra (solver image + Ipopt adapter) | One BLAS/LAPACK and one OpenMP runtime; CBWR; SPRAL preconditions; explicit ordering | Typed `linear_solver`/`ordering` → Ipopt options | Image is a build-time supplier; adapter owns selection | Worker-local; admitted threads | Refusal tests without SPRAL/MKL; selection tests in the image |
| Quality and candidate use (`quality.rs`; `workflow/numerics.rs`) | Original-coordinate qualification; the single `CandidateUse` decision | `SolveReport` + closure → `Qualification`, `CandidateUse` | Backend-neutral | None | Pure |
| Shared runners (`pse-runtime`: NLP runner, staged-sequence primitive) | One orchestration for solve, initialization, fitting, certification re-solves, lexicographic stages, homotopy, studies, NMPC | Adapters + prepared cases | Runtime → backend | `MathService` jobs (§18.8) | Workflow tests with stub adapters |
| Sensitivity and covariance (POUNCE route; Ipopt route with `pounce-sens-core` + FERAL) | Validity record; back-maps; GN approximation label | Qualified candidate → sensitivity relations | Adapter/runtime; FERAL direct dependency for inertia | Attempt-local | Analytic NLPs; oracle comparison in the parity container |
| Dynamics extensions (`dynamics/idas.rs`, `integrator.rs`) | Scheduled changes with recoverable trials; adjoint/second-order modes; typed IDA terminations | Compiled dynamic functions → trajectories and sensitivities | Unchanged direction | Worker-local | Fixture integrations |
| Operational store and catalog (`pse-operations`, new) | Transition table; queue claims; leases; catalog CAS; reader leases; protected versions | Typed repositories; migrations; read-only DataFusion provider | `pse-runtime` → `pse-operations` → `pse-engine`, `pse-schema`, `pse-model`, `pse-diagnostics`, sqlx | PostgreSQL | Pure transition table without a database; `#[sqlx::test]` isolated databases for SQL |
| Delta member I/O (`pse-catalog`) | Member writes, exact reopen, deletion/optimize as instructed | Member tables at attempt-scoped paths | Unchanged, minus the control table and file leases | Object store | Existing catalog tests |
| Composition root (`pse-runtime`, `pse-worker` binary) | Durability class; publication composition (members, then catalog commit); worker process environment | Runtime services + store | Only crate joining native and storage sides | Effects | Workflow tests with `Ephemeral` class |
| Python boundary (`pse-py`) | Typed settings projection; registry names; typed eligibility | Native classes and generated contracts | py → runtime | None | Python tests; stub drift checks |

The composition root stays `pse-runtime`. Solver selection varies independently of model
preparation; persistence varies independently of computation. The two seams everything relies
on are the backend-execution adapter with a single NLP runner (A2) and the staged-sequence
primitive (A6).

## 3. Contracts, authority and constraints

| Meaning / contract | Authoritative owner and update path | Consumer obligations / invariant | Enforcement and failure | Derived representations / evolution |
|---|---|---|---|---|
| Variable domain | Registry `ModelingVariableDomain` (ADR-0103) | Discrete variables are never relaxed implicitly; per-mode semantics hold | Admission refusal naming the variable; finite bounds required | Generated into `pse-model`; `pse_math::binding::VariableDomain` deleted (T15) |
| Lowering equivalence | Named transformations in the compiler (ADR-0104) | Realization and parameters in preparation identity and results | Refusal of incomplete FBBT, missing bounds, native-only realizations on the wrong backend | Cross-realization agreement tests |
| Problem class, intent, assurance | Registry vocabulary (ADR-0106, supersedes ADR-0090) | Class derived from facts; `certify` explicit only; assurances state their conditions | Eligibility reasons; exhaustive status maps | Generated Rust/Arrow/Python; codec round trips |
| Candidate use | `workflow/numerics.rs` completion owner (§16.6; ADR-0106) | Every workflow consumes the one value | No other acceptance rule survives (A1) | Published candidate assessment |
| Relaxed-export claims | ADR-0105 relaxation-soundness rule | Relaxed → bound and infeasibility only; candidates re-qualified | `quality.rs` in original coordinates | Gap records both sources (T07) |
| Solver image and Ipopt linear algebra | ADR-0108 (supersedes ADR-0028) | One BLAS/LAPACK, one OpenMP runtime, pinned CBWR, explicit ordering | Admission refusal on unmet precondition | Profile key records versions, branch, solver, ordering, threads |
| Publication visibility | PostgreSQL catalog transaction (ADR-0112, supersedes ADR-0091) | Expected parent, never rebased; attempt identity unique | Typed `Conflict`; settlement by catalog query | Delta members immutable; export manifest derived |
| Attempt lifecycle | Pure transition table in `pse-operations` (ADR-0112) | Planned/queued/running/completed/partial/failed/cancelled/stale distinguishable | Transactional repository; state-domain `CHECK`; append-only transitions | Published `runtime.computation_runs` snapshot |
| Operational relation meaning | Registry (D1) | SQL migrations are physical representation only | Migration-conformance test | Enums as text, parsed at the boundary |
| Python settings and names | pse-owned `Settings` types; registry enums (ADR-0113) | No library type or `Debug` string in a contract | Strict structuring; unknown versions refused | Generated stubs and contracts |

**Physical-semantics table (solver-facing quantities the target adds).**

| Quantity or model element | Dimension and unit | Basis | Reference state / convention | Validity envelope | Authority |
|---|---|---|---|---|---|
| Integer and binary decisions | Dimensionless; quantity kind of count or indicator category | — | — | Finite integral box; binary [0, 1] | ADR-0103 |
| Semi-continuous variable | The variable's physical quantity | As declared | Zero branch exact | Active interval [lb, ub], 0 < lb | ADR-0103 |
| Global dual bound, gap | Objective unit | Original | Backend tolerances, box and export fidelity recorded | Exact export, or a relaxation for bounds only | ADR-0105, ADR-0106 |
| Parametric sensitivity dx/dp | Output unit per parameter unit | Original, via presolve and normalization back-maps | Fixed discrete assignment where applicable | SOSC, LICQ, strict complementarity, stable activity | ADR-0107 |
| Parameter covariance, intervals | Parameter unit² | Original, H_phys = S_f·S⁻¹·H_norm·S⁻¹ | Declared statistical model; GN label for transient | Qualified estimate, full response rank | ADR-0107 |
| Propagated output covariance | Output unit² | Original | Linearization at the qualified candidate | As covariance | ADR-0107 |
| Weighted multi-objective | Dimensionless after declared normalization | — | Sense per objective | Declared degradation tolerance per level | ADR-0111 |

**Well-posedness statement.** Variable roles, including decisions, are explicit declarations;
discrete variables count as degrees of freedom. Structural and DoF analysis run on the lowered
problem before routing, and diagnostics map derived rows and variables back to the authored
disjunction and alternative. Rejected before any solver runs: integer or semi variables without
finite bounds; free discrete variables in root, fitting and integrated modes; native-only
realizations on backends without the handler; `bigm(derived)` over FBBT-incomplete rows; hull
without finite bounds; `Certify` with an `Unavailable` objective; SPRAL without its OpenMP
settings; unsupported combinations of class, derivative order and bounds. Diagnostics name the
variable, row or alternative responsible.

## 4. Change scenarios and composition

The capability review's scenarios [S01–S09](design_review_solver-capabilities_2026-09-27.md#4-change-scenarios-and-composition)
and the architecture's [S10–S18](../../plans/22-solver-capabilities-architecture.md#1-target-drivers-and-scenarios)
are reused with their definitions unchanged; this slot records the target's response.

| Scenario / stimulus and conditions | Expected response and change boundary | Edit/composition path | Observed or predicted impact | Acceptance and evidence |
|---|---|---|---|---|
| [S01](design_review_solver-capabilities_2026-09-27.md#s01), [S18](../../plans/22-solver-capabilities-architecture.md#s18) Add SCIP (or any backend) | One adapter, one registry value, one capability record; no runtime workflow edit | Adapter table (A2) → capability record → routing reasons; `FactorableProgram` is a genuine new representation | Additive after A2. The new vocabulary is a legitimate core concept (ADR-0106) | *Proposed*; `stub_backend_routes_through_adapter_table` |
| [S06](design_review_solver-capabilities_2026-09-27.md#s06), [S10](../../plans/22-solver-capabilities-architecture.md#s10) Authored binary commitment (price-taker) | Domain facet → `CoefficientProblem` with integer domains → HiGHS; conditional duals | Grammar, registry, `grouped.rs`, routing | A new core concept touching authoring, registry and compiler once; no SCIP | *Proposed*; `authored_milp_routes_to_highs`, `fixed_lp_duals_conditional_on_commitment` |
| [S11](../../plans/22-solver-capabilities-architecture.md#s11) Synthesis over nonlinear alternatives | Disjunction lowered by declared realization → MINLP → SCIP; relaxed rows handled by the fixed-assignment re-solve | Kernel declarations → lowering → projection → SCIP → NLP runner | Relaxed-row semantics were unstated before T07 | *Proposed*; `small_synthesis_minlp_optimal`, `gdp_hull_and_bigm_same_optimum` |
| [S12](../../plans/22-solver-capabilities-architecture.md#s12) Certify a nonconvex design; phase stability | Explicit `certify`; `GapQualified` with box, tolerances, fidelity | Projection → SCIP → `quality.rs` | Assurance rigour stated (T10) | *Proposed*; `certify_known_global_optimum`, `tpd_detects_known_instability` |
| [S02](design_review_solver-capabilities_2026-09-27.md#s02), [S13](../../plans/22-solver-capabilities-architecture.md#s13) Covariance after a fit | Reduced Hessian with validity; withheld on failure | POUNCE sensitivity or `pounce-sens-core` + FERAL | GN label was missing for transient fits (T11) | *Proposed*; `unidentifiable_fit_withholds_covariance` |
| [S07](design_review_solver-capabilities_2026-09-27.md#s07), [S14](../../plans/22-solver-capabilities-architecture.md#s14) Edit → re-solve; NMPC | Value-only rebind; warm restart recorded; per-step records | Staged-sequence primitive; N2; store | Reuse identity separates compatibility from profile stamps (A4) | *Proposed*; `value_only_study_prepares_once`, `nmpc_closed_loop_on_antiwindup` |
| [S15](../../plans/22-solver-capabilities-architecture.md#s15) 10 000-point study across workers | `SKIP LOCKED` claims; isolated failures; one publication | Store queue → workers → staged primitive | Contradicted §25's "distributed execution" exclusion (T09) | *Proposed*; `study_parallel_workers_publish_once` |
| [S16](../../plans/22-solver-capabilities-architecture.md#s16) A long SCIP solve's process dies | Lease expiry → stale; new attempt re-injects stored incumbent; cancellation survives listener loss | Heartbeat, lease, incumbent stream | Cancellation was NOTIFY-only (T03) | *Proposed*; `killed_worker_attempt_goes_stale_and_resumes_from_incumbent` |
| [S17](../../plans/22-solver-capabilities-architecture.md#s17) Concurrent publication, local or remote | Catalog CAS; loser re-prepares; lost acknowledgement settled by query; readers protected without an open session | Members to Delta, then one catalog transaction | Reader protection contradicted itself (T02) | *Proposed*; `concurrent_publishers_one_winner_no_lost_update` |
| Replace an implementation: switch Ipopt from MUMPS to SPRAL on one case | A typed setting; recorded in the profile key; no consumer change | Ipopt adapter settings | Process-wide BLAS/OpenMP consequences were unstated (T04, T05); default ordering drift (T06) | *Proposed*; N1 tests |
| Test admission or policy locally ([S08](design_review_solver-capabilities_2026-09-27.md#s08)) | Routing, lowering, projection and lifecycle legality without native or database start-up | Pure owners | Lifecycle legality needed a pure owner (T13) | *Proposed* |
| Compose a new workflow: lexicographic NLP | Stages over the staged primitive; bound transformation; per-objective records | A6 + NLP runner | Tolerance and normalization rules were missing (T12) | *Proposed*; `lexicographic_nlp_staged_matches_weighted_limit` |
| Infeasible problem | Structural refusal before solving; ℓ1 least-infeasible point (`diagnostic_only`); SCIP IIS as a certified route | N3, G5 | Explicit routes, no fallback | *Proposed*; `l1_route_returns_labelled_least_infeasible_point`, `nonlinear_iis_irreducible_flag` |
| Dynamic start and event ([S09](design_review_solver-capabilities_2026-09-27.md#s09)) | IDAS scheduled changes with recoverable trials and sensitivity re-initialization | Y1 | Within the fixed-mass index-1 profile | *Proposed*; PID fixture |
| Boundary round trip (Python settings) ([S04](design_review_solver-capabilities_2026-09-27.md#s04), [S05](design_review_solver-capabilities_2026-09-27.md#s05)) | pse-owned settings; registry names; library upgrade does not change the contract | A5 | Library types removed from contracts | *Proposed*; `test_solve_settings_backend_projection` |
| Add a unit or property model | Unchanged by this target, except that discrete and disjunctive declarations are now available to packages | Package data | No new owner | — |

## 5. Mechanisms and execution

| Stage / owner | Formulation policy | Derivative source and order | Scaling | Problem class · solver capability used | Status → outcome mapping | Tolerances · post-solve check |
|---|---|---|---|---|---|---|
| Lowering (compiler) | Declared realization per construct; ε-perspective and `smooth(ε)` approximations stated | Symbolica on lowered rows | Normalization after lowering | Produces MILP/MIQP/MINLP rows | Refusals are typed preparation errors | Cross-realization agreement on bounded fixtures |
| Factorable projection (`pse-math`) | Obligations as bounds; providers as `Aux`; fidelity per row and objective | Not used by SCIP (it differentiates its own expressions) | Original coordinates | Export for SCIP | `Unavailable` objective refuses `certify` | Projection versus evaluator; relaxed enclosure control |
| SCIP solve (adapter) | Exact or relaxed export; native indicator/SOS/logic | SCIP internal | SCIP's own | MIQP, MINLP, certify, IIS, exact MILP | Raw `SCIPgetStatus` exhaustive | Candidate re-qualified; relaxed → bound only; gap records both sources |
| Fixed-assignment re-solve (NLP runner) | Discrete assignment as an overlay | Exact Hessian | §16.1 normalization | Smooth NLP | As NLP | Original-space quality; conditional duals |
| Ipopt with typed linear solver | As NLP | Exact or limited memory | As NLP | Smooth NLP; MUMPS+METIS, SPRAL or MKL Pardiso | Unchanged, plus refusal on unmet preconditions | Numerical, not bitwise, agreement across solvers |
| Sensitivity (POUNCE/`pounce-sens-core`) | At a qualified candidate | Exact second order | Back-maps to physical | KKT with inertia | Withheld with reasons | Validity record |
| ℓ1 route (POUNCE) | Exact penalty; explicit | Exact | As NLP | Smooth NLP | Least-infeasible → `diagnostic_only` | Original-space quality |
| Lexicographic stages (A6) | Objective bounds with declared tolerance | As the stage | As the stage | LP/MILP native; others staged | Per stage | Final candidate qualified |
| Adjoint and second-order dynamics | Fixed-mass index-1 | Adjoint (Diffsol, IDAS); forward-over-adjoint | State/residual scales | ODE/DAE | Typed IDA terminations | FD comparison in tests |

| Stage / owner | Contract and mechanism | Inputs / dependencies | Effects and lifecycle | Reuse / equivalence / limits | Evidence or uncertainty |
|---|---|---|---|---|---|
| Catalog commit (`pse-operations`) | One transaction: expected-parent check, members, head advance | Written member versions; attempt identity | Visibility; typed `Conflict` | Idempotent per attempt | *Interface-checked* (PostgreSQL transaction semantics) |
| Reader lease and retention | Lease rows with expiry; two-phase deletion; xact advisory locks for maintainers | Catalog | Deletes member files only after leases drain | Remote stores qualifiable | *Proposed* |
| Job queue and cancellation | `SKIP LOCKED` claims; heartbeat leases; `cancel_requested` authority; `NOTIFY` for latency | Versioned payloads; bundles | Attempts per try | Retry policy explicit | *Interface-checked* (PG docs) |

## 6. Architectural assessment and gates

| Foundation | Scenario and evidence / scope reason | Verdict | Required action |
|---|---|---|---|
| AP-01 Separation of concerns | Authoring, lowering, projection, routing, adapters, runners, store and boundary each own one reason for change (slot 2). The catalog moves publication visibility into `pse-operations` while `pse-catalog` keeps member I/O; retention computation needed an owner (T16, corrected) | **satisfied** | — |
| AP-02 Stable contracts | `FactorableProgram`, pse-owned `Settings`, registry vocabulary and sqlx types confined to `pse-operations`; no SCIP, Clarabel, Diffsol or sqlx type crosses its owner (S04, S05) | **satisfied** | — |
| AP-03 Composition | S18 is additive once A2 exists; one NLP runner and one staged primitive compose solve, initialization, fitting, certification re-solves, lexicographic stages and NMPC; the ℓ1 realization composes with routing after T14 | **satisfied** | — |
| AP-04 Authoritative meaning | One authority per fact across stores (§9.2); corrected competing vocabularies (T01), undecided domain authority (T15), lifecycle legality and identity minting (T13) | **satisfied** after corrections | — |
| AP-05 Explicit structure and constraints | Finite-box admission; explicit intents; threads admitted; process-level numerical environment explicit (T04, T05); cancellation authority explicit (T03) | **satisfied** after corrections | — |
| AP-06 Local reasoning / testability | Routing, lowering, projection and lifecycle legality are pure; `Ephemeral` class keeps library use database-free; `#[sqlx::test]` isolates real databases; a stub adapter tests S18 | **satisfied** | — |

| Gate | Result | Evidence or scope reason | Required action |
|---|---|---|---|
| G1 Authority | **pass** (after T01, T13, T15, T16) | One `CandidateUse`; one domain enum; one lifecycle table; catalog as sole visibility authority | — |
| G2 Semantic fidelity | **pass** | Domains, intents, assurances and reasons are typed; enums as text parsed at the boundary; no `Debug` strings (ADR-0113) | — |
| G3 Validity | **pass** | Admission refusals listed in the well-posedness statement | — |
| G4 Hidden behaviour | **pass** (after T08, T14) | SCIP's nested Ipopt and the ℓ1 route are explicit and recorded; POUNCE ℓ1 fallbacks reserved | — |
| G5 Consistency and recovery | **pass** (after T02, T03) | Catalog CAS; stale attempts; reader leases; durable cancellation authority | — |
| G6 Transformation and reuse | **pass** (after T04, T06, T15) | Lowering equivalence declared; ordering and CBWR explicit and in the profile key | — |
| G7 Truthful capability claims | **pass** (after T07, T09, T10) | Relaxed-export semantics; tolerance-qualified assurances; §25 amended | — |
| G8 Library leverage | **pass** | SCIP, POUNCE sensitivity/ℓ1/convex, HiGHS lexicographic, PostgreSQL primitives; bespoke parts justified (slot 8) | — |
| G9 Architectural fitness | **pass** | All six foundations satisfied | — |
| PS-G1 Physical consistency | **pass** for the solver-facing quantities in slot 3; property physics not assessed | Count/indicator typing; physical back-maps; normalized weighted objectives (T12) | — |
| PS-G2 Well-posedness | **pass** | Discrete DoF; analysis on lowered problems mapped back to alternatives; finite-box admission | — |
| PS-G3 Numerical integrity | **pass** (after T07, T11, T15) | Re-qualification of every candidate; GN label; derived big-M completeness; ε-perspective stated | — |

## 7. Findings

All findings concern the target design text as first drafted. Each was corrected in the ADRs or
the architecture companion before acceptance; slot 11 records where. They remain here as
observations.

| ID | Finding | Principles / gate / scenario | Evidence or gap | Consequence | Correction | Verification |
|---|---|---|---|---|---|---|
| <a id="t01"></a>T01 | Two `CandidateUse` vocabularies: architecture §3 named Accept, SeedOnly, Diagnostic and Unusable, while §16.6 already owns Usable, QualifiedUnclosed and Unusable | DP-01, AP-04, PS-10 · G1 | Architecture §3 as drafted; generated `CandidateUse` in `pse-model` | A1 would introduce a second decision beside the one F13 asks to unify, and drop `qualified_unclosed` | Extend the existing enum: `usable`, `qualified_unclosed`, `seed_only`, `diagnostic_only`, `unusable`, one owner (ADR-0106 §11) | One enum in the registry; A1 test across all workflows |
| <a id="t02"></a>T02 | Reader protection by shared advisory locks contradicts "no reader holds the control plane open" | DP-19, DP-20, AP-05 · G5 · S17 | Architecture §9.5 items 3–4 as drafted; PostgreSQL 18: session-scoped locks live until the session ends, transaction-scoped locks until commit | Either every reader pins a database session for the whole Delta read (pool exhaustion, no offline or remote readers), or maintenance can delete files under a reader | Lease rows with expiry; two-phase deletion; transaction-scoped locks only between maintainers (ADR-0112 §7; architecture §9.5) | `maintenance_waits_for_reader_leases` |
| <a id="t03"></a>T03 | Cancellation relied on `NOTIFY`, which is not durable | DP-19 · G5 · S16 | Architecture §9.4 as drafted; PostgreSQL 18 `LISTEN` semantics (registration at commit; listen-then-inspect rule) | A cancel issued while a worker's listener reconnects is lost | `cancel_requested` is the authority returned by the heartbeat; `NOTIFY` for latency only; re-read state after every `LISTEN` and every `PgListener::try_recv` report of a lost connection (ADR-0112 §15; architecture §9.4) | `cancel_notify_stops_running_job` plus a dropped-listener case |
| <a id="t04"></a>T04 | The MKL determinism contract covered only Pardiso, but `pardisomkl` requires MKL as Ipopt's BLAS/LAPACK, so it governs every Ipopt route; the process's BLAS and OpenMP providers were unstated | DP-11, DP-20, AP-05 · G6 | Ipopt 3.14.20 `doc/install.dox` ("Pardiso from Intel MKL"); oneMKL CNR needs a constant thread count; netlib BLAS used elsewhere today | Silent change of MUMPS arithmetic; possible symbol interposition between two BLAS providers; two OpenMP runtimes (libiomp5 and libgomp) in one process | One oneMKL (LP64, GNU threading) as the single BLAS/LAPACK; libgomp as the single OpenMP runtime; CBWR pinned and checked; profile key (ADR-0108 §5, §14; architecture §5.4) | `single_blas_provider_in_process`; `ipopt_pardisomkl_selectable_under_cbwr` |
| <a id="t05"></a>T05 | SPRAL's process-level OpenMP requirements (`OMP_CANCELLATION=TRUE`, `OMP_PROC_BIND=TRUE`) were not part of admission | DP-15, DP-20 · G3, G7 | Ipopt 3.14.20 `doc/install.dox` (SPRAL section) | In a Python host without them, SPRAL runs outside its supported configuration or misbehaves; the capability is claimed without its precondition | Image and `pse-worker` set them; admission checks `omp_get_cancellation()` and bind policy and refuses (ADR-0108 §13) | `spral_refused_without_omp_cancellation` |
| <a id="t06"></a>T06 | Linking METIS silently changes the default MUMPS ordering for every existing Ipopt solve | DP-11, PS-11 · G6 | `IpMumpsSolverInterface.cpp`: `mumps_pivot_order` default 7, automatic | Results and iteration counts shift with no recorded cause | Explicit `ordering` in typed settings with an explicit default (`metis`), recorded in the profile key (ADR-0108 §10) | `ipopt_mumps_metis_ordering_selectable`; `ipopt_linear_solver_in_profile_key` |
| <a id="t07"></a>T07 | What an automatically routed MINLP with relaxed rows produces was unstated | PS-10, PS-12 · G7, PS-G3 · S11 | Architecture §3 routes MINLP automatically to SCIP; §5.2 forbids solution claims from relaxed exports | A relaxation's point could be read as a solution, or every rigorous-thermodynamics MINLP would be refused without saying so | Relaxed incumbent is an assignment proposal (`diagnostic_only`); the candidate comes from the fixed-assignment re-solve; a gap combines the relaxation bound with that qualified candidate (ADR-0105 §2; architecture §5.2) | `relaxed_export_bound_only`; `small_synthesis_minlp_optimal` |
| <a id="t08"></a>T08 | SCIP's nested Ipopt escaped the typed linear-solver and thread settings | DP-20 · G4 | SCIP 10 `nlpi_ipopt.cpp` parameters `nlpi/ipopt/linear_solver`, `hsllib`, `pardisolib` | NLP subsolves inside SCIP could use an unadmitted threaded solver or attempt HSL, unrecorded | Set from the typed Ipopt setting; HSL and Pardiso-project paths refused; serial unless admitted; `TPI=tny` (ADR-0105 §5, §10; architecture §5.3) | SCIP adapter reserved-option test |
| <a id="t09"></a>T09 | §25 places "distributed execution" outside the design target, which S15 contradicts | G7 · S15 | §25 recorded limits | The authority would refuse the worker model the target needs, or the target would claim what §25 excludes | Durable multi-process execution through the store is in the target; distributing one solve is not (ADR-0102 §7, ADR-0112 §19; §25 amended) | Authority text consistent |
| <a id="t10"></a>T10 | `global_bound` and `proven_infeasible` read as proofs, but SCIP's bounds hold within floating-point tolerances | PS-12, DP-22 · G7 | SCIP guarantees for bounded problems within its numerical tolerance (capability review §8.1) | Users would read a tolerance-qualified conclusion as a rigorous proof | Define both as tolerance-qualified with conditions recorded; `exact_certificate` is the only rigorous assurance (ADR-0106 §9; architecture §3) | Registry documentation; G4/G5 tests record tolerances |
| <a id="t11"></a>T11 | Gauss–Newton covariance for transient fits lacked its validity statement | PS-12 · PS-G3 · S13 | Architecture §6.4 as drafted | A curvature-neglecting approximation would be reported like an exact covariance | Record `approximation = gauss_newton` with its conditions; use the exact route when available (ADR-0107 §6) | `linear_regression_covariance_analytic`; relation column present |
| <a id="t12"></a>T12 | Multi-objective lacked degradation tolerances and a normalization rule | PS-01, PS-04, DP-11 · PS-G1 | Architecture §8 as drafted | Zero-tolerance objective bounds break constraint qualification on staged NLP levels; dimensional weighted sums add incompatible quantities | Declared absolute/relative tolerances per level; dimensionless or explicitly normalized weighted members (ADR-0111; architecture §8) | C3 tests and refusal controls |
| <a id="t13"></a>T13 | Lifecycle legality was said to be "backed by `CHECK` constraints", which cannot express transitions, and attempt identity had two minting points (`uuidv7()` defaults and the runtime's `SemanticId`) | DP-01, DP-03, DP-04, AP-06 · G1 | Architecture §9.3–§9.4 as drafted; UUIDv7 `SemanticId` minting in current source | Two partial rule sets and two identities for one attempt; legality not testable without a database | One pure transition table; database enforces value domain and append-only history; runtime mints identities (ADR-0112 §12–§13; architecture §9.3–§9.4) | `illegal_transition_rejected`; pure transition-table tests |
| <a id="t14"></a>T14 | An authored `penalty(l1)` realization conflicted with "ℓ1 is never automatic", and mixing realizations in one solve was undefined | PS-06, PS-09 · G4 | Architecture §2.5 and §6.3 as drafted | Either the realization is unusable, or the rule is broken silently | An authored realization is the explicit selection; mixing admitted with the whole-problem penalty stated (ADR-0104 §5, ADR-0109 §2; architecture §2.5) | `flash_phase_disappearance_agrees_across_realizations`; `l1_never_automatic` |
| <a id="t15"></a>T15 | Lowering equivalence and domain authority were under-specified: derived big-M without FBBT-completeness or rounding rules; nonlinear hull without the perspective approximation; the domain enum "becomes that type, or a checked 1:1 mapping" | DP-01, DP-08, PS-06 · G6, PS-G3 | Architecture §2.1, §2.4 as drafted | A default or roundoff-tight M could cut feasible points; a nonsmooth perspective at λ=0; two enums that could drift | Completeness required, outward widening, refusal naming the row; ε-perspective stated; generated enum consumed directly (ADR-0103 §2, ADR-0104 §3) | `bigm_derived_from_bounds`; `gdp_hull_and_bigm_same_optimum` |
| <a id="t16"></a>T16 | No owner for protected-version computation once the Delta control relations are deleted | AP-01, DP-01 · G1 | §20.4 computes retained versions by a DataFusion query over control records | Retention would silently lose its input, or keep a second copy of catalog facts in Delta | The catalog computes protected versions; `pse-catalog` executes deletion as instructed (ADR-0112 §7; architecture §9.5) | O8 retention tests |

**Strengths that bear on the argument.** The target keeps every current guarantee the
capability review praised: pessimistic status mapping, independent original-coordinate
qualification, readback before claims transfer, refusal before execution. The relaxation
soundness rule and the `Certify` intent keep global claims honest. The division "PostgreSQL owns
what changes; Delta owns what is published" gives each fact one authority, and the `Ephemeral`
class keeps the pure core free of a database.

## 8. Library fit and ownership cost

| Capability / contract | Integration owner / exposed types | Candidate or current mechanism | Fit and limits | Coupling, lifecycle, test, upgrade/replacement cost | Bespoke code removed / recommendation |
|---|---|---|---|---|---|
| MINLP, MIQP, global certification, IIS | `pse-backend-native::scip`; no SCIP type exported | SCIP 10.0.2 via raw `scip-sys` | The only credible open-source nonconvex MINLP route; globally optimal at small scale; tolerance-based bounds | Image build; FFI lifecycle on the owning worker; ABI check; exhaustive status map | Removes any need for bespoke OA or enumeration. **Adopt** |
| Relational operational state | `pse-operations`; sqlx types internal | PostgreSQL 18 + sqlx =0.9.0 (runtime-typed `query_as`/`FromRow`, `migrate!`, `PgPool`, `PgListener`, `PgAdvisoryLock`, `UNNEST` batches, SQLSTATE mapping) | Transactions, `SKIP LOCKED`, advisory locks, `LISTEN/NOTIFY`, `uuidv7()`; one crate covers pool, migrations, listener, TLS and a per-test database harness | A server to operate; `#[sqlx::test]` isolated databases; migrations versioned. The ban on `query!` is a convention held by review (no new alignment lint), since the `macros` feature is needed for `FromRow` and `migrate!` | Removes the Delta control table, file leases and in-memory-only run state. **Adopt** |
| Alternative PostgreSQL client stacks | — | tokio-postgres + deadpool-postgres + refinery + tokio-postgres-rustls; Cornucopia/clorinde; SeaQuery; Diesel/SeaORM; pgvector; pgrx; testcontainers | Four seams where one serves; generation against a live database; dynamic SQL owned by DataFusion; ORMs; out-of-scope data or server-side code | More integration surface or a new generated path (DP-16) | **Reject**, each with its revisit trigger in ADR-0112 |
| Large symmetric-indefinite KKT | Ipopt adapter; typed settings | SPRAL SSIDS; oneMKL Pardiso; MUMPS+METIS | Inertia-reporting; different pivoting and parallelism; performance *Proposed* | Process-wide BLAS/OpenMP/CBWR policy; larger image | Replaces HSL. **Adopt**, value measured in Q1 |
| NLP parametric sensitivity | Adapter + runtime fitting | POUNCE `sensitivity`; `pounce-sens-core` + FERAL | sIPOPT semantics with inertia; `pounce-sensitivity` API *Proposed* | Presolve policy constraint; back-maps | Removes Plan 20 §6's bespoke faer KKT LU. **Adopt** |
| Whole-model infeasibility explanation; MPCC penalty | POUNCE adapter | `pounce-l1penalty` (linked) | Exact penalty; least-infeasible otherwise | Typed method; reserved fallbacks | Removes the bespoke whole-model elastic formulation. **Adopt** |
| Batched QP, QP sensitivity, SOS bounds | POUNCE-convex adapter | `pounce-rs` feature `convex` | Overlaps HiGHS/Clarabel; SOS bound non-rigorous | One adapter; explicit only | **Adopt** as explicit alternative |
| Lexicographic LP/MILP | HiGHS adapter | `Highs_passLinearObjectives` (1.15) | Native priorities, weights, tolerances | Pin bump (C1) | Avoids staged solves for linear classes. **Adopt** |
| Adjoint and second-order dynamics | Dynamics adapters | Diffsol adjoint with checkpointing; IDAS adjoint and forward-over-adjoint | Library-owned | Operator adjoint trait impls | **Adopt** |
| DataFusion access to operational tables | `pse-operations` provider | ADBC PostgreSQL driver (`adbc_core`/`adbc_driver_manager` 0.24) or typed row-to-Arrow builders, chosen at O9; `datafusion-table-providers` 0.13.1 blocked (datafusion ^54, arrow ^58, second driver stack) | ADBC fits arrow =59.3 and avoids bespoke type mapping | `just family-check`; ADBC driver packaging to be assessed at O9 | **Decide at O9**; table providers **blocked** until a release matches `datafusion =55.1.0` |
| GDP reformulation | Compiler transformations | No Rust library reformulates this IR; Pyomo.GDP excluded (D12) | Domain-model lowering, not solver machinery | Bespoke, bounded, tested by cross-realization agreement | **Bespoke, justified** |
| Ipopt-route KKT replica for sensitivity | Runtime + FERAL | Ipopt's C API exposes no factorization | Required for the Ipopt route | Bespoke assembly only; factorization library-owned | **Bespoke, justified** |

## 9. Alternatives and tradeoffs

| Alternative | Scenarios served / change locality | Contracts, composition and test isolation | Meaning or machinery carried | Correctness / operational cost | Selection and revisit condition |
|---|---|---|---|---|---|
| Current baseline | None of S10–S17 | Unchanged | — | Honest limits (§25) | Rejected by the maintainer's direction |
| Proposed design (ADR-0102–ADR-0113) | S10–S18 through two seams (A2, A6) and one new store | Pure owners for admission, lowering, projection and legality | New vocabulary, representation and store: legitimate core concepts | Image and server to operate; small-scale global claims | **Selected**; revisit triggers in each ADR |
| Library-owned alternative: Couenne/Bonmin; sIPOPT | MINLP; sensitivity | NL/ASL or C++ bridges | Duplicate owners | Heuristic or D12-excluded | Rejected |
| Simplest viable: HiGHS MILP only; in-memory runs; Delta-only persistence | S10 only | Minimal | — | Fails S11–S17 | Rejected |
| Delta control table plus a PostgreSQL run registry | S15, S16 | Two visibility authorities | Duplicate publication records | Multi-writer still unqualified | Rejected (DP-01, R-10) |
| SQL-first generated client (tokio-postgres + Cornucopia + refinery), per the maintainer's external review | S15–S17 equally | Typed generated query interfaces; tests need containers or bespoke per-test databases | A generated path validated against a live database; four integration seams | Pipelining and binary `COPY` unneeded at this volume | Rejected in favour of sqlx runtime-typed queries (ADR-0112); revisit on a measured throughput bottleneck or SQL/schema drift escaping `#[sqlx::test]` |
| MKL only for Pardiso, netlib elsewhere | Pardiso | Impossible within one Ipopt library | Two BLAS providers | Symbol interposition risk | Rejected (T04) |

**Reference practice** (behaviour only). gPROMS offers local MINLP by outer approximation;
Aspen Plus flowsheet optimization is NLP only; IDAES users reach GDP and MINLP through Pyomo
(GDPopt, MindtPy, SCIP, BARON), and idaes-ext builds Ipopt with METIS and, when licensed, HSL.
Operational stores backed by a relational database with leased job queues are standard in
workflow systems; the design's distinguishing choice is keeping published scientific data
immutable in Delta.

## 10. Verification

| Claim / scenario / risk | Evidence label | Reasoning, test or measurement | Conditions and expected result | Result or gap |
|---|---|---|---|---|
| The target is architecturally fit (G9) | Proposed (design argument) | Slots 2, 4 and 6 of this review | Representative changes stay within their owners | Accepted as a design; implementation open |
| `pardisomkl` requires MKL as Ipopt's LAPACK; SPRAL needs OpenMP settings; MUMPS default ordering is automatic | Interface-checked | Ipopt 3.14.20 corpus files named in slot 1 | — | Consistent with ADR-0108 |
| idaes-ext builds METIS; ADR-0028's premise was wrong | Interface-checked | idaes-ext 3.4.2 `scripts/compile_solvers.sh` | — | Recorded in ADR-0108 Context |
| PostgreSQL features relied on | Interface-checked | PostgreSQL 18 documentation | — | Consistent with ADR-0112 |
| SCIP entry points and exact-mode precondition | Interface-checked | Capability review §8.1; SCIP documentation | — | Consistent with ADR-0105 |
| `pounce-sensitivity` 0.12.0 fits | Proposed | Plan 22 S1 verifies first; declared alternative `pounce-sens-core` | API composes with FERAL and back-maps | Open (S1) |
| SCIP certifies the named fixtures | Proposed | Q1 gap-closure measurement | Recorded budget | Open (Q1) |
| SPRAL or Pardiso beats MUMPS+METIS on large KKT | Proposed | Q1 benchmark, n_KKT ≥ 10⁴ | Recorded conditions | Open (Q1) |
| Each scenario's behaviour | Proposed | The Plan 22 tests named in each ADR's `verification:` | Zero-baseline targeted runs per packet | Open (packets) |

No test or probe was executed for this review. `just adr-lint` and `just adr-index` were run for
the D0 change; they establish record syntax, supersession symmetry and section citations, not
architecture.

## 11. Authority changes, exceptions and disposition

| Authority text | Conflict | Required change | Route |
|---|---|---|---|
| §25 "global MINLP … not part of the design target"; "distributed execution"; "unproved global stability"; covariance refusal; multi-writer/remote limits | Blocks S10–S17 | Amended with `> Decision:` markers (ADR-0102, ADR-0107, ADR-0112) | `design:` edit under `PSE_DESIGN_EDIT=1` in D0; blueprint revision row |
| §18.9 "Outside the matrix" | Global MINLP, disjunctive programs | Amended (ADR-0102, ADR-0105, ADR-0109, ADR-0110) | Same |
| §19.7 Optionality; §19.8 Uncertainty; §19.4 fitting limits | Refuse the target constructs | Target pointers (ADR-0102, ADR-0104, ADR-0107) | Same |
| §3.3 "general global MINLP are not admitted" | Blocks SCIP | Amended (ADR-0102, ADR-0105) | Same |
| D10 "Delta owns publication" | Catalog split | Amended (ADR-0112) | Same |
| ADR-0028, ADR-0090, ADR-0091, ADR-0016 | Made untrue by the target | Superseded by ADR-0108, ADR-0106, ADR-0112, ADR-0112 | `just adr-supersede` |
| ADR-0083, ADR-0084, ADR-0093 | Revisit triggers fire | Extended within, not superseded (ADR-0105, ADR-0110) | Recorded in those ADRs |
| ADR-0101 (proposed) objective clause | Competing objectives refused | Refined by ADR-0111; noted in ADR-0101's status history | ADR |
| Register R-10 | Decided by ADR-0112 | Row removed; R-08, R-09, R-21 move to ADR-0108; R-34 (HSL) added | Register |
| Plan 20 §4–§6 | Bespoke KKT LU; bespoke elastic policy; continuous-only price-taker and MatOpt | Amended in D0 | Living plan |
| Plan 22 Q1 and *Verification* still name HSL; O8 test name `maintenance_excludes_readers_via_advisory_lock` | Stale after ADR-0108 and T02 | Replace HSL by SPRAL/MKL Pardiso in Q1 and Verification; rename the O8 test `maintenance_waits_for_reader_leases` | Plan 22 owner (outside the D0 track's paths) |

No SHOULD deviation or MUST gap is recorded.

**Disposition.** [Plan 22](../../plans/22-solver-capabilities.md#finding-dispositions) owns the
status of T01–T16. Their design corrections landed in D0 (the ADR sections and architecture
sections cited in slot 7); implementation evidence is owned by the packets named in each
finding's verification column. Recommended rows: T01 → A1; T02, T03, T13, T16 → O3–O8; T04–T06 →
N1; T07, T08, T10 → G3–G7; T09 → D0 (authority text, done); T11 → S3; T12 → C3; T14 → M5, N3;
T15 → M1, M4.

## 12. Decision

<a id="decision"></a>

**Behavioural and semantic adequacy.** Adequate at the *Proposed* level. Every gate, including
PS-G1 (for solver-facing quantities), PS-G2 and PS-G3, passes after the T01–T16 corrections.
The design never reads a solver status as success, re-qualifies every candidate in original
coordinates, states the conditions of every derived quantity and assurance, and refuses
unsupported work before execution.

**Architectural fitness.** G9 passes; all six foundations are satisfied. A new backend is
additive through the adapter table; the new concepts (discrete domains, disjunctions,
factorable projection, global assurance, operational store) each have one owner and a pure
test surface.

**Overall.** **Accept** the Plan 22 target design at the *Proposed* evidence level, within this
scope: the classes, intents and stores of ADR-0102–ADR-0113, excluding HSL, GPU execution,
single-solve distribution, higher-index DAEs and interval-rigorous global optimization. The
strongest evidence is the source-level interface checks in slot 10; the largest uncertainties
are the unmeasured value of SCIP certification and of SPRAL/Pardiso on large KKT systems, and
the unverified `pounce-sensitivity` API. Acceptance closes no implementation work.

| Priority | Change | Findings / scenarios | Acceptance evidence | Disposition owner |
|---|---|---|---|---|
| P1 | Seams first: backend-execution adapter, one NLP runner, one `CandidateUse`, staged-sequence primitive | F13, F27–F29, T01 · S18, S14 | A1, A2, A6 tests | Plan 22 |
| P2 | Operational store and catalog with reader leases and durable cancellation | T02, T03, T09, T13, T16 · S15–S17 | O1–O9 tests | Plan 22 |
| P3 | Solver image and Ipopt linear algebra with the process-wide contract | T04–T06 | N1 tests; Q1 measurement | Plan 22 (T3 track) |
| P4 | Discrete domains, constraint forms and lowerings | T14, T15 · S10, S11 | M1–M5 tests | Plan 22 |
| P5 | SCIP projection and adapter with relaxed-export semantics | T07, T08, T10 · S01, S12 | G1–G8 tests; Q1 gap closure | Plan 22 |
| P6 | Sensitivity, covariance, multi-objective, dynamics extensions | T11, T12 · S02, S09, S13 | S1–S4, C3, Y1–Y5 tests | Plan 22 |

**What would falsify these conclusions.** SCIP failing to close gaps on the certify fixtures
within budget; `pounce-sens-core` failing to compose with FERAL and the back-maps; a measured
BLAS or OpenMP conflict that the single-provider rule cannot prevent; event volume beyond what
batched inserts sustain; a representative new backend still requiring runtime workflow edits
after A2.
