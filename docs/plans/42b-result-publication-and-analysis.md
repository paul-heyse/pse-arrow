---
title: Result publication and analysis
status: done
date: 2026-10-16
adrs: []
review_sources:
  - docs/design_review/reviews/design_review_post-plan41-simulation-efficiency-and-diagnostics_2026-10-16.md
scenario_sources:
  - docs/design_review/reviews/design_review_post-plan41-simulation-efficiency-and-diagnostics_2026-10-16.md
doc_owner: docs/plans/42-post-plan41-simulation-efficiency-and-diagnostics.md
doc_topics: [design, storage, validation, libraries]
---

# Plan 42b: Result publication and analysis

## Purpose and foundation assessment

This companion owns F01/F05 and inquiries I01/I02/I07 for [Plan42](42-post-plan41-simulation-efficiency-and-diagnostics.md).
It makes supported result analysis usable on an ordinary caller stack and separates homogeneous
file packing from bounded run-result metadata submission. It does not change scientific response
meaning, publication consistency, numerical methods or the store protocol. The coordinator owns
finding dispositions and combined acceptance; B-package progress is recorded here.

**Interface-checked — 2026-10-16:** [the publication/caller inquiry](../design_review/evidence/post-plan41-simulation-efficiency-and-diagnostics-2026-10-16/publication-and-caller-stack.md)
and current `ResultAppendQueue::flush`, `ResultPayload::from_sealed_owned`, grouped append and
the synchronous Python bridge identify the actual owners. Historical failures/probes remain at
[39d](39d-qualification-and-plan38-cutover.md#post-plan41-simulation-audit--2026-10-16).
**Proposed:** every correction and verification below; no product build/run occurred in authoring.

The existing append API already supplies bounded same-fence grouping, ordered ordinals, exact
receipts and original-operation settlement. It is preferable to a new batching protocol.
Plan41's artifact grouping remains a separate staged activation lifecycle. Result analysis
already owns protected reads, response interpretation, artifact promotion and graph activation;
preserve their meaning while correcting execution composition.

## Ordinary caller execution

### B0 — Locate the material composition boundary

I01 examines `workflow/analyses.rs::Runtime::result_analysis`, its protected-read/promotion/graph
children, original benchmark callers and `pse-py/src/workflow.rs::blocking_on_with_interrupted_value`.
Separate retained async state, compiled nested poll frames, wrapper effects and any actual
recursion. Historical prologue reservations do not prove linear simultaneous stack consumption.
Do not require a new scientific run merely to inspect state/layout.

Compare selective separation of child futures/state and narrower lifetimes first. The selected
direction preserves borrowed caller ownership and current recovery values. Heap placement is an
implementation mechanism to assess, not a guarantee: a small outer wrapper can still synchronously
poll a large child. Use a bounded ordinary-stack control capable of rejecting that false fix.

A spawned or dedicated-thread route remains unselected. If narrowing cannot provide a credible
ordinary execution route, return a concrete decision describing owned inputs, cancellation,
task failure, detach/abort/join, observation propagation and acknowledged recovery. Do not
silently change the Python boundary or use a larger-stack benchmark as the passing substitute.

### B1 — Integrate the ordinary-stack correction

Apply the B0 correction across supported Rust callers and the synchronous Python route, keeping
protected inputs, original response/artifact interpretation, uncertainty keys and graph oracles.
Tokio polls `block_on`'s root on the calling thread. Worker stack configuration does not resize
the caller, and its 100 ms timeout is cooperative, not preemption. Do not claim a production
Python crash: the review establishes exposure, not an observed Python failure.

Keep actual cancellation/drain and acknowledged interrupted values. Completion of simulation
does not make a later analysis abort acceptable. Targeted controls establish the new mechanism;
Q1 must also run both original full result cases with ordinary placement and unchanged assertions,
including graph activation, protected reads, publication/reopen/retirement and cleanup. Remove
replaced async compositions/callers/tests; do not add a second permanent execution path.

## File and metadata lifetimes

### B2 — Establish real flush boundaries and transfer ownership

I02 inventories callers of `ResultAppendQueue` table changes, `finish` and `flush`, including
solve, dynamics, fit and study output composition. Distinguish a schema/file boundary from a
consumer's genuine ACK/live-prefix requirement. Retain original operation clocks and all demanded
outputs; no ACK may escape before exact canonical settlement. A `finish` whose consumer needs
its ACK remains a flush barrier. The inventory must identify real consumers, not just rename
the current boundaries.

Before retaining several sealed files, split large encoder/batch/byte ownership from small
append metadata ownership. Current `flush` reserves three maximum-file extents and passes that
lease into a sealed payload that contains no resident bytes. Carrying many such leases would
defeat the intended bounded queue, especially in the original observer pool.

Keep encoding, source batches, file intent and transport allocations charged through their actual
last use and drain. After sealing and actual byte/transport completion, release/drop encoder
working sets and retain sufficiently charged immutable file descriptors, exact request identities,
protection and append metadata. Transfer must be explicit in ownership, not merely a smaller
accounting number while the same buffers remain live. Uncertain effects retain their original
intent and settlement route. Pressure may cause an earlier metadata flush; it cannot justify
releasing live work or changing result completeness.

### B3 — Group separately sealed nonempty metadata

Queue bounded `ResultBlockAppend` items across compatible sealed files/result sets using the
existing `append_result_blocks` owner. Preserve same attempt fence, unique immutable operations,
consecutive per-set ordinals, encoded extent limits, every exact receipt and original clocks.
File contents remain homogeneous IPC schemas. Empty declarations keep their separate opaque
ACK route; staged artifacts keep their own queue/activation contract.

Flush when the bounded metadata queue requires it, when real downstream ACK/live-prefix demand
requires it, and before completion consumes the full exact ACK inventory. A batch spanning
multiple original deadlines must honor all of them without renewal or waiting beyond the
earliest required boundary. Current server append operations still execute internally: fewer
crossings do not mean all statement work disappears. No cross-run transaction or blanket retry
framework is selected.

On response loss, settle each original immutable operation under its original meaning. Preserve
gap/conflict/deadline rejection, cancellation fences, empty/nonempty output inventory and closed
manifest reconciliation. Scientific usability never follows from sealing or a successful RPC alone.

### B4 — Investigate carrying immutable physical bytes

I07 follows cold durable physical data encoded for hashing and upload. Determine whether one
already-owned encoded representation can serve both operations with unchanged schema/content
identity, exact collision comparison, current source eligibility and protection. Compare total
retained bytes and actual upload lifetime; carrying bytes longer can offset an encoding reduction.

Adopt a narrow byte-carry correction only if the ownership and semantic comparison support it.
Fresh protection on warm hits remains required and is not a duplicate computation to eliminate.
Record a supported no-adoption outcome if the useful ownership boundary does not fit. This inquiry
does not expand into a general content cache, new protocol or environment/provenance proof.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| <a id="b0"></a>B0 | Existing analysis/caller contracts | Settle I01's first concrete borrowed-ownership correction, or return an explicit execution-boundary decision if needed; source/layout and ordinary-stack controls distinguish a superficial boxing fix. | Completed — targeted and selected assembled acceptance passed; shared scope-end checks accepted at coordinator Outcome |
| <a id="b1"></a>B1 | B0 selected remedy; any newly required decision installed | Integrate ordinary-stack Rust/Python analysis and interruption/recovery; remove replaced composition; targeted tests plus Q1 original full cases. | Completed — targeted and selected assembled acceptance passed; shared scope-end checks accepted at coordinator Outcome |
| <a id="b2"></a>B2 | Current append/file ownership and real finish consumers | Settle I02; implement actual encoder-to-small-metadata transfer with byte/transport drain and original clocks; reservation/pressure controls. | Completed — targeted and selected assembled acceptance passed; shared scope-end checks accepted at coordinator Outcome |
| <a id="b3"></a>B3 | B2 implemented lifetime and demand boundaries | Integrate bounded cross-file nonempty metadata grouping through existing append/ACK owners; replay/loss/order/deadline/empty/live-prefix and full inventory controls; remove per-file submission coupling. | Completed — targeted and selected assembled acceptance passed; shared scope-end checks accepted at coordinator Outcome |
| <a id="b4"></a>B4 | Current physical encode/upload/protection contracts | Settle I07; integrate qualified byte carrying if supported, otherwise record no-adoption with reason; content/permission/lifetime controls. | Completed — supported no-adoption outcome; inquiry settled |

B0 is the first functional-blocker investigation. B2/B3 form an independent prerequisite chain;
they do not need to wait for unrelated preparation packages. B1/B3 may touch shared artifact or
projection surfaces and must coordinate edits. Existing append primitives suffice for the narrow
route; any changed empty or settlement contract requires a new concrete rule decision.

## Verification, documentation and handoff

**Proposed:** local layout/composition, metadata extent and ownership controls first; actual store
controls for fencing, atomic group rejection and uncertain-operation settlement. Exercise interleaved
sets/files, exact replay, gaps, conflicts, deadline expiry, reply loss, empties, cancellation and a
consumer requiring early ACK. Check simultaneous encoder/metadata reservations under the original
relevant pool; do not increase a cap to conceal retained working sets.

Q1 owns both original result journeys on the ordinary caller stack and the installed Python signal/
recovery case. The managed numerical/read journey retains its original 256 MiB observer pool.
No new statistical timing threshold is required. Whole-owner query counts cannot attribute server,
transport, Arrow or CPU time, and the historical 64 MiB stack probe is not ordinary-route acceptance.

Implementation includes generated declarations/consumers and regeneration if changed; generated
SurrealQL is inspected but never hand edited. Transfer enduring ACK/file/analysis explanations to
blueprint §20 and relevant workflow owners through the design route. Preserve the existing
demanded-ACK rule; B2/B3's use of the existing API needs no publication-consistency rule change.

## Current checkpoint

B0/B1 are integrated and independently source-reviewed. Analysis selection, promotion and graph
assembly retain borrowed activity ownership and protected inputs. Actual debugger captures
located two successive failing reader boundaries: terminal selection in `Runtime::results`,
then `next_checked` through the cancellation race. Non-inlined factories now place large
borrowed state before polling; reader factories independently prepay exact boxed state rather
than reusing SDK metadata allowances. Child allocation drops before its grant. Empty/exhausted
reads return before allocation after the original cancellation check. No task, thread, stack
increase, public error representation change or second production path was introduced.

Both unchanged ordinary-stack result cases pass with their numerical/graph/publication/
retirement assertions, native joins, fixture cleanup and enclosing registered store stop.
Four targeted native controls pass, including unfunded admission, grant lifetime and actual
empty/exhausted readers under full-pool pressure. The initial pressure invocation selected an
incompatible functional CPU lane and ran no tests; the supported exclusive route passed with
registered cleanup. The managed sixteen-point result-reading case also passes with current
reader code and its original 256 MiB observer pool. Current installed Python entered-analysis SIGINT/recovery also passes with original exception/
cause, actual response drain, a following analysis and registered store stop. Coordinator Q2 is complete with the actual repaired-check results at its Outcome.

B2/B3 separate checked source/encoder/byte ownership from sealed file metadata. Ordinary queued
final tables defer metadata through the existing bounded append owner; live sample/horizon
prefixes, empty declarations and explicit finish retain demanded ACK barriers and original
clocks. Set-completion obligations survive fallible I/O and settle only at the exact final ACK.
Optional queue release drops empty vector capacity before its charge and refuses live
completion obligations. Native store controls and both assembled result journeys pass. A stale
unit fixture counted file contents toward the metadata envelope; its corrected metadata-only
admission/refusal controls pass. No wire or budget contract changed.

I07 concludes with no byte-carry change. `Operations::put_physical_rows` streams ordered IPC
blocks once to derive immutable problem/operation identity, then re-encodes and backpressures
upload. Blocks are bounded; total relation encoding is not. Carrying all first-pass bytes would
require unbounded retention, a new refusal or a spool/protocol design. The 48 MiB operation
reservation is not a total relation bound. Fresh warm-source protection remains required;
this conclusion does not claim encoding is free or measure its wall-time contribution.

## Outcome (recorded after implementation)

### What was built

**Implemented:** borrowed non-inlined analysis/artifact factories retain protected ownership.
Actual debugger captures identified two further connected reader boundaries: result selection and
`next_checked` through cancellation forwarding. Their factories prepay exact heap state before
allocation and release child state before its grant; cancellation precedes allocation and exhausted
reads return without new admission. No task, thread, stack increase, alternate production route or
public error change is used. Source/encoder/byte ownership now drains before sealed metadata enters
the existing bounded fenced append owner. Live prefixes, empty declarations and explicit finish
retain demanded exact ACK barriers, original clocks and completion obligations through fallible I/O.

**Tested — 2026-10-16, zero-failure baseline:** both unchanged `results-chain-1` and
`results-publish-chain-1` passed through `just case-measure ... --smoke --audit-diagnostics`, each
with original numerical/graph/publication/retirement assertions, one native join and fixture cleanup;
enclosing registered store stop and command exit were zero. Four native reader controls passed
through `just unit-package pse-runtime 'test(reader_future_) | test(canonical_empty_demand_has_registry_schema_without_ipc_or_absent_membership) | test(canonical_nonempty_multipage_reads_share_256mib_with_real_compiler_leases)'`
with native-solvers/canonical-tests, explicit force-validation and exclusive placement. They cover
unfunded refusal, Pending/Ready/drop grant lifetime and real empty/exhausted readers under full-pool
pressure. The current-worker managed sixteen-point reader passed with its original 256 MiB observer.
Current installed `pse.tests.canonical_signal_control --analysis` passed original exception/cause,
entered response drain, following analysis and registered cleanup. Store ACK/group/manifest,
reply-loss/rollback/order controls and the corrected metadata-envelope controls also passed.
Coordinator Q2 owns the shared scope-end result; local artifacts are under
`build/assessment/p42-execution-20261016/`.

**Interface-checked:** I01/I02 are settled by those boundary and consumer classifications. I07
concludes with **no byte carrying**: bounded IPC blocks do not bound total relation encoding, and
the 48 MiB operation grant is not a total relation limit. Streaming hash followed by backpressured
upload remains; retaining all bytes would require a new refusal, spool or protocol design without
established benefit here. Enduring lifetimes and ACK meaning are transferred to blueprint §19–§21.

### A mistake made and corrected

Initial boxing narrowed analysis but did not remove the connected-reader failures. Actual crash
captures located the remaining boundaries; their individual frame extents are not summed as an
established simultaneous stack total. Earlier aborts remain unsuccessful evidence. Exact dead
creator markers were archived/released under standing authorization, retaining incomplete outcomes,
data and pins; registered stop then succeeded. An initial pressure invocation selected a functional
CPU lane that could not admit its observer and ran no tests; the supported exclusive invocation
passed. A stale fixture charged file contents as metadata; its corrected admission/refusal controls
passed without a protocol or budget change.

### Deviations from the plan, deliberate

The correction extends to connected selection and batch-reader state because actual default-stack
journeys required it. Exact heap grants remain separate from SDK/metadata allowances. The historical
larger-stack probe is not acceptance of the ordinary route. I07 is settled by supported no-adoption;
no unbounded byte retention, cap increase or statistical performance target was introduced.
