---
title: "Plan 52: Agent effectiveness and engineering workflows"
status: in-progress
date: 2026-10-23
adrs: []
review_sources: [docs/functional_reviews/agent-effectiveness-opportunities-2026-10-23.md, docs/functional_reviews/agent-tool-discovery-and-access-2026-10-24.md, docs/functional_reviews/unified-fastmcp-capability-interface-2026-10-24.md]
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
| Traces | Server→Surrealism parentage exists; application-originating context is not established. F02 targets that exact seam, preserving finite capture and independent Plan40 qualification. |

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
selected need. Ordinary resources/prompts/skill providers fit the target. OTel, response bounds,
selective preparation and proxy providers have concrete owned routes in 52i–52k. The library
mechanism decisions remain explained there and in the source review rather than another register.

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
| RC09 | Finite application-to-database trace correlation, coordinated with Plan40 | F02 extends existing capture/transport owners, publishes blueprint §23.1 through design route and links exact Plan40 boundary without transferring its campaign. |

## Series and cross-plan ownership

| Companion | Owned target |
|---|---|
| [52a](52a-context-navigation-and-semantic-editing.md) | Task packets, precise cross-language paths, semantic editing, codemods and design consequences |
| [52b](52b-local-knowledge-and-research.md) | Source/history retrieval, exact library research and derived development knowledge |
| [52c](52c-behavioral-and-scientific-verification.md) | Discriminating test techniques, numerical investigations, reduction and safety diagnosis |
| [52d](52d-execution-and-qualification-efficiency.md) | Owned transition watch, feature resumption, build equivalence and host pressure |
| [52e](52e-reproducible-experiments-and-isolation.md) | Historical replay, database fixtures, regression experiments and optional isolation |
| [52f](52f-evidence-analysis-and-trace-correlation.md) | Explicit outcome comparisons and finite end-to-end trace investigation |
| [52g](52g-agent-and-operator-workflows.md) | Agent-facing composition, skill discovery/evaluation, integration and optional operator UI |
| [52h](52h-qualification-and-publication.md) | Actual-consumer reconciliation across 52a–52k, assembled acceptance, scope-end checks and publication |
| [52i](52i-canonical-capabilities-and-progressive-reveal.md) | Canonical exposure model, exact navigation, deterministic reveal and progressive instructions |
| [52j](52j-native-contracts-and-tool-adaptation.md) | Native schema/admission fidelity, host contracts, whole-repository tool adaptation and retirement |
| [52k](52k-unified-entry-point-and-derived-interfaces.md) | One attachment, lazy integration, loaded revisions, derived consumers and interface observation |

Plans50–51 remain completed foundations. Their qualified interfaces are reusable; their counts
are not Plan52 acceptance. Plan32 retains document/artifact lifecycle ownership. Plan40a–d's
focused deliveries exist; Plan40e retains its assembled profile/locality campaign and finding
closure. F02 neither completes that campaign nor reinstates withdrawn environment/provenance
proof work. Plan28's separate scientific campaigns are not prerequisites merely because a
representative native case is used here. Do not edit/retire those plans as part of authoring.

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
| O25 | scheduled | [F02](52f-evidence-analysis-and-trace-correlation.md#F02) | Exact finite trace continuity or explicit unsupported seam |
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
| I11 | open | [F02](52f-evidence-analysis-and-trace-correlation.md#F02) | Exact SDK/transport context propagation; no fake continuity |
| I12 | resolved | [G02](52g-agent-and-operator-workflows.md#G02) | Operator selected one canonical FastMCP endpoint on 2026-10-24; optional direct-versus-MCP adoption inquiry superseded. Actual host, schema and lifecycle questions remain Q01–Q06 below; no delivery claim |
| I13 | open | [G05](52g-agent-and-operator-workflows.md#G05) | Interactive evidence use versus standalone report/plot; no product frontend |

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
| O01 | scheduled | [CAP01/CAP02/CAP05](52i-canonical-capabilities-and-progressive-reveal.md#CAP01) | Canonical model, bounded classification and source-only dispatch delivered; ADP05 now supplies native reconciliation, while derived consumers remain MCP04. |
| O02 | scheduled | [CAP03/MCP02](52i-canonical-capabilities-and-progressive-reveal.md#CAP03) | Atomic process-scoped reveal/reset and actual Codex exact read adaptation delivered; broader host resource/prompt and adapted-tool journeys remain MCP02. |
| O03 | scheduled | [CAP04/G03](52i-canonical-capabilities-and-progressive-reveal.md#CAP04) | Progressive native-owner guidance and planning-only selection delivered; actual editing/task-effectiveness evaluation remains G03. |
| O04 | resolved | [ADP01/ADP02](52j-native-contracts-and-tool-adaptation.md#ADP01) | Typed host contracts and generated native schemas/receivers delivered with malformed/contextual refusal and actual native acceptance; see 52j Outcome. |
| O05 | resolved | [ADP03/ADP04/ADP05](52j-native-contracts-and-tool-adaptation.md#ADP03) | Typed extraction, native/specialist adaptation, single-session migration and explicit all-family native retention delivered; see 52j Outcome. |
| O06 | resolved | [MCP01/ADP04](52k-unified-entry-point-and-derived-interfaces.md#MCP01) | Cheap single bootstrap, lazy actual language/database connections and independent service identity/admission/drain delivered; Q03 resolved. |
| O07 | scheduled | [ADP02/G01/G02](52j-native-contracts-and-tool-adaptation.md#ADP02) | ADP02 outcomes, bounds, handles and cancellation are delivered; assembled G01/G02 interaction consumers retain their own obligations. |
| O08 | scheduled | [MCP04](52k-unified-entry-point-and-derived-interfaces.md#MCP04) | Derived reference; conditional complete/fresh-reveal Cyclopts CLI preserves absent/null/empty semantics or records supported non-adoption. |
| O09 | scheduled | [MCP03/CAP02/CAP04](52k-unified-entry-point-and-derived-interfaces.md#MCP03) | Coherent loaded schema/handler/instructions, relevant restart, native source freshness separate, no global per-read reconstruction. |
| O10 | scheduled | [MCP05/F01/F02](52k-unified-entry-point-and-derived-interfaces.md#MCP05) | Existing evidence/trace correlation and actual interface failures; no duplicate store or fabricated end-to-end continuity. |


## Unified-interface inquiries

The single-surface direction and a high-functioning Codex integration are settled design premises.
On 2026-10-24 the maintainer confirmed prior successful MCP attachment through environment JSON.
Q01 selects and verifies the best current configuration/presentation; it is not a feasibility gate
or a reason to defer the architecture. These implementation questions still need actual evidence;
the source review's library-client probes do not establish Codex attachment, full native fidelity
or product integration. Preserve failed/inconclusive evidence and update dependent packages here.

| Inquiry | State | Owner | Decision / completion evidence |
|---|---|---|---|
| Q01 | open | [MCP02](52k-unified-entry-point-and-derived-interfaces.md#MCP02) | Actual Codex STDIO browse/describe/select/exact-read/reset passed; host retained its initial tool inventory, so canonical same-endpoint read adaptation is selected. Broader resource/prompt and future effectful-operation presentation remain to qualify; see 52i Outcome. |
| Q02 | resolved | [ADP01](52j-native-contracts-and-tool-adaptation.md#ADP01) | Resolved by the generated directional catalogue and original codecs/receivers, authoring/native separation, local references, numeric/enum/contextual refusals, snapshot/digest fences and six actual native unit/component/integration cases. See 52j Outcome. |
| Q03 | resolved | [MCP01/ADP04](52k-unified-entry-point-and-derived-interfaces.md#MCP01) | Resolved by lazy owner-task connections, actual Python/Rust and selected database journeys, unavailable/stale/cancellation controls and authenticated service/context/kernel drain. See 52j Outcome. |
| Q04 | resolved | [ADP03](52j-native-contracts-and-tool-adaptation.md#ADP03) | Typed shared archive/replay and diagnostic kernels; fixed process boundaries for native execution; original Gate/selection/invocation owners for qualification. Actual archive/replay/action/result consumers and scoped Rust execution passed. See 52j Outcome. |
| Q05 | open | [MCP04](52k-unified-entry-point-and-derived-interfaces.md#MCP04) | Concrete CLI consumer decision; complete generation, exact activation on fresh connection and absent/null/empty forwarding. Non-adoption needs reason/revisit trigger. |
| Q06 | open | [MCP03](52k-unified-entry-point-and-derived-interfaces.md#MCP03) | Loaded guidance/schema/handler coherence, relevant restart/reload, source-only availability and actual catalogue preparation work; no blanket repo hash. |


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
| F01 | planned | [52f](52f-evidence-analysis-and-trace-correlation.md#F01) | Exact paired evidence comparisons |
| F02 | planned | [52f](52f-evidence-analysis-and-trace-correlation.md#F02) | Finite trace seam and I11 |
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
| MCP02 | in-progress | [52k](52k-unified-entry-point-and-derived-interfaces.md#MCP02) | Actual Codex source-only exact-read journey delivered; broader attachment consumers/Q01 remain |
| MCP03 | in-progress | [52k](52k-unified-entry-point-and-derived-interfaces.md#MCP03) | Native schema byte/fingerprint coherence delivered; broader loaded-handler/restart cases and Q06 remain |
| MCP04 | planned | [52k](52k-unified-entry-point-and-derived-interfaces.md#MCP04) | Derived reference and conditional faithful CLI/Q05 |
| MCP05 | planned | [52k](52k-unified-entry-point-and-derived-interfaces.md#MCP05) | Interface observation and evidence integration |

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
The original and unified-interface accepted rule tables are distinct sources. If a new rule
impact appears beyond them, present that exact consequence before
fixing the dependent target. Acceptance is the required decision/declaration in place and its
actual consumer consuming it, not completion of a generic paperwork milestone.

### Dependency order

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

For the unified extension, CAP01–CAP05, ADP01–ADP05 and MCP01 now deliver exact discovery,
guidance, real host/native operations and owned lazy specialists. Q02–Q04 are resolved at the
52j focused acceptance boundary. MCP02 retains broader actual Codex presentation, MCP03 retains
remaining loaded-handler/restart cases, and MCP04/MCP05 retain derived consumers and observation.
New entry points still extend CAP05's canonical classification through their binding owner.
G02/G03 evaluate real workflows, F01/F02 and MCP05 integrate evidence, MCP04
derives references and any selected CLI. H01 reconciles all required and conditional scope before
H02–H04. These are contract/evidence dependencies, not whole-document serial barriers. Preserve
C04's separately recorded denied execution boundary; interface work does not authorize retry.

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
