---
title: "Plan 52g: Agent and operator workflows"
status: draft
date: 2026-10-23
adrs: []
review_sources: [docs/functional_reviews/agent-effectiveness-opportunities-2026-10-23.md]
doc_role: plan
doc_owner: docs/plans/52-agent-effectiveness-and-engineering-workflows.md
doc_topics: [current-work, environment, design, diagnostics, documentation]
---

# Plan 52g: Agent and operator workflows

## Target and responsibility

**Proposed:** compose the underlying capabilities into a usable Codex/operator journey without
another dispatcher, policy ledger or mandatory UI. Owns O26/O27/O29/O30;
[the coordinator](52-agent-effectiveness-and-engineering-workflows.md) owns I12/I13, RC07/RC08
and current state. Native commands, architecture and plans keep their authority.

The actual journey is: describe task/environment → inspect changes/owners and selected history →
select a targeted command → obtain its exact handle → watch/read its native outcome → compare
explicit source cuts → inspect selected evidence → assemble a reviewable integration/handoff.
Unavailable capability is a truthful result, not permission for hidden setup or a fake success.

Reuse `workspace_context`, `activity`, `run_inspection`, `validation_result`, documentation/native
scope and existing result readers. Instructions already route by scope; refine discovery where
real tasks need it. Existing Codex access and local tools suffice; no paid evaluation platform.

## Work packages

<a id="G01"></a>
### G01 — Thin action/result composition and first consumers

Define shared interaction meanings alongside the actual owner commands: source basis, selected
context, effects, exact handle, completion/omission, continuation and next meaningful read/action.
This is not a demand for one envelope type or a universal action enum. Use CLI JSON and readable
text over literal argv; read and execute remain distinct. Existing environment/ownership/correctness
routes apply to action execution. Observation does not acquire science/cancel permission.

The early slice accompanies A01/D01/current result reads and settles only what those consumers
need. Later slices compose F01 comparison and E03 investigations. Extend existing discovery
metadata; do not maintain a second editable catalogue of CLI grammar/defaults. If a command's
structured interface changes, migrate its actual callers in the same package. Host-only outputs
stay with host owners; public Rust/Python documents use the existing catalogue/generators.

Acceptance: a fresh selected-store Codex session traverses context→targeted command→handle→watch→
real result, including environment-shaped failure, unsupported capability and truncated detail.
Configured/available/attached/exercised remain distinguishable. A command exit, process return,
test result, resource drain and scientific outcome are not collapsed into a single success bit.
The workflow is usable before knowledge models or operator UI. Remove replaced ad hoc output
parsing once consumers use real structured results.

<a id="G02"></a>
### G02 — Conditional agent transport and packaging

Resolve I12 after a useful direct G01 workflow exists. A thin MCP facade may expose selected
context/watch/result/comparison actions when typed discovery or reduced interaction friction
benefits the actual agent. It delegates to the existing owners and handles, retaining bounds,
effects and failures; it does not reimplement scheduling, result interpretation or credentials.
The native SurrealDB MCP is a database query transport, not project-specific operation semantics
or Agent Memory.

An optional Codex plugin may package existing skills/MCP wiring if the exact installed client
supports it. Reuse the session launcher/selected-store configuration rather than create another
installer. Official [plugin concepts](https://developers.openai.com/plugins/concepts/plugins)
are a lead; exact local support must be qualified. No new model/hosted-service dependency.

Acceptance: compare one real multi-step task using direct commands and the selected thin
transport, preserving the same owner outcomes and unavailable cases. If no meaningful benefit,
record direct-CLI selection and revisit on a concrete recurring discovery/interaction problem.
Positive adoption includes actual fresh-session use, cancellation/lifetime behavior and setup
instructions; an exposed tool list alone is not integration. Do not wrap every command by quota.

<a id="G03"></a>
### G03 — Progressive instructions and representative agent evaluation

Improve native AGENTS/rules/skills entrypoints with concise routing to actual environment,
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
authority route; improve presentation without silently changing governance. G01 discovery and
B02 attributed history can supply evidence but neither is a universal prerequisite.

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
and reviewable commit selection. The read does not stage, merge, delete, certify completion or
retire anything. A later authorized action uses normal ownership checks. Keep requested independent
completion audits separate; this is routine integration assistance. Migrate duplicated manual
closeout readers only when the native view actually replaces them.

<a id="G05"></a>
### G05 — Optional operator investigation view

Resolve I13 around one concrete F01 baseline/candidate investigation. First deliver readable
headless comparison and standalone scientific plots. If interaction helps Paul filter outcomes,
see missing members and inspect linked source/evidence, use a small local static interactive page
or notebook consuming the same typed data. No mandatory daemon or simulator product frontend.

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
all-capabilities prerequisite. G02/G05 are conditional consumer decisions. G03 uses actual task
observations; G04 consumes declared ownership and source relationships. Targeted command/parser/
composition fixtures accompany changed behavior, while 52h owns assembled agent/operator journeys
and scope-end checks. No tool transport or dashboard changes native terminal, scientific, scope
or authorization truth.
