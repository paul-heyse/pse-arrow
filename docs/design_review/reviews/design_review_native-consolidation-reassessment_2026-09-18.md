---
title: Native consolidation reassessment and target architecture
date: 2026-09-18
status: done
evidence: Interface-checked
---

# Native consolidation reassessment and target architecture

## 1. Decision and scope

**Decision: Revise the proposed consolidation design; adopt the target direction below.**
The first review identifies substantial duplication, but several of its proposed
replacements would weaken contracts or do not work as described at the pinned versions.
The strongest next step is a **shared compilation of semantic contracts into native
Arrow/DataFusion/Delta operations**, followed by consolidation of execution adapters and
relational compiler work. Moving repeated code into generic wrappers alone is insufficient.

This is an independent review of the implementation and of
[the initial review](design_review_consolidation-and-native-pivot_2026-09-18.md).
The initial review is useful evidence, not the recommendation boundary. Existing policies,
crate responsibilities, ADRs and implementation choices have **no veto over the proposed
architecture**. The criteria are the requested consolidation/native-library goals and
preservation of meaningful process-model behavior. Charter identifiers below describe
engineering consequences; they are not a policy-compliance score.

**Recommended center of gravity:**

1. Compile each complete relation contract once into exact Arrow fields, native local
   predicates, relational obligations, Delta storage mappings, and mechanical bindings.
2. Retain Arrow columns through validation and relational compilation. Use generated
   borrowed accessors or an indexed graph only at a genuinely specialized algorithm boundary.
3. Expose native optimization capabilities through adapters, while distinguishing pure
   transformations, validation barriers, shared computation, and durable effects.
4. Make declaration admission and reuse depend on actual contracts and owners. Digests
   accelerate lookup; they do not establish equivalence.
5. Consolidate Delta policy and operation assembly around its native builders without
   discarding publication, retention, and recovery coordination that those builders lack.
6. Extract a focused `pse-engine` and a dev-only `pse-testkit`; reuse existing crates for
   the other responsibilities. Do not introduce four new crates merely to relocate code.

**Scope:** existing Rust/Python behavior, generation, providers, preparation, execution,
memory, caching, diagnostics, and persistence. This review does not authorize implementation
or expand simulator functionality. In particular, connecting currently isolated solver
components into a new product workflow is a separate functional decision.

### 1.1 Method, coverage, and evidence limits

Reviewed on `main` at `a46f358bdfc2ca27f9f240ab6c045b63141c3ee9`, using the dirty working
copy containing the initial review and its existing book entry. Those files are preserved.
Independent work comprised dependency/feature resolution, both library skills' capability-gap
scans, targeted source/caller inspection, and small executable probes. Sources inspected
include the original review's critical paths plus local validation, P6 plan construction,
numerical expression expansion, native adapter hooks, and nested Python transfers.

The full API indices from the **DataFusion** and **Delta Lake** skills guided discovery;
selected signatures and behavior were checked against resolved local sources. No Context7
was used for Rust Arrow, DataFusion, or Delta. Context7 was used only for Python PyArrow
API discovery, followed by a probe against the installed version. A public DataFusion
rustdoc page corroborates the aggregate interface.

| Resolved component | Version / source |
|---|---|
| DataFusion | `55.1.0` |
| Arrow / Parquet | `59.3.0` |
| object_store | `0.13.2` |
| delta-rs | `deltalake-core 1.0.0`, revision `58f07cd62bfbce3649a7e1c87c696288068ae184`, workspace vendor overlay |
| Delta overlay | `vendor/delta-rs/PROVENANCE.json`; patch SHA-256 `b4047f3102627082170ee5b3babc5c28fcc36123f339df84dbf192e7677854f6` |
| serde_arrow / petgraph | `0.15.0` / `0.8.3` |
| PyArrow / PyO3 | installed `25.0.1` / resolved `0.29.2` |
| Rust | `1.98.1` |

**Implemented** means current source inspected at the cited location, not behavioral
certification. **Interface-checked** means the pinned API/source was inspected.
**Tested** is restricted to the named scratch probes in §9. All target changes are
**Proposed**. No throughput improvement, overall code reduction, or integration acceptance
is claimed. The initial review's LOC estimates are not independently re-measured here and
do not determine priorities.

The capability scans produced 84 DataFusion and 41 Delta candidate matches. These are
**Measured search results**, not defect counts: many are tests, multiline builder calls,
or false positives from newer native hooks. For example, `LeasedExec` supplies
`child_stats_requests` and `statistics_from_inputs`, which the simple statistics pattern
misses (`crates/pse-catalog/src/delta/leased.rs:220`).

**Not attacked:** whole-publication crash recovery, concurrent maintenance, complete
P0–P10 equivalence, numerical accuracy/parity, every generated relation's round trip,
canonical-hash equivalence after a proposed rewrite, or end-to-end performance. Parser
internals, all catalog declarations, all backend kernels, and the entire test corpus were
not exhaustively audited. `just doctor` reported one environment-freshness failure,
baseline zero; no environment repair or integration suite was needed for this review.
The [evidence manifest](../evidence/native-consolidation-reassessment-2026-09-18.json)
records commands, pins, probe source/results, inspected paths, and limitations.

### 1.2 Disposition of every initial finding

| Initial finding | Assessment | Improved recommendation |
|---|---|---|
| F1 — `Cell`, literal codecs | **Keep the deletion direction; change the architecture and correctness claim.** `literal_spec` deliberately preserves raw float bits, while registry hashing canonicalizes NaNs (`crates/pse-schema/src/model/cell.rs:102`, `:151`). Different purposes do not prove an identity fork. | Remove row reconstruction from production validation first. Use one lossless literal codec, one separately named semantic hash policy, native columns for bulk work, and `ScalarValue` only where a dynamic scalar is appropriate. |
| F2 — compiled declaration strings | **Keep the compaction goal; reject the proposed fallback.** The initial review's §4.2 accepts matching fingerprints and performs full comparison only on mismatch. Equal fingerprints can accompany changed declarations. `RelationSpec` alone also omits externally resolved enum/extension definitions. | Intern a complete resolved contract after exact structural admission; use its owner-bound handle thereafter. A digest mismatch can reject early; a digest match still requires exact admission when crossing an owner boundary. |
| F3 — memory | **Substantially revise.** `PoolReserver` delegates to the actual DataFusion pool; it is not an independent production budget. Arrow `claim` is infallible, replaces an existing allocation claim, and does not consume a prior `try_grow` reservation. | Consolidate on native `MemoryReservation`; implement and test one ownership-transfer bridge. Keep admission, retained allocation size, visible-copy size, and RSS distinct. Do not delete ownership machinery before its complete replacement exists. |
| F4 — framing, layouts, sort | **Consolidate framing; narrow the alleged defects.** Canonical `Layout` explicitly rejects Map, Union and run-end encoding, and dictionaries are enum strings (`crates/pse-ids/src/contract.rs:117`). The cited missing child walk is not evidence of a reachable hash collision on those rejected types. `RowConverter` is already an Arrow built-in. | Use native `DataType` plus a checked canonicalization descriptor; centralize byte framing and semantic metadata selection. Keep the current native row-order mechanism unless another native mechanism has a demonstrated advantage. Ordinary Rust callers should invoke the shared identity kernel directly; its UDF is an adapter. |
| F5 — extension triples / metrics | **Keep focused consolidation; reject uniform semantics and the blanket metrics claim.** `CacheExec::metrics` exists (`crates/pse-catalog/src/session/cache.rs:840`), as does resident resolution instrumentation. `ExecutionContract` equality is structural, unlike several opaque nodes. | Share scaffolding across a few explicit node families and generate mechanical delegation. Effects, identity, reset, dependency transfer, cancellation and output cardinality remain declared per family. |
| F6 — field-preserving UDFs | **Keep and deepen.** The missing opportunity is full native capability delegation, not just fewer wrapper files. | Share adapters that preserve relevant coercion, simplification, ordering, statistics, monotonicity, aggregate/window, and field-mapping hooks; rewrap rewritten native functions where needed. |
| F7 — table wrappers | **Keep native source reuse; reject the WorkTable equivalence.** `WorkTable::new/update` and `ReservedBatches` are crate-private, and consumption takes its batch slot. `RoundInputs` permits multiple readers of an immutable epoch and prevents replacement while readers live. | Share immutable memory and stream provider assembly. Use `StreamingTable`/`PartitionStream` for suitable sources; retain an explicit epoch owner for fixed-point inputs. A thin `MemorySourceConfig` provider may be simpler than hiding mutable `MemTable` state. |
| F8 — dependencies, configuration, caches | **Keep precision goal; substantially revise implementation.** `ExecutionContract` has one or two children; Cache may advertise zero. `Some(vec![output_columns])` is not a correct universal mapping. Whole-input evidence is conservative reuse, not missing evidence. | Keep optimizer column transfer separate from semantic dependency evidence. Preserve requirement reads, row membership, source ownership, hidden cache producers, and recursive/subquery scopes. Do not classify every future `datafusion.runtime.*` setting as semantically irrelevant by prefix. |
| F9 — relational compiler passes | **Strongly retain.** P7 loads typed vectors then resolves correspondence and provenance by scans. A regex count does not establish 127 nested-loop joins, and one physical execution for the whole pass is not established. | Native joins/aggregates/anti-joins for correspondence; one captured input per required boundary; stable support keys; bounded indexes for the remaining specialized work. Preserve explicit null, ordering, multiplicity and negative-evidence semantics. |
| F10 — graph traversal | **Retain shared graph access; reject the complexity claim.** Four full graph clones do not by themselves establish quadratic complexity. Different traversal relations can be intentional. Petgraph postorder alone does not return the current detailed cycle witness. | Define typed edge views and shared traversal utilities; avoid cloning the graph to add binding edges. Retain graph algorithms where they simplify actual graph work. Investigate the P8 omission with a payload-reference witness. |
| F11 — operator semantics | **Retain shared semantic contracts; correct evidence.** `fold.rs:79` reads `foldable`; `Pow` is explicitly non-foldable in the table. A conservative folder declining `Pow` and a predicate interpreter evaluating it are not proven contradictory results. Separate physical multiply/add kernels do not establish FMA contraction. | Bind evaluation, folding eligibility, lowering, differentiation and failure behavior explicitly. Share arithmetic kernels where meanings match. Address the concrete integer-remainder panic found by this review; verify unit conversions through differential bit tests. |
| F12 — Delta | **Retain native builders and bounded reader consolidation; revise retention and scope.** Automatic cleanup is disabled deliberately because it cannot see publication leases/protected versions. The Delta layer also owns repeated semantic validation lowerings, not just internal boilerplate. | Centralize operation policy, lifetime/attempt evidence, bounded action reading, and contract lowering. Drive lease-aware maintenance from one retention declaration; keep ordinary commits unable to delete protected history. |
| F13 — diagnostics | **Keep the declared-vocabulary drift finding; reject the literal API proposal.** `DataFusionError` is not `Clone`/`ToOwned`, so `Cow<DataFusionError>` is not the proposed drop-in type. Owned and borrowed classification serve different source-preservation needs. | Use one borrowed classification policy with separate ownership-preserving attachment. Reuse the currently unimplemented `pse-diagnostics` crate as a leaf vocabulary/diagnostic boundary, removing its unused higher-level dependencies. Model failure class and specific code explicitly if both are useful. |
| F14 — schema renderers | **Keep mechanical consolidation; narrow deletion.** Generated views are not independently authored authorities. Runtime Arrow schema reflection does not replace static Python typing or authoring validation automatically. | One structural field traversal plus separate renderer policies; native JSON serialization; named self-description projection. Keep generated typed API surfaces with actual consumers, generated from the same contract. |
| F15 — fixtures | **Keep.** Repeated fixture assembly and production assembly should share the production factory. | One `pse-testkit` for test-only roots, fault stores and assertions; production runtime assembly remains in production. A production module must not depend on testkit. |
| F16 — Python boundary | **Keep the named-report and traversal fixes.** A live probe confirms lost map-child metadata is reported as retained. Replacing the entire comparison with one Boolean would discard its useful transfer-state diagnostics. `get_all` does not validate construction. | Native nested field enumeration plus exact field comparison inside a small extension-aware transfer classifier; generated/configured boundary projections, named reports and one validation path. |
| F17 — unreachable code/dependencies/SQL | **Split three decisions.** A test-only solver is not automatically dead or a mandate to implement its product caller. Machine-like SQL is not proof a generator is missing. A bound logical plan cannot safely be cached by registry alone. | Remove proven unused dependencies and superseded entrypoints, declare experimental reachability honestly, choose readable authored SQL or an actual generator, and cache syntax separately from owner-bound plans. |

Further details and prioritized independent findings follow. The table is a complete
disposition, not an instruction to reproduce the initial review's implementation sequence.

## 2. Authority and lifecycle map

| Meaning | Authoritative form and owner | Scope / lifecycle | Derived native forms |
|---|---|---|---|
| Complete relation meaning | Resolved contract: Arrow fields plus keys, references, checks, enum domains, extension semantics and operation-relevant policies; `pse-schema` | Immutable registry revision; external declarations admitted structurally | Prepared relation handle, native predicates, Delta declaration, generated accessors |
| Documentation / display | Registry documentation fields | May change independently from execution meaning where it has no behavioral effect | Markdown, JSON Schema descriptions, diagnostics text; not automatically cache-invalidating semantics |
| Dynamic values | Arrow arrays/fields; `ScalarValue` at scalar boundaries | Owned/sliced native buffers; explicit schema | Literals, defaults, diagnostics projections; no parallel general-purpose `Cell` graph |
| Local validity | Predicates derived from the actual resolved field contract | Checked batch plus exact contract owner | Reusable `PhysicalExpr` predicates and structured violation rows |
| Relational validity | Keys, references, quantity correspondence and artifact obligations | Candidate selection / publication revision | Native grouping, anti-joins, checks; optimizer constraints only after establishment |
| Process math | Typed math relations | Model revision; stable semantic keys | Indexed graph during specialized analysis; staged native numerical program |
| Transformation behavior | Actual native plan, function implementations, and domain operation declaration | Assembly plus input binding | Optimized/physical plan; explicit dependencies and metrics |
| Resource policy | One configured native runtime/pool, cache and spill policy | Deployment / invocation scopes | Reservations and buffer ownership; metrics distinguish accounting from RSS |
| Persisted member | Native Delta table/version and durable field mapping | Single-table commit | Snapshot-bound Delta provider and native CDF |
| Coherent model result | Publication record selecting exact member versions | Publication commit | Provider catalog/schema/table hierarchy; retained reader leases |
| Durable attempt | Exact request, native transaction witness, and attempt receipt | One logical operation and its recovery attempts | Reconciliation state; never inferred solely from a digest or successful RPC |
| Diagnostic meaning | Declared failure class, specific code, structured context | Stable code schema and invocation evidence | `miette`/Python renderings; native `DataFusionError` remains causal evidence |

Keep catalog/schema/table providers as the access framework. Their hooks convey behavior
and native optimizer facts; they do not by themselves validate every domain invariant or
make several Delta tables one transaction. Domain algorithms and numerical callbacks stay
inside explicit native operation contracts, with ordinary Rust implementations where useful.

## 3. Semantic contracts and invariants

| Contract | Enforcement in the target | Failure behavior and evidence |
|---|---|---|
| Equal complete declarations are admitted; equal hashes alone are insufficient | Exact resolved comparison on foreign admission, then interned owner-bound handles | Reject altered nullability, enum membership, extension parameters, references or checks even with a copied fingerprint. Current protection: `crates/pse-schema/src/compiled_contract.rs:14`, `crates/pse-relations/src/columnar.rs:351`. |
| Physical storage, local values, relational validity and publication completeness are distinct | Separate evidence levels, all derived from one declaration | A field-checked candidate cannot advertise an unproved primary key. Existing distinction is explicit in `columnar.rs:40` and `delta/admission.rs:19`. |
| Null parent slots do not expose invalid hidden children | Predicate lowering carries the visible-occurrence mask and null policy | Validate active union arms / referenced dictionary values / visible list ranges; do not reject unreachable storage merely because it contains arbitrary bytes. |
| Literal fidelity and semantic identity have different equivalence policies | Separate named codec/hash operations over one structural value interface | Preserve NaN payloads in lossless literals; canonicalize only where the identity contract says so. |
| Requirements cannot disappear behind projection, limit, or cache reuse | Barrier dependency contract includes both value and requirement branches | A projected result cannot reuse evidence after a requirement-only input changes. |
| Declared purity and observed effects agree | Distinct native pure, barrier, shared-result and command families | No durable action during planning; effectful execution has settlement and retry rules. |
| Shared allocation remains accounted until its last owner releases it | Native reservation plus tested transfer/retention ownership | Claim transfer cannot silently move accounting to an unrelated budget; copies reserve their own allocations. |
| Mathematical edge roles and evaluation regions remain distinct | Ordered argument, payload, guard, and binding edges; region-aware lowering | A traversal must not authorize evaluation of an excluded branch or omit a payload dependency. |
| Mathematical errors remain errors, not overflow panics or silent rewrites | Exact integer kernels, finite/domain guards, explicit lowering/derivative capabilities | The minimum-integer division case produces a typed result/refusal. Native optimization is tested for failure and signed-zero behavior. |
| Publication and recovery survive partial effects | Existing publication selection, exact attempt reconciliation and retained-version protocol, consolidated around native Delta operations | A committed member alone is not a published model; missing CDF/log evidence is not proof of no change. |
| Schema transfer reports actual nested meaning | Native field enumeration with extension-aware observations | Map/union/list-view metadata differences produce a mismatch/loss observation, not “retained.” |

These are functional requirements of the target. Keeping their meaning does not require
preserving predecessor APIs, generated strings, old data objects or historical storage.

## 4. Derivation and execution design

### 4.1 Compile one contract into the native mechanisms that enforce it

The principal consolidation boundary is a **prepared relation contract**, not a new
universal row type. Resolve registry references once, normalize the semantic descriptor,
admit it structurally, and derive these products together:

| Product | Native implementation | Application-specific remainder |
|---|---|---|
| Physical fields and schema | Arrow `Field` / `Schema` / extension metadata | Meaning of units, references, tagged alternatives and collection domains |
| Local checks | Native `Expr`, comparisons, `IN`, `gcd`, nested field access, higher-order array predicates; reusable `PhysicalExpr` | Visible-parent masks, specific domain constraints, structured diagnostic mapping |
| Relational checks | DataFusion aggregates, anti-joins, projections, `UNNEST`, set/multiset difference as appropriate | Which relation/selection satisfies a reference; obligation completeness |
| Delta write checks | Delta non-nullability, CHECK, invariants and generated-column machinery where the operation fits | Exact execution/storage translation and the existing nested-predicate adapter where Delta SQL cannot parse lambdas |
| Query optimizer facts | Native `Constraints`, statistics, order/partition properties | Evidence that the selected source actually establishes the advertised fact |
| Rust/Python APIs and docs | Mechanical generation from the resolved contract | Ergonomic names, domain newtypes, authoring constructors and presentation policy |

**Current foundation:** `crates/pse-catalog/src/delta/predicates.rs:61` already lowers
field contracts into native expressions; `delta/nested_check.rs:26` compiles a predicate
once with `SessionState::create_physical_expr`. Lift those semantics into a shared
non-Delta contract compiler. The local boundary then evaluates the same predicate against
Arrow arrays instead of constructing `Cell` for each row.

Keep low-level Arrow structural validation before evaluating a predicate. The Delta writer
trusts some source nullability claims and its `validation_predicates` helper is crate-private
(`vendor/delta-rs/crates/core/src/delta_datafusion/data_validation.rs:269`). Invoke Delta's
public write path for durable enforcement; do not propose calling that private helper or
relying on it to validate arbitrary imported Arrow buffers.

A violation plan should return typed rows containing the rule identity, relation/key,
nested occurrence path, and relevant observed values. One Boolean predicate alone is
insufficient to replace today's detailed errors. Aggregate requirements into one bounded
barrier per appropriate stage and retain shared inputs once; do not indiscriminately merge
all stages into one enormous plan.

### 4.2 Contract identity, generated code, and values

Replace per-batch string declarations with a shared, immutable **resolved contract handle**.
The handle contains an internal owner identity and complete resolved semantic descriptor.
Generated adapters supply their expected contract once; the registry resolves/binds it
once. Pointer equality is then a valid fast path within that admitted owner.

For another registry, independently supplied declarations, or deserialization, compare
the complete resolved descriptor. Include referenced enum/extension definitions and the
policies that affect interpretation; comparing only `RelationSpec` is insufficient.
Use fingerprints for indexing or early rejection, followed by exact comparison on a
candidate match. Documentation can be excluded from *execution* identity after its role
is explicitly separated; it need not be erased from the registry.

Retain useful generated `Row` types and borrowed column views. Deduplicate runtime
builder/view machinery behind shared column primitives and `TypedBatch<R>`-style façades.
Generation already gives one authored definition; hundreds of generated files are not
hundreds of semantic authorities. Optimize generated size where it removes compile or
admission cost, and measure that cost.

Remove `Cell` once its active registry projections, literals, validation and adapter
consumers have native replacements. Bulk work should use arrays/builders and masks.
Converting every array element to `ScalarValue` merely to call a generic writer risks
replacing one row allocation path with another. A shared codec can have an array-oriented
entry point and a scalar adapter using the same format policy.

`serde_arrow` remains a candidate for genuinely row-oriented interchange, **not the
default hot-path architecture**. Its explicit-schema path must pass exact nested metadata,
fixed-size IDs, null-parent, enum, borrowing and allocation checks. `SemanticId`'s existing
Serde text representation cannot be assumed to serialize to fixed-size binary. A proc
macro can reduce generated text without reducing expanded compile work. Neither route
has a measured advantage in this review.

### 4.3 A focused engine crate, with explicit dependency direction

Proposed ownership:

| Crate / module | Target responsibility | Dependency constraint justified by the design |
|---|---|---|
| `pse-schema` | Resolved Arrow/domain declarations, semantic contract projection and code generation | Can use native DataFusion common/expression types where useful; no dependency on catalog/publication services |
| `pse-relations` | Arrow construction/views, exact admission handles, prepared local predicates, structured local violations | Native Arrow/DataFusion dependencies are allowed; receives actual native preparation context rather than constructing an unrelated session |
| `pse-ids` | Semantic IDs, canonical value policy and framing | Direct modular DataFusion resource types are permissible if required; no dependency on the higher `pse-engine` or catalog |
| **`pse-engine` — new** | Native session assembly, operation families, safe native adapters, scoped plan traversal mechanics, function bindings, shared runtime/cache policy interfaces | Depends on the foundation crates and DataFusion; no Delta/publication dependency |
| `pse-catalog` | Catalog/schema/table bindings, selected snapshots, artifacts and the `delta` module | Depends on engine; installs Delta-specific planners/providers into the native assembly |
| `pse-diagnostics` — repurposed | Leaf diagnostic codes/classes, structured causal attachment and common engine classification | Remove presently unused numerics/structural/relations dependencies; native errors remain sources rather than stringified replacements |
| Compiler / rules / numerics / math / quantity | Domain lowering, native relational plans, specialized graph and numerical algorithms | Consume native engine interfaces; storage coordination belongs above their reusable algorithm components |
| `pse-runtime` | Product composition, explicit runtime lifecycle and publication orchestration | May legitimately depend on both compiler and catalog |
| **`pse-testkit` — new** | Reusable fixtures, fault injection, small budgets and assertions | Dev-only; calls the production factory |

`pse-diagnostics` currently contains only its declared boundary and unused high-level
dependencies (`crates/pse-diagnostics/src/lib.rs:8`, its manifest). Repurposing it avoids
a new `pse-codes` crate and solves the low-level vocabulary dependency issue directly.
If numerical diagnostic analyses are later implemented, place them above this leaf
contract; that future implementation is not part of this consolidation.

A separate `pse-delta` is unnecessary for the inspected consumers. Keep a cohesive Delta
module behind explicit contracts; extract it only if a concrete independently useful
consumer or feature/build boundary emerges. Conversely, extracting `pse-engine` has a
real purpose: algorithms should not import a persistence crate for generic native
execution services. Moving files alone does not achieve that. Narrow `AlgorithmContext`
and move its required native services before claiming downstream Delta dependencies
have disappeared.

### 4.4 Share native execution mechanics without collapsing distinct operations

| Family | Shared mechanics | Semantics that must remain explicit |
|---|---|---|
| Pure relational transformation | Native logical operators and built-ins | No custom extension when ordinary plans express the operation |
| Finite specialized algorithm | Common logical/physical shell, input admission, typed result ports, cancellation and native metrics | Actual algorithm identity, whole/column input consumption, field derivation, partitioning, deterministic ordering, memory, reset and failure rules |
| Validation / lease barrier | Native stream forwarding and selected property delegation | Requirement children and read leases; no pushdown that bypasses required validation |
| Shared computation | Existing native-cache completion/spill mechanics | Actual producer identity, bound sources, invocation/epoch, hidden optimizer leaf and failure/cancellation ownership |
| Durable command | Deferred execution scaffold and operation report | Attempt identity, effect declaration, settlement after cancellation, commit ambiguity and durable recovery |

Use one planner dispatch/registration surface where helpful. A generic
`Algorithm<A>` does not itself guarantee one machine-code implementation: monomorphization
and delegated methods still exist. Prefer a small type-erased operation descriptor with
actual implementation identity where that simplifies dispatch, plus typed algorithm
implementations. Do not build a second scheduler or a generic workflow language.

For custom sources, use `MemorySourceConfig`, `StreamingTable`, `PartitionStream` and
`RecordBatchStreamAdapter` as appropriate. A short immutable provider is already good
native reuse; wrapping `MemTable` is optional, not intrinsically more native. Private
candidate sources must continue withholding unproved uniqueness. Epoch sources need
reader lifetime control independent of their stream adapter.

For metadata-preserving function adapters, forwarding `invoke` and `return_field` is
insufficient. Audit the **provided methods** as well as required methods. The current
min/max wrapper forwards accumulators but omits native `is_descending`,
`value_from_stats`, and `set_monotonicity` behavior
(`crates/pse-catalog/src/session/aggregate/selection.rs:44`; upstream
`datafusion-functions-aggregate-55.1.0/src/min_max.rs:353`). These are actual native
optimization opportunities. The pinned
[AggregateUDFImpl interface](https://docs.rs/datafusion-expr/55.1.0/datafusion_expr/trait.AggregateUDFImpl.html)
also exposes simplification and beneficial-ordering hooks. Delegate only when the adapter
preserves the relevant preconditions, and re-establish its field contract after a rewrite.

Use native `BaselineMetrics`/metrics sets for custom execution. Forward native statistics
with `child_stats_requests` / `statistics_from_inputs` on truly transparent wrappers;
do not manufacture exact cardinalities for algorithms or changing epoch sources.
Keep schema/ordering/partition properties truthful on reconstruction and reset.

### 4.5 Relational compiler work and specialized graphs

P7 correspondence, P8 domain assembly, P9 selection and template path resolution are the
largest remaining relational candidates. Express matching, grouping, set difference and
bounded reachability as native plans. Keep source keys alongside values, rather than
unzipping keys and later recovering their positions
(`crates/pse-compiler/src/passes/p7/inventory.rs:120`,
`passes/p7/evidence.rs:272`). Construct shared support relations with `UNNEST` and joins.
Keep absence/negative evidence alongside positive lineage.

A shared plan-construction module should use native qualified `Column` identities and
`LogicalPlanBuilder`, with explicit null equality and generated field references. Avoid a
second alias-encoding mini-language or a new AST that mirrors `LogicalPlan`. Consolidate
helpers only where they express an actual domain convention.

P6 already uses native joins and nested functions (`passes/p6/framing/dependencies.rs:23`).
It needs shared requirement/capture assembly, not a fresh pivot away from vectors.
Some intermediate materialization is purposeful: it prevents repeated producers and
unbounded plan expansion. Choose those boundaries from native plan reuse and measured
preparation/execution costs, rather than prescribing one query for each entire pass.

Keep an indexed graph for topological ordering, cycle witnesses, region-sensitive
canonicalization and symbolic differentiation. Derive it once from admitted relations,
retain stable node/row identity, and discard or explicitly cache it under complete inputs.
Use ordered edge kinds: argument, payload reference, guard, and kernel-binding input.
Provide explicit traversal views for dependency closure versus evaluation order.

Petgraph is suitable where it removes actual graph machinery. It is not a substitute for
edge semantics, stable traversal ordering or cycle witnesses. The first improvement to
`postorder_with_bindings` is an adjacency view over existing nodes plus binding edges;
its current whole-graph clone (`crates/pse-mathir/src/topo.rs:25`) is avoidable without
changing graph representation at all. Validate the P8 traversal against payload-only
references before merging it with another walk.

### 4.6 Preserve sharing during numerical lowering

An opportunity missed by the initial review is expansion **inside native expressions**.
`ScalarMath::compile` stores an owned `Expr` per node and clones child trees into parents
(`crates/pse-numerics/src/scalar_math.rs:87`, `:118`, `:142`). A repeated diamond graph can
therefore expand before any native optimizer sees it. `differentiate::compile` recursively
rebuilds values/gradients, and `EvaluationProgram` prepares each resulting expression
separately (`expressions.rs:138`). Being expressed in DataFusion does not remove this cost.

Target a compact staged native program: retain node identities while lowering, introduce
native projection aliases/stages for shared pure computations, and prepare reusable
physical expressions for those stages. Carry sparsity and derivative bindings alongside
that shared graph, rather than repeatedly cloning derivative trees. Bind changing solver
values as Arrow batches; continue avoiding query planning in a numerical callback.

Do not hoist a guarded operation into an eagerly evaluated stage. Partition reuse by
evaluation region and function volatility, and preserve source-level failure behavior.
Check DataFusion projection/CSE rewrites for renewed inlining before claiming bounded
preparation. This is a proposed architecture with a concrete adversarial oracle, not a
claim that native common-subexpression elimination already solves the current path.

The operator declaration should bind distinct capabilities: evaluate, eligible-to-fold,
differentiate, infer quantity, and lower to a native function/kernel. A descriptive table
of rule strings cannot implement all five. Share checked integer and unit arithmetic;
use native mathematical functions where their behavior matches the required semantics;
use an explicit UDF/kernel where exact failure or precision behavior needs it. A UDF is
not automatically differentiable.

### 4.7 Memory ownership: native accounting with a tested transfer boundary

Use DataFusion `MemoryConsumer` / `MemoryReservation` directly for runtime accounting,
including fallible `try_grow`, and use native pools in tests. Remove the parallel reservation
trait once its consumers use the native contract. Keep a small shared owner where one
reservation must survive several buffers; native reservation ownership does not by itself
provide an `Arc` lease for arbitrary foreign buffers.

Arrow's pool feature is useful, but its adoption must specify the full feature set:
`arrow-array/pool` enables array/batch claims and transitively the buffer/data features;
`datafusion-execution/arrow_buffer_pool` enables the native bridge. Enabling only
`arrow-buffer/pool` does not expose `RecordBatch::claim` or the DataFusion adapter.
Resolved metadata confirms these features are currently off.

At this pin `Bytes::claim` replaces its old reservation with `pool.reserve(capacity)`;
`ArrowMemoryPool::reserve` calls native **infallible** `grow`. Thus “try_grow, then claim”
is not a transfer algorithm: it can temporarily double-account, and releasing the first
reservation before claiming creates a gap. Reclaiming shared buffers against another
pool changes their accounting owner, as probe P1 demonstrates.

Implement one transfer bridge that consumes pre-acquired native capacity and owns its
release, or retain the current lease mechanism for boundaries that cannot transfer it
correctly. Adopt native Arrow claims only where allocation identity and ownership are
established. This is a small necessary adapter, not a second budget universe. Choose one
deployment pool for shared allocations; attribute invocations separately from budget ownership.

Keep three different measurements explicit:

- **Retained capacity:** actual allocation kept alive by a slice/clone, deduplicated across
  owners; needed for lifetime accounting.
- **Visible/copy estimate:** values and buffers needed for a selected representation;
  useful for copy or decoding scratch, including dictionary expansion.
- **Process RSS:** broader observation including unaccounted/native/foreign allocations.

`ArrayData::get_slice_memory_size` helps some estimates, but is not a retained-capacity
measure or a complete nested compaction oracle. Probe P1 measured 4,012 bytes for a
one-element List slice whose single visible Int32 child plus offsets would need 12 bytes:
the estimate includes the unsliced child data. Removing `Cell` decoding also removes much
of the reason for today's large decoding forecasts. Optimize that cause first.

### 4.8 Delta persistence and retention

Keep native table providers, CDF, write/update/delete/merge, optimize, vacuum, checkpoints,
commit properties and caller-session binding. Their use is already substantial. The
additional consolidation is one operation context carrying the actual session/runtime,
bound contract, commit metadata, attempt semantics, resource ownership and maintenance
policy. It configures the native builder; it does not reimplement Delta transactions.

Centralize the lossless Arrow-to-Delta storage descriptor and its encode/decode direction.
Native Arrow casts perform conversion where sufficient; mappings such as UInt64 to
Decimal128, fixed binary to binary, and restoration of list child metadata remain declared
because Delta's physical type system differs (`crates/pse-schema/src/delta.rs:142`).
Do not erase those distinctions in pursuit of one physical schema.

Move duplicated local and relational validation assembly into §4.1's contract compiler.
Derive durable CHECK constraints and read-time validation from the same declarations, with explicit
execution/storage translation. Delta can enforce local row predicates, but not an entire
multi-table publication's foreign keys or completeness by itself.

Use one bounded exact-version log/action reader for recovery and post-commit observations.
`write_count` currently reads an entire commit entry without the bounded receipt path
(`crates/pse-catalog/src/delta/dml/execution.rs:270`). Preserve the distinction between
“commit failed” and “commit succeeded but observing its metrics failed.” Native operation
metrics/hooks are preferable when they provide the required exact information; recovery
still needs a durable witness.

Consolidate retention as a declaration of minimum ages, selected/protected versions,
active readers, CDF intervals and unresolved attempts. Ordinary commits keep automatic
log cleanup disabled. Maintenance translates the effective retention set to native
`with_keep_versions`, Full/Lite vacuum and bounded log cleanup. The current choices in
`delta/maintenance.rs:575` protect CDF files and attempt evidence; replacing them with the
table property alone would lose information. A declared `logRetentionDuration` can supply
a time floor, not the complete retention decision.

Preserve the coherent-publication root and exact attempt reconciliation. Native
`SetTransaction` actions are durable evidence, not an automatic replay-deduplication API
at this pin (`vendor/delta-rs/crates/core/src/kernel/transaction/mod.rs:479`). Checkpoints,
CRC summaries, catalogs and individual table commits do not replace those semantics.

### 4.9 Reuse, settings, diagnostics, and Python boundaries

Share native traversal **mechanics**—bounded memo tables, cancellation, actual node
identity, recursion handling—and keep purpose-specific scope/context keys. One
`PlanWalk::read(plan, f)` without context parameters is insufficient for correlated
subqueries, recursive worktables, effect ancestry and hidden cache producers.

For dependency precision, expose the actual producer to an evidence traversal without
reopening its optimizer boundary. For an `ExecutionContract`, preserve requirement input
dependencies as well as output columns. For unknown extensions retain whole-input evidence.
Native `necessary_children_exprs` is an optimizer contract with one index list **per child**;
it does not describe every semantic dependency or make a zero-child cache leaf transparent.

Keep one effective configuration constructor and an explicit versioned classification of
settings. Defaults, deployment overrides and a safety clamp can be legitimate stages;
their existence alone is not conflicting authority. Unknown options remain conservatively
semantic until classified. Reuse the existing bounded native caches; merge resident and
snapshot cache bookkeeping only where keys, memory charges, failure behavior and freshness
remain parameterized explicitly.

Cache parsed rule syntax by SQL text, dialect and parser-affecting settings. Cache a bound
logical/physical plan only under its actual providers, function/rule implementations,
settings and schema ownership. `bind_declared_query` performs both parsing and binding
(`crates/pse-catalog/src/session/native.rs:75`, `:154`); registry identity alone cannot
key the latter. Native plan codecs remain diagnostic/transport encodings, not canonical
semantic hash formats or portable authority certificates.

Centralize diagnostic classification without flattening `DataFusionError::External`,
`Shared`, `Context` or `Collection`. A borrowed classification decision can be shared;
owned attachment preserves the original source, and shared attachment retains its `Arc`.
Use a declared class/code mapping instead of either handwritten spelling drift or forced
loss of useful specific codes. Generate registry and renderer projections from that owner.

At Python boundaries, use one settings declaration to generate thin projection code and
construction checks. `#[pyclass(get_all)]` is useful for a simple named immutable report;
it does not validate input or eliminate constructor conversion. Keep PyO3 attachment in
the boundary crate where practical rather than adding Python runtime coupling to every
core struct. Preserve generated typed contracts where they serve authoring/static typing.

Use PyArrow `DataType.num_fields` / `field(i)` for nested fields, including Map/Union;
handle dictionary value types and extension storage explicitly where they are type rather
than field children. Use `Field.equals(check_metadata=True)` for exact comparisons while
retaining the current distinctions among retained, storage-only, lost metadata and mismatch.
Return named resource reports, normalize duration widths, and define one ID-text admission
contract across languages.

## 5. Representative journeys

### Ordinary extension: add a nested collection constraint

Add the semantic declaration once, including null-parent behavior, element domain and
uniqueness/cardinality. The shared contract compiler produces the native local predicate,
violation projection, durable CHECK or nested-check adapter, and generated API/docs.
Existing Arrow containers and native nested functions perform the work. A new bespoke
value variant, Python validator, Delta-only predicate and per-pass Rust walker should
not all need separate edits. A truly new semantic primitive needs one implementation
binding and a cross-boundary conformance case.

**Oracle:** the same valid/invalid nested values receive the same decision and location
at local batch admission, rule output, Delta write and publication validation. Hidden
values under null parents remain unobserved; a violated active element remains visible.

### Meaningful change: change an enum domain without changing a copied fingerprint

An externally constructed registry copies the relation ID and fingerprint but changes
a resolved enum member or field nullability. The complete descriptor comparison rejects
it before generating an admitted handle. A genuinely equivalent registry can bind once,
then use its admitted handle for subsequent typed borrows. A documentation-only revision
changes documentation without inventing a new execution schema.

**Oracle:** changed semantic declarations fail even with equal IDs/digests; equal complete
declarations from distinct owners succeed only through structural admission. A handle
cannot be fabricated from an ordinal, public digest, or another registry's pointer value.

### Boundary: a typed nested value through Delta and Python

A List/Struct/Map value moves through native computation, lossless Delta storage mapping,
version-pinned scan, field restoration and Arrow C stream export. Native Arrow casts
perform mechanical conversion, while the resolved contract supplies exact metadata and
nullability. The Python consumer can accurately report storage-only handling without
pretending the extension is registered. Altering map-item metadata produces a mismatch.

**Oracle:** identical logical values and full field contracts after a round trip;
misleading metadata, wrong enum domains, unknown required contracts and unsupported
lossless storage mappings are refused explicitly. Probe P2 currently demonstrates the
map-metadata observation defect before such a complete journey is attempted.

### Interruption: a member commits before publication is acknowledged

The native Delta operation records its exact request/transaction witness; publication
selects exact member versions only after obligations succeed. A cancelled waiter cannot
make an unresolved write safe to retry blindly. Recovery reconciles the durable evidence;
maintenance sees active attempts and protected selections before deleting files or logs.
The shared command shell centralizes reporting/cancellation mechanics, while this durable
state machine remains explicit.

**Oracle:** faults before write, after member commit, after publication commit and during
metrics observation lead to distinguishable outcomes. Repeating the same request does
not duplicate effects, conflicting requests do not share a receipt, and incomplete
members are not visible as complete model artifacts. Not executed in this review.

### Graph extension: add a payload-only dependency

A new operator references a node in its payload rather than its ordered argument list.
The declared dependency-edge view includes it; graph reachability, type inference,
provenance and native lowering consume that view. Its evaluation view still honors a
guard. No global “children means everything” convention is introduced.

**Oracle:** a payload-only symbol/group is discovered, a cycle returns a concrete path,
and an excluded branch containing an invalid arithmetic operation remains unevaluated.

## 6. Acceptance gates

These are independent technical assessments. They do not claim compliance with existing
repository policy, which the requested design deliberately does not use as a constraint.
A **Fail** identifies the specific current defect or unsafe initial proposal described
in that row; it does not condemn every current implementation of that concern. The
replacement mechanisms remain Proposed until their closure evidence exists.

| Gate | Current implementation / initial proposal | Target assessment and closure evidence |
|---|---|---|
| **G1 — Authority** | Current exact compiled-contract admission is protective; initial F2 would weaken it. Local/Delta contract rules are independently implemented. | **Fail: initial admission proposal.** Use one resolved contract and exact admission, then owner-bound handles. Cross-registry adversarial test required. |
| **G2 — Semantic fidelity** | P2 confirms Python map-child metadata loss is misreported. Initial metadata/hash and NaN claims overstate different-contract behavior. | **Fail: current transfer observation.** Native structural traversal plus explicit equivalence/extension policies; full storage/consumer matrix still untested. |
| **G3 — Validity** | P1 reproduces the overflow panic in the P4 guard expression. Cell-based admission and native Delta predicates have no complete equivalence result here. | **Fail: current arithmetic guard.** Checked arithmetic and one contract lowering; negative tests for visible nested values, exact declarations and invalid references. |
| **G4 — Hidden behavior** | Explicit execution contracts and deferred Delta operations exist. A universal execution shell must not erase their effect and reset distinctions. | **Unresolved for replacement.** Planning/EXPLAIN must remain effect-free; barrier, command and shared-computation lifecycle tests required. |
| **G5 — Consistency and recovery** | Publication/attempt/lease mechanisms are inspected, not crash-tested here. Initial cleanup-property substitution lacks protected-version inputs. | **Unresolved; retain required semantics.** Fault-injection recovery and retention tests are mandatory before replacing these mechanisms. |
| **G6 — Transformation and reuse** | Whole-input dependency capture is conservative. Initial projection mapping, registry-only plan caching and memory claim transfer are incomplete. Native wrappers suppress some optimizer capabilities. | **Fail: initial reuse/transfer shortcuts.** Context-sensitive dependency evidence, full native adapter conformance, and ownership-transfer tests before finer reuse. |
| **G7 — Truthful capabilities** | Initial claims of no custom metrics, directly reusable WorkTable, and `Cow<DataFusionError>` are contradicted by source/API inspection. Several proposed benefits have no measurement. | **Fail: cited capability claims.** Public reachable APIs and actual consumer paths must be named; prototypes and measurements remain explicitly pending. |

No gate is converted to “passed” by a lower line count, a native type name, or the fact
that an earlier plan's unrelated acceptance run was green.

## 7. Principle findings

Priority distinguishes an unsafe proposed replacement from a demonstrated current defect
and from a valuable improvement. The detail in §4 is part of each proposed correction.

**Severity order:** correct the demonstrated defects in R8/R10; reject the unsafe
replacement shortcuts in R1/R3/R5/R9/R11; consolidate semantic authority and extension
paths in R2/R4/R6/R12; investigate and bound numerical preparation in R7. No performance
finding outranks correctness on the basis of an unmeasured speedup. Dependency order in
§11 is separate from severity: contract foundations enable several later deletions.

| ID / finding | Principles | Concrete evidence or gap | Consequence | Proposed correction | Verification |
|---|---|---|---|---|---|
| **R1 — Contract compaction must preserve resolved equality** | DM-02, DM-15, DM-42 | Initial §4.2 accepts equal fingerprints; current `crates/pse-schema/src/compiled_contract.rs:81` includes bound fields and resolved extension/domain facts. | Two different declarations can become one admitted typed batch if the proposed shortcut is adopted. | §4.2: exact resolved descriptors, interned handles, digest as lookup/rejection only. | Same-digest altered enum/nullability/check/reference cases; equivalent foreign registry; no repeated full-text compare on the admitted fast path. |
| **R2 — Local validation reconstructs a second value system** | DM-07, DM-25, DM-37, DM-56 | `crates/pse-relations/src/validate/mod.rs:120` and `:160` decode every visible value via `cell_at`; `session/capture/values.rs:36` delegates PSE fields there. Delta independently lowers predicates at `crates/pse-catalog/src/delta/predicates.rs:61`. | Every new semantic field contract can require independent validators; native arrays incur row reconstruction before returning to native execution. | §4.1: one prepared predicate/obligation compiler; evaluate native arrays; remove Cell consumers and the unused bundle-validator path. | All declared field families, null parents, invalid active arms, dictionaries, nested paths; identical violation decisions across local and durable boundaries. |
| **R3 — Transparent-looking nodes need complete dependency contracts** | DM-28, DM-31, DM-32 | `crates/pse-catalog/src/session/contract.rs:68` exposes value plus optional requirement child; `session/cache.rs:248` sometimes exposes none; `artifact/consumption.rs:86` conservatively stops at extensions. | Initial F8's one-list transfer either has wrong arity, misses requirements/producer reads, or fails to improve reuse at all. | §4.9: separate optimizer transfer from semantic evidence; preserve requirement/absence/key dependencies and scope. | Change only a requirement input; nested correlated query and recursive worktable; shared producer visited once; unknown extension stays conservative. |
| **R4 — Native adapters leave optimizer capabilities behind** | DM-25, DM-41, DM-44 | `crates/pse-catalog/src/session/aggregate/selection.rs:44` lacks the min/max hooks present at upstream `min_max.rs:353` and `:367`; adapter consolidation alone would retain this gap. | Queries cannot use native ordering/statistics/monotonicity information supplied by the wrapped implementation. | §4.4: conformance-driven delegation including provided methods, rewrapping and metadata postconditions. | Native versus adapted plans/results, ordered aggregation, statistics substitution, empty/all-null groups, rewritten fields and native metrics. |
| **R5 — Buffer claims are not allocation admission or ownership transfer** | DM-29, DM-35, DM-37 | Arrow `bytes.rs:112` replaces its claim; DataFusion `memory_pool/arrow.rs:62` calls `grow`; P1 observes transfer between pools and a nested slice estimate retaining excess child extent. | The initial F3 replacement can move accounting unexpectedly, double-charge or leave a gap, or misestimate scratch. | §4.7: native reservations plus one tested transfer bridge; explicit retained/copy/RSS metrics; eliminate row-decoding forecasts with R2. | Shared slices/columns/dictionaries, repeated claims, concurrent pools, failed admission, cancellation, FFI ownership and last-owner release. |
| **R6 — Relational correspondence becomes repeated decoded scans** | DM-18, DM-36, DM-38, DM-46 | `crates/pse-compiler/src/passes/p7/inventory.rs:120` separates rows/keys; `passes/p7/evidence.rs:280` recovers positions by scan; `passes/p8/expansion/axes.rs:28` rescans members per domain. | Optimizer-visible joins and source identity are lost at a boundary that does not need a graph algorithm. | §4.5: keep correspondence/provenance native, share one captured input, retain keys; use a bounded index only for specialized consumers. | Unordered inputs, duplicates, missing matches, null keys, provenance and incremental/clean equality; cold/warm scale measurements. |
| **R7 — Graph sharing is lost during native numerical lowering** | DM-18, DM-26, DM-36 | `crates/pse-numerics/src/scalar_math.rs:87`, `:118`, `:142`; `differentiate.rs:24`; `expressions.rs:138` construct/copy expression trees before physical preparation. | A compact repeated-diamond graph can cause large preparation allocations; the native engine receives the expanded tree too late. | §4.6: region-aware shared graph lowering to staged native projections/physical expressions; shared sparse derivative representation. | Repeated-diamond nodes/edges versus expression/preparation growth; conditional failure cases; Jacobian numerical oracle. No speedup established yet. |
| **R8 — Mathematical roles are conflated by the proposed single evaluator** | DM-24, DM-40, DM-44 | `fold.rs:79` respects fold eligibility and Pow is non-foldable; P4 `scalar.rs:349` instead computes `a % b` before checked division. P1 demonstrates the minimum-integer panic. `topo.rs:25` clones binding edges; P8 `axes.rs:86` omits payload references. | Blanket unification can remove useful conservative behavior while overlooking actual failure and dependency defects. | Shared checked kernels plus distinct evaluation/folding/lowering/derivative contracts; typed graph edge views, with a payload witness. | Minimum/maximum integers, nonintegral division, overflow, Pow eligibility, signed zero, domain guards, payload-only dependencies and cycle paths. |
| **R9 — Delta consolidation must include semantic obligations and retention** | DM-07, DM-14, DM-30, DM-56 | `crates/pse-catalog/src/delta/admission.rs:23` assembles relation obligations; `delta/dml/execution.rs:270` reads an unbounded commit; `delta/maintenance.rs:575` coordinates versions, CDF and attempt logs. | “Internal duplication only” misses a shared contract compiler; replacing cleanup with table defaults can delete required recovery/history evidence. | §4.8: native builder context, shared obligations/storage descriptor, bounded exact-version action reading, one retention policy with all protection inputs. | Local/Delta validation matrix; post-commit observation failure; held reader/CDF/attempt history under vacuum/log cleanup. |
| **R10 — Python transfer observations miss nested map metadata** | DM-10, DM-41, DM-42 | `python/pse/_transfer.py:34` enumerates only struct/list families; P2 changes map-item metadata and receives `retained` while PyArrow exact field equality returns false. | A consumer is told meaning survived when its nested field changed. This is a confirmed observation defect, not proof publication admission is bypassed. | §4.9: native nested enumeration and exact comparisons inside the extension-aware observation model. | P2 as a regression case; Map/Union/ListView/dictionary/extension combinations and storage-only handling. |
| **R11 — Reuse/configuration consolidation needs semantic keys** | DM-26, DM-31, DM-32, DM-33 | `crates/pse-catalog/src/session/native.rs:75`, `:154` bind SQL under actual sources; `session/config.rs:113` deliberately retains unknown settings; initial F17 suggests registry-only plan caching. | Reusing a bound plan with another provider/function assembly can use stale computation; prefix-classifying unknown settings can suppress invalidation. | §4.9: separate syntax and bound-plan caches; one effective policy constructor with versioned semantic classification. | Same SQL/registry with changed provider or UDF, parser settings, changed resource budget, unknown setting and cancelled cache population. |
| **R12 — Consolidation can create unnecessary platforms** | DM-47, DM-52, DM-57, DM-58 | Initial §4 introduces four crates; `pse-diagnostics` is presently an unimplemented boundary, while DataFusion error ownership is non-cloneable. Generated field/table output repeats a shared generator, not independent authored schemas. | A new codes crate, universal execution abstraction, macro codec, and Delta crate can add coordination without deleting semantic decisions. | §4.2–§4.4/§4.9: two justified new crates, repurpose diagnostic boundary, shared production factory, mechanical generation, explicit public APIs and actual consumers. | Dependency DAG, ordinary-extension edit count, generated/runtime conformance, borrowed/owned error cause preservation, compile and preparation measurements. |

### Applicable principle assessment

| Principle groups | Assessment in this review |
|---|---|
| DM-01–15: authority, types and identity | Applicable to resolved contracts and representation boundaries. R1 is a defect in the initial proposal; R2/R10 identify present duplication/loss. Do not equate different literal/hash policies with contradictory authority. |
| DM-16–25: composition and lowering | Applicable to the shared contract compiler, native plans, adapters and operation families. Significant opportunity remains; native APIs alone do not implement the domain contracts. |
| DM-26–35: execution, dependencies and recovery | Applicable to staged preparation, barriers, caching, ownership and Delta publication. Several replacement contracts remain unresolved; current conservative boundaries are not evidence of unsoundness. |
| DM-36–45: representations and providers | Applicable to columnar validation, graph views and native capability delegation. P1/P2 establish narrow boundary behavior; the target's full performance and conformance remain untested. |
| DM-46–55: lineage, diagnostics and verification | Applicable to stable support keys, structured violations, source-preserving diagnostics and generation. No new numerical/parity guarantee is claimed. |
| DM-56–60: architectural leverage | Applicable across scope. Favor fewer independent semantic choices over fewer generated lines; select focused extraction and native primitives before introducing a broader framework. |

Specific numerical method quality, new backend capabilities and historical migration
mechanisms are outside scope. Their absence is not marked as a completed design improvement.

## 8. Alternatives and architectural leverage

| Alternative | Duplication and extension locality | Correctness / operational implications | Cost and evidence | Decision |
|---|---|---|---|---|
| Current implementation | Native execution and durable operations are already widespread; local Cell validation, relation-to-vector correspondence and adapters still duplicate work. | Exact declaration admission, publication, epoch and ownership boundaries are useful protections. Python transfer defect and arithmetic guard defect need correction. | Baseline; no new timings measured. | Starting evidence, not target. |
| Initial four-crate / generic-wrapper proposal as written | Fewer handwritten wrappers and generated declarations; useful relational rewrite goals. | Fingerprint fallback, claim transfer, column mapping, WorkTable and error ownership are incomplete/incorrect as described. | Its reduction estimates are hypotheses, not selection evidence. | **Do not implement verbatim.** |
| **Simpler viable alternative: consolidate within existing crates** | Shared native predicate compiler, exact contract handles, wrapper delegation, graph views and stable keys; keep engine services in catalog, reuse test library modules. | Can achieve most semantic fixes without a crate split. Retains catalog coupling for reusable execution services and awkward cross-test reuse. | Lowest structural churn; no generic platform or new crate is necessary for the first correctness fixes. | **Preferred for local mechanisms; sufficient if extraction has no actual independent consumers.** |
| **Recommended target: shared contracts plus focused engine/testkit extraction** | One contract lowering and proof boundary; native relational compiler stages; complete adapters; two new crates with concrete shared consumers; diagnostic leaf repurposed. | Preserves required specialized graph, memory-transfer and publication semantics explicitly. Remaining claims have concrete oracles. | Larger API reorganization than the simpler alternative, justified by execution-service consumers and repeated fixtures; benefits must be measured. | **Selected target.** Implement target interfaces directly, delete superseded paths in each completed slice. |
| Everything represented as generic SQL/UDF operations | Appears uniform, but graph algorithms, effect lifecycles, allocation ownership and provenance may move into opaque callbacks. | UDF volatility is not a durable transaction protocol, and SQL NULL is not automatically four-valued rule truth. | More custom infrastructure can result despite more native-looking syntax. | Reject as a universal rule; use native plans wherever semantics fit. |

The recommended design is not a compatibility migration. It can be executed in dependency
order while deleting each superseded implementation and updating all of its callers.
No old/new runtime selector, legacy object retention or prolonged dual-engine qualification
is required. A temporary test oracle may compare isolated implementations during a change;
it does not justify shipping both pathways afterward.

**Retain as narrow specialized code:** canonical semantic preimages; domain graph/quantity
algorithms; finite/domain/derivative kernels not supplied by native functions; the tested
allocation transfer/foreign-owner bridge; fixed-point epoch ownership; exact attempt,
publication and lease-aware retention coordination. The interfaces around those kernels
should remain native plans, Arrow batches, native reservations and Delta builders.

## 9. Verification and measurement plan

### Evidence actually produced in this review

| ID | Command / conditions | Result and evidence label | Practical limit |
|---|---|---|---|
| E1 | `just metadata-resolve`, offline, current workspace | **Interface-checked:** exact resolved pins/features/dependency graph captured in the manifest. Arrow pool and DataFusion Arrow-bridge features are off. | Resolution is not execution qualification. |
| E2 | Both skills' `ast-grep scan`, `--filter '^project-' --json=compact crates` | **Measured:** 84 DataFusion and 41 Delta candidate matches. Matches were triaged; no claim that these are failures. | Static candidate patterns miss some current trait hooks and include tests/builder-chain false positives. |
| P1 | Scratch Cargo binary, `cargo +1.98.1 run --offline --quiet --manifest-path <scratch>/Cargo.toml --target-dir <scratch>/target`; Arrow `59.3.0`, `arrow-array/pool`, `arrow-buffer/pool`, `arrow-data/force_validate` | **Tested:** an `ArrayData` primitive slice reports 4,000 backing-buffer bytes versus 4 slice bytes; an `Int32Array` slice already narrows its visible buffer to 4. A one-element List slice reports 4,012 slice-estimate bytes. A 4,000-byte shared allocation moves from pool A to B on claim and releases after the last owner. The copied P4 guard expression panics for `i64::MIN`, `-1`; `checked_rem` returns `None`. | An API/guard probe, not an invocation of the full P4 pass or workspace memory bridge. Initial compile attempt used private `arrow_buffer::pool`; corrected public path is `arrow_buffer::TrackingMemoryPool`. Final run exited 0. |
| P2 | `.venv/bin/python`, installed PyArrow `25.0.1`, import `python/pse/_transfer.py` directly; change only map-item metadata | **Tested:** native `Field.equals(..., check_metadata=True)` is false; current `compare_schemas` returns one `retained` observation. A Struct control reports its child mismatch. | Tests observation behavior in the installed Python library, not full native publication or FFI admission. |
| E3 | `just doctor`, ordinary startup mode, baseline zero | One environment-freshness failure; all other reported startup checks passed. | No environment synchronization, native extension rebuild or integration suite was run. |
| E4 | `just docs`; `git diff --check`; review-local evidence/reference checks | Documentation-only validation is recorded in the evidence manifest after execution. | A rendered book cannot establish proposal correctness. |

P1/P2 are characterization probes: their zero exit status means they recorded the expected
observations, **not** that the current defects are repaired. The review ran no workspace
test suite and provides no new simulator acceptance result. Probe source, Cargo manifest,
lockfile, exact outputs and reproduction instructions are retained in the evidence manifest.

### Required implementation oracles

| Target claim | Evidence today | Check before claiming completion |
|---|---|---|
| One contract drives every local/durable validation path | Implemented duplicate paths; native lowering interface checked | Cross-boundary property cases over every declared semantic field family, including null parents, tagged unions, dictionary domains and detailed error locations. |
| Interned declaration handles preserve exact authority | Proposed | Adversarial copied ID/fingerprint cases, changed transitive enum/extension definitions, equivalent independent registries and handle-owner misuse. Measure preparation and repeated borrow cost. |
| Generic adapters preserve native functionality | Interface-checked | Function/aggregate provided-method matrix; ordering, stats substitution, window/reversal, metadata, scalar/array paths and optimizer reconstruction. |
| Dependency precision is safe and bounded | Proposed | Requirement-only changes, negative/absent inputs, correlated subqueries, recursive scopes, unknown extensions and deeply shared producers; compare reused results to clean computation. |
| Native memory consolidation preserves limits and lifetime | P1 plus source inspection only | Pre-allocation refusal, atomic transfer, shared slices, FFI owners, concurrent claims, cancellation and last-owner release; compare native accounted peak and process RSS separately. |
| Compiler correspondence remains native and preserves process facts | Source-inspected opportunity | Relation/provenance equality for the affected P7/P8/P9 journeys, null/multiplicity/ordering cases; measure total planning calls, preparation time, executed scans and decoded rows. |
| Numerical staging keeps compact shared computations | Proposed | Diamond/guarded graph growth, sparse Jacobian agreement, no eager invalid branch, correct volatile-function refusal, preparation versus callback allocation/timing. |
| Arithmetic sharing preserves each semantic role | P1 guard reproduced; other semantics source inspected | Integer extrema, exact and nonintegral division, overflow, fold eligibility, physical domains and unit conversion bit patterns. Independently check derivatives; don't route the only test oracle through production. |
| Delta consolidation preserves effects and recovery | Interface-checked, not adversarially executed | Fault-injected member/publication commits, ambiguous cancellation, metrics-read failure, retry mismatch, active reader/CDF/attempt retention and cleanup. |
| Python native reflection preserves transfer diagnostics | P2 demonstrated defect | Nested Map/Union/ListView/dictionary/extension matrix, raw storage-only transfer, named reports, settings validation and exact ID codecs. |
| Smaller generated output improves the system | Hypothesis | Measure generator complexity, fresh/incremental compile time, monomorphized output, admission cost, borrowed-view allocation and actual relation consumer behavior. |
| Crate/test consolidation reduces coordinated edits | Proposed | Dependency DAG has no cycle or production testkit dependency; one representative new field/operation uses the shared path and requires no parallel policy definitions. |

During implementation, use focused unit and conformance checks while completing each
target slice and its deletions. Run broad integration journeys after the complete scoped
refactor and deletion work, consistent with the maintainer's development instruction.
Do not increase timeout thresholds merely to label an unmeasured regression acceptable.

## 10. Exceptions and unresolved decisions

These are technical proof obligations, not requests for policy permission. Existing
governance can be reconciled after selecting the design. No existing policy was used to
reject a capability or preserve an obsolete pathway.

| Decision / uncertainty | Recommended default | What would change the recommendation |
|---|---|---|
| Required semantic equivalence | Preserve explicit process meaning, type/quantity identity, guarded errors and publication behavior; don't require predecessor representation compatibility. | A separately chosen weaker numerical/identity contract, with its own functional justification. |
| Field predicate ownership | Shared non-Delta compiler and prepared predicates, using actual native preparation context. | A demonstrable native gap for a field family; add one explicit Arrow/UDF kernel, not another full validator. |
| Generic row codec | Shared native column primitives plus generated typed façades. | A pinned serde_arrow prototype that preserves the full contract and improves measured maintenance/allocation/compile cost. |
| Canonical sort | Keep Arrow `RowConverter`, consolidate surrounding framing. | A demonstrably simpler/faster native sort that preserves the chosen canonical equivalence; re-version identity deliberately if that equivalence is intentionally changed. |
| Arrow pool claims | Native reservations first; claims only with a proved transfer/lifetime bridge. | An upstream API or tested local bridge that consumes pre-reserved capacity without gaps, double charge or owner reassignment hazards. |
| Extension consolidation | Separate pure, barrier, shared-result and command families; generate mechanics only. | A concrete common contract across additional families, established by adversarial lifecycle tests. |
| Graph implementation | Indexed graph view plus shared typed edges, retaining detailed cycles and guards. | Petgraph removes more machinery while preserving ordering, cycle diagnostics and preparation size. No graph ban applies. |
| Numerical native CSE | Staged sharing with explicit evaluation regions. | A probe demonstrating bounded native preparation directly from the compact graph and equivalent guarded failures. |
| Delta crate extraction | Cohesive `pse-catalog::delta` module. | An independent consumer or build/feature boundary that actually benefits from an acyclic `pse-delta` crate. |
| Dormant solver/backend surface | Retain meaningful independently exercised kernels; remove abandoned alternatives and unused dependencies. Describe actual reachability. | A separate product requirement to connect the solver end to end. Do not add that functionality to this consolidation by implication. |
| Diagnostic vocabulary | Leaf `pse-diagnostics`, one class/code declaration, preserve concrete source errors. | A real dependency cycle after removing its unused high-level dependencies; resolve that actual boundary rather than adding a speculative codes platform. |

No historical object migration is required by this recommendation. Required final state:
all active callers use the selected target mechanism, superseded implementations are
deleted, and remaining specialized code has a concrete function that native primitives
do not replace on their own.

## 11. Decision and implementation changes

**Decision: Revise.** The requested native/consolidation direction is sound. The initial
review should be used as an opportunity inventory, with §1.2's corrections applied before
any execution plan is written. The independent recommendation is organized around
contracts and execution semantics rather than the original finding count or LOC targets.

The dependency order below is design guidance, not a newly authorized implementation plan.

| Order | Change and deletion target | Why now | Acceptance evidence |
|---|---|---|---|
| **1 — establish shared meaning** | Resolved contract descriptor/handle, semantic versus presentation metadata, native local predicate/obligation compiler. Delete `Cell` validation, duplicate local validators and complete-declaration strings once all their active consumers have replacements. | Simplifies every later provider, builder, compiler and persistence path. | R1/R2 authority and cross-boundary validity oracles. |
| **2 — repair confirmed boundary defects** | Checked P4 integer arithmetic; native nested Python transfer traversal; named Python reports. Centralize literal/hash kernels while preserving their separate equivalence policies. | Prevents unsafe semantics being embedded in new abstractions. | P1/P2 regression cases plus exact integer, metadata and float-policy cases. |
| **3 — expose and consolidate native execution** | Focused `pse-engine`, complete function/provider delegation, typed operation families, context-sensitive traversal and dependency transfer. Delete duplicate shells and plan helpers that have target consumers. | Makes subsequent algorithm rewrites smaller and makes native optimization effective. | Adapter/lifecycle/dependency matrix; actual metrics; dependency DAG. |
| **4 — remove relational row shuttling** | P7/P8/P9/template correspondence and provenance as native plans; stable support keys; shared materialization where required. Delete decoded inventory scans that have no specialized use. | Removes the highest-value bespoke relational work. | Same process facts/provenance under reorder/absence/duplicates; measured preparation/scans/allocations. |
| **5 — compact specialized computation** | Typed graph adjacency/edge views, no binding-edge graph clones, shared numerical lowering and derivative stages. Delete duplicate traversal/evaluation implementations only when their contracts are represented. | Addresses costs that a mere native wrapper does not solve. | Cycle/guard witnesses and compact-DAG growth; numerical oracle. |
| **6 — finish resource and Delta consolidation** | Native reservation ownership bridge, effective cache/resource policy, Delta operation context, shared durable contract lowering, bounded exact-version reader and lease-aware retention. Delete superseded reservation and operation plumbing after full replacement. | Builds on stable contracts and operation families; avoids mixing distinct lifetime protocols. | Memory ownership/refusal and interrupted publication/retention oracles. |
| **7 — consolidate supporting surfaces and close scope** | Repurpose diagnostic leaf, native JSON/schema renderers, typed configuration projections, `pse-testkit`, readable rule-source ownership and correctly keyed syntax caching; delete proven unused dependencies and abandoned entrypoints. | Removes remaining independently maintained facts without adding simulator scope. | Generated/consumer equivalence, error preservation, dependency/usage review; broad integration once the complete target and deletions have landed. |

**Expected benefit, still Proposed:** fewer places to define a semantic rule, fewer
row-to-column transitions, more optimizer-visible computation, bounded reuse of prepared
work, and less adapter code suppressing native capabilities. The success criterion is a
smaller set of independently maintained contracts and a clearer extension path, verified
through actual process-model behavior—not simply a smaller source tree.
