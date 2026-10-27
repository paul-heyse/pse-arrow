# Apache Arrow IPC (arrow-ipc)

Realizations considered: installed `arrow-ipc` 59.3.0 (with `flatbuffers` 25.12.19; lz4/zstd
codecs can be feature-unified in by a query engine); maintenance releases 59.3.1 (2026-10-10) and
59.3.2 (2026-10-22), neither touching the IPC reader's bounds; upgrade route `arrow-ipc` 60.0.0
(released 2026-09-10/15, a family major); unreleased `main` of apache/arrow-rs as fetched
2026-10-27; open PR apache/arrow-rs#11012.
Research date 2026-10-27. Sources: installed `arrow-ipc-59.3.0/src/{reader.rs,reader/stream.rs,
writer.rs,convert.rs,compression.rs}`; `arrow-buffer-59.3.0/src/buffer/{immutable.rs,mutable.rs}`;
`arrow-data-59.3.0/src/data.rs`; `flatbuffers-25.12.19/src/verifier.rs`; arrow-rs release notes
59.3.1, 59.3.2, 60.0.0 (`gh release view`); PRs #10522, #10647, #10744, #11044, #11658 (merged)
and #11012 (open); issue #10521; `arrow-ipc/src/reader.rs` on `main` (GitHub contents API,
2026-10-27); Context7 `/apache/arrow-rs` (FileReader structure only; no bounds guidance found).

## Types

### arrow-ipc/untrusted-file-admission
- **Need it serves:** admit an Arrow IPC file or stream whose bytes are not trusted (stored by a
  previous process, imported, transported) and refuse corrupt or hostile framing with a typed
  error before any allocation sized by the input's own metadata.
- **Recognizable code shapes:** a hand-written footer/block/message preflight run before
  `FileReader`/`StreamReader`; manual `root_as_footer_with_opts` / `root_as_message_with_opts`
  calls; per-type tables of buffer counts and minimum buffer lengths; tests named "corrupt footer
  length", "oversized body length"; `catch_unwind` around a reader.
- **Mechanisms and APIs (what the stock high-level readers do at 59.3.0):**
  - `FileReaderBuilder::{new, with_projection, with_max_footer_fb_tables,
    with_max_footer_fb_depth, build}`. `build` reads the 10-byte trailer, then
    `vec![0; footer_len]` **before** seeking to `End(-10 - footer_len)`, so a lying footer length
    up to `i32::MAX` allocates (zeroed) before the seek fails. Footer FlatBuffer verification is
    configurable for tables and depth only (`max_apparent_size` stays at the 2 GiB default).
    `footer.schema().unwrap()` panics on a verified footer without a schema table.
  - `read_block` (used by `FileReader::next`/`set_index`): `bodyLength().to_usize().unwrap()`,
    `metaDataLength().to_usize().unwrap()` and `checked_add(..).unwrap()` panic on negative or
    overflowing block metadata; `MutableBuffer::from_len_zeroed(total_len)` allocates the
    advertised size before reading (an implausible size aborts the process via
    `handle_alloc_error` or panics on `Layout`), and no check compares the block span with the
    file size.
  - `StreamReader` (`MessageReader::maybe_next`): `buf.resize(meta_len)` (up to `i32::MAX`,
    zeroed) and `MutableBuffer::from_len_zeroed(message.bodyLength() as usize)` before reading the
    bytes; a negative `bodyLength` wraps to a huge length. Upstream issue #10521 reports
    `SIGABRT` from single bit flips. Wrapping the reader in `Read::take` does not help: the
    allocation precedes the read.
  - `RecordBatchDecoder` (shared by all readers and decoders): `read_buffer` calls
    `Buffer::slice_with_length(offset, length)`, which **asserts** the span is inside the body
    (panic, not error); `BooleanBuffer::new(buffer, 0, len)` for struct validity and inside
    `ArrayDataBuilder::build` asserts the validity bitmap is long enough (panic). Message
    metadata is verified with `root_as_message` and the default verifier options (1,000,000
    tables, depth 64, 2 GiB apparent size); there is no option to tighten them on the decoder.
  - `convert::fb_to_schema` is infallible and contains `panic!`/`unimplemented!`/`unwrap` on
    verifier-accepted but malformed schemas (unknown precision, `Type NONE`, list without child).
  - Decompression (`CompressionCodec::decompress_to_buffer`) allocates the advertised
    uncompressed length when a codec feature is enabled.
  - What the stock decode path does discharge: FlatBuffer structural verification (footer with
    configurable tables/depth; messages at defaults); full per-array validation through
    `ArrayDataBuilder::build` → `validate_data` (`validate` + `validate_nulls` + `validate_values`
    — buffer sizes against declared lengths, offsets monotonic and in bounds, UTF-8, null counts)
    unless the caller sets the `unsafe` `with_skip_validation`; `RecordBatch` row-count
    consistency; endianness refusal; version mismatch refusal; decoded buffers are zero-copy
    slices of the read block (except alignment copies, below).
- **Obligations discharged:** structural FlatBuffer safety (no out-of-bounds reads of metadata);
  semantic validity of decoded arrays; refusal of missing nodes/buffers ("Buffer count mismatched",
  "node not found"); refusal of unknown message types.
- **Residual caller obligations (at 59.3.0) for panic-free, prepay-before-allocation admission:**
  footer length versus actual file size; block offsets/lengths non-negative, inside the file, in
  order; metadata length ≥ 8 with continuation marker; every buffer span inside its message body;
  validity bitmap ≥ `ceil(len/8)` when `null_count > 0`; schema pre-walk (or equivalent) before
  `fb_to_schema`; refusal of compression and dictionaries if not admitted by policy; message
  verifier limits if tighter than defaults (call `root_as_message_with_opts` first); exact trailing
  node/buffer count (the decoder does not refuse surplus nodes or buffers); length bounds on
  zero-buffer fields (`Null`), which validation cannot tie to body bytes.
- **Semantic conditions:** a panic under `panic = "unwind"` can be caught but is not a typed
  refusal and may leave partial state; an allocation failure from `handle_alloc_error` aborts and
  cannot be caught. `vec![0; n]` maps lazily on Linux, so a 2 GiB footer allocation may not touch
  RSS, but it still counts against address-space limits and is an allocation the contract forbids.
- **Version / build conditions:**
  - 60.0.0 (upgrade route): #10522 bounds `StreamReader` pre-allocation (negative `bodyLength`
    becomes a parse error; at most 64 MiB reserved before bytes arrive); #10647 adds
    `convert::try_fb_to_schema` and makes all reader paths use the fallible conversion; #10744
    returns an error for a footer without schema; #10641 moves `read_block` to
    `MutableBuffer::try_from_len_zeroed` (allocation failure becomes `ArrowError::MemoryError`, but
    the advertised size is still requested).
  - `main`, unreleased as of 2026-10-27: #11044 and #11658 return errors for undersized validity
    buffers (ArrayData and struct). `read_buffer` still asserts; `read_block` still unwraps block
    metadata and sizes from it; `build` still allocates `vec![0; footer_len]` before the seek.
  - Open PR #11012 ("validate block lengths before allocation") would add file-size checks for
    blocks; not merged.
- **Limits and negative results:** no allocation cap, size hook or `max_apparent_size` option on
  `FileReaderBuilder`, `FileDecoder`, `StreamReader` or `StreamDecoder` at 59.3.0 or on `main`
  (search scope: the four types' public methods in installed 59.3.0 and `main` reader.rs). The
  high-level `FileReader` and `StreamReader` therefore cannot by themselves admit hostile input
  without prior framing checks at any released version.
- **Evidence:** Interface-checked: installed sources above at 59.3.0 (line-level reading of
  `build`, `read_block`, `MessageReader::maybe_next`, `read_buffer`, `create_struct_array`,
  `ArrayDataBuilder::build`, `fb_to_schema`); upstream PR bodies and their regression tests (#10522
  reports `i64::MAX`, `1 << 50`, `-1` body lengths panicking or aborting before the fix). No local
  probe was run: static reading settled the allocation order (see README).
- **Measured stream behaviour (added 2026-10-27):** a later hostile-stream probe at 59.3.0
  (*Tested*, release profile with `panic = "unwind"`, Linux x86_64 with heuristic overcommit,
  uncapped and under `ulimit -v`; [evidence](../../../design_review/evidence/library-leverage-arrow-ipc-stream-hostile-2026-10-27/README.md)) confirmed the
  static reading for `StreamReader` and extended it:
  - The pre-allocation applies to the *schema* message as well, inside `StreamReader::try_new`, so
    a header-only read of the schema is exposed too. Wrapping the input in `Read::take` bounds
    nothing.
  - A `bodyLength` above what the allocator or address-space cap can satisfy aborts the process
    (`alloc_zeroed` of exactly the advertised size, for example 1 PiB for `1 << 50`); below that,
    the lazy allocation succeeds and the read ends in a typed I/O error. A negative `bodyLength`
    panics on `Layout::from_size_align(usize::MAX, 64).unwrap()` without allocating. A metadata
    length prefix near `i32::MAX` reserves about 2 GiB before reading.
  - A buffer offset or length outside the body panics in `Buffer::slice_with_length` for every
    reader and decoder, because they share `read_buffer`.
  - Oversized node or batch lengths (including `-1`) are refused as typed errors by default
    `ArrayData` validation. Surplus node and buffer entries are ignored, and their cost is bounded
    by the metadata bytes actually supplied.
  - Not probed: `fb_to_schema` panics on malformed schemas, compression, dictionaries, view types
    and the skip-validation and alignment options.

### arrow-ipc/caller-owned-push-decoding
- **Need it serves:** decode IPC while the caller owns every input-sized allocation, so admission
  (memory reservation, bounds) can precede it, and reuse bytes the caller already read during
  checks.
- **Recognizable code shapes:** a preflight that reads footer and per-block metadata and then
  hands a `Read + Seek` to `FileReaderBuilder`, which reads the footer and every block again;
  manual concatenation of stream bytes before `StreamReader`.
- **Mechanisms and APIs:**
  - `reader::read_footer_length([u8; 10])`, `root_as_footer_with_opts`, `convert::fb_to_schema`
    (or `try_fb_to_schema` from 60.0.0), `FileDecoder::new(schema, version)`,
    `with_projection`, `with_require_alignment(bool)`, `read_dictionary(block, &buf)`,
    `read_record_batch(block, &buf) -> Option<RecordBatch>`. The caller reads each block into a
    `Buffer` of its own making (for example `MutableBuffer` sized after a reservation), so the
    decoder never allocates from untrusted lengths; the block must start with the encapsulated
    message (continuation + length + metadata) followed by the body.
  - `StreamDecoder::{new, with_require_alignment, decode(&mut Buffer), finish}`: push-based;
    message and body bytes are accumulated only as supplied (`extend_from_slice`), and a body that
    is fully present in the input buffer is sliced zero-copy. Advertised lengths do not cause
    pre-allocation. Ends with `finish()` refusing partial data.
  - `with_require_alignment(true)` turns a misaligned buffer into an error instead of a silent
    aligned copy (the default `false` copies, an allocation up to the buffer's size).
- **Obligations discharged:** message parsing, version agreement, node/buffer walking, array
  construction and full validation, schema/dictionary bookkeeping, projection.
- **Residual caller obligations:** all framing checks listed under
  `arrow-ipc/untrusted-file-admission` that guard panicking paths (buffer spans, validity lengths,
  schema conversion at 59.3.0); reading blocks; refusing compression if decompression allocation
  must not be advertised-size; bounding zero-buffer field lengths.
- **Semantic conditions:** `FileDecoder::read_record_batch` slices `buf` at
  `block.metaDataLength()` with an assertion, so the caller must pass a buffer at least that long.
  Decoded arrays retain the caller's block buffer (zero-copy), so the reservation for the block
  covers the batch's retained memory, plus small `ArrayData` headers. `StreamDecoder` at 59.3.0
  still calls `fb_to_schema` (panics on malformed schemas) and `header_as_schema().unwrap()`.
  Further conditions (added 2026-10-27):
  - `StreamDecoder` never pre-allocates from `bodyLength` (*Tested* by the hostile-stream probe
    cited above): a lying or negative length leaves the decoder waiting for body bytes, and
    `finish()` refuses with `IpcError("Unexpected End of Stream")`. It still panics on buffer spans
    outside the body, through the shared `read_buffer`, so a span check stays a caller obligation.
  - Alignment: the IPC writer pads to `IpcWriteOptions` alignment (default 64), and
    `arrow-buffer` `ALIGNMENT` is 64 on x86_64, so a block read into a fresh `MutableBuffer`
    decodes with `with_require_alignment(true)`. Blocks sliced from one whole-file `Vec<u8>` are
    aligned only to 8 bytes relative to its base. Upstream `arrow/examples/zero_copy_ipc.rs`
    shows the mmap variant (Context7 listing; not in the crate tarball).
  - `Bytes::claim` (arrow-buffer feature `pool`) replaces any reservation already attached, and
    enabling `pool` adds a `Mutex<Option<Box<dyn MemoryReservation>>>` to every `Bytes` in the
    family build, so a caller-owned reservation attached to its own buffer is lighter.
- **Version / build conditions:** `FileDecoder` since PR #5249 (merged 2023-12-31); `StreamDecoder` long
  available; both present at 59.3.0 and 60.0.0 with the same signatures.
- **Limits and negative results:** no decoder-level message verifier options; no decoder check
  that nodes/buffers are fully consumed.
- **Evidence:** Interface-checked: installed `reader.rs` (`FileDecoder`, `read_record_batch`) and
  `reader/stream.rs` (`StreamDecoder::decode`) at 59.3.0; the `FileDecoder` doc example
  (`read_footer_length` + `root_as_footer` + `fb_to_schema`).

### arrow-ipc/encoded-size-without-body-copy
- **Need it serves:** know the exact IPC encoded size of a batch (or of a row range) before
  committing output, without copying the body into a growing writer and without a mirrored cost
  model of the writer's padding and truncation rules.
- **Recognizable code shapes:** encode into a size-limited `io::Write`, match the limit error, halve
  the row count and retry; hand-written "visible size" functions that mirror writer truncation.
- **Mechanisms and APIs:** `writer::StreamEncoder::{try_new, try_new_with_options, encode(&batch)
  -> Vec<Buffer>, finish}`. Internally `IpcDataGenerator::encode_to_buffers` with the `Buffers`
  sink pushes each body buffer as `EncodedBuffer::Raw(buffer)` — a reference-counted slice of the
  source array's buffer — while metadata, continuation prefix and padding are small owned
  buffers. Summing `Buffer::len()` gives the exact framed message size the writer would emit with
  the same `IpcWriteOptions` (alignment, metadata version). Owned allocations in the trial are the
  FlatBuffer metadata, padding, and any re-based offsets or bit-shifted validity for sliced inputs
  (proportional to visible rows of variable-width/nested columns, not to body bytes); compression
  would allocate compressed output.
- **Obligations discharged:** exact padding (default alignment 64), truncation of sliced buffers,
  list-child slicing and FlatBuffer metadata size, all from the writer itself.
- **Residual caller obligations:** the split search (which row range to try); file-format
  overhead outside stream messages (8-byte magic+padding, footer with schema and one `Block` per
  batch, 10-byte trailer) — `FileWriter` exposes no sizing hook, so footer size is a bounded
  estimate or a final measurement; bounding the trial's own small allocations if they must be
  prepaid.
- **Semantic conditions:** a stream record-batch message has the same encapsulated bytes as the
  file-format record-batch message for the same options, so per-batch sizes transfer; the schema
  message is emitted once on the first `encode`.
- **Version / build conditions:** `StreamEncoder` is present at 59.2.0 and 59.3.0 and absent at
  58.3.0 (installed-source check); it is not in 57/58 maintenance lines. The writers changed at
  the same boundary (added 2026-10-27): at 59.2.0 and 59.3.0, `FileWriter::write` and the stream
  writer emit record-batch bodies zero-copy to the sink (`IpcDataGenerator::write` →
  `write_to_sink` with `IpcBodySink::Collect`), whereas at 58.3.0 `FileWriter::write` copied each
  body into `EncodedData.arrow_data`. Owned allocations at 59.x are the sink itself, the metadata,
  and re-based offsets or bit-shifted validity for sliced inputs. A writer allowance derived
  against the 58.x copy is still conservative at 59.x, but its stated derivation is stale.
- **Limits and negative results:** arrow-ipc 59.3.0 has no size-estimation function that avoids
  encoding (search scope: public items of installed `writer.rs`); `encoded_batch` is deprecated in
  favour of `encode`. The trial still walks every column once per attempt.
- **Evidence:** Interface-checked: installed `writer.rs` 59.3.0 (`StreamEncoder`,
  `encode_to_buffers`, `impl IpcMessageSink for Buffers`, `IpcBodySink::Collect`). Not exercised.

## Current applicability

### [CBO-005](../../../library_leverage/codebase/result-data-paths.md#cbo-005) — Arrow IPC untrusted-input verification written by hand
- **Types:** `arrow-ipc/untrusted-file-admission`, `arrow-ipc/caller-owned-push-decoding`,
  `arrow-data/validation-and-layout` (see `arrow-data.md`).
- **Realization:** installed 59.3.0. Upgrade route 60.0.0 moves the whole arrow family and, by the
  one-type-universe rule, needs a DataFusion release on arrow 60; DataFusion `main` is on arrow
  60.0.0/object_store 0.14.2 but the latest release, 55.2.0, is on arrow 59.2 — so the 60 route is
  development-only for this family as of 2026-10-27, and a family major needs an ADR here.
- **Conclusion:** fits with conditions (partial). At 59.3.0 the stock reader does **not** refuse
  hostile framing before allocation (footer length, block lengths) and panics on several
  verifier-accepted inputs, so the codebase preflight is justified in substance. `FileDecoder` fed
  with the blocks the preflight already reads can retire the `FileReaderBuilder` re-read and let
  Arrow's own validation own value checks; most of the framing preflight remains until upstream
  fixes ship.
- **What it would discharge / what remains local:**
  - Discharged by `FileDecoder` + `ArrayDataBuilder` validation: message parsing, node/buffer
    walking into arrays, value validation, and the second read of footer and blocks that
    `open_file_reader` now performs after `preflight` (`result_files.rs`).
  - Remains local at 59.3.0: file byte bound, magic, footer length versus file size, block spans
    and order, metadata length and continuation, buffer spans in body, validity-bitmap minima
    (the comment above `result_blocks::layout` names exactly this assertion hazard), schema
    pre-walk before `fb_to_schema`, no-compression/no-dictionary/V5 policy, surplus node/buffer
    refusal, `Null`-field and total value bounds, exact registry-schema equality.
  - The per-type minimum-length table in `layout` could read widths from
    `arrow_data::layout(&DataType)` (`BufferSpec::FixedWidth { byte_width }`, `BitMap`,
    `VariableWidth`) instead of restating them (see `arrow-data.md`), keeping only the node
    traversal local.
  - With arrow 60 (when available): `try_fb_to_schema(..)? == expected` subsumes the type
    whitelist in `schema_field` because the expected registry schema is trusted and dictionaries
    convert to `Dictionary` types; footer-schema-absent and stream body-length guards move
    upstream. With unreleased `main`: validity minima move upstream. Buffer spans, footer length
    and block bounds remain local on every inspected version (#11012 open).
- **Prerequisites, configuration or extension:** read blocks into a reserved `MutableBuffer`;
  `with_require_alignment(true)` so a misaligned buffer is refused rather than copied (the
  codebase's own writer uses 64-byte alignment); keep `root_as_message_with_opts` before decode to
  retain the tighter table/apparent-size limits.
- **Semantic gaps against preservation constraints:** typed errors become `ArrowError` strings
  for anything delegated; the codebase would map them to its error family (diagnostic codes on
  refusals pinned by tests may change). Deterministic bytes and schema equality are unaffected.
- **Missed manifestations (premise sharpening):** several production `StreamReader::try_new`
  sites decode IPC without the preflight: `workflow/result_physical.rs` (stored physical payload
  bytes), `tooling/review_pack/presentation.rs` (pack member file wrapped in `take`), and
  `workflow/portable_payload.rs` (schema-only read, bounded by `take(length + 8)` — this one is
  guarded for allocation but still reaches `fb_to_schema`). At 59.3.0 these are exposed to the
  `bodyLength` pre-allocation abort and `fb_to_schema` panics unless their inputs are trusted by
  a prior integrity check. `StreamDecoder` would remove the advertised pre-allocation at the pin.
- **Unsettled premises and what would settle them:** whether the inputs of those
  `StreamReader` sites are content-verified before decode (trace their callers); whether a
  maintenance backport of #10522/#10647 to 59.x is planned (none in 59.3.1/59.3.2 notes).
- **Evidence:** Interface-checked (sources above); application Proposed.

**Reviewer correction (2026-10-27).** The caller trace and the hostile-stream probe settle the
first premise, and they correct the statement that `portable_payload.rs` is guarded for
allocation: `StreamReader::try_new` pre-allocates the schema message's `bodyLength` even inside
`take(length + 8)`. By site:
- `result_physical.rs` decodes digest-verified, system-written store objects (staging assembly
  checks each block digest and the payload digest).
- The reproducer decodes IPC embedded in an exported replay document, which reaches the stock
  `StreamReader` with no digest, signature or MAC; it checks only continuation and end-of-stream
  bytes and charges `bytes×16 + 64 KiB`. A lying schema or batch `bodyLength` aborts the hosting
  interpreter, and a negative length or out-of-body span panics. Project-bundle reproducers reach
  the same decode after a self-consistency digest check only.
- `portable_payload::finalized_ipc_schema` and the review-pack preview are exposed through the
  schema message; the preview's `take` does not prevent pre-allocation.
- `pse-schema/src/builder/products.rs` also uses `StreamReader` (not traced); the
  `connected_results.rs` and `lm_inquiry.rs` sites are tests.

The comparative review recommends one IPC admission built on `FileDecoder` / `StreamDecoder` at
every trust boundary, with caller-reserved 64-byte-aligned block buffers attached to the existing
allocation scope, widths taken from `arrow_data::layout`, the double read and whole-file charges
removed, and every check 59.3.0 still needs kept (*Tested* library behaviour, *Interface-checked*
routes; [evidence](../../../design_review/evidence/library-leverage-arrow-ipc-stream-hostile-2026-10-27/README.md)). Arrow 60 is not reachable until DataFusion
moves to it. See the [comparative review](../../reviews/result-data-and-prepaid-accounting.md#cbo-005).

### [CBO-002](../../../library_leverage/codebase/cross-cutting-mechanisms.md#cbo-002) — Bounded and counting output sinks; trial-encode-and-halve (Arrow part)
- **Types:** `arrow-ipc/encoded-size-without-body-copy`, `arrow-data/memory-size-functions`.
- **Realization:** installed 59.3.0.
- **Conclusion:** fits with conditions for IPC-sized splitting: `StreamEncoder::encode` gives the
  writer's exact message size while body buffers stay zero-copy, which can replace the
  copy-into-`LimitedWriter`-then-halve loops (`result_blocks::visit_blocks_async`,
  `result_files::result_prefix_rows`) with a measured search; there is no released API that
  predicts size without encoding.
- **What it would discharge / what remains local:** discharges padding/truncation mirroring and
  the per-attempt body copy; the split policy, deterministic split points, the error
  discrimination (today matched on the string `"sealed result file limit"` / `"result block
  limit"`) and file footer overhead remain local. The JSON and generic `io::Write` sinks of
  CBO-002 are outside this library.
- **Prerequisites, configuration or extension:** encode with the same `IpcWriteOptions` the sealed
  writer uses; derive file size as 8 + schema message + Σ record messages + footer + 10, with a
  bounded footer estimate or a final `LimitedWriter` check retained as the hard limit.
- **Semantic gaps:** a sum-of-messages search can choose different split points from halving;
  where splits become durable block boundaries (ordinals, digests), the selected rule must be
  fixed deliberately.
- **Unsettled premises:** whether the trial's owned allocations (re-based offsets for sliced
  variable-width columns) need their own prepay; cost has not been measured.
- **Evidence:** Interface-checked; application Proposed.

**Reviewer correction (2026-10-27).** "Replace copy-into-`LimitedWriter`-then-halve loops"
overstates the live footprint. Production result-file splitting is `result_prefix_rows`, which
halves over the `batch_extent` estimate without encoding, followed by the hard bound in
`AdmittedResultFile::encode`; `visit_blocks_async` encodes only after its estimate passes. The
trial-encode path (`encode_result_prefix`, `visit_result_files_async`) has no production caller,
and the `"sealed result file limit"` match is reachable only from it. The comparative review
retains estimate-first splitting and adopts `StreamEncoder` sizing only if measurement shows
frequent hard-bound fallbacks. See the [comparative review](../../reviews/result-data-and-prepaid-accounting.md#cbo-002).
