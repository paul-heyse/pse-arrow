---
title: "Plan69: Intent-first CLI and Codex workflows"
status: planned
date: 2026-11-07
adrs: []
doc_role: plan
doc_owner: docs/plans/69-intent-first-cli-and-codex-workflows.md
doc_topics: [current-work, design, libraries, storage, validation]
review_sources: [docs/functional_reviews/intent-first-cli-and-codex-workflows-2026-11-07.md]
---

# Intent-first CLI and Codex workflows

## Target and baseline

**Proposed.** Make `pse` the front door for routine development and product operations. A
caller supplies the intent and any inputs the performing owner cannot safely infer. The owner
resolves context, admits the effect, and returns an inspectable outcome and reference. Ordinary
execution and retrieval require no destination path. Deliverable exports can name a destination.
Recurring source changes can become typed operations where the declarations and their consumers
support a complete preview. Novel code and scientific judgments remain authored directly.

The input review inspected `9e301a03d`. Plan authoring checked its conclusions against main at
`3419aca2a`, including the parser, Nextest configuration, project operator, host launchers,
retention predicate, and canonical schema maintenance. The reported 36 package roots establish
intent-stage selection only. The 48-hour Codex sample supplies workflow signals and reconstructed
sequences. It establishes neither execution frequency nor failure rates. Preserve the review's
aggregates and scope. Use fresh, redacted observations only where a choice needs them.
[Source review](../functional_reviews/intent-first-cli-and-codex-workflows-2026-11-07.md)

Route the observed generation and manager refusals to 69b preparation and placement. Route
argument rejection and misleading guidance to 69a discovery. Route ad hoc JSON extraction and
result-shape errors to 69c typed inspection, and caller-staged paths to 69b placement and 69d
typed dispatch. The socket failure informs the early pathname check. Compiler and native
prerequisite signals inform bounded recovery. Use Plan67’s repaired worktree registration as
the baseline when evaluating any remaining checkout friction.

The focused foundation assessment uses AP-01 for authority, H8 and H24 for composing effects
and retry, H16 and H27 for data lifetime, and PS-10, PS-12 and PS-13 for verdict, scientific
meaning and shared model checks. The parser, envelope, capture owner, product operator and
schema-maintenance boundaries can absorb the proposed workflows. The missing pieces are shared
discovery and selection, clearer preparation and recovery, and compact native-owner result views.
Assess affected fixtures and evidence under the test-assessment companion when specifying tests.
[Selected design standard](../design_review/design_principles/standard.toml)

## Shared decisions and boundaries

- Classify each action as read, prepare, apply or recover. Passive discovery and inspection
  do not prepare tools, start services or admit writes. Preparation is explicit. Give routine
  effects a single supported entry point. Keep configuration preferences, selected identity,
  and authority to apply an effect separate.
- Reuse the native wire envelope. Preserve original operation identity, verdict, settlement,
  and authoritative references across command and process boundaries. Missing prerequisites,
  uncertainty, and a completed refusal each need their own disposition. Only the effect owner
  can settle or release its resources.
- Plan66 remains the scientific target and qualification owner. Its developer and product
  packages own their native capabilities. This series owns their CLI composition and receiving
  journeys. Plan67 supplies invocation custody, fixtures, cache reuse and retirement. Plan68
  owns version-specific SurrealDB transport, migration, schema admission and receiving tests.
  Integrate against each available owner contract and wait only for the specific unmet dependency.
  [Plan66](66-integrated-simulation-target.md) · [Plan67](67-storage-lifecycle-and-build-reuse.md) ·
  [Plan68](68-surrealdb-v4-cutover.md)

The proposal schedules no binding rule change as accepted. In particular, automatic expiry of
the compact developer class needs a separate decision before activation. Until then the plan
uses the existing explicit owner release route, including for expiry-eligible records. Published
generations and original evidence retain their existing release rules. A combined worktree
mutation remains deferred under R-56. If execution exposes a needed binding change, present its
consequence for explicit acceptance or rejection and record the decision and ADR or design route
before dependent work. [Plan67 retirement](67e-resource-retirement-and-storage-verification.md#E03) ·
[Decision register](../adr/register.md)

## Companion plans and dependencies

| Owner | Scope and handoff |
|---|---|
| [69a](69a-cli-contracts-configuration-and-presentation.md) | Establish catalog, context and structured recovery first. Integrate progress as each performing owner supplies real events. |
| [69b](69b-codex-workstation-and-preparation.md) | Inspect and settle placement and bootstrap design alongside 69a. Use 69a’s action and recovery contract for live handoffs. The missing-host launcher remains a separate bootstrap boundary. |
| [69c](69c-ordinary-testing-and-managed-results.md) | Reconcile ordinary selection against existing fixture and validation contracts. Deliver test and result entry points using 69a’s catalog and recovery. Exercise cold end-to-end use once 69b’s handoff works. |
| [69d](69d-routine-product-and-data-operations.md) | Receive existing typed product operations through 69a’s context and 69b’s prepared route. Add a consumer only when its exact typed operation and acknowledgement are available. |
| [69e](69e-source-model-and-live-schema-operations.md) | Investigate source changes independently. Apply shared preview/recovery rules and 69d’s target-selection contract to live maintenance. Receive exact version-specific changes through Plan68. |

Land each producer, affected callers and removed path together. A shared design can unblock
consumer authoring before the working contract is ready for integration. Avoid repeating raw
logs, full exports, broad environment staging or test campaigns in the handoffs.

## Opportunity dispositions

| Review reference | Disposition | Owning intervention |
|---|---|---|
| [O01](../functional_reviews/intent-first-cli-and-codex-workflows-2026-11-07.md#O01) | Scheduled | 69a A01. Parser-owned front door and discovery. Conditional completion and deferred construction have explicit triggers. |
| [O02](../functional_reviews/intent-first-cli-and-codex-workflows-2026-11-07.md#O02) | Scheduled | 69a A02. Resolve defaults once with identity, provenance and effect controls. |
| [O03](../functional_reviews/intent-first-cli-and-codex-workflows-2026-11-07.md#O03) | Scheduled | 69c C01–C02. Bare ordinary tests, population evidence, explicit specialist lanes and original failed rerun. |
| [O04](../functional_reviews/intent-first-cli-and-codex-workflows-2026-11-07.md#O04) | Scheduled | 69c C03–C04. Identity-based bounded results, owner release and expiry decision. Durable cross-run history remains conditional. |
| [O05](../functional_reviews/intent-first-cli-and-codex-workflows-2026-11-07.md#O05) | Scheduled | 69d D01–D03. Typed product routes with original effect identity and supported data operations. |
| [O06](../functional_reviews/intent-first-cli-and-codex-workflows-2026-11-07.md#O06) | Scheduled | 69e E01–E03. Two source cases and guarded live schema maintenance. |
| [O07](../functional_reviews/intent-first-cli-and-codex-workflows-2026-11-07.md#O07) | Scheduled | 69b B01–B03. Workstation launch, readiness, placement and recurring operational sequences. |
| [O08](../functional_reviews/intent-first-cli-and-codex-workflows-2026-11-07.md#O08) | Scheduled | 69a A03–A04. Typed diagnostics, redacted recovery and owner-driven progress. |

## Verification and completion

**Proposed.** Run focused parser, owner and terminal checks with each package. At scope end,
select one assembled journey per row and run the applicable repository qualification. Report
command, mode, population, result and zero-failure baseline. Existing scientific verdicts
belong to their owning plan. Use comparable cold and prepared conditions or retained-byte
observations when making a time or storage claim.

| Journey | Completion evidence |
|---|---|
| Bare ordinary tests and exact rerun | Complete ordinary population including integration tests, explicit nonpass for a missing ordinary prerequisite, and a separate original rerun verdict for an exact current identity. |
| Retrieve and expire a result | Returned ID opens a compact result and selected detail. A hold blocks release. The owner releases a settled disposable record under the enabled policy, and a known retired reference differs from an unknown one within the witness lifetime. |
| Ambiguous product target | Typed choices appear before mutation. A uniquely selected authenticated target returns its original operation reference. |
| Source model change | A supported field change produces a coherent preview, generated consumers and affected checks. Stale inputs refuse before edits and partial publication has an exact recovery path. |
| Stale schema preview | Changed predecessor, target or module refuses activation. The same operation reference can reconcile interrupted application without resubmission. |
| Cold Codex checkout | Passive help stays available. An explicit preparation action establishes the supported scope and only the needed prerequisites. Continuation reuses the selected operation and a warm repeat reuses verified preparation. |
