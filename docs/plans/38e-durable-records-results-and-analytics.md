---
title: Durable records, results and analytics
status: draft
date: 2026-10-13
adrs: []
review_sources: [docs/design_review/reviews/design_review_efficiency-and-solver-target-audit_2026-10-12.md]
doc_owner: docs/plans/38-efficiency-and-solver-target.md
doc_topics: [architecture, storage, validation]
---

# Plan 38e: Durable records, results and analytics

## Purpose, default and open choice

This companion owns persistence/publication as a consumer of [Plan 38](38-efficiency-and-solver-target.md)'s scientific products, correcting F01–F04/F12 and remaining Plan 36 receipt/result/reader/retirement scope. [State evidence](../design_review/evidence/efficiency-and-solver-target-audit-2026-10-12/state-reuse-and-boundaries.md) and [pipeline evidence](../design_review/evidence/efficiency-and-solver-target-audit-2026-10-12/pipeline-diagnostics-and-advice.md) identify current guarantees and alternatives. Blueprint §19/§20/§21 and ADR-0180/0182 describe current consumers; concrete amendments follow G0.

Accepted RC01–RC05 preserve automatic durable visibility, independent writers and strong admitted-reader lifetime while local compute becomes store-independent. They authorize a bounded topology decision, not server removal or weakened reads. Current remote publication plus local admitted snapshots is a complete viable target; SQLite WAL/embedded or runner-owned alternatives must earn their full lifecycle. A new journal is not prerequisite to native reuse, policy, diagnostics or replay export.

Current result cursors support bounded projection and C-stream buffer sharing; protected readers, exact immutable scientific artifacts, independent analyses and typed unknown settlement are valuable. Source claims are **Interface-checked**; replacement schemas/protocols/checks are **Proposed**. No storage service, crash probe or measurement ran during authoring.

## Packets

| Packet | Consumes | Delivery/retirement | State |
|---|---|---|---|
| <a id="e1"></a>E1 | I-E1 public consumer/writer premise; accepted fences | Concrete chosen topology, operation/visibility/reader/backup contract and required decision routes | Planned |
| <a id="e2"></a>E2 | A admitted products/receipts; existing effects; E1 for changed topology | Edit/attempt acknowledgements, immutable occurrences and exact retry/claim settlement; remove only obsolete protocol effects | Planned |
| <a id="e3"></a>E3 | Exact demand/row/completion contract; E2 ack; E1 for representation cutover | Bounded bulk chunks and complete authoritative manifests/close, migrated writers/readers | Planned |
| <a id="e4"></a>E4 | A5 checked batches/B3 records; E3 exact snapshots for changed storage | Demand-shaped result/analytics/replay access and equally strong protected readers, no eager all-family encoding | Planned |
| <a id="e5"></a>E5 | E2/E3/E4 borrower/ack/read closure | Artifact/history/analysis retention, settled GC, backup/restore and direct ownership cutover | Planned |

E2 acknowledgement/receipt simplification and E4 batch reuse can improve current storage before E1 selects replacement. Topology-specific implementation cannot precede E1's real requirements/decision. D1 owns scientific frontier and eligible seeds; E owns claims/effects/borrowers they consume. C/F own shared capacity/transport. No document-wide barrier is implied.

## E1/I-E1 — Select a complete persistence realization

Read actual Rust/Python edit/durable execution/history/results/latest/promotion/analysis/reclaim/restart consumers. State successful acknowledgement, failed/partial visibility, terminal ordering, journal/local discovery, remote client and runner-surviving worker promises. Same-host independent writers and remote/surviving jobs are separate premises. One writer per study log does not settle shared project heads/catalog arbitration. Retain existing capability until an explicit alternative is confirmed.

| Alternative | Conditions and retained obligations |
|---|---|
| Local snapshots + current remote publication | Removes canonical compute coupling while retaining independent writers/current APIs. Simplify actual coordination with fitting native transitions; transport remains only where consumed. |
| Same-host transactional catalog + immutable files | Compare SQLite WAL exact version/binding, transaction/busy/checkpoint/backup and strong reader/file semantics. Independent processes are possible without an application server; network filesystems are a separate unsupported/qualified question. |
| Runner-owned catalog/log + child workers | Needs exclusive writer succession, project-open conflicts, bounded IPC, late-result rejection, child drain and specified runner failure/recovery. Child crash isolation alone does not remove durable outcome uncertainty. |
| Per-writer immutable logs + shared arbitration | Must justify head/discovery/latest/retire/reference/GC machinery rather than recreating a database in manifests. |
| Shared SurrealDB coordination + immutable files | Native constraints/transactions can own suitable shared effects; external files, locked read-dependent transitions and uncertain responses remain explicit. |

I-E1 ends with exact public/deployment requirements, a complete selected alternative, justified retained tradeoffs and the concrete operator decision for changes beyond accepted fences. Source/library contracts can settle semantics; throughput/source-size contests are not mandatory. Use library skills, Context7 and exact pinned sources for implementation facts; no stale maps/utilization journal. Independent consideration compares complete operation/recovery, not just normal append.

Prefer preserving currently promised durability: required edit commit precedes success acknowledgement. An independently surviving writer may acknowledge its committed append; a same-process background queue alone cannot promise survival of immediate native crash. Ephemeral work remains explicit. Automatic durable terminal recording remains default; local admission and publication visibility are separate, so offline computation must not falsely advertise recorded history. A chosen explicit-publication extension needs discoverability/history/promotion semantics and further decision if wider than RC01.

## E2 — Occurrence, acknowledgement and current permission

Classify authored revision/head, attempt accepted/started/terminal, output chunks/close, artifacts/analysis/history, reader admission/retirement and physical collection by immutable fact versus real mutable coordination. Mathematical payload/content identity deduplicates bytes; occurrence identity owns scientific attribution and idempotent intent. Neither equal body nor minted id is current writer/receiver permission. Share complete encoded receipts/allocations without dropping selected/acquired source association, absence dependencies or current eligible roots.

Use actual library atomic constraints/transactions for fitting transitions. A lost response after commit settles the same immutable intent and exact outcome; never rerun science or blindly replay mutation. Preserve original operation/deadline/owner incarnation and distinguish definite conflict/refusal from dispatched unknown. An old exact acknowledgement settles its old effect after expiry but cannot mint a fresh live root. Sequence issue/unique id alone is not current-claim proof. SurrealDB references have actual supported field scope and cannot protect external files.

While the existing tier remains, preserve source creation/reclaim, seal/active-root release, result reader/artifact admission and both serial orders with current registrations/fences. If a selected owner removes an operation, retire its machinery then; immutability/single writer alone does not delete crash/retry/read/GC obligations. Local A products do not renew store roots merely to compute unchanged mathematics, but a new imported/durable effect obtains its actual current admission.

**Targeted acceptance:** equal body/new receipt; acquired/absence/provider/collision negatives; commit-before-response-loss exactly-once settlement; cancellation/deadline and stale owner; creator/collector, pin/retire and seal/unrelated creator both orders; no late resurrection; independent writers cannot silently overwrite heads; unavailable sink versus computed result remains explicit. Required recording failure and scientific outcome are different fields, not a synthetic solver failure or false durable success.

## E3 — Exact demanded bulk output and close

Keep Full/Compact/member demand, mandatory assessed completion/start/run/attempt/diagnostic summaries and explicit requested-empty versus unrequested meaning. Compact failure detail is bounded; failure cannot force full unbounded history. Use typed original-row locators independent of fields, with truthful time/key bounds; composite id text is not a time coordinate. Preserve exact IEEE bits, metadata, original indices, duplicate/null/list semantics and no cross-attempt filling.

For a selected file tier, write independently sealed chunks or explicitly recoverable IPC segments. Format/codec validity, schema/interpretation and storage integrity remain distinct from semantic content identity. Objects are complete and required persistence established before the exact immutable manifest becomes visible; acknowledge actual committed state. A directory listing is not a snapshot and unfinished Parquet row groups are not promised crash-safe prefixes. Explicit filesystem/platform persistence includes file and parent/manifest ordering where required; an abstract object-store write does not imply fsync.

The actual writer accumulator covers authoritative expected relation/member/empty/completion acknowledgement and uncertainty recovery. It is not a list of assumed submitted writes. Close/seal is permitted only under exact completeness; acknowledged chunks plus incomplete finalization remain typed partial/unknown. Slim live completion reads avoid hydrating all output while preserving full recovery closure. Terminal selection uses declared sequence/class; newest failed/partial data cannot silently become newest usable solution.

Migrate in-memory, durable, dynamics/fitting, inspection and Python result consumers with the selected schema/format; old writers/readers/generated definitions disappear together after targeted controls. There is no compatibility production path required for obsolete design-phase data. Keep bounded backpressure/chunk reservations and original cancellation/drain across C/F resource owners.

**Targeted acceptance:** empty/sparse/full/compact demands, irregular original rows/fields and nested values; large trajectories with bounded live state; interrupted segment/manifest before and after commit/ack; corrupt/missing/schema-mismatched chunk; actual complete versus submitted-only ack; exact latest terminal-class selection; failed/partial result retained without claiming full horizon. E5/G add restore and assembled consumer checks.

## E4 — Protected read, analytics and replay consumers

Retain/reuse demanded checked batches/chunks under allocation owners, preserving existing projection/range cursors and C-stream sharing. Avoid eager encoding every advertised result table. Already copied arrays can outlive durable pins while retaining real byte owners; unread lazy chunks still require equivalent protection. F1 supplies bounded IPC/malformed-input/response-drain, and lazy qualified file-cache activation with truthful inactive reports.

Expose exact committed snapshots to DataFusion for actual analytics/history consumers, with projection/predicate/range and bounded output/spill/queue under the shared envelope. Evaluate actual access plans on populated fixtures where selection depends on them: examined versus returned rows and real index/limit pushdown, not query text alone. Keep real SQL/functions/UDF/physical inventory; no new inbound Arrow/cross-output Python product or function whitelist follows automatically. B3 compact records and B4 replay exports are consumers, not mandatory per-event persistence.

Strong admitted reads survive supported pauses until unread work closes. Compare pins/snapshot retention, supported owned file handles plus future-chunk protection, or read/GC exclusion; grace-period expiry is not equivalent. Close seals new admission, joins release/transport and reports failures honestly. Retire never skips rows, substitutes attempts or leaves a copied buffer uncharged. I-E2 settles exact selected filesystem/read/GC/backup realization and its platform limits before replacing existing ReadOwner/ResultReadDrain guarantees.

**Targeted acceptance:** repeated demanded construction reused while unused families absent; projection/time/original locator exact; paused read versus retirement/GC both orders; close/admission/release failure; arrays after stream drop; concurrent qualified cache activation/refusal rollback and inactive inspection; malformed IPC/integrity/alignment/backing; set-oriented query equivalence and bounded spill. Query/throughput benefit requires a separate named measurement.

## E5 — Borrowers, collection, backup and ownership cutover

Promoted seed/scientific artifacts and independently owned analyses retain sufficient admitted context after producer-run retirement. D1 owns initial eligible-before-cutoff selection and copied seed; E retains exact future borrower admission. Corruption/infrastructure never becomes absent fallback. Artifact/analysis creation versus collection both orders protects new borrowers. Occurrence retirement, detailed-history release and physical shared-payload collection are separate transitions. Settled replay floors/late-acquisition fences prevent resurrection; size/age/deduplication alone do not authorize deletion.

Backup begins GC exclusion before capturing an exact catalog snapshot and holds its transitive manifest/model/provider/start/artifact/payload/journal closure until copying finishes. State whether unpublished work is included. A database copy alone excludes external files; a log copy needs a consistent committed boundary. Interrupted backup is incomplete or resumable under exact identity, never advertised complete. Restore validates closure/integrity/interpretation and new selected identity/credentials before writers/reads resume; missing chunks cannot become complete science.

A selected direct cutover drains authentic owners/processes/effects, regenerates disposable data/evidence as allowed, and replaces schemas/readers/writers atomically at the chosen operational boundary. No legacy data compatibility path is required. Unrelated history/evidence is not deletion-authorized. Retire replaced machinery only after selected read/write/recovery controls and all migrated consumers pass; G owns assembled acceptance and later existing-document reconciliation remains outside authoring.

**Targeted acceptance:** independent artifact/analysis survives producer retirement; retire-first refuses late acquisition; shared payload with multiple borrowers; backup racing retire/append; incomplete copy, missing/corrupt closure and restore; old owner/credential cannot write new target; actual service/worker drain; result/start/promotion/analysis reads after restore. Historical failed/unknown records retain their evidence status.

## Investigation closure and verification

I-E1 public/deployment/topology and I-E2 reader/file/backup investigations produce concrete choices, retained obligations, affected APIs, exact library evidence and required amendments. Optional binary seed transport, read coalescing and journal discovery choose a fitting exact representation/access path or retain the existing one with a demand trigger. No probe is mandated merely because an old inquiry listed one. Current candidate/commit/access-plan questions can be settled with isolated owned fixtures, never override another owner's service.

Targeted compile/unit controls accompany changed declarations/effects and actual generator regeneration. G1–G3 own assembled native/Python/restore/query/scientific scope after functional work. An immutable fact model, prototype or component pass alone does not qualify the selected lifecycle composition.
