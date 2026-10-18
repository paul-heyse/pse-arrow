---
title: "Plan45c: Result selection, artifacts and analysis"
status: in-progress
date: 2026-10-18
adrs: []
review_sources: [docs/design_review/reviews/design_review_post-plan44-pipeline-optimization_2026-10-18.md]
doc_role: plan
doc_topics: [design, storage, numerical-execution]
---

# Plan45c: Result selection, artifacts and analysis

## Target and current foundation

This companion owns F06–F11/I04. The [coordinator](45-pipeline-execution-optimization.md) owns their dispositions and accepted RC04–RC06. **Interface-checked:** at `22ae87c16`, selected reads first freeze a full relation/range inventory; consumption discovers metadata again; analysis selects/decodes before public promotion independently repeats selection/copying, then opens/decodes copied evidence. Per-response searches and per-edge server lookups amplify that composition. Standalone IPC preflight and cold physical encoding have narrower exact duplicate passes. The [results inquiry](../design_review/evidence/post-plan44-pipeline-optimization-2026-10-18/results-inquiry.md) supplies source/library evidence and measured boundaries; all replacements below are **Proposed**.

The target preserves exact historical attempt selection, native Arrow/DataFusion semantics, actual receiving validation, artifact independence and recovery. Existing schemas, composite indexes, file-reader cache, edge-readiness classification and projection retention are foundations, not missing mechanisms. One private protected decode/admission owner should serve full and selected operations; semantic distinctions do not require duplicate codecs or a new store.

## C01 — Consume full snapshots without rediscovery

**Independent F06 slice.** Full `results`/`latest_results` readers retain their current complete relation/range `ResultSnapshot` and synchronous `snapshot()` meaning. Consumption iterates its already frozen ordered block/file descriptors rather than issuing another unrestricted metadata walk. Retain exact source row coordinates, file-reader reuse, bounded payload admission and existing errors. This correction does not depend on RC04's selected API change.

**Targeted acceptance:** same full rows/digests and ordering across one/multiple batches and empty/half-open ranges; no second unrestricted block walk; unread future files remain protected after reader/runtime drop while a snapshot/native plan escapes. Late failure remains an error rather than EOF. Snapshot aliases share protected metadata without double charging; escaped Arrow arrays retain actual allocation. Full and selected readers eventually consume the same private file/checked-batch primitive.

## C02 — Explicit selected streaming and selected snapshots

**F06/I04; accepted RC04.** `output_results` consumes an immutable typed selection before discovery: exact run/attempt/manifest, relation and stored row range, output owner/field/partition and the actual finite/missing or dense-time predicate. Return a distinct selected Rust reader preserving the selected stream behavior. It has no misleading synchronous full `snapshot()` method. A caller asking for a full relation/range snapshot explicitly uses the full operation. Python's existing selected `TableStream` semantics need not expose physical metadata; migrate its actual native ownership/wrappers and any generated declarations if affected.

The selected reader obtains bounded selected metadata pages once and decodes/verifies required payloads under one exact protected `ResultRead`; it does not capture a full `ResultSnapshot` first. Full relation schema/admission remains the initial target. Coalesce dense row intervals; retain sparse offsets only as required and charge their actual owner before growth. Missing/absent selection must not force full inventory discovery. This is a replacement selected path, not a compatibility facade.

Provide an explicit **async selected-snapshot operation** only for callers needing a reusable native plan. It freezes the complete original selected universe under exact protection, independent of whether a streaming reader is unconsumed, partly consumed or exhausted. It never means the unread tail. The selected snapshot retains full relation schema with explicit restricted row universe. Its provider applies owned selection before any DataFusion residual filtering, limit or projection; advertise generic SQL filters unsupported until separately qualified. Bounds/refusals distinguish inability to freeze a demanded snapshot from the available streaming operation without imposing an arbitrary global result cap.

**Targeted acceptance:** on one 80-state/25-time committed run compare full/one-output/absent-output constructor, first batch and EOF; keep science fixed and separate setup. Count block/descriptor/output queries and checked/decoded bytes, not just returned rows. Test dense time versus scalar numeric/missing predicates, empty selections, duplicate indexes, source row ranges/order, IEEE/unit metadata and index-versus-IPC disagreement. Execute selected native plans with residual filters/limit/projection and verify they cannot widen or change the selected universe. Retain unexecuted plans through producer retirement, missing future file and release/cancellation tests. Full snapshot behavior remains unchanged.

## C03 — One protected selection and acknowledged evidence handoff

**F07/I04; standalone artifacts remain the first realization.** Introduce an internal protected selection containing the exact read capability/metadata-release owner, sources/primary revision and admitted family dispatch. Analysis passes its existing selection into promotion; independently called public promotion obtains the same internal product itself. Copying provenance strings is not protection, and matching bytes do not imply a current receiving context.

During the once-only copy/encoding pass, produce bounded scientific facts plus provisional member ordinal/row/field coordinates, bound to exact encoded descriptors, schema, row order and bytes/digest. Do not retain every decoded trajectory batch solely to save a reread. Scalar local certificates and weaker recorded transient evidence require different sufficient facts. Share the evidence owner/index representation with C04 instead of building duplicate full fact collections.

After exact artifact/member ACK and independent staging ownership settle, bind facts to final occurrence/member locators and acquire the actual artifact input/read authority needed by graph persistence. Only that acknowledged product permits graph admission/activation. Graph structure can be privately prepared earlier but cannot become active based on provisional locators. Release staging and producer protection only through the existing acknowledged transfer. An unknown ACK retains original intent/unsettled recovery; it does not mint another occurrence or erase a real owner.

This removes repeated selection/source lookup and local reread only while byte identity, interpretation, contextual validation and final coordinate binding remain established. Independent artifact reopening, external receipt and process restart still perform their real receiving checks; those are necessary operations, not legacy paths. If a particular branch cannot safely carry evidence through activation, name and qualify its retained admission work rather than claiming universal removal.

**Targeted acceptance:** identical graph meaning and exact row/field locators; conflicting family/context rejection; valid partial/event weaker evidence; producer-first/artifact-first retirement. Inject cancellation before dispatch, during member transfer, after artifact activation, before/after graph activation and before staging release; inject lost member/artifact/graph ACK and graph failure. Recover by exact original intent and verify no active graph lacks independently retained acknowledged evidence. Count selecting/decoding/encoding/reopening work across the same complete operation before claiming benefit.

## C04 — Bounded indexed evidence qualification

**F08/I04; independent of shared-file adoption.** Replace per-response vector filtering/search with one operation-local exact sample-membership index and validated run/step/family groups. Consumers ask this owner for permission for a particular scientific coordinate; retained row locators still identify the original evidence. No persistent cache/service or compulsory relational conversion is needed for this bounded typed join.

Preserve every header/context/endpoint predicate, duplicate/conflict classification, experiment/parameter/unit relationship, checked sample/column range and scalar certificate prerequisite. Float keys follow existing equality: signed zeros compare equal and every NaN compares unequal. Infinity equality in the prototype grants no scientific validity. Partial/event trajectories retain permitted recorded sensitivities inside actual endpoint/sample coverage; they are not blanket-rejected or upgraded to local certificates.

**Targeted acceptance:** production comparisons for 80 output symbols × 5/25 sample times × 1/4 directions, with independent complete graph/locator expectations. Add conflicting duplicate headers/contexts/endpoints, missing sample/direction, wrong experiment/parameter/unit, integer boundary and scalar-family mismatch; test bounded index refusal before speculative graph effects and final release. The review's membership kernel is a starting control, not the implementation or permission oracle. Count index build/probes and full analysis work; removed sample visits are not a wall-time percentage.

## C05 — Distinct facts per bounded server graph page

**F09/I04.** In the authoritative transaction, resolve distinct endpoint/input/artifact-block coverage once per bounded page, then check every edge and its exact replay against those facts. Use set-oriented insertion where the actual pinned engine supports the consumed contract. Keep active/retiring checks, immutable replay and activation reconciliation; client validation cannot replace server enforcement. Change `pse-codegen`'s declaration/query generator and regenerate outputs through the owner.

Use actual SurrealDB 3.3 plans for relevant predicates before prescribing indexes or hashed record-range layouts. Existing composite indexes and client `EdgePublication` readiness already exist. Reader `FOR UPDATE` contention is a separate C08 experiment, not a reason to remove locking while making this correction.

**Targeted acceptance:** repeated/distinct coordinates in a bounded edge page; missing/cross-analysis endpoint/input/block, out-of-range evidence, duplicate/conflicting replay, append to active/retiring graphs and lost ACK after commit. Preserve exact retry and incomplete graph semantics. Count distinct versus per-edge server work, with query/server-wall observations scoped to the real call.

## C06 — One standalone IPC preflight

**F10, independent direct correction.** Construct one bounded Arrow reader with the expected standalone single-batch count, then retain existing schema/row/native-decoder checks. Remove only the exact duplicate preflight over the same immutable bytes. Keep hostile-footer/node/buffer allocation checks; do not implement another decoder or confuse this with a new receiver's separate admission.

**Targeted acceptance:** malformed footer/lengths/nested buffers, extra batches, wrong schema/rows, null structure and exact IEEE payloads retain outcomes; one standalone decode performs one preflight. The small correction is useful but is not presented as the explanation for multi-second publication.

## C07 — Once-encoded physical publication and qualified grouping

**F11/I04; accepted RC06 for changed publication granularity.** Encode each source block once into an owned bounded memory/managed-spill spool with exact block order, byte offset/length and row coordinates. Derive the current identity from those emitted bytes, then publish those same bytes. Preserve existing framing/block boundaries rather than hashing one representation and re-encoding a merely equivalent one. Charge retained memory/I/O buffers before allocation and reclaim owned scratch only after issued effects drain. A warm exact receipt hit obtains current protection without opening the cold spool.

The direct correction retains current per-block edit/retry/receipt behavior. Then qualify a grouped per-relation closure: bounded staged pages and one coherent activation under exact original request/parent authority, selective per-relation replay and package/context association. Encoding stays outside the guarded transaction; no unbounded edit vector or giant whole-package transaction is assumed. Bind retry to exact membership/bytes/operation, settle unknown ACK by original intent and document any changed revision/ACK granularity through RC06 before adoption. If it cannot qualify, retain spool plus existing edit sequence and keep remaining crossing cost explicit at F11.

**Targeted acceptance:** first cold, same-runtime warm and changed-row cases; exact identity, blocks, references and selected replay. Interrupt during spooling, between blocks, around ACK and context receipt; inject memory/disk refusal, parent conflict and uncertain effect. Grouping additionally tests complete activation, original-intent recovery and changed receipts. Owned scratch cleanup and source publication success are separate assertions.

## C08 — Qualified projection, shared files, access paths and explicit reuse

This package owns I04's larger alternatives, not another copy of C01–C07. Run a branch only where its named discriminator can change the complete operation; do not require every storage alternative before direct corrections proceed.

| Choice | Required contract and decisive experiment | Implementation or retained branch |
| --- | --- | --- |
| Projected decoding/admission; RC04 | Exact Arrow 59.3 projection plus declared schema/field/invariant dependencies, mandatory predicate inputs and residual-before-limit semantics. Compare full versus projected decode on the same selected snapshot. Whole-file digest verification still reads full bytes. | Implement a qualified projected admission or reuse of byte-bound established validity, not partial checks labeled full `FieldCheckedBatch`. Otherwise retain full decode/admission and state remaining cost. |
| Independently retained shared sealed files; RC05 | Exact owner/incarnation, file/batch/row/schema/digest and future-file protection, independent collection, no resurrection, ACK/crash recovery and incidental retained backing bytes. Test producer-first and artifact-first retirement in both admission orders. | Initial standalone C03 remains usable. Adopt shared members only after complete independent retention/coordinate/receiving semantics qualify and the accepted decision route runs. A borrower flag alone cannot implement this branch. |
| Actual query access and reader arbitration | Inspect generated predicates/EXPLAIN with unrelated records; compare short concurrent readers and retirement under current locks. Establish equivalent admission/retirement ordering before changing lock behavior. | Implement supported indexed/bulk/arbitration corrections inside the approved contract; otherwise keep current locks/indexes with concrete evidence. No speculative index proliferation/store rewrite. |
| Repeated analysis use | Compare fresh occurrence creation with explicit `analysis(key)` reopening under exact retained ownership. Investigate reusable bounded prepared evidence/base graph only if a distinct creation consumer needs it. | Migrate consumers that actually mean reopening to the existing API. Never silently deduplicate new occurrences or discard root/reachability input needed for graph meaning. |
| Materialized relational reuse | Distinguish cloned native plans/providers from actually materialized DataFusion results and existing projection retention, with real repeated consumers and owned limits. | Use fitting native capabilities only where they remove complete repeated work; no blanket cache or new Python query product. |

For each selected outcome, integrate consumers, retire replaced mechanisms and run its revealing semantic/failure/ownership tests. A rejected or unsupported alternative records its exact gap; it does not automatically close F06/F07/F11's remaining obligations. Any broader change to occurrence meaning, store topology or public scientific semantics returns for a concrete decision rather than being inferred from conditional approval.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| C01 | Current full snapshot contract | Full-reader frozen-inventory consumption; future-file/late-error/alias tests; duplicate walk removed | in-progress |
| C02 | RC04 decision; exact selection/protection | Distinct selected stream/snapshot/provider; all Rust/Python consumers migrated; no broad selected discovery or silent snapshot narrowing | in-progress |
| C03 | Exact byte/fact/final-locator and ACK-state contract; C04 shared evidence representation where consumed | One protected selection and standalone promotion handoff; retirement/cancel/unknown-ACK controls | in-progress |
| C04 | Existing family/permission predicates; bounded index admission | Exact production membership/grouped evidence with adverse/full-graph tests; repeated searches removed | in-progress |
| C05 | Actual generated server transaction/replay contract | Distinct fact validation per bounded page; generated output and adverse/query-plan controls | in-progress |
| C06 | Existing bounded standalone codec | Single preflight with hostile/extra-batch/schema/IEEE tests; duplicate preflight removed | in-progress |
| C07 | Current source identity/scratch ownership; RC06 before new granularity | Single spool and qualified grouped-publication decision/implementation; cold/warm/changed/interrupt controls | in-progress |
| C08 | Relevant working contracts; RC04/RC05/RC06 for affected selected branches | Complete larger I04 alternatives, supported implementation and explicit no-change/unresolved outcomes | in-progress |

## Verification and checkpoint

**Planned:** touched runtime/operations/codegen compile checks and targeted force-validated reader/codec/evidence/transaction units. Real service tests use owned registered fixtures and normal cleanup; no shared registry migration/reset or another owner's service. Reuse one committed run for matched constructor/read/analysis contrasts. Include source/result/artifact identity and original scientific assertions, not returned row counts alone. Final assembled durable/Python/DataFusion/publication and static qualification belongs to 45f.

All eight packages are open. Existing source/retained case evidence establishes the work, not the remedies. RC04–RC06 are accepted within their recorded bounds; direct full-reader, join, server-fact and codec work has an independent existing-contract path. Ordinary stream, native plan, retained artifact and reopened analysis lifetimes must all remain explicit at completion.

Current execution checkpoint: C01/C04/C06's first slice is integrated and its 26 pure reader/evidence/codec controls passed. C02/C03 selected discovery and protected fact/ACK handoff are in integration; real canonical acceptance is pending. C05 distinct bounded-page facts are generated and syntax checked; actual server replay/adverse/index-plan controls remain pending. C07 once-encoded memory/managed-spill storage passed targeted exact-byte, pressure, disk-quota and release controls. Per-block source revision/ACK granularity is retained under RC06: the current coherent edit API consumes a complete edit list, and unbounded accumulation would defeat the bounded spool; no streaming grouped receipt/retry contract has qualified. Cold/warm/changed actual publication remains to qualify. C08 branch decisions and whole-operation acceptance remain open.
