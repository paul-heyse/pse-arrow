---
title: "Plan 57a: Result data and prepaid accounting"
status: in-progress
date: 2026-10-27
adrs: [ADR-0055]
review_sources: [docs/library_leverage/reviews/result-data-and-prepaid-accounting.md]
scenario_sources: [docs/library_leverage/reviews/result-data-and-prepaid-accounting.md]
doc_role: plan
doc_owner: docs/plans/57-library-leverage-and-correctness.md
doc_topics: [current-work, libraries, storage, numerical-execution, validation]
---

# Plan 57a: Result data and prepaid accounting

## Purpose and baseline

**Proposed:** replace repeated result-boundary machinery with one bounded realization per
responsibility, retaining scientific bits, admission, ownership and deterministic publication.
This companion develops CBO-001, CBO-002, CBO-005–008 and CBO-012. The
[coordinator](57-library-leverage-and-correctness.md#finding-dispositions) owns their scheduled
dispositions; [57b](57b-mathematics-and-structural-preparation.md) and
[57c](57c-native-solvers-and-arithmetic.md) supply the mathematics/native parts of CBO-001.

**Interface-checked:** authoring source `93946f572`, with product source unchanged from
`4f52fd239`; the consumed comparative review is `93946f572`, initial review `b29fcf802`.
Use base patterns 0.2, additional patterns revision 4, Core 3.4, efficiency heuristics 1.0,
Process Simulator 1.5 and the repository binding. The review's installed Arrow family 59.3.0
and DataFusion 55.1.0 remain the realization being planned. No dependency move or new product
test result is established by this document.

The [hostile-stream evidence](../design_review/evidence/library-leverage-arrow-ipc-stream-hostile-2026-10-27/README.md)
Tests Arrow behavior on its stated fixture; reachability through runtime replay/import is
Interface-checked. It does not test the replacement below. The existing force-validation
tests remain independent correctness obligations.

## Focused foundation assessment

`pse-columnar` already owns native representation, allocation leases and `ipc_file::encode`.
`AllocationScope::{attach_buffer, attach_reserved, attach_borrowed_reserved}` can transfer
pre-acquired grants to input buffers and separately owned decoder output. Reuse those owners;
do not install an Arrow-global pool, global allocator or another memory authority.

The runtime currently has a complete sealed-file preflight but retains a `FileReader` and
charges whole-file multiples to decode one batch. Other stream entry points can call the
stock reader before equivalent framing admission. This is a placement and granularity gap,
not evidence that checks or grants may simply disappear. The existing neutral columnar
boundary can own representation checks; registry meaning, scientific lineage and logical
selection remain with their consumers.

The result producer already uses estimate-first deterministic packing and one final bounded
encode. Preserve it. The apparently duplicated trial-encode route is unconsumed production
code, so deleting it is different from changing production split decisions. Columnar accessors
already expose the fields needed for dense indexing, avoiding row hydration without a new
relational planner. Dense descriptor pruning already belongs to the indexed canonical query.

## Target responsibilities and interfaces

The columnar integration supplies admitted IPC framing/decoder state and bounded encoded
output. Its interface consumes actual byte extents, an existing allocation scope/grant and
the representation policy; it returns owned native batches or typed refusal. Local names and
module arrangement remain implementation choices within that boundary. Extend the existing
`ipc_file` ownership rather than creating a second production file loader or writer.

The consumer supplies the schema/lineage admission it actually owns. Sealed result files use
registry schema equality; exported scientific streams additionally carry `pse.canonical.*`
metadata. A safe shared decoder must not apply bare-registry equality to every valid export.
Do not turn a parsed schema into scientific permission or treat self-consistent import hashes
as authentication.

Each allocation is covered before construction. A block's `metaDataLength + bodyLength` is
its input extent, not total decode memory. Cover rounded backing capacity, footer/block/schema
inventories, array/wrapper allocations, simultaneous inputs and any permitted copies. Use
actual ownership to avoid charging the same retained backing allocation for every slice.
Post-hoc size reporting can reconcile downward; it cannot substitute for prepayment.

The shared predicate owner validates scalar/dense selection once, lowers the same IEEE
semantics for actual values and encodes the existing export metadata. It does not move the
store's indexed descriptor selection to DataFusion or change extrema/identity conventions.

## Package state

| Package | Status | Owner | Completion |
|---|---|---|---|
| A01 | implemented; joined qualification pending | Result encoding and `pse-ids` | Typed bounded/counting sink adopted at all selected consumers; dead trial path deleted |
| A02 | implemented; joined qualification pending | Columnar framing and import consumers | Complete stream admission precedes decode on replay, pack and portable/physical input routes |
| A03 | implemented; joined qualification pending | Columnar file admission and runtime readers | Cached admitted file decoder and full allocation envelope replace duplicate reads/whole-file grants |
| A04 | implemented; joined qualification pending | Runtime result selection | Column-wise indexing and one predicate/metadata owner preserve current results |
| A05 | implemented selected footprint; joined qualification pending | Integration-specific bound owners | Selected result/container bounds have a single source-derived owner and all consumers move |
| A06 | implemented; joined qualification pending | Coordinator with retained-size owners | Bounded hand-accounting census selects an idiom or justified derive route; no crate adoption implied |

<a id="a01"></a>

## A01 — Typed sinks and deterministic packing

Use one `pse-ids` sink capability for count-only, bounded retained output and forwarding to an
inner writer. It accepts the selected bound and an optional checkpoint. Distinguish typed
`LimitReached`, cancellation/expiry and real I/O failures through their error payloads, using
`io::Error::other` plus downcasting through serde/Arrow error wrappers. Keep diagnostic codes
at the existing error owner; do not identify exhaustion by message strings or generic short write.

Move the selected counting/limited sinks in result encoding, replay, progress and boundary
serialization to that capability. Preserve hashing/framing sinks and their sticky failure as
separate responsibilities; forwarding can compose them without taking their authority.
Keep production `result_prefix_rows` estimate-first splitting, the current `div_ceil(2)`
fallback and final hard byte limit. Do not add a full `StreamEncoder::encode` size pass unless
a later measured frequency of hard-limit fallbacks justifies its additional traversal/scratch.

Delete the unconsumed `encode_result_prefix`/`visit_result_files_async` trial path and its
mechanism-specific tests after confirming all consumers. Replace the live `can_split` string
discriminator and progress `full` flag with the same typed cause. Retain progress's open-keyed
JSON observation representation (CBO-008); this package changes its bounded sink, not its wire
meaning or equality-by-serialization rule.

**Acceptance, planned:** targeted `pse-ids`, `pse-columnar` and runtime encoding controls cover
exact-bound success, one-byte overflow, checked count overflow, forwarded I/O failure, checkpoint
cancellation and distinction after serde/Arrow wrapping. Existing deterministic split fixtures
must keep ordinals/digests. Cancellation must never become a request to halve and retry.

<a id="a02"></a>

## A02 — Safe stream admission at the actual entry points

Build the shared encapsulated-message preflight before selecting a decoder. Validate length
sign/overflow, actual extent and continuation/end framing; verifier table/depth bounds; safe
schema/type conversion; node and buffer counts/spans; validity minima; supported metadata
version; zero-buffer value-slot limits; and the existing dictionary/compression policy.
Reject unsupported compression before `RecordBatchDecoder`: owning encoded bytes does not
cover advertised decompressed allocation. Do not rely on catching a panic or allocation abort.

Feed complete admitted message/body buffers to Arrow 59.3.0 `StreamDecoder`, keeping array
validation enabled. Its public API has no admission callback, so validation must precede each
decode call. Prefer complete messages over arbitrary chunks; if a consumer requires chunking,
derive and prepay the decoder's internal accumulation capacity while caller input remains live.
Check actual typed-buffer addresses against `arrow_data::layout`/`BufferSpec`; writer padding
does not establish the base alignment of embedded `Vec<u8>` data. Pre-admit a necessary aligned
copy or preserve a separately admitted realignment path without refusing formerly valid input.

Migrate `reproducer_physical`, portable-payload schema admission, review-pack validation/preview
and physical-object stream decoding. Trace their actual callers, including project-bundle
replay, rather than treating the direct library probe as coverage of the product route.
Keep required registry and scientific-lineage checks with the caller. After migration, remove
their replaced stock `StreamReader` construction and duplicate framing helpers; unrelated
trusted test fixtures do not become a second production loader.

**Acceptance, planned:** use disposable subprocesses for abort-sensitive cases. Actual runtime
and Python import/replay entry points must return typed refusal for giant/negative schema and
batch lengths, truncated metadata/body, out-of-body spans, malformed schemas, too-small validity
and unsupported compression/dictionaries. Valid exported metadata and unaligned embedded input
still decode under the declared contract. Interruption retains grants until all issued work
drains. The historical hostile-input probe is an independent case source, not the new oracle's
expected-result generator. Run only this package's targeted correctness closure while building it.

<a id="a03"></a>

## A03 — One admitted file read and a complete decode envelope

Reuse A02's representation checks for files. Admit the actual size/magic/footer before reading
advertised extents, inspect the footer with bounded verification, check block order/span/overlap,
and retain the admitted schema and block inventory. The cached file state becomes a
`FileDecoder` plus this admitted inventory, not a preflight followed by `FileReader` rereading
the same metadata. Maintain protection, file affinity, immutable descriptor/digest meaning and
the existing reader/lifetime boundary in `result_snapshot`.

Before reading a selected block, acquire its input-capacity grant and the complete residual
decoder allowance. Read once into appropriately aligned storage, attach its lease through
`AllocationScope::attach_buffer`, validate that message and call
`FileDecoder::with_require_alignment(true).read_record_batch` only when actual layout supports
it. Attach separate native allocations under their own reserved grant. Inventory retention,
alignment and simultaneous old/new cached state need coverage, not just buffer lengths.

Migrate `ResultBlockDecode::access_block`, `CachedFile`, standalone file decode and inline-file
consumers together with the relevant admitted-state interface. Do not remove the ×4 transient
or ×2 cached grants until each replacement's complete envelope is derived and exercised. Their
replacement may be conservative; the plan does not promise an exact total charge, a particular
capacity increase or a weaker failure threshold. Remove the replaced reader, duplicate footer
read, local width tables and obsolete mechanism fixtures in the same integrated package.

**Acceptance, planned:** retain `standalone_decode_preflights_once_and_preserves_ieee_payloads`,
decoder-cancellation/file-affinity controls and owned-buffer alignment/slice controls where they
test surviving contracts. Add one-batch access inside a maximum-size sealed file, multiple live
slices sharing one backing owner, copied inline blocks, metadata-heavy schemas, grant refusal
before allocation and last-reader release. Exercise footer/block corruption and incompatible
schema through actual consumers. A correct allocation test must observe pre-acquisition and
retained lifetime, not infer them solely from a final pool total.

<a id="a04"></a>

## A04 — Column-wise dense results and one predicate meaning

Use generated checked symbol/time column accessors with `arrow_ord::partition::partition` and
`Partitions::ranges` for window-local run boundaries; retain contiguity and neighbour checks.
Reuse the first pass's ranges for the accepted prefix instead of hydrating each row repeatedly.
Keep current `f64::min`/`max` folding and stored signed-zero/NaN interpretation. Reconciliation
with the relation layer's total-order extrema, or introducing canonical signed zero, is a
separate semantic decision and not required by this restructuring.

One result-selection owner represents scalar/dense intent, validates finite ordered bounds and
missing-value rules, and lowers it over the declared value/time field using the existing
signed-zero normalization and finite guarding. Slice the scalar coordinate window before
`NativeResultSelection::mask`, so row-proportional expression temporaries are admitted for the
relevant window rather than the complete batch. Use `BooleanBuffer` counts/ranges/indices and
`SelectedRows::{Range, Sparse}` as appropriate; prepay bitmap, range, index and expression
temporaries before construction. Remove per-row owner formatting, duplicated validations and
independent `pse.canonical.*` metadata interpretation. The shared codec must reproduce existing
metadata bytes. The canonical SurrealQL output-range index remains the descriptor pruning owner.

**Acceptance, planned:** compare against independently specified existing cases: both zero signs,
NaN sample time with/without bounds, missing scalar values, discontiguous owners, 1,024-row
window boundaries, accepted-prefix truncation, empty/all-selected masks, `u32` overflow and
export/readback of predicate bits. Keep field admission, coordinate bits and typed refusals.
Do not substitute Arrow float comparison/extrema kernels whose total-order semantics differ.

<a id="a05"></a>

## A05 — Residual bound ownership

Move result encoding/decoding estimates to the columnar integration, selection-expression and
pruning allowances to their execution owner, and conservative container geometry to `pse-model`.
Each selected quantity has one named derivation tied to the inspected supplier realization;
consumers call it rather than copying multipliers. Distinguish predictions, exact part extents,
complete conservative envelopes and post-hoc reconciliation. Keep the existing one-pool leases.
Writer bounds must account for 59.3.0 streaming bodies, sliced-input rebasing/bit-shift scratch and
retained capacity; their old 58.x copied-body rationale is not a current derivation.

Coordinate B03's Symbolica/B-tree consumers and C04's factor bounds under the same classification,
without moving their domain ownership into columnar code. Retain conservative bounds where no
supplier prediction exists. A KLU growth rule, allocation-time callback or RSS observation is
not a complete pre-allocation bound. Do not infer result-wide completion from one migrated site.

**Acceptance, planned:** representative shape/overflow/refusal controls at each owning crate,
consumer migration across the selected footprint, and version-derived scratch/capacity cases.
A03 covers decode accounting and A04 selection, so neither is reimplemented as a parallel A05
mechanism. Structural removal is observable from migrated callers; speed/memory gains remain
unmeasured unless a later decision-driven campaign is selected.

<a id="a06"></a>

## A06 — Settle the retained-size derive question

Classify the hand-written `HeapUsage` residue by meaningful shape, excluding already generated
implementations and unrelated `*_bytes` methods. Determine where exhaustive destructuring plus
shared conservative container implementations removes omissions, and where a derive would
actually delete repeated accounting decisions. Inspect B-tree/hash-container policy and manual
overrides before selecting a derive; a crate whose blanket container impl undercounts is not
made safe by optional per-field annotations.

The output is a reasoned choice with representative migrated shapes and the remaining manual
owners, not a permanent symbol census or new lint. Prefer the existing trait/container owner
and exhaustive idiom if that covers the actual residue. If a local derive is demonstrably
worthwhile, adding a proc-macro crate first needs the ADR/formal-review route and operator
selection; no such crate is authorized by this plan. A05 need not wait for that optional choice.

## Execution checkpoint — 2026-10-27

A01/A02 are integrated in `a15da43ee`: one typed sink preserves forwarded causes, and shared
IPC message admission checks framing, verifier limits, schemas, spans and complete bodies before
Arrow materialization. Escaping batches retain schema and working grants. The selected pure
sink, IPC and runtime-refusal controls passed; assembled replay/import/export remains with J.
A03 is integrated in `9fe4dbb31`; its final columnar/runtime selection passed 22 focused
controls. A04 is integrated in `c3e2881a6`, with twelve pure selection/metadata/index controls
passing. Actual canonical, imported and Python readback stays with J.

A05 has moved the selected B-tree geometry to `pse-model::allocation`; structural/compiler/math
consumers and the non-columnar construction fixes are integrated. Native and result-specific
allowances remain with their respective allocating/integration owners, rather than a universal
container estimator. A06 found five handwritten trait owners outside generated implementations:
model Bytes, BoundaryDiagnostic and NumericalStrategy, admission ModelingRows, and modeling
DeclarationReferences. They now destructure exhaustively and share conservative container
accounting. These shapes do not justify a derive crate. No blanket hash-container implementation
was added; foreign handles and predicted scientific storage retain their distinct policies.
No measured speed or memory improvement is claimed.

A05's writer/header contribution is integrated in `c9164a7d7`. It prepays current supplier
preparation, rebasing/bitshift/generated validity, footer populations and output capacity
reallocation overlap, and migrates the selected relation, artifact, study and source-stream
writers. The final three-root force-validation selection passed 49 controls without warnings.
`algorithm_decode_extent` and `columnar_validation_extent` remain named conservative execution
owners, not supplier predictions. The durable publication-response composite retains separate
transport custody; generic arbitrary alias-header accounting is a remaining investigation,
not a claim of globally complete `FieldCheckedBatch` accounting. Selected snapshot slices and
source-writer wrappers have explicit prepayment. CBO-001's overall disposition remains with
the coordinator rather than being closed from this selected writer contribution.

## Dependencies, retained scope and qualification

A01 is independently useful. A02 supplies shared message safety; A03 consumes it and supplies
the raw/residual decode envelope to A05. A04 may proceed independently of the file migration
while coordinating the metadata codec with I01 and H's generator contracts where actually
needed. B03/C04 own their non-columnar residuals. G01 supplies no-clobber publication at the
file boundary without changing A03's semantic admission; E's transport changes must retain
the same borrowed-reservation ownership when delivering native batches.

CBO-008 retains heterogeneous JSON progress and its exact equality behavior; only its sink
joins A01. CBO-012 retains field-local nested visible-value traversal. Its CHECK-text adapter
was not fully investigated: inspect that adapter only when a concrete grammar/version change
or failure makes it relevant, rather than replacing it with a full relation plan. Arrow 60
remains a future family-major route requiring compatible DataFusion and the decision process;
do not introduce another Arrow universe to simplify this series.

Under confirmed U1, retain conditional arrow-rs improvements: typed `read_buffer` span errors,
footer-length/file-size checks and decoder-level verifier options. No submission is authorized.
Revisit individual local guards only after a released compatible family supplies the same
protection and targeted malformed-input controls pass; one fixed hazard does not discharge
the remaining admission/lineage/allocation contract or justify a second Arrow universe.

Compile the affected package and run targeted tests with explicit Arrow force-validation using
the repository recipes. [57j](57j-qualification-and-publication.md) owns the single scope-end
assembled journeys, static/manual checks and final publication. No product checks ran for this
authoring; the evidence above retains its original scope. Enduring result/storage wording,
including the stale blueprint §19.2 description, moves through the architecture route at J's
publication boundary, not by an incidental edit here.

## Outcome — implementation and functional acceptance, 2026-10-27

**Implemented:** A01–A06 deliver the selected footprint. Typed sinks replace duplicated
count/limit decisions; stream framing precedes Arrow materialization; admitted file inventories
and indexed block decoding replace duplicate footer reads and whole-file decode grants.
Selection shares predicate/metadata meaning and reads original columns. Current writer
preparation, retained capacity and shared B-tree geometry have allocating owners. The five
handwritten retained-size owners are exhaustive; no derive crate is needed.

**Tested:** the selected canonical/native command `just unit-native-package pse-runtime
'canonical-tests,native-solvers'` with J01's recorded selection passed **33/33**, including all
eleven result-data journeys, on the current force-validated closure. The five selected Python
journeys have composed passing evidence against the rebuilt native correctness extension.
[57j's Outcome](57j-qualification-and-publication.md#selected-a-c-outcome--2026-10-27) owns exact
invocations, original failures, conditions, drain evidence and pending scope-end checks. Earlier
49-control writer/header acceptance retains its recorded targeted scope.

A mistake corrected during joined preparation was the canonical pruning control's stale
`FileReader.next` call. The migrated test now uses admitted indexed decoding, preserves original
schema/content checks and proves the skipped-decode behavior with existing counters: two observed
pruned blocks and zero native decodes before independent readback. Its original compile failure
ran no acceptance tests.

Deliberately retained: estimate-first splitting, heterogeneous progress JSON, field-local checks,
named conservative execution allowances and the separate durable publication/transport composite.
Generic arbitrary `FieldCheckedBatch` alias-header accounting remains an investigation under
CBO-001. The selected writer/snapshot/source improvements do not establish globally complete
accounting. Benefits beyond deleted duplicate work remain unmeasured. Enduring result contracts
are published through blueprint revision 298; final selected closure awaits 57j's remaining
feature/manual and publication work.
