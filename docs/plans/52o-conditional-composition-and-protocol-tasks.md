---
title: "Plan 52o: Conditional composition and protocol tasks"
status: draft
date: 2026-10-24
adrs: []
review_sources: [docs/functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md]
doc_role: plan
doc_owner: docs/plans/52-agent-effectiveness-and-engineering-workflows.md
doc_topics: [current-work, design, libraries, environment, validation]
---

# Plan 52o: Conditional composition and protocol tasks

## Selection and intended value

**Proposed investigations, not selected production adoption:** assess usability-review
[O15](../functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md#O15) and
[O16](../functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md#O16) against useful
simpler alternatives. On 2026-10-24 the operator accepted RC01 but selected investigation only for
RC02/RC03. The [coordinator](52-agent-effectiveness-and-engineering-workflows.md#usability-rule-decisions)
owns that decision, inquiry state and any later adoption. A favorable experiment is not permission
to enable model-authored execution, task infrastructure or altered effect/cancellation semantics.

The inquiry must identify a real consumer advantage and a fitting lifecycle, rather than demonstrate
that a library feature can be installed. It can close with the supported existing route, explicit
limitations and an observable revisit trigger. An adopted route needs its explicit rule decision,
actual integration/caller migration and native/host acceptance before its delivery can close.
Unselected infrastructure is not required for the rest of Plan52; findings/dispositions still are.

## Foundation assessment and comparison baseline

**Interface-checked:** canonical fixed invokers delegate to normal FastMCP tool dispatch and include
effectful bootstrap routes. Exact discovery therefore does not itself enforce read-only execution.
Fixed worker actions retain attached signal/drain semantics; native watch/read already supplies
detached observation. O10's [LIFE02](52n-readiness-progress-and-edit-lifecycles.md#LIFE02) foreground
progress and early handles are the first comparison baseline. CON01/CON02 and F01 supply meaningful
typed selected questions before composition is assessed.

Core3.4, Heuristics1.0, Test Assessment1.2 and relevant PS-10/PS-11 shape this assessment.
AP-03/AP-04/AP-07 and H8/H24/H25/H27 require composition to preserve authority, examined-work bounds,
partiality and recovery. No second scheduler/result authority, semantic search or parallel MCP server
is selected. [G05](52g-agent-and-operator-workflows.md#G05) owns the separate O17 operator interaction
inquiry; a UI or elicitation candidate is not a reason to adopt tasks or CodeMode.

## Work packages

<a id="OPT01"></a>
### OPT01 — Bounded exact-discovery read-composition inquiry

Choose an actual ad hoc immutable-read investigation after meaningful selected answers exist.
Compare existing small calls, an owner-native typed composed question under G01/F01, and FastMCP4.1.3
[`CodeMode(discovery_tools=[ListTools(), GetSchemas()])`](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/docs/servers/transforms/code-mode.mdx)
or a custom exact catalogue factory. Do not include the default ranked discovery. A recurring stable
intent belongs in a typed owner operation when that preserves meaning with less machinery; F03's
multi-question analysis remains separately selectable and is not implicitly adopted here.

Inspect actual [catalogue/execution dispatch](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/fastmcp_slim/fastmcp/server/transforms/catalog.py)
and determine execution eligibility independently of what discovery lists. A prospective read route
must exclude actions, indirect action invokers and selection/control changes as appropriate to its
declared effect contract. Test actual callbacks and indirect paths. Provider placement alone is not
established isolation; do not globally replace the current bootstrap listing as a feasibility shortcut.
Retain one endpoint and truthful annotations, source cuts, native interpretation and original readers.

Bound callback count, live intermediate data, cumulative output and the full operation lifecycle.
The sandbox compute timeout excludes callback wait; it is not a complete workflow deadline. Sandbox
code can catch tool errors, unhandled failure interrupts its chain, and earlier effects are not rolled
back. These facts preclude treating sandbox failure as transactional rollback or generic retry safety.

Settle the inquiry with source/host evidence for exact discovery, supported wrapper dispatch,
execution-policy enforcement, cancellation, partial/stale members and a real useful investigation.
If benefit is unsupported or the contract needs disproportionate machinery, select typed/small reads
and record the unmet consumer trigger. If adoption is recommended, present RC02's exact execution/effect
policy for operator selection before implementation and add the selected migration/acceptance slices
to this owner. No benchmark is mandatory; any numerical context/latency claim requires actual measurement.

<a id="OPT02"></a>
### OPT02 — Detach/reconnect need and task-protocol inquiry

First compare LIFE02 foreground feedback plus original start/handle/watch/read against an actual
detach/reconnect need. If those are adequate, retain them and defer task infrastructure with a concrete
consumer trigger. Do not add Redis/Valkey or Docket merely to improve progress messages.

For a demonstrated gap, inspect FastMCP4.1.3
[`fastmcp-tasks`/`TasksExtension`](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/docs/servers/tasks.mdx),
asynchronous tool requirements, modern protocol negotiation and Docket backend lifetime. An in-memory
backend is process-local/ephemeral, not endpoint-loss durability. External persistence has additional
operational obligations requiring a concrete need and selection. Keep free-local deployment constraints.

Resolve outer protocol dispatch explicitly. Existing `capabilities_invoke_action` and CodeMode call
inner tools through `server.call_tool`; task-optional inner calls run foreground and task-required
inner calls refuse. Adding `task=True` below those wrappers is insufficient. Use the
[task/catalogue contract tests](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/tests/tasks/server/test_task_catalog_transforms.py)
to guide the exact wrapper inquiry, not as evidence of this host's negotiated behavior.

Map host task initiation/status/result/cancellation to original native identity, admission and
drain without a second result truth or automatic retry. Determine what survives observer loss,
endpoint loss and restart, and how an authentic unknown effect is inspected without replay. A host
task completion or cancellation cannot certify scientific outcome, rollback or native settlement.

Settle with the supported simpler route or a concrete recommended outer/native lifecycle. Positive
adoption requires explicit RC03 selection, exact dependency declarations, actual compatible-host
negotiation and wrapper execution, reconnect/loss/cancellation/idempotency controls and independent
native-result/drain assertions. Then integrate affected callers and delete replaced orchestration;
do not retain a parallel scheduler. Unselected backend infrastructure remains deferred.

## Dependencies, evidence and publication

**Proposed:** OPT01 depends on CON01/CON02 and relevant F01/G01 selected-read semantics, not entire
companion completion. OPT02 follows the delivered LIFE02/native observation comparison. The inquiries
are independent of each other and neither blocks ordinary contract/guidance/lifecycle improvements.
Use bounded source inspection or focused feasibility probes only where a material question needs
them; do not install a production path to research feasibility. Exact pins and affected targeted checks
apply to any deliberately selected dependency, without wholesale lockfile re-resolution.

Record inquiry conclusions once in the coordinator with evidence, supported alternative and revisit
trigger. Update this document's selected design if the operator later adopts a route. G02/G03/H02
exercise any adopted path with the actual host; H03 performs scope-end checks. Preserve the difference
between source/interface checking, in-process tests, actual host behavior and independent native truth.
**Interface-checked** foundations do not establish adoption or a measured benefit. No new task,
sandbox, service, host probe or product qualification was run to author this document.
