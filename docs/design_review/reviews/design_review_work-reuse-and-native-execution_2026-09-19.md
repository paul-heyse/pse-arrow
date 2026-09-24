---
title: Reuse established facts and eliminate unnecessary native execution work
date: 2026-09-19
status: proposed
scope: Existing native compiler, validation, execution, storage, and local assurance
evidence: Interface-checked; prior interrupted assessment results retained separately
---

# Design review: reuse established facts and eliminate unnecessary work

## 1. Decision and scope

**Decision: Revise around a retained model-level DataFusion context, native plan
composition, and reusable evidence.** Use `SessionContext` as the shared environment
for compatible model work, with one composed native capability assembly and shared
runtime. Carry checked data, bound plans, completed computations, and immutable
snapshot selections through the workflow instead of rebuilding them at each stage.

**Observable outcome:** an unchanged input can pass through multiple consumers
without repeated value admission, graph discovery, configuration assembly, or
producer execution. A changed input invalidates precisely the evidence that depends
on it. Only the outputs and obligations required by the operation are computed.

**Reviewer:** Codex. **Status:** recommendations are **Proposed**; inspected native
interfaces are **Interface-checked**. Existing mechanisms identified below are
**Implemented**. Prior test results are **Tested** only within their recorded scope.
No performance improvement from this proposal has been measured.

The SessionContext assessment is integrated into this recommendation, especially
§4.3, §4.6–§4.7, and §10. The target is selected on semantic correctness and removal
of unnecessary work, **without treating current rules or policies as design
constraints**. Conflicts are documented for the subsequent plan/decision step.
“Best performing” here denotes the strongest proposed architecture under these
contracts, not a benchmark-established universal optimum.

### Method and coverage

This is a conceptual and static review, following the user's revised direction.
The review reads actual callers and ownership boundaries, the design charter, and
the local DataFusion, Delta Lake, and datafusion-tracing skills. It does not assign
percentages of elapsed time to individual mechanisms, require profiling before
removing redundant work, or recommend increasing test timeouts.

Inspected scope includes native plan traversal, field admission, cache admission,
session binding, preparation, compiler construction, rule inputs and requirements,
Arrow ownership, Delta writes, fixture assembly, and local assessment tooling.
The stopped campaigns supply correctness counterexamples and an incomplete
acceptance baseline; they do not establish a complete performance profile.
No further test campaign or timing probe was run for this revised review.

The SessionContext follow-up inspected the exact local DataFusion source for
`SessionContext`, `SessionState`, `SessionStateBuilder`, `DataFrame`, and prepared
statements; Arrow `RecordBatch`; current PSE session construction and algorithm
execution; and the vendored Delta session and builder integration. `just doctor`
reported ready. This follow-up remains static and documentation-only.

The consumer pins are DataFusion **55.1.0**, Arrow **59.3.0**, object_store **0.13.2**,
and the vendored delta-rs capture **58f07cd62bfbce3649a7e1c87c696288068ae184** with
kernel **8ba063f8f84fec222000f66d40d70911d7c79675** (`Cargo.toml:84`, `:114`, `:126`,
`:316`). This is the unpublished Delta capture, not a published 0.32.x contract.

**Non-goals:** new simulator functionality, a new query engine, a parallel graph
authority, a generic proof language, a distributed cache service, compatibility
paths, cross-platform qualification, and wheel builds. Existing local assessment
scope remains complete; distinguishing work within that scope does not authorize
silently dropping checks.

### Baseline retained from the stopped assessment

Both owned campaigns were stopped; their reports remain under
`build/assessment/2026-09-19-local-full/` and
`build/plan10/acceptance-2026-09-19-03/`. See §9 for exact partial counts.
The newer native run spent 8.05 seconds compiling before test execution. Its long
runtime was not an uncached full rebuild. Workflow fixtures explicitly select one
worker and one target partition, and substantial test groups are serialized.
These are configuration facts, not a complete explanation of the slow workflows.

The design target below does not depend on attributing the observed delay. Removing
repeated work is justified by the work and ownership contracts themselves. No
specific speedup, memory bound, or completion time is promised.

## 2. Authority and lifecycle map

| Fact or artifact | Authority and identity | Retention boundary | Invalidating change |
|---|---|---|---|
| Relation and field meaning | Registry's resolved contract handle | Exact declaration owner/revision | Declaration, extension semantics, or predicate implementation changes |
| Model execution environment | Retained `SessionContext` with one compatible native capability assembly | Model/workspace lifetime, including compatible data revisions | Incompatible registry, implementation, semantic configuration, or security boundary |
| Query selection and attempt | Native `SessionState`/`TaskContext` plus existing invocation services | Exact selected providers and one attempt | A new query/attempt refreshes execution properties and attempt state, not the complete capability assembly |
| Local value validity | Existing `FieldCheckedBatch`, extended through preservation operations | Exact immutable batch/array owners and covered rows/fields | New values, changed interpretation, unsupported transformation, or untrusted ingress |
| Relational invariant outcome | Completed declared requirement over exact selected relations | Invariant implementation plus its complete dependency selection | Change to any input that can affect the result, including an empty or absent input |
| Intrinsic plan facts | Actual retained native plan/extension owners | Immutable producer revision | Rewritten expressions, children, implementations, or field meaning |
| Contextual plan admission | Native plan facts checked against actual bindings and lexical/effect scope | Bound assembly and relevant contextual dependencies | New binding, policy, outer reference, worktable round, or required effect |
| Prepared computation | Existing `PreparedComputation` and producer planning scope | Exact semantic assembly and compatible physical environment | Relevant plan/configuration/provider changes; execution-local state is separate |
| Completed pure result | Existing cache machinery extended to bounded model-context retention, or admitted durable artifact | Actual producer, inputs, implementation, and source selections; successful complete result only | Any semantic dependency changes or loss of valid ownership |
| Delta source selection | Snapshot/provider and exact table version | Retained snapshot with storage access | Selecting a different version or requiring current head |
| Authorization and execution resources | Current invocation and provider policies | Current attempt | New attempt, cancellation, quota pressure, policy revocation, or commit conflict |
| Diagnostic representation | Derived observation of the above | Requested observation scope | Observation policy or representation changes; it never becomes model authority |

An in-process owner identity is useful only while the actual immutable owner is
retained. A pointer to a mutable provider, a schema fingerprint, a table name, or
identical plan text does not establish unchanged values. Cross-process reuse must
continue through the existing durable artifact and source-selection contracts.

**Deliberately opaque behavior:** domain algorithms and numerical kernels may stay
ordinary specialized code behind declared operations. Their complete dependencies,
effects, output contracts, and completion must remain visible. Replacing a useful
algorithm with SQL solely for uniformity is not a performance requirement.

## 3. Semantic contracts and invariants

### Validate once for a specific meaning, input, and revision

The reusable fact is not a Boolean saying “validated.” It is evidence that a named
condition was established over particular immutable inputs under a particular
contract. Extend existing owners and admission records to retain that evidence;
do not build a second registry of truth.

| Check | Where it runs | What downstream consumers reuse | What must still run |
|---|---|---|---|
| Arrow layout and local values | Untrusted ingress or creation of changed values | Checked owner with the exact contract and covered values | Admission of new buffers/values; necessary safety checks at genuinely untrusted boundaries |
| Declared field meaning | Native binding and changed-node derivation | Actual admitted fields and producer facts | Changed expressions, schemas, or externally supplied factory results |
| Pure relational requirement | Once per invariant and complete input selection | Completed outcome and bounded structured findings | Same invariant over different inputs; checks affected by new rows or reference revisions |
| Source provenance | Source selection and derivation | Retained source witnesses and support mapping | Changed selections, newly introduced sources, or ambiguous bindings |
| Execution permission | Operation's current admission boundary | Compiled policy structure, where immutable | Current permission/effect decision when policy or attempt semantics require it |
| Resources and cancellation | Allocation, scheduling, stream polling, commit | Prepared structure and allocation ownership | Live quota acquisition, cancellation, completion, and release |
| Delta write integrity | Authoritative write/transaction boundary | Prepared input, mapping, and verified table contract | Native constraints, transaction conflict checks, and publication settlement |

**Important distinction:** `FieldCheckedBatch` currently establishes local fields
and values, not global uniqueness, foreign keys, completeness, or a successful
publication (`crates/pse-relations/src/columnar.rs:40`). Preserve that distinction.
The existing local validation context already caches prepared contracts
(`crates/pse-relations/src/validate/prepared.rs:80`) and separates preparation from
evaluation (`:180`). Each genuinely new batch still needs its applicable value
checks. Adding another prepared-predicate cache is not the recommendation.

### Preserve evidence through operations that justify it

| Operation | Evidence that can survive | Evidence that cannot be assumed |
|---|---|---|
| Clone the immutable checked owner | Existing local and completed relational facts over that same relation | Current permission or resource availability |
| Slice/filter a relation | Local per-value predicates and uniqueness of retained rows | Minimum cardinality, completeness, or constraints requiring removed rows |
| Pure projection/rename | Facts about unchanged retained fields with an exact field mapping | Constraints over dropped fields; arbitrary alias metadata does not certify meaning |
| Concatenate checked batches | Local field predicates under the same exact contract | Cross-batch uniqueness, ordering, or global cardinality; current `concat` already makes this distinction |
| Compute/cast/join/aggregate | Unchanged fields only where the operation contract proves preservation | Validity of new values, outer-join nullability, uniqueness, or arbitrary semantic metadata |
| Persist/reopen through Delta | Verified loss-aware layout and durable source identity | Arrow allocation identity, all Arrow metadata, or unsupported global constraints |

Start with these concrete preservation cases in existing modules. Unsupported
transformations establish the missing evidence at their output. Do not introduce a
general theorem prover to infer arbitrary implications between predicates.

**Completion:** an in-progress, cancelled, failed, or partly consumed computation
cannot mint a success record. A truncated findings sample can accompany a complete
violation count, but absence of sampled rows is not proof of validity.
Resource, cancellation, I/O, and ambiguous commit failures are not reusable semantic
invalidity. Retain them as attempt outcomes rather than caching them as facts about
the input.

**Force validation:** retain the required test mode and its negative controls.
Removing duplicate admission must preserve the required boundary check and explicit
adversarial verification; it must not simply disable validation to make tests fast.

### Cloning is not synonymous with copying payloads

Arrow 59.3 `RecordBatch` derives `Clone` over a `SchemaRef` and
`Vec<Arc<dyn Array>>`: cloning retains the same array buffers while cloning the
small container. DataFusion `SessionContext::clone` shares its
`Arc<RwLock<SessionState>>`; it does not clone all table data. `SessionState::clone`
does copy configuration/registry containers while retaining many `Arc`-owned
functions, providers, planners, and runtime resources. A `DataFrame` clone also
clones its boxed session state and logical plan, so it is not equivalent in cost
or lifetime to cloning the context handle.

The target preserves useful shallow clones and zero-copy Arrow ownership. It removes
unnecessary payload copies, repeated container/assembly construction, graph walks,
and re-admission. Different SessionContexts can also share the same Arrow buffers;
one context does not create zero-copy storage by itself. Filters, joins, casts,
decoding, and concatenation may still allocate new values. A deliberate compacting
copy can be appropriate when a small retained slice would otherwise pin a large
allocation. These are representation decisions, not reasons to copy every boundary.

## 4. Derivation and execution design

### 4.1 Carry checked data through internal boundaries

`EngineSession::with_checked_workspace` and `with_checked_role_inputs` already
retain checked values without repeating local predicates (`session/roles.rs:90`,
`:127`). Rule state uses them (`pse-rules/src/strata/native_state.rs:103`), and native
algorithm execution carries checked role inputs. Preserve these working paths.
The raw `with_workspace` route has test callers; its existence is not evidence that
production repeatedly copies workspace values.

The concrete gap is tuple transport: `Layout::pack` starts with checked outputs,
then returns a raw tuple batch; `Layout::member` reconstructs the member and invokes
`FieldCheckedBatch::admit` again (`pse-compiler/src/native/layout.rs:110`, `:146`).
Retain the exact member's field/value evidence with the completed tuple owner so
unpacking unchanged arrays can preserve it. Verify tuple layout, covered array
owners/slices, and contract identity. A changed, externally supplied, or rewritten
tuple must establish its missing evidence; tuple metadata alone is not a proof.

Extend the existing checked-value preservation operations to cover this boundary
and route their results into the existing checked binding APIs. Transfer/share the
actual allocation leases. Preserve raw admission for genuinely external values;
these are different trust boundaries, not legacy/new alternatives.

`NativeInput::validate` currently delegates to workspace value comparison, not a
second local predicate evaluator (`native_input.rs:183`; `engine_session.rs:403`).
Avoid potentially deep batch equality by carrying the actual bound checked owner
through the call. Independently constructed equal data may need real admission;
do not infer shared authority from equal bytes.

At a finite algorithm boundary, materialization can remain necessary. Reuse the
captured `RelationFacts` for consumers of the same producer within the invocation
instead of erasing it to a raw batch and re-admitting it. The existing capture
already validates streamed batches and concatenates checked inputs without another
local-value pass (`session/input.rs:67`; `columnar.rs:138`). Extend that mechanism.

### 4.2 Establish plan facts once; check context where context matters

Current traversal keys every visited state by `(NodeIdentity, Scope)`. `Scope`
contains the complete enclosing effect-owner chain, and each contract appends to it
(`session/traversal.rs:24`, `:118`, `:183`). A shared producer reached through distinct
parents can therefore be inspected repeatedly even when its intrinsic facts are
unchanged. Read-only visitors also use the retained traversal graph.

Separate two responsibilities within the existing native traversal/admission code:

1. **Intrinsic producer facts:** field derivation inputs, referenced providers,
   expression volatility, free outer/worktable references, required qualifications,
   and declared effects. Retain these once per immutable producer revision.
2. **Context checks:** satisfy those requirements against the actual enclosing
   bindings, lexical scope, effect permissions, and worktable generation. Retain a
   context-specific answer only for dependencies actually used by that producer.

Do not include an irrelevant full ancestor history in every intrinsic-fact key.
Do not eliminate scope from genuinely correlated queries, recursive worktables, or
effect authorization. A contracted sibling must never authorize another sibling.
Only rewrites needing reconstructed nodes should allocate a full rewrite graph;
simple classification should consume retained facts or use a lightweight visitor.

The work contract is “inspect an unchanged closed producer once per relevant
assembly, then check its contextual obligations,” not a universal claim that every
correlated plan has linear complexity. Keep all derived records bounded and tied to
owner lifetimes. They are an index over the native plan, not a second graph model.

### 4.3 Make SessionContext the retained model execution environment

#### What the pinned native implementation actually provides

These are **Interface-checked** against DataFusion 55.1 source, not inferred from
the term “session.” Upstream paths below are within that pinned crate.

| Capability | Exact behavior | Design consequence |
|---|---|---|
| Shared context | `execution/context/mod.rs:293`: cloned contexts share `Arc<RwLock<SessionState>>` | Reuse context handles for a compatible model instead of constructing independent feature assemblies |
| Query state | `SessionContext::state`, `:2054`, clones state and refreshes query execution start time | One retained context still has query-specific state; do not freeze time-sensitive semantics across independent requests |
| Shared runtime | Context docs and `SessionState.runtime_env` retain `Arc<RuntimeEnv>` | Pool, spill manager, stores, and metadata caches can be shared across queries and contexts; context count is not pool count |
| Native optimization | `SessionState::create_physical_plan`, `execution/session_state.rs:777`, optimizes the supplied logical plan before invoking the query planner | Optimization applies to the plan submitted, not automatically to all separately executed queries in the session |
| DataFrame terminal work | `dataframe/mod.rs:1480`: `collect` creates a physical plan and executes it | Keeping a context or cloning a DataFrame does not itself memoize planning or execution |
| Explicit result reuse | `DataFrame::cache`, `:2412`, delegates to a configured `CacheFactory`; otherwise materializes a `MemTable` | Retain and use PSE's actual native cache seam and its completion/lifetime semantics |
| Named prepared SQL | `SessionState.prepared_plans` and `SessionContext::execute_prepared`, `execution/context/mod.rs:1553`, retain a logical plan and substitute parameters | Useful for repeated SQL binding; not a general physical-plan or result cache and not a replacement for native compiler expressions |
| Prepared SQL time semantics | The `Prepare` handler, `execution/context/mod.rs:727`, optimizes without a query execution start time | Reuse the native distinction between reusable structure and query-time evaluation; do not retain a previously folded `now()` value as a new query's answer |
| Builder reconstruction | `SessionStateBuilder::new_from_existing`, `execution/session_state.rs:1137`, moves/repackages registries and unsets the session ID; `build`, `:1639`, initializes an empty prepared-plan map | It is not a cheap context clone. Rebuilding can also discard named prepared statements; do not assume they survive every PSE state derivation |
| Catalog sharing | `SessionState.catalog_list` is an `Arc<dyn CatalogProviderList>` | A state clone is not a deep immutable snapshot of mutable catalogs or providers; exact source selection must be explicit |
| SQL effects | `sql_with_options`, `execution/context/mod.rs:642`, plans then invokes `execute_logical_plan`; its DDL branches can change catalogs | Route effectful SQL through the owned command/publication boundary; a context API call is not necessarily lazy read-only plan construction |

The central opportunity is to share the environment **and** expose more of the
work to the native planner. Sharing a context while retaining separate
collect/register/query cycles leaves most of their preparation and materialization
cost intact. Conversely, a shared environment does not require a single enormous
plan covering unrelated operations or all future model revisions.

#### Where the present implementation stops short

The current system already retains a SessionContext in `EngineSession`, shares
runtime resources, uses shallow Arrow ownership, composes native extension planners,
and has grouped preparation/execution. It does not start a default DataFusion
engine for every query. The remaining reconstruction is concrete:

| Current path | Repeated work or lost opportunity | Target change |
|---|---|---|
| `NativeExecutionContext::candidate_roles`, `session/execution.rs:222` | Builds a temporary `EngineFactory` and rule inventory for a nested algorithm | Pass the retained model assembly plus an exact private role binding scope |
| `engine_session::build`, `:89` | Replaces the checked-value UDF, rebuilds extension registry/function inventory, and creates a new context | Construct these once per compatible semantic assembly; only changed capabilities get a new generation |
| `EngineSession::bound_state`, `:540`, and `reuse::Witness::matches`, `reuse.rs:98` | Rebuild catalogs/effective settings/inspection binding to create or compare state | Retain effective binding views and assembly facts; compare their actual owners |
| `execution_state_with_caches`, `execution.rs:155` | Rebuilds state to install fresh invocation services | Keep a narrow query/attempt derivation; remove repeated semantic registration, hashing, and default assembly |
| `execution_scope`, `resources.rs:93` | With a quota, creates a runtime facade and context around shared parent resources | Bind an attempt quota once; preserve the shared parent pool/store/cache owners rather than repeating this per nested query |
| `AlgorithmBody::run`, `pse-compiler/src/native/execution.rs:113` | Captures child data, binds roles, then invokes algorithms that prepare more queries | Lower relational work into visible native plans before execution; retain only necessary finite algorithm boundaries (§4.6) |

#### Recommended ownership and execution hierarchy

1. **Deployment resources:** one actual shared `RuntimeEnv` for compatible local
   workloads, plus the existing shared executor/thread budget. DataFusion context
   sharing does not itself configure or size the executor. Different credentials
   or storage/resource domains may legitimately require another runtime.
2. **Model context:** one retained `SessionContext` per compatible model/workspace
   capability assembly. Register functions, extension types, catalog factories,
   format handlers, analyzer/optimizer rules, PSE/Delta extension planning, statistics
   services, and cache factory once. Keep it across compatible source-data revisions.
   A change of data revision alone must not rebuild these registries.
3. **Revision and role selection:** retain immutable providers under explicit
   revision identities and construct a small selected catalog/role view for each
   workflow. Reuse provider `Arc`s and checked Arrow owners. Bind “current” to an
   exact selection once at workflow entry; it must not change halfway through a
   query. Do not replace a shared unversioned `input` table while another query uses
   it. Retain only referenced or budget-admitted revisions, not an unlimited history.
4. **Prepared workflow:** compose the requested value and obligation plans using
   this assembly; retain optimized logical work and explicit common producers.
   Use the existing preparation group and native cache seam for multiple consumers.
   Refresh only dependencies changed by a new request/revision.
5. **Attempt state:** derive native `SessionState` and `TaskContext` with fresh
   execution properties, cancellation, current resource admission, command identity,
   and settlement. Related nested work shares its workflow's intended query-time
   semantics; independent requests get fresh time/volatile evaluation. A required
   native state clone is acceptable; a new feature assembly is unnecessary.

The session owns configuration and access to model resources, not the authority to
mutate immutable facts. Different permission or semantic settings may require a
separate compatible assembly/view. There is no requirement to force every model,
user, or query into one global mutable namespace.

Use native catalog/provider contracts for selected views and native state/config
extensions for attempt services. A small PSE owner may retain the model context and
existing caches; it should not duplicate DataFusion's function/catalog registries.
The immutable assembly must not contain an ancestor query's cancellation, physical
execution, or settlement state. Avoid reference cycles from state extensions back
to the state that owns them.

#### Separate reuse by lifetime

Retain immutable registry/function/field/plan facts and eligible completed pure
results for the bounded model-context lifetime. Keep round-dependent worktable
results in their epoch; keep mutable physical operator state and effects in their
attempt. Promote only completed, immutable results into longer reuse. Preserve
their allocation leases, exact input selections, and resource charges; bound
retention and evict unused entries. Do not promote failed/in-progress streams,
effectful command completions as replay authorization, or ambient mutable reads.

Use the existing cache implementation with explicit lifetime ownership, not a new
parallel global cache. Pure source-stable results may survive a new invocation;
permissions, quota admission, cancellation, and commit checks remain current.
Prepared physical plans are retained only where their reset/reentrancy and dynamic
filter contracts permit it. Otherwise reuse the logical preparation and create
fresh physical operators. Context reuse alone proves neither safety property.

For query-time-dependent expressions, refreshing TaskContext is insufficient if an
earlier optimization already folded the value into a literal. Retain reusable
pre-evaluation structure and redo the relevant binding/folding for the new query,
or treat time as an explicit captured semantic input when that is the operation's
contract. Capture volatility and ambient dependencies before optimization removes
their syntactic evidence. No blanket optimized-plan cache should freeze them.

SessionContext's SQL/DataFrame APIs are useful entry points, not reasons to bypass
the operation contract. Pure query composition may use them directly with the
selected state. DDL/DML/publication must retain explicit effect admission and
settlement; parsing a command and executing it are separate decisions. This keeps
the full native feature set eligible while preserving the intended effects.

Delta participates in this same environment. Keep the composed
`UnifiedPlanner`/Delta extension planner and pass the concrete selected
`SessionState` to Delta builders. `DeltaOperationContext` already does this with
`RequireSessionState` (`pse-catalog/src/delta/operation.rs:97`). Do not replace it with
an unrelated default `DeltaSessionContext`, or replace the composed planner with
Delta's default planner and lose PSE extensions.

### 4.4 Reuse requirement outcomes across the workflow

The compiler installs preconditions and postconditions and places cache boundaries
around algorithm tuples (`pse-compiler/src/native.rs:136`, `:154`, `:176`). Requirement
compilation already batches declared queries and supports required/affected scopes
(`pse-rules/src/invariants/program.rs:32`). Preserve these strengths.

Make each requirement's identity and exact relation selections available to the
existing contract/completion mechanism. If stage A establishes a postcondition and
stage B requires the same invariant over the same unchanged inputs, B consumes that
completed evidence. It does not issue the same query again. A matching rule name
over a changed output is a different obligation.

Compile predicates once per contract/schema/function assembly. When several checks
consume the same expensive relation, prepare them together and share the producer
using the existing native cache/planning scope. A SQL `UNION` or a view by itself
does not promise one execution of its shared input.

Use existing declared dependencies and affected scopes for invalidation. Preserve
dependencies on empty tables, missing matches, and referenced relation revisions:
they matter for anti-joins, foreign keys, and absence tests. Begin at whole immutable
relation granularity; introduce row/partition-level reuse only where the current
dependency representation establishes its correctness.

### 4.5 Perform the smallest calculation that answers the question

`NativeConstruction::require` executes and collects every violating row only to ask
whether any row exists (`native_construction.rs:257`). Source-support construction
can execute that check before `NativeInput::build_inner` executes its producing plan
again (`native_construction.rs:419`; `native_input.rs:119`).

Use a native Boolean/count reduction when the contract needs only validity or a
count. Retain bounded examples only when a diagnostic asks for them. Use an
existence query or `LIMIT 1` only where first failure is a sufficient result and
early termination cannot skip required effects, validation, or mandated diagnostics.
Success still requires consuming the complete relevant input.

Where output values are needed too, validate and derive them from one retained
computation. Share the actual upstream producer among values, findings, support,
and provenance; avoid independently preparing and executing their common prefix.
`retain_with_count` already derives cardinality from its actual completion; do not
replace that good path with a separate counting query.

Keep native expressions and Arrow batch kernels for column operations. Reuse one
prepared physical expression or Arrow filter predicate across columns/batches only
when its schema and row domain match. Use semi/anti joins for membership questions
instead of retaining an unnecessarily wide joined relation where semantics permit.
Project only the keys/fields needed by validation and provenance. Avoid additional
sort/distinct/concatenation steps when an existing contract already supplies the
required property; do not invent uniqueness or ordering to remove an operator.

### 4.6 Let native optimization see useful structure

**Compose before executing.** The current generic algorithm body captures its
physical inputs and constructs a child session before invoking the algorithm
(`pse-compiler/src/native/execution.rs:113`, `:187`). DataFusion can optimize native
queries that the body submits, but its outer planner cannot fuse relational work
that has not been constructed until that body executes. A common SessionContext
alone does not remove this optimization boundary.

Classify each existing algorithm segment by what it actually needs:

- Lower pure relational transformations and checks during workflow composition
  into native `LogicalPlan`/`Expr` structures over actual providers. Compose adjacent
  segments so projection/filter pushdown, join planning, native expression
  simplification, and common-subexpression rules can see the relevant work together.
- Keep specialized numerical/graph algorithms behind native extension operators
  where finite data or a distinctive algorithm is required. Their boundaries expose
  actual schemas, dependency fields, effects, and physical properties. A compiler
  pass name by itself is not a reason to collect all inputs and rebuild a session.
- Keep fixed-point state and effect/publication barriers where semantics require
  them. Bind changing round inputs to the retained native preparation/reset contract;
  do not rebuild the feature environment every round.
- For multiple result schemas, retain native output plans/providers and one explicit
  shared producer/completion scope. Do not pack all outputs into a transient
  whole-result tuple just to cross an otherwise relational stage boundary. Tuple
  transport can remain at a genuine heterogeneous finite algorithm boundary.

This is a direct pivot of existing implementation, not additional simulator
functionality. Retain bounded plan segments at real semantic or reuse boundaries;
do not make every output share one unbounded plan. Registering a view preserves a
query definition, not a materialized result or execution-once guarantee. Explicit
producer sharing is still necessary where the same subtree has multiple consumers.

The engine already uses DataFusion analysis/optimization and directly invokes the
query planner for physical planning (`preparation.rs:689`). There is no proposal to
add that optimization a second time.

Align custom field admission with native lambda binding/coercion before schema
reconstruction; retain original semantic intent for authority checks. Derive fields
for changed nodes and their affected parents, retaining unchanged child evidence.
External factories or caller-provided nodes still need admission of what they
actually return (`session/cache.rs:63`), not a blanket “trusted cache” exemption.

`ExecutionContract` currently blocks predicate pushdown for all input columns and
retains all requirement fields (`session/contract.rs:124`). Reusable execution also
disables dynamic filter pushdown families (`preparation.rs:658`). These are
correctness protections, not settings to flip indiscriminately.

Make completed obligations consumable evidence so a satisfied validation wrapper
can stop imposing unnecessary barriers on an immutable value path. Keep pending
requirements as explicit native children until completion. Audit each extension's
column dependencies, schema, ordering, equivalence, partitioning, boundedness, and
filter support against its real behavior. Re-enable an optimization only after its
effects are contained within the correct invocation and cannot poison a reused
producer or hide a failing requirement.

Provider projection/filter/limit pushdown should flow to native DataFusion/Delta
providers. `Exact` means exact enforcement; `Inexact` must retain a residual filter.
Do not push a limit through an inexact filter or a validation boundary that needs
the full input. Narrow lexical projections also prevent old witness columns from
leaking into later joins and becoming ambiguous.

### 4.7 Materialize at deliberate reuse or algorithm boundaries

Use existing `execute_stream` and `OwnedComputationStream` for consumers that can
process batches incrementally. Keep `collect` for consumers that require all values,
finite numerical/graph algorithms, deliberate shared result retention, or an API
that promises a complete batch. Do not concatenate solely to cross an internal API
that could accept a stream or checked chunks.

The engine's `prepare_group_member` and `execute_group` already share planning and
invocation cache scopes (`preparation.rs:143`, `:475`). Use them for sibling outputs
and obligations rather than opening independent scopes. Deduplicate concurrent
requests for the same eligible producer within that scope; publish its reusable
completion only after success. Preserve per-consumer cancellation and ownership.

Extend this reuse to the model context for eligible immutable completed producers,
with exact dependency selection and bounded retention as described in §4.3. A
second invocation can consume the retained Arrow result without rerunning its
producer. Each consumer still has its own current attempt and effect admission.
The shared producer's lifetime must not be cancelled merely because the first
consumer drops; cancellation of all consumers and context shutdown must still
release work and resources. Do not retain the originating query's mutable state
inside the longer-lived result entry.

Native memory-table providers already share Arrow payloads. PSE's
`ImmutableTable` delegates execution to `MemorySourceConfig`
(`session/materialized.rs:45`); retaining its small immutable facade is compatible
with native execution and zero-copy input sharing. Replacing it with unrestricted
`MemTable` mutation is not required to obtain that benefit. Prefer the smallest
provider implementation that preserves the selected immutable-data contract.

Snapshot-bound reference material such as `PhysicalInventory::load`
(`pse-compiler/src/native/execution.rs:195`) should be prepared once for the selected
physical-data revision and semantic assembly, then shared by algorithms that need
it. Unrelated algorithms should not load it. Keep attempt-specific mutable numerical
workspaces separate.

Streaming controls terminal retention, not blocking join/sort state, all process
RSS, or side-effect rollback. Use participating native memory pools and spill paths
and preserve the existing lease ownership contract. Resolve the observed retained
buffer extent discrepancy before relying on larger cache retention (§7, F02).

### 4.8 Use Delta and native storage caches at their proper lifetimes

Retain immutable Delta providers for pinned snapshots; reopen/refresh and rebind
when current head or a changed version is requested. Share store registration and
native metadata caches in the existing runtime. Cache parsed snapshots, verified
table contracts, schemas, and reference selections against their actual versions.
Do not reopen unchanged tables to rediscover these facts at each consumer.

Current native metadata/cache services already exist. Configure file statistics,
listing, and predicate caches only for eligible scans with defined freshness and
working-set budgets. A listing cache is not a substitute for Delta log state.
Do not create a bespoke cache parallel to DataFusion's `CacheManager` or the
existing Delta cache service.

Delta writes already pass a physical input through `PhysicalInput::storage` into a
logical input plan (`pse-catalog/src/delta/write.rs:176`). Preserve that direct path.
Do not collect/rebuild the input merely to invoke a different write API. Keep native
Delta constraints and transaction validation at the write boundary. Remove duplicate
PSE pre-scans only when that boundary enforces the same requirement on the same
representation and provides the required failure/publication semantics.

Use existing revision dependencies first for invalidation. CDF is a conditional
source of changed keys, not a universal replacement: the captured pin has feature
restrictions, historical availability limits, and bounded-version semantics. It
does not make uniqueness or referential checks local automatically. Checkpoint only
changes actually processed. Compaction/checkpoint maintenance belongs to explicit
storage operations when file/log structure warrants it, not every model operation.

### 4.9 Make observation and concurrency match the task

Use `Contract` observation for routine execution assurance. Request complete plan
rendering, rule-by-rule traces, and row previews explicitly for their corresponding
questions. Keep phase/completion outcomes and typed obligation evidence available
without serializing the whole expanded graph and schema for every operation.
Reuse retained plan facts for inspection instead of another discovery walk.

The test helper currently forces `Diagnostic` (`pse-testkit/src/execution.rs:31`),
while other fixtures default to observation off. One functional test hits the 1 MiB
plan capture refusal. Centralize explicit policy selection in the shared fixture
assembly; full graph capture should not be incidental work for a value assertion.
Do not label partial observation complete or silently suppress requested evidence.

Separate deterministic single-worker unit fixtures from throughput-oriented workflow
execution. Choose runtime workers, partitions, test concurrency, and memory/spill
budgets together through existing resource policy. A 32-thread machine should not
be constrained to one worker for every general workflow, nor should every concurrent
test start 32 workers independently. Serialize tests only for real shared state or
resource constraints; preserve solver and numerical ordering requirements.

### 4.10 Native capabilities retained rather than reimplemented

| Need | Existing/native mechanism | Precise use in the target |
|---|---|---|
| Shared model environment | `SessionContext`, `SessionState`, `TaskContext`, `RuntimeEnv` | Retain compatible model capabilities; select immutable providers per workflow and fresh state per attempt |
| Checked Arrow values | `FieldCheckedBatch`, resolved contract handles, Arrow kernels | Carry evidence and leases through internal APIs; validate new values at their boundary |
| Expression preparation | Native analyzer, lambda resolution, `ExprSimplifier`, physical expressions | One correctly ordered preparation per semantic input; reuse prepared expressions |
| Shared plan preparation/results | `CacheFactory`, existing PSE producer scope and `CacheStore`; named prepared statements for repeated SQL where suitable | Broaden participation and eligible model-context retention; distinguish syntax/binding, logical, physical, and result reuse |
| Native planning knowledge | Catalog/provider constraints, ordering, partitioning and statistics; `SessionStateBuilder::with_statistics_registry` | Retain facts from exact admitted sources/completions; expose truthful properties to native optimizers rather than rediscovering or fabricating them |
| Data-plane reduction | Native projections, aggregates, semi/anti joins, stream APIs | Ask only for required values/counts/findings and share their common producer |
| Storage | Delta providers, native scan pruning, `CacheManager`, existing Delta services | Snapshot-aware reuse and direct plan-to-write execution |
| Assurance | Existing contract observations, datafusion-tracing phases/operators/metrics | Record proof completion/reuse/invalidation and required execution without mandatory full rendering |

**Pin-specific correction:** the skill's compact cache brief describes the default
materializing behavior. DataFusion 55.1.0 `DataFrame::cache` first delegates to a
registered `CacheFactory`; it materializes a `MemTable` only without one (upstream
`datafusion/src/dataframe/mod.rs`, `DataFrame::cache`). PSE already implements that
extension seam. Replacing its call with `.cache()` alone does not remove PSE's
admission work or establish a different reuse contract.

Reference routes used: local skill briefs `df.expressions`, `df.consume`,
`df.storage-reuse`, `df.pushdown`, `arrow.filter-reuse`, `delta.read`, `delta.session`,
`delta.schema`, `delta.cdf`, `delta.replay`, `tracing.rules.phase`, and
`tracing.preview`. Their source-backed contracts and bounded probe findings are
inputs to this review, not new PSE acceptance evidence.

The SessionContext addition also uses the skill's `sessions-and-runtime` topic and
the exact upstream sources cited in §4.3. Arrow clone behavior is from
`arrow-array-59.3.0/src/record_batch.rs:225`; DataFrame ownership is from
`datafusion-55.1.0/src/dataframe/mod.rs:228`. A SessionContext UUID is diagnostic
identity, not a complete semantic cache key.

## 5. Representative journeys

### Ordinary extension: one additional relational invariant

Declare its meaning and input relations in the existing registry. Bind its query
through the existing requirement compiler. Prepare once against the actual bound
assembly; execute once per relevant immutable input selection. Multiple consumers
use the same completed result. Adding the invariant does not add another catalog
builder, session factory, graph walk, or hand-written cache key.

### Meaningful change: one source relation changes

Keep the same compatible model SessionContext and native function/planner assembly.
Select the new revision's provider without replacing providers already retained by
running workflows. A query already bound to the previous revision completes against
that revision. New binding views contain actual provider owners, not lookups against
a mutable “latest” table during execution.

Retain unchanged checked owners, producer facts, reference inventory, and unaffected
requirement outcomes. Rebind the changed source and invalidate its dependency
closure, including negative/absence dependencies. Recompute affected plans and
obligations. A different registry/predicate implementation invalidates the matching
validation evidence even when Arrow datatypes and displayed table names agree.

### Boundary: an admitted native result becomes a rule input

Transfer the actual checked owner and lease into the private workspace. Verify
contract compatibility and ownership without copying and revalidating its values.
A raw external batch takes the raw admission route. A new computed column establishes
its own missing local evidence. A persisted/reopened result uses the existing Delta
layout and durable admission path; it does not pretend to retain an in-memory proof.

### Interruption or failure

Two consumers may await one pure producer. No reusable success becomes visible
until its stream and required checks complete. Cancellation or late stream failure
leaves a failed/incomplete attempt, not a successful validation cache entry.
Effectful commands retain single-attempt execution and explicit settlement. After
an ambiguous Delta write failure, inspect committed state through the existing
recovery protocol before retry; the captured Delta transaction marker alone does
not suppress sequential append replay.

## 6. Acceptance gates

These verdicts concern the inspected implementation and the proposed change's
unclosed obligations. They are independent; scoped passes are not whole-system
acceptance.

| Gate | Verdict | Evidence or scope rationale | Required action |
|---|---|---|---|
| G1 — Authority | Pass, inspected scope | Registry contracts, actual native owners, and Delta source selections remain authorities; proposed evidence is derived and owner-bound | Keep evidence non-forgeable and remove duplicate internal admission routes, not raw ingress |
| G2 — Semantic fidelity | Fail | Captured nested-lambda preparation and qualified-witness ambiguity prevent supported native composition; retained allocation extent also disagrees across an ownership boundary | Correct binding/field phase order, lexical projections, and owner-aware extents before relying on broader reuse |
| G3 — Validity | Unresolved | Local admission exists and hostile-input controls pass, but the invariant fixture loop stops within its first failed case and full workflow qualification is incomplete | Cover preservation, changed-input invalidation, and every invariant case independently |
| G4 — Hidden behavior | Pass, inspected scope | Effects remain explicit contract children; diagnostic overflow is an explicit `ConfigInvalid`, not hidden model mutation | Keep pending checks and effects observable when removing work; never treat cached facts as current authorization |
| G5 — Consistency and recovery | Pass, bounded existing evidence | `publication_each_object::every_actual_delta_write_boundary_preserves_a_complete_publication` passed in both stopped native runs | Retain publication protocol; qualify new concurrent cache completion and cancellation paths before acceptance |
| G6 — Transformation and reuse | Unresolved | Existing admission/owner checks are substantive, but retained model contexts, cross-attempt pure-result reuse, visible relational composition, and evidence propagation are not implemented or tested | Establish transfer/invalidation, snapshot selection, query-time, cancellation, and contextual controls in §9 |
| G7 — Truthful capability claims | Unresolved | Reports explicitly remain partial; API-doc lint is a stub and the original unsafe audit recipe masked a tool failure | Preserve incomplete/unsupported states and repair report scope before a full acceptance claim |

## 7. Principle findings

| Finding | Principle verdicts | Concrete evidence or gap | Consequence | Proposed correction | Verification |
|---|---|---|---|---|---|
| **F01. Native binding and field reconstruction do not share one valid phase order and lexical boundary.** | DM-24 **Violated**; DM-09 **Violated** for the observed ambiguous binding | `session/admission.rs:430` reconstructs schema before local expression binding; `native_construction.rs:384` creates witness aliases while retaining a growing joined scope; captured lambda and witness failures | A supported nested expression or composed source mapping cannot bind; downstream work fails after prior construction | Use native binding/coercion before field reconstruction; project/capture lexical boundaries narrowly | Focused nested-lambda and repeated witness composition controls, including hostile alias metadata |
| **F02. Buffer extent inspection loses information retained by the allocation owner.** | DM-42 **Violated** for the inspected extent representation | `owned_buffer.rs:214` wraps a leased owner; `:552` measures exposed buffer capacity rather than consulting that owner; observed 2624 versus 408 mismatch | Budget/preflight consumers can disagree about retained storage; exact impact on each quota path remains unresolved | Make the existing owner inventory authoritative for retained allocation extents; preserve fallible reservation | Shared/sliced/wrapped buffer extent, detach/drop, and reservation conservation unit tests; do not weaken equality |
| **F03. Tuple transport discards reusable checked-value evidence.** | DM-26 **Violated** | `native/layout.rs:146` packs checked outputs into a raw tuple; `:110` unpacks and calls `FieldCheckedBatch::admit`; checked workspace APIs already exist and are used | An unchanged tuple member receives local value admission again; separate workspace equality may also revisit values | Remove tuple transport at relational-only boundaries; preserve member evidence where a finite algorithm still needs it; use existing checked binding APIs | No second local predicate evaluation for unchanged retained members; rewritten tuple, foreign owner, and changed field controls remain |
| **F04. Intrinsic plan facts are repeatedly discovered under ancestor-sensitive traversal keys.** | DM-26 **Violated**; DM-34 **Unresolved** for the proposed factoring | `traversal.rs:118` keys by owner plus full scope; `:191` appends ancestor effects; freshness performs another traversal | Shared producers can expand into many inspection states unrelated to distinct intrinsic computations | Retain producer facts once; validate minimal relevant context separately; lightweight read-only inspection | Distinct-parent diamond graph operation counts and negative lexical/effect controls |
| **F05. Nested operations reconstruct session capabilities and reuse admission reconstructs the assembly it compares.** | DM-26 **Violated** | `execution.rs:222` constructs a factory for child roles; `engine_session.rs:89` rebuilds registries/inventories/context; `:540` and `reuse.rs:98` rebuild bound state/settings | Avoiding repeated execution still pays repeated session assembly; rebuilding native state can discard prepared SQL registrations | Retain the model SessionContext/capability assembly, selected catalog views, and inventories; derive only query-specific state | One capability assembly across compatible data revisions; changed semantics invalidate; named prepared SQL lifetime is explicit |
| **F06. Reusable requirement completion is not carried across all stage boundaries.** | DM-32 **Unresolved** for broader reuse; DM-33 **Unresolved** | Compiler pre/post wrappers and repeated native construction requirements have no general demonstrated cross-stage completed-outcome reuse | The same invariant can be issued again against the same inputs; skipping by name would be unsafe | Consume completed requirement evidence keyed to complete semantic dependencies within existing contracts | Same invariant/same inputs executes once; changed referenced or empty input executes again |
| **F07. Some consumers materialize substantially more than their answer requires.** | DM-38 **Violated** in `require`; DM-26 **Violated** for repeated shared prefixes | `native_construction.rs:257` collects all violations to test nonemptiness; `native_input.rs:119` separately executes output production | Extra result retention, preparation, and producer execution | Native reductions/existence queries where valid; bounded findings; shared completion for values and diagnostics | Exact results plus producer invocation counts; late-error and required-child controls |
| **F08. Broad barriers preserve safety but constrain native optimization after evidence is available.** | DM-38 **Unresolved** for the proposed narrowing; DM-24 preservation is mandatory | `contract.rs:124` retains all requirement fields and blocks predicate pushdown; `preparation.rs:658` disables dynamic filters for reusable execution | A completed obligation can continue obstructing useful projection/filtering; changing flags alone risks skipped checks or cache contamination | Consume settled obligations; expose accurate native properties; keep pending effects/requirements protected | Optimized-plan assertions plus filtered-away failure and sibling consumer controls |
| **F09. Observation defaults add work unrelated to the requested assurance.** | DM-19 **Unresolved** for fixture selection; DM-38 **Violated** | Shared test helper forces `Diagnostic`; full graph includes schemas; observed 1 MiB capture refusal; another test assumes text despite observation off | Functional assertions pay for plan rendering or fail due to unrelated observation capacity | Explicit shared fixture policy; Contract by default for contract assurance, Diagnostic when requested | Value test without full rendering; separate capture completeness/cap controls |
| **F10. Runtime policy hard-codes single-worker general workflows.** | DM-38 **Unresolved**; DM-35 **Unresolved** for revised concurrency | `tests/support/workflow_runtime.rs:30`; `.config/nextest.toml` serial/engineering groups | Available hardware cannot be used by those workflows; unconstrained parallel replacement could oversubscribe resources | Separate deterministic fixtures from bounded workflow concurrency through existing resource policy | Worker/partition policy units and dependency/ownership tests; later one consolidated workflow qualification |
| **F11. Diagnostic assertions and cause traversal are inconsistent.** | DM-47 **Unresolved** | Eleven display-code failures; three rule cause traversal failures; one numerical resource classification failure; transparent `Semantic` wrapper and forwarded diagnostic source | Equivalent code formatting can fail tests, while nested typed leaves/classification may be missed | One typed diagnostic projection and assertion path; preserve all original leaves and operational classes | Wrapper/collection matrix and allocation failures at different expression depths |
| **F12. Fixture construction and assessment granularity obscure complete results.** | DM-53 **Unresolved**; DM-59 **Violated** for the original unsafe-audit success status | Tagged enum fixture generation bypasses declared enum literal; one registry test aborts its remaining rows; old geiger recipe used `|| true` | More native behavior remains untested than aggregate test counts suggest; a tool refusal looked successful | Contract-driven fixture construction, independent per-case outcomes, strict tool/report status | Every invariant case has an outcome; runner preserves failures and interruptions; unsupported tool invocation cannot pass |
| **F13. Runtime-created relational subqueries hide optimization opportunities behind generic algorithm boundaries.** | DM-18 **Violated** for relational structure hidden from the outer planner; DM-38 **Unresolved** for the best segment boundaries | `pse-compiler/src/native/execution.rs:113` captures children; `:187` creates role session; subsequent algorithm invocation builds/executes more queries | A shared SessionContext alone cannot push downstream demand through work that exists only inside the executing body; stage capture/tuple boundaries persist | Lower relational segments into native plans during composition; keep finite specialized algorithms and real completion/effect barriers explicit | A representative relational chain exposes native operators before execution; no intermediate collection/session assembly; identical values, checks, and provenance |

**Applicability:** groups 1–3 apply to evidence authority, validity, identity, and
revision boundaries; 4–7 to composition, preparation, effects, dependencies, and
reuse; 8–10 to work elimination, provider boundaries, resource ownership, and
diagnostics; 11–12 to assurance coverage and avoiding another framework. No schema
migration or new domain modeling is proposed, so DM-51 and unrelated domain
precision/approximation decisions are outside the change. Existing numerical
ordering requirements remain constraints on concurrency, not a reviewed new policy.

Additional applicable verdicts: DM-02 and DM-23 are **Satisfied** in the inspected
owner/registry separation; DM-20 and DM-28 are **Satisfied** in the inspected explicit
inspection/effect paths; DM-07, DM-14, DM-30, DM-31, DM-40, DM-45, DM-48, and DM-54
remain **Unresolved** for the proposed changed paths until §9 controls land.
DM-56, DM-57, and DM-58 are **Satisfied** by the proposal's reuse of existing modules
and rejection of a separate proof/cache platform. DM-39 and DM-59 require retaining
the distinction between structurally removed work and an unmeasured speedup.

## 8. Alternatives and architectural leverage

| Alternative | Duplication and locality | Correctness/operational risk | Cost | Performance evidence | Decision |
|---|---|---|---|---|---|
| Current boundaries, larger limits, more threads | Repeated work and fact rediscovery remain | Can consume more resources while retaining all current failures | Small configuration edit, continuing execution cost | No demonstrated remedy for the repeated work | Reject as the target |
| Retained model SessionContext, native relational composition, owner-bound evidence, and bounded pure-result reuse | One capability assembly; native optimizer sees related work; shared facts/results have explicit lifetimes | Requires precise revision selection, invalidation, and attempt isolation | Cross-cutting replacement of session reconstruction and unnecessary algorithm barriers | Static work removal; speedup not measured | **Select as the strongest target** |
| Reuse one SessionContext but keep current operation bodies and add local memoization | Simpler viable alternative; saves some assembly while keeping independent subqueries/materialization | Easier initial change, but duplicated reuse decisions and hidden plan structure persist | Lower initial cost, incomplete removal of repeated work | Some work removal follows from retention; no end-to-end speedup measured | Insufficient as the final design; any first slice must lead directly to the selected design |
| One globally mutable context and one whole-lifecycle plan | Maximizes nominal sharing but conflates independent revisions, attempts, and lifetimes | Namespace races, accidental stale reads, over-retention, and loss of useful execution boundaries | Fewer handles but more synchronization and recovery coupling | No evidence this dominates scoped sharing | Reject on semantic and lifecycle grounds, independently of existing policy |

No new crate, persistent proof database, cache daemon, alternative semantic graph,
or general effect theorem system is justified. Existing contracts, completion
objects, prepared expressions, provider bindings, and runtime caches provide the
necessary places to retain facts around the native model context. This is an
architectural judgment, not a restriction against adding a crate if later concrete
ownership analysis justifies one. Ordinary specialized algorithm code remains
appropriate where its contract requires finite data or distinctive computation.

## 9. Verification and retained assessment results

### Structural assurance for the proposed changes

Verification should establish **which work occurs**, without needing a timing study
to justify each correction. Add focused counters/observation assertions to existing
unit fixtures, not a second execution implementation.

| Claim | Evidence now | Required focused verification | Expected result |
|---|---|---|---|
| Compatible model work shares the native environment | Interface-checked proposal | Two stages and two compatible data revisions; then change a UDF/registry contract | Shared context assembly, runtime, and unchanged provider owners; changed semantic assembly is separately prepared |
| Shallow Arrow sharing preserves payload and charges | Interface-checked; existing export controls inspected, not rerun | Clone/project/admit a retained batch through two providers and drop consumers in different orders | Same underlying buffers where the operation permits it; one retained allocation charge and correct release |
| Session sharing does not mix revision or query state | Interface-checked proposal | Concurrent queries on old/new selections; a query-time function that can be folded; cancellation and role bindings | Each uses its exact providers and attempt state; no stale folded time or mutation of another query's selected namespace |
| Relational composition removes intermediate execution boundaries | Proposed | Compose an existing relational pass chain, inspect the native plan and execute a focused unit fixture | Expected native operators/obligations visible before execution; no collect/register cycle for the relational-only segment |
| Eligible pure completions survive another invocation | Proposed | Same selected inputs in two attempts; change a dependency; evict; cancel one of two consumers | Producer runs once when eligible; changed input or eviction recomputes; effects and cancelled/incomplete results are never reused as success |
| Native state derivation preserves intended registrations | Interface-checked source behavior; target not implemented | Prepared SQL across context clone/query state versus builder reconstruction; distinguishing Delta/PSE extension and UDF | Documented native distinctions are respected; target derivation retains all required capabilities without default fallback |
| Checked internal values are not re-admitted | Interface-checked proposal | Bind/rebind one checked owner; change owner, contract, and value independently | No duplicate local predicate/copy on unchanged path; real changes re-establish evidence |
| Shared graph structure does not expand merely because parent paths differ | Interface-checked proposal | Distinct contract parents sharing a closed child, plus correlated/worktable/unqualified sibling controls | Intrinsic inspection count follows unique retained producers; required contextual checks remain |
| Preparation is retained under one assembly | Interface-checked proposal | Several outputs/requirements, then one binding/policy/UDF change | One unchanged preparation/assembly; correct affected invalidation |
| Requirements execute once when semantically identical | Proposed | Same invariant and exact inputs at successive stages; changed and empty-reference controls | Reused completed result only for identical dependency selection |
| Reduced queries preserve required behavior | Proposed | Existential validity, full findings, later invalid batch, volatile/effectful child | Correct terminal answer and completion; no skipped mandatory work |
| Cache sharing is safe under cancellation/failure | Proposed | Two consumers, early drop, late stream error, retry, quota refusal | No false success, stale failure reuse, double effect, or leaked lease |
| Native optimization preserves validation | Proposed | Projection/filter/dynamic-filter controls around pending and settled requirements | Optimizer may remove unnecessary work only after its semantic obligation is preserved |
| Delta reuse retains snapshot and commit semantics | Interface-checked proposal | Same snapshot, advanced head, changed CHECK/layout, ambiguous commit | Correct invalidation and recovery; no raw-Parquet substitute or marker-only replay claim |
| Observation is task-appropriate | Interface-checked proposal | Contract versus Diagnostic modes and explicit cap refusal | Routine assurance omits full rendering; requested evidence reports completeness truthfully |

Keep independent functional oracles for numerical/semantic outcomes. Execution
traces can show that an intended check ran or was legitimately reused; they do not
by themselves prove that the check's predicate is correct. After changes and
deletions are complete, run the consolidated local assessment once with no
fail-fast. No repeated broad integration campaign is proposed between slices.

### Existing partial native results

Baseline **zero failures**. Mode: dev build, nextest `ci`, force-validation,
no fail-fast, no retries; stopped by user. New assessment command:
`just assessment build/assessment/2026-09-19-local-full`, native gate `just test`.
The older Plan 10 campaign's native gate was also `just test`.

| Attempt | Selected | Passed | Assertion failures | Timeouts | User interruption | Not started |
|---|---:|---:|---:|---:|---:|---:|
| Local full assessment | 1085 | 1020 | 24 | 5 | 3 | 33 |
| Older campaign 03 | 1085 | 1020 | 24 | 15 | 1 | 25 |

These are overlapping attempts, not disjoint tests to add together. The 24 assertion
failure identities recur. Timeouts remain failures to complete; SIGINT outcomes are
user interruption, not product assertion failures. No whole-suite pass is claimed.

| Observed assertion group | Count | Structural resolution |
|---|---:|---|
| Diagnostic code display spelling | 11 | Compare typed diagnostic identity/class, not dotted versus `::` formatting |
| Rule cause traversal | 3 | Preserve/expose concrete typed causes through shared/transparent wrappers; actual failing leaf needs a focused follow-up |
| Numerical resource classification | 1 | Classify resource failures through nested expression errors consistently; do not merely change the expected variant |
| Numerical fixture semantic metadata | 2 | Construct literals from actual declared field metadata; retain rejection of fabricated alias metadata |
| Retained allocation extent | 1 | Resolve owner-aware extent accounting; do not relax the assertion |
| Tagged enum fixture generation | 1 | Use the declaration's enum literal construction; collect every invariant-case outcome |
| Diagnostic plan capture bound | 1 | Explicit observation policy; preserve honest requested-evidence capacity handling |
| Unresolved native lambda | 1 | Correct binding/coercion/field phase order |
| Deferred DDL root shape assumption | 1 | Assert command/effect contract rather than requiring an unwrapped root |
| Physical observation requested implicitly | 1 | Explicitly select observation or assert the intended execution contract |
| Ambiguous witness qualification | 1 | Narrow lexical scope and preserve source binding through composition |

The evidence is in `partial-results.json`, `test.xml`, `test.log`, and
`test-findings.json` under the new directory, plus `interrupted-test.xml` and
`06-test.log` under campaign 03. `stopped-processes.json` records no remaining
task-owned processes at shutdown. The earlier timing samples and graph-key model
remain diagnostic artifacts; this review does not treat them as quantified root
cause attribution.

### Existing check failures and unrun scope

The new run completed 46 checks before the interrupted native gate. Raw exit statuses
were 37 passed and nine failed, with **27 further gates not run**. The raw unsafe
audit success is invalid evidence because its former recipe masked a tool refusal.
The retained strict follow-up failed correctly. Exact commands, source drift,
excluded environments, and remaining gates are in `checks.json` and `scope.json`.

| Result | Interpretation and follow-up |
|---|---|
| `doctor` failed before `py-sync` | Environment mismatch at that time; later refresh succeeded. Current startup reports ready; preserve historical result |
| `typecheck` failed on new runner imports | Tooling namespace fix landed during collection; focused follow-up found zero errors; do not overwrite original evidence |
| `lint-license` | Regression seed file lacks required license annotation; preserve the failure seed and coordinate with the separate lint work |
| `doc-lint` | Explicit phase-0 stub, exit 2; not completed API-reference validation |
| `architecture-preflight` | Prior source seal no longer matches authorized changes; do not reseal merely to hide remaining acceptance failures |
| Dependency/advisory audits | Reports include quick-xml advisories, proc-macro-error2 maintenance status, and vendored wildcard-policy findings; qualify actual dependency paths before fixes, without implying exploitability from an audit label |
| Shear/machete | Triage real unused consumers versus macro-expanded diagnostic dependencies before deletion |
| Unsafe surface | Installed geiger invocation rejects the virtual manifest; implement supported package enumeration and aggregate outcomes rather than suppressing exit status |

The 27 unrun gates include remaining Rust modes, doctests, linked-solver validation,
Python tests, standalone engineering cases, benchmarks, feature checks, and coverage.
The full lists in the manifest are authoritative; no result is inferred for them.
Assessment source drift includes tooling edits during collection. The receipt is a
diagnostic record, not an acceptance seal for one unchanged source snapshot.

## 10. Exceptions and unresolved decisions

### Policy changes follow the target design

Current rules did not determine the SessionContext recommendation. The following
table identifies the concrete reconciliation needed before implementation planning.
It separates literal blueprint conflicts from proposed guidance and implementation
choices. No governing document is changed by this review.

| Existing rule, guidance, or implementation | Relationship to the recommended design | Recommended decision for planning |
|---|---|---|
| **Blueprint §14.3, “Snapshot sessions, shared runtime”: each snapshot has its own SessionContext and registries** | **Direct conflict.** The target retains one compatible model context across data revisions and selects immutable revision providers | Replace context-per-snapshot with snapshot isolation at provider/binding selection. Keep shared runtime and exact semantic configuration |
| **Blueprint §14.3.1: “a new context requires new preparation”** | **Too broad if context object/UUID is treated as semantic change.** A context handle, query state, resource facade, and implementation assembly are different identities | Require re-preparation for relevant semantic dependency changes; permit reuse across mechanically derived native states with the same actual assembly and eligible inputs |
| **Blueprint §14.3 complete-stage reuse and the measured-refinement trigger** | **Reconcile scope.** This review selects sub-stage producer/validation reuse and relational fusion from their contracts without a prior timing study | Explicitly allow in-process owner-bound producer/obligation reuse below stage boundaries. Keep durable publication membership separately defined; do not require a profile before removing duplicate work |
| **Blueprint §14.3.1 materialization at named stage artifacts** | **Clarify/relax where interpreted as mandatory collection at every compiler stage.** Relational-only stage names should not force materialization | Materialize for a requested artifact, explicit reuse, finite algorithm, fixed-point state, or effect/commit boundary. Preserve completion and provenance even when intermediate stage work is fused |
| **Proposed ADR-0073 N09/N11 history and Plan 10 N09/N11: no global bound-plan cache** | **No necessary conflict.** A bounded model-context cache with exact owners is not an unbounded process-global cache | State the allowed lifetime explicitly. If “global” was intended to prohibit all cross-attempt bound reuse, revise that scope rather than limit the target |
| **Proposed ADR-0070 Outcome: consumed-input identity and current boundary admission** | **Compatible with precise interpretation.** Current permission/resource/commit checks remain; unchanged semantic evidence is reused | Separate current operational admission from repeated semantic validation. Neither a session ID nor cached success may waive current effect admission |
| **Implementation: fresh invocation CacheStore and nested candidate factory construction** | **Replacement required, not a blanket normative prohibition.** Pure results and immutable assembly can have longer lifetimes than attempts | Factor model, workflow/epoch, and attempt ownership; remove nested feature reconstruction and permit eligible result promotion |
| **Implementation: one-worker workflows and broad serial test grouping** | **Configuration to change.** Neither DataFusion nor the semantic model requires all general workflows to be single-worker | Select bounded concurrency jointly with shared resources; retain single-worker scopes only for a real ordering or isolation need |
| **Implementation: optimizer opt-outs and contract pushdown barriers** | **Pin-specific correctness limits, not policy vetoes.** Removing them blindly could change results | Target full native optimization where sound; narrow barriers through completed evidence and fix or qualify the relevant rewrite/reset behavior before enabling it |

ADR-0070 and ADR-0073 currently have `status: proposed`; ADR-0067 is superseded.
References to their mechanisms are design/history evidence, not a claim that these
are all accepted decisions. Blueprint sections cited above are the current written
architecture to reconcile. The existing blueprint already supports shared immutable
owners, native plan composition, and avoiding routine collection; those supporting
rules should remain.

This proposal does not request a SHOULD exception that conceals a MUST gap.
Revision isolation, sound validation, declared effects, and complete publication
remain first-principles requirements. The existing implementation mechanisms for
meeting them are replaceable. The following remaining choices belong in the plan:

| Decision | Recommended default | Revisit condition |
|---|---|---|
| Context lifetime | One compatible model/workspace SessionContext across data revisions; exact provider selection per workflow | A changed capability assembly, credentials, or semantic setting requires a distinct context/generation |
| Evidence storage | Existing owner/admission/completion objects retained by bounded model/workflow scopes | A real cross-process consumer requires durable validation evidence beyond existing artifact admission |
| Invalidation granularity | Exact immutable relation/producer selections and declared dependencies | Existing row/partition provenance can establish a smaller complete dependency boundary |
| Arbitrary custom factories/providers | Admit actual returned output; mutable sources remain fresh | A complete immutable/revision contract is available for the implementation |
| Materialization | Remove relational-only collect/register/tuple boundaries; keep finite algorithm and deliberate shared-result boundaries | The consumer's complete-input, publication, or shared-completion semantics require retention |
| Completed requirement barrier removal | Remove only with valid completed evidence and no remaining effect | New effect, lexical dependency, or dynamic-filter scope changes the proof |
| Throughput defaults | Existing resource policy extended to task-appropriate bounded concurrency | Determinism, shared resources, or solver behavior require a narrower scope |

The owner is the maintainer acting through the implementation plan. That plan should
carry the decision/blueprint changes needed for the selected target, including the
literal context-per-snapshot conflict, rather than reinterpret the target to fit
the old rule. This review itself does not amend ADRs or the blueprint. Timing
measurements can later characterize the result; they are not a prerequisite for
removing the identified duplicate work.

## 11. Decision and implementation changes

**Revise.** Adopt the retained model-context/native-composition design, reconcile
the policies in §10, and finish its ownership/reuse contracts end to end. Correctness
defects and unresolved reuse boundaries preclude acceptance today. Action the
proposal as a hard pivot, deleting superseded session reconstruction, internal
adapters, unnecessary materialization, and duplicated checks once their replacement
boundary is established.

| Priority/order | Change | Principles | Acceptance evidence | Regression protection |
|---|---|---|---|---|
| Design decision before implementation | Reconcile context-per-snapshot, context-identity preparation, and stage-granularity/materialization guidance with §4.3 and §4.6 | DM-12, DM-26, DM-32, DM-56 | Explicit decision and updated governing descriptions of the selected target | No implementation workaround that reinstates unnecessary context or stage boundaries |
| 1 — correctness foundation | Fix lambda/field phase order, lexical witness scope, owner-aware allocation extent, and typed diagnostic traversal | DM-09, DM-24, DM-42, DM-47 | Focused existing failures plus adversarial unit controls | Keep malformed metadata, nested scopes, and budget failures explicit |
| 2 — retained native environment | Retain the model SessionContext and capability assembly; select revision/role views; remove nested factory/registry reconstruction | DM-12, DM-26, DM-31, DM-35 | Stable assembly/runtime/provider owners across compatible work; fresh attempt state | Concurrent old/new revisions, query time, cancellation, Delta/PSE extensions, and prepared SQL controls |
| 3 — native composition and evidence | Lower relational-only algorithm segments into native plans; remove needless collect/tuple/register boundaries; retain producer facts and checked member evidence at remaining finite boundaries | DM-18, DM-24, DM-26, DM-34, DM-37 | Native plan visibility, producer/assembly work counts, and no repeated admission of unchanged members | Distinct-parent DAG, lexical/effect scopes, provenance, rewritten values, and raw ingress controls |
| 4 — obligation reuse | Carry completed invariant outcomes across stages; group shared producers and affected checks | DM-07, DM-26, DM-32, DM-33 | Same semantic obligation executes once; changed dependencies invalidate | Empty-input and cross-relation negative controls |
| 5 — minimum useful work and result lifetime | Reduce validity queries, narrow projections, share output/findings/provenance, retain stable physical inventory and eligible pure completions in the bounded model context, stream eligible consumers | DM-18, DM-32, DM-36, DM-37, DM-38 | Exact outcomes, cross-attempt producer reuse, and bounded retention | Late errors, changed revisions, consumer cancellation, effects, eviction, and finite-algorithm controls |
| 6 — native optimization/storage | Narrow settled contract barriers; preserve pending requirements; reuse exact Delta snapshots/runtime services and direct write plans | DM-24, DM-28, DM-32, DM-43 | Native plan/scan assertions and snapshot/commit controls | No cache contamination, skipped check, or stale-head reuse |
| 7 — appropriate assurance/concurrency | Explicit fixture observation policy, bounded workflow parallelism, independently reported invariant cases, strict local tooling | DM-35, DM-38, DM-47, DM-53, DM-59 | Focused policy/collector tests; no hidden failures | No timeout inflation, swallowed tool exit, or forced full-plan rendering |
| 8 — consolidated closure | Delete obsolete paths, refresh source evidence, execute the full local no-fail-fast assessment once | DM-53, DM-54, DM-60 | Complete report against zero failures, with commands/modes and all exclusions explicit | Preserve durable failure artifacts and truthful partial results on interruption |

The acceptance question is first whether unnecessary work has been removed while
the same required meaning and effects remain enforced. End-to-end timing can then
describe the improvement, without becoming a gate on making the design coherent.
