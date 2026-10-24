---
title: "Plan 52k: Unified entry point and derived interfaces"
status: draft
date: 2026-10-24
adrs: []
review_sources: [docs/functional_reviews/unified-fastmcp-capability-interface-2026-10-24.md]
doc_role: plan
doc_owner: docs/plans/52-agent-effectiveness-and-engineering-workflows.md
doc_topics: [current-work, environment, design, diagnostics, documentation]
---

# Plan 52k: Unified entry point and derived interfaces

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

Current [session configuration](../../scripts/codex_session.py) separately attaches Rust,
Python and selected-database MCP services. It already preserves explicit checkout selection,
availability versus attachment, pinned interpreter paths and database selection fencing.
Retain those distinctions while replacing the separate required attachments with internal
providers. [Language navigation](../../scripts/semantic_navigation.py) already owns pinned
bridge configuration, readiness and finite-request transport cleanup. Reuse that owner rather
than recreate LSP position/edit semantics. Native database selection and credential handling
remain with [the supervisor](../../scripts/surreal_server.py).

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

## Execution and verification boundaries

Start with CAP's exact discovery/instructions contract, ADP's one useful operation/result and
MCP01/MCP02 attachment to deliver the real first journey. Continue coherent tool-family
migrations; do not wait for all catalogue prose or require every operation before any use.
MCP03 consistency should accompany relevant integration changes, not become late cleanup.
Specialist migration and result preservation remain obligations even after one host journey works.

**Proposed verification:** targeted package tests exercise the distinctions above during
implementation. [52h](52h-qualification-and-publication.md) owns final assembled qualification,
scope-end checks, publication and honest exclusions. This plan authoring implements no
production endpoint, configures no attachment and qualifies no scientific or host behavior.
The coordinator records progress, inquiry resolutions, rule routes and eventual Outcome;
this companion does not maintain a competing completion ledger.
