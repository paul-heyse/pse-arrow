---
title: "Plan 53e: Resource and evidence lifetimes"
status: draft
date: 2026-10-23
adrs: []
review_sources: [docs/design_review/reviews/design_review_test-architecture-target_2026-10-23.md]
scenario_sources: [docs/design_review/reviews/design_review_test-architecture-target_2026-10-23.md]
doc_role: plan
doc_owner: docs/plans/53-test-architecture-target.md
doc_topics: [current-work, design, validation, storage]
---

# Plan 53e: Resource and evidence lifetimes

## Target and preserved protection

**Proposed:** remove repeated selection/detail publication and unrelated cleanup amplification while retaining authentic custody and recovery under [R07](../design_review/reviews/design_review_test-architecture-target_2026-10-23.md#r07). The [coordinator](53-test-architecture-target.md#finding-dispositions) owns state. `resource_store`, `test_resources`, actual process/service owners and receipt readers retain authority; no journal, new lease service or second outcome store is selected.

DTA-041 remains withdrawn: pending/uncertain registration, legacy-writer fences, exact borrowers, late effects, terminal freeze and conservative unknown retention have live consumers. DTA-040's legitimate aggregate snapshot semantics and existing gate/combination reuse/transfer remain valid. Historical execution and present applicability are separate; do not invent a global receipt redesign or mandatory attribution proof to close them. Drop/drain is not Pass.

## Work packages

<a id="e01"></a>
### E01 — Immutable invocation selection, mutable membership

Use the existing FileStore declaration versus invocation-state seam. After actual tool collection, publish one authoritative immutable selected header, bound by invocation identity, nonce and digest. Distinguish uncollected selection from valid collected-empty. Mutable state carries phase, member/producer references, projections and terminal references, rather than the complete selected payload on each registration/settlement.

C01/C02 supply normalized identity and actual selection. Preserve invocation→identity→resource lock ordering, the original deadline, Pending/not-started→uncertain→registered transitions, `_settle_members`, context admission and pre-effects visibility. Physical separation must not create two authorities or a selection publication gap that admits effects early. Readers validate the bound header before trusting membership; mismatch, duplicate registration and late post-terminal mutation refuse.

Migrate every registration, recovery, settlement, diagnostic and receipt consumer in the same coherent cutover. For stored development state, use an explicit versioned decode with a one-time conversion only when the old record's identity/selection/terminal can be authenticated. Uncertain old records remain protected under the existing reader/fence until safe cleanup; never relabel missing selection as empty or bypass legacy-writer protection. I09's reader assessment determines whether protected old records require an explicit offline conversion before removing the old layout. This is a bounded data transition, not a permanent dual production path.

Acceptance interleaves/interrupts header and mutable saves, checks collected-empty/uncollected, stale/mismatched header, duplicate/late registration, context admission and terminal freeze. Show mutable updates no longer carry the selected payload while exact association remains. Remove repeated payload construction/writes with migrated consumers, keeping original diagnostics recoverable.

<a id="e02"></a>
### E02 — Continue independent settlement under one deadline

Change Python drain/drain-context/record-drain and Rust peer-connection joins so one independent sibling failure does not prevent remaining settlement attempts. Collect errors and uncertain states; retain the original invocation deadline and cancellation rather than restart a full timeout per sibling. Mark Drained only when all required members settle. Cancellation can stop new unsafe work but must preserve already possible effects and unknown borrowers.

Use the existing resource/process owners, not a new generic task scheduler. A failure on one reclaim target likewise must not prevent independent eligible targets from being considered. C04 consumes cleanup facts separately from frozen assurance. Test one failing sibling with a second successfully settled sibling, exhausted original deadline, cancellation, late effect and retained reader. Preserve error detail and active protection; aggregation must not turn partial cleanup into success. Remove short-circuit loops only after equivalent ordering and retention controls pass.

<a id="e03"></a>
### E03 — Service-scoped active access as a conservative derived index

Avoid global invocation scans for a service-local maintenance decision. Maintain a service/generation-scoped active-access marker under existing FileStore save ordering, including creation borrowers. Publish protective markers before authority can expose an effect; remove authoritative membership before deleting markers. The marker is a derived lookup, not a lease or second membership authority. Stale/missing/corrupt/ambiguous state cannot authorize unsafe maintenance; resolve through the authoritative owner or conservatively refuse.

Exercise registration versus maintenance races, retained-reader lifetime, service generation replacement, interruption between index/authority steps and unknown records. Preserve exact borrower membership and legacy fences. F02 consumes this protection for service claims but does not treat index absence alone as proof of service quiet. Delete repeated whole-store scans only for decisions now safely answered by the scoped contract; keep recovery paths needed for unknown state.

<a id="e04"></a>
### E04 — Separate evidence custody from reclaim candidacy

Sealing, authenticated manifests, referenced origins and retained-reader protection remain necessary even when outputs are nondisposable. Reclamation candidate selection uses an explicit nonempty scratch projection of a sealed report; all-protected reports still seal and protect reuse origins but need no futile reclaim traversal. Do not infer disposability from report existence or discard protection because nothing can be deleted.

Migrate report sealing/candidate producers and reclamation consumers together. Targeted controls cover all-protected report, actual scratch candidate, active reference, manifest/path digest tamper and failure followed by another independent candidate. Preserve original evidence inspection and origin reuse; cleanup cannot create or rewrite an assurance Pass. Delete only redundant candidate scans and obsolete coupling of sealing to disposal.

<a id="e05"></a>
### E05 — Compact atomic progress with original detail

Use one small atomic checkpoint root referencing immutable source/identity and per-gate detail. Publish detail before its root pointer, validate references/digests and reject dangling/corrupt artifacts. Preserve original failure/report detail and selectively hydrate only the requested gate or full detail. Update `validation_result.read_receipt`, `selected_result` and `reuse_checks` together through one versioned reader; Plan51 evidence/import readers are consumers too where they read this format. A result query must not trigger builds, science or unrelated service traversal.

I09 settles the exact stored-reader inventory, supported version transition and immutable detail custody before changing automatic reuse. Referenced detail must retain protection for as long as any root/origin consumer can use it. Preserve parsed native Cargo/unit-graph correspondence and actual force-validation checks; replacing a large payload with a digest does not discharge semantic validation. No append-only event log is needed: that would add replay/truncation/compaction/recovery obligations without a selected capability.

Acceptance covers detail-before-index interruption, failed atomic root replacement, missing/corrupt reference, selective `just result --gate` detail, full original diagnostic equivalence, authenticated reused origin and sealed manifest validation. Old retained roots are either authenticated/migrated by the bounded transition or explicitly refused for reuse while original inspection/protection is preserved. Remove duplicate growing payload copies and old reader branches once the selected transition is complete; do not keep a parallel result store.

## Sequence and integration

E02 and E04 can start independently. E01 needs only C01/C02's identity/catalog slice; C04 can consume existing layout until E01's coordinated migration, avoiding a whole-document cycle. E03 preserves the same authority/save-order semantics and can precede broader scheduling changes. E05's detail-reader contract can be settled early, with its final consumers migrated after C04/E04 as needed. Shared FileStore/result-reader surfaces require one editing owner; logical independence does not authorize conflicting edits.

53g owns composed interruption/retained-reader/resume qualification. No speed or capacity claim follows from less rewriting. Completion means the removed amplification is absent, exact custody/recovery semantics still hold and every stored-format reader and active-origin transition has a deliberate route.
