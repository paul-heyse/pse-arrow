---
id: ADR-0187
title: Coordinate sealed payloads and strong result snapshots
status: proposed
date: 2026-10-13
deciders: [paul-heyse]
level: decision
principles: [AP-03, AP-04, AP-07, DP-09, PS-09, PS-10]
blueprint: [§19, §20.1, §20.2, §20.4, §20.6, §21]
review: docs/design_review/reviews/design_review_sealed-payloads-and-strong-result-snapshots_2026-10-13.md
evidence: Implemented
supersedes: []
superseded-by: null
revisit: A required cross-host client, network filesystem, power-loss durability guarantee, or measured IPC representation limitation changes the selected premises.
verification: Plan 38e publication/response-loss, paused-reader/collection, independent borrower and complete offline backup/restore controls.
---

# ADR-0187: Coordinate sealed payloads and strong result snapshots

## Context

Plan 38e separates bulk scientific bytes from shared lifecycle arbitration. Current result bytes live in SurrealDB and result reads renew finite protections; a lazy file reader needs protection for future unread chunks throughout a supported pause. The maintainer selected SurrealDB coordination plus immutable files and the existing process-crash/restart guarantee.

## Scope

Amend blueprint §19/§20/§21 for private same-host Linux storage, independent clients and supervised workers. Cross-host files, online backup and power-loss qualification are outside this decision. Scientific definitions and assessments retain their existing authority. This is a target amendment, not a standards deviation; ADR status changes still follow the decision-PR route.

## Drivers

Bounded demanded output, independent durable writers, truthful acknowledgements, paused lazy reads, retained scientific artifacts and recoverable offline backups must compose. Operations owns opaque file/catalog lifecycle, Runtime owns scientific codecs and readers, and supervisor tooling owns operational freeze/drain/copy. Local mathematical execution does not acquire storage permissions merely to compute.

## Options

Retaining database bulk bytes is viable but keeps result movement within the RPC envelope. Embedded SQLite would replace coordination and require a different complete lifecycle. Per-writer logs add shared discovery/arbitration machinery. Retaining SurrealDB for fitting transactional coordination while using completed native Arrow IPC files preserves existing heads, claims and receipts and exposes library-owned file reads. This adds a filesystem/catalog recovery boundary that must be explicit. Parquet and binary seed conversion remain unselected without a concrete representation need; no performance gain is asserted.

## Outcome

Keep catalog metadata, locators, immutable receipts, claims, retirement and borrower arbitration in SurrealDB; move immutable bulk result/artifact/seed bytes into one sealed file tier rooted in selected private state and storage incarnation. Preserve JSON seed encoding. Completed uncompressed IPC files pack compatible batches from one attempt/relation/schema/interpretation; default packing targets 16 MiB with a 32 MiB hard file bound and flushes at demanded acknowledgement/live-prefix boundaries.

Reserve the original publication intent and unique physical instance before file work. Complete/validate/flush and no-replace publish the file before atomically attaching its exact descriptor/borrowers/receipt under current fences. A lost response settles the original effect; it does not rerun science or renew authority. Content, encoding, occurrence and physical-instance identities are distinct. A physical path is never reused, even for equal content. Deduplication checks bytes, not hash equality alone.

An admitted read owns a nonexpiring exact manifest ticket covering future unread files. Plans, partition streams, read-ahead and release work hold it until actual drain. Finite grant deadlines remain permission controls, not reader death proofs. Register each file owner against store incarnation, Linux boot/PID namespace/PID/start identity; managed workers also bind their invocation. Children register independently before file work. Reaping requires exact death/drain; unknown remains protected. Collection admission and new borrower admission arbitrate transactionally, and a collecting instance never revives.

Backup closes admission and excludes GC before freezing its catalog; authenticated owners drain before offline copying. Include all required sealed files and unresolved intent state, including acknowledged unpublished output. Legitimately absent/unsealed staging remains unresolved. Publish completion only after inventory/digest closure. Restore uses fresh root/incarnation/credentials and validates closure before activation. Restore preserves admitted analyses, graph/source roots and independent artifact/file obligations while rotating new-creation authority. Exact historical retirement keeps a terminal receipt without advancing the new authority floor; it cannot admit reads or reacquisition. Old-root cleanup still needs authenticated drain. Disposable design-phase data regenerates directly; no compatibility production backend is retained.

### Consequences

File bytes are immutable; a complete file is not itself an AppendAck or complete result manifest. Logical descriptors bind file digest/instance, batch index, rows, schema and interpretation. Exact IEEE values, metadata, original coordinates, demand and partiality remain mandatory. Existing Python table/stream/export/live APIs migrate without a new Python query product. Rust snapshots expose native DataFusion providers over frozen file lists and preserve the existing SQL/function/resource context.

### Compensating controls

Use existing permissions, original-intent receipts and replay floors; retain file intent protection through unknown settlement. Do not reuse physical paths or revoke reads by silence/TTL. Bound packing, lazy handles, decode, output and spill through actual allocation owners. File validity/integrity and scientific admission remain separate.

### Confirmation

The focused target review supplies architectural evidence before dependent code. Plan 38e records implemented scope and named targeted controls, including the 2026-10-14 native crash/reopen/offline backup/fresh-root restore journey and protected semantic validation; Plan 38g owns assembled qualification. No measured throughput or power-loss claim follows from this decision.

## Pros and cons

Native IPC access and bulk-byte locality remove a database payload crossing while retaining shared arbitration. The cost is explicit owner registration, cross-resource publication recovery and joint backup/GC ownership; these cannot be delegated to SurrealDB record references or file existence.

## More information

[Plan 38e](../plans/38e-durable-records-results-and-analytics.md) owns current state and finding contributions. ADR-0180/0182 retain demand, receipt and independent scientific borrower rationale; this record amends their physical realization and result-read lifetime without weakening those obligations.

## Status history

- 2026-10-13 — proposed following maintainer-selected topology/crash boundary and authorized full Plan 38e execution; review and implementation evidence remain distinct.

- 2026-10-14 — implemented with bounded targeted acceptance recorded in Plan 38e; proposed decision status and separately owed assembled qualification remain unchanged.
