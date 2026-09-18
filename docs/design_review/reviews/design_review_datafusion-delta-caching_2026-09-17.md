---
title: Caching across DataFusion and Delta — planes, authority and the target design
date: 2026-09-17
status: revise
scope: docs/capability-maps/datafusion_and_deltalake_caching_overview.md; the cache-bearing runtime, session, rule-round and Delta paths in pse-runtime, pse-catalog and pse-rules; the cache and reuse obligations of Plan 08 (SP08, SP12, E09); and the reuse planes a process-simulator workload needs regardless of the plan's current scope
---

# Caching across DataFusion and Delta — planes, authority and the target design

## 1. Decision and scope

**Decision: Revise the overview; adopt the target design in §8; amend the rules and
policies listed in §10 where the target requires it.** The overview's two persistence
tiers are rejected because they hide effects and create a second authority for table
state. Its remaining recommendations are either already implemented or reduce to
declaring, bounding and observing what DataFusion already does. The substantive gaps
lie where neither the overview nor the first pass of this review looked: the rule
engine re-plans and re-binds every relational operation on every fixed-point round;
Delta log state is rebuilt from scratch on every open and never checkpointed outside
destructive maintenance; decoded relations are never resident across invocations; and
the dependency model cannot distinguish a parameter-value change from a structural
change, so a simulator sweep recomputes everything. Each of those is addressed by a
native facility at the pinned releases, and each is Proposed until the benchmark named
in §9 exists.

**Revision note.** The first pass of this review deferred several capabilities on
proportionality grounds (no present consumer, outside the plan's local envelope, not
yet measured). The maintainer's direction is that the review target is best-in-class
design for a process-simulator environment and that rules and policies serve that
target, so a conflict is recorded as a required policy change, not as a constraint on
the design. This revision widens the target to the simulator workload — interactive
edit loops, fixed-point inference rounds, parameter sweeps, repeated cold opens,
shared remote storage — and §10 names the rule or policy each recommendation needs to
change, with the sanctioned route for changing it.

**Proposal under review.** Two things, reviewed together:

1. The document `docs/capability-maps/datafusion_and_deltalake_caching_overview.md`,
   an unlabeled narrative recommending a "CachingPolicy" root, a tiered
   `CacheFactory`, a persisted Delta snapshot cache, a byte-level `AsyncFileReader`
   cache and a patch for the CDF metadata cache.
2. The cache-bearing paths of the working tree on `wave2/semantic-compilation`
   against the obligations of [Plan 08](../../plans/08-schema-first-native-data-pivot.md)
   (SP08 "shared memory/spill/cache/task ownership", SP12/E09 "exact reuse and
   protected maintenance", N08 "native metadata and semantic dependencies"), and
   against the simulator workload the plan's successor will have to serve.

**Status of claims.** Library claims are *Interface-checked* against the pinned skill
indexes and, where the index carries only public items, against the pinned sources in
the cargo git checkouts. Code claims are *Implemented*; the invocation cache is
*Tested* at the grain of its four named unit tests. Nothing here is *Measured*: the
workspace has one Criterion benchmark (`benches/benches/canonicalization.rs`) and
nothing that times a Delta open, a log replay, a fixed-point round or a footer read.
Every performance statement below is a hypothesis with a named measurement.

**Reviewer.** Claude (design-review skill), acting for the maintainer.

**Affected revisions.** DataFusion 55.1.0, Arrow 59.3.0, object_store 0.13.2
(`Cargo.toml` pins); delta-rs `58f07cd62bfbce3649a7e1c87c696288068ae184` with the
kernel at `8ba063f8`; the working tree at commit `7bbfc12` plus uncommitted edits. The
tree was being edited by the maintainer's implementation stream while this review
ran; `crates/pse-catalog/src/delta/provider.rs` and
`crates/pse-catalog/src/session/cache.rs` changed under the review and every citation
is to the version read last.

**Observable outcome.** After the changes in §11: a fixed-point stratum is planned
once and executed per round; a publication open costs no log replay when the same
versions were opened before in the process, and a bounded number of commits
otherwise; a parameter sweep recomputes only value-bearing artifacts; a member-attempt
retry is not refused because a spill directory or cache limit differs; every cache
ceiling is declared, read back, accounted and observable; `df.cache()` stays
effect-free and can serve exact pinned-version scans across invocations; and the
overview becomes a pinned capability map whose claims can be cited.

**Baseline.** One `RuntimeEnv` per process (`crates/pse-runtime/src/env.rs:43-56`,
`crates/pse-py/src/inspection/runtime.rs:36-62`), one `SessionFactory` built from it
(`crates/pse-runtime/src/session_factory.rs:12-21`), DataFusion's cache manager at
its implicit defaults, an invocation-local `CacheFactory`
(`crates/pse-catalog/src/session/cache.rs`), every Delta open a fresh
`DeltaTableBuilder::load()` (`crates/pse-catalog/src/delta/provider.rs:28-32,67-71`),
checkpoints only inside the maintenance command
(`crates/pse-catalog/src/delta/maintenance.rs:416`), and a rule engine that compiles,
analyzes, optimizes and physically plans every candidate, probe, union and difference
plan afresh in every round (`crates/pse-rules/src/strata/rounds.rs:41-63`,
`crates/pse-rules/src/strata/native_state.rs:100-128,167-262`).

**Supported scope and non-goals.** In scope: the cache planes that exist at these
pins — DataFusion's runtime file caches, `CacheFactory`, plan re-execution through
`reset_plan_states`, Delta snapshot materialization, incremental update and
checkpoints, the CDF reader's private cache, the dormant `delta-cache` feature — plus
the settings inventory that decides whether two sessions are semantically equal, and
the dependency classification that decides what a value change invalidates. Out of
scope: the correctness of durable semantic reuse itself (Plan 08 E09; its gate stays
*Unresolved* by the plan's own statement) and distributed execution. Remote object
stores are in scope as a seam that the local design must not preclude (§8, F10),
which is a widening of Plan 08's exclusion and is recorded in §10.

**Constraints and uncertainty.** Scale is unmeasured: members per publication,
commits per table between maintenance runs, files per member, rules per stratum in
real models, and rounds per stratum are not recorded anywhere. Delta data files are
immutable and uniquely named; several validity arguments depend on that protocol fact
and say so. `reset_plan_states` refuses plans with dynamic filters and recursive
queries (DataFusion `execution_plan.md:439-454`); the prepared-round design in F2
depends on that limit being honored by the rule engine profile.

### Method and coverage

Read in full: the overview; Plan 08 §§ Context, Decisions (N08, exact-pin cautions,
policy placement table), the current remaining scope, E04–E10, SP08/SP09/SP12/SP13, the
acceptance matrix and terminal gates; the charter, directive and template; the
DataFusion skill's `sessions-and-runtime` topic, the `cache_manager`, `cache`,
`default_cache`, `session_state` (CacheFactory), `execution_plan` (`reset_plan_states`),
`work_table`, `operator_statistics` and `datafusion_execution.runtime_env` API pages,
the config-options catalog and the `dataframe/cache_factory.rs` example; the delta-rs
skill's `snapshot-and-log`, `datafusion`, `maintenance` topics and the `session`,
`table.builder`, `snapshot` API pages plus `coverage.tsv`; in the pinned delta-rs
source, `delta_datafusion/session.rs`, `table_provider.rs` (builder surface,
`DeltaScanConfig::new_from_session`, `update_datafusion_session`),
`table_provider/next/mod.rs` (`ensure_read_ready`), `next/scan/mod.rs:695-705`,
`operations/load_cdf.rs:40-140`, `kernel/snapshot/mod.rs:95-560,941-974,1340-1360`,
`kernel/snapshot/serde.rs:30-130`, `kernel/snapshot/scan.rs:470-560`,
`table/builder.rs:30-170`, `table/mod.rs:207-230`,
`kernel/transaction/mod.rs:540-620,1155-1166`, `crates/core/Cargo.toml`; in the
kernel, `snapshot/builder.rs:40-190` and `snapshot/mod.rs:845-930`. In this
repository: `pse-runtime` `env.rs`, `budget.rs`, `session_factory.rs`; `pse-catalog`
`session/{cache,factory,config,resources,materialized,execution,candidate}.rs`,
`session/preparation.rs:78-175,339`, `session/roles.rs:129-153`,
`session/snapshot_session.rs:1-70,125-165,225-260,300-320,595-690,780-800`,
`delta/{provider,changes,attempt,contract}.rs`, `delta/publication.rs:100-300`,
`delta/publish.rs:300-340,440-450,515-525`, `delta/write.rs:390-475`,
`delta/maintenance.rs:40-110,300-460`; `pse-rules` `strata/rounds.rs`,
`strata/native_state.rs:100-300`, `strata/native.rs:1-60`,
`strata/native/execution.rs:150-260`; `pse-schema` `catalog/publication.rs:1-30,40-140,195-211`,
`model/relation.rs:215-225`; `pse-py/src/inspection/{runtime.rs:30-75,handles.rs:40-110}`;
the cache unit test names in `session/cache/tests.rs`; ADR-0017/0019/0041/0042/0046/
0052/0068/0069 front matter and outcomes; blueprint D14, §3.2, §3.3.3, §5.4, §6.15.2,
§14.2, §14.3, §26 at the grep-located lines; register rows R-01 and R-22.

Grepped, not read: every product-code use of `RuntimeEnv`, `SessionStateBuilder`,
`with_session_state`, `DeltaTableBuilder`, `MemTable`, `ListingTable`,
`TableFunctionImpl`, `register_object_store`, `create_checkpoint`, `update_incremental`,
`cache_plan`, `create_physical_plan` and `execute_stream` under `crates/*/src`
excluding test modules; every `DeltaTable` field in a product struct; every
`DataFusion` symbol in the solver callback path (none found: the Ipopt inner loop does
not execute relational plans).

Not inspected: the `information_schema.df_settings` implementation that the settings
consistency check at `snapshot_session.rs:780-800` relies on; the body of
`impl CacheKey for object_store::path::Path` (the claim that it carries no table
reference is inferred from `TableScopedPath` being the only other implementor, and is
labeled as such); delta-rs's `LocalFileSystem` `ObjectMeta` derivation; the private
`WorkTable::update` body (only `name` is public at this pin, which is why F2 proposes a
PSE-owned round-buffer node rather than reusing `WorkTableExec`); any Python test;
`native::model::compile`'s composition beyond its public entry points.

Guarantees attacked: "a cache hit cannot change the answer" for each in-process plane
(§3, §5); "the invocation cache cannot serve one sibling another sibling's
differently rewritten input" (§3); "retry identity is stable across deployments" (F1,
found violated); "a prepared plan re-executed against updated round buffers is
equivalent to a fresh plan" (F2, holds only under the `reset_plan_states` limits and
with statistics-independent physical choices; recorded as the design's precondition).
Guarantees asserted, not attacked: that `LocalFileSystem` reports a changed
`ObjectMeta` for an in-place rewrite of equal size within the same second (moot under
Delta's write-once files); that `execution_scope`'s rebuilt `SessionState` reaches the
same `NativeExecutionContext` after `bind`.

## 2. Authority and lifecycle map

| Concept or fact | Semantic type and identity | Authority / owner | Revision or snapshot boundary | Permitted update path | Derived representations |
|---|---|---|---|---|---|
| Parquet footer and page-index metadata for one data file | `CachedFileMetadataEntry` keyed by `object_store::path::Path` (`FileMetadataCache = dyn Cache<Path, CachedFileMetadataEntry>`) | The file; the entry is a disposable derivation validated by `is_valid_for(&ObjectMeta)` | The file's `ObjectMeta` | Miss → parse → `put`; LRU; no table-scoped invalidation (the key carries no table reference) | Consumed by delta-rs's next scan: `CachedParquetFileReaderFactory::new(store, state.runtime_env().cache_manager.get_file_metadata_cache())` (`next/scan/mod.rs:701-703`) |
| File statistics and listings | `FileStatisticsCache`, `ListFilesCache` keyed by `TableScopedPath` | `ListingTable` only; no consumer here (no `ListingTable` under `crates/*/src`) | Schema fingerprint + `ObjectMeta`; TTL | `drop_table_entries` | None here |
| Cache ceilings (50 MiB / 20 MiB / 1 MiB / no TTL) | `datafusion.runtime.*` entries | **Undeclared** DataFusion defaults: `RuntimeEnvBuilder` in `env.rs:43-46` sets pool, temp path and temp size only | Process | Would be `ResourceBudget` → `RuntimeEnvBuilder::with_*_cache_limit` → read back via `CacheManager::get_*_limit` (F6) | `RuntimeEnv::config_entries()` |
| Invocation-shared computation | `Cache` extension node keyed by a fresh `Arc<()>` plus plan equality (`cache.rs:77-101`); completion cells in `NativeExecutionContext.caches` (`execution.rs:30-31,52-53`) | The operation that called `cache_plan` (`pse-compiler/src/native.rs:82,98`) | One `bind` of a `NativeExecutionContext` | Never reused across invocations; pool-accounted and spillable (`cache.rs:222-262`) | `NativeCacheExec`, one output partition (`:299`) |
| Prepared relational plans for a rule stratum | **None retained.** Each round calls `compile_query` per evolving rule and query (`rounds.rs:55-63`), then `prepare_rule_plan(..).execute(..)` per candidate, probe, union, difference and delta plan (`native_state.rs:112-128,131-139,167-262`); each prepare re-binds targets, opens an execution scope and re-runs analyzer and optimizer (`preparation.rs:96-105,107-175`); physical planning happens per execute (`preparation.rs:339`) | The round driver | One round | Re-created every round | — |
| Round workspace relations (facts, deltas, assertions, support) | `FieldCheckedBatch` per relation, rebound each round as fresh `CandidateTable` providers (`native_state.rs:100-110`, `roles.rs:129-153`) | The stratum (`State`) — the mutable workspace DM-29 permits | One round | `merge_round` produces the next round's batches (`native_state.rs:167-262`) | Providers are immutable per binding, so the logical plan captured at binding cannot be re-executed against the next round |
| Delta table state at a version | `Snapshot { inner: KernelSnapshot, config: DeltaTableConfig, materialized_files }` with `SnapshotIdentity { table_root, version, checkpoint_version, protocol, metadata }` (`kernel/snapshot/mod.rs:95-114,214-221`) | The `_delta_log`; materialized add batches are a derivation validated by `is_cache_for` (`:445-447`) | The Delta version | `Snapshot::update` seeded from the existing materialization (`:291-350,941-974`); **never used here** | `DeltaScan` retains its `Snapshot`; `EagerSnapshot` materializes on open when `require_files` (`:1340-1360`) |
| Checkpoint | Parquet checkpoint + `_last_checkpoint` | The log; the protocol's persistent replay cache | Written at `(version+1) % checkpointInterval == 0` only when `CommitProperties.create_checkpoint` (`transaction/mod.rs:562-566,1159-1165`) | Every PSE commit sets `with_create_checkpoint(false)` (`attempt.rs:80-90`, `publish.rs:445-446,522-523`, `maintenance.rs:398-402`); one explicit `create_checkpoint` in maintenance (`maintenance.rs:416`) | Kernel `LogSegment` |
| Decoded member relations across invocations | **None retained.** A Python handle holds its opened publication until `close()` (`handles.rs:40-62`); a second `pse.open` of the same root and version re-opens and re-decodes | — | — | — | `MaterializedTable`/`CandidateTable` already hold `OwnedRecordBatch`s that a resident tier could share by `Arc` |
| Dependency classification | `NativeDependencyKind = {operation, input, contract, function, rule, setting, policy, provider, scope, observation}`; `native_dependencies(kind, scope, name, value, identity, fingerprint, selection)` (`pse-schema/src/catalog/publication.rs:49-78`) | Registry | Registry revision | E09 will populate it | No column-level or value-only class: an `input` is an exact member version |
| CDF Parquet metadata | Private `DefaultCache::new(1 MiB)` inside `CdfLoadBuilder::new` (`load_cdf.rs:49,93-96`) | delta-rs; not injectable (no session or runtime setter on the builder) | The builder | None | — |
| `delta-cache` feature (`foyer`) | Cargo feature with zero `cfg` sites (`crates/core/Cargo.toml:81-82,133,161`; skill `coverage.tsv`) | delta-rs | — | Do not enable | — |
| Semantic settings inventory | `config_options().entries()` **chained with** `runtime_env().config_entries()` (`snapshot_session.rs:151-158`) | `SessionSemantics.settings` (`:230-246,635-646`) → `MemberAttempt.semantics` (`attempt.rs:52-54`) | Sealed at session build | Whole-struct equality in `compare` (`attempt.rs:160-166`) | `settings_hash`, `profile_hash` (`:681-686`) |

**Deliberately opaque behavior.** delta-rs registers the table's root object store
in the shared runtime registry on every read entry (`next/mod.rs:623-632`); idempotent,
documented upstream, a no-op for `file://` roots. Kernel-side incremental CRC replay
(`IncrementalReplay`, `write_checksum`) exists but delta-rs neither writes checksums
nor enables it (grep of `crates/core/src` finds only `_last_checkpoint` reading), and
`Snapshot` has no public constructor from a kernel snapshot, so it is unreachable
without an upstream or fork change (F9).

**Identity behavior.** Metadata cache entries are keyed by object path: moving a root
misses everything, correct for a disposable cache. A retained snapshot or resident
relation is keyed by `table_root` plus version: a moved root is a different table. The
invocation cache identity is minted per `cache_plan` call and survives optimizer
rewrites of that node's input (`cache.rs:118-131`). A prepared stratum plan's identity
is the tuple (rule id and version, query, delta variant, port binding, engine profile
hash, input relation schemas); round buffers are stable objects whose *content* changes.

## 3. Semantic contracts and invariants

| Contract or invariant | Representation | Enforcement boundary | Failure behavior | Verification evidence |
|---|---|---|---|---|
| A metadata-cache hit returns the footer of the file currently at that path | `CachedFileMetadataEntry::is_valid_for(&ObjectMeta)` | Inside `CachedParquetFileReaderFactory` | Stale entry replaced | Interface-checked. Depends on Delta's write-once, uniquely named data files; a vacuumed file's entry lingers until eviction and is never served because the log no longer references the path |
| A materialized file set belongs to exactly this snapshot | `SnapshotIdentity` + `MaterializedFilesPolicy::satisfies(AddStatsPolicy)` (`kernel/snapshot/mod.rs:133-146,445-452`) | `materialized_files()` filters on `is_cache_for`; serde drops incompatible payloads (`serde.rs:100-125`) | Recomputed | Interface-checked; delta-rs tests in `operations/restore.rs:535-655` |
| One computation per invocation-cache node; siblings share completion and failure | `CacheStore` keyed by identity pointer with the `Arc<()>` retained (`cache.rs:156-174`); `Shared` future (`:347-385`) | `CachePlanner` | Unbounded input refused (`:195-197`) | **Tested**: `lazy_siblings_share_input_and_release_reservations`, `abandoned_reader_does_not_restart_partial_input`, `native_spill_preserves_values_and_is_not_a_durable_memo`, `caller_cache_factory_is_retained` (`session/cache/tests.rs`); `just unit-package pse-catalog 'test(session::cache::tests::)'`: 4 passed, 0 failed per the Plan 08 receipt of 2026-09-17; not re-run here |
| Two siblings with the same identity have semantically equal inputs | `prevent_predicate_push_down_columns` returns every column (`:132-139`); no `necessary_children_exprs` override | Logical optimizer | — | Attacked: physical rules may repartition one sibling's input, but `Cached::collect` drains all partitions and the node advertises no ordering, so the shared batches are the same relation. Sound under DM-24 rewrites only |
| The cache node declares its effect | `ExecutionContract::plan(.., [OperationEffect::Read])` (`cache.rs:60-75`) | Effect admission | Refused where reads are not admitted | Implemented |
| A prepared plan re-executed against updated round buffers equals a fresh plan (F2 precondition) | `reset_plan_states` (`execution_plan.md:439-454`): resets operator state (`CrossJoinExec`'s loaded side is the documented example); refuses plans that use dynamic filters or represent a recursive query | Stratum preparation | Refuse the profile, or re-plan | Interface-checked. Physical choices made from round-one statistics (join side, hash versus merge) persist; the rule profile must either pin statistics-independent physical rules or accept the choice as an approximation policy (DM-24) |
| Semantic session identity contains only semantic inputs | `SessionSemantics.settings` | `MemberAttempt::compare` | `IdentityReused` | **Violated** — F1: `datafusion.runtime.temp_directory`, `memory_limit`, `max_temp_directory_size`, `max_spill_merge_fan_in`, `metadata_cache_limit`, `list_files_cache_limit`, `list_files_cache_ttl`, `file_statistics_cache_limit` are all in the inventory (config catalog rows 194-201). Blueprint D14 requires "implementation and semantic settings"; resource entries are neither |
| Checkpoint creation is non-destructive | Delta protocol; `cleanup_expired_logs(Some(false))` on every PSE commit | delta-rs post-commit | — | Interface-checked (`transaction/mod.rs:1064-1085`) |
| The caller's session is never silently replaced | `SessionFallbackPolicy::RequireSessionState` at all nine delta-rs call sites (`write.rs:357-358,461-462`; `publish.rs:374-375,402-403`; `dml/merge.rs:110-111`; `dml/execution.rs:189-190,211-212,228-229`; `maintenance.rs:409-410`) | delta-rs `resolve_session_state` (`session.rs:150-182`) | Error | Implemented; negative case not tested here |

**Absence and uncertainty.** A miss, an evicted entry and a never-populated cache are
indistinguishable and need not be distinguished: every plane recomputes. What must
stay distinguishable is *unknown table state* versus *empty table state*, and it does:
`verify_deltatable_existence` precedes `load` (`write.rs:401-404`), a missing commit in
a CDF window refuses (`changes.rs:82-84`), and recovery reads raw commit entries rather
than a snapshot (`attempt.rs:102-118`). Under F5 a *value-only* dependency change must
be distinguishable from a *structural* one; today it is not representable.

**Equivalence requirements.** Cache hits promise byte-identical Parquet metadata and
structurally identical add-action batches; the invocation and resident tiers promise
the same rows with no ordering promise; prepared-plan re-execution promises the same
rows and, where the profile pins statistics-independent physical rules, the same plan
shape. No plane needs approximate equivalence.

## 4. Derivation and execution design

| Stage or operation | Input revisions and dependencies | Output contract | Preconditions / assumptions | Effects and mutable ownership | Provenance / invalidation |
|---|---|---|---|---|---|
| Build the process runtime (`SharedRuntime::build`, `env.rs:31-70`) | `ResourceBudget` | One `Arc<RuntimeEnv>` with pool, disk manager and **default** cache manager | Read-back covers pool and disk only | Process-wide | — |
| Build a session (`SessionFactory::from_builder`, `factory.rs:64-105`) | Runtime, engine profile, caller builder | `SessionState` with `NativeCacheFactory` unless supplied; runtime attached | — | Shares the runtime's cache manager | Settings inventory captured at `bind` (`snapshot_session.rs:151-158`) |
| Scoped execution (`execution_scope`, `resources.rs:150-168`) | Effective policy `max_bytes` | New `RuntimeEnv` with a `ScopedPool` and the **same** `cache_manager` and registry | — | Child accounting; one new `SessionState` per prepare when a byte policy applies | Cache identity preserved |
| Fixed-point stratum (`FixedPoint::execute` → `bind_inputs` → `iterate`, `native/execution.rs:155-260`; `Round::execute`, `rounds.rs:41-108`) | Program (rules, bindings, outputs, limits) built at planning (`strata/native.rs:35-51`); round buffers | Typed multi-output batches | Bounded inputs | Per round and evolving rule: SQL → logical compile, emptiness probe (analyze, optimize, physical plan, execute), one candidate plan per delta variant (same four stages), support plans; per head: union, difference, facts and delta plans (four more); workspace rebound as fresh providers | Nothing retained between rounds |
| Open a publication (`Publication::open` → `read_optional_record` `publication.rs:147-181` → `bind_members` `:184-260` → `verify_inputs` `:278-296`) | Control root + version; each member's `table_uri` + `delta_version` | Sealed session with one `ViewTable` per member | Local `file://` root | One full `DeltaTableBuilder::load()` for the control table, **per member and per input** (`provider.rs:67-71`); each `view` consumes the table into a `DeltaScan` (`:76-87`) | No retained snapshot |
| Scan a member | `DeltaScan` over its `Snapshot`; session runtime; `DeltaScanConfig::new_from_session` inherits `parquet.pushdown_filters` (`table_provider.rs:200-209`), which `config.rs::build` leaves at its default `false` | Arrow batches | Files exist (vacuum excluded by read lease, `provider.rs:26,45`) | `ensure_read_ready` registers the store idempotently | Footers cached in the shared metadata cache; whole files decoded because pushdown is off; the predicate cache is inert |
| Write a member (`write_attempt`, `write.rs:391-475`) | `MemberAttempt`, input plan | Committed version | Existence check | `load()` before (`:405-410`), delta-rs's own snapshot inside `write`, `load()` after (`:466-471`); receipts compared by whole-struct equality | Recovery reads commit JSON |
| Change feed (`changes`, `changes.rs:36-130`) | Member, `after`, `end` | `DeltaCdfTableProvider` plan | `after ≤ end` | `load()` at `end` (`:56-64`), `load_version(after)` (`:66-72`), raw commit scan of the window (`:77-96`), `scan_cdf` with the private 1 MiB cache (`:97-105`) | Contract mismatch inside the window refuses |
| Maintenance (`maintenance.rs:300-460`) | Head root, retention rows | Outcome row | Exclusive lease; local paths | `load()` of head and target; `load_version` per protected version (`:392-396`); optional optimize; `create_checkpoint` (`:416`); vacuum with `keep_versions`; log cleanup unless attempts or control (`:441-449`) | The only checkpoint producer |
| Python open (`open_publication`, `handles.rs:57-62`) | Root, version, settings | `Publication` handle over the process runtime | — | Full publication open per call | Handle-lifetime only |

**Relationship structures.** Reuse dependency (which cached artifact depends on which
file, version or round buffer) is distinct from the read-lease ownership relation that
excludes maintenance and from the stratum's data-flow relation between rules; the
design keeps them distinct because leases are per table root, caches are per path or
version, and round buffers are per relation role.

**Provider selection and limitations.** delta-rs's next scan is the only Delta read
path in product code (`provider.rs:82`, `dml.rs:50`, `dml/execution.rs:295-296`) and
the path that consumes the runtime metadata cache. The CDF path does not. Delta
supplies file lists and statistics from the log (blueprint §5.4), so
`FileStatisticsCache` and `ListFilesCache` have no consumer. DataFusion's in-tree
precedent for re-executing a prepared plan per iteration is `RecursiveQueryExec` over
`WorkTableExec` with `reset_plan_states`; `WorkTable::update` is crate-private at this
pin, so the round buffer node is PSE-owned and reuses only the public
`reset_plan_states` contract. `object_store` 0.13.2 ships no caching store
(`ChunkedStore`, `LimitStore`, `PrefixStore`, `ThrottledStore` only), so a byte cache
is a decorator registered in the runtime's registry (F10).

**Boundary contracts.** Python `pse.open(uri, version)` → `open_publication` →
`Publication::open` on the one process runtime; every call pays the full open above.
A resident tier (F4) would let the second call return decoded batches by `Arc`,
and the Arrow C-stream export would then reference resident buffers without a copy.

**Coherent publication.** Unchanged by any plane: the publication root commit is the
boundary (Plan 08 SP09), and no cache is visible to readers as state.

## 5. Representative journeys

### Ordinary extension — declaring a new cache ceiling

Today, raising the metadata cache from 50 MiB means knowing that
`RuntimeEnvBuilder::with_metadata_cache_limit` exists and editing `env.rs`; nothing
reads it back. Under §8, it is one field on `ResourceBudget`, one builder call, one
read-back assertion and one row in `ResourceReport`: one declaration, no
re-expression (charter §E).

### Meaningful change — a member gets a new version

An edit workflow commits version *v+1* of a member. Metadata-cache entries for files
retained from *v* stay valid; new files miss once. Today the next open replays the log
from the last maintenance checkpoint. Under §8, `Snapshot::update` seeded from *v*
replays only commit *v+1* (`kernel/snapshot/scan.rs:519-530`), the retained entry for
*(root, v)* stays valid for readers pinned at *v*, and the resident relation for
*(root, v)* is untouched while *(root, v+1)* is decoded on first use.

### Meaningful change — a parameter sweep

A case study changes one parameter value in a source member. Under Plan 08's dependency
model the source member is a new exact version, so every dependent artifact — normalized
model, equations, incidence, Jacobian coordinates, numerical program — is invalidated
and recomputed. The structure of all of them is unchanged. Under F5, the compiler's
algorithm signatures declare which input columns are structural; the dependency row for
a structural consumer carries a fingerprint over those columns only; the sweep
recomputes the numerical program's values and the solver outcome and reuses everything
else. The oracle is a clean recomputation (DM-32).

### Meaningful change — one fixed-point round

Round *n* of a stratum with *k* evolving rules and *h* heads today performs *k* SQL
compilations, *k* emptiness probes, up to *k × ports* candidate plans and *4h* merge
plans, each through analyzer, optimizer and physical planner, against providers
created for this round (`rounds.rs:55-63`, `native_state.rs:167-262`). Under F2 the
stratum is prepared once: every such plan is a physical plan whose evolving leaves are
round-buffer nodes; round *n* updates the buffers, calls `reset_plan_states` and
executes. The oracle is the existing rule fixture set, which must produce identical
outputs.

### Boundary or alternate representation — Python reopens the same publication

Two consecutive `pse.open` calls on the same root and version perform identical work
today. Under §8 the second finds every `(root, version)` in the factory-owned retention
and every decoded member in the resident tier, performs zero replays and zero decodes,
and the `DeltaScan` built through `TableProviderBuilder::with_snapshot`
(`table_provider.rs:305`) is the same provider type as today.

### Interruption or failure — retry from a differently configured process

A member write records a receipt whose `semantics.settings` contains
`datafusion.runtime.temp_directory=/scratch/a`. The process crashes after the commit.
Recovery runs on a host whose budget names `/scratch/b`. `inspect` finds the receipt,
`compare` sees a different map and returns `IdentityReused` (`attempt.rs:121-127,160-166`).
The committed member can never be reconciled to this attempt. The same happens when
F6 lands and changes a cache limit while an attempt is outstanding. Under the target,
vacuum after a retained snapshot or resident relation: the maintenance lease
invalidates both for that root before any file is deleted; a reader holding a
`DeltaScan` over the old snapshot is already excluded by its read lease.

## 6. Acceptance gates

| Gate | Pass / fail / unresolved / not applicable | Evidence or scope rationale | Required action |
|---|---|---|---|
| G1 — Authority | **Pass** for the current code; **Fail** for the overview's §7 tiers and §12 persisted snapshot if adopted | One runtime and one cache manager per process; the log is the only table-state authority; every in-process cache is a validated, disposable derivation. A persisted `Snapshot` is documented by delta-rs as "a trusted persistence format, not an authentication boundary" (`serde.rs:117-125`); a catalog-backed `df.cache()` tier would publish outside the commit boundary. The resident tier in F4 is keyed by Delta's own version identity and is disposable, so it does not create authority | Reject the two tiers (F7); adopt F4's sound variant |
| G2 — Semantic fidelity | **Pass** | No plane reinterprets a value; `skip_stats` is a typed policy delta-rs refuses to satisfy for stats-needing reads | — |
| G3 — Validity | **Pass** | Entries validated before use; unbounded inputs refused by `CachePlanner`; unknown Delta state refused. F2's prepared plans must refuse profiles with dynamic filters (`reset_plan_states` limit) — a precondition, not a present defect | Add the profile check with F2 |
| G4 — Hidden behavior | **Pass**, with one recorded ambient mutation | `cache_plan` executes nothing (tested); the cache node declares `Read`; delta-rs's idempotent store registration is documented and a no-op locally | Record in the SP08 provider notes |
| G5 — Consistency and recovery | **Pass** | Caches disposable; recovery reads commit JSON; maintenance checkpoints before vacuum and the checkpoint is non-destructive | — |
| G6 — Transformation and reuse | **Pass** for present cache hits; **DM-32 violated** on the identity key (F1); **Unresolved** for the two new reuse planes until their oracles exist | No present hit changes an answer (§3). F2 and F5 introduce reuse whose validity rests on `reset_plan_states` limits and on a column-level fingerprint; both need the clean-recompute oracle before they are more than Proposed. Durable semantic reuse (E09) remains *Unresolved* per Plan 08 | Fix F1 first; ship F2 and F5 behind their oracles |
| G7 — Truthful capability claims | **Fail** for the overview as an evidence document; **Pass** for the code | The overview carries no pins or labels and one claim that does not hold for the cache Delta uses; it omits the capabilities that matter most (seeded incremental update, `with_snapshot`, checkpoint policy, `reset_plan_states`, `config_entries` read-back). Code claims about the invocation cache match its tests | Rework the overview (F8) |

An unresolved gate is not a pass. No score below offsets G7.

## 7. Principle findings

Findings are ordered by severity: correctness and authority first, then preparation
not separated from repeated execution (one structural cause with four instances, F2–F5),
then declaration and observability, then the overview's defects, then upstream and
scope seams.

| # | Finding | Principle IDs | Concrete evidence or gap | Consequence | Proposed correction | Verification |
|---|---|---|---|---|---|---|
| F1 | Non-semantic runtime resource entries are part of the member-attempt identity | DM-32, DM-33, DM-13; blueprint D14 | `snapshot_session.rs:151-158` chains `runtime_env().config_entries()` into the inventory; `:635-646` copies it into `SessionSemantics.settings`; `attempt.rs:52-54` embeds it in `MemberAttempt`; `:160-166` compares by whole-struct equality; the entries include `datafusion.runtime.temp_directory` (a path) and every cache limit | A retry or recovery from a process with a different spill directory, memory limit or cache limit is refused as `IdentityReused` for a write that committed (§5); adopting F6 invalidates every outstanding attempt | Classify `datafusion.runtime.*` as resource policy: keep it in read-back and in the `information_schema` consistency check (`:780-800`); exclude it from `SessionSemantics.settings` and `settings_hash`. Surface: `session/config.rs` and the three inventory sites | Negative unit test: two factories differing only in `datafusion.runtime.*` yield equal `SessionSemantics` and `settings_hash`; a `time_zone` difference still differs. Lifecycle test: a receipt written under spill dir A reconciles under spill dir B |
| F2 | Fixed-point rounds re-plan and re-bind every relational operation every round | DM-26, DM-29, DM-38, DM-39 | `rounds.rs:55-63` `compile_query` per evolving rule and query per round; `:140-158` an executed emptiness probe per rule per round; `native_state.rs:112-128` every candidate goes through `prepare_rule_plan(..).execute(..)`; `:167-262` four plans per head per round; `:100-110` and `roles.rs:129-153` the workspace is rebound as fresh `CandidateTable` providers each round; `preparation.rs:96-105,107-175` each prepare re-binds targets, opens an execution scope (a new `SessionState` and `RuntimeEnv` when a byte policy applies, `resources.rs:150-168`) and re-runs analyzer and optimizer; `preparation.rs:339` physical planning per execute. Blueprint §6.15.2 already describes the stratum as owning private relations for facts, new facts and support with native code scheduling rounds. DataFusion supplies `reset_plan_states` for exactly this re-execution (`execution_plan.md:439-454`) and uses it in `RecursiveQueryExec` over `WorkTableExec` | Round cost is planning-bound for small models: with 130 native rules the all-rule binding receipt is 10–12 s (Plan 08, 2026-09-17), and the interactive edit loop pays a proportional share on every round of every stratum; the cost scales with rules, not with data | Prepare each stratum once: per (rule, query, delta variant) a physical plan whose evolving leaves are PSE-owned round-buffer nodes (a `TableProvider` whose exec reads the stratum's current `FieldCheckedBatch` at `execute` time and implements `reset_state`); per round, publish the new buffers, `reset_plan_states`, execute; keep the emptiness probes and merge plans prepared the same way. The rule engine profile must refuse dynamic-filter pushdown for these plans and either pin statistics-independent physical rules or declare the round-one physical choice as an approximation policy (DM-24). `WorkTableExec` is the precedent, not a reusable part: `WorkTable::update` is private at this pin | The existing rule fixtures as the oracle (identical outputs); a per-round plan-count metric in the execution trace; a Criterion bench `fixed_point_rounds` over a representative stratum, cold and warm; a negative test that a dynamic-filter profile is refused |
| F3 | Delta log state is rebuilt from scratch on every open and never checkpointed outside destructive maintenance | DM-26, DM-33, DM-36, DM-39, DM-16 | Every open is `DeltaTableBuilder::load()` (`provider.rs:28-32,67-71`); a publication open loads the control table, every member and every input (`publication.rs:147-181,184-260,278-296`); a write loads before and after (`write.rs:405-410,466-471`); CDF loads twice and scans commit JSON (`changes.rs:56-96`); maintenance loads per protected version (`maintenance.rs:392-396`). Every PSE commit sets `with_create_checkpoint(false)`; the declared table policy sets no interval (`relation.rs:219-223`); delta-rs checkpoints only under `create_checkpoint` (`transaction/mod.rs:1159-1165`). delta-rs provides seeded incremental update (`kernel/snapshot/mod.rs:291-350,941-974`; `table/mod.rs:207-230`), a validated snapshot identity, `without_files`/`with_skip_stats` load classes (`builder.rs:140-150`) and provider construction from a retained snapshot (`table_provider.rs:299-305`); none is used | Open cost is O(commits since the last maintenance run) per table × (1 + members + inputs) per open, unbounded between maintenance runs; the only way to bound it is the exclusive, destructive-adjacent maintenance command. Verify-only loads (`contract.verify` reads schema and configuration only, `contract.rs:231-256`) still materialize every add action with statistics | (a) Declare a checkpoint policy in the registry table policy and enable `create_checkpoint` on the publication-root commit path under the shared writer lease, leaving `cleanup_expired_logs` false; (b) retain snapshots per `SnapshotIdentity` + `DeltaTableConfig` in a bounded, pool-accounted store owned by `SessionFactory`, refreshed with `update_incremental`, invalidated for a root when a maintenance lease is acquired; (c) type the load class — Query (files, stats), Metadata (`without_files`), Maintenance (files, `skip_stats`) — and bind Metadata for verify-only loads; (d) pursue kernel checksum writing and incremental CRC replay upstream (F9) so that every open is O(1) in commits | Criterion bench `delta_open`: M ∈ {1, 8, 32} members after K ∈ {1, 100, 1000} commits, checkpoint on/off × retention on/off, conditions recorded; metamorphic oracle: retained versus fresh snapshot yield identical `add_actions_table` and scan results (Q13); reader-pin race: vacuum after retention → missing-file refusal, never stale rows |
| F4 | Decoded relations are never resident across invocations or Python handles | DM-26, DM-36, DM-39 | A handle retains its publication until `close()` (`handles.rs:40-62`); a second `pse.open` re-opens and re-decodes; every compiler invocation over the same published inputs decodes the same members again; `MaterializedTable`/`CandidateTable` already hold `OwnedRecordBatch`s by `Arc` (`materialized.rs`, `candidate.rs:17`) | Repeated interactive operations over one published model pay decode cost proportional to member bytes on every invocation | A process-resident relation tier owned by `SessionFactory`, keyed by (`table_root`, version, projection, `schema_force_view_types`), holding `OwnedRecordBatch`s reserved from the pool, LRU by bytes, invalidated on the maintenance lease for the root. Route `NativeCacheFactory` to it when the input plan is a pure scan of a pinned-version Delta member with no volatile expression (exact identity, so sound under DM-32); otherwise invocation-local as today. This is the sound form of the overview's tier idea | Metamorphic: resident versus fresh scan equal at the same version; the `delta_open` bench with a decode variant; pool report shows resident bytes |
| F5 | The dependency model cannot distinguish a value-only change from a structural one | DM-33, DM-26, DM-31 | `NativeDependencyKind` = operation, input, contract, function, rule, setting, policy, provider, scope, observation; `native_dependencies.selection` is a whole-member selection (`pse-schema/src/catalog/publication.rs:49-78`); no column-level or value-class dependency; Plan 08 E09 says "add no new simulator workflow" | A parameter sweep — the central simulator workload — invalidates every structural artifact for each value change; DM-33's assessment test ("does a small case change trigger a needless full rebuild") fails by construction | Extend the dependency contract now, not the workflow: give `native_dependencies` a consumed-column projection (or a `value` kind whose fingerprint covers only declared value columns), and let `AlgorithmSpec` signatures declare which input columns are structural. Sweeps then reuse every structural artifact and recompute only the numerical program and outcome. The sweep operator itself stays in a successor plan | Q13 oracle: value-only change → structural artifacts reused byte-identically, numerical outputs recomputed; structural change → full invalidation; a negative test that a consumer reading an undeclared column is refused |
| F6 | Cache policy is neither declared, accounted nor observable | DM-31, DM-39, DM-50, DM-59 | `env.rs:43-46` configures pool, temp path and temp size only; `:47-56` reads back pool and disk only; `budget.rs:31-50` has no cache field; DataFusion defaults are 50 MiB + 20 MiB + 1 MiB; `DefaultCache` is not a `MemoryConsumer`; no table function or report row exposes `list_entries()` hit counts, replay counts or retention hits (no `TableFunctionImpl` in product code; `register_udtf` exists at this pin) | The budget's claim "everything one process may consume" (`budget.rs:29`) is overstated by up to 71 MiB today and by the resident and retention tiers tomorrow; a user cannot see why an open was slow | `CacheBudget` in `ResourceBudget`; a pool-reserving `Cache<K,V>` wrapper over `DefaultCache` (reserve on `put`, release on `remove` and eviction) so cache bytes are inside the accounted budget; read-back through `CacheManager::get_*_limit`; a `pse_cache_entries` table function over `list_entries()` plus replay and retention counters in the execution trace | Read-back unit test; `report()` shows cache bytes; governance grep for a second `RuntimeEnvBuilder::new()`; a test that the table function is non-mutating |
| F7 | The overview's tiered `CacheFactory` and persisted snapshot cache would hide effects and create a second authority | DM-20, DM-28, DM-02, DM-23 | Overview §7 ("large result → cached Parquet/Delta representation", "known reusable model → catalog-backed materialized relation") and §12 (`DeltaSnapshotCache` on "persistent local/object storage"); delta-rs `serde.rs:117-125`; Plan 08 E09 ("No serialized physical plan or object hash alone certifies semantic reuse") | `df.cache()` would become an effectful publication with no commit boundary; a persisted snapshot would answer "which files are active" without the log validating it; checkpoints already give the cross-process warm start with protocol validation | Drop both; adopt F4 for the in-process tier and F3a/F9 for the durable one | An `ast-grep` rule: no `impl CacheFactory` outside `session/cache.rs`; no `Snapshot`/`DeltaTable` serde in product code |
| F8 | The overview is an unlabeled, unpinned narrative filed among capability maps | DM-55, DM-59; docs rule | No front matter, no `pins:`/`regenerated:`, no evidence labels; cites "current delta-rs main" without a commit; claims `drop_table_entries` gives "standard table-scoped invalidation" for the runtime caches, which holds only for `TableScopedPath` keys, not the `Path`-keyed metadata cache Delta uses (Interface-checked from the `CacheKey` implementor list); omits seeded incremental update, `with_snapshot`, checkpoint policy, `reset_plan_states`, `config_entries` read-back, the predicate cache and the kernel's unreachable CRC replay | An agent finds it under `docs/capability-maps/` and cites it as evidence | Rewrite it as a pinned capability map carrying the corrections and omissions from this review, or move it to `docs/design_review/evidence/` | `just docs`; manual front-matter check until API-doc lint lands (register R-20) |
| F9 | Two delta-rs limitations block the best replay path, and both have an upstream route | DM-43, DM-31, DM-59 | CDF: `load_cdf.rs:49,93-96` builds a private `DefaultCache::new(1 MiB)` and the builder exposes no session or runtime setter. Replay: the kernel's `IncrementalReplay` and `write_checksum` (`kernel/src/snapshot/builder.rs:63-93,177-183`, `snapshot/mod.rs:850-874`) are never used by delta-rs and `Snapshot` has no public constructor from a kernel snapshot. delta-rs is pinned to a commit in `Cargo.toml:123`; the skill index and `struct_schema_review.md` pin the same commit | CDF footers are re-read per builder (no correctness effect); every open replays commits since the last checkpoint rather than reading a version checksum | Upstream two small changes (a runtime cache parameter on `CdfLoadBuilder`; builder options on `Snapshot::try_new_with_engine` plus checksum writing in the post-commit hook) and re-pin; a fork branch pin is permitted by AGENTS.md (delta-rs is not one of the four pinned families) but costs a skill-index rebuild (`build/README.md`) and a pins update — prefer upstream | The `delta_open` bench with a CDF variant sets the urgency; re-pin verified by `just family-check` and a rebuilt index |
| F10 | Two seams the plan excludes are needed by the target and cost nothing to declare now | DM-58, DM-43, DM-39 | Remote storage: Plan 08 marks "remote catalog deployment" out of scope (`08-…:63-66`) and SP12 refuses remote exclusion guarantees (`:1234-1235`); `object_store` 0.13.2 has no caching store; delta-rs's dormant `delta-cache` feature was the same idea. Pushdown: `DeltaScanConfig::new_from_session` inherits `parquet.pushdown_filters` (`table_provider.rs:200-209`), which is `false` by default and unset in `config.rs::build`, so Delta scans decode whole files and `max_predicate_cache_size` is inert | Adding shared storage later would otherwise mean re-plumbing the runtime; selective predicates over wide expression relations decode everything today | Declare the object-store cache seam in SP08's runtime row: a caching `ObjectStore` decorator (a hybrid memory/disk cache such as `foyer`) registered in the runtime registry for non-local roots, covering log JSON, checkpoints and Parquet for every consumer; build it when a remote root enters the envelope. Make `pushdown_filters` and `max_predicate_cache_size` explicit rows of the execution settings, enabled per the Q09 measurement | A `delta_scan_pushdown` bench variant; the seam is a declaration until a remote root exists |
| F11 | *Observation.* `NativeCacheExec` emits one partition | DM-36, DM-39 | `cache.rs:299` `Partitioning::UnknownPartitioning(1)` | Operators over shared inputs run single-partition unless repartitioned; hypothesis | None until Q06/Q14 measurements; if measured, emit `RoundRobinBatch(target_partitions)` over cached batches | Q14 cost envelope |

**Applicability.** Groups 7 and 3 carry F1 and the reuse keys of F2–F5; group 6
(DM-26, DM-29) carries F2–F4; group 1 and DM-20/DM-28 carry F7; group 8 bounds every
claim and carries F10/F11; group 10 through DM-50 carries F6's observability; group 12
carries the proportionality judgment that is now inverted for F2–F5 and F10 (the
simulator workload demonstrates the need). Group 2 does not bear: no plane introduces
a value distinction. Group 4 bears through DM-16 and DM-19 (checkpoint policy and load
class as declarations). Group 5 bears only through DM-24 (F2's physical-choice
approximation). Group 9 bears through DM-43 (F9, F10). Group 11 bears through
DM-53/DM-54 in §9.

**Strengths, stated as what would break without them.** Without
`RequireSessionState` at every delta-rs call, delta-rs would build its own default
session and discard the shared pool, the metadata cache and every registered function
(`session.rs:160-176` warns exactly this). Without `execution_scope` sharing the
parent's `cache_manager` (`resources.rs:160`), every byte-scoped execution would start
with an empty footer cache. Without the identity-pointer key and the retained `Arc<()>`
(`cache.rs:156-174`), a freed identity's address could be reused within one invocation
and two unrelated cache nodes would share a completion. Without the fixed-point
operator being a single `ExecutionPlan` node (`strata/native.rs:44-51`), F2's prepared
stratum would have no owner.

**Maturity, qualitative.** The invocation cache is at rating 3 within its scope.
Runtime cache configuration is at rating 1. Delta log-state reuse, prepared-round
reuse, resident relations and value-class dependencies are at rating 0 for
capabilities the libraries supply or the schema can express and the codebase does not
use; nothing is claimed for them, so these are gaps, not misrepresentations.

## 8. Alternatives and architectural leverage

| Alternative | Semantic duplication and extension locality | Correctness and operational risks | Implementation / maintenance cost | Performance evidence | Why selected or rejected |
|---|---|---|---|---|---|
| **Current baseline** | One runtime, one factory; cache ceilings implicit; load class implicit; rounds re-planned; whole-member dependencies | F1 spurious refusals; unbounded replay cost; planning-bound rounds; sweeps recompute everything | Zero | None | Rejected as final state; retained as the base |
| **Overview as written** | A "CachingPolicy" root duplicating `ResourceBudget`/`SharedRuntime`; a size-tiered `CacheFactory` with persistence (a hidden policy); a serialized-snapshot store parallel to checkpoints; a Parquet-only byte cache; a CDF fork | G1/G4 failures (F7); persisted state without log validation | High: a subsystem, an extension planner, a persistence format | None cited | Rejected except where it restates the baseline |
| **Target (selected)** | The budget is the one declaration of ceilings; the registry table policy is the one declaration of checkpoint interval; the load class is one typed enum; retention and residency are two bounded maps keyed by Delta's own identity; the stratum is one prepared program; the dependency row gains one projection | One new invalidation edge (maintenance lease → drop retention and residency for the root); one new precondition (`reset_plan_states` limits) enforced by the rule profile | Moderate: F1, F6 and F3a are single-crate edits; F2 is a `pse-rules` restructuring with the fixture set as its oracle; F3b, F4 and F5 are one module each | Hypothesis until the three benches in §9 exist | Fewest independent semantic decisions; every element is a native facility or a declaration; no new authority |

**Cache and reuse planes under the target.** This is the best-in-class utilization
the maintainer asked for, stated as contracts rather than machinery:

| Plane | Mechanism (canonical path) | Declared where | Key and validation | Invalidation | Effect declared | Label |
|---|---|---|---|---|---|---|
| Parquet metadata | `datafusion_execution::cache::cache_manager::FileMetadataCache` via delta-rs's next scan | `ResourceBudget.cache.metadata_bytes` → `RuntimeEnvBuilder::with_metadata_cache_limit`; read back; pool-accounted wrapper (F6) | `Path` + `ObjectMeta` | LRU; none by table | None (read) | Interface-checked; F6 makes it Implemented |
| File statistics, listings | `FileStatisticsCache`, `ListFilesCache` | Same budget, at DataFusion's defaults | `TableScopedPath` | `drop_table_entries` | None | Declared only; no consumer |
| Invocation sharing | `NativeCacheFactory` + `NativeCacheExec` (`session/cache.rs`) | Installed by `SessionFactory::from_builder` unless the caller supplies a factory | Identity pointer + plan | Dropped with the invocation | `OperationEffect::Read` | Implemented, Tested |
| Process-resident relations (F4) | `OwnedRecordBatch`s behind a `MaterializedTable`-shaped provider; `NativeCacheFactory` routes pure pinned-version scans here | A bounded, pool-accounted map owned by `SessionFactory`, sized in the budget | (`table_root`, version, projection, view-type flag); Delta version immutability | Maintenance lease for the root; LRU by bytes | None (read) | Proposed |
| Prepared stratum plans (F2) | Physical plans over PSE round-buffer nodes; `datafusion_physical_plan::execution_plan::reset_plan_states` per round; `RecursiveQueryExec`/`WorkTableExec` as the in-tree precedent | Built once by the fixed-point operator at first execution; rule engine profile refuses dynamic filters | (rule id and version, query, delta variant, port binding, engine profile hash, input schemas) | Dropped with the stratum; rebuilt on any key change | None (read) — the round buffers are the stratum's declared mutable workspace (DM-29) | Proposed |
| Value-class dependencies (F5) | `native_dependencies` with a consumed-column projection; `AlgorithmSpec` structural-column declarations | Registry | Fingerprint over consumed columns only | E09 native reuse queries | — | Proposed |
| Delta log state, in process (F3b) | `deltalake_core::kernel::snapshot::Snapshot` retained per `SnapshotIdentity` + `DeltaTableConfig`; refreshed by `DeltaTable::update_incremental` | A bounded, pool-accounted retention owned by `SessionFactory` | delta-rs `is_cache_for` + policy | Maintenance lease for the root; LRU by bytes | None (read) | Proposed |
| Delta log state, durable (F3a, F9) | Checkpoints now; kernel version checksums when delta-rs surfaces them | Registry table policy (`delta.checkpointInterval` or a PSE interval) + `create_checkpoint` on the root commit | Protocol | Never (append-only); log cleanup stays with maintenance | Write to `_delta_log` (an admitted publish effect) | Proposed |
| Load class (F3c) | `DeltaTableBuilder::without_files`, `with_skip_stats` | A typed enum at each open site | — | — | — | Proposed |
| Predicate cache (F10) | `datafusion.execution.parquet.max_predicate_cache_size` | Execution settings, with `pushdown_filters` explicit | — | — | — | Proposed; Q09 measures |
| Byte cache seam (F10) | A caching `ObjectStore` decorator registered in the runtime registry for non-local roots | SP08 runtime row; built when a remote root exists | Object path + `ObjectMeta` | LRU / TTL | None | Declared seam |
| CDF metadata (F9) | delta-rs private cache until upstream accepts a parameter | — | — | — | — | Upstream route |
| Observability (F6) | `pse_cache_entries` table function over `list_entries()`; replay, retention and residency counters in the execution trace | Session functions | — | — | None (read) | Proposed |
| `delta-cache` (`foyer` inside delta-rs) | — | — | — | — | — | Do not enable; dormant |

**Abstractions justified by current needs.** The retention map (F3b), the resident
map (F4) and the prepared stratum (F2) are the only new structures. Each is justified
by a named workload — cold open, interactive reopen, fixed-point rounds — and each is
gated by the bench that measures it. The dependency projection (F5) is a schema
column, not a structure.

**What remains ordinary code.** The invocation cache's collect-and-spill loop, the
attempt inspector's commit scan, delta-rs's replay, the round scheduler and the
round-buffer node: none of these should become declarations.

## 9. Verification and measurement plan

| Claim or risk | Evidence label | Test / analysis / benchmark | Conditions and expected result | Current result or remaining gap |
|---|---|---|---|---|
| Runtime resource entries never enter semantic identity (F1) | Proposed | Unit test in `pse-catalog::session::config`; lifecycle test reconciling a receipt across two budgets | Equal `SessionSemantics` for factories differing only in `datafusion.runtime.*`; recovery succeeds under a different spill directory | Not written; today refused |
| A prepared stratum equals a freshly planned one (F2) | Proposed | The existing `pse-rules` fixtures run through the prepared engine; per-round plan-count metric; Criterion `fixed_point_rounds` on a representative stratum, cold and warm; negative test that a dynamic-filter profile is refused | Identical outputs; plan count per round drops from O(rules) to zero after round one; wall time per round measured | Not written; no bench exists |
| Declared cache ceilings are what the runtime holds and reserves (F6) | Proposed | `SharedRuntime::build` read-back; pool report row; table-function non-mutation test | Limits equal the budget; cache bytes visible and reserved | Not written |
| A metadata-cache hit cannot serve a different file's footer | Interface-checked | Rely on `is_valid_for`; negative test overwriting a path with a different-size file | Miss on size change | Documents the write-once dependency |
| Retained snapshot and resident relation equal fresh load (F3b, F4) | Proposed | Metamorphic test (Q13) comparing `add_actions_table`, scan results and resident batches with a fresh `load()` and scan | Byte-equal batches, identical rows | Not written |
| Vacuum cannot be served from retention or residency (F3b, F4) | Proposed | Reader-pin race (Q13): acquire maintenance lease, vacuum, attempt scans from both tiers | Missing-file refusal; never stale rows | Not written |
| Checkpoint creation is neutral for readers (F3a) | Interface-checked | delta-rs `refresh_same_version_checkpoint_if_needed`; a PSE test that a reader at *v* observes identical rows before and after a checkpoint at *v* | Identical rows | Not written |
| Value-only change reuses structural artifacts (F5) | Proposed | Q13 oracle: value change → structural members byte-identical to the clean recompute, numerical members recomputed; structural change → full invalidation; undeclared-column read refused | As stated | Not representable today |
| Replay dominates publication open (F3) | **Hypothesis** | Criterion `delta_open`: M ∈ {1, 8, 32} members × K ∈ {1, 100, 1000} commits × {checkpoint, retention, residency} on/off; record machine, profile, `log_buffer_size` | Decides F3b/F4 build order | No bench exists |
| Pushdown and the predicate cache pay for wide expression relations (F10) | Hypothesis | `delta_scan_pushdown` bench variant with selective predicates | Decides the Q09 setting | Unmeasured |
| Single-partition cache output costs downstream parallelism (F11) | Hypothesis | Q06/Q14 measurement with a shared input consumed by two aggregations | Decides whether to repartition | Unmeasured |
| Invocation cache shares one computation and releases reservations | **Tested** | `just unit-package pse-catalog 'test(session::cache::tests::)'` | 4 passed, 0 failed, baseline 0 (Plan 08 receipt, 2026-09-17; not re-run here) | Holds at that grain |

**Cost accounting.** Metadata cache: bounded and, after F6, reserved. Retention:
add-action batches with raw statistics JSON per retained version — bound by bytes,
reserved. Residency: decoded member bytes — the largest new footprint; bound by bytes,
reserved, and cheaper than the pool's own working set only if reopen frequency is high,
which the `delta_open` bench decides. Prepared plans: physical plan objects per rule,
small; the round buffers replace the per-round providers rather than adding to them.
Replay: commits since checkpoint × `log_buffer_size` concurrent reads per open.
Checkpoint: one Parquet write per interval on the root commit path. Inspection:
`list_entries()` is O(entries) and belongs in the table function, not the hot path.

## 10. Exceptions, unresolved decisions, and the rules and policies the target requires to change

A SHOULD-level deviation is recorded with its trigger. A rule or policy that stands
between the current code and the target is recorded with the change it needs and the
sanctioned route for making it; none of these is a reason to narrow the design.

| Item | Rule or policy in the way | What must change | Route and owner |
|---|---|---|---|
| Prepared stratum plans and round buffers (F2) | None in the charter — this *is* DM-26 and DM-29. Plan 08 E04 keeps "the outer finite fixed-point operator … for its domain semantics" and the plan preserves existing outcomes; the change is internal to that operator. The `reset_plan_states` limits (no dynamic filters, no recursive query) constrain the rule engine profile | Add a profile rule that refuses dynamic-filter pushdown for prepared stratum plans; record the round-one physical choice as an approximation policy or pin statistics-independent physical rules | Ordinary PR under E04/E07 with the fixture oracle; maintainer |
| Value-class dependencies (F5) | Plan 08 E09: "Preserve existing edit semantics, add no new simulator workflow"; SP12 dependency text names whole selections | Amend E09/SP12 to admit a consumed-column projection in `native_dependencies` and structural-column declarations on `AlgorithmSpec`; keep the sweep *operator* in a successor plan | Plan amendment (Plan 08 is living); no ADR, since D14 already says reuse follows declared dependencies; maintainer |
| Remote byte-cache seam (F10) | Plan 08 out-of-scope list (`08-…:63-66`) and SP12 "no remote platform is built" (`:1234-1235`) | Keep the exclusion for *building* remote support; add the object-store decorator seam to SP08's runtime row so the local design does not preclude it | Plan amendment; maintainer |
| Checkpointing outside maintenance (F3a) | A convention, not a rule: every PSE commit sets `with_create_checkpoint(false)`; SP12 already permits "native Delta … checkpoint" | Replace the convention with a declared table policy evaluated on the root commit path | Ordinary PR under SP07/SP12; the policy participates in registry identity like the existing table policies |
| Upstream or fork changes to delta-rs (F9) | AGENTS.md pins delta-rs to a commit; the deltalake skill index and `struct_schema_review.md` carry the same pin. delta-rs is not one of the four pinned families, so re-pinning is an ordinary dependency change | Contribute the CDF cache parameter and the snapshot builder options and checksum writing upstream, then re-pin; if the upstream lag is unacceptable, pin a fork branch and rebuild the skill index (`build/README.md`) and the pins in the capability maps | Ordinary PR plus skill rebuild; maintainer decides upstream-versus-fork on the `delta_open` numbers |
| Cache bytes inside the accounted budget (F6) | ADR-0046/ADR-0068: "Limit guarantees apply to accounted consumers, not arbitrary allocator/solver memory"; caches are currently in the unaccounted set by omission | Narrow the exclusion: caches, retention and residency reserve through the pool; the remaining exclusion is the global allocator and solver processes | Short ADR under the accepted resource decision (a contract change within an accepted decision); design-review at maintainer discretion |
| Three-tier reuse replaces the stage memo | Blueprint §14.3 "Complete stage memo" and §3.2 "artifact-hash memo" still describe the deleted memo; the blueprint is off-limits to edits in passing | Supersede with the invocation / resident / durable-typed-dependency model in a `design:` PR carrying a revision row, alongside ADR-0069's acceptance | The sanctioned route; no rule change, an obligation Plan 08 SP00 already anticipates |
| Reviews deferring native capabilities for lack of a present consumer | Charter DM-58 and the directive's "do not build an unnecessary platform", as applied in the first pass of this review | No rule change: the target planes are native facilities and declarations, not platforms; the proportionality judgment must weigh the simulator workload, not only the code that exists today | Review practice; recorded here so the next review does not repeat the narrowing |
| Byte-level Parquet-only `AsyncFileReader` cache (overview §10) | — | Superseded by the object-store seam (F10), which covers log, checkpoint and data reads for every consumer | Rejected as the wrong layer, not as out of scope |
| Kernel incremental CRC replay | Unreachable through delta-rs at this commit | Covered by F9's upstream route | Revisit when delta-rs surfaces it |
| Conservative recompute before E09 (Plan 08 E07) | Already recorded by the plan | F1 must land before E09 chooses the dependency identity; F5 must land before E09 populates `native_dependencies` | Ordering only |

## 11. Decision and implementation changes

**Decision:** Revise the overview; adopt the target in §8; make the policy changes in
§10 through their sanctioned routes.
**Reason:** The current code's cache planes are sound and non-competing (G1–G6 pass);
the overview's two persistence tiers are the only proposals that would fail a gate,
and the overview fails G7 as evidence. The material gaps are four instances of one
cause — preparation not separated from repeated execution — in the rule engine, the
Delta open path, the Python boundary and the dependency model, each answered by a
native facility or a declaration and each gated by a benchmark that does not yet exist.

| Priority | Change | Principle IDs | Acceptance evidence | Regression protection |
|---|---|---|---|---|
| Correctness first | F1 — exclude `datafusion.runtime.*` from `SessionSemantics` and `settings_hash`; keep it in read-back and the `information_schema` check | DM-32, DM-33, DM-13 | Negative unit test; lifecycle reconciliation across budgets | The unit test; an `ast-grep` rule that the semantic inventory is built from `config_options().entries()` only |
| Correctness first | F7 — record in the overview's replacement that `CacheFactory` never persists and no serialized snapshot is persisted | DM-20, DM-28, DM-02, DM-23 | Governance grep | `ast-grep` rules named in §7 |
| Correctness first | F2 precondition — profile rule refusing dynamic-filter pushdown for prepared stratum plans; physical-choice policy declared | DM-24, DM-43 | Negative test | The test |
| Semantic leverage next | F6 — `CacheBudget`, pool-reserving cache wrapper, read-back, report row, `pse_cache_entries` table function | DM-31, DM-39, DM-50, DM-59 | Read-back test; report shows cache bytes; non-mutation test | Governance grep for a second `RuntimeEnvBuilder::new()` |
| Semantic leverage next | F5 — consumed-column projection on `native_dependencies`; structural-column declarations on `AlgorithmSpec` | DM-33, DM-26, DM-31 | Q13 value-only versus structural oracle | Registry identity test; the oracle |
| Semantic leverage next | F3a — declared checkpoint policy on the publication-root commit path | DM-16, DM-14, DM-26 | Reader-neutrality test; policy participates in registry identity | Registry identity test |
| Semantic leverage next | F3c — typed load class; Metadata class for verify-only loads | DM-19, DM-26 | Each open site names its class; `contract.verify` succeeds on a `without_files` table | Unit test per class |
| Semantic leverage next | F8 — rewrite the overview as a pinned capability map or move it to review evidence | DM-55, DM-59 | Front matter present; claims labeled and corrected | `just docs` |
| Measured performance where justified | F2 — prepared stratum plans over round-buffer nodes with `reset_plan_states` | DM-26, DM-29, DM-38, DM-39 | Fixture oracle identical; `fixed_point_rounds` bench shows planning removed from rounds | The fixtures; plan-count metric asserted in a test |
| Measured performance where justified | F3b and F4 — bounded, pool-accounted snapshot retention and resident relations with lease-driven invalidation; `NativeCacheFactory` routing for pinned-version scans | DM-26, DM-32, DM-33, DM-36, DM-39 | `delta_open` bench shows replay or decode dominating; Q13 metamorphic and race tests pass | The Q13 tests; the bench in `bench-smoke` |
| Measured performance where justified | F9 — upstream (or fork) delta-rs changes for the CDF cache and kernel checksums; re-pin | DM-43, DM-31 | `delta_open` CDF variant; rebuilt skill index | `just family-check`; pins in the capability maps |
| Measured performance where justified | F10 — explicit `pushdown_filters` and `max_predicate_cache_size` rows; object-store cache seam declared in SP08 | DM-58, DM-43, DM-39 | `delta_scan_pushdown` bench | Settings inventory test |
| Measured performance where justified | F11 — repartition cached output only if measured | DM-36, DM-39 | Q06/Q14 measurement | — |

**Final check.** The claims match the evidence: library behavior is Interface-checked
at the named pins, the invocation cache is Tested at the named grain, nothing is
Measured and nothing above says otherwise. The scope matches the implemented
guarantees: durable semantic reuse is left to E09, and the two new reuse planes are
Proposed behind named oracles. Later extensions have a path: a new cache ceiling is a
budget field; a new load class is an enum arm; a new Delta plane starts from delta-rs's
own identity; a new structural consumer declares its columns; and a remote root plugs
into a declared registry seam.
