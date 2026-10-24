---
title: "Plan 52n: Readiness, progress and edit lifecycles"
status: draft
date: 2026-10-24
adrs: []
review_sources: [docs/functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md]
doc_role: plan
doc_owner: docs/plans/52-agent-effectiveness-and-engineering-workflows.md
doc_topics: [current-work, environment, diagnostics, design, validation]
---

# Plan 52n: Readiness, progress and edit lifecycles

## Target and foundation assessment

**Proposed:** make preparation, running work and retained editing state independently inspectable
through their existing owners. This develops usability-review
[O08](../functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md#O08),
[O10](../functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md#O10) and
[O11](../functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md#O11).
The [coordinator](52-agent-effectiveness-and-engineering-workflows.md#usability-opportunity-dispositions)
owns state; completed ADP/MCP worker, observer and preview contracts are the starting point.

**Interface-checked:** fixed worker transport currently waits for the final structured response,
while scoped Rust execution already publishes an original invocation reference before settlement.
`Specialists` retains exact preview bytes under count/byte bounds without evicting issued handles;
reset changes presentation rather than capacity. The 2026-10-24 Plan53 preparation commit
`deeb37cc4` separates Python stages. This is inspected implementation, not evidence that its MCP
readiness/preparation consumers or remaining Plan53 qualification are complete.

Apply Core3.4, Heuristics1.0, Test Assessment1.2 and PS-10/PS-11 where science is involved.
AP-05/AP-06/AP-07 and H12/H17/H19/H25/H27 favor exposing existing states and releasing abandoned
state without introducing a second lifecycle/scheduler. No whole-host health sweep, endpoint-wide
workload allocation or new persistent preview database is selected.

## Shared lifecycle meanings

Declared availability, prepared artifacts, compatible imported bytes, live selected service and
actually exercised operation are different facts. Passive description stays source-only. Explicit
preparation and selected live checking disclose their effects, use their native environment/admission
and preserve actual binary/schema/source fences. A pure test target does not prove that its current
worker's broader import closure is prepared. Missing readiness must not trigger an automatic build
or source synchronization merely because an agent browses the catalogue.

An authentic issued handle enables original-owner watch/read. It does not convert an attached action
to safe detached execution. Preparation, execution, publication, assurance and drain retain separate
outcomes. Transport loss/cancellation cannot manufacture native settlement or retry eligibility.
Preview release ends future handle resolution; it does not undo or certify edits.

## Work packages

<a id="LIFE01"></a>
### LIFE01 — Selected readiness and explicit preparation consumers

Expose owner-defined prerequisites and readiness facts for a selected capability, with exact
preparation routes and clear evidence of which checks are passive versus live. Consume Plan53d's
actual environment/extension/stub stages and source capture, not a duplicated builder or stale
assumption that synchronization must always regenerate everything. Keep replacement, installation,
publication and preparation effects distinguishable in schemas/descriptions.

Use the original operation kernels for explicit actions. A described command is not a fabricated
readiness receipt; unknown state stays unknown until a selected check establishes it. Check only
the chosen dependencies and retain identity/source membership where it matters. Do not weaken native
byte/import fences or add a universal cache/health service. CON03 supplies the safe remedy view;
NAV03 keeps pure readiness/schema inspection independent of execution workers where actually possible.

Targeted acceptance: browse/source-only task with unavailable compiler/science/store providers;
prepared-but-incompatible imported extension; selected live service unavailable; explicit preparation
followed by a real operation. Independently check stage outcomes and absence of eager startup or
implicit publication. Keep Plan53d progress/qualification there. Remove superseded combined adapter
preparation calls only when all actual consumers use the owner stages.

<a id="LIFE02"></a>
### LIFE02 — Authentic early handles and foreground progress

Extend the existing fixed worker exchange with bounded observation of owner-issued stage/handle facts.
Use existing callbacks/structured references such as `rust_tests.execute`'s invocation-reference
publication, not log scraping or a predicted invocation ID. Bind the observation to the exact worker
request/owner and publish only once the original identity exists. Keep sensitive payloads private;
coalesce bounded stage notifications rather than one event per native primitive.

For a supporting host use FastMCP4.1.3
[`Context.report_progress(progress, total, message)`](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/fastmcp_slim/fastmcp/server/context.py)
with the actual caller progress token. Injected
[`Progress()`](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/fastmcp_slim/fastmcp/server/dependencies.py)
alone uses foreground `InMemoryProgress`, so it is not the selected wire bridge. Represent unknown
totals truthfully; no time-derived percentage. Preserve original handle/watch/read as the fallback
when notifications are not rendered. A source-only readiness read does not start observation workers.

Preserve shielded spawn/wait ownership, signaling and physical drain for attached actions. Detached
observer cancellation still leaves its subject running. Endpoint loss exposes original references
where actually available; a helper exit is not a native terminal record. Reuse existing correlation
facts only when useful; this work does not resolve F02's application/database trace seam.

Targeted acceptance: preparation refusal before handle issuance, actual handle before final return,
stage transitions, notification coalescing, no-token fallback, interrupted observer and repeated action
cancellation during spawn/execution/drain. Compare final results with original native readers. H02
requires actual Codex notification behavior for that host claim; an in-process client proves only its
own path. O16's optional tasks are evaluated later against this delivered foreground/native baseline.
Remove replaced final-only adapter observation when the bridge serves its callers; keep native owners.

<a id="LIFE03"></a>
### LIFE03 — Explicit preview retention inspection and release

Add owner-local retention status and exact-digest release to `Specialists`, exposed through the same
canonical endpoint with truthful effects. Status reports retained preview count/bytes, configured
capacity and selected handle availability without returning every source body. Keep current capacity
defaults unless evidence calls for a deliberate change; do not add a machine-memory governor.

Successful release makes future resolution of that endpoint's exact handle unavailable. Report
released versus unavailable; a well-formed unknown or foreign digest returns unavailable without
distinguishing its history or affecting siblings. Malformed digests retain structural refusal.
No tombstone ledger is needed. Release never applies/rolls back an edit or declares reconciliation
complete. Admission and release synchronize at the existing preview/apply owner: refuse release
while work is admitted or its original handle is still needed for unresolved reconciliation.
Keep that digest resolvable through the existing `edit_preview_reconcile(digest)` route. In particular,
an interrupted/partial apply cannot become unreconcilable merely because its bytes remain allocated
somewhere. Mark required reconciliation before effects; clear that obligation only from the authentic
apply/reconcile outcome. Completed reconciliation must classify all affected files, including
diverged/missing states; observing those states does not mean repair or rollback. Then an explicit
release may reclaim the retained capacity. Account for all
still-required bytes; no general lease/history framework. Reset continues to affect presentation
only; endpoint close retains its existing semantics and I14's independent inquiry.

Targeted acceptance uses small explicit capacities: fill retention, refuse without eviction, release
one handle, reclaim its available capacity, preserve siblings, return unavailable for well-formed
foreign/unknown/repeated releases, and refuse release during admitted work or unresolved reconciliation.
After an interrupted apply, the same original digest must still reconcile; only settled obligations
permit subsequent release. No release result invents rollback or erases required evidence.
Remove the close-only assumption in current guide/callers/tests
once the extension is integrated; no silent LRU, TTL or durable preview store is introduced.

## Dependencies, verification and publication

**Proposed:** LIFE01 consumes actual Plan53d contracts and CON03 diagnostics; it does not take over
Plan53 status or wait for all Plan53 qualification. LIFE02 can advance from existing original-owner
references, with CON02/CON03 contracts for useful typed presentation. LIFE03 is independent of protocol
tasks and CodeMode; coordinate source edits with NAV03 when both change specialist/worker wiring.

Use focused owner/adapter/cancellation tests and affected compilation during implementation. Existing
Python/asyncio/context-manager and native cancellation primitives are preferred where adequate;
`AsyncExitStack` or structured sibling settlement needs a real acquisition/cleanup responsibility.
AnyIO solely to rewrite working asyncio and Tokio `spawn_blocking` as an imagined abort mechanism are
not selected. No new scheduler, per-command receipt ledger or global retry policy.

G01/G02 compose prepare→execute→handle→watch/read using independent native expectations; H02/H03 own
assembled and scope-end checks. Publish actual readiness/progress/release semantics in existing
capability/environment/diagnostic guides and derived reference. **Interface-checked** foundations and
**Proposed** delivery remain distinct; no new product or actual-host test was run to author this plan.
