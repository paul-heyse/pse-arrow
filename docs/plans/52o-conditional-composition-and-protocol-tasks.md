---
title: "Plan 52o: Conditional composition and protocol tasks"
status: in-progress
date: 2026-10-24
adrs: []
review_sources: [docs/functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md, docs/functional_reviews/opentelemetry-and-agent-observability-2026-10-24.md]
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
If OPT01/OPT02 is adopted, consume [MCP06](52k-unified-entry-point-and-derived-interfaces.md#MCP06)
and [LIFE04](52n-readiness-progress-and-edit-lifecycles.md#LIFE04): preserve admitted effect/route/schema
agreement and check the required native basis at actual execution, including delayed execution.
Initiation-time currency is insufficient after relevant changes; task retries cannot bypass native
admission or replay unknown effects. These follow-ups do not depend on optional framework adoption.

Use bounded source inspection or focused feasibility probes only where a material question needs
them; do not install a production path to research feasibility. Exact pins and affected targeted checks
apply to any deliberately selected dependency, without wholesale lockfile re-resolution.

Record inquiry conclusions once in the coordinator with evidence, supported alternative and revisit
trigger. Update this document's selected design if the operator later adopts a route. G02/G03/H02
exercise any adopted path with the actual host; H03 performs scope-end checks. Preserve the difference
between source/interface checking, in-process tests, actual host behavior and independent native truth.
**Interface-checked** foundations do not establish adoption or a measured benefit. Original plan
authoring ran no task, sandbox, service, host probe or product qualification; subsequent execution
and its distinct limits are recorded below.

## Current telemetry review and planning handoff — 2026-10-24

The separate [OpenTelemetry review](../functional_reviews/opentelemetry-and-agent-observability-2026-10-24.md)
is now delivered. The operator accepted its RC01/RC02 for planning, and the
[coordinator](52-agent-effectiveness-and-engineering-workflows.md#opentelemetry-rule-decisions)
records their exact consequences and governed publication routes. New
[52p](52p-shared-telemetry-and-native-propagation.md),
[52q](52q-local-telemetry-pipeline-and-health.md),
[52r](52r-operational-metrics-and-diagnostic-integration.md) and
[52s](52s-telemetry-queries-and-agent-investigations.md) plan shared instrumentation, native local
collection/querying, metrics and agent investigations. F02 retains the actual database propagation
seam and I11; G/H retain composed workflows and qualification. This authoring implements no deployment.

OPT01/OPT02 and the usability review's task/CodeMode decisions keep their existing boundaries.
Telemetry can explain native progress and loss but cannot provide durable scheduling or recover a
native effect. Conversely, a task status cannot establish trace delivery. Consume the actual telemetry
contracts where useful to the remaining inquiry without treating them as a reason to add worker
processes, adopt tasks or infer measured composition benefits. The checkpoint and telemetry baseline
below remain the historical pause record, including the then-pending documentation/review.

## Execution checkpoint — pause handoff 2026-10-24

The maintainer authorized execution, then requested a pause after concluding current items,
repairing targeted functional/static failures, integrating work, updating documentation and committing.
OPT01/OPT02 remain **in progress**: the bounded library/native-owner inquiries below are delivered,
but the planned fresh-Codex acceptance and final adoption decisions have not run. No production
CodeMode, task extension, worker queue or telemetry deployment is enabled. The maintainer will supply
OpenTelemetry documentation for a separate thorough deployment/design review before that work begins.

The clarified consumer target includes continued inspection after Codex/attachment **and serving
process** restarts, durable queued/status/result recovery and safe reconciliation. It does not promise
uninterrupted native numerical execution. The maintainer prefers a holistic improvement to applicable
native services over adding worker processes, while retaining a serious assessment of task protocols.

### Delivered inquiry artifacts and environment

**Implemented:** three explicit acceptance drivers under `scripts/tests/`:

- `capability_composition_inquiry.py`: real Monty, public CodeMode-generated discovery/execution,
  canonical binding/visibility/dispatch and original invocation/result readers.
- `capability_tasks_inquiry.py`: standard task-extension behavior, actual STDIO endpoint loss and
  a public native-backed task-protocol projection over authentic FileStore records.
- `capability_telemetry_inquiry.py`: ordinary API-only versus SDK-active FastMCP spans, existing
  interface correlation, parentage, a Python-child propagation control and exporter-loss behavior.

These are deliberately named manual drivers, outside automatic `test_*.py` discovery. They do not
add production bindings. `capability-inquiry` in `pyproject.toml` pins fastmcp-tasks4.1.3,
pydantic-monty1.3.0, pydocket0.27.0 and opentelemetry-sdk1.45.1; existing FastMCP4.1.3, MCP2.3.0,
Pydantic2.14.1 and OTel API1.45.1 declarations remain authoritative. The development group includes
the inquiry dependencies so ordinary static checking can resolve the retained drivers. Package
availability does not register extensions, configure an SDK/exporter or start infrastructure.
Initial existing dependency versions were preserved; the lockfile added only the selected families
and required new transitives. No native extension was rebuilt.

### OPT01 — result and recommended direction

**Tested:** one selected invocation and three small frozen native-format controls were inspected
through original readers and real Monty. Independent expected facts distinguish pass, failure and
unreported evidence. These are reader fixtures, not executed Rust tests. The probe compares six
small calls, a fixed typed composition and a flexible CodeMode composition against the same facts.

The public `CodeMode.transform_tools()` hook produces tools that can be registered through the
canonical binding seam without globally installing the replacing transform. Exact factories wrap
`ListTools`, `GetSchemas` and `GetTags`; normal bootstrap, hidden initial state, selection and reset
remain intact. A typed host wrapper supplies an actual result schema. A request-local execution
guard independently limits every dispatch, including a misleading read-annotated tool that attempts
an indirect effect. The test effect count remains zero. Caught callback refusals remain incomplete;
missing references and original partial populations are not converted into complete answers.

Probe bounds are eight callbacks, 1 MiB cumulative callback data, 64 KiB final envelope, Monty's
100 MB intermediate-memory allowance, two seconds sandbox compute and a five-second callback-inclusive
deadline. These are experimental bounds, not workstation/compiler memory policy. Separate tests
force callback/final byte refusal and call-count refusal; a real waiting callback's `finally` completes
before the overall deadline returns. The stale-source case injects a prerequisite refusal; it does
not qualify a changed production callback closure. Existing coherence controls retain that ownership.

**Measured, bounded serialized-size comparison:** the final closeout control returned
7,247 bytes across six small results, 8,888 bytes for the CodeMode answer retaining full callback
provenance, and 1,973 bytes for the fixed typed answer with native references; its derived rows alone
were 276 bytes. Discovery is excluded, and absolute path lengths affect these values. This establishes
no latency, productivity, benchmark or general context-saving claim. Other replays may have different
path-dependent sizes and record them in their own results.

**Proposed:** use typed owner compositions for recurring questions and opt-in, bounded CodeMode for
useful ad hoc branching/reduction over explicitly eligible immutable-result reads. Preserve native
references, source cuts, per-reader completeness and recoverable provenance; sending full raw callback
responses again can defeat the context-transfer benefit. Do not replace provenance with an unqualified
model summary or introduce another store merely to hide it. Keep exact discovery and the execution
closure independent; read-only annotations and provider placement alone are insufficient.

Remaining before RC02 selection: fresh-Codex usability, production-shaped source fencing and concurrent
request isolation, callback/cleanup failure behavior and the final provenance/detail policy. The fixed
typed control is not delivery of G01/F01, and this inquiry does not adopt F03.

### OPT02 — integrated lifecycle result and recommended direction

**Interface-checked:** durable scientific Studies already own queued occurrences, attempt claims,
leases/heartbeat, persisted cancellation and results. Their owners are `pse-runtime`'s `workflow/study`,
`workflow/durable` and `workflow/run`, `pse-operations::canonical_studies`, and Python's `_workflow`.
A new runtime can reopen an exact StudyId and serve eligible work; expired worker claims are fenced
and worker loss is distinct from numerical failure. A process-local RunHandle is not a restart token,
and recovery cannot recreate native solver state or authorize arbitrary redispatch. Development/test
actions instead retain original invocation/terminal records under FileStore while their current MCP
worker remains attached. A persistent test receipt does not supply a durable execution queue.

**Tested:** standard TasksExtension with its memory backend negotiated modern2026-07-28 task requests,
optional/required/forbidden modes and legacy behavior. Optional inner `server.call_tool` calls remain
foreground and required inner calls refuse. An outer asynchronous task wrapper works, but its task ID
does not automatically reach the inner tool context. A forbidden task helper can execute its inline
tool before reporting that no task was produced: helper failure is not a no-effect or replay guarantee.
Protocol `completed` can carry a tool error and a failed fixture outcome. Cancellation acknowledgement
preceded fixture cleanup in the observed case; it did not certify native drain.

Fresh sessions can inspect an original memory task while its endpoint stays alive. Fresh STDIO
processes lose those protocol task IDs while original native FileStore records remain. Forced loss
of a disposable running endpoint left exactly one open native invocation without an invented terminal.
This is evidence of retained uncertainty, not continued execution or a scientific recovery test.

**Tested:** a small public `ServerExtension` plus companion task models projected a frozen native
FileStore result through `tasks/get` and task initiation over real STDIO. A fresh process recovered
the same native identity/terminal, with no Docket scheduler or private serializer call; changed source
interpretation refused. This demonstrates protocol feasibility only: it does not implement full task
conformance, authorization, cancellation, input-required behavior or live scientific recovery.

**Correction:** source comments in the companion's `wire_production` module suggested public task
claim serialization would require its private shim. Actual fresh-process probes contradicted that
assumption. MCP2.3's runner preserves modern extension `resultType` shapes; the proposal must use the
verified public behavior rather than copy the shim or retain the disproved absence claim.

**Interface-checked:** Docket0.27.0 renews active leases and can redeliver abandoned work; these are
at-least-once execution semantics, not exactly-once native effects. Graceful worker shutdown gathers
active work rather than unconditionally killing every body. Forced death remains distinct. Its
default completed result retention is fifteen minutes, while FastMCP's routing/context metadata uses
an additional fifteen-minute buffer refreshed by lookups. TasksExtension does not expose Docket's
execution TTL. A persistent backend alone therefore settles neither recovery-window nor native-effect
reconciliation requirements. These exact-source findings were not a Valkey failure/restart experiment.

**Proposed:** expose native durable start/recover/status/watch/cancel/results with their existing
authorities and use a thin negotiated task projection only where actual host behavior adds value.
For scientific work, integrate existing Study/attempt ownership rather than another queue. For
development actions, first identify and improve the actual process/admission/terminal owner where
continuity is missing; do not relabel FileStore history as a scheduler or generalize scientific
retry policy to commands. Compare the full underlying-service changes, consumers and deletions before
selecting a mechanism. Standard TasksExtension/Docket remains an alternative for a demonstrated
residual scheduling need; no additional worker/service deployment is the default.

Remaining before RC03 selection: actual Codex negotiation; native queued/running/restart reconciliation;
authentic lost-ack recovery; consistent identity/retention/result conversion; delayed execution source
fences; cancellation-to-native-drain mapping; and any required caller/schema migrations. Explicit native
handles/readers remain the supported current route. The restart consumer is now established, so it
must not be dismissed as merely a request for better progress messages.

### Telemetry baseline — separate review, no deployment

**Interface-checked:** Rust already has extensive `tracing` instrumentation, bounded Perfetto/process
captures and operation/checkpoint/service/cgroup readers. The inspected Cargo manifests have no Rust
OTel bridge/exporter. Existing FastMCP observation middleware obtains active OTel IDs and adds native
references while explicitly preserving export-unverified and continuity-unestablished status. The
ordinary endpoint configures no OTel SDK/exporter. Finite SurrealDB OTLP capture already exists under
`scripts/surreal_telemetry.py`, including bounded collection/restoration; application Rust→SDK→server
parentage is not established by server→Surrealism spans.

**Tested:** enabling an in-memory OTel SDK in the isolated probe produced actual FastMCP spans and
matching interface metadata. A successful read had a six-span parent chain; a refusal recorded ERROR
without inventing native references. Standard W3C propagation reached a Python child, not Rust or the
database. A rejecting exporter did not turn a successful read into failure, and successful force-flush
did not prove export delivery. Native result, native drain and end-to-end continuity remain separate.

**Proposed, awaiting the maintainer's documentation and separate review:** assess standard Rust
`tracing`→OTel export, explicit child/SDK propagation, selected telemetry storage and bounded exact
trace/operation queries through FastMCP before building bespoke trace-surfacing mechanisms. Assess
metrics/logs/profiling against actual questions: telemetry can explain observed stages/timing/errors;
typed native readers still own admission, outcome and drain. Cover collector absence/backpressure,
sampling/loss, retention, process incarnation, bounded overhead, shutdown and backend query design.
Reuse F02/I11 ownership for the application/database seam without treating this checkpoint as its
completion or authorization for a permanent collector/backend. No exporter, Collector, Grafana stack,
new Rust telemetry crate, host service or production telemetry configuration was deployed.

### Verification and Outcome

**Tested:** the three explicit probe commands run in the locked `capability-inquiry` environment;
their final closeout results, scoped static checks and existing endpoint guard controls are recorded
in the coordinator's pause checkpoint. Failure baseline is zero. Probe artifacts are under
`build/capabilities-52o/` and the original `build/plan52o-*-probe-20261024-*` directories. They retain
their original fixture and process boundaries; no actual-Codex, Rust scientific, persistent-backend
restart, full task-conformance or whole-Plan52 qualification is claimed.

**Mistakes corrected:** reader responses required the real typed `result.payload` shape; a raw
payload assumption failed and was repaired. The initial optional-only environment lacked runtime
codecs required by canonical source bindings; explicit `uv sync --no-install-project` supplies the
same locked runtime dependencies without building the extension. Companion serializer commentary
was checked against actual MCP2.3 behavior and corrected as above. Ruff initially excluded documentation
probe paths; the manual drivers now live in `scripts/tests/`, participate in normal quality scope,
and retain their explicit execution commands rather than automatic test discovery.

**Deliberate deviations and handoff:** telemetry was assessed only far enough to establish the
current baseline and bounded SDK behavior after the maintainer's question. The separate thorough
review/deployment remains unstarted. Actual-Codex acceptance was not started before the pause. All
owned source changes are integrated directly on main; no owned worktree requires merge or removal.
Preserve unrelated concurrent review edits. Resume from the remaining OPT01/OPT02 obligations above
after the maintainer selects the next work; do not implicitly enable these experimental paths.
