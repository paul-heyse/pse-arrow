---
title: "Plan 52: Agent effectiveness and engineering workflows"
status: in-progress
date: 2026-10-23
adrs: []
review_sources: [docs/functional_reviews/agent-effectiveness-opportunities-2026-10-23.md, docs/functional_reviews/agent-tool-discovery-and-access-2026-10-24.md, docs/functional_reviews/unified-fastmcp-capability-interface-2026-10-24.md, docs/plan_improvement_reviews/test-architecture-remaining-2026-10-24.md, docs/functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md, docs/functional_reviews/opentelemetry-and-agent-observability-2026-10-24.md]
scenario_sources: [docs/functional_reviews/agent-effectiveness-opportunities-2026-10-23.md]
doc_role: plan
doc_owner: docs/plans/52-agent-effectiveness-and-engineering-workflows.md
doc_topics: [current-work, design, environment, validation, diagnostics, libraries]
---

# Plan 52: Agent effectiveness and engineering workflows

## Purpose, scope and authority

**Proposed:** make Codex substantially more effective at understanding, designing, changing,
verifying and investigating this system. Compose the implemented Plans50–51 foundations into
useful workflows and add the capabilities identified by the
[source review](../functional_reviews/agent-effectiveness-opportunities-2026-10-23.md).
The original O01–O30/RC01–RC09 and the subsequent unified-interface review's O01–O10/
Q01–Q06/RC01–RC05 have separate source-qualified routes below. The latter review supersedes
the earlier discovery review's optional developer-server/Typer/CLI-first framing; it does not
create another backlog from that earlier review. Meaningful alternatives retain explicit dispositions.
A scheduled opportunity is not a defect finding, and authoring is not implementation acceptance.
The [usability review](../functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md)
adds its own O01–O17/RC01–RC03 below, without replacing either earlier source's identities.
The [OpenTelemetry review](../functional_reviews/opentelemetry-and-agent-observability-2026-10-24.md)
adds separately qualified O01–O20/RC01–RC02 at the
[telemetry amendment](#opentelemetry-target-and-foundation-assessment--2026-10-24).

The operator authorized this plan series and explicitly accepted RC01–RC09 on 2026-10-23,
with the review's conditions intact. This coordinator owns package progress, opportunity
and inquiry dispositions, shared decisions, cross-plan boundaries and overall completion.
Companions own detailed designs and acceptance, linking here for current state. Architecture
and ADRs retain contract/rationale authority. The maintainer subsequently authorized detailed execution of all 52a/52b on 2026-10-23,
including their required FND00 and small G01 prerequisite slices. On 2026-10-23 the maintainer also
authorized all C01–C08/D01–D04 and their necessary prerequisite slices. The maintainer
subsequently authorized E01–E04 and their necessary integration fixes. Other production
packages remain planned. On 2026-10-24 the maintainer authorized authoring
52i–52k and reconciling this series to the selected unified FastMCP target. That instruction
does not authorize implementing the new interface.

Subsequent 52i/52j/52k execution checkpoints below own their actual authorization and delivery.
On 2026-10-24 the maintainer authorized authoring 52l–52o and integrating the usability review
into remaining Plan52 scope. RC01 was explicitly accepted for planning; RC02/RC03 were selected
for investigation only. This authoring instruction does not authorize their production delivery.

On 2026-10-24 the maintainer explicitly accepted the OpenTelemetry review's RC01 and RC02
for planning and authorized authoring 52p–52s plus the affected existing-plan amendments.
The selected target includes persistent native Collector/OpenObserve and shared export/retrieval
contracts. This is plan authoring, not authorization to deploy or implement those capabilities.

**Interface-checked baseline:** `cbda2efc6` on main. Plans49–51's delivered functionality and
prior qualification remain at their current owners; the review is their evidence-informed
extension, not a request to repeat them. Core3.4, Efficient Architecture Heuristics1.0,
Test Assessment1.0, Process Simulator1.5 and the repository binding shape the focused
foundation assessment. It is not a broad diagnostic audit or a new formal review.

## Operator selections and support boundaries

- Free-to-run additions, preferably downloaded and local; existing Codex access is baseline.
  No new paid API, hosted model, Cloud database or commercial service is a dependency.
- Tailor to Paul, Codex and this workstation. Preserve sixteen compiler jobs and ordinary
  unbounded builds; scientific/reference allocations retain their semantics. Add no global
  memory/concurrency governor. Overlapping Cargo graphs in one target remain unsafe.
- Generated data and derived indexes can be regenerated. Preserve artifacts only for selected
  investigations/current references; no blanket historical archival or compatibility migration.
- One FastMCP entry point is selected for agent capabilities, with exact domain/workflow
  navigation, progressive reveal and useful instructions. No separate developer endpoint or
  semantic/ranked capability search. Existing literal/regex source search and LSP symbol
  navigation retain their native meaning. Ordinary shell/Git and native execution owners remain.
- Any additional agent CLI is a conditional Cyclopts derivative of canonical FastMCP definitions,
  never an independent Typer tree. Readable/JSON native results remain useful; local models,
  containers and interactive UI retain their existing conditional selection boundaries.
- Scientific authority, native result truth, source owners, cancellation/drain, independent
  expectations and Arrow force-validation remain intact. No new solver, model authority,
  generic scheduler, writable handoff ledger or second outcome store is selected.

## Foundation assessment and combined target

Existing `workspace_context`, language bridges, `affected`, activity/build observations,
validation evidence, exact diagnostic reproducers and Plan51 evidence analysis are substantial.
Extend their real operations instead of layering another system around their names.

| Foundation | Assessment and target |
|---|---|
| Source/context | Current context composes native scope and recipes; boundary search supplies text leads. 52a derives actual declaration/generator paths and task packets; it must not treat all generated files as one catalogue's outputs. |
| Language editing | mcpls0.8.0 provides documented edit-returning operations; actual Rust/Pyrefly support is not qualified by prior navigation. A03 exposes completeness and unsupported cases before promising rename. |
| Knowledge | Published Pagefind search exists. B01 selects exact owner-native concepts with source-fresh lexical discovery; B04 accelerates those structured queries in a separate derived view. Scientific research datasets/selections remain untouched. |
| History | The bounded retrospective scans session creation-date partitions and exact cwd, missing older active sessions/worktrees. B02 corrects discovery and logical deduplication without mandatory raw transcript ingestion. |
| Correctness | Proptest, references, coverage and narrow Miri routes exist. 52c adds discriminating semantic challenges and scoped investigations, not a mandatory test-technique campaign. |
| Execution | Gate-level reuse and result readers exist. D01 adds watch composition; D02 narrows recovery to completed feature combinations with honest source-bound reuse. |
| Build identity | Raw Cargo/toolchain TOML hashes select custom-server installation namespaces. D03 assesses effective equivalence separately from provenance without changing canonical/scientific hashes. |
| Fixtures | Private SurrealKit namespaces and ownership exist; seed SQL is explicitly refused. E02 adds deterministic fixture population with settlement/read-back rather than assume arbitrary imports are supported. |
| Evidence | Six finite Arrow/DataFusion questions and a disposable projection exist. F01 adds explicit pairing/correspondence, preserving unknowns and source populations. |
| Traces | Server→Surrealism parentage exists; application-originating context is not established. F02 owns that seam. The telemetry amendment adds continuous collection/query alongside finite capture, preserving independent Plan40 qualification. |

The resulting workflow starts with a task/source cut, discovers owners and consequences, makes
reviewable edits, selects revealing tests, watches owned work, and compares actual outcomes.
Selected research and experiments remain navigable through derived links. Default reads do not
start services, compile extensions, execute science, reopen threads or reconcile mutable owners.

### Unified-interface foundation assessment — 2026-10-24

The authoring baseline is `9f899dcc0c52a693ec05929ec6c1deabdf6b005a` plus inspected current
owners. Concurrent Plan53 source/environment edits are outside this authoring scope. The
[unified review](../functional_reviews/unified-fastmcp-capability-interface-2026-10-24.md)
already assessed the relevant native/Pydantic/schema/provider boundaries; focused source reads
confirm that `workspace_context` composes native scope/recipe facts while `codex_session`
currently attaches separate Rust/Python/database connections. These are foundations to adapt,
not evidence of an implemented unified endpoint. No living diagnostic audit is refreshed.

Reuse existing source readers, native decoders, scientific/diagnostic outcomes, workload admission,
selected-service fences and result handles. Change their exposure and parser boundaries where
useful. FastMCP owns one component/exposure declaration; Rust registries/catalogues own native
schemas; Pydantic owns host-tool requests/results; Markdown owners supply instructions. Generated
help/CLI derives from components, not a parallel schema. One STDIO process per attachment is the
initial deployment. Browse/read bootstrap is outside ordinary workload admission; effectful
operations retain their native environment/admission. Services are acquired only when needed.

AP-01–AP-07 and the review's execution-fit reasoning shape this combined target: domain metadata
must govern real selection/dispatch, native validation must precede effects, and compact discovery
must avoid repeated global reconstruction. Core3.4, Heuristics1.0, Test Assessment1.2 and Process
Simulator1.5 apply to this extension. No endpoint-lifetime native allocation, scientific mirror,
new scheduler, duplicate evidence store or JSON-per-scalar scientific inner loop is selected.
Use coarse native operations and their selected readers. No performance benefit is established.

Keep PyO3 plus Rust-authoritative JSON Schema. gRPC is unselected because schema fidelity is
already available without an additional service/ProtoJSON translation/lifetime. Revisit only
for a concrete process-isolation/deployment need or meaningful complete replacement; no equivalent
parallel production route. Buf descriptor-to-JSON-Schema remains an unqualified alternative,
not scheduled infrastructure. OpenAPI/FastAPI providers, dynamic schema conversion and custom
core-schema hooks are conditional on a real existing representation/API and fidelity checks.
Tasks/Docket, CodeMode, sampling, MCP Apps, experimental Skills and custom extensions have no
selected need at that authoring baseline. The subsequent usability extension below explicitly
schedules conditional CodeMode/tasks/operator interaction inquiries without adopting them.
Ordinary resources/prompts/skill providers fit the target. OTel, response bounds,
selective preparation and proxy providers have concrete owned routes in 52i–52k. The library
mechanism decisions remain explained there and in the source review rather than another register.

### Usability foundation assessment and combined target — 2026-10-24

**Interface-checked:** this amendment starts from completed CAP/ADP/MCP delivery and the usability
review's recorded baseline, refreshed against `deeb37cc4` and inspected concurrent Plan53 reader/
worker edits. The preparation commit separates Python stages; in-progress reader changes are
not promoted to verified prerequisites. Implementation must refresh these owner contracts before
migrating their consumers. Core3.4, Heuristics1.0, Test Assessment1.2 and relevant Process
Simulator1.5 constraints apply. No broad diagnostic audit or product campaign was run for authoring.

The present endpoint has meaningful foundations: exact discovery, generated native schemas,
strict host validation, source fences, fixed workers, original handles and lazy specialists.
Its query callable does not express every owner facet; generic bounded responses can alter payload
structure after acquisition; full guide delivery and sibling import closures amplify ordinary
selection/change. These are new evolution targets rather than reasons to reopen earlier Outcomes.
52l improves requests/results/diagnostics at their authorities; 52m improves exact guidance,
ownership and operation availability; 52n exposes selected preparation, authentic progress and
preview release. 52o compares optional composition/tasks only after useful ordinary operations exist.

Keep scientific schema/version/admission with Rust and generators, host tooling contracts with
their Pydantic owners, and original outcome/custody/drain with native readers. Typed family responses
do not imply a universal envelope. Push selected detail upstream while retaining mandatory source,
denominator/ambiguity/completeness facts. Use actual fixed-worker declarations and coherent serving
sources instead of name guesses or an incomplete dependency whitelist. No automatic build-on-read,
silent eviction, generic JSON query language, global retry/cache/response-limiting middleware or
duplicate scheduler/store/server is selected. Library mechanisms and precise limits live with the
packages in 52l–52o and the review, including candidates not selected for adoption.

Plan53d owns independent environment/extension/stub preparation and Plan53e E05 protected assessment
readers/custody. 52n consumes the former; F01/52l migrate relevant selected-answer consumers with the
latter. Actual owner contracts, callers and evidence determine readiness, not a whole-plan completion
barrier. F02/I11 trace continuity, F03's separate selection, I14's shutdown inquiry and C04's denied
coverage-guided acceptance remain independently owned. No interface improvement silently closes them.

### Shared meanings and physical composition

| Meaning | Owner and consumed behavior |
|---|---|
| Task context | A01 composes native status, Git/dirty state and selected evidence; no independent mutable checkpoint. |
| Relationship basis | A02 publishes declared generator links and semantic references separately from text/inferred leads; absence is not proof of no impact. |
| Source membership | B01/B04 retain exact selected revisions, paths, roles, dirty overlays and source cuts; historical recall never becomes current authorization. |
| Edit result | A03/A04 retain expected source version, full/partial/unsupported status and preview; generated edits point to their declarations. |
| Run observation | D01 reads exact owner handles and terminal truth; lost observation is not successful execution. |
| Reuse | D02/D03 prove only their stated unchanged premises; reused evidence and historical executables remain distinct from fresh current execution. |
| Experiment | E03 composes existing invocation/reproducer/measurement identities and explicit predicates; F01 compares their results, not execution ownership. |
| Scientific reduction | C06 owns a separate property/correspondence claim and never weakens strict original reproduction. |
| Agent interaction | G01 describes existing owners' actions/results; a common vocabulary does not require a universal result type or dispatcher. |

Parse selected sources once per operation, project compact facts, and hydrate raw detail only
when requested. B04 uses the existing separate tooling database/generation lifecycle for a
rebuildable knowledge projection; no second store of accepted facts. O02 paths can work in
memory before persistence. B01's structured native queries and literal/lexical source discovery retain their separate
meanings; capability access adds no ranked retrieval or model requirement.
Avoid one LSP request, database query, subprocess or full-history scan per returned row.

## Rule decisions

Unqualified RC IDs in the following original table belong to the 2026-10-23 review only.
The later review has a separate [accepted rule table](#unified-interface-rule-decisions).

All rows were **accepted by the operator on 2026-10-23** as conditional target directions.
They do not accept an ADR or establish a probe result. FND00 publishes each required contract
slice through its actual route before dependent implementation; routine tooling needs no new ADR.

| Decision | Accepted consequence | Route and order |
|---|---|---|
| RC01 | Derived context/index authority, dirty overlays and historical attribution; no second policy/plan owner | A01/B01/B02 owner guides; short ADR and registry declaration before B04's new native development-knowledge relation family; architecture changes through design publication. |
| RC02 | Explicit completeness/refusal semantics for semantic edits and transformations | A03/A04 tool contracts and environment guide; keep existing Python wire/generation rules. |
| RC03 | Combination-level qualification reuse with exact enumeration, context, invalidation and reused-result labels | D02 extends existing validation reuse. FND00 checks ADR-0170/validation ownership and publishes the narrower reuse unit; any actual governance change takes ADR plus bounded required review before adoption. No qualification waiver. |
| RC04 | Investigate semantic build configuration identity separately from raw provenance | D03 owns installation-equivalence decision and environment guide; ordinary build tooling, with unknown semantics retaining conservative invalidation. Canonical hash contracts are unchanged. |
| RC05 | Historical replay and scientific reduction retain distinct identities and claims | E01/C06; new runtime document/relation contracts take the applicable short ADR/catalogue route before introduction. No change to original reproduction truth. |
| RC06 | Typed evidence comparison and development-knowledge views retain supplied-source limits and native authority | F01 extends existing evidence operations; B04 new derived family follows RC01's declaration/decision route. Publish blueprint §19.4/§24.1 changes through design route if needed. |
| RC07 | Thin agent actions/results retain ownership, continuation and observation-failure meaning | G01/D01/G04 ordinary tooling and guides; no new scheduler or terminal authority. |
| RC08 | Local models/services/sandboxes activate only for justified consumers | B05/E04/G05 bounded inquiries and environment guide; free-local limits remain binding. |
| RC09 | Original finite application-to-database correlation, coordinated with Plan40; continuous deployment is added by the separately accepted telemetry RC01 | F02 retains the transport seam and I11; 52p–52s supply shared telemetry and retrieval. Publish blueprint §23.1 through the design route without transferring Plan40's campaign. |

## Series and cross-plan ownership

| Companion | Owned target |
|---|---|
| [52a](52a-context-navigation-and-semantic-editing.md) | Task packets, precise cross-language paths, semantic editing, codemods and design consequences |
| [52b](52b-local-knowledge-and-research.md) | Source/history retrieval, exact library research and derived development knowledge |
| [52c](52c-behavioral-and-scientific-verification.md) | Discriminating test techniques, numerical investigations, reduction and safety diagnosis |
| [52d](52d-execution-and-qualification-efficiency.md) | Owned transition watch, feature resumption, build equivalence and host pressure |
| [52e](52e-reproducible-experiments-and-isolation.md) | Historical replay, database fixtures, regression experiments and optional isolation |
| [52f](52f-evidence-analysis-and-trace-correlation.md) | Explicit outcome comparisons and actual database trace continuity with continuous/finite investigation |
| [52g](52g-agent-and-operator-workflows.md) | Agent-facing composition, skill discovery/evaluation, integration and optional operator UI |
| [52h](52h-qualification-and-publication.md) | Actual-consumer reconciliation across 52a–52s, assembled acceptance, scope-end checks and publication |
| [52i](52i-canonical-capabilities-and-progressive-reveal.md) | Canonical exposure model, exact navigation, deterministic reveal and progressive instructions |
| [52j](52j-native-contracts-and-tool-adaptation.md) | Native schema/admission fidelity, host contracts, whole-repository tool adaptation and retirement |
| [52k](52k-unified-entry-point-and-derived-interfaces.md) | One attachment, lazy integration, loaded revisions, derived consumers and interface observation |
| [52l](52l-typed-requests-results-and-diagnostics.md) | Constructible domain requests, typed selected answers, recoverable diagnostics and shape-first scientific result inquiry |
| [52m](52m-progressive-guidance-and-operation-availability.md) | Compact progressive guidance, accurate owners, operation dependencies/restart and supporting instruction resources |
| [52n](52n-readiness-progress-and-edit-lifecycles.md) | Selected readiness/preparation, authentic early progress/handles and explicit preview release |
| [52o](52o-conditional-composition-and-protocol-tasks.md) | Conditional exact-discovery read composition and task-protocol inquiries, with explicit adoption decisions |
| [52p](52p-shared-telemetry-and-native-propagation.md) | Shared telemetry identity/configuration, Python/Rust provider lifetimes, native propagation and stage/diagnostic projection |
| [52q](52q-local-telemetry-pipeline-and-health.md) | Persistent native Collector/OpenObserve, processing/queues, independent health, retained history and recovery |
| [52r](52r-operational-metrics-and-diagnostic-integration.md) | Population-correct operational/preparation/database/engineering metrics and sampling/distribution/exemplar meaning |
| [52s](52s-telemetry-queries-and-agent-investigations.md) | Typed backend queries, canonical FastMCP investigations, profile escalation, offline custody and selected watch/alert outcomes |

Plans50–51 remain completed foundations. Their qualified interfaces are reusable; their counts
are not Plan52 acceptance. Plan32 retains document/artifact lifecycle ownership. Plan40a–d's
focused deliveries exist; Plan40e retains its assembled profile/locality campaign and finding
closure. F02 neither completes that campaign nor reinstates withdrawn environment/provenance
proof work. Plan28's separate scientific campaigns are not prerequisites merely because a
representative native case is used here. Do not edit/retire those plans as part of authoring.

## Remaining test-architecture integration — 2026-10-24

The operator authorized plan revisions from the
[remaining-architecture review](../plan_improvement_reviews/test-architecture-remaining-2026-10-24.md),
with completed 52k as the starting condition before remaining Plan53 execution. The
[52k Outcome](52k-unified-entry-point-and-derived-interfaces.md#verification-and-outcome--2026-10-24)
now records its completed component and actual-host evidence. Existing MCP package, Q01–Q06 and
qualification owners remain unchanged; this amendment consumes delivery without extending its
claims to the remaining Plan52/Plan53 assembled scope.

**Proposed:** 53d owns preparation changes; 53e owns immutable invocation facts, protected reader/
custody migration, fixture settlement and selective discovery; 53f owns actual effect/exclusion
scope. Plan52's F01/G01/G02/G05 consumers migrate alongside relevant reader/contracts, and H01/H02
coordinate authentic joined journeys with 53g. Preserve separate native/scientific/host/custody/
cleanup assertions and do not sum shared executions into broader coverage. PI04's native question
section selection/completeness belongs to F01; PI10 supplies the joined qualification route.
These are linked amendments within existing owners, not transfer of 52k delivery or a dependency
on completing all Plan52 work.

PI09 is **separately selectable**, per the operator's 2026-10-24 choice: [52f F03](52f-evidence-analysis-and-trace-correlation.md#F03)
fully describes a finite multi-question inquiry over the existing typed question vocabulary and
one source cut. It stays deferred until a concrete multi-question consumer and separate selection
justify delivery. Its unselected implementation does not block original Plan52 or Plan53 closure.
The existing F01 single-question path and G05 optional-view inquiry retain their meanings.
The review's separate endpoint-shutdown observation is I14 below; it is not a Plan53 amendment or
an automatic reopening of delivered MCP01/Q03. No additional rule change is introduced beyond
already accepted decisions; new incompatible contract impacts use the normal explicit decision route.

## Opportunity dispositions

Rows retain their native package owners; selected implementation and inquiry outcomes are recorded
below. These source IDs refer only to the original
[2026-10-23 review](../functional_reviews/agent-effectiveness-opportunities-2026-10-23.md), not
identically numbered opportunities in either 2026-10-24 review. Conditional work needs
an explicit supported decision and fallback, not silence under a completed-series label.

| Opportunity | Disposition | Work owner | Required behavior or disposition |
|---|---|---|---|
| O01 | resolved | [A01](52a-context-navigation-and-semantic-editing.md#A01) | Source-linked task continuity and rationale |
| O02 | resolved | [A02](52a-context-navigation-and-semantic-editing.md#A02) | Declared cross-language/generator paths, with uncertain leads separate |
| O03 | resolved | [A03](52a-context-navigation-and-semantic-editing.md#A03) | Qualified semantic edit preview and honest unsupported cases |
| O04 | resolved | [A04](52a-context-navigation-and-semantic-editing.md#A04) | Tested structural transforms and declaration-aware changes |
| O05 | resolved | [B01/B05](52b-local-knowledge-and-research.md#B01) | Structured native retrieval, lexical discovery and bounded local semantic decision |
| O06 | resolved | [B03](52b-local-knowledge-and-research.md#B03) | Exact-version research and evolution packets |
| O07 | resolved | [A05](52a-context-navigation-and-semantic-editing.md#A05) | Consequences of actual design alternatives |
| O08 | resolved | [C01](52c-behavioral-and-scientific-verification.md#C01) | Source/coverage correspondence, missing behavior and exclusions |
| O09 | resolved | [C02](52c-behavioral-and-scientific-verification.md#C02) | Independent semantic properties and shrinking action sequences |
| O10 | resolved | [C03](52c-behavioral-and-scientific-verification.md#C03) | Selected plausible-wrong implementation challenges |
| O11 | in-progress | [C04](52c-behavioral-and-scientific-verification.md#C04) | Finite pure-boundary fuzzing and reduced regressions |
| O12 | resolved | [C05](52c-behavioral-and-scientific-verification.md#C05) | Legal-step numerical/derivative reference workbench |
| O13 | resolved | [C06](52c-behavioral-and-scientific-verification.md#C06) | Source-aware reduction with separate preserved-property claim |
| O14 | resolved | [C07/C08](52c-behavioral-and-scientific-verification.md#C07) | Focused lifetime instrumentation and controlled schedule inquiry |
| O15 | resolved | [D01](52d-execution-and-qualification-efficiency.md#D01) | Resumable observation of actual owned transitions |
| O16 | resolved | [D02](52d-execution-and-qualification-efficiency.md#D02) | Exact feature enumeration and safe per-combination reuse |
| O17 | resolved | [D03](52d-execution-and-qualification-efficiency.md#D03) | Safe effective-build equivalence or conservative explained fallback |
| O18 | resolved | [E01 outcome](52e-reproducible-experiments-and-isolation.md#targeted-execution-outcome--2026-10-24) | Actual selected remapped historical replay and protected retention; unsupported native closure refuses |
| O19 | resolved | [E02 outcome](52e-reproducible-experiments-and-isolation.md#targeted-execution-outcome--2026-10-24) | Actual independently settled v2 populations, permission/query equivalence, interrupted read-only settlement and isolated cleanup |
| O20 | resolved | [D04](52d-execution-and-qualification-efficiency.md#D04) | Actual pressure/placement facts and explicit experiments |
| O21 | resolved | [E03 outcome](52e-reproducible-experiments-and-isolation.md#targeted-execution-outcome--2026-10-24) | Actual Git/Rust search found known bad revision through an unsupported revision; noisy retained comparison inconclusive |
| O22 | resolved | [E04 outcome](52e-reproducible-experiments-and-isolation.md#targeted-execution-outcome--2026-10-24) | Optional Bubblewrap isolation with actual server/library parser consumers, owned drain and cleanup |
| O23 | scheduled | [F01](52f-evidence-analysis-and-trace-correlation.md#F01) | Paired comparisons preserving unmatched subjects and denominators |
| O24 | resolved | [B04](52b-local-knowledge-and-research.md#B04) | Derived development-knowledge/experiment links, separate from scientific research |
| O25 | scheduled | [F02](52f-evidence-analysis-and-trace-correlation.md#F02) | Actual application/database propagation with explicit unsupported edges; the telemetry amendment expands deployment and does not permit a missing required edge to close the stronger target |
| O26 | scheduled | [G01/G02](52g-agent-and-operator-workflows.md#G01) | Shared outcomes and selected unified FastMCP workflows; implementation in CAP/ADP/MCP packages |
| O27 | scheduled | [G03](52g-agent-and-operator-workflows.md#G03) | Progressive instruction access and representative agent evaluations |
| O28 | resolved | [B02](52b-local-knowledge-and-research.md#B02) | Activity-based session discovery, attribution and logical deduplication |
| O29 | scheduled | [G04](52g-agent-and-operator-workflows.md#G04) | Ownership-aware integration and reviewable commit selection |
| O30 | scheduled | [G05](52g-agent-and-operator-workflows.md#G05) | Bounded interactive investigation decision; headless reports always usable |

## Inquiry outcomes and remaining questions

A failed candidate selects the stated alternative; it does not erase the underlying required
capability. Record the actual result here and update its owning package before dependent work.
Tool compatibility is separate from functional delivery and measured benefit.

| Inquiry | State | Owner | Decision / completion evidence |
|---|---|---|---|
| I01 | resolved | [A03](52a-context-navigation-and-semantic-editing.md#A03) | FastMCP4.1.3/MCP2.3.0 through mcpls0.8.0: actual Rust cross-file rename and Pyrefly rename/extraction passed; formatting unsupported; full/partial/source-drift guards retained |
| I02 | resolved | [A04](52a-context-navigation-and-semantic-editing.md#A04) | ast-grep0.50.0 contextual shapes plus LibCST1.9.0 qualified imports/scopes support the selected Python3.14 migration and negative examples |
| I03 | resolved | [B01/B05](52b-local-knowledge-and-research.md#B01) | Exact native concepts selected with source-fresh lexical discovery. Qwen/llama.cpp Blackwell batch compatible but mixed usefulness; unused helper/config retired. Missing answers first trigger domain/relationship/query coverage investigation, not a semantic implementation backlog ([52b outcome](52b-local-knowledge-and-research.md#structured-execution-outcome)) |
| I04 | resolved | [B02](52b-local-knowledge-and-research.md#B02) | Select bounded incremental stream adapter. Codex0.163 protocol listing inspected, but no isolated/live read-only App Server runtime qualified; no app-server startup/state repair required |
| I05 | resolved | [C02](52c-behavioral-and-scientific-verification.md#C02) | Existing Proptest action sequences selected and actual stale-provider counterexample shrunk; add another framework only for an uncovered owner-specific behavior |
| I06 | resolved | [C07/C08](52c-behavioral-and-scientific-verification.md#C07) | Actual GDB lifecycle and deterministic production schedules selected; ASan unqualified after task denial; Loom not adopted for std/native path, with a pure-protocol revisit trigger |
| I07 | resolved | [D03](52d-execution-and-qualification-efficiency.md#targeted-execution-outcome--2026-10-24) | Parsed TOML equivalence selected inside complete context; actual cold build/reuse passed; unsupported context and unknown targets refuse |
| I08 | resolved | [E01 outcome](52e-reproducible-experiments-and-isolation.md#targeted-execution-outcome--2026-10-24) | Pure replay adopted; native archive refused before effects. Revisit only for a selected investigation that can close native deployment/ABI/license/config/data dependencies |
| I09 | resolved | [E02 outcome](52e-reproducible-experiments-and-isolation.md#targeted-execution-outcome--2026-10-24) | Deterministic regeneration adopted and actually qualified; logical restore/physical copy unselected. Revisit only for a concrete fixture question whose state cannot reasonably be regenerated |
| I10 | resolved | [E04 outcome](52e-reproducible-experiments-and-isolation.md#targeted-execution-outcome--2026-10-24) | Private directories default; optional Bubblewrap for a concrete permission-isolation consumer. Podman unselected until a different-userspace/dependency requirement recurs |
| I11 | open | [F02](52f-evidence-analysis-and-trace-correlation.md#F02) | Exact SDK task/context/header propagation and actual database/module parentage; consume TEL01/TEL03 and preserve native deadline/drain authority |
| I12 | resolved | [G02](52g-agent-and-operator-workflows.md#G02) | Operator selected one canonical FastMCP endpoint on 2026-10-24; optional direct-versus-MCP adoption inquiry superseded. Actual host, schema and lifecycle questions remain Q01–Q06 below; no delivery claim |
| I13 | open | [G05](52g-agent-and-operator-workflows.md#G05) | Interactive evidence use versus standalone report/plot; no product frontend |
| I14 | open | [Endpoint lifecycle inquiry](#endpoint-shutdown-inquiry) | Assess the separately observed non-cancellation shutdown exception path after 52k delivery; identify reachable failure/independent cleanup guarantees and select a supported disposition or separately authorized remedy. No Plan53/MCP01 completion dependency or delivery claim. |

<a id="endpoint-shutdown-inquiry"></a>
### Endpoint shutdown inquiry — separate post-delivery concern

**Interface-checked:** the improvement review and focused authoring inspection find
`Specialists.close` awaiting language/database connections sequentially without per-sibling error
collection; a non-cancellation exception can stop later close attempts. This shares neither
examined implementation nor required protocol migration with 53e E02's fixture settlement.
MCP01/Q03's recorded owned-drain evidence remains intact; no new executed failure is claimed.

**Proposed — I14:** the Plan52 endpoint lifecycle owner checks this path against the completed
52k implementation and actual connection context managers. Determine which exceptions can escape,
which independent owners remain live, required ordering/cancellation and the desired shutdown
outcome. Source evidence may settle the decision; a bounded shutdown-failure control is useful
only where reachability or protection remains uncertain. Do not run a new host campaign merely
to author the plan or import fixture clock semantics into this different lifetime.

Close the inquiry with a supported no-change judgment, or the exact gap and a separately selected
repair owner/acceptance; preserve uncertainty if it cannot yet be settled. A positive repair
selection must include independent sibling settlement and truthful retained failures, under the
endpoint's actual lifetime contract. It does not silently reopen completed MCP delivery, add a
new required Plan53 package or authorize production correction through this document revision.

## Unified-interface rule decisions

These RC identities belong exclusively to the
[2026-10-24 unified-interface review](../functional_reviews/unified-fastmcp-capability-interface-2026-10-24.md#correspondence-migration-and-rule-impacts).
RC01/RC03/RC04 are accepted through the maintainer's explicit single-surface, native-schema and
tool-refactoring selections. The maintainer explicitly accepted RC02/RC05 together on 2026-10-24.
All decisions below are recorded as accepted target directions on 2026-10-24. They do not change
formal ADR status or establish implementation. FND00 schedules each actual rule slice before
its dependent code; ordinary tooling/dependency edits do not require a placeholder ADR.

| Decision | Disposition | Work owner | Accepted consequence and route |
|---|---|---|---|
| RC01 | resolved | [MCP01/G02](52k-unified-entry-point-and-derived-interfaces.md#MCP01) | One canonical endpoint; specialists internal/lazy. Migrate session/config/environment and concise agent instructions with actual attachment; no mandatory tool use. |
| RC02 | resolved | [ADP01/ADP02](52j-native-contracts-and-tool-adaptation.md#ADP01) | Clarify Python rule for Pydantic host contracts; Rust-generated scientific contracts remain authoritative. Declare exact direct dependency when used; actual public Python-boundary changes take existing ADR/design route first. |
| RC03 | resolved | [CAP01/MCP04](52i-canonical-capabilities-and-progressive-reveal.md#CAP01) | Distinct exposure/native-schema authorities; derived help/CLI and owner metadata. Update declaration/generation guidance with consumers; never edit generated output. |
| RC04 | resolved | [ADP03/ADP05](52j-native-contracts-and-tool-adaptation.md#ADP03) | Refactor difficult tools and retire replaced parsers/scrapers/callers after proven migration. Preserve native truth, validation, source protection, admission and lifetime; amend actual affected contracts through their existing route. |
| RC05 | resolved | [MCP01/MCP03](52k-unified-entry-point-and-derived-interfaces.md#MCP01) | Cheap read/control launch outside workload admission; effectful native admission retained; relevant serving edits require restart/reload. Update tooling/environment/session guidance with implementation. |

## Unified-interface opportunity dispositions

These O IDs belong to the unified-interface review, not the original 2026-10-23 O table.
Scheduled means selected design/implementation work; it does not mean delivered functionality.
O08's CLI portion remains conditional on a concrete consumer and fidelity, while derived reference
is required. Package completion does not automatically resolve an opportunity's other consumers.

| Opportunity | Disposition | Work owner | Required behavior or disposition |
|---|---|---|---|
| O01 | resolved | [CAP01/CAP02/CAP05](52i-canonical-capabilities-and-progressive-reveal.md#CAP01) | Canonical model, classification, faithful native adaptation and complete composed reference delivered; see 52i/52j and the 52k Outcome. |
| O02 | resolved | [CAP03/MCP02](52i-canonical-capabilities-and-progressive-reveal.md#CAP03) | Atomic reveal/reset and actual fresh Codex resource/template/prompt/action/authentic-result journey delivered through the canonical endpoint; see 52k Outcome. |
| O03 | scheduled | [CAP04/G03](52i-canonical-capabilities-and-progressive-reveal.md#CAP04) | Progressive native-owner guidance and planning-only selection delivered; actual editing/task-effectiveness evaluation remains G03. |
| O04 | resolved | [ADP01/ADP02](52j-native-contracts-and-tool-adaptation.md#ADP01) | Typed host contracts and generated native schemas/receivers delivered with malformed/contextual refusal and actual native acceptance; see 52j Outcome. |
| O05 | resolved | [ADP03/ADP04/ADP05](52j-native-contracts-and-tool-adaptation.md#ADP03) | Typed extraction, native/specialist adaptation, single-session migration and explicit all-family native retention delivered; see 52j Outcome. |
| O06 | resolved | [MCP01/ADP04](52k-unified-entry-point-and-derived-interfaces.md#MCP01) | Cheap single bootstrap, lazy actual language/database connections and independent service identity/admission/drain delivered; Q03 resolved. |
| O07 | scheduled | [ADP02/G01/G02](52j-native-contracts-and-tool-adaptation.md#ADP02) | ADP02 outcomes, bounds, handles and cancellation are delivered; assembled G01/G02 interaction consumers retain their own obligations. |
| O08 | resolved | [MCP04](52k-unified-entry-point-and-derived-interfaces.md#MCP04) | Complete generated reference delivered. Maintainer-selected CLI non-adoption retains the concrete-consumer revisit trigger in Q05; no CLI equivalence is claimed. |
| O09 | resolved | [MCP03/CAP02/CAP04](52k-unified-entry-point-and-derived-interfaces.md#MCP03) | Loaded schemas/handlers/local worker closures and scoped guide snapshots are fenced; relevant changes require restart while unrelated source remains operation-owned. See 52k Outcome. |
| O10 | scheduled | [MCP05/F01/F02](52k-unified-entry-point-and-derived-interfaces.md#MCP05) | MCP05 interface correlation, partial/refused outcomes, detached observers and authentic owner references delivered; F01/F02 retain broader comparison/trace inquiry consumers without claimed end-to-end continuity. |



## Unified-interface inquiries

The single-surface direction and a high-functioning Codex integration are settled design premises.
On 2026-10-24 the maintainer confirmed prior successful MCP attachment through environment JSON.
Q01 selects and verifies the best current configuration/presentation; it is not a feasibility gate
or a reason to defer the architecture. These implementation questions still need actual evidence;
the source review's library-client probes do not establish Codex attachment, full native fidelity
or product integration. Preserve failed/inconclusive evidence and update dependent packages here.

| Inquiry | State | Owner | Decision / completion evidence |
|---|---|---|---|
| Q01 | resolved | [MCP02](52k-unified-entry-point-and-derived-interfaces.md#MCP02) | Fresh installed Codex journey passed resources, templates, compact workflow prompts, exact selected action, authentic native result and reset/second selection. Fixed-inventory hosts use canonical invokers and same-content framework projections. Ordinary host approval policy remains inherited; see 52k Outcome. |
| Q02 | resolved | [ADP01](52j-native-contracts-and-tool-adaptation.md#ADP01) | Resolved by the generated directional catalogue and original codecs/receivers, authoring/native separation, local references, numeric/enum/contextual refusals, snapshot/digest fences and six actual native unit/component/integration cases. See 52j Outcome. |
| Q03 | resolved | [MCP01/ADP04](52k-unified-entry-point-and-derived-interfaces.md#MCP01) | Resolved by lazy owner-task connections, actual Python/Rust and selected database journeys, unavailable/stale/cancellation controls and authenticated service/context/kernel drain. See 52j Outcome. |
| Q04 | resolved | [ADP03](52j-native-contracts-and-tool-adaptation.md#ADP03) | Typed shared archive/replay and diagnostic kernels; fixed process boundaries for native execution; original Gate/selection/invocation owners for qualification. Actual archive/replay/action/result consumers and scoped Rust execution passed. See 52j Outcome. |
| Q05 | resolved | [MCP04](52k-unified-entry-point-and-derived-interfaces.md#MCP04) | Maintainer selected CLI deferral on 2026-10-24; required generated reference is delivered. Revisit when a concrete terminal/non-MCP consumer needs structured access beyond existing native commands and canonical MCP; complete generation, fresh activation and absent/null/empty fidelity must qualify before adoption. |
| Q06 | resolved | [MCP03](52k-unified-entry-point-and-derived-interfaces.md#MCP03) | Bounded loaded local source/worker closure, composition verification, per-operation restart fences and scoped guide snapshots passed targeted change/restart controls. Current-source facts and arbitrary dependency readiness retain their native owners; see 52k Outcome. |



## Usability rule decisions

These identities belong only to the
[usability review's rule implications](../functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md#decision-and-rule-implications).
On 2026-10-24 the operator explicitly selected **RC01; investigate RC02/RC03**. Review approval,
authoring and favorable feasibility evidence do not independently authorize either conditional adoption.

| Decision | Disposition | Work owner | Accepted consequence and route |
|---|---|---|---|
| RC01 | implemented | [FND00](#FND00), [52l](52l-typed-requests-results-and-diagnostics.md), [52m](52m-progressive-guidance-and-operation-availability.md) | Accepted for planning on 2026-10-24: evolve request/result/error/instruction contracts with actual callers and derived reference. Preserve Rust scientific authority. Internal tooling uses ordinary changes; actual protected public boundary/diagnostic changes use the existing ADR/design route before dependent implementation. 52l/52m Outcomes record delivered contracts and actual consumers; broader protected changes retain their original decision route. |
| RC02 | deferred | [OPT01](52o-conditional-composition-and-protocol-tasks.md#OPT01) | Investigation selected; real Monty/public CodeMode and three-way reader comparison passed. Exact execution/provenance policy, fresh-host acceptance and selection remain; model-authored production execution is not enabled. See the 52o pause handoff. |
| RC03 | deferred | [OPT02](52o-conditional-composition-and-protocol-tasks.md#OPT02) | Investigation selected; consumer now includes Codex and serving-process restart recovery. Public native-backed task projection works in bounded STDIO fixtures without Docket; native lifecycle integration, actual host acceptance and selection remain. No worker/queue or production task lifecycle is enabled. |

## Usability opportunity dispositions

All O identities in this table refer exclusively to the usability review. Scheduled means planned
capability or inquiry work, not delivered functionality. Each row has one primary disposition owner;
linked consumers retain their own implementation and acceptance. Required inquiry outcomes may be
supported alternatives or conditional non-adoption; they cannot erase required ordinary capabilities.

| Opportunity | Disposition | Work owner | Required behavior or disposition |
|---|---|---|---|
| O01 | resolved | [CON01](52l-typed-requests-results-and-diagnostics.md#CON01) | Eleven owner-defined tagged alternatives/facets, independent examples and migrated callers; see the 52l Outcome. |
| O02 | resolved | [CON02](52l-typed-requests-results-and-diagnostics.md#CON02) | Fourteen typed selected host answer families, actual serialization schemas and explicit population/detail/refusal; see the 52l Outcome. |
| O03 | resolved | [NAV01](52m-progressive-guidance-and-operation-availability.md#NAV01) | Compact attributable selection receipts, deliberate complete guide reads and declared useful workflow steps. |
| O04 | resolved | [CON03](52l-typed-requests-results-and-diagnostics.md#CON03) | Safe owner-native diagnostic/location/recovery projection delivered with actual native/Python admission controls; see the 52l Outcome. |
| O05 | resolved | [NAV02](52m-progressive-guidance-and-operation-availability.md#NAV02) | Accurate module/package/worker owners and local extension through actual declarations; unresolved leads remain uncertain. |
| O06 | scheduled | [F01](52f-evidence-analysis-and-trace-correlation.md#F01) | Coordinate demand-shaped native evidence/assessment readers with 53e E05; CON02 migrates other relevant host-reader consumers. Preserve denominator/ambiguity and protected continuation, not JSON-after-truncation queries. |
| O07 | resolved | [CON04](52l-typed-requests-results-and-diagnostics.md#CON04) | U01 selected native schema/readable inspection plus exact IPC; real nineteen-column protected-result acceptance passed; see the 52l Outcome. |
| O08 | resolved | [LIFE01](52n-readiness-progress-and-edit-lifecycles.md#LIFE01) | Selected passive/live readiness and explicit preparation consumers of actual Plan53d stages, with no duplicate build authority. |
| O09 | resolved | [NAV03](52m-progressive-guidance-and-operation-availability.md#NAV03) | Relevant operation/worker dependency and restart boundaries, coherent source capture, unaffected operation availability. |
| O10 | resolved | [LIFE02](52n-readiness-progress-and-edit-lifecycles.md#LIFE02) | Authentic early owner handles/stages, bounded foreground wire progress where supported and native-reader fallback. |
| O11 | resolved | [LIFE03](52n-readiness-progress-and-edit-lifecycles.md#LIFE03) | Explicit preview retention status/release; preserve siblings and admitted work through actual settlement/reconciliation. |
| O12 | scheduled | [G01](52g-agent-and-operator-workflows.md#G01) | Meaningful recurring typed compositions using F01/native kernels; G02 actual workflow consumer. F03 remains separately selectable. |
| O13 | resolved | [NAV04](52m-progressive-guidance-and-operation-availability.md#NAV04) | Coherent supporting skill resources and U02 exact completion/provider decision for a real consumer, with visibility and lazy source contracts. |
| O14 | scheduled | [G03](52g-agent-and-operator-workflows.md#G03) | Independent schema/example/error/completeness tests and representative actual-agent evaluation; H02 owns assembled host/native qualification. |
| O15 | scheduled | [OPT01](52o-conditional-composition-and-protocol-tasks.md#OPT01) | U03 bounded real-Monty/canonical-dispatch inquiry delivered; actual host/final policy remain at the pause handoff. RC02 still required for production adoption. |
| O16 | scheduled | [OPT02](52o-conditional-composition-and-protocol-tasks.md#OPT02) | U04 library/native-owner and real STDIO fixture evidence delivered; restart recovery and integrated native service design remain. RC03 still required; no second scheduler/outcome truth. |
| O17 | scheduled | [G05](52g-agent-and-operator-workflows.md#G05) | Extend I13 to typed headless resources versus prompts/elicitation/Apps for a concrete operator decision, retaining ordinary access and native interpretation. |

## Usability inquiries

These U identities are new; existing I13 owns operator interaction and I14 owns endpoint shutdown.
Inquiry closure records its evidence and consequence here. Conditional implementation is added only
after its actual selection; non-adoption must identify the supported route and observable revisit need.

| Inquiry | State | Owner | Decision / completion evidence |
|---|---|---|---|
| U01 | resolved | [CON04](52l-typed-requests-results-and-diagnostics.md#CON04) | Selected bounded native schema/physical-value inspection alongside exact IPC under proposed ADR-0224, published before implementation. Real registered nineteen-column storage/read/IPC/lifetime assertions passed; schema inquiry does not itself validate stored IPC or scientific eligibility. |
| U02 | resolved | [NAV04](52m-progressive-guidance-and-operation-availability.md#NAV04) | Captured bounded explicit skill resources delivered and exercised by actual Codex. Retain the lazy coherent owner; no eager/live Skills provider or unused completion for argument-free prompts. Revisit native Skills for an actual negotiation consumer compatible with capture, or completion for a useful parameter and real consuming host. See the 52m Outcome. |
| U03 | open | [OPT01](52o-conditional-composition-and-protocol-tasks.md#OPT01) | Three-way comparison, exact discovery, direct/indirect dispatch guards, real Monty and full callback deadline passed. Retained provenance increased bytes in the tested CodeMode answer; typed recurring composition was smaller. Fresh Codex, production policy/source/concurrency acceptance and RC02 selection remain. |
| U04 | open | [OPT02](52o-conditional-composition-and-protocol-tasks.md#OPT02) | Restart consumer established. Memory task loss and retained native uncertainty tested; public native-backed task projection works across fresh STDIO processes. Actual Codex, queued/running recovery, cancellation/drain, retention and RC03 selection remain. The separate telemetry review is delivered and 52p–52s are planned; no task or telemetry deployment is established. |

## OpenTelemetry target and foundation assessment — 2026-10-24

**Proposed:** make ordinary execution explainable across the single FastMCP endpoint, Python
helpers/embedded boundaries, Rust operations and SurrealDB, with retained operational queries,
population-correct metrics and an explanation of missing observations. Use persistent native
Collector plus OpenObserve OSS as the first local pipeline, alongside finite native detail and
profile capture. Native observations, execution, scientific outcomes and evidence custody retain
their current authority. Telemetry supplies a diagnostic projection, not a second scheduler,
scientific result ledger, semantic search service or mandatory interface for ordinary editing.

The authoring baseline is `6e93c07c1` plus preserved concurrent Plan54 build-tooling changes.
The review's application baseline is `640035fbf`; evidence drivers/results are retained in
`6d5e5d273` and the review in `6f1a791f3`. Application telemetry has not been deployed.
Build ownership changes since the review are consumed through their current structured owners;
this amendment neither freezes the old build path nor depends on completion of Plan54.
Core3.4, Heuristics1.0, the selected test-assessment companion, PS1.5 and the binding shape this
focused assessment. It does not initiate a broad audit or reopen delivered Plan49–51 scope.

| Foundation | Current evidence and required improvement |
|---|---|
| Native observation | `pse-diagnostics` already owns operation/producer IDs, coherent callback facts, phases, clocks and coverage. Export selected projections rather than recreating these meanings from span names. Preserve passive checkpoints while the backend is unavailable. |
| Instrumentation | Rust `tracing`/DataFusion spans and caller-composed capture layers exist; the application has no Rust OTel exporter. Python API/SDK1.45.1 is available, but ordinary FastMCP serving installs no SDK. TEL01–TEL04 add process-owned composition and owner-level instrumentation. |
| Propagation | FastMCP request propagation and a Python fixture child are tested only at their named boundaries. The bounded Rust SDK spawns a task without OTel context; its HTTP request owner can inject headers. The pinned server already extracts W3C headers. F02 owns that concrete seam after TEL01/TEL03. |
| Query backend | A disposable OpenObserve OSS1.1.2 process accepted synthetic OTLP JSON and returned trace/log/metric queries. This establishes retrieval, not Collector delivery, restart durability, actual application continuity or overhead. TEL05–TEL08 and TEL13 qualify those consumed contracts. |
| Signal fidelity | Resource fields normalize differently across signals; search totals were page-local; PromQL start labels were not per-point provenance. Exemplars retained IDs but appeared on inappropriate buckets. Preserve these limits and qualify stronger claims through TEL12/TEL13. |
| Existing interface | CAP/ADP/MCP/CON/NAV/LIFE delivery supplies exact discovery, typed host/native boundaries and authentic handles. TEL14 adds telemetry capabilities through those owners; G01–G03 qualify useful combined agent workflows. |
| Evidence and operations | F01 and existing native/offline custody own exact comparisons and evidence; telemetry can explain selected executions without becoming their result store. Existing host/build/test/service readers remain independent and provide projections at their own source boundaries. |

### Shared realization and contracts

Each actual process has one deliberate telemetry composition owner. Select explicit Python SDK
bootstrap as the initial realization; experimental file configuration is an alternative with a
concrete revisit condition, not an additional precedence layer. Rust uses existing `tracing`
composition plus released OTel API/SDK/OTLP0.33.1 and bridge0.34.0 candidates, with exact final
pins and feature/runtime qualification. Standard OTLP HTTP/protobuf with an explicitly compatible
blocking HTTP client is the initial Rust export direction; the server may continue its supported
OTLP/gRPC export to the same Collector. Do not require a new workspace crate merely to name this
responsibility. Dependencies alone need no ADR; genuine protected contract changes use FND00.

Keep native entity/revision/run/attempt/operation, MCP request, process incarnation, trace/span,
artifact and instrumentation scope distinct. Use W3C parentage and explicit links at real
boundaries; shared IDs and temporal overlap remain associations. Define metric populations,
units, aggregation, reset and bounded dimensions at performing owners. SDK/Collector/backend
availability is independent of native source coverage and scientific/controller outcome.
A combined FastMCP read carries its source intervals and limitations, not a fabricated atomic cut.

Use backend-native filtering, projection and aggregation before transport. One owned mapping
translates normalized fields and preserves each signal's meaning; no universal query engine or
framework-specific interpretation loop is selected. Existing host schemas remain Pydantic-owned;
Rust scientific documents retain registry/native codecs and admission. Keep exact native records
where consumed, and remove generic capture/export/query machinery only when the replacement's
actual contracts and consumers qualify. Hot callbacks perform no synchronous network export;
ordinary metrics avoid unique request/case/model/trace labels and per-scalar span creation.

### Companion and package boundaries

| Owner | Primary responsibility and exchanged capability |
|---|---|
| [52p](52p-shared-telemetry-and-native-propagation.md) | TEL01–TEL04: identity/configuration, process providers, Python/Rust handoffs and native stage/log projections. Supplies bootstrap and context mechanisms; F02 implements database transport propagation. |
| [52q](52q-local-telemetry-pipeline-and-health.md) | TEL05–TEL08: selected native infrastructure, pipeline processing/queues, independent health and retention/recovery. Supplies actual ingest/query readiness, selected backend identity and local failure diagnostics. |
| [52r](52r-operational-metrics-and-diagnostic-integration.md) | TEL09–TEL12: owned metric populations/views, native preparation/storage and engineering projections, sampling/distribution/exemplar semantics. Supplies operational measures and their interpretation to queries. |
| [52s](52s-telemetry-queries-and-agent-investigations.md) | TEL13–TEL17: typed backend retrieval, canonical exposure/guidance, profile escalation, offline publication and watch/alert outcomes. Supplies useful operations, not another agent workflow or task scheduler. |
| [52f](52f-evidence-analysis-and-trace-correlation.md) | F02/I11 retains the actual bounded SDK task/header and application→server→module seam. F01 keeps supplied-source comparisons; F03 remains separately selectable. |
| [52g](52g-agent-and-operator-workflows.md) | G01–G03 consume actual operations in composed agent investigations; G05/I13 owns any selected operator UI. |
| [52h](52h-qualification-and-publication.md) | H01–H04 retain whole-series reconciliation, actual assembled qualification and enduring publication, including review O20. |

Conditional mechanisms are not silently required deployments. TEL15 must deliver usable existing
native/profile escalation and settle whether Pyroscope/eBPF/context-observer additions add value.
TEL17 must settle a concrete watch/alert consumer and supported OSS mechanism or an explicit
headless read alternative with a revisit trigger. TEL12/TEL13 must deliver truthful example
drill-down without assuming correct bucket membership; stronger exemplar claims require TQ01.
An inquiry outcome cannot remove required pipeline, query or native-continuity behavior.

## OpenTelemetry rule decisions

These RC IDs belong only to the [OpenTelemetry review](../functional_reviews/opentelemetry-and-agent-observability-2026-10-24.md).
The operator explicitly accepted both on **2026-10-24**. `resolved` records the selection
question, not implementation, accepted ADR status or qualified deployment. FND00 publishes
each actual governed contract slice before the dependent change; ordinary tooling/dependency
changes require no placeholder decision record.

| Decision | Disposition | Work owner | Accepted consequence and route |
|---|---|---|---|
| RC01 | resolved | [52q/TEL05](52q-local-telemetry-pipeline-and-health.md#TEL05), [F02](52f-evidence-analysis-and-trace-correlation.md#F02) | Continuous local Collector/query service plus finite capture. Expand F02's prior finite-only target and publish the resulting blueprint §23.1/environment/substrate/capability meanings through their actual design and guide routes before relying on changed contracts. No transfer of native execution/scientific authority. |
| RC02 | resolved | [52p/TEL01](52p-shared-telemetry-and-native-propagation.md#TEL01), [52s/TEL13](52s-telemetry-queries-and-agent-investigations.md#TEL13) | Shared identities/export/retrieval mappings and typed reads through the existing FastMCP endpoint. Preserve the Rust scientific/Python host boundary. FND00 classifies any actual new relation family, backend binding or protected-boundary change and supplies its required declaration/ADR/design route; new dependencies alone do not trigger it. |

## OpenTelemetry opportunity dispositions

All O IDs here are source-qualified to the OpenTelemetry review. Every opportunity has one
primary owner; supporting packages and consumer qualification do not create additional status
ledgers. `scheduled` means selected planning scope, including the stated bounded investigation
where the mechanism is conditional. Historical isolated probes remain evidence, not closure.

| Opportunity | Disposition | Work owner | Required behavior or disposition |
|---|---|---|---|
| O01 | scheduled | [TEL01](52p-shared-telemetry-and-native-propagation.md#TEL01) | One inspectable process bootstrap, effective configuration and supported provider lifecycle; explicit Python setup is baseline. |
| O02 | scheduled | [F02](52f-evidence-analysis-and-trace-correlation.md#F02) | Actual cross-boundary continuity using TEL02/TEL03 handoffs and the concrete SDK/server seam; I11 retains unsupported-edge truth. |
| O03 | scheduled | [TEL04](52p-shared-telemetry-and-native-propagation.md#TEL04) | Meaningful native stage explanations with authentic source relationships and coverage, consumed by TEL13/G01. |
| O04 | scheduled | [TEL04](52p-shared-telemetry-and-native-propagation.md#TEL04) | Queued/executing/shared-wait/response/drain distinctions survive timeout, observer loss and native settlement. |
| O05 | scheduled | [TEL09](52r-operational-metrics-and-diagnostic-integration.md#TEL09) | Population-correct metrics, units/views/dimensions/reset meaning without a second native counter authority. |
| O06 | scheduled | [TEL10](52r-operational-metrics-and-diagnostic-integration.md#TEL10) | Preparation, reuse, invalidation and shared producers explain repeated work without redefining cache dependencies. |
| O07 | scheduled | [TEL10](52r-operational-metrics-and-diagnostic-integration.md#TEL10) | Operation-associated database/storage signals and actual service identity; F02 supplies parentage, not assumed enterprise features. |
| O08 | scheduled | [TEL04](52p-shared-telemetry-and-native-propagation.md#TEL04) | Structured diagnostic projection and deliberate log/span-event representation with native outcome authority. |
| O09 | scheduled | [TEL11](52r-operational-metrics-and-diagnostic-integration.md#TEL11) | Current build/test/qualification/host owner projections, original invocation identity and exact evidence readers. |
| O10 | scheduled | [TEL07](52q-local-telemetry-pipeline-and-health.md#TEL07) | Independent pipeline-stage observations and local fallback, distinguishing attempted/received/query-visible from durable/complete. |
| O11 | scheduled | [TEL05](52q-local-telemetry-pipeline-and-health.md#TEL05) | Native supervised Collector/OpenObserve and retained local querying, with TEL06/TEL08 pipeline/lifecycle qualification. |
| O12 | scheduled | [TEL13](52s-telemetry-queries-and-agent-investigations.md#TEL13) | Backend-aware typed selected reads with real normalization, population, interval, pagination and partiality semantics. |
| O13 | scheduled | [TEL14](52s-telemetry-queries-and-agent-investigations.md#TEL14) | Existing exact discovery/reveal/resources/invoke-read route supplies telemetry investigation; G01–G03 exercise real agent use. |
| O14 | scheduled | [TEL12](52r-operational-metrics-and-diagnostic-integration.md#TEL12) | Distribution-to-related-example navigation; TQ01 qualifies stronger bucket/population attribution instead of assuming it. |
| O15 | scheduled | [TEL15](52s-telemetry-queries-and-agent-investigations.md#TEL15) | Existing finite native/profile escalation plus TQ03's justified adoption or supported alternative for advanced mechanisms. |
| O16 | scheduled | [TEL16](52s-telemetry-queries-and-agent-investigations.md#TEL16) | Bounded offline investigation with original query/source intervals, native references, loss and existing custody/publication. |
| O17 | scheduled | [TEL17](52s-telemetry-queries-and-agent-investigations.md#TEL17) | TQ04 resolves concrete watches/OSS alerts versus headless reads; G05 owns any separately selected interactive view. |
| O18 | scheduled | [TEL03](52p-shared-telemetry-and-native-propagation.md#TEL03) | Qualified export/runtime/shutdown behavior, no synchronous native hot-path network work, and preserved native outcomes. |
| O19 | scheduled | [TEL12](52r-operational-metrics-and-diagnostic-integration.md#TEL12) | Owner metrics independent of trace sampling; selected detail and evidence-based sampling mechanisms fit scientific lifetimes. |
| O20 | scheduled | [H02](52h-qualification-and-publication.md#H02) | Independent local contracts plus actual assembled SDK/Collector/backend/FastMCP/native qualification, with H01/H03/H04 reconciliation and publication. |

## OpenTelemetry inquiries

TQ IDs identify new decision questions. Existing I11 remains the sole database-continuity
inquiry and I13 remains the operator-view decision; this table does not duplicate either.

| Inquiry | State | Owner | Decision / completion evidence |
|---|---|---|---|
| TQ01 | open | [TEL12](52r-operational-metrics-and-diagnostic-integration.md#TEL12), [TEL13](52s-telemetry-queries-and-agent-investigations.md#TEL13) | Qualify actual normalized query/reset/histogram/exemplar behavior and required claims. The direct fixture establishes pointer retention only; retain limited semantics or change the specific mechanism/backend if a required stronger consumer cannot be served. |
| TQ02 | open | [TEL08](52q-local-telemetry-pipeline-and-health.md#TEL08) | Actual Collector/backend restart, queue expiry/overflow, storage failure and retention outcomes; query visibility is not crash durability. |
| TQ03 | open | [TEL15](52s-telemetry-queries-and-agent-investigations.md#TEL15) | Compare existing CPU/Perfetto/native capture with optional supported Pyroscope/eBPF/experimental context observation for one unresolved question; record adopted mechanism or sufficient existing route and revisit trigger. |
| TQ04 | open | [TEL17](52s-telemetry-queries-and-agent-investigations.md#TEL17) | Demonstrate a useful exact condition/watch and OpenObserve OSS fit, or retain headless health/query reads with a consumer-based revisit; no automatic intervention or MCP scheduler follows. |
| TQ05 | open | [TEL08](52q-local-telemetry-pipeline-and-health.md#TEL08), [TEL16](52s-telemetry-queries-and-agent-investigations.md#TEL16) | Determine whether standard finite collection/file export preserves current bounds, identities, restoration, losses and sealed evidence consumers. Delete the custom sink only after equivalent supported consumers qualify; file output alone is insufficient. |

## Package state and execution route

This is the single package progress table. The companion links explain prerequisites and
acceptance; agreement on a design alone does not establish a working prerequisite.

| Package | Status | Owner | Completion |
|---|---|---|---|
| FND00 | in-progress | [Coordinator](#FND00) | Decision/declaration slices precede dependent changes |
| A01 | done | [52a](52a-context-navigation-and-semantic-editing.md#A01) | Current task packet and historical rationale |
| A02 | done | [52a](52a-context-navigation-and-semantic-editing.md#A02) | Actual generator/consumer paths |
| A03 | done | [52a](52a-context-navigation-and-semantic-editing.md#A03) | Real semantic-edit consumer and I01 |
| A04 | done | [52a](52a-context-navigation-and-semantic-editing.md#A04) | Structural transforms and I02 |
| A05 | done | [52a](52a-context-navigation-and-semantic-editing.md#A05) | Supported alternative-impact comparison |
| B01 | done | [52b](52b-local-knowledge-and-research.md#B01) | Exact source-fresh concepts, lexical discovery and first I03 decision |
| B02 | done | [52b](52b-local-knowledge-and-research.md#B02) | Activity discovery/logical history and I04 |
| B03 | done | [52b](52b-local-knowledge-and-research.md#B03) | Exact-version dossier with unresolved semantics |
| B04 | done | [52b](52b-local-knowledge-and-research.md#B04) | Separate derived knowledge family and rebuild |
| B05 | done | [52b](52b-local-knowledge-and-research.md#B05) | Bounded local semantic decision and second I03 outcome |
| C01 | done | [52c](52c-behavioral-and-scientific-verification.md#C01) | Coverage correspondence without behavioral overclaim |
| C02 | done | [52c](52c-behavioral-and-scientific-verification.md#C02) | Scientific/stateful challenge and I05 |
| C03 | done | [52c](52c-behavioral-and-scientific-verification.md#C03) | Mutation/fault-injection consumer |
| C04 | in-progress | [52c](52c-behavioral-and-scientific-verification.md#execution-boundary--instrumentation) | Ordinary structured controls passed and harnesses compile; actual coverage-guided acceptance blocked by automatic task review |
| C05 | done | [52c](52c-behavioral-and-scientific-verification.md#C05) | Numerical reference experiment and export |
| C06 | done | [52c](52c-behavioral-and-scientific-verification.md#C06) | Meaning-preserving source-aware reduction |
| C07 | done | [52c](52c-behavioral-and-scientific-verification.md#C07) | Focused instrumentation; native I06 outcome |
| C08 | done | [52c](52c-behavioral-and-scientific-verification.md#C08) | Controlled schedules; modeled I06 outcome |
| D01 | done | [52d](52d-execution-and-qualification-efficiency.md#D01) | Cursor watch/reconciliation and real terminal |
| D02 | done | [52d](52d-execution-and-qualification-efficiency.md#D02) | Feature interruption/resume/repair |
| D03 | done | [52d](52d-execution-and-qualification-efficiency.md#D03) | Safe build equivalence decision I07 |
| D04 | done | [52d](52d-execution-and-qualification-efficiency.md#D04) | Pressure/topology and truthful unknowns |
| E01 | done | [52e](52e-reproducible-experiments-and-isolation.md#E01) | Actual remapped replay, distinct producer/result, dependency refusal and protected detail reclamation; native archival unselected |
| E02 | done | [52e](52e-reproducible-experiments-and-isolation.md#E02) | Actual v2 populations, exact rows/edges/permissions, query equivalence, SIGINT/read-only settlement and sibling-preserving reclamation passed |
| E03 | done | [52e](52e-reproducible-experiments-and-isolation.md#E03) | Actual semantic bisection visited an untestable revision, found the known bad commit and removed only its temporary Git checkout/build output after drain; main and independent worktrees preserved; retained noisy comparison inconclusive |
| E04 | done | [52e](52e-reproducible-experiments-and-isolation.md#E04) | Optional Bubblewrap witnesses and actual CLI/library parser consumers; Surrealism grammar uses existing pinned parser |
| F01 | planned | [52f](52f-evidence-analysis-and-trace-correlation.md#F01) | Exact paired comparisons, selective answer sections/completeness and protected same-cut native/canonical readers |
| F02 | planned | [52f](52f-evidence-analysis-and-trace-correlation.md#F02) | Actual SDK task/header and application/server/module continuity, finite capture plus selected continuous pipeline, and I11 |
| F03 | deferred | [52f](52f-evidence-analysis-and-trace-correlation.md#F03) | PI09 optional finite multi-question inquiry; revisit for a real repeated-question consumer over one cut and explicit selection. Unselected delivery is outside original series completion. |
| G01 | planned | [52g](52g-agent-and-operator-workflows.md#G01) | Shared interaction vocabulary and first real consumers |
| G02 | planned | [52g](52g-agent-and-operator-workflows.md#G02) | Actual unified agent workflows consuming CAP/ADP/MCP delivery; I12 selection settled |
| G03 | planned | [52g](52g-agent-and-operator-workflows.md#G03) | Task-focused instruction/evaluation consumer |
| G04 | planned | [52g](52g-agent-and-operator-workflows.md#G04) | Safe integration/closeout proposal |
| G05 | planned | [52g](52g-agent-and-operator-workflows.md#G05) | Operator investigation and I13 |
| H01 | planned | [52h](52h-qualification-and-publication.md#H01) | Scope and actual consumers reconciled |
| H02 | planned | [52h](52h-qualification-and-publication.md#H02) | Assembled functional journeys |
| H03 | planned | [52h](52h-qualification-and-publication.md#H03) | Scope-end static/manual checks |
| H04 | planned | [52h](52h-qualification-and-publication.md#H04) | Enduring contracts, actual Outcome and retirement readiness |
| CAP01 | done | [52i](52i-canonical-capabilities-and-progressive-reveal.md#CAP01) | Canonical exposure and five actual source-only bindings; 52i Outcome owns evidence |
| CAP02 | done | [52i](52i-canonical-capabilities-and-progressive-reveal.md#CAP02) | Exact catalogue, actual schemas and bounded preparation |
| CAP03 | done | [52i](52i-canonical-capabilities-and-progressive-reveal.md#CAP03) | Atomic replacement/reset and actual Codex exact-read route |
| CAP04 | done | [52i](52i-canonical-capabilities-and-progressive-reveal.md#CAP04) | Lazy protected owner guidance and planning-only selection |
| CAP05 | done | [52i](52i-canonical-capabilities-and-progressive-reveal.md#CAP05) | Bounded whole-entrypoint classification; ADP05 supplies current native adaptation/retention dispositions |
| ADP01 | done | [52j](52j-native-contracts-and-tool-adaptation.md#ADP01) | Generated native contracts, original receivers and Q02 accepted with actual native journeys |
| ADP02 | done | [52j](52j-native-contracts-and-tool-adaptation.md#ADP02) | Typed host requests, bounded independent outcomes, authentic handles and cancellation/drain |
| ADP03 | done | [52j](52j-native-contracts-and-tool-adaptation.md#ADP03) | Shared typed archive/diagnostic kernels and owner-backed qualification/scoped tests; Q04 resolved |
| ADP04 | done | [52j](52j-native-contracts-and-tool-adaptation.md#ADP04) | Actual native, Python/Rust language and selected database journeys; Q03 resolved |
| ADP05 | done | [52j](52j-native-contracts-and-tool-adaptation.md#ADP05) | 65 callable operations; 402 classified entry points with explicit native retention and replaced session dispatch retired |
| MCP01 | done | [52k](52k-unified-entry-point-and-derived-interfaces.md#MCP01) | Cheap bootstrap and owned lazy specialist connections qualified by 52j |
| MCP02 | done | [52k](52k-unified-entry-point-and-derived-interfaces.md#MCP02) | Fresh Codex resource/template/prompt/action/result/reset journey passed; Q01 resolved; see 52k Outcome |
| MCP03 | done | [52k](52k-unified-entry-point-and-derived-interfaces.md#MCP03) | Loaded source/worker/schema fences and scoped guidance restart controls delivered; Q06 resolved |
| MCP04 | done | [52k](52k-unified-entry-point-and-derived-interfaces.md#MCP04) | Complete composed reference generated and checked; Q05 resolved with selected CLI deferral |
| MCP05 | done | [52k](52k-unified-entry-point-and-derived-interfaces.md#MCP05) | Bounded interface diagnostics/correlation retain original native outcomes and authentic result owners; see 52k Outcome |
| CON01 | done | [52l](52l-typed-requests-results-and-diagnostics.md#CON01) | Eleven tagged owner requests, independent examples and migrated callers; bounded fresh-Codex acceptance passed |
| CON02 | done | [52l](52l-typed-requests-results-and-diagnostics.md#CON02) | Fourteen typed selected answer families with actual schemas and explicit detail/refusal states |
| CON03 | done | [52l](52l-typed-requests-results-and-diagnostics.md#CON03) | Safe owner/native decode diagnostics and preserved version/header/trailing/contextual admission |
| CON04 | done | [52l](52l-typed-requests-results-and-diagnostics.md#CON04) | U01 resolved; native protected schema/readable values and exact IPC, real wide-result and Python consumers; see 52l Outcome |
| NAV01 | done | [52m](52m-progressive-guidance-and-operation-availability.md#NAV01) | Compact progressive selection/guidance and declared workflow steps |
| NAV02 | done | [52m](52m-progressive-guidance-and-operation-availability.md#NAV02) | Actual module/package/operation ownership and local extension |
| NAV03 | done | [52m](52m-progressive-guidance-and-operation-availability.md#NAV03) | Operation-shaped dependency/restart fences and unaffected reads |
| NAV04 | done | [52m](52m-progressive-guidance-and-operation-availability.md#NAV04) | Supporting instruction resources and U02 exact assistance disposition |
| LIFE01 | done | [52n](52n-readiness-progress-and-edit-lifecycles.md#LIFE01) | Selected readiness and explicit owner preparation consumers |
| LIFE02 | done | [52n](52n-readiness-progress-and-edit-lifecycles.md#LIFE02) | Authentic progress/early handles with attached cancellation/drain and reader fallback |
| LIFE03 | done | [52n](52n-readiness-progress-and-edit-lifecycles.md#LIFE03) | Preview status/release with admitted-work retention and sibling preservation |
| OPT01 | in-progress | [52o](52o-conditional-composition-and-protocol-tasks.md#OPT01) | Paused after bounded real-Monty/typed/small-read comparison and dispatch/lifecycle controls; fresh host/final policy remain |
| OPT02 | in-progress | [52o](52o-conditional-composition-and-protocol-tasks.md#OPT02) | Paused after native-owner/task/STDIO-loss/projection inquiry; integrated native recovery, actual host and final selection remain |
| TEL01 | planned | [52p](52p-shared-telemetry-and-native-propagation.md#TEL01) | Shared identities, effective configuration and process composition contract |
| TEL02 | planned | [52p](52p-shared-telemetry-and-native-propagation.md#TEL02) | Python/FastMCP providers, helper carriers and logging lifecycle |
| TEL03 | planned | [52p](52p-shared-telemetry-and-native-propagation.md#TEL03) | Rust/embedded/native-child providers, context and qualified export/shutdown |
| TEL04 | planned | [52p](52p-shared-telemetry-and-native-propagation.md#TEL04) | Native stage/wait/drain/diagnostic projections and migrated owner consumers |
| TEL05 | planned | [52q](52q-local-telemetry-pipeline-and-health.md#TEL05) | Selected native Collector/OpenObserve installation and supervised owned state |
| TEL06 | planned | [52q](52q-local-telemetry-pipeline-and-health.md#TEL06) | Shared processing, routing, bounded export queues and effective configuration |
| TEL07 | planned | [52q](52q-local-telemetry-pipeline-and-health.md#TEL07) | Independent pipeline health and backend-unavailable local diagnostics |
| TEL08 | planned | [52q](52q-local-telemetry-pipeline-and-health.md#TEL08) | Retention/restart/recovery and TQ02/TQ05 supported capture outcomes |
| TEL09 | planned | [52r](52r-operational-metrics-and-diagnostic-integration.md#TEL09) | Owned metric populations, units, views, dimensions and reset semantics |
| TEL10 | planned | [52r](52r-operational-metrics-and-diagnostic-integration.md#TEL10) | Native preparation/reuse/database/storage metric projections |
| TEL11 | planned | [52r](52r-operational-metrics-and-diagnostic-integration.md#TEL11) | Current engineering build/test/qualification and host projections |
| TEL12 | planned | [52r](52r-operational-metrics-and-diagnostic-integration.md#TEL12) | Sampling/distribution/example semantics with TQ01 stronger-claim qualification |
| TEL13 | planned | [52s](52s-telemetry-queries-and-agent-investigations.md#TEL13) | Backend-aware typed queries, schema mapping, pagination and TQ01 |
| TEL14 | planned | [52s](52s-telemetry-queries-and-agent-investigations.md#TEL14) | Exact canonical exposure, resources/guidance and useful selected reads |
| TEL15 | planned | [52s](52s-telemetry-queries-and-agent-investigations.md#TEL15) | Native profiling escalation and TQ03 mechanism disposition |
| TEL16 | planned | [52s](52s-telemetry-queries-and-agent-investigations.md#TEL16) | Bounded offline evidence publication and TQ05 consumer preservation |
| TEL17 | planned | [52s](52s-telemetry-queries-and-agent-investigations.md#TEL17) | Concrete watch/alert investigation TQ04 and selected supported read/action consumers |

<a id="FND00"></a>
### FND00 — Decision and declaration readiness

Before each dependent change, identify its exact current owner and implement the accepted rule
slice through the route above. Introduce no extra architecture merely because a conceptual
package has a name. New relation/document families use registry/catalogue generators and required
short ADRs; changes to governance, D1–D14 or protected contracts take their required ADR/review
route. Existing accepted ADRs are immutable. Ordinary tool refinements and dependency additions
within contracts need neither. Do not create placeholder ADRs for every RC row.

Publish changed architecture only through the explicit design-edit route and revision row.
Record which prerequisite is merely designed versus declared, integrated and exercised. This
package is incremental: B04's schema does not block A01's source-only task packet or D01's watch.
The original, unified-interface, usability and OpenTelemetry rule tables are distinct sources. Usability RC01
is accepted for planning while RC02/RC03 authorize investigation only. If a new rule
impact appears beyond them, present that exact consequence before
fixing the dependent target. Acceptance is the required decision/declaration in place and its
actual consumer consuming it, not completion of a generic paperwork milestone.
OpenTelemetry RC01/RC02 are explicitly accepted for planning; provider/backend declaration and
any required blueprint §23.1/native contract changes precede their dependent implementation.

### Dependency order

For telemetry, TEL01's semantic/configuration slice precedes TEL02/TEL03/TEL09/TEL13 integration.
TEL05 can prepare native infrastructure independently, then TEL06/TEL07 establish the actual
pipeline/health route. F02 follows the working TEL01/TEL03 handoff, not all metric/query packages.
TEL04 and TEL10/TEL11 extend performing owners; TEL12 and TEL13 qualify the signals and retrieval
they consume rather than waiting for every producer. TEL14 binds usable reads, after which
G01–G03 exercise real investigations. TEL15/TEL16/TEL17 consume only their needed capture/custody/
query slices; TEL08's recovery and finite-sink decisions qualify the selected infrastructure.
H01–H04 reconcile and qualify the assembled required target and explicit conditional outcomes.
These are capability dependencies, not whole-companion barriers or a reason to repeat campaigns.

The linked Plan53 amendments consume completed 52k without changing MCP progress here. Shared
reader and workflow migrations follow the actual 53e E05/52f F01 contract slices, not entire
companion barriers. Optional F03 consumes F01's supplied-cut/comparison semantics only after
separate selection. I14 is a separate post-delivery disposition, not an MCP/Plan53 dependency.

Start independent source-grounded work: A01/A02, B02/B03, C01–C05, D01–D04, E02 and F01.
G01's small early interaction slice accompanies the first context/watch/result consumers; its
full implementation does not precede all other work. A03/A04 qualify tools before editing use.
B01 precedes B05. B04 consumes B01/B02/B03/A02's source facts and existing evidence generations;
it does not block in-memory paths. A05 consumes A01/A02 and selected B03 research, then gains
optional B04 retrieval. E03 can use existing reproducers before C06 reduction or E01 archives.
F02 consumes exact capture/transport ownership, not completion of Plan40e. G04 follows actual
source/consumer changes; G02 consumes real canonical operations and G05 useful comparison results.
H01–H04 close assembled scope
once selected functionality and inquiry outcomes are ready.

For the unified extension, CAP01–CAP05, ADP01–ADP05 and MCP01–MCP05 deliver exact discovery,
guidance, real host/native operations, owned lazy specialists, coherent loaded contracts, complete
derived reference and interface observation. Q01–Q06 are resolved at their recorded component
acceptance boundaries; conditional CLI delivery is deliberately not adopted.
New entry points still extend CAP05's canonical classification through their binding owner.
G02/G03 evaluate assembled real workflows; F01/F02 retain broader evidence/trace inquiries.
MCP04 supplies generated references and the conditional CLI revisit decision. H01 reconciles all required and conditional scope before
H02–H04. These are contract/evidence dependencies, not whole-document serial barriers. Preserve
C04's separately recorded denied execution boundary; interface work does not authorize retry.

For usability evolution, begin independent CON01/CON03, NAV01/NAV02 and LIFE03 slices from delivered
foundations. CON02 consumes actual selected-reader contracts (including 53e E05/F01) for each migrated
family, not an entire Plan53 milestone. CON04 settles U01 before changing native result boundaries.
NAV03 uses actual operation/worker ownership and lightweight contract separation; NAV04 consumes
NAV01's coherent resource index. LIFE01 consumes actual Plan53d stages and CON03 remedies; LIFE02
bridges issued native references with CON02/CON03 presentation without changing native ownership.
G01/G02 compose real operations and G03 evaluates the resulting instructions/requests/results.
OPT01 follows useful CON01/CON02/F01 selected reads; OPT02 follows LIFE02's foreground/native comparison.
G05/I13 separately assesses operator views. Conditional non-adoption does not block ordinary delivery;
H01 still reconciles every inquiry and any explicitly selected additional implementation before H02–H04.
No new entire-companion barrier, repeated broad campaign or second completion ledger is introduced.

Shared source files are a separate coordination concern from logical dependencies. Give one
writer the registry/manifest changes, another the current tooling owner only when editing can
be isolated, and integrate targeted checks before making dependent consumers ready. Use regular
discrete commits; a whole companion need not finish before a coherent package can land.

## Verification and current checkpoint

**Proposed verification:** packages compile affected code and use targeted tests/real bounded
consumers, deleting replaced mechanisms immediately after migration. Regeneration accompanies
changed declarations. 52h owns scope-end integration/component/native/Python journeys, hygiene
and relevant manual checks. No numerical performance/productivity benefit is promised; a measured
claim needs its actual baseline and conditions. Functional populations are representative, not
production-scale parameter/lane counts. Documentation authoring needs no product compilation.

Current checkpoint: authorized 52a/52b scope and its required FND00/G01 slices are delivered at
their selected focused acceptance boundaries. [52a's outcome](52a-context-navigation-and-semantic-editing.md#targeted-execution-outcome)
owns task/design composition, actual producer paths and Rust/Python edit consumers;
[52b's outcome](52b-local-knowledge-and-research.md#structured-execution-outcome) owns source/history/library
access and native structured knowledge. ADR-0222 preceded the new family; blueprint §24.1 now
owns enduring meaning, while formal ADR status stays proposed. B05 selected structured access
with lexical discovery and retired the unused model helper after its bounded comparison.
FastMCP4.1.3/MCP2.3.0 are installed and exercised. The subsequent 52c/52d checkpoint below owns their delivery and execution limit. Other FND00/G01
obligations and Plan52f–52h remain planned, including assembled qualification and scope-end hygiene/manual checks.
The subsequent 52e checkpoint below owns that authorized execution.
No whole-series or productivity/performance claim follows from these targeted checks.

### Owner-selected structured-query pivot

On 2026-10-23 the maintainer selected first-principles domain modeling and exact structured
queries for generated/controlled content, with native topics as secondary aids. This reopened the
affected A01/A02/A05/B01/B04 acceptance while retaining earlier checks' original scope; the
subsequent structured host/native/CLI acceptance above closes that selected pivot.
[52b's target](52b-local-knowledge-and-research.md#B04) owns concrete concepts, exact production
correspondence, native authority/state distinctions and query behavior. The requested SurrealDB,
tooling and SurrealKV source review informed the physical realization. Semantic similarity remains
unselected; no new database or authority is implicit. A03/A04/B02/B03's unaffected delivery remains.


### 52c/52d execution checkpoint — 2026-10-24

All twelve packages were authorized and their functional implementations are integrated.
[52c's Outcome](52c-behavioral-and-scientific-verification.md#targeted-execution-outcome--2026-10-24)
owns the scientific properties, actual coverage/mutation controls, numerical workbench, separate
source reduction and actual GDB/schedule consumers. C01/C02/C03/C05/C06/C07/C08 meet their selected
focused boundaries. C04's structured controls passed and optional harnesses compile, but its
actual coverage-guided campaign remains unexecuted after automatic task review denied it;
C04 and 52c remain in progress for that obligation. Normal ASan and fault-sensitivity execution
are also unqualified; the native inquiry selected the narrower tested GDB route. No automatic
retry or toolchain-incompatibility claim is implied.

[52d's Outcome](52d-execution-and-qualification-efficiency.md#targeted-execution-outcome--2026-10-24)
closes D01–D04: detached owner watches, pressure/placement facts, actual combination interruption/
resume/source-repair invalidation, and a genuine fresh complete-context main server build followed
by actual safe configuration-equivalence reuse. Unknown/moved/shared targets and unsupported
contexts refuse; no active service was restarted. Effective PATH remains conservatively bound.

ADR-0223 preceded the separate reduction family, generated contracts and Rust/public Python CLI;
original reproduction remains exact. RC03's narrower reuse unit is published at blueprint §24.1.
Test Assessment1.2 guided execution without reopening settled reviews. I05 selects existing
Proptest sequences. I06 records actual native/schedule outcomes independently. I07 selects parsed
TOML equivalence only inside the closed context. Their details and revisit triggers live at their
companion owners. Other FND00/G01 obligations and 52f–52h stay planned; full powerset, hygiene/manual
checks, assembled qualification and any performance/productivity benefit remain unestablished.

### 52e execution checkpoint — 2026-10-24

E01–E04 are authorized; [52e's Outcome](52e-reproducible-experiments-and-isolation.md#targeted-execution-outcome--2026-10-24)
owns selected historical replay and retention, actual semantic Git bisection, retained inconclusive
comparison and optional Bubblewrap/server/library parser consumers. Native archival and Podman
remain unselected under the recorded revisit conditions. FND00 required no new runtime document
family for these existing-owner tooling records.

E02's actual disposable-service acceptance passed after repairing immutable receipt context,
interrupted publication/drain recovery, full-schema transport, explicit database setup and unique
fixture identities. The Outcome retains original failures and corrected acceptance separately.
E01–E04 and I08–I10 are complete at their selected focused boundaries. Other FND00/G01
obligations, C04's separately recorded denial boundary and Plan52f–52h retain their scope.


### Unified-interface authoring checkpoint — 2026-10-24

52i–52k and the affected 52a–52h integration boundaries now express the selected single-surface
architecture. Unified O01–O10/Q01–Q06/RC01–RC05 have explicit owners above. Original delivered
Outcomes and package states remain unchanged; I12 is resolved only as a target selection.
The new CAP/ADP/MCP packages are planned. No canonical production endpoint, actual Codex refresh,
full native schema bridge, whole-inventory migration or faithful generated CLI is delivered by
this authoring. Rule directions are accepted; actual instruction/boundary changes follow FND00.
The next implementation authorization can start the first slice described above. H01–H04 remain
the assembled qualification/publication owner for expanded 52a–52k, while C04 and independent
Plan28/32/40/53 obligations retain their current owners and evidence limits.

### 52i execution checkpoint — 2026-10-24

CAP01–CAP05 are delivered at the maintainer-selected boundary: complete bounded classification,
canonical exposure, exact navigation, atomic reveal/reset and progressive owner instructions,
with existing clean source-only operations callable. The
[52i Outcome](52i-canonical-capabilities-and-progressive-reveal.md#verification-and-outcome--2026-10-24)
owns implementation, actual Codex evidence, repaired mistakes and targeted qualification.
RC02's Pydantic host-contract rule/direct dependency and RC05's fixed source-only launch guidance
are implemented for this slice; neither changes scientific contracts or native effect admission.

At this 52i boundary, ADP02/MCP01/MCP02 were partial prerequisites; native schema fidelity and
parser/specialist integration remained. The 52j checkpoint below records their subsequent
delivery. MCP03/MCP04 still own broader coherence/derived consumers; G02/G03 and H01–H04 retain
actual assembled agent evaluation and publication. Unified opportunities remain scheduled where
those consumers are outstanding.
Original a–h evidence, the C04 denial and independent Plan28/32/40/53 work are unchanged.


### 52j execution checkpoint — 2026-10-24

ADP01–ADP05 and MCP01 are delivered through isolated branch `plan52j-adapters`, integrating main
through `0930f8980` without conflicts after the maintainer's Plan53c completion signal.
The [52j Verification and Outcome](52j-native-contracts-and-tool-adaptation.md#verification-and-outcome--2026-10-24)
owns the current focused acceptance, repaired findings, native retention decisions and limits.
The canonical endpoint serves 65 real operations; 402 current entry points have specific
classifications under the declared coverage. Native schemas retain original codecs/receivers,
byte digests and restart fences. Actual host execution/result, source/project/result,
Python/Rust navigation/edit and selected database consumers passed. The owned disposable
service was stopped, its contexts/kernel scopes drained and its successful fixture retired.

Q02–Q04 are resolved. MCP02's broader actual Codex presentation, MCP03's remaining handler
coherence, MCP04/MCP05 and G/H assembled work remain with their owners. This is focused 52j
acceptance, not assembled Plan52 or scientific/performance qualification. C04 and independent
Plan28/32/40/53 scope remain unchanged. Final integration passed 115 capability tests,
147 affected owner tests, four native unit cases and the installed-server reuse check.
The maintainer authorized main publication and retirement of the fully merged worktree;
retained acceptance artifacts live in the main checkout at `build/capabilities/plan52j-acceptance/`.

### 52k execution checkpoint — 2026-10-24

MCP01–MCP05 are delivered on main following the maintainer's direct-checkout selection.
The [52k Verification and Outcome](52k-unified-entry-point-and-derived-interfaces.md#verification-and-outcome--2026-10-24)
owns actual fresh Codex acceptance, coherence controls, derived reference, diagnostics,
corrected failures and focused static results. Q01/Q06 are resolved; Q05 records the
maintainer-selected CLI deferral and concrete-consumer revisit trigger. Unified O01/O02/O08/O09
are resolved; O03/O07/O10 retain their separately owned G/F consumers.

The fresh host used seventeen canonical calls and one exact `pse-ids` unit. Original invocation
`1b7e8e57c5d5410c8a79e7341f6772b8` froze one selected pass, zero unreported subjects,
assurance pass and complete owned drain. This is component acceptance, not assembled H02,
scientific qualification or measured productivity. F/G/H, C04's denied execution boundary and
independent Plans28/32/40/53 remain unchanged. Enduring operation guidance lives in the
[capability interface guide](../dev/capability-interface.md); the completed companion remains
retained while the active coordinator and H01–H04 consume its evidence.


### 52m/52n execution checkpoint — 2026-10-24

NAV01–NAV04 and LIFE01–LIFE03 are complete on main. The
[52m Outcome](52m-progressive-guidance-and-operation-availability.md#verification-and-outcome--2026-10-24)
owns compact workflow-step reveal, bounded supporting resources, accurate source/worker ownership,
restart acceptance and U02's supported non-adoption/revisit decisions. The
[52n Outcome](52n-readiness-progress-and-edit-lifecycles.md#verification-and-outcome--2026-10-24)
owns passive/live readiness, typed owner preparation stages, actual early-handle/wire-progress
acceptance and synchronized preview release/reconciliation. Their seven usability opportunities
are resolved at those package boundaries; F/G/H and optional 52o remain separately scheduled.

**Tested:** final capability discovery ran 260 controls with zero failures; final extension/observation
controls ran 36 with zero failures. The fresh actual Codex driver made 22 canonical calls, checked
captured supporting content and independently read one exact passing Rust unit's native terminal.
Actual STDIO preparation/import/diagnostic and early native reference checks are in the companion
Outcomes. Native drain remained unobserved in the early probe and complete in the separate actual
Codex invocation. The interface preserves each fact. Codex JSONL did not expose progress notifications;
notification rendering is unestablished, independently of real STDIO wire delivery and native readers.
No productivity measurement, scientific eligibility or full H02 qualification is inferred.

**Tested:** scope-end `scripts/pse-env -- just hygiene` continued through all selected checks.
Its original result was three failures against zero: lint-py, typecheck and generated capability
reference drift. All were repaired; final `just lint-py`, `just typecheck` and
`just codegen-capabilities-check` passed, with zero Ruff findings and zero Pyrefly diagnostics
(the three existing configured suppressions are unchanged). Both Clippy default/no-default runs,
Rust documentation and the remaining hygiene checks passed in the original run. `just governance`
passed its tests, other code generation, Python contracts/stubs and family
checks; its sole capability-reference drift was repaired by the same final comparison. Aggregate raw
results remain in `build/assessment/20261024T174056.646181Z-hygiene-1582397-438970/` and
`build/assessment/20261024T174056.654740Z-governance-1582559-755dbf/`; repaired individual checks
retain their own command outputs. `just turn-end` passed. `just docs` published 640 chapters after
adding an already-linked census probe to explicit site assets; no concurrent review content was
changed. No native solver-fleet or feature-powerset campaign was selected for this host-tooling change.

**Implemented:** enduring instructions are in the existing capability/environment guides and derived
reference. Worker changes were integrated in coherent commits; unrelated concurrent audit edits
were preserved. Completed companions remain while this coordinator and G/H consumers need them.
I14, application/database tracing, remaining Plan53 and independent Plans28/32/40 keep their owners.

### OpenTelemetry authoring checkpoint — 2026-10-24

The separate functional-extension review is delivered, and RC01/RC02 are explicitly accepted for
planning. 52p–52s now develop its O01–O20 through TEL01–TEL17, five new bounded inquiries and the
existing F02/I11/G/H owners. F02 now targets actual SDK task/header continuity as well as continuous
and finite retrieval. Native authority and completed package Outcomes remain unchanged.
The dependency route starts with shared identity/configuration and native service contracts; partial
producer/query slices can integrate without waiting for a whole companion. H02 remains the assembled
consumer owner. This checkpoint records planned work, not telemetry implementation or qualification.

The 52o pause record below retains its historical then-pending review context. OPT01/OPT02 remain
open at their own owners; the telemetry amendment neither adopts a task scheduler/CodeMode nor
restarts their actual-host acceptance. Plan54's concurrent build changes remain independently owned.

### 52o pause checkpoint — 2026-10-24

The maintainer requested closeout of current items and a stop before any OpenTelemetry deployment.
The [52o pause handoff](52o-conditional-composition-and-protocol-tasks.md#execution-checkpoint--pause-handoff-2026-10-24)
owns the delivered inquiries, proposed integrated directions, corrected assumptions and remaining
OPT01/OPT02 acceptance. Neither package is complete and neither RC02 nor RC03 is accepted for production.

**Implemented/Tested:** three retained manual drivers exercise actual Monty/public CodeMode through
canonical dispatch, standard tasks and a public native-backed protocol projection over actual STDIO,
and FastMCP OTel SDK activation/export-loss behavior. Frozen native-format controls preserve original
invocation identity and distinguish pass, failure, unreported evidence and unresolved endpoint loss.
These controls do not establish live scientific recovery, actual-Codex task support, native physical
drain, persistent-backend recovery, full task conformance or assembled H02 qualification.

**Proposed:** improve existing native lifecycle owners and diagnostic integration before adding
another scheduler or per-subsystem MCP reporting. Scientific Studies already own durable claims,
leases and cancellation; development actions need separate process/terminal-owner assessment.
The public native task-projection experiment is positive, so stale companion serializer commentary
does not justify rejecting that route. Standard Docket remains an alternative with explicit
redelivery/retention/native-effect obligations. CodeMode has useful flexible composition behavior;
the tested full-provenance envelope increased serialized bytes, so no context/latency benefit is inferred.

OpenTelemetry has a promising information-access role: existing Rust spans lack an OTel exporter,
while SDK activation supplies real FastMCP spans and parentage. That is not a durable job/result owner.
The maintainer is preparing documentation for a **separate thorough review**; no deployment or Rust
instrumentation work began. F02/I11 retains the application/database seam and is not closed here.

The current changes are integrated directly on main, with unrelated concurrent review edits preserved.
No worktree remains. The fresh-Codex journey was not started before this pause; resume only from the
owning remaining obligations when instructed.

**Tested, baseline zero:** final `scripts/pse-env -- .venv/bin/python -m
scripts.tests.capability_composition_inquiry --output build/capabilities-52o/composition-final`,
the corresponding `capability_tasks_inquiry --output build/capabilities-52o/tasks-final` and
`capability_telemetry_inquiry --output build/capabilities-52o/telemetry-final` all passed. The task
driver took 6.85 seconds and telemetry 3.35 seconds in this run; these are test durations, not
performance comparisons. Their result JSONs and captured stdout/stderr remain in that parent.
The composition driver records ten named scenario groups and its independent three-way comparison.

**Tested, baseline zero:** `scripts/pse-env -- .venv/bin/python -m unittest
scripts.tests.test_capabilities scripts.tests.test_capability_coherence
scripts.tests.test_capability_observation scripts.tests.test_capability_read_contracts
scripts.tests.test_capability_rust_references` passed 55 tests in 10.45 seconds. This covers current
endpoint visibility/validation, source coherence, observation, typed readers and original references;
it does not execute a scientific or Rust product suite.

**Tested, baseline zero:** final `scripts/pse-env -- just lint-py` passed repository-wide Ruff;
`just typecheck` passed full Pyrefly including warnings with zero diagnostics and three unchanged
configured suppressions. Scoped probe checks also passed without added type suppressions.
`scripts/pse-env -- just codegen-capabilities-check` passed; no production declarations or generated
files changed. `just turn-end` passed ADR indexing and formatting. Dependency readiness passed
`just ready`; the final dependency/group update also preserves the installed native extension and
does not build it. No Clippy, full hygiene, integrated product campaign or actual-Codex probe was
selected for this pause closeout of Python inquiry drivers/documentation.

**Failed, baseline zero:** `just docs` failed because concurrent
`docs/design_review/reviews/design_review_rust-build-reuse_2026-10-24.md` links
`functional_reviews/probes/rust-build-reuse-sccache-key-shim-2026-10-24.py` without selecting that
attachment for publication. Both were already unrelated concurrent work; this closeout leaves them
and their indexes untouched rather than changing another owner's publication scope. Documentation
publication is therefore not qualified by this checkpoint. Source changes, static checks and the
targeted functional results above retain their independent passing scope.
