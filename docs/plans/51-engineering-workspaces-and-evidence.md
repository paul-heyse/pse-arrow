---
title: "Plan 51: Engineering workspaces and evidence"
status: in-progress
date: 2026-10-22
adrs: [ADR-0216, ADR-0217, ADR-0218, ADR-0219, ADR-0220]
review_sources: [docs/functional_reviews/surrealkv-surrealmx-functional-extensions-2026-10-22.md]
scenario_sources: [docs/functional_reviews/surrealkv-surrealmx-functional-extensions-2026-10-22.md]
doc_role: plan
doc_owner: docs/plans/51-engineering-workspaces-and-evidence.md
doc_topics: [current-work, design, storage, libraries, diagnostics, validation]
---

# Plan 51: Engineering workspaces and evidence

## Purpose, scope and authority

**Proposed:** deliver a headless engineering workbench over exact projects, source revisions,
scientific data and studies, and a queryable evidence workbench over selected test, diagnostic
and measurement histories. Rust, Python and CLI consumers come before a graphical frontend.
Keep scientific evidence, derived discovery and operational authority distinct while making
their actual relationships easier to navigate.

This series captures the complete [functional review](../functional_reviews/surrealkv-surrealmx-functional-extensions-2026-10-22.md):
O01–O21, RC01–RC07, alternative realizations, investigations and delivery scenarios. The
maintainer selected full coverage with the review's delivery/investigation/conditional distinctions
preserved, and storage/payload choices settled during authoring. The maintainer subsequently
authorized 51a/51b, 51c/51d and the complete remaining 51e–51g execution in separate requests.
Their actual acceptance conditions are recorded in the companion Outcomes; planning documents
alone authorize no new work.

At authoring, existing plan documents and their index were deliberately unchanged. After
implementation, Q04 reconciles delivered overlap with their existing owners; it neither reopens
Plan50 nor closes independent Plans32/40/49. Companions own design and package explanations; only this
coordinator owns opportunity disposition, inquiry outcomes, package state and completion.

## Foundation assessment and selected design

**Interface-checked:** source `8fe88f318d5f0cad20dfe9043f5e5108b109aafe`, plus the supplied
untracked local source `docs/libraries/SurrealMX.md`. Both KV/MX overviews were read completely
for the source review; relevant SurrealDB/tooling sections and exact source were followed during
authoring. Feature followups use supplied versioned primary links, not Context7. Core 3.4,
Efficient Architecture Heuristics 1.0, Process Simulator 1.5 and the binding shape this focused
foundation assessment; no broad diagnostic audit or enclosing-system certification is implied.

Independent source mapping, library research and design advice support these authoring choices:

| Design decision | Selected target and reason |
| --- | --- |
| Canonical topology | Retain the shared authenticated SurrealDB3.3.4 service with the existing MX1.0/SSI patches, gRPC and installed Surrealism policy. Independent primary, observer, Rust/Python and tooling processes already consume shared authority. New work uses that actual topology, not a generic interchangeable backend facade. |
| Bulk payloads | Retain sealed IPC/opaque files and exact admitted descriptors. Current readers use protected `Read + Seek` and checked Arrow FileReader access; they are not established mmap/zero-copy readers. Persistent MX does not become the bulk-data archive. |
| Bounded atomic metadata | Use generated typed records, native indexed selection and short guarded SSI transactions for project heads, drafts, lineage and derived index publication. Stage large objects outside activation transactions under existing owners. |
| Interactive source state | Keep immutable owned documents/RPDS and existing local admission. Persistent drafts are explicit raw-source state; invalid drafts never become scientific sources. No additional direct MX working store. |
| Evidence queries | First support explicit supplied-only Arrow/DataFusion projections over authentic exported facts. Add an explicitly imported compact persistent index in a separate tooling database on the selected service. Original invocation, resource and diagnostic owners remain authoritative and independently readable. |
| History | Use immutable context/calibration revisions with explicit effective intervals, correction ancestry and recorded cuts. Native KV `get_at` is not required; exact run receipts retain their original consumed closure. No global bitemporal framework. |
| Exchange | Use a versioned logical manifest and selected copied closure, imported under fresh authority. Engine directories/checkpoints are recovery artifacts, not the editable project interchange format. |
| Observation | Compose existing study status, accepted progress/history and live sample owners with explicit cursor gaps. gRPC supports live/query streams, but `.live()` does not establish durable reconnect replay. |

The decisive consumers are `workflow/worker.rs::ensure_managed_primary`, native execution
receiver selection, generated canonical indexes/constraints and same-snapshot readiness,
`canonical_results::open_result_file`, `result_snapshot::access_block` and
`result_files::open_file_reader`. Local edits already branch and source admission already
supports exact deltas; studies, fitted outputs, protected result selection and review packs exist.
The new capabilities are persistent project interaction, explicit research lineage and cross-run
discovery, not replacement numerical execution or a new source language.

### Fairly considered alternatives and closure of authoring decisions

Direct persistent KV can co-publish bounded bytes and membership and can reduce external-file
intents and checkpoint roots. It is technically credible. For this series it is **not selected**:
it would assume the consumed query/index/constraint and reference contracts, replace native SSI
with SI-aware application enforcement and add owned-value retrieval or a seekable chunk adapter.
Large multi-transaction products would still need manifests; long-lived seeds are an unfavorable
case for prefix-only value-log reclamation. This is a responsibility choice, not a measured
performance defeat. No execution package is required to prototype an unselected replacement.

Embedded SurrealDB is the strongest desktop alternative. It is feasible: deliberately consume
the patched MX adapter, forward the exact core Surrealism build feature, enable runtime
capabilities and install/attest the module. A plain stock SDK feature does not inherit the
custom server's patch set. Preserving independent clients would require an application broker
or changed exclusive ownership, so embedding would not automatically remove the relevant
lifecycle obligations. Direct MX is attractive for a single-process transactional collection
but would duplicate the chosen source structures/catalogue here.

Reconsider these choices for a concrete independently editable standalone-desktop requirement,
a required atomic byte/catalogue capability the chosen design cannot express, or demonstrated
workload failure of the retained topology. Neither vendor throughput nor an arbitrary prototype
is a prerequisite to a later design choice. No automatic hot/cold MX–KV tier, replication or
cross-engine transaction is assumed.

## Series and shared contracts

| Companion | Responsibility | Primary opportunities |
| --- | --- | --- |
| [51a](51a-storage-foundations-and-project-ownership.md) | Storage placement, declarations, project/evidence repositories and atomic shared operations | O10, O13, O16, O17 |
| [51b](51b-projects-revisions-and-temporal-context.md) | Projects, named branches, durable drafts, semantic comparison, historical context and reuse discovery | O01–O03, O11 |
| [51c](51c-scientific-data-studies-and-results.md) | Dataset/calibration/fit-to-parameter lineage, study/result queries and observation | O04, O05, O12, O21 |
| [51d](51d-development-and-research-evidence.md) | Typed test/diagnostic projections, offline/persistent evidence queries and comparable measurements | O06–O08 |
| [51e](51e-reproducibility-lifecycle-and-storage-health.md) | Exact portable closure, protection/retention, recovery, storage health and confidentiality decision | O09, O14, O15, O19 |
| [51f](51f-portable-projects-and-delivery-modes.md) | Logical project exchange, offline/browser and collaboration boundaries | O18, O20 |
| [51g](51g-qualification-and-publication.md) | Composed acceptance, scope-end checks, enduring publication and later reconciliation | All |

Shared meaning is fixed here; packages declare/generate concrete contracts at their existing
owners rather than inventing a type/service for every conceptual name.

- **Project/branch/source:** a project is organization and ownership, not a new physical model.
  A writable named branch selects its own canonical source-problem/head. Fork imports a protected
  immutable closure through the source owner; two writable branch labels must not accidentally
  contend on the same single source head. Bookmarks may select arbitrary exact revisions.
- **Draft/admission:** a draft has exact base, generation and original text/typed edits. Saving
  invalid text is permitted; preparation/execution requires ordinary admission. Publishing is an
  explicit head-CAS effect. Undo creates/selects a revision; savepoints never rewind science.
- **Research lineage:** measurement, calibration, observation selection, fit run/attempt and
  derived parameter source revisions have distinct identities. A parameter publication receipt
  references original admitted cells and the actual new source revisions; it owns no competing
  copy of the fitted truth. Downstream studies record that consumed ancestry.
- **Time:** acquisition time, effective calibration time, integration/model time, commit order
  and capture time are not interchangeable. Historical selectors state effective time and the
  recorded revision/cut. Exact run reconstruction never floats to a corrected latest value.
- **Evidence:** source kind/authority/codec/identity, imported snapshot identity, observed coverage
  and actual associations are explicit. A derived view cannot freeze a terminal, release a file,
  infer missing tests passed or turn sampled counters into causal/scientific conclusions.
- **Selection/protection:** selectors freeze exact source/attempt/cohort membership and completeness;
  cursors retain their issuing context. Metadata reads do not grant payload lifetime. Protected
  lazy reads use current owners; offline copied closure is self-contained and cannot borrow live
  resources merely because it retains an old path.
- **Query/visibility:** use operation-shaped indexed native queries and selected columnar batches,
  not per-result scalar fetch loops. Result-stream rows remain provisional until successful
  statement completion; imported index generations activate only after coverage reconciliation.
- **Effects:** every mutation has its actual operation identity, expected generation and original
  allowance. Conflicts retry the whole owned decision; uncertain effects use exact settlement.
  Observer exit is not cancellation/drain; science and bulk staging stay outside metadata transactions.

## Rule-impact decisions

The review lists conditional impacts, not seven required waivers. The selected target avoids
the backend/payload/resource-authority replacements. No unaccepted rule change is used to make
the design work. Maintainer selections of 2026-10-22 authorize full planning coverage and authoring-
time design selection, not a blanket waiver. Subsequent explicit execution requests authorize
51a–51g; targeted and assembled evidence lives in the companions without waiving the rule
decisions. A newly discovered actual change must be presented with its consequence and confirmed before the dependent target is revised.

| Review item | Plan disposition | Route / consequence |
| --- | --- | --- |
| RC01 | Selected new functional scope; no authority replacement | ST01/EV01 declare derived evidence semantics. Completed Plan50 stays complete and unchanged. |
| RC02 | Not activated: unified native payload replacement not selected | Preserve blueprint §20.1–§20.4 and ADR-0187 target. A later replacement needs its own explicit decision. |
| RC03 | Not activated: canonical topology retained | Preserve D10, blueprint §20.6 and ADR-0164/0214 target. ST02 integrates new consumers without changing their authority. |
| RC04 | Not activated: invocation/resource registry retained | Preserve blueprint §24.1/ADR-0174. EV readers/imports remain independently disposable projections. |
| RC05 | No expanded whole-workspace persistence selected | PJ06 exposes actual existing eligibility. Native handles, providers and Salsa owner state are not transferred. |
| RC06 | Apply existing declaration/decision routes | ST01 declares/generates the new families under proposed ADR-0216 and the maintainer-authorized 51a target. Host observations retain their original Python owners. No foundational identity/Python-boundary waiver is selected. |
| RC07 | No new browser/security/synchronization guarantee selected | LC05/DL03–DL04 settle bounded capability decisions. Any new platform/authority guarantee must receive explicit selection and required decision work before implementation. |

## Opportunity dispositions

Scheduled denotes proposed implementation or a required bounded inquiry, not delivered behavior.
Resolved here can close an authoring choice without claiming product implementation.

| Opportunity | Disposition | Work owner | Required behavior or disposition |
| --- | --- | --- | --- |
| O01 | resolved: implemented and targeted-tested | PJ01–PJ04 | Exact navigation/selection, logical comparison, guarded explicit merge/publication and compensating undo; [51b evidence](51b-projects-revisions-and-temporal-context.md#verification-and-handoff); series qualification remains Q01–Q04 |
| O02 | resolved: implemented and targeted-tested | PJ05/SC02 | Effective/recorded context, exact source reopen and frozen calibrated observation selection; actual transformed sigma/weights, invalid-domain/unknown-uncertainty refusal and unchanged old fit receipts after correction; [51c evidence](51c-scientific-data-studies-and-results.md#verification-and-handoff) |
| O03 | resolved: implemented and targeted-tested | PJ02/PJ04 | Immutable raw versions, local admission/diagnostics, savepoint restoration, original intent settlement and one-occurrence cancellation/undo custody; [51b evidence](51b-projects-revisions-and-temporal-context.md#verification-and-handoff) |
| O04 | resolved: implemented and targeted-tested | SC04/SC05 | Native snapshot cohorts and protected selected physical KPI analysis preserve complete missing/failed populations; passive per-attempt reconnect reports catch-up or explicit gaps; [51c evidence](51c-scientific-data-studies-and-results.md#verification-and-handoff) |
| O05 | resolved: implemented and targeted-tested | SC01–SC03 | Original measurements → authored calibration/transformed uncertainty → actual LM → original eligible cells → guarded parameter-source publication → downstream prediction; [51c evidence](51c-scientific-data-studies-and-results.md#verification-and-handoff) |
| O06 | resolved: implemented and targeted-tested | EV01–EV03 | Authentic declared test populations, supplied-only DataFusion questions, exact activated cuts/freshness and protected detail; [51d evidence](51d-development-and-research-evidence.md#verification-and-evolution) |
| O07 | resolved: implemented and targeted-tested | EV04/LC04 | Original diagnostic links and bounded storage-health observations preserve tool/service/scientific distinctions and current identity; metric-bound failure retains service/cgroup observations. 51e verification; final qualification at 51g. |
| O08 | resolved: implemented and targeted-tested | EV05 | Explicit authentic measurement contexts/sample populations, failed/censored and unsupported membership; [51d evidence](51d-development-and-research-evidence.md#verification-and-evolution); no quantitative improvement claim |
| O09 | resolved: implemented and targeted-tested | LC01/DL01–DL02 | Exact selected closure and explicit new supported reproduction/import |
| O10 | resolved: no additional native working store | ST02/PJ02 | Existing immutable documents/RPDS plus catalogue transactions; revisit a concrete unmet operation |
| O11 | resolved: implemented and targeted-tested | PJ06 | Passive resident/canonical discovery through existing owners, actual compatibility and explicit unchecked eligibility; no measured performance claim |
| O12 | resolved: implemented and targeted-tested | ST02/SC04/EV02 | Operation-shaped native filters and selected Arrow/DataFusion cohort/evidence analytics; no universal query framework; [51c](51c-scientific-data-studies-and-results.md#verification-and-handoff)/[51d](51d-development-and-research-evidence.md#verification-and-evolution) |
| O13 | resolved: foundation operations | ST02–ST04 | Implemented and targeted-tested project/evidence transactions, exact settlement and protected selections; [51a verification](51a-storage-foundations-and-project-ownership.md#verification-and-handoff). Future consumers retain these contracts; assembled series qualification is Q02/Q03. |
| O14 | resolved: implemented and targeted-tested | LC02/LC03 | Application-closure recovery and explicit retained-resource explanations |
| O15 | resolved: implemented and targeted-tested | LC04 | Useful MX/native storage health joined only to real owner evidence |
| O16 | resolved: native payload unification not selected | 51a design decision | Preserve sealed files; logical exchange solves handoff without a second payload backend |
| O17 | resolved: shared service target selected | ST02/ST03 | New consumers integrate with the chosen deployment; no execution-stage topology campaign |
| O18 | resolved: native/offline alternative selected | DL03 | Resolve offline/browser participation; no ready KV1.0 OPFS Tree or browser solver claim |
| O19 | resolved: no application crypto selected | LC05 | Decide a complete-artifact confidentiality capability; no default new crypto rollout |
| O20 | resolved: logical exchange; no live sync selected | DL01/DL02/DL04 | Logical editable project exchange; collaboration implementation conditional on explicit selection |
| O21 | resolved: implemented and targeted-tested; batch decision settled | SC05/SC06 | Read-only observation/reconnect with actual progress/history, source gaps and no cancellation ownership; explicit file/batch admission selected, no external acquisition/control producer |

## Inquiry outcomes and remaining questions

| Inquiry | State | Owner | Decision / completion evidence |
| --- | --- | --- | --- |
| I01 | resolved: shared topology | 51a | Actual independent clients and generated invariant consumers favor existing service; embedding remains a concrete alternative, not an unsupported feature claim. |
| I02 | resolved: sealed bulk payloads | 51a | Seekable protected FileReader path retained; no mmap premise. KV owned-value and retention tradeoffs do not justify replacement for this scope. |
| I03 | resolved: no extra native workspace | 51a/51b | No demonstrated operation gap beyond immutable sources and catalogue transactions. |
| I04 | resolved: explicit temporal records | PJ05 | Use exact source/context revisions with effective intervals and correction ancestry; no native memory-engine time travel. |
| I05 | resolved: logical exchange | LC01/DL01 | Portable manifest/copy closure, not engine directory identity or live native state. |
| I06 | resolved: native/offline selected, no browser adoption | DL03 | Maintainer selected native tools and independent logical-pack inspection. Revisit a named browser user operation; then compare logical viewer/editor before browser-native persistence. |
| I07 | resolved: no application crypto | LC05 | Local filesystem access control and explicit operator custody; host encryption is not established. Revisit a concrete confidential exchange or at-rest requirement covering all artifacts and key recovery. See execution checkpoint inventory. |
| I08 | resolved: explicit exchange and exact-parent merge | DL04 | Sole-operator logical export/import and PJ04 merge; no live sync. Revisit a concrete concurrent-writer or automatic-exchange workflow with conflict/deletion/provenance semantics. |
| I09 | resolved: file/batch admission selected | SC06 | Maintainer selected explicit file/batch observation import on 2026-10-23. No external streaming acquisition; revisit a named producer with defined clock, calibration, ordering and loss semantics. |
| I10 | resolved: owner-based observation | SC05 | Selected gRPC has streaming mechanisms; live resume uses no durable cursor. Own catch-up/gaps from admitted source/progress sequences. |
| I11 | resolved: no inferred sample joins | EV05 | Existing case_measure declares unavailable Criterion membership; retain actual population rather than fabricate it. |

An open inquiry has a required decision outcome, not an obligation to adopt the candidate.
A positive outcome includes its required selected integration and acceptance; a no-adopt outcome
includes supported alternative and observable revisit condition. If it needs a newly selected
functional/platform/rule target, obtain that selection before adding dependent implementation.

## Package state and execution route

ST01–ST04, PJ01–PJ06, SC01–SC06 and EV01–EV05 are complete at targeted acceptance.
LC01–LC05, DL01–DL04 and whole-series assembled/static qualification Q01–Q04 remain separate
remaining scope. Completed companions stay retained dependencies until the aggregate Q04 handoff.

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| ST01 | complete | [51a](51a-storage-foundations-and-project-ownership.md#ST01) | Implemented registry families, proposed ADR-0216, generated contracts; Tested independent installation/strict codecs |
| ST02 | complete | [51a](51a-storage-foundations-and-project-ownership.md#ST02) | Implemented atomic source/project/research operations and Rust/Python/CLI consumers; Tested targeted native and Python journeys |
| ST03 | complete | [51a](51a-storage-foundations-and-project-ownership.md#ST03) | Implemented independent evidence lifecycle, complete import generations and authentic invocation adapter; Tested native roles/lifecycle and public CLI |
| ST04 | complete | [51a](51a-storage-foundations-and-project-ownership.md#ST04) | Implemented issuer/generation fences, exact receipts, original allowance and protected hydration; Tested adversarial/replay/selection controls |
| PJ01 | complete | [51b](51b-projects-revisions-and-temporal-context.md#PJ01) | Implemented stable names/keys, indexed pages, bookmarks and frozen source/physical selections; Tested native/Python/actual executable reopening |
| PJ02 | complete | [51b](51b-projects-revisions-and-temporal-context.md#PJ02) | Implemented immutable raw versions/savepoints, typed exact-base recovery and version diagnostics; Tested malformed/large bodies, restoration, conflicts, cancellation and retained science |
| PJ03 | complete | [51b](51b-projects-revisions-and-temporal-context.md#PJ03) | Implemented compact source/value/structural/unknown comparison and selected detail; Tested known authored edits and explicit physical mismatch |
| PJ04 | complete | [51b](51b-projects-revisions-and-temporal-context.md#PJ04) | Implemented exact three-way decisions, private original-text proof, derived checksum composition and compensating undo; Tested conflicts, replay, ancestry, foreign issuers and unchanged old receipts |
| PJ05 | complete | [51b](51b-projects-revisions-and-temporal-context.md#PJ05) | Implemented UTC/cut/correction selection and protected authored/physical reopen; Tested native history, actual recorded study/lane and reopened native fit receipts retaining original context |
| PJ06 | complete | [51b](51b-projects-revisions-and-temporal-context.md#PJ06) | Implemented bounded passive inspection with existing eligibility owners; Tested no recency/counter/compilation effects and issuer/revision-scoped headers |
| SC01 | complete | [51c](51c-scientific-data-studies-and-results.md#SC01) | Implemented immutable raw/curated/derived references, typed detail and original source/data admission; Tested real Parquet, physical refusal, exact reopen and immutable correction |
| SC02 | complete | [51c](51c-scientific-data-studies-and-results.md#SC02) | Implemented authored calibration/transformed sigma and frozen complete observation selection; Tested actual weights, unknown sigma/exclusion, validity and temporal correction |
| SC03 | complete | [51c](51c-scientific-data-studies-and-results.md#SC03) | Implemented local preview and original eligible fit-cell guarded publication/ancestry; Tested lost reply settlement, moved head, independent covariance and actual downstream prediction |
| SC04 | complete | [51c](51c-scientific-data-studies-and-results.md#SC04) | Implemented indexed browse, native snapshot cohorts and selected protected physical KPI analysis; Tested complete failed/unattempted population, incompatible quantity and unrelated opaque payload isolation |
| SC05 | complete | [51c](51c-scientific-data-studies-and-results.md#SC05) | Implemented bounded passive per-attempt observation and issuer/context cursors; Tested running/terminal catch-up, missing prefix, current resync and exit without science cancellation |
| SC06 | complete | [51c](51c-scientific-data-studies-and-results.md#SC06) | Batch/file admission selected; no external producer or control effects; I09 retains the observable revisit trigger |
| EV01 | complete | [51d](51d-development-and-research-evidence.md#EV01) | Implemented authentic invocation/assessment/diagnostic/host/measurement adapters and declared populations; Tested codec, identity, inventory and explicit coverage controls |
| EV02 | complete | [51d](51d-development-and-research-evidence.md#EV02) | Implemented supplied-only selected Arrow/DataFusion questions and thin CLI; Tested actual executable with unavailable service/extension context and no implicit build or unrelated traversal |
| EV03 | complete | [51d](51d-development-and-research-evidence.md#EV03) | Implemented explicit activated cuts/history, deduplicated source versions and supplied freshness; Tested native generation visibility, offline/index parity, duplicates and source preservation |
| EV04 | complete | [51d](51d-development-and-research-evidence.md#EV04) | Implemented exact linked/standalone diagnostic contexts and protected/copied selected detail; Tested independent outcomes, unrelated link refusal and released detail; LC04 remains separate |
| EV05 | complete | [51d](51d-development-and-research-evidence.md#EV05) | Implemented actual context/sample/censored populations and reported statistics; Tested varied context, true sample membership, unmeasured smoke and unsupported associations; no speed claim |
| LC01 | complete | [51e](51e-reproducibility-lifecycle-and-storage-health.md#LC01) | Selected portable closure and reproduction composition |
| LC02 | complete | [51e](51e-reproducibility-lifecycle-and-storage-health.md#LC02) | Explainable application backup/recovery closure |
| LC03 | complete | [51e](51e-reproducibility-lifecycle-and-storage-health.md#LC03) | Exact retention/protection explanations and explicit release |
| LC04 | complete | [51e](51e-reproducibility-lifecycle-and-storage-health.md#LC04) | Storage-health observations and admitted actions |
| LC05 | complete: no-adopt decision | [51e](51e-reproducibility-lifecycle-and-storage-health.md#LC05) | Complete-artifact confidentiality decision |
| DL01 | complete | [51f](51f-portable-projects-and-delivery-modes.md#DL01) | Versioned logical export with explicit selected closure |
| DL02 | complete | [51f](51f-portable-projects-and-delivery-modes.md#DL02) | Fresh-authority import and independent local/project use |
| DL03 | complete: no-adopt decision | [51f](51f-portable-projects-and-delivery-modes.md#DL03) | Browser/offline capability decision |
| DL04 | complete: no-adopt decision | [51f](51f-portable-projects-and-delivery-modes.md#DL04) | Collaboration/synchronization decision |
| Q01 | complete | [51g](51g-qualification-and-publication.md#Q01) | Final actual-consumer/inquiry reconciliation |
| Q02 | complete | [51g](51g-qualification-and-publication.md#Q02) | Composed representative journeys |
| Q03 | in progress | [51g](51g-qualification-and-publication.md#Q03) | Zero-target scope-end checks |
| Q04 | in progress | [51g](51g-qualification-and-publication.md#Q04) | Enduring publication and post-implementation existing-plan reconciliation |

1. ST01 fixes declaration ownership and short ADR work; ST02's project-head and selected-source
   contracts unblock PJ01/PJ02. ST03's imported-generation contract is independent of project UX.
2. EV01/EV02 can deliver supplied-only investigation independently of service/index availability.
   EV03 consumes ST03 and those authentic adapters, not the reverse.
3. PJ03/PJ04 consume exact branch/base/delta semantics. PJ05/SC01 share only their source/time
   declaration slice; SC02/SC03 then connect the real fitting and source-admission consumers.
4. SC04 uses actual project/study/dataset references and selected result APIs; SC05 composes
   existing progress before any optional native notification. PJ06 does not gate ordinary editing.
5. LC01's closure contract and PJ01/SC01 references unblock DL01/DL02. Protection/recovery work
   uses current owners throughout, not a late retention retrofit. LC04/EV04 can proceed independently.
6. Bounded inquiries LC05/DL03/DL04/SC06 close their declared questions. Their optional adoption
   must not silently add new product or platform promises to the base delivery.
7. Q01/Q02 reconcile actual consumers, then Q03 runs scope-end checks once and repairs failures.
   Q04 publishes enduring meaning and only then performs the requested later reconciliation of
   existing plan documents. The authoring engagement does not execute Q04.

These are capability dependencies, not whole-document barriers. Shared registry, generated
queries/contracts, runtime/Python facades and metadata configuration need coordinated writers;
parallelize ready bounded work. Root owns final design, integration and acceptance.

## Cross-plan boundaries and verification

Consume currently implemented exact-source/result/progress, registry/tooling, module and resource
contracts directly. Do not make all of Plan49f–h, Plan32 or Plan40e prerequisites. Plan49g TOOL08's
strict native edge-work bound is not established by this series' output/deadline bounds; return
that limitation explicitly rather than claiming a new graph limit. Preserve canceled campaign
and historical test conditions. No old plan status is copied into this coordinator.

**Proposed verification:** new mechanisms get targeted tests and compile checks in the same
change, with force-validation for actual Arrow-consuming correctness closures. Migrate callers
and delete replaced paths when those tests pass; retain no compatibility backend or second
scientific execution path. Existing supported local studies and emergency host readers remain
independent capabilities with their actual owners, not fallbacks for failed durable writes.

At scope end, 51g owns representative integration/Python/native journeys, relevant hygiene,
governance/docs/native/powerset checks and zero-target repairs. Evidence labels distinguish
Proposed, Interface-checked, Implemented, Tested and Measured. No mandatory broad benchmark,
upstream DST duplication, new memory governor or repeated per-package hygiene is introduced.
Quantitative benefit remains a hypothesis until named measurements establish it.

## Authoring baseline

The target choices above were made from static evidence, not deferred to execution. At authoring,
all delivery packages were planned; bounded future capability decisions remain visible. No production
implementation or product qualification occurred during authoring. Relevant sources and exact
capability conditions are carried into the companions. Existing plans/index and supplied library
documents are unchanged. Implementation requires subsequent maintainer authorization.

## Execution handoff

**Implemented and Tested:** 51b's PJ01–PJ06 are complete at the targeted boundary documented
in [51b verification](51b-projects-revisions-and-temporal-context.md#verification-and-handoff).
Whole-series assembled/static qualification remains Q01–Q04. Enduring project/history/preparation
contracts are published at blueprint §14.3/§20.6 with proposed ADR-0217. 51a/51b stay retained
dependencies until Q04. That earlier handoff did not authorize 51c–51g execution or close older
plan series; the subsequent 51c/51d authorization and completion are recorded below.


## 51c/51d execution outcome

**Implemented:** the maintainer authorized SC01–SC06 and EV01–EV05 on 2026-10-23; all are
complete at their targeted functional boundary. The native research/workbench, public
Rust/Python/CLI and supplied-only/indexed evidence consumers are integrated into main. SC06
selects explicit file/batch admission and no streaming/control producer. I09 retains the
observable revisit trigger. Proposed ADR-0218/0219 and blueprint §19.4/§24.1 carry enduring
contracts; the substrate guide carries commands and limits.

**Tested:** [51c verification](51c-scientific-data-studies-and-results.md#verification-and-handoff)
owns the shared 70-pass/zero-fail native selection, actual calibrated LM/publication/downstream
prediction, protected cohorts/observer and two passing public Python journeys.
[51d verification](51d-development-and-research-evidence.md#verification-and-evolution) adds
48 passing host controls and three actual offline-executable controls. Correctness builds,
full generator publication, family and environment checks passed against the zero baseline.
Earlier failed runs were corrected at their source owners; their failures are not a quality
baseline. All three clean implementation worktrees/branches were merged and removed, preserving
qualified main repairs and the user's untracked library reference. Discrete commits retain the
implementation and corrections.

**Not run:** whole-series assembled qualification, scope-end hygiene/manual checks and performance
campaigns remain Q01–Q04. Mechanical ADR/formatting and documentation publication passed;
the format-only Ruff invocation reported 66 diagnostics against zero, retained as Q03 repair
work rather than a clean quality claim. **Measured:** no speed/capacity improvement is claimed. LC01–LC05,
DL01–DL04 and Q01–Q04 remain at their existing owners; EV04 does not complete LC04's future
storage-health collector. Keep completed 51a–51d dependencies until aggregate Q04 retirement.
Older plan scope and formal ADR acceptance are unchanged.


## 51e–51g authorized execution checkpoint

The maintainer authorized the complete remaining series on 2026-10-23. LC01–LC04 and
DL01–DL02 are implemented and targeted-tested through original source, artifact, project, recovery
and evidence owners under proposed ADR-0220. Their companions own the functional Outcomes. Q01/Q02 are complete at the composed conditions in the 51g Outcome; Q03/Q04 remain in progress.
The 16-concurrent-test startup failures remain explicitly unqualified; their exact affected
selection passes at four concurrent tests. No generated-data migration or historical preservation
campaign is required.

The maintainer selected these bounded inquiry outcomes during execution preparation:

- **I07 / LC05:** no application encryption capability. Current local filesystem observation
  is an ext4 mount on `/dev/nvme0n1p2`; that observation does not establish whether underlying
  volume encryption is configured. Inventory covers catalogue/credentials, sealed and staged
  payloads, authored source and measurements, logical packs, diagnostics/traces/crash dumps,
  temporary files, key material and application backups. Current support is local filesystem
  access control and explicit operator custody; complete-artifact confidentiality and encrypted
  exchange are not claimed. Revisit when a concrete confidential external exchange or at-rest
  protection requirement is selected. Any positive selection must cover the entire inventory,
  key custody and recovery, rather than KV-only TDE.
- **I06 / DL03:** native tools and independent offline logical-pack inspection are selected;
  no browser UI, browser persistence or browser numerical execution is adopted. Revisit when
  a named user operation needs browser participation, then assess a logical viewer/editor
  before any new browser storage topology.
- **I08 / DL04:** explicit logical export/import and ordinary exact-parent project merge are
  selected for the sole operator; no live synchronization or multiwriter service is adopted.
  Revisit when a concrete concurrent-writer or automatic-exchange workflow is required,
  including its identity, conflict, divergence, deletion and provenance semantics.

These decisions close their bounded questions; delivery and qualification of the selected
native/offline alternative remain with LC01/DL01/DL02/Q02. They do not establish host
confidentiality, distributed isolation or supported browser persistence.


## Outcome — series integration, 2026-10-23

**Implemented:** all selected ST/PJ/SC/EV/LC/DL functionality is integrated through the original
scientific/source/project/result/artifact owners and public Rust/Python/CLI consumers. The
O01–O21 and I01–I11 tables retain their single dispositions, including selected no-adopt outcomes
and observable revisit triggers. Enduring contracts are at blueprint §14.3/§19.4/§20.6/§24.1;
operator workflows are in the substrate, diagnostics and environment guides. Proposed
ADR-0216–0220 decision status remains unchanged.

**Tested:** [51g's assembled Outcome](51g-qualification-and-publication.md#verification-and-outcome--assembled-execution-2026-10-23)
owns actual commands, conditions, initial failures and repaired reruns. Its 100-case Rust
selection combines 73 passes at 16 concurrent tests with the exact affected 27 passing at four;
it is not a monolithic pass or 16-concurrent-startup qualification. Fourteen native-Python
component cases, four affected frozen-context units, selected host/offline executable controls,
actual calibrated numerical fit/import/edit/study and nonempty disposable recovery establish
only their named scope. Final feature-matrix and publication work remains Q03/Q04 until the
same owner records its actual result.

**Mistakes corrected:** initial import visibility and descriptive traversal crossed ownership
boundaries; the original pending source checkpoint and narrow owner projections now govern
availability and selected copying. Actual nonempty recovery exposed pinned SurrealQL projection,
maintenance-return/predecessor and allocation-before-effects issues, repaired at their original
owners. The health collector now separates service identity from bounded metric availability.
The scope-end matrix also exposed an omitted taskdump build configuration and temporary
merge markers read during a running check; the original adapter and exact affected reruns own
those repairs rather than a changed zero baseline.

**Deliberate deviations:** retained sealed payloads, native/offline explicit exchange and local
filesystem custody are the selected target. No browser persistence, application crypto, live
synchronization, second numerical/backend route or historical-data migration is adopted.
Quantitative speed/capacity, confidentiality and unexercised platform/whole-product scientific
claims are not established. Plan32 WC05/WC06, Plan40e and Plan49f–h/TOOL08 retain their independent
scope; no canceled campaign is reopened. All implementation worktrees were merged and removed,
with final ancestry merges verified to preserve the qualified main tree exactly.
