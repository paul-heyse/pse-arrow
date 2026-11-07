---
title: "Plan67b: Invocation custody and developer history"
status: in-progress
date: 2026-11-06
doc_role: plan
doc_owner: docs/plans/67-storage-lifecycle-and-build-reuse.md
doc_topics: [design, diagnostics, storage, validation, current-work]
review_sources: [docs/functional_reviews/storage-lifecycle-and-build-reuse-2026-11-06.md]
---

# Invocation custody and developer history

## Target and foundation

Deliver [O02](../functional_reviews/storage-lifecycle-and-build-reuse-2026-11-06.md#O02)
and [O09](../functional_reviews/storage-lifecycle-and-build-reuse-2026-11-06.md#O09).
An ordinary invocation keeps its full working inputs through actual settlement. After settlement
it leaves selected, useful facts and any explicitly held evidence. A developer can inspect a
selected failure or request a new test run after the old executable and Cargo target are gone.
A maintainer can compare two selected runs without retaining the complete working state of either.

**Interface-checked at `c2126df45`.** The native capture owner authenticates an opened original
and refuses if it changes during capture. Its private image and `.capture.json` tie bytes to the
original source and executing path. The performing report carries normalized selection, original
observations, required-capture coverage, and exact inventory commitments. Current failed rerun
uses authenticated report and inventory facts to form a new selection. It maps package, target,
features, binary ID and test name against the fresh native listing, and authenticates the new
executable. There is already a test control for rerun after old executable disposal. Build on
those responsibilities and preserve the distinction between an original native observation and
current authority. [Capture owner](../../crates/pse-host/src/discovery/capture.rs) ·
[Performing report](../../crates/pse-host/src/test_workflow/execute/report.rs) ·
[Current rerun](../../crates/pse-host/src/test_workflow/source.rs) ·
[Rerun controls](../../crates/pse-host/src/test_workflow/execute/runner/rerun_tests.rs)

[Plan66c](66c-preparation-execution-and-service-lifecycles.md) supplies real settlement.
[Plan66e](66e-protected-data-and-result-delivery.md) remains the authority for protected
scientific data, result delivery, and selected raw custody. [Plan66f](66f-developer-operations-clients-and-evidence.md)
owns native developer verdicts, report interpretation, and rerun selection. Reuse its verified
passive observations and compact delivery. [ADR-0252](../adr/0252-ephemeral-developer-evidence-relations.md)
defines finite queries over explicitly acquired developer evidence and names persistent analytical
state as a revisit trigger. The scoped persistence proposal is
[ADR-0255](../adr/0255-bounded-developer-workflow-history.md). The history pilot depends on that
decision route. Preserve the existing finite query and keep scientific `diagnostic_invocations`
separate from developer invocation identity.
The historical interface supplies only the facts its source verified. Keep authorizations,
current physical state, and scientific membership with their original owners.

## Active executable custody

Reuse a selected installed tool under its authenticated generation, where its real launch and
loader contract fit. The same native invocation should borrow one verified built image across
enumeration and run if the performing owner can establish stable access for that whole interval.
Acquire ownership and seal the image identity before it becomes discoverable to consumers.
Where this cannot hold for a mutable Cargo output, capture one private image for the active
operation and release it after the last borrower and actual descendants drain. Retain the original
mode, dependency closure, source association, refusal behavior, and exact handling of cancellation
and uncertainty. Track file lifetime separately from current process and database settlement.

Compare protected digest sharing only for identical bytes across multiple live owners that still
need private images. Require immutable source ownership, independent mode and identity checks,
no mutation or unsafe path alias, and a release protocol through the actual custody owner.
The ext4 probe refused reflinks. Use the existing verified streaming path for cross-device
export. Prefer the native list and test flow for ordinary rerun. Consider
`cargo nextest archive` and `run --archive-file` only if a named transport consumer needs
relocation, and verify the pinned Nextest release and completeness of the associated resources.
An archive brings its own retained bytes and cannot establish fixture or runtime admission.
[Nextest tool pin](../../.config/diagnostic-tools.toml) ·
[Nextest archive contract](https://nexte.st/docs/ci-features/archiving/) ·
[Original-file delivery](../../crates/pse-host/src/developer_evidence.rs)

## Structured history and retirement

Pilot a scoped developer-evidence database within the existing local SurrealDB instance, with
its own namespace or database selection, authenticated role, and schema owner. Verify the actual
credential and process boundaries before publication. Use a separately managed developer instance
if the pilot cannot isolate credentials, working set, storage lifecycle, or recovery from
canonical scientific work. Keep the native test runner independent of history availability.
Use generated or otherwise single-source declarations through the existing schema owner once
the persistence decision settles. The canonical application uses the pinned SurrealDB 3.3.4
client and a patched persistent MX server profile. A storage engine or SDK substitution needs
its own evidence and decision route. [Client pin](../../Cargo.toml) ·
[Canonical server profile](../../scripts/surrealdb-source/source-build.md) ·
[Existing derived evidence boundary](../dev/surreal-substrate.md#project-and-derived-evidence-operations)

| Consumer | Selected facts | Lifecycle |
|---|---|---|
| Developer triaging or rerunning a failure | Original invocation, action and verdict, revision and checkout identity, interpretation and content commitments, coverage and omissions, bounded typed causes. Selected rerun evidence also carries verified original package, target kind and name, effective enabled and absent features, platform, profile, filters and ignored policy, binary ID and test name. | Finite rolling window for an explicitly enabled workflow, with a longer finite failure window justified by the pilot. Persist a rerun selection only when original report and native reconciliation permit it. |
| Maintainer comparing a regression or qualification cohort | Campaign and member identity, selected measures/assertions and their units and conditions, source/toolchain/provider context, original outcomes and omissions. | A named purpose and hold until closeout or documented expiry, with explicit renewal. |
| Operator reconciling a released resource | Exact object/owner identity, bounded lifecycle status and observation times, settlement or release reference, and protection state. | Resource owner supplies state through release. Retain a bounded completed record only for a named consumer. |

Use `DEFINE TABLE ... SCHEMAFULL`, declared typed fields, and indexes for actual read and
expiry patterns. Derive immutable record IDs from the original invocation and selected fact
identities. Store a normalized bounded projection, with original references and content
commitments, without duplicating reports, complete logs, fixture databases, or result payloads.
Authenticate the original through its native adoption route before accepting the projection.
Use an exact transaction for rows that must publish together. Matching retries return the
already committed publication. Reusing an identity with different facts refuses. If the
acknowledgement is uncertain, settle against the original record/intent before retry. Bound
read windows and mutation batches, and separate replay of the same request from a new projection.
Distinct commit failure, capture omissions, native outcome, and release status remain visible.
Keep any required publication retry spool bounded under the performing owner. History
unavailability cannot rewrite the native outcome or cause indefinite retention of routine
working payloads. [SurrealQL schema](https://surrealdb.com/docs/reference/query-language/statements/define/table) ·
[Record identity](https://surrealdb.com/docs/reference/query-language/language-primitives/data-types/record-ids) ·
[Transactions](https://surrealdb.com/docs/reference/query-language/language-primitives/transactions)

Use owner-initiated expiry preview and release through the indexed lifetime fields. Recheck
identity, state, active custody, and holds within the mutation contract. Delete a bounded
selected population using existing owner semantics and record the actual outcome. A held raw
artifact remains with its native file owner and carries a verified reference, digest, size,
purpose, bound, and release condition. An expired or absent artifact has an explicit read outcome.
Default to current rows and only necessary indexes. Additional histories and changefeeds require
a demonstrated consumer and retention budget. Size row/detail bounds and each window using the
pilot's query and growth evidence, then record those limits before ordinary activation.
Verify logical expiry independently from physical bytes and resident state across repeated
ingest/expire, checkpoint/journal cycles and restart on the selected profile. If expiry
does not stabilize storage, qualify the supported recovery/reclamation path before activation.
Keep selected portable cohorts separately from live expiry, and test their actual restore.
[SurrealQL deletion](https://surrealdb.com/docs/reference/query-language/statements/delete) ·
[Backup and recovery](https://surrealdb.com/docs/manage/self-hosted/backups-and-recovery)

## Packages

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| B01 | in-progress | Current capture, report and Plan66 C/E/F owners | Choose the active custody path per real tool, inventory and test consumer. Specify the minimum native export and rerun facts, the release evidence, and the narrow persistence decision route. Check that A's independently verified native export remains readable by the selected backfill path. |
| B02 | in-progress | B01 active custody and working settlement interface | Migrate ordinary inventory/launch borrowers and remove their repeat retained copies. Release private images after drain and preserve explicit holds. Verify distinct compiler contexts, mutation/alias/load failures, interrupted publication, and later observation after a target clean. |
| B03 | in-progress | B01 projection contract and applicable persistence decision | Publish one bounded native test workflow to the selected SurrealDB pilot. Verify an independent reader, scope/credential isolation, duplicate or conflicting publication, unknown acknowledgement and unavailable history. Report actual ingestion and query bounds. |
| B04 | in-progress | B03 independent readback and B02 release for selected consumer | Deliver failure triage and named comparison, wire exact eligible rerun facts into Plan66f's current-inventory resolver, then release superseded full payloads. Verify no-match, absent/changed target, incomplete native reconciliation, held or expired raw facts, guarded expiry, repeated physical reclamation and cohort restore. |

A can release owner-approved incident data after a verified narrow export using the existing
native owner. Backfill from that export only for explicitly named uses. For later ordinary
publication, retire the replaced full-retention routes alongside their consumers. Repeated
equivalent successful or failed test runs may leave bounded facts and useful build state.
They must add zero ordinary retained executable copies after settlement.

## Execution checkpoint — November 7

The private tool controls have one stable image per active operation. The original report and
inventory commitments are sufficient for historical selection and a new inventory lookup.
The first release slice uses the foreground owner's published result and complete physical
settlement, including the absence of an enclosing borrower. It limits release to the two fixed
inventory controls or the fixed compiler adapter. The settled foreground test path also
releases compiler-captured executables identified by the authenticated original inventory.
A native SurrealKit test left the original inventory after both control copies and sidecars
were released. Its selected evidence passed independent descriptor readback. The targeted host
batch retained 6.7 MiB in its inventory directory after image release. It exercised refusal of
changed bytes, interrupted-release reconciliation, original-byte history projection, and failed
rerun from a fresh inventory after old executable disposal. Detached work, borrowed scopes and
independent holds require their own integration. ADR-0255 records the proposed history boundary.

The isolated history pilot has a separate schema and database-scoped writer and reader.
One immutable row binds the original invocation, selected facts, three original content
commitments, and finite expiry. The native publication entry acquires the original foreground
settlement before it reaches the writer. A private pinned-engine control verified schema,
role isolation, duplicate refusal, exact readback, and expiry index selection. The pinned
native fixture passed the Rust publication, competing writer, uncertain acknowledgement,
and exact retirement controls. The original native-workflow publication check has a separate
explicit selection.

Pilot retirement uses a database-clock fence and exact invocation, content and expiry
comparison. A positive hold blocks release. Successful retirement atomically replaces the
selected row with a finite receipt. Independent readback reconciles an ambiguous response.
The receipt remains through the original maximum publication window. Pruning needs an exact
owner route that advances the deployment watermark before releasing the receipt. The persistent
profile still needs physical reclamation and restart evidence.
