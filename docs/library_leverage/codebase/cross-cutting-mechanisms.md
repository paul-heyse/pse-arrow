# Cross-cutting resource and I/O mechanisms

Opportunities whose manifestations span several crates and do not belong to one subsystem: prepaid
admission and retained-size accounting, durable private files and publication, output budgets, and process
identity and supervision. The baseline and coverage are on the [entry page](README.md). Contracts are in §14.3, §20.6
and §23.1.

**System picture.** Memory and work are admitted before allocation, files are published whole, and child
processes are drained before release. These behaviours are deliberate safety contracts born of host-wide
OOM incidents and need first-class owners. Already strong:
- Several suppliers' own estimates are consumed where they exist (faer scratch queries, a FERAL work
  estimate, supplier-owned LU extents, IBEX workspace bytes), and some counts stream through a counting
  sink instead of a formula.
- `pse_model::allocation::btree_allocation_bound` is the one owner of std B-tree geometry for its callers,
  and `HeapUsage` is generated for most relation types.
- `pse-ids/src/sink.rs::BoundedSink` with a typed `SinkFailure` is the shared bounded writer.
- `SupervisedChild` (`pse-diagnostics`) is the shared child owner, and the host manager owns delegated
  cgroup subgroups with `cgroup.kill`.

Opportunities cluster where the same obligation is restated per library, per module or per consumer.

**Refresh coverage, 2026-11-02.** The coordinator reread the result-selection extents, shared bounded
sink and its tests, retained modeling-container accounting, host invocation publication and private
readers, native-pointer publication, telemetry publication, process-facts and supervised-child ownership,
and representative residual writers. The broader math/columnar/native footprint below inherits the
2026-10-30 inspection: its production source is unchanged at the new baseline, but it was not all reread.
No complete allocation/writer census or current kernel/systemd probe was performed. Changes to host
lifecycle do not by themselves discharge these cross-cutting obligations; the host-specific opportunities
are on [Host and invocation](host-and-invocation.md).

<a id="cbo-001"></a>

## CBO-001 — Prepaid prediction mirrors library internals

- **Operation and consumers.** Memory and work are admitted before allocation, and refusal is typed and
  recoverable. Consumers are result reads and publication, compiler and math preparation, structural
  analysis, fitting and dynamics sessions.
- **Current mechanism.** Each owner forecasts the library it is about to call, with multipliers that cite
  versions:
  - Arrow and DataFusion: `result_selection.rs` (`selection_window_extent`, `partition_working_extent`,
    `pruning_working_extent`), `pse-columnar` `allocation_extent.rs` and `ipc_file.rs` `writer_extent`,
    `pse-relations` `selected_take_extent`.
  - Symbolica: `pse-math` `library/admission.rs`, `reverse.rs`, `execution.rs`, and
    `native_compiled.rs::export_bound`.
  - Native solvers: SUNDIALS formulas in `pse-backend-native` (`linear_bytes`, `bbd_bytes`), fitting
    sampling, residual and surrogate sizes, and `rank_admission`.
  - Graph and std layouts: `pse-structural/dense.rs`, `semantic_extent.rs`, parser extents.
  - Workflows: literal multipliers beside reservations; the prior count was a search-shape estimate,
    not a current census.
- **Why it matters.** A pin move can invalidate a mirror. The freshly read selection formulas explicitly
  name Arrow 59.3 and DataFusion 55.1 and their allocation shapes; checked arithmetic tests protect overflow,
  not agreement with future supplier layouts. Some Symbolica paths reconcile after the fact. No general
  cross-supplier drift detector was established. Conservative factors can refuse feasible work (*Proposed*).
- **Patterns.** LOP-X001 (primary), LOP-03, LOP-09, LOP-10.
- **Evidence.** Fresh `workflow/result_selection.rs::{selection_window_extent,partition_working_extent,
  pruning_working_extent,trajectory_index_extent}` and its overflow test; other named functions retain
  the unchanged-source 2026-10-30 inspection.
- **Footprint.** Several crates share the obligation; remaining workflow multipliers were not exhaustively inspected.
- **Preservation constraints.** The real constraint is prepay. A prediction must exist before allocation
  and refuse deterministically, in the conservative direction, with a typed error. Counting must not
  allocate without bound. One pool identity (DataFusion `MemoryPool` through `AllocationLease`). No OS RSS
  claim.
- **Counterevidence and uncertainty.** Post-hoc measurement cannot discharge prepay: Arrow `get_array_memory_size`, `Bytes::claim` (its `pool.reserve` has no refusal path), Salsa `memory_usage`. An
  accounting allocator fires at allocation, where Rust failure aborts. None exists in workspace crates, and
  foreign heaps (GMP limbs, C libraries) are invisible. Symbolica mirrors already predict, then reconcile
  with typed refusal, but weighted and transpose products have no source-only envelope
  (`compilation_allocation_bound` returns `None`). Loose bounds can be the contract. Retained measurement is
  [CBO-086](cross-cutting-mechanisms.md#cbo-086).
- **Status.** Current promise (prepay); supplier-side prediction Proposed.
- **Research questions.** Which suppliers publish a symbolic-phase size or accept a budget or fallible hook? Could operators charging `MemoryReservation::try_grow` confine formulas to true prepay points? Could the pinned `allocation-counter` dev-dependency, today used for zero-allocation assertions in `pse-backend-native` and `pse-diagnostics`, also detect drift without owning admission?


<a id="cbo-003"></a>

## CBO-003 — Owned private files and durable publication have no shared owner

- **Operation and consumers.** Establish that a path is owned, private, symlink-free and bounded on read,
  then publish content whole without clobbering, with a stated durability. Consumers are host records,
  locks and generations, result export, review packs, capture handoff, build identity and the canonical
  file store.
- **Current mechanism.** Each module carries its own helpers. Host private-directory, bounded-read and
  publication helpers recur with different rules: some `chmod` an existing
  directory to 0700, `study_module` refuses, others accept anything without group or other write bits.
  Parent fsync follows publication in `server` and `telemetry_service` and not in `native::publish_pointer`,
  export or capture acknowledgement. Routes differ: `hard_link` plus `remove_file`, `O_TMPFILE` plus
  `linkat`, `persist_noclobber`, `RENAME_NOREPLACE`, `NamedTempFile::persist` (overwriting), and
  `create_new` alone. `openat2` appears once (`server/release_import.rs`); elsewhere symlink refusal is a
  path walk before a later open (`diagnostic_tools::no_symlinks`). The canonical file store adds its own
  fsync ordering, verifying read and a polled `try_lock` instance lock.
- **Why it matters.** Strictness, durability and race handling are separate decisions per site. A shared
  capability might remove repeated decisions while keeping explicit durability classes (Proposed).
- **Patterns.** LOP-09, LOP-X008, LOP-X011, LOP-01.
- **Evidence.** Fresh `invocation::{directory,evidence_file,read_metadata,publish}` distinguishes passive
  inspection from namespace initialization and syncs a no-replace publication; `telemetry_service::write_atomic`
  syncs file and parent, while `native::publish_pointer` syncs its file then replaces the pointer without
  a parent sync. Capture handoff retains explicit `.pending` staging. Store/export evidence inherits
  the unchanged 2026-10-30 inspection.
- **Footprint.** Representative host, capture, export and store boundaries; no current claim that every old
  generation/bootstrap helper survives the completed host changes.
- **Preservation constraints.** Interrupted `.incomplete` and `.pending` residue is promised (§23.1), so
  RAII temporaries would remove it. No-clobber winner detection, 0400 evidence, non-UTF-8 paths, size
  checks before decode, and passive readers that never change modes. Sealed-store crash order (data
  fsync, link, directory fsync, catalog acknowledgement) stays with its owner.
- **Counterevidence and uncertainty.** Durability levels may differ on purpose (evidence versus config; a
  once-a-second diagnostic). Export may promise process-crash safety only. `release_import` is already
  fd-first, and `object_store::LocalFileSystem` does not fsync directories.
- **Status.** Current mechanism.
- **Research questions.** Can a descriptor-relative layer (`openat2`, `cap-std`, `tempfile`) express
  "owned, private, no symlink at any component, bounded" with non-UTF-8 and no-replace? Which durability
  classes exist?

<a id="cbo-086"></a>

## CBO-086 — Retained-size walkers maintained by hand across the product graph

- **Operation and consumers.** Report the bytes an owner holds, so retention is charged to the right pool
  and released with its last reader. Consumers are prepared math bodies and plans, solve reports,
  compiler state and retention snapshots. This is measurement of built values; prediction is
  [CBO-001](cross-cutting-mechanisms.md#cbo-001).
- **Current mechanism.** Generated `pse_model::HeapUsage`, including B-tree accounting, coexists with
  hand-written retained-size visitors. The old implementation counts are not refreshed measurements:
  - `pse-math`: `stage_retained_bytes` (pointer-keyed dedupe), `allocation_components`,
    `Constant::allocated_bytes`, and `numerics/retained.rs`, which lists fields by hand.
  - `pse-backend-native`: three `Extent` visitors (`solve/retention.rs`, `solve/record.rs`,
    `solve/snapshot.rs`), one encoding std B-tree node constants.
  - `pse-modeling/src/extent.rs`, `State::known_bytes`, Salsa `heap_size` callbacks, and
    `observe_retention` over `memory_usage`.
  - `pse-runtime` `math/products.rs::own_preparation`, which splits a lease per compiled field with
    literal kinds 10 to 21.
- **Why it matters.** A field added to a retained type needs an unaided edit, and a miss under-charges
  silently. `own_preparation` rebuilds sharing that `Arc` already expresses (Proposed).
- **Patterns.** LOP-02, LOP-X001, LOP-03, LOP-09.
- **Evidence.** Fresh `pse-modeling/src/extent.rs` uses capacity-aware recursive accounting and conservative
  map allowances; its module explicitly allows repeated counting of shared values. Other named walkers
  retain their unchanged-source 2026-10-30 inspection, some read only in part.
- **Footprint.** Remaining walkers and `shared_product` located by search only.
- **Preservation constraints.** Capacity, not length. Sharing semantics differ: some owners deliberately
  deduplicate `Arc` payloads, while conservative modeling estimates allow repeated counting.
  Owner wrapper charges separate from payload, foreign payload through library APIs, conservative
  direction, typed overflow, and no claim over foreign heaps.
- **Counterevidence and uncertainty.** Symbolica evaluator internals are private, so a derive stops there.
  Exhaustive destructuring already makes a new field a compile error in some visitors. B-tree geometry
  remains a mirror inside any derive, which can undercount unless each container mapping is conservative.
  No drift was shown.
- **Status.** Current mechanism.
- **Research questions.** Do `get-size2`, `deepsize`, `datasize` or `allocative` handle capacity, `Arc`
  dedupe and custom leaf impls? Does Salsa's `heap_size` hook compose with one? Could the existing
  generator emit the walkers? Could Arrow's `claim` give buffer truth?

<a id="cbo-004"></a>

## CBO-004 — Process identity and supervision: name-based scans beside owned handles

- **Operation and consumers.** Prove that a process is the one recorded, and that its descendants drained,
  before releasing a caller or recovering a record. Consumers are compiler children, capture launches,
  the host manager and admission, payload leaders and recovery.
- **Current mechanism.** `SupervisedChild` holds a pidfd and proves drain, but enumerates group members by
  scanning `/proc`. The host owns a better primitive for whole workloads: delegated cgroup subgroups with
  `cgroup.kill` and population (`manager/subgroup/payload`). Durable identity is (boot, namespace, pid,
  start ticks) with compare-before and compare-after reads (`manager_process`, `current_payload`), a
  separate `/proc/<pid>/exe` read, and a parallel identity type in `canonical_files`. Direct children
  outside `SupervisedChild` remain in `native_child`, `capture/cpu.rs` and host `Command::status` sites.
- **Why it matters.** A rule added to one mechanism does not reach the other, and `/proc` re-reads guard
  races a handle would remove (Proposed).
- **Patterns.** LOP-X008 (primary), LOP-09, LOP-08.
- **Evidence.** Fresh `supervised_child.rs::{members,drain}`, `process_facts.rs` coherent identity reads,
  `managed/supervisor/leader.rs` executable comparison and `canonical_files.rs::ProcessIdentity`.
  Kernel/systemd observations belong to the earlier audit, not a current environment check.
- **Footprint.** `manager/subgroup` was read partly. Identity also travels in re-exec records.
- **Preservation constraints.** A nested PID namespace refuses. Serialized identity records affect recovery
  of live older records. Absence stays distinct from change. An unproved drain never releases the caller
  (`DrainFailurePolicy`: the compiler parks, capture refuses after an allowance). `pse-diagnostics` stays
  independent of the manager.
- **Counterevidence and uncertainty.** Boot, namespace and start ticks already distinguish absence,
  unknown identity and changed identity. Supervision holds pidfds for discovered members and keeps the
  leader unreaped; this is not merely signalling numeric PIDs. Host children run inside an authenticated scope.
  A probe recorded on 2026-10-27 (evidence kept, not re-run) showed that delegated leaf containment works in a `Delegate=yes` user scope but the
  user manager sets no `user.delegate` xattr and `+memory` cannot be enabled while the supervisor sits in
  the scope root. `native_child` needs no group drain.
- **Status.** Current mechanism.
- **Research questions.** Can pidfs identifiers or file handles replace start-tick comparison, through
  `rustix`? Which mechanism should a new child use, and can `SupervisedChild` use kernel population
  without a manager?

<a id="cbo-002"></a>

## CBO-002 — Residual bounded writers beside the shared typed sink

- **Operation and consumers.** Bound or measure serialized output (JSON documents, Arrow IPC, exports,
  progress payloads) while it is written, with a typed refusal at the limit.
- **Current mechanism.** `BoundedSink` already supports retained, forwarded and counting destinations,
  cancellation/deadline checkpoints, checked counts and typed failures through serde/Arrow wrappers.
  Three freshly read counterparts still impose their own byte allowance: `observation::BoundedImage`,
  `connected_results::BoundedExportFile` (with cancellation) and capture `handoff`'s `Limited` (with expiry).
  `study_variants::document_extent` separately counts, bounds and classifies exhaustion. Other writers
  hash, translate or frame output; their existence is not duplication of this operation.
- **Why it matters.** These specific counterparts repeat an already-owned cap/checkpoint contract and
  recover exhaustion through local state or text errors. Reuse could reduce separate error and partial-write
  decisions (Proposed); no incorrect current result or performance loss was demonstrated.
- **Patterns.** LOP-09, LOP-01.
- **Evidence.** Fresh shared sink production code and typed-error tests, plus the four counterparts above.
  Trial encoding/splitting remains a separate question in `result_blocks` and `result_files`.
- **Footprint.** A classified sample, not a census of every `Write` implementation.
- **Preservation constraints.** Limit errors stay distinct from I/O errors. Some modes count or hash
  without a cap, and durable block boundaries split deterministically.
- **Counterevidence and uncertainty.** The local sink carries repository-specific typed recovery and
  checkpoints; an upstream adapter need not improve it. `BoundedExportFile` exposes remaining/written
  counters and uses `u64`, while the shared sink uses `usize`; these consumer contracts need examination.
  The observational writer's simple failure path may justify retaining a tiny implementation.
- **Status.** Current mechanism; reuse benefit Proposed.
- **Research questions.** Can the existing sink express each counterpart's counter, expiry and error-mapping
  contract without additional policy? Could supplier size planning avoid trial encoding without changing
  deterministic durable block boundaries?
