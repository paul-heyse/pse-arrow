---
title: Durable records, results and analytics
status: done
date: 2026-10-14
adrs: [ADR-0187]
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
| <a id="e1"></a>E1 | I-E1 public consumer/writer premise; accepted fences | Concrete chosen topology, operation/visibility/reader/backup contract and required decision routes | Implemented; selected topology and I-E1/I-E2 decisions settled |
| <a id="e2"></a>E2 | A admitted products/receipts; existing effects; E1 for changed topology | Edit/attempt acknowledgements, immutable occurrences and exact retry/claim settlement; remove only obsolete protocol effects | Implemented; exact file/receipt ACK, original-intent settlement and owner controls passed |
| <a id="e3"></a>E3 | Exact demand/row/completion contract; E2 ack; E1 for representation cutover | Bounded bulk chunks and complete authoritative manifests/close, migrated writers/readers | Implemented; packed file publication, complete manifests and connected result controls passed |
| <a id="e4"></a>E4 | A5 checked batches/B3 records; E3 exact snapshots for changed storage | Demand-shaped result/analytics/replay access and equally strong protected readers, no eager all-family encoding | Implemented; guarded native query/read lifetime and exact byte-charge controls passed |
| <a id="e5"></a>E5 | E2/E3/E4 borrower/ack/read closure | Artifact/history/analysis retention, settled GC, backup/restore and direct ownership cutover | Implemented; independent borrowers, collection, actual crash/backup/restore and fresh creation passed |

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


## Execution checkpoint — 2026-10-13

The maintainer selected SurrealDB coordination plus immutable files and the existing process-crash/restart guarantee, then authorized all E1–E5 execution. ADR-0187 and architecture revision 165 install the concrete target; the focused target review accepts the proposed design, separately from implementation. Same-host independent writers, automatic recording and strong admitted reads are preserved. IPC files replace result transport fragmentation; JSON seeds stay JSON. Operations owns file/catalog lifecycle and strong tickets, Runtime owns scientific codecs/snapshot consumers, supervisor tooling owns offline inventory/drain/restore. The v6 descriptor cutover, strong read tickets, packed IPC writer/native query snapshots, independent scientific borrowers, irreversible collection, exact owner recovery and complete offline file backup/restore are implemented. Restore preserves admitted analyses while fencing new creation; historical cleanup keeps exact terminal settlement receipts. Independent implementation review corrections include full file-effect drain, caller-close continuations, persistent instance exclusion, original staging alias recovery, collecting tombstones and owner-registration acknowledgment uncertainty. Pure protocol/codec/tooling controls, actual v6 initialization, exact live/dead child recovery and the nonempty native analysis/artifact seed have passed; the combined store-backed lifecycle/read/restore checks and current rollout publication remain in progress. All functional packets remain open until these scoped acceptance controls pass; 38f and assembled 38g qualification remain open. Existing disposable data may regenerate after authentic drain; the quiesced primary profile and other owners are preserved.

## Investigation decisions — 2026-10-14

**Implemented** target choices, separately from the verification below:

| Question | Selected realization and retained limit |
|---|---|
| I-E1 deployment/topology | Shared SurrealDB catalog/arbitration with private immutable files; same-host independent clients and supervised workers. SQLite, runner-owned journals and per-writer discovery logs add no required capability for the selected premises and were not selected. Cross-host clients or a changed durability requirement reopen the decision. |
| I-E2 reads/collection/recovery | Exact nonexpiring manifest tickets cover future unread files, backed by authenticated Linux process/managed-owner drain. Collection is irreversible under a persistent per-instance lock. Offline backup freezes admission and GC before inventory; fresh-incarnation restore preserves scientific borrowers and fences creation. No network-filesystem, online-backup or power-loss guarantee. |
| Representation and coalescing | Complete uncompressed IPC files pack one attempt/relation/schema/interpretation, targeting 16 MiB with a 32 MiB hard bound and demanded ACK/live-prefix flushes. Opaque JSON seeds remain JSON. Binary seeds or another codec need a concrete representation/encoding limitation; no speculative conversion. |
| Analytics and discovery | SurrealDB owns exact manifest discovery. Rust snapshots expose native DataFusion `StreamingTable` over guarded readers using the model's native session/resources/functions. Built-in projection, filtering, sorting, limits and spill remain library-owned; no custom execution plan or new Python SQL product. |
| Cache qualification | General DataFusion file caches remain truthfully inactive. The private footer cache has its own byte charge. Qualified cache activation and remaining shared-boundary work stay with 38f; no cache or throughput benefit is claimed here. |

The guarded reader retains prepaid footer validation and exact file/read ownership that the
pinned Arrow file source cannot carry. This is a bounded integration choice, not a second
query engine. The old result stream codec, bulk catalog payload fields and renewing result/
artifact read protections are replaced; source-ingestion stream encoding remains a distinct
consumer. Disposable v5 stores regenerate through the explicit fresh-target v6 cutover;
initialized v5 targets refuse rather than silently migrating retained data.


## Verification — bounded implementation acceptance, 2026-10-14

Baseline: zero permitted failures. **Tested** below means the named selected controls, not
whole-product or scientific qualification. Commands used `scripts/pse-env --`, the private
`PSE_TEST_RESOURCE_STATE=/tmp/pse38e-resources-5aLWFeeA`, and disposable selected services;
the primary profile and shared registry migration fence were preserved. Rust commands used
`python -m scripts.arrow_validation nextest run --no-fail-fast`, `--locked`, and the
wrapper's explicit selected-root `force-validate` closure. Store-backed tests selected
`PSE_SURREAL_STATE=/tmp/pse38e-native-3jxbq57v`, with `--native=` and functional admission.
The released server was SurrealDB 3.3.0 on same-host Linux.

| Evidence | Actual command/selection after the shared Rust prefix, or named command | Result and scope |
|---|---|---|
| **Tested** | `-p pse-operations --lib --features canonical-tests --locked -E 'test(canonical_files::tests)'` | 10/10 passed: private file identity, publication, staging and collection controls. |
| **Tested** | `-p pse-operations --lib --features canonical-tests --locked -E 'test(canonical::transport::lifetime_tests) \| test(canonical_closed_file_caller)'` | 6/6 passed: transport/file activity ownership and closed-caller behavior. |
| **Tested** | `-p pse-runtime --lib --locked -E 'test(result_files_unit) \| test(source_stream_unit)'` | 7/7 passed: complete IPC/metadata/IEEE/footer limits and distinct retained source encoding. |
| **Tested** | `-p xtask --bin pse-canonical-recovery --no-default-features --features canonical-tools --locked -E 'test(payload_) \| test(recovery_creation_)'` | 4/4 passed: native inventory verification and creation controls. |
| **Tested** | `-p pse-operations --lib --features canonical-tests --locked --test-threads 2 -E 'test(canonical_files) \| test(canonical_artifact) \| test(canonical_analys) \| test(historical_protection) \| test(canonical_file_owner) \| test(final_fixture_teardown)'` | 38/38 passed: actual catalog/file, artifact/analysis, historical cleanup and owner lifecycle controls. |
| **Tested** | `-p pse-operations --lib --features canonical-tests --locked --test-threads 2 -E 'test(canonical_files::catalog_controls) \| test(canonical_file_owner) \| test(final_fixture)'` | Final delta 9/9 passed: exact ACK/lost-registration-ACK settlement and atomic final-fixture mint/drain. This overlaps the preceding selection. |
| **Tested** | `-p pse-runtime --lib --features canonical-tests --locked --test-threads 1 -E 'test(canonical_connected_results_server_unit)'` | Final 4/4 passed: packed nonempty/empty demanded reads, original rows, escaped arrays, retained native query plans/functions and exact memory ownership. |
| **Tested** | `python -m unittest scripts.tests.test_payload_files scripts.tests.test_surreal_server scripts.tests.test_surreal_rollout_producer scripts.tests.test_surrealkit_maintenance scripts.tests.test_host_admission` | 341/341 passed: file backup/inventory, strong owner drain, gated validation, fresh-unit maintenance, CLI module authority, frozen cutover and host absence proofs. After the final deadline-output refinement, the affected `test_payload_files` and `test_surreal_server` modules passed 243/243. |
| **Tested** | `python -m scripts.tests.canonical_recovery_check target/debug/pse-canonical-recovery --directory /tmp/pse38e-recovery-20261014-complete` | Passed one complete native journey: SIGKILL/reopen, lower/original server caps, exact protected source/product bits, large staged payload, admitted analysis/artifact JSON, offline backup v3, fresh-root restore, old credentials/creation refusal, gated protected reads, fresh creation and terminal cleanup. Numeric profile: 1 GiB server plus one 512 MiB native worker; no `--scientific`. |
| **Tested** | `python -m scripts.surreal_server recover-maintenance --state /tmp/pse38e-recovery-20261014-final/restored`; `validate --state /tmp/pse38e-recovery-20261014-accepted/restored --interpretation pse.substrate.v6 --check-command /home/paul/pse-arrow/target/debug/pse-canonical-recovery /tmp/pse38e-recovery-20261014-accepted/restored` | Retained failed attempts recovered through their ordinary exact owners; protected semantic validation passed and left its target stopped/quiesced. Earlier failed journeys keep their original result. |
| **Tested** | `/tmp/pse38e-owner-recovery-probe.py`, run under the private environment; `python -m scripts.surreal_server recover-file-owners` on its selected fixture | Actual live child remained protected; after exact SIGKILL/reap the dead owner drained; repeated recovery was idle. This is process/catalog evidence, not a solver or power-loss claim. |
| **Implemented** | `python -m scripts.arrow_validation build -p xtask --bin xtask --bin pse-canonical-recovery --no-default-features --features canonical-tools --locked`; `just codegen` | Current canonical tools built without warnings; all five generation targets completed. The native verifier was selected by exact artifact digest. Full scope-end codegen drift/hygiene remains with 38g. |
| **Implemented; Interface-checked** | Frozen rollout producer using surrealkit `1.0.0-beta.6`; explicit first `--design-phase-v6-cutover`, then ordinary forward publication | Current frozen head is `20261014040753__canonical_ddd85e3dc83e728b`. Native single/chain checks passed; initialized v5 targets refuse. No preserved-data migration was claimed. |
| **Interface-checked** | Independent target and implementation reviews, including file ownership/registration, ACK shape, shared memory ownership, validation gate and terminal maintenance deltas through `b1f215025`; final SQL-output corrections inspected by the coordinator and exercised by the actual journey | Target accepted at Proposed strength; scoped final implementation reviews found no remaining material findings. Reviewers ran no builds/services/tests. |

Earlier invocations were not promoted into passes. The affected Operations selection first
reported 31/35 passing and four failures (three fixture callback-drain races and one artificial
expiry). Connected-result invocations reported 1/4 and then 3/4 passing; missing physical ACK
binding, duplicate metadata charging and a catalog-assumption fixture were corrected. A later
closed-store setup invocation reported 0/4 passing before explicit normal activation; the final
4/4 selection above used an open store. The tooling aggregate initially reported 280/283
passing (two errors and one failure in the new ACK mock); the first host module reported
52/53 passing because its old profile fixture lacked the v6 incarnation. Corrected selections
are listed above. Initial complete-recovery invocations each failed one journey, first at fresh
maintenance startup and then at validator ACK framing; retained recovery and the final fresh
journey exercised their repairs. All counts use the zero-failure baseline and the named commands.

`just turn-end` completed with exit 0: ADR indexing and formatting passed. Its
safe-fix Ruff stage uses `--exit-zero` and emitted 49 findings before formatting
(E501 2, PT018 2, PLC0415 1, PTH115 3, S101 2, S608 7, TRY301 3, B023 17,
PT009 12), against the zero-finding target. This is not a clean lint qualification;
38g owns repair and the actual scope-end check. Formatting changed no Python AST;
Rust changes at this final step were rustfmt output, with no new functional edits.

**Not run:** compiled Python consumer journeys, linked solver/reference scientific recovery,
managed scientific worker reconstruction, full-series integration/component/performance,
`just hygiene` and manual governance/docs/native/powerset campaigns. These remain 38g after
38f functional scope. No latency, throughput, RSS benefit, numerical method superiority,
cross-host, network-filesystem, online-backup or power-loss claim is **Measured** here.

## Outcome (recorded after implementation)

### What was built

**Implemented:** E1–E5 and I-E1/I-E2 deliver the selected SurrealDB arbitration plus sealed
private files target. Native complete IPC packing, exact committed file-binding ACKs,
original-intent uncertainty settlement, nonexpiring future-file tickets and authenticated
owner recovery replace catalog bulk bytes and renewable result/artifact read ownership.
Result snapshots reuse native DataFusion execution and model resources/functions; decoded
arrays retain real charges after readers close. Independent analyses/artifacts retain future
borrower obligations through producer retirement and fresh-incarnation restore. Collection
is irreversible, per-instance physical exclusion persists, and backup v3 includes the exact
sealed/unresolved payload closure before publishing completion. Old result codecs, payload
fields and replaced renewal machinery are removed; no compatibility production backend remains.

**Tested:** the selected controls and actual complete recovery journey in Verification pass
within their stated conditions. **Interface-checked:** independent implementation challenge
found and corrected drain, lock, alias, tombstone, registration, receipt and charging gaps;
its final focused reviews found no material issue. Enduring contracts live in blueprint
§19/§20/§21, proposed ADR-0187 and the substrate guide. ADR status remains proposed pending
its decision-PR route. Whole-series finding closure and assembled qualification remain with G.

### A mistake made and corrected

The first grouped ACK omitted `file_binding`, so the writer correctly refused successful
nonempty publication instead of weakening exact equality. Actual empty-snapshot checks also
exposed a real duplicate charge for shared selection metadata; the snapshot now owns that
charge once. Fixture shutdown initially mistook response wakeup for final callback drain and
allowed independent caller minting to race its last-owner check; both now join under the
physical admission owner.

Actual recovery exposed assumptions that pure mocks missed: `reset-failed` can refuse a
fresh/collected unit, intermediate UPDATEs return records, and deadline functions return
booleans. Maintenance now proves exact fresh terminal absence, and SQL suppresses/binds
intermediate output while retaining the strict single ACK and original deadline. CLI and
reciprocal imports share one lifecycle authority. Validation opens protected file reads with
GC still frozen and writes closed, then refreezes and authenticates owner/descendant drain on
every exit. These were production/protocol corrections, not relaxed acceptance.

### Deviations from the plan, deliberate

The selected topology keeps SurrealDB rather than replacing shared arbitration. Complete
uncompressed IPC and guarded native `StreamingTable` readers fit the actual owner/footer
contracts; binary seeds, a new journal, general cache activation and a Python SQL product
were not selected. The explicit v6 fresh-target publication route replaces an ambiguous
beta migration diff; it refuses initialized v5 targets instead of backfilling disposable data.
A private test registry respected the shared migration fence. Only the maintainer-approved
stale allocation `dfb3005f1a24473191d59f4fcedec927` was archived/removed; its original terminal
outcome remains unproved. Other retained data/evidence were not deletion-authorized.

All owned worker changes are integrated on main. The three owned worktrees are removed at
handoff; private services are stopped/drained and failed-attempt evidence retained. This
companion is complete within the selected target and bounded acceptance. 38f/38g, shared
coordinator finding dispositions and later reconciliation of other active plans stay open.
The companion remains available to its active qualification readers rather than being retired
prematurely.
