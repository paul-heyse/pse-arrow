---
title: Plan 11 implementation improvement pass
status: complete
date: 2026-09-20
adrs: [ADR-0074]
phase: 1
evidence: Implemented H01-H12; Tested 97 Rust and 32 Python isolated units; final integration open
---

# Integrated implementation improvements

This follow-up implements the approved broad improvement pass over I00–I15.
[Plan 11](11-integrated-native-performance.md) remains the scope authority and the
[inventory](11-execution-inventory.md) retains the earlier package receipts.
Source capture: `build/plan11/improvements/start-source.tar.gz`; starting hashes and
scoped status are adjacent. No earlier acceptance receipt qualifies these changes.

| Item | Scope | Status |
|---|---|---|
| H01 | Shared checked-source comparisons and typed interpretation | implemented |
| H02 | Per-entry preparation and retained extension lock scope | implemented |
| H03 | Select missing obligations before construction | implemented |
| H04 | Terminal demand before preparation | implemented |
| H05 | Explicit compact sample retention | implemented |
| H06 | Transparent provider and identity projection capabilities | implemented |
| H07 | Native Arrow lookup rows, generated row access, grouped inventories | implemented |
| H08 | Actual assertion additions and grouped rule phases | implemented |
| H09 | Numerical capability binding and precise refresh invalidation | implemented |
| H10 | Native Delta pruning on narrow resident misses | implemented |
| H11 | Bounded unordered scheduling and settings precedence | implemented |
| H12 | Python equal-schema fast path | implemented |

## Implemented boundary

**H01-H03 — selection and preparation.** `FieldCheckedBatch::same_source` compares
the complete declaration, schema, row domain and actual Arrow array owners. Document
and derived-input caches use that comparison without treating equal values as the
same source. Selected policies and semantic settings are retained typed owners;
Delta resident keys retain the generated member selection instead of JSON or debug
strings. Registry validation compiles under a per-schema cell, document parsing uses
a per-source cell, and extension hits use the retained context's read lock. Related
finite output ports share in-flight preparation as well as their completed producer.

Registry-owned obligation dependencies replace repeated catalog traversal. Catalog
admission selects missing local-value checks before constructing plans; schema and
relational obligations remain. Write-evidence lookup does not allocate an empty
invocation resource. Its attempt/member/version index only locates candidates: the
existing full owner and coverage comparisons still decide eligibility.

**H04-H06 — terminals, ownership and providers.** `TerminalDemand` is selected before
native analysis/optimization. Count uses DataFusion's aggregate; pure existence and
sample requests carry safe limits, with an extra row for sample truncation. Count
cannot be consumed as source existence. Incompatible sample requests are refused.
Required effects still drain, and the existing execution contract protects pending
requirements. Volatile replay preserves demand without aggregating a count twice.

`SampleRetention::Share` remains the default. Explicit `Compact` uses the reserved
Arrow copy and keeps the charged view on resource refusal; cancellation and other
failures propagate. Transparent isolated-target reads forward native scan arguments,
filter support, statistics and existing constraints. Capture forwards scan behavior
but continues withholding unverified constraints/statistics. Identity projections
return the original physical child only when its complete schema agrees; storage
metadata changes still require a native projection.

**H07-H09 — compiler, rules and numerics.** Configuration ordering retains Arrow
`Rows`; it no longer copies each encoded row into a separate byte vector. Generated
`RelationRow::rows_at` admits a checked chunk once and decodes requested positions
in order, preserving duplicates. The generator was changed and all contract outputs
were regenerated through `just codegen-contracts`. Quantity inventories prepare and
execute independent source roots together before dependent unit derivation.
Grouped preparation retains explicit root relation declarations across native
field derivation and checks them again before analysis/optimization.

Rule additions are the actual assertion-id anti-join against current assertions.
Those additions determine affected keys. Independent additions, merged assertions,
representatives and deltas each run as a prepared group; dependent phases advance
only after group settlement. Duplicate-only rounds skip representative/delta work,
while support processing still settles. A retained numerical capability table binds
actual native UDF equality, declared arity and scalar evaluators. Scalar workspaces
invalidate precomputed variable or parameter slot sets only after bitwise input
changes; identical refreshes retain completed demand.

**H10-H12 — resident scans, scheduling and Python.** Narrow cold resident reads
preserve native projection and supported pruning. Filters remain inexact at the
wrapper so the native residual runs on both cold and warm paths. Limits never move
below a residual. Narrow misses stream without filling a complete-selection entry;
only unrestricted full reads populate that cache. Warm complete entries can supply
projected/limited readers. Existing source generations and maintenance leases still
qualify exact selection reuse.

Reusable output groups refill with bounded `buffer_unordered`, settle started work,
and restore declaration order before returning results or restoring reusable owners.
One-shot and reusable groups share result/error collation. Runtime concurrency
precedence is explicit engine settings, then supplied cache policy, then defaults.
Python transfer comparison uses native metadata-aware equality and one flattened
schema when equality covers the representation. Dictionary values and container
extension storage retain the full walk because native equality can omit nested
metadata. Duplicate paths and the existing degradation states remain observable.

## Pinned library evidence

Interface-checked: the [DataFusion skill](../../.codex/skills/datafusion/SKILL.md)
and [Delta skill](../../.codex/skills/deltalake/SKILL.md), their exact-release contracts,
and the locally resolved upstream source informed these changes. The profile is
DataFusion 55.1.0, Arrow/Parquet 59.3.0, object_store 0.13.2 and delta-rs commit
`58f07cd62bfbce3649a7e1c87c696288068ae184`. Native `RowConverter`/`Rows`, logical
aggregate/limit builders, provider scan arguments, Arrow ownership kernels and
delta-rs provider pruning are used within their pinned contracts. No family upgrade,
new dependency, new cache family or new governance control was introduced.

## Verification

Targeted isolated units are sufficient for every implementation and deletion item.
Normal compilation, static checks and pure contract generation accompany them.
The failure baseline is zero. Full integration/component assessment follows all
scope and deletions at I17, through I18; performance characterization follows at
I19. No additional enforcement infrastructure is part of this work.

Tested: **97 distinct Rust isolated units**, development nextest default profile,
with `pse-relations/force-validate`, and **32 Python unit cases**. Zero failures in
the final runs, against baseline zero. Filtered-out tests are outside these receipts.

| Scope | Distinct units | Final logs under `build/plan11/improvements/` |
|---|---:|---|
| Engine selection, ports, ownership, terminals, groups and obligations | 47 | `unit-engine-final.log`, `unit-terminals-final.log`, `unit-group-regression.log` |
| Relations, local validators and selective obligations | 13 | `unit-relations-final.log` |
| Compiler lookup and demand controls | 3 | `unit-compiler-corrected.log` |
| Rule representatives and grouped head epochs | 3 | `unit-rules-corrected.log` |
| Numerical bindings, guards, refresh and sparse evaluation | 15 | `unit-numerics-final.log` |
| Resident provider controls and native Delta boundaries | 14 | `unit-catalog-final.log` |
| Runtime construction and concurrency precedence | 2 | `unit-runtime-final.log` |
| Python schema-transfer observations | 32 | `unit-transfer-complete.log` |

Exact selected unit commands:

```sh
just unit-package pse-engine 'package(pse-engine) and (test(integrated_performance_unit::) or test(improvement_unit::) or test(session::cache::tests::) or test(session::native_operation_unit::) or test(scoped_reuse_unit::))'
just unit-package pse-engine 'package(pse-engine) and (test(preparation::terminal::improvement_unit::) or test(terminal_count_sample_and_existence) or test(physical_input::improvement_unit::))'
just unit-package pse-engine 'package(pse-engine) and (test(reusable_group_refills_slots) or test(related_reusable_roots) or test(sibling_ports_share_one_body))'
just unit-package pse-relations 'package(pse-relations) and (test(columnar::integrated_performance_unit::) or test(validate::prepared::consolidation_unit::) or test(validate::obligations::consolidation_unit::))'
just unit-package pse-compiler 'package(pse-compiler) and test(integrated_performance_unit::)'
just unit-package pse-rules 'package(pse-rules) and (test(strata::relational::tests::) or test(strata::rounds::improvement_unit::))'
just unit-package pse-numerics 'package(pse-numerics) and (test(pivot_unit::) or test(bindings::tests::) or test(graph_contract_unit::))'
just unit-package pse-catalog 'package(pse-catalog) and (test(improvement_unit::) or test(completion_unit::) or test(delta_boundary_unit::))'
just unit-package pse-runtime 'package(pse-runtime) and test(settings::delta_boundary_unit::)'
just py-unit python/pse/tests/test_transfer_contracts.py
```

The engine terminal and group commands are selective reruns plus one new ownership
unit, not additional distinct units beyond the count above. Catalog units use fake
providers, native streams and isolated boundary fixtures; no Delta commit/replay
journeys ran. Numerical units use native scalar/batch fixtures, without a solver.

Interface-checked: `just check` (workspace/all targets); `just clippy` (default and
no-default features, warnings denied); `just quality`; `just family-check`;
`just codegen-contracts-check`; `just fmt-check`; `just docs`; and `just py-sync`
(editable native development build and actual API stub). Final receipts are adjacent
to the unit logs. Cargo's existing upstream `proc-macro-error2` future-compatibility
notice and mdbook's large-search-index notice are separate from project lint failures.
No ADR, dependency pin or enforcement policy changed in this pass.

The I10 inference-cache capacity reset and hit-key allocations remain explicit I19
characterization cases. Canonical framing, hashes and trusted reopening contracts
remain unchanged. Preserve dynamic-filter restrictions, the leaf pushdown
mitigation, default-disabled predicate caching and durable settlement behavior.

## Outcome

Implemented the twelve bounded improvements within the existing Plan 11 contracts.
The complete I17 deletion barrier and I18/I19 qualification remain open; no measured
speedup or whole-workflow acceptance is claimed here.

**Mistakes made and corrected:** the initial native-equality shortcut hid a changed
dictionary child metadata field. The existing transfer test exposed this; dictionary
and container-extension schemas now retain full comparison. Initial forwarding also
treated capture as a transparent wrapper; its source constraints and statistics are
now deliberately withheld until row admission. The grouped-head unit also exposed
root relation metadata lost during shared field derivation; grouped preparation now
retains and rechecks those declarations. Focused compilation/lint failures were
corrected before final receipts.

**Deliberate boundaries:** actual admission and publication evidence remain separate
from cache interpretation. Compaction stays explicit. Provider capability forwarding
respects the capture boundary. Only complete unrestricted reads fill the resident
cache. I10 cache-capacity reset and hit-key allocation remain I19 characterization,
without a canonical framing/hash change. Integration is deferred under the user's
unit-first implementation policy, not represented as passed.
