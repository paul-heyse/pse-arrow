# Symbolica

Realizations considered: installed `symbolica` 3.0.1 (crates.io, released 2026-09-29; the newest
crates.io release on 2026-10-27); unreleased development on `symbolica-dev/symbolica` `main` at
`bfb481cdc` (2026-10-26, 108 commits after tag `v3.0.1`); unreleased `community` branch at
`f514a7f` (2026-10-24, 14 commits ahead of `main`, merged PRs #129/#130); open PRs #124, #127.
No later crates.io release exists, so every post-3.0.1 capability below is *unreleased/development*.
Research date 2026-10-27. Sources: installed source `~/.cargo/registry/src/*/symbolica-3.0.1/src/
{evaluate/function_map.rs, evaluate/evaluator.rs, evaluate/instruction.rs, evaluate/dual.rs,
evaluate/optimize.rs, evaluate/tree.rs, evaluate/backend.rs, atom.rs, utils.rs, streaming.rs,
poly/univariate.rs, poly/univariate/roots.rs}`; a blob-less clone of upstream in
`<cluster>/src-sym/sym` (`git diff v3.0.1 HEAD`, `git show origin/community:src/utils/resources.rs`);
GitHub PR metadata via `gh api repos/symbolica-dev/symbolica/pulls/{124,127,129,130,138}`;
Context7 `/websites/symbolica_io` (Evaluator `get_instructions` page).

## Types

### symbolica/evaluator-cooperative-abort
- **Need it serves:** stopping an expensive evaluator optimisation (Horner search, common
  subexpression elimination (CSE), common pair elimination (CPE)) when the caller cancels.
- **Recognizable code shapes:** an `abort_check(Some(Box::new(|| flag.load(..))))` on an
  evaluator builder; caller code that inspects its own flag after `build()` because the build
  returned `Ok`; thread-local flags consulted by a closure that outlives the call that installed it.
- **Mechanisms and APIs:** `FunctionMapBuilder`/evaluator builder methods `abort_check(Option<Box<dyn
  AbortCheck>>)`, `abort_level(usize)`, and the same setters on `OptimizationSettings`
  (`AbortCheck: Fn() -> bool + DynClone + Send + Sync`, in `utils.rs`). The callback is polled
  inside optimisation loops: every 10 000 instructions in common-pair search (`evaluator.rs`),
  between Horner/CSE/CPE iterations (`tree.rs`, `optimize.rs`). On `main`, Horner search wraps the
  requested check (`tree.rs` `search_settings.abort_check`), and `EvaluatorComposer::finish(outputs,
  OptimizationSettings)` (unreleased) accepts fresh settings per composition.
- **Obligations discharged:** prompt exit from long optimisation loops.
- **Residual caller obligations:** deciding whether the result is usable; observing cancellation
  outside optimisation (expression construction, `derivative`, `replace`, `coefficient_list`,
  `vectorize` mapping, export, root isolation have no hook); per-call rebinding of the callback.
- **Semantic conditions:** abort is *cooperative truncation, not failure*: when the check fires the
  optimiser sets `abort_level` and stops early, and `build()` can return `Ok` with a valid but less
  optimised evaluator. The settings, including the boxed callback, are stored in the built
  `ExpressionEvaluator` (`settings: OptimizationSettings`, `pub(super)`) and are reused by later
  serial passes on that evaluator; there is no public setter on the evaluator in 3.0.1 or on `main`
  (searched `src/evaluate/**` at both revisions for `settings_mut`, `set_abort`, `fn .*settings`).
  bincode decoding resets `abort_check` to `None` (`function_map.rs` `Decode`), and
  `from_instructions` (unreleased) builds with `OptimizationSettings::default()`.
- **Version / build conditions:** present at 3.0.1. Unchanged in kind on `main`.
- **Limits and negative results:** no abort or limit hook exists for symbolic manipulation or root
  isolation at 3.0.1 or `main` (grep for `abort` across `src/` outside the Python API: only evaluator,
  license, solve and a quotient flag named `abort_on_remainder`). `TermStreamer::max_mem_bytes` is a
  spill threshold, not a limit.
- **Evidence:** Interface-checked (installed 3.0.1 source; `main` diff).

### symbolica/cooperative-resource-budget (unreleased, `community` branch)
- **Need it serves:** sharing an elapsed-time and process-RSS ceiling across nested library work,
  with a typed failure.
- **Recognizable code shapes:** caller-side thread-local budget carriers re-entered on worker
  threads; RSS polling around library calls.
- **Mechanisms and APIs:** `utils::resources::{ResourceLimits { max_process_rss_bytes,
  max_elapsed }, ResourceBudget::{new, with_memory_monitor, run, capture_current, run_captured,
  checkpoint, check_allocation}, checkpoint(), check_allocation(bytes), ResourceLimitError
  {Memory, Elapsed, MonitorUnavailable}}`; `EvaluationError::ResourceLimit` in `evaluate/tree.rs`.
  The budget is ambient (thread-local, nested scopes never relax an outer one) and is propagated
  to workers by explicit capture and re-entry.
- **Obligations discharged:** a typed, nestable, cross-thread time/RSS budget for the operations
  that poll it.
- **Residual caller obligations:** polled only by tracked numerical evaluation, rational
  reconstruction and polylogarithm kernels on that branch (`git grep resources::` over `src/`);
  core symbolic operations and evaluator construction do not poll it.
- **Semantic conditions:** RSS is a *post-hoc process-wide observation*; `check_allocation` is
  documented as "a cooperative headroom check, not an allocator reservation". Not deterministic:
  the outcome depends on concurrent process memory and wall time.
- **Version / build conditions:** unreleased, `community` branch only; not in `main` or any
  release.
- **Limits and negative results:** no prepaid reservation, no per-operation work count.
- **Upstream direction (added 2026-10-27):** upstream also accepts *deterministic* resource
  controls. PR #103 adds checked series operations with structural limits that return a
  structured error, never a partial result, and notes that these are "not limits on total live
  memory … work, or wall time". PR #127 (`nsolve_bracketed`) is numerical, not certified.
  Contributor activity was checked through `gh api`.
- **Evidence:** Interface-checked (branch source, PR #129/#130 bodies).

### symbolica/public-size-observables
- **Need it serves:** sizing library work from facts the library itself reports about its inputs
  and products.
- **Recognizable code shapes:** cost formulas that call the library's own measurement functions
  before an expensive step; arithmetic over exported program lengths.
- **Mechanisms and APIs:** `AtomView::get_byte_size()` (encoded size of an expression);
  `ExpressionEvaluator::{count_operations() -> OperationCount {additions, multiplications,
  inversions, function_calls}, get_input_len(), get_output_len(), export_instructions()}`;
  `ExportedInstructions {input_count, output_count, instructions, temporary_count, constants,
  constant_functions, sub_evaluators}`; `ExpressionView` visitors for node/operand census.
- **Obligations discharged:** exact, version-tracking measurement of existing inputs/products.
- **Residual caller obligations:** any prediction of a not-yet-built product (dual vectorisation
  expansion, export source text, optimiser intermediates) remains a caller model of library
  internals.
- **Semantic conditions:** these are measurements of an existing value; they are deterministic.
- **Version / build conditions:** all present at 3.0.1.
- **Limits and negative results:** no API predicts `vectorize` (dual) output size, export source
  length or optimiser peak memory at 3.0.1 or `main`. The `Dualizer`/`Vectorize` implementation
  (`dual.rs`, 815 changed lines since 3.0.1) and `export.rs` (154 changed lines: register reuse,
  complex constants, non-overlapping buffers, composer) both changed on `main`, so cost models
  derived from 3.0.1 internals require re-derivation at the next release.
- **Evidence:** Interface-checked.

### symbolica/evaluator-instruction-interchange
- **Need it serves:** a portable, flat, slot-addressed program for an expression set, consumable
  by other runtimes or analyses.
- **Recognizable code shapes:** lowering of the same expression into several local tapes/DAGs;
  per-lowering operator tables.
- **Mechanisms and APIs:** `ExpressionEvaluator::export_instructions() -> ExportedInstructions<T>`
  with `Instruction::{Add(o, args, n_real), Mul(o, args, n_real), Pow(o, b, i64, is_real),
  Powf(o, b, e, is_real), Fun(o, (Symbol, tags, args), is_real), Assign, IfElse(cond, label), Goto,
  Label, Join(o, cond, t, f)}` and `Slot::{Param, Const, Temp, Out}`; sub-evaluator bodies for
  non-inlined functions. Unreleased on `main`: `ExpressionEvaluator::from_instructions(&exported)`
  (validated import: forward jumps only, defined-before-read, fallible stack reservation, PR #138)
  and `EvaluatorComposer` (bind evaluators to slots, `finish` prunes dead work and re-runs CSE/CPE).
- **Obligations discharged:** CSE-shared SSA-like program, constant pooling, n-ary arithmetic,
  branching (`IfElse`/`Join`), external-function references by symbol and tags.
- **Residual caller obligations:** mapping `Fun` symbols to a target operator vocabulary;
  decomposing n-ary ops; mapping negative `Pow` to division; any semantic attribute not in the
  program (fidelity, obligation/auxiliary roles, provider identity beyond tags).
- **Semantic conditions:** the stream is *post-optimisation*: Horner rewriting and CSE/CPE change
  expression shape, which changes interval-enclosure tightness in interval/FBBT consumers (still
  sound, not the authored form). Constants carry the evaluator domain `T` (exact
  `Complex<Rational>` before `map_coeff`, or `f64` after). `is_real` flags are optimiser
  assumptions.
- **Version / build conditions:** export present at 3.0.1. On `main` the schema changed:
  `Fun` tags became `Vec<Atom>` (were `Vec<String>`), so exhaustive consumers must be revised at
  the next release. Import and composer are unreleased.
- **Limits and negative results:** no declared operator-vocabulary registry shared with downstream
  tapes; no reverse (interval) propagation.
- **Evidence:** Interface-checked (3.0.1 `instruction.rs`; `main` diff; Context7 Evaluator page).

### symbolica/exact-root-isolation-and-refinement
- **Need it serves:** certified real roots of exact univariate polynomials.
- **Recognizable code shapes:** isolation followed by bisection, with fallback to a general
  root object when refinement misbehaves.
- **Mechanisms and APIs:** `UnivariatePolynomial::{isolate_real_root_intervals() -> Vec<(Rational,
  Rational, usize)>, refine_root_interval((a, b), &tol), isolate_real_roots() -> Vec<(IsolatedRoot,
  usize)>}`, `IsolatedRoot::refined(&tol)`, `enclosure()`.
- **Obligations discharged:** square-free factorisation, multiplicity, isolation, certified
  general refinement through `IsolatedRoot`.
- **Residual caller obligations:** cancellation and work bounds (no hook); choosing between the
  interval refiner and `IsolatedRoot`.
- **Semantic conditions:** `refine_root_interval` (3.0.1) square-frees the polynomial and bisects
  under a *relative* stopping rule `(b - a) / |a + b| > tol`; its precondition (a sign change
  across the open interval, at most one root, no root at an endpoint) is checked only by
  `debug_assert!`. Observed behaviour (Tested; tol = 2⁻⁵⁴, cubic inputs):
  - root at the **right** endpoint: debug build panics on the `debug_assert` when p(a) > 0 (passes
    when p(a) < 0, since an exact zero counts as "not negative"); release build returns a correct
    enclosure in both sign cases;
  - root at the **left** endpoint: debug build panics (both sign cases tried); release build
    **silently returns an interval containing no root** (bisection moves away from the root);
  - interval with `a + b = 0`: panics with `Division by 0` in debug *and* release;
  - interval straddling a root at 0 (asymmetric): does not terminate (killed after 20 s, constant
    RSS) in both profiles;
  - `isolate_real_root_intervals` itself emits intervals whose endpoint is a *different* root
    (for example `(0, 2)` beside `(2, 2)` for roots 1, 2, 3), so endpoint roots are reachable on the
    isolate→refine path; exact roots come back as degenerate intervals, and on the polynomials
    tried no zero-symmetric or zero-straddling interval was emitted.
  `IsolatedRoot::refined` is the certified route that does not share these preconditions.
  Further conditions (added 2026-10-27; Interface-checked from 3.0.1 source unless stated):
  - **Half-line confinement.** `isolate_real_root_intervals` (`IntegerRing` impl with
    `isolate_real_roots_square_free` and `max_real_root_bound`) emits only intervals inside one
    closed half-line: Vincent–Akritas–Strzeboński Möbius endpoints are b/d and a/c with
    a, b ≥ 0 and c, d > 0, the negative pass mirrors f(−x), zero comes back as `(0, 0)`, and the
    disjointness step only bisects. Intervals with a + b = 0 and a ≠ b, or straddling zero, are
    therefore unreachable on the isolate→refine path.
  - **Strict containment is not detected.** `refine_root_interval_until_disjoint`
    (`poly/univariate/roots.rs`) tests overlap only by whether the *earlier* interval's endpoints
    lie in the later one, at entry and in its loop condition, so an earlier interval that strictly
    contains a later one passes as disjoint. Square-free factors arrive in increasing-multiplicity
    order from Yun's algorithm (`poly/factor.rs` `square_free_factorization_0_char`), so
    containment is reachable for polynomials with repeated roots. For
    12x³ − 68x² + 123x − 72 = (3x − 8)(2x − 3)² the intervals are `(0, 4)` and `(1, 2)`, and
    `(0, 4)` holds both 3/2 and 8/3. An interval with no sign change of the square-free part then
    reaches `refine_root_interval`: debug panics on its sign `debug_assert`, and release converges
    to the inner root, so a caller loses the outer root without any error. An endpoint-root screen
    does not catch this.
  - **Unbounded global cache.** `isolate_roots`, `isolate_real_roots` and `IsolatedRoot::refined`
    populate `root_cache()`, a process-global `LazyLock<RootCache>` of
    `RwLock<HashMap<Vec<coeff>, …>>` with no eviction or clear API. Retention is unbounded for a
    caller with an unbounded population of distinct polynomials.
  - **Absolute radius.** `IsolatedRoot::refined` takes an *absolute* radius ("Refine this root to
    an absolute radius"), unlike the relative stop of `refine_root_interval`.
  - **Available primitives.** `UnivariatePolynomial::{gcd, derivative, evaluate}` and exact `Div`
    exist over Q, so the square-free part u = p / gcd(p, p′) is a two-call computation; isolating
    and bisecting on u avoids the multiplicity ordering above.
  - **Probe (Tested).** A sweep over (αx − β)²(γx − δ) with α, γ ∈ 1..=6 and β, δ ∈ −12..=12
    (8,556 distinct root pairs), debug and release, reproduced `(0, 4)` and `(1, 2)` exactly. No
    interval straddled zero; 90 pairs showed containment, all of which panicked in debug, and 20
    lost their simple root in release (both returned points were the double root; no non-root was
    returned). `isolate_real_roots` + `refined(2⁻⁵⁴)` alone was correct on every input. A caller
    that rejects a refined interval whose ends round to different binary64 values falls back to
    `IsolatedRoot` on most inputs, because the relative stop leaves an interval up to about one ulp
    wide; the global cache is then populated on most calls
    ([evidence](../../../design_review/evidence/library-leverage-symbolica-root-containment-2026-10-27/README.md)).
- **Version / build conditions:** 3.0.1; `refine_root_interval` is unchanged on `main`;
  `main` commit `957c2bb` fixes "root isolation stalls" in disk separation (unreleased).
  Open PR #127 adds `solve::nsolve_bracketed` (numerical safeguarded Newton with typed termination
  evidence), which is numerical, not certified.
- **Limits and negative results:** no absolute-tolerance refiner, no endpoint-robust interval
  refiner and no cancellable refiner at 3.0.1 or `main`. Callers of `refine_root_interval` must
  pre-screen endpoint roots (p(a) = 0 or p(b) = 0) and zero-symmetric/zero-straddling intervals
  themselves; build profile changes the failure mode (panic in debug, wrong result in release for
  left-endpoint roots).
- **Evidence:** Interface-checked (source); Tested — standalone probe against Symbolica 3.0.1
  (`integer-gmp`, `float-mpfr`, no default features), debug and release builds, three cubics,
  tolerance 2⁻⁵⁴, one run per case, 2026-10-27 (probe record cited from the applicability section).

## Current applicability

### [CBO-021](../../../library_leverage/codebase/mathematics-and-compilation.md#cbo-021) — Bounded and cancellable Symbolica operations through version-coupled mirrors
- **Types:** symbolica/evaluator-cooperative-abort, symbolica/public-size-observables,
  symbolica/cooperative-resource-budget, symbolica/exact-root-isolation-and-refinement.
- **Realization:** installed 3.0.1; nothing released after it.
- **Conclusion:** unsettled for replacement; fits with conditions as a narrowing. No released
  library budget, abort setter or prediction exists, so the prepaid mirrors and the thread-local
  binding stay. The library does supply public observables, and the upstream `community` branch
  shows a resource-budget design that an upstream contribution could extend.
- **What it would discharge / what remains local:**
  - *Abort binding.* 3.0.1 cannot replace it: the callback is fixed at build time and retained in
    the evaluator. Re-encoding through bincode clears it, but at the cost of a serialisation
    round trip. `EvaluatorComposer::finish` (unreleased) takes per-call settings.
  - *Size predictions.* The predictions (Dualizer expansion, export source bound) remain local.
    Their inputs are already public observables (`count_operations`, `get_byte_size`,
    `export_instructions`). The models track private algorithms, which changed on `main`.
  - *Cancellation outside optimisation.* Symbolic operations and root isolation have no hook, so
    polling between steps remains local.
- **Prerequisites, configuration or extension:** an upstream change. Two shapes would suffice:
  - an evaluator settings setter, or an abort/budget accepted per operation;
  - extending the `community` `resources::checkpoint()` polling to `derivative`, `replace`,
    `vectorize` and export.
- **Semantic gaps against the CBO's preservation constraints:**
  - The upstream budget is RSS and wall-time based. It is not prepaid, not deterministic and not
    charged before allocation. It therefore cannot replace the typed `Limit`/`WorkLimit` refusal;
    at most it adds a backstop.
  - Library abort means truncation, not failure. The local "observed abort means cancelled" rule
    must survive any route.
- **Unsettled premises and what would settle them:**
  - Whether upstream would accept an evaluator-settings setter or operation-level checkpoints.
    The maintainer's issue tracker would settle this.
  - Whether the next release keeps the `Dualizer` cost shape. Re-deriving from the release
    source would settle this; the shape has already changed on `main`.
- **Evidence:** Interface-checked (3.0.1 source; `main`/`community` source); application Proposed.

**Reviewer correction (2026-10-27).** The mirrors are already predict-then-reconcile against the
built product with a typed refusal: `bounded_evaluator` refuses after `vectorize` when the Taylor
expansion exceeds its admitted bound, and the compiled path refuses when `export_cpp_str` exceeds
its bound. Drift therefore fails loudly; the residual exposure is the transient allocation inside
the call. The comparative review retains the mirrors and adds a `symbolica` hold in
`[workspace.metadata.pse.pins]` naming every private coupling (there is none today). Under
upstream contribution the order would be a per-call abort, a deterministic pre-allocation
`vectorize` limit, then checkpoints; PR #103 shows that shape is accepted. See the
[comparative review](../../reviews/mathematics-preparation.md#cbo-021).

### [CBO-024](../../../library_leverage/codebase/mathematics-and-compilation.md#cbo-024) — One expression lowered through several local IRs
- **Types:** symbolica/evaluator-instruction-interchange.
- **Realization:** export at installed 3.0.1; import and composer upgrade-route only
  (unreleased `main`).
- **Conclusion:** fits with conditions only for the arithmetic core feeding interval/FBBT
  consumers; ruled out as the common lowering source for the whole factorable layer.
- **What it would discharge / what remains local:**
  - *Discharged.* The exported stream could replace the hand lowering from atoms into an FBBT tape
    for the arithmetic core: n-ary add/mul, integer and real powers, named functions and branches.
  - *Remains local.* Fidelity (`Exact`/`Relaxed`), the obligation versus auxiliary roles, provider
    calls, selects carrying guarded semantics, and `abs` branch identity are absent from the
    stream and remain in the local DAG.
  - *Vocabulary.* A single `Fun`-symbol → target-op table would still exist, but in one place.
- **Prerequisites, configuration or extension:**
  - Export from the exact `Complex<Rational>` evaluator, before `map_coeff`, to keep exact
    constants.
  - Decide whether Horner/CPE should be disabled for analysis exports so that shape stays close
    to the authored form.
- **Semantic gaps against the CBO's preservation constraints:**
  - Post-optimisation shape changes FBBT enclosure behaviour.
  - Branch identity to `abs` and obligation roles are lost.
  - The tag type changes on `main` (`Vec<Atom>`).
- **Unsettled premises and what would settle them:**
  - Whether the local DAG needs atom-level structure that CSE/Horner destroys. A comparison of
    FBBT tightness on representative bodies with optimisation on and off would settle this.
    This is a probe request, not authorized here.
- **Evidence:** Interface-checked; Proposed.

**Reviewer correction (2026-10-27).** The premise is rejected: there is one project-owned
factorable projection, curvature reads it directly, FBBT uses POUNCE's own format and class
evidence is atom-level by design. The instruction stream loses fidelity, obligations, selects and
`abs` identity, and its optimised shape changes FBBT tightness, so the "arithmetic core" fit is
not pursued. See the [comparative review](../../reviews/mathematics-preparation.md#cbo-024).

### [CBO-025](../../../library_leverage/codebase/mathematics-and-compilation.md#cbo-025) — Cubic-refinement fallback ladder
- **Types:** symbolica/exact-root-isolation-and-refinement.
- **Realization:** installed 3.0.1; `refine_root_interval` unchanged on unreleased `main`.
- **Conclusion:** ruled out — the library cannot carry the ladder at 3.0.1 or `main`; the probe
  shows the ladder's endpoint pre-check is a **correctness guard**, not only a precision
  convenience: without it, debug builds panic and release builds can return a root-free interval
  for a left-endpoint root, and the isolator does emit endpoint-root intervals.
- **What it would discharge / what remains local:** the library discharges isolation and the
  certified general refinement (`IsolatedRoot::refined`, which the ladder already uses as its
  fallback); the endpoint screen, the one-f64-agreement check and cancellation remain local. A
  simpler local alternative — always using `isolate_real_roots` + `IsolatedRoot::refined` and
  dropping `refine_root_interval` — is the only library-side collapse available; its cost and
  determinism were not compared.
- **Prerequisites, configuration or extension:** for a true collapse, an upstream absolute-
  tolerance refiner that handles endpoint roots (or returns a typed error) and accepts an abort
  hook.
- **Semantic gaps against the CBO's preservation constraints:** none for exactness. Residual risk
  outside the current guard: a zero-symmetric interval panics and a zero-straddling interval does
  not terminate; the ladder relies on the isolator never emitting either (not observed on three
  cubics; not established generally).
- **Unsettled premises and what would settle them:** whether `isolate_real_root_intervals` can
  emit an interval with `a + b = 0` or straddling 0 (reading its negative-root mapping, or a probe
  over polynomials with roots near 0); whether "always `IsolatedRoot`" is acceptably cheap
  (measurement, not authorized).
- **Evidence:** Interface-checked (source); Tested — probe
  `docs/design_review/evidence/library-leverage-symbolica-root-refinement-2026-10-27/README.md`
  (conditions stated there); application to the consumer Proposed.

**Reviewer correction (2026-10-27).** The statement that the ladder "relies on the isolator never
emitting" zero-symmetric or zero-straddling intervals is superseded. Half-line confinement settles
that question statically, and the containment probe found no such interval in 8,556 root pairs.
The live hazard is strict containment across square-free factors (see the type): for
(3x − 8)(2x − 3)² the ladder panics in debug, and in release returns `[1.5, 1.5]`, so 8/3 is lost
and the consumer falsely refuses a valid unique root; 20 such losses occurred in the sweep. The
`IsolatedRoot` fallback is the common path (5,651 of 8,556 release pairs), so the unbounded
`RootCache` is populated on most solves in this family, which is the case against "always
`IsolatedRoot`" (retention and absolute-radius semantics, not correctness). The comparative review
recommends, with high priority: isolate on the square-free part u = p / gcd(p, p′) with
`u.isolate_real_root_intervals()`; refine with a local exact bisection (derivative-side rule at
root endpoints, per-step cancellation and time checks, stop when the ends round to the same or
adjacent binary64, typed step cap); delete `refine_root_interval`, the p(a)/p(b) screen and the
`IsolatedRoot` fallback; and advance the identity to `symbolica.cubic-roots.v2` (*Tested*,
[evidence](../../../design_review/evidence/library-leverage-symbolica-root-containment-2026-10-27/README.md)). See the
[comparative review](../../reviews/mathematics-preparation.md#cbo-025).
