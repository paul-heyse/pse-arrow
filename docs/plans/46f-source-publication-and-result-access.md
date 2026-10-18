---
title: "Plan46f: Source, publication and result access"
status: done
date: 2026-10-18
adrs: [ADR-0180, ADR-0186]
review_sources: [docs/design_review/reviews/design_review_execution-speed-review-audit_2026-10-18.md]
doc_role: plan
doc_topics: [design, preparation, storage, performance]
---

# Plan46f: Source, publication and result access

## Target and foundation

Own AF12/AF17, I11–I14 and RC09 implementation under ADR-0180/ADR-0186. The [coordinator](46-execution-speed-and-numerical-efficiency.md) owns decisions/dispositions. F01–F05 adopted functional scope and bounded decisions are complete on assembled main, including the live ownership corrections. Final qualification and whole-series acceptance remain owned by [G06](46g-investigations-qualification-and-design-publication.md#g06). AF12 cross-sample batching and parallel protected decoding remain retained production gaps.

**Interface-checked, starting point:** ordinary edits already computed typed SourceDelta and ran changed-row builders. `canonical/delta.rs` still reconstructed parent/index products, resolved every declaration’s references and serialized every candidate; `persist(previous)` was not the ordinary edit path. Live callbacks constructed physical rows/checked Arrow, and the append helper allocated and flushed a fresh queue per sample. Explicit cursor close/drop already detached without cancelling science. Snapshot scans shared one mutexed decoder and one partition. The implementation below replaces selected work at those actual owners; its limits and verification are recorded here.

The target removes unnecessary work without inventing another source authority, publication queue or result decoder policy. Full ingress can legitimately examine the full model. Per-sample latency and flush guarantees are real consumed behavior; internal batching is not automatically semantically free.

<a id="f01"></a>

## F01 — Reuse immutable typed source/index/canonical products

**AF17; I13.** At the actual delta owner, retain appropriate immutable parent/index/per-object canonical products and re-evaluate only the dependency closure affected by edits. Carry exact lexical resolution dependencies: successful and absent lookup, shadowing, imports, names/additions/deletions, ordering, data/physical visibility and document-span changes. A changed-ID set or unchanged spelling is not that closure.

Changed source-default edits and pure runtime overlays remain different; changing a physical premise can invalidate more than a declaration's expression bytes. Preserve complete source identity, reconciliation/retry, transport/trust admission and explicit publication. Retain conservative work wherever the closure cannot yet justify narrower reuse; do not claim fully incremental resolution from partial dependencies.

**Decision/targeted acceptance:** one local value/source/default/structural edit at increasing unrelated declaration inventory; absence→definition, shadowing/import changes, deletion/reorder/spans and physical visibility. Count examined references, serialization/index construction and retained bytes separately. Compare full fresh admission as an independent semantic control where appropriate. Implement qualified incremental consumers and delete replaced broad paths for their covered scope; full import stays explicit.

**Implemented; I13 selected reuse:** `workflow/modeling/canonical.rs` and
`canonical/delta.rs` retain reconstructible products on the completely admitted immutable
revision: ordered parent metadata, the source index, parsed fields, data closure and outgoing
reference premises. Tracing records declaration/binder/type-parameter reads, exact named
lookups including absence, import roots/aliases and guarded-member reads. Additions,
removals, shadowing and changed imports invalidate the consumed premises. Changed source
or document spans rebuild their parsed products; a physical-scope change conservatively
re-examines references. Unaffected declarations reuse their admitted immutable objects,
and an affected declaration with unchanged source and resolved references avoids canonical
serialization. Retained/copy envelopes include document text, origin metadata and dependency
storage; preceding working grants do not accumulate into each new revision.

**Retained limit:** complete metadata maps are still copied, and source identity still frames
the complete ordered inventory and physical bindings. Namespace/data work remains
conservative where narrower reuse is not justified. This is selective reference, parse and
canonical-product reuse, not fully incremental model admission. Full receiving/import
admission remains explicit; these products grant neither transport trust nor publication
permission. The local-default control distinguishes examined references, canonical
serializations and updated index rows from the complete copied index inventory and retained
charge. Its 8/32/128 unrelated-declaration ladder and corrected combined lexical fixture
passed in the central functional selection recorded in the Outcome. Full metadata copying
and complete ordered identity framing remain.

<a id="f02"></a>

## F02 — Capture compact samples and encode outside native callbacks

**AF12; independent of cross-sample flush change.** Move selected physical sample payloads into bounded run-owned admitted buffers and perform checked Arrow/Collection construction at the existing supervision/publication stage. Preserve original source/sample/attempt membership and producer ownership; no second full trajectory is retained just to stream.

Bound queued samples, encoder overlap, subscriber aliases and sealed metadata together with existing memory ownership. Full-channel wait can use existing cancellation-aware wakeups rather than millisecond polling if it fits actual callback/thread ownership; this is separate from encoding cost. Current output wait compute lending remains intact.

**Targeted acceptance:** sparse/dense selected output with unchanged integrator steps/tolerances, unread/closed/dropped subscriber, cancellation during capture/encoding/wait, invalid/late sample and final release. Per-sample flush/durability can remain unchanged here. The replacement must remove callback encoding in actual native workflow callers, not add an unused asynchronous helper.

**Implemented:** `workflow/live_simulation.rs` now captures only selected physical output
values, mode/time and original run/accuracy-attempt/sample locators into the bounded,
charged native-to-supervisor channel. `workflow/run.rs` performs the existing physical
projection and checked Collection/Arrow encoding before delivery and publication. Capture,
encoding overlap, subscriber aliases and attempt summaries retain their respective owners;
Full terminal retention remains an explicit separate demand. Encoding checks the original
cancellation and absolute deadline before work, between rows and before publication. An
encoding refusal cancels the producer, drains/joins the native owner and leaves live recording
incomplete. Callback row/Collection encoding and its former native error-conversion helper
were removed from the actual workflow path.

**Implemented, integrated review follow-up `a5b82ad4f`:** independent
source review found that dropping the cursor and completing the native observer could release
the queue-envelope grant before the supervisor finished draining. The follow-up moves the
receiver, subscriber, projection and shared grant as one supervisor-owned channel bundle;
channel teardown and native join precede release of that owner. Review also found that the
selected-sized encoding grant omitted the helper's full original identity maps rebuilt for
every sample. The follow-up removes that helper. One pre-dispatch, prepaid scratch scan now
checks ambiguity across both complete original inventories under the original cancellation
and deadline, then retains only ordered selected member/quantity/canonical-unit lineage,
mode identities and original admitted source ownership. Per-sample encoding consumes that
immutable selected projection. A preparation refusal follows the existing canonical joined
refusal path before native work launches. These fixes add no public event, ACK or policy change.

**Retained limit:** one supervisor encodes serially; there is no second trajectory or encoder
queue. Per-sample recording remains unchanged. The existing full-channel polling/wait and
compute-lending behavior remain; no wakeup or wait-latency improvement is claimed. Earlier
selected controls exercised compact/full output, ordinals, escaped buffers, invalid mode/output
extent, cancellation/deadline and unread/closed subscription behavior. Central canonical live
ACK/close and the added lifetime/projection controls passed for the integrated source, as
recorded in the Outcome.

<a id="f03"></a>

## F03 — Select and implement bounded live batching semantics

**AF12; I11; RC09 before changed flush/visibility.** Compare off-callback per-sample encoding/flush with bounded larger checked chunks or a retained queue in the existing durable append owner. Define the maximum visibility/flush boundary from the actual consumer contract before relaxing current per-sample barriers; choose an explicit finite setting, not an undocumented timing heuristic.

Physical microbatching preserves each public original sample ordinal/order. Flush before required event/schedule/attempt/selection/terminal boundaries and demanded ACK consumers, with bounded memory and selected latency. A Recorded event represents the genuinely acknowledged contiguous prefix; failed/unknown append cannot advance it. Preserve selected accuracy-attempt identity and provisional versus accepted science, per-attempt close and latched incomplete recording.

No early ACK, lossy delivery or automatic subscriber detachment is authorized. Existing explicit close/drop detach remains. **Decision/targeted acceptance:** matched output density, bounded queue/encoding/append counts and latency, unread/closed subscriber, stop/event/accuracy retry, uncertain append/settlement, duplicate/reordered prevention and actual terminal drain. Select the simplest compliant boundary; if no useful batching fits, keep per-sample semantics and record that outcome without pretending F02 required a rule change.

**Implemented; I11 retained per-sample decision:** the existing durable append owner now
reuses its optional empty 256 KiB queue-control allocation across live samples. Each sample
still constructs its bounded append working state, flushes synchronously and advances
`Recorded` only after the actual append acknowledgment. The control grant carries no
scientific payload or unacknowledged prefix and retires on close/refusal. Optional allocation
refusal keeps the existing synchronous fallback. The pure control test passed repeated reuse,
refusal and final zero owned charge; central functional controls also passed authentic
per-sample prefix/ACK/drain behavior. No cross-sample batching was selected.

**No adoption of cross-sample batching:** current consumers receive per-sample visibility and
ACK barriers; the observer does not supply explicit event/schedule boundary markers or a
consumer-selected flush/ACK policy. Output density cannot silently relax that contract.
Revisit only for a concrete consumer whose explicit finite sample/time visibility limits,
demanded ACKs and event/schedule/attempt/selection/terminal boundaries can be represented
and qualified at the existing owner. This retained decision claims neither an optimal batch
size nor absence of potential batching benefit. No changed public policy, early ACK or
subscriber auto-detachment was introduced.

<a id="f04"></a>

## F04 — Qualify independent protected decoder lanes/prefetch

**AF12; I12.** Compare bounded file-affine decoder lanes or bounded prefetch on sufficiently large selected multi-file reads. Each active lane owns its reader/cache/footer/decode state and charged working envelope; cloning the same shared mutex does not remove serialization. File affinity preserves useful footer/integrity reuse and avoids uncontrolled reopen work.

Acquire decode reservation at actual admitted lane execution rather than accumulating several reservations behind a serial mutex where feasible. Retain one frozen read ticket, exact selected rows, descriptors/digests/schema/row checks, honest unsupported filter/projection pushdown and escaped Arrow ownership. Narrow selection must still reach discovery; do not regress Plan45 selected snapshots.

**Decision/targeted acceptance:** one/multiple files, selected/full rows, concurrent readers and skewed file sizes, corruption/refusal, cancellation and retained aliases. Count simultaneous decoders, opens/footer/decode work, live charged state and complete read latency. Adopt bounded parallelism only where complete behavior benefits; small simulation latency alone does not establish a need.

**Implemented:** `workflow/result_snapshot.rs` acquires its cancellation-aware decoder
turn before reserving the file-shaped working envelope. Waiters no longer accumulate decode
grants behind the serial mutex. The frozen read ticket, selected descriptors/rows, integrity,
schema/row checks, existing pushdown limits and escaped Arrow ownership remain with their
existing owners. A targeted wait-cancellation control passed without obtaining a working grant.

**I12 no adoption of automatic independent lanes/prefetch:** production still has one shared
reader/cache mutex and one native provider partition. Extra partitions alone would retain
serialization, and separate readers require bounded CPU execution outside the async poll as
well as independent charged reader/footer/decode state. The primitive inquiry covers 1/4/16
actual IPC files, 64/8192 base rows, a four-times larger final file, sparse/full batch and row
selection, integrity/open/footer/decode work, two independent reader threads and escaped
aliases. The corrected inquiry passed correctness/ownership controls with single-file cache
reuse and one final escaped batch per file held equally in all paths. Its measurements are
recorded in the Outcome; they do not qualify complete protected-query throughput. Earlier
unfair comparison timings are excluded from decision evidence.

Revisit parallel adoption when a selected large multi-file consumer has admitted bounded
execution outside the async poll, independent file-affine state and a complete protected-read
comparison showing useful benefit with comparable selected rows, retained aliases and charged
overlap. No universal absence-of-benefit claim or new decoder policy is made. Selected canonical
frozen-selection, integrity/refusal, reclamation and release controls passed on assembled main;
they do not turn the primitive lane inquiry into a qualified production parallel reader.

<a id="f05"></a>

## F05 — Investigate protection/cache payload granularity

**I14; U18.** Reassess the historical growth-fixture protection/release-grant versus optional-cache mismatch at the actual physical product/protection owner. Compare granular reusable payloads and shared fixed overhead with current complete products; this is not a proposal to blindly enlarge cache or lower charges.

Preserve escaped independent lifetimes, exact replay/current admission, overlap, uncertain release and owner-driven retirement. Optional cache refusal can be correct even when retention would help; active ownership and optional retention have different requirements. **Decision/targeted acceptance:** useful retained replay, pressure/eviction, repeated independent holders, uncertain release and final zero owned charge. Count rebuilt payload/grants and actual retained envelope separately from RSS. Implement qualified representation sharing or record the no-change reason and precise trigger; no age/size deletion or unrelated storage cleanup.

**Implemented; I14 granular receipt retention:** `workflow/physical_cache.rs` now retains
reusable receipt values and their exact checked source support separately from caller
protection/release grants. Optional `Rows` entries hold the actual worker and pool identity
owners and existing model/registry bindings; they no longer clone an unused Operations/store
handle or retain `PhysicalOwner`/source-protection grants. Their metadata allocation covers
the copied physical metadata and revisions. Exact source comparison remains mandatory before
reuse, and every returned `PhysicalRows` holder still obtains fresh native protection, including
on a cache hit. Retired/stale/refused current protection cannot be authorized by retained bytes.

**Retained limit:** this change applies to optional row receipt payloads. The physical admission
cache retains its existing protection contract, and escaped callers retain independent active
protection and release ownership. Active charges and uncertain-release recovery are unchanged;
the 16 MiB optional retention budget was not enlarged. A payload can still be refused by that
budget or actual admission pressure. Selected growth/replay, independent-holder,
pressure/eviction and final release controls passed on integrated main. These functional
passes do not establish a measured cache-hit improvement or protection-cost reduction.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| F01 | Actual SourceDelta path and complete affected binding dependencies | Selective parent/index/parsed/canonical reuse; corrected lexical/full-ingress controls | done |
| F02 | Existing live producer/supervisor/append and buffer ownership | Compact capture and supervisor encoding; joined lifetime/projection and canonical live ACK/close controls | done |
| F03 | Working F02; RC09 decision; actual visibility/ACK consumer | Per-sample policy retained, empty control grant reused; authentic per-sample ACK/drain controls | done |
| F04 | Actual protected decoder/ticket/selection contracts; G01 observations | Execution-time reservation; corrected primitive inquiry; selected protected-read controls; production serial retained | done |
| F05 | Actual physical protection/cache/escaped lifetime owner | Granular optional immutable receipt retention; fresh protection/pressure/release controls | done |

## Historical executor verification before central integration

The following receipts describe their original worker-source boundaries. Every `not_run`
and pending handoff in this section is historical; the current checkpoint and Outcome below
record the later integrated functional results. These receipts do not create outstanding
worker work or imply whole-product qualification.

**Implemented, source checkpoint:** F changes were integrated on main through `a06b58114`.
The initial worker follow-up was `72bf7156c`; review correction `a5b82ad4f` was committed in
the isolated worker checkout and awaited integration at that checkpoint. Later assembled
source and acceptance were coordinator-owned. Source commits alone did not establish
focused or assembled acceptance.

**Tested, partial — 2026-10-18:** at worker source `d8caac2e1`, the following native-local
command used the pinned toolchain, explicit Arrow force-validation, `solver-diffsol`,
`solver,klu` setup and a private resource state, with no canonical store effects:

```bash
BASH_ENV='' PSE_TEST_RESOURCE_STATE=/tmp/pse-plan46-f-resources-20261018 \
scripts/pse-env -- just unit-native-capability-package pse-runtime \
pse-runtime/solver-diffsol solver,klu \
 'test(live_simulation::tests) or test(canonical::delta::tests) or test(canonical::incremental_tests) or test(decoder_lane_inquiry) or test(live_append_control_unit)'
```

Against the zero-failure baseline, compilation passed and **17 tests ran: 16 passed, 1 failed**.
The failing `delta_products_absence_shadow_import_delete_order_and_span_match_fresh` fixture
used invalid `import q;` syntax. The follow-up corrected it to the language’s exact-version
`use` syntax; the corrected control was **not_run at that checkpoint**. Passing selected controls covered compact/full
live output and original ordinals, capture/encoding refusal and stop checkpoints,
subscription backpressure/detachment, local-default 8/32/128 source reuse, existing delta and
fresh receiving/source-size controls, empty append-control reuse, decoder wait cancellation
and the earlier primitive file inquiry. These passes apply only to that source and selection.
An earlier preparation refusal after source changed during the build executed no tests and
supplies no passing numerical evidence.

**Historical not_run, review follow-up:** `a5b82ad4f` had not been compiled or tested while the
coordinator owned the build lane. Four new controls covered a pending capture after cursor and observer drop,
an actual dropped subscription followed by native join and queue-grant release, repeated
one-output encoding with 0/64 unrelated check-observed original members and equal selected
allocation extents, and canceled/expired projection preparation. Existing invalid-mode,
compact/full, canonical expired-admission and actual ACK controls remained required at that
boundary. The focused selection was `test(live_simulation::tests)` under the appropriate
native/canonical route; the Outcome records its later central result.

**Historical not_run, follow-up source:** the corrected lexical fixture, fair single-file/alias decoder
comparisons, worker/pool-only optional cache identity and updated physical growth/replay,
pressure, eviction and release controls. Canonical live ACK/close, exact selected snapshots,
integrity/refusal, reclamation and uncertain-release controls have not run for the integrated F
changes at that checkpoint. No production speedup, canonical cache-hit result or complete F
acceptance was claimed at that boundary.

**Historical handoff:** the coordinator was to run focused controls on frozen assembled main
after D/C/E integration, using an owned store only for real canonical effects. The selected scope was
canonical delta/reload/incremental, live simulation, physical admission cache, decoder/append
controls and connected-result/protected metadata/release tests. The Outcome below records
that later functional selection; G06 owns selected source-to-answer/live/result/Python final
qualification and scope-end checks. The corrected primitive inquiry remains separate from
canonical protected-read qualification. Historical observation receipts retain their original
conditions and do not own current status.

## Current execution checkpoint — 2026-10-18

F01–F05 and the two independent F02 lifetime/scratch corrections are integrated and their
selected functional controls passed on assembled main. Broad metadata copying/full identity
framing, callback wait polling, per-sample visibility, active protection charges and the lack
of an admitted parallel decoder remain explicit limits. The F worktree is integrated and
removed. G06 owns final qualification and G07 whole-series publication; the Outcome below
supersedes historical not_run statements without reinterpreting their original conditions.

## Outcome

Recorded 2026-10-18.

**Implemented:** selective typed source/reference/parsed/canonical-product reuse, compact
native sample capture with supervisor encoding and joined queue/projection ownership,
reused empty append-control capacity under unchanged per-sample flush/ACK semantics,
execution-time decoder reservation, and optional immutable receipt-payload retention with
fresh active protection for every holder. Callback encoding and its former conversion helper,
per-sample complete identity-map rebuilding and optional-cache retention of unused store/
active-protection owners were removed from the covered production paths.

**Tested, zero baseline:** 46g's joined full-native/canonical/compiled runtime selection passed
all selected F source/reload, live sample, physical-cache, protected-result and admission
controls. The complete selection ran **97 tests: 96 passed, one stale D fixture failed**;
the corrected fixture subsequently passed in an affected native-local consumer selection
of **11 tests, 11 passed / 0 failed**. The original 97-test run remains a failed run and was
not repeated as a whole. Raw selections/results are retained in
`/tmp/pse-plan46-integrated-runtime.log` and `/tmp/pse-plan46-runtime-consumers.log`; G06
owns exact invocations and final qualification. The corrected lexical fixture, all four
new live projection/queue lifetime controls, canonical expired admission, per-sample ACK,
dropped/unread subscribers, fresh protections and queued cleanup are no longer not_run.
Independent source review of the F02/F05 queue, projection and optional-cache ownership
changes reported no remaining material issue within that scope. G06 final static, manual and
Python boundaries remain separate from these functional results.

**Measured, I12:** `decoder_lane_inquiry` ran one observation for 1/4/16 files, 64/8192 base
rows with a four-times skewed file, sparse/full selection, comparing actual independent readers
with the one-lane control and sequential file affinity. Complete decoded values and escaped
final ownership agree. For 16 files/full/8192 base rows, one-lane interleaving opened files 64
times versus 16 for file-affine routes; latency was 3.586870 ms versus 0.954362 ms for sequential
file affinity and 0.916199 ms for two independent lanes. Charged bounds were 7.93/7.66/9.46 MB.
One-file cases preserved one open in every route and threads added overhead. Sparse 16-file
reads showed no benefit from two lanes (0.708 ms versus 0.672 ms control). Earlier unfair
measurements are excluded; these are codec controls, not complete protected-query speed claims.

**I12 decision / retained AF12 limit:** preserve the existing protected decoder and execution-time
reservation. File reuse explains much of the observed improvement; two-lane timing alone
selects no production parallel reader. A qualified follow-up must preserve original row order,
frozen ticket/integrity and escaped charges across complete protected reads, and supply an
admitted off-poll CPU owner before parallel decoding. Cross-sample batching remains unselected
under I11. No mandatory cache enlargement or active-grant undercharging was introduced.

Mistakes corrected: the lexical fixture used invalid `import` syntax and now uses the
language's `use` syntax. Independent review caught queue-grant release before supervisor
drain and an encoding grant that omitted full identity-map scratch; the integrated correction
retains one supervisor-owned channel bundle through teardown/join and prepares a prepaid
selected projection before dispatch. Decoder comparison controls now preserve fair
single-file reuse and equal escaped aliases. Optional receipt retention no longer holds
unused store handles or caller active-protection grants.

Deliberate deviations: retain complete metadata map copying and ordered source identity where
narrower reuse is unqualified; retain authentic per-sample flush/ACK rather than relax visibility
without a concrete consumer contract; and retain production serial decoding. Primitive
file-affinity benefit is mostly reuse and supplies no complete protected-query speed claim.
Optional caches retain immutable payload only; every active holder obtains fresh protection.
The completed I11/I12 decisions therefore retain AF12 live-batching/parallel-decoder gaps.
