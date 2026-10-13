---
title: SurrealDB lifecycle and integration target
status: done
date: 2026-10-12
adrs: [ADR-0181, ADR-0182]
review_sources: [git:e145eb7d229000dd9dda86882fe85c0b301657e2:docs/design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md]
scenario_sources: [git:e145eb7d229000dd9dda86882fe85c0b301657e2:docs/design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#representative-changes-and-failure-journeys]
doc_owner: docs/plans/37-surrealdb-lifecycle-and-integration.md
doc_topics: [current-work, design, storage]
doc_retention: while-dependent
---

# Plan 37 — SurrealDB lifecycle and integration target

## Purpose and reading route

The completed scope implements the target developed from the [2026-10-12 review](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md), including all seven findings, nine rule decisions and consequential investigations. This distinct undertaking is complete. H1 reconciled affected existing contracts and prerequisites after bounded acceptance; their separate stopped campaigns remain open.

The target supports value edit → re-solve without structural rebuilding; sweeps, sensitivities, optimization and fitting; recycles/tears; dynamics from the same definitions; unit/property/solver extension; and compilation/testing with a responsibility's actual dependencies. Persistence preserves scientific meaning and dependencies; numerical authority stays with scientific/library owners.

**Maintainer cutover decision, 2026-10-12:** pivot hard to the target. Existing data and evidence need not survive; they can be regenerated. Build fresh target stores, generated contracts and qualification evidence. Do not build legacy migration, backfill, historical-reader or compatibility paths. Future target data still receives the retention guarantees below. Reset is an explicit authenticated operation on a selected owned store, not authority to remove another owner's resources.

| Document | Responsibility |
|---|---|
| This coordinator | Combined target, rule decisions, finding dispositions, all package status, investigations, qualification and completed existing-plan reconciliation |
| [37a — Scientific artifacts, history and policy](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/plans/37a-scientific-artifacts-history-and-policy.md) | Retained inputs, useful content, history release, replay fences and production study policy; F01–F03/F07 |
| [37b — Connection and transport lifetimes](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/plans/37b-connection-and-transport-lifetimes.md) | Caller release, physical shutdown, acknowledged restoration and exceptional completion; F04–F06 |
| [37c — Tooling and operational integration](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/plans/37c-tooling-and-operational-integration.md) | Beta tooling, generated DDL, fixtures, forward rollouts, clean rebuild and capability investigations |

This coordinator is the sole disposition and package-status owner. The archived companions record the implementation designs without duplicating status. The source review keeps its historical observations; its local SCI/CON labels are not extra findings.

## Baseline and focused foundation assessment

**Implemented, source-inspected:** commit 677394474a543c884a8a21d1a2626a6820ff834e, with the uncommitted review/evidence and user-supplied second white paper preserved. Canonical revisions, preparation, runs, attempts, original assessment, closed outputs, readers and service generations already have distinct owners. Exact publication/creation/reclamation fences, protected analysis inputs, copied immutable bytes and independent operational metadata are useful foundations. Keep them; no universal lifecycle controller is needed.

At that authoring baseline, the review's consequential gaps were: late acquisition of selected external seeds, whole-run retention, permanent detailed history, incomplete shared-close ownership, nonterminal acknowledged restoration, exceptional admission release, and policy compilation coupling. None is established as a measured throughput or capacity deficit.

Focused authoring advice found existing seed admission already defines bit-preserving content, original inventory, assessment and layout/profile/data/backend compatibility; a small self-contained seed artifact can avoid generic shared-chunk machinery. The current pse-model study types are a fitting production home for pure study policy. Before new history deletion, retired study identity must dominate creation even when a different run is supplied; retaining today's marker alone is insufficient.

For beta.6, namespace-auth test mode keeps suite and parity databases in the configured namespace, permitting pre-creation enclosing ownership. Its schema preparer rewrites definitions, so generated-input acceptance does not prove equivalent behavior. Its SQL hashes do not replace the repository's DDL-plus-codec interpretation digest.

**Interface-checked:** release-specific library sources and documentation. **Proposed:** target design and future packages. No new product tests, runtime probes, installations, services, measurements or scientific qualification occurred during authoring. Read the [supporting evidence](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/design_review/evidence/surrealdb-lifecycle-and-integration-2026-10-12/README.md) for original scope and limits. Both white papers remain leads; the stale utilization catalog/capability maps are not evidence.

Architectural owners are blueprint §3, §4, §20 and §21. Criteria remain Core 3.4, its efficiency heuristics and Process Simulator 1.5. Existing rules describe the baseline; confirmed changes below govern the target.

## Combined target and preservation

Scientific owners specify sufficient immutable content for starts, states, trajectories and analyses. Canonical operations admit that content and its dependencies, expose bounded protected reads and release independent obligations. Producer retirement must not destroy a promoted artifact. Provenance does not automatically require retaining every producer occurrence or trajectory.

New reproducible durable studies bind exact external starts before activation. Preserve original bits, identities, assessment and compatibility; distinguish any deliberate acquisition-time request. Owned immutable bytes can outlive producer permission, while fresh DB reads require current artifact authority. History release removes unselected detail only after settlement and dominating identity fences. Unknown effects, live borrowers and deliberate audit obligations remain protective. Minimal replay attribution cannot promise reconstruction of deleted full responses.

Runtime close releases its logical caller. Admitted work owns the connection activities needed through renewal/publication. Whole-connection shutdown is an explicit deployment operation that refuses live dependents or atomically closes admission and physically drains when none remain. Restoration reaches ready or terminal failure under one acknowledged budget. Terminal admission release is independent of metadata garbage collection and distinct from payload drain or DB settlement.

Retain the patched WS production integration and SDK/server 3.3.0. Target SurrealKit 1.0.0-beta.6 through a standalone adapter consuming generated schema, namespace-owned fixtures and existing exclusive maintenance. Tool cleanup, parity and lock TTL are not disposal, scientific certification or effect authority. Bootstrap metadata remains available with the DB down. Native mutable scientific work stays process-local. One declaration owns every cross-boundary vocabulary/schema.

Preserve original numerical qualification, partiality, source/start identity and exact values; creator/reclaimer conflict; no resurrection; actual native/transport drain and charge ownership; independent scientific checks; and explicit runner outcomes. These govern future target behavior despite the authorized fresh-data cutover.

## Confirmed rule decisions

The maintainer explicitly accepted all nine per-item recommendations on **2026-10-12**. RC05–RC07 confirm their limits, not unconditional adoption of conditional alternatives. Confirmation is not implementation acceptance.

| Review item | Confirmed decision | Consequence and route |
|---|---|---|
| [RC01](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#rc01) | Accept retained exact inputs for reproducible studies; distinguish deliberate late binding | A1 changes admission/input ownership. D0 routes relationship/contract rationale and blueprint §20.3/§20.6 before code. |
| [RC02](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#rc02) | Accept independent scientifically sufficient artifacts | A0/A2 implement promotion/release beyond whole-run retention; new relation/operation ADR and blueprint §20.4/§20.6 as applicable. |
| [RC03](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#rc03) | Accept releasable detail, minimal fences and selected audit history | A3 changes history/replay semantics; D0 routes the contract decision and architecture amendment. |
| [RC04](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#rc04) | Accept caller release versus authoritative physical close | B1 changes the Rust/Python boundary; required ADR/design review and blueprint §20.5/§20.6/§21 precede code. |
| [RC05](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#rc05) | Retain patched WS while comparing alternatives | B2/B3 repair it; I3 cannot authorize replacement without a concrete confirmed mechanism decision. |
| [RC06](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#rc06) | Retain local-first binding | I7 records remote extension requirements only; preserve loopback safeguards. |
| [RC07](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#rc07) | Retain truthful failure/retry; assess checkpoint sufficiency | I7 distinguishes starts/history from restart; no durable numerical resume commitment. |
| [RC08](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#rc08) | Accept actual policy compilation/test isolation | A4 moves production policy to pse-model, migrates callers and updates blueprint §3/§20.6. No new crate selected. |
| [RC09](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#rc09) | Accept beta.6 under existing schema/fixture/operational owners | C0–C3 implement the seam; normal exact-pin rules apply. Changed activation/ownership contracts route through D0. No stable0.7 fallback. |

**Additional explicit instruction OP01:** the later hard-pivot instruction removes the earlier legacy-data-preservation premise. D0 records fresh-store cutover and removal of replaced historical activation/migration paths in the relevant decision and blueprint §20.5/§20.6. Do not add a fictional RC10 to the original review or ask again for this instruction.

D0 classifies ADR needs under the repository's decision rules: amend proposed records when suitable, supersede accepted immutable records if needed, and obtain the required review for changed boundary/crate decisions. Ordinary dependency/tool additions need no invented ADR. ADR-0181 and ADR-0182 record the selected decisions; their statuses remain proposed under the decision-PR route.

## Work packages

The table owns current package status. Linked companions own the implementation visions and revealing tests. The prior approved tranche completed investigations and settled foundations. The maintainer has now authorized execution of all remaining D0/A1–A3, persistent C2/C3, Q1 and H1 scope.

| Packet | Required input | Delivery and owner | Acceptance and replacement | Status |
|---|---|---|---|---|
| D0 | Confirmed decisions/current owners | Coordinator: decision route, shared contracts, clean-cutover target | Required ADR/review/design changes before dependent implementation | Complete — target ADR/review/design routes preceded code; ADR-0181/0182 retain proposed decision-PR status |
| A0 | Exact scientific and replay owners | [37a](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/plans/37a-scientific-artifacts-history-and-policy.md#a0-scientific-closure-and-collection-decisions): settle artifact sufficiency/reduced replay | Valid/invalid selections and complete reachable mutation paths; gates A2/A3 | Complete — sufficient scientific selections, exact replay and reachable mutation paths settled; adopted A1–A3 delivered |
| A1 | D0 input contract | [37a](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/plans/37a-scientific-artifacts-history-and-policy.md#a1-retained-exact-study-inputs): exact external start admission | Both retirement orders, pending restart/retry and release; replace late-acquisition-only retained path | Complete — copied exact admission and native study replacement/retry survive producer retirement; retirement-first refuses |
| A2 | D0/A0, A1 input operation | [37a](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/plans/37a-scientific-artifacts-history-and-policy.md#a2-independent-artifact-promotion-and-retirement): promotion/read/release | Producer retires, selected content survives, unrelated outputs drain; replace coarse-only consumer obligations | Complete — independent sufficient content, typed graph evidence and public original-intent recovery passed storage/native/Python and managed consumers |
| A3 | D0/A0, A2 closure | [37a](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/plans/37a-scientific-artifacts-history-and-policy.md#a3-releasable-history-and-dominating-retirement): history release/collection | No retired identity resurrection; audit/unknown protection; delete unnecessary detail after settlement | Complete — releasable detail, audits/unknowns and dominating fences passed storage/scientific/public history controls |
| A4 | Current pure study facts | [37a](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/plans/37a-scientific-artifacts-history-and-policy.md#a4-production-policy-compile-and-test-boundary): relocate actual policy/tests | Same production policy compiles/tests without SDK/codegen/columnar; delete old implementation/imports | Complete — integrated policy relocation; targeted compile and 16 unit tests passed |
| B1 | D0 close contract | [37b](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/plans/37b-connection-and-transport-lifetimes.md#b1-logical-release-and-authoritative-shutdown): caller/activity ownership | Close/admission/renew/publication races and final drain; replace physical-close caller wiring | Complete — integrated caller/activity/physical ownership; targeted39 operations and4 runtime controls plus scoped native/Python compile passed |
| B2 | Current bounded WS contract | [37b](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/plans/37b-connection-and-transport-lifetimes.md#b2-finite-acknowledged-restoration): finite ready/failed restoration | Missing ACK/send failure cannot park future callers or dispatch partial setup | Complete — integrated finite restoration; SDK and actual canonical transport controls passed |
| B3 | Current admission/payload ownership | [37b](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/plans/37b-connection-and-transport-lifetimes.md#b3-explicit-exceptional-admission-completion): explicit terminal release | Held metadata cannot retain non-payload slots; preserve payload physical drain | Complete — integrated explicit terminal/physical owner release; payload controls retained |
| C0 | Beta.6/current generated output | [37c](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/plans/37c-tooling-and-operational-integration.md#c0-beta-input-and-command-qualification): DDL/dispatch qualification | Semantic fidelity, exact selection/auth, executable provenance; gates tool consumers | Complete for the initial qualified subset — exact beta executable, direct generated-schema comparator and sync/replay/parity passed |
| C1 | C0 and namespace owner; C2 initial frozen chain for replay/parity | [37c](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/plans/37c-tooling-and-operational-integration.md#c1-owned-declarative-database-tests): declarative tests/disposal | Register enclosing scope before creation; parity/setup/error/interrupt controls; replace duplicated assertions where fitting | Complete for the approved foundation — registered namespace owner, qualified default-actor assertions, isolation/disposal and failure retention |
| C2 | D0/C0; B1 shutdown for persistent maintenance | [37c](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/plans/37c-tooling-and-operational-integration.md#c2-frozen-forward-rollouts-and-exclusive-maintenance): forward rollouts | Frozen order/hash/repair plus behavior; owner loss fences effects; no old rollout import | Complete — final persistent fixture passed24/24 including compatible forward chain, lost activation ACK and actual caller death; frozen production v5 publication validated |
| C3 | D0/final target declarations | [37c](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/plans/37c-tooling-and-operational-integration.md#c3-clean-rebuild-and-operational-cutover): reset/reseed/activation | Owned drain/reset/rebuild and fresh identities; delete replaced historical adapters | Complete — fresh regeneration/old authority refusal passed in the24-control fixture; obsolete adapters/callers/declarations deleted |
| I1 | Concrete A0 relationships | [37c](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/plans/37c-tooling-and-operational-integration.md#bounded-capability-investigations): native references | Exact structural meaning/races, equivalent simplification or explicit non-adoption | Complete — explicit guards retained; typed structural links only where fitting |
| I2 | Notification/follow-up consumers | 37c: events/live/changefeeds | Useful replacement with retry/resync owner, or reason/trigger | Complete — explicit readiness/polling retained; revisit if measured idle load or response benefit justifies notifications |
| I3 | Protected-read workloads/transport contracts | 37c: streaming and alternative transports | Complete-operation comparison; no unconfirmed WS cutover | Complete — patched WS/paged reads retained; full-operation comparison trigger |
| I4 | Retention and backend behavior | 37c: history, files, reclaim/compaction | Explicit logical/physical outcomes and diagnostic need; no TTL/byte-return inference | Complete — owned bytes, history disabled, logical/physical reclaim distinguished |
| I5 | Actual graph/query access | 37c: indexes/functions/EXPLAIN/edges/caches | Useful shape or no-consumer decision; no wholesale conversion | Complete — selective INFO/EXPLAIN and indexed/NOINDEX fixture controls; current access paths retained |
| I6 | Fixture/inspection/maintenance consumers | 37c: testcontainers/MCP/Studio/metrics/wakeups | Minimal compatible integration or observable trigger | Complete — native fixtures and VIEWER inspection retained; optional adoption triggers |
| I7 | Confirmed local/retry limits | 37c with 37a: checkpoint and extension boundaries | Sufficiency/extension decisions; no remote/resume/search/WASM commitment | Complete — local retry and workload-specific restart boundaries |
| Q1 | Adopted functions implemented, I outcomes settled | Coordinator: assembled affected acceptance | Fresh actual Rust/native/Python/recovery artifacts and scope-end checks, zero failure baseline | Complete — affected storage/native/public Python/persistent/managed controls accepted; scope-end failures repaired and checks passed under the recorded exclusions |
| H1 | Q1 accepted, adopted findings resolved | Coordinator: enduring contracts/existing-plan reconciliation | Update other plans/status only now; retain their actual incomplete qualification | Complete — enduring owners/revision159 updated, affected Plan28/36 contracts reconciled, eligible supporting documents retired with immutable references |

### Dependency slices and editing ownership

The following records the completed dependency structure, not additional work authorization. Temporary coarse paths were removed when their independent artifact replacements passed.

Begin A0/C0 and D0 contracts already decidable. A4/B2/B3 can progress independently in current contracts. B1 waits on its public boundary decision/review. A1 may first protect an input with a conservative producer obligation, but final A2 consumers must use independent artifact ownership; a coarse run pin cannot close F02. Remove any temporary coarse path when replaced and tested.

A2 needs scientific sufficiency, not generalized chunk GC. A3 waits on dominating fences, settled creation windows and actual borrowers. Design it alongside A1/A2 so new detail does not accidentally become permanent. C0 can inspect current DDL early, then must recheck changed generated definitions. C1 sync-only fixture work can proceed after C0 and namespace ownership; its replay/parity closure also needs C2's initial frozen chain. This chain generation does not wait for persistent maintenance qualification. C2's persistent maintenance slice needs B1 quiescence and exclusion of every participating writer. C3 requires working final target contracts; Q1 rebuilds actual consumers after integration.

The coordinator owns common registry declarations, manifests, generated-output coordination, digest, decision route and activation. Independent work may share logical prerequisites but cannot race on these files. Use disjoint ownership/worktrees for concurrent editing. Codegen is part of declaration changes; never edit generated output by hand.

## Finding dispositions

| Finding reference | Scenario reference | Disposition | Decision / work owner | Evidence or revisit trigger |
|---|---|---|---|---|
| [F01](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#f01) | Review S02 | Resolved | A1 with A0/A2 | Exact copied study input is admitted before activation; actual explicit seed/retirement/replacement-worker controls preserve bits/provenance and retirement-first refusal (C-N). |
| [F02](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#f02) | Review S03 | Resolved | A0/A2 | Independent trajectory, actual endpoint, sensitivity/graph content and escaped Arrow bytes survive producer retirement; storage, scientific, public Python and managed Rust controls passed (C-S/C-N/C-P/C-R). |
| [F03](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#f03) | Review S03/S08 | Resolved | A0/A3 | Explicit history release, audits, unknown-effect protection, study cascade and permanent identity/effect fences passed storage/scientific/public controls (C-S/C-N/C-P). |
| [F04](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#f04) | Review S04 | Resolved | B1 | Actual caller/activity admission, protected cleanup and two-driver close controls passed; ordinary runtime close no longer disconnects shared clients. The final affected consumer evidence is recorded in Q1 below. |
| [F05](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#f05) | Review S05 | Resolved | B2 | Finite acknowledged ready/failure and send-failure controls; physical drain, no partial-session dispatch. |
| [F06](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#f06) | Review S05 | Resolved | B3 | Held metadata/cancellation terminal controls release nonbyte activity/admission; malformed payload physical drain retained. |
| [F07](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#f07) | Review S06 | Resolved | A4 | Actual policy/test closure excludes unrelated integration dependencies. |

## Verification and completion

The adopted target is verified through scoped mechanism, scientific consumer, public Python
and persistent lifecycle controls. Source authoring and an accepted target review establish
neither implementation acceptance nor test success. All reported tests use a zero failure
baseline; filtered/deselected tests remain unexercised. Arrow-consuming correctness closures
explicitly carry force-validation. Local fixtures use the selected SurrealDB 3.3.0 RocksDB
service and the patched production SDK; standalone tooling uses surrealkit 1.0.0-beta.6.

The final affected-scope evidence is below. Results retain their original
source/build and allocation conditions; a later source correction does not retroactively
change an earlier run. **Tested — Q1 accepted, 2026-10-12:** all selected affected consumers and required scope-end checks passed after the stated repairs. This is bounded lifecycle/integration acceptance, not whole-product qualification.

| Evidence | Command and conditions | Result against zero failures | Scope and exclusions |
|---|---|---|---|
| **Tested** final storage/protection | C-S below; main, owned v5 science-store, exclusive outer allocation; nextest `d5335b1b-813c-48ed-bc01-f334e9559095` | 23 passed, 0 failed; 207 filtered/skipped by selection; 252.857 s; outer exit 0 and actual context/service drain | Artifact owner/read/creator races, inactive original-intent settlement, history fences/audits/study cascade, exact catalog authority, final source-protection teardown. Not a full operations suite. |
| **Tested** final scientific/native consumers | C-N below; rebuilt main native/canonical-tests closure; nextest `336f5a20-6e3e-4140-ae31-d0bbccdf0c54` | 14 passed, 0 failed; 873 filtered/skipped by selection; 252.911 s; outer exit 0 and actual context/service drain | Retained external inputs and worker replacement, producer retirement with original trajectory/endpoint/analysis evidence, seed admission/recovery and caller continuation. The then-current nested wrapper widened the actual observer kernel cap to the exclusive sibling's 144 GiB; no 4 GiB observer or performance claim follows. That placement defect was repaired separately. |
| **Tested** installed public Python boundary | C-P below; freshly rebuilt native correctness extension and current generated documents, serial selected files on owned v5 science-store | 11 passed, 0 failed, 1 deselected; 250.85 s; outer exit 0 and actual context/service drain | Logical release, admitted read/native continuation, independent callers, artifact promotion/read/recovery and typed history. `eligible_deployment_receipt` was deliberately deselected; no fabricated deployment receipt or test pass is claimed for it. |
| **Tested** persistent lifecycle | C-M below; fresh owned v5 profiles, exact matching force-validated canonical tool, pinned beta.6 and authenticated exclusive native route | 24 controls passed, 0 failed; `checks.json` complete with empty cleanup_errors; outer exit 0; native scopes/listeners/contexts independently drained | Independent start/complete settlement, wrong/competing authority refusal, compatible synthetic forward chain, lost activation-ACK replay, fresh identities/credentials, actual caller SIGKILL with unknown SQL retained. Not historical migration, arbitrary data transformation or physical power-loss proof. |
| **Tested** charged publication and lane controls | C-L below; current main native/canonical-tests closure, owned v5 science-store; nextest `0aa204f2-4d64-473e-9bcb-f8f4059b013e` | 9 passed, 0 failed; 880 filtered/skipped by selection; 19.603 s; outer exit 0 and actual context/service drain | Refusal before publication-future construction, charged ownership through actual operation return, exact retry using retained preparation material, and six scheduler lane/refill/stop/failure controls. |
| **Tested** managed Rust primary | C-R below; rebuilt worker after `00f477260`, exact owned reference on port 19264, supported 160 GiB allocation (140 GiB native, 16 GiB server, 4 GiB observer); nextest `657348a4-594a-4b3a-882c-4883d29b36c8` | 1 passed, 0 failed; 905 filtered/skipped by selection; 58.084 s; outer exit 0 and actual context/service drain | Actual 16-native-entry publication, protected partial-history serving, independent typed artifact graph evidence and escaped Arrow continuation after retirement. No readiness-only, stack-policy or wider Plan28 qualification claim. |
| **Tested** managed public Python composition | C-Y below; freshly rebuilt native correctness extension/worker, exact owned reference on port 19264, supported reference allocation; JUnit `build/p37-managed-python-final-20261012/native-python.xml` | 1 passed, 0 failed; pytest elapsed 45.13 s; outer exit 0 and actual context/service drain | Actual public study cancellation drains all 16 native owners and never starts tail work. No wider Python or Plan28/36 campaign inference. |
| **Tested** script controls and **Interface-checked** final Python typing | C-T below; admitted light script units and strict project-wide Pyrefly | Initial lint-repair selection: 131 passed in 33.455 s; final typing selections: 75 passed in 18.872 s and 56 passed in 6.061 s; all 0 failed/outer exit 0. Strict typecheck: 0 diagnostics, 3 existing suppressions unchanged | Targeted native-operation, frozen-producer, tool/maintenance and fixture-admission controls; annotation/cast repairs preserve runtime ACK/unknown/deadline checks. Does not rerun the actual persistent database journey. |
| **Tested** scope-end checks | C-Q below: hygiene plus exact failing-recipe reruns, governance, docs, native lints and depth-2/no-default feature checks | Initial hygiene: 28 attempted, 6 failed. All six repaired; final individual reruns passed. Governance 12/12 (25 governance tests); both native lints, no-default and corrected combination gates and warning-free docs passed, exit 0 | Named static/build checks establish their contracts; no numerical or performance qualification follows. |
| **Implemented**, source-reviewed H1 | Architecture revision159; operational guides; affected Plan28/36 prospective contracts; resolved companions/inquiries/reviews retired with immutable references | Enduring owners and navigation reconciled; retained coordinator owns complete package/finding tables | Separate campaigns remain open/stopped; ADR statuses remain proposed. |

The two Rust XML reports were independently read at drafting time under the private registry
invocations `root-registry/invocations/6ca61d9f71384533a2524791c345687c.xml` and
`science-registry/invocations/2b36dcc4d4ec41d7a18bbc7e4cc0aa3c.xml` below the workarea.
The later lane/publication and managed Rust reports are
`science-registry/invocations/de983b4021bc4d94920fa618bb115176.xml` and
`science-registry/invocations/c226e163e0ff4110972dac0050613c00.xml`, respectively.
Persistent evidence is `build/p37-persistent-target-v5-20261012-e/checks.json`.
These local ignored reports retain exact selection/outcome; the owning plan records scope.

### Commands for the final affected controls

The recorded workarea is local evidence placement, not a product path requirement. The
owned harness validates its enclosing exclusive allocation, starts only its selected service,
and in `finally` stops that service and checks all contexts drained. The reference harness
also authenticates the exact managed-primary reference and performs supported offline
owner-code readmission when required. The installed extension was freshly built with
`just py-sync-native-correctness` (outer exit 0, clean 3 min build and actual compiled-API
stub generation). The ordinary harness invokes the normal
selected Rust route with `--execution-effects canonical --ignore-default-filter
--test-threads 1`; Python consumes the captured native correctness extension. The commands
below describe already executed controls, not authorization to rerun or dispose resources.

```bash
workarea=/home/paul/.cache/pse-arrow/plan37-completion-20261012
```

**C-S — storage/protection:**

```bash
PSE_TEST_RESOURCE_STATE="$workarea/root-registry" \
PSE_SURREAL_STATE="$workarea/science-store" \
scripts/pse-env --resource-class exclusive -- .venv/bin/python \
  "$workarea/run-owned-controls.py" rust pse-operations \
  canonical-tests,canonical-maintenance \
  'test(canonical_history_tests) | test(canonical_artifact_tests) | test(canonical::maintenance::tests::) | test(source_protection_)'
```

**C-N — scientific/native consumers:**

```bash
PSE_TEST_RESOURCE_STATE="$workarea/science-registry" \
PSE_SURREAL_STATE="$workarea/science-store" \
scripts/pse-env --resource-class exclusive --native=solver,klu,isolation,uno,petsc -- \
  .venv/bin/python "$workarea/run-owned-controls.py" rust pse-runtime \
  canonical-tests,native-solvers \
  'test(canonical_admitted_seed_shares_payload_and_refuses_fresh_retired_acquisition) | test(canonical_original_dependencies_and_retained_sensitivity_persist_exact_sources) | test(canonical_scientific_trajectory_and_endpoint_survive_producer_retirement) | test(canonical_study_explicit_) | test(artifact_recovery_refuses_malformed_and_oversized_documents) | test(canonical_durable_constant_completion_reopens_exact_manifest) | test(canonical_study_equal_bindings_and_failed_usable_dependency) | test(canonical_study_summary_live_owner_and_expired_writer_rebuild_without_science) | test(canonical_study_expired_claim_recovers_without_scientific_rerun) | test(caller_lifetime_tests)'
```

**C-P — installed public Python controls:**

```bash
PSE_TEST_RESOURCE_STATE="$workarea/root-registry" \
PSE_SURREAL_STATE="$workarea/science-store" \
scripts/pse-env --resource-class exclusive --native=solver,klu,isolation,uno,petsc -- \
  .venv/bin/python "$workarea/run-owned-controls.py" python \
  python/pse/tests/test_canonical_results.py python/pse/tests/test_history_lifecycle.py \
  -k 'not eligible_deployment_receipt' -n0 \
  --basetemp="$workarea/python-controls-current-final" -q --tb=short
```

**C-M — persistent lifecycle:**

```bash
PSE_TEST_RESOURCE_STATE="$workarea/ops-registry" \
scripts/pse-env --resource-class exclusive --native= -- .venv/bin/python \
  -m scripts.tests.surrealkit_persistent_fixture_check \
  --directory /home/paul/pse-arrow/build/p37-persistent-target-v5-20261012-e \
  --binary "$workarea/tools/v1.0.0-beta.6/x86_64-unknown-linux-gnu/surrealkit" \
  --canonical-binary "$workarea/tools/xtask-aa3123fc273b3a37609b1848692e581e8c362c8c4418a324248722a89833f289" \
  --port 18341
```

**C-L — charged publication, scheduler lanes and retained preparation retry:**

```bash
PSE_TEST_RESOURCE_STATE="$workarea/science-registry" \
PSE_SURREAL_STATE="$workarea/science-store" \
scripts/pse-env --resource-class exclusive --native=solver,klu,isolation,uno,petsc -- \
  .venv/bin/python "$workarea/run-owned-controls.py" rust pse-runtime \
  canonical-tests,native-solvers \
  'test(charged_publication_) | test(worker_refills_after_progress_even_when_another_lane_was_idle) | test(worker_returns_actual_lane_owner_after_failure_and_drain) | test(worker_refill_respects_total_action_limit) | test(worker_failure_drains_all_issued_lanes_before_returning) | test(worker_stop_drains_every_issued_lane) | test(mandatory_readiness_progress_preserves_until_idle_and_action_budget) | test(ordinary_preparation_failed_publication_retains_shared_material_for_exact_retry)'
```

**C-R — actual managed Rust primary:**

```bash
PSE_TEST_RESOURCE_STATE="$workarea/science-registry" \
PSE_SURREAL_STATE="$workarea/science-reference-store" \
scripts/pse-env --resource-class reference --native=solver,klu,isolation,uno,petsc -- \
  .venv/bin/python "$workarea/run-owned-reference.py" rust \
  'test(managed_primary_publication_serves_exact_partial_history_and_escaped_arrow_retirement)'
```

**C-Y — actual managed public Python cancellation:**

```bash
PSE_TEST_RESOURCE_STATE="$workarea/root-registry" \
PSE_SURREAL_STATE="$workarea/science-reference-store" \
scripts/pse-env --resource-class reference --native=solver,klu,isolation,uno,petsc -- \
  .venv/bin/python "$workarea/run-owned-reference.py" python \
  --junitxml=/home/paul/pse-arrow/build/p37-managed-python-final-20261012/native-python.xml \
  --basetemp="$workarea/python-managed-final" \
  python/pse/tests/test_studies.py::test_public_study_cancel_drains_sixteen_native_owners_and_never_starts_tail \
  -q --tb=short
```

**C-T — scope-end typing/lint repair controls:**

```bash
# Initial lint-repair selection: 131/131, 33.455 s.
PSE_TEST_RESOURCE_STATE="$workarea/root-registry" \
scripts/pse-env --resource-class light -- .venv/bin/python -m unittest \
  scripts.tests.test_native_operation scripts.tests.test_surrealkit \
  scripts.tests.test_surrealkit_maintenance scripts.tests.test_surreal_rollout_producer \
  scripts.tests.test_surrealkit_persistent_fixture

# Final typing repair: complementary selections, 75/75 and 56/56.
PSE_TEST_RESOURCE_STATE="$workarea/root-registry" \
scripts/pse-env --resource-class light -- .venv/bin/python -m unittest \
  scripts.tests.test_native_operation scripts.tests.test_surrealkit_persistent_fixture
PSE_TEST_RESOURCE_STATE="$workarea/science-registry" \
scripts/pse-env --resource-class light -- .venv/bin/python -m unittest \
  scripts.tests.test_surreal_rollout_producer scripts.tests.test_surrealkit \
  scripts.tests.test_surrealkit_maintenance

PSE_TEST_RESOURCE_STATE="$workarea/root-registry" \
scripts/pse-env --resource-class light -- just typecheck
```

The initial lint repair passed `just lint-py` with zero findings. The final strict typing
run reported zero diagnostics, with the same three pre-existing suppressions; this does not
claim the complete final hygiene bundle. Final source-only independent review found no
material issue in the typing repairs. The final static/manual results and repairs are recorded in C-Q below.

### C-Q — scope-end checks and repaired failures

**Tested:** every command below ran through `scripts/pse-env` with the private root registry.
Compile/native commands used `--resource-class compile --native=solver,klu,isolation,uno,petsc`;
metadata/typing commands used the light class and documentation commands the docs environment.
Baseline: zero failures and warnings; each final command exited 0.

- `just hygiene`: all 28 checks attempted; 22 passed and 6 failed on the initial run
  (`20261013T002335.323655Z-hygiene-661057-3d8b63`). Failures were ADR evidence metadata,
  two exact third-party identifiers in typos, a fixture license header, Python typing, and
  the two Clippy feature modes. Existing type suppressions remain three, unchanged.
- `just adr-frontmatter-check lint-typos lint-license typecheck lint-py`: all passed after
  repairs; strict typing reported 0 diagnostics. No ignores or assertion weakening were added.
- `just clippy`: an intermediate rerun exposed the study binding tuple's type complexity;
  naming its existing exact tuple fixed it. Final default and no-default modes passed 2/2
  (`20261013T003337.646885Z-clippy-1469389-6362e2`). ConnectionAccess's derived default and
  removal of an identity conversion preserve behavior; independent source review found no issue.
- `just governance`: 12/12 gates passed, including 25/25 actual governance tests and all generated
  output/current compiled-API checks (`20261013T003443.683105Z-governance-1533514-6df4c5`).
- `just lint-native-contracts` and `just lint-native-data`: both passed with force-validation
  and warnings denied, using the full selected native environment.
- `just features-powerset`: all 392 depth2 combinations attempted; six xtask canonical-tools combinations failed because their recovery binary imports the existing optional pse-model dependency without activating it. The no-default gate passed (23.2s). The original report remains failed (`20261013T003722.452059Z-features-powerset-1697575-49e6dc`). Adding dep:pse-model to canonical-tools fixes the real dependency boundary; the full `just features-combinations` rerun passed 392/392, exit 0. No version or lockfile changed. This compiles features, not solves.
- `just ready`: after the optional dependency activation repair, skills-sync and doctor passed 2/2 (`20261013T013410.414579Z-ready-3105292-0f3385`); no version or lock resolution changed.
- `just docs`: the initial build exited0 with four unclosed generic-type HTML warnings.
  Marking those existing type names as code fixed the rendering; the corrected build passed
  without warnings. This formatting change does not revise historical review judgments.
- Root `just turn-end`: ADR index and formatting passed. The final H1 publishing and
  lifecycle checks follow the documentation-only reconciliation; no product rerun is inferred.

An attempted validation command using individual gate names as `--group` was rejected by
argument parsing (exit 2) before checks; it establishes no validation. The valid named recipes
above supplied the actual reruns. Initial and intermediate failed reports remain failed.
Static repairs changed annotations, metadata, equivalent defaults/conversions, one missing optional dependency activation and formatting;
actual functional results retain their original source/allocation conditions above.

### C-H — final documentation and handoff checks

**Tested, 2026-10-12:** `python3 -m scripts.document_lifecycle validate` passed
with no errors (381 native scope items validated). The Plan37 `scope --path
docs/plans/37-surrealdb-lifecycle-and-integration.md` projection reported 29 complete,
zero open/blocked/deferred/unknown: 22 packets and seven findings. Both commands used
`scripts/pse-env --docs --resource-class light` and the private root registry.
`just docs` published 382 chapters with scoped search and no warnings. Root
`just turn-end` passed ADR index and formatting, 2/2 against the zero-failure baseline.
Independent final source/document review found no material issue. All 15 retired paths
are recoverable at the immutable commit cited above; the pre-existing `.design-edit`
authorization was restored byte-for-byte. The repository has only the main checkout.

**Failed — additional whole-site offline link audit:** the first `lychee --offline
--no-progress --include-fragments --exclude-path docs/book/404.html docs/book` attempt
exited 137 when its 2 GiB light allocation was OOM-killed. Under an explicit finite
`PSE_MEMORY_MAX=8G` allocation and `--max-concurrency 8`, the audit completed. The
new vendor source link and two existing source links in the affected substrate guide
were repaired as repository links. The final whole-site command still exited 2:
312 broken-link occurrences across 16 other retained pages and their print aggregate,
against a zero-error target. None of the 18 affected H1 published inputs had link
errors. These remaining historical source/role links belong to other documents;
whole-site link health is not established or silently baselined. The passing rendering,
metadata and affected lifecycle results do not turn this additional audit into a pass.

**Tested — owned-resource drain checks:** the five selected owned services were independently rechecked inactive,
nonresident (`MainPID=0`, empty `ControlGroup`) with all contexts drained. A global
`just activity --json` snapshot exited 1 because a unit disappeared during observation;
it cannot establish whole-host quiescence. Exact owned-service observations, preserved
unknown-effect records and the bounded inventory below supply the handoff instead.

### Qualification boundary

No representative performance, reclaimed-byte/physical-space, physical power-loss, remote
or distributed deployment, arbitrary-data transformation, historical-data compatibility,
numerical checkpoint/resume or whole-product qualification is claimed. The selected native
science checks retain independent original assessments and partiality; immutable reuse does
not certify newly computed states. The stopped Plan28 and Plan36 campaigns remain incomplete
under their own plans. Their original failures, inconclusive cases and unexercised scope are
not closed by Plan37's focused consumers or by H1 document reconciliation.

Failed predecessor persistent profiles and the final process-death profile remain pinned
with their original uncertainties. The final killed caller's SQL remains unknown even though
its exact processes/listeners drained. Earlier failed scientific fixture databases and runner
outcomes remain retained: authentic dead borrower/scope reconciliation enabled service/host
release without fabricated ACKs, database disposal or relabeling a failure as a pass. Terminal
runner proof, native/service drain, remote SQL settlement and logical versus physical disposal
are separate facts. Final inspection found the owned science service (19261), reference service (19264) and three persistent fixture profiles (18341–18343) inactive, nonresident and all contexts drained. The forward profile carries its new regeneration identity. Failed scientific resources `46e14e12463a4f7dba1402e361111047` and `a0a5be7daa6b43c49045ef979b4f9dfd` remain failure-pinned, authentically drained and without borrowers. Five persistent fixture evidence records, including `65ee24cc3bce4301b4c7dc74174d7912`, remain pinned/incomplete rather than asserting SQL/runner settlement; final process-death pending identity `2c0b0363884e43c1afdc85d766a0ca21` stays unknown. All five persistent fixture allocation nonces are absent from the host ledger. Independent exact observations found all 19 native runners and 8 servers absent, their units inactive/absent and all 27 recorded cgroups absent. The earlier b fixture retains one uncompleted borrower-release record (resource `9f47dcd33b684f6ab4bd4ac25b4157a3`, token `9e838abd83f44429a18005e7be291bf3`, absent PID 715157/start 538104); its cause is unestablished, and it was not erased or treated as live work. The evidence registration defaults are not a native-drain or SQL-settlement claim. No failed database or uncertainty record was discarded for closure. Remaining exact resource/process observations are bounded to owned profiles; unrelated host work is preserved.

## Current checkpoint and final handoff

**2026-10-12 — replacement current checkpoint.** Maintainer-authorized remaining Plan37
implementation is integrated on main: retained exact study inputs, independent scientifically
sufficient artifacts and typed graph evidence, explicit releasable history with permanent
identity/effect fences, Rust/Python lifecycle adapters and settlement-only original-intent
recovery. Current source documents/IDs and scientific qualification meaning are preserved;
absent or inactive content cannot be reopened through recovery. Independent source review
led to corrections for selected-client authority, released-owner resurrection, exact operation
attribution, shared-parent audit order, post-release reads and finite original-intent equality.

Persistent maintenance now consumes the frozen generated v5 publication, validates exact
phase definitions, settles the original operation, and activates through an atomic receipt
with lost-ACK replay. Clean design-phase regeneration creates fresh identities/credentials;
obsolete historical preparation/backfill/readers/callers/declarations are removed. Current
versioned documents, split-product decoding, normal backup/restore and anti-forgery checks
remain. OP01 requires no preservation/migration of design-phase data.

The completed evidence establishes 23 storage, 14 scientific/native, 11 selected public
Python, 24 persistent, 9 charged-publication/lane, 1 managed Rust and 1 managed Python controls
as bounded above; final typing/script controls are recorded separately. Q1 affected acceptance and H1 are complete; static/manual repairs and results follow C-Q. Other plans remain stopped/open under their own owners. ADR-0181 and ADR-0182 remain proposed;
implementation authorization and target review acceptance did not change their decision-PR
status. Packages and finding dispositions remain in the existing
owning tables; this checkpoint does not duplicate or change them.

### Historical investigation/foundation tranche

The prior approved tranche began on baseline `a6aedd31` and settled A0/I1-I7, A4/B1-B3 and
bounded disposable C0/C1/C2 foundations. The contribution evidence below is historical under
its original source, fixture, allocation and exclusion conditions. At that time production
A1-A3, persistent C2/C3 and assembled Q1 were not implemented/accepted. The later results above
are separate; the foundation evidence is not an assembled current-source pass.

A0 selected small owned seed capsules, artifact-owned bounded IPC copies and explicit
reduced-history ownership. I1-I7 retain explicit conflict/scientific/borrower guards, bounded
readiness polling, patched WS and protected page/block reads, explicit history/native bytes,
selective registry-owned query access paths, owned native fixtures/VIEWER inspection, and
truthful local retry. Optional references/LIVE/streaming/files/compaction/containers/collectors/
remote checkpoints remain consumer- and trigger-dependent. Their enduring choices and
observable triggers transfer to the architecture/developer owners through H1; they establish
neither missing library capabilities nor representative performance.

Archived inquiries retain the original evidence and limits:
[scientific closure/replay](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/plans/37-investigation-evidence/scientific-closure-and-replay.md),
[operational capabilities](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/plans/37-investigation-evidence/operational-capabilities.md), and
[beta tooling qualification](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/plans/37-investigation-evidence/beta-tooling-qualification.md).
The enduring contracts and I1-I7 revisit triggers belong to the architecture/developer owners
reconciled in H1; the retired companions are not an implementation backlog.

The [original review](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md)
supplied the finding/decision scope. D0 created proposed ADR-0181/revision157; the
[independent B1 target review](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/design_review/reviews/design_review_b1-connection-lifetimes_2026-10-12.md)
accepted its logical caller/activity/physical-drain contract before implementation.
Remaining-scope D0 created proposed ADR-0182/revision158 with
[independent bounded target acceptance](https://github.com/paul-heyse/pse-arrow/blob/e145eb7d229000dd9dda86882fe85c0b301657e2/docs/design_review/reviews/design_review_plan37-remaining-target_2026-10-12.md)
at Proposed strength. These target decisions are distinct from implementation tests and
Q1 acceptance.

A4 integrated as `6255da3a9`: actual production policy and all16 direct tests moved to `pse-model`; operations export/source removed, actual callers rewired and petgraph ownership moved without dependency-version changes. **Tested:** `just check-package pse-model`, `just unit-package pse-model study_policy` (16 passed,0 failed,38 filtered), and `just check-library pse-operations -p pse-runtime` (0 failures/warnings), through the worktree's admitted environment. Actual normal/test closures contain63 packages and exclude SDK/codegen/columnar/Arrow/DataFusion. This establishes the policy boundary, not assembled Plan37 qualification.

B2/B3 integrated as `32d73ed0c` and their clean worktree was removed. **Tested:** final vendored SDK WS/query/select controls52/52, engine-api lifetime controls6/6 and actual `test(canonical::transport::tests)`19/19, zero failures, through the owned compile/native-local route with canonical force-validation. Wasm shared-source changes were not target-compiled. Two pre-existing vendor manifest homepage warnings remain reported; no product compiler warnings were observed. The patch adds a separate clearable nonbyte activity owner, preserving payload classification and remote-unknown semantics.

B1 integrated as `b8a2756ee`: ordinary runtime/Python close releases logical admission; actual operations, durable supervisors, heartbeat/progress publication, protected acquisition/release and detached native joins retain admitted activity. Physical refusal preserves usable admission, while accepted close excludes new work and drains both drivers with resumable partial-close ownership. Idle aliases and sealed remote-recovery intents do not pin a connection indefinitely. Independent source review found no material defect. **Tested:** through the worktree's private registry and plain compile-admitted route against the owned3.3.0 fixture, `just unit-package pse-runtime caller_lifetime_tests --features canonical-tests --execution-effects canonical` passed4/4 (546 skipped), and serial `just unit-package pse-operations 'test(canonical::lifetime::tests) | test(canonical::transport::lifetime_tests) | test(canonical::transport::tests) | test(canonical::canonical_server_unit::source_protection)' --features canonical-tests --execution-effects canonical` passed39/39 (188 skipped), both with zero failures/warnings and force-validation. `scripts/pse-env --native --resource-class compile -- just check-library pse-py -p pse-runtime --features pse-runtime/native-solvers,pse-py/native-solvers` passed with zero Rust warnings/errors. An earlier native-observer route compiled but ran no tests because its requested CPU width did not fit the inherited lane; that failed attempt remains unqualified. The final tests use the existing supported route; no capacity contract was changed. The Python exposed signature is unchanged, so no generated stub was edited or regenerated. This foundation evidence did not rebuild or exercise the installed extension and assembled scientific journeys; their later evidence is listed separately above.

C0/C1 uses a fresh owned beta-profile service and private resource registry; original application stores remain untouched. **Tested:** the complete generated-contract fixture helper passed in `generated-oracle-v4`: direct generated installation and seven semantic cases, sync/replay/parity15/15, cross-namespace credential refusal, exact child/namespace disposal and unrelated-fixture preservation. Selected INFO/EXPLAIN/index-versus-NOINDEX evidence settles I5 with current access paths retained; it does not measure representative scale. Script/resource/maintenance controls passed97/97 against zero failures; the subsequently expanded maintenance module passed22/22 and existing supervisor module200/200. Failed earlier probes remain separately retained; one SQL-shell namespace recreation was corrected from the archived exact resource authority and independently checked absent. CLI child drain and remote SQL settlement are not interchangeable.

C2's approved maintenance foundation is implemented and independently source-reviewed. **Tested:** `scripts.tests.surrealkit_rollout_fixture_check` passed the disposable three-rollout chain/catch-up journey (seven successful plan/up commands, four offline invalid-chain refusals), and its `owner-revocation` mode passed against a separate registered service generation in `maintenance-owner-revocation-v2`, through the private-registry native-light route. The actual lock was unexpired when this test revoked its own reservation identity. The exact beta child stopped with `owner_lost` and authentic native drain; listener drain completed. Exact status and one 15-second repair retained SQL uncertainty; repair refused the still-live beta lock. Ordinary open/start and the operation settlement callback refused; pending nonce/input remained unknown with admission closed and no activation. Root independently verified the exact child scopes and selected service drained. This tests reservation revocation, not process death, successful interrupted-SQL settlement, arbitrary transformations or persistent rollout qualification. The first failed observer probe remains failed; its original start was separately reconciled only after exact stored step acknowledgements, manifest/catalog comparison and authentic drain. No replay, TTL manipulation, rollout completion or activation followed.


The historical root `just turn-end` initially found 24 internal trailing-whitespace errors
in new async wrappers; removing whitespace-only lines allowed `just fmt` to complete. Its
remaining Ruff diagnostics were not a clean lint result. The private-registry `just ready`
environment bundle passed; the unrelated default registry migration fence was preserved.
These historical observations are not substitutes for the final static/manual row above.

## Outcome (recorded after implementation)

### What was built

**Implemented:** independent sufficient scientific artifacts and exact retained external
study inputs; typed graph evidence and original provenance; separate explicit payload/history
release with dominating identity/effect fences and bounded collection; logical caller release
with admitted continuation and deployment-owned physical drain; finite patched-WS restoration
and exceptional terminal ownership; isolated production study policy; qualified beta tooling,
immutable frozen v5 publication, exclusive phase settlement/activation and fresh design-phase
regeneration. All production consumers moved to the target; replaced historical adapters,
callers, tests and declarations were deleted. Versioned scientific documents and current
backup/restore remain supported without historical-data migration.

**Tested:** the bounded 23/14/11/24 controls, 9 charged-publication/lane controls, managed
Rust 1/1, managed Python 1/1, final 75/56 script controls and the historical foundation
results above, under their named commands, fixtures and exclusions. **Interface-checked:**
final strict Pyrefly, zero diagnostics with existing suppressions unchanged. **Tested:** required C-Q static/manual checks passed after the recorded failures and repairs; Q1 affected acceptance is complete. The additional C-H whole-site link audit remains failed outside the affected inputs.
**Implemented:** H1 transferred enduring meaning to blueprint §19.2/§20.3–§20.6/§21/§24.2 and the operational guides, reconciled affected Plan28/36 prospective contracts, and retired three companions, three investigations, three resolved reviews and six supporting evidence documents. Immutable historical links remain. The highest-numbered coordinator and its native scope bindings stay retained. Enduring operation belongs to [the operational store guide](../dev/operational-store.md) and [the substrate guide and capability choices](../dev/surreal-substrate.md).
No performance or physical-space improvement was measured, and no whole-product scientific
qualification follows. Other plans retain their incomplete campaigns and original evidence.

### A mistake made and corrected

The first beta disposal helper selected the namespace being removed; the pinned SQL shell
then recreated it. Disposal now uses an unselected root session and verifies catalog absence.
The original failed invocation remains failed; only its authenticated recreated fixture was
corrected. Persistent maintenance also exposed lexical filename order diverging from beta's
validated chain. Execution now captures the pinned beta lint order and requires a duplicate-
free complete frozen inventory; independent settlement still requires the initial baseline
and every earlier exact ACK.

Actual scientific tests exposed protection teardown ordering: continuation ownership must
release before waking final drain, both client aliases must drop before their fixture owner,
and final explicit protection finish must end its activity before fixture teardown. The final
23 storage/protection and 14 scientific controls exercise the corrected lifetime path; the
two original fixture failures retain truthful runner outcomes and pinned databases.

Managed-worker execution exposed nested workload placement widening the observer's actual
kernel cap and large inline preparation/publication futures exhausting the worker stack
before native entry. The placement wrapper now preserves authenticated finite enclosing
placement. Actual source/debugger diagnosis led to generic corrections in `00f477260`:
box the lane future at scheduler ingress; infer its extent from an uncalled closure's future
type rather than constructing an inert sample on the stack; and charge the complete active
lane state and scheduler metadata before allocation. Canonical publication likewise reserves
its scientific-runtime budget before constructing/boxing the publication future and retains
that owner until the operation returns. These mechanisms preserve original clocks, operation
ownership and preparation material for exact retry; no increased stack or weakened capacity
policy substitutes for them. The 9 helper/lane/retry controls and actual managed Rust/Python
passes above exercise the corrected routes.

That managed artifact fixture also incorrectly used semantic labels as graph endpoints and
confused a projected reader slice with the complete original IPC member. The fixture now
binds endpoints to `node.key`, retains the full three-row original member, and asserts the
original evidence row 2 and bit-exact block bytes after producer retirement. Those repairs
correct the test's identity/evidence interpretation without relaxing production validation.

### Deviations from the plan, deliberate

OP01's fresh target pivot is the approved target, not a deviation; old records were not
converted or retained merely for compatibility. Retained scientific selections use owned
bounded copies, including small seed capsules, rather than a general shared-block GC system.
Optional capability non-adoptions retain named consumers and observable revisit triggers.
No remote checkpoint, alternative production transport, automatic deletion scheduler or
physical-space promise was added.

The disposable beta fixture permits its independently authenticated parity clone without
changing ordinary simulator session capacity. Its early maintenance control revoked its own
reservation with the composer alive and therefore established reservation-loss fencing only.
The later persistent 24-control journey separately exercised actual caller death and retained
unknown SQL; neither result is physical power-loss proof. Installed Python evidence excludes
the deployment-receipt test rather than inventing a qualification receipt. The bounded
implementation corrections above introduce no further deliberate departure from the target.
Q1 and H1 are complete under these exclusions; no further Plan37 implementation is scheduled.
