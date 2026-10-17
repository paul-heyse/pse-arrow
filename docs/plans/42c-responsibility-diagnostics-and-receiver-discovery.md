---
title: Responsibility diagnostics and receiver discovery
status: done
date: 2026-10-16
adrs: []
review_sources:
  - docs/design_review/reviews/design_review_post-plan41-simulation-efficiency-and-diagnostics_2026-10-16.md
scenario_sources:
  - docs/design_review/reviews/design_review_post-plan41-simulation-efficiency-and-diagnostics_2026-10-16.md
doc_owner: docs/plans/42-post-plan41-simulation-efficiency-and-diagnostics.md
doc_topics: [design, validation, libraries, environment]
---

# Plan 42c: Responsibility diagnostics and receiver discovery

## Purpose and consumed foundations

This companion supplies F06/F07 and inquiry I03 to [Plan42](42-post-plan41-simulation-efficiency-and-diagnostics.md).
It preserves useful scientific-responsibility explanation through bounded retention and export,
and associates managed receivers early enough to observe short runs and their last published
prefix after exit. It does not create scientific authority, a workflow scheduler, new numerical
counters invented by the observer, or a mandatory deep profiler. The coordinator owns finding
dispositions and accepted RC01/RC02; this document owns C-package progress.

**Interface-checked — 2026-10-16:** the [diagnostic inquiry](../design_review/evidence/post-plan41-simulation-efficiency-and-diagnostics-2026-10-16/diagnostic-lifetimes.md)
and current `pse-diagnostics/src/observation.rs`, runtime `observation.rs`, worker readiness and
`scripts/case_audit.py::inspect_managed` establish the boundaries below. The coordinator records
the exact baseline. **Proposed:** all future contracts, corrections and verification; no new
diagnostic capture, service or scientific run occurred while authoring.

Fit and shooting already propagate operation context and author trial/cache/upgrade/experiment/
window facts. The hub's weak relationships and completed-node eviction lose some owners before
export; the bounded publisher independently omits other available owners. Both failures need
correction. Tracing context and exporter replacement alone cannot recover discarded facts.

The collector already retains its discovery cursor, rotates reads and traverses terminal related
owners. These repairs retain their focused evidence, but no subsequent managed scientific replay
established assembled coverage. The remaining problem is exact early handoff and post-exit lookup,
not a reason to revert pagination or to claim historical coverage.

## Producer-owned responsibility summaries

### C1 — Settle the bounded summary contract

I03 specifies a generic producer-declared summary purpose/eligibility and publication lifetime
at the low diagnostic boundary. The composition that understands a fit, shooting job, study or
other responsibility selects its compact facts. The hub must not parse `fit.*` names, import
workflow types or maintain a central workflow-name registry. Another scientific workflow obtains
the same behavior by supplying its owned facts and summary declaration.

Separate execution completion from summary publication lifetime. A completed operation remains
completed. Diagnostic retention carries copied facts/relationships only, not a native handle,
scientific payload, cancellation lease or permission. The target must survive repetitive inner
attempts without retaining every attempt or rebuilding totals from overlapping reports.

Define admission before relying on retention: bounded bytes/counts, selection when several
responsibilities compete, active-work preservation, compact content, refusal/loss and exact
release. Work within the existing 16 MiB default hub and 1 MiB process image rather than raise
them as the remedy. Counters/gauges, units, resets, freshness and unavailable detail retain their
producer semantics. A refused/unsupported summary remains explicitly partial, never zero-filled.

Terminal consumption means compact final facts have transferred to a declared bounded observation
output/reader/publication owner; it cannot mean an unknown external observer has acknowledged them.
Settle the concrete handoff, finite fallback when publication is absent/fails, and release/eviction
policy before dependent implementation. Scientific return must not wait indefinitely for an
observer. Delayed or absent publication must report loss/staleness, not invent a final checkpoint.
This bounded choice is required inquiry completion, not an indefinitely deferred policy decision.

The contract must answer: what survives execution completion; which owner charges it; how a local
reader and process exporter see the same meaning; what happens under pressure, opt-out and writer
failure; and when the charge ends. Install RC01's governing amendment through G0 before relying
on its changed retention/export guarantees. API names and local data layout remain implementation
choices within this settled meaning.

### C2 — Retain compact facts without retaining scientific execution

Implement the C1 policy in the hub and producer compositions. Preserve selected summaries while
ordinary completed attempts remain evictable. A strong external diagnostic reader can keep copied
data charged, but does not by itself keep an owner discoverable in the catalogue; implement the
actual retained-summary route, not just an extra reader handle.

Fit/shooting producers supply their existing authored cumulative facts and bounded representative
detail. Migrate other current responsibilities that require the same generic contract, identified
by their actual composition rather than strings. Preserve current active operations, process-local
IDs and relationships. Refusal cannot block or alter numerical requests. Native callbacks keep
nonblocking owner-safe publication: no formatting, storage I/O or unsynchronized foreign getters.

### C3 — Export summaries before expendable completed detail

Make compact retained summaries available to independent readers within the existing bounded
checkpoint. Keep a useful active-work view and give selected summary content a credible route
before repetitive completed attempts/detail exhaust the image. Separate essential identity/
coverage/cumulative content from optional history/exemplars. State omissions when simultaneous
demand exceeds the bounds; priority cannot promise unlimited complete trees.

Version and regenerate affected Rust/Python/CLI observation contracts at their declaration owner
when their wire shape changes; migrate writers/readers together without a second production
format path. Atomic file publication remains separate from the non-atomic hub/list snapshot cut.
Retained summary availability and export inclusion receive distinct controls. Never infer elapsed
CPU shares or iteration counts from operation sequence IDs, or sum inclusive parent/child/final/
strategy reports into a synthetic total.

## Exact receiver handoff and terminal inspection

### C4 — Publish the actual receiver's diagnostic locator

The current readiness writer is `xtask/src/bin/pse-worker.rs`; supervision consumes its
`primary-receiver.json`. Extend that existing handoff, under accepted RC02/G0, with the actual
runtime diagnostic checkpoint/process locator associated with the registered database/receiver.
Consume the runtime's existing process identity and checkpoint owner rather than independently
reconstructing a path from PID or scanning catalogue history.

The locator must bind the actual process instance/start identity, checkpoint and registered
receiver association. Its meaning is observation, not build/environment proof. Wrong database,
replaced receiver or PID reuse cannot match. Readiness publication remains atomic under its
existing writer/registration lifetime. Examine supervision, measurement, managed Rust/Python
fixtures and audit consumers together so startup and later readers agree about the handoff.

Observation opt-out, unavailable locator or optional publisher failure is explicit diagnostic
partiality; it does not make a scientifically ready receiver unready or grant admission bypass.
Keep existing nonce/service identity and stop safeguards. The diagnostic locator grants neither
service-stop nor allocation-release authority. Do not turn this additive observation into a
second supervisor or a requirement for a global collector.

### C5 — Read the validated last prefix after exit

Use the ready locator directly for early association; preserve bounded catalogue discovery for
genuinely unassociated observations with explicit incomplete coverage. Validate embedded checkpoint
identity against the exact handoff/cache. Once association is valid, cached terminal inspection
must not require another successful `/proc` lookup before reading the checkpoint.

Distinguish live operational attachment from retained file inspection. A stale/missing/mismatched
checkpoint is partial/unavailable; a valid file after drain is the last published prefix, not
proof of final science, final callback delivery or native join. Never rebind a retained path to
a replacement receiver. Copy only bounded validated data; observer stop/drop cannot cancel
science or release someone else's resources.

Migrate `case_audit.py` and affected direct inspection/discovery consumers. Preserve cursor/read
rotation and terminal traversal controls. Reuse existing runtime locator validation where it fits,
keeping caller-local and externally executed native facts distinct. Do not lengthen scientific
runs merely to make eventual discovery succeed.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| <a id="c1"></a>C1 | Existing producer facts, hub/export budgets and accepted RC01 | Settle I03's role, bounded admission, output handoff and finite release; expose refusal/failure/opt-out behavior and supply G0's concrete amendment. | Completed — targeted and selected assembled acceptance passed; shared scope-end checks accepted at coordinator Outcome |
| <a id="c2"></a>C2 | C1 contract and G0 RC01 amendment | Generic retained summaries and migrated producer compositions; pure pressure/refusal/completion/release controls show no scientific ownership or false activity. | Completed — targeted and selected assembled acceptance passed; shared scope-end checks accepted at coordinator Outcome |
| <a id="c3"></a>C3 | C2 actual retained facts and current publisher | Bounded compact export priority and affected generated consumers; separate omission/retention controls preserve active work, cumulative meaning and loss. | Completed — targeted and selected assembled acceptance passed; shared scope-end checks accepted at coordinator Outcome |
| <a id="c4"></a>C4 | Actual runtime locator/readiness source; G0 RC02 amendment before contract wiring | Exact ready handoff through writer/supervision/managed consumers; identity, opt-out and optional-writer controls. | Completed — targeted and selected assembled acceptance passed; shared scope-end checks accepted at coordinator Outcome |
| <a id="c5"></a>C5 | Existing prefix semantics; C4 implemented for direct managed association | Validated cached terminal reads without live `/proc`, retained bounded discovery/rotation and actual managed association; synthetic controls plus Q1 real journey. | Completed — targeted and selected assembled acceptance passed; shared scope-end checks accepted at coordinator Outcome |

Contract investigation can proceed before G0 installation. Cached terminal handling already
permitted by current semantics need not wait for all of C4; its assembled short-run association
claim does require the implemented handoff. C2 retention precedes C3 export acceptance; another
exporter cannot bypass that prerequisite. Shared declarations/publication/worker files require
coordinated integration with 42b and generated Python consumers.

## Verification and documentation

**Proposed:** pure low-dependency hub controls force many inner attempts and competing summary
owners through existing finite limits. Test completed summary discovery, bounded admission/refusal,
active-work preservation, terminal publication transfer, finite release, writer failure and
opt-out. Test export separately with an available owner that would formerly be omitted. Assert
explicit loss/reset/freshness and cumulative versus latest/gauge meaning, not just JSON presence.

Synthetic collector/readiness controls cover short receivers, a large historical catalogue,
cached read after process disappearance, PID reuse, wrong database/start/checkpoint, optional
observation and missing file. A locator is not cleanup authority. Real selected fit/shooting and
managed numerical/read journeys belong to Q1; they establish assembled behavior without requiring
a full all-context campaign. Use another caller while work is busy, then inspect the retained
prefix after actual registered drain. Existing unit passes do not retroactively qualify it.

Update blueprint §23.1 through G0 and the diagnostics/operational guides with delivered role,
retention/refusal/release, export priority, exact locator and last-prefix semantics. Preserve
current/recent navigation, useful optional profiles, scientific permission and no-extra-science
inspection. Plan40e retains E0–E6 acceptance: this companion supplies focused corrections and
shared journey evidence, not completion of Plan40's broader profile/locality scope.

## Current checkpoint

C1–C5 source changes are integrated. Producers declare generic compact responsibility summaries;
fit, shooting and study compositions retain current/cumulative facts without native/scientific
ownership. Default terminal eligibility is 60 seconds, independently of recent history. Existing
hub byte/count bounds govern admission; active identity/coverage precedes terminal summaries,
then optional history and ordinary completed detail in the 1 MiB export. Refusal, expiry and
pressure loss remain explicit. Pruning occurs on observations/admission and ordinary publication;
idle bounded diagnostic bytes remain charged until interaction/drop. Copied readers become stale
at eligibility expiry. No autonomous reaper or physical expiry-latency promise is selected.

Checkpoint version 3, typed Rust/Python documents and native read/inspect interfaces replace the
old shape together. Exact expected identity is checked against the decoded image subsequently
inspected and copied. The actual readiness consumer publishes the receiver state/database,
registration nonce, process/start identity and checkpoint through a bounded generic diagnostic
responsibility. Separate copied selection fields permit unrelated incomplete handoffs to be
skipped; unknown or matching incomplete selection remains unavailable. Optional supervisor copy
requires the same marker across existing dispatch admission, matching launch nonce and actual PID
start, without repeating environment/generation qualification. Pending publication, opt-out and
writer failure remain independent of scientific readiness.

New association ages the selected ready summary using the same image; expired or clock-unavailable
summary eligibility refuses association. Once cached, exact terminal-prefix reading needs neither
another live process lookup nor a replacement registration. The full selected script modules pass 87 controls with zero failures on the integrated checkout;
the pure diagnostic owner and related runtime controls also pass. Generation and installation of
the native correctness extension completed with successful enclosing cleanup after inotify was
restored. Twelve installed Python diagnostic tests passed; the new exact-prefix fixture was
corrected to use the generated immutable tuple shape and its affected rerun passed. The assembled fit/shooting/managed numerical and cleanup journeys passed with original
assertions, including the managed observer's unchanged 256 MiB pool. Fit retained 47 trial demands,
19 new trials and 33 experiment attempts; shooting retained 28 demands, seven new trials and
12 integrations, with current/cumulative and partiality fields. Managed discovery selected the
actual caller readiness responsibility rather than a later marker scan. After actual receiver drain/process exit, the installed typed
reader validated its exact retained checkpoint identity and copied 36 operations as a last
published prefix; no final-science claim follows. Coordinator Q2 and final reconciliation are complete. Independent review
corrected retained optional-vector capacity, missing curtailment after phase omission, local versus
passive expiry, same-database receiver selection and readiness-copy association.

## Outcome (recorded after implementation)

### What was built

**Implemented:** C1–C5 add generic producer-declared responsibility summaries with the accepted
60-second terminal eligibility, finite independent retention and compact export priority under
existing registry/image bounds. Active identity/coverage precedes terminal summaries, then optional
history; refusal, pressure loss and expiry remain explicit. Same-image typed checkpoint-v3 validation,
inspection and copy replace the old shape. Actual consumed readiness supplies receiver state/database,
nonce, process/start identity and checkpoint through bounded diagnostic responsibility. Exact cached
association permits last-prefix reads after process exit without adopting a replacement receiver.
Observation neither keeps science active nor grants cleanup or scientific authority.

**Tested — 2026-10-16, zero-failure baseline:** pure hub retention/export/expiry/refusal and runtime
readiness/identity controls passed. `python -m unittest scripts.tests.test_case_audit
scripts.tests.test_primary_observation scripts.tests.test_pse_env` passed 87 controls after integration;
the installed native-correctness `pytest python/pse/tests/test_diagnostics.py -m unit -q` passed all
13 controls. Selected `just case-measure ... --smoke --audit-diagnostics` fit, shooting and managed
journeys passed original numerical and cleanup assertions. Fit retained 47 trial demands, 19 new
trials and 33 experiment attempts; shooting retained 28 demands, seven new trials and 12 integrations.
These facts retain their actual cumulative/current/partial meanings. The original 256 MiB managed
observer used the actual caller readiness responsibility. After receiver drain and process exit,
the typed reader validated the retained exact identity and copied 36 operations as a last published
prefix; that copy alone establishes neither final science nor final callbacks. The coordinator Outcome records
the completed shared scope-end result.

**Interface-checked:** independent review found no missing producer/consumer transfer. I03 is
settled by bounded admission, publication and semantic expiry: pruning is interaction/publication
based, with no autonomous reaper or promised physical expiry latency. Governing amendments precede
implementation at blueprint §23.1/revision 179; the diagnostics guide describes retention, loss,
readiness and last-prefix semantics. Plan40e retains its broader profile/locality campaign.

### A mistake made and corrected

The exact-prefix Python fixture initially used a mutable list where the generated document requires
an immutable tuple; its affected rerun and then the complete 13-test module passed. Independent
review also corrected optional vector capacity release, curtailment after omitted phases, passive
expiry interpretation and same-database/readiness-copy association. No omitted historical metric
was reconstructed or treated as zero.

### Deviations from the plan, deliberate

Semantic expiry is bounded and truthful without a background diagnostic reaper. Exact readiness
association avoids catalogue-history dependence while bounded cursor/owner rotation remains as an
explicitly partial fallback. Scientific runs were not lengthened for observation, and the existing
256 MiB observer, 1 MiB image cap and scientific permission contracts were preserved.
