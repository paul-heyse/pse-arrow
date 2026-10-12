> Supporting evidence for [the solver methods, selection and diagnostics review](../../reviews/design_review_solver-methods-selection-and-diagnostics_2026-10-12.md). Wave-1 worker return, A2 (code-mapper): observation, persistence, identity and replay, published unchanged by the coordinator. Baseline: `ff5a4aa2f` plus the Plan 36 working tree (diff fingerprint `45ceea51b0698679`). It is read-only and static: nothing was executed. Its judgments are the worker's; the principal review owns the verdicts.

# A2: observation, persistence, identity and replay

**Baseline:** working tree at HEAD `ff5a4aa2f` plus Plan 36, read statically. Nothing was run, so every claim is *Interface-checked* (I read the source) or *Proposed*.

## Summary

1. **Typed decision evidence is thin.** It is only `WorkEvidence` (four optional counters), a few flags and a Boolean-like `CallbackEvidence`. All richer solver statistics (POUNCE `SolveStatistics`, timings, callback seconds, native options) go to the string-keyed `metrics` map. That map flattens to `runtime.solve_metrics` rows, which are generic name/value pairs.
2. **There is no typed record of a whole ladder with time.** `solve_strategy_events` records each rung's events with work-charge counts and the observation, transition and permission. It carries no wall time or duration anywhere. Per-rung start and end are not recorded, and the work counts are the only cost measure.
3. **Attempt-level time and resources are not recorded.**
   - Neither `SolveReport` nor `canonical_attempts` has a wall-time field.
   - Time survives only as per-event `elapsed_seconds`, per-demand callback seconds, POUNCE `timing.*` metrics, and `at` stamps on durable progress events.
   - The `ResourceReport` (pool peak, process peak RSS) is process-wide, not per attempt.
4. **Studies do not summarise across points.** `StudyReport` holds per-occurrence outcomes plus a lifecycle `StudyDecision`. No success rate, iteration distribution or time is aggregated.
5. **Identities are rich but built for exact reuse.** Preparation, request, profile, numerical, physical and environment hashes are all persisted in `runtime.run_lineage`. But `preparation_identity` and `profile_key` fold in controls, backend settings and the native build. A cross-run history key needs a coarser, structure-only key, and none is persisted for that purpose.
6. **Replay is not complete.** The request bytes, immutable revision, effective options and native defaults, start snapshot and provenance are stored. Thread count and `time_limit` are inside the controls hash but only the hash is stored in lineage. No model-file export (SCIP or HiGHS) exists in production code, and there is no host or CPU identity.
7. **Python reaches the generic tables, not typed statistics.** Metrics, progress, lineage and strategy events are reachable as `TableStream`s. Dynamics `Report.statistics` is not in the persisted relations I found.
8. **Diagnostics are on request only.** I found no automatic trigger on a failed solve. Modeling diagnostics are returned as in-memory handles, not stored. Certificates and `local_validity` rows are produced automatically when the adapter or request supplies them.

## 1. Observation flow, adapter to decision

**Carrier: `SolveReport`** (`BN/solve.rs:1942-1994`). Fact: it is built by `SolveReport::new` (`BN/solve.rs:2088-2127`) with `evidence = Evidence{abandoned, ..default}`, `metrics = {}`, and `events`/`dropped_events` copied from `execution.progress.snapshot()`.

- **Typed (decision-readable):**
  - `evidence: Evidence` (`BN/solve.rs:1855`). It holds `work: WorkEvidence`, `abandoned`, `callback: CallbackEvidence`, `start_submitted`, `reused_native_state`, `restart`, `working_set_submitted`, `kkt`, `coefficient`, `original_bound`, `conic`, `local` and others.
  - Also typed: `termination`, `qualification`, `quality`, `observation`, `certificate`, `global`, `least_infeasible`, `warm_start`, `start_receipt`, `preprocessing`.
  - `WorkEvidence` (`BN/solve.rs:1842`) is four `Option<u64>` counters: evaluations, iterations, factorizations, proof_steps.
- **Untyped:** `metrics: BTreeMap<String, Metric>`, with the comment "Decisions read `evidence`, never these keys" (`BN/solve.rs:1977`).
- **Also untyped:** `provenance: BTreeMap<String,String>`, `options`, `native_defaults`.
- **Optional typed library statistics:**
  - `highs_diagnostics` is requested explicitly.
  - `pounce_statistics: Option<Box<pounce_rs::SolveStatistics>>` is the complete POUNCE statistics (`BN/solve.rs:1958`). It is set at `BN/pounce.rs:781`. It is also JSON-flattened into `metrics["statistics.*"]` (`BN/pounce.rs:772-778`).

**How counters are filled.**
- `CallbackState::finish` (`BN/callback.rs:400-440`) sets `evidence.work.evaluations` to the sum of per-demand call counts. It writes `callback.trial_rejections`, `callback.regime_crossings`, `callback.terminal_failure`, `callback.<demand>.calls` and `callback.<demand>.seconds` into `metrics`.
- `CallbackEvidence` carries only `trial_rejections`, `regime_crossings` and `terminal_failure`. Per-demand seconds exist only as metrics (`BN/callback.rs:226-227, 323`).
- Iterations are set per adapter. Examples: `BN/uno.rs:418` and `BN/ipopt/sequence.rs:578` set `evidence.work.iterations`; `BN/pounce/observed.rs:93` sets factorizations. Other adapters (e.g. `BN/conic.rs:851`, `BN/execution/pounce_convex.rs:670`) write `"iterations"` only as a metric.
- *Interpretation:* an adapter that writes iterations only as a metric leaves `WorkEvidence.iterations = None` in its typed evidence. Which adapters do this requires a per-adapter audit that B-workers cover.

**Progress.**
- `Execution` (`BN/solve.rs:728-760`) has `started: Instant` and `progress: Arc<Progress>` bounded by `Controls.history`.
- `Event{phase, elapsed, values, incumbent}` (`BN/solve.rs:~435`) are copied into `SolveReport.events`, with `dropped_events` counting overflow.
- `IncumbentEvent` is typed (objective, dual_bound, gap, nodes, seconds, optional primal).

**Wall time.**
- Fact: `SolveReport` has no total-elapsed field.
- Time appears as `Event.elapsed`, `IncumbentEvent.seconds`, `callback.*.seconds`, POUNCE `timing.*` (`BN/pounce.rs:~790`), and the HiGHS event `seconds` (`BN/highs.rs:1406`).
- *Search coverage:* `BN/solve.rs` for `Instant`/`elapsed`/`wall`, plus `rg` for `"seconds"`/`"solve_seconds"`/`"wall"` across `BN`. I did not audit every adapter file for ad hoc timing metrics.

**Driver: `Trace` (`RT/math/strategy.rs:846`).**
- Fields: `declaration`, `original`, `backend`, `profile`, `start`, `starts`, `products: Vec<RungProducts>`, `events: Vec<Event>`.
- The driver `Event` (`RT/math/strategy.rs:705`) has mechanism, kind, phase, original conclusion, decision hash, observation, transition, permission, `work: Option<WorkCharge>` and `cause`. It has no timestamp or duration.
- `report_work` (`RT/math/strategy.rs:~1806-1833`) copies `evidence.work` into the charge. This is how the typed evidence reaches `WorkObservation`.

**Decision inputs.** `automatic_decision_key` (`RT/math/strategy.rs:168-235`) hashes the composition request, candidates, decision, the last observation (native termination, original conclusion, permission) and `WorkObservation`. Counts are the only cost measure that enters automatic selection. No wall time or per-iteration measure enters it.

## 2. Persistence

All relation facts are from `docs/generated/relations/runtime.md`, which is generated and read-only.

| Relation | What it keeps | What it lacks |
|---|---|---|
| `solve_runs` v5 | Termination, qualification, objective, validation error, transformation hash, commitment | iterations, evaluations, time |
| `computation_runs` v2 | Kind, source and profile identity, state, termination, backend, native code and status | iteration or time columns |
| `solve_metrics` | `(run_id, step, namespace, name)` → one of real, integer, boolean, text, unavailable | typing beyond that, no solver-statistics schema |
| `solve_strategy_events` | Per rung event: mechanism, kind, phase, scope, identities, `attempts/evaluations/iterations/factorizations/proof_steps`, failure diagnostics, arclength `path_events`, surrogate fields | any time or duration |
| `solve_strategy_products` | Product receipts, source dependencies | not a cost record |
| `resolved_numerics`, `route_decisions` | Budgets and provenance; capability admission and eligibility per backend | outcome of the chosen route |
| `run_lineage` | Hashes listed in §4 | no outcome or cost |
| `execution_statistics`, `cache_statistics` | Process-level name/count sidecars | not per attempt |

**`push_native_metrics`** (`RT/workflow/results.rs:752`) writes, in order:
- qualification;
- start snapshot (`request`) and warm-start snapshot (`available`);
- every `metrics` entry (namespace `metric`);
- every `provenance` entry (namespace `native_provenance`);
- effective `option` and `native_default` entries;
- events (`push_events`, `RT/workflow/results.rs:543`), one namespace `event.<i>.<phase>` each;
- the HiGHS rays, IIS and ranging when the HiGHS feature is on;
- presolve and preprocessing receipts.
- Every step is skipped when the caller did not request the `solve_metrics` relation (`builder.wanted()`, `RT/workflow/result_export.rs:619`).
- *Interpretation:* typed `WorkEvidence` is not written to `solve_metrics` as such. It is persisted only as the strategy-event counts and whatever the adapter also put in `metrics`.

**Durable progress.**
- Terminal events are written to the reserved `__progress` result set (`RT/workflow/durable.rs`, 512 events per receipt, bound at `RT/workflow/progress.rs`).
- They are read back by `Runtime::progress` (`RT/workflow/progress.rs:30`).
- Coverage is typed: `ProgressCoverage::Sampled{observed, queued, dropped_before_queue, acknowledged, unacknowledged, writer}` (`RT/workflow/progress_documents.rs:14-34`).
- Durable events carry `at` (Unix microseconds) and `sequence`/`step` (`python/pse/contracts/documents/__init__.py:5248`). This is the only per-event wall clock in storage.

**Operational store.**
- `canonical_attempts` fields: key, run, generation, claim_operation, request (bytes), worker, `expires_at`, ingestion_open, closed, terminal, `outcome` (string), `completion` (bytes), seed_manifest, closed_manifest and others (`surreal.surql:48-64`).
- No start, end or duration column. `expires_at` is a lease expiry.
- `canonical_runs` keeps `request` and `source_selection` bytes and `terminal_class`.
- `canonical_study_points` keeps policy, descriptor, facts, outcome, start bytes and the settled flag.

## 3. Studies (Q2)

- *Interface-checked.* `StudyReport` (`RT/workflow/study_execution.rs:89-107`) holds one `PointOutcome` per occurrence, `results`, `StudyDecision{actions, conclusion}`, and `PreparationCounts{views, observations, rebuilt, shared}`.
- `record_execution` (`RT/workflow/study_execution.rs:956-1070`) builds each `PointAttemptOutcome{attempt_id, lifecycle, scientific, diagnostic, start, effect}` (`crates/pse-model/src/study.rs:353`).
- `study_outcomes` (runtime.md:2784) is one row per occurrence: state, attempt state, usable, seed permission, candidate use, effect, start provenance, diagnostic.
- *Absence claim, with coverage:*
  - I searched `RT` and `crates/pse-operations/src` for success_rate, failure_count and percentile; there were no hits.
  - I read `StudyReport`, `PointAttemptOutcome` and the `study_outcomes` columns.
  - I did not read the Python `StudyResults` summaries.
  - Result: no cross-point iteration, evaluation or time aggregate in the Rust study reports. Per-point solver cost lives only in each point's own `solve_metrics` and strategy events.
- Preparation reuse is counted (`PreparationCounts`), but it is an in-process observation ("not cross-process execution or reuse evidence", `RT/workflow/study_execution.rs:100-103`).

## 4. Identity keys for a history layer (Q3)

Persisted location for all of these is `runtime.run_lineage` unless noted: `request_identity`, `preparation_identity`, `profile_identity`, `numerical_identity`, `physical_identity`, `environment_identity` (`RT/workflow/completion.rs:~170-225`).

| Identity | Definition | Changes that invalidate it |
|---|---|---|
| `preparation_identity` (`SolvePreparationV3`, `RT/math/solves.rs:1531`) | Hash of snapshot identity (the adapter inventory), `strategy_profile()`, `numerics.key`, each provider's configuration key, factorable program key, `plan.structure().key()`, artifact keys, and the compatibility stamp (layout, profile, data, backend) | Structure, artifacts, providers, numerics, **any** profile or controls change, **data** (via compatibility.data), backend, native build, strategy composition |
| `seed_preparation_identity` (`SolveSeedPreparationV1`, `RT/math/solves.rs:~1590`) | Providers + structure key + artifacts + `compatibility.layout` + backend | Structure, artifacts, coordinate layout, backend. Not values, start policy, options or limits (stated in its doc) |
| `Compatibility{layout, profile, data, backend}` (`BN/solve.rs:892`) | Seed coordinate layout; native session profile; parameters, fixed values, coefficients and bounds | `layout`: coordinates. `profile`: session controls and settings. `data`: numeric values and bounds. `same_session` ignores `data`. Stored with seeds in `canonical_result_seeds` (layout, preparation, profile, data, backend) |
| `profile_key` (`SolverProfileV5`, `RT/math/solves.rs:5369`; persisted as `profile_identity`) | `hash_session` + composition + reconstruction + full `Controls::identity()` + `execution::LINKED.build_identity()` + selection + sensitivity request | Any control (`time_limit`, `iterations`, `threads`, `history`, hessian, reuse, start, options, `foreign_bytes`), backend settings, the selection, the **native build** |
| `Controls::identity` (`NativeControlsV3`, `BN/solve.rs:355`) | serde-derived hash of all `Controls` | Any control field above |
| `hash_session` (`RT/math/solves.rs:2385`) | Same controls with `time_limit`, `iterations`, `history`, `reuse`, `start`, `foreign_bytes` zeroed, plus presolve and numerics keys and `backend.identity()` | Session-relevant settings only, so limits don't change it |
| `request_identity` (`SolveRequestV3`/`SolveStrategyRequestV3`) | Preparation + numerics + hessian + compatibility + explicit start content + proposal start + automatic products | All of the above plus start payload |
| `numerical_identity`, `physical_identity` | `request.solve.numerics().key`, `request.source.physical.key` | Numerics policy; physical-source data |
| `automatic_decision_key` (`NumericalDecisionV2`) | Hash of one automatic step: request, candidates, decision, last outcome, work | One decision's inputs; persisted as `decision_identity` in `solve_strategy_events` |
| Event identities (`solve_strategy_events`) | `strategy_identity`, `original_identity`, `profile_identity`, `start_identity`, `derived_identity` | Per-rung inputs |
| `route_decisions` (v2) | `request_identity`, `snapshot`, eligibility per backend | Request or capability inventory |
| `environment_identity` (`CompletedEnvironmentV1`, `RT/workflow/completion.rs:196-205`) | Hash of GMP/MPFR versions + convention (`MathEnvironment::identity`, `PM/lib.rs:193-207`) + `attestation().build` + backend name + every `native.provenance` pair | GMP/MPFR, build attestation, backend, or any provenance string (library version, FERAL effective config) |
| `NativeBuildV1` (`BN/execution.rs:629`) | POUNCE source text + each linked adapter's build hash | Any linked adapter or POUNCE source change |

Gaps for a history key (*Interpretation*):
- **No structure-only persisted key.** The closest, `seed_preparation_identity`, is stored in `canonical_result_seeds.preparation` but is built for seed reuse. It omits bounds and values.
- **Class and size features are not hashed.** Nothing persists problem size (rows, columns, nonzeros) or a problem-class descriptor beside the outcome. `route_decisions` carries classes, but with no outcome.
- **`preparation_identity` folds in the full profile.** Two runs differing only in `threads` or `time_limit` get different `profile_identity`. `profile_key` hashes full controls; only `hash_session` zeroes limits.
- **No outcome-bearing record shares a key with these hashes.** `run_lineage` has no outcome and `solve_runs` has no hashes. They join via `run_id`/`step`.
- Fact: I found no persisted "settings identity" keyed by `NativeControlsV3` by itself. It is only inside `profile_key`.

## 5. Replay (Q4)

**What is stored today:**
- `canonical_runs.request` and `source_selection` bytes, and `canonical_run_sources` (revision link).
- The immutable revision. `run_lineage` doc says "source provider and parameter data are retained with the immutable revision" (runtime.md:2248).
- Effective options (`option`) and native defaults (`native_default`) as `solve_metrics` rows (`RT/workflow/results.rs:818-841`).
- The submitted start (`start/request`) and any available warm start (`start/available`) as text snapshots.
- `native_provenance` entries (POUNCE version, FERAL effective configuration at `BN/pounce.rs:861-868`, HiGHS version string, MKL build).
- `profile_identity`, `numerical_identity`, `environment_identity`, backend, and the complete native termination.

**What is missing (*Interpretation*):**
- The resolved `Controls` values themselves (threads, `time_limit`, `iterations`, `history`, `foreign_bytes`). `option` rows are the native option map; `Controls` appear in lineage only as a hash.
- Host identity, CPU or microarchitecture, and the actual granted thread count. `ExecutionScope::admitted_threads` exists at runtime (`BN/solve.rs:845`) but is not persisted. A `linear.threads` metric exists for POUNCE only (`BN/pounce.rs:~750`).
- Native-build contents: only the hash `NativeBuildV1`, no component list. `BuildInfo` (`crates/pse-buildinfo/src/lib.rs`) has version, rustc, profile, git SHA and lockfile checksums, but `captured()` leaves git and lock fields empty. They are filled only by the outer observation.
- Random seeds: no RNG state in `Controls`, and I found none in `BN`. Only `RT/math/surrogate/library.rs` matched my RNG grep (`rng|random_seed|StdRng|seed_from`) in `BN` and `RT/math`. I did not read the match.
- A portable problem export. I searched `BN` and `RT` for `writeModel`, `Highs_writeModel`, `SCIPwrite*`, `.mps` and `.cip`: no production hits. Only a test fixture `tests/fixtures/scip/bt-pr-liquid-tpd.cip` (`BN/scip_tests.rs:1586`) is used, as an offline experiment. Neither HiGHS `writeModel` nor SCIP write APIs are called from production code.
- Time-dependent state: deadlines and cancellation timing; the event stream is bounded by `history`, so replay of the callback sequence is not guaranteed.

**Existing replay machinery is not solve replay.** `ReplayAdmission` / `LocalReplay` (`RT/math/portable.rs:418`) admits reconstruction of prepared mathematical artifacts for a producer or receiving root. It governs reuse of preparation, not repetition of a numerical solve.

**Proposed:** to reproduce a failed solve, add a `Controls` snapshot to lineage (or persist the full `Controls` document next to `profile_identity`), record granted threads and a coarse host identity in the environment hash inputs, and persist the SCIP/HiGHS model (or the POUNCE layout) for failed attempts on demand. The library write APIs are cheap, but I did not verify their API surface.

## 6. Python exposure (Q5)

- `StoredResult` (`python/pse/_runs.py:51`): `run_id`, `usable`, `completion`, `diagnostics`, `run_key`, `attempt_key`, `table(relation)`, `attempt_record()`, `progress()` (a `ProgressStream` with `coverage`), `export`.
- `RunResult`/`RunHandle`: `table(name)`, `tables()`, `diagnostics()`, `accuracy_goals`, live `progress()` and `progress_count()`, `completion`. The typed fields on `ProgressEventDocument` are `at`, `elapsed_seconds`, `incumbent`, `phase`, `sequence`, `step`, `values` (typed metric documents).
- `PreparedOperation`: `identity`, `route`, `eligibility`, `strategy_profile`, `composition_request`, `numerical_strategy`.
- Any relation, including `runtime.solve_metrics`, `runtime.solve_strategy_events`, `runtime.run_lineage` and `runtime.route_decisions`, comes back as an Arrow `TableStream`. So metrics, strategy events and the environment hash are reachable, untyped.
- **No typed per-solver statistics** for the solve path. POUNCE statistics survive only as flattened `statistics.*` metrics.
- **Dynamics statistics:** `dynamics::Report.statistics: Vec<serde_json::Value>` is held in memory (`BN/dynamics.rs:2128`). Shooting concatenates windows' statistics into the in-memory report (`RT/workflow/shooting.rs:1525`). I found no relation column or Python API for it. *Search covered* `RT/workflow/simulation_results.rs`, `RT/workflow/dynamics*` and `python/pse/_native.pyi` for `statistics`. The only `statistics` hits in `_native.pyi` are `statistics_bytes` (cache policy, lines 50/77) and the diagnostics `statistics()` at line 323; the latter is the `ModelingDiagnostics` handle (`dict[str,int]`). I did not exhaustively read `pse-py/src` for another dynamics path. A probe to settle this is below.
- `ModelingDiagnostics` (`python/pse/_modeling.py:439`): `table(name)`, `complete`, `profile`, `rank`, `cutoff`, `findings()`, `statistics()` (`dict[str,int]`, structural counts such as `free_variables`), `coordinates()`, `singular_modes()`.

## 7. On-request diagnostics (Q6)

- *Fact:* `diagnose_case`, `diagnose_linear` and `diagnose_jacobian_optimization` are called from `pse-py/src/workflow/modeling.rs:516, 582, 642` and from `conformance.rs`/`diagnostic_samples.rs`, in the callers I found by `rg`. I found no call from a solve-failure path.
- *Fact:* `RT/workflow/modeling/diagnostics.rs` has no reference to the canonical store, durable or `store`. The results are returned as handles (Python `ModelingDiagnostics`), which export `runtime.modeling_diagnostics`/`modeling_findings` on demand. `modeling_linear_diagnostics` and `modeling_nonlinear_explanations` have their own `diagnostic_linear.rs`/`diagnostic_nonlinear.rs` paths.
- **Automatic on failure:**
  - `infeasibility_certificates` rows are written whenever a `SolveReport.certificate` exists (`push_certificate`, `RT/workflow/results.rs:603`, called from `result_export.rs:1292` and `modeling_results.rs:398`). The certificate comes from the adapter, not from a diagnostic trigger.
  - `local_validity` rows are produced with the requested KKT-point quantities (`RT/workflow/local_analysis.rs`, `fitting/results.rs`). Requested, not failure-triggered.
  - `modeling_checks` rows come from result publication (`results.rs`, `modeling_results.rs`).
  - *Interpretation:* the least-infeasible candidate is also automatic (`SolveReport.least_infeasible`).
- No failure-conditioned diagnosis, such as running the structural or conditioning diagnostics after a failed solve.

## 8. Resource observations (Q7) and environment (Q8)

- **Per attempt:**
  - `Controls.foreign_bytes` and `Execution.memory` are inputs (the allowance admitted).
  - `failure_bytes()` and `report_allowance()` exist on `SolveReport`/`Controls` as retained-size bounds.
  - `WorkAdmission::admit/observe` receives `WorkEvidence` (`BN/solve.rs:685-687`).
- **Process-wide:**
  - `ResourceReport{limit_bytes, pool_peak_bytes, pool_reserved_now, process_peak_rss_bytes, top_consumers, caches}` (`RT/peak.rs`, `RT/env.rs:119`).
  - `observation_peak_bytes()` / `reset_observation_peak()` (`RT/env.rs:111-117`) allow a pool peak since the last reset.
  - Persisted only in `cache_statistics` / `execution_statistics` sidecars, which are not per attempt.
- **Absent:**
  - I found no per-attempt foreign allocation peak, admission wait time or queue time. *Search:* `rg` for peak/high_water/wait_seconds/queue_wait/waited in `RT`, `pse-engine`, `pse-columnar` and `BN`.
  - `TaskAdmission` (`RT/math/strategy/admission.rs:12`) has a ledger of work counts and memory leases, with no timing fields.
- **Environment identity:** yes, one hash (`environment_identity`, §4). It contains only GMP/MPFR, a build attestation hash, backend and native provenance strings. No hostname, CPU model, core count, OS or governor.

## 9. Tracing and measurement infrastructure

- *Fact:* in the files I searched, `tracing` appears in `pse-engine`, `pse-compiler/workspace.rs`, `pse-runtime/math.rs` (`pse.case.program_assembly`, `RT/math.rs:816`), and as `tracing::warn!` for canonical failures (`RT/workflow/diagnostics.rs:73`, `crates/pse-operations/src/canonical.rs:1683`). `PM/execution.rs` also matched.
- *Fact:* `BN` has no tracing in the grep for `tracing::`, `info_span`, `span!` and `#[instrument]`. `tracing-subscriber` is a dependency only of `pse-testkit` (`crates/pse-testkit/Cargo.toml:25`).
- *Interpretation:* no production subscriber or exporter exists in the workspace; the spans reach no sink outside tests.
- `just case-measure` and `benches/` (`docs/dev/validation-assessment.md`, "Case measurements") are a qualification campaign: they record host placement, competing owners and intervals separately, but nothing from them flows into runtime storage. Fact from the document: "Long-lived service RSS/high-water observations are not attributed to an individual case."

## 10. Probes I did not run

1. **Is typed `WorkEvidence.iterations` ever `None` where the native library reports iterations?**
   - Command: `rg -n 'evidence\.work\.iterations\s*=' crates/pse-backend-native/src` against `rg -n '"iterations"' crates/pse-backend-native/src`.
   - Expected: conic and `pounce_convex` list iterations only in `metrics`.
2. **Dynamics statistics reach storage?**
   - Command: `rg -n 'statistics' crates/pse-runtime/src/workflow/simulation_results.rs crates/pse-runtime/src/workflow/dynamics.rs crates/pse-py/src`.
   - Expected: no relation row.
3. **Does a durable run store the `time_limit`?**
   - Command: read `canonical_runs.request` bytes of a recent durable run through `python -I -c "..."` against `$PSE_SURREAL_STATE`.
   - Expected: the request document contains the profile, so `Controls` may be recoverable. This could change the §5 "missing" list.
4. **Is POUNCE `timing.*` present in a stored `solve_metrics` result?**
   - Run the smallest NLP through the Python API with `solve_metrics` requested and list `namespace='metric'`, `name LIKE 'timing.%'`.

## Key files

- `crates/pse-backend-native/src/solve.rs`
- `crates/pse-backend-native/src/callback.rs`
- `crates/pse-backend-native/src/pounce.rs`
- `crates/pse-runtime/src/math/strategy.rs`
- `crates/pse-runtime/src/math/solves.rs`
- `crates/pse-runtime/src/workflow/results.rs`
- `crates/pse-runtime/src/workflow/completion.rs`
- `crates/pse-runtime/src/workflow/study_execution.rs`
- `crates/pse-runtime/src/workflow/progress.rs`
- `crates/pse-runtime/src/workflow/progress_documents.rs`
- `crates/pse-runtime/src/env.rs`
- `crates/pse-operations/src/generated/surreal.surql`
- `docs/generated/relations/runtime.md`
- `python/pse/_runs.py`
- `python/pse/_modeling.py`