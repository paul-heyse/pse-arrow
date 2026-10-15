---
title: Simulation workflow, result and resource evidence
doc_role: evidence
doc_owner: docs/design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md
doc_retention: while-dependent
---

# Workflows, results and resources

Supporting inquiry for the [principal review](../../reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md). Inspected 2026-10-15 against `a08f5fa091db1840fc95b46c4d4f849ce4c5de62` plus the current dirty working tree. This is source/interface evidence, not a separate disposition ledger. No product code, tests, dependency or host configuration was changed, and no simulation was rerun for this inquiry.

## Actual audit evidence and scope

The existing [39d audit](../../../plans/39d-qualification-and-plan38-cutover.md#simulation-audit--2026-10-15) owns its operational observations. The coordinator reread its case outcomes and decisive source during this review. Local artifacts are `build/assessment/p39-simulation-audit-20261015T184030Z/`, particularly `independent-cases/`, `remaining-cases/`, `harness-closeout/`, `managed-closeout/`, their case-audit JSON and process logs. These are dated evidence, not new qualification.

The scalar preparation, fitting and shooting cases completed their original assertions. The cold case was interrupted after approximately 519 seconds before a numerical RunHandle; the scalar sweep was interrupted after 234 native joins, not after completing all 1,000 original point checks. Three cold-preparation-dependent positions were not run. Both result-analysis cases refused before graph activation. The valid managed retry passed sixteen completed/usable/fresh one-attempt outcomes and distinct retained run identities, then failed its first persisted result read before the numerical values and native metric checks in `persisted`. Its primary and fixture drained. A public ephemeral physical-study control also failed before execution.

Those distinctions exclude numerical-convergence attribution for the cold run, whole-sweep success and full managed-study acceptance. The 65 query builders/154 successful statements sampled during the sweep describe one complete operation, not an isolated publication phase or 65 unnecessary requests. The last server statement and native join samples are not total transport and solve time.

## Local study admission — AUD39-04

**Implemented / source-inspected:** Python `NativeModelingPackage::admit_study` in `crates/pse-py/src/workflow/modeling.rs` sends stored physical-source documents to `ModelingPackage::admit_study_input_sources`. In `workflow/study.rs`, that method calls `recorded_revision`, constructs `Operations` from a canonical store and publishes physical sources before admitting the definition. `StudyDefinition::physical` is a recorded `PhysicalSource`. The lower executor supports explicit ephemeral execution, but public admission cannot express the genuinely local source ownership its computation needs. Local modeling packages can have no canonical revision.

The existing failed public test is a concrete trigger. Synthetic canonical identity in lower fixtures is not a sound public solution. Initializing a store merely to manufacture those identities would retain the unwanted coupling.

**Proposed corrective contract:** one owned study intent must describe model revision, admitted physical context, finite/lazy occurrences, dependencies, output demand and cancellation independently of retention placement. A local admitted study retains actual immutable scientific owners in its runtime and cannot masquerade as a portable durable descriptor. Durable publication obtains recorded source handles explicitly before serialization/submission. Public Rust and Python entry points consume the corresponding honest ownership form; scientific admission remains common, not two independently interpreted policies.

The exact local-versus-recorded representation is a subsequent design detail, but its semantics are consequential: no fake receipt, no silent local fallback for a durable request, and no requirement that local execution acquire unrelated infrastructure. Breaking the current persisted definition/Python contract needs explicit versioning and rule-impact consideration, rather than a compatibility shim.

## Response evidence versus scalar local analysis — AUD39-06

`workflow/analyses.rs::Runtime::result_analysis` unconditionally promotes `ScientificArtifactKind::LocalAnalysis`, then builds graph edges from response and parametric sensitivity rows. `workflow/artifacts.rs::promote_scientific_artifact` requires a nonempty `runtime.local_validity` member for that kind. Dynamic trajectory projection supplies forward response rows but not that scalar local-analysis validity product. The two audit failures are therefore a composition/meaning mismatch, not a reason to remove the admission check.

An important existing alternative: `ScientificArtifactKind::Trajectory` already includes response sensitivities, scientific contexts, endpoint interpretation, samples, events and computation metadata. A new artifact family is not automatically necessary. Existing local validity also carries root and KKT-specific facts; it must not be populated with fabricated KKT verdicts for a transient derivative.

**Proposed target:** response evidence declares its scientific family and scope. Integrated responses retain trajectory/time/event/derivative-method assumptions; implicit root responses retain rank, branch and local-neighborhood conditions; optimal responses retain active-set/KKT conditions. An analysis consumer dispatches on admitted meaning, not on the shared table name or numeric finiteness. Existing trajectory and local-analysis artifacts can provide physical storage where their promises fit; new persistent kinds/relations are justified only by a missing consumed contract.

A trajectory response can be preserved as recorded response evidence without being promoted to an optimal-solution sensitivity. Conditions needed for a stronger derivative claim must be checked by its producing numerical owner and carried or withheld explicitly. Graph reachability does not certify a derivative, and accepted primal integration does not establish derivative validity by itself. Preserve independently retained artifacts, exact run/attempt/field coordinates, units, native-to-physical scaling and partial trajectory meaning.

## Metadata reservation overlap — AUD39-07

`workflow/connected_results.rs::{response_owner,results,result_selection}` reserves `3 * MAX_FILE_BYTES` for the selection response. It splits only the retained set-name allocation, while the enclosing reservation remains alive through `ResultSnapshot::capture`. `workflow/result_snapshot.rs::capture` charges retained read metadata and separately reserves the same 96 MiB allowance for each metadata page. `ResultSnapshot::reader` uses that same file-size-derived bound again for file-open metadata. Similar response allowances appear in artifact and analysis consumers.

These metadata pages do not transport IPC payloads. `canonical_results.rs::{result_block_page,result_file_batch_descriptors}` bounds pages to 64 entries and validates exact protected set/ordinal coverage. File payload decoding has separate file-shaped grants. The shared metadata allowance nevertheless derives from maximum physical file bytes.

The valid managed retry's 256 MiB shared fair pool held a 96 MiB selection allowance and two real 64 MiB compiler workspace leases when the next 96 MiB page was requested; only approximately 27.5 MiB remained. Its external observer process cap was 4 GiB. This establishes the reported pool-composition failure, not host OOM, actual payload size or redundant compiler work.

DataFusion 55.1.0's installed `MemoryReservation::{split,try_shrink,new_empty}` provide transfer and exact-release primitives. They do not decide application metadata bounds or make active memory disappear. Its `FairSpillPool` requires consumers with actual spill behavior; setting a spillable flag is not an eviction implementation for native workspaces or protected metadata.

**Proposed target:** separate temporary RPC allowances, validated retained metadata, frozen inventory and file decode ownership. Transfer each retained value into its continuing owner, and retire/reuse only the temporary allowance whose actual SDK work is drained. Bound metadata responses by their decoded descriptor contract plus bounded transport/intermediate copies, not by physical file size. Account for potentially overlapping SDK requests and cancellation suffixes.

The vendored SDK's `RequestContext` retains payload ownership through router work; `canonical_transport.rs::response_ownership_scope` composes enclosing/current owners and propagated tasks. Those protections must survive any reduction. A returned future alone is not blanket authority to shrink every aliased request owner.

`MathService::reserve` already trims idle compiler and native lanes on pressure; the result readers reserve directly from the pool. Shared optional-retention pressure is a composition opportunity after removing oversized fixed overlaps. Preserve active borrowers and useful warm generations; do not infer redundancy from reservation names. Reconcile the observer's declared workload and pool only after establishing required live ownership. No arbitrary cap increase is selected here.

## Durable point protocol — AUD39-05

`workflow/durable.rs::DurableAttempt::finish` drains progress, exports selected tables and seeds, prepares original completion, reconciles ACK inventory and seals while maintaining renewal. `store_tables` already distinguishes mandatory summaries from explicit detail demand. Original independent assessment and original completion are not redundant solver-status checks.

`workflow/result_projection.rs::ResultAppendQueue` already batches compatible blocks into file payloads, and canonical `append_result_blocks` supports bounded metadata batches. The queue flushes on relation-name changes and at nonempty table completion. Empty memberships use individual `declare_empty_result_set` calls. The useful remaining comparison is bounded metadata grouping across separately encoded compatible file intents and empty memberships, not a claim that all current appends are scalar or that unlike schemas can share an IPC file.

Potential reductions include carrying immutable metadata acknowledged by registration, grouping compatible finalization calls and sharing scientific source protection under an actual operation-owned lifetime. Changing run/attempt/cancellation state still requires a current fence; deployment, retirement, expiry and active-reader protection remain. Claim, native execution and publication have different lifetimes; reducing round trips must not create a long transaction around a solve or weaken retry idempotency.

The failed `canonical_append_queue_packs_relations_and_flushes_before_empty_membership` test expects two different relations to remain unacknowledged after each complete table call, zero append requests, and two queued items. Current `finish` explicitly flushes a nonempty table, so the first ACK can legitimately exist. The test's no-ACK assumption does not establish corruption. Further, `flush` encodes one file with the first item's schema: preserving different schemas in separate files is necessary. Reconcile the test and any older batching promise with the target before changing either. A future metadata-group test should assert actual ACKs and per-file schemas, not absence of all legitimate earlier effects.

Contrary evidence retained: registered file-owner lookup is cached; protected execution queries address the run directly; intent checks on either side of the OS instance lock protect different intervals; sealing retains the original completion rather than reopening it. No optimization is scheduled against these mechanisms merely because they appear twice in an operation.

## Library research and verification boundary

Context7 was resolved and queried for Apache DataFusion (`/apache/datafusion`, memory-reservation/pool ownership) and SurrealDB (`/surrealdb/surrealdb`, batched statement responses/errors). The returned current-main documentation was used as discovery only. Exact installed DataFusion 55.1.0 source and this checkout's vendored SurrealDB 3.3.0 SDK supplied the relevant implementation contracts. Current-main transaction/disconnect behavior was not transferred to this native fork.

Primary discovery references: [DataFusion memory pools](https://github.com/apache/datafusion/blob/main/execution/src/memory_pool/mod.rs), [DataFusion pool policies](https://github.com/apache/datafusion/blob/main/execution/src/memory_pool/pool.rs), [SurrealDB query response API](https://github.com/surrealdb/surrealdb/blob/main/src/method/query.rs). Exact local source remains the evidence for versioned conclusions.

No new product tests, builds, services or quantitative probes ran. Suitable later controls are public local/durable study admission; family-specific response certification and withholding; response-transfer/cancellation ownership; nonempty snapshot paging under the observer pool; multiple schemas with exact ACK inventory; and fresh original numerical checks after complete managed result reopening. The old empty-demand `0..0` reader control bypasses the failing snapshot page and is insufficient for that overlap.
