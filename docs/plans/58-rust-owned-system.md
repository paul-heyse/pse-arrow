---
title: "Plan58: Rust-owned system"
status: draft
date: 2026-10-28
adrs: []
review_sources: [docs/functional_reviews/rust-first-system-functional-opportunities-2026-10-28.md]
doc_role: plan
doc_owner: docs/plans/58-rust-owned-system.md
doc_topics: [current-work, architecture, design, libraries, validation]
---

# Rust-owned system

## Purpose and authorization

**Proposed:** deliver the supported simulator through complete Rust-owned capabilities, with first-party host/development operation independent of Python. This is a new design and delivery series, not a continuation of Plan57's packages. The operator authorized authoring on 2026-10-28 and explicitly superseded all Plan57 execution scope. Creating this series authorizes no production implementation, dependency change, service restart or qualification campaign.

The sole substantive review input is the [Rust-first functional review](../functional_reviews/rust-first-system-functional-opportunities-2026-10-28.md). External pivot reviews are excluded until detailed planning is complete. Baseline is `0659343f123fefa2371ac7d1659e3af7f75f9949` plus user-added zbus/systemd, sd-notify and tokio-graceful-shutdown overviews. Those untracked documents remain user-owned. Core 3.4, efficiency heuristics 1.0, test assessment 1.2 and process-simulator 1.5 apply.

The latest operator instruction overrides both the review's mandatory-CLI recommendation and the earlier planning answer: **CLIs are optional conveniences**, not completion requirements. No broad command framework, universal operation catalogue or agent execution engine is selected. RMCP remains conditional. PyO3 gets a future interface seam; implementation is deferred. Production robustness and simplicity outrank peripheral interface parity.

## Target and integral foundation assessment

The delivered product is a Rust library/application system. Typed authored definitions and context bindings feed existing scientific preparation and class-specific execution owners. Direct Rust consumers use typed operations, results and explicit lifetimes; serialization occurs at actual external boundaries. No Rust workflow calls Python to acquire ordinary product or host capabilities. Existing useful executables may remain thin composition roots; their command surface cannot dictate domain APIs or pull development dependencies into production.

Four capabilities have different prerequisites: pure/local scientific execution; connection/authentication/bounded durable control; protected retained-data reading; and supervised native work. Compact reads and result export must not construct mathematics/provider services. Local pure work needs neither mandatory database nor daemon. Durable work uses the current supervised canonical SurrealDB/MX path; the database does not own host process death or native thread joins.

Current strengths inspected through the review and focused source assessment include Rust authoring/compiler/runtime, maintained Symbolica/faer, native joined pools, canonical command/receipt policy, result snapshots/providers and Rust generation. Existing Python runtime facade, control transport, MCP workers, environment/compiler wrappers and test-resource calls add boundaries to replace or delete. Their behavior is assessed by consumer need; no ledger, receipt, scan, wrapper or validation pass is preserved merely because it exists.

Preserve physical meaning, original-space outcome assessment, supported square/root and optimization modes, dynamics/events, fitting/sensitivity, initialization/recycles, studies/sampling, diagnostics and project/source/result history within current supported limits. This does not promise unimplemented IDAES families or broaden numerical claims. Python reference tools may remain isolated independent oracles. The unresolved historical SIGSEGV and arithmetic limits remain evidence to understand where relevant, not inherited implementation packages or proof of language causation.

### Physical organization and chosen mechanisms

| Responsibility | Selected target and limit |
| --- | --- |
| Native product | Existing domain/runtime owners with independently constructible scientific, control and retained-data capabilities; no frontend-specific policy. |
| Host/bootstrap | A small planned Rust tool/library root independent of Arrow, math, native solvers, database and generated product schemas. New crate and dependency direction receive ADR/design review before implementation. A tiny shell seed may locate/build/install it, with no duplicate host policy. |
| Linux control | Typed zbus/systemd user-manager control is the planned replacement for the Python systemctl/systemd-run orchestration. Used interfaces must work on host systemd255. Select Tokio-backed zbus, not an extra runtime. No permanent parallel Python controller or administrative-credential fallback. |
| Runtime lifecycle | Existing Tokio/Futures/Rayon ownership and explicit native OS joins remain the foundation. Compare hierarchical shutdown only for long-lived application subsystems before selecting it; do not insert a second scheduler or treat task abort as native release. |
| Canonical state | Effective patched SurrealDB3.3.4/MX1 SSI and synchronous persistence initially retained. Narrow Rust command/connection owner and existing policy/receipts; no mandatory new RPC, embedded backend or module-hosted science. |
| Data | Protected Arrow IPC snapshots and configured DataFusion providers, selective typed metadata, existing charged Bytes custody. Flight/remote objects/Parquet derivatives require their actual consumer. |
| Consumers | Native Rust first; useful low-cost executable adapters optional. Delete Python MCP without feature parity. No RMCP or PyO3 delivery dependency. |

Library research informs rather than follows the interfaces. The review's full overview research and focused exact-source checks are reused. New lifecycle overviews were read end to end and checked through Context7/primary sources during preparation. zbus6.1 requires Tokio ^1.53.3 while current pin is1.53.2; B owns a deliberate exact1.53.3 move after resolution/feature validation. Proxy generation targeting a newer systemd is not evidence of runtime support. sd-notify0.5's send/absent-socket success is not readiness observation; abstract sockets/barrier and early environment mutation require explicit disposition. tokio-graceful-shutdown0.20 abort/timeout cannot establish native completion. These are bounded integration questions, not an excuse for a fallback framework.

## Confirmed target/rule directions

| Direction | Operator disposition on 2026-10-28 | Route before dependent implementation |
| --- | --- | --- |
| Rust-owned product; Python no mandatory authoring/result boundary | Explicitly selected in this engagement | T01: D12/blueprint §21 boundary ADR and required design review; supersede affected immutable decisions rather than edit them. |
| Python FastMCP may retire; MCP optional; CLI not required | Explicitly selected, with later CLI correction governing | T01/G: revise single-endpoint/tooling/interface requirements where removal changes authority; retain useful effect semantics independently. |
| First-party host/developer responsibilities may move to Rust | Explicit end-to-end scope selection | T01/B/F: host identity, bootstrap/environment and governance owner changes through applicable ADR/design/instruction route. |
| New host root and library choices | Proposed technical realization within authorized plan creation | T01: crate addition/dependency direction review before code; ordinary exact dependency changes need no separate dependency ADR. |

No blanket weakening of physical release, scientific validity, protected data or unknown-effect semantics is authorized. Any newly discovered rule change needs its specific operator disposition before dependent work, per create-plan. This series is not itself formal ADR acceptance.

## Documents and responsibility boundaries

| Companion | Owns |
| --- | --- |
| [58a](58a-native-product-and-scientific-workflows.md) | Rust product/scientific capabilities and preparation; incremental/adaptive extensions |
| [58b](58b-bootstrap-and-host-control.md) | Cold bootstrap, environment, typed Linux process/service control |
| [58c](58c-runtime-lifecycles.md) | Native task/resource composition, stop/drain/failure policy |
| [58d](58d-canonical-control-and-connectivity.md) | Independent control, shared durable policy and synchronization |
| [58e](58e-retained-data-and-delivery.md) | Protected data, relational access and delivery |
| [58f](58f-development-and-assurance.md) | Retained first-party development/assessment/configuration workflows |
| [58g](58g-boundary-retirement-and-optional-consumers.md) | Python/MCP deletion and future optional consumers |
| [58h](58h-qualification-and-publication.md) | Joined acceptance, source transition and enduring publication |

Companions own package state. This coordinator owns opportunity disposition and shared decisions; no second package ledger. Letters are reading groups, not serial execution lanes.

## Foundation state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| T01 | planned | Coordinator | Required decision/design route and shared capability/dependency contracts established before contract-changing implementation |
| T02 | planned | Coordinator | Source/consumer inventory reconciled to this target; exact next implementation slice agreed without inherited Plan57 scope |

<a id="t01"></a>
### T01 — Establish decision and integration authority

Prepare the Rust boundary/host-root ADR and affected architecture/instruction amendments, with required independent design assessment under the binding. The functional review/focused authoring assessment does not substitute for that review. If a new diagnostic design engagement is needed, refresh its applicable audit through that route; no broad audit was initiated by plan authoring. Include concrete consumers, crate/dependency closure, typed operations, external effect identity, host release and failure boundaries. Review native product, bootstrap and database interactions together before consumer work depends on them. Acceptance is a coherent scoped decision and explicit remaining limits, not an invented runtime pass. Formal ADR status changes only by the prescribed PR route.

<a id="t02"></a>
### T02 — Reconcile current source and first executable slice

Map retained user/scientific outcomes to actual Rust owners and frontend-only behavior slated for deletion. Inspect Rust→Python inversions in product, fixture, compiler and generation paths. Decide retained third-party/oracle use separately from first-party Python policy. Freeze only the first working contracts needed by a coherent production/bootstrap slice; do not enumerate a universal dispatcher. Implementers may make local choices but may not select competing lifecycle or authority models accidentally. A01/B01/C01/D01/E01/F01/G01 consume this result as relevant; investigations can proceed before final integration authority when they do not mutate production.

## Opportunity dispositions

References O01–O15 mean anchors in the source functional review. Scheduled means selected planned work, never implemented or resolved. Conditional/deferred opportunities have a concrete revisit and do not block core completion.

| Opportunity | Disposition | Work owner | Required evidence or trigger |
| --- | --- | --- | --- |
| O01 | scheduled | 58a | Complete supported native product journeys |
| O02 | scheduled | 58f | Retained developer capabilities independent of Python/MCP; CLI facade optional |
| O03 | scheduled | 58g | Retire Python MCP; reduced RMCP deferred until a useful actual-host consumer |
| O04 | deferred | 58g | Define seam now; implement only for a concrete Python consumer |
| O05 | scheduled | 58b | Cold-start and exact original process/resource settlement |
| O06 | scheduled | 58c | Native cancellation/destruction/join semantics and concurrent composition |
| O07 | scheduled | 58d | Connection-only control and original durable command reconciliation |
| O08 | scheduled | 58e | Protected result access without mathematical construction |
| O09 | scheduled | 58f | Shared typed Rust contracts/diagnostics with responsibility-owned generation |
| O10 | scheduled | 58a | Preserve/extend consumed library preparation and qualified arithmetic |
| O11 | scheduled | 58h | Owner tests, original consumer journeys and independent oracle isolation |
| O12 | scheduled | 58a | Native incremental workspace; disk persistence conditional on a concrete benefit |
| O13 | deferred | 58a | Bounded comparison when adaptive external batch study needs more than current actor |
| O14 | deferred | [58f F06](58f-development-and-assurance.md#f06) | Discrete explanation consumer beyond ordinary validation justifies Z3 |
| O15 | deferred | 58e | Actual remote interoperable batch/artifact consumer justifies Flight or signing |

## Dependencies, migration and completion

T01/T02 precede target-changing implementation. Establish a cold host root and independently constructible native capabilities early; neither waits on a full developer UI or optional consumer. A/C compose scientific preparation and execution, D/E establish narrow control/data entry, F consumes B and domain owners for retained tools. G migrates/removes each frontend only after retained callers work; do not defer essential consumer migration to a final cleanup wave. H selects joined evidence once all selected functional scope exists. Parallel independent work retains ordinary host capacity; no CPU/RAM partitions or global serialization are introduced.

Delete replaced code/callers/tests/fixtures as soon as targeted replacement evidence and caller migration establish it. No compatibility facade, permanent dual controller, second schema authority or retained old implementation as evidence. Do not erase scientific/reference evidence, credentials or databases to make migration easier. New storage formats or data placement need explicit disposition; no data reset is authorized.

**Plan57 supersession:** its coordinator and companions are historical and authorize no work. Existing code and contracts are baseline evidence, not automatically accepted target foundations. No CBO or unfinished package is imported. Superseded documents remain temporarily while ADR/architecture/review readers need their historical evidence; H retires them after exact inbound-consumer preflight and immutable-reference repair. Lifecycle scope selectors for their packages are removed now so old rows cannot remain an active backlog. No prior failure is relabeled passed or complete.

## Verification and current checkpoint

**Proposed:** target, packages and conditional branches. **Interface-checked:** functional review research, focused source/foundation assessment and lifecycle library source contracts. No product implementation, dependency bump, test, native build or service was run to author these plans. Documentation integrity/publication checks are reported separately from product qualification. Quantitative speed/memory/robustness benefits remain unmeasured.

All execution packages start planned; optional deferred delivery does not block the native core. The first executable scope is T01/T02 and the shared contract/cold-start design slices of A/B/C/D/E, followed by the smallest original native workflow with its actual host/control/data consumers. Production execution remains paused until separately authorized. External reviews remain excluded until detailed planning is complete.


Focused independent planning assessment found the target coherent after correcting native
study submission ownership and the inherited Python compiler-wrapper cold-start gap. It
confirmed the selected zbus/Tokio manifest compatibility and the separation of control, data,
execution and containment prerequisites. Actual host255 behavior, dependency resolution and
assembled acceptance remain scheduled evidence. This is Interface-checked planning advice,
not a formal system/scientific acceptance or permission to execute production changes.
