---
title: "Plan58: Rust-owned system"
status: in-progress
date: 2026-10-28
adrs: []
review_sources: [docs/functional_reviews/rust-first-system-functional-opportunities-2026-10-28.md, docs/plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md, docs/functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md]
doc_role: plan
doc_owner: docs/plans/58-rust-owned-system.md
doc_topics: [current-work, architecture, design, libraries, validation]
---

# Rust-owned system

## Purpose and authorization

**Implemented:** the selected A–G functional scope delivers Rust-owned product, host and development capabilities. [The current Outcome](#remaining-bg-outcome--2026-10-29) records acceptance and exclusions; 58h remains unexecuted. This is a new design and delivery series, not a continuation of Plan57's packages. The operator authorized authoring on 2026-10-28 and explicitly superseded all Plan57 execution scope. That original authoring authorization was followed by the explicit execution selections below; creating a plan alone grants no execution authority.

The [Rust-first functional review](../functional_reviews/rust-first-system-functional-opportunities-2026-10-28.md)
and the [hard-cutover improvement review](../plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md)
supply this target. The latter incorporates the colleague's review/evidence with explicit adapted
and declined proposals; its eighteen-cluster and Wave0–5 dispositions remain the source rationale.
The initial amendment baseline was `c3b7f136d`, with source assessed at `bd1c4891e`.
The 2026-10-29 database-boundary amendment consumes review commit `a6a144bce` and
the implemented native checkpoints below; those earlier source identities are historical bases. User-supplied untracked library/external-review inputs remain untouched. Core 3.4,
efficiency heuristics 1.0, test assessment 1.2 and process-simulator 1.5 apply.
Initial document amendments were authorized on 2026-10-28; subsequent selected execution
is recorded below. The operator authorized the database-boundary document amendment on
2026-10-29; that amendment grants no additional production execution.

The latest operator instruction overrides both the review's mandatory-CLI recommendation and the earlier planning answer: **CLIs are optional conveniences**, not completion requirements. No broad command framework, universal operation catalogue or agent execution engine is selected. RMCP remains conditional. PyO3 gets a future interface seam; implementation is deferred. Production robustness and simplicity outrank peripheral interface parity.

## Target and integral foundation assessment

The delivered product is a Rust library/application system. Typed authored definitions and context bindings feed existing scientific preparation and class-specific execution owners. Direct Rust consumers use typed operations, results and explicit lifetimes; serialization occurs at actual external boundaries. No Rust workflow calls Python to acquire ordinary product or host capabilities. Existing useful executables may remain thin composition roots; their command surface cannot dictate domain APIs or pull development dependencies into production.

Four capabilities have different prerequisites: pure/local scientific execution; connection/authentication/bounded durable control; protected retained-data reading; and supervised native work. Compact reads and result export must not construct mathematics/provider services. Local pure work needs neither mandatory database nor daemon. Durable work uses canonical SurrealDB/MX as its governing execution model: definitions,
dependencies, requests, realized points, attempts, assignments, transitions and result references
are authoritative data used for progression and later repeated analysis. Rust workers perform
scientific effects; the database does not own host process death or native thread joins.
58d owns this model and minimal schema evolution together, avoiding divided transition/version
policy. A owns scientific callers, C physical execution, B service mechanics and E payloads.

The original review inspected Rust authoring/compiler/runtime, maintained Symbolica/faer, native joined pools, canonical command/receipt policy, result snapshots/providers and Rust generation. It identified Python runtime facade, control transport, MCP workers, environment/compiler wrappers and test-resource boundaries; subsequent checkpoints record the selected host/compiler and asyncio-wait retirements, while remaining consumers keep their owning packages. Their behavior is assessed by consumer need; no ledger, receipt, scan, wrapper or validation pass is preserved merely because it exists.

Preserve physical meaning, original-space outcome assessment, supported square/root and optimization modes, dynamics/events, fitting/sensitivity, initialization/recycles, studies/sampling, diagnostics and project/source/result history within current supported limits. This does not promise unimplemented IDAES families or broaden numerical claims. Python reference tools may remain isolated independent oracles. The unresolved historical SIGSEGV and arithmetic limits remain evidence to understand where relevant, not inherited implementation packages or proof of language causation.

### Physical organization and chosen mechanisms

| Responsibility | Selected target and limit |
| --- | --- |
| Native product | Existing domain/runtime owners with independently constructible scientific, control and retained-data capabilities; no frontend-specific policy. |
| Host/bootstrap | The implemented small Rust tool/library root independent of Arrow, math, native solvers, database and generated product schemas. New crate and dependency direction receive ADR/design review before implementation. A tiny shell seed may locate/build/install it, with no duplicate host policy. |
| Linux control | Typed zbus/systemd user-manager control implements the selected ordinary foreground replacement; broader B process/service obligations remain open. Used interfaces must work on host systemd255. Select Tokio-backed zbus, not an extra runtime. No permanent parallel Python controller or administrative-credential fallback. |
| Runtime lifecycle | Existing Tokio/Futures/Rayon ownership and explicit native OS joins remain the foundation. Compare hierarchical shutdown only for long-lived application subsystems before selecting it; do not insert a second scheduler or treat task abort as native release. |
| Canonical state | Effective patched SurrealDB3.3.4/MX1 SSI and synchronous persistence initially retained. Bounded store-side progression/recheck/claim, native schema/migration and narrow actor command/connection ownership; no mandatory new RPC, embedded backend or module-hosted science. |
| Data | Protected Arrow IPC snapshots and configured DataFusion providers, selective typed metadata, existing charged Bytes custody. Flight/remote objects/Parquet derivatives require their actual consumer. |
| Consumers | Native Rust first; useful low-cost executable adapters optional. Delete Python MCP without feature parity. No RMCP or PyO3 delivery dependency. |

Library research informs rather than follows the interfaces. The review's full overview research and focused exact-source checks are reused. New lifecycle overviews were read end to end and checked through Context7/primary sources during preparation. The initial zbus6.1 assessment required moving Tokio1.53.2 to1.53.3; B's implemented host checkpoint records the integrated dependency move. Proxy generation targeting a newer systemd is not evidence of runtime support. sd-notify0.5's send/absent-socket success is not readiness observation; abstract sockets/barrier and early environment mutation require explicit disposition. tokio-graceful-shutdown0.20 abort/timeout cannot establish native completion. These are bounded integration questions, not an excuse for a fallback framework.

## Confirmed target/rule directions

| Direction | Operator disposition on 2026-10-28 | Route before dependent implementation |
| --- | --- | --- |
| Rust-owned product; Python no mandatory authoring/result boundary | Explicitly selected in this engagement | T01: D12/blueprint §21 boundary ADR and required design review; supersede affected immutable decisions rather than edit them. |
| Python FastMCP may retire; MCP optional; CLI not required | Explicitly selected, with later CLI correction governing | T01/G: revise single-endpoint/tooling/interface requirements where removal changes authority; retain useful effect semantics independently. |
| First-party host/developer responsibilities may move to Rust | Explicit end-to-end scope selection | T01/B/F: host identity, bootstrap/environment and governance owner changes through applicable ADR/design/instruction route. |
| Canonical data model governs durable work | Explicitly selected; local pure computation remains direct | T01/D03: affected execution/schema authority through ADR/design route before dependent implementation. |
| Durable migration infrastructure now; development reconstruction instead of exhaustive recovery gates | Explicit balanced durability direction; no existing-data reset | T01/D05/H: supported predecessor/successor, write fences and representative migration now; persistent-production release qualification later. |
| Typed fixture construction and ownership replace global permission/terminal protocols | Selected simplification in the authorized amendment target | T01/B/C/F: revise fixture/resource instructions and affected contracts; retain real physical release and shared-service uncertainty. |
| Bespoke build observation and derivative signals retire | Explicitly selected; scientific calculation observation retained | T01/B/F: amend observation/consumer requirements, remove build alerts/queries with producers. |
| Cache-service death alone does not invalidate a valid compiler result | Explicit answer: “Keep compiler result”, 2026-10-28 | T01/B/F: replace continuity requirement and its watcher/tests; preserve compiler failures, actual inputs and owned-process cleanup; reconcile Plan54/current environment contracts. |
| New host root and library choices | Proposed technical realization within authorized plan creation | T01: crate addition/dependency direction review before code; ordinary exact dependency changes need no separate dependency ADR. |

No blanket weakening of physical release, scientific validity, protected data or unknown-effect semantics is authorized. Any newly discovered rule change needs its specific operator disposition before dependent work, per create-plan. This series is not itself formal ADR acceptance.

## Documents and responsibility boundaries

| Companion | Owns |
| --- | --- |
| [58a](58a-native-product-and-scientific-workflows.md) | Rust product/scientific capabilities and preparation; incremental/adaptive extensions |
| [58b](58b-bootstrap-and-host-control.md) | Cold bootstrap, environment, typed Linux process/service control |
| [58c](58c-runtime-lifecycles.md) | Native task/resource composition, stop/drain/failure policy |
| [58d](58d-canonical-control-and-connectivity.md) | Canonical durable execution, schema evolution/write admission and independent actor control/connectivity |
| [58e](58e-retained-data-and-delivery.md) | Protected data, relational access and delivery |
| [58f](58f-development-and-assurance.md) | Retained first-party development/assessment/configuration workflows |
| [58g](58g-boundary-retirement-and-optional-consumers.md) | Python/MCP deletion and future optional consumers |
| [58h](58h-qualification-and-publication.md) | Joined acceptance, source transition and enduring publication |

Companions own package state. This coordinator owns opportunity/improvement disposition and shared decisions; no second package ledger. Letters are reading groups, not serial execution lanes.

## Foundation state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| T01 | done | Coordinator | Required decision/design route and shared capability/dependency contracts established before contract-changing implementation |
| T02 | done | Coordinator | Source/consumer inventory reconciled to this target; exact next implementation slice agreed without inherited Plan57 scope |

<a id="t01"></a>
### T01 — Establish decision and integration authority

Prepare the Rust boundary/host-root ADR and affected architecture/instruction amendments, with required independent design assessment under the binding. The functional review/focused authoring assessment does not substitute for that review. If a new diagnostic design engagement is needed, refresh its applicable audit through that route; no broad audit was initiated by plan authoring. Include concrete consumers, crate/dependency closure, typed operations, external effect identity, host release and failure boundaries. Include accepted guarantee reductions above, schema migration/open-handle and in-flight publication fencing, and fixture construction versus real external uncertainty. Reconcile affected Plan53/54 interfaces without absorbing their unrelated scope. Review native product, bootstrap and database interactions together before consumer work depends on them. Acceptance is a coherent scoped decision and explicit remaining limits, not an invented runtime pass. Formal ADR status changes only by the prescribed PR route.

<a id="t02"></a>
### T02 — Reconcile current source and first executable slice

Map retained user/scientific outcomes to actual Rust owners and frontend-only behavior slated for deletion. Inspect Rust→Python inversions in product, fixture, compiler and generation paths. Decide retained third-party/oracle use separately from first-party Python policy. Freeze only the first working contracts needed by a coherent production/bootstrap slice; do not enumerate a universal dispatcher. Implementers may make local choices but may not select competing lifecycle or authority models accidentally. Select integrated cutover sets by retained outcome: native owner, actual callers, configuration,
generation, targeted tests and obsolete implementation deletion. Temporary broken development
paths during a set are permitted but not accepted delivery. No FastMCP proxy, compatibility DTO,
Python admission fallback or second catalogue is built to hide that state. A set is not a whole
repository mega-commit; independent ready sets can proceed separately. Verify Wave0 candidates
against actual consumers (F/G), without treating a live consumer as a preservation requirement.
A01/B01/C01/D01/E01/F01/G01 consume this result as relevant; investigations can proceed before final integration authority when they do not mutate production.

## Opportunity dispositions

References O01–O15 mean anchors in the source functional review. Resolved rows refer to selected functional acceptance in the companion Outcomes and the current shared Outcome; H qualification remains separately scheduled. Conditional/deferred opportunities have a concrete revisit and do not block core completion.

| Opportunity | Disposition | Work owner | Required evidence or trigger |
| --- | --- | --- | --- |
| O01 | resolved | [58a Outcome](58a-native-product-and-scientific-workflows.md#outcome-recorded-after-implementation) | Supported native construction/scientific/source/study callers pass; full assembled/reference qualification remains H-owned. |
| O02 | resolved | 58f | Retained developer capabilities independent of Python/MCP; CLI facade optional |
| O03 | resolved | 58g | Retire Python MCP; reduced RMCP deferred until a useful actual-host consumer |
| O04 | deferred | 58g | Define seam now; implement only for a concrete Python consumer |
| O05 | resolved | 58b | Cold-start and exact original process/resource settlement |
| O06 | resolved | 58c | Native cancellation/destruction/join semantics and concurrent composition |
| O07 | resolved | 58d | Connection-only control and original durable command reconciliation |
| O08 | resolved | 58e | Protected result access without mathematical construction |
| O09 | resolved | 58f | Shared typed Rust contracts/diagnostics with responsibility-owned generation |
| O10 | resolved | [58a Outcome](58a-native-product-and-scientific-workflows.md#outcome-recorded-after-implementation) | Existing library-owned preparation/arithmetic and class-specific limits preserved through native callers; optional new engines/diagnostics remain conditional. |
| O11 | scheduled | 58h | Owner tests, original consumer journeys and independent oracle isolation |
| O12 | resolved | [58a Outcome](58a-native-product-and-scientific-workflows.md#outcome-recorded-after-implementation) | Native value/structure/delete/repair and exact-parent source/history callers pass; disk persistence remains conditional on a concrete benefit. |
| O13 | deferred | 58a | Bounded comparison when adaptive external batch study needs more than current actor |
| O14 | deferred | [58f F06](58f-development-and-assurance.md#f06) | Discrete explanation consumer beyond ordinary validation justifies Z3 |
| O15 | deferred | 58e | Actual remote interoperable batch/artifact consumer justifies Flight or signing |

## SurrealDB boundary dispositions

These O01–O10 references belong exclusively to the
[2026-10-29 database-boundary review](../functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md),
not the earlier review's Opportunity dispositions above. This table alone owns their disposition;
companions own implementation packages. Resolved rows record the selected functional implementation and explicit investigation decisions, supported by the B/D/E Outcomes. They do not qualify H campaigns or future production recovery. Conditional mechanisms inside a scheduled opportunity are not mandatory adoption.

| Review opportunity | Disposition | Primary work owner | Evidence or revisit trigger |
| --- | --- | --- | --- |
| [O01](../functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md#O01) | resolved | 58d D03 | Complete bounded progression/action/premise handoff; competing workers, sparse continuation, stale preparation and fenced acceptance. |
| [O02](../functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md#O02) | resolved | 58d D03 | Co-design access paths and summaries using existing references/inline graph mechanisms; decide justified projections/events/views rather than mandate new graph storage. |
| [O03](../functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md#O03) | resolved | 58d D03 | Canonical catch-up and observer contract; select events/live/feed only for a named consumer with actor, ordering, retry and finality evidence. |
| [O04](../functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md#O04) | resolved | 58d D01 | Bounded exact-release SDK/API/gRPC comparison: delegated duty, retained gap and removed code; no unconditional client replacement. |
| [O05](../functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md#O05) | resolved | [58a A05 Outcome](58a-native-product-and-scientific-workflows.md#outcome-recorded-after-implementation) | Native repeated analysis preserves original revisions/context/realized points/attempts through replacement and later correction; D03/E02/H retain their broader obligations. |
| [O06](../functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md#O06) | resolved | 58d D03 | Retain patched persistent MX/SSI; KV/embedded alternatives remain conditional on a concrete history, capacity, encryption or deployment need and complete compatibility/migration evidence. |
| [O07](../functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md#O07) | resolved | 58d D03 | Retain one stateless shared readiness policy; broader module effects require permission, parent rollback, mutability, complete-output and resource evidence. |
| [O08](../functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md#O08) | resolved | 58e E01 | Independent protected reads over bounded inline IPC/sealed payloads and canonical manifests; buckets/payload relocation require a concrete lifecycle benefit. |
| [O09](../functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md#O09) | deferred | 58f F05 | Named shared multi-key working collection whose transactional/query contract removes more machinery than it introduces; discovery can identify a valuable future consumer, but no second authority follows automatically. |
| [O10](../functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md#O10) | resolved | 58d D05 | Native version/fence/migration infrastructure, stale handles and original in-flight settlement; H owns later non-regenerable/shared deployment recovery qualification. |

### Database-boundary target and integration order

**Proposed:** one canonical authority on patched persistent MX, complete bounded canonical
transitions, native scientific execution and physical custody, and protected IPC data with
canonical manifests. State authority, storage backend, execution placement and connectivity
are separate choices. [58d's shared contract](58d-canonical-control-and-connectivity.md#state-placement-and-shared-contract)
owns the detailed placement and transaction handoff; A owns scientific interpretation,
B process mechanics, C actual preparation/admission/drain, and E protected data custody.
Temporary shared state may justify database semantics without needing crash recovery, but
single-owner numerical working state remains native. No new rule reduction is selected.

Settle D's bounded action/premise, actor/connection and version/write-fence contracts first,
with C's native preparation/launch-capacity handoff and E's payload acceptance boundary.
Those early contracts permit parallel design; consumer cutover requires the actual working
capability. Do not encode a whole-D03/whole-C03 dependency cycle: the preparation interface
is an early shared slice, while integrated claim/execution tests exercise both implementations.
D05's minimal schema-ready/version capability and B's usable server establishment are real
prerequisites for database consumers, not source-level declarations of readiness.

D03's first integrated delivery co-designs indexed bounded progression with the worker
consumer and deletion of subsumed authoritative choreography. D01's connection comparison
settles supplier coverage and retained custody gaps before that transport's consumers move.
A05/D03/E02 then demonstrate repeated analysis against the same recorded lineage and protected
outputs. G/F participate in each cut, not a final cleanup wave. H joins owner-local cases
at scope end. One canonical transition does not imply that every loop, CandidateGroup,
request identity, resource lease or file-publication protocol can disappear.

The review's scenarios are developed at their native owners and referenced by H; they are
not a second scenario registry here. D's [decision investigations](58d-canonical-control-and-connectivity.md#decision-investigations)
record alternatives, evidence, exit decisions and constrained consumers. Retaining a current
mechanism can be a successful investigation outcome. Conditional KV, embedded/direct-engine,
event/feed, bucket or broader module adoption cannot hold unrelated native delivery open.
No generic performance ladder, fixed crossover threshold or mandatory benchmark campaign is
selected. Existing local science remains database-free. Formal ADR status is unchanged;
any selected contract/engine/privilege change follows the existing decision route before
its implementation. No existing store, authored input or independent evidence is disposable
by virtue of this amendment.

## Improvement dispositions

I01–I14 link to the [source improvement review](../plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md).
Resolved rows refer to the functional companion Outcomes. Mixed rows I01/I02/I03/I06/I10 remain scheduled only for their named H acceptance: their native implementation and consumer retirement are delivered. I14 remains H-owned. Conditional mechanisms
inside them remain conditional; their declined alternatives are not a deferred backlog.

| Finding reference | Scenario reference | Disposition | Decision/work owner | Evidence or revisit trigger |
| --- | --- | --- | --- | --- |
| [I01](../plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md#I01) | T02 / H01 | scheduled | 58h H01; delivered T02/A–G | Native owner/caller/configuration/test/deletion cutovers delivered without bridges; H01 joined acceptance remains. |
| [I02](../plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md#I02) | D03 / H02 | scheduled | 58h H02; delivered D03/A05/C03/E01 | Native canonical progression, claims, preparation recheck, replacement and lineage controls delivered; H02 assembled campaign remains. |
| [I03](../plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md#I03) | D05 / H03 | scheduled | 58h H03; delivered D05/B/F | Native migration/version/fence and representative transformation/refusal/interruption delivered; broader persistent-production recovery qualification remains. |
| [I04](../plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md#I04) | F02 / H03 | resolved | 58b/58c/58f | Typed actual fixtures, cleanup on failed assertion, no permission registry or success-gated retention. |
| [I05](../plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md#I05) | B04 / F02 | resolved | 58b B04; 58f F02/F05 | Native cache hit/miss/compiler failure, independent seed; custom build observation removed. |
| [I06](../plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md#I06) | F02/F04 / H03 | scheduled | 58h H03; delivered F02/F04 | Native nextest/setup reporting and target/host force-validation delivered and checked; global feature/runtime qualification remains. |
| [I07](../plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md#I07) | A07 / F04 / G03 | resolved | 58a/58f/58g | Delete Python projections; native candidate/isolated oracle exchange rejects wrong candidate. |
| [I08](../plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md#I08) | T02 / F03 / G03 | resolved | 58f/58g | Each Wave0 candidate has verified deletion/replacement including generator/configuration consumers. |
| [I09](../plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md#I09) | A01 / F01 / G04 | resolved | 58a/58f/58g | Owner-local typed operations; universal catalogue, mandatory faces and interim proxy declined. |
| [I10](../plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md#I10) | B03 / F01 / H02 | scheduled | 58h H02; delivered B03/F01/A/E/G | Native configuration/scientific observation delivered and build-specific producers/consumers retired; H02 assembled observation acceptance remains. |
| [I11](../plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md#I11) | B01–B05 / C04 | resolved | 58b/58c/58f | Independent lean root, actual containment, ordinary unbounded mode; no required daemon. |
| [I12](../plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md#I12) | B01 / F03 / E03 | resolved | 58b/58f/58e | Compatible atomic mechanics reused; executable replacement, immutable payload and scientific commit semantics distinct. |
| [I13](../plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md#I13) | A01 / F03 / G04 | resolved | 58a/58f/58g | Explicit needed generation; no runtime regeneration, mandatory foreign ABI or replacement Python product. |
| [I14](../plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md#I14) | H01–H04 | scheduled | 58h; 58f F04 | Lean target acceptance, independent science and actual uncertainty; no universal evidence reconciler. |

## Dependencies, migration and completion

T01/T02 precede target-changing implementation. Establish a cold host root and independently constructible native capabilities early; neither waits on a full developer UI or optional consumer. Settle D03/D05 durable progression, version/admission and in-flight migration contracts alongside the independent capability design. B/native fixture/compiler inversions unblock development early. A/C compose scientific preparation and physical execution against D’s usable policy; D/E establish narrow control/data entry, F consumes B and domain owners for retained tools. G migrates/removes each frontend only after retained callers work; do not defer essential consumer migration to a final cleanup wave. H selects joined evidence once all selected functional scope exists. Parallel independent work retains ordinary host capacity; no CPU/RAM partitions or global serialization are introduced.

Delete replaced code/callers/tests/fixtures as soon as targeted replacement evidence and caller migration establish it. No compatibility facade, permanent dual controller, second schema authority or retained old implementation as evidence. Do not erase scientific/reference evidence, credentials or databases to make migration easier. New storage formats or data placement need explicit disposition; no data reset is authorized.

**Plan57 supersession:** its coordinator and companions are historical and authorize no work. Existing code and contracts are baseline evidence, not automatically accepted target foundations. No CBO or unfinished package is imported. Superseded documents remain temporarily while ADR/architecture/review readers need their historical evidence; H retires them after exact inbound-consumer preflight and immutable-reference repair. Lifecycle scope selectors for their packages are removed now so old rows cannot remain an active backlog. No prior failure is relabeled passed or complete.

## Verification and current checkpoint

The following authoring assessment retains its original evidence scope; later execution and
the 2026-10-29 database-boundary amendment are recorded separately below.

**Proposed:** target, packages and conditional branches. **Interface-checked:** functional review research, focused source/foundation assessment and lifecycle library source contracts. No product implementation, dependency bump, test, native build or service was run to author these plans. Documentation integrity/publication checks are reported separately from product qualification. Quantitative speed/memory/robustness benefits remain unmeasured.

At authoring all execution packages started planned; T01/T02 foundation delivery is recorded below, while B and other runtime packages retain their own current state; optional deferred delivery does not block the native core. The first executable scope is T01/T02 and the shared contract/cold-start design slices of A/B/C/D/E, followed by the smallest original native workflow with its actual host/control/data consumers. Production execution was paused at authoring; the maintainer subsequently authorized T01/T02 and B01–B05 as recorded in the execution checkpoint below. Other implementation and broad qualification remain separately selectable. Improvement recommendations I01–I14 are now
scheduled in this coordinator and their companion packages. No new companion is needed: expanded
58d keeps canonical transition/schema/admission reasoning together. The initial execution began with T01/T02 and independent cold-start/host/native capability
slices. The remaining durable work begins with D03/D05's bounded action, migration/drain and
actor/connection contracts and their implemented prerequisites, preserving the completed slices. Full production recovery qualification has an owner/trigger in H;
it is not a regenerable-development acceptance gate.


Focused independent planning assessment found the target coherent after correcting native
study submission ownership and the inherited Python compiler-wrapper cold-start gap. It
confirmed the selected zbus/Tokio manifest compatibility and the separation of control, data,
execution and containment prerequisites. Actual host255 behavior, dependency resolution and
assembled acceptance remain scheduled evidence. This is Interface-checked planning advice,
not a formal system/scientific acceptance or permission to execute production changes.

The amendment foundation assessment reused the improvement review and inspected current study
functions/worker progression plus `canonical.rs::WriteAdmission` and maintenance seams.
Current file-backed admission reads do not alone establish future stale-handle or cross-process
migration fencing. D05 therefore schedules that bounded design/verification before dependent
code. This correction prevents typed construction from being mistaken for permanent authority.
That assessment did not establish implementation. Current package states and the later
execution checkpoints own subsequent progress; document validation is separate from runtime qualification.

## Execution checkpoint — 2026-10-28

The maintainer authorized T01/T02 and B01–B05 with the necessary C/D/F prerequisite slices.
T01's scoped decision/review basis is established: [ADR-0233](../adr/0233-rust-owned-host-and-execution-boundaries.md),
[formal combined review](../design_review/reviews/design_review_rust-owned-host-and-boundaries_2026-10-28.md)
and scoped audit publication `8fed0a30c`. The review accepts the bounded Proposed target with
Interface-checked mechanisms; ADR status remains proposed. No implementation/numerical/runtime
acceptance is inferred. T02 establishes the source mapping and B01/B02 first implementation
slice below; later cutovers still reconcile their actual consumer closure before acceptance.

T02 source mapping identifies these integrated cuts:

| Cutover | Performing owner and actual consumers | Remaining boundary |
| --- | --- | --- |
| Cold construction | New lean host root/explicit seed; selected executable publication | Excludes xtask's unconditional Arrow/schema closure; no bootstrap Python-wrapper recursion. |
| Environment and launch | Host owner; pse-env, direnv's direct Python call, Just shell/interpreters and native-env shims | Preserve precedence, secrets, exits and selected scientific exclusions; remove old host callers with owners. |
| Passive ownership diagnosis | Native manager/process/cgroup reads; activity/watch consumers | Keep usable without admission/DB/compiler; build-stage/freshness observation intentionally removed. |
| Fixture physical ownership | B/C owned/borrowed resources consumed by F02 | Broader runner, reports and evidence remain F-owned; no global fixture permission table. |
| Compiler/cache | Native tool selection and F02 setup; actual compiler callers | No per-compiler Python watcher; cache-service loss cannot poison a successful result. |
| Service establishment | B process/credential mechanics composed with working D schema-ready/admission capability | B04 cannot close on D design alone; full D03 progression is not a prerequisite to the cold host root. |

Every retained direct caller of retiring Python host owners must migrate or retire before that
cut is accepted. Unfinished F features may remain unavailable with no acceptance credit; they
cannot keep the Python controller as a fallback. Existing non-host report/artifact functions
remain only after removing their old-host dependencies. Plans53/54 keep historical evidence
and unrelated work; changed fixture and cache-continuity assumptions route through ADR-0233.

The subsequent selected database-free ordinary wave now connects the native host to actual
pse-env/direnv/hook/Just/Cargo consumers, two-scope caller-death containment, native cache and
telemetry configuration, generic Cargo/nextest correctness and scoped G retirement. The
[58b checkpoint](58b-bootstrap-and-host-control.md#execution-checkpoint--2026-10-28) owns current
supported placement, targeted results and remaining host evidence; C/F/G companions retain their
own selected-slice status. Their full packages remain in progress where broader service,
frontend, fixture, developer or retirement obligations remain. No D readiness slice was needed
for the first consumer, and no database reset or service cutover was performed.

Unsupported finite/scientific/store/reference modes and provider profiles explicitly refuse without a Python
fallback. Unmigrated Python helper libraries remain only for their later owning cutovers, not as
accepted alternate entry points. Python pse-env/Arrow command entry points, native export shims,
per-compiler watcher and bespoke build-observation/measurement paths have retired. The native
ordinary consumer transition is concrete; it does not establish all-series or package completion.
H's broad qualification remains unselected. Shared status stays here and in existing companions.

The follow-on database-free compiler/KLU wave adds explicit native provider preparation and
selection for the existing KINSOL Runtime consumer, plus the necessary F01/F02/C04 and G
retirement slices. The [58b compiler/KLU checkpoint](58b-bootstrap-and-host-control.md#explicit-compilerklu-consumer-checkpoint--2026-10-28)
owns implementation, verification and remaining limits. Preparation is an explicit operation;
launch never builds a missing provider. The operator selected a Rust-owned subdirectory of the
existing cache with legacy generations preserved and no automatic collection. Other provider
profiles, services, finite/reference/timing modes and H qualification remain outside this wave.


The subsequent native local-operation wave selects A01/A03, source-admission-only A02,
local C01/C02, F02 and the necessary compiled-stub F03/G retirement slices. The
[58a checkpoint](58a-native-product-and-scientific-workflows.md#native-local-operation-checkpoint--2026-10-28)
owns composed results. It uses public Rust constructors and retires Python asyncio run
waiting; no database/service, additional solver, mandatory CLI or new lifecycle framework
is selected. Broader package and H qualification obligations remain with their owners.

## Database-boundary amendment checkpoint — 2026-10-29

**Proposed:** review-qualified O01–O10 dispositions, updated A–H responsibilities,
owner-local acceptance, dependency handoffs and conditional investigations. The focused
foundation assessment reuses the review's actual worker/generator/module/transport/payload
inspection: existing graph features and one shared readiness policy are foundations;
repeated outer progression is the consolidation opportunity, not every native loop.
No completed T/B/A/C/F/G slice is reopened or promoted by document adoption. Persisted
interpretation/write fences remain dynamic external premises, not permanent authority from
Rust construction. Existing native host and local KINSOL evidence retains its exact scope.

This amendment changes plans and their metadata only. Production/database execution,
service readiness, schema application and H qualification receive no new acceptance credit.
No new guarantee reduction, data reset, global resource ledger, mandatory frontend or
performance threshold is selected. Formal ADR status remains unchanged; newly selected
engine/transport/privilege contracts must follow the decision route before dependent code.


## Native host execution handoff — 2026-10-29

Current assignments have concluded at the maintainer's request; changes are integrated in main
and no auxiliary worktree or test campaign remains. [58b's current handoff](58b-bootstrap-and-host-control.md#current-execution-handoff--2026-10-29)
owns host verification, source/lint closeout and remaining stage 5/6 obligations. [58d](58d-canonical-control-and-connectivity.md#native-host-dependency-execution-checkpoint--2026-10-29)
owns the working native service/readiness/migration prerequisite, including the final 78-test
selection with all five live cases passing. These slices unblock a selected retained canonical
consumer without requiring whole-D completion. B02–B05 and D01/D05 remain in progress; C04/F02
have contributing slices only. Remaining Python consumer retirement, broader host/fault/provider
acceptance and H qualification are open. No whole-series or performance claim follows.


## Full B/C/D execution selection — 2026-10-29

The maintainer selected execution of all remaining 58b/58c/58d scope and the minimum A/E/F/G
prerequisite and retirement slices needed for actual completion. Other companion obligations
and the full historical/release H campaign remain separately owned. The execution sequence is
shared contract and exact-v2 preservation; host/native ownership prerequisites and canonical
control/progression; actual migration/fence acceptance; worker/preparation/claim integration;
publication/delivery settlement; integrated caller retirement; then selected assembled acceptance.
Existing targeted evidence retains its original scope and does not close these packages.

C01's selected composition remains the existing tokens, scoped Futures, result-bearing joins
and actual native-thread/process owners. Flat process roots do not justify a hierarchical
shutdown dependency. Type=exec and D-owned schema readiness remain distinct; no sd-notify,
SDK replacement, KV/MX tiering, broader module permissions or new frontend is selected.

Before changing generated control/study declarations, preserve the exact current schema-v2
DDL and digest. The supported bounded migration chain becomes pre-epoch-v10 → schema-v2 →
schema-v3 while scientific interpretation remains v10. Historical original target-v2 requests
must settle as target-v2 receipts, never silently retarget to v3 or mint current readiness.
New declarations and consumers share one unreleased v3 target; no existing user store is
migrated or reset by implementing or testing this scope.

## Maintainer closeout and pause — 2026-10-29

Implementation is paused after concluding the current assignments in main. The
[58b native scientific owner/workstation checkpoint](58b-bootstrap-and-host-control.md#native-scientific-owner-and-workstation-closeout--2026-10-29),
[58c targeted lifecycle closeout](58c-runtime-lifecycles.md#native-scientific-worker-closeout--2026-10-29)
and [58d composition checkpoint](58d-canonical-control-and-connectivity.md#native-scientific-composition-closeout--2026-10-29)
retain the current implementation, evidence and next obligations. B/C/D remain in progress;
this checkpoint does not promote source-migrated historical fixtures, focused tests or the
native primary cut into whole-series acceptance. G02 stays complete; other G/F consumers
retain their actual owners. No worktree remains to integrate, no push is authorized and
no broad qualification/performance campaign is running. Resume from those checkpoints rather
than relaunching earlier interrupted runs or reconstructing the configured user database.


## Native application handoff — 2026-10-29

[58a's Outcome](58a-native-product-and-scientific-workflows.md#outcome-recorded-after-implementation)
closes A01–A07's selected native functional/caller scope and owns its repaired scope-end
checks. The completed companion remains for live F/G/H and architectural readers. This is
not full first-party Python retirement or assembled Plan58 qualification. F/G retain actual
campaign/reference/source/result-adapter consumers; H retains the complete matrix, reference,
performance and later persistent-production obligations. B/C/D remain paused at their existing
checkpoints. The comprehensive Clippy request also corrects native immutable-provider staging
and the Clippy-only graph-option translation; it does not resume unrelated B/C/D production scope.


## Remaining B–G execution selection — 2026-10-29

The maintainer authorized all remaining Plan58 scope except 58h, following the detailed
execution approach at baseline `160441979`. T01/T02, A01–A07, B01 and G02 remain delivered
prerequisites. Existing B–G package tables continue owning progress; this selection does not
close them or restart completed native scientific callers.

Execute native host/fixture/tooling foundations, then paired canonical migration/control and
worker admission/progression, protected data/publication, retained developer consumers and
final boundary retirement. Independent generation work can proceed alongside those cuts.
A consumer requires the actual usable capability, not completion of an entire lettered plan.
Each replacement includes callers, declarations, targeted tests and deletion. The root owns
shared manifests, generated publication, the Cargo driver, commits and status reconciliation.
Ordinary work remains uncapped at 32 Cargo jobs and 16 test lanes.

The maintainer selected preserving manifest-driven conformance and paired thermodynamic
campaigns as callable Rust capabilities. Their Python orchestration retires; a new CLI framework
is not required. Obsolete Python measurement drivers may retire where needed for G, while
scientific scenario requirements and existing results remain for H. Replacement measurement
campaigns, full assembled/reference/feature qualification, performance claims and production
recovery remain H-owned and are not authorized by this selection.

C01 retains existing Tokio/tokio-util/Futures/Rayon supervision and original native joins;
no hierarchical shutdown library is selected. D01 retains the bounded Tonic/Hyper native
client and constrained actor API with one canonical policy. Exact SDK3.3.4 source exposes
provisional streaming but not the selected cumulative bounds/physical-drain completion contract;
its gRPC capacity argument is unused. Typed SDK values are not the missing capability.
These source-backed choices do not establish runtime acceptance.

Native provider activation must separate construction-only external prefix inputs from launch:
an absent own-prefix override selects the completed immutable snapshot; the exact recorded
source or selected generation remains accepted, and conflicting explicit overrides refuse.
All other configuration, tool, payload and pin checks remain. Long-hold module construction
uses an owned workspace source snapshot and alternate module-owned manifest with the selected
patched official CLI; production sources are unchanged and new CLI provenance is not represented
as a historical Python packer. Constructor tests are B scope; saturation campaigns remain H.

The four independent reference candidates already use native exchanges. Remaining actual
scientific consumer cuts are uncertainty/covariance adoption and research selection/protected
reopen, not hypothetical remaining parity families. Orphan parity context/support retires with
its old extension-dependent preflight. Independent oracle drivers remain isolated.

Final product-extension/crate and governance retirement consumes the proposed ADR-0233
amendment and required scoped design review before code. Conditional RMCP/PyO3 delivery,
new storage/transport/reactive mechanisms and F06 stay deferred without a concrete trigger.
There is no data reset or automatic cache/generation collection in this authorization.


### Remaining retirement decision coverage

The [independent bounded target extension](../design_review/reviews/design_review_rust-owned-host-and-boundaries_2026-10-28.md#13-remaining-native-development-and-product-retirement--2026-10-29)
accepts the proposed ADR-0233 amendment; it supplies no implementation or H acceptance.

| Finding reference | Disposition | Work owner | Evidence or completion obligation |
| --- | --- | --- | --- |
| Retirement review RC07 | resolved | T01 / G03 | Required scoped review/proposed ADR preceded removal; revisions311–313 reconcile D12/workspace/§21 and adjacent owners. Native consumers migrated or explicitly retired; formal ADR status unchanged. |
| Retirement review RC08 | resolved | F02/F03/G03 | Native generation/fixtures/environment/governance integrated and old Python mechanisms deleted (600d4df30); owner tests and zero-target checks below preserve actual admission and scientific identity. |
| Retirement review RC09 | resolved | G01/G03; H scenario owner | Exact original-source/scenario/input/expectation/custody handoffs are recorded in 58h before driver retirement; historical results retained. This resolves transfer/deletion, not H execution. |

## Remaining B–G Outcome — 2026-10-29

### What was built

**Implemented:** all selected remaining B–G functional packages, integrated with the completed
A01–A07 callers. The native host constructs providers, modules, supervised work and private
canonical fixtures without the application or Python bootstrap. Runtime owners distinguish
observer detachment, cancellation, actual native completion and durable result acceptance.
Canonical SurrealDB/MX remains authoritative for durable progression, claims, lineage and
migration/write admission; native workers retain scientific execution and physical release.
Protected readers and columnar delivery construct without mathematics. Native development,
generation, configuration, conformance and campaign APIs replace retained first-party Python
operations. `600d4df30` removes the required product/extension/controller boundary;
`32e3fe21d` completes independent reference typing/environment contracts and `6dc84501d`
settles native Cargo cfg resolution and operator ownership. Companion Outcomes own package
acceptance. F06 and optional adapter implementations remain deferred, not missing core work.

The exact-release SDK investigation retains the existing bounded Tonic/Hyper client because
the selected physical-drain/cancellation and cumulative-bound contracts are not supplied by
that SDK interface. Existing Tokio/tokio-util/Futures/Rayon composition remains the runtime
mechanism. No second scheduler, universal command catalogue, mandatory CLI/MCP, storage engine,
reactive feed or scientific execution inside database modules was introduced.

### Verification

**Tested:** the following scope-end commands ran on local Linux, pinned toolchain and locked
resolution, ordinary uncapped 32 Cargo jobs/16 nextest lanes. Every failure/warning target is
zero. Test selections below overlap; their counts are not a total of unique tests. Excluded
or ignored tests were not qualified by a passing selection. All Cargo correctness closures
use the native force-validation adapter; native profiles select prepared immutable providers.

```bash
scripts/pse-env -- scripts/cargo-bin/cargo-pse-correctness nextest run \
  --no-fail-fast -p pse-host -p pse-codegen -p pse-columnar -p pse-buildinfo \
  -p pse-diagnostics -p xtask --lib --bins --tests --no-default-features \
  --features xtask/package-fixtures --locked
```

Result: **723 passed, zero failed**, run `7152b88a-c932-41f1-84be-83a9b7687485`.
One internal subprocess fixture is intentionally excluded from direct discovery and exercised
by its cancellation caller. This includes actual native host/TTY/finite/unbounded containment,
original-owner death/drain, declarations, generation, docs/tool controls and columnar custody.

```bash
scripts/pse-env -- scripts/cargo-bin/cargo-pse-correctness nextest run \
  --no-fail-fast -p pse-host -p xtask --lib --no-default-features \
  --features xtask/evidence-tools --locked \
  -E 'package(pse-host) | test(development::workspace_sources::tests::)'
```

Result: **199 passed, zero failed, 111 excluded**, run
`c19a62f9-893b-4b91-a939-6f2fa8b87db3`, after the final cfg-marker/tail and bounded
workspace-source traversal fixes. This is receiving-host/corpus acceptance, not a full evidence
corpus refresh or source-completeness claim.

```bash
scripts/pse-env --native=compiler,solver,klu,isolation,uno,petsc -- \
  scripts/cargo-bin/cargo-pse-correctness nextest run --no-fail-fast \
  -p pse-runtime --lib \
  --features native-study-tests,product-operators,solver-scip,solver-ipopt --locked \
  -E 'test(execution_lifecycle::tests::native_sessions_) | test(tooling::telemetry::tests::) | test(math::staged::tests::observation_native_session_) | test(math::staged::tests::native_session_close_) | test(math::portable::tests::deployment_startup_) | test(math::portable::tests::async_local_replay_) | test(math::portable::tests::scoped_local_replay_) | test(math::portable::tests::local_replay_admission_) | test(math::portable::tests::deployment_receipt_current_shape_) | test(math::portable::tests::portable_body_relative_) | test(math::portable::tests::portable_envelope_)'
```

Result: **22 passed, zero failed, 1,272 excluded**, run
`f3701183-8c34-47b8-b09c-9daa92a6db3a`. This includes actual native-session close/startup,
producer lifetime/link propagation and replay admission controls, not every solver journey.

Actual canonical operator qualification first built and captured the selected native tool:

```bash
scripts/pse-env -- scripts/cargo-bin/cargo-pse-correctness build \
  -p xtask --bin xtask --no-default-features --features canonical-tools --locked
```

The caller copied that executable into a private immutable selected-tool directory before any
later Cargo build. With `PSE_CANONICAL_TOOLS` selecting that captured executable and
`PSE_DIAGNOSTIC_EVIDENCE` selecting its private diagnostic directory, the following command used
the native-prepared production module receipt (the absolute paths below are run-local inputs):

```bash
scripts/pse-env -- target/debug/pse-canonical-test \
  --root "$PWD" --correctness "$PWD/target/host-tools/current/cargo-pse-correctness" \
  --nextest "$(command -v cargo-nextest)" \
  --helper "$PWD/target/debug/pse-canonical-service" \
  --module-receipt "$pse_module_receipt" --state-parent /tmp \
  --build-arg=-p --build-arg=pse-operations --build-arg=--lib \
  --build-arg=--features --build-arg=canonical-tests,native-service \
  --build-arg=--locked -- --run-ignored only \
  -E 'test(canonical_operator_graph_evidence) | test(canonical_operator_query_evidence)'
```

Result: **2 passed, zero failed, 497 excluded**, run
`1275d920-610d-43e5-800a-903ad9846cf9`. The fixture builds/lists the exact source tests before
starting a fresh private service and settles its original owner afterward. Existing configured
canonical data was not used. Earlier D/C checkpoint tests retain their original migration,
competing-claim, lost-ACK, actor, worker replacement and publication scope; the narrow operator
run does not replace or enlarge those results.

The explicit Trace Processor consumer test
`capture_standalone_trace_processor_reads_numerical_events_with_clock_alignment` passed
(one selected ignored test, 1,293 excluded; run `b566ac36-1ca7-42d1-bd91-1dcd5b6ec3f2`)
with `PSE_TRACE_PROCESSOR` selecting the existing 58.2 executable. It checks actual query-file
execution, clock alignment and original child settlement; it does not qualify optional-tool
installation/provenance or a performance campaign. Earlier native covariance/research/protected
reads and managed-Ipopt/SCIP-limit tests retain their explicit companion scope and failure repairs.

| Command/mode | Result against zero target | Boundary |
| --- | --- | --- |
| `scripts/pse-env --native=compiler,solver,klu,isolation,uno,petsc -- just run-all clippy-default clippy-no-default lint-solver-contracts governance audit-shear ready` | Passed; default/no-default/native Clippy zero warnings; governance26 passed; generated8 schema targets, family/dependency checks and required readiness checks passed | Optional unavailable services/tools remain reported, not auto-repaired or falsely ready. |
| `scripts/pse-env --native=compiler,solver,klu,isolation,uno,petsc -- scripts/cargo-bin/cargo-pse-correctness clippy --keep-going --workspace --all-targets --all-features --locked -- -D warnings` | Passed, zero warnings, diagnostic-taskdump profile after final source fixes | Selected native profile; full Clippy surface, not full feature runtime qualification. |
| `scripts/pse-env -- just run-all lint-py typecheck lint-typos lint-shell lint-actions lint-ast lint-zizmor` | Passed; Ruff zero; Pyrefly3.13/3.12 zero diagnostics; retained reference-driver controls4 passed separately | No scientific oracle campaign; existing explicit security-lint policy exclusions remain visible. |
| `scripts/pse-env -- env CARGO_HACK_CARGO_SRC="$PWD/scripts/cargo-bin/cargo-pse-correctness" cargo hack check -p pse-host --no-default-features --locked` | Passed | One actual native cargo-hack delegation; no global powerset claim. |

The initial `just hygiene` aggregate reported failures in `lint-py`, `audit-shear`,
`clippy-default` and `clippy-no-default`; those actual findings were corrected and all four
recipes passed in the scope-end commands above. Its other checks, including rustdoc, agent
wiring, ADR/register and generated-output checks, passed. This is composed qualification,
not a claim that the original failing aggregate passed.

**Tested:** `scripts/pse-env -- just turn-end` passed (native ADR index, Rust/TOML/Python formatting); `scripts/pse-env -- just docs` published 912 chapters with scoped search; `scripts/pse-env -- target/debug/xtask lifecycle validate` validated 1,578 native scope items with zero errors. Publication includes concurrent library inputs without adopting or committing them. These are documentation/process results, not product or scientific qualification.

**Tested:** the B05 tracked-source archive at `6dc84501d` initially had neither `target/` nor
`.venv`. Under the original checkout's uncapped `scripts/pse-env`, its own
`scripts/bootstrap-host`, `scripts/pse-host --version` and `scripts/pse-host --help` passed;
no `.venv` was created. Existing acquired toolchain/dependency sources/compiler cache were
available. This establishes application-independent cold-target construction, not cold-machine
acquisition, other-platform support or a timing benefit.

### A mistake made and corrected

The diagnostic Cargo profile initially supplied `tokio_unstable` only to compilation. Cargo's
cfg-based dependency resolution therefore omitted Tokio's conditional backtrace dependency.
The native adapter now supplies the same cfg to dependency discovery and compilation,
preserves active inherited flag precedence, bounds the supported checkout/platform and keeps
arguments after `--` literal. Actual all-features compilation exposed and verified this repair;
a compile-only flag assertion would have missed it.

The first operator rerun also selected mutable `target/debug/xtask`; an intervening smaller
feature build replaced it. Capturing the explicitly built tool before building the fixture
fixed the test setup. Real inspector startup now opens its store inside the existing awaited
disconnect path. Assertions decode the actual tagged SDK values and compare actual statement
indices rather than stale incidental counts. Telemetry tests now release the captured initiating
context before asserting span completion and recognize W3C extraction's remote-link bit while
preserving exact trace/span identity. None of these repairs relax scientific acceptance.

### Deviations from the plan, deliberate

The SDK, shutdown-library and optional interface investigations close with retained working
mechanisms where substitution would lose guarantees or add machinery. Obsolete measurement,
recovery-presentation and first-party Python wrappers retire rather than being translated for
parity. Exact scientific scenarios/inputs/expectations and observation/custody requirements
transfer to H before driver deletion; historical results remain. Independent third-party Python
oracles remain isolated. No compatibility product, automatic cache collection or data reset
was added. The separately owned compiler-cache service remains active.

**Not run:** 58h campaigns, full scientific/reference qualification, global feature powersets,
new performance campaigns, release/remote/other-platform qualification and exhaustive persistent
production recovery. Benefits remain **unmeasured**. These are owned H/future-release obligations,
not implicit passes or blockers to this selected functional delivery. Plan58 remains in progress.
Completed companions and source reviews remain for their H/proposed-ADR consumers; unrelated
Plan56/57 historical readers and concurrent library-document edits are preserved.
