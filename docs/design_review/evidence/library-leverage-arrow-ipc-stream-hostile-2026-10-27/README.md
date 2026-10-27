# Hostile Arrow IPC streams at arrow-ipc 59.3.0, and the replay/import decode paths

Probe evidence for the comparative library-leverage review, opportunity CBO-005: Arrow IPC
untrusted-input verification. It records what the stock stream readers do with lying length
fields, and traces which repository decode paths an externally supplied document can reach.

## Questions

1. **Library.** With arrow-ipc 59.3.0, what do `StreamReader` and the push decoder
   `StreamDecoder` do with a stream whose messages carry hostile length fields? Do they abort
   on allocation failure, panic, or return a typed error, and how much memory do they attempt?
   The hostile fields tested:
   - a record-batch `bodyLength` of 1<<50 or -1;
   - buffer and field-node entries that point far outside the body.
2. **Repository (static trace).** Is an exported replay document authenticated before the
   reproducer decodes the IPC it embeds? Can a user-supplied document reach `StreamReader`
   directly? What bounds and authentication come before the review-pack and portable-payload
   stream decodes?

## Versions and host

- Probe pins: `arrow-ipc`, `arrow-array` and `arrow-schema` at `=59.3.0`; `flatbuffers`
  `=25.12.19` and `libc` `=0.2.190`. All three match `[workspace.dependencies]`. The lock is
  the workspace `Cargo.lock` copied in; cargo pruned it to the probe closure, which resolves
  `arrow-buffer` 59.3.0 and `arrow-data` 59.3.0.
- Toolchain: rustc 1.101.0-nightly (c1070d693 2026-09-28), the release profile with
  `panic = "unwind"`.
- Host:
  - Linux 7.0.0-38-generic, x86_64, 32 logical CPUs;
  - 188 GiB RAM and 7 GiB swap;
  - `vm.overcommit_memory = 0` (heuristic);
  - `PSE_BUILD_MODE=unbounded`, so `scripts/pse-env` applies no OS memory cap.
- Memory cap: the uncapped runs (`cap=none`) used the pse-env default. The capped runs used
  `ulimit -v 4194304` (4 GiB of address space) inside the child shell. One extra run used
  `ulimit -v 1048576` (1 GiB).

## Method

`probe/src/main.rs`:
1. Writes one valid stream with `StreamWriter`: schema `x: Int64` (not null) and one batch
   of 4 rows. The batch has `bodyLength` 128 and buffers `[(0,1), (64,32)]`.
2. Splits the stream into framed messages.
3. Rebuilds the hostile message with arrow-ipc's own generated flatbuffer builders
   (`MessageBuilder`, `RecordBatchBuilder`, `convert::schema_to_fb_offset`), so every crafted
   message passes `root_as_message` verification.
4. Writes each message back with continuation framing and an end-of-stream marker.

A logging `#[global_allocator]` uses a raw `write(2)` to report every request of 1 MiB or
more before forwarding it. The attempted size is therefore visible even when the process
aborts.

Each case runs in a fresh child shell under each cap, in four modes:

| Mode | What it calls |
|---|---|
| `reader` | `StreamReader::try_new(File, None)`, then iterates the batches |
| `schema-take` | `StreamReader::try_new(File.take(meta_len + 8), None)`. This mirrors the repository's schema-only header reads. |
| `decoder` | `StreamDecoder::decode` with the whole input in one `Buffer`, then `finish()` |
| `decoder-chunk64` | The same, pushing 64-byte chunks |

Commands, run from the repository root:

```bash
S=<scratch>/probes/ipc-replay
E=docs/design_review/evidence/library-leverage-arrow-ipc-stream-hostile-2026-10-27/probe
scripts/pse-env -- cargo build --release --manifest-path $E/Cargo.toml --target-dir $S/target
scripts/pse-env -- bash $E/run.sh $S/target/release/arrow-ipc-stream-hostile-probe $S/cases > $E/outputs/runs.txt
```

Raw output:
- `probe/outputs/runs.txt`: 104 runs, with full stderr including the allocator reports and
  panic messages.
- `probe/outputs/summary.txt`: one line per run.
- `probe/outputs/meta-len-1gib.txt`: the extra 1 GiB-cap run.

## Outcome per case (Tested: commands above, host above)

Exit status 134 / signal 6 is an abort. Exit status 101 is a panic. "Typed" means an
`ArrowError` was returned and exit status was 0.

| Case (input bytes) | `StreamReader` (`reader`) | `schema-take` | `StreamDecoder` (both push modes) |
|---|---|---|---|
| valid (408) | 4 rows | schema ok | 4 rows, `finish` Ok |
| batch `bodyLength` = 1<<50 | **abort**: `alloc_zeroed` of 1,125,899,906,842,624 B, uncapped and capped | ok (the batch is never read) | typed: `finish` gives "Unexpected End of Stream". No large allocation. |
| batch `bodyLength` = 1<<38 (256 GiB) | **abort** uncapped and capped. Overcommit heuristic: larger than RAM+swap. | ok | typed, as above |
| batch `bodyLength` = 1<<33 (8 GiB) | uncapped: lazy 8 GiB `alloc_zeroed` succeeds, then typed `Io: failed to fill whole buffer`. 4 GiB cap: **abort**. | ok | typed, as above |
| batch `bodyLength` = -1 | **panic**: `MutableBuffer::from_len_zeroed` → `Layout::from_size_align(usize::MAX, 64).unwrap()` → `LayoutError`. Nothing allocated. | ok | typed: `finish` gives "Unexpected End of Stream" |
| **schema** `bodyLength` = 1<<50 | **abort** inside `try_new` (1 PiB attempted) | **abort**: `take` does not help | typed: "Unexpected End of Stream" |
| **schema** `bodyLength` = -1 | **panic** (`LayoutError`) inside `try_new` | **panic** | typed: "Unexpected End of Stream" |
| buffer `length` = 1<<40 | **panic**: `Buffer::slice_with_length` assertion "the offset of the new Buffer cannot exceed the existing length … selflen=128" | ok | **panic**: same assertion |
| buffer `offset` = 1<<40 | **panic**: same assertion | ok | **panic**: same assertion |
| node / batch `length` = 1<<40 | typed: "Need at least 8796093022208 bytes in buffers[0]" | ok | typed, same message |
| node / batch `length` = -1 | typed: "Need at least 18446744073709551615 bytes" | ok | typed, same message |
| 200,000 extra nodes and 400,000 extra buffers pointing outside the body (9.6 MB of metadata) | 4 rows: the extra entries are ignored. Allocation is bounded by the metadata actually present. | ok | 4 rows: the extra entries are ignored |
| metadata length prefix 0x7FFF_FFF0 (152-byte input) | `Vec::resize` reserves 2,147,483,632 B before reading, then typed `Io` EOF under no cap or the 4 GiB cap. **Aborts** under the 1 GiB cap. | ok | typed: "Unexpected End of Stream". It accumulates only supplied bytes. |

Findings:

- **`StreamReader` / `MessageReader::maybe_next` allocates before it reads** (*Tested*,
  confirmed in source). For every message, including the first (schema) message read by
  `try_new`, it calls `MutableBuffer::from_len_zeroed(message.bodyLength() as usize)`. It
  also calls `self.buf.resize(meta_len)` before `read_exact`.
  - The attempted size is exactly the advertised length.
  - A length above what the allocator or cap can satisfy aborts the process through
    `handle_alloc_error`, whatever cap is in force. On this host that is anything above
    about 195 GiB with no cap, or above the `ulimit` with one.
  - Below that size, the allocation is lazy, and the stream ends in a typed I/O error.
  - A negative `bodyLength` panics.
  - Wrapping the reader in `Read::take` does not bound any of this.
- **`StreamDecoder` never allocates from an advertised length** (*Tested*). Its message and
  body buffers grow only by the bytes supplied. A body that never arrives, including a
  negative `bodyLength` cast to `usize::MAX`, leaves the decoder in the body state, and
  `finish()` returns a typed `IpcError("Unexpected End of Stream")`.
- **Both readers share the record-batch decode, so both panic on out-of-body buffer spans**
  (*Tested*). `read_buffer` calls `Buffer::slice_with_length` without a bounds check, so an
  offset or length outside the body panics rather than returning an error. The layout check
  (spans within the body) therefore stays necessary even after moving to `StreamDecoder`.
- **Oversized node and batch lengths are caught by `ArrayData` validation** as typed errors
  (*Tested*). This is the default validation, which the probe does not skip.
- **Extra node and buffer entries are ignored** (*Tested*). Their cost is bounded by the
  metadata bytes actually supplied.

## Repository trace (Interface-checked: static source reading at 9cb8e4f4a; not run end to end)

### Reproducer / replay: no authentication before decode

Route from Python:
1. `pse._workflow.Runtime.run_reproducer(DiagnosticReproducer)`
   (`python/pse/_workflow.py`).
2. `crates/pse-py/src/workflow.rs` `run_reproducer` takes a
   `documents::RecordedInput<DiagnosticReproducer>`. Its `FromPyObject` encodes the value
   to JSON, caps it at `frozen_inspection::MAXIMUM_DOCUMENT_BYTES` (64 MiB) and decodes it.
3. `impl RecordedDocument for DiagnosticReproducer {}` (`crates/pse-py/src/documents.rs`)
   uses the default `validate_recorded`, which is a no-op.
4. `Runtime::run_reproducer` (`crates/pse-runtime/src/workflow/reproducer.rs`) runs the
   following, in order:
   - `allowance`;
   - `DiagnosticOperation::new`;
   - `extent` (encoded length ≤ 64 MiB);
   - `recorded_source_complete`;
   - `reproducer_physical`.

`recorded_source_complete` checks only these:
- the objects and physical sources are non-empty;
- names and logical names are unique;
- references resolve;
- the required logical objects are present;
- each `physical.ipc` starts with `FF FF FF FF` and ends with `FF FF FF FF 00 00 00 00`.

`reproducer_physical` does this:
1. Sums the actual `ipc.len()` (≤ 64 MiB).
2. Reserves `bytes × 16 + 64 KiB` from the math pool. The charge is based on the actual
   bytes, not the advertised lengths.
3. Looks up the relation in the registry.
4. Calls `StreamReader::try_new(Cursor::new(&physical.ipc), None)` and then `reader.next()`.

The identity premises are compared only after this decode:
- `ReplaySourceDocument::reconstruct` → `admit_replay_source`
  (`crates/pse-runtime/src/workflow/modeling/canonical.rs`) compares `document.build`,
  `interpretation` and `physical_identity` against the linked build and the decoded physical
  context.
- `FieldCheckedBatch::admit` (registry schema admission) also runs only after decode.

There is no digest, signature or MAC over the document or its `physical_sources` anywhere on
this route. `identity` and `physical_identity` are content hashes carried in the document
itself, and the decoded content is compared against them only after decode.

`ReproducerExtractionRequest::extract` → `DiagnosticReproducer::extract_review_case` runs
`ReviewContextDocument::validate` (structure, codec names and bounds; no digest). It then
copies `physical_sources` verbatim into the reproducer.

**A user-supplied replay document reaches the stock `StreamReader` directly.** The probe shows
what that does:
- A 408-byte embedded stream whose schema or batch `bodyLength` is 1<<50 passes every
  preceding check, then makes the process attempt a 1 PiB `alloc_zeroed` and **abort
  (SIGABRT)**. This takes down the hosting Python interpreter.
- A negative `bodyLength`, or a buffer span outside the body, **panics**. The future is
  driven on the calling thread by `blocking_on_with_interrupted_value` → `executor.block_on`,
  so the unwind reaches the PyO3 method trampoline. PyO3 normally converts that into
  `PanicException`; that conversion was not exercised here.
- A hostile metadata-length prefix makes the reader reserve up to 2 GiB before reading.

The project-bundle route reaches the same decode:
1. `ProjectBundleReader::open` checks each member's digest against `project.json` in the
   same directory. That is self-consistency, not authentication.
2. `ProjectBundleReader::reproducer(index)` hands an inline manifest reproducer to
   `Runtime::run_reproducer`.

### Portable payload and review packs: schema-only reads, still exposed through the schema message

`portable_payload::finalized_ipc_schema`
(`crates/pse-runtime/src/workflow/portable_payload.rs`) checks these, in order:
1. `readable_bytes ≥ 8`;
2. the final 8 bytes are the end-of-stream marker;
3. the file does not start with `ARROW1`;
4. the leading continuation word;
5. `0 < meta_len ≤ maximum_document_bytes` and `meta_len ≤ readable_bytes − 8`.

It then calls `StreamReader::try_new(file.take(meta_len + 8), None)`.

The batches are never decoded on this path, but `try_new` still calls
`from_len_zeroed(schema.bodyLength)`. A lying schema-message `bodyLength` therefore aborts
(1<<50) or panics (-1) inside this bounded header read. That is the `schema-take` rows above.

Callers:
- `ProjectBundleReader::open` → `validate_scientific_ipc`, after a manifest-relative digest
  check.
- `tooling/review_pack.rs` `PackReader::open` → `validate_encoding` → `finalized_ipc_schema`.
  This runs after these checks:
  - index/selection consistency;
  - `safe_relative`, a `members/` prefix and `open_contained`;
  - finalized/size boundaries.

  It runs with no digest check.

The review-pack preview (`tooling/review_pack/presentation.rs`, `orientation`) does this:
1. Skips the preview when `readable_bytes > maximum_document_bytes`.
2. Otherwise opens `open_contained(..).take(readable_bytes)` and iterates a stock
   `StreamReader` over at most the first 20 rows.

It is called from `finalize_reports` on a pack the process is assembling. A member there
already passed `validate_encoding` when it was written or opened. No authentication precedes
the preview, and `take` does not bound the advertised-length pre-allocation. A batch
`bodyLength` lie in a member therefore reaches the same abort or panic as the reproducer.
Whether a foreign member can reach `finalize_reports` without first being written by this
process was not traced.

## Limits

- The probe exercises arrow-ipc directly, not the pse runtime. The repository consequences
  are inferred from the static trace plus the library behaviour, and labelled
  *Interface-checked*.
- Abort thresholds depend on the allocator and the overcommit policy. Uncapped runs on this
  host abort above about RAM+swap. Lazily backed allocations below that succeed and end in a
  typed error. A deployment with `RLIMIT_AS`, a cgroup `memory.max` or `overcommit_memory = 2`
  aborts at much smaller advertised lengths.
- `fb_to_schema` panics on malformed schema flatbuffers were not probed.
- Compression codecs, dictionaries, view types, `with_skip_validation` and
  `with_require_alignment` were not probed.
- The other `StreamReader` call sites were not traced:
  - `result_physical.rs`;
  - `result_blocks.rs`;
  - `connected_results.rs`;
  - `fitting/oracle/lm_inquiry.rs`.
- arrow 60's stream body bounds were not tested; that version is outside the one type
  universe.
