# Cross-cutting resource and I/O mechanisms

Opportunities whose manifestations span several crates and do not belong to one subsystem.
The baseline and coverage are on the [entry page](README.md). Evidence cites a path and symbol
at source baseline `dbc4c9436`.

<a id="cbo-001"></a>

## CBO-001 — Prepaid memory and work accounting mirrors library internals

- **Operation and consumers.**
  - Memory and work are admitted before allocation for:
    - decoded Arrow batches, selection offsets, metadata and encoder scratch;
    - prepared bodies, plans and graphs;
    - sparse factorizations;
    - native solver workspaces.
  - Refusal is a typed, recoverable failure. This deliberate safety contract was born of
    host-wide OOM incidents.
  - Consumers are every result path, compiler preparation, structural analysis and native
    session admission.
- **Current mechanism.**
  - **Hand-derived extents beside each allocation.** These include `*_extent` helpers in
    `pse-runtime/src/workflow/{replay,durable,result_projection,connected_results}.rs`, and
    multipliers such as the selection estimate in `connected_results.rs` (`select_original_rows`)
    and the snapshot estimate in `result_snapshot.rs`.
  - **Mirrors of third-party layouts.**
    - `pse-structural/src/flowsheet.rs` restates the pinned petgraph node layout.
    - `pse-compiler/src/workspace/modeling/executable/implicit.rs` does `size_of::<petgraph::graph::Node>`
      arithmetic.
    - `pse-backend-native/src/kinsol/block.rs` holds a cubic FERAL fill bound.
    - `dynamics/idas/bbd.rs` has `bbd_bytes`.
  - **Two accounting systems in one crate.** Salsa `heap_size=` closures and `memory_usage`
    coexist in `pse-compiler/src/workspace.rs` with about 64 local `allocation_bound` /
    `retained_bytes` sites. `pse-math` has about 110 more.
  - **Native reporters are used where they exist**, for example `KINGetWorkSpace` and
    `SUNLinSol_KLUGetMemUsage`.
- **Why it matters.**
  - One meaning ("bytes this owner will hold") is re-derived by hand in hundreds of places.
  - The multipliers carry no derivation, so they cannot track a library upgrade or a struct
    change.
  - A drifted estimate either wastes admission capacity or under-admits. The second case
    defeats the contract.
- **Patterns.** [LOP-X001](../assessment-patterns.md#lop-x001) (primary), LOP-02, LOP-03.
- **Footprint.**
  - Inspected: about a dozen representative sites across four crates.
  - Suspected: `HeapUsage` is implemented in about 260 files, part of it generated. No census
    was taken.
- **Preservation constraints.**
  - Reservation happens before allocation.
  - Refusal is deterministic and typed.
  - Counting must not itself allocate without bound.
  - One deployment pool identity (the DataFusion `MemoryPool` through `AllocationLease`).
  - Conservative direction of error.
- **Counterevidence.**
  - Library accounting such as `get_array_memory_size` or Salsa `memory_usage` is post-hoc, so it
    cannot replace a prediction.
  - Some estimates are intentionally loose upper bounds on future work.
  - Partial generation already exists for `heap_bytes`.
- **Status.** Current promise (prepay admission). Library or derive support would be a proposed
  refinement.
- **Research questions.**
  - Which libraries expose a symbolic-phase size (faer and FERAL symbolic LU, KLU analyze,
    Arrow `get_slice_memory_size`)?
  - Can a retained-size derive (`get-size`/`deepsize`-style crates, fit unknown) or the existing
    generator cover owned structs?
  - Could DataFusion `MemoryReservation::try_grow` sized from Arrow's own functions replace the
    Arrow-shaped multipliers?
- **Library assessment correction (2026-10-27).** The petgraph "mirror" premise is sharpened: petgraph storage is sized with `size_of` over its public types, so it tracks the real layout. The genuine hand mirrors are std `BTreeMap` node geometry and per-node workspace multipliers, which also appear in `pse-modeling/src/check/phases.rs`. FERAL LDLᵀ already uses the library's own work estimate, so the FERAL mirror is LU-only. `KINGetWorkSpace` is deprecated in SUNDIALS 7.3 and removed in 8.0. `SUNLinSol_KLUGetMemUsage` is a local addition, not upstream. Arrow's `get_slice_memory_size` does not account for nested slices, which justifies the local `visible_size`. See [faer](../libraries/faer/README.md), [feral](../libraries/feral/README.md), [sundials](../libraries/sundials/README.md), [arrow](../libraries/arrow/README.md) and [petgraph](../libraries/petgraph-and-rustworkx-core/README.md).

<a id="cbo-002"></a>

## CBO-002 — Bounded and counting output sinks re-implemented per site

- **Operation and consumers.**
  - Bound or measure serialized output (JSON documents, Arrow IPC, exports, progress
    payloads) while it is written, with a typed refusal when a limit is reached.
  - Where output must be split, trial-encode and halve the row range until each part fits.
- **Current mechanism.**
  - There are 36 non-test `impl io::Write for …` sinks: 20 in `pse-runtime`, the rest spread over
    `pse-columnar`, `pse-ids`, `pse-math`, `pse-engine`, `pse-operations`, `pse-buildinfo` and
    `pse-compiler`. Examples:
    - `pse-columnar/src/ipc_file.rs`
    - `workflow/result_blocks.rs`
    - `result_inspection.rs`
    - `reproducer.rs`
    - `replay.rs`
    - `durable.rs` (two)
    - `connected_results.rs`
    - `study_variants.rs`
  - Halving loops appear in `result_blocks.rs` (`visit_blocks_async`), `result_files.rs`
    (result prefix encoding) and the progress writer in `durable.rs`. One of them recognises
    the limit by matching the error string `"result block limit"`.
- **Why it matters.**
  - One hard output budget is maintained in about 36 forms, with different error shapes.
  - The string-matched discrimination is brittle.
  - A single owned sink, together with a size-targeted chunking helper, would make the policy
    explicit once.
- **Patterns.** LOP-09, LOP-02, LOP-01.
- **Footprint.** All sink sites were located with `rg`. Four halving loops were read.
- **Preservation constraints.**
  - Limit errors must stay distinguishable from I/O errors.
  - Some modes count or hash without capping.
  - Where row boundaries become durable block boundaries, ordinals and digests must be split
    deterministically.
- **Counterevidence.**
  - Several sinks hash or count rather than cap, so they are different responsibilities that
    look alike.
  - The total amount of code is modest.
- **Status.** Proposed addition.
- **Research questions.**
  - Is a library writer adapter (a `Take`-style bounded writer, or `bytes::BufMut::limit`)
    adequate, or is the remedy a local shared type?
  - Can Arrow batch sizes be predicted (`get_array_memory_size`, IPC message-size estimation),
    so that trial encoding is avoided?
- **Library assessment correction (2026-10-27).** The baseline count is 32 `io::Write` plus 6 `fmt::Write` sinks outside tests. At least five of them check cancellation or expiry on each write, and three hash rather than cap. `ArrowError::IoError` keeps the original `io::Error`, so a typed `io::Error::other` plus downcast can replace the string match ([rust-std-io](../libraries/rust-std-io/README.md)).

<a id="cbo-003"></a>

## CBO-003 — Durable file publication and the sealed-file byte layer on `std::fs`

- **Operation and consumers.**
  - Publish a file without exposing partial content and without overwriting an existing file.
  - Persist, verify and recover sealed payload files whose catalog rows live in SurrealDB.
  - Exclude concurrent instances.
  - Consumers are result exports, pack building, capture tooling, build identity and the
    canonical file store with its offline recovery bins.
- **Current mechanism.**
  - Sixteen non-test `hard_link(` publication sites follow the same sequence: `create_new`
    staging, `sync_all`, hard-link to the destination, remove staging. They appear in
    `pse-runtime/src/workflow/connected_results.rs` (IPC export), `tooling/review_pack.rs`,
    `tooling/capture/*`, `study_workbench.rs`, `pse-buildinfo/src/identity.rs` and
    `pse-operations/src/canonical_files*.rs`.
  - The canonical file store (`canonical_files.rs` with `canonical_files/offline.rs`, about
    4.2k lines) adds:
    - its own data and directory fsync ordering;
    - hard-link and alias rejection;
    - a SHA-256 verifying read loop;
    - an instance lock implemented as `File::try_lock` polled with a 5 ms sleep.
  - The workspace already pins `tempfile`, and `object_store` 0.13.2 is in the type universe.
  - Synchronous writes run inside an async export loop.
- **Why it matters.**
  - The generic durable-write layer (atomic publish, fsync discipline, digest-verified reads,
    advisory locking) lives beside the domain protocol that owns the scientific meaning.
  - Cleanup of abandoned staging files on failure or cancellation is decided separately at each
    site.
- **Patterns.** LOP-09, LOP-01, LOP-08.
- **Footprint.** The export path and the file-store write and verify paths were read. The other
  sites were located with `rg`.
- **Preservation constraints.**
  - Crash-consistency order: data fsync, link or rename, directory fsync, then catalog
    acknowledgement.
  - No-clobber publication.
  - Hard-link rejection on recovery.
  - The offline copy-boundary guard.
  - The instance lock must conflict with online `flock` users.
  - Byte-limit and cancellation checkpoints.
- **Counterevidence.**
  - The tests deliberately exercise adversarial filesystem behaviour, which generic
    abstractions may hide.
  - `object_store::LocalFileSystem` does not fsync directories.
  - `hard_link` may have been chosen for its exact no-overwrite semantics.
- **Status.** Current promise (durability). Consolidating it is a proposed addition.
- **Research questions.**
  - Do `tempfile::NamedTempFile::persist_noclobber`, `atomic-write-file` or `cap-std`
    reproduce the full fsync, link and lock order?
  - Can the polling `try_lock` become a blocking lock on a blocking thread?
  - Could reads go through the registered `ObjectStore`?
  - The equivalent Python tooling shape is [CBO-047](developer-tooling.md#cbo-047).
- **Library assessment correction (2026-10-27).** Only 7 production `hard_link(` lines in 6 files remain; the other occurrences are tests, and `pse-buildinfo/src/identity.rs` has no production use. The fsync sequence is not uniform: the export path skips the directory fsync, and the capture acknowledgement publisher has no `sync_all`. See [rustix](../libraries/rustix/README.md) (`RENAME_NOREPLACE`) and [tempfile](../libraries/tempfile/README.md).

<a id="cbo-004"></a>

## CBO-004 — Child-process and process-group supervision implemented per consumer

- **Operation and consumers.** Start an external process, bound its output, observe it, cancel it
  and prove its whole process group has drained before releasing the caller. Consumers are:
  - native compilation of scalar evaluators with `g++`;
  - the native child-process route;
  - diagnostic capture launches;
  - the managed-primary supervisor calls;
  - script-side workload wrappers.
- **Current mechanism.**
  - `pse-math/src/native_compiled.rs` (`CompilerChild`) scans `/proc` for group members, checks
    identity through field 19 of `/proc/<pid>/stat`, and uses `pidfd_open`, `poll`, `WNOWAIT`
    and process-group kill. It also has a pipe-reader thread with a capture cap and a deliberate
    "unproved drain parks forever" stance.
  - `pse-runtime/src/tooling/capture/launch.rs` uses `rustix` pidfd, `waitid` and process-group
    kill.
  - `pse-runtime/src/math/native_child.rs` runs a child over a `UnixStream`.
  - `workflow/worker.rs` spawns `python3 scripts/surreal_server.py` through `tokio::process`
    with a 64 KiB output bound and a kill-and-join sequence.
  - The Python scripts carry their own process-identity and drain logic (see
    [CBO-048](developer-tooling.md#cbo-048)).
  - Generic supervision therefore lives inside a numerics module and three runtime modules, each
    with its own drain proof.
- **Why it matters.**
  - The no-orphan guarantee and its identity rules (start ticks against PID reuse) are proved
    separately at each site.
  - A stricter rule added at one site does not reach the others.
- **Patterns.** LOP-09 (primary), LOP-01, LOP-08.
- **Footprint.**
  - Inspected: the compiler child and capture launch.
  - Located only: the native child and supervisor spawn.
- **Preservation constraints.**
  - No orphaned descendants.
  - Cancellation and deadlines are observed.
  - An unproved drain never releases the caller.
  - Bounded output capture.
  - Deterministic compiler flags (`-ffp-contract=off`, `-march=x86-64`).
- **Counterevidence.**
  - The semantics are stricter than most crates offer: pidfd identity plus re-inventory after a
    fork.
  - Running a workload in a systemd scope or cgroup is a different mechanism that the repo
    already uses for whole commands.
- **Status.** Current promise.
- **Research questions.**
  - Do `process-wrap`/`command-group` or `tokio::process` with process groups cover the drain
    proof?
  - Would a transient systemd scope or a cgroup `cgroup.kill` per child replace `/proc`
    scanning?
  - Can one local supervised-child type serve all four Rust sites?
- **Library assessment correction (2026-10-27).** Only two of the four sites, `CompilerChild` and capture launch, prove that the whole process group drained, and they duplicate one design. `native_child` and the `worker.rs` control call kill and wait the direct child only. See [linux-cgroup-v2](../libraries/linux-cgroup-v2/README.md) and [linux-pidfd](../libraries/linux-pidfd/README.md).

