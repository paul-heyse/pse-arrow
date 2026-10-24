# FastMCP usability and tooling evolution

## Recommended direction and review boundary

**Proposed:** evolve the single FastMCP endpoint into a precise, economical working interface
over useful domain operations. Improve the underlying operations where their contracts impede
discovery, expression, selective reading or recovery. Do not make the interface a more elaborate
wrapper around command-shaped inputs and generic result dictionaries.

The current direction is worth retaining: exact domain/workflow navigation, optional tool use,
one attachment, loaded-contract fences, lazy specialists, original scientific/result owners,
and resource/prompt projections for hosts with a fixed callable inventory. The largest next
benefits come from making the next valid request evident, returning a useful small answer,
and keeping unrelated tools usable while source changes. More framework features are useful
only where they improve these complete interactions.

This is the maintainer-authorized **functional-extension review**, combining functional value,
library investigation and design-principle assessment. It is not a diagnostic audit, a completion
audit of Plan52k, implementation authorization or a revision of accepted plans. Previously
delivered component acceptance remains at its recorded boundary. Developer/agent use on this
single-operator workstation is primary; operator and eventual product uses are considered where
they have a concrete connection to that work. No semantic/ranked discovery, separate developer
endpoint, handwritten parallel CLI or Docker deployment is proposed.

The maintainer explicitly selected conditional reconsideration of Code Mode and background
tasks. This review compares them without treating either as already selected. Native execution,
scientific admission, evidence custody and terminal/drain outcomes remain distinct from MCP
presentation, progress, task state or successful transport.

## Source basis and evidence limits

**Interface-checked:** the preparation read [the FastMCP overview](../libraries/fastmcp.md) end to
end; this review used it for breadth and checked consequential details against version-specific
Context7 documentation, tagged upstream source and installed/pinned interfaces. The adopted
versions are FastMCP 4.1.3, MCP SDK 2.3.0 and Pydantic 2.14.1. Rust schema/codec inspection used
the declared Serde, Schemars 1.2.2, serde_json 1.0.151, miette 7.6.0, Tokio 1.53.2 and PyO3
0.29.2 families. Candidate packages below are research dispositions, not installed dependencies.

The initial source baseline was clean commit `1d854ee666f62b2e5ee987d2d202717ed411f64d`.
During review, independent implementation changed validation, native test/fixture and service
owners, including `scripts/native_tests.py`, `scripts/validation.py`,
`scripts/validation_scope.py`, `scripts/coverage_correspondence.py` and
`scripts/surreal_server.py`, plus associated Rust/Python consumers. Those edits were preserved
and were not audited or qualified here. Static conclusions about interface composition refer to
the capability implementation at the initial commit; changing supporting owners and interrupted
live probes retain their narrower evidence. Commits `7e33c1954` and `8f6418dcc` appeared during
review. Near closeout, an uncommitted inventory change also registered new explicit Python
preparation recipes, consistent with O08's existing owner direction. These later changes were
not qualified here; planning must reconcile current delivery before selecting remaining work.
The publication commit is not the inspected implementation baseline.

The assessment standard is Core 3.4, Efficient Architecture Heuristics 1.0 and the pse-arrow
binding selected by `standard.toml`. Process Simulator 1.5 applies where scientific result
interpretation, source identity, reuse or cancellation is exposed. The review does not propose
new numerical methods or reopen the physical model. Blueprint §19.2, §21.1/§21.5 and §23.1/§23.2
remain relevant contract owners. No living diagnostic account was refreshed.

### Actual interface observations

**Tested — bounded read/presentation probes, not product qualification:** the current Codex
session exposes the eleven canonical bootstrap tools. Native calls to `capabilities_reset`,
domain/workflow browsing and exact description succeeded. Description of `workspace_inventory`
reported relevant restart requirements in the already-loaded process. This establishes actual
attachment and honest loaded-state reporting, not fresh execution of its selected operations.

A fresh FastMCP `Client(StdioTransport(...))` used the project-configured `just --quiet
capabilities` launch from `docs/plans`. The fourteen-case sequential probe described below used
only selected reads, presentation and intentional invalid-input controls; it started no test,
solver, database or diagnostic subject. Seven calls returned and seven were refused by the
restart fence after `scripts/surreal_server.py` changed during the probe. The latter did **not**
exercise their intended downstream validation/hidden-call/result behavior and are not passes
for those scenarios. The child connection closed normally. There was no unit-test suite or
whole-product failure count to promote from this probe.

**Measured — representation sizes only:** for that fresh process, the table reports UTF-8
bytes of `json.dumps(structured_content, ensure_ascii=False, separators=(",", ":"))`.
It excludes duplicated text blocks, observation metadata, transport framing, model tokenization
and host truncation. It establishes neither end-to-end latency nor productivity improvement.

| Probe case | Observation |
|---|---|
| Bootstrap listing | Eleven tools; fresh loaded revision `1fe313ac7b94f02fbd546d304ce5d30eaade7cb622fcbe1835467849d6d77994` |
| Describe concept query | 2,021 bytes; `concept`, `identity`, `document`, offset and limit exposed; no supported-concept enum or concept-specific inputs |
| Select four exact context tools | 4,186 bytes; four callable tools and 2,936 bytes of tooling instructions |
| Exact `plan-package` query, coordinator identity ending `#MCP03`, limit one | Returned, `complete=true`, 11,040 bytes |
| Select result reader plus concept query | 48,316 bytes; includes the 43,364-byte qualification guide |
| Select `inspect-plan` | 79,137 bytes; 24 callable operations and six non-root guide bodies, including qualification |
| Get `workflow_inspect-plan` through prompt projection | Returned compact-reference prompt wrapper, 12,599 bytes |
| Reset | Returned, 377 bytes |
| Research-question request, wrong limit type, recipe/task/inventory/result reads and post-reset hidden read | All seven encountered restart refusal after the concurrent service-source edit; their intended inner behaviors remain untested in this run |

The earlier [Plan52k hands-on assessment](../plans/52k-unified-entry-point-and-derived-interfaces.md#post-delivery-attachment-and-hands-on-assessment--2026-10-24)
separately records twenty protocol calls, the authentic frozen single-test result, recipe-owner
misresolution, identity-grammar recovery and a generic hidden-tool error. Its artifacts remain
at `build/capabilities-52k/usability-assessment/`; these observations retain their original
conditions rather than becoming fresh successes here. Plan52k also owns the earlier actual
Codex action journey. No actions were repeated merely to support this review.

## Proposed working model

An agent should navigate exact intent, inspect the applicable contract, construct a valid
request from its schema, receive a useful selected answer and follow an authentic continuation
or recovery reference when necessary. It should not have to infer undocumented string grammars,
decode a large scientific export to discover its structure, read every guide for a broad family,
or guess whether a failed transport permits retry.

Preserve a small set of explicit distinctions: capability versus operation, loaded contract
versus current source, presentation selection versus permission, prepared versus attached/live,
request versus native attempt, answer completeness versus optional detail completeness, and
handle release versus cancellation/reconciliation. These concepts belong to their existing
semantic owners. They do not require a universal workflow engine or a new ontology.

The following opportunities have stable review-local identities. **All resulting changes are
Proposed.** Current-code observations are Interface-checked unless the probe above or the cited
prior evidence explicitly establishes Tested/Measured behavior.

## Opportunities

<a id="O01"></a>
### O01 — Make meaningful requests constructible from the advertised schema

**Value:** an agent can issue its first valid exact query without opening implementation code.
It can express supported questions rather than discover that a valid concept has no usable
argument combination in MCP.

[`operations.py`](../../scripts/capabilities/operations.py) exposes only concept/identity/document
and pagination for `workspace_concepts_query`. Its owner,
[`workspace_domains.query`](../../scripts/workspace_domains.py), additionally supports exact
topics, role, owner, origin, question, selected reference, publication scope and native status.
Research questions require origin/question; evidence-related alternatives need explicit supplied
references. This is a capability-expression gap, not simply missing prose around an enum.

Develop concept-specific typed request alternatives at the native tooling owner, with exact
identity forms, examples and supported facets. Reuse them across MCP and independently useful
native callers. Derive catalogue choices from that authority rather than copying a second enum
into an adapter. Pydantic `Field(description=..., examples=...)`, `Annotated` constraints and
tagged unions can expose host alternatives; FastMCP parses parameter docstrings as well.
[Pydantic schema metadata](https://github.com/pydantic/pydantic/blob/v2.14.1/docs/concepts/json_schema.md)
and [discriminated unions](https://github.com/pydantic/pydantic/blob/v2.14.1/docs/concepts/unions.md)
provide the concrete mechanisms.

Compare improving the present callable with separate operations where concepts genuinely have
different responsibilities. Do not split every enum member into a tool automatically. Derive
scientific request descriptions/examples through Rust registry declarations and Schemars;
do not introduce scientific Pydantic mirrors. Delivery is demonstrated by representative valid
first requests, visible alternatives and malformed/contextually invalid requests reaching the
correct native boundary. AP-04/AP-05 and DP-02/DP-24 are central.

<a id="O02"></a>
### O02 — Return typed, useful selected answers rather than opaque projected bags

**Value:** an agent or program can distinguish a complete small answer, partial optional detail,
native refusal and unavailable evidence without reconstructing meanings inside arbitrary JSON.

Most host readers return `ReadResult` with a generic `dict[str, JsonValue]` payload. Its
[`bounded()`](../../scripts/capabilities/operations.py) walker can omit keys, truncate arrays or
substitute null after the owner has computed the result. Explicit omissions are a useful safety
net, but they cannot make the transformed object conform to an original complete native schema.
Likewise, attaching richer `outputSchema` alone cannot repair an untyped or altered result.

Give recurring host result families meaningful typed responses and explicit complete/partial/
unavailable alternatives where their interpretation differs. Keep mandatory outcome, source cut,
coverage/denominator and continuation facts compact; select explanatory detail explicitly. Generate
the serialization-direction schema of the response actually returned. Preserve native schemas,
units and unknown/nonfinite distinctions through the existing receiving boundary.

Use Pydantic serialization for host-owned views, not to assert that arbitrary `include`/`exclude`
results still satisfy a complete-model schema. Prefer owner-shaped answers to another generic
response format or a universal schema-walking projection engine. Delivery needs independently
expected complete and incomplete answers plus structural checks of actual serialized output,
not only round trips through the same generator. AP-02/AP-04/AP-05; DP-08/DP-21.

<a id="O03"></a>
### O03 — Make instruction delivery and workflow reveal genuinely progressive

**Value:** ordinary selections provide the next useful instructions without repeatedly moving
large unrelated guides into the model context.

[`capabilities_select`](../../scripts/capabilities/server.py) returns guide bodies; its 256 KiB
instruction bound exceeds common useful response sizes. Workflow prompts already use resource
references, but a preceding selection can still return approximately 79 KB and reveal 24
operations for `inspect-plan`. Family expansion pulls in tools and instructions beyond the
immediate inspection question. This is observed response expansion, not a claim that instructions
may be silently dropped or that their relevance can be chosen by similarity.

Prefer a compact selection receipt with exact selected IDs, guide/resource references, revisions,
completeness and clear access instructions. Read complete owned bodies deliberately. Preserve
atomic selection and the ability to obtain all applicable instructions. Separate the workflow's
intent index from activating every operation under a mentioned family; offer explicit relevant
steps or narrower declared selections. Repeated identical selection should not resend all bodies
merely because it is idempotent with respect to visibility.

Compare this with retaining an explicit opt-in full instruction bundle for a consumer that needs
one. Do not rely on raising host token limits, lossy summaries or generic text truncation. Keep
native and tool-projected content consistent. G03's real-agent tasks should assess whether the
new flow actually improves use, including planning-only retrieval. AP-03/AP-06/AP-07; H6/H8/H26.

<a id="O04"></a>
### O04 — Make failures actionable while retaining native recovery authority

**Value:** an agent knows whether to select a tool, correct an argument, restart the attachment,
prepare a dependency or inspect an unsettled native attempt. It does not guess a retry policy.

The earlier hidden-call probe returned a generic invoker error. Host validation in
[`NativeTool.run`](../../scripts/capabilities/native.py) already exposes bounded locations/codes;
[`scientific_error`](../../scripts/capabilities/native_worker.py) already preserves native
classifications and source spans. The gap is uneven fidelity across layers and loss of richer
owner-provided interpretation/recovery data, not universal absence of structured errors.

Define a narrow recoverable diagnostic view at the error's semantic owner: stable code, stage,
safe field/path information, applicable remedy and authentic inspection/settlement references
when that owner has issued them. A hidden selected-operation refusal should explain selection;
a restart refusal should identify the affected contract. Preserve native `DiagnosticReport`
envelope/related/context/recovery information through a deliberately privacy-filtered projection
where useful. Do not echo source values, credentials or unbounded exception messages, and do
not invent a recovery handle when failure happened before native admission.

For structural JSON decode paths, consider `serde_path_to_error` 0.1.20 around the existing Serde
deserializer: structured path segments, original inner error, and serde_json classification/
line/column can provide useful locations. Preserve version-first native admission and call
`Deserializer::end()` after successful decoding to retain trailing-input rejection. These are
decode locations, not invented scientific provenance. [Candidate API](https://docs.rs/serde_path_to_error/0.1.20/serde_path_to_error/).

Retain thiserror/miette and the existing PyO3 report bridge. Compare extending their safe
projection with generic error middleware; the latter does not know native settlement and may
log source-bearing messages. No blanket retries. Delivery includes masked sensitive input,
nested field failure, unsupported version, hidden tool and interrupted/unknown-effect cases.
DP-19/DP-21; PS-10 where scientific results are involved.

<a id="O05"></a>
### O05 — Derive accurate owners and capabilities without guessing module-file paths

**Value:** recipe and operation inspection reliably lead an agent to the declaration it can
change, and a normal extension does not require unrelated catalogue fixes.

[`inventory._leaf_sources`](../../scripts/capabilities/inventory.py) uses textual extraction
and maps dotted modules to `.py`. For `python -m scripts.capabilities`, it guesses the missing
`scripts/capabilities.py`; the real entry is a package with `__main__.py`. Nested module names
can also exceed that extractor's recognized spelling. The earlier runtime recipe observation
exposed the resulting unavailable leaf and session-family association. A conservative source
lead is legitimate, but it should not masquerade as an established executable owner.

Consume explicit operation/module/worker ownership already available at registration and the
structured Just declaration. Resolve known module versus package entry semantics without
importing arbitrary parent modules for discovery. Report unresolved dynamic command structure
honestly. Keep native route classification, actual callable binding and parser coverage separate.
Do not maintain a new exhaustive editable command catalogue to fix this inference.

Use existing provider/registration metadata and generated reference links to make ordinary
extensions local. A provider is useful where it owns a real component source or lifecycle;
replacing a simple static registration loop with a custom provider alone buys little. Delivery
includes module/package/nested module examples and a new capability whose derived reference
and owner links follow its single declaration. AP-01/AP-04/AP-06; DP-01/DP-16/DP-24.

<a id="O06"></a>
### O06 — Push requested detail into original readers and analysis engines

**Value:** a selected failure, gate, source relation or evidence comparison is useful and complete
without first loading and copying an entire report into MCP.

[`run_inspection.product_result`](../../scripts/run_inspection.py) constructs selected subject
views before display pagination; assessment readers have their own broad acquisition patterns.
The generic MCP bound occurs afterward. A smaller response therefore does not establish less
examined work. Some global denominator/ambiguity checks remain necessary even for a small answer.

Extend existing native selectors with explicit answer sections, supported predicates, independent
detail completeness and protected continuation. Keep mandatory semantic populations and source
cuts intact. Reuse Arrow/DataFusion/SurrealDB execution where it already owns the relevant facts;
allow selections to reach that owner rather than filtering serialized JSON afterward. Exact
export remains legitimate. A generic JSON query language over an already curtailed result could
mistake omitted data for absent facts and is not the recommended solution.

This is substantially **already owned** by Plan53e E05 and Plan52f F01 with their actual reader
consumers. Integrate with that work rather than open a competing selective-reader project. Optional
F03's repeated finite questions over one cut remains separately selectable. Delivery should show
an actual complete selected answer from a large protected source, with omitted optional sections
distinguished from an incomplete population. AP-07; H5–H8/H16/H26.

<a id="O07"></a>
### O07 — Discover scientific result shape before exporting data

**Value:** an agent can ask a small physical question about an exact result relation instead of
decoding a wide Arrow export merely to learn its fields.

The current native result route preserves Arrow IPC and exact row windows, an important strength.
Its bounded export can refuse even a single sufficiently wide row, and the request does not
provide column selection. Add a bounded owner-native inquiry into relation/schema, followed by
supported field/row selection at the protected result owner. Preserve exact IPC export as the
programmatic path. An optional small readable physical view must preserve field identity, units,
basis, validity and nonfinite distinctions; JSON normalization is not a substitute.

This is a new scientific-tool usability inquiry, not a claim that the existing IPC contract is
incorrect. Compare owner-native projection with a schema-only inquiry plus the current reader;
do not build an unrestricted query service or per-row scientific RPC loop. Inspect
[`native_worker.read_result`](../../scripts/capabilities/native_worker.py), native result
selection and the Python inspection stream together. Reuse existing Arrow/DataFusion capabilities
only where the result contract permits them. Acceptance requires a real wide typed result and
independent verification of the selected physical answer. PS-01/PS-12; H5/H7/H15/H16.

<a id="O08"></a>
### O08 — Make readiness and preparation independently useful capabilities

**Value:** an agent can distinguish a declared tool, prepared artifact, compatible imported
extension, live selected service and actually exercised operation, then choose the explicit
preparation it needs.

Current prerequisites are mainly prose, and some preparation routes couple synchronization,
extension installation and generated output. Even a pure scoped Rust unit can require the existing
broader native worker preparation; its apparent test purity cannot establish the import closure.

Expose bounded, owner-defined readiness facts and exact preparation routes. Passive descriptions
must stay source-only; explicit live checks disclose their I/O and inspect only selected
dependencies. Distinguish preparation from destructive replacement, source synchronization and
publication. Retain current binary/schema/imported-byte fences and conservative handling of
unknown inputs. A universal health sweep or automatic build on read would undermine the cheap
control surface.

Plan53d D03/I06 already owns independent Python preparation stages; its other preparation/discovery
packages are related consumers. This review supplies MCP interaction requirements, not duplicate
build authority. Delivery includes a source-only task with unavailable science providers and an
explicit selected preparation followed by a real operation. AP-05/AP-06/AP-07; H12/H18/H19/H27.

<a id="O09"></a>
### O09 — Make restart requirements follow actual operation dependencies

**Value:** editing execution or coverage code need not disable an unrelated metadata/result read,
while changed handlers, schemas and actual workers still refuse before effects.

The live and fresh-process refusals demonstrate safe coherence enforcement and a practical
availability cost during concurrent development. [`local_imports`](../../scripts/capabilities/coherence.py)
walks all imports anywhere in a module. An `operations.py` binding inherits sibling lazy-import
closures: workspace tasks reach affected/coverage/validation owners, so a simple inventory read
can inherit those dependencies. Native bindings also import the shared result/adapter module.
Worker inference by a `native_` name prefix includes the native worker even for the source-only
`native_contract_schema` operation. `Interface.common` checks its exact common paths; it does
not itself take their transitive closure, so that is not the cause asserted here.

Separate lightweight shared contract/result definitions from operation composition along genuine
reasons for change. Bind actual fixed-worker dependencies with operation definitions and preserve
source capture/composition validation. Avoid an incomplete handwritten whitelist, function-level
hot reload, automatic restart during effectful work or deletion of fences. Make loaded revision
and per-operation restart status easy to inspect and explain how to reconnect without replay.

Delivery distinguishes an unrelated coverage edit, an actual handler/schema edit, an actual child
worker edit and fresh-endpoint recovery. This is proposed evolution of MCP03, not a silent
revocation of its earlier acceptance. AP-01/AP-06/AP-07; DP-09; H5/H13/H18.

<a id="O10"></a>
### O10 — Report authentic progress and native handles before final return

**Value:** long preparation, execution and drain are distinguishable while the request is still
open; a returned original handle supports a separate watch/read without restarting work.

[`process.run_worker`](../../scripts/capabilities/process.py) currently exchanges final request/
response files and waits. [`rust_tests.execute`](../../scripts/capabilities/rust_tests.py) already
writes an authentic invocation reference when the native runner issues one, before terminal
settlement. A real observation bridge can expose that event and selected native stages while
retaining the existing action and its admission owner.

FastMCP `Context.report_progress(progress, total, message)` can send foreground notifications
when the caller supplies a progress token. Injected `Progress()` outside Docket instead uses
`InMemoryProgress`; adding that dependency alone does not produce wire progress. Use real stage
facts and unknown totals, not invented percentages or elapsed-time guesses. Bound/coalesce
notifications and retain original-reader access when a host does not render them.
[Context source](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/fastmcp_slim/fastmcp/server/context.py),
[progress dependency source](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/fastmcp_slim/fastmcp/server/dependencies.py).

An early handle is not permission to detach an attached action safely. Preserve its signal/drain
contract, distinguish observer cancellation, and advertise only an actually issued owner handle.
Compare this smaller improvement with O16's task protocol. Delivery requires a supporting host
and fallback reader, preparation refusal before handle issuance, and interrupted observation
without fabricated native settlement. Plan52g G01/G02 and 52h remain related assembled owners.

<a id="O11"></a>
### O11 — Add deliberate release and inspection of retained edit previews

**Value:** a long editing session can discard an abandoned or reconciled preview without ending
the endpoint and losing every other handle.

[`Specialists.retain_preview`](../../scripts/capabilities/specialists.py) deliberately never evicts
issued handles and bounds retention at 32 previews/64 MiB. Reset changes presentation; it does
not reclaim previews. Preserve those strong semantics while adding exact-digest release and a
compact retention/status inquiry. Release means future resolution is unavailable, not rollback,
cancellation, successful apply or reconciliation.

The owner must define in-flight behavior: already admitted work retains what it needs to settle;
release cannot invalidate its captured bytes or erase required reconciliation evidence. Compare
explicit release with retaining the current close-only lifetime; do not add silent LRU eviction,
TTL or a persistent preview database by default. Delivery includes capacity recovery for one
explicitly released handle, sibling preservation and admitted-apply/reconciliation behavior.
This is a new functional extension, not an existing Plan52k completion obligation. DP-19/DP-20;
H17/H25/H27.

<a id="O12"></a>
### O12 — Compose recurring questions through meaningful owner-native operations

**Value:** a repeated investigation can obtain a coherent answer without manually joining several
large intermediate MCP replies or independently interpreting their status meanings.

For stable recurring intent, prefer a typed bulk operation such as a selected invocation
investigation or explicit baseline/candidate comparison. Its owner defines source cuts,
correspondence, unavailable members and independently complete sections. Reuse the same kernels
and result readers; do not put scientific or qualification rules into a prompt, middleware or
generic orchestration callback. Independent reads can share immutable preparation where the
owner guarantees it, but one RPC does not imply atomic underlying observations.

Compare a composed operation with the existing small calls and conditional Code Mode. A new
operation needs coherent user intent, not merely a desire to reduce tool count. Do not invent a
general DAG language, automatic next-action agent or cross-store transaction. Plan52f F01 and
52g G01/G02 already contain important consumers; separately selectable F03 remains conditional.
Delivery needs an independently expected composed answer and partial/stale-member cases.
AP-03/AP-04/AP-07; H8/H12/H24/H25.

<a id="O13"></a>
### O13 — Improve reusable instruction packages and exact argument assistance

**Value:** selected instructions can lead directly to relevant supporting material and valid
exact identifiers without requiring a source-directory hunt.

Consider bounded supporting-file manifests/resources for process skills, retaining source hashes,
coherent loaded interpretation and explicit retrieval. FastMCP `SkillProvider` supplies manifests
and supporting resources, but its constructor reads/scans the package and subsequent reads can
observe live files; `SkillsDirectoryProvider(reload=True)` rescans. Those are not drop-in
replacements for the current lazy captured-guide/restart contract. Reuse framework mechanisms
only where their lifecycle fits; neither serving nor downloading a skill activates it.
[Skill provider](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/fastmcp_slim/fastmcp/server/providers/skills/skill_provider.py),
[directory provider](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/fastmcp_slim/fastmcp/server/providers/skills/directory_provider.py).

`@mcp.completion` can assist prompt arguments and resource-template parameters, including exact
IDs for `pse://capabilities/{capability}`. It does not autocomplete ordinary tool arguments.
Completion does not automatically resolve the referenced component or inherit its visibility;
implement the intended exact discovery policy explicitly. Existing argument-free workflow prompts
gain nothing from completion alone. Prefer O01's discoverable request vocabulary for ordinary
tool use. [Completion contracts](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/docs/servers/completions.mdx).

Delivery needs a host that actually uses completion or supporting skill resources and controls
for hidden/unknown references. Defer experimental skills negotiation unless a real consumer needs
it. AP-02/AP-05/AP-06; DP-24.

<a id="O14"></a>
### O14 — Qualify the interface as a working agent contract

**Value:** new operations remain usable and diagnosable as the code changes; tests can detect
misleading schemas, ineffective examples, missing effects and broken recovery routes.

Extend focused tests with independently expected first-valid-request examples, tagged alternatives,
missing/null/empty distinctions, nested validation paths, resource-reference scope and serialization
direction. Adopted `jsonschema.Draft202012Validator` can check published schemas and actual
envelopes; `iter_errors()` exposes paths, schema paths and nested branch context. Use an explicit
`referencing.Registry` only when resource resolution needs it. Schema acceptance is not proof of
scientific eligibility, and optional preflight must preserve native version-first admission.
[Validation errors](https://python-jsonschema.readthedocs.io/en/stable/errors/),
[resource resolution](https://python-jsonschema.readthedocs.io/en/stable/referencing/).

Use actual Codex journeys for discovery→valid request→selected answer→next inspection, with
planning-only, stale, partial and unavailable cases. In-process MCP tests cannot establish host
refresh, notification, task or app rendering. Keep tests proportionate; extending adopted proptest
for relevant Rust properties is preferable to adding a new property-test framework without a
distinct need. Do not derive every expected semantic answer from the schema being tested.

The existing correlation middleware remains valuable. Add only diagnostic facts that answer a
real question: selected operation, serving revision, original handle and where acquisition stopped.
Raw arguments and duplicate telemetry stores are unnecessary. F02's application/database trace
continuity inquiry remains separate from matching correlation labels. Plan52g G03 and 52h own
representative evaluation/assembled qualification. AP-06; DP-21/DP-22/DP-23.

<a id="O15"></a>
### O15 — Conditionally offer bounded, exact-discovery Code Mode

**Potential value:** for an ad hoc investigation, compose several selected immutable reads and
reduce intermediates without returning every intermediate object to the model.

FastMCP 4.1.3 can use `CodeMode(discovery_tools=[ListTools(), GetSchemas()])` or a custom exact
catalogue factory rather than its default ranked search. However, discovery filtering is not
execution eligibility. The execution callback refreshes the visible catalogue and delegates to
normal `ctx.fastmcp.call_tool`; the present visible catalogue includes bootstrap action invokers.
A read-only composition route must constrain actual execution, including indirect calls, rather
than simply hide actions from its discovery tool.

Do not install the transform globally by default: it replaces bootstrap listings as well, its
synthetic execute tool has no automatic effect annotations, and provider placement alone is not
established isolation. Preserve exact navigation, truthful host annotations, original source cuts,
native result meaning, per-call cancellation and limits on callback count, live results and output.
The documented sandbox computation timeout excludes host callback wait time, so it is not a total
workflow deadline. An unhandled failure interrupts the chain; sandbox code may catch tool-call
errors, and earlier effects are not rolled back.

**Disposition:** a bounded read-only feasibility inquiry after O01/O02/O06, compared with a typed
bulk read for recurring intent. Require an actual consumer advantage, execution-eligibility tests
and real-host support before selection. No measured context or latency benefit is claimed.
This may require an explicit authority decision for a model-authored composition capability;
it cannot be smuggled in as ordinary exact invocation.
[Code Mode documentation](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/docs/servers/transforms/code-mode.mdx),
[catalogue behavior](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/fastmcp_slim/fastmcp/server/transforms/catalog.py).

<a id="O16"></a>
### O16 — Reconsider task protocol only for a real detach/reconnect need

**Potential value:** a compatible host can initiate a long operation, stop waiting, and later
poll its status/result without losing its native ownership or replaying effects.

`fastmcp-tasks`/`TasksExtension` adds Docket execution infrastructure for asynchronous tools with
modern protocol/task-capability negotiation. An in-memory backend avoids
an external service but remains ephemeral and process-local; it does not establish durability
after endpoint shutdown. Redis/Valkey is additional operational machinery, not a prerequisite
for making ordinary progress understandable.

Crucially, task-optional inner calls through `server.call_tool` run foreground; task-required
inner calls refuse. The present `capabilities_invoke_action` and Code Mode use inner dispatch.
Adding `task=True` to an underlying native tool alone does not task-enable those invocation paths.
The outer protocol tool, negotiated host support and native lifecycle mapping would need explicit
design. Task completion/cancellation is not scientific assurance, rollback or native drain.
[Tasks documentation](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/docs/servers/tasks.mdx),
[catalogue-transform contract tests](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/tests/tasks/server/test_task_catalog_transforms.py).

**Disposition:** defer default adoption. First deliver O10 and compare supported native
start/handle/watch/result lifetimes. Revisit for a demonstrated detach/reconnect consumer that
those routes cannot adequately serve. Test the actual wrapper path, host negotiation, endpoint
loss, idempotency and owned drain before claiming value. No second scheduler/result authority or
scheduler-wide retry policy is recommended. DP-19/DP-20; H25/H27.

<a id="O17"></a>
### O17 — Use structured operator interaction where it improves a real decision

**Potential value:** the sole operator can inspect a selected failure table, exact comparison,
edit preview or physical result without reading a large JSON/prose dump, and supply an explicit
choice before an action.

Compare ordinary typed parameters/resources with parameterized prompts, elicitation and
FastMCPApp/Prefab tables/forms/charts. Reuse the same owner-native operations and selection
contracts; display state and UI visibility are not execution authority. Prefer fixed typed
components to generative UI when the intended interaction is already known. No frontend,
HTTP service, OAuth system or enterprise gateway is justified solely by framework availability.
Actual host rendering is a separate prerequisite; an ordinary successful tool call cannot prove
MCP Apps support. [Apps mechanisms](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/docs/apps/overview.mdx).

Modern elicitation uses `InputRequiredResult`: the invocation ends, and answers cause a fresh
call through the normal pipeline. Gather missing choices before native effects, preserve original
request identity where required, and do not repeat an effectful prefix on re-entry. Ordinary
required parameters remain the simplest general route. Do not add confirmation ceremonies to
already-authorized work. [Elicitation semantics](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/docs/servers/elicitation.mdx).

**Disposition:** conditional operator enhancement, related to Plan52g G05's existing inquiry.
Select a concrete decision and compare a readable artifact with an interactive view; retain the
ordinary agent/programmatic path and original physical/evidence interpretation. AP-02/AP-03;
PS-12 where scientific data is displayed.

## Library and mechanism decisions

These are proposed review dispositions, not manifest edits or a package-adoption quota.

| Mechanism | Recommended role and important limit |
|---|---|
| FastMCP tools, native resources/prompts and their tool projections | Retain one implementation/dispatch path; improve contracts and progressive content first. |
| Pydantic 2.14.1 | Host request alternatives, descriptions/examples, reusable constraints and actual response schemas. No scientific mirror, experimental partial acceptance or custom core-schema machinery without a concrete need. |
| Serde/Schemars 1.2.2 | Enrich the Rust/registry-owned schema authority. Doc comments, `description`, `example`, Serde tags and separate input/output generation are already fitting mechanisms; edit generators/declarations, never generated files. [Derive API](https://docs.rs/schemars/1.2.2/schemars/derive.JsonSchema.html). |
| serde_path_to_error 0.1.20 | Focused new Rust candidate for O04. Verify custom/untagged decoder limitations and preserve original version/trailing-input semantics before adoption. |
| thiserror/miette and current PyO3 diagnostic bridge | Extend useful structured projection rather than add another error framework. Native messages and context still need deliberate redaction. |
| jsonschema / referencing | Use adopted validation for independent contract checks. Direct `referencing` use would require an exact direct dependency declaration; a transitive installation is not that declaration. |
| Standard-library context managers / asyncio; AnyIO | Prefer current adequate primitives. `AsyncExitStack` or structured sibling settlement can help scoped lifetimes; adding AnyIO solely to rewrite working asyncio is not justified. Cleanup policy and native drain remain owned semantics. |
| Tokio/tokio-util; tracing/OpenTelemetry | Reuse native cancellation/observation mechanisms where the seam needs them. `spawn_blocking` cannot abort already-running work; a span is not a settlement record. Preserve F02's unresolved propagation boundary. |
| Adopted proptest and current MCP test clients | Focused codec, path, selection and sequence properties. Hypothesis/hypothesis-jsonschema remain possible for a distinct Python need; no new framework solely to duplicate existing schema cases. |
| Arrow/DataFusion/SurrealDB | Native selective physical/evidence answers under current owners, not a new generic query or result store. |
| FastMCP providers / `Depends` / async generator dependencies | Use for independently owned acquisition/lifecycle or real dynamic sourcing. Static ordinary modules can remain simpler. Provider-bundled extensions do not justify implicitly starting task infrastructure. |
| Global response limiting | Do not select: it removes targeted advertised output schemas and can replace an oversized result with truncated text after computation. It does not bound upstream work and loses structured content/media. [Exact implementation](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/fastmcp_slim/fastmcp/server/middleware/response_limiting.py). |
| Generic retry/cache/error middleware | No global adoption. Cache identity, native settlement, source freshness and privacy require operation-specific semantics. A duplicate cache or retry manager would need a concrete benefit. |
| JMESPath/jq/new query DSL; gRPC/another MCP server | No demonstrated need. Improve typed owner operations and the existing PyO3/JSON schema route first; a different service boundary needs a separate concrete lifecycle/deployment benefit. |
| Generated Cyclopts CLI, HTTP/authentication and remote federation | Keep the existing concrete-consumer revisit decisions. This review establishes no new consumer requiring them and does not reopen Docker. |

The [FastMCP overview](../libraries/fastmcp.md), [Pydantic overview](../libraries/pydantic.md),
tagged sources cited above, and current contracts supply concrete planning leads. A recommendation
that needs a dependency change still uses exact pins and targeted affected checks if selected.
No library or licence was excluded categorically.

## Priorities, interactions and current-plan correspondence

Start with O01–O05 and O09: first-valid requests, trustworthy typed answers, compact guidance,
recoverable errors, accurate owner discovery and dependency-aligned availability improve nearly
every workflow. O06/O08 are important underlying prerequisites already substantially scheduled;
consume and refine their existing owners rather than create competing implementations. O10/O11
make long execution and long editing sessions more usable. O07 merits a bounded result-owner
design inquiry before any new scientific readable view. O12/O13/O14 improve composition,
instruction packages and evolution around those contracts. O15–O17 remain explicitly conditional.

These are dependency/benefit judgments, not a requirement to finish an entire foundation project
before delivering a useful operation. A small query or error improvement can land with its actual
consumer. Conversely, a new task/code execution framework cannot compensate for inadequate
operation semantics or owner-selected result detail.

| Existing owner | Relationship to this review |
|---|---|
| Plan52i CAP / Plan52j ADP / completed Plan52k MCP | Preserve delivered discovery, native schemas, adaptation, lazy acquisition and component evidence. O01–O05/O09/O13 propose evolution, not automatic reopening or completion rescoring. |
| Plan52f F01; optional F03 | O06/O12 consume native selective evidence/comparison work. F03 remains separately selectable; no implicit adoption through Code Mode. |
| Plan52f F02/I11 | O10/O14 use interface/native observation without claiming unresolved application→database trace continuity. |
| Plan52g G01/G02/G03/G05 and Plan52h | Compose useful interactions, assess actual agent behavior, evaluate optional operator views and retain assembled qualification. Component/source-only probes here do not close them. |
| Plan53d D01–D04/I06 | Preparation/discovery/source-capture and independent Python stage work underlies O08 and parts of O05/O06. Concurrent implementation is not reviewed as complete here. |
| Plan53e E01–E05 | Native identities, settlement, custody and protected selective readers constrain O04/O06/O10. MCP cannot reinterpret their outcomes. |
| Plan53f and 53g | Execution requirements/exclusion and assembled test assurance remain distinct from FastMCP annotations and transport/task status. Shared runs retain separate assertions. |
| Plan52 I14 endpoint-shutdown inquiry | Keep its existing owner. Independent sibling cleanup is relevant to provider/dependency refactoring, but this review neither silently resolves the inquiry nor imports test-fixture settlement policy. |

### Decision and rule implications

No rule change is accepted by this review. The following decisions should be explicit if selected:

- **RC01 — Scoped interface contract evolution:** accept revised request/result/error and
  progressive-instruction shapes together with actual callers and derived reference. Preserve
  native scientific schema ownership; public boundary/diagnostic changes take the normal decision
  route. Internal host tooling changes need no placeholder ADR.
- **RC02 — Optional model-authored read composition:** O15 may extend the existing fixed-operation
  interface with constrained code composition. Select its actual execution policy and host effect
  contract explicitly; exact discovery alone is not this decision.
- **RC03 — Optional protocol task lifecycle:** O16 would change how the host initiates/observes
  work. Select actual outer dispatch, native ownership and loss/reconnect semantics before adding
  infrastructure. Existing foreground behavior remains the comparison baseline.

O11's explicit preview release also needs a clear owner-local lifetime contract; it is not an
instruction to add a general lease system. Richer operator interfaces remain conditional consumer
choices. None changes presentation selection into authority, introduces semantic search or
authorizes production repair through a favorable review.

## Design-principle assessment and delivery evidence

| Foundation | Assessment of the proposed realization |
|---|---|
| AP-01 — Separate reasons for change | Operation-shaped module/worker dependencies and owner-native diagnostics can localize changes. A generic provider or contract framework must demonstrate a real responsibility rather than just relocate registration. |
| AP-02 — Meaningful stable contracts | Typed request/result alternatives and explicit evolution improve replaceability. Native scientific meaning, source protection and interpretation remain constraints; matching JSON shape alone is insufficient. |
| AP-03 — Composition | Useful typed bulk questions and carefully bounded Code Mode can compose existing capabilities. Prompts, tasks and middleware must not independently reconstruct execution or scientific policy. |
| AP-04 — Domain model and authority | The main opportunity is to expose and improve actual query, selection, readiness, outcome and lifecycle models. A catalogue plus opaque payload bags is insufficient where callers must reason about hidden distinctions. |
| AP-05 — Explicit structure | Schema metadata, alternatives, truthful effects, source cuts and actionable errors make valid use inspectable. Hints/visibility/completion do not independently enforce admission. |
| AP-06 — Local reasoning and testability | Narrow dependencies and independent examples support focused changes and source-only operation. Actual host tests remain necessary for features implemented by the host rather than the server. |
| AP-07 — Execution fit | Select requested sections before acquisition, reuse immutable preparation where valid and move compact results. More pages, one RPC, response trimming or a sandbox does not establish reduced examined work or total cost. |

Scientific assessment remains separate: preserve PS-01/PS-12 physical interpretation,
PS-10 truthful outcomes and PS-11 actual source/preparation/reuse dependencies. No new solver,
physical validity or whole-system G9 acceptance is claimed. The current strengths and the
identified opportunities can coexist; a working endpoint does not establish optimal usability.

Useful delivery evidence, if opportunities are selected, includes:

1. An unfamiliar agent constructs representative valid requests from schemas/examples and gets
   precise recovery from invalid concept/identity/visibility cases without source spelunking.
2. A selected large result returns a useful complete small answer; optional details are independently
   bounded and protected. Wide scientific fields preserve exact physical/schema meaning.
3. Workflow selection supplies a compact attributable instruction index; complete bodies remain
   deliberately accessible, and planning-only retrieval executes no implementation.
4. A changed unrelated owner leaves an unaffected operation usable, while actual changed handlers,
   schemas and workers refuse before effects and recover through a fresh compatible attachment.
5. Preparation, execution and drain are visibly distinct; an early handle is authentic and a lost
   observer cannot turn a native unknown effect into a retry-safe outcome.
6. Explicit preview release recovers selected capacity without evicting siblings or changing an
   admitted edit's settlement/reconciliation obligations.
7. Conditional Code Mode/tasks/apps demonstrate actual host behavior, original native outcomes,
   and a material consumer advantage over the simpler existing route before adoption.

No numerical usability score, benchmark platform, broad product qualification or mandatory
library quota follows. The next authorized planning step can translate selected opportunities
into existing or new plan scope with these mechanisms and limits intact. This review itself
changes only the review publication and its navigation entry.
