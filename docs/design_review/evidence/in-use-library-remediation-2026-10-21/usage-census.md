# Usage census of in-use third-party libraries (pse-arrow, HEAD 0530dcaa1 = baseline a791c533e + audit pages)

> **Supporting evidence** for [Design review: in-use-library remediation of the first design-principle audit](../../reviews/design_review_in-use-library-remediation_2026-10-21.md).
> Role: Factual usage census of in-use third-party libraries (enabled/disabled features, API areas called).
> Baseline: source `a791c533e`, audit publication `0530dcaa1`, 2026-10-21. Paths under `/tmp/...scratchpad/` refer to the review's working copies; their published equivalents are in this folder. The principal review owns the judgment; this file is input to it.

**Method.** I read the workspace manifests and the per-crate `Cargo.toml` files. Enabled features come from `cargo tree --offline --locked -e features -i <crate> --depth 1`, run through `scripts/pse-env`. Native crates were also traced with `-p pse-backend-native --features native-solvers`, and FeOS/quantity with `-p xtask --all-features`. Offered features come from the pinned sources in `~/.cargo/registry/src/...`, `~/.cargo/git/checkouts/pounce-*/7f5e2e9*` and `vendor/`.

API-area counts are `rg` over `crates/*/src`, excluding `generated/`, `tests/` and `*tests.rs`. Inline `#[cfg(test)]` modules inside `src` are not excluded unless I say so. Counts are approximate occurrence counts, not call graphs. A zero means "not found by that pattern in that scope", not proven absence.

---

## 1. Canonical store (SurrealDB family)

| Library | Pin | Enabled features | Users | API areas used (representative) |
|---|---|---|---|---|
| `surrealdb` (SDK, vendored+patched) | =3.3.0, `path = vendor/surrealdb` | `protocol-ws`, `rustls` only (default off) | pse-operations | **Connection:** `Surreal::new::<Ws>((address, Config::new().bounded_requests().websocket(...)))` at `crates/pse-operations/src/canonical.rs:2057-2065`. **Auth/selection:** `signin(Root{..})`, `use_ns/use_db`, patched `RequestContext::selection_checkpoint` at `canonical_transport.rs:186,235-240,270-272`. **Query:** `db.query(sql).bind(..)` then `IndexedResults::take(n)`; about 248 `.query(` / 677 `.bind(` / 47 `.take(` in non-test src. Example: `canonical_studies.rs:1535` (`RETURN fn::pse_study_v1::claim(...)`), `canonical_staging.rs:1447,1461`. **Record API:** `db.select((table,key))` (14 sites), e.g. `canonical_staging.rs:793,1680`. **Request contexts:** `surrealdb::opt::RequestContext::new` at `canonical_execution.rs:1628`; `opt::ResponseLimits` (2 sites); `method::QueryStatisticsObserver`/`Stats`. **Typed errors:** `types::QueryError::{TransactionConflict, NotExecuted, Cancelled}` and `Error::is_thrown()`, e.g. `canonical.rs:173,967,1104`. **Values:** `types::Object`, `Value`, `Bytes`, `Number`, `RecordId::new`, `encode/decode`. **Transactions:** written as SurrealQL `BEGIN; … COMMIT;` text inside `.query()` (about 99 `BEGIN` / 86 `COMMIT` string occurrences in non-test src; `CANCEL` 2). `surrealdb::method::Transaction` appears 2 times. **Not found in product src:** SDK `.live()` / `LIVE SELECT` / `KILL` (the `.live()` hits in `canonical.rs` are a local drain counter), `.run()` of the SDK, `export/import`. |
| `surrealdb-syn` / `surrealdb-sql` | =3.3.0 | optional, behind `pse-operations/canonical-maintenance` | pse-operations (maintenance only), xtask | `surrealdb_syn::parse` plus the `surrealdb_sql` AST, used to compare deployed definitions (`canonical_maintenance.rs:13,93`). Ordinary open never calls it (`canonical_maintenance.rs:4`). |
| `surrealdb-protocol` | =0.13.1, `flatbuffers` | dev-only | pse-operations [dev] | Mock-peer/transport tests. |
| `tokio-tungstenite` | =0.28.0 | default | pse-operations [dev] | Mock WebSocket peer for transport tests. |
| Server (external `surreal` binary) | `v3.3.0` (`scripts/surreal_server.py:1457`) | n/a | `scripts/surreal_server.py`, activated from `crates/pse-runtime/src/workflow/worker.rs:1621-1660` via `supervisor_script ensure-primary` | Storage URL `rocksdb://<state>/database?sync=every&versioned=false` (`surreal_server.py:6221`). It sets about 20 `SURREAL_*` env knobs (`:5356-5390`), including `SURREAL_WEBSOCKET_MAX_ATTACHED_SESSIONS=3`, RocksDB cache/write buffers/threads and `SURREAL_RUNTIME_WORKER_THREADS=4`. Runtime DDL: `DEFINE NAMESPACE/DATABASE IF NOT EXISTS` (`canonical_transport.rs:306,317`). |

### Census of `crates/pse-operations/src/generated/surreal.surql` (2840 lines)

These are `grep -c` line counts, so a line holding several occurrences counts once.

| Construct | Count | Notes |
|---|---|---|
| `DEFINE TABLE` | 62 | All `SCHEMAFULL`. 6 are `TYPE RELATION IN … OUT …`, 5 of those `ENFORCED` (`:20,34,287,308,523`); `canonical_study_dependencies` (`:596`) is not `ENFORCED`. No `TYPE NORMAL`/`TYPE ANY` written explicitly. |
| `DEFINE FIELD` | 500 | 12 use `OVERWRITE` (the relation `in`/`out` fields). Types: `string` 293, `decimal` 71, `bool` 32, `bytes` 29, `int` 3, `option<string>` 29, `option<decimal>` 11, `option<bytes>` 10, `option<object>` 4, `option<float>` 4, `option<int>` 2, `record<…>` 12 (relation in/out only), `array<…>` 17. |
| `ASSERT` | 110 | Integer ranges on `decimal` (`math::floor($value) = $value`, `0dec..u64::MAX`, e.g. `:11`). Enumerations use `IN ['a','b',…]` (14 lines). No `TYPE "a" \| "b"` literal unions (0). |
| `DEFINE INDEX` | 151 | All `FIELDS`; 83 `UNIQUE`. No `COUNT`, `FULLTEXT`/`SEARCH`, `HNSW`/`MTREE`. One query uses `WITH INDEX guard_incarnation` (`:2100`). |
| `DEFINE FUNCTION` | 116 | Namespaces: `fn::pse_execution_v1` 32, `pse_study_v1` 21, `pse_history_v1` 20, `pse_files_v1` 18, `pse_artifact_v1` 14, `pse_retention_v1` 6, `pse_analysis_v1` 5 (first at `:778`). Typed parameters and return types (`-> bool/decimal`). |
| Function bodies | — | `LET` 492 lines, `IF` 496, `THROW` 306, `RETURN` 227, `FOR` 169. `SELECT` 360 (124 lines with `FOR UPDATE` row locks). `UPDATE` 64, `CREATE` 38, `DELETE` 36, `INSERT` 9, `UPSERT` 5, `ONLY` 383. `ORDER BY` 32, `LIMIT` 91. `CONTENT` 21, `SET` 83, `RETURN NONE` 11. |
| Builtins | — | `type::record(` 405, `array::len(` 161, `math::floor(` 82, `object::remove(` 30, `time::micros(` 29, `string::len(` 19, `array::distinct(` 15, `bytes::len(` 12, `object::extend(` 8, `type::bytes(` 6, `array::sort(` 6, `rand::uuid::v4()` 1 (`:794`). `$authority` appears as a local variable, not `$auth` session state; `$session` 0. |
| Zero occurrences | 0 | `DEFINE EVENT`, `DEFINE PARAM`, `DEFINE ACCESS`, `DEFINE USER`, `DEFINE ANALYZER`, `DEFINE SEQUENCE`, `DEFINE API`, `DEFINE BUCKET`, `DEFINE CONFIG`, `DEFINE MODULE`, `CHANGEFEED`, `PERMISSIONS`, `READONLY`, `DEFAULT`, `VALUE` clause, `FLEXIBLE`, `REFERENCE`/`ON DELETE`, `COMPUTED`, `RELATE`, `LIVE`, `sequence::nextval`, `time::now`, `crypto::`, `COMMENT`, `IF NOT EXISTS`, `GROUP BY`, `SPLIT`, `FETCH`, `TIMEOUT` (inside the schema file), `datetime`/`duration` types. |
| Rust-side query strings | — | `FOR UPDATE` 47, `TIMEOUT $pse_rpc_timeout` 15, `INFO FOR` 3, `fn::pse_` calls 181. Example: `canonical_staging.rs:603,1437,1641`. |
| Sibling generated | — | `generated/surreal.rs` (132 `fn::pse_` references); `generated/surreal-policy.json` carries the block, file, index and `max_message_bytes=4194304` limits. |

**Notably unused SurrealDB features (facts).**
- SDK features not enabled: `kv-mem`, `kv-rocksdb`, `kv-surrealkv`, `kv-tikv`, `kv-indxdb` (embedded engines), `protocol-http`, `protocol-grpc`, `ml`, `scripting`, `jwks`, `http`, `native-tls`, `allocation-tracking`.
- SurrealQL features with zero uses: `DEFINE EVENT`, `CHANGEFEED`/`SHOW CHANGES`, `LIVE`/`KILL` (SDK `.live()` too), `DEFINE SEQUENCE`/`sequence::nextval`, `DEFINE ACCESS`/`USER`/`PERMISSIONS` (all auth is `Root`), record `REFERENCE`/`ON DELETE`, `DEFAULT`/`VALUE`/`READONLY`/`COMPUTED` field clauses, literal-union field types, `COUNT` indexes, `RELATE` (relation rows are written as table rows whose `in`/`out` are asserted against key fields).
- The vendored patch notes that the pinned RPC decoder's LIVE grammar handles CREATE/UPDATE/DELETE but not KILLED (`vendor/surrealdb/PATCH.md`).

---

## 2. Data, relations and results (Arrow / DataFusion / Parquet / object_store)

| Library | Pin | Enabled (resolved) | Users | API areas used (representative) |
|---|---|---|---|---|
| `arrow` (+`arrow-array/-buffer/-schema/-select/-cast/-ord/-row/-ipc`) | =59.3.0 (family) | Declared: `ipc`, `ffi`, `canonical_extension_types` (+`serde` on schema). Unified in from DataFusion: `csv`, `json`, `prettyprint`, `chrono-tz`. `arrow-ipc` gets `lz4`/`zstd`. | pse-columnar, pse-relations, pse-schema, pse-codegen, plus DataFusion re-exports in pse-engine/runtime | **IPC stream:** `StreamWriter::try_new_with_options(IpcWriteOptions::try_new(align,false,V5))` at `pse-columnar/src/canon/stages.rs:169-170`; `StreamReader` at `pse-schema/src/builder/products.rs:120`. **Row format:** `RowConverter`/`SortField` at `canon/stages.rs:11,59`. **Kernels:** `take_record_batch`, `concat`, `cast_with_options` (3), `lexsort_to_indices`. **C stream FFI:** `ArrowArrayStreamReader`/`FFI_ArrowArrayStream` at `pse-engine/src/arrow_stream.rs:226,279`. **Extension types:** `try_extension_type` at `pse-relations/src/ext/mod.rs:187`; `MemoryExtensionTypeRegistry::new_with_canonical_extension_types` at `pse-engine/src/session/registry.rs:24`. |
| `datafusion` (+ common/expr/functions*/execution/catalog/physical-expr/optimizer) | =55.1.0 | Default features: `sql`, `parquet`, `compression` (bzip2/flate2/liblzma/zstd), `crypto/datetime/encoding/regex/string/unicode/nested_expressions`, `recursive_protection` | pse-engine (primary), pse-relations, pse-rules, pse-runtime, pse-admission, pse-py, pse-testkit | **Planner extension:** `impl QueryPlanner for UnifiedPlanner` (`pse-engine/src/session/planner.rs:46`); 3 `ExtensionPlanner` + 3 `UserDefinedLogicalNodeCore` (`operation.rs:256,313`, `session/contract.rs:87,171`, `session/cache.rs:365,677`). **Custom physical:** 6 `impl ExecutionPlan` (`operation.rs:439`, `operation/ownership.rs:196`, `session/native_function_unit.rs:224`); `PlanProperties` 18, `MetricsSet`/`ExecutionPlanMetricsSet`. **Providers:** 9 `impl TableProvider` (`session/resolution/provider.rs:22`, `session/physical_input.rs:58`, `session/mutation/native.rs:91`); `CatalogProvider`/`CatalogProviderList` (`provider/catalog.rs:102`, `provider/list.rs:105`); `TableFunctionImpl` (`cache_service/inspection.rs:71`); `MemTable` 35. **UDFs:** 15 `ScalarUDFImpl` (`pse-relations/src/identity.rs:55`, `validate/prepared/predicate.rs:49`, `pse-engine/src/session/scalar.rs:125`) and 2 `AggregateUDFImpl` (`session/aggregate.rs:51`). **Logical:** `LogicalPlanBuilder` 168, `TreeNode` 164, `transform_up` 10. Recursive CTE via `LogicalPlan::RecursiveQuery`/`CteWorkTable` (`session/admission.rs:163,560`). `ExprSimplifier` (`pse-relations/src/validate/planner.rs:85`); `create_physical_expr` 6. **DML:** `InsertOp` (`session/commands/insert.rs:14`). **SQL:** `.sql(` 12 (e.g. `pse-relations/src/validate/obligations/consolidation_unit.rs:391`); `datafusion::sql::sqlparser` AST matching (`session/native.rs:67`, `session/commands.rs:24`). **Memory:** custom `impl MemoryPool` ×3 (`pse-engine/src/resources.rs:153`, `session/resources.rs:51`, `pse-columnar/src/owned_buffer.rs:1487`); `TrackConsumersPool<FairSpillPool>` (`resources.rs:205,226`; `pse-runtime/src/env.rs:9`); `GreedyMemoryPool`; `RuntimeEnvBuilder`. **Config:** `SessionStateBuilder` 69, `with_target_partitions` (`session/preparation.rs:1594,1657,1709`). |
| `datafusion-tracing` / `instrumented-object-store` | =55.0.0 | default | pse-engine | `instrument_with_info_spans!`, `instrument_rules_with_info_spans`, `RuleInstrumentationOptions::phase_only` (`session/assurance.rs:103`); `instrument_object_store` (`stores.rs:35,71`). |
| `parquet` | =59.3.0 | Declared `arrow`, `async`, `object_store`. Unified: brotli, flate2, lz4, snap, zstd, simdutf8. | pse-admission, pse-runtime, xtask (opt) | **Product read:** `ParquetRecordBatchReaderBuilder::try_new` (`pse-admission/src/authoring_driver/document/data.rs:18,84`). **Writers:** `ArrowWriter::try_new(.., None)` sites at `data.rs:309`, `owned.rs:721`, `pse-runtime/.../canonical/delta.rs:1338` all sit after `#[cfg(test)]`. No non-test writer found in pse-runtime/engine/columnar. |
| `object_store` | =0.13.2 | `fs`, `tokio` (default) | pse-engine, pse-runtime, pse-testkit | `impl ObjectStore for SharedIo` (`pse-engine/src/stores/admission.rs:93`); testkit `CountingStore`/`FaultStore`; `memory::InMemory`, `path::Path`, `PutOptions`/`GetOptions`/`MultipartUpload`, `Error::Precondition`. |
| `sqlparser` (direct) | =0.62.0 | default + `visitor`, `recursive-protection` (unified) | pse-schema | `GenericDialect` / `Parser` / `Tokenizer` for canonical SQL text (`pse-schema/src/fingerprint.rs:186`). |

**Notably unused Arrow / DataFusion / Parquet features (facts).**
- Arrow offers `async`, `pool`, `pyarrow` and `ipc_compression` (all off; the pin comment bans compression on identity paths). `force_validate` is off by default and on only in correctness tests.
- DataFusion offers `avro`, `serde`, `parquet_encryption`, `backtrace`, `math_expressions`/`array_expressions` (aliases) and `force_hash_collisions`, all off.
- DataFusion APIs with zero uses anywhere in crates: `ListingTable`, `ParquetFormat`/`ParquetSource`/`FileScanConfig`, `DataSink`/`DataSinkExec`, `write_parquet`, `OptimizerRule`/`AnalyzerRule`/`PhysicalOptimizerRule`, `WindowUDFImpl`, `Unparser`, `DiskManager`, `ConfigExtension`/`TableOptions`, `with_information_schema`, `SpawnedTask`.
- Parquet offers `encryption`, `variant_experimental`, `geospatial`, `arrow_canonical_extension_types`, `crc` and `json`, all off.
- Parquet APIs with zero product uses: `AsyncArrowWriter`, `ParquetRecordBatchStream`, `ParquetObjectReader`, `WriterProperties`, statistics/bloom/page-index/`RowSelection` options.
- `object_store` offers `aws`, `azure`, `gcp`, `http` and `cloud`, all off.
- No `deltalake` crate is declared in `[workspace.dependencies]`. AGENTS.md mentions Delta; `pse-runtime/.../canonical/delta.rs` is a local module name.

---

## 3. Preparation (symbolic math, structure, incremental compile)

| Library | Pin | Enabled | Users | API areas used (representative) |
|---|---|---|---|---|
| `symbolica` | =3.0.1 | `integer-gmp`, `float-mpfr` (default off) | pse-math only | **Atoms:** `Atom`, `AtomView`, `AtomCore`, `Symbol`/`SymbolBuilder`/`NamespacedSymbol`/`SymbolAttribute`, `Indeterminate`, `FunctionBuilder` (`pse-math/src/library.rs:8-10`). **Calculus/algebra:** `.derivative(` 28 (`reverse.rs:218`), `.expand(` 9, `.together(` 1, `.replace(` 23, `to_polynomial` 1 (`coefficients.rs:199`), `PolyVariable`. **Evaluation:** `ExpressionEvaluator<Complex<Rational>>` / `<f64>` built from `evaluator_multiple` with an empty `FunctionMap` (`library.rs:257-262`, `native_compiled.rs:108-125`); `export_instructions()` / `ExportedInstructions` / `Instruction` / `Slot` / `OperationCount` (`library.rs:330-380`). **Native code:** `export_cpp` + `CompiledRealEvaluator` + `InlineASM` + `ExportSettings` (`native_compiled.rs:12-18,116`). `jit_compile`/`JITCompilationSettings` appear only in a test probe (`product_realization_probe.rs:75,86`). **Dual numbers:** `Dualizer` / `HyperDual` (`library/admission.rs:4,675,1032`). **Licensing/settings:** `LicenseManager::set_license_key` (`lib.rs:69`), `GLOBAL_SETTINGS` (`lib.rs:76`). Optimizer budget fields (`cores`, `horner_iterations`, `cpe_iterations`) at `library.rs:265`. |
| `gmp-mpfr-sys` | =1.7.1 | `mpfr` (+`mpc` unified) | pse-math | Version probes only (`gmp::version`, `mpfr::get_version`, `lib.rs:219-223`). |
| `root1d` | =0.4.1 | none | pse-math | `root1d::toms748` plus its error variants. |
| `petgraph` | =0.8.3 | `stable_graph`, `graphmap` (+`matrix_graph`, `std` unified) | pse-authoring, codegen, compiler, model, modeling, runtime, structural | `DiGraph`/`NodeIndex`, `algo::{kosaraju_scc, toposort, has_path_connecting}`, `visit::{Dfs, Reversed, NodeFiltered, EdgeRef}`, `graphmap::DiGraphMap`. |
| `rustworkx-core` | =0.18.1 | default | pse-structural | `dag_algo::lexicographical_topological_sort`, `connectivity::find_cycle`. |
| `salsa` | =0.28.4 | `macros`, `inventory`, `salsa_unstable` | pse-compiler | `#[salsa::tracked]` 17, `#[salsa::input]` 12, `#[salsa::interned]` 4, `#[salsa::db]`, `Durability::HIGH`, `Cancelled::catch` 7, `Event`/`EventKind` observation, `Storage`/`Setter`. |
| `pounce-presolve` (also native) | fork rev 7f5e2e9 | default | pse-math, pse-structural, pse-backend-native | Structure: `matching::hopcroft_karp`, `DulmageMendelsohnPartition::from_matching`, `BlockTriangularForm::of_component`, `SquareComponents::of_square_part`, `EqualityIncidence`. FBBT: `fbbt::{forward_pass, forward_result, Interval}`. `PresolveTnlp`/`LinearEqElimTnlp`/`PresolveMap` (`pse-backend-native/src/presolve/pipeline.rs:23`). |
| `rpds` | =1.2.1 | none | pse-modeling, pse-runtime | `RedBlackTreeMapSync` (3). |
| `hashbrown` | =0.17.1 | `equivalent`, `inline-more` | pse-math | `HashMap` + `EntryRef` + `Equivalent` (`library/admission.rs:8`). |
| `typed-index-collections` / `enum-map` | =3.5.0 / =3.1.0 | default | pse-math (+kernels, backend-native for enum-map) | `TiVec`/`TiSlice` (11); `EnumMap::from_fn`, `#[derive(Enum)]`. |

**Notably unused preparation features (facts).**
- Symbolica features off: `python_api`, `serde`, `bincode`, `c_api`, `native_code_generation` (a cargo feature distinct from the `export_cpp`/`compile` path used), `faster_alloc`, `integer-malachite`, `float-astro`, `mathematica_api`, `wasm`.
- Symbolica APIs with zero hits in pse-math/compiler/kernels src: `.series(`, `.nsolve`, `OptimizationSettings` (by that name), `replace_all`, `apart`, external-function registration. JIT appears only in a test.
- No `numerica::` or `graphica::` paths appear anywhere in crates.
- salsa features off: `persistence`, `accumulator`, `detailed-trace`, `shuttle`.
- petgraph features off: `rayon`, `serde-1`, `dot_parser`, `generate`.
- rpds `rayon` is off.

---

## 4. Native solvers

| Library | Pin | Enabled (with `native-solvers`) | Users | API areas used (representative) |
|---|---|---|---|---|
| Ipopt 3.14.20 (C++ via `pse-ipopt-sys`, bindgen'd C header + C++ bridge) | image | `linear_solver ∈ {mumps, spral, pardisomkl}`, `--without-hsl`; sIpopt built (`docker/solvers/README.md`) | pse-backend-native (`ipopt` feature) | The C bindings (`CreateIpoptProblem`, `IpoptSolve`, `SetIntermediateCallback`, `GetIpoptCurrentIterate/Violations`, …; 13 fns in `pse-ipopt-sys/src/bindings.rs`) are referenced only as link probes (`lib.rs:41-44`). Product use goes through `bridge/pse_ipopt_sequence.cpp`, which uses `IpoptApplication`, `OptimizeTNLP` 2, `ReOptimizeTNLP` 4, `TNLP`, `intermediate_callback` with `get_curr_iterate`/`get_curr_violations`, and `finalize_solution`. The bridge carries typed bridge statuses (`pse_ipopt_sequence.h`). Rust callers: `IpoptGetAvailableLinearSolvers`, `GetIpoptVersion`. I found no sIpopt (`SensApplication`) use in the bridge. |
| `pounce-rs` + `pounce-nlp/-common/-feral/-linsol/-qp/-sens-core/-presolve` (paul-heyse fork) | git rev `7f5e2e9…` (=0.12.0) | `pounce-rs`: `qp`, `convex`, default. `pounce-nlp`: `serde`. Others: default. | pse-backend-native (pounce-nlp/presolve also pse-math) | `pounce_rs::{IpoptApplication, TNLP, ApplicationReturnStatus}` (`pounce.rs:67`); `pounce_algorithm::{alg_builder::LinearBackendFactory, second_opinion, …}` (`pounce/retained.rs:12`, `pounce/profiles.rs:9`). `pounce_nlp::tnlp::{Linearity, InfeasibilityProof::{IntervalArithmetic, BoundPropagation}, SparsityRequest, StartingPoint}`, `expression_provider::{FbbtTape, FbbtOp}`, `derivative_test::DerivativeTestReport`, `constant_derivatives::DerivativeProofs`, `start_conditioner::jitter_start`. `pounce_common::observed::{Scope, Observer, Abort, StorageScope, …}` observation hooks (`pounce/observed.rs:8`). `pounce_sens_core::{SensApplication, SensOptions, IndexSchurData, SensBacksolver}` (`kkt/sensitivity.rs:44`, `kkt.rs:57`). `pounce_qp::{ParametricActiveSetSolver, QpSolver}` (`kkt/path/qp.rs:16`). `pounce_feral::{FeralConfig, FeralSolverInterface}`. |
| `feral` (inside pounce fork) | =0.18.0 (same rev) | default | pse-backend-native | `Solver::new`, `CscMatrix::from_triplets`, `SparseLu::factor_markowitz`, `scaling::ScalingStrategy::Mc64Symmetric`, `symbolic::OrderingMethod`, `FactorStatus::{Success, WrongInertia, Singular, FatalError}`, plus typed `FeralError::*` variants. |
| `highs` / `highs-sys` | =2.4.0 / =1.15.0 | `highs-sys`: `build`, `highs_release`, default | pse-backend-native (`highs`, `uno` features) | The `highs::Model` / `ColProblem` safe type is the owner (`highs.rs:86,372-375`). About 60 raw `ffi::Highs_*` calls run on its pointer: `passLp`/`passMip`/`passHessian`, `run`, `presolve`/`postsolve`/`getPresolvedLp`, `getBasis`/`setBasis`, `getBasisInverseRow`, `getRanging`, `getIis`, `feasibilityRelaxation`, `getPrimalRay`/`getDualRay`, `setCallback`/`startCallback`, `passLinearObjectives` (multi-objective), `setSparseSolution`, `resetGlobalScheduler`. |
| `clarabel` | =0.11.1 | `serde`, `sdp`, `blas-src`, `lapack-src`, `pardiso-mkl` | pse-backend-native | `DefaultSolver`/`DefaultSettings`/`DefaultInfo`/`IPSolver`, `algebra::CscMatrix` (`conic.rs:17-19`). Cones: Zero, Nonnegative, SecondOrder, Exponential, Power, GenPower, PSDTriangle. All 15 `SolverStatus` variants are mapped. Reusable-data updates are noted at `execution/clarabel.rs:43`. |
| `scip-sys` (SCIP 10.0.2 image) | =0.1.28 | `bindgen`, default | pse-backend-native (`scip`) | About 120 distinct `SCIP*` calls. Model building: `SCIPcreateExprVar`/`Sum`, `SCIPcreateConsBasicNonlinear`, `SCIPcreateConsBasicIndicator`, `SCIPsimplifyExpr`. Solving: `SCIPsolve`, `SCIPsolveConcurrent`, `SCIPinterruptSolve`. Exact mode: `SCIPisExact`, `SCIPrational*`, `SCIPgetSolOrigObjExact`, `…ExactLinear`. IIS: `SCIPiisGetSubscip`, `SCIPiisIsSubscipInfeasible/Irreducible`. Event handlers: `SCIPincludeEventhdlrBasic`, `SCIPcatchEvent`. Statistics: presol/prop/sepa/nlpi counters. Parameters: get/set. `SCIPreadProb`. |
| `sundials-sys` (vendored) | =0.6.2 | `kinsol`, `idas`, `klu`, `build_libraries`, `static_libraries` | pse-backend-native (`kinsol`, `idas`) | **KINSOL** (28 fns): `KINInit`, `KINSol`, `KINSetLinearSolver`, `KINSetJacFn`, `KINSetJacTimesVecFn`, `KINSetPreconditioner`, Anderson acceleration (`KINSetMAA`, `SetDampingAA`, `SetOrthAA`, `SetDelayAA`), `KINSetEta*`, `KINSetConstraints`. **IDAS** (about 90 fns): `IDAInit`/`ReInit`, `IDACalcIC`, roots (`IDARootInit`), forward sensitivities (`IDASensInit`, `IDAGetSensDky`), quadratures and quadrature sensitivities, adjoint (`IDAAdjInit`, `IDACreateB`, `IDASolveB`, `IDACalcICB`), `IDABBDPrecInit`. Calls like `IDASetAcceptedStepBudget`, `IDASetReplayStepLimit` and `IDAGetCheckpointUsage` are named by the code; I did not check whether upstream SUNDIALS has them. **SUN:** `SUNLinSol_KLU` (+`KLUSetObserver`, `KLUBeginApplication`, `KLUGetWork` look non-upstream; not checked), `SUNLinSol_Dense`, `SUNLinSol_SPGMR`/`SPFGMR`, `SUNSparseMatrix`, `N_VNew_Serial`. |
| `suitesparse_sys` (vendored) | =0.1.4 | `klu`, `amd`, `colamd`, `btf`, `static_libraries` | pse-backend-native | Direct `klu_l_{analyze, factor, refactor, solve, rcond, condest, free_*}` (`dynamics/linear.rs:24`), plus version constants. |
| `diffsol` / `diffsol-la` / `diffsol-nl` (vendored) | =0.16.2 / =0.1.1 / =0.1.1 | `faer` (`-la`/`-nl` also get `nalgebra` unified) | pse-backend-native | `OdeBuilder`, `OdeSolverMethod`, the `OdeEquations` op traits (`NonLinearOpJacobian/Sens/SensAdjoint`, `ConstantOpSens*`, `LinearOpTranspose`), `FaerSparseMat`/`FaerVec`/`FaerContext` (`dynamics/integrator.rs:8-13`). Methods: Bdf 4, Esdirk34 3, TrBdf2 3, Tsit45 4, Sdirk 1, ExplicitRk 1. Adjoint: `AdjointOdeSolverMethod` + `Checkpointing` + `HermiteInterpolator` (`dynamics/integrator/adjoint.rs:21-24`). `interpolate`/`interpolate_sens`, `set_stop_time`. A custom `diffsol_la::LinearSolver` impl (`dynamics/linear.rs:17`); `NewtonNonlinearSolver`, `NoLineSearch`. |
| `faer` | =0.24.4 | `std`, `sparse-linalg` (+`linalg`, `rayon`, `npy`, `rand` unified) | pse-backend-native, pse-math, pse-runtime | Sparse CSC assembly: `Triplet::new` 94, `SparseColMat::try_new_from_triplets` 59, `SymbolicSparseColMat(Ref)`. `sparse_dense_matmul` 12. Dense `Mat`/`MatRef`/`MatMut::from_column_major_slice*`, `Par::Seq` 27, `linalg::svd`, `temp_mat_scratch`. |
| `finitediff` | =0.2.0 | default (`ndarray`) | pse-backend-native | `vec::{forward,central}_jacobian_vec_prod`. |
| Uno (C++ via `pse-uno-sys`) / PETSc (via `pse-petsc-sys`) | image, patched | n/a | pse-backend-native (`uno`, `petsc`) | Uno: `pse_uno_{create, solve, get_result, get_vectors, get_*_option, destroy}`. PETSc: `pse_petsc_*` C shims over SNES (incl. TR, NASM), TS (pseudo-transient), Vec/Mat/Scatter, options and log events. |

**Notably unused native-solver features (facts).**
- pounce-rs offers `sensitivity`, `ma57` and `full`, all off; `pounce-sens-core` is used directly instead. Of the fork's crates, these are not depended on: `pounce-convex`, `pounce-l1penalty` (though `pounce::Method::L1ExactPenalty` is used), `pounce-restoration`, `pounce-sensitivity`, `pounce-nl`, `pounce-observability`, `pounce-solve-report`, `pounce-cinterface`.
- CHANGELOG 0.12.0 lists `TnlpPresolveSession`, `ConvexPresolveSession`, `wrap_with_presolve` and `run_second_opinion_ladder`. Whether they are used: `second_opinion` is imported (`pounce/profiles.rs:9`); the session types were not searched for.
- feral's `lu-ft-invariant-check` is off.
- Clarabel options off: `faer-sparse` (the pin comment says it would bring a second faer), `sdp-mkl`/`-openblas`/`-netlib`/`-accelerate`/`-r`, `pardiso-panua`, `python`, `julia`, `buildinfo`.
- scip-sys `bundled`/`from-source` are deliberately refused.
- sundials-sys components off: `cvode`, `cvodes`, `arkode`, `ida` (non-S), `nvecopenmp`, `nvecpthreads`.
- suitesparse components off: `cholmod`, `umfpack`, `spqr`, `ldl`, `camd`, `ccolamd`, `cxsparse`, `paru`, `spex`, `mongoose`, `graphblas`/`lagraph`, `rbio`, `openmp`.
- diffsol features off: `diffsl*` (the DiffSL DSL with LLVM/Cranelift JIT), `sundials`, `suitesparse`, `nalgebra`, `cuda`, `autodiff`.
- faer `nightly`/`unstable`/`perf-warn` are off.
- HiGHS-sys `discover`/`libz`/`ninja` are off.
- Ipopt's sIpopt library is built into the image but no use was found in the bridge.

---

## 5. Boundary and generation (Python, serialization, schemas)

| Library | Pin | Enabled | Users | API areas used (representative) |
|---|---|---|---|---|
| `pyo3` | =0.29.2 | `extension-module`, `abi3-py311`, `experimental-inspect` (+`macros`, `chrono`, `indexmap` unified) | pse-py | `#[pyclass]`/`#[pyfunction]`/`#[pymethods]` 129 attributes, 45 `frozen` pyclasses, `py.detach` 68. |
| `pyo3-arrow` | =0.19.0 | `buffer_protocol`, default | pse-py | `PyRecordBatch::new` only (8; `crates/pse-py/src/workflow.rs:1953-1967`, `diagnostics.rs:327-354`). |
| `pyo3-async-runtimes` | =0.29.0 | `tokio-runtime` | pse-py | `tokio::{init_with_runtime, get_runtime, get_current_locals, future_into_py_with_locals}`. |
| `pyo3-introspection` | =0.29.2 | — | xtask | `.pyi` stub generation. |
| `serde` / `serde_json` | =1.0.229 / =1.0.151 | serde `derive`, `rc`; serde_json `float_roundtrip` (+`preserve_order`, `unbounded_depth` unified) | nearly all crates | Manifest and document JSON. |
| `schemars` | =1.2.2 | `derive`, default | 16 crates | `JsonSchema` derive (912), custom `json_schema` impls (201), `schema_for` (10). The generator publishes schemas and msgspec types. |
| `nutype` | =0.8.0 | `serde`, `derive_unchecked` | pse-model | `#[nutype]` validated settings domains. |
| `serde-saphyr` | =1.3.0 | default | pse-admission, pse-runtime | `from_str_with_options`, `budget`, `Spanned`, `to_string`. |
| `toml` | =1.1.6 | `parse`, `serde`, `display` | pse-admission, pse-runtime, xtask | `toml::Value`, `Spanned`, `from_str`. |
| `winnow` | =1.0.4 | `std`, `ascii`, `parser` | pse-authoring | `LocatingSlice`, `token::{one_of, take_while, any}`, `ascii::{float, multispace0}`, `ContextError`. |
| `blake3` | =1.8.7 | default | pse-ids only | `Hasher::new_derive_key` (7), `hash`, `derive_key`. |
| `uuid` | =1.27.0 | `v7` (+`v4`, `serde` unified) | pse-operations (16 files), authoring, engine, codegen | `Uuid::new_v4` 57, `parse_str` 10, `now_v7` 3. |
| `strum`, `hex`, `bytes`, `chrono` (`std`) | pinned | — | various | Enum derives, hex encoding, `Bytes`, timestamps. |
| **pyarrow** | ==25.0.1 | — | `python/pse/{_transfer, _inspection, _array, _runs, diagnostics, conformance, contracts/extension_types}.py` | `pa.ExtensionType` subclasses (`contracts/extension_types.py:39,111-150`). Schema/field/type inspection (`pa.BaseExtensionType` in `_transfer.py:42-213`). `__arrow_c_stream__` passthrough (`_inspection.py:35`). `pa.ipc.new_stream` (`diagnostics.py:1018`). `to_numpy(zero_copy_only=…)` (`_array.py:75`). |
| **attrs** | ==26.1.0 | — | 18 product modules incl. `contracts/*.py`, `_workflow.py`, `codec/__init__.py`, `governance.py` | `attrs.frozen` 532, `attrs.field` 3164, `validators.instance_of` 2836, `optional` 914, `deep_iterable` 447, `min_len`/`max_len`/`and_`. |
| **cattrs** | ==26.2.0 | — | `codec/__init__.py`, `codec/_generated.py` | `cattrs.Converter()`, `register_structure_hook_factory`, `gen.make_dict_structure_fn(..., _cattrs_forbid_extra_keys=True)`, `make_dict_unstructure_fn`, `override` (`codec/__init__.py:22-61`). |
| **msgspec** | ==0.21.1 | — | `codec/*`, `contracts/documents/__init__.py`, `contracts/extension_types.py`, `diagnostics.py`, `_diagnostic_query.py`, `_diagnostic_launch.py`, `conformance.py`, `governance.py` | `msgspec.Struct` 707 (`forbid_unknown_fields=True` 709 across python/pse, mostly `contracts/documents/__init__.py`), `msgspec.Meta` 839, `field` 91, `convert` 35, `json.encode`/`json.decode`, `structs.replace`, `inspect.type_info`/`StructType`/`UnionType`, `Raw`. |

**Notably unused boundary features (facts).**
- pyo3 features off: `experimental-async`, `multiple-pymethods`, `num-bigint`, `bigdecimal`, `py-clone`, `abi3t`, `generate-import-lib`, `auto-initialize`.
- pyo3-async-runtimes `unstable-streams` and `attributes` are off.
- pyo3-arrow: only `PyRecordBatch` is used. `PyTable`, `PyArray` and `PyRecordBatchReader` (stream export) have zero uses; the Python side passes the C stream through `__arrow_c_stream__`.
- `msgspec.msgpack` has zero uses; `msgspec.json.Decoder`/`Encoder` were not found by my pattern.
- schemars `preserve_order`/`raw_value` are off.
- nutype `regex`/`new_unchecked`/`valuable` are off.
- blake3 `rayon`/`mmap`/`traits-preview` are off.
- serde_json `arbitrary_precision`/`raw_value` are off.

---

## 6. Runtime infrastructure and fitting

| Library | Pin | Enabled | Users | API areas used (representative) |
|---|---|---|---|---|
| `tokio` | =1.53.2 | Declared `rt-multi-thread`, `sync`, `macros`, `time`, `process`, `io-util`. Unified: `fs`, `net`, `signal`, `io-std`. | most runtime crates | `time::timeout` 172, `sync::{oneshot 101, Notify 79, mpsc 41, Mutex 26, Semaphore 16, Barrier 12, watch 10}`, `task::{yield_now 48, JoinSet 11}`, `runtime::{Handle, Builder}`, `process::Command` (`worker.rs:1642`), `net::TcpListener` 9. |
| `tokio-util` | =0.7.19 | `rt` | pse-columnar, pse-operations | `CancellationToken`, `TaskTracker`/`TaskTrackerToken`. |
| `rustix` | =1.1.5 | `fs`, `time`, `thread`, `process`, `event` | pse-runtime (8 files), diagnostics, math | `process::pidfd_open` + `event::poll`, `process::waitid(NOWAIT)`, `Signal::KILL`, `clock_gettime(Monotonic)`. |
| `tracing` / `tracing-subscriber` / `tracing-perfetto-sdk-schema` + `prost` | pinned | subscriber `env-filter`, `fmt`, `registry` | pse-runtime (`capture/timeline.rs:11,22`), engine, testkit | Perfetto protobuf timeline via `prost::Message`. |
| `rayon` | =1.12.0 | default | pse-backend-native (pounce), pse-math, pse-runtime | `ThreadPoolBuilder::new` 4, `current_thread_index`, `current_num_threads`. |
| `quick_cache` | =0.6.24 | default (`ahash`, `parking_lot`) | pse-runtime | `unsync::Cache` + `Weighter` + `OptionsBuilder` (`math/retention/weighted.rs:5`). |
| `once_cell`, `rustc-hash`, `pin-project-lite`, `event-listener`, `url`, `libc` | pinned | — | various | Utility. |
| `egobox-ego` / `-moe` / `-gp` / (`-doe` dev) | =0.38.1 / =0.35.5 / =0.36.4 / =0.35.1 | ego `persistent`. Unified on moe/gp: `serializable`, `serde`, `typetag`, `bincode`. | pse-runtime | `EgorConfig`, `EgorSolver`, `EgorState`, `InfillOptimizer`, `Constraints`/`Cstr`/`CstrFn`, `RuntimeFlags`, `to_xtypes`, `GpMixtureParams`, `ThetaTuning` (`math/surrogate/library.rs:5-13`). |
| `argmin` | =0.11.0 | default off (`serde1`, `rand`, `ctrlc` unified via egobox) | pse-runtime | `core::{CostFunction, Problem, Solver, State, TerminationStatus, Error}` (`math/surrogate/library.rs:4`). |
| `levenberg-marquardt` | git rust-cv rev `e67066c` (=0.15.0) | default off | pse-runtime | `LeastSquaresProblem`, `LevenbergMarquardt`, `TerminationReason` (`workflow/fitting/oracle/residual.rs:5`, `lm_inquiry.rs:5`). |
| `nalgebra` | =0.35.0 | `std` | pse-runtime, xtask (opt) | `DMatrix`, `DVector`, `Dyn`, `storage::Owned` (LM residual types). |
| `ndarray` | =0.16.1 | default | pse-runtime | `Array1`/`Array2`, `array!` (egobox interface). |
| `statrs` | =0.18.0 | Declared default off. `cargo tree` shows `default`, `nalgebra`, `rand` enabled anyway (unified from another dependent). | pse-runtime | `distribution::{Normal, ChiSquared, ContinuousCDF}` (`workflow/fitting/covariance.rs:36`, `profile.rs:37`). |
| `feos` / `feos-core` / `quantity` | =0.10.1 / =0.10.1 / =0.14.0 | `feos`: `pcsaft`, `association`; `quantity`: `nalgebra`, `num-dual` | xtask only (`thermodynamic-oracles`) | `xtask/src/feos_reference.rs`: "Independent oracle generation only. No product provider imports FeOS." |

**Notably unused runtime and fitting features (facts).**
- tokio features off: `io-uring`, `taskdump`, `schedule-latency`, `test-util` (`full` not used).
- egobox `blas`/`basin` are off.
- argmin `full` (solver catalogue features) is off.
- levenberg-marquardt `minpack-compat` is off.
- ndarray `rayon`/`blas`/`serde` are off.
- nalgebra `sparse`/`serde-serialize` are off.
- quick_cache `stats`/`sharded-lock` are off.
- FeOS models off: `dft`, `epcsaft`, `gc_pcsaft`, `saftvrmie`, `saftvrqmie`, `pets`, `uvtheory`, `multiparameter`, `rayon`.

---

## 7. Vendored and patched libraries (one line each, from the PATCH/notes files)

| Item | Change |
|---|---|
| `vendor/surrealdb` + `vendor/surrealdb-engine-api` (3.3.0, upstream rev `238bfeb`) | Adds request-local deadline/cancellation `RequestContext`. `Config::bounded_requests()` gives 32 application + 2 control admissions, 34 queued routes, one session and 64 replay entries. Adds `selection_checkpoint` replay compaction, physical socket teardown on `disconnect()` and finite 5 s version/restore lifecycles. Adds `ResponseLimits` FlatBuffers ingress limits (node/depth/string/blob), redacted logs and a lifetime owner (`vendor/surrealdb/PATCH.md`, `vendor/patches.md`). |
| `vendor/sundials-sys` 0.6.2 | Build-freshness only: provider env observation, KLU linkage to the sibling `suitesparse-sys`, `SUN_JOB_*` reproducibility watches, bindgen `-fno-builtin`, byte-compare output writes. "All upstream feature selections and numerical Rust/C sources are preserved" (`vendor/native-supplier-notes.md`). This contradicts the vendor-unmodified reading if `IDASetAcceptedStepBudget`, `IDASetReplayStepLimit`, `SUNLinSol_KLUSetObserver` and the other names flagged in §4 are not upstream SUNDIALS; that is unresolved. |
| `vendor/suitesparse-sys` 0.1.4 | Same build-freshness corrections; one `links = "suitesparse"` source; two CRLF files kept. |
| `vendor/diffsol` 0.16.2, `diffsol-la` 0.1.1, `diffsol-nl` 0.1.1 | Manifests repointed to the sibling vendored crates (single SuiteSparse link source). The `sundials` build branch observes its inputs. Numerical sources unchanged. |
| `vendor/paste` 1.0.15 | `Segment::Env` refuses before reading the environment (no ambient env in macro expansion). |
| `vendor/pest_derive` 2.9.2 | Refuses `grammar = "file"`; requires literal `grammar_inline`. |
| `vendor/py_literal` 0.4.0 | Grammar file inlined as the sole literal grammar (hash-pinned); debug macro temporary renamed. |
| `vendor/feral` | No such directory in pse-arrow. FERAL 0.18.0 lives inside the pounce fork at `vendor/feral`, from published MIT FERAL, upstream `32c936b` (`source-extension.md`). |
| pounce fork (`github.com/paul-heyse/pounce` rev `7f5e2e9`, from upstream `a31dab9`) | "Observed execution source extension": a synchronous observer bound to the post-classification TNLP layout, Begin/End factor events, sticky observer abort that stops numerical fallback, `StorageScope::{Application, Linear}` reservations. Adds a bounded tiny-dense FERAL profile (`FeralConfig::bounded_dense_max_dimension = 1..=7`, refuses unsupported geometry) with source-derived allocation bounds (`source-extension.md`). |
| `docker/solvers/petsc-pse.patch` | `PetscStrlcat`: `PetscAssert` becomes `PetscCheck`, adds an existing-length < capacity check, and fixes the `strncat` off-by-one (`n-len-1`). |
| `docker/solvers/uno-pse.patch` (374 lines) | Changes Uno's HiGHS QP subproblem (`HiGHSSolver.cpp/.hpp`, `HiGHSQuadraticProgram.cpp`) and `NoRelaxation.cpp`. It removes the hard "no explicit Hessian, cannot be solved with HiGHS" refusal and reworks Hessian sparsity/value evaluation, option output and `solve_subproblem` signatures. Read from the diff hunks only; the image README does not describe it. |
| `levenberg-marquardt` | Git rev of upstream `rust-cv`, no local patch file found; no reason recorded beside the pin. |
| Image components (not crates) | Ipopt 3.14.20 (MUMPS/SPRAL/Pardiso-MKL, no HSL, sIpopt built), SCIP 10.0.2 + SoPlex 8.0.2 + PaPILO 3.0.0 (exact mode, `IPOPT=ON`), oneMKL 2026.1 with `MKL_CBWR=COMPATIBLE` (`docker/solvers/README.md`). |

---

## Unresolved edges and limits

1. **SUNDIALS names that may not be upstream.** `IDASetAcceptedStepBudget`, `IDASetReplayStepLimit`, `IDAGetCheckpointUsage`, `IDAGetAcceptedBudgetSteps`, `IDAGetReplayNumSteps`, `SUNLinSol_KLUSetObserver`, `SUNLinSol_KLUBeginApplication` and `SUNLinSol_KLUGetWork` appear as calls. The notes say numerical sources are unmodified, so these may be pse-side shims; I did not check. Probe: `rg -n 'IDASetAcceptedStepBudget|KLUSetObserver' vendor/sundials-sys crates/pse-backend-native` to locate their definitions.
2. **pounce 0.12 session APIs.** I did not search for `TnlpPresolveSession`, `ConvexPresolveSession` or `wrap_with_presolve`.
3. **Count method.** Counts are line-based `grep`/`rg` occurrences and include inline `#[cfg(test)]` modules, except the Parquet writers, which I checked by hand. My first DataFusion count pass mis-passed its globs and was redone; the table uses the corrected counts.
4. **No pinned upstream docs consulted.** Context7 was not consulted. This census reads only installed manifests and repository source; "offered features" is the `[features]` table of each pinned source.