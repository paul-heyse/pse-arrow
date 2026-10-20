---
title: End-to-end execution performance — measured diagnosis and best-in-class target
date: 2026-10-20
tier: design
purpose: target
decision: Revise
doc_role: review
doc_topics: [design, performance, preparation, numerical-execution, workflows]
standard:
  core: "3.4"
  efficient-architecture: "1.0"
  process-simulator: "1.5"
---

# End-to-end execution performance — measured diagnosis and best-in-class target

## 1. Decision and short version

**Decision: Revise.**

- **Architectural fitness fails (G9).** AP-07 is violated for the functional target's own workloads. AP-02 and AP-03 are violated narrowly.
- **Behavioral adequacy is not accepted in scope.**
  - G2 fails for two native stopping contracts.
  - G7 fails for start receipts.
  - PS-G3 is unresolved. A 64-unknown root ended on KINSOL's step-size safeguard and was refused. The eight coordinates the harness reports are within 4.1e-6 of the analytic answer, but the full candidate's quality is unknown.
- **No implemented speedup is claimed.** The scope accepted is the target design and the diagnosis below.

### What the measurements say

All runs re-executed the campaign's own cases, binaries and conditions with diagnostics off. Details are in the [measurement evidence](../evidence/end-to-end-execution-performance-2026-10-20/measurements.md).

**Small problems (2–16 unknowns)** across KINSOL, Ipopt, POUNCE, IDAS and recycles finish in about 1.7–1.8 s per process.

- **Registry: about 0.94 s, roughly 53% of CPU.** The schema registry is reassembled and fingerprinted in every process.
- **Authored-document admission: about 0.5–0.7 s per package, about 40% of CPU.**
- **Workflow invocation, including the native solve: about 11 ms, ≤ 3% of CPU.**
- **libc: 54% of cycles in `compact-root-banded-4`.** This is self time in allocation, copying and comparison (*Measured*: gdb timing, lldb stack sampling, perf DSO self time).

**Size growth.** The same analytic root model was swept from N = 16 to 256 unknowns.

- Process time is 2.95 s at N = 16 and 2.25 s at N = 32, then rises to 53.7 s at N = 256.
- Above a ≈2.1 s fixed baseline, growth follows **≈N^2.5–3.0 over N = 64–256**. The source confirms the cubic structure (P01).
- At N = 256, **94% of CPU is modeling admission**: lexical name/type environment construction in `pse_modeling::check`.
- Native work is 67 ms, which is 0.13% of the run.

**Reference property packages** (Peng–Robinson inline/nested, PC-SAFT) show 36–41 s `modeling.prepare` spans in the campaign's diagnostics-on timelines. About 2 s of each is math/publish. A one-block heater spent 58 s in source admission and never reached its solver.

**Long campaign cases.** The 35–59 s fitting, shooting, QP and mixed-scale cases spend only 1.2–6.4 s inside their workflow invocation, of which native work is ≤ 1.3 s. The remaining 30–55 s lies before the invocation, in preparation and admission (§3.1).

**Durable studies** spend about 1.2 s of coordination per trivial point.

### What this means

The binding constraint today is not the numerical engine.

- **Preparation work is not proportional to change.**
  - It is repeated in every process.
  - It is repeated for every package.
  - It is repeated for every declaration.
- **Preparation work is not proportional to size.** Modeling admission is cubic.

By extrapolation of the measured exponent, a model of about 1,000 unknowns of the same shape would spend roughly 50 minutes in admission. A 5,000-unknown plant would need days. This is a *Hypothesis* by extrapolation, not a measurement.

The edit → re-solve loop, studies and recycles named in the functional target cannot reach industrial size until that is fixed.

### On the hypothesis that numerical methods are the main lever

The evidence ranks numerical methods third. Ahead of them come:

1. modeling admission and its reuse;
2. fixed per-process and per-package preparation.

Numerical work becomes the leading term only after those walls are removed and models reach flowsheet scale. The review still sets a full numerical target (§4.4, P09–P17), for two reasons:

- the target needs it;
- several numerical findings are correctness-relevant and should be fixed regardless:
  - KINSOL's step semantics;
  - cone feasibility translation;
  - primal warm starts that are silently treated as cold.

### Target in one paragraph

**Compile once, persist, bind values, solve natively, publish off-path.**

- **Registry and modeling.**
  - A schema registry frozen at build time.
  - A modeling compiler that is linear-time, separately compiled per package, parallel, and content-addressed across processes.
- **Admission and preparation.**
  - Typed document admission that validates where trust changes rather than on every hop.
  - Prepared structures with indexed, coarsened execution groups.
  - Fused evaluators with sparse colored Jacobians, cached as compiled artifacts.
- **Native execution.**
  - Class-specific native solvers whose stopping rules match original-space acceptance.
  - Genuine warm starts, accelerated recycles, reused dynamic sessions and least-squares-aware fitting.
- **Coordination and results.**
  - One CPU budget with work stealing across cases, blocks, experiments and windows.
  - Results and diagnostics produced off the critical path, losslessly at phase boundaries.

## 2. Scope, baseline and evidence

| Field | Content |
|---|---|
| Subject and boundary | End-to-end problem execution, in order: process start and registry; authored source and document admission; modeling compilation; structural and mathematical preparation; native execution; original-space assessment; result retention and publication. Also covered: local studies, durable dispatch, dynamics, fitting and shooting. Excluded: SurrealDB internals (only dispatch and polling behavior is assessed), Python API ergonomics, GPU/distributed execution. |
| Standard | Core 3.4, Efficient Architecture Heuristics 1.0, Process Simulator 1.5, pse-arrow binding |
| Tier / purpose | Design / target |
| Reviewer / date | Claude (Anthropic), independent audit lane, 2026-10-20. Five focused read-only source reviews fed the principal reviewer: compiler, mathematics, native adapters, orchestration, dynamics/fitting. The principal reviewer re-read the decisive source facts and ran every measurement. |
| Decisions | Behavioral adequacy: not accepted in scope. Architectural fitness: fails. Overall: **Revise** (§1, §8) |
| Disposition owner | None yet. No plan has adopted these findings; plan creation selects the owner (binding). |

**Analysis modes and workloads.**

- **Analysis modes in scope:**
  - square simulation (roots, recycles);
  - optimization (NLP, LP/MILP, conic);
  - dynamics (index-1 DAE/ODE with sensitivities);
  - estimation (steady and dynamic fitting, shooting).
- **Workloads considered:** the profile's five, at the scales the campaign exercised (2–256 unknowns, 1–16 points, 1–8 batch members), plus a reasoned extension to flowsheet scale (10³–10⁴ equations, 10³–10⁴ study points, many cores).
- **Resource envelope:** one 16-core/32-thread workstation.

**Inputs.**

- **R1.** The external review [`docs/design_review_execution-performance-target_2026-10-20.md`](../../design_review_execution-performance-target_2026-10-20.md), untracked at review time. It is source-only; its author did not have the diagnostic bundle.
- **The campaign bundle** `build/external-pipeline-review-20261019/` and its [briefing](../../external-pipeline-performance-review-context.md).
- **Source** at `1ee81798a`. This is the campaign baseline `106aef5ec` plus a documentation-only commit.

**Evidence labels.**

- *Measured* claims name their run, conditions and tool in the evidence file. They are single observations on a non-quiesced workstation.
- *Implemented* claims were traced in source.
- *Interface-checked* claims were read against vendored SUNDIALS 7.1.1.
- Everything in §4 and the corrections are *Proposed*.

**What could not be re-run.** Canonical-store cases (`cold-small-1` and the PR/PC-SAFT preparations) were not re-run. The maintainer's test-resource registry requires a fenced migration recovery that this review did not perform. Those cases rely on the campaign's timelines. Their open compiler spans are not exclusive-time evidence, because phase ends can be dropped (P19). The CPU attribution of the same code path rests on the re-run `compact-property-value` and the banded sweep.

## 3. Where the time goes

### 3.1 Small problems are fixed-cost bound

| Case (plain re-run) | Wall | Registry | Document admission | Modeling compiler | Math + native |
|---|---:|---:|---:|---:|---:|
| `compact-root-banded-4` (KINSOL) | 1.72 s | 53% | 41% | 1% | 2% |
| `review-nlp-ipopt-2-cold-exact` | 1.77 s | 52% | 38% | 1% | <1% |
| `review-pounce-convex-8-1w` | 6.12 s | 14% | 79% | 1% | 3% |
| `compact-property-value` | 6.98 s | 12% | 33% | 31% (+22% revision job) | 2% |

Shares are of on-CPU samples. The workflow invocation of `compact-root-banded-4` (staged execution plus original assessment) takes 11.1 ms. KINSOL's own work is about 1 ms.

**POUNCE-convex with eight members.** It takes 6.1 s against 1.8 s for one member. The extra ≈0.6 s per member is authored-package admission. The solves are about 2% of samples. This is why the campaign's 1-versus-4-worker contrast shows no change (5.55 vs 5.62 s): the serial share is admission, not solving.

**Long campaign cases are preparation-bound too.** These are the campaign's own observation files (selected recording on). The invocation is measured from establishment to response-ready. Everything before it is preparation and admission.

| Case | Harness total (s) | Workflow invocation (s) | Native inside it (s) |
|---|---:|---:|---:|
| `fit-curved-second-cold-1` | 59.3 | 5.13 | 1.33 |
| `mixed-scale-1-1` | 59.5 | 6.44 | <0.01 |
| `qp-1-1` | 50.8 | 3.34 | <0.01 |
| `shooting-single-idas-cold-1` | 40.4 | 3.13 | 0.21 |
| `sparse-fit-12-1` | 35.3 | 2.38 | 0.02 |
| `review-sparse-fit-2-1` | 12.7 | 1.26 | 0.01 |

### 3.2 Growth is cubic in modeling admission

| N | 16 | 32 | 64 | 128 | 192 | 256 |
|---|---:|---:|---:|---:|---:|---:|
| Process wall (s) | 2.95 | 2.25 | 3.30 | 8.67 | 24.35 | 53.68 |
| Workflow invocation (s) | 0.109 | 0.072 | 0.103 | 0.235 | 0.377 | 0.802 |
| Native worker time (s) | 0.004 | 0.006 | 0.012 | 0.023 | 0.043 | 0.068 |

- **Admission.** Above a ≈2.1 s baseline, time grows ≈N^2.5–3.0. The source structure is cubic. At N = 256, 94% of samples sit under `check_with_phases`:
  - `expression::check_declaration → declaration_environment → named_types → resolve_with_types` accounts for 60% of all samples;
  - `phases::signature → resolve → named_types_with` accounts for most of the rest.
- **Inside the invocation.** Non-native orchestration is 0.09, 0.21, 0.34 and 0.74 s at N = 64, 128, 192 and 256.
  - It grows about N^1.1–1.2 up to N = 192, then steepens sharply (≈N^2.7 from 192 to 256).
  - Native work grows roughly linearly (P06).
- **The N = 64 failure.** By *strong inference*, it is a numerical-contract failure (P12), not a preparation failure.

### 3.3 Width, durability and diagnostics

**Width.** A 16-point study takes 2.02 s with one worker and 2.31 s with four. The campaign's paired medians agree (P05).

**Accounted memory.** Pool peak is 13–22 GB while RSS is 0.29 GB (P08).

**Durable 16-point study** (campaign timeline):

- 19.3 s of dispatch, execution and results;
- one 14 s activity;
- 92 result reopen/access spans totalling 3 s;
- about 7 ms of native work (P07).

**Ordinary diagnostics.** These leave science untouched: `prepare_start_join` is 0.26–0.28 s in every mode. They add 0.13–0.25 s after the run for inspection, export and analysis.

**Timeline capture.**

- It emits about 2,500 `log event` instants per second.
- It dropped 52,822 packets in `cold-small-1`.
- Phase ends can be lost, so the trace shows compiler spans that never close and cannot attribute the ≈34 s of PR admission by itself. CPU sampling (§3.2) places this kind of time in modeling checking (P19).

## 4. Best-in-class target state

### 4.1 Invariants of the target

1. **Work is proportional to change.**
   - Every product lives exactly as long as its premises: registry, package interface, checked package, structure, symbolic factorization, coloring, compiled evaluator, native session, linearization.
   - Stable products are content-addressed, so they survive across processes.
   - A value edit binds values. A structural edit re-checks only the changed declarations and their dependents.
2. **Work is proportional to size.**
   - Name resolution, admission, block construction, scheduling and readiness are near-linear in declarations, blocks, points and events.
   - Quadratic or cubic scans are defects even when today's fixtures hide them.
3. **Execution units are chosen for execution.**
   - Semantic distinctions stay in contracts: blocks, guards, demands, relations, studies.
   - Physical execution groups, fuses and batches them: coarsened block groups, fused evaluator modules, batched study points, persistent pools.
4. **Native libraries own iteration, with honest contracts.**
   - Native stopping rules are stagnation safeguards. Original-space acceptance stays authoritative at 0.001.
   - Warm states are actually warm.
   - Jacobians are sparse and exact, obtained through coloring.
   - Linear algebra matches size and structure: dense for tiny, KLU/SPRAL/MUMPS for sparse, Krylov with real preconditioners for large coupled systems.
5. **Validate where trust changes.**
   - Registry-owned data keeps its validated status (H19).
   - Full validation happens at Python/IPC ingress and canonical admission, not on every internal hop.
6. **One CPU and memory budget, events not polling.**
   - A persistent pool of native-capable threads steals work across cases, blocks, experiments and windows.
   - Reservations reflect computed bounds.
   - Durable coordination is driven by notifications.
7. **Results and diagnostics are off the critical path.**
   - Original assessment stays on the critical path.
   - Projection, encoding, export and evidence assembly run in bounded lanes.
   - Phase boundaries are lossless; per-event detail is opt-in and aggregated by default.

### 4.2 Lifetimes and the target pipeline

Each row is a product with its own validity, owner and storage. Rows are ordered from most stable to most volatile, and each is prepared from the rows above it. Every product is keyed by its complete premises, so a stale reuse is a contract violation rather than a cache miss.

| Product | Validity (invalidated by) | Owner | Storage in target | Today |
|---|---|---|---|---|
| Frozen schema registry with fingerprints | Registry declarations, generator version | `pse-schema` + codegen | Compiled into the binary (static arena) | Rebuilt per process, ≈0.94 s (P03) |
| Package interface (exported names, types, signatures, members) | That package's source and its imports' interfaces | `pse-modeling`/`pse-compiler` | Persistent, content-addressed | None; full re-check (P01, P02) |
| Checked package | Package source, dependency interfaces, physical registry, compiler version | same | Persistent, content-addressed; in-memory Salsa with fine-grained queries | Per process, whole-namespace inputs |
| Admitted documents (typed rows) | Document bytes, registry | `pse-admission` | Per session; validated once at ingress | Re-validated through DataFusion per package (P04) |
| Selected/specialized model and structure (incidence, BTF, execution groups, sparsity, coloring, symbolic factorization) | Selection, structure-affecting bounds and roles | `pse-compiler`, `pse-math` | Per session; persistable keyed by structure hash | Rebuilt per run; B×R scans (P06) |
| Executable mathematics (fused evaluator modules) | Bodies, demand, layout, evaluator policy, ISA/flags | `pse-math` | Persistent compiled-artifact cache | Per region, per order, per process (P10, P11) |
| Case binding (values, bounds, nominals, starts) | The edit | runtime | Per point / per edit | Good: ≈19 ms value preparation (PR inline) |
| Linearization (factor/preconditioner, complete child tokens) | Point, coefficients, provider context/branch | `pse-math`/native | Per Newton point | Reconstructed from n JVPs on the action route (P09) |
| Native lane (session, workspaces, callback scratch) | Layout and options compatibility | `pse-backend-native` | Per worker thread, reused | Often recreated per solve or window (P15, P17) |
| Results and evidence | Consumer demand, retention policy | `pse-runtime` results | Bounded lanes, streamed | Partly inline on the coordinator (P05) |

### 4.3 Layer targets

| Layer | Target mechanism | Removes |
|---|---|---|
| **L0 Process** | Registry emitted by codegen as a static arena whose fingerprints are checked at build. Long-lived runtimes in Python and in durable workers. | ≈0.94 s per process; most `serde_json`/`sqlparser`/blake3 startup work |
| **L1 Modeling admission** | **Indexed, lazy lexical resolution:** interned names; per-scope `name → id` tables built once; `TypeLookup` resolved on demand; a candidate-scoped memo for admission-time evaluation. **Fine-grained incremental queries:** `visible_name(scope,name)`, `type_of(decl)`, `members(scope)`, with early cutoff and high durability for library packages. **Separate compilation** against imported interfaces, with a **persistent content-addressed cache** of interfaces and checked artifacts. **Parallel checking** of read-only phases (Salsa `rayon`, import DAG). Typed IR instead of JSON masks. | Cubic admission (P01); re-admission of unchanged libraries in each process (P02) |
| **L2 Document admission** | A typed loader validating during decode. Validation templates per relation prepared once per registry; `DomainEnvironment` built once. Registry-produced batches carry a trusted-contract handle (pointer identity), so internal hops skip re-validation. One validation scope per `RunResult`. | ≈0.5 s per package (P04) |
| **L3 Structure** | Indexed block views (row/column → block maps built once). **Physical grouping** separate from semantic BTF: chains of tiny blocks run in one native session or one sparse solve, independent groups go to the scheduler. Sessions reused by structural shape. Blocks prepared in parallel. | B×R scans and per-block lifecycles (P06) |
| **L4 Mathematics** | One fused program per (body set, demand) producing values, first-order and weighted second-order terms, with CSE across outputs and orders; guards checked afterwards in original order. Sparse Jacobians by distance-2 coloring (CPR), including implicit states and derived families. Values reused across orders at one point. Compiled artifacts per model in one translation unit, compiled in parallel and cached on disk. Later: batched/SIMD/JIT evaluation across instances and study points. | n-directional Jacobians (P09); fragmented programs and repeated value passes (P10); per-region compilers (P11) |
| **L5 Native execution** | Portfolio in §4.4. Stopping semantics aligned with acceptance (P12). Real warm starts (P13). Anderson-accelerated recycles (P14). Reused IDAS sessions and lean callbacks (P15). Least-squares-aware fitting and parallel experiments/windows (P16). Native handles retained per lane; events only when detail is selected (P17). | Wasted iterations, false `Limit` outcomes, setup churn |
| **L6 Orchestration** | Lanes as spawned tasks reporting completions by message. One shared admission per (operation, binding shape). Event-driven frontier. Persistent 32 MiB-stack worker pool with work stealing under one CPU budget. Compatible batch partitions with per-member pre/post inside the pool. Computed memory bounds. Durable claims in batches with notifications. | Width without throughput (P05); polling (P07); inflated reservations (P08) |
| **L7 Results/diagnostics** | Lazy, streamed result materialization in bounded lanes. Lossless phase boundaries (per-thread rings). Aggregated counters with pre-resolved handles. Spans for lowering, specialization and materialization. | Coordinator stalls; misattribution (P19) |

### 4.4 Numerical method portfolio

The accuracy target stays 0.001 in original space. Native tolerances are method controls, never acceptance.

| Workload | Target route | Key settings and choices |
|---|---|---|
| Tiny blocks (n ≤ ~32) | KINSOL with dense LU in one session per execution group, or a grouped sparse solve over the whole triangular chain | Explicit `KINSetMaxNewtonStep` in normalized units. Step tolerance at stagnation level (≈ uround^(2/3)). One polish step plus original check before a `Limit` refusal. |
| Large coupled roots | Newton with fixed-pattern KLU (symbolic analysis once per structure); Krylov with BlockFactor/ILU for very large | `setup_interval` adaptive (default 10 is reasonable). Sparse Jacobian by coloring. |
| Recycles/tears | Anderson (m = 3–5, delay 1) through `KINSetMAA` by default; Wegstein/Broyden options; Newton on tears using a directional-derivative tear Jacobian when available | Index-vector port exchange; acceleration restart diagnostics |
| Smooth sparse NLP | Ipopt or POUNCE with exact sparse derivatives; SPRAL or MUMPS selected by size; limited-memory where Hessian fill dominates | Warm starts: primal-only seeds get a restart profile (small μ₀, small pushes); dual seeds get barrier warm start; qualify `warm_start_same_structure` on unchanged structure |
| Re-solve after edit / continuation | Tangent predictor (already strong in studies) plus warm native state | Retained native application per lane; numeric options outside the structural signature |
| Convex QP/cones | Clarabel / POUNCE-convex with tolerance translated from the absolute requirement | Bounded tighten-and-resolve; original cone check retained |
| Nested property solves (implicit) | Retained sparse F_y factor with complete child linearization tokens; action-only rebinding | CPR coloring for F_y construction; provider-effect invariants preserved |
| Dynamics (index-1 DAE) | IDAS with a persistent session per lane (`IDAReInit`), KLU with symbolic retention; Krylov plus block preconditioner for large coupled systems | Jacobian shared by `res`/`resS` per (t,y); staggered corrector where qualified; IDAS-chosen h₀; statistics per segment |
| Sensitivities | Chosen automatically. Forward when parameters are few relative to the outputs that need gradients (typical least squares). Adjoint when a few functionals depend on many parameters. | Avoid per-sample backward re-initialization for pure ODE contracts |
| Least-squares fitting | Gauss–Newton/LM (trust-region reflective) for unconstrained or bounded fits; Ipopt with GN Hessian when constrained | One integration with sensitivities per trial; parallel experiments by default; integrator tolerance tied to optimizer stationarity |
| Multiple shooting | Windows in parallel lanes at a fixed trial | Shared CPU budget with the outer optimizer |

### 4.5 Target outcome measures

These are *Proposed* design aims to qualify against, not measured results.

| Measure | Today (measured) | Target aim |
|---|---|---|
| Process ready (registry + runtime) | ≈1.0 s | ≤ 50 ms |
| Small authored package admission | ≈0.5–0.7 s | ≤ 20 ms |
| Modeling admission growth | ≈N^2.5–3.0 (256 → 51 s) | ≈ N log N; ≤ 1 ms per declaration cold |
| Reference thermo library, second process | 36–41 s `modeling.prepare` (PR/PC-SAFT); heater source admission 58 s (campaign) | ≤ 100 ms (persistent cache load) |
| Value edit → re-solve preparation | ≈19 ms (PR inline) | ≤ 10 ms, plus a warm native solve |
| Source (declaration) edit, e.g. adding a unit | Full re-admission of the package namespace | ∝ changed declarations and dependents |
| Independent study throughput | Flat with width | ≥ 0.8 × cores for non-trivial points |
| Durable per-point coordination | ≈1.2 s | ≤ 20 ms |
| Per-callback bookkeeping (ordinary observation) | ≈5–30 µs (source estimate) | ≤ 1 µs |
| Ordinary diagnostics overhead | +0.13–0.25 s per process | ≤ 5% |

A **representative flowsheet benchmark** should become the north star: 2,000–5,000 equations, PR thermodynamics, a recycle, a column and a parameter sweep. The campaign's fixtures cannot distinguish the large-scale numerical choices.

### 4.6 Physical semantics and well-posedness the target must preserve

Every optimization above changes how and when work is done, not what is computed or accepted.

| Element | Dimension, basis, convention | Validity envelope | Authority that must remain unchanged |
|---|---|---|---|
| Model variables and residual rows | Authored physical units, material basis, reference states | Declared domains and bounds | Typed authored model; compiler admission. Caches (P02) are keyed by complete premises, including the physical registry. |
| Normalized coordinates x = S z (shifted for one-sided bounds) | Declared nominal scales | Same | Numerical policy §16; native coordinates stay derived. Native tolerances (P12) are method controls only. |
| Original-space acceptance | Per-row and per-variable budgets at engineering accuracy 0.001 | — | Independent original assessment after every new numerical state, including polish steps, warm starts, grouped blocks and cached linearizations |
| Property providers and implicit (nested) states | Provider input context, phase/branch selection | Provider envelope | Provider effects are re-validated on context, branch or point change. Linearization reuse (P09/P10) requires complete child tokens. |
| Dynamic states | Differential/algebraic roles, initial conditions, events | Integrator error weights | Consistent initialization at every (re)start, including with reused sessions (P15) |
| Cone coordinates | Declared cone blocks | — | Reconstructed b − Ax checked geometrically (P12) |

**Well-posedness.**

- Variable roles are authored.
- Degree-of-freedom and structural analysis (matching, BTF) run in preparation, before any solver. Structurally singular or over-specified systems are refused there, with diagnostics in model terms.
- The proposed physical grouping (P06) is applied *after* BTF and does not replace it:
  - semantic blocks remain the unit of diagnosis and of root/branch selection;
  - grouping changes only the execution unit.
- Lazy name resolution (P01) must reproduce exactly the visibility, shadowing and refusal behavior of the current checker. The existing exhaustive oracle is the acceptance test.

### 4.7 Numerical stages (slot 5)

| Stage | Formulation policy | Derivative source / order | Scaling | Class · solver capability | Status → outcome | Tolerances · post-solve check |
|---|---|---|---|---|---|---|
| Block roots / recycles | Guards as Require stages, value-only nonsmooth switching | Exact sparse first order (target: colored) | Declared nominals, normalized | Square NLE · KINSOL Newton/line search; fixed point + Anderson (target default) | `Limit`/`StepLtStptol` → Limit; target adds a polish step before refusal | Residual test exact in scaled form; step tolerance is a stagnation floor (target); original check |
| NLP | Same | Exact first and second order (flat: direct weighted Hessian; providers: jets → target: pullbacks) | Same | NLP · Ipopt/POUNCE/Uno | Native status is evidence only | Native `tol`/`dual_inf` are controls; original check decides |
| Conic | Linear/cone coefficients | n/a | Equilibration inside Clarabel | Cone · Clarabel, POUNCE-convex | `Solved` remains provisional | Target: translated relative tolerance plus bounded re-solve; geometric b − Ax check |
| Nested property (implicit) | Inner root with branch/anchor selection | IFT with sparse LU (target: colored F_y, token reuse) | Provider scaling | Inner NLE · KINSOL/faer | Inner failure → provider error | `verify` plus selection verification |
| Dynamics | Segments, events, consistent initialization | Analytic Jacobian; forward or adjoint sensitivities | Uniform atol on normalized states | Index-1 DAE · IDAS/Diffsol | Native flags → segment outcomes | rtol/atol from policy; outputs re-evaluated |
| Estimation | Weighted least squares | Responses via sensitivities | Parameter scales | Today: general NLP · Ipopt/L-BFGS; target: LM/GN for least squares | Optimizer status → fit outcome | Stationarity tied to integration accuracy (target); original fit assessment |

## 5. Findings

Finding IDs are local to this review (P01–P19). R1's findings are cited as R1-Fnn.

| ID | Finding | Area | Diagnosis | Priority |
|---|---|---|---|---|
| [P01](#p01) | Modeling admission is cubic in declaration count | Preparation | Measured + Implemented | Critical |
| [P02](#p02) | Admission is neither incremental across edits nor reusable across processes, nor parallel | Preparation | Implemented; Measured effect | Critical |
| [P03](#p03) | The schema registry is rebuilt in every process, declared twice and fingerprinted three times | Preparation | Measured + Implemented | High (easy) |
| [P04](#p04) | Authored-document admission re-validates and re-plans per package | Preparation | Measured + Implemented | High |
| [P05](#p05) | Studies and batches do not convert width into throughput | Orchestration | Measured + Implemented | High |
| [P06](#p06) | Block preparation and per-block lifecycles grow super-linearly | Orchestration | Measured + Implemented | High at scale |
| [P07](#p07) | Durable execution is coordination-bound; source shows polling and rescans | Orchestration | Measured totals (campaign) + Implemented mechanism | Medium-high |
| [P08](#p08) | Opaque worst-case reservations inflate accounted memory 50–75× | Orchestration | Measured + Implemented | Medium |
| [P09](#p09) | Implicit and derived-family Jacobians use n directional products; nothing uses coloring | Mathematics | Implemented | High at scale |
| [P10](#p10) | Evaluator programs are fragmented and recompute shared values | Mathematics | Implemented | Medium-high |
| [P11](#p11) | Compiled evaluation compiles per region per order and never persists | Mathematics | Implemented | Medium (prerequisite) |
| [P12](#p12) | Native stopping semantics do not match original-space acceptance | Numerical | Interface-checked + Measured | High (correctness) |
| [P13](#p13) | Accepted starts rarely save iterations | Numerical | Implemented | Medium-high |
| [P14](#p14) | Recycles default to unaccelerated substitution | Numerical | Implemented | High for recycles |
| [P15](#p15) | Dynamic integration repeats native setup and per-step bookkeeping | Numerical | Implemented | High for dynamics |
| [P16](#p16) | Parameter estimation and shooting ignore least-squares structure and parallelism | Numerical | Implemented | High for fitting |
| [P17](#p17) | Per-solve native rebuilds and per-callback bookkeeping dominate small solves | Native/code | Implemented | Medium |
| [P18](#p18) | Untyped internal representations drive allocation churn | Cross-cutting | Measured + Implemented | Medium (pervasive) |
| [P19](#p19) | Diagnostics lose phase boundaries and flood the trace | Diagnostics | Measured + Implemented | Medium |

### 5.1 Preparation and admission

<a id="p01"></a>**P01 — Modeling admission is cubic in declaration count.** (AP-07, H9/H10/H11, PSE-S07)

*Measured.* See the banded sweep in §3.2. Above baseline the local exponents are ≈2.5 (64 → 128), 3.0 (128 → 192) and 2.9 (192 → 256). At N = 256, 94% of samples are in `pse_modeling::check::check_with_phases`.

*Implemented.* `CheckedPackage::named_types_with` (`pse-modeling/src/check.rs`) builds the complete name → `Type` map for one owner:

- For every visible prefix (the package plus each import alias on the lexical chain), it walks **every** name below that prefix.
- For each such name it calls `resolve_with_types`. That walks the owner chain and scans children and imports at each level.
- Then, per owner, it does the same for relative names and import aliases, formatting strings and cloning `Type` trees.

This map is built once per declaration in `resolved`/`typed` and again in `checked_expressions` (`declaration_environment`). In a flat package that is N owners × N names × O(children) resolution, which is cubic.

There is a second source of the same cost. `specialize/value.rs::Evaluator::named_types` skips the `LexicalBases` memo for the admission reader. It therefore rebuilds the environment for each arithmetic root, reduction and fold evaluated during `complete`. `entity::derive` creates one such evaluator per derivation × entity; the chemistry `molar_mass` sum runs over 28 species × 21 elements.

Earlier remedies (AUD39-01/02/09, Plan 42a A2) removed particular call sites. The generator of the cost — eager whole-environment construction — remains.

*Consequence.*

- A 256-unknown model spends 51 s in admission.
- Reference thermo packages show 36–41 s `modeling.prepare` spans, and the heater shows 58 s of source admission (campaign).
- By extrapolation (*Hypothesis*), about 1,000 unknowns of the same shape would take about 50 minutes.

Industrial flowsheets are infeasible, structural edits are slow, and every cold process pays the cost. This is the single largest gap against the functional target.

*Correction.* Replace eager environments with **indexed lazy resolution**:

- Each checked package owns interned names and per-scope `name → id` tables (members, children, import aliases).
- `qualified_name` is cached.
- A `TypeLookup` resolves a name on demand through the same shadowing, import-precedence and guarded-member rules.
- Iteration-only consumers (`extent_types`, kinds) get a per-package kind index.
- Admission-time evaluation uses a candidate-scoped memo keyed by (owner, reader). It does not outlive the candidate, so it does not reintroduce a retained owner-only cache.
- The existing `check/named_types_tests.rs::exhaustive_named_types` is the equivalence oracle.

*Verification.* Repeat the banded/diagonal/coupled sweep to N = 2,048. The exponent should be ≤ ≈1.2. Counts of name resolutions should be ∝ references, and the oracle should match. Relocated complexity would show up as an unchanged exponent.

<a id="p02"></a>**P02 — Admission is neither incremental across edits nor reusable across processes, nor parallel.** (AP-07, H12/H13/H14, PSE-S01/S07; rule impact RC01)

*Implemented.*

- **Coarse Salsa inputs.** Every `TypeInput`/`MemberInput` embeds `Namespace(Arc<whole CheckedPackage>)` plus the source-order type prefix, so any edit invalidates every typed/resolved/member memo.
- **Deep equality.** `CheckedPackage` derives only `PartialEq`, so setter guards deep-compare the whole package per declaration and per occurrence.
- **Small retention limits.** `WorkspaceLimits` sets `query_values: 64` and `retained_entries: 16_384`. A reference-scale selection exceeds those, which forces the observed `compiler.workspace_rebuild`.
- **Fresh workspaces.** Persist and delta admission use them.
- **Serial.** `check_declarations` runs sequentially, and Salsa is built without `rayon`.
- **No persistence.** Blueprint §14.4 states: "There is no persistent Salsa store; a new process rebuilds from the admitted revision, and durable reuse applies only to published data."

*Measured effect.* Every new process re-admits unchanged reference libraries. In the campaign that meant 36–41 s `modeling.prepare` spans for PR/PC-SAFT and 58 s of heater source admission.

*Correction.*

- **Separate compilation.** Each package is checked against its imports' *interfaces*: exported names, types, function signatures and members.
- **Persistent cache.** Interfaces and checked artifacts are stored in a compact binary encoding, keyed by package source hash, dependency interface hashes, physical/registry identity, and compiler/interpretation version. They are loaded instead of re-checked; a fresh check remains the fallback.
- **Fine-grained queries.** Inputs move to tracked per-declaration queries with early cutoff and high durability for library packages, so an edit rechecks only dependents.
- **Pointer-or-hash equality** on package-level values.
- **Retention limits** scale with N.
- **Parallel read-only phases:** expression checking per declaration, and packages along the import DAG.

The cache is an accelerator over complete premises. It is not a new authority. It must refuse on registry, interpretation or compiler change.

*Verification.*

- A second process loads the reference library in ≤ 100 ms (aim) with a byte-identical checked-package digest.
- A one-declaration edit executes queries ∝ its dependents.
- The expression phase scales with cores.

<a id="p03"></a>**P03 — The schema registry is rebuilt in every process, declared twice and fingerprinted three times.** (AP-07, H12/H14/H20)

*Measured.* `pse_schema::catalog::assemble` takes 0.937 s per process (gdb). It is 52–53% of small-case CPU. Callgrind attributes 14.2 G of the 14.3 G instructions it captured to `RegistryBuilder::build`.

*Implemented.* The process-wide `OnceLock` builds the registry by:

- `semantic_product` per relation in `RegistryBuilder::build`;
- again through `ContractArena::resolve → ExpectedContract::capture`;
- a third time through `builder/native.rs::BOOTSTRAP`. `materialize` forces that unconditionally, and it re-declares the catalog, recompiling SQL obligations.

Each `semantic_description`:

- clones fields and metadata maps;
- canonicalizes check SQL through `sqlparser`;
- filters about 225 invariants by formatted qualified name;
- builds `serde_json::json!`, then hashes `canonical_json` (`to_value` → `sort_all_objects` → `to_string` → blake3).

Closure descriptions are not memoized across roots (≈3·R·K̄ descriptions for about 168 relations).

*Consequence.* Every Python session, test process, benchmark, CLI and durable worker pays about 1 s before doing anything. It is the largest single term for small solves.

*Correction.*

1. **Immediately:** memoize one description and hash per relation; derive closure digests from sorted member hashes; reuse them in `ContractArena`; make `BOOTSTRAP` lazy; index invariants by relation key.
2. **Target:** emit the frozen registry arena — specs, Arrow schemas, expected contracts, fingerprints — from codegen as static data. A build-time digest over the declarations protects it. This preserves "one authoritative declaration" (prime directive 3), because the generator owns it.

*Verification.*

- Fingerprints are byte-identical to today's.
- Registry-ready time is ≤ 10 ms (aim).

<a id="p04"></a>**P04 — Authored-document admission re-validates and re-plans per package.** (AP-07, H18/H19/H20)

*Measured.* Each small package costs ≈0.5–0.7 s. Eight packages add ≈0.6 s each; this is 38–41% of small-case CPU and 79% at eight packages. The leading frames:

- `pse_relations::validate::predicates::field_local` (15–24%);
- `validate_field`;
- `pse_engine::session::admission::{restore_semantic_fields, admit_field, admit_expression}`;
- DataFusion physical planning and `filter_record_batch`.

*Implemented.*

- **Validation templates.** `occurrences::Node::prepare` calls `field_local` for every nested field node. That calls `DomainEnvironment::current(registry)`, which rebuilds a map of all registry enums (about 230 in `docs/generated/enums.md`, ≈2,200 strings) each time. It then plans a DataFusion expression.
- **Schema re-checking.** `validate_schema` re-parses JSON facets and re-serializes them canonically. `execution_field` re-validates the registry's own expected field on every admission.
- **Full column validation.** `validate_full` runs on every column of untrusted batches.
- **Validation scopes.** Every activity gets a new one, and `RunResult` gets two.
- **Hashing.** `SchemaRef` is hashed in a mutex-guarded map.

*Consequence.* A fixed ≈0.5 s per package dwarfs the science. It also limits studies that require distinct packages.

*Correction.*

- Prepare validation templates once per relation and registry, statically where possible. Build `DomainEnvironment` once.
- Registry-produced batches carry a trusted-contract handle, so internal hops take the pointer-identity fast path.
- Full validation stays at untrusted ingress (Python, IPC, canonical admission).
- A typed document loader validates while decoding into typed rows, instead of building Arrow batches and planning DataFusion expressions per field for small authored documents. DataFusion remains the engine for bulk relational queries.

*Verification.* Small-package admission ≤ 20 ms (aim). Refusal diagnostics are identical on a malformed-document corpus.

### 5.2 Orchestration and parallelism

<a id="p05"></a>**P05 — Studies and batches do not convert width into throughput.** (AP-03, AP-07, H21/H22, PSE-S03/S07)

*Measured.*

- 16 points: 2.02 s with one worker, 2.31 s with four.
- Paired medians for a 4-point study: 1.68 vs 1.62 s.
- Eight convex members: 5.55 vs 5.62 s.

Per-point science is milliseconds, so fixed costs alone would flatten these curves. The runs show the symptom but do not single out the coordinator mechanism. The mechanisms below are *Implemented* (source).

*Implemented.*

- **Single coordinator.** `study_execution.rs::study_inner` polls every lane in one `FuturesUnordered` on the caller's task. Preparation glue, `RunResult` completion, batch assessment preparation, Compact capture (R1-F10) and `staged.close` therefore serialize.
- **Per-lane admission.** Each lane owns its own `DeclaredStudyAdmission`, so W lanes perform W admissions.
- **Width cap.** Width is `min(cores, jobs = 4)`. The jobs semaphore is `try_acquire`d rather than queued.
- **Per-point fixed costs.** Each point gets a new activity scope, two `RunResult` validation scopes, and an OS thread per math job (32 MiB stack).
- **Batching.** Batching is all-or-nothing on profile (R1-F08). Batched members still run transport/lowering (`Transported::new`) and finish/assessment serially. The only parallel part is `par_iter` over the native solves. POUNCE-convex builds a rayon pool per batch.
- **Frontier rescans** (R1-F09) are real but immaterial below about 1,000 points.

*Correction.*

- Spawn lanes as tasks that report completions by message; capture and close move into the lane.
- Share one `Arc` admission per (operation, binding shape); points bind values only.
- Use an event-driven frontier with dependency counters.
- Keep a persistent pool of native-capable worker threads with work stealing across points, blocks, experiments and windows under one CPU budget.
- Default width to available cores, subject to memory admission.
- Partition batches by compatibility, and run per-member pre/post inside the pool.

*Verification.* On a heavier per-point model (for example a PR flash per point, 256–10,000 points), throughput is ≥ 0.8 × cores. Coordinator work is ∝ events.

<a id="p06"></a>**P06 — Block preparation and per-block lifecycles grow super-linearly.** (AP-07, H2/H9; confirms R1-F03)

*Measured.* Inside the workflow invocation, non-native orchestration grows from 0.09 s at N = 64 to 0.74 s at N = 256. That is about N^1.1–1.2 up to N = 192, steepening sharply above it. Native time grows from 12 to 67 ms.

*Implemented.*

- `Assembly::conditional` filters all parent rows per block (B×R).
- `validate_prepared_block_overrides` scans rows twice per block.
- `prepare_blocks` is a serial loop. Each block gets function assembly, a binding job thread, its own binding and `prepare_conditional`, with `ReusePolicy::Fresh`.
- Each differently shaped component creates a new KINSOL session (SUNContext, KINCreate, KLU).

*Consequence.* This is masked today by P01. At 10⁴ blocks it becomes quadratic preparation plus thousands of native lifecycles.

*Correction.*

- Build row/column → block indexes once.
- Keep BTF for diagnosis and selection semantics, but let a **physical grouping** choose execution units. Chains of tiny blocks run inside one native session (sequential scalar Newton) or as one fixed-pattern sparse solve. Independent groups go to the scheduler.
- Reuse sessions keyed by structural shape.
- Prepare blocks in parallel.
- Preserve per-block root/branch selection and initial-start semantics, and qualify trajectory changes (R1 caution applies).

*Verification.* Per-block invocation time is flat in N. Native creations are ≤ the number of groups. Original checks are identical.

<a id="p07"></a>**P07 — Durable execution is coordination-bound; source shows polling and rescans.** (AP-07, H21/H25)

*Measured (campaign totals).* 16 trivial points take 19.3 s of dispatch, execution and results, about 1.2 s per point. Native time is ≈7 ms. The timeline does not decompose that time.

The attribution to polling intervals and per-turn rescans below is a source-based inference. At 16 points the O(N²) term is small; the fixed poll intervals and per-action round trips are the likelier contributors.

*Implemented.*

- `worker.rs::work_once_scoped` rescans study page → readiness → candidates and performs one action per turn. That is O(N) store queries per action and O(N²) per study.
- The idle poll is 500 ms; the observer loop sleeps 100 ms.
- Each 10 s heartbeat spawns a supervisor process.

*Correction.*

- Claim batches per turn.
- Maintain readiness incrementally with stored dependency counters.
- Use live-query/push notifications.
- Keep a warm managed primary with no per-heartbeat process.
- Share the bulk-dispatch capability with the local path (R1-C03).

*Verification.* Per-point coordination ≤ 20 ms. Store queries per action are O(1).

<a id="p08"></a>**P08 — Opaque worst-case reservations inflate accounted memory 50–75×.** (AP-07, H17/H21)

*Measured.* Pool peak is 13–22 GB against RSS of 0.29 GB.

*Implemented.*

- `ConstructionEstimate::from_bound(None, cap)` becomes `Opaque(cap)`, with `worker_bytes` = 8 GiB or `workspace_bytes` = 4 GiB.
- Each job also reserves stack × (cores + 1) when it uses more than one core (one stack otherwise), plus 64 MiB foreign and 16 MiB inner-session allowances.

*Consequence.* Under a realistic memory budget, admission would serialize or refuse parallel work that easily fits. Memory diagnostics also mislead.

*Correction.*

- Use the computed allocation bounds that already exist (`coefficient_allocation_bound`, `validation_allocation_bound`, class bounds).
- Use grow-on-demand grants (`ConstructionOwner` already supports growth).
- Size stack reservations by the persistent pool, not per job.

*Verification.* Accounted peak is within a small factor of RSS. Parallel admission works at a 16 GB budget.

### 5.3 Mathematical realization

<a id="p09"></a>**P09 — Implicit and derived-family Jacobians use n directional products; nothing uses coloring.** (AP-07, H9/H10; confirms R1-F01)

*Implemented.*

- **Implicit products.** `implicit_products.rs::Problem::demanded_derivatives` resizes a dense n × n tangent and runs n unit-seed JVPs. It then copies the sparse pattern and calls `factorize_ift`. That function's reuse test compares premise, point, children and **every matrix value bit**, so it runs only after reconstruction. The CompleteJets/SelectedColumns route evaluates the full n + p jet first.
- **Derived families.** `derived.rs::FamilyRoot::jacobian` (homotopy, pseudo-transient, projection) clones the pattern and runs n unit actions with an O(n·m) scan. `BoundPseudoTime::jacobian` runs n JVPs with an O(n·nnz) per-column filter.
- **No coloring on these routes.** Implicit-product and derived-family Jacobians use no coloring. Coloring does exist elsewhere and can be reused: a greedy CPR coloring of the IDAS adjoint coupling block in `dynamics/idas.rs`, and POUNCE's CPR finite-difference Hessian option.

*Consequence.*

- On the action route (Jacobian, transpose and weighted-second actions), every implicit linearization costs n residual products and O(n²) dense memory. Value, CompleteJets and SelectedColumns return earlier.
- Every derived-family Jacobian costs n residual products.

At flowsheet scale this becomes a leading numerical cost.

*Correction.*

- Use structural sparsity plus distance-2 coloring (CPR): c ≪ n compressed directions recover exact entries. A direct sparse SelectedPartials program over unknown coordinates is the alternative.
- Retain the sparse F_y and its factor with complete child-linearization tokens; new directions, cotangents or weights bind actions only (R1 safety boundary preserved).
- Skipping reconstruction when tokens match changes the §14.3 premise of actual matrix identity (RC04). Coloring alone needs no rule change.

*Verification.* JVPs per linearization equal the color count. Derivative checks agree. Provider, branch or context changes still invalidate.

<a id="p10"></a>**P10 — Evaluator programs are fragmented and recompute shared values.** (AP-07, H2/H8; refines R1-F02 and the Oct-17 claims)

*Implemented.*

- **Fragmentation.** `typed.rs::obligation` gives every guarded operation (log, powers, division) its own one-expression `Block` plus a `Require`, and `exp` always gets its own block. `execution.rs::arithmetic_region` merges only consecutive independent blocks and stops at every `Require`. Each tiny region carries its own Symbolica evaluator, gather/scatter, Taylor frame and finiteness check.
- **No CSE beyond a region.** Nothing is shared across regions, derivative orders, demands (Objective/Constraints/All are separate requests) or instances.
- **Repeated value passes.** Per accepted interior-point iterate, values are computed about three times. The direct product re-runs `evaluate_seeded(Value)`.
- **Provider Hessians.** Provider-bearing bodies (every thermo body) build per-output hyper-dual pair jets, with a binary search on every access.
- **Serial instances.** Instances are evaluated serially.
- **Correction to R1-F02.** The complete-triangle cutoff applies only to flat bodies. Others use reverse pullbacks.

*Correction.*

- Build one fused program per (body set, demand) that produces values plus first-order (plus weighted second-order) terms, with CSE across outputs and orders.
- Check guards afterwards, in original order. f64 does not trap, so the first-violation attribution is unchanged.
- Reuse values across orders at one point. Provider bodies qualify only with complete linearization tokens (RC04).
- Contract weighted Hessians through stage pullbacks instead of per-output jets.
- Batch instances.

*Verification.* Evaluator calls per iteration fall. Guard diagnostics and derivative checks are unchanged.

<a id="p11"></a>**P11 — Compiled evaluation compiles per region per order and never persists.** (AP-07, H12/H14; confirms and widens R1-F12)

*Implemented.*

- Every Block region of every compiled order, plus the direct product, gets its own temporary directory with `g++ -c` and `g++ -shared` children.
- The only cache is the in-process artifact map.
- Artifact fetches are awaited serially.
- This path is **opt-in**: the default is interpreted. It therefore does not explain the campaign's times.

*Consequence.* Compiled evaluation cannot become the default, because cold compile cost multiplies with P10's fragmentation and repeats in every process.

*Correction.*

- One translation unit per artifact covering all regions and orders.
- Compiles in parallel.
- A persistent content-addressed artifact cache keyed by exported source, evaluator identity, compiler version and conservative FP flags.
- Then evaluate an in-process JIT as a qualified alternative (RC02).

*Verification.* Compiler children ≤ artifacts. A warm process compiles nothing. Outputs are bit-identical under the declared flags.

### 5.4 Numerical methods

<a id="p12"></a>**P12 — Native stopping semantics do not match original-space acceptance.** (G2, PS-G3; confirms R1-F04 and R1-F05)

**KINSOL** (*Interface-checked* against vendored SUNDIALS 7.1.1).

- The adapter sets `step_tolerance = feasibility` and variable scales `qS/t`.
- With those scales, `KINScSNorm` declares a step too small (the stagnation stop, `KIN_STEP_LT_STPTOL`) when |Δx_i| ≤ (a/q)·t_i + a·|x_i − S_i·o_i|. That is not the documented physical step budget (numerical-execution §16.6).
- The line-search floor tracks `scsteptol` and sits roughly 10⁷–10⁸× above KINSOL's default.
- *Measured.* The banded N = 64 run exits `KIN_STEP_LT_STPTOL → Limit` and is refused (`infeasible; unqualified; native_outcome`).
  - The eight coordinates the harness prints are within 4.1e-6 of the analytic solution.
  - The full candidate's residual is not reported.
  - N = 128 and 256 pass.

  The link between the step semantics and this refusal is a *strong inference*.

**Clarabel.**

- `tol_feas = feasibility` is a *relative* residual. The SOC/PSD original violations of 0.0044 and 0.0019 are consistent with its normalization.
- POUNCE-convex's `tol = 0.1·min(budgets)` may carry the same risk (*Hypothesis*).

*Consequence.* False `Limit` outcomes, fallbacks and refused "Solved" results waste whole solves and reduce reliability.

*Correction.*

- **KINSOL.**
  - The step tolerance becomes a stagnation floor (≈ uround^(2/3) in scaled units); acceptance remains the residual test plus the original check.
  - Set `KINSetMaxNewtonStep` explicitly in normalized units.
  - On `Limit` with a candidate, take one polish Newton step plus an original check before refusal.
- **Cones.** Translate the absolute requirement into a relative tolerance using data and solution norm bounds. Add a bounded tighten-and-resolve loop with a fresh original check. Native slack is never substituted.

*Verification.*

- Banded N = 64 passes.
- R1's scaled/shifted KINSOL probe is explained.
- SOC/PSD pass with ≤ 1 re-solve.

<a id="p13"></a>**P13 — Accepted starts rarely save iterations.** (G7, AP-02; widens R1-F14; rule impact RC05)

*Implemented.*

- **Primal seeds lose warm status.** `presolve/pipeline.rs::Pipeline::new` builds `warm` only when duals or working state exist. A primal-only seed is passed as `initial`.
  - Receipts read `start_submitted = warm.is_some()` on Uno, the Ipopt sequence and POUNCE.
  - POUNCE then applies cold IPM defaults (μ₀ = 0.1, push 0.01) and discards the seed's quality.
- **Ipopt rebuilds its internal spaces on every call.** The application and TNLP are retained, but every call, including `ReOptimizeTNLP`, sets `warm_start_same_structure=no`. Retention itself requires an identical option table, including `tol` and `max_iter`.
- **Uno** is created and destroyed per solve.
- **Benchmark design.** The "accepted" benchmark cases solve cold first, then seeded, so the campaign could not show a benefit.

*Consequence.* NLP edit → re-solve and continuation restart nearly cold. That loses the main lever for interactive optimization, and receipts misreport what happened.

*Correction.*

- Transport primal-only seeds as warm, with a restart profile: small μ₀, small bound pushes.
- Use dual warm starts when available.
- Qualify `warm_start_same_structure` for unchanged structure signatures. The §18.3 rationale about value-dependent scaling becomes the settling question.
- Split structural from numeric options in the application signature.
- Retain the Uno handle.
- Make receipts truthful.

*Verification.* Iteration counts for seeded versus cold on perturbed NLPs. The aim is ≥ 2–5× fewer iterations. Results are identical within acceptance.

<a id="p14"></a>**P14 — Recycles default to unaccelerated substitution.** (AP-07, PSE target "recycles and tears"; rule impact RC06)

*Implemented.*

- `strategies/requests.rs::RecycleRequest::from_model` sets `anderson: 0`, `anderson_delay: 0` and damping 1.0, giving KINSOL fixed point with plain substitution.
- Ports exchange `BTreeMap<SemanticId, f64>` on every iteration.
- §17.4 deliberately keeps "current damping/Anderson defaults".

*Consequence.* Iterations scale like ln(tol)/ln(ρ). A tear with spectral radius 0.98 needs ≈340 iterations, where Anderson typically needs 10–30. Recycles are where the target says convergence dominates the user's experience.

*Correction.*

- Default to Anderson (m = 3–5, delay 1) through KINSOL's existing support, keeping it explicit and reported.
- Offer Wegstein/Broyden.
- Run Newton on tears with a directional-derivative Jacobian when providers allow.
- Use index-vector port exchange.

*Verification.* Iterations on the stagnating and coupled fixtures. The converged solution is unchanged within acceptance.

<a id="p15"></a>**P15 — Dynamic integration repeats native setup and per-step bookkeeping.** (AP-07, H12/H26; confirms R1-F06 and R1-C01; refutes an Oct-17 claim)

*Implemented.*

- **Session lifecycle.**
  - Each shooting window or attempt creates a new IDAS `Session`, redoing SUNContext/IDACreate and KLU symbolic analysis.
  - The Initial function is evaluated twice per attempt.
  - A fixed h₀ is forced at every reinitialization.
- **Callbacks.**
  - The residual and sensitivity callbacks copy vectors and allocate.
  - With the Simultaneous corrector, `resS` re-evaluates the full Jacobian on every Newton iteration, because the value cache does not satisfy derivative demand.
  - Anchored windows rebuild CSC from triplets and reserve memory on every derivative callback.
  - The guard program runs on every callback.
  - Diffsol formats a string per callback.
- **Per-sample work.**
  - `statistics()` runs about 30 `IDAGet*` calls and rebuilds JSON after every sample.
  - The adjoint backward pass re-initializes, with `IDACalcICB`, at every observed sample.
- **Preconditioning.** Krylov preconditioning is None/Jacobi only; BlockFactor is refused even though `precondition_setup` evaluates the full Jacobian.
- **Refuted.** Stop time is set per segment, not per sample, so the Oct-17 claim is refuted.

*Correction.*

- Keep a persistent session per lane, reset with `IDAReInit`/`SensReInit`/`QuadReInit`, retaining KLU symbolic analysis.
- Use borrowed preallocated callback buffers.
- Evaluate the Jacobian once per (t, y) and share it between `res` and `resS`, or use the staggered corrector where qualified.
- Let IDAS choose h₀.
- Gather statistics per segment.
- Precompute the anchored CSC pattern.
- Run guards at accepted steps.
- Add a block/ILU preconditioner (RC07).
- Avoid per-sample backward re-initialization where the contract has no algebraic states.

*Verification.* Native creations per study, callback allocations (zero), Jacobian evaluations per step, and trajectories identical within tolerance.

<a id="p16"></a>**P16 — Parameter estimation and shooting ignore least-squares structure and parallelism.** (AP-07, PSE target "fitting"; confirms R1-F07)

*Implemented.*

- **Formulation.** Fits are general Ipopt NLPs; there is no Gauss–Newton/LM engine.
- **Repeated integration.** `FitOracle::objective` integrates without sensitivities, then the gradient re-integrates (`derivative_upgrades`), and prediction allowances are evaluated twice.
- **Exact Hessians.** These add a forward pass with sensitivities and 2n backward problems per direction per call.
- **Adjoint for small problems.** Gradient fits use the adjoint even for 2 parameters.
- **Serial by default.** `experiment_workers` defaults to 1. Shooting windows and integral adjoints are serial.
- **No warm state.** Shooting uses L-BFGS with no warm start. The final assessment rebuilds workers and re-integrates. Profile likelihood builds a new oracle per pin.
- **Where the campaign time went.** `sparse-fit-*` cases are steady, not dynamic.
  - The campaign's long fitting and shooting cases spent most of their time *before* the workflow invocation:
    - `shooting-single-idas-cold-1`: 40.4 s harness, 3.1 s invocation;
    - `sparse-fit-12-1`: 35.3 s, 2.4 s;
    - `fit-curved-second-cold-1`: 59.3 s, 5.1 s.
  - Their measured time is mostly P01/P02 preparation. The items above are source-established and become material once preparation is fixed.

*Correction.*

- **Formulation.** Use LM / trust-region reflective for unconstrained or bounded least squares. Use Ipopt with a Gauss–Newton Hessian when constrained. Use the exact Hessian only for covariance or on request.
- **Integration.** Integrate once with forward sensitivities per trial when parameters are few, and choose forward or adjoint automatically.
- **Parallelism.** Run experiments and windows in parallel by default under one budget.
- **Tolerances.** Tie integrator tolerance to optimizer stationarity. The 0.001 result accuracy is never loosened.
- **Reuse.** Reuse workers for assessment.

*Verification.* Integrations per accepted iterate, iteration counts, and identical estimates and covariances within tolerance.

<a id="p17"></a>**P17 — Per-solve native rebuilds and per-callback bookkeeping dominate small solves.** (AP-07, H26; confirms R1-F13 and R1-F11, which is immaterial today)

*Implemented.*

- **POUNCE** runs `app.initialize()` per solve, plus a full option snapshot into two maps, about 44 timing metrics and identity JSON.
- **Clarabel** builds a new solver and serializes its settings per solve.
- **HiGHS.** Releasing Uno/HiGHS calls `Highs_resetGlobalScheduler`, which tears down worker threads.
- **POUNCE-convex** builds a rayon pool per batch and a new FERAL interface per member, and compares options through `format!("{:?}")`.
- **Per callback:** two `stopped()` checks, an `Execution` clone (about six Arc increments), a phase `try_lock` with a clock read and an allocation, `BTreeMap` counters and two `catch_unwind`s.
- **KINSOL's residual** builds a String-keyed event map with four `SemanticId::to_string` calls per evaluation.
- **Transport** recovery is O(m²) in IDs (R1-F11). That is microseconds at today's sizes.

*Correction.*

- Keep native handles per lane, keyed by structure and structural options.
- Build events only when detail is selected.
- Use pre-resolved metric handles with atomic counters.
- Check the deadline once per callback.
- Map transport rows by index.

*Verification.* Per-evaluation overhead ≤ 1 µs in ordinary mode, and per-solve fixed cost ≤ 100 µs for small problems (aims).

### 5.5 Cross-cutting

<a id="p18"></a>**P18 — Untyped internal representations drive allocation churn.** (AP-07, H15/H16)

*Measured.*

- libc allocation and copying take 54% of process cycles (perf DSO).
- `IndexMap<String, serde_json::Value>::insert_full`, `sort_all_objects`, SipHash over `String`, and `HashMap<String,String>` clone/eq are among the top self symbols.

*Implemented.* `serde_json::Value` serves as an internal IR in four places:

- fingerprints (`canonical_json`);
- declaration masking (`expressions::Environment::new`, a JSON round trip per declaration);
- source persistence (`ObjectEdit` payloads);
- progress events.

Workspace feature unification turns on `preserve_order`. Hot maps are keyed by `String` with `format!` lookups (`p.names`).

*Correction.*

- Use typed IR with streaming `FramedHasher` digests over typed structs.
- Use interned symbols and IDs on hot paths.
- Keep JSON only at boundaries.
- As a cheap interim, evaluate a faster global allocator (for example mimalloc), measured, not assumed.

*Verification.* Allocation counts per admission (heaptrack) and the libc share of cycles.

<a id="p19"></a>**P19 — Diagnostics lose phase boundaries and flood the trace.** (H26, G4)

*Measured.*

- 52,822 dropped timeline packets in `cold-small-1`.
- About 2,500 log events per second.
- Compiler spans that never close.

With dropped ends and un-instrumented stages, the trace cannot establish where the ≈34 s of PR admission went. Neither the campaign summary nor R1 could attribute it; CPU sampling (§3.2) was needed.

*Implemented.*

- `Node::update` drops updates on `try_lock` contention, including phase ends.
- The ring holds 256 events; open phases are capped at 128.
- Each Salsa event triggers a formatted, locked metric update.
- Lowering, specialization and most result materialization have no spans.

*Consequence.* The instrument built to find bottlenecks could not attribute the largest one, and it adds hot-path cost (P17).

*Correction.*

- Lossless phase boundaries through per-thread SPSC rings drained off-path.
- Aggregated per-kind counters and histograms.
- Per-event attribution behind a flag.
- Spans for lowering, specialization and materialization.

*Verification.* A forced-contention control loses no phase end, and `cold-small-1`'s trace closes every span.

## 6. Relationship to R1

R1 is careful and source-accurate in what it examined. Its main limitation is the one it states itself: the bundle was unavailable, so it ranked by consequence without measurement. The measurements change the ordering and add the dominant causes.

| R1 item | This review |
|---|---|
| F01 dense implicit Jacobian | Confirmed on the action route (P09). The same pattern appears in derived-family roots. Existing IDAS CPR coloring is not used for these. |
| F02 HVP complete-support cutoff | Confirmed for flat bodies only; others use reverse pullbacks (P10). |
| F03 block scans and tiny-block lifecycles | Confirmed. Measured orchestration growth is about N^1.1–1.2 to N = 192, then steeper (P06). |
| F04 KINSOL relative step test | Confirmed; plausible cause of the measured N = 64 failure (P12) |
| F05 Clarabel relative tolerance | Confirmed (P12) |
| F06 IDAS per window, callback allocations | Confirmed and extended (P15) |
| F07 serial shooting windows | Confirmed. The 40 s shooting case has one window, and 37 s of it precede the workflow invocation (P01/P02). |
| F08 incompatible batches serialize | Confirmed. Even compatible batches serialize per-member pre/post (P05). The measured convex case is dominated by per-member package admission (P04). |
| F09 frontier rescans | Confirmed; immaterial below ≈1,000 points (P05) |
| F10 Compact capture on coordinator | Confirmed; part of the single-task coordinator (P05) |
| F11 O(m²) quality rows | Confirmed; microseconds today (P17) |
| F12 compiler child per evaluator | Confirmed and wider: per region per order. Opt-in path, so not a cause of campaign times (P11). |
| F13 several updates per callback | Confirmed; plus KINSOL residual event allocation (P17) |
| F14 Uno start receipt | Confirmed and wider: Ipopt/POUNCE receipts too, and primal-only seeds get cold IPM defaults (P13) |
| C01 coupled DAE preconditioning | Confirmed (P15, RC07) |
| C02 SIMD/JIT | Retained as a later expansion after P10/P11 (RC02) |
| C03 shared bulk dispatch / durable commit | Retained; durable coordination is measured to dominate (P07) |
| Priority order | Revised. R1 put F11/F04/F14 first and F01/F02 as the first major work. The measured order is P01–P04 first, then orchestration, then numerics. F04/F14 stay early because they are correctness and receipt defects. |

From the 2026-10-17 review, the IDAS per-sample stop-time claim is refuted at this commit: stop time is set per segment. Its KINSOL `NoInitSetup(0)` observation is accurate but is KINSOL's default. Setup is reused for up to 10 iterations (`setup_interval` 10; nested implicit solves use 1). Its other claims are confirmed: Ipopt `warm_start_same_structure=no`, `MKL_CBWR=COMPATIBLE`, FERAL `fma=false`.

## 7. Change scenarios

| ID | Stimulus | Expected response in target | Today | Findings |
|---|---|---|---|---|
| S01 (PSE-S01) | Add a property model to a library | Recheck that package and its dependents; other libraries load from cache | Whole-namespace invalidation; cubic re-check of everything | P01, P02 |
| S02 | Value edit → re-solve | Bind values (≈ms); warm native solve | Preparation ≈19 ms (good); NLP restarts nearly cold | P13 |
| S03 | Open an existing flowsheet in a new process | Registry static; libraries from cache; structure from cache | ≈1 s registry plus full admission (PR/PC-SAFT `modeling.prepare` 36–41 s; heater 58 s) | P02, P03 |
| S04 (PSE-S03/S07) | Sweep 10,000 points | Points bind values; throughput ∝ cores; durable overhead ≪ point cost | Width flat; durable ≈1.2 s/point; O(N²) readiness | P05, P07 |
| S05 (PSE-S07) | Grow a flowsheet to 5,000 equations | Linear admission and preparation; numerics dominate | Admission ≈N^2.5–3.0 (cubic in source); block preparation super-linear at the top of the range | P01, P06, P09 |
| S06 (PSE-S02) | Substitute an evaluator realization (compiled/JIT) | Integration owner absorbs it; artifacts cached | Per-region compilers, no cache, SIMD/JIT excluded | P10, P11, RC02 |
| S07 | Recycle with slow contraction | Accelerated convergence by default; diagnostics report acceleration | Hundreds of substitutions | P14 |
| S08 (PSE-S07) | Interrupt during admission | Cooperative stop within bounded time; caches untouched | Cancellation propagation was AUD39-03's subject; its current status was not re-verified here. Long admissions make a timely stop depend on cancellation checks inside the checker. | P01 |
| S09 | Dynamic start and event, inside a fit or shooting loop | Persistent session per lane re-initialized consistently; one integration with sensitivities per trial; windows and experiments in parallel | New native session per window; objective then gradient re-integration; serial lanes | P15, P16 |
| S10 | Infeasible or ill-posed case | Structural refusal before solving (unchanged); numerical infeasibility recognized without exhausting fallbacks | Structural path sound. Native stopping misalignment can turn near-solutions into refusals. | P12 |

## 8. Foundations and gates

| Foundation | Verdict | Reason |
|---|---|---|
| AP-01 Separation of concerns | Satisfied | Physics, mathematics, native iteration, orchestration and results have distinct owners. Corrections stay within them. |
| AP-02 Stable contracts | **Violated (narrow)** | The KINSOL step equivalence promise (§16.6) differs from the native operation. Start receipts conflate primal and warm payloads (P12, P13). |
| AP-03 Composition | **Violated (narrow)** | Study lanes and batches compose width without throughput. Per-lane admissions duplicate work (P05). |
| AP-04 Domain model and semantic authority | Satisfied in inspected paths | Typed authored model, explicit demands and original assessment govern behavior. |
| AP-05 Explicit structure | Satisfied | Prepared plans, BTF, demand and lifetimes are explicit. They are physically mis-sized, not missing. |
| AP-06 Local reasoning/testability | Satisfied | Corrections have local oracles: `exhaustive_named_types`, fingerprint equality, derivative checks. |
| AP-07 Execution fits workload | **Violated** | Cubic admission (H11 feasibility), per-process registry, per-package re-validation, flat width, polling, n-directional Jacobians (P01–P10, P14–P16) |

| Gate | Judgment | Evidence / action |
|---|---|---|
| G1 Authority | Pass | Proposed caches are keyed accelerators. The registry stays generator-owned. |
| G2 Semantic fidelity | **Fail** | KINSOL step semantics and Clarabel feasibility translation (P12) |
| G3 Validity | Pass for inspected mechanisms | Proposed persistent caches and token reuse need their own invalidation qualification (P02, P09). |
| G4 Hidden behavior | Pass | Diagnostic loss is visible as notices. Lossless boundaries are proposed (P19). |
| G5 Consistency and recovery | Pass at inspected level | Durable redesign (P07) must preserve leases and acknowledgements. |
| G6 Transformation and reuse | Pass (baseline validity) | Defects are efficiency defects, not stale reuse. |
| G7 Truthful capability claims | **Fail** | Start receipts (P13) |
| G8 Library leverage | Pass, with under-use noted | Native libraries own iteration. Available capabilities are unused: KINSOL Anderson by default, Salsa parallelism and durability, Ipopt re-optimization (P02, P13, P14). |
| G9 Architectural fitness | **Fail** | AP-07, plus narrow AP-02/AP-03 |
| PS-G1 Physical consistency | Unresolved | Not re-qualified by this review |
| PS-G2 Well-posedness | Pass in inspected admission | Grouping (P06) must keep structural vs numerical distinctions. |
| PS-G3 Numerical integrity | Unresolved | The N = 64 root stops on the step safeguard and is refused; its reported coordinates are near-analytic. Cone tolerance translation (P12). |

## 9. Library fit

| Capability | Current | Recommendation |
|---|---|---|
| Incremental compilation (Salsa 0.28) | Whole-namespace inputs; no `rayon`; LRU 64 | Tracked per-declaration queries, durability levels, parallel queries; size-aware limits (P01, P02) |
| Registry data | Built at runtime via serde_json/sqlparser/blake3 | Codegen static arena, or a zero-copy archive (`include_bytes!` of an rkyv/bincode image) (P03) |
| KINSOL 7.1.1 | Fixed point without acceleration; scaled step test misread | `KINSetMAA`/damping defaults; stagnation-floor step tolerance; `KINSetMaxNewtonStep`; dense LU for tiny blocks (P12, P14) |
| IDAS 7.1.1 | Session per window; stock KLU | `IDAReInit` family; symbolic retention; preconditioner callbacks (P15) |
| Ipopt / POUNCE | `warm_start_same_structure=no`; cold primal seeds | Warm-start options and restart profile; retained applications (P13) |
| Clarabel 0.11 | Relative tolerance used as absolute | Translated tolerances with bounded refinement (P12) |
| Symbolica 3.0.1 | Evaluator per tiny region; C++ export per region | One `evaluator_multiple` over fused outputs; one export unit per artifact; JIT later (P10, P11) |
| Sparse coloring | Greedy CPR only in the IDAS adjoint coupling block; POUNCE CPR finite-difference Hessian option | Generalize the existing greedy coloring to implicit F_y and derived-family Jacobians over existing CSC patterns (P09) |
| Allocator | glibc malloc | Measure mimalloc as an interim (P18) |

## 10. Alternatives

| Alternative | Serves | Trade-offs | Selection |
|---|---|---|---|
| Current baseline | Correct small cases | Cubic admission; ≈1.5 s fixed per process plus package; width unused | Not fit |
| **Simplest viable:** local fixes only (memoize registry descriptions and make `BOOTSTRAP` lazy; per-declaration `named_types` memo; trusted-schema fast path; KINSOL/cone/receipt fixes; Anderson default) | Removes most cold cost for one process; no rule changes except RC03/RC06 | Libraries are still re-admitted per process; structural edits still re-check everything; numerics unchanged | **Recommended first wave.** Revisit if a cubic term persists after it. |
| **Proposed target** (§4) | All target workloads to industrial size | Larger change to compiler incrementality and caches (RC01), evaluator realization (RC02) and orchestration | Recommended destination |
| Library-owned: delegate model compilation to an external algebraic modeling layer (CasADi/Pyomo style) | Fast expression graphs | Loses typed physical authority, admission semantics and diagnostics in original terms; the binding's model authority rule | Rejected |
| Library-owned: run everything through DataFusion | Uniform relational engine | Wrong engine for per-declaration name resolution and small document validation (P04) | Rejected for those paths; kept for bulk queries |

## 11. Priorities and prerequisite order

Consequence and dependency differ. The waves below give dependency order; each wave's items are independent of each other.

| Wave | Change | Findings | Acceptance evidence |
|---|---|---|---|
| 1 — remove the walls (no or minor rule change) | Registry memoization and lazy `BOOTSTRAP`, then static codegen | P03 | Fingerprint equality; ready ≤ 50 ms |
| 1 | Indexed lazy lexical resolution; candidate-scoped admission memo; typed declaration masks | P01, P18 | Sweep exponent ≤ 1.2; oracle equality |
| 1 | Trusted-contract fast path; `DomainEnvironment` once; templates per relation | P04 | ≤ 20 ms per small package; refusal equivalence |
| 1 | KINSOL stagnation semantics, polish step, explicit max step; cone tolerance translation | P12 | Banded 64; SOC/PSD; scaled/shifted probe |
| 1 | Primal warm-start transport and truthful receipts | P13 | Iteration ratio; receipts |
| 1 | Anderson default for recycles (RC06) | P14 | Stagnating fixture iterations |
| 1 | Lossless phase boundaries and spans for lowering/specialization/materialization | P19 | Contention control; closed spans |
| 2 — reuse and parallelism | Persistent per-package interface and artifact cache; fine-grained queries; parallel checking (RC01) | P02 | Second-process load ≤ 100 ms; dependents-only recheck |
| 2 | Spawned lanes, shared admission, event frontier, persistent pool, batch partitioning | P05 | ≥ 0.8 × cores throughput |
| 2 | Indexed blocks, physical grouping, session reuse by shape | P06 | Flat per-block cost |
| 2 | Computed reservations | P08 | Accounted ≈ RSS order |
| 2 | Durable batch claims, incremental readiness, notifications | P07 | ≤ 20 ms per point |
| 3 — numerical engine at scale | Coloring and token-based linearization reuse (RC04) | P09 | JVPs = colors |
| 3 | Fused evaluators, value reuse across orders | P10 | Calls per iteration |
| 3 | One-unit compiled artifacts plus persistent cache, then compiled default | P11 | Zero warm compiles |
| 3 | IDAS session reuse, shared Jacobian, per-segment statistics, preconditioner (RC07) | P15 | Creations, Jacobian evaluations per step |
| 3 | LM/GN fitting, sensitivity choice, parallel experiments/windows | P16, R1-F07 | Integrations per iterate |
| 3 | Native handle retention; lean callbacks | P17 | ≤ 1 µs per callback |
| 4 — expansion | Batched/SIMD/JIT evaluation across points (RC02); process-isolated PETSc/Uno lanes; GPU only for coherent large batches | R1-C02, R1 concurrency limits | Separate qualification |

Prerequisites:

- P02's persistent cache needs P01's interface/lookup design, so that interfaces are small and stable.
- P10's fusion should precede making P11's compiled path the default.
- P05's shared admission benefits from P04.
- P09's token-based reuse needs the complete linearization association R1 describes.

## 12. Verification and settling experiments

| Claim | Label | Settling evidence |
|---|---|---|
| Admission grows ≈N^2.5–3.0 and is dominated by `named_types` | Measured (sweep N = 16–256, lldb); cubic structure Implemented (source) | After P01, the same sweep to N = 2,048 plus counts |
| Registry ≈0.94 s per process | Measured (gdb) | Ready time after P03; fingerprint equality |
| Package admission ≈0.5 s | Measured (lldb, POUNCE-convex 8) | Per-package time after P04 |
| Long fitting/shooting/QP cases are preparation-bound | Measured (campaign invocation endpoints, §3.1) | Same cases after P01–P04 |
| Width yields no throughput | Symptom measured (16 points; paired 4 points); mechanism Implemented (source) | A heavier per-point study at 1/4/8/16 workers after P05 |
| KINSOL step semantics cause the N = 64 refusal | Strong inference (source + measured candidate) | The R1 scaled/shifted probe; banded 64 after P12 |
| Recycle acceleration ≈10× fewer iterations | Hypothesis (standard theory) | Stagnating fixture with m = 3–5 |
| Representative-flowsheet behavior | Not examined | Build the north-star flowsheet benchmark (§4.5) |

**Model conformance (PS-13).** No unit or property model is changed by this review. The proposed admission cache, lazy resolution and evaluator fusion must leave every model's checked package and evaluated outputs identical. The existing model checks and parity references are the acceptance evidence for that equivalence.

Re-measure on a quiesced host with the release (not force-validate) profile before quoting speedups. The figures in §3 are single observations on a profiling build with Arrow force-validation. Arrow validation frames (`ArrayData` construction) were ≤ ≈3% of samples. The dominant costs are production code paths, not force-validation.

## 13. Rule impacts

These are proposed changes. None takes effect until plan creation confirms it with the maintainer.

| ID | Rule and location | Proposed change | Depends | If kept |
|---|---|---|---|---|
| review#RC01 | Blueprint §14.4: "There is no persistent Salsa store; a new process rebuilds from the admitted revision"; "Mutable admission has no retained owner-only type cache" | Admit a persistent, content-addressed cache of package interfaces and checked artifacts, plus fine-grained incremental admission queries | P02 (and P01 memo scope) | P01 still applies in-process; every process keeps re-admitting libraries |
| review#RC02 | Blueprint §14.5 limits (JIT/SIMD evaluators, GPU, distributed not admitted); ADR-0191/0193 scalar scope | Admit qualified batched/vectorized/JIT realizations with explicit identity and conservative FP | P10/P11 expansion, R1-C02 | Scalar fusion and caching only |
| review#RC03 | Numerical-execution §16.6 KINSOL row: "the scaled-step tolerance is `feasibility`, so KINSOL's residual and step tests reduce to the original per-row and per-coordinate budgets" | State and implement the step tolerance as a stagnation safeguard; acceptance stays residual + original | P12 | The documented equivalence remains false |
| review#RC04 | Blueprint §14.3: "Compatible factor/first-product upgrades require actual matrix identity"; provider effects rerun on equal input | Reuse a linearization when complete child linearization tokens match | P09, P10 value reuse | Coloring still reduces construction cost; reconstruction per action remains |
| review#RC05 | Numerical-execution §18.3: every Ipopt application sets `warm_start_same_structure=no` | Qualify structure-retaining re-optimization when the structure signature matches | P13 | Warm starts limited to start points and duals |
| review#RC06 | Numerical-execution §17.4: "Current damping/Anderson defaults remain" | Anderson default for recycles, reported explicitly | P14 | Plain substitution by default |
| review#RC07 | Numerical-execution, *Root primal transformations and linear choices*: "No coupled index-1 DAE BlockFactor/Schur … is newly qualified"; the IDAS adapter refuses BlockFactor | Admit a qualified coupled preconditioner | P15 | None/Jacobi or direct KLU only |
| review#RC08 | The ordinary-observation freshness contract identified as R1-RC03 (cumulative callback facts published at every safe return) | Lossless boundaries with aggregated, coalesced facts | P17, P19 | Per-return publication cost remains |

## 14. Coverage and limits

**Examined.**

- The campaign bundle: matrix, outcomes, paired summaries, timelines and CPU excerpts.
- Re-executions of 15 ephemeral cases and sweeps.
- Source of the compiler, mathematics, native adapters, orchestration, dynamics and fitting owners, through focused reviews; decisive facts were re-read by the principal reviewer.
- The vendored SUNDIALS sources.

**Not examined or not re-run.**

- Canonical-store cases.
- Python-path profiles beyond the campaign's observer probes.
- Release-profile timings.
- Large coupled sparse numerics: admission currently prevents reaching them.
- PETSc/SCIP/HiGHS internals.
- Physical-consistency qualification.

**Proposed versus established.** Every "aim" and every correction is *Proposed*. Measured figures are single descriptive observations, not a statistical comparison. The disposition of each finding belongs to the plan that adopts it.
