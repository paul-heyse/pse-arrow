---
title: Sealed payloads and strong result snapshots target review
date: 2026-10-13
tier: design
purpose: target
standard: core-3.4
profile: process-simulator-1.5
subject: ADR-0187 and Plan 38e persistence delta
baseline: 1c2b0427132b41a954edd352fce70f75484ea111
evidence: Proposed
disposition_owner: docs/plans/38e-durable-records-results-and-analytics.md
---

# Sealed payloads and strong result snapshots

**Accept at Proposed strength.** ADR-0187 supplies a coherent target for private same-host Linux storage with SurrealDB coordination and immutable files, including independent clients and supervised workers. Behavioral/semantic adequacy and architectural fitness both pass for this proposed delta. This accepts the design and its stated limits; it does not establish implementation, crash qualification, scientific qualification, performance, or completion of Plan 38e.

The important improvement is the separation of three authorities: immutable bytes establish a file realization, an authenticated original-operation receipt establishes an append, and exact reconciled membership establishes a result manifest. Scientific assessments retain their existing owner. Strong admitted reads protect future unread files through actual drain, rather than interpreting elapsed time as reader death. The same physical-instance lifecycle connects publication, independent borrowers, collection and offline backup. No material finding requires revision of the proposed target.

## Scope and evidence

This independent delegated review applies Core 3.4, its efficient-architecture heuristics companion, ProcessSimulator 1.5 and the pse-arrow binding. It is a design-tier, target-purpose review of the complete ADR-0187 delta, using Plan 38e E1–E5 as the functional outcome. The reviewer did not author the ADR. Current finding disposition and implementation status belong to [Plan 38e](../../plans/38e-durable-records-results-and-analytics.md); assembled qualification belongs to Plan 38g.

The examined baseline was `main` at `1c2b0427132b41a954edd352fce70f75484ea111`, with only the new proposed [ADR-0187](../../adr/0187-sealed-payloads-and-strong-result-snapshots.md) present as an untracked change before this review was written. The selected premises are same-host independent participants, private local Linux storage, automatic required durable recording, supported paused lazy readers, and process-crash/restart recovery. Cross-host files, network filesystems, online backup and power-loss qualification are excluded explicitly, rather than implicitly supported.

Source inspection covered:

- Blueprint §19/§19.2/§21 and §20.1/§20.2/§20.4/§20.6, for scientific result meaning, publication, retention and public consumers.
- `crates/pse-operations/src/canonical_execution.rs`: `AppendAck`, `ResultBlockAppend`, `append_execution_batch`, `close_acknowledged`; `canonical_results.rs`: `ResultReadDrain`, `ReadOwner`, `ResultRead`, renewal and closed membership; `canonical_result_retention.rs`: original analysis settlement, run retirement and bounded collection.
- `crates/pse-operations/src/canonical_artifacts.rs`: independent artifact coordinates, complete original creation request, borrowed inputs and independently retained obligations.
- `crates/pse-runtime/src/workflow/connected_results.rs`: lazy reader state, cancellation, original selection, metadata and export publication.
- `scripts/surreal_server.py`: offline backup, private offline state, drain ownership, fresh restore identity and validation before reopening.
- Locked Arrow 59.3.0, DataFusion 55.1.0 and SurrealDB 3.3.0, plus their local capability skill entrypoints. This review does not claim exact new API composition has been compiled.

Current paths are **Interface-checked** evidence for consumed contracts and replacement surfaces. The new architecture and its benefits are **Proposed**. No service, build, test, crash probe or measurement ran. This is not a review of the solver, formulation, full Python implementation or whole operational subsystem; no absence claim is made about unexamined source.

## Responsibilities and contracts

Operations owns opaque file instances, catalog attachment, original receipts, live borrower arbitration, irreversible collection and recovery state. Runtime owns scientific interpretation, schema/codec qualification, original coordinates, assessed completion and result readers. Supervisor tooling owns operational admission closure, authenticated drain, consistent offline copy and restore activation. Local mathematical execution requires its actual scientific inputs, not permission to a storage service. Necessary durable effects acquire current permission at their effect boundary.

These boundaries fit the target: changing packing or file representation belongs to storage/codec integration; adding a scientific result family belongs to its scientific authority and ordinary declared result contract. Neither should require consumers to infer usability from files or reconstruct retention policy. Native Arrow/DataFusion types are useful consumed data/query contracts; an additional neutral query language or fetch-one abstraction would discard useful composition.

| Distinction or operation | Target contract and owner |
|---|---|
| Content, encoding, occurrence, physical instance | Separate identities. Byte equality supports deduplication; occurrence preserves attribution; a unique never-reused path selects one physical realization. Operations owns realization and lifecycle. |
| Publication | Reserve original intent/instance before file work; complete, validate, flush and publish without replacement before a fenced atomic catalog attachment of descriptor, borrowers and receipt. |
| Exact append and close | File existence is insufficient. Original authenticated acknowledgement settles the old effect; complete result membership is reconciled separately. Missing response remains uncertainty until exact settlement. |
| Admitted read | A nonexpiring exact manifest ticket protects every future unread file, including deferred plan/partition work. Plans, streams, read-ahead and release work retain it until actual drain. |
| File owner | Bind store incarnation and authenticated Linux boot/PID namespace/PID/start identity; managed workers additionally bind invocation. Children register independently before file work. Unknown liveness remains protected. |
| Borrower and collection | New borrower admission and collection admission share transactional arbitration. Collection is irreversible for that physical instance; equal-content future work uses another instance. |
| Backup and restore | Close admission and exclude GC before catalog freeze, drain actual owners, copy required sealed closure plus unresolved intent state, then publish inventory/digest completion. Restore validates closure under fresh root/incarnation/credentials before activation. |

The physical-semantics contract is preservation rather than new physical modeling:

The coordinator's subsequent ABI clarification is compatible with this judgment: Operations owns file-instance, owner, borrower and manifest-ticket relations, with monotone `pending → sealed → attached → collecting → collected` lifecycle distinctions (collection may also settle abandoned work under the original intent). Logical append/scientific descriptors bind immutable instance UUID, encoding, file digest, batch index where applicable, byte extent and schema; absolute root and storage incarnation are physical placement, not semantic-manifest inputs. JSON seed descriptors do not acquire an artificial IPC batch index.

Controlled closed restore may therefore rehome catalog placement under a fresh fixed root/incarnation while retaining exact bytes and semantic manifest identity. Owner capabilities bind the incarnation; an old collector retains only its authenticated old-root target and cannot discover a new deletion target through mutable placement. Fresh-root restore is not a new scientific publication or a revival of an old owner. Likewise, finite source grants remain permissions to create new effects; an admitted strong ticket alone continues its exact protected read after those grants expire and prevents the associated required context from retiring. This is consistent with S02/S08 provided the ticket covers the complete consumed closure rather than only currently opened files.

| Quantity/model element | Units, basis and conventions | Validity and authority |
|---|---|---|
| Stored result values and seed/artifact evidence | Preserve exact IEEE values, schema metadata, original row/field coordinates, units, basis, reference state and source interpretation; no implicit numeric conversion. | Original scientific admission and assessments remain authoritative under blueprint §19.2/§20.4; file validity/digest does not establish scientific validity. |
| Dynamic or failed/partial observations | Preserve actual time/key bounds, coverage, requested-empty versus unrequested, attempt and completion class. | A closed file or prefix does not certify full horizon, successful solve or complete demanded output. |

Well-posedness, derivatives, solver selection, tolerances and independent post-solve checks are unchanged neighboring contracts. This delta formulates, evaluates and solves no equations, so new numerical-stage columns and unit/property conformance results are not applicable. Persisted outcomes must continue consuming those existing assessments without reinterpretation.

## Revealing scenarios

The following scenarios specialize binding PSE-S04/PSE-S07. They assess the combined design rather than asserting that any control has passed.

| ID | Legitimate stimulus | Required result and architectural reasoning |
|---|---|---|
| S01 | Writer crashes before/after file publication or catalog commit; commit reply is lost. | Reserved original intent identifies the same physical effect. A completed unattached file stays protected for settlement. An exact old acknowledgement settles the original effect after permission expiry without renewing authority, recomputing science or creating another occurrence. |
| S02 | Reader pauses before opening a later file while producer retires and GC runs. | The exact manifest ticket covers that future file throughout the pause. Open handles for only earlier files are insufficient. Clone/plan/stream/read-ahead/release owners retain protection until drained; elapsed deadlines do not revoke it. Collection-first refuses new admission; read-first retains the instance. |
| S03 | An independent client or worker dies, its parent dies, or a numeric PID is reused. | Reaping uses complete authenticated identity and actual death/drain. Parent exit is insufficient for independently surviving children. Unknown observation conservatively retains protection. This is necessary local lifecycle work, not a distributed lease protocol. |
| S04 | Delayed GC finishes after equal bytes are published again. | Never-reused physical paths and irreversible collecting state prevent deletion of the later instance. Digest/content identity cannot substitute for instance identity or permit revival. |
| S05 | A long dynamic result is streamed live; a consumer requests an early prefix. | Packing flushes at acknowledgement/live-prefix boundaries into completed files. The visible prefix contains sealed acknowledged chunks; no reader relies on an unfinished IPC footer. Result close still checks actual demanded membership and completion, preserving partiality. |
| S06 | Artifact or analysis retains selected evidence after producer retirement. | Independent exact borrowers retain the needed sealed bytes/context. Producer retirement and physical collection remain distinct; late acquisition after retirement cannot resurrect an old authority. JSON seeds remain exact under their existing interpretation. |
| S07 | Backup encounters acknowledged output awaiting final result publication, an unresolved intent, or absent staging. | Required sealed bytes remain part of backup closure even without final result publication. Unresolved intent state is copied as unresolved. A legitimately absent/unsealed stage is not fabricated into a file or complete result. Missing required sealed content prevents completion. |
| S08 | Restore runs under a fresh state root and old clients retain credentials or paths. | Validate transitive file/catalog/interpretation closure before activation; bind new root/incarnation/credentials and reject old authority. Restored uncertain work remains uncertain. Old-root cleanup separately proves authenticated drain. |
| S09 | Add a result relation or replace IPC encoding. | Scientific meaning extends its semantic owner; codecs and descriptors absorb representation changes. Existing Python table/stream/export/live contracts and Rust query context retain their capabilities. No storage-specific branch is added to numerical computation. |

These scenarios fit together. Strong read tickets and independent borrower retention share physical collection arbitration, but differ from finite permission to create a new effect. Publication intents protect uncertain external effects; they are not scientific success receipts. Backup must freeze those same distinctions rather than invent a second interpretation of completeness. Conservative unknown-owner protection can retain disk indefinitely until resolution; that is the explicit price of the selected paused-reader guarantee, not permission to infer death from silence.

## Execution fit, library choice and alternatives

The proposal gives demanded bulk bytes a credible local physical route while preserving set-oriented analytics. Compatible batches share completed files within one attempt/relation/schema/interpretation; default 16 MiB packing with a 32 MiB hard file bound avoids a file per scalar and bounds the publication unit. Flush-on-ack/prefix preserves latency and recovery semantics when packing cannot wait. Lazy handles, decode, output and spill retain actual allocation owners; file-size bounds alone would not bound the combined working set. Implementation must use truthful admission/splitting for its actual supported row and metadata shapes rather than treating packing targets as proof of feasibility.

Native DataFusion providers over frozen file lists retain projection/query composition and the configured SQL/functions/resource context. An exact file inventory must remain protected for deferred physical execution as well as initial planning. Arrow owns file encoding/reading, DataFusion owns query planning/execution, and SurrealDB owns suitable transactional coordination. Application code still must compose external-file publication and lifecycle because database record references cannot protect filesystem bytes. That gap justifies the added intent/instance/owner machinery; no new generic database, journal, scheduler or query product is justified.

| Alternative | Assessment for the selected functional target |
|---|---|
| Keep database bulk bytes and simplify existing publication | Simplest viable unchanged topology; retains coordinated history but keeps bulk results inside database RPC transport. Viable baseline, not an invalid design. |
| Selected SurrealDB catalog plus immutable files | Retains existing shared heads/claims/receipts while removing the bulk-byte RPC crossing. Adds a real cross-resource recovery and GC/backup composition cost, explicitly covered by S01–S08. No throughput gain is established. |
| Embedded SQLite plus immutable files | Viable alternative topology, but substitutes shared arbitration, deployment and lifecycle in addition to changing byte placement. It does not eliminate external-file publication/reader/backup obligations. No such additional substitution is needed for this target. |
| Per-writer or runner-owned logs | Must introduce discovery, succession, shared-head arbitration and independent borrower/GC coordination. The selected independent participants make this more machinery without a demonstrated functional advantage. |
| Parquet or binary seed conversion | Possible representation alternatives when an actual access/encoding need warrants them. Current target gains native IPC locality while retaining existing JSON seed meaning; no need establishes another conversion now. |

This is a qualitative assessment under AP-07 and heuristics H1/H2/H4/H8/H17/H23–H25/H27–H28. It supports removal of a crossing and credible lifecycle composition, not a measured latency, capacity or memory claim. Library API/feature composition and actual populated query behavior remain implementation obligations.

## Foundation and gate judgments

Every verdict below applies to the **Proposed** target, not current implementation conformance.

| Foundation | Verdict | Basis |
|---|---|---|
| AP-01 | satisfied | Scientific meaning, opaque lifecycle and offline operations have separate owners; representation change does not authorize mathematical policy changes. |
| AP-02 | satisfied | Explicit append/manifest/read/restore contracts preserve existing public capabilities and expose the selected platform limits. |
| AP-03 | satisfied | Native codecs/query providers compose with catalog transactions and supervisor lifecycle; consumers do not reconstruct the protocol. |
| AP-04 | satisfied | Content/encoding/occurrence/instance, permission/read lifetime, uncertainty/ack/complete result and borrower/retirement/collection distinctions govern the proposed operations. |
| AP-05 | satisfied | Reservation, no-replace publication, fenced attach, nonreviving collection and gated restore have explicit enforcement boundaries. |
| AP-06 | satisfied | Codec/integrity, catalog arbitration, owner observation and supervisor backup can be checked at their actual boundaries; local science needs no store startup. Joint lifecycle controls remain necessary. |
| AP-07 | satisfied | Bounded completed files, demand/prefix flushing, frozen native queries and actual allocation/drain owners provide a credible route; added recovery work purchases concrete independent-client and paused-reader protection. |

| Gate | Verdict | Basis or scope reason |
|---|---|---|
| G1 | pass | Scientific interpretation stays with scientific owners; catalog owns exact attachment/lifecycle, files immutable realization. |
| G2 | pass | Original values, metadata, coordinates, demand and partiality are preserved; no implicit reinterpretation of absent staging or old versions. |
| G3 | pass | File/schema/integrity validation precedes use; invalid required closure blocks backup/restore completion and activation. |
| G4 | pass | Compute, storage effects, admission, reaping and offline copying are explicit operations. |
| G5 | pass | S01–S08 cover coherent visibility, exact settlement, future unread protection, irreversible GC and complete offline recovery. |
| G6 | pass | Exact descriptors bind physical instance/digest/batch/rows/schema/interpretation; byte equality and scientific eligibility remain separate. |
| G7 | pass | Proposed strength and process-crash/platform limits are explicit; no performance, power-loss or implemented qualification claim. |
| G8 | pass | Established libraries retain storage coordination, IPC and query execution; custom composition addresses the external-resource lifecycle gap. |
| G9 | pass | All seven applicable foundations are satisfied for this delta. |
| PS-G1 | pass for preservation | Physical metadata/context survives storage unchanged; physical balance/envelope algorithms are outside scope. |
| PS-G2 | not applicable | This target introduces no formulation, variable roles, structural analysis or solver admission. |
| PS-G3 | pass for preservation | File/ACK/manifest completion cannot promote failed/partial scientific outcomes; numerical algorithms and assessments are unchanged. |

## Findings, rule impacts and follow-up

No material findings or SHOULD deviations were identified. The obligations discussed above preserve the accepted proposal; they are not implementation defects established by this review. Their current delivery state remains with Plan 38e, avoiding a second progress ledger.

| Stable impact | Current rule and owner | Proposed change | If current rule is retained |
|---|---|---|---|
| <a id="rc01"></a>RC01 | Blueprint §20.1/§20.2 database bulk realization and result descriptors | Catalog coordination plus sealed external bulk instances, with original-intent recovery and exact descriptor attachment; ADR-0187 supplies the decision route. | Keep bulk bytes in the database; the selected file cutover does not apply. |
| <a id="rc02"></a>RC02 | Blueprint §20.4 finite live protections/renewal for continued result reads | Nonexpiring exact file-manifest tickets until actual drain/authenticated death; finite creation permissions remain distinct. | Finite renewal alone cannot provide the selected supported paused lazy file-read guarantee. |
| <a id="rc03"></a>RC03 | Blueprint §20.6 operational backup and §19/§21 result-consumer realization | Extend supervisor backup/restore to required external-file and unresolved-intent closure, fresh storage incarnation and gated validation; migrate public consumers directly. | Database-only backup cannot close external-file recovery; retain database bytes until the joint lifecycle is implemented. |

These impacts are already the expressed subject of proposed ADR-0187 and the selected Plan 38e target. This review changes no authority or ADR status. Architecture amendments, decision status and plan disposition belong to the coordinator.

Plan 38e's named targeted controls should distinguish actual mechanism from renamed current behavior: commit-before-response-loss settlement; paused unread files versus retirement/GC in both orders; authenticated independent owner/child reaping and unknown retention; delayed collection/equal-content instance isolation; closed-file live prefixes and exact close; independent artifact/analysis retention; incomplete/missing/corrupt backup closure; and fresh-root restored reads with old-owner rejection. Necessary compile/codec/query integration evidence belongs there. Plan 38g remains the assembled native/Python/scientific/restore qualification boundary.

All product builds, tests, services, probes, performance campaigns and formatting/static checks are **not_run** in this review by assignment. Read-only document/source inspection supports the stated design judgment; it does not count as a test pass against the repository's zero-failure target.

**Decision:** behavioral/semantic adequacy passes and architectural fitness passes for the scoped proposed target; **Accept at Proposed strength**. Implementation, targeted controls and assembled qualification remain open with their existing owners.
