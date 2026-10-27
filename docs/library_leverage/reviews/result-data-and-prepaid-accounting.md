# Result data paths and prepaid accounting

**Group G4 of the [comparative review](README.md), 2026-10-27.** Focused
comparative advice under `leverage-opportunity-review`. It recommends bounded targets and
leaves the rest unresolved where it has to. It authorizes nothing.

| Item | Value |
| --- | --- |
| Reassessment | 2026-10-27 at source `4f52fd239`; consumes the complete original review and updated inputs at `b29fcf802`, base patterns 0.2 and additional patterns revision 3 (refined to 4 here). All named dispositions were reconsidered with representative current source, consumers and inherited evidence; unchanged evidence retains its original conditions. |
| Original source inspected | `main` at `3e9a2525b`. None of the G4 files read here changed between the audit baseline `dbc4c9436` and `HEAD`. Concurrent uncommitted edits elsewhere (`pse-math`, `pse-operations` transport and staging) were not used as evidence. One fact, the source-object digest checks in `canonical_staging.rs`, was confirmed against the committed `HEAD` version. |
| Library realizations | arrow family 59.3.0 (`arrow-ipc`, `arrow-buffer`, `arrow-data`, `arrow-ord`, `arrow-select`, `arrow-schema`), with 58.3.0 and 59.2.0 sources for comparison. DataFusion 55.1.0. `flatbuffers` 25.12.19. Everything else as named in the library accounts consumed. |
| Inputs consumed | Codebase account (CBO-001, -002, -005, -006, -007, -008, -012 with their 2026-10-27 corrections). Library accounts: `arrow/{ipc,data,compute,json}`, `datafusion`, `rust-std-io`, `bytes`, `retained-size-crates`, `salsa`, `feral`, `faer`, `suitesparse/klu`, `sundials`, `petgraph-and-rustworkx-core`, `object-store`. Additional patterns revision 2: LOP-X001 and LOP-X009 are central. |
| Criteria | Core 3.4 and the efficient-architecture heuristics, process-simulator profile 1.5, pse-arrow binding. Architecture owners: §19.2 (results), §20.6 (payload results), §4 relations. |
| Evidence | Enabling mechanisms are *Interface-checked* by reading installed source, as named in each section. Every target is *Proposed*. The CBO-005 revealing case was *Tested* afterwards by the coordinator-run [hostile-stream probe](../../design_review/evidence/library-leverage-arrow-ipc-stream-hostile-2026-10-27/README.md); nothing else is *Tested* or *Measured*. |

## Group summary

**The joint decision.** The G4 opportunities share one obligation: result data must cross
storage, transport and selection boundaries without allocating anything that was not prepaid,
and without changing scientific bits or coordinates. They also share one root cause.
Size information sits at the wrong granularity or in the wrong owner:
- the read path charges a whole sealed file to decode one batch;
- the write path charges a writer-shaped multiple whose derivation predates the current Arrow
  writer;
- the dense selection path hydrates rows to learn facts the column layout states directly;
- hostile-input guards sit on the best-protected path (digest-verified sealed files), while
  weaker import paths call the stock decoder.

The recommended direction is to let Arrow's lower-level, caller-owned APIs carry size and
admission:
- `FileDecoder` and `StreamDecoder`, fed buffers the caller has already reserved;
- `BooleanBuffer` bit tools and `partition` for selections;
- typed `io::Error` payloads for refusals.

Domain policy stays local: registry-schema equality, no dictionaries or compression, IEEE-bit
predicates and coordinates, the dense integrity refusals, and the preflight checks that arrow
59.3.0 still needs. Library-internal residual bounds move into one named owner per library
quantity.

**Dispositions.**
- CBO-001: Recommend a cross-library target. Its derive sub-question stays unresolved.
- CBO-002: Recommend a small local consolidation. Retain the estimate-first splitting.
- CBO-005: Recommend.
- CBO-006 and CBO-007: Recommend, judged jointly. The "dense selection outside the planner"
  premise is rejected.
- CBO-008: Retain current.
- CBO-012: Retain current.

**Interactions with other groups.**
- **G5 and G6** supply the math and native parts of CBO-001. The taxonomy and bound-owner rule
  below are the joint target they plug into.
- **G2:** cancellation checkpoints inside output sinks (CBO-002) must use G2's cancellation
  types and stay distinguishable from limit refusals.
- **G3:** dense-output window selection already runs in generated SurrealQL over the
  `result_output_range` index. That is why the DataFusion route of CBO-007 is rejected.
- **G8:** the registry generator already emits `HeapUsage` for registry types (LOP-X003/X007).
  Boundary-document codecs touch CBO-008.
- **G1:** sealed files are read through `ProtectedFile` after full-length digest validation. The
  trust-boundary argument in CBO-005 depends on that.

**Reinforcing and competing choices.**
- The `FileDecoder` admission (CBO-005) is also the largest concrete CBO-001 improvement on the
  result path. Choosing one without the other would leave either the double read or the
  whole-file charge in place.
- The single output-predicate type (CBO-007) is the prerequisite for the column-wise dense
  window (CBO-006).
- The typed limit marker (CBO-002) is a small prerequisite for typed admission refusals
  (CBO-005).
- No two recommendations compete.

**Consequence-based priority**, kept separate from prerequisite order:
1. CBO-005 with the result-path part of CBO-001. Today one batch decode transiently reserves 4×
   the whole file plus 2× for a cached reader. Footer and block metadata are read twice. A
   replay document can drive the stock `StreamReader` into advertised-length allocation.
2. CBO-006 and CBO-007. The read path allocates a formatted string per row, and window
   semantics are defined in three places.
3. The cross-library parts of CBO-001. The drift risk is diffuse and long-term.
4. CBO-002.
5. CBO-008 and CBO-012 need no work.

**Prerequisite order:**
1. Typed refusals (CBO-002 marker, typed `ResultBlockError` variants).
2. The shared IPC admission (CBO-005).
3. Stronger input extents and complete residual read-side bounds (CBO-001, result part).
4. The predicate type (CBO-007).
5. The dense column-wise path (CBO-006).

---

<a id="cbo-001"></a>

## CBO-001 — Prepaid memory and work accounting mirrors library internals

### Responsibility and target intent

The promised behaviour is a deterministic, typed, recoverable refusal **before** an allocation
the deployment pool has not granted. Its sources:
- DP-20;
- the §19.2 statement that tables "reserve their buffers before construction";
- the §20.6 statement that "File framing/layout/integrity admission precedes decoder
  allocation";
- the OOM incidents recorded in AGENTS.md.

There is one pool identity: DataFusion `MemoryPool` through `pse_columnar::AllocationLease`.
Retained allocations keep their reservation until the last reader drops them (ADR-0055,
`pse-columnar/src/owned_buffer.rs`). Errors must lean conservative.

The following are incidental, not promised:
- which particular multiplier implements a bound;
- the granularity at which a charge is taken;
- whether a charge is later reconciled down to the measured size.

### What the inspection changes in the premise

- **"Two accounting systems in one crate" is not two systems.** Salsa's `heap_size` closures
  return the local `retained_bytes`, so Salsa consumes the local measure
  ([salsa](../libraries/salsa/README.md), confirmed).
- **The retained-size "derive" largely exists already.** `HeapUsage` has about 771
  implementations in 214 files, and 204 of those files are registry-generated. The
  hand-written residue is about 205 `fn retained_bytes` and about 465 `fn *_bytes(` outside
  generated paths, counted with `rg` (the second count includes some non-measurement
  functions).
- **The largest result-path over-admission is a granularity mismatch, not a missing library
  number.** `ResultBlockDecode::access_block` (`result_snapshot.rs`) reserves
  `payload_bytes × 4 + schema × 4 + 64 KiB` to decode **one** batch, plus
  `payload_bytes × 2 + …` held by the cached reader. For sealed placement, `payload_bytes` is
  the whole file (`ResultPayload::byte_len` returns `descriptor().bytes`), and a file can be up
  to `RESULT_BLOCK_BYTES` = 32 MiB. The exact block span is available from the footer the
  preflight already verifies.
- **A writer mirror outlived its derivation.** `AdmittedResultFile::encode` reserves
  `byte_limit × 3`, and `result_blocks::encode_block` says it bounds scratch "before
  IpcDataGenerator packs/copies the body". Arrow-ipc 58.3.0 did copy each body into
  `EncodedData.arrow_data`. At 59.2.0 and 59.3.0, `FileWriter::write` calls
  `IpcDataGenerator::write`/`write_to_sink` with `IpcBodySink::Collect`, which streams the
  bodies zero-copy. The ×3 is still conservative: `Vec` doubling, plus the owned re-based
  offsets or bit-shifted validity of sliced inputs, still fit under it. But nothing records
  why, and the next writer change could move it in either direction. This is the LOP-X001
  hazard, observed rather than hypothetical.

### Credible complete options

1. **Retain current.** Keep the scattered multipliers and container-granularity charges.
2. **Classify each charge by its kind of size information and take the strongest kind each
   library offers (recommended).** This uses the LOP-X001 revision-2 kinds. Residual
   library-internal bounds move into one named owner per library quantity. Post-hoc
   measurement is used only to reconcile a charge downward after the fact.
3. **Allocation-time enforcement.** Candidates:
   - arrow-buffer `pool` with DataFusion `arrow_buffer_pool` (`ArrowMemoryPool`,
     `RecordBatch::claim`);
   - a counting global allocator;
   - SuiteSparse allocator hooks;
   - cgroup `memory.max`.
4. **Derived retained measurement.** Candidates: `get-size2` 0.11.x, a local derive, or the
   existing registry generator.
5. **Upstream hooks (U1).** Candidates:
   - a FERAL `SparseLu` conservative owner bound;
   - a released Symbolica budget or abort carrier;
   - arrow-rs decoder spans returning errors (see CBO-005);
   - a SUNDIALS 8 replacement for the workspace queries.
6. **A coarse per-operation envelope** that replaces per-allocation predictions with periodic
   checks.

### Comparative argument

Option 2 is the only one that keeps the prepay semantics while removing derivation-free
mirrors.
- **Option 3** cannot replace prepayment. `arrow_buffer::MemoryPool::reserve` is infallible,
  so a claim over-fills the pool instead of refusing. `Bytes::claim` replaces any earlier
  reservation (`arrow-buffer-59.3.0/src/bytes.rs`). Enabling `pool` also changes the Arrow
  build for the whole family. SuiteSparse hooks are process-global and fire during
  factorization, not before. RSS and cgroup limits are non-deterministic. All of these can
  serve only as a backstop, and the host-level one belongs to G1.
- **Option 6** weakens the guarantee: allocations would no longer be charged before they
  happen. It is excluded under the no-compromise rule.
- **Option 4** is a measurement tool. It helps the *retained* side, such as cache and copy
  charges, but it never predicts.

Under option 2, each library route contributes as follows. The math and native rows come from
the G5 and G6 accounts; G4 states the target.

| Kind (LOP-X001 rev. 2) | Library routes at the current pins | Role in the target |
| --- | --- | --- |
| Symbolic-phase prediction | FERAL LDLᵀ `symbolic_factorize(..).work_estimate()` (already used); faer `*_scratch() -> StackReq` (already used for LU scratch); faer sparse Cholesky/LDLᵀ/QR `len_val()`; KLU `klu_l_symbolic.lnz/unz` (an estimate, not a bound) | Charge directly where it is a bound. An estimate plus the library's growth rule is not a bound: KLU keeps growing (`memgrow·lusize + 2n + 1`, through `realloc` with old and new both live) until the factor fits, so use G6's per-block dense bound and count the realloc transient. *(Revised after the numerical and solver methods challenge.)* |
| Published conservative bound | FERAL `Solver::conservative_storage_bytes(n, nrhs)` under `with_bounded_storage_max_dimension` | Charge directly. |
| Library-polled checkpoint | Symbolica budget or abort carrier, unreleased (G5); DataFusion operators reserve internally against the shared pool | Use where released. Interruption must mean "cancelled", never a silently truncated result. |
| Exact pre-materialization step | Arrow selection counts; verified file-block input spans; serde counting sinks; `StreamEncoder::encode` message sizes | Exact logical/input extents inform admission. Also bound rounded capacity, metadata, wrappers, scratch and copies before allocation; an exact part is not an exact complete decode envelope. |
| Reuse-time exactness | KLU `klu_l_refactor` allocates nothing; faer symbolic reuse with numeric refresh | Bound the first factorization, then charge refreshes exactly (G6). |
| Post-hoc measurement | Arrow `get_array_memory_size`; DataFusion `get_record_batch_memory_size`; KLU `Common.memusage`; Salsa `memory_usage` (unstable); SUNDIALS workspace queries (deprecated in 7.3, removed in 8.0) | Never a prepayment. Use only to reconcile downward (`split`/`shrink`/`try_resize`) and for observation. `AllocationScope::attach_borrowed_reserved` already transfers only the actual native extents. |
| No library information | DataFusion `PhysicalExpr` temporaries (`rows × 64` in `select_original_rows`); pruning intermediates (`fields × 512 + 2048` in `prune_members`); FERAL Markowitz LU fill (G6); std `BTreeMap` node geometry; Symbolica 3.0.1 cost formulas (G5); IDAS/BBD internals (G6) | **One named bound owner per quantity.** It sits in that library's integration owner, records its derivation and the pinned version it was derived against, and is the single place to re-derive on a pin move. |

Measured against the heuristics:
- **H1, H4:** the pool and lease stay. What changes is where each number comes from.
- **DP-16, LOP-11:** one place per quantity to revisit on a version move.
- **H21, DP-20:** stronger part extents plus complete residual bounds can reduce unnecessary shared-pool reservation under 16-lane
  reads. The current read-path over-reservation is up to about 6× the file size per active
  decoder.
- The accounting can use existing leases and named bound owners. CBO-005 still requires a
  shared bounded framing/decoding mechanism and complete residual derivations; no new general
  accounting framework or checker is justified by this recommendation.

**Derived retained measurement (option 4).**
- The registry generator already owns this for generated types and should keep it
  (LOP-X003/X007 apply).
- `get-size2` cannot host the conservative container policy. Its `BTreeMap` and `HashMap` impls
  under-count, and because the crate owns both the trait and the std impls, the only override
  is a per-field `size_fn`. Forgetting that attribute would silently under-count, which weakens
  the guarantee. It is excluded unless upstream adds a policy hook (U1).
- Two credible local routes remain:
  - **Container impls plus an idiom.** Add conservative `HeapUsage` impls for
    `BTreeMap`/`BTreeSet`/`HashMap` with one documented geometry in `pse-model`, replacing
    today's `size_of::<(K, V)>() + 64` per entry and the separate "eleven key/value slots"
    comments. Then use an exhaustive-destructuring idiom (`let Self { a, b, c } = self;`) in
    hand-written impls, so that adding a field is a compile error until it is accounted for.
  - **A local derive macro** over `HeapUsage`. That requires a proc-macro crate.

### Revealing cases

- **A one-batch decode from a 32 MiB sealed file.**
  - Today: about 128 MiB transient plus 64 MiB cached.
  - Target: exact raw input extent plus the complete pre-admitted metadata/schema, backing
    capacity, array/wrapper and permitted-copy envelope described by CBO-005. Attach the input
    through `AllocationScope::attach_buffer` and residual allocations through their reserved
    owners. Those primitives exist; the complete new IPC allocation envelope is not implemented.

  Prepayment is preserved only when that full envelope is established. No exact-total charge
  or reduced-refusal result follows from the raw span alone.
- **FERAL Markowitz LU.** No library number exists, so the target can only consolidate the cubic
  bound into one owner. If that bound refuses ordinary supported blocks (H11), the remedy is an
  upstream owner bound (U1) or a different factor route. That is G6's decision, not a weakening
  here.
- **The ×3 writer grant.** A stale-but-conservative mirror is invisible until it becomes stale
  in the other direction. The bound-owner rule (a derivation plus the pinned version beside the
  number) is what makes a pin move revisit it.

### Disposition

**Recommend**, as a bounded cross-library target.

- **Enabling mechanisms (G4 part):**
  - `arrow-ipc` 59.3.0 `FileDecoder::{new, with_require_alignment, read_record_batch}` and
    `StreamDecoder::decode`, fed caller-reserved buffers, composed with the existing
    `pse_columnar::AllocationScope::attach_buffer`;
  - `arrow-buffer` 59.3.0 `BooleanBuffer::{count_set_bits, set_slices, set_indices_u32,
    collect_bool}` for exact selection charges;
  - the existing DataFusion `MemoryReservation::{try_grow, split, shrink}` for reconciliation.
- **Responsibility boundaries.**
  - The pool, leases and ADR-0055 ownership stay local and unchanged.
  - Each library integration owner holds its residual bounds:
    - `pse-columnar` for IPC encode and decode extents;
    - `pse-engine` or the result-selection owner for expression temporaries and pruning
      intermediates;
    - `pse-model` for container geometry;
    - G5 and G6 owners for math and native.
- **Behaviour changes.**
  - Charges use stronger exact part extents and complete residual bounds where available.
  - Fewer capacity refusals are an expected benefit only after that complete envelope is derived.
  - Bits, coordinates and refusal classes do not change.
- **Machinery that disappears:**
  - the ×4 and ×2 whole-file decode grants;
  - the second footer and block read;
  - the `num_rows × 4` offset pre-charge in favour of `count × 4` or zero for a range;
  - inline magic multipliers at call sites.
- **Machinery added:** named bound functions with derivations, and conservative container
  `HeapUsage` impls.
- **Transition.** Local and incremental, owner by owner. The result-path part travels with
  CBO-005.

**Unresolved sub-question: a derive for hand-written measures.** The choice is between
container impls plus the destructuring idiom and a local proc-macro derive.
- **What it decides:** whether field-drift protection comes from the compiler (both options
  give it) or the derive's maintenance is worth a new crate.
- **Settling evidence:** a census of the roughly 205 `retained_bytes` and 465 `*_bytes` bodies,
  splitting plain field sums from deliberate policy (fixed per-entry allowances, shared-input
  exclusion, `Arc` policy). The samples read here (`pse-math/src/execution.rs`,
  `pse-modeling/src/extent.rs`) are mostly policy, which favours the idiom.

**U1 branches.**
- **If upstream contribution is acceptable:** contribute the FERAL LU owner bound, the Symbolica
  budget release and the arrow-rs decoder error paths. Each residual owner then retires its
  bound only once the release is reachable under the pins.
- **If not:** the residual owners are permanent, and the target is unchanged.

### Rule impacts

- None for adopting library size APIs, consolidating bounds or reconciling charges. These are
  refactors within contracts.
- A new proc-macro crate for a local derive would be a crate addition and needs an ADR.
- Arrow 60 is a pinned-family major and needs an ADR. It also needs a DataFusion release on
  arrow 60.
- The prepayment contract is unchanged, so no design review is triggered.

### Evidence

Interface-checked against these sources:
- `arrow-ipc-59.3.0/src/{reader.rs,reader/stream.rs,writer.rs}` and `arrow-ipc-58.3.0/src/writer.rs`;
- `arrow-buffer-59.3.0/src/{pool.rs,bytes.rs,buffer/boolean.rs}`;
- repository sources `result_snapshot.rs` (`access_block`, `prune_members`),
  `connected_results.rs` (`select_original_rows`),
  `pse-relations/src/result_batch.rs` (`AdmittedResultFile::encode`),
  `pse-columnar/src/owned_buffer.rs` (`attach_buffer`, `attach_borrowed_reserved`), and
  `pse-model/src/lib.rs` (`HeapUsage`).

Math and native routes are inherited from the G5 and G6 library accounts and were not
re-verified here. The target is Proposed.

---

<a id="cbo-002"></a>

## CBO-002 — Bounded and counting output sinks re-implemented per site

### Responsibility and target intent

The promised behaviour has three parts:
- refuse when serialized output would exceed a hard bound, with a refusal that is
  distinguishable from I/O failure;
- count exactly, without materializing, so that a copy can be prepaid (the exact
  pre-materialization kind under CBO-001);
- where output must be split, choose deterministic split points, because block boundaries
  become durable ordinals and digests.

The following are incidental:
- the per-site struct shapes;
- string discrimination;
- the `ErrorKind` each site picks (`Other`, `OutOfMemory`, `InvalidData`).

### What the inspection changes in the premise

- **The trial-encode loop in `result_files.rs` is dead in production.** `encode_result_prefix`
  and `visit_result_files_async` have no caller outside `result_files.rs` and
  `result_projection.rs` tests. Production result files are split by `result_prefix_rows`,
  which halves over the `batch_extent` estimate without encoding. `AdmittedResultFile::encode`
  then enforces the 32 MiB hard bound once. The `"sealed result file limit"` string match in
  `splittable` is reachable only from the dead path.
- **One live string match remains.** `result_blocks::can_split` matches `"result block limit"`
  through `ArrowError::IoError` for canonical source objects (3 MiB). Even there, encoding runs
  only after a `visible_size` estimate passes, so halving after encoding is a rare fallback.
- **The counting sinks are the JSON analogue of `count_set_bits`.** They are legitimate
  prepayment machinery, not defects.

### Credible complete options

1. **Retain current.**
2. **Use `bytes::BufMut::limit().writer()`.** Ruled out:
   - its refusal is a generic `WriteZero` short write;
   - it has no counting-only mode, no forwarding and no checkpoint
   ([bytes](../libraries/bytes/README.md)).
3. **Exact split sizing with `arrow-ipc` 59.3.0 `StreamEncoder::encode`.** This gives exact
   message sizes with zero-copy bodies.
4. **A local shared sink module plus a typed marker (recommended).** The marker uses
   `io::Error::other(LimitReached)` and is recovered with `get_ref().downcast_ref` /
   `io::Error::downcast`. It survives through `serde_json::Error → io::Error` and
   `ArrowError::IoError(_, io::Error)`, as confirmed in `arrow-schema-59.3.0/src/error.rs`.
   Dead code is deleted.

### Comparative argument

**StreamEncoder (option 3)** would add an extra full walk per attempt, and its owned
re-based-offset allocations for sliced variable-width columns would themselves need
prepayment. The 16 MiB file target is a soft packing goal and the 32 MiB bound is already
enforced by the sink, so exactness buys little. Halving after encoding is already rare.
Adopt it only if measurement shows frequent hard-bound fallbacks.

**The shared sink (option 4)** is a clear but minor win:
- one typed refusal instead of about fifteen near-identical counting structs with three error
  kinds;
- removal of the brittle string match;
- one place for the per-write checkpoint (cancellation or expiry), which at least five sinks
  carry.

**Hashing sinks stay separate.** `EncodingHasher`, `DigestWriter` and the framed `PartWriter`
with its sticky failure are different responsibilities. The shared sink may wrap them as inner
writers but must not absorb them.

**Placement:** `pse-ids`. It already hosts the encoding sinks and sits beneath every consumer
crate (`pse-diagnostics → pse-ids → everything`).

### Revealing case

A sink that checks cancellation on every write, merged with a limit:
- The target keeps `Cancelled` and `LimitReached` as distinct typed payloads, so a
  cancellation is never retried by halving.
- A split helper that preserves the current rule (`rows.div_ceil(2)`) keeps durable split
  points byte-identical. Changing the rule would alter ordinals and digests for the same input,
  and nothing requires that.

### Disposition

**Recommend a small local consolidation.**
1. Add one `pse-ids` sink module: count-only, or a `Vec` with exact `try_reserve_exact`
   capacity, or forwarding to an inner writer. It takes an optional checkpoint closure and
   emits typed `LimitReached` / checkpoint payloads.
2. Replace the string matches with `downcast_ref`.
3. Delete `encode_result_prefix`, `visit_result_files_async` and their tests (DP-16).
4. Turn `ResultBlockError::Invalid(&'static str)` matching (`splittable`, `can_split`) into
   typed refusal variants.
5. Route the progress writer's `full` flag through the same marker (see CBO-008).

**Retain** the estimate-first splitting and the hard bound in the final sink, and do not adopt
`StreamEncoder` sizing.

**Rule impacts:** none.

**Evidence:** Interface-checked:
- the sink census;
- `result_files.rs`, `result_blocks.rs`, `durable.rs` (`ProgressBuffer`), `replay.rs`
  (`ExtentWriter`), `result_projection.rs` (callers);
- `arrow-schema-59.3.0/src/error.rs`, `arrow-ipc-59.3.0/src/writer.rs` (`StreamEncoder`).

The target is Proposed.

---

<a id="cbo-005"></a>

## CBO-005 — Arrow IPC untrusted-input verification written by hand

### Responsibility and target intent

§20.6 promises that "File framing/layout/integrity admission precedes decoder allocation",
with typed refusals. The domain policy is:
- exact registry-schema equality;
- metadata version V5 and little-endian only;
- no dictionaries, no compression, no variadic buffers;
- bounded tables, depth, batches and value slots.

The current file preflight (`result_files::preflight`, `result_blocks::{schema_field,layout}`)
is justified **at arrow 59.3.0**, as the library assessment established check by check
([arrow/ipc](../libraries/arrow/ipc.md), LOP-X009). Incidental mechanisms:
- reading the footer and every block's metadata twice (once in the preflight, again in
  `FileReaderBuilder::build` and `FileReader::next`);
- reading every block's metadata at open, even for batches the pruning step will never decode;
- restating Arrow buffer widths in local match arms.

### What the inspection adds

- **Guard placement is inverted relative to trust.**
  - Sealed files are opened through `ProtectedFile` only after a length check and a full
    streaming digest check against the store descriptor (`canonical_files::validate_file`), and
    were written by the system's own encoder. They get the full preflight.
  - Canonical physical-source objects (`result_physical.rs`) are digest-verified
    (`canonical_staging` assembly) and get no preflight.
  - The **reproducer** decodes IPC embedded in an exported replay document with only
    continuation and end-of-stream checks (`reproducer::recorded_source_complete`), using the
    stock `StreamReader` and a `bytes × 16 + 64 KiB` charge.
  - Review-pack previews wrap a `StreamReader` in `take`, which does not prevent
    advertised-length pre-allocation (allocation precedes the read).

  At 59.3.0, a lying `bodyLength` or metadata length on those import paths allocates before
  reading. A large positive length can abort through `handle_alloc_error`.
- **Arrow already provides the lower layer that would give the codebase allocation ownership.**
  - `FileDecoder::read_record_batch(&Block, &Buffer)` reads from a buffer the caller allocated.
  - `StreamDecoder::decode(&mut Buffer)` accumulates message and body bytes only as supplied;
    a negative `bodyLength` never completes and `finish()` refuses it.
  - Both still reach `fb_to_schema` and the `read_buffer` span assertion, so the layout checks
    remain necessary.
- **Alignment depends on both the backing allocation and each typed buffer offset.** The
  writer's default 64-byte padding is useful, but an embedded `Vec<u8>` has no guaranteed
  8-byte base alignment. Check the actual typed-buffer addresses against `BufferSpec` from
  `arrow_data::layout`; header padding alone does not establish zero-copy eligibility.
- **§19.2's "Retained scientific rows" text no longer matches the code.** It describes
  independent 512 KiB IPC streams with a preflight. The code (and §20.6) use sealed IPC files
  with a 16 MiB target and the 32 MiB `RESULT_BLOCK_BYTES` bound, plus the inline arm. This is
  an owner inconsistency for the architecture route to fix, not a leverage question.

### Credible complete options

1. **Retain current.** Keep the full preflight on sealed and inline files and the stock readers
   elsewhere.
2. **Move to arrow 60.0.0.** It brings `try_fb_to_schema`, stream body bounds and a
   missing-footer-schema error. This route is not available until a DataFusion release on arrow
   60 exists, and it is a family major.
3. **One shared bounded IPC admission owner on `FileDecoder` and `StreamDecoder` (recommended).**
4. **Rely on digest verification and retire the preflight for verified paths.** Excluded:
   it weakens a declared §20.6 guarantee. It is recorded below as an architectural question.
5. **Contribute the missing checks upstream (U1).**

### Comparative argument

**Reassessment correction (2026-10-27).** Recommend a shared bounded framing and allocation
owner, with consumer-specific schema and lineage admission. `FileDecoder` and `StreamDecoder`
delegate array decoding; neither is an allocation-limit or untrusted-message admission API.
The complete envelope below is a prerequisite to removing the current conservative grants.

1. **Metadata admission.** Check actual file/message spans, verifier table/depth limits,
   supported headers and types, node/value limits, buffer spans, dictionary/compression policy
   and the consumer's schema before delegation. Keep the admitted footer, block inventory and
   schema for file decoding. Their allocations need admission as well as the body allocation.
2. **Per-block input.** `metaDataLength + bodyLength` is the exact input extent, not the entire
   decoder allocation. Pre-admit rounded backing capacity, metadata/schema materialization,
   array/wrapper inventories and permitted copies before constructing them. Attach the input
   lease through `AllocationScope::attach_buffer`; separately attach the residual allocations
   using the existing reserved-allocation mechanisms. Zero-copy slices inherit the input owner.
3. **Alignment.** Read into an aligned allocation and check each typed buffer address using
   `arrow_data::layout`. For embedded files and streams, pre-admit any realignment copy or
   refuse according to the consumer's supported contract. `require_alignment(true)` without
   establishing those premises can reject valid documents.
4. **Streams.** Validate each complete encapsulated message outside `StreamDecoder` before
   feeding it: the decoder has no pre-decode admission callback. Prefer admitted complete
   message/body buffers. Arbitrary chunking also needs a separate bound for decoder-owned
   accumulation: `reader/stream.rs` copies fragmented metadata and bodies into `self.buf`
   while caller input may remain live. Refuse compression before delegation under the current
   no-compression policy; decompression can allocate from the embedded decoded length.
5. **Consumer contracts.** Keep registry-schema equality for result files. Scientific exports
   also carry `pse.canonical.*` metadata produced by `connected_results::write_ipc` and checked
   by `portable_payload::validate_scientific_lineage`. Bare-registry equality applied unchanged
   to every stream would reject valid exports. Share transport-safety checks while retaining
   lineage, value-slot, surplus-node/buffer and `Null`-length policy with its semantic owner.

**Heuristics:** H4 (delegate the decoding, keep the gap), H16 (one read, zero-copy), H18 (put
admission at the trust boundaries that lack it), H21 (exact charges in the shared pool).

**Option 2** retires a few checks later, one at a time, as LOP-X009 describes:
- `try_fb_to_schema` reduces the schema pre-walk;
- #10522 bounds stream bodies;
- validity minima are fixed upstream but unreleased.

Buffer spans, footer length and block bounds remain local on every inspected version (#11012
is open). It is a later refinement, not an alternative.

### Revealing case

A valid inline result payload decoded with `require_alignment(true)` from a whole-file `Vec`
could be refused. The `Vec<u8>` base has byte alignment, and block offsets start after the
8-byte magic. That would reject valid work. Hence the inline branch above: alignment
enforcement belongs where the caller owns the allocation.

The second case is a replay document with a lying `bodyLength`. The
[hostile-stream probe](../../design_review/evidence/library-leverage-arrow-ipc-stream-hostile-2026-10-27/README.md)
(*Tested*, arrow-ipc 59.3.0, this host, uncapped and under 4 GiB and 1 GiB address-space caps)
settled what happens today:

- A 408-byte stream whose record batch claims `bodyLength` = 1<<50 makes `StreamReader` call
  `MutableBuffer::from_len_zeroed` for the claimed size before reading, and the process aborts
  (signal 6) through `handle_alloc_error`. The same claim on the schema message aborts inside
  `StreamReader::try_new`. A negative length panics. Wrapping the reader in `Read::take` does
  not help.
- For the tested lying raw-body lengths, `StreamDecoder` avoids `StreamReader`'s eager
  advertised-body allocation and `finish()` reports incomplete input. This does not establish
  absence of internal accumulation, schema/wrapper allocations or advertised decoded-size
  allocations for compressed inputs.
- Both readers share the record-batch decode and **panic** on buffer offsets or lengths outside
  the body (`Buffer::slice_with_length`), so the local span check stays necessary after the move.
- Oversized node and batch lengths are already typed errors through `ArrayData` validation.

An abort is not an unwind, so `panic = "unwind"` and PyO3's exception translation do not
contain it: a Python caller loses the interpreter. With the target the same document meets a
typed refusal before any input-sized allocation.

### Disposition

**Recommend.** The bounded target is the shared admission module above, built on:
- `arrow-ipc` 59.3.0 `FileDecoder::{new(schema, MetadataVersion::V5), with_require_alignment,
  read_record_batch}` and `StreamDecoder::{new, with_require_alignment, decode, finish}`;
- `reader::read_footer_length`, `root_as_footer_with_opts`, `root_as_message_with_opts`;
- `arrow-data` 59.3.0 `layout(&DataType)`;
- the existing `AllocationScope::attach_buffer` and DataFusion reservations.

Consumer integration:
- `ResultBlockDecode::access_block` and `CachedFile` switch from `FileReader` to the cached
  decoder state.
- `reproducer_physical`, `review_pack::presentation`, `portable_payload` and `result_physical`
  call the stream admission.

Eligible for removal after the complete replacement allocation envelope is established:
`FileReaderBuilder`, the second footer and block read, the ×4 and ×2 grants, and the local width
tables. A raw input-span lease alone does not justify that removal. Typed refusals map
`ArrowError` from delegated validation into the existing
`ResultBlockError` family. Diagnostic codes pinned by tests may need updating; bits, schemas and
deterministic bytes do not change.

**Premise settled by the probe's repository trace** (*Interface-checked*, static reading). The
replay document is not authenticated before decode. `run_reproducer` takes a
`RecordedInput<DiagnosticReproducer>` whose `validate_recorded` is the default no-op, and identity
checks run only after the decode. `ProjectBundleReader` and `PackReader` reach the stock reader in
the same way. The import-path part of the target is therefore the highest-priority part: it
removes a reachable process abort, so it is a defect repair as much as a leverage change, and it
does not need to wait for the rest of the module.

**Architectural question for the owner.** Should the preflight remain defence-in-depth on
digest-verified, system-written files (H20)? Retaining it is the no-compromise default here.

**U1 branch.** If upstream contribution is acceptable, offer:
- `read_buffer` span errors instead of assertions;
- footer-length versus file-size checking in `FileReaderBuilder`;
- verifier options on the decoders.

These are not available until released and reachable through a DataFusion release on that
arrow line.

### Rule impacts

- None at 59.3.0. The contract is retained and strengthened at import paths.
- The arrow 60 family major needs an ADR and travels with DataFusion and object_store.
- Correcting the stale §19.2 text goes through the architecture amendment route (a revision
  row in `blueprint.md`).

### Evidence

Interface-checked against:
- `arrow-ipc-59.3.0/src/reader.rs` (`FileReaderBuilder::build` allocates `vec![0; footer_len]`
  before the seek; `FileDecoder`), `reader/stream.rs` (`StreamDecoder::decode`) and
  `writer.rs` (header padding);
- `arrow-buffer-59.3.0` `ALIGNMENT`;
- repository files `result_files.rs`, `result_blocks.rs`, `result_snapshot.rs`,
  `reproducer.rs`, `presentation.rs`, `portable_payload.rs`, `result_physical.rs`,
  `pse-operations/src/canonical_files.rs` (`validate_file`) and `canonical_staging.rs`;
- the Context7 `/apache/arrow-rs` listing of the upstream `zero_copy_ipc.rs` FileDecoder
  example.

The target is Proposed.

---

<a id="cbo-006"></a><a id="cbo-007"></a>

## CBO-006 and CBO-007 — Dense trajectory indexing and selection; output-predicate semantics in several forms

These two are judged together. The predicate definition (CBO-007) decides what the
column-wise dense path (CBO-006) may evaluate.

### Responsibility and target intent

**Store path** (`result_projection::trajectory_outputs`). For each window of at most 1,024 rows,
derive per-symbol contiguous output ranges with time minimum and maximum. Refuse a symbol that
appears in two separate runs. The function runs twice per window: once for candidates, once for
the accepted prefix.

**Read path** (`connected_results::select_original_rows`).
- Dense branch:
  - refuse adjacent rows that belong to the same output;
  - refuse rows in the output range that belong to another symbol;
  - recompute extrema and compare them with the recorded `coordinate_min`/`coordinate_max`;
  - select rows whose `time` falls in the window.
- Scalar branch: turn a DataFusion mask into offsets inside the coordinate window.

**Validation** (finite bounds, `min ≤ max`, `missing` excluding bounds) is written three times:
`output_results`, `scalar_expression` and `ReviewOutputSelection::bounds`.

**Representation:**
- `PackResultPredicate` and `ReviewOutputSelection` carry IEEE bits;
- `OutputFilter` carries `f64`s;
- IPC export encodes `pse.canonical.*` metadata in `connected_results.rs`, and
  `portable_payload.rs` decodes the same keys independently.

**Promised:**
- exact original coordinates and IEEE-bit predicate identity;
- recorded coordinate bits (SurrealQL stores `coordinate_min.bits` and asserts a finite
  projection);
- the dense refusals;
- `u32` take admission;
- prepayment.

**Incidental:**
- row hydration (`View::row` decodes all seven fields, up to three times per row on the store
  path);
- a `format!("{relation}:{symbol}:value")` allocation per row on the read path, although
  `OutputFilter.owner` already holds the `SemanticId`;
- the `num_rows × 4` offset pre-charge;
- the mask-to-index loop over every row.

### What the inspection changes in the premise

- **Dense selection across outputs is not "outside the planner". It is inside the canonical
  store.** The generated SurrealQL reads `canonical_result_block_outputs` through the
  `result_output_range` index (`result_set, output, partition, start`). It applies
  `coordinate_max.projection >= $minimum AND coordinate_min.projection <= $maximum` in the same
  query. A DataFusion `PruningPredicate` over output statistics would first fetch the
  descriptors the store already filters, which goes against H3 and H7. The DataFusion route of
  CBO-007 is therefore **premise rejected**.
- **Arrow's float comparison kernels use IEEE totalOrder, not only its extrema**
  (`arrow-ord-59.3.0/src/cmp.rs`). Replacing the window test with `gt_eq`/`lt_eq` and scalar
  bounds would change results:
  - `time = -0.0` with lower bound `0.0` would be dropped;
  - a positive NaN would pass every lower bound.

  The scalar path already corrects for this with a CASE normalization and a finite guard
  (`scalar_expression`).
- **The recorded extrema are compared with IEEE `==` (`Option<f64>`).** They are produced by
  `f64::min`/`f64::max`, which ignore NaN. Swapping in Arrow `min`/`max` would change the stored
  bits for runs containing NaN, or both zero signs.

### Credible complete options

1. **Retain current.**
2. **Use the Arrow kernels the library account named** (`partition`, `arrow_arith` `min`/`max`,
   `gt_eq`/`lt_eq`, `set_indices_u32`). This changes extrema and window semantics.
3. **Local column-wise restructuring with generated typed accessors and IEEE-preserving
   evaluation, plus one predicate type (recommended).**
4. **A DataFusion pruning route for dense outputs.** Rejected above.

### Comparative argument

**Option 3** removes avoidable work while retaining the current numerical/encoding contract.
- **Store path.**
  - Use `View::from_checked(..).symbol_id_column()` and `time_column()` (generated, field
    checking preserved).
  - Find run boundaries with `arrow_ord::partition::partition(&[symbol_slice])`. `distinct`
    supports `FixedSizeBinary`, and the scratch is a bitmap of n−1 bits plus the ranges.
  - Check contiguity as a set over run keys.
  - Compute extrema as the **same** `f64::min`/`f64::max` fold over `time_column().values()[range]`,
    retaining its current signed-zero/NaN interpretation. Rust permits either zero input for
    mixed-sign `min`/`max`; introducing an explicit canonical zero is a separate encoding change,
    not proof of preserving today's stored bits. The repository also has Arrow-total-order
    finite extrema in `pse-relations`; name the distinction, and select any reconciliation
    through the numerical/metadata owner rather than bundling it into this restructuring.
  - Reuse the first call's ranges for the accepted prefix.
- **Read path.**
  - Check identity by comparing the symbol column's 16-byte values over the range and the two
    neighbours with `filter.owner`, with no per-row string.
  - Compute extrema with the same fold.
  - Evaluate the window once, from the shared predicate (below), over `batch.slice(first, len)`.
  - Convert the resulting `BooleanBuffer`:
    - `count_set_bits()` gives the exact charge;
    - a single `set_slices()` run becomes `SelectedRows::Range` with no offset vector;
    - otherwise `set_indices_u32()` gives the offsets.
- **Scalar branch.**
  - Slice the batch to the coordinate window *before* `NativeResultSelection::mask`. This is
    zero-copy, so the predicate and its `rows × 64` temporaries apply only to the window.
  - Fold validity into the mask and use the same bit-to-selection conversion.
- **One `OutputPredicate` value (CBO-007).**
  - Hold IEEE bits, as `PackResultPredicate` and `ReviewOutputSelection` already do.
  - Have one validating constructor whose dense/scalar distinction comes from the relation's
    declared result selection. Dense refuses `missing`.
  - Lower it once: `scalar_expression`, generalized to the declared field, evaluates `value` for
    scalar outputs and `time` for dense outputs, with the existing signed-zero normalization and
    finite guard.
  - Encode the `pse.canonical.*` metadata in one codec shared by export (`connected_results`)
    and readback (`portable_payload`).

  `OutputFilter`, `PackResultPredicate` and `ReviewOutputSelection` become projections of it.

Dense integrity refusals stay local and explicit; pruning and predicates never refuse. This
follows DP-01, DP-14, H5, H7 and H16. **The single lowering matters most:** the window
semantics stop being defined twice (a Rust `>=` in one place, a normalized DataFusion
expression in the other).

**Semantic parity with stored data.**
- Stored dense outputs cannot hold infinite times: an infinite extremum fails the finite
  projection assertion.
- NaN times are excluded by both IEEE `>=` and the finite guard when any bound is given, and
  included by both when there is none.
- The single lowering therefore selects the same rows as today for every storable output.

### Revealing case

An output whose first sample has `time = -0.0`, read with `minimum = 0.0`:
- IEEE `>=` (today's dense path) selects it;
- Arrow `gt_eq` (option 2) drops it;
- the normalized shared lowering (option 3) selects it.

A second case is a trajectory with a NaN sample time. Option 2's `max` would return NaN. The
store would then refuse a projection that is accepted today, rejecting previously valid work.
Option 3 keeps today's behaviour. Whether NaN times should be refused at all is a domain
question for the relation's declared `time` domain (`real()`, unconstrained), not a leverage
choice.

### Disposition

**Recommend option 3.** The CBO-007 "dense selection outside the planner" premise is
**rejected**: selection across outputs belongs to the store's index.

Enabling mechanisms:
- `arrow-ord` 59.3.0 `partition::partition`, `Partitions::ranges`;
- `arrow-buffer` 59.3.0 `BooleanBuffer::{collect_bool, count_set_bits, set_slices,
  set_indices_u32}`;
- generated `simulation_samples::View` column accessors;
- the existing DataFusion physical expression and `SelectedRows::{Range, Sparse}`.

The deliberate exclusions are Arrow `min`/`max` and the comparison kernels on `time`.

Removed:
- per-row hydration and formatting;
- the duplicate extrema pass;
- two of the three validations;
- the duplicated metadata key handling;
- the `num_rows × 4` pre-charge.

The restructuring should preserve current extrema production and readback interpretation.
Choosing a new signed-zero canonicalization, or reconciling existing extrema definitions, is a
separate numerical/metadata semantic decision. Unchanged IEEE comparisons alone do not prove
unchanged durable coordinate bits. No-migration and unchanged-byte claims are conditional on
retaining those current encodings.

**Rule impacts.** None, as long as the `pse.canonical.*` export metadata bytes stay identical.
Changing that encoding would touch the metadata contract and need an ADR.

**Evidence.** Interface-checked:
- `arrow-ord-59.3.0/src/{partition.rs,cmp.rs}`, `arrow-buffer-59.3.0/src/buffer/boolean.rs`,
  `arrow-array-59.3.0/src/arithmetic.rs` (totalOrder);
- repository files `result_projection.rs`, `connected_results.rs`, `result_snapshot.rs`
  (`SelectedRows::from_offsets`), `result_selection.rs`, `review_export.rs`,
  `portable_payload.rs`, `pse-relations/src/result_batch.rs` (`extrema`), and the generated
  `surreal.surql` (output query, index, coordinate assertions).

The target is Proposed.

---

<a id="cbo-008"></a>

## CBO-008 — Progress events and completion receipts stored as JSON payloads beside typed relations

### Responsibility and target intent

Retain ordered, bounded, replayable progress under exact manifest admission (§19.2,
`Runtime::progress`). Replay and the reproducer compare diagnostic observations exactly.

### Inspection

The data is not relation-shaped. `ProgressEventDocument` contains:
- an optional event source;
- free-text phases;
- a `BTreeMap<String, ProgressMetricDocument>` of **backend-specific** keys whose values form a
  tagged union: `Integer`, `Real(Observation)` with explicit non-finite evidence, `Text`,
  `Boolean`, `Unavailable(reason)`;
- an optional incumbent.

A registered relation would need a `Map<Utf8, Union>` column or an entity-attribute-value
normalization. `arrow-json` would become a second encoder beside the registry's single
generated codec.

The reproducer's `serde_json::to_vec` equality is deterministic for these types:
- maps are `BTreeMap`;
- finite reals serialize exactly through ryu, including `-0.0`;
- non-finite values are tagged.

So equality by serialization is a correct, if implicit, exact comparison.

### Options

1. **Retain current.**
2. **Make progress a registered relation.**
3. **Keep it schema-declared but unregistered, written with `arrow-json` `Decoder::serialize`.**
4. **Add a typed exact-equality implementation on `Observation`.**

### Comparative argument

- **Options 2 and 3** add a union or map schema, a second codec or a registry change. In
  exchange they gain nothing a consumer needs: progress is sparse, append-heavy and read in
  order, and human-readable JSON helps diagnosis.
- **Option 4** would make the comparison explicit, but it is a style refinement with no
  correctness gain.
- **The one real overlap** is the overflow handling, which belongs with CBO-002's shared marker
  and split rule.

### Revealing case

An Ipopt event with solver-specific metric keys and a NaN real fits the current document
unchanged. It does not fit a fixed relation schema.

### Disposition

**Retain current.** The relation-shaped premise is not supported by the data. Route the
progress writer's `full` flag and its halving through CBO-002's typed marker and split rule.

**Rule impacts:** none.

**Evidence:** Interface-checked: `durable.rs` (`ProgressBuffer`, the writer task,
`decode_progress`), `progress_documents.rs`, `reproducer.rs` (`ReproductionPredicate::matches`),
`pse-model/src/diagnostic.rs` (`Observation`, `ValidityLineage`).

---

<a id="cbo-012"></a>

## CBO-012 — Imperative nested-occurrence traversal beside the plan-based validation route

### Responsibility and target intent

Relation admission must find violating rows and nested occurrences, with exact paths for
sampled diagnostics and memory bounded by the pool.

### Inspection

The two routes do different jobs, as the library assessment found.
- `validate/occurrences.rs` performs **field-local** checks over visible values. It uses one
  DataFusion `PhysicalExpr` per node, applied over 4,096-row chunks with `take` or slicing. It
  covers unions, dictionaries, run-end encoding and tagged alternatives. Location coordinates
  are built only for retained samples (`Traversal::location`).
- `obligations/nested_values.rs` builds **cross-row** obligation plans. It already synthesizes
  ordinals with `range(0, array_length)`, unnested in lockstep.
- DataFusion 55.1.0 cannot help further here:
  - it has no `WITH ORDINALITY` (PR #22310 was closed unmerged);
  - its lambdas take no index parameter;
  - it cannot unnest unions or run-end-encoded storage;
  - `UnnestExec` reserves no memory, which conflicts with pool-bounded admission.

### Options

1. **Retain current.**
2. **Partial convergence** on `array_positions(array_transform(..))` for list-of-primitive
   fields at depth one.
3. **A full plan route.**

### Comparative argument

- **Option 3** would lose union and tagged-alternative visibility and pool-bounded memory.
- **Option 2** would split one field-local mechanism into two by depth and storage form, for an
  unmeasured gain.

### Revealing case

A tagged alternative inside a list cannot be unnested by the plan route, yet its visible-value
semantics must hold.

### Disposition

**Retain current.** The CHECK-text parser adapter in `predicates.rs` (the LOP-11 part) was not
inspected in depth and stays open for G3 or G8 if anyone owns it.

**Rule impacts:** none.

**Evidence:** Interface-checked: `validate/occurrences.rs` (`Node`, `Traversal`, `Selection`,
`CHUNK`), `obligations/nested_values.rs`, `prepared.rs` (chunked use), and the DataFusion
account's 55.1.0 negative results (inherited).

## Pattern notes

- **[LOP-X001](../assessment-patterns.md#lop-x001) refined (revision 3):** charge granularity (a container-sized
  charge where the library exposes exact part extents), stale-but-conservative derivations recorded
  beside their pinned version in one owner per quantity, and post-allocation reconciliation as a
  complement to prepayment, never a replacement.
- **[LOP-X009](../assessment-patterns.md#lop-x009) refined (revision 3):** map guards to trust boundaries. CBO-005's
  inverted placement, full preflight on digest-verified system-written files and the stock reader on
  import paths, is the prompting case, and the hostile-stream probe showed its consequence.
- **New [LOP-X013](../assessment-patterns.md#lop-x013): a library kernel's value semantics differ from the domain's.**
  Arrow's totalOrder comparison and extrema kernels against the recorded `f64::min`/`max` trajectory
  extrema (CBO-006) are the prompting case.

## Limits

- **Math and native sources.** I did not inspect them beyond the
  `pse-modeling/src/extent.rs` and `pse-math/src/execution.rs` samples. The CBO-001 table
  inherits those routes from the G5 and G6 library accounts.
- **Large-file reads.** I did not read `durable.rs` (6.9k lines) or `connected_results.rs`
  (4.4k lines) beyond the progress writer, the selection functions and the export metadata. I
  did not trace the review-pack index authentication or the portable-payload callers.
- **`owned_buffer.rs`** (1.76k lines) was read only for `attach_buffer`,
  `attach_borrowed_reserved` and `payload_wrapper_units`.
- **`validate/predicates.rs`.** I did not judge its Delta CHECK-text adapter.
- **DataFusion.** I did not re-check `ScalarValue` float ordering, because the DataFusion
  pruning route was rejected on placement grounds.
- **No measurements.** Every value claim is qualitative, and the over-admission figures are
  multipliers read from source, not measured peaks.
- **Census method.** The counts (sinks, `HeapUsage`, `*_bytes`) come from `rg` patterns over
  `crates/`, excluding `generated/` where stated. Treat them as approximate.
