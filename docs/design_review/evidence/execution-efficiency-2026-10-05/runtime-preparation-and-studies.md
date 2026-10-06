# Runtime preparation and study execution

Supporting source assessment for the [principal review](../../reviews/design_review_execution-efficiency-and-surrealdb_2026-10-05.md).
The runtime investigator inspected main `6498b013e579ec8039573200c92282eb8715b52e` plus concurrent
numerical changes on 2026-10-05. No product tests or measurements were run. The mechanisms below
are **Interface-checked** candidates for independent judgment, not a second disposition ledger.

## Durable dispatch recomputes the whole study policy for one occurrence

[`Studies::admit_dispatch`](../../../../crates/pse-operations/src/studies.rs) calls `points` to
decode the complete point snapshot in its transaction, reconstructs graph/facts with
`policy_snapshot`, calls `study_policy::transition`, then searches the returned action list for
the requested occurrence. [`transition`](../../../../crates/pse-operations/src/study_policy.rs)
admits the graph, reconstructs all-point maps and derives actions for every point. The worker's
[`run_study_point` path](../../../../crates/pse-runtime/src/workflow/worker.rs) invokes admission
for individual dispatched points.

For N individually dispatched occurrences, this repeats whole-study work at least N times;
there is an N-squared contribution just from repeatedly loading/processing N point rows, before
graph validation, receipt and recovery work. Map/order operations can add factors; this is not
an exact timing complexity claim for the complete solver. Dependencies and current state are
real policy inputs, and serialization/fencing cannot simply be removed.

**Proposed alternatives:** admit immutable graph structure once and evaluate a requested point
against its required dependencies plus revision-fenced current study facts; or derive and admit
a set of ready independent actions in one coherent transition. Global cancellation, predecessor
failure, warm-start lineage, claimed-point identity and changed state must retain their current
semantics. A database graph traversal helps only if the operation stops constructing all actions
for each individual dispatch. Moving this algorithm unchanged to SurrealDB retains the waste.

**Verification:** compare targeted/batched policy conclusions with the existing complete policy
on small DAGs, cancellation, failures and dependency changes; exercise competing claims and
stale fences. Compare matched in-process/durable studies over growing point counts, keeping
science and final outcomes identical. No new such run was performed here.

## Narrow symbolic demand still prepares a dense formal slot universe

[`PreparedBody::support_with_allowance`](../../../../crates/pse-math/src/execution.rs) constructs
formal parameters, a symbol map and `Facts::Dense` over `self.slots` for derivative preparation
before analyzing selected outputs and coordinates. `compile_scope` also sets up full-body slot
structures and per-coordinate reachability. Output/stage pruning is present: this is not
evidence that all body arithmetic or all derivatives are compiled for every request.

The exposed distinction is dense setup proportional to declared body slots versus the smaller
dependency closure potentially sufficient for a narrow request. The preparation controls in
[`.config/preparation-cases.json`](../../../../.config/preparation-cases.json) include 65,536 body
slots and up to 1,048,576 occurrences. Those declared limits are workload premises, not measured
successful capacity. A sparse request can still require many slots; necessity follows dependency
and guard/provider semantics, not the number of returned outputs.

**Proposed alternative:** build a compact library-local coordinate universe for the demanded
dependency closure, with explicit semantic-to-local mapping and zero derivatives represented
according to their contract. Retain guards, selected branches, provider dependencies and public
coordinate meaning. Establish whether any actual library/evaluator consumer requires the full
dense signature before selecting the remedy. Compare with the current dense path on both sparse
and genuinely dense requests; compaction may not help the latter.

The compiler artifact identity explicitly names `interpreted-f64;numerica-jets;real-algebra;no-jit;no-simd`.
No JIT compilation overhead is established for this path. Investigating other library evaluator
realizations is eligible, but no claim of a faster JIT or SIMD target follows from their names.

## Complete ephemeral preparation before scheduling: a tradeoff to resolve

[`ModelingPackage::study_inner`](../../../../crates/pse-runtime/src/workflow/study_execution.rs)
awaits `prepare_bound_operation` for every point before starting scheduling. It retains prepared
items across the complete study. Fresh independent operations are subsequently grouped into
`staged.batch`; that grouping does not batch their earlier preparation. This can delay the first
result and prepare points that become unusable after a predecessor fails.

There is important contrary evidence: complete admission before effects is a real promise;
[`views.rs`](../../../../crates/pse-runtime/src/workflow/modeling/views.rs) and
[`math/retention.rs`](../../../../crates/pse-runtime/src/math/retention.rs) retain shared structure,
and value rebinding in [`math/modeling.rs`](../../../../crates/pse-runtime/src/math/modeling.rs)
reuses compatible products. Per-point calls do not prove repeated symbolic compilation.

Separate pure whole-study admission from expensive attempt preparation when supported by the
contract, or bulk-bind compatible points. Do not publish effects before an admission failure that
the API promises to reject atomically. This remains a conditional improvement until actual
prepared product identity, retained extent and admission semantics settle its cost/benefit.

## Workspace rotation under request diversity: a conditional reuse cliff

[`CompilerWorkspace::modeling_request`](../../../../crates/pse-compiler/src/workspace.rs) keeps
distinct root/instance/binding/limit requests in a vector, searches by equality, and rebuilds the
workspace when aggregate request storage exceeds its allowance. `rebuild` creates a fresh Salsa
database and republishes the current model revision. `trim_queries` first applies query LRU
limits, then also rebuilds if retained entries/bytes remain excessive.

That is lawful conservative reclamation, with a potential loss of unrelated warm memo values
under high request diversity. The inspected
[`reuse_tests`](../../../../crates/pse-compiler/src/workspace/modeling/reuse_tests.rs) cover A/B/A
request reuse and span-only edits retaining body mathematics, contrary to a claim of universal
invalidation. No observed production reset rate is available in this review. Assess selective
request retention or compact lookup only when supported selection workloads demonstrate the
cliff; preserve exact dependency tracking and account for foreign heaps outside current counters.

## Scientific and lifecycle strengths

Preparation distinguishes structural products, changing values and attempt-owned numerical
state. Artifact requests include body/output/coordinate/order/profile demand. Runtime caches
distinguish package, body, solver, observation and parametric products. Native adapters retain
compatible KINSOL, Ipopt, POUNCE, HiGHS and PETSc state with explicit compatibility checks.
These inspected mechanisms refute a blanket assertion that every study point reconstructs its
entire solver and model.

Recycles, dynamics and fitting consume the common modeled intent but retain different numerical
requirements. Current Plan 27 accuracy integration changes active numerical paths; it has not
completed the enclosing campaign. Do not classify required root/branch certification, original
equation checking, derivative accuracy or new-state physical validation as waste because it is
expensive. Compare preparation, factorization, iteration and post-solve assessment separately.

## Existing evaluator reuse and eligible compiled/batch alternatives

**Interface-checked follow-up.** The current integration already calls
`Atom::evaluator_multiple` through [`library.rs`](../../../../crates/pse-math/src/library.rs)
for a multi-output `ExpressionEvaluator`. `compile_stages` in
[`execution.rs`](../../../../crates/pse-math/src/execution.rs) retains that evaluator in
`CompiledStage::Block`, and worker execution reuses it. This preserves multi-expression
optimization/common-subexpression opportunities; it is not a scalar tree interpreter recreated
for every output. [`library/admission.rs`](../../../../crates/pse-math/src/library/admission.rs)
also uses `ExpressionEvaluator::vectorize(Dualizer)` for HyperDual derivative/Taylor coefficient
evaluation. That algebraic vectorization is distinct from data-parallel SIMD or native codegen.

The shared `symbolica-faer-oximo` skill's `symbolica.evaluate` and `symbolica.codegen` cards,
task routes and API/source inventory expose conditional alternatives: `jit_compile` to a
`JITCompiledEvaluator`, native compiled evaluators with `BatchEvaluator::evaluate_batch`,
and wide `f64x4` SIMD evaluators. Current official
[Symbolica numerical-evaluation documentation](https://symbolica.io/docs/numerical_evaluation.html),
queried through Context7, describes compiled C++/SIMD and multi-expression optimization.
These are capability leads, not a measured advantage for the simulator.

Version/feature limits matter. The skill capture is 3.0.0; the inspected consumer lock resolves
Symbolica 3.0.1. Its manifest disables default features and enables `integer-gmp,float-mpfr`,
without `native_code_generation`. The investigator checked the consumer's local crate source
for the feature gates as well as the skill inventory. Existing production does not build the
JIT/native exporter merely because those interfaces exist upstream. Cross-version/feature
integration still needs qualification before implementation.

**Proposed comparison:** consider native evaluation for long-lived stable expression blocks
whose solver callbacks repeat often enough to amortize generation/compilation/loading. Consider
batch/SIMD evaluation where independent point lanes actually exist and guards/providers permit
the mapping; a single-point solver callback is not automatically such a batch. Keep the current
retained evaluator as the comparison baseline. Include first-use compilation, retained code,
worker memory, native-call overhead, value/Jacobian/Hessian fidelity, cancellation, domain errors,
provider derivative contracts and portability in the complete operation.

No inspected operation measurement establishes evaluator dispatch as the dominant expense or
selects a faster backend. Compiled evaluation occurs after symbolic support/stage construction,
so it cannot by itself remove the full-slot preparation described above. Missing optional codegen
is not a demonstrated G8 violation; the current route already delegates evaluator mathematics.

## Existing evidence and limits

[Plan 25k K4](../../../plans/25k-integrated-qualification-and-closure.md#k4--measurements-that-distinguish-the-design)
records untimed observations of multi-minute medium structural preparation and a 32-block warm-up
exceeding fifteen minutes before its first observation. It explicitly narrows the planned
measurement campaign; these observations are not a completed timing distribution or attribution
to any candidate above. Existing selectors cover cold/warm, structure/specialization edits,
eight-point in-process/durable studies, derivative demand, retention and worker admission.
At inspection start, the campaign and Plan 27 handoff remained incomplete. Concurrent numerical
and plan edits continued during this review; this note does not certify their final state.

Use those measurement owners and existing counters where they can change a design judgment.
`case-measure` requires its functional prerequisite; diagnostic observation must not be relabeled
qualified performance. The review itself has not executed K4, established a speedup, qualified
large-model capacity or rerun the scientific conformance corpus.
