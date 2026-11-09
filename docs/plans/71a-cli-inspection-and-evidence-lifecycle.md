---
title: "Plan71a: CLI inspection and evidence lifecycle"
status: draft
date: 2026-11-09
adrs: []
doc_role: plan
doc_owner: docs/plans/71-workflow-and-simulation-test-feedback.md
doc_topics: [current-work, diagnostics, validation, storage]
review_sources: [docs/functional_reviews/workflow-simulation-test-execution-and-diagnostics-2026-11-09.md]
---

# CLI inspection and evidence lifecycle

The [coordinator](71-workflow-and-simulation-test-feedback.md) owns O01 disposition and
assembled acceptance. This companion owns the common inspection and release contract.
The host already has an invocation directory and terminal result, activity and checkout
queue observations, bounded log reads, and compact result export and expiry. The runtime
has [bounded checkpoints and a prepared passive reader](../dev/simulation-diagnostics.md#standalone-reads-and-the-cli).
The [current compact result owner](../../crates/pse-host/src/invocation/compact.rs)
uses a 64 KiB envelope, seven-day expiry and guarded release; neither that envelope
nor its timer owns arbitrary linked reports. The new design makes those boundaries visible
and adds lifecycle coverage where the new workflow depends on it.

## Packages

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| [A01](#a01) | Planned | Existing invocation and observation owners | Exact, bounded CLI references and evidence-class contracts work in first producer and reader. |
| [A02](#a02) | Planned | A01 reference contract | Host phases and owner joins are readable live and after completion. |
| [A03](#a03) | Planned | A01; each receiving producer as introduced | Each managed class has a working limit and release path; supported commands and retention are documented. |

<a id="a01"></a>
## A01 — Use the original owner as the retrieval boundary

Bind views to the original invocation UUID and, when applicable, to its original
compiler context, Nextest run, binary/test/attempt, physical fixture owner or scientific
operation/case. Authenticate the relation to the producer before joining. A separate
Nextest list or later process instance is not the original run. An observer response
has its own success and coverage; it cannot supply missing terminal, cancel or drain
facts. Views preserve observed, incomplete, stale, expired, not captured and unavailable
states. An unknown identity after a retirement witness expires remains unknown.

Use one consistent CLI route for each information class. Exact operands below are
target interfaces; unchanged selectors retain their existing behavior.

| User question | Route and owner |
| --- | --- |
| What is my run doing? What stopped it? | `pse inspect invocation ID` shows compact progress and original outcome. `--detail failures`, `provenance`, and `outputs` link to the owner facts; `pse wait ID` waits for the original outcome. |
| What was selected and why, and what actually ran? | `pse inspect test-selection --admitted` explains planned selection; `pse inspect tests ID` reads original population and bounded attempt/compare views owned by [71b](71b-nextest-execution-and-original-attempt-evidence.md#b02). |
| What did the original producer print or retain? | `pse inspect log ID` reads byte ranges; add an exact phase/stream selector for private phase captures. `inspect invocation ID --detail outputs` lists available attachments, owner, coverage, expiry and release eligibility. An explicit artifact retrieval/export reads the selected original under its owner, with a fresh caller destination and receipt. |
| What happened inside conformance or a scientific operation? | `inspect invocation ID` shows the managed conformance summary and linked case pages; `pse observe` reads deeper existing product diagnostics from an explicitly prepared reader. A passive read can report unprepared; it cannot build or start a provider, fixture or runtime. |
| What can be kept or released? | Use the existing `pse tools export-result`, `hold-result`, `retire-result` and `sweep-results` family. Extend selection to owner-backed artifact classes and expose the original evidence and eligibility reason; only the exact preview token authorizes a manual release. |

Use existing human and typed output, field selection, record budgets, cursors bound to
the original selection, and original log ranges. Compose over current evidence in
place. Avoid extra copies, background collection from a read, and mandatory deep
parsing on the routine summary path. Human help and the developer guide must be enough
to follow each reference without opening a state directory. Validate one host and one
conformance original, including foreign, changed, missing and expired references.

<a id="a02"></a>
## A02 — Expose progress that the owners actually observe

Project the checkout writer queue, Cargo compile/link when distinguishable, binary
capture and listing, and known predecessor wait between host groups before the run
timer. Nextest supplies native attempt facts; where eligibility is all it supplies,
leave the wait reason unknown. [TargetEffect](../../crates/pse-host/src/fixtures/target_settlement.rs)
already associates run, binary, test and attempt with the fixture owner. Use that join
for setup, effect and drain; connect the case/operation facts from C02 and the science
reader for native admission, preparation, solve and publication. Do not treat a pipe
closing, a timeout, elapsed wall time or lack of samples as settled, blocked or idle.
Preserve original clocks and age; cross-process times without alignment cannot give
exclusive time, and inclusive times can overlap.

Read existing facts first and retain only bounded last useful state and first actionable
failure. Coalesce repetitive updates; preserve unavailable, omitted and partial facts.
Use no separate polling ledger. Qualification distinguishes queue, fixture setup,
publication and drain failure in both live and retained CLI views.

<a id="a03"></a>
## A03 — Enforce bounded ownership from creation through release

These defaults apply to new managed data and existing outputs explicitly enrolled by
their owner. They do not classify or purge historical directories.

| Artifact class and owner | Collection and default release |
| --- | --- |
| Compact ordinary invocation and retirement witness — host | Reuse 64 KiB compact envelope, seven days after verified settlement, and seven more days for the minimal retirement witness. Keep identity, selection and useful failure refs; a link is not a hold on bulk data. |
| Private test inventory, report/JUnit and required phase capture — original workflow owner | Preserve the existing required capture bounds and exact original result contract. Enroll owned settled results for release after seven days. Resolve consumers before release; keep the compact outcome readable if detail is released early. Never change pass or exact rerun semantics to fit a storage cap. |
| Compact conformance observations and findings — campaign/runtime owner | At most 4 MiB per managed run for new diagnostic state, with bounded rows, concise diagnostics and explicit omissions; release seven days after settlement. The diagnostic budget cannot replace required scientific coverage or the owner report. |
| Optional runner recording, trace, native profile or selected diagnostic payload — initiating diagnostic owner | Off by default; share a 32 MiB allocation per run and 512 MiB per checkout for these new managed attachments, including staging. Cap capture before collecting; a tool without a usable bound is not admitted as a managed attachment. Release 24 hours after settlement unless held. |
| Scratch and spill — producing owner | Reserve against the existing selected workload budget. Dispose of private regenerable scratch after settlement and transfer of selected evidence. Preserve and surface unresolved scratch for recovery. |
| Prepared helpers and images, Cargo, and sccache — their existing owners | Retire exact unreferenced captured images through leases and owner policy; no result TTL on mutable target state, shared cache or reusable inputs. |
| Explicit external exports and protected scientific results — receiving caller or scientific owner | Publish only by request to a fresh destination or established protected product route; return receipt and identify custody. Managed sweeps never remove caller files or protected payloads. |

Reuse the existing [summary expiry and release routes](../dev/agent-environment.md#reading-and-expiring-ordinary-test-summaries)
and fixed authenticated sweep/timer, including explicit activation. Add owner-backed
discovery and release for enrolled classes rather than sweeping filesystem roots.
Time starts after verified settlement, not producer start or last read. Refuse deletion
when identity changed, evidence is under active use or hold, ownership is foreign, or
settlement is unresolved. A named hold carries a date or resolvable condition; conditions
without an owner witness require explicit clear. Expose held or unresolved allocated
bytes, release blockers and the appropriate recovery path through the CLI. Do not
refresh retention on reads. Bound the newly enrolled routine result store to 4 GiB
per checkout, including outstanding capture reservations, separately from optional
attachments and the existing build and product stores. At admission, release only
already eligible bytes under owner rules; if required capture still cannot be reserved,
report a capacity refusal and its release or recovery route before performing the
affected run. A full optional quota refuses only optional capture. Surface inactive
cleanup through the CLI; do not silently shorten retention or delete protected data.
Release and remove replaced retention paths with
their migrated producer, never as a second cleanup schedule.

Use controlled clocks, small budgets and real files to demonstrate truncation,
expiry and byte release, hold/reader refusal, replaced paths, unsettled outcomes and
unavailable detail after release. The tests must show both the original verdict and
the release outcome; a pruned index entry alone is insufficient. Document the setup,
diagnostic retrieval and routine closeout in the existing operator guide.
