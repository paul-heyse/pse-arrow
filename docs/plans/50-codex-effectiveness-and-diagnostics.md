---
title: "Plan 50: Codex effectiveness and diagnostic tooling"
status: in-progress
date: 2026-10-22
adrs: []
review_sources: [docs/functional_reviews/codex-effectiveness-and-diagnostics-2026-10-22.md]
scenario_sources: [docs/functional_reviews/codex-effectiveness-and-diagnostics-2026-10-22.md]
doc_role: plan
doc_owner: docs/plans/50-codex-effectiveness-and-diagnostics.md
doc_topics: [current-work, design, diagnostics, environment, libraries, validation]
---

# Plan 50: Codex effectiveness and diagnostic tooling

## Purpose and authoring basis

Give Codex a dependable local development loop and substantially better ways to understand,
change, verify and diagnose this system. Cover all twenty opportunities in the
[functional review](../functional_reviews/codex-effectiveness-and-diagnostics-2026-10-22.md),
including investigations and meaningful alternatives. Useful outcomes are complete operator
workflows: a correctly selected session, a source-aware change, an explained wait, a readable
run, a controlled fixture, a diagnosed query and a reproducible failure.

**Interface-checked:** source `bb61ae3f518696d653e06819f9749b76117924f1`, including the review
of source `79a8dbfc9`. Additional working inputs are the supplied untracked
`docs/libraries/surrealdb_tooling_idea.md` and comprehensive
`docs/libraries/surrealdb_tooling.md`. The latter was read comprehensively and followed into
selected primary sources; its material additions are accounted below and in 50d/50e.
These local inputs are orientation, not declarations of this custom build's capabilities.

The maintainer authorized the entire target for implementation on 2026-10-22. Package state
and the checkpoint below distinguish implemented mechanisms from remaining acceptance;
authoring evidence alone establishes neither. Earlier review observations retain their named
conditions. Core 3.4, Efficient Architecture
Heuristics 1.0, Process Simulator 1.5 and the repository binding guide the focused foundation
assessment; this is not a new broad diagnostic audit or an enclosing-system certification.

This coordinator owns opportunity dispositions, package progress, inquiry outcomes, shared
decisions, cross-plan dependencies and completion. Companions own their local designs and
acceptance descriptions; they link this state rather than copy it.

## Operator selections and rule boundaries

- Tailor the result to Codex, its Codex subagents, one operator and this Linux workstation.
  Existing explicit selections of sixteen compiler jobs, unbounded ordinary builds and
  regenerable generated data remain the starting point. Scientific/reference allocations
  retain their own semantics. Add no new global capacity governor or routine approval flow.
- Reuse current owners and installed tools where suitable. Direct CLIs and literal arguments
  remain first-class; MCP is an interaction choice. Do not introduce a universal debugger,
  query catalogue, scheduler, result ledger, graph service or cleanup daemon.
- C05 selects explicit owner release under blueprint §24.1/§20.6. Failed/incomplete results
  remain truthful; unknown external effects, active borrowers and protected references are
  not cleared by an abandonment label. Automatic cleanup retains its existing stronger
  eligibility conditions. Missing-terminal abandonment needs a new membership fence in the
  existing owner, not terminal finalization; verify the selection/composer contract before
  dependent implementation. The old blocked invocation is already retired and is not work here.
- Active diagnostics have separate requests and declared effects. Existing passive readers
  and exporter-only `CaptureSession` do not gain hidden process control or task re-polling.
  Python runtime capture options do not inherently require a new interpreter build.
- The maintainer accepted C05 separate abandonment membership closure on 2026-10-22; proposed ADR-0174 and blueprint §24.1 are amended before dependent implementation. Existing terminal truth and all release protections remain. No broader rule waiver is selected by this plan. Its disposal workflow uses existing explicit-release
  authority; its diagnostic selection preserves ordinary build and observation contracts.
  If an investigation finds that a required behavior needs an actual rule change, present the
  exact change for explicit operator acceptance before fixing that dependent target, following
  create-plan and blueprint §24.4. Do not infer acceptance from the review or this authoring request.
  ADR/design publication follows the existing decision table; accepted ADRs are immutable.

## Foundation assessment and combined target

### Existing foundations and necessary changes

The environment, supervisor, admission, test-resource and diagnostic owners are already
substantial. Improve composition and explanations rather than replacing their semantics.
The source review identifies genuine additions alongside discoverability problems and repairs
already landed in Plan49. Do not repeat the checkout-Python, sixteen-job aggregate, transaction
conflict or charged-future fixes as unwritten packages.

Several source checks materially shape this target:

1. `BASH_ENV` currently survives the machine bridge into nested Just/compiler shells. Select
   one initial activation per top-level command; preserve later explicit workload selections.
2. `activity` already reports unit races truthfully. Admission has a detached read-only owner
   snapshot, but its live retry reason is local to `acquire`. C02 adds producer-owned wait
   observations rather than pretending a persistent wait queue already exists.
3. Product runners already reconcile selection, reports, process/drain and a frozen terminal.
   C03 reads those facts; it does not invent another result composition or infer pass from JUnit.
4. Native MCP, SurrealKit and eleven MX gauges already exist. The SDK statistics observer is
   also integrated in `canonical_transport`: expose/compose existing query facts before adding
   instrumentation. Root-level selection VIEWER credentials satisfy the source-level full-metrics
   gate; OWNER credentials are unnecessary for that observer.
5. Existing capture is CPU/timeline oriented and noncontrolling. GDB/py-spy live attachment,
   scheduler capture and cooperative task dumps have different effects and coverage. E01 gives
   each action an owned request/result using existing subject identities, not new enum variants
   under an incompatible passive promise.
6. DataFusion logical graph output is structured, but current physical-plan rendering is bounded
   text. F02 first extends the engine observation owner with structured physical nodes/ordered
   edges associated with its metric ordinals. It never parses text into authoritative topology.
7. Frozen review context is not necessarily authored text. F03 uses existing source/replay
   reconstruction and admission codecs for executable representations; it never guesses text
   from IPC bytes or treats attribution as a replay recipe.

These changes localize extension at the existing semantic owners and remove repeated manual
interpretation. Snapshot selection, field projection and bounded neighborhoods keep auxiliary
work proportional to the question. Expensive capture, schema rehearsal and numerical rerun
remain explicit operations rather than startup or polling obligations.

### Shared interface and observation meanings

The following are common contracts, not a demand for a new root type or physical service:

| Meaning | Owner and target behavior |
| --- | --- |
| Selected session/store | 50a freezes the explicit state, endpoint and verified service/credential identity for attachment. A changed selection requires deliberate reconnection; it never follows another store silently. |
| Run handle | 50c uses the existing invocation identity/declaration and authentic frozen terminal. Paths and digest references resolve through current resource readers, not a new index of outcomes. |
| Process/executor subject | Existing `ProcessLocator`, start identity, `OperationId`, native-thread/clock facts and actual executor handle identify a diagnostic target. An operation ID alone is not permission or sufficient information to attach. |
| Observation versus action | No-write snapshots never reconcile, resume, cancel or execute science and do not protect report payloads. Protected report reads use explicit temporary borrow metadata. Action-specific requests state effects, scope and output; failed collection does not qualify a target outcome. |
| Standalone diagnostic | E01 reads its action document directly, without requiring a test invocation. C03 follows associated diagnostic artifacts only for genuine run links; no invented invocation identity. |
| Completeness | Preserve selected, reported, unreported, partial, stale, unavailable and ambiguous correspondence separately. A fully reported failing test suite can have a complete terminal. |
| Time and attribution | Inclusive spans, sampled CPU, scheduling classes, shared metrics and active probes retain their populations. Neither wall-minus-CPU nor shared counter deltas prove an operation's wait cause. |
| Source and executable context | Links identify declaration/generator/edit owners. New experiments/reproduction retain their own identity and the original result; unsupported source/provider context refuses explicitly. |
| Command output | Existing command declarations supply grammar/defaults; semantic owners supply interpretations. Human/JSON projections expose the same facts with selective detail and exact raw-artifact references. |

New Rust/Python product documents enter Plan49's existing compiled catalogue and generators
when they cross that boundary. Host-only command observations stay with their existing Python
owners; do not force every operational result through a scientific product schema.

## Series and cross-plan ownership

| Companion | Responsibility |
| --- | --- |
| [50a](50a-session-and-environment-integration.md) | Environment activation, selected-store/Codex attachment and actual capability readiness |
| [50b](50b-change-scoping-and-development-feedback.md) | Rust/Python navigation, task context, command output and build feedback |
| [50c](50c-execution-inspection-and-lifecycle.md) | Wait/run evidence, reference composition and explicit generated-state disposition |
| [50d](50d-database-diagnostics-and-query-development.md) | Actual query, MX, language/schema tooling and database log/trace workflows |
| [50e](50e-native-and-python-diagnostics.md) | Mixed stacks/crashes, OS scheduling, async task inspection and allocation diagnosis |
| [50f](50f-preparation-explanations-and-reproducers.md) | Edit/preparation, mathematical/relational comparison and supported executable reproducers |
| [50g](50g-qualification-and-publication.md) | Representative composed acceptance, scope-end checks and enduring publication |

Plan49g TOOL01 supplies catalogue projection; BND05 is downstream scientific-boundary adoption, not a general prerequisite; TOOL05 owns its product/tooling separation;
TOOL08 supplies canonical blocker/source/consumer explanations; TOOL04/RUN01 retains its
maintenance/recovery-disposition work. Consume exact delivered slices rather than declaring
all of Plan49g a prerequisite. In particular, O19's canonical graph consumer can wait for TOOL08
without blocking its math/engine comparison. Plan49f keeps independent Python/scientific scope
and 49h its own qualification. Neither is absorbed or marked complete here.

Plan32 owns shared document/artifact lifecycle, including its reopened representation question.
This series uses the current metadata/native bindings without resolving that larger question.
Plan40e retains its assembled diagnostic campaign; share relevant actual evidence and avoid
duplicated tests, but Plan50 acceptance neither restarts that excluded campaign nor closes it.
The current-work index links these owners. Plan29–31 and completed diagnostic plans are foundations,
not a backlog to reopen. No changes to their historical outcomes are required.

## Package state

The entire target is authorized for implementation. Current delivery and acceptance state is recorded below; package acceptance includes migrating actual consumers and
deleting replaced paths/tests when the replacement is verified. Routine naming and file layout
remain implementation judgment within each companion's contracts.

| Package | Status | Owner | Required capability / completion |
| --- | --- | --- | --- |
| A01 | in-progress | [50a](50a-session-and-environment-integration.md#A01) | One-time machine activation preserves nested workload/compiler/native selections |
| A02 | in-progress | [50a](50a-session-and-environment-integration.md#A02) | Explicit selected-store and Rust navigation session attachment; literal Codex overrides |
| A03 | in-progress | [50a](50a-session-and-environment-integration.md#A03) | Configured/enabled/attached/readiness states and current operating guidance |
| B01 | in-progress | [50b](50b-change-scoping-and-development-feedback.md#B01) | Rust and Python semantic navigation plus explicit cross-boundary impact leads |
| B02 | in-progress | [50b](50b-change-scoping-and-development-feedback.md#B02) | Task-shaped command/context retrieval with selective output and native owners |
| B03 | in-progress | [50b](50b-change-scoping-and-development-feedback.md#B03) | Actual compile/link/cache/lock/admission feedback without inferred lock ownership |
| B04 | decision selected | [50b](50b-change-scoping-and-development-feedback.md#B04) | Bounded semantic-index/agent-knowledge decision with a concrete retrieval consumer |
| C01 | in-progress | [50c](50c-execution-inspection-and-lifecycle.md#C01) | Shared invocation/subject/artifact projection on existing authorities |
| C02 | in-progress | [50c](50c-execution-inspection-and-lifecycle.md#C02) | Read-only wait/owner/park explanations with producer-owned transient wait facts |
| C03 | in-progress | [50c](50c-execution-inspection-and-lifecycle.md#C03) | Product-test reader over current selection/report/terminal/drain/resource evidence |
| C04 | in-progress | [50c](50c-execution-inspection-and-lifecycle.md#C04) | One-owner reference workflow returns separate execution and teardown outcomes |
| C05 | in-progress | [50c](50c-execution-inspection-and-lifecycle.md#C05) | Explicit abandonment/release under existing drain/reference/unknown-effect rules |
| D01 | in-progress | [50d](50d-database-diagnostics-and-query-development.md#D01) | Production-owned readiness query/schema/index/plan inspection |
| D02 | in-progress | [50d](50d-database-diagnostics-and-query-development.md#D02) | Authenticated identity-bound MX/server snapshots and honest comparisons |
| D03 | in-progress | [50d](50d-database-diagnostics-and-query-development.md#D03) | Source-mapped exact-parser and compatible advisory LSP feedback |
| D04 | in-progress | [50d](50d-database-diagnostics-and-query-development.md#D04) | Disposable query/index/schema rehearsal through existing SurrealKit owners |
| D05 | in-progress | [50d](50d-database-diagnostics-and-query-development.md#D05) | Structured scoped logs and existing query-statistic retrieval, explicit restart effects |
| D06 | in-progress | [50d](50d-database-diagnostics-and-query-development.md#D06) | Close optional telemetry/MCP/graph/engine-tool decisions with scoped outcomes |
| E01 | in-progress | [50e](50e-native-and-python-diagnostics.md#E01) | Action-specific active-diagnostic subjects/results and direct tool capability checks |
| E02 | in-progress | [50e](50e-native-and-python-diagnostics.md#E02) | Mixed-language stacks and actual-host crash/core inspection |
| E03 | in-progress | [50e](50e-native-and-python-diagnostics.md#E03) | Scheduling capture plus owner/thread/clock interpretation |
| E04 | in-progress | [50e](50e-native-and-python-diagnostics.md#E04) | Cooperative actual-executor task dump and independently observable noncompletion |
| E05 | in-progress | [50e](50e-native-and-python-diagnostics.md#E05) | Selected Heaptrack/native allocation workflow and Memray coverage decision |
| E06 | in-progress | [50e](50e-native-and-python-diagnostics.md#E06) | Concrete rr/deeper-counter feasibility and adopt/no-adopt disposition |
| F01 | in-progress | [50f](50f-preparation-explanations-and-reproducers.md#F01) | Source/edit/query/product comparison with no invented invalidation cause |
| F02 | in-progress | [50f](50f-preparation-explanations-and-reproducers.md#F02) | Structured physical observation prerequisite and math/engine before-after explanations |
| F03 | in-progress | [50f](50f-preparation-explanations-and-reproducers.md#F03) | Supported frozen-context extraction through owned reconstruction/admission codecs |
| F04 | in-progress | [50f](50f-preparation-explanations-and-reproducers.md#F04) | Small local diagnostic/reproducer controls, coordinated with existing acceptance |
| G01 | in-progress | [50g](50g-qualification-and-publication.md#G01) | Final actual consumer/contract/decision selection before composed qualification |
| G02 | in-progress | [50g](50g-qualification-and-publication.md#G02) | Six representative composed workflows and negative cases |
| G03 | in-progress | [50g](50g-qualification-and-publication.md#G03) | Relevant zero-target scope-end static/manual checks, failures repaired |
| G04 | in-progress | [50g](50g-qualification-and-publication.md#G04) | Enduring documentation, source dispositions and bounded final Outcome |

## Opportunity dispositions

References below identify the source review's opportunities, not diagnostic findings. Scheduled
means included in this proposed target; it grants no execution authorization or Tested claim.

| Opportunity | Review scenarios | Disposition | Work owner | Required evidence or question |
| --- | --- | --- | --- | --- |
| O01 | Fresh session/change | scheduled | A01/A03 | Nested actual tool environments and checkout-switch/failure behavior |
| O02 | Session/query | scheduled | A02/A03/B01 | Real selected-store and semantic navigation interactions, stale selection refusal |
| O03 | Rust/Python change | scheduled | B01 | Actual generator/facade/PyO3/test consumers and unresolved dynamic reach |
| O04 | Change/frozen failure | scheduled | B02/C03 | Useful selective context/commands/artifacts, no duplicate catalogue |
| O05 | Apparently idle test | scheduled | B03 | Actual compilation/cache/lock/admission stages with coverage limits |
| O06 | Idle/cancellation | scheduled | C01/C02 | Read-only observation and real producer wait association |
| O07 | Test/frozen failure | scheduled | C03 | Correct complete-failing/incomplete/interrupted/unreported distinctions |
| O08 | Reference execution | scheduled | C04 | Setup/runtime ownership, literal command, interruption and resume intent |
| O09 | Retained resources | scheduled | C05 | Exact explicit disposition, authenticated drain, references and protected unknowns |
| O10 | Query regression | scheduled | D01/D04 | Actual production query and representative disposable alternative |
| O11 | Reader/version retention | scheduled | D02 | Root VIEWER metrics, normal absence, restarted-identity refusal |
| O12 | Query/schema edit | scheduled | D03/D04 | Exact parser authority, LSP limits, source mappings, rollout parity |
| O13 | Cross-process failure | scheduled | D05/D06 | Scoped logs/statistics and justified optional trace outcome |
| O14 | Mixed stack/crash | scheduled | E01/E02 | Actual interpreter/native coverage and host core route |
| O15 | Low-CPU wall time | scheduled | E03/E06 | Scheduling classes versus evidence-supported wait causes |
| O16 | Pending async work | scheduled | E04 | Actual executor cooperation, independent observer and incomplete dump |
| O17 | Allocation churn | scheduled | E05 | Actual allocator/process population and symbol interpretation |
| O18 | Edit/recompute | scheduled | F01 | Semantic labels and source/value/structural comparison |
| O19 | Generated/relational work | scheduled | F02; 49g TOOL08 for canonical graph | Structured snapshot-local graph and unambiguous/ambiguous comparison |
| O20 | Failure reproduction | scheduled | F03/F04 | Supported executable representation and original failure predicate |

## Inquiry outcomes

Each open inquiry is bounded design/implementation work in the named package. Completion requires
the documented decision and its required selected integration, not adoption of every candidate.
Do not leave a failed feasibility check as an unowned promise to investigate later.

| Inquiry | State | Owner | Question and evidence | Adoption / no-adoption consequence |
| --- | --- | --- | --- | --- |
| I01 | adopted; integration qualification open | B01 | Pinned mcpls 0.8.0 with checkout Pyrefly and pinned rust-analyzer; actual client resolved Python facade and Rust definition/reference/implementation requests | Replaces the selected session’s Rust-only bridge and supplies Python navigation without another type checker. Generated-boundary composed journey remains G02 work. |
| I02 | no-adopt 0.6.3 | D03 | Exact LSP release 0.6.3 rejects actual engine-valid production and generated schema forms; a readiness-only workspace can appear falsely clean | Retain exact selected-engine parser and source/binding feedback. Revisit when the LSP handles this pinned parser’s production/schema forms; no second language authority. |
| I03 | open | E02 | CPython perf/JIT and py-spy coverage on one real Python/Rust/native journey | Choose useful complementary captures; keep unsupported thread/symbol populations explicit |
| I04 | adopted | E03 | Actual scheduler capture and selected PID/thread timehist succeeded with CAP_PERFMON on a private perf ELF and read-only selected tracefs metadata | Retain exact process/thread/clock scope and host-wide raw population. No global paranoid/kptr relaxation or writable tracing controls; reboot persistence remains untested. |
| I05 | open | E04 | Actual executor cooperation, issued-request cancellation/lifetime and independent observation of a wedged runtime | Adopt bounded task dumps with target-side ownership through settlement; collector exit is not dump drain. Console remains optional for a demonstrated question |
| I06 | adopted | E05 | Actual Heaptrack native allocations and pinned Memray 1.20.0 Python/new-thread/native captures plus decoding succeeded | Memray is an explicit separate dependency group/environment; native-only threads can lack Python frames. Disable ambient decoder debuginfod by default; opt-in retains its deadline and partial outcome. No exact operation-private peak claim. |
| I07 | open | F03 | Which complete frozen representations reconstruct executable requests through existing codecs? | Deliver named supported extraction; missing executable/provider/source premises refuse, never guessed conversion |
| I08 | adopted | D06 | Actual finite operator capture exported server executor to Surrealism parent/child spans and restored baseline; application Rust trace context was absent | Explicit trace-capture owns temporary sink and admitted service restart/restore. Preserve the application-side correlation gap; no permanent telemetry stack. |
| I09 | no additional index | B04 | Actual maintained mcpls symbol/reference queries plus bounded catalogue/source leads resolve the current TaskDumpTicket/Runtime owner and consumer question | Use native LSP and derived context; an extra indexed graph would duplicate current source reach without resolving dynamic PyO3 correspondence. Revisit a repeated diagnosed retrieval gap, not a periodic index campaign. |
| I10 | adopted | E06 | Actual Python→ctypes→native SIGSEGV recording, replay and GDB reverse-step succeeded with the pinned rr 5.10.0 portable runtime | Explicit rr-record/rr-replay actions retain the full bundle and trace identities. This is perturbing diagnostic execution, never parallel timing qualification. |
| I11 | no additional deployment | D06 | Current canonical graph consumer uses actual production SQL and native plans; no diagnosed missing GQL behavior, engine defect or server-task question requires another deployment | Keep graph/Console/upstream harness as explicit future choices on a concrete unanswered question; no engine switch or whole upstream campaign. |
| I12 | no-adopt | D06 | Actual native MCP database reads and structured owner commands cover the current operator consumers | No extra operations MCP or permanent telemetry platform. Revisit a repeated structured interaction gap that existing commands cannot serve. |

### Additional overview leads and alternatives

The comprehensive tooling overview broadens awareness without changing the selected engine.
Its §§6/11 lead to existing `Query::observe_statistics`/`CanonicalStatistics` facts, consumed in
D05 rather than reinstrumented. §§5/14 lead to existing SurrealKit sync/replay parity and proposed
actor-specific permission tests in D04, requiring narrow wrapper-admission changes. §12 module introspection reuses the exact bucket/export
verification owner. §9 graph/index plans inform D01 and the existing TOOL08 consumer.

Server Tokio Console needs a server build feature, unlike the overview's environment-only
example; app task dumps belong to the actual Python/Rust executor, a different target. GQL
availability must distinguish server endpoint, transitively compiled core and enabled MCP tool.
The selected server omits the server `gql` feature, while MCP pulls core GQL; neither blanket
"available" nor "absent" is accurate. I11 settles only a concrete useful question.

The upstream language-test harness has a remote mode, useful for a minimized engine reproducer
on an owned disposable service, not the canonical store or an automatic whole-corpus run.
`crud-bench`, backend diagnostics and alternative storage tools are reference mechanisms for a
diagnosed need, not a new benchmark requirement or storage-selection package.

Formatter adoption remains conditional on provenance and engine-accepted formatting behavior;
it is not required to finish D03's parser feedback. Cloud control, Kubernetes, Surreal Sync,
PostgreSQL wire and TS type generation are not selected: no consumer was established for this
local Rust/Python target. Reopen at a real deployment, ingestion or client requirement. Permanent
telemetry and a new operations MCP require I12's positive outcome; the core target does not depend
on them. These are explicit dispositions of alternatives, not a second deferred-work register.

## Dependency-ordered execution route

1. Resolve the small shared subject/run projection in C01 and action-specific boundaries in
   E01, and verify exact external prerequisite slices. These are narrow contracts with actual
   first consumers, not barriers requiring whole companions to finish.
2. A01–A03 and B01–B03 improve the ordinary loop; C02/C03 use C01. D02 consumes selected-service
   validation from A02; D01/D03/D04 compose actual query construction and fixture lifetime.
3. C04/C05 use existing supervisor/resource rules plus C01/C03; D05 logging activation uses the
   same owned quiesce/drain/start sequence, without silently widening allocation reconfigure.
4. E02–E06 can proceed independently after E01's relevant slice. F01/F02 consume existing
   observations in parallel. F03 first proves its reconstruction route; F04 supplies small
   diagnostic controls as those features land.
5. G01 selects affected real consumers; G02 composes representative journeys after functional
   delivery. G03 runs scope-end checks once, fixing failures. G04 publishes enduring contracts
   and exact qualification boundaries and reconciles this coordinator.

Shared editing surfaces do not imply runtime dependencies: the justfile, diagnostics command,
catalogue/generators, supervisor and source observation owners need one writer when executing
concurrently. Use ready parallel work on this workstation; independent check work can share the
checkout, artifact-producing builds may need genuine isolation. Do not share target directories
or run known incompatible Clippy artifact graphs concurrently.

## Verification, checkpoint and completion

**Proposed:** each package uses focused compile checks and targeted controls, with explicit
force-validation for actual Arrow-consuming correctness closures. New mechanisms get tests
with the change; delete replaced mechanisms/callers/tests once verified. No per-package hygiene,
whole-scientific campaign, mandatory benchmark or per-command plan receipt is introduced.

**Proposed assembled acceptance:** six review journeys plus consequential refusal/interruption
cases in 50g. Static/manual checks occur at scope end, against zero findings, with their actual
commands and modes reported. Documentation authoring checks qualify only documents. Any speed,
memory or developer-effort improvement remains a hypothesis until a relevant named measurement;
useful delivered diagnostics do not require proving a universal performance gain.

Completion requires all O01–O20 routes, selected consumer migrations and replacement deletions;
all inquiries have explicit outcomes; required composed acceptance and scope-end checks are
recorded with exclusions; enduring operation/contract meaning is published through its owner.
An unavailable optional tool is an honest inquiry outcome, not permission to mark a required
workflow delivered without its explicitly supported alternative or narrowed scope.

Current checkpoint: the maintainer authorized all Plan50 scope on 2026-10-22. Discrete
commits deliver selected session attachment, native metrics, restricted query rehearsal,
run/wait/build inspection, reference composition and missing-terminal abandonment. Maintained
Rust/Pyrefly navigation now has actual client evidence. The explicit bounded retrospective route
projects checkout/time-scoped diagnostic leads without copying raw prompts or environments.

Database query/graph/log/trace consumers, structured preparation comparisons, supported executable
reproduction and active host/executor diagnostics are being integrated and tested. The canonical
graph slice consumes Plan49g TOOL08 without closing the remainder of that plan. Root VIEWER and
disposable rehearsal controls retain their own conditions; they do not qualify all database work.
The user’s cargo clean interrupted two active checks; those missing-output failures require reruns,
not source repairs. Cold compile work is underway. Ordinary and diagnostic Python builds remain
separate; the coherent ordinary editable extension must be restored before mixed-language journeys.
Scheduler trace collection requires actual host permission; CPU evidence does not satisfy it.
Composed acceptance, final generation, scope-end hygiene/manual checks and enduring publication
remain open. No enclosing completion or quantitative effectiveness claim is made.
