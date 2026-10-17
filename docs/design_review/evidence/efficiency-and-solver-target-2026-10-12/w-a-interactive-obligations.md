> Supporting evidence for [the efficiency and solver target review](https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/design_review/reviews/design_review_efficiency-and-solver-target_2026-10-12.md). Inquiry W-A (code-mapper): interactive obligations ledger. Baseline: `main` at `6c368c88a`. This contribution is preserved as returned; its local labels are not finding identifiers, and the principal review owns the judgment.

# W-A: interactive journeys — obligations ledger

Baseline: `main` at `6c368c88a`, clean tree, 2026-10-12. Static source reading only; no
builds, tests, probes or services. Paths are relative to `/home/paul/pse-arrow`.
Evidence labels: rows citing a code path are *Implemented* (the path exists) and
*Interface-checked* (I read the source); nothing here is *Tested* or *Measured*.
Interpretation is marked **[I]**; everything else is observed fact.

Abbreviations used in the tables:

- **PY→RS**: Python → PyO3 call. Every `pse-py` entry that does async work goes through
  `crates/pse-py/src/workflow.rs::blocking_on_with_recovery`: `py.detach` (GIL released),
  `executor.block_on(tokio::time::timeout(100ms, fut))` in a loop, `py.check_signals()`
  between polls, and a cancel closure on signal. The executor is one process-static
  multi-thread tokio runtime (`crates/pse-py/src/inspection/runtime.rs::EXECUTOR`,
  `worker_threads = budget.threads.pool_threads`).
- **WS**: WebSocket RPC to the supervised SurrealDB 3.3.0 server process (loopback,
  `pse_operations::canonical::CanonicalStore`, `surrealdb::engine::remote::ws::Ws`;
  `REQUEST_TIMEOUT` 30 s). **WS-W** marks a write or guarded transaction. **WS-R** marks a read.
- **PROT**: `CanonicalStore::protect(revision, 3600s)` followed later by `release`/`finish`.
  The grant is a WS-W, issued from a `tokio::spawn`ed owner task
  (`crates/pse-operations/src/canonical.rs::protect`). Every protected read therefore costs
  at least two writes.
- **ACT**: `canonical_store().admit_activity()?.scope(..)`. This counts in-flight activity
  on the canonical connection for drain/close (`pse_operations::canonical::lifetime`). Nearly
  every `pse-runtime` workflow method wraps itself in one, so the scopes nest several deep.
- **RES**: `MathService::reserve(name, bytes)` / `MemoryConsumer::try_grow`. These are charges
  against the single DataFusion `FairSpillPool` (`crates/pse-runtime/src/env.rs`).
- **JOB**: `MathService::job`/`submit_with`. Each spawns a fresh OS thread named `pse-math`
  with the policy stack and joins it via `spawn_blocking`
  (`crates/pse-runtime/src/math/jobs.rs` ~L1077). It is admitted by a CPU semaphore and a
  population lane.
- **SESSION**: `NativeService::open_session[_queued]` spawns one dedicated `pse-math` thread
  for a staged sequence, which owns retained native state. Closing joins it
  (`crates/pse-runtime/src/math/staged.rs::session_on_reserved`).
- **VAL**: registry field validation. `pse_relations::validate::ValidationContext` uses the
  `pse_engine::validation::NativeValidation(SessionState)` planner. Declared local value
  domains are lowered to DataFusion `Expr`, then to `PhysicalExpr`, and evaluated over
  Arrow batches (`crates/pse-engine/src/validation.rs`,
  `crates/pse-relations/src/validate/predicates.rs`). Triggered by every generated
  `Builder::with_registry(..).finish()` and every `FieldCheckedBatch::admit*`.

---

## Journey 1 — first solve from authored definitions, through publication

Python: `rt = pse.Runtime(settings, substrate=...)`, then `phys = rt.physical_from_documents(docs)`,
then `pkg = rt.modeling_from_documents([bundle], phys)`, then `res = pkg.solve_case(case, settings)`.
The alternative split route is `pkg.prepare_solve(..)` followed by `rt.start([op])`.

| # | Stage | Owner | Scientific work | Obligations discharged (mechanism) | Crossings, txn lifetimes | Repeats per edit/solve; grows with | Library vs bespoke |
|---|---|---|---|---|---|---|---|
| 1.1 | Process runtime acquisition | `pse-py/src/inspection/runtime.rs::acquire` → `pse_runtime::SharedRuntime::build` (`env.rs`) | none | capacity: one process budget; refuses a differing budget (`PROCESS_RUNTIME` mutex). Memory pool = DataFusion `TrackConsumersPool<FairSpillPool>` + `PeakRecordingPool`; `compiler_cpu` tokio `Semaphore(min(cores, pool_threads))`; `MathService`; engine `SessionFactory` + `BoundModel(registry)` | PY→RS; builds the tokio executor once (`OnceLock`); `pyo3_async_runtimes::tokio::init_with_runtime` | once per process | DataFusion RuntimeEnv/memory pool/DiskManager; tokio; bespoke budget checks |
| 1.2 | Runtime attach and startup attestation | `pse-py/src/workflow.rs::NativeRuntime::new` | none | assurance: checks the loaded module path equals the imported `pse._native` (`pse_buildinfo::loaded_module_path`). Outer checkout/build observation under a startup deadline (`MathService::observe_deployment_runtime`, unsafe). Local replay admission (`observe_local_runtime`), which is **refused under the abi3 boundary**: `local_python_configuration` returns `Err`, so `producer = None` unless a receipt path is given | PY→RS; JOB thread for observation; signal-cancellable | once per `Runtime` | bespoke (`pse-runtime/src/math/portable.rs`) |
| 1.3 | Canonical store connect/open | `pse_operations::canonical::CanonicalStore::connect`/`open`; `workflow::Runtime::from_shared` (`workflow/mod.rs`) | none | durability class: **durable by default** (`Durability::Durable(Operations{lease 30s, heartbeat 10s})`); `ephemeral=True` only switches result retention. Interpretation/schema verification at open. Deployment-identity checks (`check_deployment`: same pool, spill, cache manager, object store registry, CPU admission). Registers `PhysicalCache` with the cache service | WS (authenticate, USE ns/db, interpretation probe) | once per `Runtime` | SurrealDB SDK (vendored request-lifecycle patch) + bespoke canonical lifetime |
| 1.4 | Physical admission | `NativeRuntime::physical_from_documents` → `pse_admission::authoring_driver::document::load_package_documents_owned` → `workflow::PhysicalContext::from_documents` → `pse_admission::physical::PhysicalDocuments::from_documents` → `PhysicalInventory::load` | parse physical documents; quantities/units/kinds inventory; physical preconditions | validation: VAL on parsed relation batches; relational/key checks through `EngineFactory::candidate_checked` (checked workspace). Inventory = DataFusion `LogicalPlanBuilder::scan(...).sort(pk)` per input relation + `declare_relation_output`, `prepare_many` + `execute_group`, `plans::units`, `plans::context` (DataFusion distinct queries). Capacity: one-partition inventory sessions; RES for inventory | PY→RS; DataFusion execution on the tokio executor; **no store I/O** | once per physical document set; grows with physical package size | **DataFusion** (planning and execution), registry-generated relation builders; bespoke inventory/precondition logic (`pse_relations::physical`) |
| 1.5 | Modeling document load | `pse-py/src/workflow/modeling.rs::from_documents` → `load_package_documents_owned` per bundle → `OwnedDocumentSet::try_from_bundles` | parse authored text (`pse_authoring` parser) into declaration rows and per-section relation batches | validation: VAL per relation batch at load; capacity: `ParseBudget::default()`, allocation leases | PY→RS (GIL released); synchronous parse inside the async block on the executor thread | per load; grows with **total document text** | bespoke parser (`pse-authoring`); Arrow builders |
| 1.6 | Source admission checks | `workflow::Runtime::modeling_from_documents` (`workflow/modeling.rs`) → `OwnedDocumentSet::validate_context` → `document_inputs` → `p1::source_context_batches`, `validate_import_versions`, `physical_scope` | package/manifest dependency graph, import-version admissibility | validation: **VAL re-run** (`validate_context` re-admits every batch against the same registry/context); graph limits (`GraphLimits`) | ACT | per load; grows with package and document count | bespoke + registry builders |
| 1.7 | Canonical source persistence | `workflow/modeling/canonical.rs::persist` → `CanonicalStore::edit(problem, parent=None, operation, edits)` | parsed-field memo per declaration (`parsed_fields`, `modeling_cache`); source revision identity hash (`math::modeling::source_revision`); for data closures, a full `MathService::modeling_revision` (compiler) to compute record suppliers | durability: one immutable revision (head-CAS per problem; idempotency by minted `operation` id; `SourceEditUnsettled` recovery). Validation: VAL by building the `modeling_declarations` batch, **then dropping it** (validation only). Reuse: one object edit per declaration, plus package, document, data, fit, physical-scope and ingress-frame objects with references. RES `modeling:canonical-ingress` | WS-W: one guarded edit transaction (payload bounded by `PAYLOAD_BYTES` 3 MiB batches) | per package admission; grows with **number of declarations and documents** | SurrealDB (records, references, transaction); bespoke object/membership model (`pse-operations/src/canonical_staging.rs`, `canonical_selection.rs`) |
| 1.8 | Declared execution: root read | `ModelingPackage::declared_execution` → `declared_execution_with_demand` → `canonical.rs::declaration(root)` | read the root declaration row; resolve authored route, procedure, limits and solver policy (`declared_policy`, `declared_limits`, `declared_solver`) | coordination: separate PROT + object read + release just for this row | WS-W (protect), WS-R (object), WS-W (release) | **every solve**, including value edits | bespoke |
| 1.9 | Selected admission (structural resolution plus compiler) | `ModelingPackage::prepare` → `engagement.rs::engage` → `preparation.rs::checked_selection_scoped` → `canonical.rs::checked_selection` | lexical/selected-closure hydration from the store (`select_into`); Salsa compiler admission of the selected rows (`MathService::compiler_lane` → `modeling_revision_at` on a JOB thread) | coordination: PROT on the revision (held through preparation; renewed by engagement); selected-qualification turn (`selected_qualification_turn`, single-writer per request); single-flight `selected_flights.load_owned`. Reuse: `modeling_cache.selected(request)`, qualified against the store by `qualify_selection_window` (dependency recheck). Assurance: per-call deadline (`UNSCOPED_PREPARATION_LIMIT` 600 s), cancellation | WS-W (protect), WS-R (paged inventory/objects), JOB thread for the compiler; Salsa DB behind `Mutex` (single-writer lane per lineage) | first solve: full hydration + compile; later: cache candidate + store recheck; grows with **selected closure size** | **Salsa** (`pse-compiler/src/workspace/modeling/granular.rs`, 12 inputs/tracked); bespoke selection, flight and qualification (`pse-columnar::flight`) |
| 1.10 | Specialization basis and body descriptions | `preparation.rs::prepare_selected_basis` → `MathService::plan_frontier_keyed`/`complete_frontier_keyed`; `establish_preparation` → `portable::describe_body`; `portable::publish_description` | specialization (instances, bindings, demand), body planning, symbolic bodies (Symbolica via the compiler workspace) | reuse: `modeling_cache.basis(BasisKey{admission, root, instance, bindings, limits})`; body cache `modeling_cache.body(identity)`; store-received bodies are skipped under Python (`producer None`). Durability: **every new body description is published to the store** (`publish_description_until`, producer key `pse.unqualified-producer.v1:<build>` when unqualified). Assurance: `acknowledge_description_until` on rebinding | WS-R (presence), WS-W per unsettled description publication; JOB threads | first solve; on later solves only for new or changed bodies; grows with **distinct body count** | Salsa + Symbolica (via `pse-compiler`/`pse-math`); bespoke portable descriptions |
| 1.11 | Engagement close | `PreparationEngagement::close` (`engagement.rs`) | none | coordination/drain: waits for active admissions (`AtomicUsize` + `Notify`), then `release` of the protection | WS-W (release) | every solve | bespoke |
| 1.12 | Case resolution and bound view | `cases.rs::resolve_prepared_case` → `resolve_starts`, `inner_registrations` (implicit providers), `variable_states`, `views.rs::bound_case` | starts, fixed/free states, implicit-provider registration, bound structure (`PreparedModeling::bound_structure`), solver view | reuse: `modeling_cache.solver(view_key)`; hit, then `MathService::rebind(view, values)`; miss, then `prepare_modeling_view` (JOB). RES charges | in-process; JOB threads | every solve; view rebuilt only on a structure/order/profile key change | pse-compiler (Salsa-adjacent) + bespoke; Symbolica evaluators inside views **[I: per R2]** |
| 1.13 | Solve preparation | `cases.rs::finish_case` → `math/solves.rs::prepare_solve` (`policy_resolution_owner().resolve`, `prepare_resolved`); optional order upgrade, accuracy programs, causal supplier | numerical policy resolution (scaling/tolerances), method/backend routing, provider binding, optional sensitivity and selected-output programs | validation: typed admission errors with attribution (`diagnostics::attribute`); cancellation via `tokio::select!` | in-process | every solve; policy resolution cached by `modeling_cache.numerics` **[I: key not traced]** | bespoke routing; libraries are selected here (R4) |
| 1.14 | Run launch and durable registration | `run.rs::ModelingSolvePreparation::start` → `Runtime::launch` → `durable.rs::DurableAttempt::register` → `register_selection` (`put_physical_rows`, `begin_run`, `claim_run`) → `start` (`Heartbeat::spawn`) | none | durability: run row + attempt fence **before any native effect**; physical source rows stored once (IPC-hashed, `physical:rows:<hash>` problems), then protected on later runs (`physical_cache.protected_rows`). Coordination: lease claim (30 s), heartbeat renewal (10 s), durable cancellation flag. Observation: progress tapped to a store stream (`Progress::tapped`, batch 64, flush 200 ms) | `tokio::spawn` supervisor task (`inherit_activity`); WS-W (begin_run, claim_run, physical protections), periodic WS-W heartbeats, progress WS-W | **every solve**; grows with physical source size on the first run only | SurrealDB; bespoke lease/fence/heartbeat (`pse-operations/src/canonical_execution.rs`) |
| 1.15 | Native session and solve | `staged.rs::Staged::open_queued` → `NativeSession` (SESSION thread); `Staged::execute` → `session.prepare(assessment)`, `session.step(...)` | assessment program (original-model obligations), native solve (Ipopt/KINSOL/HiGHS/... via `pse-backend-native`), in-thread original-space assessment and candidate decision | capacity: population ticket, CPU permits per step, RES `math:solve-results`, `modeling:qualified-result`, session lease (stack + foreign + inner-session bytes). Cancellation: `CancelSource` → native stop flag; teardown joined. Observation: progress events | **SESSION OS thread** (new per launch); native FFI inside it | **every solve: a fresh thread and session**, closed after the run (`staged.close()`), so native retained state does not survive across Python calls | native solver libraries (R4); bespoke session protocol |
| 1.16 | Completion capture | `run.rs::RunResult::completed` → `assess_candidates`, `completion.rs::capture_completion` | candidate usability (`permits_result`/`permits_seed`), solve_runs rows, accuracy goals, lineage rows | assurance: completion is immutable; exporters copy it | in-process | every solve | bespoke |
| 1.17 | Durable publication | `DurableAttempt::finish` → `store_tables` → `result_projection::store_run_result_table_queued` (`ResultAppendQueue`); `store_seeds`; `terminate` (closed manifest, terminal class) | encode **every advertised relation** (modeling: ~20 relations, `results.rs::table_ids`) to bounded IPC blocks; derive scalar row/output indexes; store warm-start seeds as chunks | durability: closed immutable manifest; payload digests; per-row locators (`canonical_result_rows`, block outputs). Coordination: heartbeat alive through every export phase; `close_result_ingestion` on rejection. Validation: VAL in the relation builders. Recovery: `export_failure`, `recover_closed_attempt` | WS-W per result batch (≤ 4 MiB message) plus manifest/terminal transaction | **every solve**; grows with **result size** (variables × constraints × relations) and number of seeds | Arrow IPC `StreamWriter`; SurrealDB; bespoke manifest/index scheme |
| 1.18 | Python result object | `NativeModelingResult` (`pse-py/src/workflow/modeling.rs`); `ModelingResult` (`python/pse/_modeling.py`) | none | in-memory `Arc<ModelingResult>`; no store read needed for `.table()` | RS→PY | per solve | PyO3 |

Session state after journey 1 is listed under answer (a) below.

---

## Journey 2 — value-only edit (parameter, fixed value or start), then re-solve

There are two routes.

**Route A**, the sanctioned value path: `pkg.solve_bound(case, PointOverlay, settings)`.
The `with_delta` doc says "numerical assignment remains the separate `solve_bound` intent"
(`workflow/modeling/canonical/delta.rs::with_delta`). Parameters and fixed coordinates replace
values; free coordinates replace starts. The parent revision is unchanged.

**Route B**: an authored value literal changed by `with_delta`/`reload_from_parent`. This
produces a new revision and follows journey 3. Blueprint §14.4 (ADR-0164 box): "Binding edits
reuse structure only when those values are not consumed structural literals."

Route A stages:

| # | Stage | Owner | Scientific work | Obligations (mechanism) | Crossings, txn | Repeats; grows with | Library vs bespoke |
|---|---|---|---|---|---|---|---|
| 2.1 | Decode overlay and settings | `pse-py/.../modeling.rs::solve_bound`; `documents::decode` (msgspec JSON → serde) | none | validation: bounded decode (`workspace_bytes/4`) | PY→RS | every edit; grows with the overlay | msgspec / serde_json |
| 2.2 | Attribution reservation and path demand | `bound.rs::prepare_bound_solve` | collect `BindingTarget::Path` demand | RES `modeling:bound-source` | in-process | every edit | bespoke |
| 2.3 | Declared execution with demand | `declared_execution_with_demand` (same as 1.8–1.11) | as in 1.8–1.11 | as in 1.8–1.11: PROT for the root row; engagement PROT; selected-cache candidate **rechecked against the store**; basis lookup keyed by `Bindings` **including `demand`** | ≥ 6 WS round trips (2 protects, ≥ 1 object read, ≥ 1 qualification query, 2 releases) | **every edit**. A path set not seen before misses the basis cache, so specialization re-runs for that demand set; repeated identical path sets hit | bespoke + Salsa |
| 2.4 | Overlay admission | `admit_overlay`, `validate_binding` (`workflow/bindings.rs`) | typed quantity/unit admission of the assignments against the compiled model | validation: physical type and unit match; role (parameter/fixed/free) | in-process | every edit; grows with the overlay | bespoke (`pse-quantity`) |
| 2.5 | Case resolution and rebind | `resolve_prepared_case` → `bound_case` | starts, states, providers; bound structure; **rebind** values into the cached view | reuse: view cache hit when states and order are unchanged; a fixed/free flip changes `bound_structure` and so the view key | in-process; JOB if missed | every edit; a rebind grows with the number of value coordinates **[I]** | pse-compiler/pse-math |
| 2.6 | Solve preparation | `finish_case` → `prepare_solve` | numerics policy, routing, provider binding (as 1.13) | as 1.13 | in-process | every edit | bespoke |
| 2.7 | Run, durable record, native solve | as 1.14–1.17 (`solve_case` → `prepared.start()` → `launch`) | full solve. **No warm start from the previous result** unless the caller uses `with_stored_start`/`with_primal_start` (`run.rs`), and the Python `solve_bound` exposes neither | as 1.14–1.17: new run + attempt + lease + heartbeat + all-relation export + seeds + terminal manifest; new SESSION thread | WS-W many; SESSION thread; FFI | **every edit**; grows with result size | as 1.14–1.17 |

Route A writes, per value edit: two protection grants and two releases (2.3), the run
registration, claim and heartbeats, physical-row protections, about 20 result relations as IPC
blocks with row indexes, the seed chunks, and the terminal manifest. It also starts and joins
one native session thread. The edit itself touches **no** source revision.

---

## Journey 3 — structural or expression edit, then re-solve

Python: `pkg2 = pkg.with_delta(SourceDelta)`, `pkg.with_declarations(rows)` or
`pkg.reload_from_parent(docs)`, then `pkg2.solve_case(...)`.

| # | Stage | Owner | Scientific work | Obligations (mechanism) | Crossings, txn | Repeats; grows with | Library vs bespoke |
|---|---|---|---|---|---|---|---|
| 3.1 | Parent protection | `canonical/delta.rs::with_delta` (`operation_scope(deadline)`) | none | coordination: PROT on the parent for the edit; the delta must name the exact parent key (`delta.parent`) | WS-W | per edit | bespoke |
| 3.2 | Parent hydration | `delta_parent` | read the ingress frame and the **complete parent membership** in 16 MiB pages, plus every declaration object (`membership_page`, `selected_objects`) | capacity: paged RES | WS-R × (pages + object chunks) | per edit; grows with **package size, not edit size** | bespoke |
| 3.3 | Intent validation | `apply_delta_parent` → `intentions(...)` | disjoint add/replace/remove sets; document identity (`named_id(package, path)`); IR edits may not assert source text | validation: VAL by building `modeling_declarations` and `documents` batches for the changed rows, then dropping them | in-process | per edit; grows with edit size | registry builders |
| 3.4 | Index and data closure | `source_index` (parsed fields for **all** parent rows, cached by content hash in `modeling_cache.parsed_fields`); `data_closure`; `MathService::modeling_revision` for record suppliers when data rows exist | parse field expressions, record suppliers | reuse: parsed-field cache | in-process; compiler on the calling thread | per edit; grows with package size (cache-mitigated) | bespoke + compiler |
| 3.5 | New revision publication | `CanonicalStore::edit(problem, Some(parent), op, edits)` (inside the delta path; `persist(.., previous)` for `with_declarations`/reload) | revision identity hash | durability: head-CAS on the parent, immutable object versions, membership intervals. `persist(previous)` for `with_declarations` additionally pages the **old membership** to diff, and returns the parent unchanged when there are no edits and the identity is equal | WS-W (one guarded transaction); WS-R pages | per edit | SurrealDB + bespoke |
| 3.6 | Parent release | `read.finish()` | none | coordination | WS-W | per edit | bespoke |
| 3.7 | Re-solve: declared execution | as 1.8 on the new revision | as 1.8 | PROT on the new revision | WS | per solve | bespoke |
| 3.8 | Re-solve: selected admission | `checked_selection` | candidate reuse when the selected dependency receipt is unchanged (`qualified_selected_candidate`); otherwise `select_into` re-hydrates the selected closure and the compiler lane re-admits. Salsa early-cutoff reuses unchanged declaration/expression phases (`with_modeling_compiler`, `LaneStamp` ordering; an out-of-order stamp uses a **detached** workspace) | coordination: qualification turn, single-flight, lane head; reuse: Salsa memo, `modeling_cache.selected` | WS-R (recheck/hydrate), JOB thread | per structural edit; grows with **selected closure** (hydration) and **changed phases** (Salsa) | **Salsa**; bespoke selection |
| 3.9 | Re-solve: basis, bodies, descriptions | as 1.10 | new frontier; unchanged bodies hit `modeling_cache.body`; changed bodies are described and **published** | durability of descriptions; acknowledgment of rebinding descriptions (`acknowledge_description_until`) | WS-W per new or changed body | per edit; grows with **changed bodies** | Salsa/Symbolica + bespoke |
| 3.10 | Re-solve: view, solve preparation, run, publication | as 1.12–1.17 | new view if the structure key changed; full solve | as 1.12–1.17 | as 1.14–1.17 | per solve | as above |

`reload_from_parent` (text route; `canonical/reload.rs`): it compares checksums and reparses
changed documents (§19, `workflows-and-results.md`). It then feeds the same delta/parent path,
with `RetainedIngress` holding the initial loader bytes (`canonical::reload::retain_initial`
is called after `modeling_from_documents`).

---

## Journey 4 — reading results back in Python, including inspection queries

| # | Stage | Owner | Work | Obligations (mechanism) | Crossings, txn | Repeats; grows with | Library vs bespoke |
|---|---|---|---|---|---|---|---|
| 4.1 | In-memory result tables | `ModelingResult.table(name)` → `NativeModelingResult::table` → `RunResult::into_table_cursor` (`workflow/results.rs`) → `ResultCursor` → `TableStream::from_result` | **re-encode** the relation from the immutable completion product, in chunks of 1024 rows | validation: VAL in relation builders; `EncodingState` caches **only failures**, so successful encodings are not memoized. Export: `checked_export` wraps buffers with owner-retaining wrappers (`retain_payload`), zero-copy | RS→PY via Arrow C stream (`pyo3_arrow`, `__arrow_c_stream__`); `py.detach` for cursor creation | per read; grows with relation size. The same relations were already encoded once for 1.17 | Arrow, pyo3-arrow; bespoke cursor |
| 4.2 | Canonical result read | `Runtime.results(run, attempt, relation, start, end)` → `connected_results.rs::Runtime::results` → `CanonicalStore::read_results` | manifest admission, block streaming | coordination: **reads are mutations**. `read_results` runs a `protected_query` with `QueryEffect::Mutation` inside `PROTECTED_BEGIN … COMMIT`, establishing a read protection (`READ_LIFETIME` 60 s). Capacity: RES `READ_BYTES` 48 MiB per reader | WS-W (protected manifest read), then WS-R per block | per call; grows with blocks read | SurrealDB; Arrow IPC `StreamDecoder` |
| 4.3 | Block decode and re-admission | `CanonicalResultReader::next_batch` → `result_blocks::decode_owned_result_block` → `FieldCheckedBatch::admit_owned` | decode IPC into owned buffers | validation: **VAL again** on every decoded block (already validated at write, 1.17). Bounds on fields/depth/rows before decode. Integrity: manifest digest | RS→PY via `TableStream::from_canonical`, which `block_on`s the async reader from the Python-side pull (refuses inside a runtime) | per block | Arrow IPC; DataFusion-backed VAL |
| 4.4 | Output and latest selectors | `output_results` (`results` + `check_output_field`, `verify_scalar_row`, `scalar_predicate` over stored row indexes); `latest_results` (SurrealQL `SELECT … FROM canonical_runs WHERE problem=$problem … ORDER BY sequence DESC LIMIT 1`, then as 4.2) | finite/missing predicates, time bounds | assurance: each index row is checked against the IPC row | WS-R/W | per call | SurrealQL + bespoke indexes |
| 4.5 | Run, attempt and manifest records | `run_record`, `attempt_record`, `result_manifest` → generated one-row `Builder` | none | VAL on the one-row batch; `result_manifest` uses `read_results` (protected) | WS | per call | registry builders |
| 4.6 | Progress | `Runtime.progress(run, attempt)` → `ProgressStream` | bounded event history | reads closed progress chunks (`read_chunks`) | WS-R | per call; grows with events | bespoke |
| 4.7 | Pre-run inspection | `ModelingPackage.inspect(case, settings)` → `NativeModelingPackage::inspect` → **`declared_execution`** → `DeclaredExecution::inspection()` | **full preparation** (1.8–1.11: root read, protection, selected admission, specialization, body description publication) to project member lineage and topology | as 1.8–1.11 | WS (as 1.8–1.11) | per call; cache-mitigated | bespoke + Salsa |
| 4.8 | Knowledge, declarations, source tables | `knowledge(owner)`; `declarations()` → `declaration_inventory` (full membership read); `source_tables()` → `encode_source_tables` (`full_source`, merges physical + modeling rows; VAL builders) | explicit exports | RES `modeling:source-export`; VAL on rebuilt batches | WS-R (full inventory) | per call; grows with package size | registry builders |
| 4.9 | Diagnostics (`diagnose`, `diagnose_linear`, `diagnose_jacobian`, `explain_nonlinear`) | `pse-py/.../modeling.rs` → `pse_runtime::workflow::modeling::diagnostics*` | structural rank and singular modes, linear/Jacobian diagnostics, elastic explanation | as 1.8–1.13 for preparation; native analyses on JOB/SESSION threads | WS + threads | per call | bespoke + faer/HiGHS **[I: per R2/R4]** |
| 4.10 | Resource usage | `Runtime.resource_usage()` → `SharedRuntime::report` | none | observation: DataFusion `TrackConsumersPool::metrics`, `PeakRecordingPool`, cache-service report, process peak RSS | in-process | per call | DataFusion memory pool |

---

## (a) Session state, and where the canonical store is required

**State held in the Python session** (`python/pse`): only frozen handles. These are
`Runtime(_handle: _NativeRuntime)`, `PhysicalContext(_handle)`,
`ModelingPackage(_handle)`, `PreparedOperation`, `ModelingResult`, `RunHandle` and
`TableStream`. Python holds no model state of its own. Settings are msgspec structs that are
JSON-encoded on every call (`codec.encode_json`).

**State held by the Rust side between edits.**

- Process-global (`pse-py/src/inspection/runtime.rs`): `PROCESS_RUNTIME`, which is the
  `Arc<Runtime{shared: SharedRuntime, executor, model: BoundModel}>`, and the static tokio
  executor.
- `SharedRuntime` (`env.rs`): the DataFusion `RuntimeEnv` and memory pool, the
  `compiler_cpu` semaphore, `NativeCacheService`, and `MathService` (`math.rs`).
- `MathService` holds:
  - `modeling_cache`, a DataFusion `DefaultCache`-backed `WeightedCache` with these families:
    ingress, parsed_fields, selected, numerics, body, encoded_body, frontier, basis,
    solver views, program and package;
  - `compiler_lanes`, the Salsa `CompilerWorkspace` per lineage/context, bounded by `jobs`;
  - the artifact cache;
  - `selected_flights`;
  - qualification turns;
  - population and CPU admission.
- `workflow::Runtime` (`workflow/mod.rs`): the `CanonicalDeployment` (WS client, attestation,
  optional producer), `Durability`, and `PhysicalCache`.
- `NativeModelingPackage` holds `native::ModelingPackage{runtime, revision: SourceRevision
  (canonical row + identity), physical, providers, accelerators, quantities,
  held_preparation: None}`, plus `physical_sources` bytes. Its docstring says "modeling
  sources live only in the canonical store" (`pse-py/src/workflow/modeling.rs`).
- Not retained between Python calls:
  - native sessions or solver state, because `launch` opens and closes a `Staged` per run;
  - protections, because the engagement closes after `prepare`;
  - the warm start, except as stored seeds in the canonical store.

**Steps in the edit → re-solve loop that require the canonical store** (all observed):

1. Runtime construction always connects to and opens the store, even with `ephemeral=True`
   (`NativeRuntime::new`).
2. Every package method enters `admit_activity()` on the canonical connection. It fails with
   `CallerClosed`/`ConnectionUnavailable` when the store is unusable.
3. Package admission persists source (`persist` → `edit`). The package holds only a
   revision reference (`ModelingPackage.revision`).
4. Every preparation:
   - reads the root declaration from the store under protection (1.8);
   - protects the revision and qualifies or hydrates the selected closure from the store
     (1.9);
   - publishes new body descriptions (1.10).
5. Every structural edit hydrates the full parent from the store and publishes a new
   revision (3.2–3.5).
6. Every durable run registers, claims, heartbeats and exports to the store. Ephemeral runs
   skip only 1.14 and 1.17 (`run.rs::attempt_for`).
7. Canonical result reads (4.2–4.5) and `modeling_revision(key)` reopen.

**The contracts that require this:**

- Blueprint §19 (`workflows-and-results.md`, *Cases, results and analytics*): "Ordinary
  execution durably retains its actual scientific outcome in the canonical substrate …
  Explicit ephemeral execution keeps result observations local … it still admits canonical
  sources and is not a fallback."
- §21: `pse.Runtime(... ephemeral=False)` reads "Ordinary construction is durable … preserving
  canonical source admission."
- §14.3 (`mathematics-and-compilation.md`, ADR-0164 box): "Database structural resolution
  supplies a semantically sufficient immutable selection to the shared
  admission/inference/specialization kernel. Protection covers source and interpretation
  inputs through off-transaction preparation, then retained product roots are admitted
  atomically before protection ends."
- §14.4 (ADR-0164 box): "Persisted complete dependencies govern reuse across restart; Salsa
  is an optional local accelerator."
- §14.4 text: "No compiler, protected read, attempt fence or mutable evaluator is retained
  as the basis … A previously qualified complete source receipt may be adopted only after a
  guarded store check verifies a fresh live pin".
- Scope table (`scope-and-open-design.md`, "Canonical store availability"): "Source admission,
  preparation and durable execution depend on the authenticated local store … no
  durable-to-ephemeral fallback; ephemeral execution omits result retention but still
  requires canonical sources."
- ADR-0164 (status **proposed**; Outcome): canonical revisions, protected selection and
  durable results are in SurrealDB. Its Options paragraph records that an embedded database
  was considered and set aside because it "introduces the engine into application builds and
  each process's resource envelope".
- ADR-0178 (proposed) adds the preparation engagement and settlement obligations (§14.3).

**Facts bearing on whether a live in-process model could serve the loop** (no recommendation):

- The scientific kernels are in-process already: the Salsa compiler lane, the specialization
  and view caches, `rebind`, `prepare_solve` and the native solve. None of them reads the store
  except through the selection/protection wrapper. `bound_case` already implements "prepared
  once per structure, rebound per values" in memory (`views.rs`).
- The store's contributions to the interactive loop are these:
  - source identity and immutable selection, with premises protected against concurrent
    retention or reclamation;
  - reuse qualification against current premises;
  - durable result retention, and replay/restart reuse of described bodies;
  - cross-process coordination: leases, heartbeats and durable cancellation.
- Under Python as built (abi3), deployment-local replay is refused (`local_python_configuration`
  returns `Err`), so store-received bodies are not reused unless an operator receipt is given.
  The store is still written with descriptions (1.10) that this session cannot receive.
- `HeldPreparationScope`/`held_preparation` exists (`engagement.rs`): a lane-owned set of
  engagements that keeps protections renewed across operations, avoiding re-selection.
  Python's `ModelingPackage` path sets `held_preparation: None` (`package_from_canonical`), so
  the Python interactive loop does not use it. Studies and lanes do **[I: W-B to confirm]**.
- `ModelingPackage` carries `physical`, `quantities`, `providers` and `accelerators` in memory.
  Declarations are not carried; they are re-read from the store as needed.

---

## (b) Engine (`pse-engine` / DataFusion) use in these journeys

`pse-engine` is 35,184 lines (27,919 of them in `session/`). In these journeys it is used for
the following.

1. **Resource substrate (every journey).** `EngineResources::build` supplies the DataFusion
   `RuntimeEnv`, `FairSpillPool` wrapped in `TrackConsumersPool` and `PeakRecordingPool`, the
   `DiskManager` spill directory and the `CacheManager`. All math, result and canonical
   reservations (RES) charge this pool. The math retention caches use DataFusion `DefaultCache`
   with `CacheKey`/`CacheValue` (`math/retention.rs`, `physical_cache.rs`). `CpuAdmission`, the
   compiler CPU semaphore, is installed as a `SessionConfig` extension and checked at
   `Runtime::from_shared`. This is **accounting and caching, not query work**.
2. **Field validation (VAL).** `NativeValidation(SessionState)` turns registry-declared local
   value domains into SQL/`Expr`, simplifies and coerces them, and builds `PhysicalExpr`s that
   are evaluated over Arrow batches (`pse-engine/src/validation.rs`,
   `pse-relations/src/validate/predicates.rs`). In these journeys it runs at:
   - document load (1.5) and re-validation (1.6);
   - source persistence builders (1.7, built only to validate, then dropped);
   - delta builders (3.3, same pattern);
   - result relation encoding (1.17, 4.1);
   - **decoded canonical blocks** (4.3);
   - one-row record builders (4.5);
   - source export (4.8).

   It validates per-row declared domains: ranges, enums, nested collections and nullability.
   The module doc says batch admission "does not certify keys or foreign keys"
   (`validate/mod.rs`).
3. **Physical admission (1.4).** `EngineFactory::candidate_checked` registers checked
   relations as table providers. `PhysicalInventory::load` builds a
   `LogicalPlanBuilder::scan(...).sort(primary key)` plus `declare_relation_output` per
   physical input relation and runs them through `prepare_many` and
   `PreparedComputation::execute_group` (one-partition sessions). `plans::units` and
   `plans::context` (`LogicalPlanBuilder::scan … distinct`) select unit and context rows.
   Relational obligations (keys, references, quantities, source spans) are expressed as
   `BoundObligation{plan: LogicalPlan}` "rows violating the obligation"
   (`validate/obligations.rs`). This is the only DataFusion **query execution** found in
   journeys 1–4.
4. **Arrow stream export.** `pse_engine::arrow_stream::BatchStream` backs the Python
   `TableStream` (`pse-py/src/inspection/stream.rs`).
5. **Error type.** `pse_engine::EngineError` is the common error wrapper (about 95 uses in
   `pse-runtime`/`pse-py`).

Not used in these journeys (search: `rg "SessionContext|LogicalPlanBuilder|\.sql\(|DataFrame|TableProvider"`
over `pse-runtime/src`, `pse-py/src` and `pse-admission/src`, excluding tests):

- no DataFusion SQL;
- no DataFrame or listing tables over results;
- no DataFusion scans of canonical results (result reads are SurrealDB plus IPC decode).

`pse_admission::authoring_driver::native::relation_plan`, the DataFusion grouped
document-parsing plan, has production code but only test callers
(`crates/pse-admission/tests/authoring_native_parser.rs`). The production modeling loader
calls the parser directly. The search was limited to those crates; other `pse-engine`
`session/*` machinery (operations, mutation, materialized, aggregate) was not traced and may
serve W-B journeys or none.

---

## (c) Immutable or validated state rebuilt or rechecked, and where

| What | Rebuilt or rechecked at | Crossing | Observed source |
|---|---|---|---|
| Relation batches validated at document load | Re-validated in `modeling_from_documents` (`OwnedDocumentSet::validate_context`, same registry and context) | PY→RS call 1.5 → 1.6, same process | `workflow/modeling.rs::modeling_from_documents` |
| Declaration rows already admitted | Rebuilt into a `modeling_declarations` batch only for VAL, then dropped (`persist`, `apply_delta_parent`) | RS → WS write | `canonical.rs::persist`, `delta.rs` |
| Root declaration of the case | Re-read from the store under its own protect/release on every solve, inspect and bound solve | WS (3 RPCs) | `canonical.rs::declaration` via `declared_execution_with_demand` |
| Selected admission (compiled closure) | Cached in memory, but **rechecked against the store** on every preparation (`qualified_selected_candidate` → `qualify_selection_window`), under a fresh protection | WS | `canonical.rs::checked_selection` |
| Specialization basis | Keyed by `Bindings` including path `demand`, so a value overlay with a new path set re-specializes. Additional acquisition premises are re-qualified against the store | in-process + WS | `preparation.rs::prepare_selected_basis` |
| Body descriptions | Rebinding descriptions re-acknowledged in the store (`acknowledge_description_until`); unsettled ones republished | WS-W | `preparation.rs::establish_preparation` |
| Revision protection | Granted and released per call (1.8, 1.9 and 1.11 in one solve; 3.1 and 3.6 per edit), plus heartbeat renewals | WS-W | `canonical.rs::protect`; `engagement.rs` |
| Physical source rows | IPC-encoded and hashed once per `PhysicalCache` generation; re-protected on every durable run | WS-W | `durable.rs::put_physical_rows`, `physical_cache.rs` |
| Parent package on structural edit | Full membership and all declaration objects re-read from the store (paged). Parsed fields come from the content-hash cache | WS-R | `delta.rs::delta_parent`; `canonical.rs::persist(previous)` |
| Solver view | Rebound per values (`rebind`); rebuilt when the bound-structure, order or profile key changes | in-process | `views.rs::bound_case` |
| Numerical policy, providers, assessment program | Resolved per solve (`prepare_solve` → `policy_resolution_owner().resolve`, `inner_registrations`, `assessment`). Some caching via `modeling_cache.numerics` **[I: key and coverage not traced]** | in-process | `cases.rs::finish_case`, `staged.rs::execute` |
| Native session | New OS thread and native session per Python `solve_*`; closed after the run, so no cross-call native retention | thread | `run.rs::launch`, `math/staged.rs` |
| Result relations | Encoded once for durable export (1.17) and **again for every Python `.table()` read** (4.1; `EncodingState` memoizes only failures) | in-process → WS / PY | `durable.rs::store_tables`; `results.rs::into_table_cursor` |
| Stored result blocks | VAL re-run on decode (`FieldCheckedBatch::admit_owned`) after VAL at write | WS → RS → PY | `connected_results.rs` |
| Settings and overlay documents | JSON-encoded in Python (msgspec) and decoded in Rust (serde) on every call; `solve_profile` re-derived each call | PY→RS | `pse-py/src/workflow/settings.rs`, `documents.rs` |
| Deployment identity | `check_deployment` pointer checks at `Runtime::from_shared`; module-path identity at `NativeRuntime::new` | once per `Runtime` | `workflow/mod.rs`, `pse-py/src/workflow.rs` |

---

## Uncertainties and unfinished edges

- Store RPC counts per stage are read from code structure, not observed. Paging, retries
  (`GUARDED_ATTEMPTS` 32) and protection-owner tasks can add round trips.
- `modeling_cache.numerics` and `assessment` program caching keys were not traced. The claim
  that they are resolved per solve rests on the call sites only.
- How much Salsa early-cutoff reuses across a structural edit (3.8) is asserted from
  §14.3/§14.4 and the lane code, not from a traced edit.
- `HeldPreparationScope` users outside the Python package path were not enumerated (W-B).
- `pse-engine/src/session/*` areas beyond validation, resources and physical admission were not
  mapped for these journeys. The absence of other uses is limited to the search scope stated
  in (b).
- Whether `ephemeral=True` also skips physical-row storage: `put_physical_rows` is called only
  from durable registration (`register_selection`), so it is skipped **[I: from call sites]**.
