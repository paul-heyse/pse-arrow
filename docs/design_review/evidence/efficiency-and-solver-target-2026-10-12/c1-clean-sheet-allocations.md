> Supporting evidence for [the efficiency and solver target review](https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/design_review/reviews/design_review_efficiency-and-solver-target_2026-10-12.md). Inquiry C1 (design-reviewer): clean-sheet responsibility allocations, designed without reading the code, plans, ADRs or prior reviews. Baseline: `main` at `6c368c88a`. This contribution is preserved as returned; its local labels are not finding identifiers, and the principal review owns the judgment.

# C1: clean-sheet responsibility allocations

Inquiry C1 (clean-sheet architect) of the consolidated efficiency and solver target review.
Focused advice; a bounded supporting contribution. It has no verdict, no finding IDs and no
decision.

- **What was read.** The common brief. The standard: core 3.4 (§1, AP-01–AP-07, the DP
  refinements they cite), heuristics H1–H28 and their safeguards, Process Simulator 1.5
  (functional target, PS-01–PS-13, review additions), the pse-arrow binding (PSE-S01–S07). Blueprint
  §0 and §0.2 only. The capability-surface and library-shaped-allocation sections of R1–R5. The
  design-review skill and the process-simulator profile skill.
- **What was not read.** Source code, plans, ADRs, other architecture sections, prior reviews or
  evidence, operational guides, and the "current use" sections of R1–R5. Some R1–R5
  responsibility sections mention current mechanisms in passing. Where this design meets one,
  it is listed only as an open premise in §5.
- **Evidence labels.** Every allocation, benefit and claim of removed machinery here is
  *Proposed*. Library facts are cited from R1–R5, where they are *Interface-checked* against pinned
  sources. I did not re-read those sources, so for this file they are secondhand. Nothing was built, run or measured.

---

## 0. Position in one page

A best-in-class simulator with this target has one decisive physical choice: **the live model in the
owning process is the working authority for edit → re-solve and for study execution. Durable state is
an immutable record of facts the user must keep.** The durable record holds definition revisions,
sealed results, attempt observations and study completion facts. Most expensive lifecycle machinery,
such as leases, fences, settlement, receipts, durable activity and transport drain, comes from *mutable*
lifecycle state shared across processes. It does not come from durability. Keep durable writes
immutable and content-addressed, and give each project a single writer. Then most of that
machinery is unnecessary, and the rest becomes idempotent retry by identity.

The rest of the allocation follows from five boundaries, each forced by a library contract (R2 §4 Q1, R5 §4.1):

1. **Definition → pattern → value** lifetimes. Salsa owns everything pure up to and including
   pattern-lifetime structure and compiled programs. It is the *only* reuse authority for pure products.
2. **An immutable `PreparedProblem` bundle** is the handoff from Salsa to execution. Studies, workers and
   native sessions consume the bundle, not the database. Edits never wait for running work.
3. **Values bind into slot vectors**, not into Salsa revisions, for studies. An interactive value edit is one
   low-durability input write.
4. **A lane** is one long-lived OS thread that owns native sessions, with a declared core width. A lane is
   the unit of native-state ownership, capacity charge, cancellation and drain. Library thread counts
   are derived from the lane width at entry. Ambient global pools are never used.
5. **Results and observations are Arrow in memory first.** Durable publication writes immutable Parquet
   files sealed by one catalog commit. DataFusion serves analytics and inspection. It is never in
   the authoring-validation or method-admission path.

Two allocations follow. **L, "live model, immutable record"**, is the recommended allocation under the
stated assumptions. **S, "shared durable coordination"**, is what changes when several independent
writers or hosts must coordinate through shared durable state. S is given as deltas from L, so the
coordinator can see which machinery each assumption buys.

---

## 1. Assumptions

| # | Assumption | If it fails, what changes |
|---|---|---|
| AS1 | One user on a local workstation: 32 logical / 16 physical CPUs and about 188 GiB, shared with other workloads. No hostile writers. | Multiple users or hosts lead to S (§4). |
| AS2 | At any time, **one owner process** per project holds durable write authority. It is the interactive Python process by default, or a detached study runner when a study must outlive the session. | Several independent writer processes lead to S: a store server, store-native uniqueness on claims, and leases only for mutable shared state. |
| AS3 | Product concurrency covers: user edits and re-solves in one session; studies running beside the session, either as in-process lanes or as child worker processes of the owner; and read-only inspection of published results from other processes. | Concurrent mutation of one project from two sessions leads to S. |
| AS4 | Failure domains: a Rust panic (`unwind`, recoverable); a native crash (process loss); OOM or SIGKILL; power loss; user interrupt. Native crashes are possible but not routine. | If native crashes are routine for a method family, that family runs in isolated worker processes by default (L-variant P, §2.5). |
| AS5 | Published results are immutable. Definitions are immutable revisions with mutable named heads. | Mutable results would need MVCC or protection protocols, and so S-like machinery. |
| AS6 | A durable study must survive interruption and resume without recomputing committed points. It need not keep running while **no** owner process is alive, except through a detached runner. | Work that continues across host loss or on remote workers leads to S. |
| AS7 | Process-global native constraints hold at the pins: the PETSc build is not thread-safe, Ipopt serializes MUMPS process-wide, and a HiGHS executor is fixed per thread until reset (R4 §0.3, §1.10). | A thread-safe PETSc rebuild, for example, moves PETSc methods from "single lane or process lane" to "any lane". |

---

## 2. Allocation L: live model, immutable record

### 2.1 State: what lives where, at what grain

| State | Durable? | Shared or queryable by whom | Grain | Process-local form and lifetime |
|---|---|---|---|---|
| **Authored definitions and revisions**: units, property and reaction packages, flowsheets, study definitions, method profiles, parameter data with provenance | Yes. A canonical serialized form, content-addressed. Named heads are the only mutable pointers. | Lineage queries by the owner; read by workers as a snapshot | One revision per definition; heads per project | Fine-grained Salsa inputs: per declaration and per field, with separate value inputs. Lives for the session. |
| **Prepared structure**: specialization, incidence, DM/BTF, block DAG, form facts, colorings, symbolic factorizations, slot layout, scaling policy, problem class | No. Derived, rebuildable. | Not shared; worker processes re-derive it | Per pattern, and per weakly connected component where useful | Salsa memos behind `Arc`, LRU per function. Frozen into a `PreparedProblem` for execution. |
| **Evaluators**: optimized programs per distinct template body, Dualized derivative programs, optional JIT code | No. Optionally kept in a content-keyed disk cache for worker or cold start. | Not shared | Per body content, demanded outputs, derivative order and backend profile | `Arc` program in Salsa; per-thread scratch in the lane; JIT code shared through `Arc<Memory>` (R2 F4). |
| **Native sessions**: Ipopt or POUNCE applications, KINSOL/IDAS with their contexts, factorizations, KLU pivot orders | Never | Never | Per lane, keyed by (pattern key, method profile, program keys) | Owned by the lane thread, which is thread-affine and `!Send` (R4 §2.2). Retained for warm re-solve; dropped on lane end or eviction. |
| **Value bindings and starts** | Only as inputs recorded with the results they produce | — | Per case or point | A dense slot vector over the prepared layout. A start carries provenance: authored, previous attempt, or neighbouring point. |
| **Study points** | The study definition and its point generator are durable. **Point completion facts** are immutable appends sealed with the point results. | Owner. Progress is *derived* from completion facts. | Per point; commits are batched per flush | An in-memory work queue of pending points. Nothing durable while a point is in flight. |
| **Results** | Yes. Immutable Parquet files plus one catalog record per result set, sealed by one commit. | Any reader, through committed manifests | Per result set: an interactive solve, or a study flush batch | Arrow batches in memory, handed to Python zero-copy. "Publish" is a sink operation off the return path. |
| **Attempt and observation records** | Yes. An append-only Parquet log with T0 rows (one per attempt) and an optional bounded T1 per-iteration list. | Analytics (DataFusion) and the history layer | Per attempt, flushed with its results | Typed in-memory evidence during the solve. Layer 2 reads the live form; the persisted record is a copy. |
| **Method-history records** | No separate authority. A **derived** aggregate over the observation log. It may be materialized as a rebuildable Parquet summary when the log grows. | The advisory layer | Per (feature signature, method profile and version) | An in-memory index built at session start and updated as attempts complete. Its snapshot identity is recorded at each use. |
| **Diagnostics** | Yes when attached to a published outcome; otherwise recomputable | Python and analytics | Per attempt or per prepared problem | Typed diagnostic values: code, severity, stage, domain identities, observed values (DP-21). Structural diagnostics are Salsa products. |

Content addressing gives the durable record two properties. **Idempotent publication**: a retry of the same commit is detected as already present. **Lineage without the store on the solve path**: a result references the revision hash, and publishing commits that revision first if it is absent.

### 2.2 Owners

| Responsibility | Library capability used | Bespoke composition the project owns, and why no library owns it |
|---|---|---|
| **Authored model** | Symbolica atoms for equations (R2 F2). petgraph for topology with typed edges. Typed validation at the Python→Rust boundary. | The physical type system: dimension, basis, reference state, gauge/absolute and sign conventions (PS-01). Material systems and envelopes (PS-02). Balance generation from contributions (PS-03). Variable roles (PS-04). Domain obligations captured *before* simplification (PS-06). This is domain meaning. A runtime-authored unit system also needs more than compile-time dimension crates offer. |
| **Preparation, reuse and invalidation** | Salsa 0.28: fine-grained inputs, durability tiers, backdating, per-key single flight, cancellation, LRU (R2 F1). Structure from pounce-presolve (matching, DM, BTF, coupling, FBBT, warm maps), KLU/BTF, petgraph/rustworkx (condensation, `layers`, `longest_path`, colouring), faer and KLU symbolic factorization (R2 F6–F8). Symbolica form facts (R2 §3). | The query graph and the `PreparedProblem` freeze. Durable content identities, because Salsa ids are not durable (R2 Q2). Choosing one structure producer per fact and keeping a deliberate cross-check as an independent oracle (R2 A3). |
| **Evaluation** | Symbolica/Numerica optimized evaluators (Horner, CPE, non-inlined `FunctionBody`). Dualizer derivative programs. SIMD `f64x4/x8` lanes. JIT `batch_evaluate` as a qualified option. `ErrorPropagatingFloat`/`DoubleFloat` re-evaluation for diagnosis (R2 F3–F4). num-dual `implicit_derivative*` for opaque inner solves. FeOS `State<D: DualNum>` behind the provider contract (R2 F5, F9). | Program per distinct body, with **parameters as evaluator inputs, not baked constants**, so a parameter edit never recompiles. Scatter maps from instance outputs into the global sparse pattern. Guard and domain stages, and lane divergence falling back to scalar. Qualifying the determinism class of each backend profile (DP-11: FMA, `symjit.toml` isolation). |
| **Method choice (all three layers)** | None owns cross-library choice (R4 §5). Within one solve, libraries own their own composition: Ipopt restoration, PETSc NPC/COMPOSITE, POUNCE ladders. | The method-choice domain model in §2.4. It is pure and testable from features alone (PSE-S05). |
| **Native execution** | Class-specific libraries. Square: KINSOL with KLU refactor, PETSc SNES (TR, NLEQERR, QN, Anderson, VI, NEWTONAL), pounce `block_solve` for blocks of size 8 or less, argmin Brent, Symbolica root isolation. NLP: Ipopt (MUMPS, SPRAL or Pardiso, never HSL), POUNCE/FERAL (sessions, batches, sensitivities), Uno. Least squares: TAO BRGN/POUNDERS. Dynamics: IDAS (`IDACalcIC`, sensitivities, roots), Diffsol, CVODES. LP/MILP/QP: HiGHS. Conic: Clarabel. MINLP: SCIP. | One adapter per library family as its integration owner. Each adapter provides: a capability declaration; a status → typed outcome map (PS-10); a statistics → typed observation map; the library's own cooperative stop; and its warm and retained session kinds. Bounded domain composition only where no library supplies the whole contract (PS-09): block sequencing, parameter continuation stepping and multiple-shooting orchestration. |
| **Observation and measurement** | Typed statistics where libraries provide them: FERAL `FactorStats`, POUNCE `SolveStatistics`/`IterRecord`, PETSc convergence history, IDAS getters, HiGHS info, KLU `rcond`/`condest`/`rgrowth`. Ipopt `IpoptData` read inside the intermediate callback. tracing spans for phase timing, folded into rows by a typed `Layer` (R3 S2). | One tiered attempt schema (Arrow) covering decisions, history and efficiency telemetry (R3 §4.3). Its adapter-specific part is a per-adapter struct column. Truncation and drop accounting are explicit. tracing is never the source of decision data. |
| **Results and analytics** | Arrow with extension types carrying physical meaning through IPC, Parquet and C Data (R3 A6). Parquet with statistics, page index, projection and `RowSelection` (R3 P1–P2). A DataFusion `TableProvider`/`ListingTable` over committed manifests, with aggregates, windows, percentiles and spill under a bounded `MemoryPool` (R3 C1–C5). statrs for confidence intervals. pyo3-arrow C stream for Python (R3 A3). | The registry of result schemas, derived from the domain types. The manifest-to-provider seam. Result identity on decoded values rather than encoded bytes, which keeps the Parquet encoding free. Validity conditions for derived quantities (PS-12). |
| **Persistence and retention** | **Catalog:** SurrealDB 3.3 *embedded* in the owner (SurrealKV or RocksDB). It gives transactions, UNIQUE identity, `REFERENCE ON DELETE REJECT` for pinned result sets, and a directory lock that enforces one owner (R1 F1–F3, F10). **Bulk:** Parquet files through object_store `LocalFileSystem`: temp file, then hard-link or rename (R3 §1.5). | Explicit fsync of the file and the directory, because object_store does not fsync (R3 §1.5). The publication order. An orphan sweep at owner start. Retention policy: pins, recency, and partitioning and compaction of the history log. |
| **Capacity and parallelism** | rayon scoped pools (`build_scoped`, `spawn_handler` for ICVs), faer `Par::Rayon(n)` inside the lane pool, library entry counts (OpenMP/MKL local, HiGHS `threads`, FERAL `parallel`, SCIP threads, Symbolica `n_cores`), cgroup/affinity, owned `Semaphore` for the reserve (R5 §1, §2). | The lane model and width planning in §2.5. Per-lane memory admission from pattern facts, because native heaps are opaque (R5 §4.1). |
| **Cancellation and recovery** | `CancellationToken` trees. `TaskTracker` tokens held by lane threads until join. Each library's cooperative hook. Embedded-catalog drop-cancellation. pidfd or pipe EOF to detect child death (R1 F17, R5 §4.2). | Polling the token at native checkpoints. Mapping stop fidelity per adapter. Resuming a study from completion facts. |
| **Python boundary** | pyo3: `frozen` `Send + Sync` pyclasses holding handles; `detach` around every wait; `check_signals` polling. pyo3-async-runtimes: an asyncio cancel drops the future. pyo3-arrow: zero copy (R5 §1.6, §4.3). | A `Session` handle to the owner thread that holds the Salsa database and lanes. Signals and asyncio cancels become the session's token. Inbound value vectors are copied once into slot storage, because they are small. |

**IDAES capability families (§0.2) under this allocation.** Blocks, flowsheets, ports, balances, the unit library, property and reaction frameworks, and costing are authored definitions through the same Salsa preparation. Helmholtz, CoolProp and FeOS are provider contracts with envelopes and derivative orders. Initialization and homotopy are plans over scoped overlays (§2.4). Scaling is a resolved policy in the `PreparedProblem`. Diagnostics are the toolbox in §2.4. Dynamics are native integrators over the same definitions. Estimation and sweeps are study workflows. Serialization and tables are the canonical definition form plus the Arrow results. No family needs its own persistence, reuse or execution path.

### 2.3 The one reuse authority and edit propagation

**Salsa is the single reuse authority for every pure product**, from parse through specialization, structure, form facts, colorings, symbolic factorizations and compiled programs (R2 A1 + A3 composed). Value-lifetime native state is not reuse in the cache sense. It is **retention owned by a lane**, keyed by Salsa-produced identities. A warm start or a reused numeric factorization is a recorded input of the result it influences (PS-11), never a cache hit. An optional disk artifact cache sits in front of program compilation. It is a content-keyed memo: its key includes body content, settings and library versions. It is not a second authority, and only workers and cold starts consult it.

**How an edit propagates (Proposed):**

1. **Value edit**, such as a parameter, fixed value, bound or start. Python sends the edit to the owner thread. That thread sets one LOW-durability Salsa input and starts a new revision. Structure memos depend only on HIGH inputs, so they verify without walking their dependencies (R2 F1). The binding query re-packs the slot vector, and programs are untouched because parameters are evaluator inputs. The lane holding the session for this pattern key re-solves warm: Ipopt `ReOptimizeTNLP`, POUNCE `solve_warm`, KINSOL with retained setup, or KLU `refactor` behind an `rcond`/growth guard. *Rebuilt: the binding vector. Repeated: native iterations, with an independent post-solve check of the new numerical state (PS-10, PS-13).*
2. **Expression edit** inside one template body. That body's program query re-runs. Every other body's program verifies cheaply. If the incidence pattern is unchanged, the pattern products backdate and the lane keeps its symbolic factorization.
3. **Structural edit**, such as adding a unit, reconnecting a port or fixing or freeing a variable. The specialization re-runs and produces a new pattern key. Pattern products are recomputed **per weakly connected component**: a component whose content is unchanged backdates (pounce `SquareComponents`, R2 F7). Global BTF over a changed component is legitimate global work. The lane's retained session is keyed by the old pattern, so it is not reused. A warm *start* can still be translated through identity maps, recorded as a translated start.
4. **Concurrency with running work.** A study froze its `PreparedProblem` and released every database handle before it executed. Salsa writes block until outstanding handles drop (R2 F1), so this freeze is what lets edits proceed during a study. An edit cancels only *in-flight preparation*: Salsa cancellation stores no memo (B022), and the newest edit wins.

### 2.4 Method-choice domain model

**Concepts and lifetimes**

| Concept | Content | Lifetime and owner |
|---|---|---|
| `MethodProfile` (authored policy data, versioned) | Library and version; typed options; a **capability declaration**: problem classes, derivative orders required, bounds/VI/integrality support, warm-start kinds accepted and produced, stop fidelity, statistics fidelity, **thread-safety class** (`PerThread` / `PerContext` / `ProcessExclusive`), determinism class, width sensitivity | Definition lifetime |
| Selection features | **Definition facts**: per-block algebraic form, smoothness and domain guards, derivative availability, provider envelopes. **Pattern facts**: problem class (PS-09, derived), DOF, DM parts, BTF block count and sizes, largest SCC, coupling class, sparsity, fill bound, bounded and integer sets, DAE index. **Value facts**: bound activity, FBBT enclosures, start provenance and residual at start, scaling magnitudes, condition estimate at start. | Definition and pattern facts are Salsa products. Value facts are computed at binding. |
| Execution context | Lane width, concurrent lanes, process kind, requested determinism class, budget | Per study or solve |
| **Admission** (layer 1, hard) | A pure function of features, context and profiles. It returns the admitted set, plus a typed refusal reason per excluded profile: class mismatch, derivative order, unsupported bounds, process-exclusive under multi-lane, and so on. | Memoizable per (pattern key, context) as a Salsa query |
| **Preference** (layer 1, ordering) | Authored rules with stated mathematical rationale that order admitted profiles into tiers. Examples: an affine block goes to direct sparse LU. A one-variable polynomial goes to certified root isolation with a bound-selected branch. A bracketed scalar goes to Brent. A block of 8 or fewer goes to pounce `block_solve`. A large smooth SCC goes to Newton with KLU or Krylov. A bounded square system goes to VI or IPM. A KKT system under multi-lane goes to SPRAL, Pardiso or FERAL, not MUMPS. | Pattern lifetime, with value refinements |
| **Plan** | Steps, each a profile with options and an observer policy. **Cause-typed transitions** `(step, Cause) → (step′, StartTranslation)`. Finite caps on transitions and on budget (PS-09). | Per solve; recorded in full |
| `Cause` | Typed; never parsed from strings. Terminal (from the status map): converged, locally infeasible, iteration or time limit, restoration failure, evaluation domain error naming the equation or variable, linear-solver singular. Observer: stagnation, excessive inertia corrections, divergence. Verification: unscaled residual, bound, domain or closure failure. External: cancel, budget. | Attempt |
| **Observer policy** (layer 2, in-solve) | A predicate over the typed evidence window, run inside the library's per-iteration hook. It is cheap and deterministic given the evidence, and it returns `Stop(cause)`. Within-solve switching that a library owns (Ipopt restoration, PETSc NPC/COMPOSITE) stays library-owned; layer 2 adds only switches *between* attempts. | Attempt |
| **Advice** (layer 3, advisory) | Reads the in-memory history index for the feature signature. It may only **permute profiles within one preference tier** or **tune declared-tunable options within declared ranges**. Invariant: admitted set after advice = admitted set before. | Recorded per attempt: history snapshot id (the catalog commit up to which observations were aggregated), the permutation and tunings applied, and the effective plan. Replay uses the recorded plan, not the history. |
| `AttemptRecord` | Identities (attempt, point, study, revision hash, pattern key, program keys, profile and version), feature snapshot reference, context, advice used, start provenance and reuse kinds, T0 summary, T1 bounded trajectory, outcome, cause, independent verification, phase timings | Attempt → durable log |

**Cooperative stop.** There is one contract: `StopRequest{cause}`. The same request serves user cancel, budget expiry and layer-2 policy. Each adapter maps it to its library's own mechanism and declares a **stop fidelity** (R4 §1.11):

- **Retains iterate and statistics:** Ipopt intermediate callback returning false, POUNCE, Uno returning 0, PETSc user reason, HiGHS interrupt, SCIP interrupt, Clarabel callback.
- **Retains iterate at step boundaries:** IDAS `IDA_ONE_STEP` loop, Diffsol `step()`.
- **Boundary-only, iterate uncertain:** KINSOL 7.1.1, whose evaluation latch is a failure.

The plan uses stop fidelity to decide what a transition can carry. A stop is never thrown through a bridge, because throwing discards what the library retains.

**Per-block and decomposed execution.** A `BlockPlan` covers the BTF DAG and gives each block a plan chosen from its own features. The sequencer is bounded domain composition (PS-09). It solves blocks in topological order, wavefront-parallel within the lane pool (rustworkx `layers`). Each block failure is a typed cause with a declared escalation: a stronger block method, merging with adjacent blocks, or a simultaneous whole-system solve. Following IDAES practice, a final simultaneous polish follows a BT initialization (R4 §2.3). Library-owned decompositions are used where they exist: PETSc NASM/ASPIN and field-split, the POUNCE Schur and multi-RHS paths, and SCIP concurrent racing. The project orchestrates only independent native solves, such as multiple-shooting segments, each with its own `SUNContext`.

**Recycles and tears** are a declared policy (PS-08). Equation-oriented, a recycle is an SCC, and its plan is chosen like any block's. Sequential-modular, tear candidates come from topology (petgraph greedy feedback arc set, recorded as a heuristic decision, DP-07). The fixed point is library-owned: KINSOL fixed point with Anderson, or PETSc Anderson/NGMRES/QN. Iteration history is part of T1.

**Initialization and homotopy** are plans over **scoped overlays**. A temporary fix or relaxation is a different specialization, so it gets its own pattern key, memoized once per study. It is never an edit, and it is restored by construction (DP-05, PS-08). Continuation uses library predictors: POUNCE sensitivities, PETSc NEWTONAL, or num-dual implicit derivatives. The project owns only the explicit stepping caps.

**Diagnostics as product capability.** Some checks always run and are cheap: DOF, structural rank and DM parts naming model elements (PS-04), and the post-solve independent verification of residuals, bounds, domains and closure (PS-10). Deeper analyses run on demand, as pure functions over the `PreparedProblem` plus attempt evidence (H6): faer `partial_svd` extreme singular values mapped to variables and equations, KLU `rcond`/`condest`, FERAL κ₁ and `n_tiny`, FBBT enclosures, rounding-error re-evaluation, scaling sanity and residual ranking. Results are typed diagnostics, exposed to Python as Arrow tables.

### 2.5 Capacity and parallelism model

The library facts in R5 §2 and §4.1 decide the shape. No library lends or borrows threads. Every team is sized by its caller at entry, or self-sized from the environment. Memory is opaque except in DataFusion. So the model is **one CPU budget per process, split at entry by construction, with no runtime brokerage across libraries**.

- **Host.** The project's process tree runs in one cgroup slice with a CPU quota and `MemoryMax`, coexisting with other workloads. The budget B is `available_parallelism` inside the slice.
- **Case level: lanes.** At study start, study policy picks the lane count n and width w from the case count and per-case pattern facts (largest SCC, KKT size), so that n × w ≤ B − interactive reserve. Lanes pull points from one in-memory queue, which gives dynamic balance under skew (H22). With 16 cases on 16 cores, each lane gets width 1. A single large case gets one lane of width 16.
- **Structure level.** Each lane with w > 1 owns a scoped rayon pool of width w, joined at lane end. Wavefront blocks, colour-group FD or dual columns, batched instance chunks and faer `Par::Rayon(w)` all run in it. Native owner threads stay *outside* rayon workers to avoid the cross-pool `install` hazard (R5 §1.1). The faer global parallelism is set to `Seq` and the rayon global pool is unused, so no ambient fallback exists.
- **Library level.** At entry, the lane thread sets every library count from w: OpenMP/MKL local threads, HiGHS `threads` (fixed per thread, so lane widths stay fixed for the lane's life), FERAL `parallel` only when w > 1, SCIP threads ≤ w, Symbolica `n_cores` during construction. Lanes are long-lived, so persistent OpenMP and HiGHS teams stay bounded in number (R5 §4.1).
- **Process-exclusive libraries.** Their thread-safety class is an **admission fact** and a capability, not a benchmark. Under multi-lane execution, PETSc in this build and Ipopt with MUMPS are admitted only on one designated serial lane, or in a process lane (variant P). Otherwise layer 1 prefers an equivalent thread-safe profile: Ipopt with SPRAL or Pardiso, or POUNCE/FERAL. The library-required exclusion lock stays inside its adapter.
- **Memory.** Before admitting lanes, conservative per-lane estimates come from pattern facts: symbolic factor bounds from faer, KLU or FERAL; evaluator workspaces; and the trajectory retention policy. If the estimates exceed the budget, the plan uses fewer or narrower lanes, so nothing waits in a queue. DataFusion analytics run under their own `MemoryPool` budget.
- **I/O and async.** A small tokio runtime owns catalog transactions, file publication, Python async and timers. It runs no numerical CPU work. Interactive DataFusion queries run on a separate runtime within their pool budget.
- **Interactive versus batch.** A reserved interactive lane keeps edit → re-solve responsive during a study. The study partitions the rest of the budget.
- **Variant P (process lanes).** The owner spawns worker processes, each pinned to a disjoint CPU and memory slice, running the same lane model inside. Library defaults then self-size correctly, except HiGHS, whose `hardware_concurrency` ignores affinity. Process-global libraries become naturally exclusive (R5 A3). The owner ships the definition snapshot and the content keys. A worker re-prepares, through the disk artifact cache when warm. Values and outcomes cross as Arrow IPC streams over a pipe. Workers write result files under temporary names; **only the owner commits the catalog**. Use P for process-exclusive families under case parallelism, for crash-prone families, or when the shared-memory benefit of in-process lanes is small.

### 2.6 Failure and recovery model

| Scenario | Must survive? | How |
|---|---|---|
| Numerical failure | Yes | Typed outcome and cause, a plan transition, or a typed partial or best-iterate result labelled as such (PS-10, PS-12) |
| Rust panic in an adapter or callback | Yes | `catch_unwind` at callback boundaries. The lane reports the failure; the session lives on. |
| User interrupt or asyncio cancel | Yes | Token → native checkpoint → cooperative stop. Outcome `Cancelled` with the retained iterate. Nothing is published as a solution. The capacity charge ends at drain, when the lane rejoins its queue, not when the caller stops waiting (DP-20). |
| Uninterruptible native work (a factorization, HiGHS QP/PDLP) | Bounded | The charge remains until the next checkpoint. In variant P, the owner kills the worker after a grace period. In process, the only option is to wait; that is a stated limit. |
| Interruption during publication | Yes | Write files under temp names → fsync file → link or rename to content-addressed names → fsync directory → **one** catalog transaction with the manifest, digests and row counts. A crash before commit leaves orphans, swept at the next owner start. Readers resolve only committed manifests. |
| SIGKILL, OOM or power loss during a study | Committed work survives | Each flush seals its points atomically. Resume re-plans from completion facts and re-runs only missing points. Point identity is the study revision, the point index and the seed, so duplicate work is harmless. |
| Native crash in an in-process lane | The process is lost | The durable record survives. A cold restart re-prepares, faster with the artifact cache. Variant P confines the crash to one worker: the owner sees pipe EOF or a pidfd event and re-dispatches its points. |
| Owner exit while a study should continue | Only by choice | A **detached runner** becomes the owner. It is still one writer, so no leases are needed. The Python session reconnects as a client for catalog operations and keeps its own live model for interactive solves. |
| A catalog write is unacknowledged (crash during commit) | Yes | Every durable write is an immutable, content-addressed append. A retry either inserts it or meets UNIQUE `AlreadyExists`, which counts as success (R1 F2). |
| A stale reader in another process after retention GC | Typed refusal | GC deletes only unpinned sets whose retention has expired, after a grace period. A reader that hits a deleted file gets a typed `ResultSetRetired`. |

### 2.7 Extension scenarios

| Extension | Owners that change | How much |
|---|---|---|
| **A unit or property model** (PSE-S01) | Authored definitions only, plus a provider adapter if the science is a new external library | One declaration. Balances, DOF, closure, derivative checks, envelopes, method admission, persistence and Python access all follow from existing contracts (PS-13). |
| **A solver or method profile** (PSE-S02) | One new adapter as integration owner: capability declaration, status map, statistics map, stop mapping, warm kinds. Plus preference rules in authored policy, and one per-adapter struct column in the observation schema. | Admission, the planner, studies, persistence and Python are unchanged. A *profile* of an existing library is policy data only. |
| **A study workflow** (PSE-S03), such as DOE, profile likelihood or rolling horizon | A new study definition kind: point generator, sink policy and summary query | It composes prepare → bind → plan → execute → publish. Analytics are DataFusion queries, and egobox `q_batch` covers library-driven proposals. No new lifecycle. |
| **A result representation** (PSE-S04) | A result schema with its extension types, and its writer | Parquet, DataFusion and pyo3-arrow carry it. Physical meaning stays with its owner. |
| **A library upgrade** (PSE-S06) | The integration owner. Library version is part of the profile, program and artifact keys, so caches invalidate by construction. History comparability is per profile version. | Local, except a type-universe family move (Arrow/DataFusion/Parquet), which moves the results plane together. |
| **Scale, skew or interruption** (PSE-S07) | No owner. Policy chooses width and lanes. | More blocks: wavefront parallelism. A larger SCC: wider lanes. More cases: lanes or process lanes. A longer trajectory: streamed Parquet row groups with bounded live retention. More history: partitioned log and compaction. Interruption: as in §2.6. |

### 2.8 Machinery L deliberately does not need

| Not needed | Reason |
|---|---|
| Durable leases, heartbeats, fencing tokens or generation guards for in-process or child work | One owner per project. The OS reports child death. Commits are idempotent by identity, so duplicate work costs time but not correctness (H25). |
| An unknown-outcome settlement or receipt protocol | The catalog is embedded, so there is no network hop. Every durable write is an immutable content-addressed append, so a retry is self-settling. |
| Durable records of transient coordination: activity, readiness, progress, admission state, maintenance | Progress is derived from completion facts. In-flight state lives and dies with its process (H24: disposable computation is not checkpointed). |
| A cross-process or host capacity ledger | The budget is per owner within its cgroup slice. Worker processes receive disjoint slices from the owner. |
| A second reuse or invalidation authority for pure products | Salsa only. The disk cache is a content-keyed memo, and lane retention keys on Salsa identities (DP-09). |
| A query engine in authoring validation or method admission | Typed Rust validation at the boundary, plus Salsa. Admission is a pure function over features. DataFusion's planner contributes to set-oriented analytics, not to these (R3 §4.2). |
| A store round trip, or per-declaration RPC, on the edit → re-solve path | The live model is the working authority. Durability is a sink. |
| Transport-level request admission, replay or drain for a database protocol | An embedded catalog has none. |
| Change notification (live queries, changefeeds) | The single owner knows its own commits. Cross-process readers poll manifests on demand. |
| Text or journal parsing of solver statistics where a typed source exists | Prefer the typed paths. Where only text exists (MUMPS, SPRAL or Pardiso inside Ipopt), record the absence (R4 §5). |
| Project-owned Newton steps, line searches, factorizations or acceleration | PS-09. Only the bounded composition named in §2.2 remains. |
| Ambient global thread pools and independently maximized library teams | Widths are derived from the lane at entry. Global defaults are disabled. |
| Compatibility and migration machinery for process-local formats | Versioning applies only to durable formats: canonical definitions, result and observation schemas, catalog schema (DP-24). |
| A separate durable method-history store | History is an aggregate derived from the observation log. Only its snapshot identity is recorded. |
| Bulk scientific payloads inside the transactional store | Parquet owns compression, pruning, analytics, direct space return on delete, and selective reopen (R3 §4.1). |

### 2.9 Open premises for L

Each of these must be checked against real code or workloads before L is adopted.

1. **Product concurrency (decisive).** No product feature needs several independent writer processes against shared durable state, or work that continues with no owner alive. *Settled by:* W-B (a)'s inventory of product writers and readers and the contracts it cites. If such a feature exists, S applies to that feature only.
2. **Salsa under concurrency.** Freezing a `PreparedProblem` and releasing handles is enough for edit-while-study, and parallel preparation of 16 distinct cases through cloned handles has no material contention. *Settled by:* a code reading of handle lifetimes, and the salsa skill's existing probes on per-key blocking.
3. **Program determinism.** Symbolica optimization with `n_cores > 1` is deterministic, or programs are built with one core. This is a reuse-correctness premise, unverified (R2 §7).
4. **Evaluator backend.** The interpreter, with SIMD lanes where guards agree, is enough. Otherwise the JIT must be qualified: FMA off, `symjit.toml` isolation, an equivalence policy (R2 F4).
5. **Parameters as evaluator inputs.** Authored parameters reach Symbolica evaluators as runtime parameters for every body class, including property kernels.
6. **Result identity.** Result identity can be defined on decoded values, so the Parquet encoding is free; and doubles stay exact under the chosen encodings (R3 S1).
7. **Catalog choice.** Embedded SurrealDB's build cost, its lack of SemVer stability, and its one-process-per-directory rule are acceptable (R1 F10). The alternatives are a single-writer manifest log over Parquet/Arrow queried by DataFusion, which needs no database, or a Delta-style table format for commit-atomic Parquet. Delta is not in the lock, and its type-universe coupling would have to be checked.
8. **Native crash rate** is low enough for in-process interactive solves. Otherwise variant P is the default for the affected families.
9. **Process-exclusive families under case parallelism.** Either a thread-safe equivalent profile exists for each mathematically required method (MUMPS → SPRAL/Pardiso/FERAL), or PETSc is rebuilt with `--with-threadsafety`, or variant P is used (R4 §1.10).
10. **KINSOL stop fidelity.** Boundary-only stops are acceptable, or SUNDIALS ≥ 7.3 is taken for `KINSetDampingFn`/`DepthFn` (R4 §1.3, §2.5).
11. **The history-layer feature signature** is stable enough to key advice: coarse bins over pattern and value facts plus profile version. *Proposed*; it needs real attempt logs.
12. **Workload shape.** Per-block method policy depends on block-size and SCC distributions in real flowsheets, and lane-width policy depends on whether one large case benefits from width. Both are workload facts, not benchmarks for adoption.
13. **Memory estimates.** Pattern facts give conservative but non-prohibitive per-lane memory estimates. The faer numeric growth has no allocator hook (R2 F6).

---

## 3. Contrasts the coordinator may use when mapping the ledgers

These are interpretations from this clean sheet. They are not findings.

- **Durable versus live authority.** In L, the canonical durable store never sits on the value-edit critical path. Wherever the W-A ledger shows durable-store work per edit, the brief's question (a) applies: which contract requires it, and is that contract a product need or a rule?
- **Mutable shared state.** Each durable *mutable* lifecycle record in the ledgers should be traced to a product feature in W-B (a). Without one, L's immutable-facts discipline removes the record and its protocol together.
- **Reuse authorities.** Any second keyed retention of *pure* products, beside Salsa, is a duplicate authority under L. Native-state retention keyed on Salsa identities is not.
- **Validation placement.** Under L, a query planner in authoring validation or admission is placement by availability rather than by fit (DP-13).
- **Capacity.** Under L, capacity comes from the libraries' own entry-time contracts, partitioned at study start. Runtime brokerage of permits across nested pools exists only where release-while-waiting is unavoidable. L avoids that by admitting process-exclusive families onto one lane or into a process lane.
- **Observation.** One typed record serves selection, history, diagnosis and efficiency telemetry. String metrics, journal text or tracing used as decision sources are mismatches.

---

## 4. Allocation S: shared durable coordination (deltas from L)

**When it applies.** Assumption AS2, AS3 or AS6 fails. That happens when independent processes or hosts must mutate shared project state concurrently, or when work must continue with no owner alive. Everything not listed here is as in L. That includes the authored model, Salsa reuse, the `PreparedProblem`, evaluation, method choice, adapters, the observation schema, the Parquet results plane, lanes and the Python boundary.

| Point | Delta from L |
|---|---|
| **State** | The catalog becomes **shared mutable coordination state**: point claims, attempt ownership, retention protections held by readers. Results remain immutable files on a shared object_store. |
| **Owners** | Persistence moves to a **SurrealDB 3.3 server**. The store natively owns: UNIQUE `(study, point)` claims; `DEFINE SEQUENCE` fencing generations; `REFERENCE ON DELETE REJECT`/`CASCADE` for result sets that readers or pins protect; synchronous `DEFINE EVENT … THROW` for legal state transitions; `GROUP BY` views for outstanding counts, mindful of hot keys; CHANGEFEED with a consumer cursor for progress readers; LIVE SELECT only as a wake-up hint (R1 A). The project still owns: lease expiry (no TTL or scheduler exists, R1 F14); guard keys for negative or range premises; conflict retry; settlement of unknown outcomes by original identity; the reaper; process supervision. |
| **Reuse authority** | Unchanged in each process: Salsa. Cross-process reuse uses the content-keyed artifact cache on shared storage. It is still a memo, not an authority. |
| **Method choice** | Unchanged. The history index aggregates the shared observation log, and its snapshot identity is a commit versionstamp. |
| **Capacity** | Adds a host-level budget across independent owners: a cgroup per client slice, or a ledger row per admitted study. The ledger is advisory accounting, because the store cannot admit CPU (R1 §3.2). |
| **Failure and recovery** | Adds client death, worker death through lease expiry and reclaim, server restart through its WAL, and unknown commit outcomes over the network, settled by identity. Immutable appends keep settlement trivial. Only claims and protections need the full protocol. |
| **Extension** | As in L. Each SurrealDB upgrade re-qualifies trigger, reference and event semantics (R1 A cost). |
| **Machinery needed that L avoids** | Leases with a heartbeat and reaper. Fencing on commit. Request deadlines and transport drain for the WebSocket protocol (R1 F10). Server supervision. Changefeed cursors with gap recovery. Hot-key partitioning for views and events. |
| **Machinery still not needed** | Durable activity, progress or readiness records beyond claims (progress stays derived). Store round trips on the edit → re-solve path, because the client's live model still serves the loop and "save" is a commit. Bulk payloads in the store. Durable records for any in-process activity. |
| **Open premises** | Which product features truly need cross-process mutation. Whether claims can be coarse (per flush batch, not per point) to bound write and conflict load. Whether an embedded catalog in a detached runner (L) covers "a study outlives its client" without S. |

**Choosing between L and S.** S is justified only by the product features in open premise 1, and only for the state those features share. A hybrid is natural. Studies that must be claimed across hosts use S's claim tables. Everything else, including single-host studies, keeps L's single-writer immutable record. The **deployment axis**, embedded versus server, is independent of the **mutability axis**, immutable facts versus shared lifecycle state. Most machinery follows from the mutability axis (R1 hybrid note).
