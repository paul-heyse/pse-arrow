# Mathematics, compilation and structural analysis

This page covers `pse-math` (Symbolica/Numerica integration), `pse-compiler` (Salsa preparation), `pse-modeling`
(checking and specialization), `pse-quantity`, `pse-structural` and `pse-kernels`. Contracts are owned by
[mathematics and compilation](../../authoritative_design/sections/mathematics-and-compilation.md) and
[§15–§16](../../authoritative_design/sections/numerical-execution.md#section-15). The runtime layer that retains and
binds these products is on [modeling and preparation workflows](modeling-and-preparation-workflows.md). The baseline
is on the [entry page](README.md). Mechanisms below are Interface-checked by static reading; every value claim is
Proposed, and nothing was built, run or measured.

**System picture.** Library use here is the strongest in the workspace.

- **Symbolica / Numerica.** Coefficient extraction, derivatives, `replace_multiple`, certified real-root isolation and
  refinement, univariate polynomials, evaluator optimisation (Horner, CSE, Dualizer) and C++ export. `jets.rs` states
  the code is "never a project differentiation engine", and `SymbolicComposition` lets Symbolica differentiate an
  abstract composition to obtain the second-order chain rule. Collocation nodes come from Symbolica Jacobi roots.
- **Numerical libraries.** faer owns canonical CSC structure, sparse LU, products and SVD; `root1d::toms748` does
  scalar bracketing.
- **Graph and structure libraries.** pounce-presolve does Hopcroft-Karp, Dulmage-Mendelsohn, block-triangular form
  and interval FBBT; petgraph and rustworkx-core do SCC, ordering and cycles; weighted tear selection is a HiGHS MILP
  (`pse-backend-native/src/tears.rs`).
- **Salsa.** Durability tiers, interned keys, per-query `lru` and `heap_size`, `Cancelled::catch` and event hooks.
- **Vendored Symbolica.** `vendor/deltas/symbolica.patch` is small (332 lines over `univariate.rs` and
  `univariate/roots.rs`): bounded square-free and isolation overloads with step callbacks, limited to degree three. It
  retains the unbounded APIs.

Legitimately custom: original-residual verification after every accelerated root, sealed exact-rational evidence,
identity hashing, FMA-free native compilation flags, guarded branch semantics, the provider envelope, and physical
admission (quantity inference, obligations, fidelity, applicability). Opportunities cluster in three places: seams
the supplier does not expose (CBO-021, CBO-106, CBO-101); several local owners of one derived fact (CBO-100, CBO-103,
CBO-104, CBO-105, CBO-109, CBO-110); and a second incremental or lifetime system beside Salsa (CBO-107, CBO-108,
CBO-022). The prediction and retained-size accounting that surrounds all of these is on
[cross-cutting mechanisms](cross-cutting-mechanisms.md#cbo-001).

**Coverage on this page.** `pse-math` was read closely in `library.rs`, `library/admission.rs`, the vendored delta and
its notes, `native_compiled.rs`, `jets.rs`, `construction.rs`, `rational_binary64.rs` and `implicit_cubic.rs`. Read
in part: `execution.rs` (regions, `analyze`, block compile, `direct_product_shape`; not `compile_scope`, `prune` or
the lift tail), `reverse.rs` and `products.rs` (compile halves, not `evaluate_pullbacks`), `assembly.rs` (evaluation
loop; not weighted Hessian products), the implicit layer (`implicit_reconstruction.rs` accuracy and refinement
bodies at outline) and `factorable.rs` (4k lines, outlined and sampled). `typed.rs`, `guarded.rs`, `typed_provider.rs`
and `facts.rs` were skimmed at specific paths. In `pse-modeling`, `pse-compiler` and `pse-quantity`: `granular.rs` and
`workspace.rs` were read to specific sections and `premises.rs` fully; `specialize.rs` `instantiate_body`,
`specialize/{functions,process,operations,transformations}`, `check.rs` `inventory_package` and `finish_package`,
`resolved/{selection,region_composition,fragment}` and `implicit::project` (about 15k lines together) were not read.
No candidate depends on them except that the site count in CBO-109 is a lower bound.

<a id="cbo-106"></a>

## CBO-106 — The exponential barrier compensates for a live supplier defect

- **Operation and consumers.** `BodyBuilder` unary `Exp` (`typed.rs`) always materializes its argument as a stage and
  evaluates `E^slot`. Every body containing an exponential, and the flat routes that substitute the exponent back, are
  consumers.
- **Current mechanism.** `simplify_exp_log` in `vendor/symbolica/src/normalize.rs` is byte-identical in 3.0.0, 3.0.1 and the vendored tree; 3.0.2 was not compared. In the `Mul` branch `out` is shared across factors. After one factor returns true, a later factor can overwrite `out` and still return false, and `to_pow(out, rest)` then uses the wrong base. Canonical order narrows the shape: `cmp_factors` sorts `Add` before `Fun`, so `exp(log(x) * (a + log(y) * log(z)))` visits the `Add` first and `log(x)` then restores the correct base. The overwrite needs two `Add` factors, the first returning true and a later one writing `out` and returning false: for example `(b + log(u)) * (a + log(y) * log(z))`, when the first sum sorts first. Read from source, not run. The barrier protects stage evaluation only. `flat_expressions` (`ProductRoute::Direct`) in `products.rs`, `analyze` flattening, `PreparedBody::expression` and `typed_external::expand` substitute the exponent back with `replace`, which renormalizes (`atom/representation.rs`), so `E^slot` can reach `simplify_exp_log` again. Reachability through these routes, and whether any authored body produces the shape, are not established.
- **Why it matters.** Two points, kept apart. First, a suspected correctness gap from reading, not run: a flat
  arithmetic body takes the Direct route for weighted Hessian and transpose actions, so a numerical product could be
  wrong. No control covers this shape. Second, leverage: the supplier simplification fires only when the exponent
  contains `log`, so the barrier could be conditional, or a one-function fix in the existing delta (a local buffer
  in the `Mul` loop) could remove it and let exponentials join Horner/CSE scope. An upstream report is natural.
- **Patterns.** LOP-X017, LOP-11, LOP-10.
- **Evidence.** `simplify_exp_log` read in full; `normalize.rs` power branch; `typed.rs` Exp arm; the control
  `exponential_of_nested_logarithms_preserves_values_and_derivatives` in `typed_tests.rs`; `direct_product_shape` and
  `flat_expressions`. By trace the control's exponent does not reach the overwrite, and it evaluates through
  `worker.evaluate`, so it checks the staged path only.
- **Footprint.** Inspected as named. Other flatteners suspected, not traced.
- **Preservation constraints.** No silent term loss on any route; the positivity obligation of `log` retained when
  `exp(log x * y)` becomes `x^y`; identity changes for bodies whose form changes.
- **Counterevidence and uncertainty.** The shape is narrow, and no authored body is known to produce it. Original-residual verification may catch a wrong product downstream on some consumers, and the flat route's own checks do not. A targeted control would settle reachability cheaply.
- **Status.** Current mechanism.
- **Research questions.** Does the defect reproduce on the staged and the flat routes with an exponent of this shape?
  Would the supplier accept the one-function fix?

<a id="cbo-021"></a>

## CBO-021 — Caller-side seams around the already vendored Symbolica

- **Operation and consumers.** Run every Symbolica construction under the repository's budgets and cancellation, keep
  evaluators reusable across attempts, and inspect them: evaluator build, `vectorize`, derivative, export and root
  isolation. Consumers are admission, class evidence, the cubic accelerator, native compilation and collocation.
- **Current mechanism.** The seams the supplier lacks are written in callers.
  - **Abort.** `library.rs` binds `AbortBinding`, `DetachAbort` and a thread-local `CURRENT_ABORT` because Symbolica
    3.0.1 stores the builder's abort callback in its evaluator settings with no setter. An abort truncates
    optimisation rather than failing (library assessment, 2026-10-27), so `AbortBinding::finish` refuses a truncated
    build.
  - **Census and reset.** `EvaluatorFacts::inspect` calls `export_instructions()` and walks a full copy at each of the
    exact, Taylor and floating representations. `BlockEvaluator::reset` uses `map_coeff` with a `Cell` index to
    overwrite non-constant stack slots with NaN, and derives `resettable` from the export.
  - **Instruction copies.** `ExpressionEvaluator` holds instructions and stack in one cloneable struct, so N attempt
    workers copy and charge N instruction streams (`BuildAllowance::instructions`). Native code is shared; interpreted
    workers are not.
  - **Cost mirrors.** The prepaid cost formulas are [CBO-001](cross-cutting-mechanisms.md#cbo-001).
  - The `symbolica` hold in `[workspace.metadata.pse.pins]` names build-time abort among the release-specific
    couplings. The pinned version is `=3.0.1`.
- **Why it matters.** With a maintained delta already paid for, a seam may be cheaper in the supplier than in several
  callers. The delta shows a callback seam is workable at small size: `square_free_part_bounded` and
  `isolate_real_root_intervals_square_free_bounded`, with step callbacks admitted through `real_points`. A
  program/stack split alone could cut per-attempt copied bytes under 16-lane use. Symbolica 3.0.2 (released
  2026-10-30) changes every `evaluate/*` file, including `dual.rs` and `export.rs`, with 66 files changed. It carries
  no bounded overloads and keeps `root_cache`, while the delta's own target changed by about 55 lines, so the rebase
  of the delta looks cheap and the evaluator-facing assumptions do not.
- **Patterns.** LOP-X017, LOP-X001, LOP-10, LOP-07, LOP-11.
- **Evidence.** `library.rs` read entirely; `evaluate/evaluator.rs` struct and `try_evaluate`; `BlockEvaluator`;
  `vendor/deltas/symbolica.patch` hunks and `vendor/symbolica-bounded-roots.md`; an archive diff of 3.0.1 to 3.0.2
  `src`, API list and named hunks only.
- **Footprint.** Inspected as above. `cloned_stage_descriptors` and `Worker::clone` outlined only. New 3.0.2 items
  (`InheritedLibraryUnlock::{capture, activate}`, `horner_use_cpe`, `check_zero`) are unassessed.
- **Preservation constraints.** Never publish a truncated evaluator; a retained product never holds the producer's
  abort flag; stale scratch cannot leak across scope; private mutable scratch per worker; deterministic typed refusal
  (`MathError::Limit`, `WorkLimit`); cancellation observed before results are retained.
- **Counterevidence and uncertainty.** Each added seam raises rebase and requalification cost, and a delta kept to a
  narrow purpose has value as such. Upstream may not accept seams. A seam that changes numerics is a scientific change.
  The unreleased `community` branch adds a thread-local resource budget by RSS and wall time, which is neither
  prepaid nor deterministic, so it would not replace the mirrors. `class_evidence` charge-and-checkpoint is a separate,
  cleaner allowance and not a supplier mirror.
- **Status.** Current promise (bounded, cancellable construction). Moving seams into the supplier is Proposed.
- **Research questions.** How large is a program plus caller-owned stack split in `evaluate/evaluator.rs`? Can
  `abort_check` move to a passed parameter? Does `InheritedLibraryUnlock` remove any licence or worker constraint?

<a id="cbo-107"></a>

## CBO-107 — Salsa memoizes around a driver that feeds results back as inputs

- **Operation and consumers.** Incremental modeling check for a live revision (`workspace/modeling/granular.rs`,
  `check/phases.rs::Phases`), consumed by every admission and preparation path.
- **Current mechanism.**
  - `Driver` implements `Phases`. The pure checker pushes each stage's staged product through
    `RefCell<&mut CompilerDatabase>` as a Salsa input, and tracked queries re-expose it: `resolved` clones
    `input.resolution`, and `occurrence` binds an input it was handed.
  - There are 22 compare-then-set setter calls in `granular.rs` and a separate `BTreeMap` of input handles in `State`.
  - Query labels reach through `salsa::plumbing` (`fn_ingredient_`, `ZalsaDatabase`, `AsId`) at four sites in
    `granular.rs`, and `current_revision` at one in `workspace.rs`. Only the query-name half has a public replacement
    (`ingredient_debug_name`).
  - Transient refusal escapes tracked code by `resume_unwind(Cancelled::Local)` and a `Mutex<Option<MathError>>`,
    because a returned error would be memoized. Failure rotates the whole generation.
  - Compilation for one case is single-threaded; `parallel_probe_tests.rs` is a feasibility probe with "no production
    scheduler".
  - The inversion is limited to the checking phases. Downstream `projection`, `admitted` and `semantic_body` are
    ordinary pull queries.
- **Why it matters.** A pull model would derive dependencies instead of relying on hand-chosen staged projections,
  could remove `State` and the setter blocks, simplify each added phase, and might allow parallel checking of
  declarations under the workstation's lanes. See also [CBO-087](modeling-and-preparation-workflows.md#cbo-087).
- **Patterns.** LOP-X019, LOP-10, LOP-08, LOP-07.
- **Evidence.** `granular.rs` through `Driver::inventory`, its query list and `check_modeling`; `workspace.rs` first
  700 lines, `trim_queries`, `body_refused`, `observe_query_event`; `Phases` in `phases.rs`; `check.rs`
  `check_declarations`. (Interface-checked.)
- **Footprint.** Unread: `shared_packages.rs` beyond its interned types.
- **Preservation constraints.** `pse-modeling` stays Salsa-free, or that premise is consciously changed (`Fresh` and
  `Driver` share one checker); source-order signature semantics including `Lookup::Pending` versus `Absent`; no
  memoized transient refusal; exclusive candidate staging; span-only edits must not invalidate type work; the §14.4
  single-writer lane.
- **Counterevidence and uncertainty.** The firewall pattern works and is tested (`granular_tests`, `reuse_tests`).
  Source-order environments may need driver control, and parallelism may yield little on packages of about 5.9k
  lines. Whether pull queries can express rollback of failed candidates is undetermined.
- **Status.** Current mechanism.
- **Research questions.** How do Salsa tracked structs and `par_map` apply here, and how do other Salsa-based
  analyzers model source-order scopes as pull queries? Is there a non-memoized error path upstream?

<a id="cbo-108"></a>

## CBO-108 — Physical dependency tracking on thread-local capture beside Salsa

- **Operation and consumers.** Narrowed reuse of checked and specialized products when the physical registry or
  preconditions change, and capture of receipts for durable recipes.
- **Current mechanism.**
  - `pse-quantity/src/premises.rs` keeps a `thread_local!` stack of `Capture`. Registry accessors call
    `premises::observe(self, Read::..)` through a macro in `registry.rs`; `Read` has about 30 variants including
    absence and population reads. `PhysicalPremises::matches` revalidates by projecting a sparse registry from those
    reads. Capture identity uses raw pointers of the registry and facts.
  - Adjacent ambient collectors have the same shape: `resolved_receipts.rs` `SESSION`, `pse_math::typed_receipts`
    `SESSION` (nested with the former in `typed_math/direct.rs`) and `scientific_selection.rs::Collector`.
  - Salsa sees the whole registry as one input (`Inventory.quantities: Arc<QuantityRegistry>`, `Durability::HIGH`),
    so any registry change looks total and narrowing happens outside it (`executable.rs` around `premises::capture`).
- **Why it matters.** This is a second incremental dependency system with its own read vocabulary and completeness
  discipline. Each new registry accessor must remember to observe, and a missed one is an unsound reuse rather than a
  slow one. A generated or typed observing layer could make a missed read a compile error, and per-entity Salsa
  inputs could give in-workspace narrowing.
- **Patterns.** LOP-X019, LOP-X002, LOP-07, LOP-10, LOP-09.
- **Evidence.** `premises.rs` to `capture`, plus `observing`, `without_recording`, `matches`, `observe`, `covers`;
  `scientific_selection.rs::Collector`; call sites by `rg`; `typed_math/direct.rs` capture nesting. (Interface-checked.)
- **Footprint.** Whether all `registry.rs` accessors are covered was sampled only; the macro sits at one site and
  other accessors are by hand.
- **Preservation constraints.** Premises survive fresh workspaces and restart (§14.4: persisted complete dependencies
  govern reuse across restart, Salsa being an optional local accelerator, and it does not persist); absence and
  population reads tracked; a foreign registry makes a product ineligible; failed checks discard partial closures;
  `pse-quantity` carries no Salsa dependency.
- **Counterevidence and uncertainty.** The persistence need is legitimate, so Salsa cannot simply replace it. Whether
  Salsa exposes a query's dependency edges for export is not known. Physical registries may change rarely, which
  lowers the value of narrowing.
- **Status.** Current mechanism.
- **Research questions.** Does Salsa expose query dependency edges for export? Could a typed registry view make an
  unobserved read unrepresentable?

<a id="cbo-110"></a>

## CBO-110 — Parallel wire hierarchies with hand capture and restore

- **Operation and consumers.** Durable restoration of resolved values, types, contracts, graphs and provider
  specifications, and the public serialized form of preparation bindings. Consumers are portable recipes, replay and
  the project preparation boundary.
- **Current mechanism.**
  - About 4k lines of wire mirrors with `capture` and `restore` pairs: `ValueWire` in `portable.rs`,
    `resolved/{records,support_record,operation_record}.rs`, `pse-quantity/resolved_receipts.rs` (`ContractWire`),
    `pse-kernels/portable.rs` and `typed_math_portable.rs` (`SpecWire`). Payloads are JSON via serde with a
    byte-counting writer preflight. `unit.rs` shows the same shape for conversion plans.
  - In the runtime, `project_preparation_wire.rs` replicates `pse_modeling` structural `Value` shapes as
    `PreparationStructuralValue`, `PreparationFact` and `PreparationFormulation` with hand `From` in both directions
    and explicit serde tags; the module calls it "mechanical".
- **Why it matters.** Each new `Value` variant requires edits in two directions, and a new field in a scientific type
  means several independent edits. These types will keep changing as the physical semantics grow. A derive or serde
  remote definition could carry the mechanical translation. The identity-framing half of the same declaration is
  [CBO-039](authoring-and-identity.md#cbo-039).
- **Patterns.** LOP-02, LOP-X007, LOP-06.
- **Evidence.** Heads of the listed wire files; `project_preparation_wire.rs` first 110 of 270 lines. Claims rest
  partly on structure. (Interface-checked.)
- **Footprint.** Unread: full `capture` and `restore` bodies, and consumers in `pse-runtime/src/math`; the owning
  `Value` for the runtime wire.
- **Preservation constraints.** Stable tags at the public boundary, `deny_unknown_fields`, float bits, schema
  generation with `schemars`; wire is untrusted until qualified (strict restoration, size bound); canonical
  first-use ordinals in framing are real logic.
- **Counterevidence and uncertainty.** Wire and memory forms intentionally differ (arena indices, canonical order,
  validation on restore), and the preparation wire may deliberately decouple tags from internal enums. The main
  consumer of the portable recipes is the unconsumed durable tier ([CBO-099](modeling-and-preparation-workflows.md#cbo-099)),
  which lowers the near-term cost of leaving it. A local derive is real maintenance and may be the actual answer.
- **Status.** Current mechanism.
- **Research questions.** Would a small local derive, or a serde-driven canonical form, serve both wire and frame? Would
  a binary validated format replace JSON for recipes? What else consumes the JSON payload besides restoration?

<a id="cbo-101"></a>

## CBO-101 — C++ compiler driver and code cache beside an in-process JIT

- **Operation and consumers.** Optional scalar native evaluation (`compiled-evaluator`, `Evaluator::CompiledScalar`)
  for arithmetic blocks.
- **Current mechanism.** `native_compiled.rs` and `native_compiled/cache.rs` (about 1,465 lines with tests): C++
  export, packing of eight functions per module, tempdirs, a `prlimit`-wrapped `g++` compile and link under
  `SupervisedChild`, stderr capture, module load, per-worker `load_new_function`, and a disposable on-disk cache with
  corruption checks. The supplier's `ExpressionEvaluator::jit_compile` exists, and the JIT crate it uses is already in
  the lockfile through `native_code_generation`. `product_realization_probe.rs` is a private bounded inquiry that
  compares interpreted, packed scalar, reload and scalar JIT and states "no production realization".
- **Why it matters.** A JIT could remove the child-process lifecycle, file limits, tempdirs, module packing and the
  cache, and compile latency, making native evaluation attractive for short-lived products.
- **Patterns.** LOP-08, LOP-01, LOP-07, LOP-X006.
- **Evidence.** `native_compiled.rs` to the tests, cache outline, probe head and benchmark section, `backend.rs`
  `jit_compile` signatures; supplier JIT settings read in the vendored source. (Interface-checked.)
- **Footprint.** Consumers in `pse-runtime` unread. The probe's recorded result was not found; it prints timing and
  asserts numerical closeness to 1e-11.
- **Preservation constraints.** The fixed floating profile (`-ffp-contract=off`, no fast math, generic x86-64), no FMA
  contraction, values agreeing with the interpreted program under the semantic contract, deadline and cancel during
  preparation, bounded retained code, and artifact identity.
- **Counterevidence and uncertainty.** Three contract gaps were read. The JIT default configuration enables fast math,
  whose main effect is fused multiply-add, though the probe turns it off with SIMD and threads. The default
  configuration reads `SYMJIT_TOML` or `./symjit.toml` and unwraps on parse, so an ambient file can change code
  generation or panic in process. `jit_compile` is synchronous with no abort, deadline or size prediction, so it swaps a
  lifecycle burden for an unbounded construction step; whether that matters depends on unmeasured compile cost. The
  compiled evaluator holds an `Applet` with no shared-code handle and no `Clone` was found, so N workers may each need
  code. Exact `IF` and `Pow` expansion differences against the interpreter are unestablished, and transcendental
  sources were not compared bit for bit.
- **Status.** Current mechanism; a prior private inquiry exists.
- **Research questions.** Does the JIT guarantee no fused operations and a stable math function source, and can it be
  configured without ambient files? If the C++ route stays, could a content-addressed cache replace `cache.rs`?

<a id="cbo-100"></a>

## CBO-100 — Derivative product construction has several routes

- **Operation and consumers.** Produce value, partials, Jacobian actions, weighted Hessians and Hessian actions for a
  guarded body, for solvers and for implicit-function products.
- **Current mechanism.** Route choice among `RegionMode::{Taylor, SelectedFirst, Symbolic}` in
  `RegionConstruction::new`, `ProductRoute::{Direct, Reverse, Regional}` in `PreparedSupport::preflight_product`, and
  `ProviderLift`/`SymbolicComposition` in `jets.rs`. Library primitives are `Dualizer` and `Atom::derivative`; the crate
  composes them. `LocalPartials` and `SymbolicRegion` alias local partials, `flat_expressions` flattens for direct
  weighted products, and `compile_pullbacks` and `evaluate_pullbacks` implement reverse sweeps by stage with adjoint
  and tangent buffers. Each route has its own preflight, operation account and worker layout.
- **Why it matters.** Every primitive is library-owned, but the route portfolio, frame layout and sweep ordering are
  a generic AD system's responsibility. A new product or stage kind must be considered on each route, as the size of
  `product_contract_tests.rs` (1,191 lines) and `derivative_tests.rs` (2,778 lines) suggests. A shared-DAG derivative
  owner might also supply sparse second-order products that the Dualizer's convolution makes expensive.
- **Patterns.** LOP-09, LOP-01, LOP-05.
- **Evidence.** `RegionConstruction::new`, `SymbolicRegion::new`, `SymbolicComposition::new`, `flat_expressions`, the
  first part of `compile_direct_product`, `plan_pullbacks`, `compile_pullbacks` and the block section of
  `compile_stages`. (Interface-checked.)
- **Footprint.** `execution.rs`, `jets.rs`, `products.rs`, `reverse.rs`, `library/admission.rs`. Unread:
  `evaluate_pullbacks` and the remainder of `compile_direct_product`.
- **Preservation constraints.** Original ordered guards retained under cancellation of derivative terms,
  derivative-order-tagged obligations, branch-local order, no finite differences, provider barriers with
  owner-qualified linearization, exact symbolic rationals before floating conversion, and selected-entry semantics
  (§14.3).
- **Counterevidence and uncertainty.** The routes exist for reasons: tests compare actual work and the demand
  contract (ADR-0193) was deliberately layered. Symbolica has only forward dual vectorization on evaluators, so no
  present library route replaces reverse. Provider and guard barriers remain whichever engine owns arithmetic. The
  gain is unproved.
- **Status.** Current mechanism.
- **Research questions.** Which AD systems work on a shared expression DAG with edge-pushing or sparse Hessian
  products and accept external fallible nodes? Would a supplier-owned reverse transform on `Instr`, in the manner of
  `Dualizer`, fit?

<a id="cbo-022"></a>

## CBO-022 — Which prepared-mathematics lifetimes should share one eviction policy

- **Operation and consumers.** Avoid repeating evaluator preparation and native compilation across cases, rebinds and
  sessions, and release retained products when an allocation fails. Consumers are compiler preparation, runtime
  retention and native evaluation.
- **Current mechanism.** Reuse is held at several lifetimes, each with its natural key.
  - **Runtime retention.** `retention::Pressure` coordinates DataFusion `DefaultCache` (artifacts and scalars) and a
    local `WeightedCache`, a single-shard `quick_cache::unsync::Cache` behind a `Mutex` written because equal keys may
    carry different capacity weights. It adds two `RetentionFence`s, a demand counter and a two-step trim that lowers
    cache limits. `Key` has 13 production product kinds. Native lanes, compiler lanes and scratch are trimmed by
    separate steps in `quiescent_phase_owned_construction`.
  - **Read-through.** `artifact_until`, the scalar loader, `plan_frontier_keyed` and `complete_frontier_keyed` each
    repeat lookup, generation capture, re-check inside the producer, retain, and three-arm outcome mapping over five
    `Flights` fields.
  - **Other lifetimes.** `ScalarCache` is construction-scoped and frozen into `ScalarProducts`. The compiler's
    `semantic_body` (`lru=64`) falls through to runtime `body_retention` with an identity and physical re-check on each
    hit. The disk cache for compiled output (`native_compiled/cache.rs`, 496 lines; 64 entries, 64 MiB, scan limit 1024,
    under the temp directory) is ADR-0212 (proposed; implemented and tested 2026-10-20).
- **Why it matters.** One "evictable under pressure" policy is expressed over two caches and several resident
  populations, and the read-through recipe is written per product and can diverge. The sound question is which
  lifetimes should share one policy, not whether to merge the caches.
- **Patterns.** LOP-09, LOP-03, LOP-07, LOP-X005.
- **Evidence.** `retention.rs` `Pressure`, `retention/weighted.rs`, `math.rs` `MathService::new`, `jobs.rs`
  `quiescent_phase_owned_construction`, `artifacts.rs`, `preparation.rs`; `grouped.rs` `semantic_body`; `cache.rs`
  limits and `Cache::local`. (Interface-checked.)
- **Footprint.** `selected_flights` and `basis_flights` not traced. The engine-side host of `DefaultCache` was not read.
- **Preservation constraints.** Lock order; epoch fencing against publication after clear; immutable insertion
  weights; an oversized replacement keeps the old entry; the cache service report contract (`CacheComponent`);
  completion-owned producers surviving waiter stops; generation fencing; a producer deadline separate from the waiter
  deadline; identity follows the immutable dependency closure; compiled bytes carry headers and checksums.
- **Counterevidence and uncertainty.** The lifetimes are lawful: Salsa memos do not outlive the process, runtime
  retention spans workspaces, and atoms are expensive to hash. `WeightedCache` was written around a real semantic gap.
  Unique capacity released differs from entry weight, and a single mechanism may lose reporting. `moka`-style loaders
  cancel differently, so a library is unlikely to replace `Flights`. `Key::EncodedBody` serves only the durable tier
  ([CBO-099](modeling-and-preparation-workflows.md#cbo-099)), so only the disk cache gives cross-process reuse on the
  production path, by inference not traced beyond `native_compiled.rs`.
- **Status.** Current promise.
- **Research questions.** Does a weigher plus eviction listener, or `quick_cache` lifecycle hooks, cover the
  pressure barrier? Would a content-addressed cache crate carry the disk layer?

<a id="cbo-111"></a>

## CBO-111 — Structural analysis caps set by the matching kernel's recursion

- **Operation and consumers.** `CaseIncidence::analyze` for case diagnostics, Dulmage-Mendelsohn and block-triangular
  decomposition, and automatic blocks; implicit reconstruction through `prepare_reduced_supplier`.
- **Current mechanism.** `pse_structural::dense::decompose` is the one decomposition owner and validates the library's
  output. `MATCHING_ROWS` = 100,000 is the "qualified bound for pounce's recursive augmenting path search", and a
  dedicated thread with a 32 MiB stack (`MATCHING_STACK`) is spawned per analysis. `workspace.rs` fixes
  `GraphLimits { nodes: 200_000, edges: 1_000_000 }` at two call sites. A prior correction reports that `CaseLimits`
  also caps a case at 100,000 scalars, which was not read here. Math jobs spawn with `service.policy.stack_bytes`, and
  `MathPolicy::validate` refuses a stack below `MATCHING_STACK`.
- **Why it matters.** Discretized dynamic and multi-stage flowsheet problems could plausibly exceed 100k equations.
  The cap then refuses structural analysis whatever memory is available. An iterative kernel or a larger
  qualification would remove a kernel-imposed limit and the extra thread.
- **Patterns.** LOP-X001, LOP-10, LOP-01.
- **Evidence.** `incidence.rs` (constants, `memory_extent`, `new`, `analyze`), `dense.rs::decompose` head,
  `structural_plan`, `analyze_partition`; `math/jobs.rs`. (Interface-checked.)
- **Footprint.** The pounce-presolve source was not read.
- **Preservation constraints.** Result validation of library output, stable block identities, minimum-`SemanticId`
  tie-breaking, conservative all-branch support, cancellation between stages, and the `PROVENANCE` string in identity.
- **Counterevidence and uncertainty.** No current model's size is known, and the cap may be far from real use.
  Library output validation looks right and should stay.
- **Status.** Current mechanism.
- **Research questions.** Does a pounce version offer iterative matching, or another bipartite matching kernel? What
  bound does the contract actually need?

<a id="cbo-102"></a>

## CBO-102 — Case evaluation runs identical programs one instance at a time

- **Operation and consumers.** `CaseWorker` evaluation for solvers. Many instances share one compiled program
  (`group.request`; identical local demands share a program, §14.1).
- **Current mechanism.** The loop over `structure().instances()` fills inputs, compares bits, tries
  `instance_reuse.project` and `project_compatible`, then calls `workers[group.request].evaluate_trial_into` per
  instance and scatter-accumulates later. `instance_reuse` is a local equal-input reuse layer. The supplier offers
  SIMD export, `vectorize` and JIT `batch_evaluate`; by search no production path uses `batch_evaluate`, only the
  realization probe.
- **Why it matters.** Flowsheets with repeated units and discretized dynamics are large populations of equal
  programs, and a column-oriented evaluation of a group could improve hot-path throughput.
- **Patterns.** LOP-04, LOP-05, LOP-X013.
- **Evidence.** The evaluation loop and observation calls in `assembly.rs`, `Worker` and `CompiledStage` outline.
  (Interface-checked.)
- **Footprint.** `assembly.rs`, `assembly_projection.rs`, `execution.rs`, the native route. Later accumulation paths
  unread.
- **Preservation constraints.** First failing instance attribution (`MathError::Instance`), ordered obligations and
  early exit, provider effects never skipped, bit-level input reuse, no fused arithmetic, deterministic results.
- **Counterevidence and uncertainty.** Bodies with barriers need per-stage batching with divergent branch masks.
  Interpreted batch may not beat scalar. SIMD `exp` and `log` can differ from scalar paths, a reproducibility risk.
  No workload data.
- **Status.** Proposed.
- **Research questions.** Which supplier backends truly batch? Can stages be batched between barriers with an
  all-instances guard check?

<a id="cbo-104"></a>

## CBO-104 — Support analysis is a generic sparsity propagator with repeated symbolic work

- **Operation and consumers.** `PreparedBody::support_with_allowance` via `analyze`; every plan pattern and demand
  depends on it.
- **Current mechanism.** Forward propagation of `BTreeSet<usize>` and `BTreeSet<(usize, usize)>` over stages. On each
  block there is optional flattening by repeated `replace` to a bound of 1 MB or 16,384 operations, symbolic
  derivatives for second-order terms, and a chain-rule union through the DAG; provider outputs get `dense_second`. The
  derived atoms are discarded and compile repeats them: `LocalPartials::new`, `SelectedRegion::new`,
  `compile_direct_product` and `flat_expressions` differentiate or flatten again, and `SymbolicComposition::new` is
  rebuilt per definition though it depends only on arity and local layout. `ScalarCache` memoizes only after the
  symbolic work. The bounded flatten recurs with separate limits in `analyze`, `flat_expressions`/`direct_product_shape`,
  `typed_external::expand` (`1 << 20`), `typed_witness::witness_resolve` and `implicit_selection::independent_call_expression`.
- **Why it matters.** This is preparation time on the admission path, with memory charged twice. The generic part
  (nonlinear interaction sets) is separable from the owned parts (`controls`, `switches`, obligations, branch union).
- **Patterns.** LOP-07, LOP-01, LOP-04.
- **Evidence.** `analyze`, `Fact`, `Facts`, `SupportAllowance`, `SymbolicRegion::new`, `SymbolicComposition::new`.
  (Interface-checked.)
- **Footprint.** `execution.rs`, `jets.rs`, `products.rs`, `class_evidence.rs`.
- **Preservation constraints.** Conservative support including branches (structural zeros must stay zero in layouts),
  full inventories, chargeable work, and the `Atom::num(0)` derivative treatment.
- **Counterevidence and uncertainty.** Flattening is explicitly optional and gives a tighter pattern than propagation,
  and expression reuse across order upgrades would need valid scope. `partials` is already memoized per region
  pointer. A memo for `SymbolicComposition` is cheap, but its benefit is unknown.
- **Status.** Current mechanism.
- **Research questions.** Can a forward pass over `export_instructions` produce sparsity with bitsets, as the
  Dualizer's zero components already consume similar facts? The supplier's alias facility has no derivative.

<a id="cbo-103"></a>

## CBO-103 — Polynomial and affine recognition repeated by consumers

- **Operation and consumers.** Answer "what polynomial is this, in these variables?" for class facts, coefficient
  programs, accelerators, cone recognition and branch identity.
- **Current mechanism.**
  - `class_evidence::prove_affine`: one symbolic derivative per candidate, then zero substitution for the constant.
  - `coefficients::coefficient_projection`: `expand().to_polynomial_in_vars::<u16>` with a degree check per term.
  - `implicit_cubic::CubicRoots::new` and `implicit_selection::graph_equivalence`: each builds powers and calls
    `coefficient_list` with parallel position lookups.
  - `implicit_scalar::BracketedRoot::new`: `is_polynomial`.
  - `curvature::{polynomial, expand, multiply}`: a local `BTreeMap` polynomial of degree at most two over factorable nodes.
  - Branch `identity` in `factorable`, `typed_piecewise` and `typed_witness`: `expand`, `together` and `is_zero`.
- **Why it matters.** Semantic differences (exactness, variable scope, which symbols may hide as polynomial
  variables) are decided locally, and a comment in `BracketedRoot::new` records one hazard. One polynomial view with
  explicit admitted variables could reduce those decisions.
- **Patterns.** LOP-09, LOP-03.
- **Evidence.** Each named function read except the `shape_facts` body. (Interface-checked.)
- **Footprint.** As listed; `class_evidence::shape_facts` is a suspected further member.
- **Preservation constraints.** Exact rationals where ADR-0144 requires, nested dependence caught
  (`get_all_symbols(true)`), refusal rather than misclassification, finite allowance charging, and the §7.5 single
  eligibility rule.
- **Counterevidence and uncertainty.** The curvature polynomial works on a shared DAG without flattening and may
  correctly stay separate. A derivative-based affine proof also proves independence of nested symbols, which
  polynomial conversion does not give automatically. The merge may be only partial.
- **Status.** Current mechanism.
- **Research questions.** Do `MultivariatePolynomial` over `Rational` and the supplier's `is_polynomial` variables give
  an honest shared owner?

<a id="cbo-105"></a>

## CBO-105 — Several interval and constant sweeps over one factorable DAG

- **Operation and consumers.** Enclosures for presolve admission, FBBT, DCP signs, projection ranges and constant
  evaluation.
- **Current mechanism.** There is one project-owned factorable projection (`factorable::FactorableProgram`), with
  further typed projections (`RootIsolationProgram`, `PointArithmeticProgram`). `presolve::tape` re-emits one tape per
  root with `reachable` walks, then `pounce_presolve::fbbt::forward_pass`. `curvature::Pass::interval` is a separate
  forward sweep with pounce `Interval` methods. `curvature::constant_value` and `factorable::power` duplicate
  evaluator semantics. `presolve::tape` emits `Op::ConstInterval` from `Constant::enclosure`, so exact-rational leaves
  no longer shrink enclosures through cancellation.
- **Why it matters.** A program with many rows sharing DAG nodes re-walks and re-emits shared subexpressions for each
  root and obligation. Interval semantics live in more than one sweep, notably `curvature::power_interval` against the
  tape's `PowInt`, `Sqrt` and `Opaque` dispatch, so enclosure strength can differ by consumer. That is the drift risk.
- **Patterns.** LOP-09, LOP-07, LOP-06.
- **Evidence.** `tape`, `admission`, `Pass::interval`, `constant_value`; `factorable.rs` outline only (4,007 lines,
  with the `atom`, `branch` and `identity` parts sampled). (Interface-checked.)
- **Footprint.** `presolve.rs`, `curvature.rs`, `factorable.rs`. Runtime consumers (SCIP, IBEX) belong to the solver
  pages.
- **Preservation constraints.** Outward rounding, exact rational leaf enclosures including the double-rounded
  projection, opaque nodes refuse, and budget refusal retains prior tapes. Each lowering carries distinct semantics
  (fidelity, obligations, tape operators) and exhaustive matches catch a new operator.
- **Counterevidence and uncertainty.** Tapes are the supplier's per-row vocabulary, and row independence helps bound
  refusal. Class evidence stays at atom level because it proves properties of the original aggregated expression.
  DCP needs per-node ranges anyway.
- **Status.** Current mechanism.
- **Research questions.** Can pounce accept a shared multi-root tape? Is one interval library (pounce, Numerica
  `float/interval.rs`, or FILIB through IBEX) a sensible single authority?

<a id="cbo-109"></a>

## CBO-109 — Specialization builds two representations in lockstep

- **Operation and consumers.** `specialize` producing a `SpecializedModel` consumed by `executable.rs::projection`,
  checks and tests.
- **Current mechanism.** `resolved/construction.rs::Rewritten { inspection: Expr, root: NodeIx }`, `RewrittenPredicate`
  and `RewrittenEquation`, where `Construction::pair(Node, ExprKind)` builds each parent in both forms; `inspection`
  appears across `specialize.rs`, `rewrite.rs`, `functions.rs`, `process.rs`, `forms.rs` and `annotation.rs`.
  `reductions.rs` writes synthetic `f_<hash>_inspection_<at>_<ctx>` functions into `model.functions` for inspection
  only. Physical inference runs twice, over `expression.rs::infer` (800 lines over `ExprKind`) and
  `resolved/support.rs::expression_inner` (over `Node`), sharing the `pse_quantity::infer` kernel. Production readers of
  the inspection form found: `conditional.rs`, `specialize.rs` `finish`, frames via `render_expr` in `applicability.rs`
  and `contextual.rs`, and `extent.rs`; many tests read it, and nothing in `pse-runtime`, `pse-backend-native` or
  `executable.rs`.
- **Why it matters.** Each new node kind costs edits in both trees plus typing in both, and retained `Expr` copies
  count against memory. If the inspection form is mostly test-facing, a renderer or lazily derived view from `Node`
  could remove a parallel construction.
- **Patterns.** LOP-X020, LOP-06, LOP-09, LOP-02.
- **Evidence.** `construction.rs` head, `resolved.rs::Node`, `support.rs::expression_inner` start, `infer` Binary
  arm, `reductions.rs`, `Symbol`/`Row`/`Contribution`, targeted `rg`. (Interface-checked.)
- **Footprint.** Absence claim limited to field-name patterns over `pse-compiler`, `pse-runtime` and
  `pse-backend-native`; a destructuring read or `Debug` projection would not match. Unread: `functions.rs` and
  `process.rs` in full, so the affected-site count is a lower bound.
- **Preservation constraints.** §7.7 says synthetic expressions are AST nodes carrying the originating operation, so
  the AST is a stated contract and an amendment needs a decision; occurrence coordinates and span attribution survive.
- **Counterevidence and uncertainty.** The two typing walks are not equivalent: `infer` handles polymorphic schemes
  and source lookups, `expression_inner` selected concrete contracts, and polymorphic checking happens before `Node`
  exists. Independence may be intended as assurance. The stronger half is the lockstep construction.
- **Status.** Current mechanism; part stated contract (§7.7).
- **Research questions.** None library-specific. Could a rendering of `Node` serve the inspection consumers?

<a id="cbo-112"></a>

## CBO-112 — Tabular knowledge hydrated to scalars and admitted by a private engine

- **Operation and consumers.** Package tables and data documents, including Parquet, entering checked packages
  (`data.rs` at 2.4k lines, `document.rs`, `entity.rs`): keys, uniqueness, foreign references, completeness up to
  `CLAIM_LIMIT` = 1,000,000 tuples, and derived columns in dependency order.
- **Current mechanism.** `pse-admission` decodes Arrow into `document::Values` (`Vec<Option<T>>`,
  `Vec<Option<String>>`). `document::admit` builds a per-column `Vec<Value>` by applying `conversion.apply` per cell,
  then `transpose` builds per-row `Vec<Value>` in `BTreeMap`-based rows. Derived columns are evaluated row by row
  with a static evaluator. A separate declarative mechanism (`pse-rules` with DataFusion) expresses other relational
  obligations.
- **Why it matters.** If banks grow to many rows, the path holds several full representations and scalar evaluation.
  Column-oriented admission could reduce memory and time and unify constraint expression. Benefit depends on real bank
  sizes.
- **Patterns.** LOP-06, LOP-05, LOP-04, LOP-09.
- **Evidence.** `data.rs` header and row identity, `document.rs::Values`, `admit` and `column`, `data.rs::reference`,
  `entity.rs` header. (Interface-checked.)
- **Footprint.** Not traced: `data.rs` phase 2 and 3 loops end to end. The decode path in `pse-admission` may belong
  to the data-plane pages. Bank row counts were not measured.
- **Preservation constraints.** The declared storage unit as the single authority, row identity framing over canonical
  keys, test-only taint through lineage, named-row refusals, cycle refusal naming members, `pse-modeling` staying
  Arrow-free, and precharged admission.
- **Counterevidence and uncertainty.** Row-level refusal, unit-aware values and derived functions are semantics
  DataFusion does not supply. Current banks in `packages/` may be small. The purity boundary is deliberate.
- **Status.** Current mechanism.
- **Research questions.** Could typed admission run over Arrow arrays with the unit and refusal logic retained, using
  DataFusion only for key, uniqueness and reference checks?

## Examined and not retained

- **Stage control against supplier conditionals.** The supplier's `IF` is lazy in lowering, but its condition is only
  a nonzero test and no comparison primitive exists, so authored guards would need a nonexportable external function.
  Guards also need typed attribution, ordered early exit and `require_separation`. The stage boundary is the right
  owner; a comparison primitive would reopen only guard-free pure branches.
- **Two rational-to-binary64 conversions.** Evaluator constants use `to_f64` while certified proof constants use
  `rational_binary64::nearest`. The hold in `[workspace.metadata.pse.pins]` retains the non-nearest realization
  deliberately, and a correction would be a domain decision with identity consequences.
- **Symbolic closed-form solving for small implicit blocks.** The supplier's `solve` module was inspected at signature
  level only. Thermodynamic blocks are mostly nonpolynomial, branch selection is the hard part, and no consumer was
  demonstrated.
- **Form-lowering reformulation kernels.** Derived big-M values already come from the library's FBBT enclosure, and
  `classify_root` is a small conservative check over `Node`; sharing it with the math layer would cross the
  deliberate pure boundary for little gain.
- **Static evaluation mixing checked `i64` and `f64`.** `chain_value` uses `powf` and feeds identity through bits, but
  authored constants may rarely hit it, recording realized values makes regeneration portability moot, and the
  same-host contract is honest. Weak.

## Retired opportunities

<a id="cbo-023"></a>

- **CBO-023 — Structural decomposition and dependency closure owned in several places.** Retired 2026-10-30:
  resolved by `3b12321fd`, which made `pse_structural::dense::decompose` the one decomposition owner; the residue is
  the matching kernel's size caps. Successor: [CBO-111](mathematics-and-compilation.md#cbo-111). Last described at
  [revision 93946f572](https://github.com/paul-heyse/pse-arrow/blob/93946f5721a27786fd3c4f281eb6f8036dde405b/docs/library_leverage/codebase/mathematics-and-compilation.md#cbo-023).

<a id="cbo-024"></a>

- **CBO-024 — One expression lowered through several local IRs with a repeated operator vocabulary.** Retired
  2026-10-30: the premise is gone as an opportunity, since one project-owned factorable projection exists and each
  lowering carries distinct semantics; the side observation on tape constants was resolved by `73afd65b0`
  (`presolve::tape` emits `Op::ConstInterval` from `Constant::enclosure`). Successor:
  [CBO-105](mathematics-and-compilation.md#cbo-105). Last described at
  [revision 93946f572](https://github.com/paul-heyse/pse-arrow/blob/93946f5721a27786fd3c4f281eb6f8036dde405b/docs/library_leverage/codebase/mathematics-and-compilation.md#cbo-024).

<a id="cbo-025"></a>

- **CBO-025 — Exact rational LDLᵀ and a cubic-refinement fallback ladder carried locally.** Retired 2026-10-30:
  a value judgment that this lead is no longer worth retaining. exact LDLᵀ is right as is, because `numerica` `Matrix` has no symmetric pivoted LDLᵀ with Gram factors, and the
  refinement ladder was replaced by `74cc0636c` (bounded square-free isolation in the vendored delta, then local exact
  bisection under `BracketEnvelope`). Successor: [CBO-021](mathematics-and-compilation.md#cbo-021). Last described at
  [revision 93946f572](https://github.com/paul-heyse/pse-arrow/blob/93946f5721a27786fd3c4f281eb6f8036dde405b/docs/library_leverage/codebase/mathematics-and-compilation.md#cbo-025).
