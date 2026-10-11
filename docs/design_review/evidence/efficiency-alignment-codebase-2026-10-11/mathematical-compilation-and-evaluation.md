# Mathematical compilation, evaluation and properties: bounded supporting assessment

Inquiry **B** of the [principal efficiency-alignment review](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md). This is supporting
evidence, not an independent principal review, finding ledger or qualification receipt. The
principal review owns the combined judgment, finding identifiers (`F01`…) and rule impacts
(`RC01`…); local labels below (`B-nn`) are investigation labels only. Coordinator source
checks, reconciliation and probe results are in the [evidence README](README.md).

Baseline: `main` at `b01a45d71a5f5ab29991ad193679f5588ac3662f` (2026-10-11), production source clean.
Assessed by a fresh read-only `design-reviewer` against Core 3.4, Heuristics for Efficient Architecture
1.0, Process Simulator 1.5 and the [pse-arrow binding](../../design_principles/binding/pse-arrow.md).
No builds, tests or services were run by the inquiry; probes it proposes are listed for later
settlement unless the README records them as executed.

---

Bounded supporting assessment for the efficiency-alignment review (baseline `main` at
`b01a45d71a5f5ab29991ad193679f5588ac3662f`, Core 3.4 / Process Simulator 1.5 / Heuristics 1.0).
Local labels B-01…B-10. The principal reviewer owns the combined judgment and finding IDs.

## 1. Scope examined and coverage limits

**Read in detail.** `pse-math`: `library.rs` and `library/admission.rs`, which hold Symbolica
evaluator construction and Dualizer vectorization. `jets.rs`, which holds `JetLayout` and
`ProviderLift`. In `execution.rs` I read `PreparedBody::compact`, `compile_scope`,
`compile_stages`, `evaluate_stages`, `CompiledBody`/`Worker`, and part of the support analysis
`analyze`. I also read `assembly.rs` (`CasePlan::prepare_with_source_support`, `conditional`,
`CaseAssembly`, `CaseWorker`) and `binding.rs` (`CaseValues`, `InstanceBinding::values_selected`,
`restricted`). The rest of the `pse-math` reading covered `sparse.rs::AssemblyMatrix`,
`normalization.rs`, `numerics.rs::{TargetAccess, resolve, engineering_error_quantity}`,
`implicit.rs` (`Problem`, `derivatives`, `Factory`, `Nested`), `implicit_cubic.rs`,
`implicit_accelerators.rs`, `implicit_selection.rs::{anchor, verify}`, the entry of
`implicit_regimes.rs::RegimeSelection::evaluate`, `presolve.rs::presolve_domain_facts` and
`class_evidence.rs::{class_evidence, shape_facts}`.

In `pse-compiler` I read `workspace.rs::{ArtifactRequest, artifact_requests, prepare_class,
ValueProducts}` and `executable/solve.rs::{values_match, rebind}`. In the `pse-runtime` math
service I read `math.rs` (`MathService::{workspace, assemble, worker, with_owned_worker}`,
`attempt_providers`), `math/artifacts.rs`, `math/retention.rs::{Pressure::try_grow,
BodyRetention}`, parts of `math/jobs.rs` and `math/modeling.rs`, and parts of `math/solves.rs`:
`discover_class`, `prepare_solve`, `prepare_resolved`, `case_worker`, `OriginalCase` and the
factorable re-solve closures. From `pse-kernels` I read `Provider`, `ProviderValues::validate`
and `ProviderFactory`. From `pse-quantity` I read `CanonicalConversionPlan::apply`. I also read
the seams `pse-backend-native/src/assembled.rs::AlgebraicOracle` and
`workflow/dynamics.rs::DynamicWorker::{assign, evaluate}`.

**Libraries.** I read the pinned Symbolica 3.0.1 source under `~/.cargo/registry`:
`evaluate/evaluator.rs` (the `ExpressionEvaluator` struct, `try_evaluate`, the interpreter
loop), `evaluate/instruction.rs::Instr`, `evaluate/dual.rs::{vectorize, Dualizer}`,
`evaluate/backend.rs::{jit_compile, JITCompiledEvaluator}` and the `Cargo.toml` features. I
also used the `symbolica-faer-oximo` skill and the prior evidence in
`docs/design_review/evidence/production-execution-efficiency-2026-10-07/math/README.md`.
Context7 was not needed: the pinned source was decisive.

**Not read, or only skimmed:** `typed.rs`/`typed_math.rs` lowering internals,
`derived.rs` (4,385 lines), `factorable.rs`, `curvature.rs`/`convexity.rs`, `collocation.rs`,
`surrogate.rs`, `prediction.rs`, `math/portable.rs` beyond `describe_body`,
`implicit_regimes.rs` proof transport, `implicit_reconstruction.rs` and
`composite_reconstruction.rs`. The KINSOL inner solver was read only at the session-retention
level. No build, test or probe was run. Every quantitative consequence below is unmeasured.
The existing benchmarks top out at 32 PC-SAFT heater blocks (192 variables, `warm-large-1`).
They do not exercise the "many small blocks" or 10⁴-instance premises, so materiality at those
scales is reasoned from structure.

## 2. Complete operations examined

**(E1) Evaluator lifecycle (cold structure → warm re-solve).**
1. Body admission runs `typed_math::Request::admit` inside a fresh Salsa workspace. The result
   is retained in `ModelingCache::Body(SemanticBodyHash)` (owner A/compiler).
2. `CasePlan::prepare` runs symbolic support analysis (`execution.rs::analyze` takes symbolic
   first derivatives for second-order support) and fixes the faer Jacobian and Hessian
   patterns. It also builds one `LocalDemand` for each distinct (body, outputs, coordinates)
   per demand group (Objective, Constraints, All).
3. `artifact_requests` keys each demand by body key, environment, order, outputs, coordinates
   and profile. The key does not include revision attribution, so value edits and unrelated
   revisions reuse programs.
4. `MathService::artifact` does a single-flight `DefaultCache` lookup. On a miss it runs
   `job_retained` on a `pse-math` thread. There `ArtifactRequest::build` calls
   `PreparedSupport::compile`, then `compile_scope`, which loops over Value, First and Second.
   Each pass calls `compile_stages`, and `library::bounded_evaluator` then calls
   `exact_evaluator` (Horner/CPE optimization), `vectorize(Dualizer)` and `map_coeff`.
5. `MathService::assemble` assembles once per `Preparation` (`OnceLock`), shared by rebinds.
6. Per attempt, `MathService::worker` builds `attempt_providers` and calls
   `CaseAssembly::worker_scoped`, which deep-clones every program and order and both assembly
   matrices on the session thread.

**(E2) Per callback (Ipopt, KINSOL, Diffsol/IDAS).**
1. `AlgebraicOracle::trial` (or `DynamicWorker::assign`) inserts each coordinate into
   `CaseValues.scalars: BTreeMap<SemanticId, f64>`.
2. `CaseWorker::evaluate` loops over instances:
   - `values_selected` builds a duplicate-check `BTreeSet`, does map lookups and affine
     conversions, and builds a `Vec`;
   - it builds a `bits` `Vec` and compares it with the group cache;
   - `Worker::evaluate_seeded` NaN-fills the whole frame, seeds input jets, and runs
     `evaluate_stages` (Domain and Applicability stages, Blocks through the Symbolica
     interpreter, Provider stages);
   - it then allocates an `Evaluation` with dense output × n × n Hessians.
3. Assembly: `constraints` does a `rows: BTreeMap<SemanticId, GlobalRow>` lookup per
   contribution, and `hessian` does one per term. Every term also binary-searches its group.
   The faer `AssemblyMatrix` refill then copies into the solver buffer. Normalization in
   `tnlp.rs` belongs to C.

**(E3) Implicit property providers inside a body evaluation.**
- `Nested::evaluate` runs the selection anchor, `ConfigurationWorker::resolve`, a
  `numerical_product_key` BLAKE3 hash and an inner solve on a memo miss. The inner solve is
  `CubicRoots` (exact rational isolation), KINSOL (retained thread-local sessions) or Affine.
- It then runs `Problem::verify`, `Problem::derivatives` (which verifies again, evaluates the
  residual jet, and does one faer LU with multiple right-hand sides) and
  `SelectionWorker::verify`.
- Live state: `Problem` holds `Mutex<Worker>`, `Mutex<providers>` and
  `Mutex<FactorWorkspace>` per provider worker (per attempt). `CubicRoots` holds one
  `Mutex<ExpressionEvaluator>` that all attempts share through the factory.

**(E4) Per prepared solve and study point.**
- `PreparedCase::rebind` rebuilds `presolve_domain_facts`, a whole-model `factorable_program`
  with values substituted, whenever a consumed value changed.
- `MathService::prepare_solve` then runs `numerics::resolve`, `discover_class`
  (`prepare_class`, a symbolic affine/quadratic proof, when Auto selection and the class is
  unassessed), `Normalization::from_policy` and `Tolerances::from_policy`.
- The original PS-10 evaluation runs once per candidate with a fresh worker.

## 3. Candidate findings

### B-01 — Inner-loop value transport and assembly go through semantic-identity maps

**Cause.** Several steps run on every callback:

- The solver point is written into a `BTreeMap<SemanticId, f64>` (16-byte keys) by
  `pse-backend-native/src/assembled.rs::AlgebraicOracle::trial`. Dynamics does the same in
  `workflow/dynamics.rs::DynamicWorker::assign`.
- Every instance then re-gathers its inputs by map lookup in
  `pse-math/src/binding.rs::InstanceBinding::values_selected`. That function first rebuilds a
  `BTreeSet` of the compiled body's immutable `input_formals` to re-prove they have no
  duplicates.
- `pse-math/src/assembly.rs::CaseWorker::evaluate` allocates an input `Vec` and a `bits` `Vec`
  per instance.
- Assembly resolves rows by semantic id per contribution (`CaseWorker::constraints`:
  `self.assembly.rows[&id]`) and per Hessian term (`CaseWorker::hessian`:
  `multipliers[self.assembly.rows[&id].get()]`). Every Jacobian and Hessian term also runs a
  `group.outputs.binary_search` (`CaseWorker::result`).

All of these mappings are fixed when `CasePlan` and `CaseAssembly` are built.

**Consequence.** Per callback, identity work grows as O((n + Σ instance inputs) · log N + nnz ·
log R), separately from the arithmetic. With many small bodies (balances, splitters, linear
specs) this bookkeeping plausibly exceeds the interpreted arithmetic itself. It recurs on every
residual, Jacobian and Hessian call. It also recurs on every Diffsol/IDAS right-hand-side call,
which runs 10⁴–10⁶ times on long trajectories, and on every point of a sweep.

It runs against DP-10 ("keep wide provenance out of inner loops that need compact IDs"), H15
(operation-shaped representation), H12 (preparation repeated per value) and H19 (re-proving an
immutable no-duplicates property). Workloads: interactive re-solve, studies, dynamics, and
larger sparse structures with many small blocks.

**Simpler conforming alternative.** When a worker is constructed, compile a dense value layout
inside the existing assembly owner:

- a column-indexed `x` buffer;
- a bound parameter and fixed vector, filled once per attempt or rebind;
- per instance, a gather list of (dense index, conversion plan);
- per Jacobian or Hessian term, precomputed (group-cache slot, local row offset, global row
  index).

`trial` then writes a slice, and assembly reads only by index. `CaseValues` stays the boundary
and semantic representation for rebind, PS-10 original evaluation and diagnostics.
`constraint_sources` hydrates `OutputValue` identities on demand. A whole-vector "same x"
check could short-circuit the per-instance gather when consecutive callbacks share x. This
needs no new crate or library.

**Preservation.**
- Repeated sources stay aliases, and the conversions stay exact and admitted.
- A missing scalar is refused at bind time, before any use.
- Nonfinite trials are refused.
- The per-instance bit-exact cache semantics, instance-attributed errors
  (`MathError::Instance`), applicability attribution and the fresh worker used for PS-10
  original evaluation all stay as they are.

**Uncertainty.** The share of callback time is unknown. For a few large PC-SAFT bodies,
arithmetic probably dominates. Probe P1 discriminates.

**Evidence.**
- Diagnosis: Implemented mechanism, source-traced (Interface-checked).
- Consequence magnitude and remedy: Proposed.
- Verification: compare residuals, Jacobians, Hessians and errors bit for bit with alias,
  conversion, permuted-coordinate and missing-value controls, then run P1 before and after.

### B-02 — Guard and applicability stages copy the entire body frame on every evaluation

**Cause.** In `pse-math/src/execution.rs::compile_stages`, each `CompiledStage::Domain` and
`CompiledStage::Applicability` gets a local value frame sized to *every* formal slot of the body:
`frame: vec![f64::NAN; parameters.len()]`.

On every evaluation, `evaluate_stages` copies all slots' value components into it:
`for (slot, value) in local.iter_mut().enumerate() { *value = frame[slot * width]; }`. Only
after that does it evaluate the stage's small nested program. At order ≥ First the source frame
is strided by the Taylor width, so each copied slot touches its own cache line.

**Consequence.** Guard checking costs O(guarded stages × body slots) per evaluation, rather
than O(guard inputs). That is quadratic in body size for property bodies such as PC-SAFT and
cubic EOS, which carry many `log`/`sqrt`/division domain obligations and per-component
applicability claims. It is paid on every callback, every instance and every order. H10
(amplification hidden behind a fixed-size answer), H16 (copy without purpose) and AP-07/G9
apply.

**Simpler conforming alternative.** At compile time, record the read set of each nested
Domain or Applicability program and copy only those slots. Alternatively, lower the nested
program to read component 0 of the main frame directly through compacted slot indices; the
`compact`/`lower_stages` machinery already exists.

**Preservation.**
- Guards keep preceding the algebra they protect (PS-06).
- NaN poisoning of unassigned reads stays.
- Domain refusals keep their exact `ValidityLineage`, and applicability claims keep their
  per-trial refusal.
- Branch-local semantics stay the same.

**Uncertainty.** Real stage and slot populations of the reference property bodies are unknown.
A count from compiled bodies (P1 or a static dump) would settle it.

**Evidence.**
- Diagnosis: Interface-checked, source-traced.
- Magnitude and remedy: Proposed.

### B-03 — Frame, Hessian and evaluation results are materialized at the widest shape on every call

**Cause.** There are three parts:

1. `CompiledBody::worker_scoped` sizes the frame for the *highest* compiled order
   (`slots × width(last layout)`). `Worker::evaluate_seeded` then runs
   `self.frame.fill(f64::NAN)` over that whole frame on every call, even at Value order, which
   uses only the first `slots` entries. Value calls (line-search `eval_f`/`eval_g`, KINSOL
   residuals, PS-10 assessment) therefore fill W_max = 1 + n + n(n+1)/2 times more memory than
   they use.
2. Every call allocates a fresh `Evaluation` (`values`, `jacobian`, an `applicability` `Vec`,
   `hessians`). The Hessians are expanded into a *dense, both-triangle* outputs × n × n array,
   although the Taylor jet already holds the packed upper triangle and `CaseWorker::hessian`
   reads only the `hessian_terms`. The same dense shape is retained per instance and group in
   `GroupCache`.
3. Finiteness is re-scanned at several layers for the same values:
   - each Block output (`evaluate_stages`);
   - each result jet (`evaluate_seeded`);
   - `AssemblyMatrix::add`;
   - `ProviderValues::validate`, which also rebuilds a `BTreeSet` of `request.outputs` per call;
   - `tnlp.rs`.

**Consequence.** Memory traffic and allocator churn per call scale with slots × W_max and with
outputs × n², rather than with demanded work. This matters most for line-search-heavy NLPs and
root solves on bodies with many local coordinates. The live working set per instance also
grows with n² (H17). H15, H16 and H26 apply. For one large body, the Value-order overfill alone
can rival the interpreted Value evaluation; this is unmeasured.

**Simpler conforming alternative.**
- Fill only the `slots × width(order)` prefix used by the requested order. This preserves the
  exact unassigned-slot guarantee.
- Keep reusable `Evaluation` buffers in the `GroupCache`.
- Let the case refill read packed jet components through component indices precomputed in
  `hessian_terms`, and expand dense Hessians only for consumers whose contract needs them.
  Those consumers are public `Evaluation` users and implicit IFT.
- Retain one finiteness check per publication boundary.

**Preservation.** Keep the "nonfinite or unassigned result jet" refusal, the clearing of a
failed cache, the requested-order-only publication, the symmetric Hessian semantics for
existing consumers and the independent native-boundary checks owned by C.

**Evidence.**
- Diagnosis: Interface-checked.
- Remedy: Proposed.
- Verification: bit-identical outputs, plus P1 sample shares in `slice::fill` and the allocator.

### B-04 — Applicability evidence is fully materialized on every evaluation and then discarded

**Cause.** Every Applicability stage evaluation calls
`pse-model/src/applicability.rs::Node::assess`, then `observe`. That builds complete
`Observation` records on each callback:

- `claim.clone()`, including the `owner_lineage` `Vec` and several ids;
- input `name.clone()`, which is a `String`, per input;
- the `permissions` list re-filtered with `covers` and cloned.

The records are appended into `Evaluation.applicability`. Only the last evaluation per group is
ever read, by `CaseWorker::applicability_observations` ("from the exact last demanded
programs"). `unknown_allowed`/`extrapolation_allowed` derive only from static claim and
permission data. Per trial, only the region classification (`predicates`) and the input values
change.

**Consequence.** Rich scientific provenance is built and dropped on every residual, Jacobian
and Hessian call of every instance that carries claims. That includes the PC-SAFT parameter
families exercised by the benchmarks (`permission selected_unknown_fit …`). This is the
inner-loop wide-provenance pattern DP-10 names, together with H26 (auxiliary work out of
proportion). Studies and dynamics multiply it.

**Simpler conforming alternative.**
- Precompute each node's permission flags and admitted-outcome table when the stage is
  compiled.
- In the loop, compute the outcome code and refuse immediately when the trial is not admitted.
- Keep a compact (node, outcome, input values) record per trial.
- Hydrate full `Observation`s only for the final accepted point, or for a refusal diagnostic.

**Preservation.**
- Refusal happens at the exact offending trial, with complete attribution (ADR-0140/0141
  meaning).
- Observations come only from active demanded branches, after rewriting, derived arguments
  and differentiation.
- The result keeps the instance attribution.

**Evidence.**
- Diagnosis: Interface-checked.
- Remedy: Proposed.
- Verification: compare the final observation sets and the refusal diagnostics before and
  after, across union regions, unknown evidence and extrapolation permissions.

### B-05 — The cubic EOS accelerator runs exact rational root isolation on every inner call, behind a cross-attempt mutex

**Cause.** This confirms L1's lead.

- `pse-math/src/implicit_cubic.rs::CubicRoots::solve` runs on every memo miss of a nested
  density solve, which means every new outer point.
- It evaluates the four f64 coefficients, converts each to an exact `Rational` (an f64 becomes
  a large-denominator GMP rational) and builds a `UnivariatePolynomial` over Q. It then calls
  `isolate_real_roots()` and refines *every* real root to absolute width 2⁻⁵⁴ in exact
  arithmetic.
- It verifies each refined root against the original residual (`problem.verify`, up to three
  residual evaluations). After that, `Nested::evaluate` verifies again, and
  `Problem::derivatives` verifies once more (see B-06).
- The coefficient evaluator sits in `Mutex<ExpressionEvaluator<f64>>` inside one
  `Arc<CubicRoots>`. That `Arc` is admitted once by `implicit_accelerators.rs::Cubic` and
  shared through `Factory`/`Nested.solver` across *all* attempts that use the prepared view.

**Consequence.**
- Peng–Robinson density, used for both phases and every stream, sits in the property inner
  loop of flash, recycle and sweep workloads.
- Certified bignum isolation and refinement per call is far heavier than the f64 work needed.
  The certification also applies to the *rounded* f64 coefficients, not to the original model;
  original-residual verification remains the actual acceptance.
- All in-process study lanes and concurrent cases serialize on one mutex. This shared mutable
  evaluator scratch departs from the stated attempt-owned evaluator contract (§14.3; DP-19).
- H9 (algorithm not chosen for the workload shape), H22 (shared ownership ordering independent
  work), H4 and AP-07.

**Simpler conforming alternative.**
- Isolate the roots in f64. Critical points of the cubic come from a quadratic, then the
  algorithm uses sign evaluations on the monotone segments intersected with the declared
  branch interval, and safeguarded Newton or bisection inside each bracket.
- Fall back to the existing exact isolation only when a sign test is ambiguous within a
  rigorous rounding bound (near-double roots).
- Isolate only within the declared branch interval rather than refining all roots.
- Give each attempt its own coefficient evaluator, for example by cloning it in `create_scoped`
  or holding it inside `Nested`. This removes the cross-attempt mutex.

**Preservation.** Keep "exactly one original-residual-verified root in the declared branch
interval", explicit refusal on ambiguity or degeneracy, the time-limit and cancellation checks,
the derivative contract and deterministic results.

**Uncertainty.** The per-call cost and the contention are unmeasured; see P3. The behavior of
ambiguous cases near the critical temperature needs explicit controls.

**Evidence.**
- Diagnosis: Interface-checked.
- Remedy: Proposed.

### B-06 — Implicit providers repeat configuration and verification for unchanged inputs

**Cause.** `pse-math/src/implicit.rs::Nested::evaluate` memoizes the solve and derivative
products by `numerical_product_key`. On a memo hit it still repeats several steps:

- `selection.anchor`, which is an evaluator call;
- `configuration.resolve`;
- re-hashing the key with BLAKE3 over all inputs and options;
- `problem.verify`, which is a residual evaluation;
- `selection.verify`, which is another evaluator call;
- copying `ProviderValues`.

On a miss, `Problem::derivatives` starts with `self.verify(...)` immediately after
`Nested::evaluate` has already verified the same (inputs, point, options). It then evaluates
the residual jet, whose values already contain the residual.

A body that uses a provider value and its physical partials compiles one `Stage::Provider` per
`partial` (`execution.rs::compile_stages`). So each outer evaluation calls the same provider
several times at identical inputs, and each call pays the hit-path cost.

**Consequence.** Several redundant residual evaluations and hashes occur per instance per
callback in implicit property bodies. Each one re-establishes a conclusion whose premises
(identical inputs, options and solver identity) have not changed. H19, H20 and H14 apply.

**Alternative.** Treat a key hit as already verified for that key, because the product was
verified when created. Derive `derivatives`' residual check from the jet values. Evaluate a
provider once per distinct (spec, inputs) per body evaluation and fan the partials out to its
stages, which is a compile-time grouping.

**Preservation.**
- Independent original-residual verification of each *new* point stays.
- Selection proofs stay.
- Deadline and cancellation checkpoints stay.
- The memo key stays complete: inputs, options, solver and order.

**Evidence.**
- Diagnosis: Interface-checked.
- Remedy: Proposed.

### B-07 — Artifact construction re-optimizes the same scalar program for every order, and builds unused orders

**Cause.** `execution.rs::compile_scope` calls `compile_stages` once each for Value, First and
Second. For every Block, `library/admission.rs::bounded_evaluator` calls
`library.rs::exact_evaluator`. That function runs `Atom::evaluator_multiple(...).build()` with
Horner and common-pair optimization over the *same* expressions each time, before the
order-specific `vectorize`. `ProviderLift::compile` is likewise rebuilt per order.

`CasePlan::prepare_with_source_support` gives every demand group the plan's order. When an
instance contributes to both rows and the objective, the separate Constraints and Objective
requests are therefore compiled to Second order. Only `Demand::All` is ever evaluated at Second
(`CaseWorker::hessian`), so those Second-order programs are never used. Each request still
compiles its own Value and First programs over overlapping outputs.

**Consequence.** Cold preparation, structural edits and every new process repeat symbolic
optimization up to three times per block. Workers then clone the programs that were built but
never used (B-08). This is H12 and H14 on the interactive structural-edit path.

**Alternative.**
- Build the exact scalar evaluator once per Block and clone it before each order's
  `vectorize`; `ExpressionEvaluator<Complex<Rational>>` is `Clone`.
- Compile the Constraints and Objective demands only to the order they are evaluated at when
  an All demand exists.

**Preservation.** Per-order admission and allowances (operation counts, Taylor bounds,
`reserve_export`), cancellation, artifact-key completeness (a changed order ceiling changes the
key correctly) and the exact Rational path all stay.

**Uncertainty.** The share of `pse.case.program_optimization` time is unknown; P2 measures it.

**Evidence.**
- Diagnosis: Interface-checked.
- Remedy: Proposed.

### B-08 — Every attempt deep-clones every compiled program at every order

**Cause.** This confirms L1's lead, and I judge it material.

- `CaseAssembly::worker_scoped` calls `CompiledBody::worker_scoped` for every program. That
  executes `programs: self.programs.as_ref().clone()`, cloning all orders. Each clone copies
  Symbolica's `ExpressionEvaluator`: the stack plus `instructions`, which are
  `Vec<(Instr, …)>` with a heap `Vec` per `Add`/`Mul`/`ExternalFun` instruction. The clone
  also copies the stage buffers, provider lifts and both `AssemblyMatrix`es.
- `MathService::worker` does this for every attempt and step. That includes a new session per
  study point (operation map b.3 #15) and each re-solve oracle in `math/solves.rs`.
- Second-order vectorized programs are the largest, because Taylor expansion multiplies the
  instruction counts. They are cloned even for routes that never request Hessians, such as
  roots/KINSOL or a quasi-Newton NLP.

**Consequence.**
- Construction cost per attempt is proportional to the total instruction count, paid as one
  allocation per instruction.
- Under sixteen concurrent cases, sixteen full copies of every instruction stream are live and
  charged (H17, H21).
- A 1,000-point sweep pays the copy 1,000 times.
- H6 (demand mismatch), H12 and H17 apply.

**Alternatives.** These are listed in order of increasing scope:

1. Clone each order's program lazily on first use, so a route that never asks for an order
   never copies it.
2. Reuse a reset `Vec<Worker>` (scratch only) across *sequential* attempts on the same session
   thread or lane. Fresh providers and caches are built per attempt, and the charge is
   transferred.
3. Library-level fixes:
   - an upstream Symbolica API that separates shared instructions from a per-worker stack (not
     present in 3.0.1);
   - JIT `Applet`s, which are excluded today by §14.5.

**Preservation.** No mutable semantic state may cross attempts (DP-19). The evaluator stack and
frame carry no meaning between evaluations, because they are NaN-poisoned or overwritten.
Caches, providers, scope and cancellation stay per attempt. The fresh, independent worker for
PS-10 original evaluation stays. Charges end at actual drain.

**Rule impact.** Alternative 2 needs RC-B1.

**Evidence.**
- Diagnosis: Interface-checked.
- Remedy: Proposed.

### B-09 — Value-independent numerical-policy resolution is recomputed per prepared solve, with a declarations × targets scan (PE01 residual)

**Cause.** `math/solves.rs::MathService::prepare_solve` calls
`pse_math::numerics::resolve(quantities, preconditions, targets, declarations, profile)` on
every preparation. That includes every study point and every value edit. None of its inputs are
case values.

Inside `resolve`, the validation loop runs `targets.iter().any(...)` for each declaration even
though `target_access` (an index) was built a few lines earlier. The per-target loop then runs
`all.iter().filter(...)` over all declarations. Per target, it also does registry unit
conversions and clones provenance strings. PE01's indexed projection fixed `from_policy`, not
this step.

**Consequence.** The work is O(targets × declarations) per solve preparation. With per-variable
authored nominals it is quadratic in model size, and it repeats identically across sweep points
(H9, H12; PS-11).

**Alternative.**
- Index declarations by (kind, id) once, using the `CheckedInventory` already used by
  `TargetAccess`.
- Memoize the `ResolvedNumericalPolicy` with the prepared view, keyed on the policy key, the
  declaration identities, the targets and the physical context. Its `NumericalResolvedV2` key
  already frames these.

**Preservation.** Conflict detection, strict-nominal refusal, engineering scale and rule checks
with actual physical preconditions (F08), provenance and caller order all stay.

**Evidence.**
- Diagnosis: Interface-checked.
- Remedy: Proposed.

### B-10 — Upper-bound reservations charged as immediate demand also trigger retention trimming (parallel F02 residual)

**Cause.** Three reservations are made at full upper bounds:

- `math.rs::MathService::workspace` reserves `policy.workspace_bytes`, 4 GiB by default, for
  every fresh compiler workspace. Several are created per preparation miss (operation map a.3
  #2, #3 and #6).
- `math/artifacts.rs::MathService::artifact` reserves `policy.worker_bytes`, 8 GiB, for every
  artifact build whose body is not flat arithmetic.
  `execution.rs::selected_compilation_allocation_bound` returns `None` whenever
  `!is_flat_arithmetic()`, which covers any guard, branch or provider, and so essentially every
  property body.
- `prepare_class`, rebind and inner-analysis fallbacks use the same `unwrap_or(worker_bytes)`.

When `try_grow` fails, `retention::Pressure::try_grow` and `jobs.rs` admission raise a pressure
demand. That demand trims both math caches and suppresses retention.

**Consequence.** Under sixteen concurrent cold cases, or on a workstation-sized pool (the
budget tests use 32 GiB), phantom maximum reservations queue or refuse construction that would
actually fit. Worse, they evict genuinely useful retained programs and products, which makes
the next preparation colder (H17, H21, DP-20). The capacity owner is D. The math-side cause is
that construction bounds are absent for non-flat bodies.

**Alternative.**
- Extend construction bounds to guarded and provider bodies, by summing per-stage source
  bounds as `compile_stages` already does.
- Reserve the compiler workspace incrementally, or at the admitted input extent.
- Do not let a reservation that exists only as a cap raise a pressure demand until actual
  growth requires it.

**Preservation.** Honest refusal of genuinely oversized work, foreign allowances charged at
their lifetime, cancellation and drain.

**Evidence.**
- Diagnosis: Interface-checked.
- Consequence: Proposed. Pool sizes in deployment were not traced.

## 4. Opportunities and uncertainties that are not findings

- **Value rebinding re-derives symbolic products.** `PreparedCase::rebind` rebuilds the
  whole-model `factorable_program(values)` behind `presolve_domain_facts` whenever a consumed
  fixed value changes. For Auto optimization, `discover_class` then re-runs `shape_facts`
  (value-substituted aggregation, affine proofs and symbolic derivatives) per prepared solve.
  This is structure-derived work repeated per sweep point (PS-11, H12). However, value
  substitution legitimately changes enclosures, and can change class (zero coefficients). The
  necessary-versus-repeated split is unresolved, so P2 should measure `pse.case.value_rebind`
  first.
- **JIT/compiled evaluators.** Symbolica 3.0.1 offers `jit_compile` (symjit, under the
  `native_code_generation` feature, which `default-features = false` disables) with
  `batch_evaluate`. These could remove interpretive dispatch over Taylor-expanded scalar
  instruction streams and batch instances of one body (H8, H15). Determinism (FMA, DP-11),
  trusted-code and W^X ownership, and the ABI string `…no-jit;no-simd` in `artifact_requests`
  make this a measured decision (RC-B2), not a finding.
- **Redundant First-order evaluation.** Ipopt's per-iterate sequence evaluates each constraint
  body at Value, First (`eval_jac_g`) and Second (`eval_h`). Constraints and All share the
  request for row-only instances, but their caches are separate. The First evaluation is
  subsumed by Second. Its share is about 1/n of the Second cost, so it is minor.
- **Forward-mode hyper-dual versus symbolic derivative programs.** This is a choice of
  derivative strategy, not a defect. Block-local active axes and structural Taylor zeros
  already prune the work. There is no evidence that symbolic derivative evaluators would be
  better for the reference bodies.
- **Guards as separate stages.** Domain and Branch barriers sit outside the Symbolica program,
  although `Instr::IfElse` exists in the library. Moving them in would lose typed refusal and
  lineage (PS-06). Keep the barriers, and fix only their frame cost (B-02).
- **Deterministic inner implicit starts.** "No previous trial is used implicitly" is a
  deliberate DP-11/DP-09 choice. A verified chart-scoped warm start could cut inner iterations,
  but it carries a risk of switching roots. This is a design option, not a defect.
- **Data-table parameters lower to body literals.** For example, `segment_sigma(pkg, j)` becomes
  a literal. An edit to property data therefore re-specializes and recompiles the bodies, but
  constant folding speeds up evaluation. Under the target "a value change must not rebuild
  structure", this is a definition edit, not a case-value edit. It is worth stating explicitly
  in PS-11 terms.
- **FeOS as a PC-SAFT provider.** FeOS is only an xtask oracle today. A compiled FeOS/num-dual
  provider would trade authored-model authority and extension locality for speed. The tradeoff
  is unmeasured (P4).
- **Seams to C and D.**
  - `DynamicWorker::evaluate` clones the whole `SparseColMat` into the result and the cache on
    each Jacobian evaluation, and returns another clone on a cache hit.
  - The `tnlp.rs` normalization allocates a `Vec` per callback.
  - `CasePlan::conditional` still filters all `structure.rows()` once per block, which is
    O(B·R) for many scalar blocks. This is a minor F07 residual.

## 5. Strengths and preservation constraints

- Artifact keys (`artifact_requests`) exclude revision attribution. Value edits and unrelated
  revisions therefore reuse compiled programs. Single-flight, epoch-fenced `DefaultCache`
  retention and assembly once per prepared structure (`Preparation::executable`) are sound.
- `Pressure` yields retention under actual exhaustion, which resolves the state F02 lead.
- Unit conversion in the loop is a precompiled affine `CanonicalConversionPlan::apply`, with no
  registry access.
- faer patterns are fixed once, and `AssemblyMatrix` refills by `Addend` without restructuring.
- Implicit IFT derivatives use one retained symbolic LU, a retained `NumericLu` and
  `MemBuffer`, and multiple right-hand sides. KINSOL inner sessions are retained
  thread-locally with an LRU and a budget.
- Block-local active axes and Dualizer structural zero components are implemented, which
  resolves the production review §8 Taylor-zeros lead.
- `CaseWorker` caches by bit-exact inputs and order, clears on failure, and keeps evaluation on
  the requested demand only.
- PS-10 original evaluation uses a fresh worker (`OriginalCase::evaluate`). Its independence is
  a preservation constraint for B-01 and B-08.
- Bounded admission happens before allocation (`evaluator_construction`, `reserve_export`, the
  Taylor expansion bound).
- `DynamicLayout` is shared across workers, which resolves Enhancement review F05.

## 6. Positions on prior leads

| Lead | Position | Evidence |
|---|---|---|
| 10-09 F07 (conditional blocks copy the coordinate universe) | Resolved (minor residual) | `binding.rs::CoordinateUniverse` is shared; `CaseStructure::restricted` and the `row_bindings` index exist; the residual per-block row filter is in `CasePlan::conditional` |
| 10-09 F08 (difference inference lacks facts) | Resolved | `numerics.rs::engineering_error_quantity` takes `preconditions`; `NoInvariantFacts` remains only in receipt replay (`typed_math_portable.rs`) |
| 10-09 F10 (fitting response search) | Resolved (spot-checked; D owns) | No repeated per-observation filtering in `fitting/oracle.rs` or `sparse.rs::Layout` |
| PE01 (quadratic coordinate projection) | Residual | `Normalization`/`Tolerances::from_policy` use `TargetAccess`; `numerics::resolve` still scans (B-09) |
| PA03 (memo hit performs publication) | Moved | `retention.rs::BodyRetention::{get, retain}` are pure; per-body acknowledgment and `describe_body` re-hashing now sit in `settle_preparation` (A/E/H) |
| Enh F05 (dynamic layouts rebuilt) | Resolved | `dynamics.rs::DynamicLayout` is behind an `Arc`, with `assert_independent_layout`; evaluator clones remain (B-08) |
| Production §8 Taylor zeros | Resolved | `compile_stages` derives `zero_components` from `coordinate_support` |
| Production §8 reusable numeric factors | Resolved for implicit IFT; native KKT not examined (C) | `implicit.rs::FactorWorkspace` |
| Parallel F02 (maximum charged as demand) | Residual | B-10 |
| State F02 (retention does not yield) | Resolved | `retention.rs::Pressure::try_grow` and `jobs.rs` pressure demand/retry |
| EI05 | Not examined in depth (H) | Per-call `numerical_product_key` BLAKE3 noted in B-06 |
| RC02/RC03 (durable fast hashing / persisted runtime) | Not selected; no B evidence of need | Cross-process evaluator persistence (Symbolica serde/bincode features not enabled) is unmeasured; P2 |

## 7. Candidate rule impacts

- **RC-B1.** §14.3: "Mutable evaluator and solver state belongs to the attempt"; DP-19 as
  applied in `CaseAssembly::worker_scoped`. The proposed change: allow *scratch-only* evaluator
  storage, which is poisoned or reset and carries no semantic state, to be transferred between
  sequential attempts on one lane, with the charge transferred. If the rule is kept, B-08 is
  limited to lazy per-order cloning.
- **RC-B2 (conditional, only if the JIT opportunity is pursued).** §14.5 says "JIT/SIMD
  evaluators … are not admitted", and the `artifact_requests` ABI string is "no-jit;no-simd".
  The change would admit a qualified JIT profile under DP-11 determinism controls. If the rule
  is kept, the interpreted-evaluator remedies B-02, B-03 and B-08 still stand.
- **None for B-05.** The cross-attempt mutex is non-conformance with the existing
  attempt-ownership rule.

## 8. Proposed light probes

- **P1 — evaluation hot-path profile (B-01 to B-04, B-06).**
  - Command: `scripts/pse-env --native --resource-class light -- perf record -g -F 499 -o
    <scratch>/b-warm-large.data -- just case-measure <scratch>/b-wl --case warm-large-1`,
    then `perf report --no-children --sort symbol`.
  - Discriminating outcome: the sample shares of
    - `InstanceBinding::values_selected`, `alloc::collections::btree::*` and the allocator (B-01);
    - `evaluate_stages` outside `evaluate_program` (B-02);
    - `slice::fill` and `Worker::evaluate_seeded` (B-03);
    - `applicability::Node::observe` and `Claim::clone` (B-04),

    versus `ExpressionEvaluator::evaluate_program`. A combined share of 25% or more confirms
    materiality; under 10% demotes these to minor.
- **P2 — construction and rebind phases (B-07, §4 rebind, RC02).**
  - Command: `just case-measure <scratch>/b-cold --case cold-large-1` and
    `just case-measure <scratch>/b-sweep --case value-sweep-1000-1`, then read the phases JSON.
  - Discriminating outcome: `pse.case.compiled_order_construction` counts and seconds by order
    (three exact builds per block expected), the `pse.case.program_optimization` share of cold
    preparation, and the `pse.case.value_rebind` seconds per point relative to the solve.
- **P3 — cubic accelerator cost and contention (B-05).**
  - Method: a scratch-crate microbenchmark outside the repository, depending on `pse-math` by
    path. Time `CubicRoots::solve` against an f64 bracketed solve over 10⁴ PR coefficient sets,
    on one thread and then on eight threads sharing one `Arc<CubicRoots>`.
  - Discriminating outcome: per-call microseconds, and throughput scaling across threads (flat
    means the mutex binds).
- **P4 (optional) — authored PC-SAFT versus FeOS.** Time one body evaluation of the
  `pcsaft-05` preparation fixture against FeOS 0.10.1 `HyperDual64` residual Helmholtz at the
  same state. This informs only the §4 tradeoff.
