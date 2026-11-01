---
title: "Plan60e: Validation relevance and workflow adoption"
status: active
date: 2026-10-31
doc_role: plan
doc_owner: docs/plans/60-engineering-code-intelligence.md
doc_topics: [current-work, design, environment, diagnostics, validation]
review_sources: [docs/functional_reviews/engineering-workflow-code-intelligence-2026-10-31.md]
adrs: []
---

# Validation relevance and workflow adoption

## Purpose and foundation assessment

**Proposed:** turn existing and new evidence into useful agent inquiry and test-relevance
workflows for [O07/O09 and consumer O08](60-engineering-code-intelligence.md#opportunity-dispositions).
The source review's process/value table identifies design audits/reviews, functional exploration,
all three library-leverage stages, plan preparation/creation/execution and improvement as
consumers. Skills retain reasoning, depth and publication discretion; the CLI does not score
their compliance or automate conclusions.

Current `pse-cli` guidance, saved queries/scopes/JSON/explain, original diagnostic inspection,
native test inventories/Cargo units, fixture declarations, retained outcomes and current
test-selection owners already support useful journeys. Use them first. A related test is not
coverage, a source reference is not an assertion, and an assertion is not proof of oracle
independence. Test Assessment1.2 informs the evidence distinctions; no new runner or test-control
framework follows. Blueprint §21.7 and the existing qualification guide retain authority.

## Target interactions

Offer short optional examples where they change agent decisions: authority-to-consumers,
wrapper-to-library API, proposed change-to-contract/test footprint, original failure-to-helper
or setup definition. Identify when source/syntax is enough, when selected HIR or acquired API
evidence helps, and when an explicit compiler probe is justified. A useful question should not
force expensive preparation. Teach stopping at a sufficient bounded answer and expanding a
particular unknown, rather than prescribing a mandatory skill pipeline.

Validation relevance joins original test population/setup/fixture and source evidence. Classify
direct test reference, shared fixture/setup dependency, observable assertion structure and an
agent's proposed revealing check separately. Preserve the original test-selection owner for
actual current selection and the original report for outcome/phase. Never rerun tests as a side
effect of inspection or infer adequacy from filename/relationship matches.

## Work packages

<a id="e01"></a>
### E01 — Guidance using delivered capabilities first

Consumes the delivered Plan59 public surface; can start alongside A01. In `pse-cli`'s focused
references and materially relevant process-skill references, add concise examples only where
they teach a decision or remove repeated manual evidence assembly. Cover the requested process
categories through shared examples rather than duplicate command catalogs in every skill:
design audit/review; functional exploration; codebase/library/opportunity leverage; execution;
plan preparation/creation; plan improvement. Keep each process's existing lens, authority and
publication boundary. Targeted guidance is a valid outcome without new CLI code.

Examples must use actual parser/schema syntax, returned guarded references and visible provider
limits. Start with existing library signature queries, `--docs`/declaration inspection,
importability/use joins and source-only inquiries. Explain effective query/provenance via
existing `search --explain` and native grammar/error facilities. Do not add generic briefs,
mandatory review stages or another process DSL. If shared-query discovery actually becomes
difficult, assess the coordinator's passive-catalog trigger with A02; it is not required now.

Acceptance: an agent can choose the smaller source-only route, compose a current API question
and stop/expand based on an explicit unknown. Exercise examples through actual public commands,
not just help compilation. Delete obsolete advice for superseded manual assembly in the edited
references, preserving scope-specific guidance and distinct process roles. Later new-feature
examples belong with their implementing packages/E03, not speculative working commands here.

<a id="e02"></a>
### E02 — Explain test, fixture and setup relevance

Consumes A01, B01's relevant source relationships and original native inventory/Cargo-unit/
fixture/report owners. Extend existing inspection and test-selection views to connect an
original failure to its phase, production definition, fixture/setup helper and candidate tests.
Expose observable assertion structure separately from an inferred contract or proposed check.
Reuse current conservative affected selection and exact current result-to-package selection;
do not relabel either as dependency-impact proof.

Acceptance distinguishes a production compiler-selection error from a stripped-PATH fake's
missing compiler inputs, direct test usage from shared-helper fan-out, and a referenced operation
from an assertion about its important invariant. Include a test whose expected value shares
the production calculation: links may be correct while independent assurance remains unknown.
Missing/malformed/partial original reports remain incomplete. Inspecting relevance must not
execute tests, generate fresh inventories or silently normalize an original failure away.
Delete replaced heuristic filename/status assembly only where actual consumers migrate; retain
native execution, selection and original reports. No automatic rerun, coverage gate or event
reconciliation is added.

<a id="e03"></a>
### E03 — Integrate actual engineering journeys and focused adoption

Consumes implemented A02/A03, B02/B03, C01/C02, D02/D03 and E01/E02. Integrate guidance and
public consumers incrementally as each prerequisite becomes usable, then exercise the assembled
journeys below. These are acceptance questions, not a compulsory process for future agents.

| Journey | Useful answer and revealing challenge |
| --- | --- |
| Responsibility trace | Follow a selected request/policy type through construction/validation/dispatch and shared consumers; namesakes, callbacks and an out-of-output-scope intermediate challenge the graph |
| Library fit | Query a structured shape, inspect associated APIs/docs/conditions and actual local wrappers; absent defining artifact or existential applicability cannot become a fit verdict |
| Change footprint | Align source/API observations and selected before/after worlds, then follow relevant original contracts/packages; cfg-only change and historical blobs challenge correspondence |
| Lifetime investigation | Expand captures/adjustments/await/error observations around cancellation and CPU completion; interior mutation, guard retention and unknown indirect work challenge any broad claim |
| Contract association | Navigate explicit code/generator/section/plan/library links with their basis/status; a rejected-alternative mention cannot become authority |
| Original failure to validation | Follow original result/phase into production/helper/setup and candidate tests; preserve weak/shared assertions and incomplete outcomes without automatic execution |

Use an unfamiliar subsystem as well as the review's Request/compiler/cancellation examples so
the interface is not tailored to one fixture. Select modest demand and compact views, inspect
one guarded expansion, and observe actual omissions/context. Validate saved/JSON/flag forms
where relevant and actual complete human/JSON/NDJSON output. Assert that source-only paths
remain available and missing prepared capabilities are explicit, without implicit builds or
acquisition. Record command, context, question and actual scope of evidence in final outcomes;
do not require a new per-query receipt system.

Revise targeted skill references based on real use, remove replaced extraction workarounds and
keep full original evidence reachable. A new command earns its place only when current
composition/guidance cannot serve a named question. No numerical latency/token/memory benefit
is asserted here; measurements are needed only for claims of that kind. This package supplies
actual consumer evidence to Q01, not the final whole-series static qualification itself.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| E01 | planned | CLI/process guidance owner | Tested optional examples using delivered interfaces across the requested process categories |
| E02 | planned | Test-relevance inspection owner | Original test/setup/assertion/outcome evidence with honest selection and assurance limits |
| E03 | planned | Engineering workflow integration owner | New capabilities exercised through actual bounded agent journeys and updated focused guidance |

## Verification and handoff

**Proposed:** targeted relevance/adapter tests accompany implementation, with actual public
command examples exercising their consumers. Respect force-validation for actual Arrow-consuming
closures and the repository environment routes for workloads. No unrelated scientific campaign
is needed to publish guidance. Q01 owns the final integrated/static campaign and durable contract
publication; these proposed journeys supply no new Tested or Measured claim until run.
