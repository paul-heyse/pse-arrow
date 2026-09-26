---
title: Native DataFusion and Delta cache capabilities at the selected pins
status: active
date: 2026-09-17
---

# Native DataFusion and Delta cache capabilities

**Interface-checked; implementation described below is not performance certification.**
This replaces the speculative cache-tier overview. Execution authority is
[Plan 09](../plans/09-native-caching-and-pivot-completion.md) and the proposed
[ADR-0070](../adr/0070-native-cache-lifetimes-and-consumed-inputs.md).

## Versions and reproducibility

DataFusion 55.1.0, Arrow/Parquet 59.3.0 and object_store 0.13.2 resolve in the root
Cargo lockfile. Delta upstream is `58f07cd62bfbce3649a7e1c87c696288068ae184`, with
kernel `8ba063f8f84fec222000f66d40d70911d7c79675`. The selected Delta core/derive
source is a reproducible repository override, not an unmodified upstream release.
[The patch manifest](https://github.com/paul-heyse/pse-arrow/blob/main/tooling/delta-native-seams.md) describes the additions;
`vendor/delta-rs/PROVENANCE.json` supplies per-file and patch hashes. Run
`just delta-source <immutable-upstream-checkout>` to reproduce and verify it.

The local [DataFusion skill](https://github.com/paul-heyse/pse-arrow/blob/main/.codex/skills/datafusion/SKILL.md) indexes the
upstream API. The [Delta skill overlay](https://github.com/paul-heyse/pse-arrow/blob/a46f358bdfc2ca27f9f240ab6c045b63141c3ee9/.codex/skills/deltalake/content/overlays/pse-native-cache-seams.md)
distinguishes selected-source additions from upstream symbols.

## Capability and production routing

| Plane | Pinned native capability | PSE use and limitation |
|---|---|---|
| File metadata | `CacheManagerConfig`, `CachedFileMetadataEntry`, `DefaultCache<Path, _>` | Pool-reserved capacity envelope and storage-generation namespace. Path alone is not an object-store identity. Ordinary Delta and patched CDF share the qualified cache. |
| Statistics/listing | Native `TableScopedPath` caches, byte LRU and TTL | Declared ceilings and finite listing TTL. No duplicate Delta active-file inventory. Unqualified stores have cache reuse disabled. |
| Invocation results | `CacheFactory`, logical extensions, native execution and `SpillManager` | Lazy pure input, invocation-owned completions, explicit round epochs; no storage writes from planning and no durable identity inferred from a physical plan. |
| Prepared rounds | `reset_plan_states`, conservative source statistics | Prepare once; reset actual physical state per drained epoch. Inner plans reject dynamic filters, recursive queries and unsupported reset contracts. Ordinary scan pushdown remains available. |
| Snapshots | Native Snapshot/EagerSnapshot, exact load and forward `load_version` update | Query/metadata/maintenance capabilities remain distinct. Exact cached version plus uncached latest observation and fresh reader lease; no serialized snapshot authority. |
| Resident relations | Native selected scans, owned Arrow buffers, `DefaultCache` | Lazy complete selection below residual filters/limits. Exact schema, member/revision, storage generation, policy and implementation identity. Idle entries own no file lease. Spilled/oversize results are not retained. |
| Durable equivalence | Native grouped-count bidirectional set difference | Compare actual consumed columns plus keys, membership and multiplicity. Complete bounded CDF images can reduce the comparison. Unknown/opaque consumers stay whole-input; missing evidence never means equality. |
| Replay persistence | Delta checkpoint hooks and kernel checksum facilities | Registry checkpoint interval applies to control/member commits. CRC replay and checksum cadence are explicit resource policy, initially disabled pending measurements. CRC does not replace Add-file replay. |
| Scan decoding | Parquet pushdown, page indexes, row-group/bloom pruning, predicate cache | Typed pushdown policy and per-reader reservations. Dynamic-filter reuse restrictions are separate. Footer/predicate tuning requires measurements. |
| Object bytes | `RuntimeEnv.object_store_registry` and `DeltaTableBuilder::with_storage_backend` | One actual registered decorator receives log/checkpoint/data IO and conditional writes. No remote byte-cache backend is claimed. |
| Inspection | Native table functions and plan metrics | `pse_cache_statistics()`, `pse_execution_statistics()`, `pse_cache_entries(limit)`. Binding/EXPLAIN do not enumerate caches. Details hold an insertion guard, reserve for the sampled inventory extent, and may refuse. |

## Accounting and validity

`ResourceBudget.cache` / `CacheBudget` is the single deployment declaration.
`NativeCacheService` is shared by factories and scoped sessions. There is no second
`CachingPolicy`, persistent result cache, bespoke LRU or custom Delta replay engine.
Resource settings remain visible in read-back but are excluded from semantic identity;
unknown settings remain conservative semantic inputs.

Native file-cache capacities are reserved before construction. Their upstream Arc
clones escape the cache and do not expose per-reader accounting; report those fields
as unavailable. Snapshot and resident owners keep their reservations through eviction.
Resident buffer accounting deduplicates actual allocation addresses across retained
batches; exported buffers retain the native owner. Allocation addresses are never
semantic identities or persisted keys.

Snapshot replay and decoded fills share one bounded load gate and staging policy.
Native allocators do not offer reserve-before-every-allocation hooks: staging limits
bound admitted work, not arbitrary allocator peaks. Measure RSS separately. Native
snapshot extents are conservative estimates, not exact allocator accounting.

Cache statistics distinguish policy ceiling, reserved capacity, retained native extent,
all live values, pinned readers, active loads and staging. Upstream eviction/replay/
decode counters without a deployment-wide hook are NULL, not fabricated zeros.
Native physical-plan metrics remain the source for reader/operator observations.

Entry listing clones native inventories. An outer SQL LIMIT cannot bound that clone;
the function's limit and inspection reservation govern it. Refusal is explicit, and
inspection does not issue storage IO, increment lookup hits or populate entries.

## Maintenance and recovery

Native Delta fence commits precede destructive maintenance on control and affected
member roots under the supported local lease protocol. Invalidation clears retained
entries and rejects late fills; readers keep existing owners until release. Every
new consumer checks the uncached native latest generation. Cache eviction never
removes a data file. Unmanaged external deletion and remote reader exclusion remain
unqualified.

Post-commit snapshot/checksum acceleration failure is reported separately from a
successful mutation. Recovery settles the actual commit/attempt and never repeats its
input to retry a cache. Ordinary writes keep expired-log cleanup disabled; only
explicit admitted maintenance can perform destructive cleanup.

## Evidence boundary

**Implemented:** service, namespace/capacity adapters, snapshot/selection ownership,
prepared epochs, consumed-input contracts, native comparison, CDF injection and CRC
interfaces in the source paths named by Plan 09. **Tested/Measured:** see the current
Plan 09 checkpoint and campaign receipts for exact commands and source boundaries.
This map does not claim that all correctness gates passed or that a speedup was measured.
