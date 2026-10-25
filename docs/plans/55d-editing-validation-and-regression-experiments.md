---
title: "Plan 55d: Editing, validation and regression experiments"
status: complete
date: 2026-10-25
adrs: []
review_sources: [docs/functional_reviews/mcp-development-effectiveness-2026-10-25.md]
doc_role: plan
doc_owner: docs/plans/55-mcp-development-effectiveness.md
doc_topics: [current-work, design, validation, environment]
---

# Plan 55d: Editing, validation and regression experiments

## Target and foundation

**Proposed:** help Codex choose and understand appropriate validation for a change, while
assessing selected further editing/experiment adaptations on actual consumer value. Owns O05
and the O03/O11/O12 inquiries. The [coordinator](55-mcp-development-effectiveness.md) owns
dispositions; positive optional delivery is distinct from completing its inquiry.

**Interface-checked:** `workspace_integration_view` currently emits nearest-package seeds and
suggested routes; [affected.py](../../scripts/affected.py) already owns Cargo reverse closure,
workspace configuration widening, scientific-input consumers and composed commands. Assessment
selection/read/run and Rust test description/run are callable. Do not duplicate those decisions.

[Specialists](../../scripts/capabilities/specialists.py) retains edit previews/reconciliation
state in process-local maps and clears retention at close. Native invocation references resolve
the original execution registry independently. Read-only annotations do not prove that a read is
stateless: a preview-detail read can require endpoint-owned state.
[semantic_navigation.py](../../scripts/semantic_navigation.py),
[structural_migrations.py](../../scripts/structural_migrations.py) and
[semantic_edits.py](../../scripts/semantic_edits.py) already support complete saved previews,
source/protection checks, pre-mutation reconciliation destinations and reconciliation.
[regression_experiments.py](../../scripts/regression_experiments.py) has specific predicate,
comparison, copied-source and bisect behavior. It is not an arbitrary-command bisector.

## Selected validation design

Compose the existing affected owner's structured selection into the current integration/task
view or a narrow typed selected read where needed. Report why each selected package, mode and
population applies, explicit exclusions and unresolved feature/dynamic/cross-language consumers.
A target name and a nearest-directory match are not coverage. User-provided explicit targets
remain explicit selection, not inferred completeness. B02/B03 add attributable relationships
without taking ownership of affected selection.

Use original evidence to explain prior comparable preparation/execution observations only when
source, features, target/mode, native environment and test population make them applicable.
State missing comparability; do not invent an estimator or run a benchmark to populate a help
answer. Historical duration/telemetry does not select away correctness or scientific coverage.
Preserve force-validation, original workload/native pool contracts and the environment's
high-throughput ordinary build configuration. More context must not spawn one test/build per
conceptual relationship.

## Package state

| Package | Status | Owner | Completion |
|---|---|---|---|
| D01 | complete | 55d | Existing affected selection is available with exact reasons/population/unknowns |
| D02 | complete | 55d | Applicable original evidence explains validation choices and preparation versus execution |
| D03 | complete | 55d | Foreground connection selected and actual five-request session accepted |
| D04 | complete | 55d | JSONL foreground session implemented; no endpoint-death durability promise |
| D05 | complete | 55d | Four genuine pipeline-review candidates establish finite migration access value; decision below |
| D06 | complete | 55d | Original migration retained through common preview/apply/reconcile owner |
| D07 | complete | 55d | Fixed-predicate direct owner retained; optional adaptation deferred with trigger below |
| D08 | deferred | 55d | Revisit on an assigned fixed-predicate regression whose native route cannot provide the needed agent handoff |

<a id="D01"></a>
### D01 — Attributed affected-validation preview

Call the affected owner once for the selected change population and project its original result
through strict capability contracts. If a useful structured native result is absent, add it at
that owner rather than parsing human command output or reimplementing dependency closure in
FastMCP. Preserve selected source/configuration, force-validation mode, reasons for widening,
scientific consumers, exclusions and unknowns. A04 uses this selection for actual-request
explanation. Starting a compiler, test discovery campaign or test execution is a separate action.

**Acceptance:** a Rust edit, registry declaration change and Python/native boundary change yield
independently expected inclusions/exclusions and the existing owner's composed route. An unrelated
same-directory file does not acquire fabricated coverage. Broad configuration edits widen for
the right declared reason; unresolved dynamic consumers remain explicit. Actual selected run
results stay in the assessment/native owner.

<a id="D02"></a>
### D02 — Explain evidence applicability and validation work

Consume D01's selection and existing assessment/evidence readers. Attach exact original references
and applicability observations, not copied mutable test status. Separate preparation/build cost,
subject duration, selected/executed/failed population and partial evidence. B02 owns any new
producer relationship required to connect an executed observation; do not add a second ledger.
For a new population with no comparable history, return that limitation and useful original
selection rather than reducing assurance to the fastest historical cases.

**Acceptance:** cold preparation and a slow subject are explained differently. Changed features,
source/native identity or population prevent a prior pass/duration becoming current acceptance.
An unexecuted selected test provides no success claim. The agent chooses a valid targeted route
and reads its original outcome. No automatic test pruning or performance campaign is delivered.

<a id="D03"></a>
### D03 — Decide a useful CLI edit lifetime

Use one actual shell-using Codex preview→detail→explicit apply→reconcile workflow. Compare:
(1) a foreground process-scoped Client/session the agent can drive; (2) explicit complete saved-
preview export/adoption through the native owner; (3) retaining CLI read/recovery only and using
the existing MCP/native edit routes. Prefer the foreground option if it solves the actual
interaction without detach/broker obligations. Saved-preview adoption is a distinct input,
not resurrection of an expired opaque handle.

Assess connection loss, source changes, retention/release and interrupted apply before adopting.
Separate request/client loss while the endpoint survives from endpoint-process death, which
destroys its retained preview/reconciliation maps. A foreground session alone supplies no
crash-resumable state. Select an original native complete-preview/receipt route before effects
if that stronger recovery is needed, or explicitly retain unknown/unsupported post-death recovery.
Reuse complete immutable bytes/source basis, protected paths and original reconciliation. A
digest grants no authorization; absent endpoint state does not prove an effect never happened.
Do not make handles durable, transfer authorization or erase unresolved prior mutation merely
to enable CLI editing. A03's fresh-client recovery remains useful even if this inquiry defers.

**Decision output:** select one supported interaction/loss boundary or defer with the exact unmet
consumer/revisit condition. Name D04's bounded target and exclusions if adopted; route any actual
changed lifetime/authorization rule for maintainer confirmation before dependent implementation.
Evaluate the real agent interaction, not only a library demo. No new daemon is a presumed outcome.

<a id="D04"></a>
### D04 — Deliver only the selected CLI edit workflow

Requires D03's positive decision and A02's faithful CLI/runtime. Reuse original preview/apply/
reconcile owners and explicit lifetime declarations. Client/request loss while the endpoint
survives uses its original retained state. Endpoint-process death only has a recoverable preview
if D03 explicitly selected native-owned complete preview/receipt custody before effects;
otherwise report the uncertain effect and unsupported recovery boundary without replay. Do not
silently add durable adoption to the foreground-session branch. If artifact adoption is chosen,
bind full preview contents, checkout/protection/source basis and retention, and preserve earlier
uncertain apply status. Existing native saved-preview commands remain first-class alternatives.

**Acceptance if adopted:** actual preview/detail/apply/reconcile; source drift refusal; altered
saved input refusal where applicable; interrupted/lost-ACK mutation recovered through the original
owner without replay or false rollback. A limitation/deferral alone closes D03, not an adopted D04.

<a id="D05"></a>
### D05 — Assess one structural migration

Select an actual recurring migration supported by existing structural/codemod owners whose
declaration, consumer and generator obligations can be stated. Compare canonical preview access
with direct native use/manual scoped edits. Do not expose all scripts, invent an unconstrained
rewrite language or relax generated/protected paths. Language symbol semantics remain distinct
from semantic-similarity search.

**Decision output:** adopt a named migration with finite inputs, exact preview/reconciliation
contract and real consumer benefit, or defer with the absent value/observable trigger. One
representative change and revealing stale-source/protection control suffice for the inquiry;
large repository rewrites are not inquiry fixtures.

<a id="D06"></a>
### D06 — Deliver the selected structural operation

Requires positive D05 and the actual edit lifetime it consumes; MCP-only retained preview need
not wait for optional CLI editing. Register the one selected operation, preserve full owned
preview/protection/source checks, and migrate its actual consumers/references. Generated changes
go through their generator. Delete replaced adaptation after callers move; retain independent
direct native functionality.

**Acceptance if adopted:** the real migration changes only the intended authoritative population,
regeneration yields expected consumers, changed source refuses, and interruption is reconciled.
The existing operation's oracle/protection is exercised, not mirrored in another MCP rewriter.

<a id="D07"></a>
### D07 — Assess bounded comparison/bisect access

Choose an existing deterministic predicate and regression consumer, then compare current direct
experiment use with a canonical typed adaptation. Inspect source/configuration variants,
population selection, repetitions, isolation, environment differences, restoration, resource
ownership and original results. Current pure-Rust archive support does not establish native
replay support. No arbitrary shell/predicate execution or implicit broad feature campaign.

**Decision output:** adopt the smallest named comparison/bisect workflow with explicit predicate
meaning and supported source/native combinations, or defer with the concrete unmet consumer.
Use a representative bounded regression; a benchmark or exhaustive matrix is not required to
demonstrate orchestration. Numerical/speed claims require separate applicable measurements.

<a id="D08"></a>
### D08 — Deliver the selected experiment adaptation

Requires positive D07. Dispatch through the original experiment/workload owners, preserving
source isolation and original per-subject reference, terminal/drain and cleanup state. Controller
failure must not erase subjects' outcomes. Cancellation retains authenticated resource ownership
and an exact next original reader; output disappearance never authorizes duplicate work.

**Acceptance if adopted:** the actual agent compares/bisects the selected deterministic case,
independently checks the predicate and follows original evidence. Unsupported native/source
combinations refuse; interrupted execution settles/restores or reports the remaining obligation.
No extra execution ledger, scheduler or privileged free-form command runner is introduced.

## Inquiry decisions and qualification

### Inquiry decisions — 2026-10-25

D03 selects a foreground JSONL client retaining one endpoint for explicit requests. This uses
the shell agent's existing interactive process support, without a daemon, detached session or
saved-handle resurrection. D04 has proved the actual preview/detail/apply/reconcile/release interaction;
its evidence is recorded in the Outcome below. Endpoint death loses its
retained previews; crash-resumable adoption is not selected.

**Tested, bounded:** D05 ran the existing `json_source_bytes` transformation in memory against
current `scripts/pipeline_review.py`: four AST-grep candidates resolved to four qualified
stdlib JSON edits, with no source publication. That real multi-call consumer plus the shared
retained-preview workflow justifies one finite canonical preview operation. D06 reuses the
original transformer and edit owners; it does not apply those production edits automatically.

**Interface-checked:** D07 inspected the fixed `authoring-load-package` predicate, original
classification/isolation/bisect/receipt owners and their existing native `just experiment`
route. No presently assigned regression needs a new canonical adapter, and that fixed predicate
does not serve arbitrary future failures. Retain that direct route and defer D08 until an actual
assigned fixed-predicate investigation demonstrates missing agent handoff/control value. Paired
performance comparison likewise has no selected measurement consumer. No new bisect or subjects
were executed for this decision; existing retained tests are not fresh qualification.

## Verification and Outcome — 2026-10-25

**Implemented:** D01/D02 consume original affected selection and source/mode/profile/feature-bound
evidence applicability, with exact reasons, unknowns and preparation/execution distinctions.
Preview is passive and does not compile or discover/run tests. D03/D04 deliver a foreground JSONL
client retaining endpoint custody. D05/D06 expose the existing finite JSON source migration
through shared preview/detail/apply/reconcile/release owners; no alternate rewriter or auto-apply
was introduced. The D07 inquiry is complete and D08 remains explicitly deferred under its table
trigger; no regression/bisect campaign or performance consumer was selected.

**Tested:** affected/evidence/structural controls, including independently expected Rust,
registry and Python/native selections, pass in the full 2,277-case setup population. A real
five-request CLI session completed preview/detail/apply/reconcile/release, with independently
checked source bytes and normal EOF cleanup. Source-change, omitted/default and unavailable
ownership controls remain explicit. Native request explanation followed the same original
compilation receiver and completed successfully. [55f](55f-qualification-and-publication.md#verification-and-outcome--2026-10-25)
owns exact conditions and evidence; overlapping focused populations are not summed.

**Deliberate limit:** endpoint death loses preview handles; no daemon, saved-handle adoption or
crash-resumable edit contract was selected. Direct native experiment commands remain available.
Original assessment references do not prune required validation or turn historical passes into
current evidence. No performance/productivity improvement was measured.
