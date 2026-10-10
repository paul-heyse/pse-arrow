# Execution and results: bounded supporting assessment

This is supporting evidence for the [principal efficiency review](../../reviews/design_review_efficiency-principles-codebase_2026-10-09.md), not an independent
principal review or qualification receipt. The examined baseline is committed HEAD
`4c24721e691187e1a5b28398b29722fbde671da8` (graph compilation implementation work), which
incorporates the earlier `46545b2ad3e2999b4335692ac49af6091438c3fc` plus dirty graph/hash
implementation. Concurrent work is preserved.

The assessment applies Core 3.4, Efficiency Heuristics 1.0, Process Simulator 1.5 and the
pse-arrow binding selected by [the standard](../../design_principles/standard.toml).
The scoped architectural judgment is **Revise for AP-07/G9**. The inspected execution
architecture has strong ownership and scientific-assurance boundaries. The remaining
findings concern physical execution and lifecycle choices; they do not establish a
scientific-integrity failure.

Production execution, result publication/serving and lifecycle composition were inspected.
Compiler/preparation internals and exhaustive columnar assurance remain with the sibling
and principal assessments. Relevant owners are blueprint §18, §19, §20.4 and §21.1 in
[numerical execution](../../../authoritative_design/sections/numerical-execution.md),
[workflows and results](../../../authoritative_design/sections/workflows-and-results.md),
[identity and publication](../../../authoritative_design/sections/identity-and-publication.md)
and [schema and relations](../../../authoritative_design/sections/schema-and-relations.md).

## EX-01: selective transport first materializes the complete result

**Observed cause.** `RunResult::tables()` caches one `encode()` of the complete relation map.
`table(name)` calls that operation before selecting the requested relation, in
[workflow/results.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-runtime/src/workflow/results.rs).
`DurableAttempt::store_tables()` likewise calls `result.tables()` before publishing its
first canonical result block, in
[workflow/durable.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-runtime/src/workflow/durable.rs). Bounded IPC
publication therefore begins after whole-result Arrow construction.

For trajectories, `ModelingTrajectory::tables()` builds and caches the complete map, and
`encode_tables()` expands every sample/output and available sensitivity into generated
rows, in [modeling/trajectory.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-runtime/src/workflow/modeling/trajectory.rs).
The joined native report remains retained. `encode_fit()` reserves `p.bytes * 4` before
building its result relations, including the one-row computation header, in
[fitting/results.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-runtime/src/workflow/fitting/results.rs).
Python's `NativeRunResult::tables()` invokes complete encoding merely to list table names,
in [pse-py/workflow.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-py/src/workflow.rs).

**Trigger and consequence.** A completed run with substantial trajectory, sensitivity or
fitting output followed by canonical publication, a small-table request or table-name
discovery retains the native report and complete Arrow map simultaneously. A small
requested result can refuse because unrelated relations cannot fit. Blueprint §19.2
explicitly acknowledges this whole-map choice. The limitation is honest, but that does
not establish normal-workload feasibility under AP-07/H11.

**Proposed correction.** Use a relation-aware exporter over the immutable completion,
with chunk iteration for canonical publication and streaming consumers. Discover relation
names from the completion/request inventory. A retained single-batch API can materialize
its requested relation without encoding unrelated relations. Existing generated builders,
checked batches, block writers and manifest sealing provide much of this mechanism;
neither another numerical engine nor another persistence substrate is required.

The target must preserve the joined completion as qualification authority, exact row
identities and units, partial-result interpretation, escaped-buffer ownership and final
complete-manifest activation. Interrupted chunk publication must remain incomplete and
recoverable. The existing sticky `RunResult` encoding-error contract needs an explicit
decision: silently substituting per-relation errors would change observable behavior.

This correction removes an unnecessary complete Arrow representation; it does not remove
the native report's inherent retention. Its maturity is **Proposed**, using existing
components rather than a demonstrated implementation. No RSS reduction, latency
improvement or crossover size is measured here.

**Verification needed.** Exercise a large completed trajectory with a header-only request,
relation-name discovery and canonical publication under a budget sufficient for the native
report plus bounded transport but insufficient for the complete Arrow map. Confirm exact
row/manifest equality, retained scientific checks, partial-publication recovery and escaped
array validity. Compare peak live allocation and publication latency on the same workload;
do not lower accuracy, sensitivity demand or output completeness to make it fit.

**Rule impact.** Revise blueprint §19.2's complete-map/sticky-error choices through the
applicable design route. Preserve scientific and publication invariants. H5, H6, H8, H10
and H16 support the correction; limits alone do not resolve it.

## EX-02: fitting repeatedly searches prepared observation/response relationships

**Observed cause.** `FitOracle::evaluate_demand()` loops over experiments and repeatedly
scans the global measurement vector for observations belonging to each experiment, in
[fitting/oracle.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-runtime/src/workflow/fitting/oracle.rs).
`FitProblem` retains that global vector without an observation index per experiment, in
[workflow/fitting.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-runtime/src/workflow/fitting.rs).

Transient forward-response refill is the sharper amplification. For each included
observation, the oracle filters the experiment's complete response-term vector and then
linearly searches its bindings. `Layout::new()` creates a response term for each included
observation/free-binding pair, in
[fitting/sparse.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-runtime/src/workflow/fitting/sparse.rs).

**Trigger and consequence.** For one transient experiment with `m` included observations
and `b` eligible free bindings, matching inspects `m²b` response terms to emit `mb` addends.
This is a source-derived operation count, not a timing estimate. It affects response-mode
evaluation. `response_rank()` switches even a gradient-only fit to response mode and reruns
evaluation at the candidate. Global observation selection separately contributes repeated
experiment-by-measurement scans.

**Proposed correction.** Prepare observation-index lists per experiment and direct response
ranges or indexed entries, including the local binding index. Refill can then follow the
actual observations and sparse contributions. Existing `Mapping` is the natural owner;
there is no need for a second scientific declaration, generic cache or new dependency.

Preserve source order, repeated-observation multiplicity, excluded/missing observations,
shared-parameter addends and sample/output identity. Preserve the forward/adjoint upgrade
allowance and independent final candidate and response-rank checks. The remedy is a mature
within-contract indexing refactor, currently **Proposed**. Native integration may dominate
small cases, so the static amplification does not establish user-visible speed or its
priority relative to EX-01.

**Verification needed.** Compare predictions, sparse contribution values, gradients and
final rank diagnostics for steady/transient mixtures, repeated measurements, excluded
observations and shared parameters. Exercise both direct Responses mode and the final
gradient-to-response upgrade. A size sweep varying observation count independently of
binding count can test the matching-work diagnosis without changing scientific work.

**Rule impact.** No enduring contract change or dependency is inherently necessary. H5, H9,
H12 and H15 support moving stable relationship resolution out of numerical evaluation.
Independent original-space verification remains required by PS-10/PS-12.

## EX-03: retired analysis graphs retain inaccessible derived payloads

**Observed cause.** `Runtime::forget_analysis_results()` describes withdrawal of derived
retention while preserving method/configuration/input lineage, in
[workflow/retention.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-runtime/src/workflow/retention.rs).
Its function declaration deletes source roots and writes a retirement tombstone, in
[surreal_retention.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-codegen/src/codegen/surreal_retention.rs).
`fn::pse_analysis_v1::available` and `begin` thereafter refuse access and re-admission, in
[surreal_analyses.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-codegen/src/codegen/surreal_analyses.rs).

There is no corresponding production analysis-node/edge deletion or analysis reclamation
operation in the inspected generator declarations, operations, runtime, scripts and Python
consumers. Searches covered `canonical_analysis_nodes`/`canonical_analysis_edges` deletion
and analysis reclamation/collection names, excluding generated copies and tests. Run
reclamation explicitly deletes result payload/index records; source reclamation removes
source intervals and blocks. Neither collects analysis graph rows. This bounded search
does not assert the absence of an unknown external administrative deletion procedure.

**Trigger and consequence.** Each analysis admits at most 4,096 nodes and 8,192 edges, as
declared in [canonical_analyses.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-operations/src/canonical_analyses.rs).
Repeated distinct analyses can therefore accumulate inaccessible derived records even
after explicit retirement. A per-analysis bound does not bound lifetime accumulation.
Failed staged publication can leave a partial graph; its documented retirement escape
releases input obligations without collecting those graph records.

This is a lifecycle-efficiency gap, not a claim that the current API promises immediate
physical deletion. Blueprint §20.4 distinguishes durable lineage receipts from reclaimable
payloads and derived indexes. If permanent retired graphs are intentional scientific
history, that policy needs a useful reopening/audit purpose and an explicit retention
model; the current tombstone prevents ordinary analysis access.

**Proposed correction.** Add bounded resumable analysis collection that retains the header,
method/configuration, input lineage, retirement fence and required digest/count receipts,
while deleting derived edges before nodes. Use the analysis guard, fence late staging and
replay, and retain interruption-safe progress/completion state. Historical input lineage
can explain derivation without retaining the source or result payload.

The remedy is **Proposed** and architecturally straightforward, but needs more detailed
lifecycle design than EX-02. No storage-size or reclamation-throughput result is measured.

**Verification needed.** Retire and collect both complete and partially staged analyses;
interrupt and resume every cleanup phase; race retirement against page staging and
activation; confirm ordinary reopening remains fenced, immutable receipts remain and
source/result retention obligations are released only by their owning operations.

**Rule impact.** Clarify or amend blueprint §20.4's analysis lifecycle and implement through
the generator/operation owners. Preserve immutable lineage rather than equating it with
permanent derived graph payload. H10, H14, H16 and lifecycle ownership support this change.

## Strengths that the corrections must retain

- `Jobs::submit_with()` acquires entry admission before spawning work. Finite population,
  stack/foreign-workspace estimates, cancellation and an original deadline are explicit.
  Memory-pressure waiting releases CPU ownership. Cancellation/deadline paths retain leases
  through actual native-thread join and destruction, in
  [math/jobs.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-runtime/src/math/jobs.rs).
- Staged sessions reserve additional native-team stack demand on scope changes and retain
  it through teardown, in [math/staged.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-runtime/src/math/staged.rs).
  Native exclusion waits cooperate with CPU pause/resume. Persistent native state is useful
  reuse rather than automatically a resource leak.
- Study execution uses bounded `FuturesUnordered` lanes, preserves private state for a
  single continuation successor and bounds ready/inflight/retained owners, in
  [study_execution.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-runtime/src/workflow/study_execution.rs).
  Durable worker lanes replenish while other lanes remain active, in
  [worker.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-runtime/src/workflow/worker.rs). Earlier serial-study,
  pre-spawn population and nominal-team-reservation diagnoses are not current facts.
- `attach_nlp()` performs fresh original-space checks rather than promoting callback or
  backend termination into qualification. Physical tolerances use indexed target access,
  in [quality.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-backend-native/src/quality.rs). Typed callback
  classification remains separate from scientific acceptance, in
  [callback.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-backend-native/src/callback.rs).
- Fitting's exact-point reuse distinguishes response demand, preserves upgrade allowances
  and deliberately reruns final response-rank calculation. EX-02 removes relationship
  searches, not this scientific work.
- Connected result readers acquire exact protected manifests, page selected metadata,
  fetch block metadata/payload together, validate membership/order and retain independent
  allocation ownership with escaped arrays, in
  [connected_results.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-runtime/src/workflow/connected_results.rs)
  and [canonical_results.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-operations/src/canonical_results.rs).
  Expiry/reclamation refuses later reads. File export finishes and synchronizes staging
  before final publication. Python's canonical Arrow stream consumes through the reader
  owner, in [inspection/stream.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-py/src/inspection/stream.rs).
- Source/run retention is explicit, checks owning roots/protections and has bounded
  resumable cleanup. EX-03 challenges inaccessible derived payload accumulation, not the
  useful preservation of occurrence and lineage receipts.

## Retracted hypothesis and bounded opportunities

**Retracted scalar-paging hypothesis.** The preliminary suggestion that ordinary scalar
pages necessarily decode one block `ceil(rows/64)` times is not established and must not
transfer into the principal findings. Ordinary scalar keys are unique within the exact
entity/field/partition; solve steps and fitting experiments partition their observations.
The producer check in
[result_projection.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-runtime/src/workflow/result_projection.rs)
narrows the diagnosis. The remaining observation is that a selected scalar read rebuilds
and sorts the block's complete scalar-cell metadata before checking selected cells. Direct
selected-field verification after complete block admission is a bounded opportunity, not
the retracted page-amplification finding.

**Root-owned gather opportunity.** Selected dense results call `FieldCheckedBatch::take_reserved`.
Its ordinary-layout scratch/index multiplicity accounting and decoded-extent reservation
can depend on the complete source even for a small subset, in
[columnar.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-relations/src/columnar.rs). The principal review owns
whether this is a separate finding or an opportunity given bounded blocks and ownership
guarantees. Do not conflate whole-block scalar admission with repeated decoding per page,
and do not repeat the historical quadratic gather forecast as still present.

**Bounded analysis scale.** Result/dependency analysis selects roots after admitting and
constructing the complete bounded graph. A result evidence relation above 4,096 rows
refuses before root selection; the methods reserve 128 MiB, in
[analyses.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-runtime/src/workflow/analyses.rs). This is an honestly
bounded initial method, but a narrow root does not provide a larger-scale route. Whether
that prevents ordinary supported analysis needs an explicit workload expectation.

## Independent judgment and evidence limits

| Foundation | Scoped assessment |
|---|---|
| AP-01–AP-03 | Strong separation/composition among immutable scientific products, private native sessions, durable attempts and transport owners. |
| AP-04–AP-05 | Explicit outcome, provenance, protection and ownership distinctions at inspected crossings; compiler/schema completeness remains with adjacent assessments. |
| AP-06 | Local contracts are inspectable and have focused controls; assembled present-baseline qualification is separate. |
| AP-07/G9 | Revise for EX-01–EX-03; honest refusals and budgets alone do not establish normal-workload feasibility. |
| PS-G1–PS-G3 | No new failing trigger established here. Preserve physical typing, structural admission, original-space verification and typed outcomes. Whole scientific gate acceptance is not established by this static review. |

Coverage included native admission/cancellation/join and staged sessions; callback/quality
boundaries; studies, worker lanes, continuation and recovery crossings; recycle/fitting
execution; complete-result encoding/publication; connected reads, exports, analysis staging
and retirement; and Python result/Arrow boundaries. It does not establish native-library
thread safety for arbitrary external embeddings, complete solver-class correctness,
server isolation under every concurrent schedule or representative end-to-end performance.

Existing Oct5/7/8 reviews and Plan28/30 supplied leads rather than transferable findings.
The historic SHA-heavy observation used a different installed artifact. Focused controls
and old artifact measurements do not qualify this committed baseline. The current plan
owns qualification and disposition status: [current work](../../../plans/README.md).

No new builds, tests, probes or formatters were run for this assessment or its publication;
their status is **not_run**. Proposed verification above is not an executed receipt.
Source-derived operation counts and observed materialization/retention paths support the
findings; no current latency, RSS, throughput or universal best-in-class claim follows.

The findings support keeping library-owned mathematics/native solvers and Rust semantic
authority. They do not establish that moving numerical execution into SurrealDB or
replacing the store corrects these causes. Existing builders, mappings and lifecycle
primitives supply simpler correction paths first.
