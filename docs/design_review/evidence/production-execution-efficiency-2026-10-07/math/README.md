# Math and native library consumption

Supporting evidence for the [principal review](../../../reviews/design_review_production-execution-efficiency_2026-10-07.md).

Bounded source review, 2026-10-07. Baseline: HEAD
`219338742915fb16c02534c80430bd2ea97bf94a` plus the existing dirty Plan 28 tree.
This evidence supplies library-fit judgments to the coordinating review; it does
not decide the review verdict or change plan status. No production files, tests,
configuration, plans or shared skills were changed. Tests and benchmarks: **not_run**,
as assigned. The existing roughly 284-second flash whole-test observation does not
identify native solve time, derivative time or compiler time.

## Versions and documentation transfer

`Cargo.toml` and `Cargo.lock` select Symbolica/Numerica/Graphica 3.0.1, faer
0.24.4 and Salsa 0.28.4. Symbolica has `integer-gmp` and `float-mpfr` with
default features disabled; faer has `std` and `sparse-linalg`. POUNCE and
pounce-presolve are 0.12.0 from revision
`ff6386944421e8037069e521222ada9f22d457e3`, rather than an interchangeable
crates.io release. Native Ipopt installation identity is resolved by
`crates/pse-ipopt-sys/build.rs` and its runtime owner; this review did not execute
a linked-runtime version query.

The `symbolica-faer-oximo`, `native-solver-libraries` and `salsa` skills were read.
The math skill covers Symbolica 3.0.0, so its probes do not qualify 3.0.1. Salsa
and faer skill versions match the consumer. Context7 resolve/query selected
`/websites/symbolica_io`, `/sarah-quinones/faer-rs`, `/salsa-rs/salsa` and
`/coin-or/ipopt`. Those current documentation results are discovery evidence,
then checked against local exact-release registry/git source. In particular,
the Context7 Symbolica Rust examples still show an older evaluator signature;
the actual 3.0.1 builder and current caller win.

Relevant upstream pages are [Symbolica numerical evaluation](https://symbolica.io/docs/numerical_evaluation),
[Symbolica 3.0.1 source](https://docs.rs/crate/symbolica/3.0.1/source/src/evaluate/),
[faer 0.24.4 source](https://docs.rs/crate/faer/0.24.4/source/src/sparse/),
[Salsa 0.28.4 source](https://docs.rs/crate/salsa/0.28.4/source/src/),
and [Ipopt TNLP interfaces](https://coin-or.github.io/Ipopt/INTERFACES.html).
Exact local library source was read under the Cargo registry's release-named
directories and the POUNCE checkout at `ff63869`; the source locations below are
relative to those library roots and are not repository-owned copies.

## Implemented strengths

**Implemented, source-inspected:** `crates/pse-math/src/library.rs::exact_evaluator`
builds multi-output Symbolica evaluators with bounded Horner/CPE budgets,
optimizer concurrency, cancellation, common-pair cache/distance and Horner
variable limits. Real arithmetic admission occurs before mapping exact
coefficients to f64. This already consumes cross-output common-expression
optimization; introducing another arithmetic interpreter would duplicate the
library's role. Exact-release backing: Symbolica `src/evaluate/optimize.rs` and
the evaluator builder exercised by its `src/evaluate.rs` examples/tests.

**Implemented, source-inspected:** `execution.rs::compile_stages` selects a
block's active derivative coordinates using structural support, and
`library/admission.rs::bounded_evaluator` vectorizes the scalar evaluator through
Numerica's re-exported HyperDual and Symbolica Dualizer. Demand/output selection,
control barriers and directional products already exist. It would be inaccurate
to describe the whole pipeline as evaluating every global variable's derivatives.
Exact-release `symbolica/src/evaluate/dual.rs::vectorize` ends by calling
`remove_common_pairs`, `optimize_stack`, and `fix_labels`. Thus **post-vectorization
stack reuse and CPE are already consumed indirectly in 3.0.1**. The consumer's
final comment saying its stack optimizer is unsupported is stale; an additional
`optimize_stack` call is not an established improvement. The 3.0.0 skill also
documents this public method; its mere presence does not establish a new 3.0.1
feature. The construction envelope's comment still names 3.0.0: reviewed source
applicability is distinct from newly executed boundary qualification.

**Implemented, source-inspected:** immutable `CompiledBody` templates and
attempt-owned `Worker` scratch are separate (`execution.rs::worker_scoped`).
Cloning workers clones library instruction vectors as well as numeric storage:
Symbolica `ExpressionEvaluator` derives Clone over owned stack/instructions.
This is purposeful worker isolation, not proof that copies are free.
`assembly.rs::Attempt::evaluate` already caches exact input bits and derivative
order for each demand/occurrence, clears failed results, and checks execution
scope before/after evaluation. Ignore of `new_x`/`new_lambda` in the custom Ipopt
bridge therefore does **not** prove evaluator recomputation at unchanged points.

**Implemented, source-inspected:** Salsa inputs own physical environment and
modeling configuration, tracked queries own selection/specialization/document
projection, and native compilation remains outside pure queries.
`workspace.rs::inventory` assigns HIGH durability to immutable physical context
and MEDIUM to providers. Modeling queries use LRU and heap callbacks;
`publish_modeling_revision` avoids setting an equal checked catalog, preserving
the existing revision when no input changes. `trim_queries` triggers immediate
LRU eviction, checks reported metadata/heap usage and rebuilds a bounded
generation if necessary. This consumes existing Salsa facilities; neither a
second incremental engine nor persistence is an automatic efficiency improvement.
Salsa's LRU drops memoized values, not all keys/metadata. `heap_size` is accounting,
not eviction, and unreported foreign-library heaps remain outside this report.
Backing: Salsa 0.28.4 `src/function/eviction/lru.rs`, memory-usage tests, and the
matching skill's memory/backdating/durability contracts and original probes.

**Implemented, source-inspected:** implicit differentiation retains
`SymbolicLu` in `pse-math/src/implicit.rs::Problem`, factors current numerical
values, solves multiple parameter right-hand sides together, and checks backward
error. Dynamics `dynamics/linear.rs::FaerLu` similarly retains symbolic analysis
between numerical linearizations and reports factorization failures through the
Diffsol recovery boundary. Native Ipopt retains the same application/TNLP and
calls `ReOptimizeTNLP`; POUNCE separately retains application and factor factories
with compatibility/profile checks, explicit reuse policies and reported counters.
Relevant owners: `ipopt/sequence.rs`, `pse-ipopt-sys/bridge/pse_ipopt_sequence.cpp`,
`pounce.rs`, `pounce/retained.rs`. Existing tests inspected in those files were
not rerun and supply no new speed measurements.

## Narrow opportunities and conditions

| Opportunity | Actual consumed path and library fit | Conditions and limits |
|---|---|---|
| Supply proven zero Taylor input components | `bounded_evaluator` passes `vec![]` to `Dualizer::new`. Symbolica 3.0.1 `dual.rs` accepts zero components per parameter and uses them in `map_instruction::is_zero` to suppress arithmetic. Current `compile_stages` selects the union of active block coordinates, so individual inputs can still be independent of coordinates in that union. | **Proposed.** Derive zeros from existing complete structural support for each actual block input, not numerical zeros at one trial. Confirm tuple ordering against implementation: `(parameter index, component index)` is what the lookup consumes, despite ambiguous constructor prose. Preserve ancestor-closed Taylor shapes, provider composition, branch/control dependencies, raw derivative factorial conventions and finite construction admission. Reduced Taylor width per output is a larger distinct design, not supplied automatically by this API. |
| Retain faer numeric storage and factor scratch | `implicit.rs` and `dynamics/linear.rs` call high-level `Lu::try_new_with_symbolic` for each numeric refresh. Exact faer 0.24.4 `src/sparse/solvers.rs` allocates a new `NumericLu` and `MemBuffer` there. Its lower-level `src/sparse/linalg/lu.rs::SymbolicLu::factorize_numeric_lu` accepts mutable NumericLu plus caller-provided scratch; response actions already consume this lower-level API. | **Proposed.** This can avoid repeated object/scratch allocation, not required numerical refactorization. Bind retention to unchanged sparsity, index width, numerical backend/parallel profile and attempt ownership; preserve pivot/rank/nonfinite/error recovery. Include retained storage in existing admission. No allocation or latency savings were measured. |
| Narrow catalog invalidation only if edit workloads justify it | `Catalog.checked` is one checked-package input; `selected` reads it before specializing a root. Any changed catalog revisits selection even for an unrelated root; Salsa equality/backdating may then prevent dependent execution. | **Conditional proposal.** Finer inputs could reduce selection work but must retain package closure, visibility, global checks, documents and physical interpretation. Source establishes the invalidation boundary, not expensive downstream reruns or a need for a new query graph. Do not replace current bounded generation rebuild merely to keep old keys indefinitely. |

## Native and domain boundaries retained

`presolve.rs::Policy` consumes the pinned library option registry, applies eligible
affine elimination/row/rank/auxiliary passes and explicitly refuses required
unavailable passes. FBBT and bound tightening are disabled where original-bound
multiplier recovery is unavailable. Exact pinned POUNCE source includes FBBT
expression-DAG interfaces and limits, but supplying a tape alone does not repair
original-coordinate bound-dual recovery. Enabling those passes is therefore not
a missed safe efficiency feature. Existing equality/coefficient thresholds and
LICQ diagnostic scope remain explicit. No tolerance tuning is proposed.

Contextual engineering accuracy, selected implicit sheet/root, same-point/source
derivative evidence, domain obligations, original residual/bound checks and
typed physical meanings remain necessary consumers. Fresh original-space response
factors in `square_response/actions.rs` are purposeful independent response work;
retaining symbolic analysis or scratch there requires a matching support/profile
key and must still factor the actual current Jacobian. Retaining a numeric factor
from another point or suppressing final original checks is not authorized by a
native solver's reuse flag.

Numerica's exact types support the existing Taylor path through Symbolica's
re-export; Graphica's graph algorithms do not replace evaluator optimization or
native sparse factorization. No concrete caller requiring a Graphica replacement
was identified in this bounded math scope. JIT/export backends, broader Oximo
adapters and a Pyomo production route were not proposed: current library/caller
contracts here do not establish an end-to-end benefit or equivalent contextual
arithmetic evidence. This is not an exhaustive unused-capability inventory.

## Evidence limits and handoff

Search coverage: current `pse-math` library/admission/jets/execution/assembly/
implicit paths; compiler workspace/modeling selection and retention; native
Ipopt bridge/session, POUNCE session/retention/presolve, dynamics linear solver
and original-space response factor construction. Exact library source checked
the consumed API boundaries above. Broader workflow lifetime, cold/warm build,
storage costs and which production journey retains a native session belong to
the mapper/coordinator; a retained session implementation alone does not establish
its consumption by every journey.

No new tests, numerical-convergence-order tests, timing instruments, probes or
qualification claims. Inspection commands and Context7 calls completed; a few
initial searches used nonexistent candidate module paths and were corrected by
file discovery. Every material negative statement above is scoped to the named
callers and releases. Root owns integration, final verification and verdict.
