> Supporting evidence for [the efficiency and solver target review](https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/design_review/reviews/design_review_efficiency-and-solver-target_2026-10-12.md). Inquiry R2 (library-research), Part A: preparation, reuse and evaluation. Baseline: `main` at `6c368c88a`. This contribution is preserved as returned; its local labels are not finding identifiers, and the principal review owns the judgment.

# R2 Part A: preparation, reuse and evaluation libraries

Supporting contribution to the consolidated efficiency and solver-target review. No verdict,
finding IDs or decision. Baseline `main` at `6c368c88a`, 2026-10-12, clean tree.

**Method.** Static reading only: no probes, builds, tests or services. Sources were the pinned
registry trees under `~/.cargo/registry/src/*/` (salsa 0.28.4 and 0.28.5, symbolica/numerica
3.0.1, symjit 2.26.0, faer 0.24.4, suitesparse_sys 0.1.4 with vendored SuiteSparse 7.7 / KLU
2.3.x, num-dual 0.14.2, feos-core 0.10.1, quick_cache 0.6.24, rpds 1.2.1, petgraph 0.8.3,
rustworkx-core 0.18.1, finitediff 0.2.0) and the POUNCE checkout
`~/.cargo/git/checkouts/pounce-8316ac8e070882e5/ff63869`. The skills `salsa`,
`symbolica-faer-oximo`, `rust-graphs` and `rust-reasoning` were loaded. They pin Salsa 0.28.4
exactly. The `symbolica-faer-oximo` skill pins Symbolica 3.0.0, so every Symbolica claim here was
read against the 3.0.1 source instead. Latest releases were checked on crates.io on 2026-10-12.

**Evidence labels.** Library facts are *Interface-checked* (pinned source or skill index).
Probe outcomes quoted from skills or prior evidence keep their original label and scope.
Benefits are *Proposed*.

**Workspace pins and features**, from `Cargo.toml` `[workspace.dependencies]`:

| Library | Pin | Features as declared | Latest (crates.io, 2026-10-12) |
|---|---|---|---|
| salsa | =0.28.4 | `macros, inventory, salsa_unstable`. Default features are off, so `accumulator`, `rayon` and `persistence` are off. | 0.28.5 (2026-09-24) |
| symbolica / numerica | =3.0.1 | `integer-gmp, float-mpfr`. Default features are off, so `native_code_generation`, `serde`, `bincode` and `faster_alloc` are off. | 3.0.1 |
| faer | =0.24.4 | `std, sparse-linalg`. `rayon` is unified on through diffsol-la. | 0.24.4 |
| suitesparse_sys | =0.1.4 | `static_libraries, klu, amd, colamd, btf` | 0.1.4 |
| sundials-sys | =0.6.2 | `kinsol, klu, build_libraries, static_libraries` | 0.6.2 |
| pounce-presolve (and family) | git rev `ff638694` | — | — |
| feos / feos-core | =0.10.1 | `pcsaft`. Used only by the `xtask` oracle. | feos 0.10.2 (2026-10-12) |
| num-dual | 0.14.2 (transitive through FeOS) | — | 0.15.1 |
| quantity | =0.14.0 | — | — |
| quick_cache | =0.6.24 | — | 0.7.0 |
| rpds | =1.2.1 | default features off | 1.2.1 |
| petgraph | =0.8.3 | `stable_graph, graphmap` | 0.8.3 |
| rustworkx-core | =0.18.1 | — | 0.18.1 |
| finitediff | =0.2.0 | — | 0.2.0 |
| biodivine-lib-bdd, OxiDD, z3 | not in `Cargo.lock` | — | — |

---

## 1. Capability survey at the pinned versions

This section is neutral and organized by capability family. It does not refer to current code.

### F1. Incremental computation and invalidation (Salsa 0.28.4)

| Capability | What it does | Contract and guarantees | Limits |
|---|---|---|---|
| `#[salsa::input]` + setters + durability | Externally set roots. Only inputs have setters. `with_durability(HIGH/MEDIUM/LOW)` controls verification cost, never the answer (skill B004/B005). | A write starts a new revision. All inputs at HIGH durability allow a memo to be verified without walking its dependencies when only LOW inputs changed. A later plain `.to(v)` keeps the earlier durability. | Setters need `&mut db`, so there is one writer. A write blocks until every cloned handle has dropped (`salsa.cancellation` brief, B012). |
| `#[salsa::tracked]` fn | Pure memoized query. Reads are recorded and the result is reused until a read changes. Extra arguments are interned into a hidden key. | **Backdating / early cutoff**: if a re-executed query returns an equal value, dependents do not re-run (B001). This requires `SalsaValue + PartialEq`. `no_eq` disables it. Field-level tracking (B002). Output types must be `Send + Sync` (`src/function.rs:69`). | A plain database field read is **untracked and silently stale** (B003). The query must be pure, with no I/O or effects. Nothing measures the cost of dependency recording. |
| `#[salsa::tracked]` struct | An entity created inside a query. Its identity is its untracked fields. | Changing an identity field gives a different struct, and every query keyed on the old one starts over (B018). | It can be created only inside a tracked fn; otherwise it panics (B007). |
| `#[salsa::interned]` | Value-identity deduplication. `revisions = N` sets GC by slot reuse. | After N newer revisions an unused LOW-durability slot is **reused under a higher generation**, so an old id then names other data (B019). Values interned by a HIGH query, or outside any query, are immortal (B020/B021). | Ids are not durable identity. |
| `lru = N`, `set_lru_capacity`, `trigger_lru_eviction`, `heap_size`, `memory_usage` | Per-function memo bounds and accounting. | LRU evicts at the next revision, or immediately on `trigger_lru_eviction` (B014/B032). `heap_size` is reporting only (B031). | The limit is a count bound per function. There is **no byte-budget eviction** (only accounting), no global budget and no memory measurement in the skill. 0.28.5 changes `memory_usage` to `&mut self` ("require exclusive access", PR #1328). |
| Cancellation | A write cancels other handles. `cancellation_token().cancel()` cancels one computation without a write. `Cancelled::catch`. | Unwinds at the next query fetch with `Cancelled::Local` and stores no memo (B022/B023). A thread waiting on the same key **recomputes** instead of being cancelled (B037). An idle cancel is applied to that handle's next call (B040). Fixpoint cycles ignore local cancel (B024). | Cancellation is cooperative and needs `panic = unwind`. A long query body that calls no queries is not interrupted unless it calls `unwind_if_revision_cancelled`. |
| Parallel use | Clone the database per thread. Two threads asking for the same key: the second blocks on the first (`WillBlockOn`, B013). | Per-key query synchronization is sharded (0.28.3, PR #1273). | ParallelDatabase, Snapshot and `par_map` are the removed older API. Salsa provides no scheduler: parallel queries need caller-provided threads. |
| Cycles | `cycle_result` gives an immediate fallback. `cycle_fn` + `cycle_initial` iterate to a fixpoint. | Without a recovery option a cycle panics (B010). | Accumulators inside a fixpoint panic (B029). Convergence is not measured. |
| Accumulators (feature `accumulator`) | A side channel collected along the call graph. Accumulated values survive revalidation, and they are not part of value equality. | Diagnostics stay out of the value, so backdating is not defeated. | Not persisted (B026). Off in the workspace. |
| `specify` | Provide a value for a tracked-struct-keyed query from its creating query. | Specifying another query's key panics (B009). Excludes `lru`. | Only tracked-struct keys. |
| Events | `Storage::new(Some(callback))` delivers `WillExecute`, `DidValidateMemoizedValue`, `WillBlockOn`, `WillIterateCycle`, `DidDiscard`, and others. | Always delivered. `detailed-trace` adds tracing spans. | No timing data. |
| Persistence (feature `persistence`) | `as_serialize` / `deserialize` of the whole database through serde. | A persisted query is not re-executed after restore (B025). Restored interned ids are kept (B034). | Every persisted key and value must be serde. A persisted query over a non-persisted input **panics** at serialize (B027). A restored query re-runs once an interned dependency's slot is GC-reused (B036). Only serde_json was exercised. |
| Database lifetime | `#[salsa::db]` struct with a `Storage<Self>` field. | One storage, many handles. Memos live as long as the storage. | No partial or transactional input batches: a batch of setters is visible as it is applied, and atomicity is the caller's job (Plan 36 preparation evidence I-A1 says the same). |

**Responsibilities it could serve:** dependency-tracked semantic preparation (parse, resolve,
type, occurrence, specialization, lowering, symbolic products), early cutoff for unrelated or
equal edits, dedupe of identical requests, invalidation authority over *pure, Send+Sync,
PartialEq* values, and cooperative cancellation of stale preparation.

**Newer release.** 0.28.5 has two fixes: unvalidated interned-data and memo access is now
guarded, and `memory_usage` now requires `&mut`. No new capability.

### F2. Symbolic algebra and form facts (Symbolica 3.0.1)

- **Derivatives and series.** `derivative`. `series` (`src/derivative.rs:376`, `AtomCore::series`).
- **Polynomials.** `coefficient_list`, degree, factorization (`poly/factor.rs`), GCD, resultants, Gröbner bases with `solve` (`poly/groebner.rs:1694`).
- **Linear systems.** `solve_linear_system`, including the parametric variant (`solve.rs:1181`).
- **Univariate roots.**
  - `isolate_real_root_intervals`: complete distinct real roots with multiplicities (exact, VAS-CF).
  - `refine_root_interval`.
  - `isolate_real_roots`.
  - Float `roots` and `roots_hot_start` (Aberth; explicitly uncertified).
- **Numerical solving.** `nsolve` and `nsolve_system` (Newton) on `AtomCore`.
- **Solution sets.** `solution_set.rs` describes conditional and parametric solution sets with domain and coverage.
- **Process-global state.** The symbol registry and namespaces are global. Registration order must be fixed before parallel work (skill card `symbolica.symbols`).
- **Licence clamp.** `LicenseManager::max_threads(requested)` (`src/license.rs:679`) clamps Symbolica's own worker threads to 1 when unlicensed. It does not clamp when a key or library unlock is present. `execution_capabilities()` reports the licence state and whether `native_code_generation` is compiled in.
- **Responsibilities it could serve:**
  - form recognition: affine, polynomial or cubic structure, and closed-form solvability per block;
  - class evidence: linear or quadratic;
  - symbolic Jacobian structure;
  - explicit-assignment detection for single-variable blocks, through linear solve or root isolation;
  - series-based predictors.

### F3. Evaluator construction and optimization (Symbolica/Numerica 3.0.1)

- **Optimizing builder.**
  - `Atom::evaluator` and `evaluator_multiple` produce a builder.
  - `OptimizationSettings` (`evaluate/function_map.rs:501`) holds `horner_iterations`, `n_cores` (licence-clamped), `cpe_iterations` (common-pair elimination), `hot_start`, an `abort_check` callback with `abort_level`, `max_horner_scheme_variables`, `max_common_pair_cache_entries`, `max_common_pair_distance` and `direct_translation`.
  - Optimization is cancellable through `abort_check`.
- **Non-inlined function bodies.** `FunctionBody` holds "immutable, coefficient-independent instructions for a single non-inlined function" (`evaluate/evaluator.rs:18`). These sub-programs are shared inside one evaluator.
- **Post-processing.** `export_instructions` (for inspection or budgeting), `optimize_stack`, and `map_coeff` (from `Complex<Rational>` to the evaluation domain).
- **Taylor-mode derivatives.** `vectorize(Dualizer)` (`evaluate/dual.rs:127,521`) rewrites a scalar program into a dual or hyper-dual program, with structural zero components. Derivative programs are generated, not taped.
- **Serialization.** Optimization settings and evaluators serialize under `serde` and `bincode` (both off in the workspace).
- **Contract.** `ExpressionEvaluator<T>` is `#[derive(Clone)]` and **owns both its program and its mutable stack** (`evaluator.rs:7-16`). Evaluation takes `&mut self`, so each concurrent user needs its own copy. The API does not split an immutable program from per-thread scratch (prior L1-03).
- **Responsibilities it could serve:** template-body compilation once per distinct body, derivative-program generation, and bounded construction (abort plus export-based budgeting).

### F4. Evaluator execution backends (Symbolica 3.0.1 + symjit 2.26.0)

| Backend | Feature | Domains | Contract and limits |
|---|---|---|---|
| Interpreted `ExpressionEvaluator<T>` | none | `f64`, `Complex<f64>`, **`wide::f64x4/f64x8` (SIMD lanes)**, `DoubleFloat` (double-double), `Float` (MPFR), **`ErrorPropagatingFloat<f64/Float>`** (`evaluate/domain.rs:42-291`) | Deterministic interpreter. SIMD lanes evaluate four or eight parameter sets per pass, with every lane following the same instruction stream: there is no per-lane branching outside `Instr::IfElse`. `ErrorPropagatingFloat` estimates accumulated rounding error per evaluation. `DoubleFloat` and `Float` give extended-precision re-evaluation. Numerica `RealBall` is certified only for + − × ÷ and is not an evaluation domain here. |
| JIT: `jit_compile(JITCompilationSettings)` → `JITCompiledEvaluator<T>` | `native_code_generation` | `f64`, `wide::f64x4`, `Complex<f64>`, `Complex<f64x4>` | `evaluate`, plus `BatchEvaluator::batch_evaluate(args, out, rows)` for many parameter rows (`backend.rs:945-1116`). The machine code sits behind `Arc<Memory>` (`symjit machine.rs:15`), so a clone shares the code and copies buffers. `Memory` is `Send + Sync`, and the mmap is made executable through `PROT_EXEC`. **`Config::default()` reads `symjit.toml` from the CWD, or from `$SYMJIT_TOML`, and otherwise enables `FASTMATH` (FMA contraction), `USE_SIMD` and `PARALLEL_MUL`** (`symjit config.rs:896-918`). Symbolica starts from `Config::default()` and then applies only `opt_level`, `direct_translation` and string options (`backend.rs:728-734`). Turning FMA off would need an explicit option; that this works is not checked. The compressed IR is serializable, and settings travel with it. |
| AOT C++: `export_cpp` → `compile` (external compiler) → `CompiledRealEvaluator` / `CompiledSimdRealEvaluator` / complex variants, CUDA load settings | `native_code_generation` | f64, SIMD, complex, CUDA | Needs a host C++ compiler at run time, and loads a shared object through `libloading`. Determinism depends on compiler flags. |

- **Newer release:** none; 3.0.1 is the latest.
- **Responsibilities it could serve:**
  - the inner-loop residual, Jacobian and Hessian evaluation;
  - batched evaluation of many identical bodies (instances, collocation points, study points, finite-difference columns);
  - error-aware or extended-precision re-evaluation for diagnosis (cancellation and precision-loss detection).

### F5. Other derivative facilities

- **num-dual 0.14.2.**
  - Number types: `Dual`, `DualVec`, `Dual2`, `Dual2Vec`, `HyperDual`, `HyperDualVec`, `Dual3`, `HyperHyperDual`.
  - Drivers: `first_derivative`, `gradient`, `jacobian`, `hessian`, `partial_hessian` and third derivatives (`src/explicit.rs`).
  - **`implicit_derivative`, `implicit_derivative_binary`, `implicit_derivative_vec` and `implicit_derivative_sp`** (`src/implicit.rs:21-141`) lift a converged root to dual numbers through the implicit function theorem.
  - It is a different type universe from num-dual 0.15 (skill warning). Newer: 0.15.1.
- **finitediff 0.2.0.** Forward and central gradient, Jacobian, Jacobian–vector product, Hessian and Hessian–vector product. **Sparse Jacobians use caller-supplied perturbation vectors** (`forward_jacobian_pert`); the crate computes no column grouping or colouring.
- **Responsibilities:** derivatives for opaque providers, implicit-function derivatives for inner solves, and an independent derivative check.

### F6. Sparse and dense factorization

**faer 0.24.4.**
- **Sparse LU.** `SymbolicLu::try_new` and `Lu::try_new_with_symbolic`. The symbolic step applies a COLAMD ordering and selects supernodal or simplicial factorization through `SupernodalThreshold`, with `FORCE_SIMPLICIAL` available. The numeric step repeats the partial-pivot search on every factorization, so there is no pivot-reuse "refactor".
- **Sparse Cholesky.** LLT and LDLT with `LdltRegularization { dynamic_regularization_signs, delta, epsilon }`. `LdltInfo.dynamic_regularization_count` counts corrected pivots. There is also a supernodal **intranode Bunch–Kaufman LBLᵀ** for symmetric indefinite matrices (`sparse/linalg/cholesky.rs:3070-3457`).
- **Sparse QR** (supernodal Householder). It is **not rank-revealing** (prior `sparse-rank.md`).
- **Scratch.** All scratch is sized up front through `StackReq`/`MemStack`. Numeric factor vectors grow privately through `try_reserve`, with no allocator hook (prior F20 evidence).
- **Dense.** LU with partial or full pivoting, QR with or without column pivoting, LLT, LDLT and Bunch–Kaufman, SVD, EVD (Schur, tridiagonal) and GEVD.
- **Matrix-free (`operator`).** `conjugate_gradient`, `bicgstab` and `lsmr`. **`partial_eigen` / `partial_self_adjoint_eigen` / `partial_svd`** (Krylov–Schur) compute the eigenvalues and singular values of largest magnitude (`operator/eigen/mod.rs:1271-1396`). Smallest values need shift-invert through a factor the caller supplies.
- **Parallelism.** `Par::{Seq, Rayon(n)}` runs in the *current* rayon pool, so it honours `ThreadPool::install`. `set_global_parallelism` and `get_global_parallelism` control a process-global default (`lib.rs:1112-1141`). The `Par` docs mark both modes as "non portable across different platforms", meaning results may differ with thread count.
- **Responsibilities:** retained symbolic analysis per sparsity pattern, numeric refactorization per value, implicit-function linear solves, inertia evidence through regularization counts, rank and conditioning screens through partial SVD, and iterative least squares.

**SuiteSparse 7.7 through suitesparse_sys 0.1.4 (KLU 2.3.x, BTF, AMD, COLAMD).**
- **KLU analysis.** `klu_l_analyze` / `klu_l_analyze_given` run a BTF permutation followed by AMD or COLAMD per block. The symbolic object exposes the **block count, block boundaries, largest block, structural rank and number of off-diagonal entries**.
- **KLU numeric.** `klu_l_factor` performs a full pivot search. **`klu_l_refactor`** reuses the pivot sequence. Diagnostics are `klu_l_rcond`, `klu_l_condest`, `klu_l_rgrowth` and `klu_l_flops`. `klu_l_sort`, `klu_l_extract` and `klu_l_scale` are also available.
- **Standalone ordering.** `btf_l_maxtrans` (matching), `btf_l_strongcomp` and `btf_l_order`, plus AMD and COLAMD.
- **SPQR** is blocked at this pin. The binding includes an absent `spqr.h`, and the managed native prefixes have no SPQR or CHOLMOD (prior `sparse-rank.md`).
- **Through SUNDIALS.** `SUNLinSol_KLU` refactors after the first factorization.
- **Responsibilities:** per-pattern BTF structure facts at analysis time, cheap repeated factorization at fixed pattern, and pivot-growth and condition diagnostics.

**FERAL / pounce-linsol** (at the POUNCE rev) provide `SparseLu::factor_markowitz`, a factor with symbolic reuse, `refactor` and `condition_estimate_1`. R4 owns the KKT story; they are listed here only as a structure and factor source.

### F7. Structural analysis and presolve (pounce-presolve at `ff63869`)

These are standalone functions, usable outside the TNLP wrapper (`crates/pounce-presolve/src/*`):

- **Matching and decomposition.** `EqualityIncidence` and `InequalityIncidence`. `hopcroft_karp` (maximum bipartite matching). `DulmageMendelsohnPartition::from_matching` (under-, square- and over-determined parts). `SquareComponents` (weakly connected components). `BlockTriangularForm::of_component` (Tarjan SCC plus topological order).
- **Coupling.** `classify_block` returns an `AuxiliaryCouplingClass` from objective-gradient and inequality support.
- **Structural LICQ verdict.**
- **FBBT.** Forward and reverse interval passes, with outward rounding, over an expression DAG.
- **Row and variable reduction.** Redundant-row removal. Linear-equality elimination plan (`linear_eq_plan`/`linear_eq_elim`).
- **Small-block solves.** `block_solve` provides a damped or relaxed Newton for square blocks of at most 8 dimensions, and a `BlockSolver` trait for larger blocks.
- **Warm starts.** `warm::{PresolveMap, PresolveFingerprint, WarmPoint}` map warm points between original and reduced space and detect a stale transformation.
- **Lifetimes.** Matching, DM and BTF depend on the pattern. Coupling depends on gradient support. FBBT and redundancy depend on bounds and values.
- **Responsibilities:** structural well-posedness, block decomposition for sequential or parallel block solves, presolve and its reverse map, and warm-point projection.

### F8. Graph algorithms (petgraph 0.8.3, rustworkx-core 0.18.1)

- **petgraph.**
  - Components and ordering: `tarjan_scc` and `kosaraju_scc`, both returning **reverse topological order** (skill B002), `condensation` (`algo/mod.rs:477`), `toposort`, `has_path_connecting` and `dominators::simple_fast`.
  - Matching and colouring: general `maximum_matching` and `dsatur_coloring`.
  - Feedback arc set: `greedy_feedback_arc_set`, a heuristic. There is no exact minimum directed feedback arc set (skill brief `graph.cycles`).
  - Container trade-off: `StableGraph` loses `NodeCompactIndexable`, which blocks nine algorithms, including isomorphism and `connected_components`. `Graph` reuses indices on removal.
- **rustworkx-core** (zero-copy from petgraph): `lexicographical_topological_sort`, **`dag_algo::layers`** (topological wavefronts), **`longest_path`** (critical path), `greedy_node_color`, `max_weight_matching`, `stoer_wagner_min_cut` and centralities.
- **Not in the pinned set:** a graph partitioner (METIS-style nested dissection or k-way) and a bipartite Hopcroft–Karp. Bipartite matching comes from pounce-presolve or BTF.
- **Responsibilities:**
  - block and unit dependency DAGs;
  - parallel wavefronts of independent blocks or cases (layers);
  - critical-path ordering (longest path);
  - column colouring for finite-difference or dual seeding (dsatur or greedy colouring on the column-intersection graph);
  - tear candidates;
  - reachability.

### F9. Thermodynamic property providers (FeOS / feos-core 0.10.1, quantity 0.14, num-dual 0.14)

- **Models.** Equations of state with the `pcsaft` feature. `State<E, N, D: DualNum>` is **generic over dual numbers**, so property derivatives propagate through the density iteration.
- **States and equilibria.** `StateVec` for batches. `PhaseEquilibrium` covers TP flash, bubble and dew points, pure VLE, PX flashes, stability analysis and phase envelopes. Critical points.
- **Parameter derivatives.** **`feos_core::ad::ParametersAD`** gives derivatives with respect to model parameters, with datasets, for fitting. The former feos-ad crate is merged into feos-core 0.10.
- **Other.** A `rayon` feature. Typed SI quantities through `quantity`.
- **Newer release.** **0.10.2 (2026-10-12)** fixes an off-by-one in the `_density_iteration` convergence check, a transposed Jacobian in the `PcSaftBinary` association term, gc-PC-SAFT binary parameter parsing, and a DFT association fraction. All four affect numerical results at 0.10.1. Source: upstream CHANGELOG, fetched.
- **Responsibilities:** a library-owned property provider (compiled Helmholtz with automatic-differentiation derivatives) as an alternative to authored package science, and an independent reference oracle.

### F10. Caches and persistent structures

- **quick_cache 0.6.24.**
  - Variants: `sync` and `unsync`, sharded.
  - Policy and bounds: CLOCK-PRO-style admission with a `Weighter` for byte-weighted capacity, and `set_capacity`.
  - Single flight: **`get_value_or_guard` / `get_or_insert_with` / `entry`** insert a placeholder; concurrent callers wait on it or receive a guard.
  - Lifecycle: `Lifecycle::{is_pinned, begin_request, on_evict}`.
  - Accounting: `memory_used`, `hits` and `misses`.
  - Newer: 0.7.0; its changes were not examined.
- **rpds 1.2.1.** Persistent `HashTrieMap`, `RedBlackTreeMap`, `Vector`, `List`, `Queue` and `Stack`, with `*Sync` (Arc) variants. Updates share structure in O(log n). Snapshots are free.
- **Responsibilities:** bounded artifact retention with single flight; and immutable snapshots of large maps (inventories, symbol tables), so an edit copies only a path.

### F11. Condition algebra (rust-reasoning; not in the workspace)

- **biodivine-lib-bdd 0.6.3.** Canonical equality per variable set, implication, compatibility, restriction and `binary_op_with_limit` (a node cap where `None` means too large, never false). It is theory-blind: `x>3 ∧ x<2` is satisfiable to it (B012).
- **z3 0.21.1.** Theory atoms over bounds. A timeout yields `Unknown`.
- **Relevance (only this).** Algebra over applicability and regime guards:
  - coverage: do the declared regimes cover the domain?
  - exclusivity: are the regimes disjoint?
  - implication: does guard A imply guard B?
  - canonical identity of condition sets, usable as cache keys.

  Answers must stay *unknown* on refusal.

---

## 2. Current use

These are pointers only. Search: `rg -l '<lib>::'` over `crates/` and `xtask/` (Rust). The
counts are files per crate.

| Family | Where |
|---|---|
| F1 Salsa | `pse-compiler` only (9 files). `#[salsa::db] CompilerDatabase` is at `crates/pse-compiler/src/workspace.rs:188`, with an event callback at `:1218`. Granular queries (`returns(clone), lru=64, heap_size`) are in `workspace/modeling/granular.rs:13-702`. `selected` and `specialized` are in `workspace/modeling.rs:303,312`. `projection` and `admitted` are in `modeling/executable.rs:387,1345`. Interned `BodyKey` and the body query are in `executable/grouped.rs:31,76`. `Durability::HIGH` is set on context inputs (`workspace.rs:1349-1351`). Token cancellation and `Cancelled::Local` resume carry resource refusals (`workspace.rs:220-240`). `trigger_lru_eviction`, `memory_usage` and generation `rebuild` are at `workspace.rs:1238-1330`. The runtime holds a retained `Arc<Mutex<CompilerWorkspace>>` (`pse-runtime/src/math.rs:564`). |
| F2–F4 Symbolica | `pse-math` only (27 files). `library.rs:160-185` runs `evaluator_multiple` with Horner, CPE, cores and abort. `library/admission.rs:384` runs `vectorize(Dualizer)`, with export-based reservation. The licence key comes from `SYMBOLICA_LICENSE` (`lib.rs:67`). `implicit_cubic.rs:198` uses `isolate_real_root_intervals`, and `collocation.rs:138` uses `isolate_real_roots`. The evaluator ABI string is `"pse-math-evaluator-abi-v4;demanded-support;interpreted-f64;numerica-jets;real-algebra;no-jit;no-simd"` (`pse-compiler/src/workspace.rs:448`). `CompiledBody::worker_scoped` now creates `programs: EnumMap::default()`, which materializes programs lazily (`pse-math/src/execution.rs:2186`). No use was found of `ErrorPropagatingFloat`, `DoubleFloat`, `Float`, `f64x4/f64x8` or `series`. |
| F5 | num-dual: none directly. finitediff: one file in `pse-backend-native` (derivative diagnostics). |
| F6 faer | `pse-backend-native` (61 files), `pse-runtime` (11) and `pse-math` (7). `Par` is passed explicitly: `pse-math/src/implicit.rs:1047-1049` uses `Par::Seq` or `Par::rayon(scope.admitted_threads())`, and `Par::Seq` is used in `sparse.rs`, `convexity.rs`, `square_response.rs` and `kkt/path/arclength.rs`. |
| F6 KLU | `pse-backend-native/src/dynamics/linear.rs` calls `klu_l_analyze`, `klu_l_factor`, **`klu_l_refactor` (`:223`)**, `klu_l_rcond` and `klu_l_condest`. SUNDIALS KLU is used through `kinsol.rs` and `dynamics/idas.rs`. BTF, AMD and COLAMD are not called directly. |
| F6 FERAL | `kinsol/block.rs:281` and `conditioning.rs:52` call `SparseLu::factor_markowitz`. |
| F7 pounce-presolve | `pse-structural/src/incidence.rs` (matching, DM, components, BTF, coupling); `pse-math/src/implicit_reconstruction.rs:1385-1404`; FBBT in `pse-math` tests and presolve; `pse-backend-native` presolve. |
| F8 graphs | petgraph is used in `pse-modeling` (5 files), `pse-structural` (3), `pse-compiler` (2), and `pse-runtime`, `pse-model`, `pse-codegen` and `pse-authoring` (one file each). rustworkx-core is used in `pse-structural` (2 files), including `lexicographical_topological_sort` for block order. |
| F9 FeOS | `xtask` (oracle generation under the `thermodynamic-oracles` feature). There is one fixture reference in `pse-runtime` tests (`data/oracles/feos-0.10.1`). |
| F10 | quick_cache: `pse-runtime/src/math/retention/weighted.rs` (`unsync::Cache` with a `Weighter`). rpds: `pse-modeling/src/check/phases.rs:10,512` (`RedBlackTreeMapSync` for `Types` and `Members`). |
| F11 | Not used. Applicability and regimes are bespoke: `pse-model/src/applicability.rs` (`covers`) and `pse-math/src/implicit_regimes.rs` (3,240 lines). |

---

## 3. Structure facts the libraries can produce, and their lifetimes

These answer the question "which structure facts, at what lifetime". Interpretation.

| Fact | Producer (pinned) | Input | Natural lifetime | Downstream use |
|---|---|---|---|---|
| Maximum matching, DM parts, structural rank | pounce-presolve `hopcroft_karp` + DM; KLU `btf_l_maxtrans`; KLU symbolic structural rank | equality incidence pattern | **pattern** (structural edit) | well-posedness refusal, over- and under-determination diagnosis |
| BTF block order, block sizes, largest block, off-diagonal count | pounce-presolve `BlockTriangularForm`; `klu_l_analyze` (nblocks, block boundaries); petgraph `tarjan_scc` + `condensation` | pattern | pattern | per-block method choice (size 1, small, large SCC), sequential schedule |
| Independent-block wavefronts, critical path | rustworkx `dag_algo::layers`, `longest_path` over the block DAG | condensation DAG | pattern | structure-level parallelism, ordering |
| Coupling class per block | pounce `classify_block` | pattern + objective-gradient and inequality support | pattern + objective support | eliminable or auxiliary blocks |
| Per-block algebraic form: affine, polynomial degree, cubic, closed-form single-variable solve | Symbolica `coefficient_list`, degree, `solve_linear_system`, root isolation | template body expression | **template body** (definition edit) | direct LU, closed form, Newton or least squares per block |
| Structural Jacobian and Hessian sparsity per body | Symbolica `derivative` support / Dualizer zero components | body | template body | pattern assembly, seeding |
| Column colouring (seed groups) | petgraph `dsatur_coloring` / rustworkx `greedy_node_color` on the column-intersection graph | pattern | pattern | grouped finite-difference columns (finitediff `*_pert`), batched dual seeds |
| Fill-reducing ordering, supernodal partition, elimination tree, factor nnz bound | faer `SymbolicLu` / `SymbolicCholesky`; KLU symbolic (AMD/COLAMD per block) | pattern | pattern | factor memory admission (an upper estimate, not an allocator bound), solver choice |
| Pivot sequence | KLU numeric (reused by `klu_l_refactor`) | values | first factor, until the condition or growth guard trips | cheap refactor across iterations, points or time steps |
| rcond, condest, pivot growth, flops | KLU; FERAL `condition_estimate_1` | values | **per factorization** | conditioning diagnosis, refactor fallback trigger |
| Inertia evidence (regularization count) | faer LDLT `dynamic_regularization_count`; LBLᵀ | values | per factorization | KKT inertia screen (R4 owns the KKT route) |
| Extreme singular values | faer `partial_svd` (largest); shift-invert with a retained factor for the smallest | values | per point | rank and conditioning screens without a dense SVD (not a certificate) |
| Bound enclosures | pounce FBBT | bounds + expression DAG | **value** (bounds and fixed values) | presolve, domain diagnosis |
| Rounding-error estimate of a residual | Symbolica `ErrorPropagatingFloat` evaluation | values | per evaluation (diagnostic) | precision-loss and cancellation diagnosis |
| Regime coverage, exclusivity and implication | biodivine BDD (+ z3 for theory atoms) | guard conditions | template or definition | applicability admission, canonical condition keys |

These group into three natural lifetimes:

- **Template body.** Changes on a definition edit.
- **Pattern.** Changes on a structural edit: fixed/free status, rows, bindings or contributions.
- **Value.** Changes per point or iterate.

Every pattern-lifetime fact above is computable once per pattern and shared by evaluation,
factorization, method choice and diagnosis.

---

## 4. Responsibility questions, answered neutrally

### Q1. What could carry one coherent preparation and reuse lifetime, from edit to study point?

Facts that bound such a lifetime (interpretation, labelled):

1. **Edit to semantic products.**
   - Salsa can carry this stage: pure, `Send + Sync`, `PartialEq` values, with backdating and durability.
   - Template-body compilation (Symbolica optimize and Dualizer) is a pure function of the body request, so it is *expressible* as a tracked query.
   - Today, by contract, it is outside Salsa: §14.3 says "Symbolic initialization, native compilation and evaluation are effects outside tracked Salsa queries".
   - Salsa itself does not forbid it. The constraints are the deterministic `abort_check` cancellation, which must not cache a cancelled result (the workspace already converts refusals to `Cancelled::Local`), and resource charging (an effect).
2. **Pattern-lifetime products.** These are the incidence, matching, BTF, orderings, symbolic factorization, colouring and assembly patterns.
   - They are pure functions of the bound structure and are therefore Salsa-expressible.
   - **faer `SymbolicLu` and `SymbolicCholesky` are immutable** and can be shared.
   - The KLU symbolic object is a raw C allocation. It can be shared read-only, but it is not `PartialEq`, and its lifetime must be owned explicitly.
3. **Value lifetime.** These are the numeric factors, evaluator scratch, native sessions (Ipopt, POUNCE, KINSOL, IDAS) and starts.
   - They are mutable, and some are FFI handles. They fail Salsa's purity and value-equality contract, so they cannot be memos.
   - Each library defines the reuse unit here:
     - **faer numeric refactor at a fixed symbolic object;**
     - **KLU `refactor` at a fixed pivot order;**
     - evaluator scratch per thread (`&mut self`);
     - JIT code shared through `Arc<Memory>`.
4. **Study points.** No library here owns cross-point reuse policy. Each library provides only the primitive: symbolic reuse, refactor, `roots_hot_start`, the POUNCE `PresolveMap`/`WarmPoint`, and `batch_evaluate` over rows.

**Library-forced lifetime boundaries** (facts):

| Boundary | Forced by |
|---|---|
| definition → pattern | Symbolica program identity; the Salsa tracked-struct and interned identity rule (B018) |
| pattern → value | the faer/KLU symbolic–numeric split; Salsa's purity and `Send + Sync + PartialEq` output bound |
| value → per-thread scratch | `ExpressionEvaluator::evaluate(&mut self)`; JIT batch buffers |

A single carrier across all three boundaries does not exist in these libraries. A
single *key discipline* could: definition key, then pattern key, then value or point key.

### Q2. Could Salsa be the single reuse and invalidation authority?

**For it** (facts):
- complete dependency recording including absence;
- early cutoff;
- durability tiers;
- per-function LRU;
- cancellation tokens;
- per-key single flight across cloned handles (B013);
- an event stream for observation;
- interned dedupe with GC.

**Against it, or needing project machinery** (facts):
- *Single writer.* A write waits for all handles to drop. Concurrent studies cannot mutate inputs while long queries run, and the caller must drain them.
- *No byte budgets.* Eviction is by count only. `heap_size` and `memory_usage` report but do not evict. The workspace's generation `rebuild` exists for this reason.
- *Untracked state goes silently stale* (B003). Every consulted context must be an input.
- *Interned ids are not durable*, because their slots are reused (B019). Ids cannot leave the process or be persisted outside Salsa's own persistence.
- *Persistence* is serde-whole-database, with transitive persist obligations. It panics on a non-persisted input (B027), and it is not a substitute for a durable store across versions.
- *No scheduler, no async, no capacity charging.*
- *Values must be `Send + Sync`*. Native sessions and FFI handles are excluded.
- *Waiters recompute on cancel* (B037), which interacts with charged work.
- *Process-local only.* It cannot coordinate across worker processes.

**Interpretation.** Salsa can be the single invalidation authority **for pure products up to and
including pattern-lifetime structure and compiled programs**. It cannot be the authority for
value-lifetime numeric state, durable cross-process reuse, or byte-bounded retention. Those need
an owner keyed by Salsa-produced identities. Within one process, the current split has two
reuse authorities for pure products: Salsa (compiler) and the runtime's single-flight artifact
retention (`pse-runtime/src/math/retention.rs`, `jobs.rs:770-860`).

### Q3. What does Symbolica offer for the inner loop and for many identical bodies?

- **Interpreted, with no feature change.**
  - Evaluation in `wide::f64x4/f64x8` lanes runs four or eight instances, collocation points, finite-difference columns or study points per instruction stream.
  - Constraint: there is no per-lane divergence, so Domain, Branch and Require stages must agree across lanes or fall back to scalar.
  - `ErrorPropagatingFloat`, `DoubleFloat` and `Float` re-evaluation give diagnostic precision evidence.
- **With `native_code_generation`.**
  - JIT gives `JITCompiledEvaluator<f64|f64x4>` with **`batch_evaluate(rows)`**. Code is shared by `Arc` across clones, so the per-thread state is only the buffers. This removes the per-attempt program-copy concern by construction.
  - **Obligations it raises** (facts):
    - FMA is on by default (`FASTMATH`), against the repository's no-FMA invariant;
    - `symjit.toml` in the CWD or `$SYMJIT_TOML` silently changes code generation;
    - executable mmap (`PROT_EXEC`);
    - the code is not bit-identical to the interpreter;
    - qualification under DP-11 determinism.
  - The AOT `export_cpp`/`compile` route needs a run-time C++ toolchain and `dlopen`.
- **Non-inlined `FunctionBody`** sub-programs share repeated sub-expressions, such as property kernels called by several bodies, inside one evaluator.
- **Construction cost controls.** `hot_start` (reuse a previous Horner scheme), `n_cores` (licence-clamped) and `abort_check`.

### Q4. Structure facts and lifetimes

See §3.

---

## 5. Library-shaped allocations

These are complete sketches, each *Proposed*. The coordinator decides adoption.

### A1. "Salsa owns all pure preparation and reuse up to the native boundary"

- **Shape.**
  - One long-lived Salsa database per live lineage. Its inputs are declaration fields, context, bindings, fixed/free status and bounds. Fixed values are separate inputs at LOW durability; the context is HIGH.
  - Tracked queries run parse, resolve, check, specialize, lowering and template-body symbolic products. They then run **compiled programs** (Symbolica optimize and Dualizer output, as `Arc` values compared by content hash), **pattern products** (incidence, DM, BTF, colouring, `SymbolicLu`/`SymbolicCholesky` behind `Arc` and compared by pattern hash), assembly patterns, block fact records (§3), and value-consuming facts such as presolve enclosures and class facts.
  - Below the native boundary, numeric factors, evaluator scratch and sessions are owned by attempt or lane objects keyed by Salsa-produced pattern and program identities.
- **Obligations the library discharges.** Dependency recording and invalidation, early cutoff for equal results, dedupe of identical requests (interned), single flight within the process, cooperative cancellation of stale work, count-based eviction, and an execution event stream for observation and tests.
- **Obligations that remain the project's.**
  - byte budgets and capacity charging (Salsa's are count-only);
  - atomic multi-input publication and the drain before a write;
  - mapping resource refusals to non-cached unwinds;
  - durable cross-process reuse (persisted dependencies per ADR-0164, or none);
  - stable identities, because Salsa ids are not durable;
  - native-state lifetimes and cross-process coordination.
- **Lost or weakened.**
  - A write blocks on readers, so long tracked queries such as program optimization delay edits unless cancelled. Cancellation discards their work: no memo is stored (B022).
  - A rotation or `rebuild` discards compiled programs together with semantic products, unless programs are stored outside.
  - The "fresh workspace gives an equal product" oracle (§14.3) must then include compiled programs, so Symbolica determinism under `cores > 1` becomes a reuse-correctness premise. That premise is not established here.
  - Program optimization inside a query makes construction time part of query time. Abort is through `abort_check`.
- **Cost.**
  - *Integration:* move program and pattern construction into queries; convert effects such as charging into pre-reservation or post-accounting.
  - *Lifecycle:* one database per lane; pinning HIGH-durability context.
  - *Testing:* the event-count controls the skill uses.
  - *Upgrade:* Salsa minors break APIs often (0.22–0.28 renamed options, removed `Update`), and `salsa_unstable` `memory_usage` changes at 0.28.5.
- **What it makes unnecessary** (at the level of responsibility):
  - the runtime's separate single-flight artifact cache and its retention keys for pure programs;
  - the separate request dedupe and bucket tables;
  - hand-maintained "what to rebuild on which edit" tables for pure products. The §14.3 change table becomes an emergent property;
  - re-deriving structural analysis outside compiler preparation, such as the callback oracles' fresh analysis.

### A2. "Evaluator-centric preparation with compiled, batched evaluators"

- **Shape.**
  - The unit of reuse is the **compiled template program**: Symbolica optimize, then Dualizer, then JIT (`native_code_generation`) or interpreted SIMD. It is keyed by body content, demanded outputs, coordinates, order and backend profile.
  - All instances of a body, all finite-difference or colouring column groups, all collocation nodes and all study points are evaluated by `batch_evaluate(rows)`, or by `f64x4/f64x8` lanes. Scalar fallback is used where guard stages diverge.
  - Derivatives come from Dualizer programs. Implicit inner solves lift through num-dual `implicit_derivative*` when an opaque provider is used.
  - Optional library-owned property providers: FeOS `State<D: DualNum>`, `ParametersAD` and `StateVec` for EoS families where authored symbolic size grows badly.
  - Diagnostic re-evaluation in `ErrorPropagatingFloat`/`DoubleFloat`.
- **Obligations the library discharges.** Instruction-stream optimization (Horner, CPE), derivative program generation, machine-code generation and sharing (`Arc<Memory>`), vectorized evaluation over rows or lanes, precision-error estimation, and (with FeOS) phase-equilibrium and density-root solving with derivatives.
- **Obligations that remain the project's.**
  - guard, domain and applicability semantics outside the program, and lane-divergence handling;
  - determinism qualification: FMA off, `symjit.toml` isolation, a bit-equivalence policy against the interpreter;
  - W^X and executable-memory ownership;
  - per-thread buffers;
  - artifact identity including backend profile and library versions;
  - compile-time admission (JIT compile time and code size are unbounded unless measured);
  - FeOS unit, basis and reference-state mapping and branch checks.
- **Lost or weakened.**
  - Interpreter-only determinism (the `no-jit;no-simd` ABI) and FMA-free reproducibility across machines.
  - Exactness of authored-model authority and extension locality, where FeOS providers replace authored science (ADR-0098 knowledge boundary).
  - Simple per-instance refusal attribution inside a SIMD lane.
  - The JIT applies "fastmath" unless overridden.
  - The AOT C++ route adds a run-time toolchain dependency.
- **Cost.**
  - *Integration:* enabling `native_code_generation` adds symjit, libloading and self_cell. Batched instance layout. Lane masking for guards.
  - *Lifecycle:* JIT code lifetime bound to the program owner.
  - *Testing:* interpreter-versus-JIT equivalence per body class; determinism on the qualified host.
  - *Upgrade:* Symbolica and symjit move together; serialized IR is bound to the settings and version.
- **What it makes unnecessary.**
  - per-attempt program cloning, since code is shared and only buffers are per thread;
  - per-instance interpretive dispatch;
  - bespoke batching of identical bodies;
  - bespoke precision diagnostics;
  - (with FeOS) authored large-symbolic EoS bodies and their size admission for those families.

### A3. "One pattern-lifetime structure and factorization plane"

- **Shape.** A single immutable product per (pattern, block) computes everything that depends only on structure, once:
  - matching, DM and BTF (pounce-presolve or `klu_l_analyze`);
  - condensation, wavefront layers and critical path (petgraph/rustworkx);
  - per-block form facts (Symbolica);
  - column colouring;
  - fill-reducing ordering and symbolic factorization (faer `SymbolicLu`/`SymbolicCholesky`, KLU symbolic per block).

  Consumers read it at the value level:
  - per-block method choice (closed form, affine LU, Newton, least squares);
  - structure-level parallelism over independent wavefront blocks;
  - numeric refactorization (faer numeric at fixed symbolic; KLU `refactor` at a fixed pivot order with an `rcond`/`rgrowth` fallback);
  - grouped finite-difference or dual seeding;
  - diagnostics (structural rank, rcond, condest, pivot growth, regularization count, partial SVD).
- **Obligations the library discharges.** Structural decomposition and its correctness; ordering and symbolic factorization; refactor at a fixed pattern; condition and growth estimates; inertia evidence; DAG scheduling primitives (layers, longest path); colouring.
- **Obligations that remain the project's.**
  - mapping block identities to semantic rows and columns;
  - choosing between the duplicate producers (pounce BTF, KLU BTF, petgraph SCC) and cross-checking them;
  - refactor fallback thresholds;
  - factor-memory admission (faer numeric growth has no allocator hook, per prior F20; KLU allocates through SuiteSparse_config);
  - the thread budget for structure-level parallelism (`Par` and pools);
  - per-block method policy.
- **Lost or weakened.**
  - Duplicate independent structure computations currently serve as cross-validation (`incidence.rs` cross-checks the library results). One plane removes that redundancy unless it is kept deliberately.
  - Refactor at a fixed pivot order trades stability for speed, so it needs a guard.
  - KLU's block BTF and pounce's BTF can differ in block boundaries: both are valid, but they are not identical.
- **Cost.**
  - *Integration:* moderate. Most producers are already linked.
  - *Lifecycle:* immutable `Arc` keyed by pattern hash, which fits A1 directly.
  - *Testing:* structural oracles exist (pounce versus petgraph cross-checks).
  - *Upgrade:* low. faer, SuiteSparse and petgraph APIs are stable at these pins; pounce is a fork at a fixed rev.
- **What it makes unnecessary.**
  - re-analysis of structure in callback oracles and in native admission re-checks;
  - per-consumer symbolic factorization;
  - full pivot searches on each linearization where refactor applies;
  - bespoke block-routing heuristics that ignore form facts. The solver evidence notes that `affine_rows` is hard-coded to false.

A1 and A3 compose: A3's products are A1 queries. A2 is orthogonal and can sit on either.

---

## 6. Prior evidence reused, and what changed since

Reused, not redone:
- efficiency L1 (`numerical-library-utilization.md`) and L-B (`mathematical-compilation-and-evaluation.md`);
- Plan 36 `preparation.md` (I-A1 granular Salsa design and its 0.28.4 controls), `sparse-rank.md` (SPQR blocker) and `numerics.md` (F18 cubic route, F20 sparse response);
- solver A3 `structure-and-decomposition-map.md` (BTF facts are not joined to block routing; `coupling` has no consumer; there is no symbolic assignment detection).

Observed changes at `6c368c88a`, by source pointer:

| Prior lead | Now |
|---|---|
| L1-02: faer global `Par` in implicit blocks | `implicit.rs:1047-1049` now takes `Par::Seq` or the admitted `Par::rayon(n)`. |
| L1-03 / B-08: per-attempt deep clone of programs | `CompiledBody::worker_scoped` now starts from `EnumMap::default()` and copies lazily (`execution.rs:2186`). Per-order copies on demand remain, so the copy is now lazy rather than removed. |
| L1-04 / B-05: exact rational isolation | `implicit_cubic.rs:198` uses `isolate_real_root_intervals` (the F18 route). |
| L1-05: KLU full factor | `dynamics/linear.rs:223-249` now uses `klu_l_refactor` with `rcond`/`condest`. FERAL block preconditioners still call `factor_markowitz` (`kinsol/block.rs:281`). |
| L1-06 / I-A1: granular Salsa | `workspace/modeling/granular.rs` holds granular inputs and queries. Request interning is retained, as in Plan 36. |
| JIT / SIMD rule (RC-B2) | Unchanged; the ABI string is still `no-jit;no-simd` (v4). **New fact:** the symjit default enables FMA and reads `symjit.toml` (§F4). |
| Versions | salsa 0.28.5, num-dual 0.15.1, quick_cache 0.7.0 and **feos 0.10.2** (correctness fixes relevant to the oracle) are newer than the pins. |

## 7. Uncertainties and limits

- Nothing was measured. All cost statements are qualitative.
- Whether symjit honours a string option that turns `FASTMATH` off is not checked.
- Determinism of Symbolica optimization with `cores > 1` is not checked.
- quick_cache 0.7.0 changes were not examined.
- How often JIT compile time would repeat is not checked.
- Graph partitioning (nested dissection or k-way): absent from the pinned petgraph, rustworkx-core and pounce-presolve sources, by search of the module lists and the rust-graphs coverage catalog. Unpinned leiden-rs offers communities, not fill-reducing partitions.
- Salsa persistence beyond serde_json is unexercised (skill limit).
- A single structure plane's equivalence across KLU BTF and pounce BTF is not established.
- R4 owns native sessions, the KKT route and solver statistics; they are referenced here only as reuse units.
