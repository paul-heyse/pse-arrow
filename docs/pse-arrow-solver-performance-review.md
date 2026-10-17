# pse-arrow solver performance: diagnostics & optimization review

Oct 17, 2026 · @Paul · reviewed at `9cf6737`

## Bottom line

You already have good operation-level observability: the live hub, the Perfetto timeline and perf/samply capture. What's missing is visibility *inside* the solver iteration and *inside* the evaluator, and the yardstick you measure with is itself distorted. Six points drive everything else in this review:

1. **Fix the yardstick first.** Three things distort the numbers today:
   - The 52-case end-to-end timing corpus is built with the **dev profile plus Arrow force-validation**.
   - POUNCE's detailed phase timers **read zero by default**.
   - The always-on observation hub does `format!` calls, locks and linear name scans inside the inner loops. Its cost has never been measured.
2. **The evaluator is probably the biggest speed lever, not the solver.** This is inferred from code structure, not yet measured. Equations run as thousands of one-expression Symbolica bytecode blocks: a 3-component PC-SAFT body is 5,510 slots. Hessians are full per-output second-order jets that are only then multiplied by λ and summed. Instances are evaluated serially. The point cache is order-gated, so one interior-point iteration can evaluate each instance up to 5 times.
3. **The linear algebra runs with the brakes on, deliberately.** All BLAS uses `MKL_CBWR=COMPATIBLE`, which is the SSE2 code path. The C/Fortran solvers are built at generic `-O2`. FERAL runs with FMA forced off and serial by default. Each choice is defensible for reproducibility, but none has been priced.
4. **Structure is analysed well but not used for speed.** Dulmage–Mendelsohn and block-triangular form are computed for every solve. The default square solve is still one global KINSOL Newton iteration. The root route has no presolve, and recycles default to plain substitution.
5. **Re-solves throw away work.** Ipopt always runs with `warm_start_same_structure=no`, so METIS and the MUMPS analysis repeat on every solve in a sequence. KINSOL repeats its setup on every call. The post-solve KKT analysis re-factors from scratch each time.
6. **The single biggest gain in insight is a per-attempt "flight recorder".** It would hold:
   - a complete, columnar per-iteration trace;
   - linear-algebra statistics for each factorization;
   - worst-row and worst-column attribution mapped back to units and equations;
   - an evaluation ledger for each equation template.

   Section 4 specifies it. Without it, every speed decision above is a guess.

Two caveats. These findings come from reading the code, not from running it here; the native solver stack isn't buildable in this workspace. Wherever this doc says *likely* or *probably*, the figure comes from code structure and is the first thing to measure.

## How the pipeline executes today

Every solve passes through eight layers. You have good visibility at the two ends (preparation, operation phases) and little in the middle, which is where iteration time is spent.

| Layer | What actually runs (code owner) | What you can see today | Blind spot |
| --- | --- | --- | --- |
| 1. Compile | Authored `.pse` lowered to Symbolica `ExpressionEvaluator` bytecode, one small block per `let`/argument; hyper-dual jets of order ≤2 from `Dualizer` (`pse-math` `typed.rs`, `library/admission.rs`, `execution.rs`) | Preparation counters, compile spans, Salsa reuse | Program size per body: stages, blocks, op counts after dualization, frame bytes. These are computed for limit checks and then discarded |
| 2. Structure | Hopcroft–Karp, Dulmage–Mendelsohn, block-triangular form via pounce-presolve; normalization from nominals and tolerance budgets (`pse-structural`, `pse-math/normalization.rs`) | Structural admission errors with semantic IDs | Block-size histogram, largest strongly connected component, nominal-fallback counts, row/column-norm spread. These are on request only, or not computed at all |
| 3. Presolve | pounce-presolve affine elimination, redundant rows, structural rank, **NLP route only** (`presolve.rs`) | Pass flags; original vs presolved dimensions are computed | The dimension reduction is not exported as numbers; the root route has no presolve |
| 4. Route | KINSOL for square systems; Ipopt then POUNCE for NLP; HiGHS; Clarabel; Diffsol/IDAS (`execution/*.rs`, `math/strategy.rs`) | Chosen adapter, typed settings, provenance | Why a rung failed, and what the next rung inherited |
| 5. Callback chain | Ipopt C++ bridge → `foreign_nlp` → presolve `Transport` → `tnlp::Adapter` (normalization per element) → `CaseWorker` (`uno/callback.rs`, `presolve/pipeline.rs`, `tnlp.rs`) | Count and wall time per demand (f, g, ∇f, J, H) | About 6–8 O(nnz) passes and 4 allocations of size nnz per Jacobian/Hessian call; the split between these and real evaluation |
| 6. Evaluate | Serial loop over instances; per-(instance, demand) bit-exact cache that is order-gated; Hessian = `Demand::All` at second order (`assembly.rs`) | Nothing below the callback | Cost per template, unit and order; cache hits; dispatch vs arithmetic |
| 7. Linear algebra | MUMPS 5.9.1 sequential+METIS (Ipopt default), FERAL (POUNCE), KLU (KINSOL/IDAS), faer LU (Diffsol); oneMKL BLAS at `CBWR=COMPATIBLE` | Ipopt: δw per iteration. POUNCE: factor count, fill, pivot extremes, last inertia | MUMPS INFOG, inertia-correction count per iteration, δc, delayed pivots, refinement steps, KLU statistics |
| 8. Post-solve | Original-space quality check; KKT analysis with 1–3 fresh FERAL factorizations and a Hager–Higham estimate under `optimize` (`quality.rs`, `kkt.rs`, `conditioning.rs`) | Verdicts and per-row violations | Its cost relative to the solve itself |

The per-iteration Ipopt record (inf\_pr, inf\_du, μ, d\_norm, δw, α, line-search trials) does exist. But it lives in a 256-event rolling buffer shared with failure events, and reaches the hub only as latest-value gauges. A complete trace survives only through the durable `ProgressTap`, which drops events under lock contention.

## Step zero: fix the yardstick

Five fixes, most of them a day or less, should come before any optimization work. Without them a 10% gain is indistinguishable from build or observer noise.

| # | Problem (verified in source) | Fix | Effort |
| --- | --- | --- | --- |
| Y1 | `.config/process-cases.json` sets `"cargo_profile": "dev"`, and `scripts/case_measure.py` builds with `native-process,pse-relations/force-validate`. The dev profile keeps debug-assertions and overflow checks on for **all** crates, dependencies included. | Add `[profile.measure]` inheriting `release`. Time with it, without force-validate, and keep force-validate only for the functional-reference pass. Relabel past receipts as dev-profile numbers. | S |
| Y2 | Release builds your own crates at `opt-level = 2`, thin LTO, 16 codegen units. Generic faer/diffsol code is monomorphized *inside* `pse-math` and `pse-backend-native`, so it gets O2, not the dependency O3. | Try `[profile.release.package.pse-math]` (and `pse-backend-native`, `pse-kernels`, `pse-runtime`) at `opt-level = 3`, then a fat-LTO, `codegen-units = 1` variant (`dist` already exists). Optimization level does not change Rust floating-point semantics. | S |
| Y3 | The hub is on by default. Each `observation::scoped()` call does a `format!`, takes `try_lock` and scans up to 128 metric names linearly (`pse-diagnostics` `metric_index`). This runs on every Diffsol step, every NLP callback (twice on the presolved path), every KLU/faer solve and every provider evaluation. No benchmark measures it. | Run an A/B with `PSE_DIAGNOSTICS=off` on `value-sweep-1000-1`, `shooting-single-idas-cold-1`, `fit-gradient-cold-1` and `flash-cold-1`. Then replace names with handles registered in advance (an index plus a `&'static str`), batch the Diffsol counters every N steps, sample detail events for phases faster than about 1 kHz, and publish once per callback chain. | M |
| Y4 | POUNCE's 21 detailed timers stay at 0.0 unless `timing_statistics=yes`, which `pounce.rs` never sets. `IterRecord` history needs `enable_iter_history()` plus a collector, and neither is called, so the truncation code at `pounce.rs` around line 838 acts on an empty vector. Ipopt timers are also `NotRequested` outside one test. | Set `timing_statistics=yes` on both adapters; it costs an rusage call per timed section. Enable the iteration history or delete the dead code. Correct the evidence matrix (section 11). | S |
| Y5 | Host state is not recorded: no governor, boost or SMT state in `measurement_context()`. On your 9950X3D the timing cores 0–7 (and SMT siblings 16–23) sit on the **96 MB V-Cache CCD**; functional cores 8–15 sit on the 32 MB CCD. | Keep that split, which is the right call for timing. Record the CCD, boost and governor with every receipt. Never compare factorization-heavy numbers across the two CCDs. | S |

One possible correctness issue to check while doing Y3. On the presolved Ipopt path, two nested `CallbackState`s publish the same metric names (`callback.jacobian.completed_calls`, …) with different observation generations. If they share one context, counters could read as resets and the `.seconds` gauges could flip between the inclusive and the inner value.

## Target diagnostic architecture: a per-attempt flight recorder

The target is a recorder for every solve attempt with seven levels. The cheap levels L0–L3 are always on; L4–L6 run on request or on failure. It extends your existing hub and the `AttemptObservation` idea from the 2026-10-12 solver review rather than replacing them.

| Level | Question it answers | Record | Default |
| --- | --- | --- | --- |
| L0 Phase | Where did wall time go, and was it on-CPU? | Existing phases, plus `CLOCK_THREAD_CPUTIME_ID` at phase boundaries (on-CPU vs wall, so waits become visible) and log₂ duration histograms with min/max instead of sum plus count | On |
| L1 Attempt | What ran, and how was its time split? | Effective settings as resolved; iterations; evaluation counts per demand; time split into evaluation, callback overhead, linear algebra and solver-internal; termination; strategy rung | On |
| L2 Iteration | How did convergence progress, and why was each step limited? | **NLP:** μ, inf\_pr, inf\_du, complementarity, objective, ‖d‖, α\_pr, α\_du, acceptance character (f/h), line-search trials, δw, δc, number of factorizations, inertia corrections, inertia (+/−/0), refinement steps and residual ratio, restoration / second-order-correction / watchdog flags, eval/factor/solve ns. **KINSOL:** scaled ‖F‖, step length, line-search λ, Jacobian refresh, linear iterations. **IDAS/Diffsol:** t, h, order, error-test and Newton failures, setup reason, step-limiting variable (all failures plus every Nth step) | On |
| L3 Factorization | Is linear algebra slow, unstable or wasted? | n, nnz(A), nnz(L), fill, flops, max front, ordering and scaling actually used, delayed pivots, tiny pivots, pivot extremes, inertia, fresh vs reused symbolic analysis, symbolic/numeric/solve ns, free condition estimates (KLU rcond, FERAL κ̂) | On |
| L4 Model attribution | *Which part of the flowsheet* blocks convergence? | Per iteration: the top-k rows of inf\_pr and columns of inf\_du mapped to semantic IDs, BTF block and unit; variables pinned at bounds; nonfinite and domain events per equation | Top-1 on; top-k on failure |
| L5 Evaluation ledger | Which equation template or unit costs the time? | Per template × order × demand: calls, cache hits, misses due to order, ns, provider calls, inner implicit solves. A static census at compile: stages by kind, blocks, op counts before and after dualization, jet width, frame bytes | Counters on; census at compile |
| L6 Hardware | Is the code compute-, memory- or latency-bound? | Hardware counters per phase (IPC, last-level-cache misses, branch misses), off-CPU stacks, heap profile | On request |

L4 is what sets an equation-oriented simulator apart. "The reboiler energy balance in C-101 holds most of the primal infeasibility from iteration 12 to 40 while the reflux flow sits on its bound" is the finding that tells you whether to change scaling, the start point, the formulation or the method. Your normalization and semantic-ID maps already make this a lookup, not a research problem.

Four implementation rules keep it cheap and consistent with your contracts:

- **No strings in the hot path.** Metric keys are interned at preparation, and records are fixed-width numeric structs.
- **Columnar from the start.** Each attempt owns a preallocated struct-of-arrays ring (for example 4,096 iterations, then a decimated tail). At attempt end or stop it flushes to an Arrow `RecordBatch`, which lands in your store or Parquet and is queried with DataFusion. Arrow is already your data plane, so this adds no new technology.
- **Unknown is not zero.** Fields a backend cannot supply are typed `Unavailable`, which matches your existing coverage semantics.
- **A budget that is measured.** L0–L3 should cost under 1% of solve time on the small-case corpus. Prove it with the Y3 A/B; don't assume it.

On top of L2 and L3 sit cheap automatic classifiers, flagged per attempt:

- restoration entered (and from which iterate);
- an inertia correction in more than 30% of iterations;
- line-search collapse (α < 1e-8 three times running);
- μ stall;
- inf\_pr stagnation over 10 iterations;
- an unchanged worst row for more than 20 iterations;
- recycle contraction ratio above 0.9.

A failed attempt then triggers L4 top-k plus a sparse smallest-singular-vector analysis on the offending BTF block.

## Numerical-method and problem-structuring opportunities

The methods themselves are sound: Ipopt-class interior point, KLU Newton, BDF/IDAS, an exact MILP tear selector, and three VLE formulations. The gains are in exploiting structure you already compute and in not repeating work across solves. Impact ratings are inferred from code structure. The last column says what to measure before committing.

| # | Opportunity | Evidence | Impact | Effort | Decide by measuring |
| --- | --- | --- | --- | --- | --- |
| N1 | **Block-decomposed root solve as the default.** Newton per block in BTF order; 1×1 blocks solved by scalar bracketing (`implicit_scalar` exists); small blocks coalesced; one evaluator restricted per block; finish with one global polish and the existing original-space check | Default root solve is one global KINSOL Newton (`settings/kinsol.rs`). Block solves exist only as opt-in initialization, capped at stages×blocks ≤ 4096 (`math/initialization.rs`) | High | M–H | Block-size histogram and largest strongly connected component across the corpus |
| N2 | **Weighted second order instead of per-output Hessians.** Build the symbolic gradient of Σ wᵣ fᵣ with the weights as parameters, then apply the first-order `Dualizer` to it. Same for IDAS `weighted_hessian` → Hessian-vector product | `CaseWorker::hessian` evaluates `Demand::All` at second order, then scatters λ·H per output. Cost goes from about ops·m·(1+3n+2n²) to about ops\_grad·(1+2n), with Symbolica still the differentiator (ADR-0082) | High for exact-Hessian NLPs | H | Hessian share of callback time (L5) |
| N3 | **Keep analysis across re-solves.** Allow Ipopt `warm_start_same_structure=yes` for signature-identical problems, or route sequences to POUNCE, whose FERAL keeps symbolic state. Use `KINSetNoInitSetup(1)` on compatible re-solves. Reuse symbolic analysis in `conditioning::factor` | The bridge forces `warm_start_same_structure=no` on every call; KINSOL uses `KINSetNoInitSetup(0)`; conditioning starts from a fresh `feral::Solver` each time | High for continuation, sweeps, recycles | M | Symbolic-analysis share of factorization time (L3) |
| N4 | **Choose the linear solver by size.** Benchmark MUMPS with METIS vs AMD, SPRAL and Pardiso across KKT sizes; select automatically with typed reasons; type and record `mumps_pivtol`, `mumps_mem_percent` and permuting scaling. In FERAL, run the ordering race once per pattern and persist the winner | Only explicit selection today; METIS by default even for small KKTs, where AMD is usually cheaper | Medium | M | Factor ns and fill vs n and nnz (L3) |
| N5 | **Tune Newton and barrier settings.** KINSOL `msbset` 10 → try 1–3 when Jacobians are cheap. Add adaptive μ as a declared Ipopt retry rung (POUNCE has one). Align POUNCE `tol` (min of budgets) with Ipopt (max). Revisit `bound_relax_factor=0` while keeping the original-bound certification | `kinsol.rs` `KINSetMaxSetupCalls(10)`; `solve.rs` pins `bound_relax_factor` to 0; the two adapters compute `tol` differently | Medium | S | Jacobian evals per iteration; iterations and restoration entries |
| N6 | **Jacobian-based equilibration as a separate conditioning layer.** Curtis–Reid, geometric-mean or Ruiz at the start point for rows without authored scaling, kept separate from acceptance budgets; most valuable for KINSOL | Row scales come from tolerance budgets (`with_row_budgets`); magnitude-aware scaling exists only where authored or inside the solver | Medium–High (robustness, iterations) | M | Row/column-norm spread at the start; iterations before and after |
| N7 | **Better recycle convergence.** Default to Anderson acceleration (memory 3–5, delay 1–2, damping); add Newton/Broyden on the tear residual; derive the contraction ratio from `RawMapHistory` | `requests.rs` defaults `anderson: 0`, i.e. plain substitution (also flagged as F10 in the 2026-10-12 audit) | High for recycle-heavy plants | M | Sweeps to converge; contraction ratio |
| N8 | **Presolve the root and initialization routes.** Aggregate the per-coordinate connection equalities and fixed variables before KINSOL; roots need no multiplier recovery | `process_connection` emits one `sa == sb` per coordinate; presolve runs on the NLP route only | Medium–High | M | Rows removed per case |
| N9 | **Stop forcing IDAS onto every output sample.** Set the stop time only at segment ends and discontinuities and interpolate outputs. Benchmark the staggered vs simultaneous sensitivity corrector, and KLU vs faer as the Diffsol default | `IDASetStopTime` before every sample (`idas.rs` `segment()`), which caps the step at the sample spacing | Medium–High on dense output grids | S | Steps and linear setups per simulated second |
| N10 | **Retune refinement inside FERAL.** Restore POUNCE's carve-out that switches off inner refinement under L-BFGS, and benchmark a refinement residual target | pse-arrow passes `FeralConfig::default()` (10 inner steps) nested inside POUNCE's outer refinement; POUNCE's own notes record backsolve at 49 of 62 s on one L-BFGS case | High on ill-conditioned KKTs | S to test | Refinement steps per solve (L3) |
| N11 | **Formulation robustness.** ε-continuation for smooth and complementarity VLE inside the existing homotopy; express `rr_correction`'s absolute 1e-12/1e-6/1e-8 relative to nominals; allow bound tightening for initialize/root intents, where bound duals don't matter | `equilibrium.pse`; `presolve.rs` marks `Fbbt`/`LinearBounds` ineligible everywhere | Medium | S–M | Failure rate and iterations on the flash cases |

N1, N3 and N7 compound. A plant with recycles that is solved block by block, with symbolic reuse and Anderson acceleration on the tears, is the configuration where best-in-class equation-oriented tools get their speed.

## Execution-efficiency opportunities

The evaluator is where code efficiency is most likely to pay, because every solver iteration multiplies its cost. Measure L5 first (E1), since its numbers decide whether E2/E3, N2 or E5 comes next.

### Evaluation engine

| # | Opportunity | Evidence | Impact | Effort |
| --- | --- | --- | --- | --- |
| E1 | **Evaluation ledger plus a static program census** (L5). Fixed arrays indexed by request × order × demand inside `CaseWorker::evaluate_inner`. Publish the `count_operations()` results that are already computed for limit checks. Add a bench that replays one body's worker on recorded points | Only the outer callback is timed; op counts are computed and discarded (`library/admission.rs`) | Prerequisite | S |
| E2 | **Value numbering over the stage program.** Merge blocks with identical normalized expressions over identical input slots, keeping the first occurrence's source attribution and `independent()`'s distinct-argument rule | Every `let`, function argument and call site materializes its own block, so `moment(…)` and the potential are re-lowered for each response | High (thermo bodies) | M |
| E3 | **Fuse straight-line blocks** into one multi-output dualized evaluator per region between branches and providers. This cuts dispatch and gather/scatter, and lets Horner and pair elimination work across blocks | One expression per block; per-block gather, finiteness scan and scatter in `evaluate_stages` | High | H |
| E4 | **One cache, ordered by order.** A higher-order result at the same bits serves lower-order requests; objective and constraint groups merge per instance; `new_x` is honoured | `worker.cache.is_some_and(\|o\| o >= order)` per demand group; Hessian uses `Demand::All`, which never sees the other caches | Medium (removes 1–2 of 5 evaluations per iteration) | S–M |
| E5 | **Parallelize identical templates.** Step 1: rayon over instance chunks with per-thread workers and a serial scatter, so floating-point order stays deterministic. Step 2: struct-of-arrays lanes for SIMD | The `evaluate_inner` loop is serial; instances already share `workers[group.request]` | High for columns and repeated stages | M, then H |
| E6 | **Compile derivative orders natively,** one module per body or region (after E2/E3), keeping `-ffp-contract=off -march=x86-64` | `compiled-evaluator` is Value-only, one module per block, and enabled by no consumer | High potential | M |
| E7 | **Hot-path hygiene.** Poison only slots that can be read before being written, instead of NaN-filling the whole frame (about 0.9 MB per instance per Hessian call for a 5,510-slot body at n = 5). Precompute per-term output offsets to replace `binary_search` and the `BTreeMap` lookup per term. Reuse output vectors | `execution.rs` `evaluate_seeded`; `assembly.rs` `hessian`/`constraints`/`gradient` | Low–Medium each | S |
| E8 | **Cheaper implicit inner solves.** An n-only program for the inner Newton Jacobian; a point cache shared by residual and Jacobian; verification and derivatives merged into one evaluation that reuses the converged factor; the second-order implicit-function right-hand side as (Hᵢ·T) then Tᵀ(Hᵢ·T), not the dense quadruple loop | `pse-math/implicit.rs`; `pse-backend-native/implicit.rs` discards the p parameter columns it computed | Medium–High for flash- and density-root-heavy plants | S–M |

### Callback chain and solver wrappers

| # | Opportunity | Evidence | Impact | Effort |
| --- | --- | --- | --- | --- |
| E9 | **One pass from evaluator to solver buffer.** Write into the caller's slice through one scratch buffer owned by the context; precompute a fused scale per nonzero; replace `Transport::refill` with a permutation | About 6–8 O(nnz) passes and 4 allocations of size nnz per Jacobian/Hessian call on the presolved Ipopt path | High for many small solves | M |
| E10 | **Remove fixed per-solve costs.** Drop the bridge's `std::set` duplicate check in `coo()`, since the CSC is already canonical. Forward `structural_analysis` through `Transport` so `validate_nlp` doesn't re-analyse. Reuse the per-solve rayon pool in `pounce.rs`. Use CSR preallocation in PETSc instead of one `MatSetValues` per nonzero | `pse_ipopt_sequence.cpp`; `pounce.rs` around line 177; `pse_petsc.c` | Low–Medium | S |
| E11 | **Strip FERAL/POUNCE overhead per factor (you own the fork).** Cache the `FERAL_AUTO_CB_DEBUG` and `POUNCE_DBG_*` env reads; skip SipHash when the caller certifies the pattern; drop the MC64 `Vec` clone, the `dump_rhs` clone and per-solve `SolveWorkspace` allocation; cut the four KKT value copies; give numeric factors a reusable arena | `feral` `solver.rs`, `solve.rs`; `pounce` `t_sym_solver.rs`, `std_aug_system_solver.rs` | Medium on small and medium KKTs | S–M |
| E12 | **Replace polling with wake-ups.** The 10 ms admission/resume retries (`math/jobs.rs`, `staged.rs`), the 2 ms native-child poll, and Uno's process-global mutex taken through a 1 ms sleep loop | Measured as off-CPU time once L0 thread CPU time exists | Low–Medium (latency) | S |
| E13 | **Try a different allocator.** A/B mimalloc as `#[global_allocator]` in `pse-py` and the worker binaries; tune `MALLOC_ARENA_MAX` for multi-lane studies | System malloc everywhere; heavy allocation in Salsa, Symbolica and the Arrow builders | 0–10% (inferred) | S |

## Tooling: what to install and enable

Your machine already has most of the core stack: perf 7.0, samply, hotspot, heaptrack, valgrind, bpftrace, cargo-flamegraph, hyperfine, cargo-asm. What's missing is **hardware-level analysis for Zen 5, deterministic regression counts, an iteration-level live timeline, and off-CPU visibility**. The last one is blocked by `perf_event_paranoid = 2`.

### Host settings to change (development box only)

- [ ] `sudo sysctl kernel.perf_event_paranoid=1` (persist in `/etc/sysctl.d/`). This enables kernel stacks and `sched_switch` for off-CPU analysis. The alternative is to grant `cap_perfmon` to perf only.
- [ ] `sudo sysctl kernel.kptr_restrict=0`, so kernel symbols resolve in profiles.
- [ ] Add a `profiling` recipe that builds `--profile profiling` with `RUSTFLAGS="-C force-frame-pointers=yes"`, and a solver-image variant with `-fno-omit-frame-pointer`. Capture can then use `--call-graph fp` at 499–999 Hz instead of 99 Hz DWARF. MKL stays DWARF-only because it is a closed binary.

### System tools

| Tool | Use it for | Status | Action |
| --- | --- | --- | --- |
| [AMD uProf](https://www.amd.com/en/developer/uprof.html) (`AMDuProfCLI`, `AMDuProfPcm`) | Zen 5 pipeline analysis: IBS precise sampling, cache/memory-bandwidth counters per CCD, roofline | Not installed | Install 5.3. It's AMD's own tool and the most direct view of V-Cache effects. Its page lists IBS metrics for Zen 5 but some features as EPYC-only, so check Ryzen 9000 coverage |
| perf (`stat`, `record -b`, `c2c`, `sched`/`trace`) | Counters per phase (IPC, last-level-cache and branch misses); branch stacks for PGO/BOLT; false sharing (FERAL's mutex-guarded contribution blocks); off-CPU | Installed; hardware events unused | Add `Counters` and `OffCpu` capture kinds |
| bcc tools (`offcputime`, `profile`) | Off-CPU flame graphs for admission waits, joins and polls | Not installed | `apt install bpfcc-tools` |
| [Tracy](https://github.com/wolfpld/tracy) profiler GUI | Live timeline at nanosecond scale for single-solve deep dives (every callback, factorization and iteration) | Not installed | Build the GUI at the same version as the `tracy-client` crate you pin; the protocol must match |
| [coz](https://github.com/plasma-umass/coz) | Causal profiling: "speeding up X by 10% speeds up the solve by Y%", with a progress point per iteration or per case | Not installed | `apt install coz-profiler`; use the `coz` crate for progress points |
| llvm-bolt + [cargo-pgo](https://github.com/Kobzol/cargo-pgo) | PGO, then BOLT post-link layout, trained on the corpus | Not installed (llvm-tools is pinned) | `cargo install cargo-pgo`; LLVM apt for `llvm-bolt` |
| valgrind (cachegrind, callgrind, DHAT) | Deterministic instruction and cache counts, through gungraun | Installed | Use through gungraun (below) |
| heaptrack, hotspot | Native heap including MKL/MUMPS; perf.data GUI | Installed | Add a heap capture kind using `LD_PRELOAD` heaptrack |
| [AOCL](https://www.amd.com/en/developer/aocl.html) 5.x (BLIS, libFLAME) | A BLAS/LAPACK candidate tuned for Zen, to A/B against oneMKL | Not installed | Optional: build an image variant (decision D1) |

### Rust crates

| Crate | Role | Where it plugs in |
| --- | --- | --- |
| [gungraun](https://docs.rs/crate/gungraun/) 0.20 (successor to iai-callgrind) | Instruction-count benchmarks that don't depend on machine noise, for regression detection | Kernels: one body's evaluator, CSC refill, KLU refactor/solve, FERAL factor, canonicalization |
| `tracy-client` / `tracing-tracy` | Tracy integration as a feature-gated `tracing` layer | Compose through your caller-owned subscriber (`session.layer()` pattern); never global |
| `perf-event2` | In-process `perf_event_open` counter groups read at phase boundaries | L6 per-phase IPC and misses without an external perf process |
| `dhat` | Rust heap profiling (allocation counts and sizes per call site) in tests and benches | Feature-gated `#[global_allocator]` in bench and test binaries |
| `mimalloc` | Allocator A/B (E13) | `pse-py` and worker binaries |
| `coz` | Progress points for causal profiling | Iteration end, case end |
| `inferno` | Collapsed stacks and **differential flame graphs** between commits | Post-process the `perf.data` your capture already keeps |

### Numerical libraries worth a look

- **Sparse smallest singular vectors for the on-failure diagnostic.** Shift-invert Lanczos on JᵀJ reusing the KLU/FERAL factor is a few hundred lines of Rust. ARPACK-ng or PRIMME are the off-the-shelf alternative. Today's SVD is dense and capped at `dense_entries`.
- **SUNDIALS newer than 7.1.1.** Your own 2026-10-12 evidence notes that later releases add per-iteration KINSOL hooks (`KINDampingFn`, `KINDepthFn`) and in-process logger callbacks, which removes the main obstacle to KINSOL L2 traces.
- **HSL MA57/MA97 as a comparator only.** ADR-0108 excludes HSL from the product, but it is the reference Ipopt linear solver in the literature. A local benchmark run against it (CoinHSL is free for academic and personal use) tells you how far MUMPS, SPRAL and FERAL are from best in class on *your* KKT systems.

### Ipopt options for deep dives (not reserved, so they can be passed through `controls.options` today)

`timing_statistics=yes`, `print_timing_statistics=yes`, `print_info_string=yes`, `output_file=<path>` with `file_print_level=5` (8 for linear-algebra detail), and `derivative_test=second-order` on small cases.

## Experiment protocol

Use three tiers of evidence, from noiseless to noisy, and lean on the noiseless ones. Iteration and factorization counts don't vary between runs. They catch most numerical regressions at zero noise, and they are what most method changes in section 5 are about.

| Tier | Metric | Noise | Use |
| --- | --- | --- | --- |
| 1. Algorithmic | Iterations, factorizations, inertia corrections, evaluations per demand, restoration entries, sweeps per recycle (from L1–L3) | None (deterministic build) | Regression alerts on every commit; the primary metric for N-items |
| 2. Instructions | gungraun instruction and cache counts per kernel | None | Regression alerts on E-items; diagnosing compute vs memory behaviour |
| 3. Wall time | Median of ≥10 runs, interleaved A/B/A/B, bootstrap confidence interval, timing cores 0–7 only | Low if pinned | Final verdict; shifted geometric mean across the corpus |

**Comparing methods or settings:** use Dolan–Moré performance profiles, the standard in optimization benchmarking. For solver s on problem p, with time t (failures counted as ∞):

```latex
r_{p,s} = \frac{t_{p,s}}{\min_{s'} t_{p,s'}}, \qquad \rho_s(\tau) = \frac{\left|\{\, p : r_{p,s} \le \tau \,\}\right|}{|P|}
```

ρ(1) is how often a configuration is fastest, and ρ at large τ is its robustness. One plot answers "MUMPS+METIS vs AMD vs FERAL vs SPRAL" or "msbset 1 vs 10" without averaging away failures.

**Corpus.** The 52 workloads in `process-cases.json` are a good base. Add scale tiers where best-in-class tools differ most:

- a 30–60-stage column with five or more components;
- a plant with three or more nested recycles;
- an exact-Hessian optimization of about 10⁵ variables;
- a dynamic start-up with dense output.

Tag each case by its dominant cost class (evaluation-, factorization- or overhead-bound) once the recorder exists.

**One home for results.** Write receipts to a table in your own store, keyed by commit, host fingerprint (CPU, CCD, boost), build profile, CBWR branch and threads, and query them with DataFusion. That keeps one source of truth across your build and audit lanes, rather than adding bencher.dev or CodSpeed as a second ledger. An option-sweep harness over linear solver × ordering × μ strategy × Hessian mode × scaling then produces the performance profiles, and the flight recorder explains each winner.

## Prioritized roadmap

There are five phases, and measurement gates each one. Phases 0 and 1 are mostly small changes, and they decide the order of everything after them.

**Roadmap: the yardstick and the recorder gate every optimization phase**

1. **Phase 0: Fix the yardstick** (Y1–Y5, D7)
    - Measurement profile without force-validate; opt-level 3 trial
    - Observer A/B with `PSE_DIAGNOSTICS=off`; turn on POUNCE and Ipopt timers
    - Record CCD, boost and governor with every receipt

    *Gate: trustworthy baseline receipts and the observer's measured cost*

2. **Phase 1: Build the recorder** (L0–L3, L5, E1)
    - Columnar iteration trace; factorization stats from FERAL, KLU, MUMPS
    - Evaluation ledger and program census; thread CPU time, histograms
    - Counters, off-CPU and heap capture kinds

    *Gate: every corpus case tagged evaluation-, factorization- or overhead-bound*

3. **Phase 2: Configuration wins** (N3, N5, N7, N9, N10, E4)
    - Symbolic reuse across re-solves; Anderson on recycles; IDAS stop times
    - KINSOL setup interval; refinement budget; unified evaluation cache
    - Price D1–D2: BLAS branch, FMA, ISA level

    *Gate: performance profiles beat the Phase 1 baseline on tier 1 and tier 3 metrics*

4. **Phase 3: Exploit structure** (N1, N6, N8, E2, E5, E9, L4)
    - Block-decomposed root solve; presolve on the root route; equilibration
    - Value numbering; parallel templates; single-pass callbacks
    - Model attribution and on-failure sparse SVD

    *Gate: evaluation share and iteration counts re-measured on the scale tiers*

5. **Phase 4: Deep engine work** (N2, E3, E6, E11, PGO)
    - Weighted second-order Hessians; fused multi-output blocks
    - Native compiled derivative programs; SIMD lanes; PGO and BOLT
    - FERAL dense kernels and reusable factor arenas

Phase 2 is configuration-level and can start for any item once Phase 1 shows where its case spends time. Phases 3 and 4 are design work that would go through your plan and ADR route. Items N2, E3 and E6 touch the compilation contract (ADR-0082).

## Decisions only you can make

Several of the largest potential gains are reproducibility or certification choices, not engineering ones. Each would go through your ADR route (ADR-0108 and the AGENTS.md invariants). Price each one with the measurement in the last column before deciding.

| # | Decision | Today | Option | What you trade | Measure first |
| --- | --- | --- | --- | --- | --- |
| D1 | BLAS code path | `MKL_CBWR=COMPATIBLE`, enforced by `pse-env`, which refuses `AUTO`. Intel documents this as the SSE2 branch, and on AMD only `AUTO` or `COMPATIBLE` exist | `AUTO` (AVX2/AVX-512 kernels) with fixed thread counts, or AOCL BLIS/libFLAME | Bitwise equality across different CPUs, for faster dense kernels in MUMPS, SPRAL and Pardiso fronts | Share of factorization time in `mkl_blas_*` symbols (perf report by DSO) on the largest cases. If it's under 10%, keep COMPATIBLE |
| D2 | Floating-point contraction and target ISA | C/Fortran at `-O2` with no `-march`; FERAL `fma=false`; Rust at baseline x86-64 | C/Fortran at `-march=x86-64-v3 -ffp-contract=off` (the pattern `native_compiled.rs` already uses); FERAL FMA only for fronts above a size threshold | `-ffp-contract=off` keeps v3 contraction-free, so mainly portability to pre-2015 CPUs; FMA changes the last bits | Front-size distribution and dense-kernel share (FERAL `work_estimate`, L3) |
| D3 | What "reproducible" means | The AGENTS.md rule against `target-cpu=native` cites FMA contraction | Restate it as *same-host, same-thread-count determinism* plus cross-host agreement within tolerance | Clarity. rustc never contracts `a*b+c` on its own, and faer/pulp and FERAL already pick SIMD kernels at run time, so cross-host bitwise equality is already not guaranteed for those kernels | None; this is a wording decision that unblocks D1 and D2 |
| D4 | Ipopt state across a sequence | `warm_start_same_structure=no` on every call, so scaling and the MUMPS analysis are rebuilt | `yes` under the full signature identity you already compute, or route sequences to POUNCE | Gradient scaling stays fixed from the first solve of the sequence | Analysis share of time on continuation and sweep cases (L3) |
| D5 | Default parallelism | `Controls.threads = 1`; MUMPS sequential; FERAL serial | Admitted threads for KKTs above a size threshold; FERAL's parallel path is documented as byte-exact deterministic | Throughput of concurrent studies vs latency of a single solve | Speed-up vs n and nnz; study throughput |
| D6 | Certification margins | `bound_relax_factor=0`; acceptable termination off; POUNCE `tol` = min of budgets | 1e-8 relaxation with post-solve projection and the existing original-space check | Iterations and restoration entries vs an extra projection step | Restoration-entry rate on bound-heavy flowsheets |
| D7 | Diagnostics overhead budget | Undefined; the hub is always on and unmeasured | Make it explicit, for example L0–L3 at ≤1% of solve time on the small-case corpus | Sets how much L2/L3 detail can be always-on | Y3 A/B |

## Evidence basis, verification and corrections

This review is static. It covers `paul-heyse/pse-arrow` at `9cf6737` (your local checkout is identical and clean) and the POUNCE fork at `d02beafd`. Five parallel reviews covered: solver adapters; evaluation and derivatives; structure and formulation; runtime and performance infrastructure; POUNCE/FERAL. The native solver image isn't available in this workspace, so nothing was built or timed. Impact estimates are inferences to be confirmed by the measurements named in each row.

**Re-checked directly in source** (the load-bearing claims):

- `cargo_profile: "dev"` in `.config/process-cases.json`; force-validate features in `case_measure.py`
- `observation::scoped` formats a name on every call; `metric_index` is a linear scan
- `pounce.rs` sets `feral.fma = false` and `parallel = threads > 1`, and never sets `timing_statistics` or iteration history. POUNCE gates these at `application.rs` (around lines 4345 and 943)
- the Ipopt bridge forces `warm_start_same_structure=no`; `bound_relax_factor = 0` in `solve.rs`
- KINSOL `KINSetMaxSetupCalls(10)` and `KINSetNoInitSetup(0)`; `IDASetStopTime` inside the per-sample loop
- Symbolica `Optimization::default()` is interpreted, Horner 10; the evaluation cache is order-gated; the Hessian evaluates `Demand::All` at second order
- Uno's 1 ms sleep loop on a process mutex; `std::set` in the bridge's `coo()`; recycle `anderson: 0` default in `requests.rs`
- `MKL_CBWR=COMPATIBLE` pinned, with `AUTO` refused; timing cores 0–7 on the 96 MB L3 CCD (checked on your machine via sysfs)

**Corrections to existing documents** (for your audit lane):

1. `docs/design_review/evidence/solver-methods-and-diagnostics-2026-10-12/diagnostics-and-measurement.md` §6 lists POUNCE as "Captured: `IterRecord` trajectory" and "Captured: 22 phase timers". With the adapter as written, default runs capture neither: the history is empty and the detailed timers read 0.0. Either enable them (Y4) or amend the matrix.
2. Blueprint §15 opens by saying there is "no named structural/numerical check catalogue". The code now has one: `FindingKind` with 25 rules mapped to `DiagnosticRule`. The section should be amended through your design route.

**Sources** (opened during this review):

- [Intel oneMKL: specifying code branches (CBWR)](https://www.intel.com/content/www/us/en/docs/onemkl/developer-guide-linux/2025-0/specifying-code-branches.html): COMPATIBLE = SSE2 branch; non-Intel CPUs support only AUTO or COMPATIBLE
- [gungraun on docs.rs](https://docs.rs/crate/gungraun/): successor to iai-callgrind, 0.20.0 (2026-09-26)
- [AOCL 5.1 release notes](https://www.amd.com/en/developer/resources/technical-articles/2025/what-s-new-in-aocl-5-1--faster--and-more-scalable-math-libraries.html)
- [AMD uProf](https://www.amd.com/en/developer/uprof.html): 5.3, June 2026
