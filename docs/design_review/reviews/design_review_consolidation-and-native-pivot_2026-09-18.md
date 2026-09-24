---
title: Consolidation and native-pivot design review
date: 2026-09-18
status: done
evidence: Interface-checked
---

# Consolidation and native-pivot design review

## 1. Decision and scope

**Decision: Revise.** The codebase has completed its pivot onto DataFusion and delta-rs at
the *execution* layer, and that part holds up: every Delta write, merge, optimize, vacuum,
checkpoint and CDF read goes through the delta-rs builder, and every rule query, dependency
inventory and admission check is a native plan. The remaining bespoke mass sits one layer
below and one layer above that: a second value model (`Cell`) with three implementations
of one literal grammar, a per-relation generated codec that is 40 % of a 29 MB tree, a
parallel memory-accounting universe over DataFusion's pool, nine hand-written extension
node triples with identical scaffolding, and five compiler passes that decode relations
into `Vec`s and re-join them by linear scan. Those are the findings below. Each is stated
with the library surface that replaces it at the pinned versions, and with the blocker
found in code where one exists.

**Proposal.** Frame the best-in-class design along two axes the maintainer asked for:
(1) maximal deduplication and consolidation; (2) maximal use of Arrow 59.3, DataFusion 55.1
and delta-rs built-ins over bespoke code. This review deliberately does **not** judge
alignment with repository rules, ADRs or the blueprint. Where the target design would
require a rule, ADR or blueprint change, §10 lists it so the policy decision can be taken
afterwards on its own merits.

**Status of the target.** *Proposed.* Every replacement API named below is
*Interface-checked* against the pinned local API index
(`.codex/skills/datafusion/content`, `.codex/skills/deltalake/content`), the pinned crate
sources in the cargo registry, or the vendored delta-rs tree. Every code fact is
*Implemented* (read at the cited line). Line and byte counts are *Measured* with the
commands in the [evidence manifest](../evidence/consolidation-native-pivot-2026-09-18.json).
Nothing here is *Tested*: no library replacement was built, and no test was run.

**Reviewer.** Claude Code (Fable 5.1), at the maintainer's request, on the `main`
working copy at commit `a46f358` (Plan 09 complete).

**Affected revisions.** Hand-written Rust in all 23 `pse-*` crates, `xtask`, `tests/`,
`python/pse`; the generated trees are characterized, not audited.

**Observable outcome of the target design.** One row model instead of two; one literal
grammar instead of three; one memory-accounting mechanism instead of two; one extension
node scaffold instead of nine; one relational plan vocabulary instead of five; one
diagnostic-code vocabulary instead of two; a generated tree an order of magnitude smaller;
compiler passes whose joins are plans the optimizer can see. Two new crates carry the
shared substrate so that six crates and `xtask` stop depending on the 34 k-line catalog
crate for things that are not catalog.

**Baseline.** Plan 09 (`docs/plans/09-native-caching-and-pivot-completion.md`) is done and
its [acceptance review](design_review_native-cache-pivot-acceptance_2026-09-17.md) accepts
the local scope. The 2026-09-14 [capability review](design_review_full-arrow-datafusion-capabilities_2026-09-14.md)
raised F08 (repeated row conversion) and F12 (relational correspondence as repeated Rust
scans); F08 is materially improved by the generated borrowed views, F12 is still open in
P7/P8/P9 and is finding F9 here.

**Supported scope and non-goals.** In scope: duplication of meaning and bespoke
re-implementation of library capability. Out of scope: numerical correctness, IDAES
parity, performance claims (none are made), remote deployment, and whether any rule that
the target design touches *should* change.

**Method and coverage.** Seven parallel read-only surveys, one per subsystem, each
returning `path:line` evidence; I then re-read every line cited in §7 myself and
re-measured every count I report. What was inspected: all hand-written source in
`pse-ids`, `pse-schema`, `pse-relations`, `pse-catalog`, `pse-rules`, `pse-compiler`,
`pse-authoring`, `pse-templates`, `pse-mathir`, `pse-quantity`, `pse-material`,
`pse-numerics`, `pse-backend-native`, `pse-runtime`, `pse-py`; `xtask`, `tests/`,
`python/pse`; three sampled generated relation modules. What was **not** inspected at the
grain cited: `pse-schema/src/catalog/s6_*.rs` declaration bodies beyond `s6_7`, `p6/**`,
`p3/config/*` bodies, `pse-authoring/src/dsl/{lexer,parser}.rs`, `pse-mathir/src/payload.rs`,
`python/pse/parity/**`, `benches/`, `tests/structural/`. Guarantees **not attacked**: the
Delta attempt/publication recovery protocol (read, not adversarially exercised), canonical
hash stability across the proposed sort change (named as the blocker of F4), and the
serde_arrow nested-metadata round trip (named as a spike in F1). Silence on any of those
is "not attacked", not "clean".

**Constraints and uncertainty.** The three surveys of `pse-catalog` and `pse-compiler`
covered 68 k of the 125 k hand-written lines; findings there are complete for the modules
named and not for the crates as a whole. The two disagreements between survey counts and
my re-measurement are recorded in the manifest and the smaller number is used.

## 2. Authority and lifecycle map

The map is organized by *meaning*, because the review's first axis is meaning re-expressed
in several places. The "authority today" column is what the code treats as authoritative;
the "parallel representations" column is what the target design removes or derives.

| Meaning | Authority today | Parallel representations found | Target authority |
|---|---|---|---|
| A typed value's Arrow representation | The registry `Field` (`pse-schema/src/arrow.rs`) | `Cell` (`pse-schema/src/model/cell.rs:16`), `CellCodec` and `ArrowValue` per generated row, `cells.rs` builder walk, `encode.rs` UDF walk, `owned.rs` byte walk, three codegen type tables (F1) | The registry `Field` plus `datafusion::common::ScalarValue` for dynamic single values; one generic row codec |
| The lossless literal of a value | `Cell::literal_spec` (`cell.rs:104`) | `cell_codec::decode` (`cell_codec.rs:19`), `scalar/encode.rs:77`, `strata/relational.rs:70` (F1) | One writer over `ScalarValue`; the UDF calls it |
| A relation's declared contract | `RelationSpec` in the registry | `COMPILED_DECLARATION` 10.6 MB of string literals, 14 positional `Vec<Cell>` projections in `builder.rs:449-738`, Python attrs classes, Markdown, JSON Schema (F2, F14) | `Arc<RelationSpec>` identity + `ContentHash` fingerprint; one `FieldVisitor` for the three renderers |
| Memory accounting | DataFusion `MemoryPool` (`pse-runtime/src/env.rs:36-56`) | `pse_ids::resource::{MemoryReserver, Reservation, ReservationLease, FixedBudget}`, `PoolReserver`, `OwnedRecordBatch`, `attach_reservation`, five `ArrayData` byte walks (F3) | DataFusion pool + Arrow `pool` feature (`RecordBatch::claim`) + `ArrowMemoryPool` |
| Canonical framing bytes | `pse_ids::FrameSink` (`frame.rs:22`), dead | `canon/stages.rs:204` `part`, `mathir/graph.rs:585-616` `put_*`, `mathir/hash.rs:87-98` `frame` (F4) | `FrameSink`, the only writer |
| Identity formulas | `pse_ids::derive` | Namespace-string conventions in seven crates; `templates/identity.rs:10` and `p3/products/expansion.rs:142-147` compute one formula in Rust and as a DataFusion expression (F4) | One `ScalarUDF` per formula, invoked from Rust on scalars |
| A domain algorithm as a plan node | none shared | 9 `UserDefinedLogicalNodeCore` + 9 `ExtensionPlanner` + 16 `ExecutionPlan` impls with identical scaffolding (F5) | `Algorithm<A: DomainAlgorithm>` in `pse-engine` |
| A built-in whose output field drops metadata | `field_transfer.rs:55-67` dispatcher | `element.rs`, `structure.rs`, `list_concat.rs`, `aggregate/selection.rs`, `aggregate.rs` (F6) | One `FieldTransfer` scalar adapter and one aggregate adapter |
| An owned set of batches as a table | `MemorySourceConfig::try_new_exec` | `MaterializedTable`, `CandidateTable`, `RoundTable`+`RoundExec`, `Statistics`+`StatisticsExec`, `ResolutionProvider`+`ResolutionExec` (F7) | `MemTable` newtype and `StreamingTable`+`PartitionStream` |
| Column-level consumption of a plan | `artifact/consumption.rs:20` | Relation-level walk in `session/input.rs:254`; six memoized DAG walks with the same shape (F8) | One `PlanWalk` helper; `necessary_children_exprs` on pass-through nodes |
| Relational correspondence in a pass | DataFusion plans (P3, P5, P6, P8-contexts, P9-selections) | 127 nested-loop equality filters in P7, P8-expansion, P9, `pse-templates/paths.rs` (F9) | Plans; keys retained as `(RelationKey, ContentHash)` |
| Expression-graph traversal | `pse_mathir::walk::walk` (`walk.rs:23`), zero callers | 16 hand-rolled worklists in 14 files, two of which disagree on the successor set (F10) | `walk`, backed by petgraph `DfsPostOrder` |
| Per-opcode semantics | `OPERATOR_TABLE` (`math/operators.rs:217`) | `fold.rs:362-388`, `p4/predicates/scalar.rs:340-386`, `scalar_math.rs:181-285`, `differentiate.rs:95-175`, `xtask/.../evaluate.rs:34-44` (F11) | The table drives evaluation and lowering; one evaluator |
| Unit conversion | `pse_quantity::unit::convert_value` (`unit.rs:179`) | `scalar_math.rs:218-220` re-derives `a*b+c` (F11) | A `ScalarUDF` wrapping `convert_value` |
| Delta storage layout | `pse-schema/src/delta.rs:142-248` type table | `layout.rs` direction as a name-string switch at four sites; three admission entry points (F12) | One declarative field-storage contract with a direction enum |
| Delta log retention | `delta.enableExpiredLogCleanup` in the registry (`relation.rs:222`) | `.with_cleanup_expired_logs(Some(false))` at five commit sites; `log_cutoff_ms` as a caller argument (F12) | The registry property, read by the commit path |
| Failure taxonomy | `FailureClass` enum, 24 members (`enums_platform.rs:378`) | 130 inline `#[diagnostic(code(...))]` sites, 58 spellings, 12 of which are members (F13) | One generated code type; codes minted, not typed |
| Test fixture "registry + session + store + root" | none | 14 helpers, one in production (`pse-py/src/inspection/runtime.rs:34`), one in `xtask` reaching into `tests/support` by `#[path]` (F15) | `pse-testkit` dev crate |
| Python settings shape | Rust core structs | 26–39 hand restatements per shape in `pse-py`; `ResourceUsage` as a positional tuple re-named in Python (F16) | `#[pyclass(get_all)]` on flattened structs; a named struct |

**Deliberately opaque behavior that should stay bespoke.** The fixed-point engine's
round driver (DataFusion's `RecursiveQuery` cannot host it, §8); the Delta member-attempt
receipt protocol; the `pse.canon.v2` preimage layout; float canonicalization
(`float.rs:40-62`); the canonical metadata relation (`stages.rs:84-133`); the strict-IPC
aligned copy (`owned_buffer.rs:280-297`); the `Finite` and `Vector` UDFs; the `pse.*`
extension formatter (correctly built on `ArrayFormatterFactory`); the `Flights` single-flight
map (`cache_service/flight.rs`, no DataFusion equivalent at 55.1).

## 3. Semantic contracts and invariants

Only the contracts the findings put at risk are listed.

| Contract or invariant | Representation | Enforcement boundary | Failure behavior today | Verification evidence |
|---|---|---|---|---|
| Two equal keys render to equal literals | `Cell::literal_spec` and `scalar/encode.rs` must agree byte for byte | `validate/bundle.rs:80` (string set) vs `scalar::literal` in rules framing | No differential test exists; `encode.rs` handles 18 `DataType` arms to `literal_spec`'s 11 kinds | Gap (F1) |
| Float identity policy | `cell.rs:110` writes raw bits; `cell.rs:167` and `normalize.rs:131-133` write canonical NaN bits | Hashing vs literal rendering | A NaN-payload-differing pair hashes equal and renders unequal | Implemented divergence (F1) |
| Reserve before allocate | `Reservation::try_grow` before the allocation | `PoolReservation::try_grow` (`reserve.rs:81-92`) | The local pre-check compares one reservation against the whole pool and never fires under contention; DataFusion's pool is the real refusal | Implemented (F3) |
| A sliced batch reserves its visible bytes | `validation_extent::visible_buffers` | Twelve call sites across authoring and compiler | `buffer.len()` reports the parent buffer for an `ArrayData` slice, so a small slice of a large batch can be refused | Implemented over-estimate (F3) |
| Nested metadata enters the logical hash | `stages.rs:120-132` | `pse.canon.v2` frame | `Map`, `Union`, `Dictionary` and `RunEndEncoded` children are skipped, while `field_contract.rs` compares them | Implemented gap (F4) |
| Row order is identity | `admit_and_order` sorts by `RowConverter` bytes | Logical hash | Switching to `lexsort_to_indices` needs an equivalence proof over the layout matrix before either implementation is deleted | Named blocker (F4) |
| Column-level dependencies for cached plans | `necessary_children_exprs` | `consumption.rs:86-99` | Any plan containing any extension node records whole-relation evidence; no extension node overrides the method | Implemented (F8) |
| Null-equality in joins | Explicit `NullEquality` argument | `native_rows::join` uses `NullEqualsNothing`; `relational::keyed_join` uses `NullEqualsNull` | Two helpers with the same name and different defaults | Implemented (F9) |
| One successor function per graph | `children ∪ payload.referenced_nodes()` | Traversal loops | `p8/expansion/axes.rs:86` omits `referenced_nodes`; `p10/contracts.rs:43` includes it | Implemented divergence (F10) |
| Integer folding is exact | `OPERATOR_TABLE.foldable` | Two evaluators | `fold.rs:383` and `p4/predicates/scalar.rs:349-352` disagree on `Pow` | Implemented divergence (F11) |
| `(v*scale)+offset` in that rounding order, never an FMA | `unit.rs:6-14` | `convert_value` | `scalar_math.rs:218-220` rebuilds it as a DataFusion expression tree, which makes no such promise | Implemented (F11) |
| A diagnostic code is a `FailureClass` member | Registry enum | `builder.rs:1685-1697` for algorithm diagnostics only | 46 of 58 inline spellings are not members; nothing checks them | Measured (F13) |

**Absence and uncertainty.** `Cell::Null` versus a nullable `Field` versus `ScalarValue::Null`
are three ways to say "absent"; the target keeps two (the `Field` nullability contract and
the array validity bitmap) and drops the third. Unknown configuration keys remain semantic
inputs (`config.rs:111`), which is right; the consequence that a DataFusion patch adding a
`datafusion.runtime.*` key invalidates every settings hash is stated in F8.

**Equivalence requirements.** Byte equality for the canonical preimage (must survive F4);
structural equality for `RelationSpec` (replaces string equality of the compiled
declaration in F2); semantic equality for the three literal encoders (F1).

## 4. Derivation and execution design

This section is the target design. It is stated as crate boundaries and the single owner
of each meaning, because that is what the findings change.

### 4.1 Crate topology

```text
pse-ids          identity, framing (FrameSink only), float policy, pse.canon.v2
pse-quantity     unchanged surface; registry lookups keyed, not scanned
pse-material     unchanged
pse-schema       registry, declarations, codegen; RelationSpec is the row authority;
                 Cell deleted, ScalarValue for dynamic values
pse-relations    generated: constants + Row struct + spec(); generic TypedBatch<R>
                 and RowSink<R>; columnar validators; no per-relation codec
pse-engine  NEW  session assembly and profile; settings classification; Algorithm<A>
                 node/planner/exec; OneShotCommand<R>; FieldTransfer adapters;
                 PhysicalInput; StreamingTable-based sources; PlanWalk; FieldRule
                 walker; relational plan vocabulary; memory claims; NativeExecutionContext;
                 function inventory; failure classification
pse-catalog      providers, bindings, snapshot sessions, artifacts, cache service
pse-delta   NEW  write/publish/maintenance/attempt/layout/admission/changes/lease
pse-mathir       one traversal (walk) on petgraph DfsPostOrder; row types are the
                 generated relation rows
pse-rules        FixedPoint implements DomainAlgorithm; round table is WorkTable-shaped
pse-compiler     passes are plans; DomainAlgorithm impls; no inventory Vecs
pse-templates    identity formulas as ScalarUDFs; path resolution as plans or indices
pse-numerics     Finite/Vector UDFs; ScalarMath driven by OPERATOR_TABLE; unit conversion UDF
pse-backend-native  Solve implements DomainAlgorithm (once it has a caller)
pse-testkit NEW  dev-only: Fixture, FaultStore, id(), ThreadBudget::single()
pse-codes   NEW  leaf: FailureClass type generated from the registry; code-minting derive
```

**Why exactly these four new crates and not more.** `pse-engine` exists because six
crates (`pse-compiler`, `pse-rules`, `pse-authoring`, `pse-backend-native`, `pse-py`,
`pse-runtime`) and `xtask` need the extension substrate and the session assembly but not
Delta durability, and today they get both by depending on the 34 k-line `pse-catalog`
(`cargo metadata`, §1). `pse-numerics` builds its UDFs on `datafusion` directly and would
use only the `FieldTransfer` adapter. `pse-delta` exists so that `pse-catalog` stops owning a 12 k-line
durability layer that has its own protocol. `pse-testkit` exists because the same fixture
is assembled 14 ways and the three `tests/*/src/lib.rs` targets export nothing (F15).
`pse-codes` exists because the taxonomy must be citable from crates *below* `pse-schema`
(F13). No other consolidation in this review needs a crate; each has an existing owner
named in §7.

### 4.2 The value and row layer

| Stage | Today | Target | Owner |
|---|---|---|---|
| Registry self-description rows | 14 positional `Vec<Cell>` projections (`builder.rs:449-738`) | `RelationSpec` and siblings derive a row projection by field name; rows built through `ScalarValue::iter_to_array` per column | `pse-schema` |
| Typed row ↔ batch | Generated `CellCodec` + `ArrowValue` + `Builder` + `View` per relation (≈ 40 % of 446 k lines) | Generated `Row` struct only; one generic `TypedBatch<R>` (borrowed columns downcast once via `AsArray`) and one `RowSink<R>` | `pse-relations` |
| Codec implementation for `RowSink<R>` | — | Either `serde_arrow::{to_record_batch, from_record_batch}` with `SchemaLike for Vec<FieldRef>` from the registry (Interface-checked, `serde_arrow-0.15.0/src/arrow_impl.rs:144,175,412`) and `RecordBatch::with_schema` to restore schema-level metadata; or a `#[derive(ArrowRow)]` proc-macro producing today's `ArrowValue` body at compile time. The serde_arrow route needs a spike (§9) | `pse-relations` |
| Dynamic single values | `Cell` (11 variants, tagged JSON literal) | `ScalarValue` (Interface-checked: Null/Boolean/Int64/UInt64/Float64/Utf8/FixedSizeBinary/List/Struct/Dictionary, `datafusion-common-55.1.0/src/scalar/mod.rs:360-473`). The `Text`-vs-`Enum` tag moves to where it already lives, the `Field`'s extension metadata. NaN canonicalization is applied at the hashing site, as `cell.rs:167` already does | `pse-schema` |
| Literal grammar | Three implementations | One writer over `ScalarValue`; the `pse_literal` UDF calls it through `ScalarValue::try_from_array` | `pse-schema` |
| Declaration identity at a boundary | `COMPILED_DECLARATION` string compare (`columnar.rs:340`) | `Arc::ptr_eq` when the registry is shared; fingerprint compare otherwise; full `RelationSpec` compare on fingerprint mismatch | `pse-relations` |
| Local value validation | `Cell`-per-row loops (`validate/values.rs`) | Columnar predicates as `capture/values.rs:19-29` already does (`logical_null_count`, `arrow_row`) | `pse-relations` |
| Key uniqueness / FK / ordinal range | SQL invariants (authority) + `validate_bundle` Rust twin (no production caller) + `RowConverter` in canon | SQL invariants only; `validate_bundle` deleted | `pse-schema` + `pse-rules` |

### 4.3 The engine substrate

| Stage | Today | Target |
|---|---|---|
| Domain algorithm node | 9 triples; identical `Debug`, `Eq`/`Hash`/`Ord` by `Arc` pointer, `prevent_predicate_push_down_columns`, planner prologue, `PlanProperties`, run-once guard, `execute` wrapper | `trait DomainAlgorithm { name, output_schema(inputs), effects, expressions (default empty), run(streams, ctx) -> stream }` plus `Algorithm<A>`, one `ExtensionPlanner`, one `AlgorithmExec` with `BaselineMetrics`. `Solve` keeps an expression-rewrite hook. `Cache` (`cache.rs:247-252`, empty `inputs()` as an optimizer leaf) stays out of the first cut |
| Effectful one-shot command | `DeltaWrite`, `DeltaPublish`, `Maintenance`, `Command` with the same nine constructs | `OneShotCommand<R>` |
| Built-in that drops field metadata | Four scalar wrappers, two aggregate wrappers | `FieldTransfer { native, derive }` overriding only `return_field_from_args`, with a pre-invoke hook for the two that must force the array path or cast; `AggregateFieldAdapter` likewise. `field_transfer.rs:55-67` is already the dispatcher |
| Owned batches as a table | Five provider+exec pairs | `MemTable` newtype whose DML methods return `Err` (`MemTable` implements `insert_into`/`delete_from`/`update`, Interface-checked); `StreamingTable`+`PartitionStream` for produce-once sources, as `delta/dependencies.rs:53-56` already does |
| Plan walks | Six memoized walks with the same five moving parts | `PlanWalk::read(plan, f)` and `PlanWalk::rewrite(plan, f)`; the `Subquery`/`RecursiveQuery` scope rule in one place |
| Field compatibility | Seven recursive walkers with different rule subsets | One recursive walker parameterized by a `FieldRule` |
| Settings classification | Hand list of 12 keys (`config.rs:113-130`); predicate cache written by three sites with three rules | `key.starts_with("datafusion.runtime.")` plus the four named `execution.*` keys; one writer for the predicate cache |
| Memory | `MemoryReserver` trait, `PoolReserver` adapter, `ReservationLease`, `attach_reservation`, `retained_allocations`, five byte walks | Enable the Arrow `pool` feature (`arrow-buffer-59.3.0/Cargo.toml:43`); `RecordBatch::claim(&ArrowMemoryPool)` for ownership; `MemoryReservation::try_grow` before allocation for refusal; `ArrayData::get_slice_memory_size` for measurement. `pse-ids` loses `resource.rs` and most of `owned_buffer.rs` |
| Cancellation | `pse_ids::CancellationToken` wrapping tokio-util, plus `pse_runtime::CancelSource` wrapping that | `tokio_util::sync::CancellationToken` re-exported, one `checkpoint()` extension trait |
| Failure classification | `classify` and `classify_borrowed` (same 13-arm match), 18 wrapper functions, a competing classifier in `pse-numerics` | `failure::classify(Cow<DataFusionError>, PlanOrigin)` once; `trait FromEngine` implemented per error enum |

### 4.4 The compiler and math layer

| Stage | Today | Target |
|---|---|---|
| Pass inputs | P4 (17), P7 (48), P8 (23), P9 (8+8) relations decoded to `Vec<Row>`, one plan-prepare-execute each, then 127 nested-loop equality filters | Inputs stay plans; joins, group-bys, set differences and closures are `Join`, `Aggregate`, `EXCEPT` and `to_recursive_query` (already proven at `p3/products/expansion.rs:294-296`); P7 keeps `(RelationKey, ContentHash)` keys instead of positions |
| Plan vocabulary | Five copies of `project/append/filter/join/union/distinct` with three incompatible column-name encodings | One module in `pse-engine`; `c(alias, field)` has one encoding; `NullEquality` is a required argument |
| Provenance | `rule_support_edges`, `constructed_supports` (five verbatim shared columns), in-row `algorithm_support` list decoded twice, `p7/evidence.rs` 30 hand-expanded semi-joins | One shared support column group; one `support_to_derivations` plan; the in-row list decoded once by UNNEST |
| Graph traversal | 16 worklists, two disagreeing | `walk` on petgraph `DfsPostOrder` with the cycle path preserved |
| Operator semantics | Table plus four dispatches it does not drive | `OPERATOR_TABLE` drives evaluation and lowering; one folder; unit conversion as a UDF over `convert_value` |
| Math IR relations | `VecSink` tuple rows (`NodeRow`) mirroring `math_expr_nodes`; hand-written five-relation sorted EXCEPT | Generated relation rows are the row type; the reconstruction check is an `EXCEPT` plan |
| Paths | Seven encodings of instance→member | `normalized.expression_paths` plus one segment lift and one suffix rule |

### 4.5 The Delta layer

Already on delta-rs builders end to end (§7 F12 lists the evidence). Target: `OneShotCommand`
for the three command triples; one bounded commit-log reader (`attempt::receipt_bytes`) for
the five readers; the storage contract as one declaration with a `Direction` enum instead
of `name == "pse_delta_decode"` at four sites; `delta.logRetentionDuration` declared in the
registry and derived exactly as `checkpoints.rs:61-62` derives it; the five
`with_cleanup_expired_logs(Some(false))` sites replaced by the declared property; one
obligation descriptor and executor for the four admission validators.

**Relationship structures.** Ownership (registry → generated → consumers), dependency
(plan walks), provenance (support relations) and scheduling (round epochs) are kept
distinct in the target; the finding is that each is currently *implemented* several times,
not that they are conflated.

**Provider selection and limitations.** DataFusion 55.1.0, Arrow 59.3.0, delta-rs
`58f07cd6` (vendored). Two library limits shape the target and are named as such:
`arrow_buffer::MemoryPool::reserve` is infallible (`pool.rs:73-77`), so refusal must stay
a separate `try_grow`; `ArrayConcat` and `ArrayElement` ship no `return_field_from_args`
(`api/datafusion_functions_nested.concat.md:128`, `extract.md:140`), so the metadata
adapter must stay.

**Boundary contracts.** Python receives the exact Arrow schema through `pyo3-arrow` today
(`stream.rs:46,79`); the generated attrs contracts are a strict subset of it (F14). The
target keeps the JSON boundary (cattrs targets) and derives everything else at runtime.

**Coherent publication.** Unchanged: the control-table commit is the atomicity point and
the member attempt protocol is bespoke by necessity (`kernel/transaction/mod.rs:479-481`
pushes `Txn` actions without deduplicating).

## 5. Representative journeys

### Ordinary extension: add one relation

| Step | Today | Target |
|---|---|---|
| Declare | One `relation(...)` call in `pse-schema/src/catalog/s6_*.rs` | Same |
| Generate | ≈ 1,000 lines of Rust (row, `CellCodec`, `ArrowValue`, view, builder, a 24 KB declaration literal), an attrs class, a Markdown row, a JSON Schema entry | ≈ 60 lines of Rust (constants, row struct, `spec()`); the same three renderers from one `FieldVisitor` |
| Consume | `View::from_checked` compares a 24 KB string per call | `TypedBatch::<Row>::from_checked` compares an `Arc` pointer or a 32-byte fingerprint |

Semantic decisions added: one. Files touched by hand: one. This journey is fine today and
is included to show what the generated volume is *for*: nothing the consumer needs that a
generic implementation cannot supply.

### Ordinary extension: add one domain algorithm node

Today: a `UserDefinedLogicalNodeCore` impl (≈ 80 lines of scaffolding), an
`ExtensionPlanner` (≈ 30), an `ExecutionPlan` (≈ 90), and a registration in
`UnifiedPlanner::new` or `pse_compiler::query_planner`. Target: `impl DomainAlgorithm for
MyAlgorithm` (five items) and nothing else. The fourth node added this way would be the
first to get `EXPLAIN ANALYZE` metrics for free, because none of the existing nine
implements `metrics()`.

### Meaningful change: rename a column

Today: the registry fingerprint changes (correct), `COMPILED_DECLARATION` changes in one
generated file (correct), and the same column is re-spelled in the Python contract, the
Markdown page and the JSON Schema by three renderers that must agree by construction.
`builder.rs:449`'s positional `Vec<Cell>` does not change and does not need to, but if the
rename were a *reorder* of two same-typed columns, every test passes and the registry's
self-description silently transposes (`registry_assembles.rs:404` checks width only).
Target: one renderer walk; row projection by field name.

### Boundary: a value crosses into a UDF and back

Today: the value is rendered by `scalar/encode.rs` (18 arms), framed by
`relational.rs:70`, and, if it is a primary key, compared against a `literal_spec`
string from `cell.rs:104` (11 kinds). The two encoders were written to avoid each other
(`scalar.rs:75`: "without `Cell` reconstruction"). Target: one writer.

### Interruption: a write attempt is interrupted after provisioning

Unchanged by this review and read rather than attacked: `write_attempt` re-opens the table
and asks the log (`write.rs:405-499`); a builder success with no matching transaction is
`unresolved` (`write.rs:530-532`). The consolidation into `OneShotCommand` must preserve
the `started.swap(true)` run-once guard verbatim.

## 6. Acceptance gates

Gates are the charter's, settled on their own evidence. They judge the *current* code; the
target design's claims are Proposed and cannot pass a gate.

| Gate | Verdict | Evidence or scope rationale | Required action |
|---|---|---|---|
| G1 — Authority | **Fail** | Three writers of one literal grammar (F1); `Layout` mirrors `DataType` for 18 of 19 variants and is checked against a rebuild of itself (`preflight.rs:217-229`, F4); the domain-product identity formula in Rust and as a plan expression, tied together only by a test (F4); the failure taxonomy declared twice with 46 non-member spellings (F13) | Single owners per §4 |
| G2 — Semantic fidelity | **Fail** | Two integer folders disagree on `Pow` (F11); two traversals disagree on the successor set (F10); the unit-conversion FMA guarantee is dropped by the plan lowering (F11); the canonical metadata walk skips four container kinds (F4) | One evaluator, one traversal, one conversion UDF, complete walk |
| G3 — Validity | **Unresolved** | `validation_extent` over-reserves for slices (F3); `contract_metadata` documents a filter it does not implement (`contract.rs:222-228`, F4); `validate_bundle` is documented as called by publication and is not (`validate/mod.rs:5`, F2) | Decide the intended contract in each case, then fix the code or the doc |
| G4 — Hidden behavior | **Fail** | Every plan containing an extension node silently records whole-relation dependencies (F8); the predicate cache size is written by three sites with three rules, last writer wins (F8); the per-commit `cleanup_expired_logs` override makes the registry property unobservable (F12); `Solve` and `ScalarMath` have no production callers and three declared dependencies have zero uses (F17) | Override `necessary_children_exprs`; one writer; declared property; wire or scope out |
| G5 — Consistency and recovery | **Pass (scope)** | The Delta attempt/publication protocol was read and found single-authority on disagreement (`write.rs:437-441`, `publish.rs:623-625`); not adversarially exercised here | None from this review |
| G6 — Transformation and reuse | **Fail** | P7 executes 48 separate plans per invocation then joins by scan (F9); `postorder_with_bindings` clones the whole graph four times per canonicalization (F10); the 130 SQL rule files are machine-generated with no generator checked in (F17) | Plans in passes; one traversal; recover or delete the rule generator |
| G7 — Truthful capability claims | **Fail** | `pse_ids::FrameSink` is documented as the canon framing and has zero callers (F4); `serde_arrow` is a declared dependency with zero uses; `num-dual` likewise in `pse-numerics`; `datafusion-proto`, `serde_json`, `pse-quantity` likewise in `pse-rules` (F17) | Delete or use |

## 7. Principle findings

**Applicability.** Groups 1–3 (authority, types, identity), 5 (derivation), 7
(dependencies and reuse), 8 (execution layout), 11 (generation and verification) and 12
(leverage) bear on this scope and carry the findings. Group 4 (declarative composition)
applies only through F9 and F11. Group 6 (effects and recovery) was read for G5 and
raised nothing. Group 9 (providers) bears on F6, F7 and F12. Group 10 (provenance) bears
on F8 and F9.

Findings are grouped by cause. Evidence labels: **Implemented** = read at the line;
**Measured** = counted with the command in the manifest; **Interface-checked** = the named
library surface was opened at the pin. Severity order follows the charter: authority and
correctness first, then duplication and extension locality, then measured cost.

| # | Finding | Principle IDs | Concrete evidence or gap | Consequence | Proposed correction | Verification |
|---|---|---|---|---|---|---|
| F1 | **One value grammar, three implementations, and a second row model** | DM-02, DM-15, DM-24, DM-56 | Implemented: `cell.rs:104` `literal_spec`, `cell_codec.rs:19` `decode`, `scalar/encode.rs:77` `value` with byte-identical float rendering (`cell.rs:110` and `encode.rs:111` both `"\"{:016x}\""`), `strata/relational.rs:70` `frame` rebuilding the `List`/`Struct` arm. `scalar.rs:75` names the split: "Encodes an actual typed value without `Cell` reconstruction". Key identity depends on both: `validate/bundle.rs:80` `values[*index].literal_spec()` and `scalar::literal` in rules framing. `encode.rs` covers 18 `DataType` arms, `literal_spec` 11 kinds. Measured: six hand-written value-shape walks plus three codegen tables (§2). `pse-compiler` has zero `Cell` references and 140 `FieldCheckedBatch` sites, so the columnar path already suffices for the largest consumer | Two keys that are equal under one encoder and unequal under the other are a silent identity fork; a new storage type must be added to three walks | Delete `Cell`/`CellCodec`; `ScalarValue` (Interface-checked, `datafusion-common-55.1.0/src/scalar/mod.rs:360-473`) for dynamic values; one literal writer; the `Text`/`Enum` distinction taken from the `Field` extension metadata where it already lives. `pse-schema` gains a `datafusion-common` dependency | A differential test asserting `encode::value(array, field, row) == literal_spec(cell_at(...))` over every declared relation must exist *before* deletion; then `just test`, `just codegen-check`, baseline zero |
| F2 | **A 10.6 MB restatement of the registry, compared by string** | DM-02, DM-52, DM-56 | Measured: 442 `COMPILED_DECLARATION` literals, 10,614,155 bytes of a 29,000,623-byte generated tree; mean 24 KB, one 577 KB line. Implemented: each column entry restates the unbound field, the bound field and the extension spec (`compiled_contract.rs:81-91`); compared as `self.declaration.as_ref() != declaration` at `columnar.rs:340` and stored per batch as `Arc<str>` (`columnar.rs:48`). `validate_bundle` re-implements PK/FK/ordinal in Rust with no production caller (`validate/mod.rs:5` says publication calls it; `rg` finds only `tests/admission.rs:353`). Generated codec is 130,469 lines (29.2 %) of `CellCodec`+`ArrowValue`+`into_cells`, ≈ 40 % with view/builder bodies | Every consumer pays a 24 KB compare per typed borrow; a doc string change regenerates 10 MB; the row model is maintained twice (F1) | `Arc<RelationSpec>` identity plus fingerprint; drop the inlined doc and the derived second spelling; generic `TypedBatch<R>`/`RowSink<R>`; delete `validate_bundle`. Codec via serde_arrow (`to_record_batch`, `from_record_batch`, `SchemaLike for Vec<FieldRef>`, Interface-checked) after the spike in §9, or a derive macro | `just codegen && just codegen-check`; the generated `equal_fingerprint_cannot_admit_a_changed_declaration` test must still reject a nullability-only change |
| F3 | **A parallel memory-accounting universe over DataFusion's pool** | DM-25, DM-37, DM-56, DM-58 | Implemented: `MemoryReserver`/`Reservation` (`resource.rs:31-65`) mirror `MemoryConsumer`/`MemoryReservation`; `PoolReserver` (`reserve.rs:49-109`) adapts one way; `ReservationLease` needs a `Mutex` only because the trait chose `&mut self`; `PoolReservation::try_grow` pre-checks one reservation against the whole pool limit (`reserve.rs:81-92`) so it never fires under contention; `export_query_batch` charges a byte a second time by documented design (`owned_buffer.rs:305-307`); five recursive `ArrayData` byte walks, two byte-identical (`canon/api.rs:169-177`, `validation_extent.rs:50-58`); `visible_buffers` uses `buffer.len()` which is the parent buffer for a slice. Interface-checked: `arrow_buffer::pool::{MemoryPool, MemoryReservation}` and `Bytes::claim` (`arrow-buffer-59.3.0/src/bytes.rs:110-114`, behind feature `pool`, off today per `cargo tree`), `RecordBatch::claim`, `datafusion_execution::memory_pool::arrow::ArrowMemoryPool` (`arrow.rs:41-67`), `ArrayData::get_slice_memory_size`. Blocker: `arrow_buffer::MemoryPool::reserve` is infallible (`pool.rs:73-77`) | Two accounting authorities that can disagree on the same allocation; a small sliced batch can be refused; ≈ 250 lines of lease plumbing exist to reconstruct what the shared buffer can carry | Enable the `pool` feature; `RecordBatch::claim` for ownership, `MemoryReservation::try_grow` before allocation for refusal, `get_slice_memory_size` for measurement; delete the trait, `FixedBudget` (use `GreedyMemoryPool`), `PoolReserver`, `ReservationLease`, `attach_reservation`, `retained_allocations`; one `CancellationToken` | `just test` under `tests/lifecycle/tests/query_memory_budget.rs`; peak-RSS measurement before and after (Measured, conditions recorded) |
| F4 | **Identity framing written four times; `Layout` mirrors `DataType`; one formula in two languages** | DM-02, DM-15, DM-56 | Implemented: `FrameSink` (`frame.rs:22-82`) has zero callers and `lib.rs:17` documents it as the canon framing; byte-identical private copies at `stages.rs:204-210`, `mathir/graph.rs:585-616`, `mathir/hash.rs:87-98`. `Layout` (`contract.rs:124-176`) maps one-to-one to `DataType` for 18 of 19 variants and `preflight.rs:217-229` rebuilds the contract to check the layouts against themselves; `contract_metadata` (`contract.rs:222-228`) documents a filter the body does not perform. `templates/identity.rs:10-16` and `p3/products/expansion.rs:142-147` compute `pse:domain-product:v1:` once in Rust and once as `concat(lit(...), ...)`, with `expansion.rs:114` admitting "Identity framing exactly matches". `stages.rs:120-132` skips `Map`/`Union`/`Dictionary`/`RunEndEncoded` children. Interface-checked: `arrow_ord::sort::lexsort_to_indices` and `arrow_ord::partition::partition` could replace `stages.rs:31-81` | A framing change must be made in four places or identity silently forks; metadata on skipped container children never enters the hash while the contract comparator checks it; an identity formula edit in one language is not caught by the compiler | `FrameSink` the only writer; `Layout` reduced to `DataType` plus an enum-domain map with the rejection matrix kept as `admit(&DataType)`; every identity formula a `ScalarUDF` invoked from Rust on scalars; complete the metadata walk with `child_fields`. **Do not** swap the sort until an equivalence property test over the full layout matrix passes against `crates/pse-ids/tests/golden_vectors.rs` | Golden vectors unchanged; new mathir-node preimage golden vector (none exists today); property test `lexsort == RowConverter order` |
| F5 | **Nine extension node triples with identical scaffolding, none with metrics** | DM-17, DM-25, DM-52, DM-56 | Implemented: `DomainAlgorithm` (`compiler/native.rs:211-283`), `FixedPoint` (`rules/strata/native.rs:135-191`), `Solve` (`backend-native/native.rs:122-199`) share `Arc::ptr_eq` equality, `Arc::as_ptr` hashing, the same `PartialOrd`, `Debug` → `fmt_for_explain`, `prevent_predicate_push_down_columns` over all fields, an arity-guard `with_exprs_and_inputs`; the six catalog nodes match (`session/contract.rs`, `cache.rs`, `commands/deferred.rs`, `delta/{write,publish,maintenance}.rs`). Measured: 9 `UserDefinedLogicalNodeCore`, 9 `ExtensionPlanner`, 16 hand-written `ExecutionPlan` impls; `apply_expressions` returning `Continue` appears 15 times in `pse-catalog` alone; `UnifiedPlanner::new` registers six planners by hand (`planner.rs:31-39`); `pse_compiler::query_planner` two more. No node implements `metrics()`. `pack` is character-identical in `compiler/native/layout.rs:195-213` and `rules/strata/native/output.rs:167-185` | Adding an algorithm is ≈ 200 lines of scaffolding plus a registration; `EXPLAIN ANALYZE` is blind to every domain node; a fix to the run-once guard must be made nine times | `Algorithm<A: DomainAlgorithm>` + one planner + one exec with `BaselineMetrics` in `pse-engine`; `OneShotCommand<R>` for the four effectful commands; `Cache` excluded from the first cut (`cache.rs:247-252` returns empty `inputs()` as an optimizer-leaf trick) | `just test`; any test asserting `EXPLAIN` text re-read before the change; ≈ 800–950 lines removed (Measured after) |
| F6 | **The metadata-preserving wrapper pattern, five times** | DM-25, DM-41, DM-56 | Implemented: `structure.rs:26-70` wraps `named_struct` and overrides only `return_field_from_args`; `element.rs` wraps `array_element`; `list_concat.rs:55-147` wraps `array_concat` and re-casts (`:130-132` "The native constructor resets the list's child labels"); `aggregate/selection.rs` wraps `min`/`max` and `aggregate.rs` wraps `array_agg` for the same reason (`aggregate.rs:171-173` names the upstream cause). The dispatcher exists: `field_transfer.rs:55-67`. Interface-checked: `ArrayConcat` and `ArrayElement` ship no `return_field_from_args` at 55.1 (`api/datafusion_functions_nested.concat.md:128`, `extract.md:140`); `NamedStructFunc` does. Group C (four `cast_field` callers) is already consolidated in `list_field.rs:87-126` | Each DataFusion bump re-audits five files for one upstream behaviour; a sixth built-in with the same defect gets a sixth file | `FieldTransfer { native, derive }` and `AggregateFieldAdapter`, with a pre-invoke hook for the two that force the array path or cast | `crates/pse-catalog/tests/{native_nested_output,native_union_metadata,columnar_arguments}.rs` |
| F7 | **Owned batches exposed as a table, five hand-written ways** | DM-25, DM-52, DM-58 | Implemented: `MaterializedTable`, `CandidateTable` end at `MemorySourceConfig::try_new_exec`; `RoundTable`+`RoundExec`, `Statistics`+`StatisticsExec`, `ResolutionProvider`+`ResolutionExec` each hand-roll a run-once `ExecutionPlan` around one batch or stream. `delta/dependencies.rs:53-56` already uses `StreamingTable`+`PartitionStream` for the same shape. Interface-checked: `MemTable::{try_new, with_constraints, with_column_defaults}` and its DML methods (`api/datafusion_catalog.memory.table.md`); `StreamingTableExec::try_new` with projection, limit, metrics; `PartitionStream` doc: "less boiler plate than implementing `ExecutionPlan` directly". Blockers quoted: `materialized.rs:4-6` (no DML hooks wanted) and `round.rs:298` (`plan.is::<RoundExec>()` in the reset allow-list) | ≈ 400 lines of provider/exec scaffolding; each has its own projection/limit handling or none | `MemTable` newtype refusing DML; `StreamingTable` for the three produce-once sources; the allow-list keyed on the `PartitionStream` type, not the exec type. `RoundTable` is the exact functional twin of `WorkTable` (Interface-checked, `api/datafusion_physical_plan.work_table.md`) | `crates/pse-catalog/tests/{provider_contracts,native_input_ports}.rs`; fixed-point round tests |
| F8 | **Dependencies, settings and cache state: three hidden behaviours and six copies of one walk** | DM-04, DM-31, DM-32, DM-56 | Implemented: `consumption.rs:86-99` returns whole-relation evidence for any plan containing any `Extension`, and no node overrides `necessary_children_exprs` (grep: zero); `Cache::schema()` and `ExecutionContract::schema()` are pass-throughs. `max_predicate_cache_size` is written at `config.rs:66` (`Some(0)`), `factory.rs:52` (budget) and `cache_service/mod.rs:238-243` (clamp). `is_resource_setting` (`config.rs:113-130`) hand-lists all eight `datafusion.runtime.*` keys, so a patch adding a ninth changes every settings hash. `ResidentCache` and `SnapshotCache` are the same 11-field struct (`resident.rs:72-83`, `snapshot.rs:105-116`). Six memoized plan walks share `admission::identity`, a visited map, a reservation and the `Subquery` special case (`derive.rs:48`, `cache/physical.rs:91`, `observation/graph.rs:71`, `isolation.rs:40`, `input.rs:269`, `freshness.rs:52`). Interface-checked: `necessary_children_exprs` is a provided method on `UserDefinedLogicalNodeCore`; `LogicalPlan::columns_referenced` does **not** exist at this pin | Column-aware reuse (Plan 09 D07) is defeated for every cached or contracted plan; last-writer-wins on a resource setting; a DataFusion patch invalidates every resident cache entry | `Some(vec![output_columns.to_vec()])` on the two pass-through nodes and narrow the opaque gate to unknown extensions; one predicate-cache writer; namespace-prefix classification; `LoadedCache<K,V>`; `PlanWalk::{read,rewrite}` | `consumption.rs::opaque_projection_stops_before_expanding_shared_producers` re-proved with a timeout; `crates/pse-catalog/tests/native_artifact_lifecycle.rs`; `session_validation_budget.rs` |
| F9 | **Passes that decode relations into `Vec`s and join by scan** | DM-18, DM-25, DM-36, DM-38, DM-56 | Measured: 127 `.filter(|row| row.x == y)` equality filters in `pse-compiler`, `pse-templates`, `pse-authoring`; every file with three or more has zero DataFusion joins (`p7/transfers.rs` 13, `templates/paths.rs` 12, `p9/parameters.rs` 8, ...). Implemented: P7 loads 48 relations by 41 separate plan executions (`p7/inventory.rs:131-180`) and strips keys (`p7/inventory.rs:123-126` `.unzip()`), then recovers provenance by `position()` scan (`p7/evidence.rs:279-282`); `p7/evidence.rs:59-66` expands 30 semi-joins by macro; `p3/source/support.rs:335-352` is a per-row transitive closure while `p3/products/expansion.rs:294-296` already uses `to_recursive_query`; `located_input` (`native_rows.rs:411`) and `p8/inventory.rs:204` are the same loader; five plan-vocabulary copies with three incompatible column encodings (`native_construction.rs:36`, `native_sources.rs:21-23`, `native_rows.rs:276`) and two `join` defaults (`native_rows.rs:253` `NullEqualsNothing`, `relational.rs:255` `NullEqualsNull`); `union` duplicated with the same comment (`native_construction.rs:99`, `relational.rs:214`) | The optimizer cannot see the joins; P7 provenance is O(n) per lookup; the same alias-chain cycle check is written twice (`p7/promises.rs:194-204`, `p9/outputs.rs:399-408`) | Inputs stay plans; joins/aggregates/EXCEPT/recursive CTE; P7 retains `(RelationKey, ContentHash)`; one vocabulary in `pse-engine` with `NullEquality` explicit | `just test-package pse-compiler`; incremental-versus-clean oracle (`tests/engine/tests/incremental_equals_clean_p0_p3.rs`); measured P7 wall time before and after, conditions recorded |
| F10 | **Sixteen graph traversals, two of which disagree, while the shared one has no callers** | DM-02, DM-24, DM-56 | Implemented: `walk.rs:23` is `pub` with zero callers; the successor `children ∪ payload.referenced_nodes()` is hand-rolled at 16 sites in 14 files; `p8/expansion/axes.rs:86` extends with `children` only, `p10/contracts.rs:42-43` with both. `topo.rs:25` clones the whole graph per call, four times per canonicalization. The mathir `Graph` is a second representation of `math_expr_nodes` with four in-memory forms between two relation states, and its reconstruction check is a hand-written five-relation sorted EXCEPT (`vec_sink.rs:253-310`) | "Reachable from this root" has two answers in one pass family; canonicalization cost is quadratic in graph size for no reason | `walk` on petgraph `DfsPostOrder` (petgraph is a workspace dependency used at `p5/tears.rs:192`) with the cycle path preserved; generated relation rows as the `VecSink` row type; the reconstruction check as an `EXCEPT` plan | `crates/pse-mathir` tests; a test asserting the two traversal sets are equal on a graph with `referenced_nodes` |
| F11 | **Operator and unit semantics: the table declares what four dispatches do not read** | DM-02, DM-19, DM-24, DM-40 | Implemented: `OPERATOR_TABLE` (`operators.rs:217`) carries `foldable`, `domain_restrictions`, `lowering`; evaluation at `fold.rs:362-388` and again at `p4/predicates/scalar.rs:340-386` (the second folds `Pow`, the first does not, `fold.rs:383` vs `scalar.rs:349-352`); lowering at `scalar_math.rs:181-285`; derivatives at `differentiate.rs:95-175`; a fourth evaluator in `xtask/src/codegen/physical/tests/formulas/evaluate.rs:34-44`. `unit.rs:6-14` promises two roundings and no FMA; `scalar_math.rs:218-220` rebuilds `a*b+c` as an `Expr`, which makes no such promise, and no test cross-checks them | Two folders can produce different constants for the same subtree; the columnar path can contract the conversion into an FMA on a target that permits it | One evaluator driven by the table; `convert_value` exposed as a `ScalarUDF`; the xtask probe deleted or routed through production | Differential test across evaluators; `just test-package pse-numerics` |
| F12 | **Delta layer: internal duplication only, and one declaration that hides another** | DM-02, DM-04, DM-56 | Implemented: every operation is a delta-rs builder (`write.rs:368` `table.write(...).with_input_plan(...)`, `dml/execution.rs:185-226`, `dml/merge.rs:106-138`, `maintenance.rs:566-602`, `changes.rs:88`). Bespoke and justified: attempt receipts (`attempt.rs:110-208`, because `kernel/transaction/mod.rs:479-481` pushes `Txn` without dedup), publication CAS (`publish.rs:410-427`), leases (`lease.rs`, no delta-rs equivalent). Duplicated: three command triples (F5); five commit-log readers, one unbounded (`dml/execution.rs:270-292` vs `attempt.rs:299-326`); the storage direction as `name == "pse_delta_decode"` at `layout.rs:173,185,252,271`; `numerical.rs:133-158` and `quantities.rs:133-158` identical; `fn column(name: &str) -> Expr` seven times; `.with_cleanup_expired_logs(Some(false))` at five sites overriding the registry's `delta.enableExpiredLogCleanup` (`relation.rs:222`) per `kernel/transaction/mod.rs:1065-1069`; `log_cutoff_ms` a caller argument while `delta.logRetentionDuration` is not declared. Interface-checked: `DeltaScan::insert_into` cannot carry commit properties (`data_sink.rs:54` takes three arguments), so `WritableTable::insert_into` stays | A maintenance run reads the registry property and observes nothing; a write-count read has no memory bound; a direction typo is a runtime string mismatch | `OneShotCommand`; `receipt_bytes` the only reader; a `Direction` enum; declare `logRetentionDuration` and derive the cutoff as `checkpoints.rs:61-62` does; one obligation descriptor for the four validators | `crates/pse-catalog/tests/unified_delta_contracts.rs`; `tests/lifecycle` |
| F13 | **The failure taxonomy is declared twice and enforced once** | DM-02, DM-47, DM-60 | Measured: `FailureClass` has 24 members (`enums_platform.rs:378`); 130 inline `#[diagnostic(code(...))]` sites with 58 distinct spellings; 12 spellings are members, 46 are not, 12 members are never used inline. `builder.rs:1685-1697` checks membership for algorithm diagnostics only; `tests/governance/tests/error_taxonomy.rs` checks derives, not codes. `Cancelled` is byte-identical in three crates (`pse-catalog:50`, `pse-ids:199`, `pse-runtime:45`); `Internal` in five; `fn external` 20 copies; `fn invalid` 71 definitions; `classify` and `classify_borrowed` are the same 13-arm match (`failure.rs:110`, `:175`); `pse-numerics/src/error.rs:51` is a competing classifier; `RuntimeError` is `CatalogError` minus the catalog arms | A diagnostic code can be misspelled and nothing notices; a `FailureClass` row and a `code(...)` can name the same idea two ways | `pse-codes` leaf crate with the generated type and a code-minting derive; `Classified` wrapper variant per crate; `classify(Cow<DataFusionError>)` once; `trait FromEngine` per crate | Governance test: every inline code is a member, baseline zero; `just governance` |
| F14 | **One relation described six times; three renderers are one walk** | DM-52, DM-56 | Implemented: declaration (`s4_schema.rs:62`), positional `Vec<Cell>` projection (`builder.rs:449-456`, one of 14, tested for width only), generated Rust, Python attrs, Markdown, JSON Schema; `codegen/python/types.rs:86`, `markdown/mod.rs:27`, `jsonschema.rs:188` walk the same arms with four scalar-storage tables; `jsonschema.rs:13-30` re-implements JSON escaping in a crate that depends on `serde_json` and re-parses its own output at `:137`. Python receives the exact Arrow schema already (`stream.rs:46`, `_inspection.py:45-47`); nothing in the generated contracts is not derivable from it | Four tables to update per new storage type; a positional reorder passes every test | One `FieldVisitor`; JSON Schema built on `serde_json::Value`; row projections by field name; Python contracts kept only as JSON boundary targets | `just codegen-check`; a test binding each `Cell` position to a column name until the projection is derived |
| F15 | **The test fixture is assembled fourteen ways, one of them in production** | DM-52, DM-56, DM-60 | Implemented: 14 helpers (`tests/support/session_factory.rs:16`, `native_publication.rs:36`, `native_pipeline.rs:38`, `lifecycle/tests/support/mod.rs:47`, ...), `xtask/src/inspection_fixture/environment.rs:5` reaching into `tests/support` by `#[path]`, `pse-py/src/inspection/runtime.rs:34` a fourth copy in production; 20 identical `fn id(value: u8) -> SemanticId`; `ThreadBudget{1,1}` as 52 literals in three spellings with no `impl ThreadBudget`; the three `tests/*/src/lib.rs` gate their modules with `#[cfg(test)]` so 60 `#[path]` inclusions exist; two hand-rolled `TempDir`s while `tempfile` is pinned; `insta` pinned and unused for the fixture writer | A fixture fix is a fourteen-file change; the production runtime assembly can drift from the tested one | `pse-testkit` dev crate; `ThreadBudget::single()`; `tempfile`; `insta` for fixtures | `just test`, baseline zero |
| F16 | **The Python boundary restates settings by hand and drops names** | DM-06, DM-41, DM-52 | Implemented: `settings.rs` and `cache_settings.rs` each write 13 ctor parameters and 13 getters for structs the Rust core already declares (`budget.rs:33`, `policy.rs:10`), while `cache_report.rs:7` proves `#[pyclass(get_all)]` works in the same directory; `handles.rs:110-115` returns `(limit, reserved_now, peak, rss)` while `peak.rs:10-22` declares `limit, peak, reserved_now` and `_inspection.py:120` re-names positionally; `listing_ttl_ms` is `Option<u64>` in and `Option<u128>` out; `_transfer.py:34-44` hand-enumerates struct/list children (omitting map and union) where `DataType.num_fields`/`field(i)` and `Field.equals(check_metadata=True)` exist on pyarrow 25.0.1; hex-id admission is lowercase-only in Python (`values.py:34`) and case-insensitive in Rust (`id.rs:56`) | A reorder in `handles.rs` silently mislabels every counter; the same id text is accepted on one side of the boundary and rejected on the other | Flattened `#[pyclass(get_all)]` structs; a named `ResourceUsage` pyclass; pyarrow's own schema comparison; one admission rule | `just py-test`, baseline zero |
| F17 | **Declared but unreachable: a solver stack, a rule generator and five dependencies** | DM-58, DM-59 | Implemented: `Solve::plan` has only test callers; `pse-runtime/Cargo.toml:34` declares `pse-backend-native` with zero uses; `ScalarMath` has no caller outside `pse-numerics` tests; `num-dual` (`pse-numerics/Cargo.toml:22`), `datafusion-proto`, `pse-quantity`, `serde_json` (`pse-rules/Cargo.toml:25,30,31`) and `serde_arrow` (`pse-relations/Cargo.toml:23`) have zero source uses; `FrameSink` and `walk` are documented entry points with zero callers; the 130 `.sql` rule files are machine-generated (`p5-scope-port-no.sql:5-24` `q5 AS (...) c0`) with no generator in the tree and are re-parsed per planning invocation (`plan/mod.rs:102-104`) | Roughly 1,750 lines are counted as load-bearing and are not; the rule SQL is the only authority for a form nobody authored | Wire `ExprGraph → ScalarMath → Solve::plan` into `pse-runtime` or scope the stack out; delete the five dependencies; recover the rule generator or declare the SQL authored; cache parsed rule plans per registry | `just governance` (`dependency_floors`, `every_crate_registered`); `cargo udeps`-style check added to governance |

### Observations that did not rise to findings

Each moves none of the three severity axes and is recorded so the silence is not read as
clean: `formatter.rs:102` hard-codes `FormatOptions::default()` for leaf rendering
instead of threading the caller's options; `document/rename.rs:229` renders target paths
lossily and privately, so no round-trip test is possible; `normalized.source_occurrences`
is built and never read; `VacuumBuilder::with_keep_versions`, which the entire retention
model depends on, warns "experimental API" at runtime (`vendor/.../vacuum.rs:310`); the
`retain_*` constructors in `leased.rs` are four spellings of one function; the P7 support
key is a positional ordinal (`p7.rs:46`) where every other pass uses a content hash.

### Applicable principle verdicts

| Principle IDs | Verdict | Basis |
|---|---|---|
| DM-02, DM-15 | Violated | F1, F4, F13: three grammars, four framings, two taxonomies |
| DM-24, DM-40 | Violated | F10, F11: disagreeing traversals and folders; FMA guarantee dropped |
| DM-04, DM-31, DM-32 | Violated | F8: whole-relation dependencies behind every extension node |
| DM-25, DM-52, DM-56 | Violated | F2, F5, F6, F7, F9, F14, F15: same meaning re-expressed per relation, per node, per pass, per renderer, per fixture |
| DM-36, DM-37, DM-38 | Violated | F3, F9: parallel accounting; decode-then-scan passes |
| DM-58, DM-59 | Violated | F17: unreachable machinery counted as capability |
| DM-14, DM-29, DM-30 | Satisfied in inspected scope | Delta attempt and publication protocol single-authority on disagreement |
| DM-41, DM-43 | Satisfied for the Delta builders; Violated at the Python boundary | F12 (delta-rs used correctly), F16 |
| DM-07, DM-09 | Unresolved | G3: three documented contracts the code does not implement |

## 8. Alternatives and architectural leverage

| Alternative | Semantic duplication and extension locality | Correctness and operational risks | Implementation / maintenance cost | Performance evidence | Why selected or rejected |
|---|---|---|---|---|---|
| **Current baseline** | The 17 findings: two row models, three grammars, nine scaffolds, five vocabularies, 14 fixtures | Silent forks named in §3; hidden whole-relation dependencies | 125 k hand-written lines plus 474 k generated; each library bump re-audits five wrapper files | None claimed | Baseline |
| **Proposed design (§4)** | One owner per meaning; four new crates carry the shared substrate; generated tree shrinks to declarations | Two named risks: the canonical sort equivalence (F4, gated) and serde_arrow nested metadata (F2, spiked); refusal must stay a separate `try_grow` because Arrow's pool reserve is infallible | Estimated ≈ 3–4 k hand-written lines removed net of the new crates, and ≈ 180 k generated lines; the P7/P8/P9 rewrite is the one large item | None claimed; F3 and F9 name the measurements to take | Selected: every item has a library surface confirmed at the pin or an existing in-tree owner |
| **Simpler viable alternative** | Do every deletion and generic helper *inside existing crates*: no `pse-engine`, no `pse-delta`, no `pse-codes`; keep `Cell` but make `literal_spec` the sole writer; keep the generated codec but drop `COMPILED_DECLARATION`; skip serde_arrow; skip the sort change | Same correctness fixes as the proposal (F8, F10, F11, F13 membership check) | Roughly half the proposal's surface; no crate-graph change | None | **Wins for F1's grammar, F2's declaration literal, F4's framing, F8, F10, F11, F13's check, F16.** Loses for F5/F6/F7 (the substrate would still live in `pse-catalog` and six crates would still pull deltalake for it), F13's citation from low crates, F15. The recommended path is therefore the simpler alternative first, then the two substrate crates as the second wave, with `pse-delta` optional |

**Abstractions justified by current needs.** `pse-engine`: demonstrated by nine
consumers of the node scaffold and six crates depending on `pse-catalog` for non-catalog
reasons. `pse-testkit`: demonstrated by fourteen fixtures. `pse-codes`: demonstrated by
the layering blocker (`pse-ids` cannot cite a type generated into `pse-relations`).
`pse-delta`: *not* demonstrated by a consumer; it is proposed for cohesion only and is the
first thing to drop if the maintainer prefers fewer crates.

**What remains ordinary code.** The fixed-point round driver; the attempt receipt
protocol; `pse.canon.v2`; the unit-inference driver (`canonicalize.rs` is a driver, not a
rewriter); `Finite`, `Vector`, `Key`, `Codec` UDFs; `Flights`; the extension formatter;
the Ipopt driver.

## 9. Verification and measurement plan

| Claim or risk | Evidence label today | Test / analysis / benchmark | Conditions and expected result | Current result or remaining gap |
|---|---|---|---|---|
| Three literal encoders agree | Implemented (divergence in arm coverage) | New differential test over every declared relation and every value kind, including NaN payloads and `-0.0` | Byte-equal output; then one encoder is deleted | Test does not exist |
| serde_arrow preserves nested `ARROW:extension:*` metadata and emits no `SERDE_ARROW:` keys for platform rows | Interface-checked (entry points only) | Spike: round-trip every generated `Row` through `to_record_batch`/`from_record_batch` with the registry `Vec<FieldRef>`; compare `Field` metadata recursively; restore schema metadata with `with_schema` | Equal fields and metadata for all 449 relations | Not run; `SemanticId::serialize` emits hex text (`id.rs:232-234`) and must serialize as bytes on the row path |
| `lexsort_to_indices` order equals `RowConverter` order | Interface-checked | Property test over the `Layout` matrix incl. `Dictionary`, nested `List` with interior nulls, `FixedSizeBinary` | Identical index vectors; golden vectors unchanged | Not run; F4 gated on it |
| Arrow `pool` accounting matches today's reservations | Interface-checked | `tests/lifecycle/tests/query_memory_budget.rs` plus peak-RSS before/after on the heater publication | Same refusals; no double charge on export | Not run |
| Column-level dependencies after `necessary_children_exprs` | Implemented gap | `native_artifact_lifecycle.rs`; re-run the 2⁴⁰-occurrence opaque test with a timeout | Projection evidence for cached plans; no DAG blowup | Not run |
| Traversal unification changes no reachable set | Implemented divergence | A graph with `referenced_nodes` traversed by both loops before deletion | Sets equal, or the divergence is a bug report | Not run |
| P7/P8/P9 as plans | Implemented | `incremental_equals_clean_p0_p3.rs` and the engineering publications (`just engineering-inspection`) | Same outputs; P7 wall time recorded with conditions | Not run; no speed claim is made |
| Every inline diagnostic code is a `FailureClass` member | Measured (46 non-members) | Governance test over `crates/*/src` | Zero non-members | Fails today by 46 |
| Rule SQL generator | Implemented gap | Locate or rewrite; regenerate and diff the 130 files | Byte-identical | Unknown |

**Cost accounting.** Construction: the generated tree drops from 474 k to an estimated
60–90 k lines. Validation: per-borrow declaration compare drops from a 24 KB string to a
pointer. Preparation: P7 drops from 41 plan executions to one. Memory: one accounting
authority. None of these is a measured speedup; each is a named measurement to take.

## 10. Exceptions and unresolved decisions

This section is the list the maintainer asked for: the rules, ADRs and blueprint sections
the target design would touch, so the policy question can be judged separately from the
design. Nothing here is a recommendation to change a rule; it is the inventory.

| Target-design item | Rule, ADR or blueprint section touched | What would have to change |
|---|---|---|
| `pse-schema` uses `ScalarValue` (F1) | Blueprint §3.2: "`pse-mathir`, `pse-quantity`, `pse-material`, and `pse-ids` depend on no engine crate; `pse-catalog` and `pse-rules` are the only crates that depend on DataFusion planning types" | The second clause is already false (`pse-compiler`, `pse-authoring`, `pse-numerics`, `pse-backend-native` depend on `datafusion`); a revision row should record the actual layering and admit `datafusion-common` in `pse-schema` |
| Row codec via serde_arrow (F2) | `clippy.toml` and `banned_patterns.rs` ban `SchemaLike::from_type`/`from_samples` and the literal `"SERDE_ARROW:"`; blueprint §4.2 lists a "`serde_arrow` schema … never inferred from samples" as a generated artifact | The inference bans stay (the target uses `SchemaLike for Vec<FieldRef>` from the registry, not inference); the literal ban needs the spike result to confirm no key is emitted |
| Generated artifacts shrink (F2) | Blueprint §4.2 table (typed view, typed builder, validators per relation); ADR-0051 (generated trees committed and diff-checked); ADR-0060 (generated Rust manifest wire) | §4.2's rows become "one generic view/builder over a generated row"; ADR-0051 is unaffected in mechanism |
| Arrow `pool` feature and `RecordBatch::claim` (F3) | ADR-0046 (shared accounted runtime memory), ADR-0055 (owned Arrow reservations), blueprint §14.3 "Accounted allocation" paragraph, §5.3 step 8 | The *guarantee* (reserve before allocate, one owner per buffer) is unchanged; the *mechanism* named in ADR-0055 (`Bytes::from_owner` leases) is replaced; a short ADR |
| `pse_ids::resource` deleted; `tokio_util` token re-exported (F3) | Blueprint §3.2 crate responsibilities for `pse-ids`; `.claude/rules/rust.md` says nothing that blocks it | Responsibility line for `pse-ids` |
| `lexsort_to_indices` in canon (F4) | Blueprint §5.3 step 1 names `arrow_row::RowConverter`; ADR-0045, ADR-0050 | Only if the equivalence test passes; then §5.3 step 1 wording |
| Four new crates (§4.1) | AGENTS.md "adds or removes a crate" → ADR and design review; blueprint §3.2 layout; `tests/governance/tests/every_crate_registered.rs` (`layout_additions.toml`) | One ADR per crate or one ADR for the set; registration rows |
| `pse-codes` generated outside `crates/pse-relations/src/generated` (F13) | `codegen/rust/mod.rs:28` `ROOT`; prime directive 2's generated-path list; `Language::Rust.roots()` | Add a root |
| Identity formulas as `ScalarUDF`s callable from Rust (F4) | ADR-0023 (`blake3` in `pse-ids` only), governance `blake3_owner` | Unchanged if the UDFs live in a crate that calls `pse_ids::derive`; `pse-templates` would depend on `datafusion-expr` |
| P7/P8/P9 as plans (F9) | ADR-0056 (wave-one pass availability), ADR-0063 (indexed template realization), blueprint §14.1 pass contracts | Pass contracts unchanged; the ADRs describe outputs, not the in-memory method |
| `validate_bundle` deleted (F2) | Blueprint §4.2 "Validators" row; `validate/mod.rs:5` doc | Doc correction only; the SQL invariants are already the authority |
| Delta `logRetentionDuration` declared (F12) | ADR-0068 (unified DataFusion/Delta), blueprint §20.1 | A registry default; no decision changes |
| `pse-testkit` and un-gating `tests/*/src/lib.rs` (F15) | Blueprint §24.1 test layers; ADR-0038 workspace layout additions | Layout row |
| Python contracts as JSON-only targets (F14, F16) | Blueprint §21.5 Python contracts; ADR-0024 (`pyo3-arrow`) | §21.5 narrows to the JSON boundary |

**Owner.** The maintainer. **Revisit trigger.** Any of the four spikes in §9 failing
changes the corresponding correction from "replace" to "consolidate in place".

## 11. Decision and implementation changes

**Decision: Revise.** The reason is not volume. It is that three of the findings (F1, F4,
F13) are competing authorities for one fact, four (F8, F10, F11, F12) are behaviours the
code hides or contradicts, and the rest are the same meaning re-expressed at a scale that
makes every library bump a five-file audit. The Delta layer and the rule query compiler
are already where the target wants them and are excluded from the "revise".

| Priority | Change | Principle IDs | Acceptance evidence | Regression protection |
|---|---|---|---|---|
| 1 — correctness | `necessary_children_exprs` on `Cache` and `ExecutionContract`; narrow the opaque gate (F8) | DM-31, DM-32 | Column-level evidence recorded for a cached plan; opaque test re-proved | `native_artifact_lifecycle.rs` |
| 1 — correctness | One integer/float folder driven by `OPERATOR_TABLE`; `convert_value` as a UDF; one traversal (F10, F11) | DM-24, DM-40 | Differential tests across the former copies | Property tests over opcodes and graphs |
| 1 — correctness | One literal writer with a differential test; then delete the other two (F1) | DM-02, DM-15 | Byte-equal over all relations | The differential test stays |
| 1 — correctness | Complete the canonical metadata walk; fix or document `contract_metadata`; `visible_buffers` via `get_slice_memory_size` (F3, F4) | DM-07, DM-15 | Golden vectors unchanged or a versioned frame | `golden_vectors.rs` |
| 1 — correctness | Governance test: inline codes ⊆ `FailureClass` (F13) | DM-47, DM-60 | Zero non-members | `just governance` |
| 2 — leverage | Delete `COMPILED_DECLARATION`, `validate_bundle`, `FrameSink` copies, `Layout`, the five dead dependencies, `FixedBudget`/`PoolReserver` (F2, F3, F4, F17) | DM-56, DM-58 | `just codegen-check`, `just governance`, baseline zero | `cargo udeps`-style governance |
| 2 — leverage | `FieldTransfer`, `OneShotCommand`, `Algorithm<A>`, `LoadedCache`, `PlanWalk`, `FieldRule`, one plan vocabulary, `MemTable`/`StreamingTable` sources (F5, F6, F7, F8, F9) | DM-25, DM-52 | Line counts after; `EXPLAIN ANALYZE` metrics present | Existing catalog and compiler suites |
| 2 — leverage | `pse-engine`, `pse-testkit`, `pse-codes`; `pse-delta` optional (§4.1) | DM-57 | Dependency graph: six crates no longer depend on `deltalake` transitively for non-Delta reasons | `every_crate_registered`, `family-check` |
| 3 — measured | Arrow `pool` + `claim`; P7/P8/P9 as plans; serde_arrow codec; `lexsort` in canon, each behind its §9 spike (F2, F3, F4, F9) | DM-36, DM-39 | The named measurement, conditions recorded | The spike tests become permanent |

**Final check.** The claims match the evidence: every replacement API was opened at the
pin, every count was re-measured, and the four items that could not be established without
running code are gated behind named spikes rather than asserted. The scope matches the
guarantees: no performance is claimed, no rule is judged, the Delta protocol is read and
not attacked. Later extensions have a clear path: one declaration, one `DomainAlgorithm`
impl, one `FieldTransfer` derivation, one fixture.
