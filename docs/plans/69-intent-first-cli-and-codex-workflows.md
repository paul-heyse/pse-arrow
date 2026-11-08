---
title: "Plan69: Intent-first CLI and Codex workflows"
status: in-progress
date: 2026-11-07
adrs: []
doc_role: plan
doc_owner: docs/plans/69-intent-first-cli-and-codex-workflows.md
doc_topics: [current-work, design, libraries, storage, validation]
review_sources:
  - docs/functional_reviews/intent-first-cli-and-codex-workflows-2026-11-07.md
  - docs/plan_improvement_reviews/plan69-cli-library-leverage-2026-11-07.md
  - docs/plan_improvement_reviews/plan69-surrealdb-and-rust-tooling-2026-11-08.md
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

The library review inspected `ed8e3fce2`. The plan revision rechecked the owner
seams against `2fdaf1ab6`. **Interface-checked** baseline evidence includes the
eager format-prefix tree build and the existing typed catalog extensions.
**Tested** baseline evidence includes progress on PTY stderr during successful
machine-format search and isolated contracts for deferral, file acquisition,
provenance, diagnostic conversion and progress lifetime. The review records
versions, conditions and corrected probe controls. These observations motivate
the scheduled packages. The target acceptance below remains **Proposed**.
[CLI library review](../plan_improvement_reviews/plan69-cli-library-leverage-2026-11-07.md) ·
[Baseline evidence](../plan_improvement_reviews/plan69-cli-library-leverage-2026-11-07.md#evidence-and-limits)

The [SurrealDB and Rust tooling review](../plan_improvement_reviews/plan69-surrealdb-and-rust-tooling-2026-11-08.md)
inspected `e7bcacac8`. Plan authoring rechecked the owners at `622261d8f`, which includes
Plan 68's completed v4 cutover and its remaining separate qualification and custody.
**Interface-checked.** The current host prepares SurrealKit 1.0.0-rc.1 and a separate
v4 Study extension. The maintenance consumer verifies exact settlement and activation
but lacks its production issuer and settler. The source preview currently uses textual
anchors, and the SurrealQL parser already returns structured spans the receiver flattens.
**Tested in the review.** Scoped `pse-host --lib` admission reported a Cargo target
and unavailable exact test inventory without building or listing tests. Use this
evidence for admission only. The remaining live product and rollout journeys still
need their own evidence.
[Plan 68 current status](68-surrealdb-v4-cutover.md#current-status) ·
[Review evidence](../plan_improvement_reviews/plan69-surrealdb-and-rust-tooling-2026-11-08.md#ordering-and-evidence-limits)

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

The maintainer selected automatic expiry for newly classified, settled and regenerable
compact developer results on November 7. C04 owns the bounded release and timer activation.
Explicit owner preview and release remain available. Published generations and original
evidence retain their existing release rules. A combined worktree
mutation remains deferred under R-56. If execution exposes a needed binding change, present its
consequence for explicit acceptance or rejection and record the decision and ADR or design route
before dependent work. [Plan67 retirement](67e-resource-retirement-and-storage-verification.md#E03) ·
[Decision register](../adr/register.md)

## Companion plans and dependencies

| Owner | Scope and handoff |
|---|---|
| [69a](69a-cli-contracts-configuration-and-presentation.md) | Own catalog metadata and typed syntax, resolved context and provenance, and the original diagnostic projection. |
| [69b](69b-codex-workstation-and-preparation.md) | Own explicit prepared v4 Study continuation and socket refusal/drain receiving. Supply exact SurrealKit recovery if E03 selects the standalone tool. Use 69a’s recovery projection. |
| [69c](69c-ordinary-testing-and-managed-results.md) | Own test and result entry points. C05 explains source candidates, Cargo admission, Nextest and rustdoc lanes through 69a’s catalog. Existing execution and custody evidence retain their owners. |
| [69d](69d-routine-product-and-data-operations.md) | Receive live typed product operations through 69a’s context and 69b’s prepared Study route. D04 investigates a bounded diagnostic consumer of Plan 68c's existing observations. |
| [69e](69e-source-model-and-live-schema-operations.md) | Own exact source evidence and guarded publication, local structural-query reuse, and native rollout issue, settlement and activation. E03 uses 69d target selection, 69a recovery, and Plan 68's exact v4 contracts. |
| [69f](69f-parser-startup-and-argument-forwarding.md) | Parse strictly before recovering usage format and preserve OS-native argv. Compare selective construction only after F01 and A01's discovery contracts are ready. |
| [69g](69g-output-policy-and-owner-driven-progress.md) | Own A04. Fix machine-format search output first, then apply one presentation policy as owner events and A03 projections become available. |

Land each producer, affected callers and removed path together. A shared design can unblock
consumer authoring before the working contract is ready for integration. Avoid repeating raw
logs, full exports, broad environment staging or test campaigns in the handoffs.

Start A04's machine-output fix and A03's typed projection early. A04's first
deliverable can use the existing format and renderer. F01 uses the current usage
envelope and can proceed alongside A01. F02 depends on both F01 and the scoped
and full A01 discovery contracts. In A02, resolve acquisition before specifying
attribution. B–E receive those shared contracts as needed. Additional progress
presentation uses each receiver's existing owner events. Conditional experiments
do not gate receiving work that already has the required contract.

Use B02's v4 Study preparation to unblock D01–D03 and B03 without waiting for E03.
E03's passive preview, E02 source evidence, C05 and E04 can advance on their own
available prerequisites. Before exposing E03 apply, settle the executable authority
investigation, receive the matching B02 recovery if applicable, and verify the original
nonce issuer and settler. D04 requires a bounded adoption or deferral decision.
Its candidate CLI route depends on that decision.

## Execution checkpoint

The CLI now discovers parser-checked examples, resolves private project targets once,
and presents effective settings with provenance. The captured project receiver checks
the expected issuer and exposes bounded research and descriptive closure reads. The
performing route carries original import issue, apply, checkpoint and recovery identities.
The checkout retains at most 64 original import intents and releases an intent after
authoritative project completion. Prepared host receipts include the authenticated
selected generation.

Detached and borrowed test presentations can enter the compact expiry class after the
owner proves native drain and image release. The original settlement bytes stay immutable.
Bare ordinary test admission now has a compile-free Cargo graph check. Exact scoped test
enumeration and execution use nextest. Guarded analyzer candidates can narrow the Cargo
target and test name before a build. Discovery acquires only nextest-selected images
and shares immutable bytes across active invocations. [Analyzer and capture boundaries](69c-ordinary-testing-and-managed-results.md#analyzer-and-capture-boundaries)
Source-change preview uses an isolated checkout with one retained snapshot per checkout.
It accepts reviewed source postimage digests, publishes the staged generator delta and
recovers by exact file identity. The
installed-catalog observer uses an authenticated Viewer connection and shares the
normalized inventory with maintenance validation.

The companion plans retain assembled qualification that remains open. Project packs and
study deliverables share an issuer-bound publication identity and durable original custody.
The live v4 project receiver verified configured issuer selection and original acknowledgement
readback. The exchange journey passed protected pack reopen and bounded Study publication
after correcting v4 protected-read result binding.
The owned v4 probe verified SurrealKit phase execution and exposed an unresolved journal
step after a committed SQL effect. The in-process native service can bind a passive preview
to its physical listener. The configured-state CLI still needs the persistent service
handoff and the producer of the exact pending and settled nonce before exposing apply.
F02 retains eager construction after selective deferral failed catalog
parity. A04 keeps transfer progress conditional on an owner exposing actual transfer events.

## Opportunity dispositions

| Review reference | Disposition | Owning intervention |
|---|---|---|
| [O01](../functional_reviews/intent-first-cli-and-codex-workflows-2026-11-07.md#O01) | Scheduled | 69a A01 and 69f F01–F02. Parser-owned front door, discovery and exact forwarding. Completion and deferral have explicit triggers. |
| [O02](../functional_reviews/intent-first-cli-and-codex-workflows-2026-11-07.md#O02) | Resolved | 69a A02 verified effective defaults and provenance. The captured v4 receiver preserved the issuer and refused ambiguous or foreign targets before effects. |
| [O03](../functional_reviews/intent-first-cli-and-codex-workflows-2026-11-07.md#O03) | Resolved | 69c C01–C02 verified ordinary admission, scoped Nextest execution and the exact failed-test rerun. C05 verified the separate Cargo, Nextest and rustdoc evidence. |
| [O04](../functional_reviews/intent-first-cli-and-codex-workflows-2026-11-07.md#O04) | Resolved | 69c C03–C04 verified bounded reads of original results and owner release under the selected expiry policy. Durable cross-run history remains conditional. |
| [O05](../functional_reviews/intent-first-cli-and-codex-workflows-2026-11-07.md#O05) | Resolved | 69d D01–D03 verified authenticated selection, original acknowledgement after reply loss, protected import reopen and bounded Study publication. |
| [O06](../functional_reviews/intent-first-cli-and-codex-workflows-2026-11-07.md#O06) | Scheduled | 69e E01–E03. Two source cases and guarded live schema maintenance. |
| [O07](../functional_reviews/intent-first-cli-and-codex-workflows-2026-11-07.md#O07) | Scheduled | 69b B01–B03. Workstation launch, readiness, placement and recurring operational sequences. |
| [O08](../functional_reviews/intent-first-cli-and-codex-workflows-2026-11-07.md#O08) | Scheduled | 69a A03 and 69g A04. Typed diagnostics, redacted recovery and owner-driven progress. |

## Improvement dispositions

| Review reference | Disposition | Owning intervention |
|---|---|---|
| [I01](../plan_improvement_reviews/plan69-cli-library-leverage-2026-11-07.md#I01) | Scheduled | 69f F01 strict parse and usage fallback. F02 evaluates deferral after F01 and A01. Forwarding grammar changes are conditional. |
| [I02](../plan_improvement_reviews/plan69-cli-library-leverage-2026-11-07.md#I02) | Scheduled | 69a A01 extends typed catalog associations, checks the package candidate domain, and evaluates optional completion and help controls on demand. |
| [I03](../plan_improvement_reviews/plan69-cli-library-leverage-2026-11-07.md#I03) | Scheduled | 69a A02 acquires and resolves once. B02, D01 and E03 carry or revalidate the selection across owners. |
| [I04](../plan_improvement_reviews/plan69-cli-library-leverage-2026-11-07.md#I04) | Scheduled | 69a A02 projects effective provenance. C03 exposes trustworthy captured detail. Mixed-origin arrays remain conditional. |
| [I05](../plan_improvement_reviews/plan69-cli-library-leverage-2026-11-07.md#I05) | Scheduled | 69a A03 defines the typed projection and B–E verify their receiving adapters. |
| [I06](../plan_improvement_reviews/plan69-cli-library-leverage-2026-11-07.md#I06) | Resolved | 69g A04 verified the shared terminal policy with JSON and NDJSON over pipes and PTYs and with narrow monochrome human output. Rollout receiving remains an assembled obligation. |
| [I07](../plan_improvement_reviews/plan69-cli-library-leverage-2026-11-07.md#I07) | Deferred pending a publication event producer | 69g A04 owns display lifetime. The captured transfer owner currently publishes a terminal receipt. Revisit counted transfer when an intermediate stage has authoritative byte events. |
| [I08](../plan_improvement_reviews/plan69-cli-library-leverage-2026-11-07.md#I08) | Scheduled | Coordinator owns the assembled journeys below. The producing packages own isolated checks and B–E own their receiving evidence. |

Carry the review appendix as the source of other feature dispositions. Revisit a
deferred capability only on its recorded condition. The integrated recommendations
preserve existing binding decisions. If an investigation reveals a proposed change to
a governed decision, record its consequences and use the existing decision route
before any package relies on it.
[Guide coverage and dispositions](../plan_improvement_reviews/plan69-cli-library-leverage-2026-11-07.md#appendix--guide-coverage-and-dispositions)

## SurrealDB and Rust tooling dispositions

| Review reference | Disposition | Owning intervention |
|---|---|---|
| [I01](../plan_improvement_reviews/plan69-surrealdb-and-rust-tooling-2026-11-08.md#I01) | Scheduled | [69e E03](69e-source-model-and-live-schema-operations.md#e03--inspect-preview-apply-and-recover-live-schema) owns passive preview, the prepared execution decision, and original nonce settlement and activation. |
| [I02](../plan_improvement_reviews/plan69-surrealdb-and-rust-tooling-2026-11-08.md#I02) | Scheduled | [69b B02–B03](69b-codex-workstation-and-preparation.md#exact-v4-prerequisites) prepare and receive the native fixture. [69d D01–D03](69d-routine-product-and-data-operations.md#live-v4-receiving) own live product receiving. [69e E02](69e-source-model-and-live-schema-operations.md#optional-field-receiving) owns optional-field checks and the external fixture candidate. |
| [I03](../plan_improvement_reviews/plan69-surrealdb-and-rust-tooling-2026-11-08.md#I03) | Scheduled | [69e E02](69e-source-model-and-live-schema-operations.md#exact-authored-and-generated-evidence) owns structural spans and the bounded affected view. [69a A03](69a-cli-contracts-configuration-and-presentation.md#a03--original-outcomes-and-safe-recovery) receives original diagnostics. |
| [I04](../plan_improvement_reviews/plan69-surrealdb-and-rust-tooling-2026-11-08.md#I04) | Resolved | [69c C05](69c-ordinary-testing-and-managed-results.md#c05--explain-test-selection-and-doctest-ownership) verified the scoped selection explanation, doctest routing, and the separate fresh Nextest inventory and original verdict. |
| [I05](../plan_improvement_reviews/plan69-surrealdb-and-rust-tooling-2026-11-08.md#I05) | Deferred pending a named recurring consumer | [69d D04](69d-routine-product-and-data-operations.md#d04--decide-whether-typed-graph-diagnosis-belongs-at-the-front-door) completed the comparison and retains the explicit graph diagnostic. A04 transfer display depends on actual owner events. C04 retains the existing expiry policy. |
| [I06](../plan_improvement_reviews/plan69-surrealdb-and-rust-tooling-2026-11-08.md#I06) | Resolved | [69e E04](69e-source-model-and-live-schema-operations.md#e04--reuse-coordinates-in-bounded-structural-browsing) verified reused source coordinates with exact UTF-8 captures and multiline edits. Range pushdown has a separate revisit condition. |
| [I07](../plan_improvement_reviews/plan69-surrealdb-and-rust-tooling-2026-11-08.md#I07) | Resolved | [69b B02](69b-codex-workstation-and-preparation.md#exact-v4-prerequisites) verified exact standalone tool preparation and recovery. E03 still owns admission to live apply. |

The [coverage appendix](../plan_improvement_reviews/plan69-surrealdb-and-rust-tooling-2026-11-08.md#coverage-appendix)
retains the detailed guide dispositions. Revisit optional analyzers, language services,
embedded compilation, and external fixtures at the owning package's trigger. Route
broader query, transport, migration, or backend investigations to Plan 68 and
scientific consumers to Plan 66. Any discovered change to a governed decision follows
the existing decision route before dependent work adopts it.

## Verification and completion

**Proposed.** Run focused parser, owner and terminal checks with each package. At scope end,
select one assembled journey per row and run the applicable repository qualification. Report
command, mode, population, result and zero-failure baseline. Existing scientific verdicts
belong to their owning plan. Use isolated fixtures for parser, configuration, and
terminal controls. Reuse owner evidence when the premise matches, and add a
receiving check where a boundary can lose meaning. Use comparable cold and prepared
conditions or retained-byte observations when making a time or storage claim.

| Journey | Completion evidence |
|---|---|
| Bare ordinary tests and exact rerun | Catalog syntax, advisory source candidates, Cargo admission, Nextest inventory, and verdict are distinct. Preserve workspace root and feature-unit equivalence evidence, and exercise scoped admission, exact inventory and verdict, missing prerequisites and doctest-only routing. A full test-name census requires a separate use case. The exact failed rerun preserves applicable forwarded filters and produces its own original verdict. |
| Retrieve and expire a result | A returned ID opens a compact result and bounded detail with diagnostic relationships and provenance where captured. Holds block release. The owner releases settled disposable records under the enabled policy. Retired and unknown differ within the witness lifetime. |
| Live product target and original effects | Typed choices appear before mutation and foreign issuer refuses. The authenticated target and original acknowledgement survive dispatch and uncertain recovery. Protected import reopen and Study publication use their original identities on the prepared v4 fixture. Any export counter describes only its observed stage. |
| Source model change | Supported change yields unique authored spans, bounded affected reasons, generated consumers, and exact staged diagnostic labels. Stale inputs refuse before edits and partial publication retains exact recovery. Check optional-field source publication, SurrealQL syntax and predecessor-row behavior separately. |
| Passive, stale and interrupted schema rollout | Preview leaves installed state unchanged. Changed predecessor, instance, target or module refuses before writes. A lost reply before step settlement remains distinct from committed activation in both outputs. Reconcile the original nonce and prove a single activation without blind SQL resubmission. |
| Cold Codex checkout | Help and catalog stay passive. Explicit preparation establishes scope and prerequisites. Verify exact v4 Study continuation and warm reuse. Reject invalid socket paths before launch and release scratch after actual drain. Receive exact standalone tool recovery if E03 selects it. |
