# Assurance, identity and observation: bounded supporting assessment

Inquiry **H** of the [principal efficiency-alignment review](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md). This is supporting
evidence, not an independent principal review, finding ledger or qualification receipt. The
principal review owns the combined judgment, finding identifiers (`F01`…) and rule impacts
(`RC01`…); local labels below (`H-nn`) are investigation labels only. Coordinator source
checks, reconciliation and probe results are in the [evidence README](README.md).

Baseline: `main` at `b01a45d71a5f5ab29991ad193679f5588ac3662f` (2026-10-11), production source clean.
Assessed by a fresh read-only `design-reviewer` against Core 3.4, Heuristics for Efficient Architecture
1.0, Process Simulator 1.5 and the [pse-arrow binding](../../design_principles/binding/pse-arrow.md).
No builds, tests or services were run by the inquiry; probes it proposes are listed for later
settlement unless the README records them as executed.

---

Bounded supporting assessment for the efficiency-alignment review (baseline `main` at
b01a45d71a5f5ab29991ad193679f5588ac3662f, 2026-10-11). Standard: core 3.4 with heuristics 1.0,
process-simulator 1.5. Local labels H-01 to H-05. The principal reviewer owns the combined
judgment and the final finding IDs.

## 1. Scope examined and coverage limits

**Read in full or at the symbol level:**

- `crates/pse-buildinfo/identity.rs`: `FileObservation`, `LocalRuntimeObservation`, `DeploymentAssociation`, `observe_outer`/`observe_deployment`, `namespace_digest`, `with_local_runtime_scope`.
- `crates/pse-buildinfo/{build.rs,src/lib.rs}`.
- `crates/pse-runtime/src/math/portable.rs` from receiving admission through reconstruction, plus `describe_body`/`publish_description`.
- `crates/pse-runtime/src/workflow/modeling/preparation.rs`: `prepare_selected`, `settle_preparation`.
- `workflow/modeling.rs::ModelingPackage::prepare` and `modeling/cases.rs::resolve_case`, where the second prepare happens.
- `math/retention.rs::ModelingCache::put`.
- `workflow/native_deployment.rs`, `workflow/worker.rs::ensure_managed_primary`, `workflow/study_execution.rs::study_durable` and `workflow/canonical.rs`.
- `crates/pse-py/src/workflow.rs`: `NativeRuntime::new`, `local_python_configuration`.
- `xtask/src/bin/{pse-worker.rs,support/deployment.rs}`.
- `scripts/surreal_server.py`: `ensure_primary`, `_ensure_primary`, `checked_primary`, `recorded_primary`, `verify_generation`, `primary_ready`, `_primary_process`, `primary_environment`, `publish_generation`.
- `scripts/native_cache.py::{valid,prepare}` and `scripts/validation_scope.py` input scopes.
- `crates/pse-operations/src/canonical_execution.rs::{hash,result_payload_digest,validate_result_batch,append_execution_batch}`, `canonical_results.rs::validate_result_block`, `canonical_selection.rs::{ProductDescription::prepare,matches_read,acknowledge_description_until}` and `canonical_staging.rs::product_blob`.
- `crates/pse-ids/src/derive.rs::FramedHasher`.
- `crates/pse-structural/src/flowsheet.rs`: FlowProjectionV2 framing.
- `crates/pse-modeling/src/expression/admission.rs::ExpressionOccurrence`.
- `crates/pse-backend-native/src/{callback.rs::CallbackState::evaluate,tnlp.rs::intermediate_callback,solve.rs::Progress::push}` and `workflow/durable.rs::Tap`.
- The tracing span inventory.
- `docs/authoritative_design/sections/mathematics-and-compilation.md`, the "Deployment-local replay" paragraph.

**Leads read:** PA01–PA04 and their evidence document; PE05/PE06; 10-09 F02/F05; Plan 33 EI05; graph-review U01; Plan 28i Outcome.

**Not examined or only sampled:**

- `xtask/src/producer_identity.rs`, which is build-time capture and cold.
- The engine session assurance in `pse-engine/session/assurance.rs`, which is F's.
- Arrow field validation on reopen (`FieldCheckedBatch::admit_owned`), which is F's.
- The `with_declarations`/`full_source` per-row hashing internals, which are A's.
- The whole `surreal_server.py` lifecycle beyond the ensure-primary ready path.
- Python test fixtures beyond `canonical_fixture.py` and the conftest fixture scopes.

**Effects:** no builds, tests or services were run. The only measurements are file sizes (`ls`) and the host's CPU flags.

## 2. Complete operations examined

**(a) Python `pse.Runtime(...)` construction.**

1. `pse-py workflow.rs::NativeRuntime::new` → `pse_buildinfo::loaded_module_path` + `/proc/self/maps` association (cheap).
2. `observe_deployment` → `observe_outer`. This reads every file under `crates`, `vendor`, `xtask`, `python` and `scripts` into memory (≈35 MiB at baseline), BLAKE3-hashes them, reads nine config files and spawns `git rev-parse HEAD`. **This runs with the GIL held.**
3. Optional strict producer: `verify_receipt_artifact` → `DeploymentAssociation::verify_current`, SHA-256 of the artifact, consumed files and namespaces, also with the GIL held.
4. Otherwise `MathService::observe_local_runtime` (job-owned, cancellable) → `ReplayAdmission::observe_local_scoped` → under the glibc loader write lock (`with_local_runtime_scope`), `LocalRuntimeObservation::capture_scoped`. This makes:
   - one SHA-256 pass over every executable-mapped ELF (path-deduplicated);
   - a second pass (`verify_scoped` per loaded file) to close the capture;
   - two GIL-acquiring calls to `local_python_configuration`.

   The live state is a `LocalReplay` in `CanonicalDeployment::producer`.

**Measured (ls, 2026-10-11):**

- `python/pse/_native.abi3.so` is 785,556,497 bytes.
- `target/debug/pse-worker` is 726,959,592 bytes.
- The extension's linked solver libraries add roughly 200 MB (MKL core/thread, SCIP, PETSc, Ipopt …).
- The host CPU exposes `sha_ni`, and the pinned `sha2 0.11.0` selects its x86-SHA backend at runtime.

**(b) Warm value edit → re-solve.**

1. `with_declarations` creates a new revision (A's; per-row `edit()` BLAKE3 version keys).
2. `prepare_solve` → `declared_execution` → `ModelingPackage::prepare` → `checked_selection_scoped` → `prepare_selected`.
3. `basis_key` includes `SelectedDependencies`. A new revision's premises differ, so the basis misses and is built fresh with no descriptions.
4. `settle_preparation` therefore finds no matching description for any portable body. It calls `portable::qualify_replay` once per prepare call, then `describe_body` and `publish_description` (one RPC each) per body.
5. `qualify_replay` → `job_scoped` → `ReplayAdmission::qualify_receiving` → `LocalRuntimeObservation::verify_scoped` → a full `capture_scoped` (two passes) under the loader write lock.
6. `cases.rs::resolve_case` calls `self.prepare(...)` again with a demand-extended key. That is a second basis miss and a second `receiving_checked` scope, hence a second qualification.
7. Execution (a.4 in the operation map) adds no host observation.

**(c) Durable `pkg.study(defn)`.** The path is `study_execution.rs::study_durable` → `Runtime::ensure_managed_primary`:

- In-process, and synchronously on a Tokio worker thread: `NativeExecutionSelection::from_store` → `from_state`, which runs `FileObservation::capture` of the supervisor script and the **worker executable**.
- It spawns `python surreal_server.py ensure-primary`. With the primary already ready, that process runs:
  - `checked_primary` → `recorded_primary` → `verify_generation`, which hashes every generation file including `bin/pse-worker`, then `file_digest` of the script and the worker;
  - `primary_environment` (when `PSE_WORKER_BINARY` is set, the worker digest again);
  - `primary_ready` → `checked_primary` again → `_primary_process` → `file_digest(/proc/<pid>/exe)`.
- Back in-process: `selected.require_current()` hashes the worker again.
- Then `start_defined_study` and a 100 ms poll loop.

**(d) Managed worker start.** In `pse-worker.rs::runtime`:

- `from_state` hashes the worker once.
- `deployment::open` → `observe_deployment`. The generation directory lacks `crates/pse-*`, so `workspace_root` fails and it falls back to `FileObservation::capture(artifact)`: worker again.
- `observe_local_runtime`: two passes over the worker and its libraries.
- `require_current`, and `require_current` again in `publish_ready`.

**(e) Study point N+1, in-process or worker.** Two prepare calls per point. With the same revision, the basis hits and `matches_read` succeeds, so each portable body gets only `acknowledge_description_until` (one RPC) and **no host observation**. This holds unless `ModelingCache::put` bypassed retention (pressure demands non-zero, or eviction). In that case the point misses and repeats (b)'s qualification and publication.

**(f) Result block write and reopen.** The payload digest (BLAKE3, ≤512 KiB) is computed:

- in the runtime (`result_projection.rs`);
- in `append_execution_batch`;
- again in `validate_result_batch`;
- again in `validate_result_block`.

Reopen computes it twice per block (`validate_result_block`). Cell, block and output metadata digests form the idempotent request fingerprint.

**(g) Native callback.** Per callback, `CallbackState::evaluate` performs two `catch_unwind`s and `work_admission.admit`/`observe`, each taking the strategy-admission state lock. It records `counts`/`seconds` with `entry(demand.into())`, which allocates a `String` key twice per callback, and calls `Instant::now()`. Each Ipopt/POUNCE iteration allocates a `BTreeMap<String, Metric>` event that a durable `Tap` sends to the progress writer, batched JSON per run.

## 3. Candidate findings

### H-01 — Receiving-runtime validity is re-proved by complete host byte re-observation at each publication/acquisition, not carried for its lifetime (residual PA01)

**Observed cause.** The assurance has no owned validity lifetime between process start and later uses. Two things follow.

*(i) Each qualification repeats the complete host observation.*

- `math/portable.rs::ReplayAdmission::qualify_receiving` calls `pse_buildinfo::LocalRuntimeObservation::verify_scoped`. That re-runs `capture_scoped` over every executable mapping. `capture_scoped` itself hashes each file twice: once in `observe`, then again in the closing `file.verify_scoped` loop.
- So each qualification is two SHA-256 passes over the whole loaded image. In a dev-profile Python process that is roughly 1 GB or more per pass (≈785 MB extension plus ≈200 MB linked solver libraries, plus Python/NumPy/PyArrow modules).
- All of it runs inside `with_local_runtime_scope`, i.e. under glibc's `dl_load_write_lock`. The exclusion therefore lasts as long as the hashing.

*(ii) Qualification is triggered by cache-shaped events, not by a change in runtime premises.*

- `preparation.rs::prepare_selected`/`settle_preparation` hold `receiving_checked` only per prepare call.
- `ModelingPackage::prepare` runs twice per `prepare_solve` (`declared_execution`, then `cases.rs::resolve_case`); dynamics/implicit/fitting call it more often.
- `ProductDescription::matches_read` binds every description to the entire read's premise sequence. Any new revision therefore invalidates every body's description, even bodies unaffected by the edit.
- Descriptions survive only inside a retained `PreparedBasis`. `ModelingCache::put` silently bypasses retention under pressure demands, so a study under memory pressure repeats qualification per point.

The contract text drives this: `mathematics-and-compilation.md` says "absent acknowledgment requires current admission before a new eligible publication", and the implementation reads "current admission" as complete re-observation.

**Consequence.**

- *Interactive edit → re-solve (Python, default local producer):* each value edit can trigger two full receiving observations, i.e. four full-image SHA passes under loader exclusion, before the solve starts. This is the hashing shape of the 2026-10-08 incident, moved from "per body" to "per preparation call".
- *Sixteen concurrent cases / studies at the memory limit (PSE-S07):* basis retention is bypassed exactly when memory is scarce, so per-point qualification reappears at the worst moment. Loader exclusion also serializes any concurrent `dlopen` (e.g. MKL kernel loading, NSS) behind the hashing.
- **Principles:** AP-07/G9, DP-03 ("reuse immutable established validity while its premises hold"), DP-10, H18, H19, H20, H26, H28.

**Simpler conforming alternative (Proposed).** Make the startup observation the owned validity for the process image. Re-establish validity by checking premises cheaply and fall back to full re-observation only when a premise changes:

- executable mapping set from `/proc/self/maps` (already parsed);
- glibc `dlpi_adds`/`dlpi_subs` loader counters (already read by `loader_scope.rs::read_counters`);
- per-file `(dev, ino, size, mtime_ns, ctime_ns)` recorded at capture, with git-index-style "racily clean" handling for files modified at or after capture start;
- the effective configuration bytes;
- `verify_loaded_module`'s inode association.

Done this way, every failure class the current re-hash catches is still caught, except same-inode content mutation with a forged ctime. That class needs privilege, and the documented contract already excludes it ("External/privileged file mutation is outside the existing generation-use contract"; `native_cache.py::seal_generation`). The re-hash also does not cover RTLD_GLOBAL promotion, which the 10-08 platform notes flag as unattested by counters. Hashing again would add no protection there. That premise needs its own decision either way.

Separately, decouple the trigger. A qualification outcome valid for the image does not need `receiving_checked` per prepare call, nor does it need to depend on whether a basis stayed resident.

**Library/platform-owned options:**

- fs-verity (`FS_IOC_ENABLE_VERITY`/`FS_IOC_MEASURE_VERITY`). The kernel enforces immutability and returns the digest in O(1). It needs the ext4 `verity` feature; whether that is enabled on this host is unknown.
- `rustix::fs::statx` for nanosecond timestamps.
- BLAKE3 `update_mmap_rayon` for any remaining full hashes. This is secondary (RC02).

**Preservation constraints:**

- The startup capture keeps establishing the content identity that keys the `LocalReplay` producer.
- Strict `QualifiedProducer` keeps its stronger receipt path.
- Unknown or changed context still refuses replay. Late load or unload still invalidates.
- The configuration observer remains.
- Interruption, drain and charge ownership (PA04 fixes) remain.
- Persisted inputs keep exact payload/dependency/interpretation qualification at acquisition. This finding concerns host observation only.

**Uncertainty.**

- (1) Whether a value edit's second prepare call actually reaches `settle_preparation` without descriptions. Source says yes; probe P1.
- (2) Whether the Python producer is `Some(Local)` in practice. An anonymous executable mapping (MKL JIT, libffi closures) makes `executable_mapping_files` refuse. Then there is no observation cost, but persisted reuse is silently disabled for the process; see probe P4.
- (3) No elapsed-time claim is made.

**Evidence label:** Interface-checked (source-traced). File sizes: Measured (`ls`).

**Suggested verification:** P1 and P4. Controls for unchanged sequential prepares, late `dlopen`, atomic replacement and in-place rewrite of a mapped library, and configuration change.

### H-02 — Managed-primary readiness and worker start repeatedly SHA-256 the same immutable worker executable

**Observed cause.** Every crossing between the observer, the Python supervisor and the worker independently re-proves "this is the admitted executable" by complete content hashing.

Per `pkg.study()` on an already-ready primary, the 727 MB dev `pse-worker` is hashed about 7–8 times:

| Where | Function | Hashes of the worker |
|---|---|---|
| In-process Rust | `native_deployment.rs::NativeExecutionSelection::from_state` | 1 |
| In-process Rust | `require_current` | 1 |
| Supervisor | `surreal_server.py::checked_primary` (`verify_generation`, whose manifest includes `bin/pse-worker`, plus a separate `file_digest(worker)`), called from `_ensure_primary` and again from `primary_ready` | 4 |
| Supervisor | `_primary_process` (`file_digest(/proc/<pid>/exe)`) | 1 |
| Supervisor | `primary_environment` (when `PSE_WORKER_BINARY` is set) | 0–1 |

At worker start (`pse-worker.rs::runtime`/`publish_ready`, `support/deployment.rs::open`) there are six more:

- `from_state`: 1;
- the `observe_deployment` fallback capture: 1;
- the local runtime capture: 2;
- `require_current` twice: 2.

The in-process checks run synchronously inside `async fn ensure_managed_primary`, on an executor thread, through non-interruptible `FileObservation::capture`. Each study call also spawns a Python interpreter that loads a 7,291-line supervisor.

**Consequence.**

- *Interactive edit → small durable study:* roughly 5 GB of SHA-256 plus interpreter start precedes every study, although nothing changed.
- *PSE-S07:* repeated starts multiply the work.
- The uncertainty this protects against enters only at generation admission and at exec:
  - the generation directory is content-addressed (`publish_generation`, identity = SHA of the file manifest) and sealed;
  - Linux refuses writes to a running executable (ETXTBSY), so `/proc/<pid>/exe` cannot change in place while the primary runs.
- **Principles:** H18, H19, H20, H25, H26, AP-07/G9, DP-10, DP-20 (blocking executor thread).

**Simpler conforming alternative (Proposed).**

- Verify generation content once at admission and once when launching a worker from it, which is the real trust transition.
- Record `(dev, ino, size, mtime, ctime)` for the generation files and the launched pid's exe inode.
- Readiness compares those identities (`stat`/`readlink /proc/<pid>/exe` + `fstat`), with no content hashing.
- `NativeExecutionSelection` keeps a stat identity rather than a SHA. If it keeps a SHA, it hashes once per selection, not again in `require_current` within the same call.
- Answering the ready-path query in Rust from the supervisor's readiness record, without spawning Python, is E's lifecycle question; this is flagged as a seam.

**Preservation:**

- Refuse replaced or changed receiver bytes; atomic replacement changes the inode.
- Refuse a pid running another executable.
- Readmission stays required after deliberate change.
- Supervisor-side placement, observer and allocation checks are untouched.

**Uncertainty.**

- Release-profile binaries are smaller.
- Whether `PSE_WORKER_BINARY` is set in ordinary sessions.
- Exact per-call wall time; probe P2.

**Evidence label:** Interface-checked (Rust and Python source-traced); sizes Measured.

**Suggested verification:** P2. Controls: replaced generation file, replaced worker binary at the same path, a pid running a different binary, sealed-mode loss.

### H-03 — Startup assurance and outer provenance are repeated per `Runtime` object instead of once per process image; the outer source digest is weak provenance

**Observed cause.**

- `pse-py workflow.rs::NativeRuntime::new` performs a complete `observe_outer` (whole-checkout read plus `git` subprocess, with the GIL held) and a complete `LocalRuntimeObservation` capture (two image passes) on every `pse.Runtime(...)`.
- `python/pse/tests/canonical_fixture.py::CanonicalFixture.runtime` constructs at least one Runtime per test. The loaded image cannot change between constructions except by detectable loading, so the second and later observations re-prove a property established in the same process (H19).
- The `OuterAttestation.source` digest, recorded in `durable.rs::DurableAttempt::register_selection` (`begin_run` attestation) and `study.rs` study creation, hashes the dirty checkout at construction time. It does not identify the source that built the loaded artifact, which may predate the checkout (H20, DP-21 lineage accuracy).
- The executable identity that is actually relevant is already computed by the local runtime observation.

**Consequence.**

- *PSE-S05 (local Python tests):* every test pays a multi-GB SHA and a whole-tree BLAKE3 before any test-specific work. The 10-08 evidence recorded 20 TB of logical reads over a worker's lifetime under the older per-body protocol; per-test startup is the remaining repeated term.
- Interactive sessions that create runtimes, and multi-runtime processes, repeat it as well.
- Holding the GIL during `observe_outer` blocks other Python threads and delays Ctrl-C handling.
- **Principles:** H12, H19, H20, H26, DP-21.

**Simpler conforming alternative (Proposed).**

- Keep a process-wide owned startup observation (`OnceLock` keyed by anchor/role), revalidated with H-01's cheap premise check.
- Record run provenance as the observed executable identity: the `LocalReplay` identity, or the strict producer key.
- If checkout context is still wanted, record `git` HEAD plus a dirty indicator through git's own stat-cached index (`gix` status, or `git status --porcelain=v2`/`diff-index`), which avoids rereading unchanged files. Compute it lazily and outside the GIL.

**Preservation:**

- Installed artifacts keep an explicit "source unavailable".
- Strict-producer qualification keeps its receipt check.
- No scientific key uses the outer digest; source-confirmed.

**Uncertainty.** Whether the maintainer values the dirty-tree digest for audit (rule RC-H3). Magnitude needs probe P3.

**Evidence label:** Interface-checked; tree size Measured (`find`/`ls`, ≈35 MiB excluding the `.so`).

### H-04 — Per-callback accounting allocates and locks on every function/derivative evaluation (low priority; C's area)

**Observed cause.** `pse-backend-native/src/callback.rs::CallbackState::evaluate` runs per residual/gradient/Jacobian/Hessian callback. Each call:

- takes the strategy work-admission lock twice (`strategy/admission.rs` `admit`/`observe` → `state()`);
- allocates two `String` keys (`counts.entry(demand.into())`, `seconds.entry(demand.into())`) even when the key exists;
- calls `Instant::now()`/`elapsed()`.

Per iteration, `tnlp.rs::intermediate_callback` builds a `BTreeMap<String, Metric>` with about 11 owned keys. The durable `Tap` sends it to persistence for every study point, independent of `controls.history`.

**Consequence.** On the "many small blocks" workload, where a compiled evaluation can take microseconds, fixed per-callback accounting can become a visible share of each evaluation. **Principles:** H26, DP-10 ("keep wide provenance out of inner loops"), DP-21.

**Alternative (Proposed):**

- Static `&'static str` demand keys or a small fixed-index counter array.
- Batched admission per iteration where the budget contract allows: admit an evaluation quota and settle at iteration end.
- Keep `catch_unwind` and the `stopped()` checks.
- Per-iteration metrics are already compact; event persistence demand is D/E's.

**Preservation:** hard work caps must stop at the same bound (H11). Unknown-work spans and typed termination stay unchanged.

**Uncertainty:** magnitude is entirely unmeasured. It could be negligible next to evaluator cost; probe P5.

**Evidence label:** Interface-checked.

### H-05 — Identity framing repeats whole-read dependency encoding and digests of identical bytes inside one process (low priority; overlaps PA03 (B) and E)

**Observed cause.** Per portable body per publication:

- `canonical_selection.rs::ProductDescription::prepare` serializes the whole read's dependency premises twice (`dependency_bytes` counter, `serde_json::to_vec(&BorrowedDependencies(read))`) and clones them once (`snapshot_dependencies`).
- `canonical_staging.rs::product_blob` copies payload plus dependencies and hashes the same blob twice under two frames (`version`, `digest`).
- `portable.rs::describe_body` hashes request + producer + payload.
- `ProductDescription::publication` hashes payload + dependencies again at publish.

Result blocks are digested four times on write (`result_projection.rs`, `append_execution_batch`, `validate_result_batch`, `validate_result_block`) and twice on reopen. No trust boundary separates these computations; they establish the same representation equality (H20).

**Consequence.** The dependency premises are identical for every body in one read, so per-edit publication work grows with bodies × closure premises. That is extension scenario PSE-S01, where package size grows. The block digests are small next to RPCs.

**Alternative (Proposed):**

- Encode a read's dependency premises once into a shared `Arc<[u8]>`, together with their digest.
- Carry a verified digest through the type (e.g. a `VerifiedBatch` that `validate_result_block` compares by field equality).
- Keep exactly one digest verification at the storage read boundary, where corruption can enter.

**Preservation:** storage-boundary integrity verification, idempotent request fingerprints, and exact dependency qualification per consumer read.

**Uncertainty:** premise-set sizes for realistic packages.

**Evidence label:** Interface-checked.

## 4. Opportunities and uncertainties that are not findings

- **Write admission per mutation.** `canonical.rs::CanonicalStore::ensure_writes` → `WriteAdmission::read` reads and parses `config.json` on every mutating call. The cost is small next to the RPC, so this is not an efficiency finding. Enforcement is client-side only; no server-side write-admission check appears in `generated/surreal.surql`. Placement is E's/Plan 35's concern.
- **Dead setting.** `budget.rs::ResourceBudget::hashing_may_use_pool` is a public setting with no consumer (`update_rayon` is unused). This is minor DP-16 machinery: either wire it to H-01/H-02's remaining hashes or delete it.
- **RC02 (fast durable hashing).** Swapping SHA-256 for BLAKE3 in `FileObservation` would speed the remaining hashing but not remove it (H28). The managed receiver association is also compared with Python `hashlib` SHA-256 values. It stays secondary to H-01/H-02.
- **GIL coupling.** `local_python_configuration` attaches the GIL from a math job thread twice per qualification. This is acceptable while the bridge releases the GIL in slices, but it couples preparation latency to application GIL holders. It disappears with H-01's remedy.
- **Lazy executable loading (uncertainty).** MKL loads CPU-specific kernels (`libmkl_avx2`/`def`) lazily. If any executable mapping appears after the startup capture, every later `qualify_receiving` returns `None`, and the process silently publishes under an unqualified producer and never reuses persisted products. This is correctness-safe but would defeat reuse; probe P4.
- **Seams to A.** `with_declarations` rehydrates, copies and re-hashes every unchanged data document and source object per edit, through `full_source` and `persist` → `edit()`. A also owns the second `prepare` per `prepare_solve`.
- **Durable per-iteration progress.** Durable progress persists every iteration event for every study point regardless of consumer demand (H6). The DTO is compact and excludes primal vectors. This is D's/E's.

## 5. Strengths and preservation constraints observed

- **Semantic identity hashing.** One BLAKE3 framed hasher (`pse_ids::FramedHasher`, derive-key contexts, length-framed parts) serves all semantic identities; no SHA appears on scientific paths. **FlowProjectionV2** (`pse-structural/src/flowsheet.rs`) frames incrementally while it builds, with counts and role tags.
- **Positional occurrence identity.** `ExpressionOccurrence::{Node{body,position}}` replaced the subtree rendering and hashing.
- **Cache keys.** `BasisKey` uses an FxHash prehash with exact field equality.
- **Recipe encoding cache.** `encoded_body` keeps encodings by semantic identity, so recipes are not re-encoded.
- **No host observation on acknowledged reuse.** Exact already-rooted acknowledgment preserves provenance without host observation. Unchanged-revision study points do no receiving observation.
- **PA04 fixes.** `capture_scoped` checks a stop/deadline checkpoint per 64 KiB chunk. `qualify_replay`/`observe_local_runtime` are completion-owned jobs that cancel and drain. Exact-path deduplication within a capture. The per-body cohort protocol is deleted.
- **PS-10 assessment.** The original-model assessment is an independent post-solve check and must remain independent.
- **Bounded progress DTO.** It excludes primal vectors. Tracing spans are coarse: construction, assembly, structural analysis.
- **Native cache.** `native_cache.py::prepare` verifies once per native operation, hashes outside the publication lock, and seals generations read-only.
- **Validation scope.** `validation_scope.py` input scopes now include `scripts`, `benches` and native selectors (version 3).

## 6. Positions on prior leads

| Lead | Position | Evidence |
|---|---|---|
| PA01 | **Residual / moved** | Per-body cohort removed. Full re-observation now runs per prepare call needing publication or acquisition (×2 per `prepare_solve`), per Runtime construction, and per point under cache-pressure bypass (H-01, H-03). |
| PA04 | **Resolved for local replay; residual elsewhere** | `capture_scoped`/job drain implemented. `FileObservation::capture` (unscoped) in `NativeExecutionSelection::{from_state,require_current}`, `observe_deployment` and `verify_receipt_artifact` remains non-interruptible, partly on an executor thread or with the GIL held (H-02, H-03). Loader-lock acquisition is honestly documented as non-interruptible. |
| PE05 | **Residual (G owns)** | `native_cache.py::prepare`: remembered per native operation. Full `installation_files` hash on each new operation despite sealed generations. H-01's stat-identity remedy applies. |
| PE06 | **Moved** | `pse-buildinfo/build.rs` no longer watches the tree; complete outer observation moved to every Runtime/worker construction (H-03). |
| 10-09 F02 | **Resolved (coverage)** | `validation_scope.py` `RUST_INPUTS` includes `scripts` and `benches`; product environment includes the UNO/PETSc/SuiteSparse selectors; `INPUT_SCOPE_VERSION = 3`. Applicability cost is G's. |
| 10-09 F05 | **Resolved** | `pse-modeling/src/expression/admission.rs::ExpressionOccurrence` is positional; no subtree text or hash. |
| EI05 | **Confirmed adequate** | Semantic BLAKE3, prehash + exact equality, interner retained; no avoidable work found. |
| Graph U01 | **Resolved** | FlowProjectionV2 adds node/port/connection/binding counts and role tags (ADR-0167). |
| RC02 | **Confirmed unselected; secondary** | Remaining SHA cost is in host/deployment observation. Removing repetitions (H-01/H-02) comes first. |
| 2026-10-08 incident question | **Answered** | At baseline, whole-image hashing runs per Runtime, per prepare call after a revision change (×2), per point only under basis-retention bypass, and per durable study call (worker executable ×7–8). |

## 7. Candidate rule impacts

- **RC-H1.** `mathematics-and-compilation.md` "Deployment-local replay" paragraph and ADR-0164: "absent acknowledgment requires current admission before a new eligible publication", read as complete re-observation.
  - *Proposed:* receiving validity is owned per process image. "Current admission" is a premise check against the startup observation (mapping set, loader counters, file stat identity, configuration), with full re-observation on change.
  - *Depends:* H-01, H-03.
  - *If kept:* H-01 reduces to trigger reductions (one qualification per edit operation, decoupled from basis residency).
- **RC-H2.** `NativeExecutionSelection::require_current` doc and the `surreal_server.py` receiver checks ("refuse … replaced executable bytes before reporting readiness").
  - *Proposed:* content verification at generation admission and exec; stat/inode identity at readiness.
  - *Depends:* H-02.
  - *If kept:* deduplicate to one verification per ensure-primary call.
- **RC-H3.** ADR-0164: "complete dirty outer observation remains at deployment/run admission".
  - *Proposed:* once per process, or a git stat-cache-derived checkout state, with the executable identity as the run provenance.
  - *Depends:* H-03.
  - *If kept:* memoize per process and move it outside the GIL.

## 8. Proposed light probes

All run under `scripts/pse-env --resource-class light --` against a disposable database, as
`canonical_fixture.py` does. `$S` denotes a scratch directory outside the repository.

**P1 — Receiving observations per value edit.**

- *Question:* does a warm value edit followed by `prepare_solve` perform 0, 1 or 2 receiving observations?
- *Script:* `$S/p1.py` constructs a Runtime and package, then runs solve → `with_declarations` (one numeric edit) → solve. It drops markers with `os.open("/nonexistent/pse-probe-<phase>")`.
- *Command:* `strace -f -qq -e trace=openat -o $S/p1.strace .venv/bin/python -I $S/p1.py`, then count `_native.abi3.so` opens between markers.
- *Discriminating outcome:* each observation opens the extension three times (ELF-magic check, capture, end verification). After the edit marker, 0 refutes H-01's edit-path trigger; 3 means one observation; 6 means two.

**P2 — Worker hashing per durable study call.**

- *Question:* how much hashing does `pkg.study()` do against an already-ready managed primary?
- *Command:* `strace -f -qq -e trace=openat,execve -o $S/p2.strace .venv/bin/python -I $S/p2.py`, then count opens of the generation's `bin/pse-worker` and `/proc/*/exe`, and time `study()`.
- *Discriminating outcome:* five or more worker opens confirms H-02. One or none refutes it.

**P3 — Runtime construction cost attribution.**

- *Command:* `perf record -e cycles:u -F 199 -g -o $S/p3.data -- .venv/bin/python -I -c "<construct pse.Runtime twice>"`, then `perf report --no-children --sort symbol`.
- *Discriminating outcome:* `sha2::…::compress` + `blake3` dominating, with a comparable share on the second construction, confirms H-03. A negligible share refutes it.

**P4 — Late executable mappings.**

- *Method:* in one Python process, diff the executable-mapping path set from `/proc/self/maps` after `pse.Runtime(...)` and again after the first solve.
- *Discriminating outcome:* any new executable mapping means later `qualify_receiving` returns `None` (silent loss of local reuse/publication eligibility). An empty diff clears this uncertainty.

**P5 — Per-callback accounting share (optional).**

- *Method:* `perf record` on a many-small-block NLP fixture solve.
- *Discriminating outcome:* a share of `CallbackState::evaluate` + allocation/`Mutex` above about 5% of evaluation callbacks makes H-04 material. Otherwise drop H-04.
