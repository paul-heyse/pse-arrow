# Numerical, symbolic and scientific library utilization: bounded supporting assessment

Inquiry **L1** of the [principal efficiency-alignment review](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md). This is supporting
evidence, not an independent principal review, finding ledger or qualification receipt. The
principal review owns the combined judgment, finding identifiers (`F01`…) and rule impacts
(`RC01`…); local labels below (`L1-nn`) are investigation labels only. Coordinator source
checks, reconciliation and probe results are in the [evidence README](README.md).

Baseline: `main` at `b01a45d71a5f5ab29991ad193679f5588ac3662f` (2026-10-11), production source clean.
Assessed by a fresh read-only `library-research` against Core 3.4, Heuristics for Efficient Architecture
1.0, Process Simulator 1.5 and the [pse-arrow binding](../../design_principles/binding/pse-arrow.md).
No builds, tests or services were run by the inquiry; probes it proposes are listed for later
settlement unless the README records them as executed.

---

**Inquiry L1** of the efficiency-alignment review. Baseline is `main` at `b01a45d71`. Everything here comes from reading source and docs; nothing was built, run, written to the repository or edited in a library skill. No scratch files were written.

## Per-library table

| library@pin | integration owner(s) | capabilities used (path) | notable unused relevant capabilities | bespoke equivalents found | assessment |
|---|---|---|---|---|---|
| **Symbolica 3.0.1** (+Numerica 3.0.1; `integer-gmp`, `float-mpfr`; default features off) | `pse-math` only (`library.rs`, `library/admission.rs`, `execution.rs`, `jets.rs`, `factorable.rs`, `implicit_*`, `collocation.rs`) | `Atom::evaluator_multiple` builder with Horner/CPE/abort (`library::exact_evaluator`); `vectorize(Dualizer(HyperDual, zero_components))` (`library::admission::bounded_evaluator`); `export_instructions` for budgeting; `derivative` (`typed_partial`, `class_evidence`, `jets`); `UnivariatePolynomial` exact roots (`collocation`, `implicit_cubic`); `coefficient_list` | `native_code_generation` (symjit `jit_compile`, `export_cpp`, `BatchEvaluator`); interpreted `ExpressionEvaluator<wide::f64x4/f64x8>` (Numerica `impl Real` for SIMD types, not feature-gated); evaluator serialization (`serde`/`bincode`); float roots `roots`/`roots_hot_start` (Aberth) | Per-attempt deep clone of evaluator programs (`CompiledBody::worker_scoped`); exact rational root isolation on float-derived coefficients for every inner solve (`CubicRoots`) | adequate core; **candidate** (L1-03, L1-05); one rule impact (JIT/SIMD) |
| **faer 0.24.4** (`std`, `sparse-linalg`; `rayon` turned on workspace-wide via diffsol-la) | `pse-math`, `pse-backend-native`, `pse-runtime` | Symbolic sparse LU retained with numeric refactorization (`pse-math::implicit::FactorWorkspace`, `pse-backend-native::dynamics::linear::FaerLu`, `square_response::actions::SparseFactor`); `SymbolicSparseColMat` patterns plus refill (`pse-math::assembly::CasePlan::jacobian_pattern`); dense SVD/EVD (`square_response::spectrum`, `convexity`); sparse-dense matmul (fitting) | Explicit `Par` admission in `pse-math` (the global default is used there) | Dense O(n³) Root response over a sparse Jacobian (`square_response::response`) | adequate; **finding-candidate** (L1-01, L1-02) |
| **SUNDIALS (sundials-sys 0.6.2)**: IDAS, KINSOL, KLU | `pse-backend-native::dynamics::idas`, `kinsol` | IDAS: init/ReInit, SensInit/SensReInit, quadrature, roots, adjoint (`IDAAdjInit`, `IDASolveB`), `SUNLinSol_KLU`, Krylov with preconditioner; KINSOL: KLU/dense/Krylov, Anderson acceleration (`KINSetMAA`), `KINSetMaxSetupCalls`, `KINSetNoInitSetup` | none material | — | adequate (strength) |
| **suitesparse_sys 0.1.4** (klu, amd, colamd, btf) | `pse-backend-native::dynamics::linear::Klu` | `klu_l_analyze` once per pattern; `klu_l_factor` per linearisation | `klu_l_refactor` (reuses the pivot order); `spqr` feature (rank-revealing sparse QR) | Full KLU factor on every Diffsol linearisation | candidate (L1-06) |
| **diffsol 0.16.2 / diffsol-la 0.1.1** (faer) | `pse-backend-native::dynamics` | BDF/SDIRK/ERK with pse-owned `LinearSolver` (FaerLu, Klu) | — | — | adequate (examined shallowly) |
| **Ipopt** (`pse-ipopt-sys`, C++ bridge) | `pse-backend-native::ipopt` | `ReOptimizeTNLP` + `warm_start_same_structure` via `bridge/pse_ipopt_sequence.cpp`; `warm_start_init_point` | — | — | adequate (strength) |
| **POUNCE family @ ff638694** (nlp, rs, presolve, linsol, feral, sens-core, qp, convex) and **FERAL 0.18.0** | `pse-backend-native` (`pounce.rs`, `pounce/retained.rs`, `kkt/*`, `presolve/*`, `execution/pounce_convex.rs`, `conditioning.rs`, `kinsol/block.rs`), `pse-structural`, `pse-math` | Retained FERAL backends through `LinearBackendFactory` leases; restoration; `jitter_start`; presolve/FBBT/DM (`hopcroft_karp`, `DulmageMendelsohnPartition`, `BlockTriangularForm`); `SensApplication`/`SensBacksolver` (KKT, sensitivity, advanced step); `solve_qp_batch_parallel_warm`; FERAL `condition_estimate_1`, `SparseLu::factor_markowitz` | `feral::SparseLu::factor(a, &SparseLuSymbolic)` (ordering reuse) | Full Markowitz factor on every KINSOL preconditioner setup | adequate; minor candidate (L1-06) |
| **HiGHS 2.4 / highs-sys 1.15**; **Clarabel 0.11.1**; **SCIP (scip-sys 0.1.28)**; **Uno**; **PETSc** | `pse-backend-native::{highs,execution/*,scip,uno,petsc}`, `tears.rs` | Declared reuse: HiGHS coefficient/bound updates; Clarabel data updates; SCIP reoptimization; tear selection as a HiGHS MILP | not assessed | — | examined shallowly |
| **Salsa 0.28.4** (`salsa_unstable`) | `pse-compiler::workspace*` | 7 tracked queries (`returns(clone)`, `lru=64`, `heap_size`); interned `BodyKey`/`PlanKey`/`NameKey`/`SelectionKey`; `Durability::HIGH/MEDIUM` on `Inventory`; equality-guarded `set_checked`; `memory_usage` | `#[salsa::interned(revisions = N)]` LRU garbage collection for request identities | Hand-rolled `Request` dedupe table plus a rebuild of the whole database on a byte budget (`modeling_request_in_bucket`) | adequate; low candidate (L1-07) |
| **petgraph 0.8.3**, **rustworkx-core 0.18.1** | `pse-structural`, `pse-compiler`, `pse-modeling`, `pse-operations`, `pse-authoring`, `pse-codegen` | `kosaraju_scc`, `toposort`, `greedy_feedback_arc_set`, `find_cycle`, `lexicographical_topological_sort`, `has_path_connecting`, `Dfs`, `NodeFiltered` | — | No bespoke SCC/matching/topological sort found (search scope below) | adequate; one access-path note (§4) |
| **egobox (ego 0.38.1, moe, gp, doe)**, **argmin 0.11**, **ndarray 0.16.1** | `pse-math::surrogate` | EGO with persistent mixture surrogate, LHS design | — | — | adequate functionally; **candidate** on placement (L1-08) |
| **statrs 0.18**, **finitediff 0.2** | `pse-runtime::workflow::fitting::{covariance,profile}`; `pse-backend-native::derivative_diagnostics` | Normal and χ² CDFs; FD Jacobian-vector products (diagnostics only) | — | — | adequate (cold paths) |
| **rayon 1.12** | `pse-backend-native::pounce` (scoped pool per call), `pounce_convex` (`par_iter`), `pse-math::surrogate` | Admitted scoped pools | — | — | adequate here; scheduling belongs to D |
| **FeOS/feos-core 0.10.1, quantity 0.14, num-dual, nalgebra 0.35** | `xtask` (`thermodynamic-oracles` feature) only; nalgebra also via diffsol-la | Reference oracle generation | FeOS PC-SAFT/phase-equilibrium as a production provider (deliberately excluded) | PC-SAFT is authored as package science and compiled through Symbolica | not a finding (§4) |
| teqp (Python) | parity group | — | — | — | not examined |

## 1. Scope examined and coverage limits

**Read in depth:**
- `pse-math`: `library.rs`, `library/admission.rs`, `execution.rs` (`compile_scope`, `compile_stages` Block arm, `Worker`), `assembly.rs` (`CaseWorker`), `implicit.rs` (factor workspace), `implicit_cubic.rs`, `implicit_accelerators.rs`.
- `pse-backend-native`: `dynamics/linear.rs`, `dynamics.rs` (eliminator), the `dynamics/idas.rs` API inventory, `kinsol/block.rs`, the `kinsol.rs` API inventory, `square_response.rs`, the `square_response/actions.rs` header, `kkt.rs` header, `pounce.rs` (pool and session), `pounce/retained.rs`, `ipopt/{settings,sequence}.rs`, `pse-ipopt-sys` bridge references, `execution/*` reuse declarations, `presolve.rs` header, `recycle.rs`/`tears.rs` headers.
- `pse-compiler`: Salsa declarations and request handling in `workspace.rs`, `workspace/modeling.rs` and `executable{,/grouped,/implicit}.rs`.
- `pse-runtime`: `math.rs::MathService::worker`, the `math/solves.rs` sensitivity path, `math/portable.rs` reconstruction, `math/prediction.rs` and `study.rs` root predictor.
- Pins and features from `Cargo.toml` and `cargo tree --offline -e features`.

**Upstream sources read at the pins:** symbolica-3.0.1 (`evaluate/{evaluator,backend,domain,dual}.rs`, `poly/univariate*.rs`, Cargo features), numerica-3.0.1 (`domains/float/simd.rs`), faer-0.24.4 (`lib.rs` parallelism), salsa-0.28.4 (`interned.rs`), sundials-sys-0.6.2 and suitesparse_sys-0.1.4 features, diffsol-0.16.2 features, and the POUNCE checkout `ff63869` (`vendor/feral/src/lu/*`, `pounce-algorithm/src/application.rs`, `pounce-convex/src/batch.rs`).

**Version note:** the `symbolica-faer-oximo` skill pins Symbolica 3.0.0 while the workspace pins 3.0.1. Every Symbolica claim below was checked directly against the 3.0.1 source in the cargo registry.

**Examined shallowly:** HiGHS, Clarabel, SCIP, Uno and PETSc (only their declared reuse contracts and headers), diffsol internals, egobox/argmin internals, statrs, finitediff, and the strategy and continuation layer (`math/strategy*.rs`, `solves/paths.rs`), which I read only to check PS-09 headers.

**Not examined:**
- teqp.
- The IDAS per-callback path. That belongs to C.
- Hot-path allocations in `Worker::evaluate_seeded` and `CaseWorker::evaluate`, which belong to B. I note one in §4.
- The retention cache for `CompiledBody` (B).
- The `pse-quantity` crate. It is an in-house domain model, not a library wrapper.

**Absence-claim scope:** the search for bespoke graph algorithms was `rg -i "fn (tarjan|hopcroft|augment|scc|topo|matching|dulmage|btf|feedback|cycle)"` over `crates/`, and it found nothing. The search for `set_global_parallelism` covered `crates/` and `xtask`. Context7 was not needed; pinned upstream source was sufficient and more exact.

## 2. Operations examined

- **Evaluator construction (preparation).** `PreparedBody::compile` → `compile_scope` → `compile_stages` → `library::admission::bounded_evaluator`. This runs Symbolica `evaluator_multiple` (Horner 10 / CPE 1 / one core), then `vectorize` with HyperDual over the block's active axes, with structural `zero_components`. The result is `CompiledBody`, an immutable value held in an `Arc` and charged through `retained_bytes`. Bodies are deduplicated by content hash (`CasePlan::bodies: BTreeMap<ContentHash, Arc<PreparedBody>>`).
- **Attempt start.** `MathService::worker` → `CaseAssembly::worker_scoped`. This clones every program (`self.programs.as_ref().clone()`, which deep-copies each `ExpressionEvaluator`'s instructions and stack), the `AssemblyMatrix` for the Jacobian and Hessian, and the group caches. It is charged as `numeric_worker_bytes`.
- **Per callback.** `CaseWorker::evaluate` iterates over instances. For each one it gathers inputs into a new `Vec`, builds a bit-key `Vec` for the input-equality cache, and on a miss calls `Worker::evaluate` (the frame jet interpreter; a dense n×n Hessian per output is allocated). It then refills the faer sparse matrix through precomputed `jacobian_terms`/`hessian_terms`. Structure is not rebuilt.
- **Inner implicit solves.** `implicit::Problem` holds `Mutex<Worker>` and `Mutex<FactorWorkspace>` and is local to the attempt. It is created from `Factory`, which is cloned per attempt and shares `Arc<dyn InnerSolver>`. Its faer numeric LU runs at `get_global_parallelism()`. The cubic accelerator does exact root isolation over rationals.
- **Native solves.**
  - Ipopt goes through the C++ bridge with `ReOptimizeTNLP`.
  - POUNCE uses a per-call scoped rayon pool plus a retained FERAL backend pool.
  - KINSOL uses KLU or a FERAL block preconditioner; recycle is handled as an Anderson fixed point.
  - IDAS memory is reinitialized in place.
  - Diffsol uses FaerLu with retained numeric storage, or Klu with a full factor each time.
- **Post-solve analysis.** KKT analysis and sensitivity use FERAL and `pounce-sens-core`, independently of the solver's own factors (PS-10). Root parameter response (`square_response::response`) densifies the Jacobian, runs a full SVD and then a dense LU.
- **Study prediction.** `restore_root_predictor` uses the sparse `SparsePredictor`/`SparseFactor`, with faer symbolic LU retained.
- **Compiler.** Salsa `selected` → `specialized` → `projection` → `admitted`, keyed by `(Inventory, Catalog, Request)`, plus `semantic_body` keyed by interned body content.

## 3. Candidate findings

### L1-01: Root parameter response densifies a sparse system and uses a full dense SVD plus LU

- **Cause.** `pse-runtime::math::solves` (sensitivity branch) builds `faer::Mat::zeros(n, n)` and `Mat::zeros(n, np)` from the sparse Jacobian. It then calls `pse-backend-native::square_response::response`, which:
  - computes every singular value with `spectrum` → `faer::linalg::svd::svd` on the dense n×n matrix to decide rank against `DEFAULT_RELATIVE_RANK_CUTOFF`;
  - solves with dense `lu::partial_pivoting`;
  - checks backward error densely.
  
  `workspace_bytes(n, np)` is O(n²) and is checked against `request.bytes`; past that, the response is refused with `Withheld::Memory`. `pse-runtime::workflow::fitting::oracle` also calls `square_response::response`. The sparse sibling `square_response::actions::SparseFactor` (faer symbolic LU, retained) is used for study prediction and engineering accuracy. Its own doc says it is "estimated evidence, not a numerical-rank certificate".
- **Consequence.** The workloads affected are *larger sparse structures* and *studies/fitting with sensitivities*. Cost grows as O(n³) time and O(n²) memory in the flowsheet size, not with nonzeros. Large flowsheets either pay a dense SVD on every sensitivity request or get no Root response. This is H5/H9 (full work where sparse structure exists) and DP-10.
- **Simpler or library alternative.** Keep the rank verdict but obtain it sparsely. Options:
  - Factor once with the faer sparse LU already retained in `SparseFactor`, and use FERAL's `condition_estimate_1` (already used in `pse-backend-native::conditioning`) as the screen. Escalate to a dense SVD only for small n, or when the estimate is near the cutoff.
  - Enable `suitesparse_sys`'s `spqr` feature (available at 0.1.4) for a rank-revealing sparse QR.
  
  Either way, solve the np right-hand sides through the same sparse factor.
- **Preservation constraints.**
  - PS-10 and the response's typed verdicts: `Withheld::Rank`, the backward-error check, physical back-mapping and refusal semantics.
  - A condition estimate is a lower bound, not a certificate. So the dense SVD (or an equivalent certificate) must stay the authority wherever a certificate is promised. This is the main semantic uncertainty.
- **Uncertainty.** I did not establish how often public Root responses are requested per study point. Study warm starts already use the sparse predictor.
- **Evidence and verification.** *Interface-checked* (source-traced). Seam with C. Suggested verification is probe P2.

### L1-02: Implicit-block factorizations in `pse-math` use faer's process-global parallelism, which is the global rayon pool

- **Cause.** `pse-math::implicit::Problem` construction stores `parallel = faer::get_global_parallelism()` and uses it for `factorize_numeric_lu` and the solves. faer's `rayon` feature is unified on for the whole workspace by `diffsol-la` (`cargo tree -e features -i faer@0.24.4` shows `faer feature "rayon"` ← `diffsol-la`).
  - In faer-0.24.4 `lib.rs`, `GLOBAL_PARALLELISM` starts at `2` when `rayon` is enabled, and `get_global_parallelism` maps that to `Par::rayon(0)`, which is `rayon::current_num_threads()`. Outside an installed pool, that is the global pool sized to the host's logical CPUs (32 here).
  - No call to `set_global_parallelism` or `disable_global_parallelism` exists in `crates/` or `xtask`.
  - By contrast, `pse-backend-native::dynamics::linear::FaerLu::set_sparsity` deliberately takes `op.context().par`. Its comment reads: "a process-global default may create an unadmitted team alongside independent trajectory workers".
- **Consequence.** Under the *sixteen cases on sixteen cores* premise, inner implicit solves can dispatch onto a 32-thread pool that host admission does not know about. This oversubscribes and bypasses the finite-capacity coordination (H21, DP-20). The scratch size also depends on `Par`.
- **Simpler alternative.** Pass the admitted `Par` explicitly, as the dynamics adapter does (likely `Par::Seq` for inner blocks). Alternatively, set the faer global parallelism to `Seq` once at process start, with a guard so a library-default change cannot silently re-enable it.
- **Preservation.** Results must not depend on the thread count. Sequential faer is deterministic.
- **Uncertainty.** faer's sparse LU may never actually fork for the small blocks typical of implicit bodies. The static evidence proves the setting, not observed threads.
- **Evidence and verification.** *Interface-checked*. Seam with D (capacity). Probe P1.

### L1-03: Every attempt deep-copies all Symbolica evaluator programs

- **Cause.** `pse-math::execution::CompiledBody::worker_scoped` does `programs: self.programs.as_ref().clone()`. Each `CompiledStage::Block` owns an `ExpressionEvaluator<f64>`, which in symbolica-3.0.1 `evaluate/evaluator.rs` is `#[derive(Clone)]` with an owned `instructions: Vec<…>` and `stack: Vec<T>`. `CaseAssembly::worker_scoped` does this for every program at every attempt (`MathService::worker`): multistart seeds, staged and initialization steps, study points, fitting oracles and dynamic workers. The accounting acknowledges the cost: `worker_bytes` includes `retained_instructions`.
- **Consequence.**
  - Memory: in the 16-case premise and for *increasing expression populations* (for example the 5,510-slot PC-SAFT tangent-plane body noted in `library.rs`), instruction memory is multiplied by the number of concurrent attempts and charged against the worker budget.
  - Time: each attempt start in a *thousand-point sweep* copies programs whose content has not changed (H12, H17).
- **Alternatives.**
  1. *Design-level, no library change:* keep `CaseWorker`s per execution thread and assembly identity, reset their state between attempts, and keep DP-19 attempt ownership by clearing caches and providers.
  2. *Library-owned:* Symbolica's `native_code_generation` (`jit_compile` → `JITCompiledEvaluator`, an immutable symjit `Applet` plus per-evaluator buffers). This also gives batch evaluation, but needs the rule change in §7 and costs compile time.
  3. *Upstream:* split the immutable program from the mutable stack in Symbolica's API.
- **Preservation.** Mutable state stays owned by the attempt. Cancellation scope must stay per attempt. Memory accounting must stay complete.
- **Uncertainty.** The size of the copy relative to solve time is unmeasured.
- **Evidence and verification.** *Interface-checked*. Seam with B and D. Probe P4.

### L1-04: Cubic implicit accelerator does exact rational root isolation per call, behind a shared lock

- **Cause.** `pse-math::implicit_cubic::CubicRoots::solve` runs on every inner solve. It:
  1. evaluates the coefficients under `Mutex<ExpressionEvaluator>`, which is shared across attempts and cases because `Factory.solver: Arc<dyn InnerSolver>` is cloned along with the factory;
  2. converts each f64 coefficient exactly to `Rational`;
  3. calls `UnivariatePolynomial::isolate_real_roots`;
  4. refines each root to 2⁻⁵⁴ with `refined(&precision)`;
  5. verifies each candidate against the original residual.
- **Consequence.** If cubic equation-of-state compressibility roots are inside each outer residual or derivative evaluation, this puts GMP rational arithmetic on the per-callback hot path (H15). That matters for *edit → re-solve* and sweeps over cubic-EOS flowsheets. The shared mutex adds contention across the 16 cases, though the critical section is short.
- **Library alternative.** Symbolica 3.0.1 `UnivariatePolynomial::roots` / `roots_hot_start` (Aberth over floats, with a warm start from the previous iterate's roots), followed by the existing verification against the original residual. Fall back to exact isolation only when root separation or branch uniqueness is ambiguous.
- **Preservation.** The "exactly one original-residual-verified root in the declared branch interval" contract requires complete enumeration. Aberth's documentation explicitly says it gives no certification, so the exact fallback must stay wherever completeness is in doubt.
- **Uncertainty.** I did not establish how often this runs per outer evaluation.
- **Evidence and verification.** *Interface-checked*. Seam with B. Probe P3.

### L1-05: KLU and FERAL block factorizations repeat the pivot search on a fixed pattern

- **Cause.**
  - `pse-backend-native::dynamics::linear::Klu::set_linearisation` frees the numeric object and calls `klu_l_factor` on every linearisation. SuiteSparse provides `klu_l_refactor`, which reuses the pivot sequence; SUNDIALS' own `SUNLinSol_KLU`, used by the IDAS path, does exactly this after the first factorization.
  - `kinsol::block::Blocks` rebuilds `Vec<Vec<(usize, f64)>>` columns and calls `feral::SparseLu::factor_markowitz` on every preconditioner setup. FERAL 0.18 offers `SparseLu::factor(a, &SparseLuSymbolic, params)` and `refactor`, which reuse the ordering.
- **Consequence.** Repeated pivot search on *long trajectories* (Diffsol KLU route) and on repeated KINSOL setups for recycles. This is a modest H12 cost.
- **Alternative.** Use `klu_l_refactor`, with a `klu_l_rcond`-guarded fallback to a full factor. Retain the FERAL symbolic object per block and refactor.
- **Preservation.** Zero-pivot and singular refusals; the finiteness check; numerical stability, because reusing a pivot order can degrade it. That is why refactor needs a condition/growth guard and a fallback.
- **Evidence.** *Interface-checked*. Low priority. Seam with C.

### L1-06: Salsa request identity is re-implemented by hand

- **Cause.** `pse-compiler::workspace::modeling::modeling_request_in_bucket` keeps `FxHashMap<u64, Vec<Request>>` with an equality scan over `#[salsa::input] Request`. On a byte budget (and `retention_exceeded`) it calls `self.rebuild(self.inputs.clone())`, which discards every memo. salsa-0.28.4 `interned.rs` already supports deduplication by value with LRU garbage collection ("Interned values are garbage collected and their memory reused based on an LRU heuristic"; `revisions` option; `heap_size` is already used elsewhere in this crate).
- **Consequence.** Bookkeeping that the library already owns. Under many distinct bindings (structural variants in studies), the rebuild drops the whole database instead of evicting cold identities (H25, H27).
- **Alternative.** Make `Request` `#[salsa::interned(revisions = N, heap_size = …)]`.
- **Preservation.** The memory ceiling and `memory_usage` accounting.
- **Uncertainty.** I could not confirm how often rotation happens; it may be rare.
- **Evidence.** *Interface-checked*. Low priority. Seam with A.

### L1-07: The surrogate stack places a large cold dependency closure under every `pse-math` consumer

- **Cause.** `egobox-{ego,moe,gp,doe}` and `argmin` are normal dependencies of `pse-math`, which `pse-compiler`, `pse-backend-native` and `pse-runtime` all consume. `cargo tree -p pse-math` lists 262 crates; pruning egobox leaves 191. The 71 reached only through egobox include `clap`, `env_logger`, `zip`, `linfa*`, `ndarray-*`, `sprs` and `nalgebra 0.33.3`. That last one arrives via `statrs` with nalgebra enabled through `egobox-moe`, which defeats the pin note "off: its nalgebra 0.33 and rand would split the pinned graph"; the graph now holds nalgebra 0.33.3 and 0.35.0. `pse-math::surrogate` is the only consumer.
- **Consequence.** *Compiling or testing one responsibility* (PSE-S05, AP-06): the compiler, which never runs surrogates, builds a surrogate-modelling stack (H5).
- **Alternative.** Move `surrogate.rs` into its own crate consumed only where surrogate studies run. A feature gate does not work here because ADR-0122 unifies features.
- **Preservation.** The surrogate's admission and accounting contracts.
- **Rule impact.** A crate addition needs an ADR (§7).
- **Evidence.** *Interface-checked* via `cargo tree --offline`. Seam with G.

## 4. Opportunities and uncertainties that are not findings

- **SIMD and batch evaluation of identical programs.** Bodies are deduplicated by content, and `CaseWorker::evaluate` interprets one instance at a time. Numerica 3.0.1 implements `Real` for `wide::f64x4/f64x8` without a feature gate, so `ExpressionEvaluator<f64x4>` can evaluate four instances of a body in one pass. The compiled `BatchEvaluator` requires `native_code_generation`. This could matter for collocation and many identical units. I did not raise it as a finding because:
  - the Require/Branch/Domain stages diverge per instance, so lanes need uniform branches or guarded masking;
  - the design says "JIT/SIMD evaluators are not admitted";
  - there is no measured cost.
- **Persisting optimized evaluators.** Symbolica 3.0.1 can serialize an `ExpressionEvaluator` (`serde`/`bincode` features). `pse-runtime::math::portable` persists admitted bodies and recompiles after reload, so Horner/CPE and dualization run again in every process. This becomes relevant only if cold-start or Python-session reuse becomes a target (RC03, unselected). Any persisted evaluator would have to be bound to the Symbolica version and optimizer options.
- **FeOS as a production provider.** PC-SAFT ships as authored package science (`packages/reference/thermodynamics/models/pcsaft.pse`), and FeOS 0.10.1 is reference-only (architecture overview, physical-semantics §9). Symbolic size grows at least quadratically with the number of components; `library.rs` notes 5,510 slots for a 3-component tangent-plane body. That is a scaling risk for many-component PC-SAFT. A FeOS provider (num-dual Helmholtz, its own phase-equilibrium routines) would remove that growth but goes against the deliberate knowledge-boundary decision. I record it as a measured-scaling question (probe P5), not as a finding.
- **Compiler access path.** `pse-compiler::workspace::modeling::executable::implicit` calls `has_path_connecting` for each child of each implicit stage, and scans every `model.symbols` entry once per stage. One reverse DFS per stage, or descendant intervals precomputed over the instance hierarchy, would make this linear. I did not establish the population of `p.implicit` (H9). This belongs to A/B.
- **Hot-path allocations.** `CaseWorker::evaluate` allocates `inputs` and `bits` `Vec`s per instance per callback, and `Worker::evaluate_seeded` allocates a dense `outputs × n × n` Hessian. This is B's area; it is not a library-fit problem.
- **POUNCE scoped pool.** `pounce::admitted` builds a scoped rayon pool on every call with more than one thread. Thread creation per solve is probably negligible next to the NLP. Noted for D.
- **Continuation, multistart and strategy (PS-09).** Module headers state that "Libraries own iteration". Corrector, fixed-point and pseudo-time iteration are delegated to KINSOL, PETSc and NLP solvers. Examined only shallowly; I saw no bespoke iteration loop.

## 5. Strengths and preservation constraints

- **Symbolica Taylor-mode derivatives** come from library `vectorize(Dualizer(HyperDual, zero_components))`, with structural zeros derived from conservative per-input support (`execution::compile_stages`, Block arm). Construction is bounded before allocation (`library::admission`). Preserve the admission bounds.
- **Sparse structure is prepared once and refilled per value**: `CasePlan` patterns plus the `jacobian_terms`/`hessian_terms` refill. This is the DP-10 shape.
- **faer symbolic LU is retained, and numeric storage is retained**, in `implicit::FactorWorkspace`, `dynamics::linear::FaerLu` and `square_response::actions`. The F04 eliminator factors once per sample (`dynamics.rs`; a test asserts `factorizations == 1`).
- **Ipopt uses `ReOptimizeTNLP`** through the C++ bridge rather than the C API's per-solve reconstruction. POUNCE keeps FERAL backends through the library's `LinearBackendFactory` hook. Both are good library-native reuse.
- **Post-solve analysis is library-owned and independent**: `pounce-sens-core` sensitivity, advanced-step prediction from one backsolve, and FERAL inertia and condition estimates. This satisfies PS-10.
- **IDAS and KINSOL are used through their full native surface**: reinitialization, sensitivity and adjoint on IDAS; Anderson acceleration and setup reuse on KINSOL. Recycles are KINSOL maps (`recycle.rs`: "no recycle iteration algorithm lives here").
- **Structure and presolve come from libraries**: `pounce-presolve` matching, DM and BTF; petgraph and rustworkx for SCCs and ordering; tears as a HiGHS MILP with a petgraph witness.
- **Salsa**: `Durability::HIGH/MEDIUM` on inventory fields, an equality-guarded `set_checked`, LRU with heap sizing, and body-level interning.
- Elsewhere, faer is called with explicit `Par::Seq` (fitting, square response, dynamics).

## 6. Positions on prior leads

| Lead | Position | Evidence |
|---|---|---|
| Production review §8: structural Taylor zeros | **Resolved** | `zero_components` computed in `execution::compile_stages` and passed to `Dualizer::new` in `library::admission::bounded_evaluator` |
| Production review §8: reusable numeric factor storage | **Residual** | faer paths retain `NumericLu`/`MemBuffer` (`FaerLu`, `FactorWorkspace`, `SparseFactor`); KLU and FERAL block paths still factor fully each time (L1-05) |
| Plan 28 enhancements F04 (IDAS refactors one sample matrix) | **Resolved** | `dynamics.rs` eliminator holds one `Lu` per sample; test asserts one factorization |
| Plan 28 enhancements F05 (dynamic workers rebuild layouts) | **Not examined** | C/D |
| Production review §8: Salsa "durability/backdating/LRU used" | **Confirmed**; finer catalog inputs still coarse (whole `CheckedPackage` input) | `workspace.rs` durability; `modeling.rs` `Catalog { checked }` |
| Production review §8: Ipopt/POUNCE retention | **Confirmed** strength | `pse_ipopt_sequence.cpp` `ReOptimizeTNLP`; `pounce/retained.rs` |
| Production review §8: Symbolica `optimize_stack` | **Confirmed** adequate | `bounded_evaluator` comment; architecture limit on `optimize_stack` after vectorization |
| RC02/RC03 | **Not examined** (L1 note: evaluator serialization exists upstream, §4) | symbolica-3.0.1 `evaluator.rs` serde/bincode impls |
| Solver-acceleration review, 2026-10-03 ("nothing owns decisions between attempts") | **Not examined** in depth | `math/strategy.rs` exists; C owns |
| PE01, PA03, EI05, state F02 | **Not examined** | B/H |

## 7. Candidate rule impacts

1. **"JIT and SIMD evaluation … are not admitted"** (workspace-and-dependencies §3.3, mathematics-and-compilation limits; Symbolica `native_code_generation` disabled in `Cargo.toml`).
   - *Proposed change:* allow an admitted Symbolica JIT or SIMD evaluator profile for bodies above a size threshold, and for batched identical-instance and study evaluation, with qualification under the existing FMA and `target-cpu` invariants (symjit code generation must not contract FMA unless the result is qualified).
   - *If the rule is kept:* L1-03 reduces to design-level worker reuse, and the §4 SIMD item is dropped.
2. **Crate topology (ADR required for a crate addition).**
   - *Proposed:* a separate surrogate crate (L1-07).
   - *If the rule is kept:* accept the closure cost, or move egobox to the consumer crate (`pse-runtime`) if `pse-math` does not need it.
3. **FeOS reference-only** (ADR-0098, physical-semantics §9). No change is proposed. P5 would only inform whether many-component PC-SAFT stays within budget.

## 8. Proposed light probes

All run under `scripts/pse-env --resource-class light` in scratch crates or existing test binaries; no store or services.

- **P1 (L1-02).** A scratch binary depending on `faer =0.24.4` with features `std,sparse-linalg,rayon`. It prints `faer::get_global_parallelism()` and factorizes a 40×40 and a 400×400 random sparse matrix with that `Par`. Run under `strace -f -e trace=clone3,clone -c`, once with the default and once with `faer::set_global_parallelism(Par::Seq)`.
  - *Discriminating outcome:* `Rayon(32)`, plus thread creation in the default run only, confirms that unadmitted threads are reachable. No threads even at 400 lowers the severity to configuration hygiene.
- **P2 (L1-01).** A `pse-backend-native` unit-test-style harness timing `square_response::response` against `SparseFactor::prepare` + solve on banded sparse square systems with n ∈ {200, 1000, 3000} and np = 3.
  - *Discriminating outcome:* dense time growing roughly cubically and above the solve's own cost at n ≈ 1000 confirms the material consequence.
- **P3 (L1-04).** Count `CubicRoots::solve` calls during one reference PR-flash solve (existing tracing span or a counter in a test build), and time `isolate_real_roots`+`refined` against `roots_hot_start` on PR-like coefficients.
  - *Discriminating outcome:* calls ≫ number of outer iterations, and exact isolation ≫ 10 µs per call, makes L1-04 material.
- **P4 (L1-03).** For a reference flowsheet case, log `CaseAssembly::numeric_worker_bytes()`, the per-`CompiledBody` `retained_bytes()`, and wall time of `worker_scoped` against one solve attempt.
  - *Discriminating outcome:* worker construction above roughly 5% of attempt time, or instruction bytes above roughly 10 MB per worker, makes L1-03 material for 16-way concurrency.
- **P5 (§4 FeOS).** Construction size (`count_operations`, slot count) and evaluation time of the authored PC-SAFT tangent-plane body for nc ∈ {2, 3, 5, 8}.
  - *Discriminating outcome:* superquadratic growth in slots or construction time indicates many-component limits relevant to the knowledge-boundary decision.
