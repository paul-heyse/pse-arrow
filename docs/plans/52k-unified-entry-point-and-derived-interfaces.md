---
title: "Plan 52k: Unified entry point and derived interfaces"
status: complete
date: 2026-10-24
adrs: []
review_sources: [docs/functional_reviews/unified-fastmcp-capability-interface-2026-10-24.md]
doc_role: plan
doc_owner: docs/plans/52-agent-effectiveness-and-engineering-workflows.md
doc_topics: [current-work, environment, design, diagnostics, documentation]
---

# Plan 52k: Unified entry point and derived interfaces

## Scheduled follow-up — 2026-10-24

The maintainer authorized MCP06 after the current-checkout MCP assessment. MCP01–MCP05 and their
completed Outcomes remain the baseline; the [coordinator amendment](52-agent-effectiveness-and-engineering-workflows.md#execution-freshness-amendment--2026-10-24)
owns the added scope and progress. Plan authoring does not establish implementation or qualification.

## Subsequent usability extension

Completed MCP delivery and Q01–Q06 remain at their recorded Outcome. New guidance/dependency
availability work belongs to [52m](52m-progressive-guidance-and-operation-availability.md),
lifecycle extensions to [52n](52n-readiness-progress-and-edit-lifecycles.md), and conditional
CodeMode/task inquiries to [52o](52o-conditional-composition-and-protocol-tasks.md). Inquiry is not
adoption or a reopening of MCP acceptance; the separate I14 shutdown inquiry retains its coordinator
owner.

## Execution checkpoint — 2026-10-24

The maintainer authorized execution and then selected direct work on main because Plan53
implementation is not currently concurrent. No new worktree was created. The existing
single-entry-point and native ownership decisions remain in force. MCP01 is delivered;
the remaining MCP02–MCP05 consumers and their focused qualification are now delivered.
The [Outcome](#verification-and-outcome--2026-10-24) records acceptance and limits;
the coordinator alone owns current package and inquiry state.

The maintainer selected deferral of the conditional Cyclopts CLI. Required generated reference
material remains in scope. Revisit CLI delivery when a concrete terminal/non-MCP consumer
needs structured access that the existing native commands and canonical MCP route do not
adequately supply; fidelity and fresh-selection acceptance then precede adoption.

## Prerequisite delivery — 2026-10-24

The [52i Outcome](52i-canonical-capabilities-and-progressive-reveal.md#verification-and-outcome--2026-10-24)
records MCP01's fixed source-only STDIO launch and MCP02's actual Codex browse/describe/select/
exact-read/reset journey. This host retained its initial tool inventory; the selected same-endpoint
read invoker delegates canonical FastMCP dispatch. These are delivered prerequisites, not full
specialist migration or resource/prompt/effectful-tool qualification at that earlier boundary.
The [52j execution](52j-native-contracts-and-tool-adaptation.md) now composes native/host actions
and lazy specialists through the same endpoint, including exact action invocation for hosts with
a fixed initial inventory. Its focused acceptance supplies these prerequisite slices; MCP02's
broader actual Codex presentation, MCP03's remaining coherence work, MCP04's derivative-consumer
decision and MCP05's observation work retain their owners. MCP01's lazy owner slice is qualified
by 52j. MCP02–MCP05 below retain their remaining presentation, coherence, derived-consumer and observation obligations;
the numeric coordinator alone owns their progress state.

## Target and responsibility

**Proposed:** one canonical FastMCP entry point serves agent-facing capabilities through the
exact catalogue, selective reveal and instructions developed in [52i](52i-canonical-capabilities-and-progressive-reveal.md).
Initially each local attachment runs its own STDIO process from that same implementation.
Programming agents and prospective consumers select relevant capabilities within the same
surface. There is no separately maintained developer server, parallel handwritten CLI grammar,
semantic capability search or new operation scheduler.

This document owns attachment/bootstrap, specialist connection lifetimes, actual host
presentation, loaded-interface consistency, conditional generated interfaces and interface
observation. [52j](52j-native-contracts-and-tool-adaptation.md) owns operation contracts and
tool adaptation; [52g](52g-agent-and-operator-workflows.md) owns assembled agent workflows.
The [coordinator](52-agent-effectiveness-and-engineering-workflows.md) alone owns package,
opportunity and inquiry status. References below mean the
[unified-interface review](../functional_reviews/unified-fastmcp-capability-interface-2026-10-24.md),
not the earlier reviews' similarly numbered opportunities.

## Foundation assessment and preserved authority

The review inspected FastMCP4.1.3/MCP2.3.0/Pydantic2.14.1 and tested a bounded synthetic
two-process reveal scenario. That establishes a usable library mechanism, not actual Codex
attachment/refresh, assembled native behavior or measured productivity. The implementation
must qualify those separate consumers. A high-functioning Codex integration is a settled design
premise: the maintainer confirmed on 2026-10-24 that environment-JSON MCP attachment has worked
previously. Q01 chooses and tests the best supported configuration/reveal behavior; it does not
question feasibility or block the architecture. Recheck selected versions and consequential source
contracts when executing; do not substitute older FastMCP documentation for the reviewed APIs.

The review baseline's [session configuration](https://github.com/paul-heyse/pse-arrow/blob/e0a7fe7ea21976f5edef43244e4c04983be2d3e8/scripts/codex_session.py) separately attached
Rust, Python and selected-database MCP services. Plan52j replaces those declarations with the
canonical attachment and internal lazy connections, preserving explicit checkout selection,
local preparation versus actual attachment, pinned interpreter paths and database selection
fencing. [Language navigation](https://github.com/paul-heyse/pse-arrow/blob/e0a7fe7ea21976f5edef43244e4c04983be2d3e8/scripts/semantic_navigation.py) owns pinned
bridge configuration, readiness and finite-request transport cleanup. Reuse that owner rather
than recreate LSP position/edit semantics. Native database selection and credential handling
remain with [the supervisor](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/surreal_server.py).

The [environment guide](../dev/agent-environment.md) and environment/admission owners govern
execution configuration. A long-lived read/control endpoint must not acquire workload
admission or prepare compilers merely to stay reachable. This follows the existing read-only
`activity` exception's purpose, rather than generalizing its implementation into an arbitrary
command bypass. Effectful tools still use their existing workload/native/store owners.

The operator accepted review RC02/RC05 on 2026-10-24; RC01/RC03/RC04 follow explicit selections
for one interface, derived projections and permitted interface refactoring. The coordinator
records decisions and routes. MCP01 implements the launch guidance portion of RC05 before
depending on it. Governed boundary or authority changes, if actually needed, use the existing
ADR/design route; ordinary tooling changes do not require a placeholder ADR.

## Runtime and interaction design

One explicit checkout attachment carries configured project/service selectors where needed.
Presentation selection is separate from that context. Revealing a database workflow never
changes the selected database; reconnecting never replays an effect. A missing selected
service produces its native unavailable/identity refusal while source, instructions and
offline evidence remain usable. Credentials are acquired internally and redacted, not exposed
as public tool parameters or copied into catalogue descriptions.

Local composition should remain ordinary modules/providers. Use scoped dependency injection
and async context managers where they improve acquisition/teardown. A FastMCP mount enters
child lifespans at parent startup: [composition](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/docs/servers/composition.mdx)
does not make an eager LSP/store lifespan lazy. Acquire an upstream connection on the first
operation that needs it, preserve identity/readiness checks, and close owned transports on
failure and endpoint shutdown. Decide reuse at that owner from actual acquisition/lifetime
semantics; do not reopen a compiler-backed language server for each symbol merely because
requests are logically independent. Do not keep unrelated services alive by catalogue access.

Process-local reveal is deliberate. In MCP2026-07-28, FastMCP STDIO/in-memory visibility follows
the server process; authenticated HTTP follows the principal across connections, and
unauthenticated HTTP cannot persist the same built-in selection. Two clients sharing one
in-memory server therefore do not establish independent attachment state. HTTP/shared-process
deployment is outside initial support and requires explicit scope/identity design before use.
See [Context](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/fastmcp_slim/fastmcp/server/context.py).

CAP owns deterministic selected-set replacement, serialized changes and reset using the
framework visibility mechanisms. MCP02 establishes whether the real Codex host incorporates
their list-change notifications. An explicit refreshed library `list_tools()` is comparison
evidence, not the host result. If normal reveal is unavailable, choose exact description plus
canonical invocation within this same endpoint only after observing the host limitation.
Such an adapter delegates the registered tool's validation, middleware and effects; it is not
an arbitrary shell/SQL dispatcher or a second action language.

Loaded handlers, schemas, exposure metadata and instruction interpretation form one loaded
interface revision. Initially restart after relevant integration changes. Source-inspection
tools still inspect current source using their own freshness contracts. They must not imply
that imported handlers have reloaded. For a changed serving guide, either serve its declared
compatible snapshot or report the relevant restart requirement; do not silently mix new
instructions with an old contract. A scoped fingerprint can identify a relevant change,
but cannot prove arbitrary dependency freshness. No per-read whole-repository hash is required.

Large evidence stays with its existing reader and handle. Response-size middleware may bound
rendering, but must preserve completeness/omission meanings supplied by the operation owner.
Cancelling an observer detaches it; cancelling a native action uses its native contract.
Thread timeout does not undo native work. Tasks/Docket, a new cancellation manager, writable
handoff ledger and duplicate telemetry store are not selected.

## Work packages

<a id="MCP01"></a>
### MCP01 — Cheap attachment and lazy specialist lifetimes

Create the composition/launch owner using the pinned checkout interpreter and explicit
configuration. Consume the CAP bootstrap metadata contract and an ADP operation binding;
neither companion must be complete to deliver the first useful source-only slice. Starting
the endpoint performs no build, package sync, database startup or workload allocation.
Expose observed dependency availability and owned preparation references without effects.

Move selected-session wiring from separately required specialist attachments to the canonical
entry point as each internal integration becomes usable. Coordinate adapter normalization
with ADP; preserve the source, language and database owners' behavior. Lazily acquire internal
proxy/provider connections, retain selected-service identity fencing, and define readiness,
reuse, failed acquisition and shutdown at the integration owner. Use native execution
admission only for selected effectful actions. Remove replaced attachment configuration and
its obsolete tests when real callers migrate; retain independently useful native commands.

Resolve Q03 through an actual language request and selected-database request, including
unavailable dependency, identity change, cancellation and owned transport teardown. A running
observer may not cancel the subject it watches. A long-lived endpoint must not accidentally
own native work whose lifetime belongs to another process/allocation owner.

Targeted acceptance: start with compiler/admission/store unavailable and obtain root guidance,
source/offline reads and a truthful unavailable action. Observe that only a requested specialist
starts, repeated suitable requests reuse its preparation, and unrelated capabilities survive
its failure. Confirm two endpoint processes have independent presentation and teardown.
Inspect actual owned processes/connections rather than equating a Python object close with
successful cleanup. Source O06/O02; Q03; RC01/RC05.

<a id="MCP02"></a>
### MCP02 — Actual Codex reveal, instructions and invocation

Consume usable CAP browse/describe/reveal/reset and instruction-serving slices, MCP01 launch,
and one actual ADP operation with its result reader. Use the established environment-JSON configuration route to attach a fresh Codex session to
the canonical endpoint, refining configuration as useful for the current installed host. Observe bootstrap listing, select an exact workflow, obtain substantial
instructions, invoke the real operation, read its native outcome, reset and select another
workflow. Keep configured, available, attached and exercised separate.

Settle Q01 as implementation verification with actual callable inventory refresh, resource/template
and prompt access. Successful integration is the premise; tests select the best supported route.
Where the host cannot present resources/prompts, consider FastMCP `ResourcesAsTools` or
`PromptsAsTools` over the same content. Select same-endpoint exact invocation only if dynamic
tool refresh cannot support ordinary revealed calls; preserve direct tool semantics and
avoid exposing every rich schema initially as a workaround. Record supported host behavior
and the smallest required adaptation at its owning integration, not a new developer interface.

Targeted acceptance includes hidden-call refusal, idempotent selection, concurrent selection
updates, reset, independent second attachment and planning-only skill retrieval without
implementation effects. Independently expected identities and side effects must expose a
disconnected catalogue or incorrect reveal. The prior synthetic probe remains historical
library evidence and cannot close this package. G02/G03 consume the resulting supported
interaction; H02 qualifies its assembled workflows. Source O02/O03; Q01.

<a id="MCP03"></a>
### MCP03 — Coherent loaded contracts and relevant restart

Bind loaded component schemas, handlers, exposure metadata and applicable guide interpretation
to one interface revision. Use the composition owner and existing per-owner source cuts to
define which relevant changes require restart; fresh source facts remain operation-owned.
Coordinate native schema identity with ADP and instruction identity with CAP. Do not make
unrelated model/document edits invalidate all serving operations.

Settle Q06 using an edit to a serving tool schema/handler and an edit to a selected guide,
alongside an unrelated source edit. Demonstrate either a compatible declared snapshot or an
explicit restart-required result. Restart then serves matching new handler/schema/guidance.
Hot reload is optional only if it can establish the same atomic consistency and retire old
resources; retaining multiple versions is not required. Update launch/help instructions and
remove any replaced assumptions that a running endpoint automatically imports new files.

Targeted acceptance: a new parameter must never be advertised while an old handler executes;
native source navigation still sees current supported files. Inspect catalogue work separately
from output size: avoid repeated global schema/provider construction or eager service startup.
A bounded metadata index per loaded revision and targeted provider lookup may suffice; do not
invent a general cache. Source O09/O06; Q06; RC05.

<a id="MCP04"></a>
### MCP04 — Derived reference material and conditional Cyclopts CLI

Derive signatures, component identities and reference links from canonical FastMCP definitions;
retain authored workflow rationale at its Markdown owner. Consume CAP's registered catalogue,
ADP's faithful contracts and MCP02's invocation route. Generated documentation must identify
planned/unavailable capabilities rather than advertise them as exercised operations.

CLI delivery requires a concrete terminal/consumer need. If selected, settle Q05 before
advertising equivalence. FastMCP4.1.3's [generator](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/fastmcp_slim/fastmcp/cli/generate.py)
uses `client.list_tools()` and therefore sees only current reveal. Generate from a deliberate
complete selection of those same canonical definitions, not another all-tools server or
editable catalogue. A generated invocation creates a fresh connection: derive exact target
activation before its call, or consume the qualified same-endpoint invocation path.

Correct the generator/owned derivation's filtering of `None` and empty lists before mutation
use, then regenerate output. Preserve absent, explicit null and empty values, required nullable
arguments, defaults, JSON/recursive schemas, native refusal and exit/error meaning. Do not
hand-edit generated output or retain a Typer/independent agent CLI. An upstream fix or narrow
owned generation integration is preferable to copying a broad CLI framework.

Targeted acceptance uses independent expected absent/null/empty payloads, hidden catalogue
members and fresh STDIO connections; a schema-generated round trip alone is inadequate.
If no CLI is adopted, document the consumer decision and observable revisit trigger in the
coordinator, preserving conditional status rather than inventing CLI equivalence evidence.
Derived reference documentation remains required. Source O08/O01; Q05; RC03.

<a id="MCP05"></a>
### MCP05 — Interface observation and qualification integration

Connect selected exposure, request/provider and native invocation/result references where
they serve an actual diagnostic read. Use FastMCP middleware/OpenTelemetry hooks and existing
Plan51 evidence/traces. Distinguish transport, provider, adapter and native execution failures;
unknown timing and absent continuity stay explicit. A transport success cannot promote a
scientific refusal or incomplete test population to success. Redact credentials/raw sensitive
input at the existing owner rather than disabling masking globally.

Consume ADP's bounded outcomes/handles and MCP01 lifetimes; coordinate with F01 comparison,
F02 finite trace inquiry and G02 workflow integration. Return useful native evidence pointers
when the dependency fails before native work starts, when an operation returns partial data,
and when observation detaches from ongoing work. Do not add another evidence database or
claim end-to-end Surrealism trace continuity merely because spans share a label.

Targeted acceptance uses an actual failed composed interaction, partial result, disconnected
observer and native result read. Confirm correlation leads to the authentic owner and that
response bounds retain omission/continuation meaning. H01/H02 integrate actual-host reveal,
lazy failure, relevant restart, migrated specialist consumers and any adopted generated CLI.
H03 owns scope-end static/manual checks; no full campaign recurs per package. Source O10/O07;
Q06. Quantitative overhead/productivity claims require separately named conditions and
measurements; functional completion needs no invented performance score.

<a id="MCP06"></a>
### MCP06 — Execution identity, environment selection and wire agreement

**Proposed:** make the selected checkout, effective launch selection and admitted operation contract
agree across the existing endpoint/worker boundary. Reuse MCP03/NAV03 source fences, catalogue/tool
definitions, fixed worker envelopes and LIFE04's native artifact basis. Preserve one explicit checkout
per attachment: show its identity and use its interpreter/cwd; do not add a general per-call worktree
router, second service environment or fresh process for every lightweight read. Existing native
helpers and lazy managed specialist lifetimes remain selected by their actual ownership needs.

At dispatch, distinguish explicit operator overrides from inherited derived defaults. Resolve the
selected operation's launch defaults through `pse_env`/existing owners without silently retaining an
obsolete derived selection or overwriting an intentional override. Keep deliberately process-owned
SDK/provider settings fixed until their disclosed reattachment boundary; request-local credentials
remain private. Existing compiler/test-width refresh logic is a foundation, not a reason to reset
all environment variables. Carry only the selected non-secret identity needed for diagnosis.

Fingerprint the complete exposed `to_mcp_tool()` contract, including schemas, annotations, metadata
and descriptions, from canonical definitions at the composition owner. Separate underlying operation
identity from attachment reveal/transforms so visibility changes do not invalidate unrelated native
artifacts. Carry the admitted schema/effect/route revision through fixed helper dispatch and compare
against the executing definition before effects; mismatch returns a bounded stale-contract remedy.
A post-call ledger or refreshed catalogue alone cannot establish that agreement. Keep schema
compatibility, loaded-source identity, launch selection and artifact currency distinguishable;
no new universal identity store, hot reload or atomic whole-checkout snapshot is selected.

Targeted acceptance overlaps two different selected environments without shared-state mutation,
changes a derived default while preserving an explicit override, and rejects changed schema/effect/
route before effects through direct and fixed invoker paths. Include independent checkout attachments,
unchanged/unrelated-source reads, metadata/annotation/transform fingerprint changes and relevant
restart recovery. Existing typed bounded answers/detail references must preserve output schemas and
completeness; a rendering limiter must not silently erase their advertised contract. Regenerate
reference/examples from their current owners and remove replaced incomplete identity/dispatch logic
and its obsolete tests once callers migrate. G02/G03 and H02 own the current actual-host journey.

## Execution and verification boundaries

Start with CAP's exact discovery/instructions contract, ADP's one useful operation/result and
MCP01/MCP02 attachment to deliver the real first journey. Continue coherent tool-family
migrations; do not wait for all catalogue prose or require every operation before any use.
MCP03 consistency should accompany relevant integration changes, not become late cleanup.
Specialist migration and result preservation remain obligations even after one host journey works.

**Tested:** targeted implementation and actual-host evidence is recorded below.
[52h](52h-qualification-and-publication.md) retains final assembled qualification,
series-wide scope-end checks, publication and honest exclusions. This component's closure
does not close scientific/product qualification. The coordinator records package progress,
inquiry resolutions and rule routes; this companion owns its detailed execution evidence.

## Verification and Outcome — 2026-10-24

**Implemented:** one optional source-only STDIO attachment retains the delivered MCP01 lazy
specialists. Eleven bootstrap tools provide exact navigation/invocation plus framework projections
of the same resources/templates/prompts. Selected workflow prompts are compact indexes with
digests and references to complete owned instructions. Fixed-inventory Codex hosts use canonical
invokers rather than a second operation language. Only real inherited specialist transports are
disabled; no invalid disabled transportless tables are synthesized.

**Implemented:** a bounded local source cut captured before binding imports ties handlers,
schemas, metadata and fixed child-worker implementations to one loaded revision. Composition
checks and relevant dispatch fences reject changed owners before effects; bootstrap describes
the loaded state and restart cause. Guide edits affect that guide: complete captured snapshots
remain attributable, uncaptured changed content refuses, and unrelated guides remain usable.
Current scientific/source freshness and arbitrary third-party readiness remain operation-owned.

**Implemented:** complete composed reference generation includes hidden operations, signatures,
defaults, schemas, resources/templates/prompts, native routes and explicit unavailable/deferred
classifications. The dedicated `docs/generated/capabilities/` namespace has one generator;
Rust Markdown generation preserves it. Comparison is read-only and diagnoses missing/stale/extra
files and symlinks. Publication removes only owned stale output. The former marker-owned
flat output is retired. Source/schema links follow package repository metadata without copying
unpublished JSON assets into the site.

**Implemented:** bounded middleware correlation preserves original payloads and owner metadata,
joins nested invocation, records failure stage/interface timing and points to existing authentic
native readers. Known missing references, observer detachment, native timing, transport delivery
and trace continuity retain explicit limitations. Structured diagnostics exclude raw arguments,
credentials, paths and exception text. Existing recording OpenTelemetry spans receive attributes;
no new exporter, evidence store or scheduler is introduced. The direct dependency declaration
pins `opentelemetry-api==1.45.1`, already resolved transitively at that version.

**Tested — zero failure baseline, local pinned environment:**

- `scripts/pse-env -- .venv/bin/python -m unittest discover -s scripts/tests -p 'test_capabilit*.py'`:
  162 passed. Controls include hidden calls, native/projected content identity, pure planning
  retrieval, atomic selection/reset, scoped guide changes, worker/source restart fences,
  failed/partial interactions, detached watch followed by authentic result read, redaction,
  reference fidelity and non-mutating generation checks.
- `scripts/pse-env -- .venv/bin/python -m unittest scripts.tests.test_codex_session scripts.tests.test_validation scripts.tests.test_test_invocation scripts.tests.test_test_run scripts.tests.test_validation_result`:
  99 affected-owner tests passed. Subsequent typed session/driver repairs also passed their
  selected 15-test run; capability driver tests are included in the final 162-test suite.
- `scripts/pse-env -- .venv/bin/python -m scripts.tests.capability_codex_check --output build/capabilities-52k/codex-host-stable --timeout 600`:
  fresh actual installed Codex passed seventeen canonical calls: discovery, schema descriptions,
  complete skill resource, capability template, workflow prompt, reset, exact Rust action,
  authentic result reader, reset, second workflow/prompt and final reset. The single authorized
  `pse-ids::float::tests::f32_nan_variants_collapse_too` unit passed. Independent original-owner
  verification found one selected subject, zero unreported subjects/report errors, frozen assurance
  pass, workload exit zero and complete owned drain. Invocation:
  `1b7e8e57c5d5410c8a79e7341f6772b8`. Evidence remains under that output directory, including raw
  events, launch configuration and independently read native result. Ordinary approval policy is
  unchanged; only this bounded acceptance session explicitly approves its selected action wrapper.
- `scripts/pse-env -- .venv/bin/python -m scripts.arrow_validation test -p xtask --no-default-features --locked codegen::tests::markdown_generation_preserves_capability_reference_owner_and_checks_own_stale_files -- --exact`:
  one regression passed, 81 filtered; Rust generation preserves the capability owner's bytes while
  detecting/pruning its own stale Markdown.
- `scripts/pse-env -- just codegen` and `scripts/pse-env -- just codegen-check` passed publication
  and every declared generation comparison; `scripts/pse-env -- just docs` published 630 chapters.
  Historical unsupported-server-profile cleanup deferrals remain separate and preserved.
- Scoped `.venv/bin/ruff check` and `.venv/bin/pyrefly check --min-severity warn` through
  `scripts/pse-env --`, over `scripts/capabilities`, `scripts/codex_session.py`, capability tests,
  the actual-host driver and session tests: zero findings/diagnostics.
- `scripts/pse-env -- .venv/bin/python -m scripts.arrow_validation clippy -p xtask --all-targets --no-default-features --locked -- -D warnings`:
  passed with zero warnings. `just ready`, `just lint-agents` and root `just turn-end` passed.

Independent implementation review found two material omissions and rechecked their repairs:
string-selected worker modules needed explicit source-cut membership, and the existing Rust
Markdown generator needed to exclude the capability reference's dedicated owner namespace.
Its source-only recheck covered five real operation/worker bindings with zero failures.

**Mistakes corrected:** initial disabled specialist placeholders were rejected by the actual host;
the launcher now configures only inherited real transports. Pure framework content projections
needed truthful read-only annotations. Embedding about 136 KB of guide bodies in prompts hit the
host display bound; compact exact references now preserve full resource content. Host completion
order for independent descriptions is legitimately concurrent and the checker now accepts either
order before selection. A native worker build exposed missing PETSc provider preparation;
the Rust-test helper now uses its declared solver/KLU/isolation/Uno/PETSc environment. One later
attempt correctly refused when parallel code generation changed execution inputs during
preparation; the fresh stable-tree journey passed after generation completed. Failed host artifacts
remain alongside the successful run and do not count as passing evidence.

**Deliberate deviations:** no worktree was created after the maintainer selected main. The
maintainer deferred Cyclopts CLI delivery: existing native commands and canonical MCP meet current
consumers; Q05 records the concrete terminal/non-MCP need that triggers reconsideration. No CLI
fidelity or fresh-connection equivalence is claimed. Prompts summarize references rather than
republish full instructions. The current native runner's broader worker preparation remains its
existing mechanism, separate from the single selected test; Plan53 build restructuring is not
silently folded into this scope.

**Limits / handoff:** this completes MCP01–MCP05 component obligations, not F/G/H assembled
agent-task evaluation, workspace-wide Clippy/feature matrices, scientific campaigns, performance
or measured productivity. No end-to-end trace continuity is established. The host emitted its
existing skill-scan traversal-limit diagnostic; the exact requested skill was independently read
through its complete owned resource and the journey passed. C04's separately denied execution
and independent Plan28/32/40/53 work are unchanged. Enduring workflows live in the
[capability interface guide](../dev/capability-interface.md). This completed companion remains
retained for the active coordinator and H01–H04 consumers rather than retiring their evidence.

### Post-delivery attachment and hands-on assessment — 2026-10-24

**Implemented:** the maintainer's request to use the interface in an ordinary Codex session
exposed an attachment gap: the explicit `codex-session` launcher supplied the endpoint, but
ordinary project startup did not. The project config now registers the existing fixed
`just --quiet capabilities` recipe with finite startup/tool timeouts and disables the inherited
standalone Rust transport. This changes neither approval policy nor native operation admission.
No new launcher, global registration or hard-coded checkout path was introduced.

**Tested:** `codex mcp get pse-capabilities --json` through the checkout environment resolved
the enabled required attachment from both root and `docs/plans`. A direct persistent FastMCP
client launched that exact configured transport from `docs/plans`, received eleven bootstrap
tools, and exercised twenty read/presentation calls. The agent used discovery, descriptions,
selection, actual plan/package context, recipe inspection, exact concept querying, the original
frozen test result reader, workflow prompt and reset. It independently obtained the current
MCP03 complete state and original invocation `1b7e8e57c5d5410c8a79e7341f6772b8` with one pass,
assurance pass and complete drain. Two concept requests refused (one unsupported-vocabulary
control and one identity-grammar recovery); the corrected exact identity succeeded. A hidden
read after reset refused as intended, but its client-facing error was generic. Raw request/
response evidence remains at `build/capabilities-52k/usability-assessment/`.

This is hands-on protocol evidence, not proof that the already-running parent Codex session
has reloaded native tools. That session exposed no canonical tools at inspection; a fresh/reloaded
attachment is required for the maintainer-requested direct native-tool assessment. The prior fresh
Codex action journey retains its separate accepted evidence.

**Proposed improvement opportunities from actual use:** expose the supported concept vocabulary
and identity examples in the callable schema, and return actionable visibility/selection errors
instead of a generic invoker failure. Correct recipe-owner metadata still pointing to the removed
`scripts/capabilities.py`. Reduce routine context transfer through instruction-resource references
and selected detail sections: selecting two read tools returned about 48 KB, and `inspect-plan`
revealed 24 operations with about 80 KB of response content. The exact single-package query was
about 12 KB. These are observed response sizes, not measured productivity claims. CAP/ADP's
existing definition owners and G03's remaining task-effectiveness work are the appropriate routes;
this assessment does not authorize or claim those additional implementation changes.

## MCP06 verification and outcome — 2026-10-24

**Implemented:** the existing attachment and environment owners now disclose checkout/interpreter
identity and distinguish intentional caller selections from derived defaults. Request-private
admission carries the original complete tool contract, selected source basis, fixed worker route
and request bytes to the executing owner. Child verification checks the admitted source set
without parsing unrelated imported implementations. Fixed native read routes derive from their
original bindings. The shared qualification module retains distinct `qualification` and
`rust-tests` admission routes; revealing or resetting tools does not replace native admission.

**Tested — zero failure target, source-only attachment conditions:**
`scripts/pse-env -- .venv/bin/python -m scripts.tests.plan52_execution_identity_acceptance --worktree /home/paul/pse-arrow-wt/plan52-attachment-acceptance --output build/plan52-execution-identity-20261024.json`
passed two real STDIO attachments. The main checkout retained its explicit compiler-width value
16; the independent checkout resolved its default 32 and its own checkout-relative wrapper. These
are selected precedence controls, not host resource recommendations. Reveal/reset state stayed
independent. A changed serving source refused in the affected attachment, the other attachment
remained usable, restored source was checked, and fresh reattachment passed. The receipt records
both original checkout/interpreter identities. This source-only experiment compiled no native
artifact and establishes no cross-checkout artifact equivalence or scientific outcome.

**Tested:** the fresh G01 actual-Codex action/result journey passed after the route correction;
its exact command, original invocation and independent result are owned by
[52g's bounded workflow Outcome](52g-agent-and-operator-workflows.md#verification-and-outcome--bounded-workflow-consumers-2026-10-24).
The current-artifact F01 consumer used the same canonical native read binding after explicit
selected-environment preparation; [LIFE04's Outcome](52n-readiness-progress-and-edit-lifecycles.md#life04-verification-and-outcome--2026-10-24)
records its source cut, artifact and original owner evidence. These positive consumers complement
route/source/wire controls; they do not turn a declared route into proof that every native read
was exercised through the actual host.

**Implemented; Tested — mistake corrected:** current-checkout execution exposed a valid Rust
request being checked against the shared module's wrong worker route. The narrow route selection
and real private-handoff regressions preserve wrong-route refusal while allowing the actual
original Rust owner. Independent review also corrected an incomplete native-operation allowlist,
unrelated-source expansion during child verification and an import-currency race. The import and
file-witness contract belongs to LIFE04. Earlier refused host turns remain attributed rather than
being recast as completed actions.

**Proposed — remaining boundaries:** ordinary approval policy and the original native effect,
handle, drain and scientific authorities remain unchanged. Optional production CodeMode/tasks and
the operator UI are unselected pending their existing operator decisions. The native consumer
recorded after review baseline `3e99a220f74b31a063d3678534ff445d38ad8b7f` precedes the study-result
writer repair; a later Rust artifact requires its own current basis. Full H02/H03 and trace-continuity
claims remain with their owning plans. This update records MCP06's bounded delivery and consumers,
without reopening or strengthening the historical MCP01–MCP05 qualification.

### Current-source assembled consumer — 2026-10-25

**Tested, bounded:** [52h's actual Codex recovery](52h-qualification-and-publication.md#actual-codex-telemetry-and-current-native-recovery-boundary)
records post-writer-repair current native admission, exactly one selected-endpoint preparation,
two manifest-protected schema pages and the original typed context-bound value refusal. The
failed enclosing values assessor remains failed; no positive row-value or full scientific claim
is inferred. Full scope-end and composite repaired qualification remain at52h, preserving the
earlier MCP01–MCP05 and source-specific MCP06 observations.
