---
title: "Plan 52g: Agent and operator workflows"
status: draft
date: 2026-10-23
adrs: []
review_sources: [docs/functional_reviews/agent-effectiveness-opportunities-2026-10-23.md, docs/functional_reviews/unified-fastmcp-capability-interface-2026-10-24.md, docs/plan_improvement_reviews/test-architecture-remaining-2026-10-24.md, docs/functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md]
doc_role: plan
doc_owner: docs/plans/52-agent-effectiveness-and-engineering-workflows.md
doc_topics: [current-work, environment, design, diagnostics, documentation]
---

# Plan 52g: Agent and operator workflows

## Target and responsibility

**Proposed:** compose the underlying capabilities into a usable Codex/operator journey without
another dispatcher, policy ledger or mandatory UI. Owns O26/O27/O29/O30;
[the coordinator](52-agent-effectiveness-and-engineering-workflows.md) owns I12/I13, RC07/RC08
and current state. Source IDs in this paragraph belong to the original review. The subsequent
[unified-interface review](../functional_reviews/unified-fastmcp-capability-interface-2026-10-24.md)
selects one canonical FastMCP entry point. [52i](52i-canonical-capabilities-and-progressive-reveal.md),
[52j](52j-native-contracts-and-tool-adaptation.md) and
[52k](52k-unified-entry-point-and-derived-interfaces.md) own its exposure, adaptation and runtime
implementation. This companion owns useful composed workflows and actual agent evaluation; it
does not duplicate those implementations. Native commands, architecture and plans keep authority.

The actual journey is: describe task/environment → inspect changes/owners and selected history →
select a targeted command → obtain its exact handle → watch/read its native outcome → compare
explicit source cuts → inspect selected evidence → assemble a reviewable integration/handoff.
Unavailable capability is a truthful result, not permission for hidden setup or a fake success.

Reuse `workspace_context`, `activity`, `run_inspection`, `validation_result`, documentation/native
scope and existing result readers. Instructions already route by scope; refine discovery where
real tasks need it. Existing Codex access and local tools suffice; no paid evaluation platform.

## Test-architecture consumer integration

**Proposed:** consume [PI02/PI04/PI10](../plan_improvement_reviews/test-architecture-remaining-2026-10-24.md#PI10)
through delivered 52k contracts and revised 53d/53e owners. G01/G02 retain actual structured
handles, independently useful preparation responsibilities and demand-shaped protected result
reads; they do not define another preparation policy or evidence format. An available native route
and a callable canonical operation remain distinct exposure decisions at the existing catalogue.
Source-only describe/browse does not ensure a build, import the extension or acquire a store.

When readers change, migrate G01/G02/G05 with E05's versioned owner boundary: mandatory native
outcome and selection/source-cut summary, explicit selected/failure/source detail and full export,
plus per-section presentation completeness. A compact answer is useful only if essential native
claims survive and omitted detail has a meaningful next read. Protected evidence links retain
their original custody, including reused older origins and service-down inspection. No generic
wire envelope, parser mirror, operator UI or new result authority is required.

G02/G05 may consume optional 52f F03 only after separate selection. Neither the comparison workflow
nor original Plan53 completion depends on it. The operator's existing optional view decision I13
remains separate from F03 and is not implicitly selected by plan amendment.

## Work packages

The [usability review](../functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md)
extends the remaining workflows: O12 has its primary disposition in G01, O14 in G03 and O17 in G05.
52l–52n own the new request/result/diagnostic, guidance/dependency and lifecycle implementations.
52o owns conditional CodeMode/task inquiries; neither is implicitly adopted by these workflows.
The coordinator retains all source-qualified opportunity and inquiry state.

<a id="G01"></a>
### G01 — Thin action/result composition and first consumers

Define shared interaction meanings alongside the actual owner commands: source basis, selected
context, effects, exact handle, completion/omission, continuation and next meaningful read/action.
This is not a demand for one envelope type or a universal action enum. Preserve independently
useful native JSON/readable results and exact argv at genuine process boundaries; expose typed
operations through the canonical FastMCP bindings rather than parsing formatted output. Read and
execute remain distinct. Existing environment/ownership/correctness
routes apply to action execution. Observation does not acquire science/cancel permission.

The early slice accompanies A01/D01/current result reads and settles only what those consumers
need. Later slices compose F01 comparison and E03 investigations. Extend existing discovery
metadata through CAP01/CAP02; do not maintain a second editable catalogue of CLI grammar/defaults.
A01/D01/current result early slices remain delivered at their recorded scope; they do not establish
the rest of G01 or the canonical endpoint. ADP02 implements operation contracts/handles while
G01 verifies that composed consumers retain the shared meanings. If a command's
structured interface changes, migrate its actual callers in the same package. Host-only outputs
stay with host owners; public Rust/Python documents use the existing catalogue/generators.

For usability O12, identify recurring coherent questions such as selected invocation investigation
or explicit baseline/candidate comparison. Compare a typed owner-native composed operation with the
existing small calls; expose new operations only where intent and semantic ownership justify them.
Reuse F01's pairing and original readers, retaining source cuts, unavailable/stale members and
independently complete sections. Shared immutable preparation needs an actual owner guarantee;
one RPC does not imply atomic observation. Do not introduce a general DAG, automatic next-action
agent or prompt-owned scientific/qualification interpretation. Optional F03 remains separately selected.

Acceptance: a fresh selected-store Codex session traverses context→targeted command→handle→watch→
real result, including environment-shaped failure, unsupported capability and truncated detail.
Configured/available/attached/exercised remain distinguishable. A command exit, process return,
test result, resource drain and scientific outcome are not collapsed into a single success bit.
The workflow is usable before knowledge models or operator UI. Remove replaced ad hoc output
parsing once consumers use real structured results.

Include independently expected composed answers and partial/stale-member cases, using CON02's
actual serialization contract. LIFE01/LIFE02 supply explicit selected preparation and authentic early
handles/progress; observer loss is not native settlement or permission to replay an attached action.

<a id="G02"></a>
### G02 — Unified agent workflow integration

I12's former optional transport choice is superseded by the maintainer's selected single FastMCP
surface. Compose actual CAP/ADP/MCP deliveries into useful workflows: exact domain/workflow browse
→ relevant guidance → canonical operation → native handle → watch/read/compare → next action.
Use the qualified MCP02 host route; one attachment internally integrates language/database services
as needed. The SurrealDB MCP remains an internal specialist transport, not project admission or
agent memory. No developer-only endpoint, independent Typer tree or generic shell forwarding.

Migrate existing session instructions and actual agent consumers alongside their usable operations.
CAP/ADP/MCP packages own component registration, parser extraction, lifetimes, revision consistency
and derived documentation/conditional CLI. G02 owns their composed usability and semantic preservation.
A native command remains legitimate for its native purpose; do not preserve a replaced agent
parser/discovery path solely as compatibility. Keep browse/read distinct from execution and setup.

Acceptance: an actual fresh Codex attachment completes context→targeted operation→handle→watch/read
and an evidence investigation, including missing dependency, incomplete/native refusal, detached
observation and unrelated dirty state. Check native outcomes independently of the adapter's output;
an exposed list alone does not qualify integration. Source-only use survives unavailable native
services. Two attachments retain independent reveal/context, and reconnect never replays effects.
A selected CLI consumer, if any, uses MCP04's faithful derivative rather than another interface.
No measured productivity claim is required; behavioral observations and support limits are required.

Consume NAV01's compact instruction-resource index and CON01's meaningful request alternatives
in the real workflow. An unfamiliar agent should construct a valid request from advertised examples,
obtain a useful selected answer and follow an authentic next inspection. Qualify whichever ordinary
revealed-tool or canonical-invoker route the host actually uses. Native preparation/custody contracts
remain Plan53's; adopting an optional 52o route adds its actual-host/effect assertions here without
creating another endpoint or replacing native outcomes with task status.

<a id="G03"></a>
### G03 — Progressive instructions and representative agent evaluation

Consume CAP04's progressive instruction resources and MCP02's actual-host presentation, and
improve native AGENTS/rules/skills entrypoints with concise routing to actual environment,
source/generator, review/planning and qualification owners. Fix repeated discoverability gaps
at their existing owner rather than adding another general policy page. Keep shared library
skills repository-agnostic. Preserve planning-only boundaries and the distinction between a
review recommendation, accepted target, implementation and qualification.

Use the official guidance on
[skills/prompts](https://developers.openai.com/blog/rethinking-skills-and-prompts-for-gpt-6-astra)
and [skill evaluations](https://developers.openai.com/blog/eval-skills) for representative local
task probes, not a mandatory benchmark platform or a model migration. Select four concrete
situations: generated-contract edit, stale-evidence refusal, partial test outcome and planning-only
request. Define independent expected workflow/outcome observations and inspect actual agent
execution using existing Codex access and explicit operator judgment. Static lint alone cannot
prove agent behavior; one successful task does not certify all future reliability.
Qualitatively inspect avoidable builds, broad scans, repeated source reconstruction and excess
instruction loading as well as correctness. No productivity score or timing claim is required.

Acceptance: these bounded tasks reveal correct owner selection, meaningful verification and
truthful partial outcomes without unauthorized implementation in the planning task. Retain
useful local fixtures/evaluation prompts and rationale; do not create another incident ledger,
paid scoring API or permanent evaluation service. Instruction/skill changes follow their existing
authority route; improve presentation without silently changing governance. CAP/G01 discovery and B02 attributed history can supply evidence. Keep CAP04 instruction
serving separate from this package's actual-agent evaluation. Actual Codex access is required for
behavior claims; preserve bounded prompts/observations when an evaluation is incomplete.

For usability O14, pair these agent tasks with focused independent contract expectations: first-valid
examples, tagged branches, absent/null/empty values, nested diagnostic paths, escaped JSON Pointers,
reference-resource scope, input/output direction and complete/partial/unavailable serialized answers.
CON packages own their local tests; G03 evaluates whether those contracts and NAV guidance work together.
Use adopted `jsonschema.Draft202012Validator.check_schema()`/`iter_errors()` where appropriate;
schema acceptance is not native scientific eligibility. Adopted proptest can cover relevant Rust
properties without a new framework quota. Do not derive all semantic expectations from the producer.

Actual Codex evidence is necessary for host refresh, progress notifications and any selected
completion/task/App behavior; in-process clients prove their own path only. Correlate selected
operation, serving revision, original handle and acquisition stage when they answer a real question.
Keep raw arguments, duplicate telemetry stores and F02's unresolved propagation edge outside this work.

<a id="G04"></a>
### G04 — Ownership-aware integration and closeout assistance

Compose Git state, explicit task/worker ownership, A02 generator relationships, native plan state
and selected outcomes into a read-only integration view. Identify changed consumers, unresolved
conflicts, required regeneration, references to retiring material, proposed commit selection and
remaining verification. Git diff alone cannot infer who owns a hunk. Preserve owned/unknown/
unrelated distinctions and explain overlapping changes before proposing an integration action.

Use existing lifecycle/retirement preflight and native checkpoints; do not keep a second closeout
ledger or infer plan completion from file resemblance. Assistance can explain both sides of a
conflict and relevant contracts; a clean text merge is not semantic acceptance. Ordinary discrete
commits remain the execution rhythm, not a new approval gate or requirement for a worktree per edit.

Acceptance: overlapping task edits plus a generated consumer and unrelated dirty file produce
an explicit owned/unknown exclusion, correct generator/regeneration route, targeted test scope
and reviewable commit selection. Expose this read through its canonical CAP/ADP binding, retaining explicit ownership inputs.
The read does not stage, merge, delete, certify completion or
retire anything. A later authorized action uses normal ownership checks. Keep requested independent
completion audits separate; this is routine integration assistance. Migrate duplicated manual
closeout readers only when the native view actually replaces them.

<a id="G05"></a>
### G05 — Optional operator investigation view

Resolve I13 around one concrete F01 baseline/candidate investigation. First deliver readable
headless comparison and standalone scientific plots. For usability O17, compare ordinary typed
parameters/resources with parameterized prompts, elicitation and structured FastMCPApp/Prefab
tables/forms/charts, as well as a small local static interactive page or notebook. Select interaction
only where it helps Paul filter outcomes, see missing members, inspect linked evidence or make a
specific decision. All views consume the same typed owner data/APIs; no mandatory daemon, simulator
product frontend, new HTTP/OAuth system or generic database writer follows from framework availability.

Use the [FastMCP4.1.3 Apps contract](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/docs/apps/overview.mdx)
for fixed typed components when the decision is known. Actual host rendering must be demonstrated;
successful tool transport is insufficient. Modern
[elicitation](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/docs/servers/elicitation.mdx) returns
`InputRequiredResult` and answers initiate a fresh invocation through the pipeline. Gather missing
choices before effects; preserve needed request identity and prevent an effectful prefix from running
again on re-entry. Prefer ordinary parameters when sufficient, and add no confirmation ceremony for
already-authorized work. A preview or physical-result view must preserve its native interpretation
and LIFE03's exact retained-handle semantics rather than infer authority from UI state.

SurrealDB Studio remains a useful schema/query consumer. It must not become a second schema
writer or the authority for scientific results. Typed local queries/APIs, including
[DEFINE API](https://surrealdb.com/docs/reference/query-language/statements/define/api), are an
option only if the selected view needs live data and the exact server supports the contract.
Do not expose a generic database writer merely to display a comparison. Source/evidence links
retain original protection/released-detail semantics; no duplication of durable scientific data.

Acceptance: Paul can inspect a failure→pass pair, unmatched/unexecuted subjects and an inconclusive
measurement, then follow a native result/source. Offline/headless use remains available. Record
whether interaction improves that investigation; if not, select reports/plots and revisit when
a concrete filtering/navigation need is unmet. Positive adoption includes actual operator use
and source freshness, not only a screenshot or a running local server.

## Verification and integration

**Proposed:** G01 is a continuing integration responsibility with an early small slice, not an
all-capabilities prerequisite. G02 implements the selected canonical workflow integration; only G05 retains its optional
operator-view decision. CAP/ADP/MCP own underlying interface mechanisms, and any CLI remains
conditional under MCP04. G03 uses actual task
observations; G04 consumes declared ownership and source relationships. Targeted command/parser/
composition fixtures accompany changed behavior, while 52h and 53g coordinate applicable describe→prepare→execute→watch/read, relevant schema/restart, interrupted protected-read and explicit-comparison executions without duplicating acceptance authority. Share runs only when source/inputs/population/mode/guarantees agree; host attachment, native outcome, scientific adequacy, custody and cleanup retain separate assertions. 52h owns assembled agent/operator journeys
and scope-end checks. No tool transport or dashboard changes native terminal, scientific, scope
or authorization truth.
