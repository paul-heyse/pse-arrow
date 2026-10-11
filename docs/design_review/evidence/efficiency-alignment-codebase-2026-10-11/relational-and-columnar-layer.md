# Relational and columnar layer: bounded supporting assessment

Inquiry **F** of the [principal efficiency-alignment review](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md). This is supporting
evidence, not an independent principal review, finding ledger or qualification receipt. The
principal review owns the combined judgment, finding identifiers (`F01`…) and rule impacts
(`RC01`…); local labels below (`F-nn`) are investigation labels only. Coordinator source
checks, reconciliation and probe results are in the [evidence README](README.md).

Baseline: `main` at `b01a45d71a5f5ab29991ad193679f5588ac3662f` (2026-10-11), production source clean.
Assessed by a fresh read-only `design-reviewer` against Core 3.4, Heuristics for Efficient Architecture
1.0, Process Simulator 1.5 and the [pse-arrow binding](../../design_principles/binding/pse-arrow.md).
No builds, tests or services were run by the inquiry; probes it proposes are listed for later
settlement unless the README records them as executed.

---

This is a focused, bounded supporting assessment, not a principal review. Baseline: `main` at
`b01a45d71a5f5ab29991ad193679f5588ac3662f`, read-only. Standard: Core 3.4, Heuristics 1.0,
Process Simulator 1.5 and the pse-arrow binding. Labels are local (F-01 …).

**Bottom line.** In production, this layer has a much narrower operational role than its size
and the architecture text suggest. DataFusion *query execution* runs on one production path:
physical-inventory admission, a cold, cached path. DataFusion's *expression machinery* runs on
nearly every path, through `ValidationContext` local-field validation of every relation built or
admitted. Its *memory pool* accounts every platform allocation. The most consequential defect is
validation-owner identity (F-01). The owner that should amortize predicate preparation, and carry
established validity across crossings, is minted per call, per session and per query. So stable
preparation repeats, and validity is re-proved at internal crossings that change no premise.

## 1. Scope examined and coverage limits

**Read**

- `crates/pse-engine`: `session/{factory,engine_session,assembly,preparation (entry points and checked_relation),execution (validation binding),roles (checked workspace),materialized,registry,assurance}.rs`, `cache_service/{mod,envelope,policy,settings,syntax,inspection,metrics}.rs`, `resources.rs`, `validation.rs`.
- `crates/pse-relations`: `validate/{mod,planner,prepared,occurrences,row_checks}.rs`, `columnar.rs` (the `FieldCheckedBatch` admission and readmit API, `BatchBuilder`, `take_reserved`, `concat`, `retained`), `columnar/{collection,row}.rs`, plus one generated builder/view (`generated/runtime/solve_variables.rs`, read only).
- `crates/pse-rules`: `invariants.rs`, the head of `invariants/program.rs`, `references.rs`.
- `crates/pse-schema`: `arrow.rs` (`relation_schema_ref`, `native_checks`), `builder.rs::contract`.
- `crates/pse-columnar`: `resource.rs`, `flight.rs`, `owned_buffer.rs::export`.
- Consumers: `pse-runtime/src/{physical.rs,physical/plans.rs,workflow/physical.rs,workflow/physical_cache.rs,workflow/mod.rs (Runtime),env.rs,session_factory.rs,math.rs (MathService::new/reserve),math/jobs.rs (reserve_entry),math/retention.rs (retry_growth)}`, `workflow/{results,result_export,modeling_results,modeling.rs::modeling_from_documents,worker.rs::{physical_from_sources,package_from_sources}}`, `authoring_driver/{native,p0}.rs`, `document/owned.rs::validate_context`, and `pse-py/src/{inspection/*,workflow.rs::physical_from_documents,workflow/modeling.rs}`.
- Pinned upstream: `datafusion-execution-55.1.0/src/memory_pool/{pool,peak_recording,mod}.rs`, `arrow-data-59.3.0/src/data.rs` (`force_validate`), `arrow-ipc-59.3.0` (reader validation default).

**Not read or not settled**

- Most `pse-engine/session/*` internals beyond the entry points: `round`, `traversal`, `cache/{logical,admission,physical}`, `commands`, `mutation`, `schema_transform`, `obligation`, `freshness`, `operation/`, `provider/`.
- The generated code beyond one relation.
- `pse-schema` catalog contents.
- Engine reachability is a name/call-site search over `pse-runtime/src`, `pse-rules/src` and `pse-py/src` (non-test), not a complete call graph.
- No builds, tests or probes were run, so every frequency and cost statement is static.
- Library skills (datafusion, serde-arrow) were not loaded. Library facts were checked directly in the pinned upstream source under `~/.cargo/registry/src`.

## 2. Operations examined

**(a) Production use of DataFusion query execution: physical-inventory admission.** This is the
only production use found. Searches for `prepare_rule_plan`, `prepare_many`, `execute_group`,
`.sql(` and `SessionContext` across consumers find, outside tests, only `physical.rs` and
`physical/plans.rs`.

| Stage | Owner | Crossing / live state | Validation |
|---|---|---|---|
| Physical source bytes | `workflow/worker.rs::Operations::sources` | SurrealDB RPCs (E's area) | — |
| Document load | `worker.rs::physical_from_sources` → `load_package_documents_owned(…, &validation)` | Factory context V1 | Builder `finish` evaluates (scan 1) |
| Physical admission | `workflow/physical.rs::PhysicalContext::from_documents` → `documents.validate_context(…, sessions.validation_context())` | Factory context V2 (see F-01) | Re-evaluates if V2 ≠ V1 (scan 2) |
| Inventory session | `sessions.clone().with_target_partitions(MIN).candidate_checked(physical…)` | Fresh `models` map, new `ModelAssembly`; `roles.rs::with_checked_workspace` → `readmit_context(session.validation_context())` | Session context V3, re-evaluates (scan 3) |
| Inventory queries | `physical.rs::PhysicalInventory::load`: about 13 scan+sort roots via `prepare_many`/`execute_group`; `plans::units` (merge + conflicts); `plans::context` (distinct) | Full preparation pipeline per root: rebind, semantic-field restore, effect isolation, logical-cache staging, analyzer, freshness, optimizer, physical planning, CPU admission | Per-query context V4…Vn from `NativeExecutionContext::bind`; `checked_relation` admits outputs (scan 4) |
| Retention | `workflow/physical_cache.rs::PhysicalCache` | Single-slot `Mutex<Option<Admission>>` and `Mutex<Option<Rows>>`; flights coalesce identical loads | — |

**(b) Production use of DataFusion expressions: local field validation.** Every generated
builder (`columnar/row.rs::RowBuilder::with_registry` → `BatchBuilder::new` →
`ValidationContext::relation`) prepares the relation contract. `BatchBuilder::finish` evaluates
it through `PreparedLocalContract::evaluate`. Entry points:

- result encoding (`results.rs::encode`, once per relation cursor);
- canonical source ingress (`modeling/canonical.rs::persist`, `full_source`);
- analysis and study tables;
- document loads;
- IPC reopen (`FieldCheckedBatch::admit_owned` per block).

**(c) Production use of DataFusion's memory pool.** The single deployment pool is built by
`resources.rs::EngineResources::build`. It is a four-layer stack: `ReleaseNotifyingPool` →
`PeakRecordingPool` → `TrackConsumersPool` → `FairSpillPool`. Platform reservations use it
everywhere (`MemoryConsumer::new(..).register(pool)`).

**(d) Test-only in production terms.** No production caller was found for:

- `pse_rules::invariants::run_invariants` (only `crates/pse-rules/tests/`);
- `authoring_driver::native::relation_plan` and `authoring_driver::p0::resolve` (only `crates/pse-runtime/tests/`);
- `EngineSession` SQL entry points and `cache_service::syntax::parse` (via `session/native.rs` and `commands/insert.rs`);
- `cache_service::{bind_state, bind_state_with_generation}` (only `cache_service/tests.rs`);
- `schema_transform`, which is not referenced outside `pse-engine`;
- the inspection UDTFs `pse_cache_statistics` and related functions (reachable only through a SQL session; pse-py exposes no SQL entry);
- `FieldCheckedBatch::filter_reserved`.

## 3. Candidate findings

### F-01: Validation-owner identity is transient

**Summary.** Predicate preparation repeats, and established validity is re-proved at internal
crossings.

**Observed cause** (*Interface-checked*, source-traced)

- **Factory-level context.** `session/factory.rs::EngineFactory::validation_context` stores the
  context in a `OnceLock` inside a `ModelAssembly`. `EngineFactory::model_assembly` keeps only
  `Weak<ModelAssembly>` in `models`. The only strong holders are `EngineSession.native`,
  `assembly::Selection.native` and the struct in `session/reuse.rs`. In production a session
  exists only during physical-inventory admission. That session is opened through
  `with_target_partitions`, whose `refresh_cache_identity` replaces `models`, so it never holds
  the deployment factory's assembly. The `retained_extension` doc comment forbids extensions from
  retaining assemblies.

  Therefore each `Runtime::validation_context()` call (`workflow/mod.rs`) rebuilds the
  `ModelAssembly` and returns a fresh, empty `ValidationContext`. The rebuild covers:
  `SessionStateBuilder::new_from_existing(..).build()`, the extension-type registry,
  `cache_service::inspection::bind` (which mints yet another `ValidationContext`),
  `function_inventory` sorting and hashing, and `config::inventory`.

  Callers include every per-relation result encode (`modeling_results.rs::encode_modeling` and
  peers, one per `ResultCursor`), canonical source ingress (`canonical.rs::persist`,
  `full_source`, `fit_declaration`), analyses, fitting results, study tables, `physical.rs` and
  Python ingress (`pse-py/src/workflow.rs:physical_from_documents`,
  `workflow/modeling.rs` package load).

- **Further per-instance owners.**
  - `engine_session.rs::EngineSession::validation_context`: one per `Selection`.
  - `engine_session.rs::bind_candidates`: one local context.
  - `execution.rs::NativeExecutionContext::bind`: one per prepared query, used by
    `preparation.rs::checked_relation`.

- **Reuse test is instance identity.** `columnar.rs::LocalFieldAdmission::matches` →
  `PreparedLocalContract::same_native_owner` compares a per-instance `Arc<()>`.
  `readmit_context`/`validate_context` therefore re-evaluate whenever the instance differs, even
  when functions and settings are identical.

- **`Collection::finish` drops the admission.** `columnar/collection.rs::Collection::finish`
  rebuilds the batch with `FieldCheckedBatch::from_parts(...)`, whose admission is `None`. That
  discards the admission `BatchBuilder::finish` just minted after a full scan. `concat`,
  `take_reserved` and `canonicalize` do carry admission, so this is an isolated defect in the
  carrying mechanism.

**Consequences**

- **Repeated preparation (H12, H14, DP-10).** Each relation built or admitted prepares its
  contract from scratch. Preparation means a per-field `PhysicalExpr` via
  `occurrences::Node::prepare` → `prepare_expression` (coerce, simplify, create), plus SQL CHECK
  binding in `row_checks::bind`. Each pass also rebuilds the assembly. In a durable modeling run,
  about 22 relation encodes each fetch a context. Over a thousand-point sweep this becomes on the
  order of 22k assembly rebuilds and 22k contract preparations of unchanged declarations
  (count inferred from call sites, unmeasured).
- **Re-proved validity (H19, H20).**
  - `package_from_sources` loads under one context, then `modeling_from_documents` →
    `OwnedDocumentSet::validate_context` under another. That re-encodes every modeling
    declaration through `modeling_validation_required`/`modeling_batch` and re-evaluates every
    source batch.
  - Physical admission scans the same physical rows three to four times (operation table (a)).
- **Unpredictable reuse.** Reuse depends on unrelated concurrent session liveness: an
  overlapping session can keep an assembly alive briefly.
- **A cache on the wrong path.** The syntax cache exists for SQL entry points with no production
  caller, while the validation SQL that does repeat bypasses it.

**Alternative.** Give validation one immutable owner per registry-implementation owner and
factory implementation identity:

1. The factory holds its assembly strongly, either with a strong per-registry slot or with the
   validation context directly on `EngineFactory`. No ownership cycle exists: the assembly holds
   the session state and registry, not the factory.
2. Sessions and per-query execution contexts reuse that owner unless their bound state changes
   function implementations or semantic settings. `ModelAssembly` already computes
   `functions: ContentHash`, `settings` and `generation`.
3. `LocalFieldAdmission` is keyed by that semantic implementation identity instead of an
   instance `Arc<()>`.
4. `Collection::finish` carries `checked.admission`.

This is library-native: DataFusion `SessionState` is immutable and cloneable, and prepared
`Arc<dyn PhysicalExpr>` values are reusable across batches.

**Preserve**

- A context whose function bindings, UDF implementations or semantic settings differ must still
  re-evaluate.
- The registry implementation-owner check stays.
- SQL CHECKs stay bound by the actual engine.
- Raw ingress (IPC, FFI and external batches) still admits.

**Uncertainty.** The per-call rebuild follows from enumerating strong holders (search scope as
listed in §1), but it has not been observed. The rebuild and preparation costs are unmeasured
(probe P1).

**Heuristics / principles:** H12, H14, H19, H20; AP-07, DP-09, DP-10.

**Label.** Diagnosis *Interface-checked*. Remedy *Proposed*.

### F-02: Local-field evaluation does failure-diagnostic work on the success path

**Observed cause** (*Interface-checked*)

- `validate/prepared.rs::PreparedLocalContract::evaluate_selected` runs `to_data().validate_full()`
  on every column of every evaluated batch. That is Arrow structural validation (offsets,
  UTF-8, nulls), including on arrays that Arrow builders, DataFusion kernels or the IPC reader
  just produced. arrow-ipc 59.3 validates by default, so on reopen the IPC reader validates
  first and `admit_owned` validates again.
- `validate/occurrences.rs::Node::root/visit` materializes one `Location { row, Arc<PathCoordinate> }`
  per row per field. It allocates a new `Arc<PathCoordinate>` per value for every struct child
  (`PathCoordinate::field`) and every list element (`PathCoordinate::index`), and builds
  `indices` and `visible` vectors.
- `gather` returns the original array when every row is visible. It copies (`take`) for columns
  with nulls or masks, and for list-child chunks that are not the whole child array, even when
  those are contiguous ranges a `slice` would cover.
- `columnar.rs::BatchBuilder::finish` re-evaluates the whole concatenation, including chunks
  appended through `append_batch`/`append_view` from already-checked views.

**Consequence.** Validation cost grows with rows × fields × nesting depth, beyond the predicate
kernels themselves. It is paid at every relation construction or admission: durable result
encoding for long trajectories and large sweeps, every reopened block, document loads. Under
F-01 it is paid repeatedly. Diagnostic paths are needed only for failing values (H26).

**Heuristics / principles:** H26, H15, H20, H16; AP-07, DP-10.

**Alternative**

- Evaluate each field predicate directly on the visible column slice.
- Find violating rows from the boolean result, via `BooleanArray` true-count or `filter` on the
  negation.
- Build `PathCoordinate`s only for failing rows, lazily.
- Use `slice` for contiguous list-child ranges.
- Keep `validate_full` at genuine raw ingress (FFI/C-stream import, unchecked or `unsafe`
  construction), not after safe constructors or the validating IPC decoder.
- Let `finish` evaluate only rows pushed since the last admitted chunk.

**Preserve**

- Exact diagnostics: rule, nested path, row.
- Masked children (inactive tagged arms, null parents) are never evaluated.
- Bounded findings and cancellation.
- Structural safety at the FFI boundary.

**Uncertainty.** The share of encode/reopen time is unmeasured (probe P3). Nesting depth of
high-volume relations was not surveyed.

**Label.** Diagnosis *Interface-checked*. Remedy *Proposed*.

### F-03: Accounting granularity amplifies global pool locking and wakeups

**Observed cause** (*Interface-checked* against pinned DataFusion 55.1 source)

- **Per-registration cost.** In `datafusion-execution/src/memory_pool/pool.rs`,
  `TrackConsumersPool::register` allocates a `String` name and inserts into a global
  `Mutex<HashMap>`. Every `try_grow`/`shrink` takes `FairSpillPool`'s global state mutex and the
  tracker mutex; drop takes them again plus `unregister`.
- **Notify on every departure.** `pse-engine/src/resources.rs::ReleaseNotifyingPool::{unregister,shrink}`
  calls `notify_waiters()` on every consumer departure, including zero-byte ones. Its comment
  justifies zero-byte notification by `FairSpillPool` allowance changes. Upstream, `unregister`
  changes `FairSpillPool` state only for `can_spill` consumers, and the only spillable pse
  consumer found is in `session/cache.rs`.
- **Consumers churn per row.**
  - `pse-runtime/src/workflow/result_export.rs::SelectedCollection::push_ref` registers a
    `"result:row-copy"` consumer per result row.
  - `result_export.rs::Rows::working` registers one per metric row.
  - `columnar/collection.rs::Collection::push` does a `try_grow` per row.
- **Waiters retry per wake.** `math/jobs.rs::reserve_entry` and `admitted` wake on every
  notification, retry `try_grow`, and on refusal call `retention.rs::retry_growth`. That trims
  retention, and a second trim is with `all = true`.

**Consequence.** With sixteen concurrent cases encoding results, each result row takes several
global mutex round trips. Each row-copy departure wakes every memory-admission waiter, and each
waiter retries and trims. Wake work therefore scales with rows × waiters, exactly when memory is
scarce.

**Heuristics / principles:** H26, H21, H10; DP-20.

**Alternative**

- Reserve at chunk granularity: cursors already batch 1,024 rows.
- Use one registered consumer per collection or cursor and `try_resize`/`shrink` it, instead of
  per-row registration.
- Notify only on actual byte release, or on spillable departure.
- Consider whether transient scratch consumers need individual `TrackConsumersPool` entries,
  since the top-consumer report serves only exhaustion messages.

All of this uses DataFusion's own `MemoryReservation` API (`try_resize`, `split`, `take`).

**Preserve.** Reserve-before-allocate, the finite total, exhaustion diagnostics naming top
consumers, and progress for waiters when capacity is actually freed.

**Uncertainty.** Contention magnitude is unmeasured (probe P2). Ownership is shared: the per-row
call sites belong to E, the admission loop to D, and the pool composition to this layer.

**Label.** Diagnosis *Interface-checked*. Remedy *Proposed*.

### F-04: Engine machinery is out of proportion to its production role

**Observed cause** (*Interface-checked*, search scope §1, not a full call graph)

- Production query execution reaches only physical-inventory admission (§2(a)).
- The registry invariant programs (`pse-rules/src/invariants*.rs`, about 1.1k non-test lines)
  have no production caller. Architecture §14.2 says "registry invariant queries support explicit
  inspection", and the architecture-overview library table lists "invariant checks, inspection"
  as DataFusion roles. No inspection entry point exists in `pse-py`.
- Also test-only in production terms: authoring native transforms, SQL entry plus syntax cache,
  DML commands and mutation, `schema_transform` (1.1k lines), object-store file-cache binding
  (the metadata/statistics/listing `Envelope`s, `StoreBinding`) and the inspection UDTFs.
- A name-level scan finds 122 of 226 `pub fn` names in `pse-engine` unreferenced by
  `pse-runtime`, `pse-rules` and `pse-py` non-test code. This is a crude indicator only.
- Python cache settings default to zero bytes for the file-cache envelopes
  (`cache_service/settings.rs`), so no capacity is wasted by default.

**Consequence (H27, DP-16, PSE-S06)**

- The layer's upgrade, compile and test surface is carried through every DataFusion upgrade,
  every `pse-runtime` build and every assembly rebuild (F-01) without a production consumer.
- The documented inspection capability does not exist for users.
- The workload a relational engine would actually serve has no relational route: inspecting
  results across the points of a study or sweep, and diagnosing a recycle or flowsheet over many
  runs. Results are reopened only through bespoke per-relation, per-range readers
  (`connected_results.rs`).

**Alternatives** (the principal chooses; both conform)

1. **Give the capability a real role.** A DataFusion `TableProvider` over canonical result
   blocks would push run, relation, partition and ordinal-range filters into
   `result_block_page`, and expose relational inspection of studies and invariants through the
   Python boundary (H3, H8; DataFusion's native pushdown and statistics). This is the
   library-owned route for set-oriented result work.
2. **Retire the unreachable machinery** under DP-16's deletion rule, keeping physical inventory,
   validation, the pool and the containers.

**Preserve.** Full capability eligibility (a library feature may stay without a consumer). The
point here is bespoke wrapper machinery without an architectural role, not library features.

**Uncertainty.** Internal reachability of `round`, `traversal`, `cache/*` and `operation/*` was
not traced. Some may serve physical inventory through `prepare_scoped`. The build and test
consequence belongs to inquiry G.

**Label.** Diagnosis *Interface-checked*. Remedy *Proposed*.

### F-05: Physical-admission retention is single-slot

**Observed cause** (*Interface-checked*). `workflow/physical_cache.rs::PhysicalCache` retains
one `admission` and one `rows` entry. A miss evicts the previous entry before loading (the
`admission_fence.clear` in `physical_source`). `Runtime::physical_source` runs per study point
(operation map b.3 #6) and per durable registration.

**Consequence.** Interleaved independent cases with distinct physical sources in one process
re-run the miss path per point: source RPCs, document parse and load, DataFusion inventory
admission (§2(a)). The sixteen-case premise and concurrent studies are the scenario (H14).

**Alternative.** A small bounded LRU keyed exactly as now (`PhysicalAdmissionKey`), within the
existing 16 MiB retention limit.

**Preserve.** Per-caller fresh protections, epoch fences, clear semantics.

**Uncertainty.** This is material only if distinct physical sources co-occur. Shipped packages
may share one physical package, and the flights already coalesce identical sources. Probe P4.

**Label.** *Interface-checked*. Remedy *Proposed*.

### F-06: The interactive runtime is built with Arrow `force_validate`

**Observed cause** (*Interface-checked*)

- `justfile` recipes `py-sync` and `py-sync-native` build the editable extension with
  `--features force-validate`. That is the operator's Python runtime for edit → re-solve,
  sweeps and studies.
- In `arrow-data-59.3.0/src/data.rs`, `force_validate` makes every skip-validation construction
  run `validate_data()`.
- The release maturin configuration (`pyproject.toml` `[tool.maturin]`) enables no features, and
  `py-measure-production` exists separately.

**Consequence.** The interactive runtime pays full Arrow validation on every array construction
(builders, kernels, IPC decode, DataFusion operators), on top of F-02's checks. Measured
production builds differ from the runtime the operator actually uses (H20, H26).

**Alternative.** Build the interactive extension without `force-validate`, and give Python
correctness tests a separately built force-validated extension or environment.

**Preserve.** ADR-0170's guarantee that every Arrow-consuming correctness closure runs
force-validated.

**Uncertainty.** The operator may intend the editable extension as a test environment. Inquiry G
owns build selection; this is a seam.

**Label.** *Interface-checked*. Magnitude unmeasured (probe P5, not light).

## 4. Opportunities and uncertainties that are not findings

- **Physical inventory through the full session pipeline** (§2(a)). Fixed planning machinery
  likely dominates for small reference relations, but the path is cold and cached. Relational
  merge and conflict detection fit DataFusion (DP-13). Retain; F-01 removes its repeated scans.
- **`take_reserved` row-by-row copy.** `columnar.rs::FieldCheckedBatch::take_reserved` copies
  per index with `MutableArrayData::try_extend` for every supported type. The stated reason
  (Arrow list-take child preallocation) applies only to lists, so primitive and string columns
  lose the vectorized `arrow::compute::take` kernel (DP-14). The only production caller,
  `connected_results.rs` `output_results`, selects small subsets. Low priority.
- **Seam A.** `canonical.rs::persist` builds and drops a full `modeling_declarations` batch on
  every edit purely to validate. This is legitimate at the Python edit ingress (H18), but it
  re-validates unchanged rows, and F-01 makes its contract preparation repeat.
- **Seam E.** `result_projection` constructs `View::from_checked` per row in scalar-cell
  indexing (operation map a.4).
- **`validate_schema` per reopened block.** A full field and metadata comparison per block at
  the trust boundary is legitimate. Keep it.
- **`FieldCheckedBatch::retained` copy.** It copies batches without a pool-accounted owner
  (`owned_buffer.rs::OwnedRecordBatch::export`). This is justified for isolation and accounting
  at external ingress (H16).

## 5. Strengths and preservation constraints

- **Registry and contract memoization.** `pse_schema::relation_schema_ref` serves from the frozen
  `ContractArena`, and `Registry::contract` returns arena handles. No per-call registry
  reassembly.
- **`FieldCheckedBatch` capability design.** Local-only evidence, explicitly not uniqueness or
  completeness. `concat` carries admission only across one owner. `take_reserved`/`canonicalize`
  transfer validity through known permutations. `CompletedComputation::checked_relation`
  memoizes admitted output. These distinctions must survive F-01's remedy.
- **`ImmutableTable`** (`session/materialized.rs`). A thin, stated-reason wrapper over native
  `MemorySourceConfig` (immutability, no DML hooks). Good library fit.
- **The leaf-expression pushdown opt-out** in `factory.rs::from_builder` is narrowly scoped with
  a stated upstream reason (DP-15).
- **`pse-rules/src/references.rs::admit_entities`.** A near-linear, path-memoized closure over
  the complete input. That is legitimate global work, and a small owned fold beats SQL here
  (DP-14).
- **Correct placement.** Mathematics is not evaluated through DataFusion (ADR-0082 placement is
  sound).
- **`pse-columnar` ownership.** `flight.rs` completion-owned single flights and
  `resource.rs::AllocationLease` reserve-before-allocate ownership are sound.
- **The IPC reader's default validation** at reopen is the right trust-boundary check. Keep it;
  remove the second check (F-02).

## 6. Positions on prior leads

| Lead | Position | Evidence |
|---|---|---|
| 10-09 evidence: registry `OnceLock`, no per-call reassembly | Confirmed (adequate) | `pse-schema/src/arrow.rs::relation_schema_ref`, `builder.rs::contract` |
| 10-09 evidence: `ContractArena` shares resolved schemas | Confirmed | Same |
| 10-09 evidence: prepared contexts and `ImplementationCache` reuse construction | Residual (refuted in effect) | Reuse holds within one `ValidationContext`; production mints contexts per call, session and query (F-01) |
| 10-09 evidence: `readmit_context` reuses same-owner admission | Confirmed mechanism, ineffective at crossings | Instance-keyed `same_native_owner`; `Collection::finish` drops admission (F-01) |
| 10-09 evidence: `take_source_copies` counts over the whole source; `take_reserved` forecasts whole-source extent | Resolved, residual | `selected_take_extent` and index-only sparse counting now exist; whole-source fallback remains for dictionary/union/view; new row-by-row copy note in §4 |
| 10-09 evidence: `filter_reserved` has no production caller | Confirmed still present | Only uses are in `columnar.rs` tests |
| 10-09 F04 (force-validate selection; build side owned by G) | Not examined (build side); runtime side moved to F-06 | `justfile::py-sync{,-native}` |
| Scoped Arrow validation review / ADR-0170 | Not examined for test closures; its target does not address the interactive runtime build | F-06 |
| Relation-local result transport review / ADR-0169 | Confirmed implemented (relation-local projections); residual: each relation projection acquires its own validation context | `results.rs::encode` → `modeling_results.rs::encode_modeling` → `Runtime::validation_context` |

## 7. Candidate rule impacts

| ID | Rule and where recorded | Proposed change | Depends on | If the rule is kept |
|---|---|---|---|---|
| RC-F1 | Blueprint §4.6: predicates are "bound once against the actual invocation through an explicit immutable ValidationContext created by its actual engine factory or session"; ADR-0150 supplement | Bind once per semantic implementation identity (registry owner + function/setting identity), reused across sessions and queries; admission keyed on that identity | F-01 | Keep instance binding but hold the factory context strongly for the factory lifetime. This removes per-call rebuilds, not cross-session re-evaluation. |
| RC-F2 | Architecture overview library table and §3.3.1/§14.2 DataFusion roles ("invariant checks, inspection"); §5.4 provider catalog | Either commit to an exposed relational inspection role over results and invariants, or narrow the roles to admission/inventory, validation, pool and containers and retire unreachable machinery | F-04 | The documented capability remains unimplemented; the machinery remains as maintenance burden |
| RC-F3 | AGENTS.md "Correctness tests retain explicit force-validation"; `justfile::py-sync`/`py-sync-native`; `docs/dev/validation-assessment.md` | The interactive extension builds without `force-validate`; Python correctness runs use a force-validated build | F-06 | Interactive runs carry force-validation cost; record it as an accepted cost |

## 8. Proposed light probes

Run under `scripts/pse-env --resource-class light`. Each needs a scratch test, not committed.

- **P1 (F-01).** In a scratch `pse-runtime` lib test:
  1. Build `SharedRuntime::build(<test budget>)` and `session_factory(native_engine_profile())`.
  2. Call `factory.validation_context(&shared_registry())` twice. Assert and print
     `Arc::ptr_eq` and `shared.caches().execution_report()` `assembly_builds`.
  3. Build a `solve_variables::Builder` under each context and print `ctx.prepared_count()`.

  Command: `scripts/pse-env --resource-class light -- cargo t -p pse-runtime --lib <scratch_test> -- --nocapture`.
  Discriminating outcome: `ptr_eq == false` and `assembly_builds` rising by 2 confirms F-01;
  `true` refutes it.
- **P2 (F-03).** Scratch test on `EngineResources::build(..)`'s pool: 16 threads × 100k
  iterations of register + `try_grow(512)` + drop, against one reservation per thread with
  `try_grow`/`shrink` per 1,024 rows. Also count wakeups of one parked `released().notified()`
  waiter. Discriminating outcome: per-row cost and wake count relative to a typical per-row
  encode cost.
- **P3 (F-02).** Scratch release-profile test: build a 100k-row `solve_variables` batch and a
  nested relation; time `PreparedLocalContract::evaluate`, separating the `validate_full` share
  and the `Node::visit` location work from the predicate kernels. Discriminating outcome: whether
  the non-kernel share is material.
- **P4 (F-05).** Read-only: count distinct physical-document identities across
  `tests/fixtures/packages/*` and the shipped reference packages, for example
  `rg -l "quantity_types" tests/fixtures/packages`. If one physical package serves all, F-05 is
  low.
- **P5 (F-06, not light).** Time an existing short Python durable sweep under the
  `py-measure-force` and `py-measure-production` builds.
