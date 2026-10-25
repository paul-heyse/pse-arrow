---
title: "Plan 55a: Capability contracts, clients and recovery"
status: in-progress
date: 2026-10-25
adrs: []
review_sources: [docs/functional_reviews/mcp-development-effectiveness-2026-10-25.md]
doc_role: plan
doc_owner: docs/plans/55-mcp-development-effectiveness.md
doc_topics: [current-work, design, environment]
---

# Plan 55a: Capability contracts, clients and recovery

## Target and foundation

**Proposed:** give Codex constructible requests, meaningful outcomes and an independent faithful
CLI consumer of the canonical endpoint. Owns review O01/O02/O04/O10; the
[coordinator](55-mcp-development-effectiveness.md) owns opportunity disposition and shared rules.
Advanced CLI editing is a separate [55d inquiry](55d-editing-validation-and-regression-experiments.md#D03).

**Interface-checked:** at `391c949b5`, `catalogue.py` derives workflow membership from declarations;
`coherence.py` exposes `operation_wire_revision` in descriptions; `execution_basis.py` binds the
wire/source/worker handoff; fixed invokers call the canonical server. Pydantic host contracts,
native generated codecs and path-aware diagnostics already exist. Extend their consumed meaning,
not another validation/catalogue framework.

The review's pinned FastMCP 4.1.3 generator drops explicit null/empty lists, discovers only visible
tools and reconnects for calls. Generating a selected tool did not preserve live selection.
These facts require an owned adaptation, not an assumption that changing help fixes behavior.
Use [FastMCP §11.5](../libraries/fastmcp.md#115-cli-and-generated-tool-interfaces),
[Pydantic §3.4](../libraries/pydantic.md#34-missing-null-defaulted-and-explicitly-provided-values)
and the [tagged emitter](https://github.com/prefecthq/fastmcp/blob/v4.1.3/src/fastmcp/cli/generate.py).
Installed pinned source and review controls take precedence over older generic snippets.

## Resulting interaction and ownership

For an exact capability, the agent reads its contract/example and an explanation of the actual
request, then uses MCP or the derived CLI. The CLI resolves its selected checkout/environment,
connects, describes that operation, checks its generated contract basis, establishes selection
and invokes through normal dispatch in one Client context. Native admission remains at the
performing owner. Discovery or an eligible CLI lifetime grants no scientific permission.

Use a deterministic repo-owned schema-to-Cyclopts emitter, narrowly adapting the pinned FastMCP
mapping/template behavior, plus one shared CLI runtime. Derive commands from `Catalogue.tools`
and canonical wire schemas, not progressive `list_tools()` discovery. No handwritten callback
implementation per operation; no textual surgery on emitted files, site-packages patch, full
FastMCP fork or dependency on an upstream merge. Generic fidelity fixes are suitable upstream
contributions, but endpoint selection/coherence/lifetime behavior is local integration.

The full command index describes native/deferred entries accurately; it does not turn them
into commands that bypass admission. Retain progressive MCP reveal. Machine JSON/file input
and output are first-class alongside flags/help; global options must not collide with tool
fields. Cyclopts is lock-resolved at 5.3.0; make it an exact direct declaration if importing it.
No unrelated dependency resolution or major upgrade is required.

Compare the described **operation** wire revision, not the catalogue/source revision. An
unrelated tool/source edit need not invalidate a compatible generated command. The live server
still applies its relevant-source fence and worker admission independently. Incompatible old
defaults/aliases can be schema-valid, so this client check must happen before effects. Do not
silently send baked defaults for omitted arguments. Static help identifies its basis and can
work without endpoint startup, but does not establish current callable compatibility.

## Package state

| Package | Status | Owner | Completion |
|---|---|---|---|
| A01 | in-progress | 55a | Meaningful host outcomes and exact contract/example projections used by consumers |
| A02 | in-progress | 55a | Faithful derived CLI with selected-checkout dispatch and explicit supported lifetimes |
| A03 | in-progress | 55a | Layer-specific independent diagnosis and qualified host refresh guidance |
| A04 | in-progress | 55a | Actual-request explanation for one native validation and one model-development route |

<a id="A01"></a>
### A01 — Constructible directional contracts and examples

Extend authoritative host declarations where outcomes are still generic and the actual owner
has distinct selected/admitted/observed/unavailable/refused states. Preserve those distinctions;
do not impose a universal state machine or equate accepted work with completed work. Describe
prerequisites/effects, exact identity grammar, reference lifetime and original next readers.
Add the smallest owner-authored CLI lifetime distinction to the existing capability model:
single invocation, retained endpoint session, or native-route-only, with the owning reason.
Read-only annotations and operation names cannot supply that decision reliably.

Examples belong to the schema owner: Pydantic `Field`/JSON Schema metadata for host records;
Schemars examples and generated native documents for scientific requests. Expose exact,
fingerprint-qualified contracts/examples through existing SchemaSet/resource projections and
fixed resource tools. Reuse immutable schema/TypeAdapter preparation by its actual basis rather
than caching mutable operational results. Native codecs validate science; JSON Schema error
explanation is supplemental. Placeholder identities are labeled illustrative, never eligible.

**Acceptance:** construct a rich-union request from an actual selected example; check it against
the correct validation schema and the returned serialization variant. Independently specified
invalid identity/union/presence cases retain useful error paths. Scientific examples migrate
with [C03](55c-native-model-and-result-inspection.md#C03); host projection never silently truncates
them. Delete replaced generic projection code only after its consumers have moved.

<a id="A02"></a>
### A02 — Derived CLI and faithful request/result delivery

Consume A01 and existing catalogue/operation basis. Put emitted command declarations under the
existing generated tooling ownership, register their generator/protection and canonical
codegen/check route, and keep the shared connection/runtime code authored. Generation must not
prepare native libraries, query a store or depend on a live selected session. Codegen comparison
detects drift without rewriting outputs. Add current-checkout launch/help to existing tooling
entry points; do not bake this workstation's absolute checkout into the delivered client.

Preserve omission, explicit null, empty arrays/maps, zero/false, defaults, aliases, nested unions
and exact-width values. JSON/file input bypasses argv ambiguity, not strict receiver validation.
Preserve typed refusals, meaningful content/structured content/metadata, source basis, uncertainty
and original owner references in the machine result; readable rendering is a view of that result.
Do not claim multimodal byte preservation from placeholder output. Classify initial operation
eligibility by declared lifetime, not name prefixes: stateless queries and durable original-owner
reads are useful immediately; endpoint-bound sequences receive an explicit supported-route
limitation until D03/D04 decide otherwise.

One invocation owns one fresh Client and its exact select→call sequence. Fail compatibility,
selection and input checks before effects; do not retry an action on transport uncertainty.
On error, retain enough independent launch/response diagnostics to follow A03's recovery.

**Acceptance:** extend the review's hand-specified null/empty controls through emitted CLI and
direct JSON, plus omission/zero/false/nested aliases and typed refusal. Check hidden-operation
dispatch on a fresh connection; relevant old wire refusal before effects; unrelated operation
change tolerance; exact original-reference/result retention; help without server; broken-server
invocation failure; selected checkout from a subdirectory. A missing ACK has no automatic replay.
Use small disposable tools for fidelity and one actual source-only operation, not a compiler run.

<a id="A03"></a>
### A03 — Availability diagnosis and real host recovery

Use existing fixed launch, `pse-env --explain`, loaded/current coherence, readiness and original
work readers to distinguish stale attachment, absent host attachment, failed server launch,
unavailable native prerequisite and lost effect ACK. Independent diagnosis must remain usable
when endpoint startup fails. It reports the stage actually reached and selected paths/bases;
never expose secret environment values or imply that a hash explains its own mismatch.

A fresh handshake is an explicit bounded child/connection, not passive preparation or repair
of the old process. Diagnose without package sync, native build, service startup, hot reload or
global process killing. Supported recovery is fresh CLI for attachment failure; existing direct
owner commands/launch diagnostics for endpoint failure; explicit original preparation for native
unavailability; original invocation/status/result/reconcile for uncertain admitted actions.

Assess the installed Codex frontend's actual refresh route. The
[official app-server API](https://learn.chatgpt.com/docs/app-server#api-overview) documents
`config/mcpServer/reload`, `mcpServerStatus/list` and startup notifications, but does not establish
which are exposed to this session. Prefer that supported route if demonstrated; otherwise
document the actual frontend restart/reattach procedure and the independent CLI. No invented
universal reconnect command or new broker. Avoid disrupting a live effect merely to test reload.

**Acceptance:** controlled stale-interface, missing-launch and unavailable-prerequisite cases
produce different useful remedies. Qualify actual host refresh with a benign source-only read
and preserve active work ownership. Record unavailable host controls honestly. Durable work
remains readable after reconnect; endpoint preview handles are not falsely declared recovered.

<a id="A04"></a>
### A04 — Explain the intended typed invocation

Compose current descriptions/requirements, readiness observations, execution-basis binding and
the actual operation's selection/preparation owner for one native validation route and one
model-development inspection route. Return what that particular request would attempt here,
its source/schema basis, known population, required preparation choices and remaining admission
unknowns. Reuse current readiness tools and progressive resources/workflows; add a narrow typed
operation only where those cannot express the useful request-specific answer.

Keep description, input validation, passive selection, live checks, admission and effects
distinct. Explanation must not prepare providers/start services or promise universal dry run.
Native model inspection's potential preparation remains explicit. It does not infer a cache
miss cause from absence, or treat a service unnecessary for the request as a broken requirement.

**Acceptance:** explain then invoke the same request through the original receiver; changed
source invalidates applicability, a missing optional backend does not block an unrelated read,
and unresolved runtime admission stays unknown. D01 supplies affected-population meaning and
C01 supplies scientific inspection policy without moving either decision into this host layer.

## Verification and current checkpoint

**Proposed verification:** targeted generator/runtime/contract/readiness tests accompany their
functional changes; regenerate when declarations/generators change. Actual MCP/CLI recovery joins
55f at scope end. No performance benefit is established by fewer printed bytes or a successful
transport. All packages are planned, with no new implementation/test evidence from authoring.
