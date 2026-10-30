---
title: "Plan 55: MCP development effectiveness"
status: complete
date: 2026-10-25
adrs: []
review_sources: [docs/functional_reviews/mcp-development-effectiveness-2026-10-25.md]
doc_role: plan
doc_owner: docs/plans/55-mcp-development-effectiveness.md
doc_topics: [current-work, design, environment, diagnostics, validation]
---

# Plan 55: MCP development effectiveness

## Target and authorization

**Implemented:** make the canonical capability interface a useful working route for Codex to
understand changes, find their authoritative producers, inspect scientific meaning, choose and
perform appropriate validation, make controlled edits and explain execution. Add faithful
Cyclopts-derived access when the host's MCP attachment is stale or unavailable. Developer use
on the maintainer's workstation is primary; production investigation and eventual external
agent consumers benefit from the same contracts.

The maintainer authorized creation of this new series on 2026-10-25 from the
[MCP development-effectiveness review](../functional_reviews/mcp-development-effectiveness-2026-10-25.md).
All O01–O16 and their material recommendations, alternatives and uncertainty have an owner below.
The maintainer selected **investigate, then decide** for O03/O11/O12/O16: their inquiries were required, while optional delivery depended on the recorded judgment.
The maintainer subsequently authorized the entire implementation and positive inquiry selections
on 2026-10-25. The source review remains evidence, not live status.

The coordinator owns the combined target, opportunity dispositions, shared decisions and series
completion. Companions own package state and detailed acceptance. Current code and architecture
are the baseline; the series has no dependency on another implementation-plan series.

| Companion | Owned design and delivered boundary |
|---|---|
| [55a: Capability contracts, clients and recovery](55a-capability-contracts-clients-and-recovery.md) | Constructible contracts, faithful derived CLI, connection diagnosis and intended-request explanations |
| [55b: Structured change context and knowledge](55b-structured-change-context-and-knowledge.md) | Native indexed knowledge queries and attributed producer/consumer/contract/evidence relationships |
| [55c: Native model and result inspection](55c-native-model-and-result-inspection.md) | Model declaration/inspection/comparison and meaningful bounded nested result reads |
| [55d: Editing, validation and regression experiments](55d-editing-validation-and-regression-experiments.md) | Existing affected-selection integration and bounded decisions about further edit/experiment adaptation |
| [55e: Diagnostics and investigation handoffs](55e-diagnostics-and-investigation-handoffs.md) | Typed metric dimensions, exact invocation explanation, retained handoffs and query-investigation inquiry |
| [55f: Qualification and publication](55f-qualification-and-publication.md) | Assembled agent journeys, scope-end checks and enduring operating/architecture publication |

## Baseline and focused foundation assessment

**Interface-checked:** authoring inspected clean implementation baseline
`391c949b5` and the review's source/installed-library evidence. Its newly authored proposals do
not change that implementation. The review's seventeen subprocess cases and fresh/attached-client
observations were not rerun for plan authoring. Its stale attachment is not a reproduced transport
disconnection; its generator argument-loss controls establish the named local behavior only.

Use Core 3.4, Efficient Architecture Heuristics 1.0, Test Assessment 1.2, Process Simulator 1.5
and the pse-arrow binding selected by
[standard.toml](../design_review/design_principles/standard.toml). Blueprint §19.2, §21.1/§21.5,
§23.1/§23.2 and §24.2 own the relevant execution, boundary and qualification contracts.
This integral foundation assessment initiates neither a broad audit nor a new diagnostic review.

| Foundation inspected | Assessment and planned consequence |
|---|---|
| Catalogue, strict FastMCP dispatch, loaded-source coherence and execution basis | Reuse them. Current descriptions expose an operation wire revision; compare that operation for CLI compatibility, leaving relevant source/native admission with the server. No second version ledger or blanket CLI source snapshot. |
| FastMCP 4.1.3 generator | Stock discovery sees visible tools; emitted calls create fresh connections and discard null/empty lists. Own a narrow schema-to-Cyclopts adaptation and shared runtime, rather than patching generated files/site-packages or writing callbacks per operation. |
| Native scientific schema/codec boundary | Rust and directional generation remain authoritative. New scientific reads must preserve the original document exactly or refuse meaningfully; generic host truncation is not an admissible projection. |
| Workspace concepts, producer triples and knowledge store | Source and indexed routes already exist. Adapt native queries and strengthen actual producers before adding graph traversal; unknown relationships stay unknown. |
| Affected selection and edit/experiment owners | Reuse their decisions, custody and cleanup. Nearest-package integration seeds are not the full affected closure; endpoint edit handles are not durable invocation references. |
| Result inspection | Context descriptors can prevent acquisition; nested storage is also unsupported by current scalar cells. Compact context alone may not make nested values readable. Extend the native owner, preserving original full-batch admission/validation. |
| Telemetry declarations, query adapter and retained investigation packs | Extend their finite dimension/query vocabulary and attributable compositions. A new pipeline, state ledger or metric family is unnecessary where producers already supply the facts. |

Relevant implementation entry points are the [capability catalogue](https://github.com/paul-heyse/pse-arrow/blob/e0a7fe7ea21976f5edef43244e4c04983be2d3e8/scripts/capabilities/catalogue.py),
[coherence](https://github.com/paul-heyse/pse-arrow/blob/e0a7fe7ea21976f5edef43244e4c04983be2d3e8/scripts/capabilities/coherence.py), [native contracts](https://github.com/paul-heyse/pse-arrow/blob/e0a7fe7ea21976f5edef43244e4c04983be2d3e8/scripts/capabilities/native_contracts.py),
[workspace domains](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/workspace_domains.py), [affected selector](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/affected.py),
[native knowledge](https://github.com/paul-heyse/pse-arrow/blob/1bee18378a20670af31838be0d35e3173917fc58/crates/pse-operations/src/knowledge.rs),
[result inspector](../../crates/pse-runtime/src/workflow/result_inspection.rs) and
[telemetry query owner](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/telemetry_investigation.py). Companion explanations identify
their consumed contracts and narrower source pointers.

## Shared target decisions

One canonical declaration supplies capability identity, exposure, requirements, workflows and
guidance. MCP and CLI are consumers of that declaration, not separate implementations. The CLI
may derive the complete callable command set even though live tool visibility is progressive;
it must select and dispatch through the canonical endpoint. Native/deferred routes remain
described as such and do not become callable simply because they are inventoried.

Scientific operations keep Rust meaning, generated directional documents, native prerequisites,
source cuts and original result/evidence custody. Host Pydantic contracts describe host requests
and views. Source/query/test relationships remain with their original producers. Protocol
connection state never becomes workload settlement or scientific success.

Preserve structured identity/facet/relationship discovery and optional tool use. Exact lexical
filtering and language-server symbol operations have useful distinct roles; no semantic
similarity index or ranking is selected. Keep direct source/editor/native routes first-class.
Do not add another general server, rmcp/gRPC boundary, database/KV/agent-memory store, Docker
deployment, broker, automatic hot reload or background scheduler for this scope.

Keep current SurrealDB/MX integration, SSI/statistics/persistence controls, Arrow/DataFusion
finite evidence analysis, source history and execution supervision. CodeMode, protocol tasks,
Apps/UI and multi-question batching remain outside this target. Their library availability does
not establish a consumer here. Ordinary workload resource settings remain environment-owned;
the series introduces no lower compiler-job limit or memory budget.

### Ownership and integration

55a owns host outcome/example conventions, operation contract compatibility and CLI dispatch.
55c owns scientific schema changes and scientific examples consumed by that presentation.
55b owns relationship production/query meaning; 55d consumes those facts without another test
coverage ledger. 55e owns observation/investigation projections, while original execution readers
remain authoritative. 55f assembles acceptance; it does not postpone necessary consumer migration.

Register new operations through the current inventory/catalogue and generated references, with
exact guidance, effects, prerequisites, lifetime and original-reader routes. Where replaced
presentation or adaptation has migrated and targeted tests pass, delete its old callers/tests
in the same implementation slice. Existing independent native commands are not obsolete merely
because MCP gains access to them.

## Opportunity dispositions

These are final delivery/inquiry dispositions. Each opportunity has one primary work owner;
contributing packages do not create another status ledger. Package Outcomes and 55f retain the
actual acceptance boundaries, rather than transferring a pass to unexercised scope.

| Opportunity | Disposition | Work owner | Completion or trigger |
|---|---|---|---|
| [O01](../functional_reviews/mcp-development-effectiveness-2026-10-25.md#O01) | complete | [55a A02](55a-capability-contracts-clients-and-recovery.md#A02) | Faithful schema-derived CLI, same-connection selection, operation compatibility and original references |
| [O02](../functional_reviews/mcp-development-effectiveness-2026-10-25.md#O02) | complete | [55a A03](55a-capability-contracts-clients-and-recovery.md#A03) | Distinct stale attachment/startup/prerequisite/lost-ACK routes, actual host refresh assessment |
| [O03](../functional_reviews/mcp-development-effectiveness-2026-10-25.md#O03) | complete inquiry | [55d D03](55d-editing-validation-and-regression-experiments.md#D03) | Foreground session selected and accepted; no crash-durable adoption |
| [O04](../functional_reviews/mcp-development-effectiveness-2026-10-25.md#O04) | complete | [55a A04](55a-capability-contracts-clients-and-recovery.md#A04) | Explain one actual typed request without silently preparing/admitting it |
| [O05](../functional_reviews/mcp-development-effectiveness-2026-10-25.md#O05) | complete | [55d D01/D02](55d-editing-validation-and-regression-experiments.md#D01) | Original affected closure and comparable prior evidence with reasons/unknowns, no automatic pruning |
| [O06](../functional_reviews/mcp-development-effectiveness-2026-10-25.md#O06) | complete | [55b B02/B03](55b-structured-change-context-and-knowledge.md#B02) | Genuine attributed producer relationships and one useful change-to-evidence inquiry |
| [O07](../functional_reviews/mcp-development-effectiveness-2026-10-25.md#O07) | complete | [55b B01](55b-structured-change-context-and-knowledge.md#B01) | Existing indexed query/explain through typed access with explicit generation/freshness |
| [O08](../functional_reviews/mcp-development-effectiveness-2026-10-25.md#O08) | complete | [55c C01](55c-native-model-and-result-inspection.md#C01) | Exact native declaration/inspection/comparison access with honest admission effects |
| [O09](../functional_reviews/mcp-development-effectiveness-2026-10-25.md#O09) | complete | [55c C02/C03](55c-native-model-and-result-inspection.md#C02) | A representative nested scientific value becomes readable with sufficient context |
| [O10](../functional_reviews/mcp-development-effectiveness-2026-10-25.md#O10) | complete | [55a A01](55a-capability-contracts-clients-and-recovery.md#A01) | Directional examples and meaningful outcome variants; C03 supplies native examples and migration |
| [O11](../functional_reviews/mcp-development-effectiveness-2026-10-25.md#O11) | complete inquiry | [55d D05](55d-editing-validation-and-regression-experiments.md#D05) | Finite JSON migration selected and accepted through shared edit custody |
| [O12](../functional_reviews/mcp-development-effectiveness-2026-10-25.md#O12) | complete inquiry | [55d D07](55d-editing-validation-and-regression-experiments.md#D07) | Existing direct experiment route retained; D08 deferred at its concrete-consumer trigger |
| [O13](../functional_reviews/mcp-development-effectiveness-2026-10-25.md#O13) | complete | [55e E01](55e-diagnostics-and-investigation-handoffs.md#E01) | Instrument-specific filters/grouping preserve the selected population and estimate semantics |
| [O14](../functional_reviews/mcp-development-effectiveness-2026-10-25.md#O14) | complete | [55e E02](55e-diagnostics-and-investigation-handoffs.md#E02) | Explain an exact invocation across original owners without equating telemetry with outcome |
| [O15](../functional_reviews/mcp-development-effectiveness-2026-10-25.md#O15) | complete | [55e E03](55e-diagnostics-and-investigation-handoffs.md#E03) | Question-bearing retained handoff follows original references without replay |
| [O16](../functional_reviews/mcp-development-effectiveness-2026-10-25.md#O16) | complete inquiry | [55e E04](55e-diagnostics-and-investigation-handoffs.md#E04) | Existing exact query/explain/engine readers retained; E05 deferred at its correlation-gap trigger |

### Inquiry decisions

An inquiry names a concrete consumer, compares existing direct/typed routes with the proposed
addition, and records an explicit adoption or deferral with reason, evidence and observable
revisit condition. Deferral is a valid completed inquiry, not a failed feature delivery. The
corresponding delivery package stays deferred until a positive adoption is recorded within
authorized scope. A recommendation alone grants no new effect or architecture permission.
The coordinator disposition links the package's decision rather than duplicating its details.

### Rule decisions

No rule relaxation was selected. The original document-creation boundary remains historical.
The additional CLI consumer fits the existing single-interface direction; no existing scientific or edit-admission guarantee is relaxed.
Before implementation, use the current decision route for a fired deferred-decision trigger,
native boundary/relation changes or a changed preview lifetime/authorization contract. Any actual
rule change must be explicitly presented to the maintainer and confirmed before dependent work.
Ordinary exact dependency additions/tooling changes require no ADR solely for that reason.
Implementation extended proposed ADR-0224 before native inspection v2 changes; its formal
status remains proposed. Maintainer-authorized enduring publication is recorded at blueprint
revisions 272/273. Existing accepted ADRs were not amended.

## Dependencies and execution route

Start with A01's contract/presentation foundation and the bounded A03 host-refresh assessment.
A02 consumes A01's operation basis/presence/result conventions. B01/B02, C01/C02, D01 and E01
can develop independently through their original owners; they do not require a finished CLI.
B03 consumes actual B02 assertions and B01 indexed access when the selected inquiry uses it.
C03 participates in C01/C02 with required native/public/generated migrations in each coherent
slice; its final reconciliation follows both deliveries. D02 consumes D01's actual
selection. E02 uses original invocation readers and incorporates E01 dimensions where useful;
E03 consumes its reference vocabulary. D03/D05/D07/E04 settle their own optional branches.

Coordinate shared edits to catalogue/inventory/native contracts/generation even when semantic
packages are independent. Integrate necessary declarations, consumers, generated outputs,
deletions and targeted checks in coherent commits. A plan document is not a synchronization
barrier, and conceptual records do not require a process/service per distinction.

F01 joins the chosen capabilities in actual agent journeys; F02 runs relevant assembled/static
checks once functional scope is implemented; F03 publishes enduring contracts and supported
limits. There is no required fresh exhaustive feature matrix, solver fleet or performance
campaign merely because the interface grows. Broaden checks only for changed behavior or a
diagnosed unresolved risk.

## Verification and Outcome — 2026-10-25

**Implemented:** all selected A01–A04, B01–B03, C01–C03, D01–D06, E01–E03 and F01–F03 delivery
is complete. O03/O11 inquiries selected foreground session editing and the existing finite
structural migration; both were delivered and actually exercised. O12/O16 inquiries selected
existing direct routes, with D08/E05 explicitly deferred at their package-owned observable
triggers. O01–O16 therefore have no unresolved selection or required delivery in this series.

**Tested:** [55f's Outcome](55f-qualification-and-publication.md#verification-and-outcome--2026-10-25)
owns the composed local Linux qualification: 2,277 setup cases, 50 rebuilt public-native Python
unit cases, selected force-validated Rust/native/model/result/knowledge controls, actual canonical
MCP/CLI journeys, corrected static failures and manual checks against zero. It records exact
commands, original evidence, unavailable historical fixture, repaired failures and exclusions.
These are selected interface/developer/scientific-read boundaries, not a fresh whole-product
qualification or a measured productivity/speed improvement.

The user additionally requested an implementable host restart route. Guarded `just capability
reconnect` uses the installed Codex app-server API and queues a global refresh; a fresh actual
Codex attachment subsequently exercised current model-exposed tools. The root's already-active
old attachment remains a distinct observation, with turn-boundary/new-context refresh documented.
No automatic effect replay, configuration rewrite or hot reloading was added.

**Corrected mistakes and deliberate decisions:** actual journeys exposed missing child environment
inheritance, duplicate governing-section extents, version-two native custody admission, and recursive
composite metadata that exhausted the unchanged 64 KiB inspection bound. Repairs preserve original
owners, exact meaning and explicit partial-context limits. A tuple/list test-harness comparator
was corrected by comparing original JSON, preserving its initial failed receipt. Overlapping
same-target Clippy variants encountered the already-documented Cargo lock cycle; cancelling only
the owned blocked variant and running those variants sequentially restored progress with 32
compiler jobs and no memory cap. The narrow schema-derived CLI emitter, foreground-only custody
and optional feature deferrals are recorded in their package Outcomes.

**Publication and lifecycle:** enduring contracts/operating routes are at blueprint §21.1/§21.5,
§23.1/§24.1/§24.2 and the capability interface/environment guides. Plan55 is removed from Current
publication. This highest-numbered series and consumed review remain retained for qualification
and proposed-ADR readers; they authorize no new work. Native scope tables retain one status owner.
All final owned test contexts/scopes drained and their fixture/initializer copies were retired by
the original owner. Earlier failed-fixture evidence and unrelated data/caches are preserved;
there is no remaining worktree or owned background campaign for this series.
