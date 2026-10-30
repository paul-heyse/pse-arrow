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

**Coverage on this page.** Prepay accounting was read in `pse-math` (`library/admission`, `reverse`,
`native_compiled`, parts of `execution`), `pse-runtime` `result_selection` in full and `math/` admission
paths, `pse-columnar` `allocation_extent`, `owned_buffer` and `ipc_file`, `pse-relations` `columnar`,
and `pse-backend-native` dynamics layout. Unread: the `workflow` multipliers beyond a sample, the
`pse-modeling` `workspace` bound methods, and no census of the hand-written walkers was taken. File
publication was read in the host (`invocation`, `server`, `telemetry_service`, `native`, `admission`,
`product_reader`), runtime export, pack and capture code, and the canonical store write and verify paths.
`study_module::exact_path`, `admission::read_private_json`, `generation/bootstrap` and `local_claim` were
skimmed. Process supervision was read in `SupervisedChild`, host `manager`, `ancestry` and `leader`.

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
  - Workflows: literal multipliers beside about 73 `reserve` calls.
- **Why it matters.** A pin move can invalidate a mirror, and there is no general drift detector. Some Symbolica paths reconcile after the fact, and no equivalent was found on the Arrow and DataFusion formulas (a regular-expression search, so weak evidence). A Salsa citation already lags its pin. Conservative factors can refuse feasible work (*Proposed*).
- **Patterns.** LOP-X001 (primary), LOP-03, LOP-09, LOP-10.
- **Evidence.** The functions above were read; counts are `rg` shape indicators.
- **Footprint.** About two dozen sites in seven crates inspected; remaining workflow multipliers unread.
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
  publication helpers recur in about ten modules with different rules: some `chmod` an existing
  directory to 0700, `study_module` refuses, others accept anything without group or other write bits.
  Parent fsync follows publication in `server` and `telemetry_service` and not in `native::publish_pointer`,
  export or capture acknowledgement. Routes differ: `hard_link` plus `remove_file`, `O_TMPFILE` plus
  `linkat`, `persist_noclobber`, `RENAME_NOREPLACE`, `NamedTempFile::persist` (overwriting), and
  `create_new` alone. `openat2` appears once (`server/release_import.rs`); elsewhere symlink refusal is a
  path walk before a later open (`diagnostic_tools::no_symlinks`). The canonical file store adds its own
  fsync ordering, verifying read and a polled `try_lock` instance lock.
- **Why it matters.** Strictness, durability and race handling are separate decisions per site, and a fix
  has dozens of sites (Proposed). Export writes block inside an async function.
- **Patterns.** LOP-09, LOP-X008, LOP-X011, LOP-01.
- **Evidence.** Read: `invocation::publish`, `server`, `telemetry_service::write_atomic`, `native`,
  `product_reader`, `export_ipc_with_prefix`, `review_pack`, `capture/{recording,handoff}`,
  `canonical_files` and `offline`.
- **Footprint.** Located only: `generation/bootstrap`, `local_claim`, `native_service` private helpers.
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
- **Current mechanism.** `pse_model::HeapUsage` has about 835 impls in about 230 files, mostly in generated
  paths, with B-tree impls. Beside it are about 206 hand-written non-generated `fn retained_bytes`:
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
- **Evidence.** `stage_retained_bytes`, `numerics/retained.rs`, `solve/retention.rs`, `extent.rs`,
  `known_bytes`, `observe_retention` and `own_preparation`, some read in part.
- **Footprint.** Remaining walkers and `shared_product` located by search only.
- **Preservation constraints.** Capacity, not length. Shared `Arc` values counted once, deliberately.
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
- **Evidence.** `supervised_child.rs`, `ancestry::ancestors`, `validate_live_controller`,
  `leader::current_payload`, `manager_process` and `process_facts` were read. Kernel 7.0 and systemd 255
  were checked.
- **Footprint.** `manager/subgroup` was read partly. Identity also travels in re-exec records.
- **Preservation constraints.** A nested PID namespace refuses. Serialized identity records affect recovery
  of live older records. Absence stays distinct from change. An unproved drain never releases the caller
  (`DrainFailurePolicy`: the compiler parks, capture refuses after an allowance). `pse-diagnostics` stays
  independent of the manager.
- **Counterevidence and uncertainty.** Boot, namespace and start ticks are already sound (`pid_max`
  4194304, `CLK_TCK` 100), so the gain is modest. Host children run inside an authenticated scope.
  A probe recorded on 2026-10-27 (evidence kept, not re-run) showed that delegated leaf containment works in a `Delegate=yes` user scope but the
  user manager sets no `user.delegate` xattr and `+memory` cannot be enabled while the supervisor sits in
  the scope root. `native_child` needs no group drain.
- **Status.** Current mechanism.
- **Research questions.** Can pidfs identifiers or file handles replace start-tick comparison, through
  `rustix`? Which mechanism should a new child use, and can `SupervisedChild` use kernel population
  without a manager?

<a id="cbo-002"></a>

## CBO-002 — Residual bounded and counting sinks (weak)

- **Operation and consumers.** Bound or measure serialized output (JSON documents, Arrow IPC, exports,
  progress payloads) while it is written, with a typed refusal at the limit.
- **Current mechanism.** `BoundedSink` and `SinkFailure` (`pse-ids/src/sink.rs`) are used in about eight
  files across runtime, columnar, operations and math, and `result_blocks::can_split` matches
  `SinkFailure::LimitReached` by type. About 30 non-test `impl Write` sinks remain (a count that depends
  on the pattern), in `pse-runtime` `workflow` and `tooling`, plus halving loops over row ranges in
  `result_blocks::visit_blocks_async` and `result_files::result_prefix_rows`.
- **Why it matters.** Some remaining sinks may restate a cap that `BoundedSink` already owns (Proposed).
- **Patterns.** LOP-09, LOP-01.
- **Evidence.** `sink.rs`, `can_split`, `result_prefix_rows`; the sinks were counted by `rg`.
- **Footprint.** Not classified: how many sinks cap, hash, count or check cancellation.
- **Preservation constraints.** Limit errors stay distinct from I/O errors. Some modes count or hash
  without a cap, and durable block boundaries split deterministically.
- **Counterevidence and uncertainty.** The shared type is local because of cancellation and typed recovery,
  and no library writer adapter carries those. The residue may be mostly non-capping sinks.
- **Status.** Current mechanism. Weak.
- **Research questions.** How many remaining sinks are caps? Could size-targeted chunking avoid trial
  encoding?
