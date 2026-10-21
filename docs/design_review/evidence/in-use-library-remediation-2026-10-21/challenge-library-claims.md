# C3 library-claim verification of the principal draft

> **Supporting evidence** for [Design review: in-use-library remediation of the first design-principle audit](../../reviews/design_review_in-use-library-remediation_2026-10-21.md).
> Role: Independent challenge C3 of the first draft (library-claim verification).
> Baseline: source `a791c533e`, audit publication `0530dcaa1`, 2026-10-21. Paths under `/tmp/...scratchpad/` refer to the review's working copies; their published equivalents are in this folder. The principal review owns the judgment; this file is input to it.

**Result:** 7 claims are wrong or partly wrong, and 9 are verified but miss a condition that matters. A few more need only a citation fix, and about 40 check out. No recommended item falls outside the search space.

Everything was checked read-only against the pinned sources, plus one file from Uno upstream at `805a27ec`. Nothing was built or run.

Path prefixes used in the tables:
- `$REPO` = `/home/paul/pse-arrow`
- `$REG` = `/home/paul/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f`
- `$PNC` = `/home/paul/.cargo/git/checkouts/pounce-8316ac8e070882e5/7f5e2e9`
- `$LM` = `/home/paul/.cargo/git/checkouts/levenberg-marquardt-1b6a798b28a10100/e67066c`
- `$SUN` = `/home/paul/pse-arrow/vendor/sundials-sys/vendor`

## A. Wrong, or right but missing a condition that changes the recommendation

| Claim | Draft location | Verdict | Evidence | Correction |
|---|---|---|---|---|
| `SUNContext_PushErrHandler` "captures native error text and stops default stderr logging" | §6 table | **wrong (partly)** | `SUNContext_Create` installs `SUNLogErrHandlerFn` (`$SUN/src/sundials/sundials_context.c:90`). Push links the new handler in front of the existing ones (`:135-149`). `SUNHandleErrWithMsg` calls **every** handler in the chain (`$SUN/include/sundials/priv/sundials_errors_impl.h:156-161`). The header line is `sundials_context.h:38-39`. | Capturing text works: KINSOL reports through this path (`kinsol.c:2602`). To stop stderr logging, call `SUNContext_ClearErrHandlers` (`sundials_context.h:44`) and then push. Otherwise the default logger keeps running. |
| strum `EnumDiscriminants` "already derived on registry enums"; `EnumMap` gives per-kind tables | R11 | **wrong (partly)** | The registry enums derive `EnumString`, `Display`, `VariantArray` and `IntoStaticStr` only (`$REPO/crates/pse-model/src/generated/enums.rs:15-18`). No crate derives `EnumDiscriminants`. It exists at `$REG/strum_macros-0.28.0/src/lib.rs:950`. `EnumMap` comes from **enum-map 3.1.0** (a direct dependency, used at `pse-backend-native/src/transport.rs:915`), not from strum. | Say "add `EnumDiscriminants` (strum 0.28) with `#[strum_discriminants(derive(enum_map::Enum))]` so `enum_map::EnumMap` can be used". Both are in scope. |
| POUNCE "library's private `run_second_opinion_ladder`" | R14 | **wrong** | It is `pub fn` in a `pub mod` (`$PNC/crates/pounce-restoration/src/second_opinion_driver.rs:139`; `lib.rs:26`), and pounce-restoration is in the lockfile. It drives an `IpoptApplication`. | Drop "private". The reason for rejecting it still holds: it hides the rungs inside one call over an Ipopt application. |
| datafusion-execution `arrow_buffer_pool` with `RecordBatch::claim` | §6 candidates | **wrong (partly)** | `arrow_buffer_pool = ["arrow-buffer/pool"]` (`$REG/datafusion-execution-55.1.0/Cargo.toml`). It provides `memory_pool::arrow::ArrowMemoryPool` (`src/memory_pool/arrow.rs:36`) and `Buffer::claim`. **`RecordBatch::claim` needs `arrow-array/pool`** (`$REG/arrow-array-59.3.0/src/record_batch.rs:797-798`), which `arrow/pool` also enables (`arrow-59.3.0/Cargo.toml:69`). The mutex claim is right: with `pool` on, every `Bytes` carries `Mutex<Option<Box<dyn MemoryReservation>>>` (`arrow-buffer-59.3.0/src/bytes.rs:59-60`). | Name both features: `datafusion-execution/arrow_buffer_pool` plus `arrow/pool`. Through unification both apply process-wide. |
| Salsa LRU keeps memo headers and edges, "so evicted queries re-execute on demand and downstream memos can still backdate" | R8, §8 | **wrong (partly)** | Eviction sets `memo.value = None` (`$REG/salsa-0.28.4/src/function/memo.rs:63-79`). Backdating needs the old value: `old_memo.value().is_some_and(..)` (`function/backdate.rs:22-25`). An evicted memo that re-executes therefore **cannot backdate**, and its dependents re-run. Unchanged evicted memos are still deep-verified through the kept edges. | Make it a condition: the catalog-firewall accessors and `specialized`, whose backdating R8 relies on, must not be LRU-evicted. |
| R8's LRU removes routine rotation (F09) | R8, RC03 | **condition missing** | Evicted memos still count in `memory_usage` (metadata plus revisions; heap 0) (`salsa-0.28.4/src/function/memo.rs:354-367`). `retention_exceeded` compares the **entry count** against 16,384 (`$REPO/crates/pse-compiler/src/workspace.rs:1749-1764`). | LRU lowers bytes, not entries. The ≈54.5k-entry footprint would still trigger rotation. R8 must also change the entry criterion (count only memos with values, or use bytes only), or F09 persists. |
| levenberg-marquardt: "set `with_gtol` from the stationarity request" | F05, §6 | **semantic mismatch** | `gtol` is MINPACK's scale-free cosine test, \|(Jᵀr)ᵢ\| / (‖Jeᵢ‖‖r‖) ≤ gtol (`$LM/src/lm.rs:177-182`, `:461-468`). The repository's stationarity is a normalized absolute gradient (`$REPO/crates/pse-runtime/src/workflow/fitting/oracle/residual.rs:380-400`). `with_ftol`, `with_xtol` and `with_gtol` exist (`lm.rs:154`, `:168`, `:194`). | The request cannot be passed straight through. State a conversion (or a separate declared gtol) and keep the independent check. The LM probe should test the mapping, not only that the setter exists. |
| Symbolica `hot_start` on evaluator rebuilds | §6 candidates | **condition missing** | The direct-translation path returns `EvaluationError::UnsupportedHotStart` (`$REG/symbolica-3.0.1/src/evaluate/tree.rs:327-329`). Only the tree-optimizer path accepts it (`:2664`). | Add: "only off the direct-translation path". This conflicts with R7's `direct_translation(true)` alias candidate. |
| R4 decodes only the surviving blocks with `FileDecoder::with_projection` | R4 step 3 | **verified, but a simpler in-use route exists** | `FileDecoder::with_projection` exists (`$REG/arrow-ipc-59.3.0/src/reader.rs:1044`). The current decoder is `FileReader<ProtectedFile>` (`$REPO/crates/pse-runtime/src/workflow/result_snapshot.rs:89`), which already has `set_index` (`reader.rs:1417`) and `FileReaderBuilder::with_projection` (`:1183`). IPC decodes whole record batches (`FileDecoder::read_record_batch(&Block, ..)`, `:1116`). Result blocks, however, are sub-batch `[start,end)` ranges (`surreal.surql:417-423`). | Prefer `FileReader::set_index` with a projection inside the existing decoder. Decode savings happen only when whole record batches are pruned. Pruning a sub-batch block saves filter work, not decoding. |
| `PruningPredicateBuilder…try_build(expr)` over `PrunableStatistics` | R4 | **verified, with conditions** | `try_build(Arc<dyn PhysicalExpr>)` (`$REG/datafusion-pruning-55.1.0/src/pruning_predicate.rs:483-486`). `PrunableStatistics` reads only `Precision::Exact` min/max, null counts and row counts (`$REG/datafusion-common-55.1.0/src/pruning.rs:342`, `:377-415`). | The scan must physically plan the pushed logical filters. Committed statistics must be `Exact`. `IS NULL` (missing) pruning needs exact null counts. |
| `PyOnceLock::import` caches the per-value module lookups | R17, §6 | **verified, with a condition** | `$REG/pyo3-0.29.2/src/sync/once_lock.rs:158`. | A `static` inside the generic `EnumValue<T>` impl is shared by every `T`. Each `RegistryEnum` impl needs its own cell; the `vocabularies!` macro (`$REPO/crates/pse-py/src/enums.rs:27`) can emit it. |
| `$this.id` in a field `ASSERT` is "Unresolved" | R3, §12 | **can be upgraded to Interface-checked** | Production DDL already uses `$this.…` in field ASSERTs (`$REPO/crates/pse-operations/src/generated/surreal.surql:426-430`). Field clauses are evaluated against `self.doc.current`, and the id is fixed at record-id generation (`$REG/surrealdb-core-3.3.0/src/doc/field.rs:613-619`, `:655`). `record::id` is at `fnc/mod.rs:336`. | Still probe UPSERT and INSERT … ON DUPLICATE KEY. Also note the remaining `WHERE key = $k` reads (for example `canonical_studies.rs:2354`): once the index is dropped they must become record-id fetches. |
| `evaluator_multiple` at `symbolica-3.0.1/src/atom/alias.rs:178` | R7 | **wrong citation** | That line is `AliasedAtom::evaluator_multiple`. The repository and the probe use `AtomCore::evaluator_multiple` (`$REG/symbolica-3.0.1/src/atom/core.rs:1042`; `$REPO/crates/pse-math/src/library.rs:273`). | Cite `core.rs:1042`. |
| "`FunctionMap::add_aliases` with `direct_translation(true)` (`:156`, `:457`)" | R7 | **verified, owner misattributed** | `add_aliases` is on `FunctionMap` (`evaluate/function_map.rs:156`). Its doc says aliases "always inline in the scope where they are used" (`:153-154`). `direct_translation` is on **`EvaluatorBuilder`** (`:457`, impl at `:296`). The direct path still applies Horner (`tree.rs:323-370`). | Name `EvaluatorBuilder::direct_translation`. The rest stands as an unprobed candidate. |
| `is_thrown()` evidence "types `error.rs:172-179`" | R2 | **wrong citation** | `:172-179` is the `Error::thrown` constructor. `is_thrown` is at `$REG/surrealdb-types-3.3.0/src/error.rs:449` (on `Error`) and `:630` (on `ErrorDetails`). The prefix `"An error occurred: {0}"` is verified at `surrealdb-core-3.3.0/src/exec/error.rs:27`. | Fix the citation. |
| "Vendored decoder lacks `KILLED`" | §6 | **verified, too imprecise** | The patch's LIVE verifier accepts only `CREATE`, `UPDATE` and `DELETE` (`$REPO/vendor/surrealdb/src/engine/remote/ws/metadata.rs:93`; `PATCH.md:68-69`). Upstream surrealdb-types parses `KILLED` (`notification.rs:56`), and the SDK stream handles `Action::Killed` (`vendor/surrealdb/src/method/live.rs:300`). | Say "the integration patch's LIVE verifier", so it is not read as an upstream gap. |

## B. Verified

| Claim | Draft location | Verdict | Evidence | Correction |
|---|---|---|---|---|
| Async `DEFINE EVENT` deletes queued work with a warning after retries | C1 alternatives | verified | `$REG/surrealdb-core-3.3.0/src/doc/event.rs:483-503`, `:522-527` | Add the path. |
| `DEFINE SEQUENCE` is i64 and batch-allocated outside commit order | C1, F03 | verified | `$REG/surrealdb-datastore-3.3.0/src/sequences.rs:3-19`, `:114-121`, `:619-638` | Add the path. |
| gRPC engine exists in the same SDK | C1 alternatives | verified | `protocol-grpc` feature (pulls in tonic) in `$REPO/vendor/surrealdb/Cargo.toml`; `src/engine/remote/grpc.rs` | — |
| `ENFORCED`; `REFERENCE … ON DELETE REJECT`; study dependencies lack `ENFORCED` | R3, C1 | verified | `$REG/surrealdb-syn-3.3.0/src/parser/stmt/parts.rs:467`; `surreal.surql:596` vs `:20`, `:34` | — |
| `surrealdb_syn::value` sits behind `canonical-maintenance` | R2 | verified | `surrealdb-syn-3.3.0/src/lib.rs:232`; `$REPO/crates/pse-operations/Cargo.toml:27` | — |
| `$before` in a field `ASSERT` is the field's old value | R1 | verified | `doc/field.rs:668-671` (`before` = `self.old`) | — |
| 62 `<t>_key UNIQUE` indexes; key–id check is client-side only | F07, R3 | verified (line drift) | Emitted at `$REPO/crates/pse-codegen/src/codegen/surreal.rs:135`; checked at `:148`; 62 tables and 62 indexes in the surql | Fix the citations (draft has `:124-127`). |
| `PruningPredicate::try_new` deprecated; builder `new` / `with_file_schema` | R4 | verified | `pruning_predicate.rs:413-431`, `:597`; reachable as `datafusion::physical_optimizer::pruning` (`datafusion-55.1.0/src/lib.rs:834`) | — |
| `PrunableStatistics::new(Vec<Arc<Statistics>>, SchemaRef)` | R4 | verified | `datafusion-common-55.1.0/src/pruning.rs:314-330` | — |
| `normalize_float_zero` in `apply_cmp` | R4 | verified | `datafusion-physical-expr-common-55.1.0/src/datum.rs:26`, `:88-104` | — |
| `scan_with_args` and `supports_filters_pushdown` exist | R4 | verified | `datafusion-catalog-55.1.0/src/cte_worktable.rs:95` (trait in datafusion-session) | — |
| `lexsort_to_indices` already used | R4 | verified | `$REPO/crates/pse-schema/src/builder/native.rs:108` | — |
| Parquet float statistics skip NaN but are reported `Exact` | R4 alternatives | verified (inventory-level) | `parquet-59.3.0/src/column/writer/encoder.rs:432-440`; `datafusion-datasource-parquet-55.1.0/src/metadata.rs:427`, `:486`; `ParquetAccessPlan` at `access_plan.rs:96` | — |
| F06: cache limits set to 0 | F06 | verified | `$REPO/crates/pse-engine/src/cache_service/mod.rs:356-361`; `resources.rs:242`; `CacheManagerConfig` at `datafusion-execution-55.1.0/src/cache/cache_manager.rs:413-451` | — |
| `JoinSetTracer` propagation | R22 | verified | `datafusion-common-runtime-55.1.0/src/trace_utils.rs:28`; `$REPO/crates/pse-engine/src/session/assurance.rs:48`, `:78` | — |
| Symbolica `Dualizer`, `merge`, `count_operations`, `export_cpp`, symbolic `derivative` | R7, §4.4 | verified | `evaluate/dual.rs:521`; `optimize.rs:239`; `evaluator.rs:745`; `export.rs:332`; `atom/core.rs:688` | — |
| `InliningPolicy::Never` sub-evaluator cannot be vectorized | R7 | verified | `function_map.rs:34-45`; `dual.rs:134`; repo test `pse-math/src/library/admission.rs:1048-1050` | — |
| `Indeterminate::Function` formals "as Dualizer does" | C3 alternatives | **partly unverifiable** | The variant exists (`atom.rs:2316`). Its use inside Dualizer was not found (`dual.rs:570` uses `Indeterminate::try_from`). | Keep it a probe item. |
| Salsa `set_lru_capacity`, input `heap_size`, `memory_usage`, tracked fields, `persistence` | R8 | verified | `salsa-macro-rules-0.28.4/src/setup_tracked_fn.rs:507` (needs an `lru=` attribute); `salsa-macros-0.28.4/src/input.rs:68`; `salsa-0.28.4/src/input.rs:359`; `salsa_struct.rs:44`, `:81`; `persistence` pulls in serde and erased-serde (Cargo features) | — |
| `Storage` clones; a setter cancels and blocks until the others drop | §4.4 | verified | `salsa-0.28.4/src/storage.rs:159-189` | — |
| tokio-util `child_token`, `drop_guard`, mutexed `is_cancelled` | R10 | verified | `tokio-util-0.7.19/src/sync/cancellation_token.rs:204`, `:275`; `tree_node.rs:82-84`; `TaskTracker` and `AbortOnDropHandle` under `rt` (enabled) | — |
| F01 line and `child_with_deadline` | F01, R10 | verified | `$REPO/crates/pse-runtime/src/workflow/horizon/driver.rs:142`; `cancel.rs:48-52` | — |
| tracing `Layer` with per-span `extensions_mut` | R22 | verified | Default `fmt` feature implies `registry`; `tracing-subscriber-0.3.23/src/registry/mod.rs:489`; normal dependency of pse-runtime (`Cargo.toml:74`) | — |
| rayon `ThreadPool::drop` only terminates; `spawn_handler`, `build_scoped`, `build_global`, `start_handler` | F04 | verified | `rayon-core-1.13.0/src/thread_pool/mod.rs:399-402`; `lib.rs:277`, `:322`, `:433`, `:638` | — |
| LM `with_tol` sets `gtol = 0` | F05 | verified | `$LM/src/lm.rs:199-215` | — |
| quick_cache `get_or_insert_async`, weighter | §4.4 | verified | `quick_cache-0.6.24/src/sync.rs:79`, `:634`; `Lifecycle` at `lib.rs:167` | — |
| PyO3 `INPUT_TYPE` and `type_hint_identifier!` | R17 | verified | Gated by `experimental-inspect` (`conversion.rs:519-520`), which is enabled (`$REPO/crates/pse-py/Cargo.toml:72`); `inspect.rs:21-31` | — |
| `Duration` ↔ `timedelta` works under abi3 | R18 | verified, with limits | `pyo3-0.29.2/src/conversions/std/time.rs:5-63`; negative values refused (`:49`) | Microsecond resolution. |
| pyo3-arrow `AnyRecordBatch`; no `INPUT_TYPE` | R18 | verified | `pyo3-arrow-0.19.0/src/input.rs:24`; `ffi/from_python/input.rs:8-20`; no `INPUT_TYPE` anywhere in the crate | — |
| schemars `for_serialize` / `for_deserialize`, serialize-contract required fields, `try_from`, `extend` | R11, R17, R19 | verified | `schemars-1.2.2/src/generate.rs:176-183`; `schemars_derive-1.2.2/src/schema_exprs.rs:56-60`, `:727-740`; `extend` already used at `pse-model/src/scalars.rs:65` | `x-pse-order` itself is a post-hoc edit (`pse-codegen/src/codegen/jsonschema.rs:275`), but `extend` exists as a precedent. |
| nutype `PositiveCount` | R11 | verified | `$REPO/crates/pse-model/src/scalars.rs:65` | — |
| msgspec hooks and `msgspec.toml` | R17, R21 | verified | `.venv/.../msgspec/json.pyi:19-33`; `toml.py:25-36` (decode via stdlib `tomllib`) | The bytes-subclass behaviour is still unresolved (binary-only module). |
| Uno status constants (3 / 5) | F02, R15 | verified | Uno 2.x `include/uno/Uno_C_API.h:43-59` (23 identical cached copies under `/home/paul/.cache/pse-arrow/native/uno/`). The bridge is C++, so `static_assert` is viable. | — |
| Uno patch HiGHS options; `qp_regularization_value` default 1e-7 | §4.6 hunks | verified | `$REPO/docker/solvers/uno-pse.patch:93-124`, `:164`, `:284`; `highs-sys-1.15.0/HiGHS/highs/lp_data/HConst.h:356`, `HighsOptions.h:1395-1397` | The HiGHS version in the Uno image is unconfirmed. |
| Uno `"ipopt"` preset | R15 | verified | `uno/options/Presets.cpp` at `805a27ec` (GitHub) | Its linear-solver availability in the image is unverified. |
| Ipopt small step maps to `Numerical`; KINSOL `Function` variants | F02, R14 | verified | `$REPO/crates/pse-backend-native/src/ipopt.rs:57-62`; `kinsol.rs:243-258` | — |
| POUNCE `second_opinion_rungs` | R14 | verified | `$PNC/crates/pounce-algorithm/src/second_opinion.rs:241`; used at `pse-backend-native/src/pounce/profiles.rs:67` | — |
| FERAL `work_estimate().peak_bytes` | R16 | verified, with a note | `$PNC/vendor/feral/src/symbolic/mod.rs:885-935` | `KktFactor::bytes()` (`kkt.rs:762-764`) counts retained memory, where `factor_bytes` is the right figure. Use `peak_bytes` only for admission before numeric factorization. |
| PETSc `PetscMallocSet`, `SNESANDERSON`, `SNESNASM` | R16, C6 | verified (exist) | Image headers: `petscsys.h:1206`, `petscsnes.h:41-42` | The "before `PetscInitialize`" ordering was not checked. |
| SuiteSparse allocator setters | R16 | verified | `$REPO/vendor/suitesparse-sys/vendor/SuiteSparse_config/SuiteSparse_config.h:291-294` | Whether KLU allocates only through them is unresolved. |
| `IDAReInit` / `IDASensReInit` in use | R12 | verified | `$REPO/crates/pse-backend-native/src/dynamics/idas.rs:10`, `:2064` | — |
| pounce-presolve HK/DM/BTF used in production | §2.3 | verified | `$REPO/crates/pse-structural/src/incidence.rs:6` | — |

**Search space:** every recommended item is a direct dependency, a feature of one, or an already-resolved sibling: datafusion-pruning through datafusion-physical-optimizer, pounce-restoration, enum-map, tracing-subscriber, and Uno's own presets. BQPD and MUMPS/SPRAL are mentioned only as absent or as major additions, and the draft recommends neither. Some enabling features have side effects through workspace unification:
- `arrow/pool` adds a mutex to every Arrow `Bytes` in the process.
- `protocol-grpc` pulls in tonic.
- salsa `persistence` pulls in serde and erased-serde.

## Systemic issues

1. **Salsa retention (R8, RC03).** The draft relies on "LRU keeps edges" in two places where the library behaves differently:
   - An evicted memo whose inputs changed cannot backdate, so firewall queries must stay resident.
   - Evicted memos still count as entries, so the entry limit keeps triggering rotation.

   As written, R8 does not settle F09 at defaults. It needs the entry-criterion change and resident firewall memos stated as part of the remedy. This is the most consequential correction.
2. **Library semantics assumed from names.** Three claims follow the API name rather than its behaviour:
   - LM `gtol` is a cosine test, not a gradient tolerance.
   - SUNDIALS handlers chain rather than replace.
   - Symbolica `hot_start` is refused on the direct-translation path.

   In each case the setter exists but the meaning differs, so the "clear win" or "candidate" labels in §6 should carry the condition.
3. **Attribution and visibility errors.** strum vs enum-map, POUNCE "private", `EnumDiscriminants` "already derived", and the `RecordBatch::claim` feature. These are cheap to fix but would mislead plan creation.
4. **R4 granularity.** Arrow IPC decodes whole record batches, while result blocks are sub-batch ranges. The decode benefit happens only at batch level, and the in-use `FileReader::set_index` route is simpler than `FileDecoder`. The exact-statistics requirement of `PrunableStatistics` should be written into the append contract.
5. **Citation drift.** Several cited lines are off by a few lines or point at a sibling item: types `error.rs:172` vs `:449`, `alias.rs:178` vs `core.rs:1042`, generator `surreal.rs:124` vs `:135`. None changes a verdict.
6. **One open item can close.** `$this` in field ASSERTs is already used in production DDL, so §12's R3 row can move from "Unresolved" to *Interface-checked*, with only the UPSERT forms left to probe.