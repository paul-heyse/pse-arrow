---
title: Studies, response artifacts and result resources
status: planned
date: 2026-10-15
adrs: []
review_sources:
  - docs/design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md
scenario_sources:
  - docs/design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md
doc_owner: docs/plans/41-simulation-pipeline-efficiency-and-effectiveness.md
doc_topics: [design, storage, preparation, validation]
---

# Plan 41c: Studies, response artifacts and result resources

## Purpose and boundary

This companion owns corrective delivery for review F09–F11 and bounded O02 under [Plan 41](41-simulation-pipeline-efficiency-and-effectiveness.md). Scientific source admission, response interpretation, metadata lifetime and durable publication are distinct responsibilities. Their composition must support public local studies, truthful analysis and feasible nonempty retained-result reads without a universal lifecycle rewrite.

**Interface-checked basis:** the principal review, [workflow/resource inquiry](../design_review/evidence/simulation-pipeline-efficiency-and-effectiveness-2026-10-15/workflows-results-and-resources.md), current public admission, unconditional LocalAnalysis dispatch and file-shaped metadata grants. **Proposed:** contracts, packages and future acceptance here. Blueprint §0.5/§19–§21 governs the current source, analysis and publication promises. RC01's proposed change awaits operator confirmation; RC02 preserves existing artifact promises unless a separate concrete extension is accepted. The coordinator owns those decisions and finding dispositions.

This document does not weaken original-space assessment, scalar local validity, scientific identity, protected read tickets, ACK inventory, source retirement protection or SDK ownership through actual drain. It does not claim a safe refusal is a physical error. The managed audit's sixteen usable outcomes preceded a failed persisted read; its numerical result checks were not completed.

## Foundations and affected owners

`pse-py` modeling admission forwards source documents to runtime `ModelingPackage::admit_study_input_sources`. Runtime `workflow/study.rs` obtains a recorded revision and publishes physical sources. The lower ephemeral executor is useful but cannot repair this public dependency. `StudyDefinition` currently carries recorded physical-source identity. Existing occurrence policy, explicit/lazy variants, preparation lanes, continuation ownership and typed outcomes should remain the common scientific machinery.

`workflow/analyses.rs::result_analysis` always promotes LocalAnalysis, while `workflow/artifacts.rs` requires its scalar local-validity evidence. Trajectory already includes response sensitivities, samples, events, endpoint interpretation and computation metadata. Existing artifact kinds are therefore the first composition seam; an added persistent family is not a prerequisite established by this review.

`workflow/connected_results.rs` and `result_snapshot.rs` already separate shared snapshot/reader ownership, protected files, decoded batches and retained metadata. The defect is oversized temporary allowances and their overlap, not the absence of all transfer accounting. Selection, page and file-open responses use `3 * MAX_FILE_BYTES` despite bounded metadata contents. Artifact/analysis response helpers share that shape. Canonical pages have at most 64 descriptors; row count alone is not a complete byte bound if variable fields are unbounded.

The vendored SDK `RequestContext` and canonical `response_ownership_scope` retain request payload owners through router suffixes. Caller future completion/drop is not blanket drain. DataFusion reservations provide transfer/shrink primitives, but neither choose application bounds nor magically evict native state. Existing fixed workspace leases are not shown duplicate or shrinkable from RSS. Use the evidence matrix for exact installed/vendored interfaces; resolve new API questions through the library skills/Context7 and exact source, not stale capability catalogs.

## Local scientific admission and explicit recording

C0 defines the honest ownership forms consumed by C1 before changing APIs. One scientific study intent describes model/physical context, finite or lazy occurrences, dependency/start policy, output demand and original operation controls. A runtime-local admitted form retains actual immutable model/physical owners; it is not a fabricated portable descriptor and need not serialize. A recorded descriptor contains genuinely published handles suitable for durable transport. Shared scientific admission interprets both forms through their admitted scientific context rather than maintaining two policies.

Public Rust and Python local calls must admit and execute physical studies without initializing or contacting a canonical store. Input/source owners remain live through pending/lazy preparation, issued work and result use for their declared lifetime. Lazy variants admit selected occurrences under current prerequisites rather than publishing the whole hypothetical domain. Repeated bindings remain distinct occurrences; ordering and causal predecessor policy are unchanged.

Durable execution requires an explicit recording operation that publishes model/physical sources and constructs the portable descriptor. A durable request with unavailable recording fails clearly; it never silently becomes local. Local inputs cannot acquire fake recorded IDs merely to pass a lower executor fixture. Migrate public ingress, direct Rust consumers, Python wrappers, generated declarations/stubs, durable submission/decoding and test builders together. Regenerate from declarations, never edit generated output. Do not retain the old compulsory-publication local path as a compatibility API.

RC01 must be accepted and its required boundary amendment/ADR installed before dependent production changes. C0 records exact public and persisted contract changes and the actual encoding/version route. This design-phase project does not require historical data preservation or a second reader/executor solely for old fixtures; regenerate owned disposable fixtures where appropriate, without inferring deletion authority over unrelated data/services. If RC01 is rejected, document F09's unresolved limitation rather than substituting store setup.

## Response meaning and artifact composition

Select response interpretation from admitted scientific family and exact run/attempt/field context, not relation name, graph reachability or numeric finiteness:

| Family | Consumed meaning and preservation |
| --- | --- |
| Implicit/root | Rank, branch, neighborhood, original coordinates/units and admitted derivative order; unsupported or invalid response stays withheld. |
| Optimal | Actual active-set/KKT/curvature conditions under the producing owner's contract; primal success alone does not certify sensitivity or covariance. |
| Transient | Time origin, interval/sample/endpoint, mode/event/schedule, start, method/accuracy and actual sensitivity capability; a trajectory response is not a scalar KKT response. |

Reuse Trajectory for response evidence within its current promise and LocalAnalysis where its validity promise applies. Keep LocalAnalysis's nonempty validity guard. Existing transient response recording can be weaker than strong derivative certification; expose the weaker meaning explicitly and withhold downstream claims requiring unavailable premises. Do not populate synthetic KKT rows or remove admission checks to satisfy an old test.

C0/C2 identify how current producer metadata supplies family and scope and which Rust/Python/graph consumers need it. Prefer existing descriptors/relations; a missing persistent validity distinction triggers RC02's concrete decision route before new authority is introduced. Graph edges preserve quantities, units, scaling, response family and evidence coordinates rather than flattening them into an unqualified dependency. Artifact and analysis ownership transfer retains honest unsettled outcomes on failure. Independently retained evidence remains meaningful after producer/graph retirement; graph deletion cannot invalidate another owner's retained artifact.

## Metadata ownership and feasible reads

C3 separates four lifetimes: temporary RPC/SDK representations, validated retained metadata, frozen snapshot inventory and file payload/decode buffers. Each continuing value has one charge at its actual owner; simultaneous copies and live request suffixes remain charged for their real overlap. Aliases do not independently duplicate immutable ownership, and a shared owner cannot shrink storage still retained by a router/task.

Derive finite response allowances from each endpoint's metadata shape, variable-length limits and bounded transport/intermediate copies. Inspect selection/header, result block pages, file/batch descriptors, file-open metadata and artifact/analysis responses, including corresponding canonical SDK paths. Where a necessary bound is not enforced, establish it at the proper protocol/decoder boundary before relying on it; a small sampled response or 64-row count alone is not a safe maximum. Keep checked extent arithmetic and explicit refusal.

On validated return, transfer retained storage to its continuing owner and release only the temporary storage whose actual ownership has ended. Scope sequential page grants so they do not accumulate accidentally; preserve genuine overlapping RPC suffixes on cancellation. Protected read/file owners survive until every consumer/SDK borrower drains. Decode grants remain file-shaped and separately admitted. Avoid charging an IPC maximum for descriptor-only work, removing accounting or increasing caps as the corrective mechanism.

Qualify nonempty selection→page→file-open→decode journeys with the observer workload that exposed the defect, including actual compiler leases and cancellation. A `0..0` empty selection does not exercise it. Result-side pressure integration with demonstrably idle retention is a subsequent B4 decision after C3 is implemented and remaining contention is understood. Do not introduce a new manager or automatically evict useful warm generations.

## Durable grouping investigation

C4 first settles the operation boundary and existing queue oracle. `ResultAppendQueue` batches compatible blocks and flushes at relation changes/nonempty table completion; table completion can legitimately produce an ACK. The historical mixed-relation test's assumption that no ACK exists after completed tables is not a sound corruption oracle. IPC files require their own correct schemas.

Compare grouping compatible metadata finalization/file intents, empty memberships and already acknowledged immutable metadata with the present protocol. Keep differently shaped IPC data in separate files. Group only bounded metadata work; do not hold a transaction across a native solve. Changing run/attempt/cancellation states still requires current fences. Preserve complete demanded outputs, original independent assessment/completion, exact ACK/manifest and idempotent retry identity. Operation-owned source protection may replace repeated admission only if its actual permission covers the whole effect/recovery lifetime.

Settling evidence separates publication phases from whole-operation query builders/statements, reconciles the table-completion contract and tests per-file schemas and real ACK inventory. Adopt grouping where it removes unnecessary crossings while retaining the complete failure contract; otherwise record a supported no-adoption decision and correct any unsound test oracle. Queue tests must not be rewritten merely to mirror incidental implementation. Unknown partial append/seal outcomes remain explicit and protected until reconciled.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| C0 | Principal review and focused current sources; pending rule recommendations visible | Settle local/recorded ownership and common admission, existing-family artifact dispatch and exact metadata transition/bound inputs. Map actual ingress/serialization/graph/read consumers. Identify any additional concrete rule impact before fixing dependent target. | Planned |
| C1 | C0 local/recorded contract; accepted RC01 and installed G0 amendment | Implement public local study admission and explicit durable recording, migrate Rust/Python/portable consumers and regenerate declarations. Tests: no-store physical local explicit/lazy studies, honest durable refusal/publication, duplicate occurrence order, cancellation and owner lifetime. Delete compulsory-publication local APIs, fake-receipt fixtures and replaced tests. | Planned |
| C2 | C0 family contract; current artifact promises or separately accepted extension | Implement family-qualified analysis/artifact/graph composition with actual producer evidence. Tests: valid and withheld root/optimal/transient, event/partial trajectory, units/coordinates, scalar guard and independent retention/transfer failures. Remove unconditional LocalAnalysis dispatch and tests asserting the invalid universal promise. | Planned |
| C3 | C0 per-endpoint bounds/ownership; F0 observation meanings where consumed | Replace file-sized metadata grants across actual selection/page/file-open/artifact/analysis consumers; transfer retained storage and preserve SDK suffix ownership. Tests: nonempty reads under intended pool/workspace load, bounded/unbounded-field refusal, concurrent/cancelled suffixes, retained aliases and eventual exact release, separate payload admission. Delete obsolete broad grant constants/helpers only when all users migrate. | Planned |
| C4 | Current queue/ACK semantics settled; F1 relevant protocol facts; C3 only where new grouping consumes its grants | Investigate and deliver justified metadata grouping, or record no-adoption with evidence. Reconcile mixed-schema queue oracle first; test complete outputs, distinct schemas, empty membership, real ACKs, retry/partial effects, source expiry/retirement and attempt fences. Remove replaced protocol/test paths together. | Planned |
| C5 | C1–C3 integrated and C4 settled; B1 cancellation for dependent actual runs | Complete actual consumer/deletion inventory and targeted public Rust/Python/local/durable/managed read controls. Supply scoped evidence to G1/F2 without claiming original audit campaign completion. | Planned |

## Verification and current checkpoint

**Proposed:** compile touched owners and run focused controls during implementation. Pure admission/family/extent tests use actual local dependencies; canonical protocol/SDK lifetime behavior uses owned service fixtures only where necessary. Actual Arrow correctness closures explicitly use force-validation. Nontrivial numerical response controls retain independent original checks; a populated graph or successful serializer is not sensitivity qualification. Code generation accompanies declaration changes.

41f owns combined scope-end journeys/hygiene/manual checks. Report actual command, mode, zero baseline and exclusions at the owning outcome; no statistical timing, environment attestation or arbitrary capacity gate is added. All C packages remain Planned. The next dependency is C0's concrete contract assessment; C1 remains conditional on RC01/G0. Existing plans/review/source data are not modified by this authored companion.
