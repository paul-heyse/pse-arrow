---
title: "Plan 51: Engineering workspaces and evidence"
status: draft
date: 2026-10-22
adrs: []
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
preserved, and storage/payload choices settled during authoring. Plan creation is authorized;
production implementation is not authorized by these documents alone.

Existing plan documents and their index are deliberately unchanged. This coordinator records
dependencies and the post-implementation reconciliation obligation; it does not reopen Plan50
or close independent Plans32/40/49. Companions own design and package explanations; only this
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
time design selection, not implementation or a blanket waiver. A newly discovered actual change
must be presented with its consequence and confirmed before the dependent target is revised.

| Review item | Plan disposition | Route / consequence |
| --- | --- | --- |
| RC01 | Selected new functional scope; no authority replacement | ST01/EV01 declare derived evidence semantics. Completed Plan50 stays complete and unchanged. |
| RC02 | Not activated: unified native payload replacement not selected | Preserve blueprint §20.1–§20.4 and ADR-0187 target. A later replacement needs its own explicit decision. |
| RC03 | Not activated: canonical topology retained | Preserve D10, blueprint §20.6 and ADR-0164/0214 target. ST02 integrates new consumers without changing their authority. |
| RC04 | Not activated: invocation/resource registry retained | Preserve blueprint §24.1/ADR-0174. EV readers/imports remain independently disposable projections. |
| RC05 | No expanded whole-workspace persistence selected | PJ06 exposes actual existing eligibility. Native handles, providers and Salsa owner state are not transferred. |
| RC06 | Apply existing declaration/decision routes | ST01 schedules short ADRs for new relation families/public meanings where required, then registry/codegen. Host observations retain their original Python owners. No foundational identity/Python-boundary waiver is selected. |
| RC07 | No new browser/security/synchronization guarantee selected | LC05/DL03–DL04 settle bounded capability decisions. Any new platform/authority guarantee must receive explicit selection and required decision work before implementation. |

## Opportunity dispositions

Scheduled denotes proposed implementation or a required bounded inquiry, not delivered behavior.
Resolved here can close an authoring choice without claiming product implementation.

| Opportunity | Disposition | Work owner | Required behavior or disposition |
| --- | --- | --- | --- |
| O01 | scheduled | PJ01–PJ04 | Persistent project/branch navigation, exact comparison and explicit merge/publication |
| O02 | scheduled | PJ05/SC02 | Effective-time and recorded-cut context without changing exact run receipts |
| O03 | scheduled | PJ02/PJ04 | Recoverable raw drafts, short commands, explicit admission and conflicting-edit behavior |
| O04 | scheduled | SC04/SC05 | Indexed study selection and protected original result access |
| O05 | scheduled | SC01–SC03 | Raw measurement → calibration → admitted fit → parameter revision → downstream study |
| O06 | scheduled | EV01–EV03 | Authentic selected test facts, honest denominators and protected artifact retrieval |
| O07 | scheduled | EV04/LC04 | Real diagnostic relationships and separate tool/target/scientific outcomes |
| O08 | scheduled | EV05 | Comparable measurement cohorts with actual sample membership and unknown conditions |
| O09 | scheduled | LC01/DL01–DL02 | Exact selected closure and explicit new supported reproduction/import |
| O10 | resolved: no additional native working store | ST02/PJ02 | Existing immutable documents/RPDS plus catalogue transactions; revisit a concrete unmet operation |
| O11 | scheduled | PJ06 | Discover existing compatible preparation artifacts and truthful reuse explanations |
| O12 | scheduled | ST02/SC04/EV02 | Operation-shaped native indexes and selected columnar analytics; no universal query framework |
| O13 | scheduled | ST02–ST04 | Coherent atomic metadata operations without replacing scientific or host lifecycle owners |
| O14 | scheduled | LC02/LC03 | Application-closure recovery and explicit retained-resource explanations |
| O15 | scheduled | LC04 | Useful MX/native storage health joined only to real owner evidence |
| O16 | resolved: native payload unification not selected | 51a design decision | Preserve sealed files; logical exchange solves handoff without a second payload backend |
| O17 | resolved: shared service target selected | ST02/ST03 | New consumers integrate with the chosen deployment; no execution-stage topology campaign |
| O18 | scheduled: bounded inquiry | DL03 | Resolve offline/browser participation; no ready KV1.0 OPFS Tree or browser solver claim |
| O19 | scheduled: bounded inquiry | LC05 | Decide a complete-artifact confidentiality capability; no default new crypto rollout |
| O20 | scheduled | DL01/DL02/DL04 | Logical editable project exchange; collaboration implementation conditional on explicit selection |
| O21 | scheduled | SC05/SC06 | Read-only observation/reconnect now; separately bounded external-acquisition decision |

## Inquiry outcomes and remaining questions

| Inquiry | State | Owner | Decision / completion evidence |
| --- | --- | --- | --- |
| I01 | resolved: shared topology | 51a | Actual independent clients and generated invariant consumers favor existing service; embedding remains a concrete alternative, not an unsupported feature claim. |
| I02 | resolved: sealed bulk payloads | 51a | Seekable protected FileReader path retained; no mmap premise. KV owned-value and retention tradeoffs do not justify replacement for this scope. |
| I03 | resolved: no extra native workspace | 51a/51b | No demonstrated operation gap beyond immutable sources and catalogue transactions. |
| I04 | resolved: explicit temporal records | PJ05 | Use exact source/context revisions with effective intervals and correction ancestry; no native memory-engine time travel. |
| I05 | resolved: logical exchange | LC01/DL01 | Portable manifest/copy closure, not engine directory identity or live native state. |
| I06 | open: bounded browser decision | DL03 | Check actual pinned worker/Tree/persistence support and a concrete offline inspection/edit consumer; adopt with explicit supported scope or no-adopt with a new observable trigger. |
| I07 | open: bounded confidentiality decision | LC05 | Compare full payload/metadata/key coverage against the selected exchange/local-use threat; no partial KV-only protection claim. |
| I08 | open: bounded collaboration decision | DL04 | Decide whether any multi-writer/server synchronization is needed beyond explicit logical import and objectwise merge. No CRDT/replication assumption. |
| I09 | open: bounded acquisition decision | SC06 | Identify a real external observation producer and clock/calibration contract; no native real-time control selected. |
| I10 | resolved: owner-based observation | SC05 | Selected gRPC has streaming mechanisms; live resume uses no durable cursor. Own catch-up/gaps from admitted source/progress sequences. |
| I11 | resolved: no inferred sample joins | EV05 | Existing case_measure declares unavailable Criterion membership; retain actual population rather than fabricate it. |

An open inquiry has a required decision outcome, not an obligation to adopt the candidate.
A positive outcome includes its required selected integration and acceptance; a no-adopt outcome
includes supported alternative and observable revisit condition. If it needs a newly selected
functional/platform/rule target, obtain that selection before adding dependent implementation.

## Package state and execution route

All rows below are proposed work; authoring and mechanical document checks establish none of them.

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| ST01 | planned | [51a](51a-storage-foundations-and-project-ownership.md#ST01) | Owned declarations and applicable short decision records |
| ST02 | planned | [51a](51a-storage-foundations-and-project-ownership.md#ST02) | Atomic project/research repository operations and real consumers |
| ST03 | planned | [51a](51a-storage-foundations-and-project-ownership.md#ST03) | Disposable derived evidence database and generation publication |
| ST04 | planned | [51a](51a-storage-foundations-and-project-ownership.md#ST04) | Shared selection, error/effect and lifetime behavior |
| PJ01 | planned | [51b](51b-projects-revisions-and-temporal-context.md#PJ01) | Project and branch ownership/navigation |
| PJ02 | planned | [51b](51b-projects-revisions-and-temporal-context.md#PJ02) | Durable raw drafts and local admission |
| PJ03 | planned | [51b](51b-projects-revisions-and-temporal-context.md#PJ03) | Source/value/structural comparison |
| PJ04 | planned | [51b](51b-projects-revisions-and-temporal-context.md#PJ04) | Explicit publication, objectwise merge and undo |
| PJ05 | planned | [51b](51b-projects-revisions-and-temporal-context.md#PJ05) | Historical context selectors and corrections |
| PJ06 | planned | [51b](51b-projects-revisions-and-temporal-context.md#PJ06) | Compatible preparation discovery |
| SC01 | planned | [51c](51c-scientific-data-studies-and-results.md#SC01) | Dataset/measurement versions and admitted source bridge |
| SC02 | planned | [51c](51c-scientific-data-studies-and-results.md#SC02) | Explicit calibration and observation selection |
| SC03 | planned | [51c](51c-scientific-data-studies-and-results.md#SC03) | Qualified fit-to-parameter-source publication |
| SC04 | planned | [51c](51c-scientific-data-studies-and-results.md#SC04) | Cohort/result query and lazy detail consumer |
| SC05 | planned | [51c](51c-scientific-data-studies-and-results.md#SC05) | Study progress observer and reconnect |
| SC06 | planned | [51c](51c-scientific-data-studies-and-results.md#SC06) | Explicit external-acquisition disposition |
| EV01 | planned | [51d](51d-development-and-research-evidence.md#EV01) | Authentic typed source adapters and snapshot coverage |
| EV02 | planned | [51d](51d-development-and-research-evidence.md#EV02) | Supplied-only offline columnar query |
| EV03 | planned | [51d](51d-development-and-research-evidence.md#EV03) | Idempotent persistent index import/query |
| EV04 | planned | [51d](51d-development-and-research-evidence.md#EV04) | Linked and standalone diagnostics with protected details |
| EV05 | planned | [51d](51d-development-and-research-evidence.md#EV05) | Comparable performance populations |
| LC01 | planned | [51e](51e-reproducibility-lifecycle-and-storage-health.md#LC01) | Selected portable closure and reproduction composition |
| LC02 | planned | [51e](51e-reproducibility-lifecycle-and-storage-health.md#LC02) | Explainable application backup/recovery closure |
| LC03 | planned | [51e](51e-reproducibility-lifecycle-and-storage-health.md#LC03) | Exact retention/protection explanations and explicit release |
| LC04 | planned | [51e](51e-reproducibility-lifecycle-and-storage-health.md#LC04) | Storage-health observations and admitted actions |
| LC05 | planned | [51e](51e-reproducibility-lifecycle-and-storage-health.md#LC05) | Complete-artifact confidentiality decision |
| DL01 | planned | [51f](51f-portable-projects-and-delivery-modes.md#DL01) | Versioned logical export with explicit selected closure |
| DL02 | planned | [51f](51f-portable-projects-and-delivery-modes.md#DL02) | Fresh-authority import and independent local/project use |
| DL03 | planned | [51f](51f-portable-projects-and-delivery-modes.md#DL03) | Browser/offline capability decision |
| DL04 | planned | [51f](51f-portable-projects-and-delivery-modes.md#DL04) | Collaboration/synchronization decision |
| Q01 | planned | [51g](51g-qualification-and-publication.md#Q01) | Final actual-consumer/inquiry reconciliation |
| Q02 | planned | [51g](51g-qualification-and-publication.md#Q02) | Composed representative journeys |
| Q03 | planned | [51g](51g-qualification-and-publication.md#Q03) | Zero-target scope-end checks |
| Q04 | planned | [51g](51g-qualification-and-publication.md#Q04) | Enduring publication and post-implementation existing-plan reconciliation |

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

## Authoring checkpoint

The target choices above are made from static evidence, not deferred to execution. All delivery
packages remain planned; bounded future capability decisions remain visible. No production
implementation or product qualification occurred during authoring. Relevant sources and exact
capability conditions are carried into the companions. Existing plans/index and supplied library
documents are unchanged. Implementation requires subsequent maintainer authorization.
