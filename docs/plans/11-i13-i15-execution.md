---
title: Integrated native performance I13-I15 execution checkpoint
status: complete
date: 2026-09-20
adrs: [ADR-0074]
phase: 1
evidence: Implemented I13-I15; Tested 57 Rust and 65 Python isolated units; final integration open
---

# I13-I15 execution checkpoint

[Plan 11](11-integrated-native-performance.md) owns scope. This checkpoint records
I13-I15 implementation and its development evidence. ADR-0074 remains proposed.
Starting source capture:
`build/plan11/i13-i15-start-20260920T061255Z/source.tar.gz`.
Other shared-tree changes predate this work and are preserved.

Targeted isolated units suffice for every implementation and deletion. Normal
workspace compilation, Clippy, Python quality, formatting and pure generation remain
available. Integration, component, storage, solver, parity and performance campaigns
remain **not run** until all I00-I17 implementation/deletion scope is complete;
I18/I19 own final qualification. No additional campaign controls are introduced.

## Implemented boundary

### I13 — Write evidence and settlement

`delta/write_evidence.rs` retains private completion evidence in the actual native
invocation. It binds the full member attempt, selected inputs, actual registry and
compiled contract owner, complete-content coverage and returned committed version.
Reservations precede retention. Budget refusal declines this optimization. Only an
unequivocally successful native write can finish a pending witness. Full replacement
or newly provisioned content can establish local values; an append cannot certify
untouched rows. Recovered receipts, persisted properties and caller fingerprints
cannot manufacture a local witness.

`ObligationTemplates::bind_classified` preserves relation and obligation category.
Catalog admission omits only local-value queries backed by the exact completion.
Schema, completeness profile, primary-key and token consistency, references, source
spans, quantities, ordinal and numerical obligations remain. External/reopened data
has no volatile write proof and receives all unresolved admission.

`delta/write.rs` uses the native returned table and exact expected transition for
known success, then admits that state to the existing snapshot cache. Provisioning
success no longer searches history for its own receipt. Uncertain creation/write
outcomes and retries still inspect actual committed state before executing input
again. Primary native failures survive secondary observation failures. Existing
OCC read dependencies, exact operation receipts and zero automatic commit retries
remain authoritative; application transaction markers alone never justify replay.

`delta/publish.rs` retains the native update row count. Zero matching rows is a
parent conflict. Exactly one row plus the expected version establishes successful
control publication directly; unexpected count/version is a typed failure carrying
known commit information where available. Successful control creation validates its
initial version and parent without a redundant control read. Ambiguous outcomes
retain reconciliation. The control relation remains the coherent multi-member
visibility boundary.

SQL insertion uses native `Precision::Exact` input cardinality only after a successful
commit. Otherwise it reads the exact version's native `WriteMetrics`, with native
before/after aggregate fallback for missing or unavailable metrics. Deletion retains
native counts or exact snapshot-difference fallback, without reevaluating predicates.
A counting failure after a data commit preserves that committed version.

`DurableLayout` classifies each storage-to-execution mapping once. Identity,
string/binary widening/view mappings and the declared Int64 timestamp storage pair
avoid an unnecessary inverse cast. Unsigned, narrowing, fixed-size and nested
mappings retain the loss check. Existing Arrow casts, parent-null masks, schema and
metadata checks remain in force.

### I14 — Completed retention and bounded scheduling

The engine's model-result family uses DataFusion `DefaultCache` with a finite byte
limit. The actual producer token and dependency-scoped admission witness establish
reuse; hashes locate candidates only. The existing witness includes relevant source,
function, policy, planning and registry owners, with complete selection for opaque
resolution. Unrelated binding edits do not discard a transparent producer's result.
Mutable, observed, unqualified and pending-requirement selections are ineligible.

Only successful exhausted resident completions enter model retention. Spilled
results stay invocation-local. Idle keys hold a weak selection and no producing
plan or session. Native allocation owners continue charging retained/exported data
after eviction; invalidation prevents a late fill from repopulating the old epoch.
Completed leaf readers expose exact observed row cardinality through DataFusion
`StatisticsContext`/`StatisticsArgs`, without inventing ordering or partition facts.

An owned `AttemptScope` qualifies snapshot/resident in-flight keys. Completed values
remain shareable across attempts, while cancellation and partial fills stay local to
their invocation. No cross-attempt in-flight sharing is enabled. Related output ports
continue using the existing typed completion cells and round epochs.

Query and output admission use shared cancellation-aware Tokio semaphores. Each
active execution retains its permit through stream exhaustion/error/drop. Nested
work inherits that specific parent's live permit, independently of sibling cache
sharing; an escaped child with an expired parent permit must acquire admission.
Member writes fully settle effectful children before taking their output slot, so
nested writers cannot hold a slot while waiting for the same capacity. Pure input
can stream directly. Snapshot replay holds the existing bounded load/staging gate;
decoded fills run under outer query admission and the shared reserving pool, without
holding a replay permit across child execution. Decode reports distinguish active
fills from replay staging bytes.

Independent member opens and prepared output groups use bounded native futures and
restore declared order. Reusable rule roots also execute under a bounded ordered
buffer, advancing the shared epoch once and settling started streams before returning.
Failure stops newly admitted work and retains all observed causes; failed epochs
cannot resume. Expensive shared producers retain one completion, ordinary pure
single consumers stream, and no unbounded fan-out queue is introduced.

`ResourceBudget` and Python `EngineSettings` expose query/output concurrency and
model-result capacity. Solver concurrency and per-solve foreign-memory allowance
must be configured together and fit aggregate working capacity. `SharedRuntime`
retains one solver resource owner when that build capability is enabled; defaults
do not silently enable solver admission. Existing shared memory, spill, reader and
prefetch budgets remain in force.

Workflow fixtures use the canonical registry and two workers/partitions by default,
with an explicit configurable tiny route. Nextest coordinates ordinary engineering
workflows at two scheduler slots each, at most two simultaneous processes with
32 GiB declared ceilings each. Blanket catalog/engine serialization is removed;
existing resource-heavy and solver cases retain their justified groups.

### I15 — Python startup and Arrow ownership

Inspection uses `pse_schema::shared_registry()` and the existing compatible process
runtime. Normal import registers extension types and checks package/native version
plus the compiled registry fingerprint. It does not walk generated contract modules.
The fingerprint uses the same prefixed content-hash representation as generated
Python contracts.

`python-contracts-check` is an ordinary quality step. Pure Python code generation
lints its actual temporary candidate before comparison or publication, without
executing normal package startup or silently importing installed contract classes.
Recursive annotation checks reject `Any`, unparameterized containers and missing
types through nested attrs classes and composite annotations. Cycles terminate.
Dynamic converter hook generation performs this admission before accepting an
untrusted class. The ban is preserved at generation, quality and dynamic boundaries.

Arrow C streams remain the bulk route. Known Python-attached Rust-only reader
acquisition, close, cancellation and failure-state waits use `Python::detach`;
Python diagnostics are built after reattachment. The callback remains the native
Arrow/pyo3-arrow reader, with no speculative Python attachment cycle or prefetch
thread. One-use export, terminal errors, cancellation and last-array ownership use
the existing `BatchStream` state machine.

Schema metadata comparison now uses one native empty-table zero-column projection.
It preserves native duplicate metadata keys without reconstructing a Python dict.
The actual two-PyArrow-reader plus independent Python worker case is authored in
`test_publication_streams.py` as integration coverage for I18. Its runtime attachment
behavior is deliberately not claimed by the isolated FFI or metadata units.

## Library contracts

Interface-checked: DataFusion 55.1.0, Arrow/Parquet 59.3.0 and object_store 0.13.2
from the DataFusion skill and locked source; Delta capture
`58f07cd62bfbce3649a7e1c87c696288068ae184` and kernel
`8ba063f8f84fec222000f66d40d70911d7c79675` from the Delta skill and locked source.
No Context7 claims were used for those libraries.

Native `WriteBuilder` returns a table rather than a metrics tuple; update/delete
supply their native metrics, and exact commit actions carry write metrics. Native
checks and physical-input bridges preserve the selected session. DataFusion's
byte-LRU, completion streams, reserving memory pool and statistics APIs supply the
underlying mechanisms. Conservative ownership/eligibility proof is platform logic.
Pinned PyO3 0.29.2/pyo3-arrow 0.19.0 and PyArrow 25.0.1 source informed attachment and
projection choices; actual threaded consumer qualification remains I18 work.

## Deletion closure

| Row | Replaced path | Necessary remaining behavior |
|---|---|---|
| L03 | Repeated local-value query after exact certified handoff | New/untrusted values and unresolved relational obligations |
| L15 | Unconditional successful member/control reconciliation, redundant inverse casts/counts | Ambiguous recovery, exact transition/conflict checks and loss-aware fallback |
| L16 | Serial independent opens/outputs/rule roots, blanket catalog/engine test groups, one-worker workflow default | Bounded admission, deterministic assembly, dependency and epoch barriers |
| L17 | Rebuilt inspection registry, exhaustive import scan, per-field schema removal | Compatibility, recursive dynamic admission, native metadata and C-stream lifecycle |

## Verification

Tested: **57 Rust units** on the default nextest profile, with explicit
`pse-relations/force-validate`, and **65 Python units** on CPython 3.14.7 / PyArrow
25.0.1. Baseline: **zero failures**. Rust counts are 25 engine, 24 catalog and 8
runtime; Python deselects 10 component/integration cases. These are isolated
in-memory/native-boundary tests, not publication or solver journeys.

```sh
just unit-package pse-engine 'package(pse-engine) and (test(session::cache::tests::) or test(session::cache::admission::tests::) or test(session::execution::tests::) or test(cache_service::flight::) or test(cache_service::policy::tests::) or test(cache_service::load::integrated_performance_unit::) or test(session::reuse::scoped_reuse_unit::))'
just unit-package pse-catalog 'package(pse-catalog) and (test(delta::layout::tests::) or test(delta::contract::tests::) or test(delta::attempt::tests::) or test(delta::publish::completion_unit::) or test(delta::settlement::delta_boundary_unit::) or test(delta::admission::tests::) or test(inspection::stream::delta_boundary_unit::) or test(delta::write_evidence::completion_unit::) or test(delta::dml::execution::count_unit::))'
just unit-package pse-runtime 'package(pse-runtime) and (test(budget::tests::) or test(budget::concurrency_unit::) or test(settings::delta_boundary_unit::))'
just py-unit python/pse/tests/test_any_lint.py python/pse/tests/test_transfer_contracts.py python/pse/tests/test_publication_streams.py python/pse/tests/test_native_boundary_contracts.py
```

Logs: `engine-units.log`, `catalog-units.log`, `runtime-units.log` and
`python-units.log` in `build/plan11/i13-i15-final/`. They cover exact write identity,
append refusal, returned transition/no-op conflicts, loss-aware mappings, publication
obligations, nested/sibling permit ownership, unrelated-source reuse, grouped epochs,
late errors, escaped array ownership, attempt-local cancellation, exact cardinality,
resource limits, dynamic/candidate annotation rejection and duplicate metadata.

Interface-checked: `just check` (workspace, all targets), `just py-sync` (editable
dev extension and stubs from the compiled API), and `just quality` (15/15 steps,
including candidate-contract lint, Python types/imports and repository configuration)
pass at baseline zero. Static closure also passes:

| Command | Mode and outcome | Receipt under `build/plan11/i13-i15-final/` |
|---|---|---|
| `just clippy` | Workspace/all targets, default and no-default features, `-D warnings`, both passed | `clippy.log`; assessment `20260920T071158.116668Z` |
| `just family-check` | Single resolved pinned family and evidence-lock agreement | `family-check.log` |
| `just codegen-contracts-check` | Pure Rust/Python/docs comparison, 3/3 passed; Python lints actual candidate | `codegen-contracts-check.log`; assessment `20260920T071007.193555Z` |
| `just fmt-check` | Rust and TOML formatting | `fmt-check.log` |
| `just adr-lint` | Front matter, index and register | `adr-lint.log` |
| `just docs` | Book build | `docs.log` |

Python quality's full 15-step receipt is assessment `20260920T070747.015951Z`.
All command exits are zero; no performance speedup or durable integration result is
claimed. Cargo's existing upstream future-incompatibility notice and mdBook's large
search-index notice are visible in their logs.


## Outcome

Implemented: exact write completion and known-success settlement, bounded completed
model reuse and scheduling, canonical Python owners, explicit contract lint and native
metadata projection. No dependency pin or canonical identity format changed.

A mistake made and corrected: the first Python fingerprint export used unprefixed
hex, while the generated constant uses `blake3:`; actual import rejected the mismatch.
The first zero-column projection used `Schema.empty_table()`, which tried constructing
unsupported nested extension/union/run-end arrays. The existing transfer units caught
it; `Table.from_batches([], schema=...).select([])` preserves metadata without creating
arrays. The runtime annotation unit now checks the native report's generated property
stub rather than requiring the removed duplicate Python report declaration.

Deliberate boundaries: cross-attempt in-flight sharing is disabled; spilled results
remain invocation-local; append evidence cannot certify old content; unqualified
sources and pending requirements decline model retention. C-stream attachment under
actual threaded consumers, durable fault recovery, integrated workflows and timing
remain I18/I19 evidence. These are qualification boundaries, not postponed I13-I15
implementation or deletions.

## Resume

I13-I15 and L03/L15-L17 are complete for implementation/deletion. **Continue with
I16**, which owns typed diagnostics,
fixture/observation and assessment tooling; I17 owns the complete deletion and source
barrier. Existing actual-boundary cases in `unified_delta_contracts`,
`unified_delta_dml`, `publication_each_object`, `native_artifact_lifecycle` and Python
publication streams remain authored for I18 execution. I19 owns measurements.
