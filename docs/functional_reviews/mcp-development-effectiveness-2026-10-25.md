# MCP for effective codebase development

## Recommendation and scope

**Proposed:** extend the single canonical FastMCP interface around complete development
questions: understand a change, find its authoritative producer, inspect its scientific meaning,
choose appropriate validation, perform a controlled edit, and explain the resulting execution.
Provide a faithful Cyclopts-derived CLI as an independent client when Codex's MCP attachment
is unavailable or stale. Preserve direct native commands and source/editor tools as useful
first-class alternatives.

The most valuable work is a combination of exposing existing native capabilities and improving
their underlying contracts. Another general server, an unrestricted command runner, semantic
search, or a wholesale Rust MCP rewrite would not solve the demonstrated problems. A generated
CLI is worthwhile, but the stock FastMCP 4.1.3 generator is not a ready-to-use recovery interface
for this endpoint: targeted probes found argument-presence and connection-selection limitations.
Endpoint-owned edit previews introduce a further lifetime constraint.

This is the authorized **functional-extension review**, with library-informed design assessment.
Codex-driven development on the maintainer's workstation is primary; production management and
eventual agent consumers are secondary. The inquiry includes tooling beyond Plans47/50/51/52.
The supplied libraries broadened exploration rather than fixing an adoption list. This review
does not reopen a diagnostic audit, change accepted plans, select new implementation, or claim
that proposed capabilities are delivered. The completed [Plan52 baseline](../plans/README.md)
is the starting point, including its concrete-consumer conditions for deferred mechanisms.

## Evidence and current foundation

The inspected source baseline is clean commit `bbdb3c32cbb120ab308647525ef5ae27e5610553`.
Only this review and its navigation entry were authored. Library research, workflow mapping and
independent architecture assessment contributed to one integrated judgment. No database service,
compiler, scientific workload, test campaign, telemetry deployment or configuration change was
started. Read-only client probes used disposable source fixtures and temporary outputs.

The assessment applies Core 3.4, Efficient Architecture Heuristics 1.0, Test Assessment 1.2,
Process Simulator 1.5 and the pse-arrow binding selected by
[`standard.toml`](../design_review/design_principles/standard.toml). Relevant architectural owners
include blueprint §19.2, §21.1/§21.5, §23.1/§23.2 and §24.2. The principles shape realizations:
one owner per meaning, structured conceptual models, selective work at the performing owner,
appropriate library reuse, honest evidence and explicit lifetimes. Scientific inspection must
preserve physical context, admission and truthful outcomes; successful transport is insufficient.

### Library coverage and verification depth

The following supplied documents were read end to end. Consequential mechanisms were checked
through scoped Context7 queries, followed by pinned/tagged upstream or installed source where
version and behavior mattered. Generic or older FastMCP snippets were not treated as 4.1.3 proof.

| Reference | Coverage and consequence |
|---|---|
| [FastMCP](../libraries/fastmcp.md) | Full document; providers/visibility, strict dispatch, client lifetime, resources/prompts, CLI generation, telemetry, tasks/CodeMode and Apps. Installed 4.1.3 generator and actual clients were inspected/probed. |
| [Pydantic](../libraries/pydantic.md) | Additional full read; 2.14.1 directional schemas, discriminated contracts, explicit missing/null/default semantics, examples and reusable adapters. These apply to host contracts, not replacement of native scientific codecs. |
| [SurrealDB tooling](../libraries/surrealdb_tooling.md) | Full document; query plans, graph/index investigation, service and development tooling; current owners and 3.3.4 parser/source checked. |
| [SurrealDB](../libraries/surrealdb.md) | Full document; structured records, attributed graph relationships, exact predicates, lexical filtering, query explanation and explicit lifecycle. |
| [SurrealKV](../libraries/SurrealKV.md) | Full document; standalone temporal KV, transactions, checkpoints and diagnostics assessed against the existing database authority. No additional store recommended. |
| [SurrealMX](../libraries/SurrealMX.md) | Full document; transactions, SSI, persistence, reader retention and statistics. Current patched MX 1.0 integration distinguished from upstream 0.27 assumptions. |
| [Serde, serde_json and Schemars](https://github.com/paul-heyse/pse-arrow/blob/24852b36d2552709a8d304faf15e24157e2abdda/docs/libraries/Serde_serde_json_and_Schemars.md) | Full document; generated directional scientific contracts, native path errors, schema examples and typed result inspection. |
| [rmcp](../libraries/rmcp.md) | Full document; typed JSON, context/cancellation, protocol and transport composition. Versioned 3.7.0 APIs assessed; rmcp is not a repository dependency. |
| [OpenTelemetry Rust](../libraries/OpenTelemetry_Rust.md) | Full document; propagation, tracing bridge, metrics, links and observation limits; current Rust 0.33.1 / tracing-opentelemetry 0.34.0 source checked. |
| [OpenTelemetry Python](../libraries/OpenTelemetry_Python.md) | Full document; SDK 1.45.1, metric views/cardinality, trace links and existing query/retention owners. |

### What is already available

The current endpoint has eleven bootstrap tools and exact domain/workflow/capability selection.
Fixed invokers dispatch through `server.call_tool`, preserving visibility, validation,
middleware, source fences and native admission. Catalogue membership and selected presentation
derive from existing declarations. A frozen Codex tool inventory already has a supported route;
it does not alone justify another interface.

| Workflow | Existing owner or callable behavior | Remaining useful increment |
|---|---|---|
| Understand/edit a task | `workspace_task_context`, domain/concept/source readers, Git rationale, producer triples and protected paths | Attributed cross-boundary impact/evidence paths; stronger producer facts where currently unknown |
| Edit symbols safely | Language navigation, rename/action preview, apply, reconcile, status and release | Selected structural migrations; explicit lifetime handling for a CLI |
| Select/run validation | Assessment selection/run, integration views, Rust test description/run, compilation and contract generation/checks | Compose the existing affected-test closure and explain the chosen population/unknowns |
| Inspect scientific work | Native admission, retained revisions, preparation discovery, result schema/value inspection | Model declaration/comparison/inspection adaptation and native nested/context paging |
| Investigate execution | Original run/assessment readers, workload watch, build observation, telemetry health/query, captures and retained selections | Intent-focused explanation and instrument-specific filters/grouping |
| Research implementation | Exact source concepts, `knowledge_source_corpus`, `library_source_packet`; native `just knowledge query/explain` | Expose indexed questions without implicit import, refresh or relevance ranking |
| Reproduce failures | Existing experiment isolation, Rust archive/inspection/replay and regression experiment owners | Carefully selected comparison/bisect operations, preserving original restrictions |

Relevant sources include [the capability catalogue](https://github.com/paul-heyse/pse-arrow/blob/e0a7fe7ea21976f5edef43244e4c04983be2d3e8/scripts/capabilities/catalogue.py),
[inventory and retention dispositions](https://github.com/paul-heyse/pse-arrow/blob/e0a7fe7ea21976f5edef43244e4c04983be2d3e8/scripts/capabilities/inventory.py),
[composition](https://github.com/paul-heyse/pse-arrow/blob/e0a7fe7ea21976f5edef43244e4c04983be2d3e8/scripts/capabilities/server.py), [source coherence](https://github.com/paul-heyse/pse-arrow/blob/e0a7fe7ea21976f5edef43244e4c04983be2d3e8/scripts/capabilities/coherence.py),
[workspace task context](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/workspace_tasks.py) and
[native bindings](https://github.com/paul-heyse/pse-arrow/blob/e0a7fe7ea21976f5edef43244e4c04983be2d3e8/scripts/capabilities/native_contracts.py).
An inventory classification of a native route is not evidence that it is callable through MCP.

### Targeted observations

**Tested — bounded interface and disposable-fixture probes:** the attached Codex session's
`capabilities_domains` succeeded but reported `restart_required=true`, loaded revision
`99cf47f29c385f45b275aef49821f8d4ff3535cd49f186d5c0bf835c18481a82` and six changed interface
files. This is demonstrated stale attachment, **not a reproduced transport disconnection**.
A fresh STDIO client against current source reported revision
`7db3aaad83f6c1e6f8e9ed1ea3dece34e30dd7c64252a1e9dce3fd7e76e66e8a`, no restart requirement,
eleven bootstrap tools, successful description/selection and a successful fixed-invoker
`workspace_task_context` read for `scripts/capabilities/server.py`.

| Probe | Expected question and observed result |
|---|---|
| Fresh generic `fastmcp call` using the canonical STDIO configuration | Reached current `capabilities_domains`, exit 0, independently of the stale Codex attachment. |
| New client after closing the selecting client | Invocation without selection returned `pse.capabilities.unknown_or_hidden_selection`, stage `host_selection`, with the exact browse/describe/select remedy. Selection is not durable across these connections. |
| Stock `fastmcp generate-cli` against the unselected endpoint | Generated eleven visible bootstrap commands. `workspace_task_context` was absent, as expected under progressive visibility. Generated bootstrap invocation succeeded. |
| Generator given the actual selected `workspace_task_context` tool | Its generated command existed, but a fresh invocation returned the same hidden-selection refusal, exit 1. Merely generating a larger command inventory does not establish selection on later connections. |
| Direct JSON call to a disposable tool with optional nullable/defaulted scalar and list | Explicit `{"value":null,"items":[]}` arrived unchanged, exit 0. This is the forwarding control. |
| Generated Cyclopts command for that same request | Exit 0 but returned `{"value":"DEFAULT","items":["DEFAULT"]}`. Separate null-only and empty-only controls preserved the other supplied value and lost the selected null/empty value. Successful exit did not establish argument fidelity. |
| Generated tool help with the disposable server file temporarily unavailable | Help succeeded, exit 0. Actual invocation failed, exit 1, because the generated client still needed its endpoint. The fixture was restored. |

The generator observations are explained by installed
`fastmcp/cli/generate.py::generate_cli_command`, `generate_cli_script` and the emitted
`_call_tool`: discovery uses `client.list_tools()`; invocation creates a new `Client`; argument
filtering removes both `None` and empty lists. Its `_print_tool_result` also returns immediately
after printing structured content. That last fact is **Interface-checked**, not a demonstrated
loss of a particular production result: CLI planning must determine which remaining content,
metadata and original-owner references are contractually meaningful and preserve them.

Three initial CLI attempts used the incorrectly guessed command spelling `patch-probe` rather
than generated `call-tool patch_probe`. Those exit-2 setup errors did not reach the intended
server and are not library failures. The corrected seven-command follow-up and selected-tool
probe establish the observations above. There were seventeen subprocess cases in total: nine
initial, seven corrected/control cases and one selected-tool invocation. Expected negative cases
are not failing product tests; exit-0 argument changes are the demonstrated fidelity failures.
There was no product-suite run or productivity/performance measurement.

Raw scripts, separated stdout/stderr, JSON captures and generated clients are retained locally at
`/tmp/pse-mcp-review-20261025-r6pix8qh`. They are temporary review evidence, not a required runtime
dependency or a new evidence store. The essential reproduction is recorded below so conclusions
do not depend on retention of that directory.

```python
# Disposable echo.py, served only for the review.
from fastmcp import FastMCP
mcp = FastMCP("review-only-fidelity")
@mcp.tool()
def patch_probe(value: str | None = "DEFAULT",
                items: list[str] | None = ["DEFAULT"]) -> dict:
    return {"value": value, "items": items}
if __name__ == "__main__":
    mcp.run(transport="stdio", show_banner=False)
```

Run with the checkout's pinned environment, substituting an owned temporary directory:

```bash
scripts/pse-env -- .venv/bin/fastmcp call /tmp/owned-probe/echo.py patch_probe --input-json '{"value":null,"items":[]}' --json
scripts/pse-env -- .venv/bin/fastmcp generate-cli /tmp/owned-probe/echo.py /tmp/owned-probe/echo_cli.py --no-skill
scripts/pse-env -- .venv/bin/python /tmp/owned-probe/echo_cli.py call-tool patch_probe --value null --items '[]'
```

The reference oracle is the explicit JSON request itself. For canonical recovery, the tested
MCPConfig selected the checkout `.venv/bin/python -m scripts.capabilities`, with explicit checkout
`cwd` and `OTEL_SDK_DISABLED=true` to avoid exporting review observations. It was a temporary
client configuration, not an edit to Codex's configuration. A fresh `Client(StdioTransport(...))`
described and selected the exact capability before invoking it in the same context. To reproduce
the generated selected-command limitation, pass that actual selected MCP Tool to upstream
`generate_cli_script`, close the selecting client, then invoke the emitted command. No mutation
tool or native extension rebuild is required for these controls.

## Opportunities

All O01–O16 are **Proposed**. Current-source and library interface checks support feasibility;
only the bounded observations above and explicitly attributed prior evidence are **Tested**.
Representative interactions below illustrate value and delivery demonstrations, not already
accepted command names, complete schemas or implementation packages.

<a id="O01"></a>
### O01 — A faithful derived CLI for independent capability access

When Codex's attachment is stale or unavailable, an agent using the shell should be able to
describe an exact capability and invoke it against the selected checkout's current source and
environment. This now supplies a concrete consumer for reconsidering the earlier Q05 CLI
deferral. It does not imply another developer server or independently authored operation set.

Derive command names, descriptions, input schemas, effect classification and result handling
from the same capability declarations and native directional schemas. Use FastMCP's
`generate-cli`/Cyclopts machinery for CLI presentation, with a small owned adaptation for this
endpoint's selection and fidelity requirements. Prefer an upstream generator correction where
appropriate; pin any necessary local generator adaptation and test its emitted behavior. Do not
patch installed/generated files by hand or duplicate business logic in CLI functions.

The adapter must preserve omitted versus explicit null/empty/default values, nested unions,
aliases, exact-width values and typed refusals. Pydantic's `model_fields_set`, explicit missing
semantics and directional schema modes help distinguish presence; they do not repair a lossy
argv filter automatically. Offer lossless JSON/file input and machine-readable results alongside
friendly flags. Preserve error codes, source revision, observation limits and original native
references instead of treating pretty printed structured content as the whole protocol result.

For each stateless command, connect, establish the exact selection and invoke within one Client
context through the canonical dispatch. The CLI may derive a complete command index from the
catalogue, but runtime visibility/admission still applies. Resolve checkout/interpreter/config at
invocation; a generated file containing this probe's absolute paths is not a portable deployment.
Static help may remain available without a running endpoint and must identify its schema basis.
Before effects, compare the generated command's contract fingerprint with the current described
wire contract; refuse with a regeneration route on incompatibility. A stale CLI's old defaults or
aliases can remain schema-valid while encoding old meaning, so server validation/source fencing
alone cannot establish client-contract compatibility. Reuse catalogue wire identity/execution
basis rather than another version registry. Preserve omitted arguments instead of automatically
sending baked defaults; raw JSON input must preserve presence too.

**Alternative and burden:** generic `fastmcp call` already reaches the endpoint for bootstrap
reads and is a useful immediate diagnostic. It lacks the complete typed, selected workflow by
itself. Direct native commands remain the fallback when the endpoint cannot start. Maintaining
one schema-derived client is preferable to a parallel handwritten CLI or universal RPC router.

**Delivery demonstration:** the actual shell-using agent obtains help, selects/invokes a hidden
read on fresh current source, preserves all presence controls, receives an authentic refusal and
reads an original durable invocation reference after connection loss. O03 limits edit workflows.
Sources: FastMCP overview §8.1/§11.5/§14.1; Pydantic §3.4/§7.1;
[FastMCP 4.1.3 generator](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/src/fastmcp/cli/generate.py).

<a id="O02"></a>
### O02 — Diagnose availability and recover the right layer

An agent should quickly distinguish a stale loaded interface, missing host attachment, broken
server startup, unavailable native prerequisite and an action whose acknowledgement was lost.
Today source fences, bootstrap reads, typed readiness, observation metadata and original native
readers already supply many facts. The extension is a concise independent diagnosis and an
accurate next route, including when the MCP connection itself cannot answer.

Reuse the fixed launcher and existing environment explanation to report configuration path,
selected checkout, interpreter, loaded/current interface basis and the stage actually reached.
Retain stdout as protocol/output and stderr as launch diagnostics. Bound any fresh handshake
probe and classify it as a new process/connection, not repair of the old one. Passive diagnosis
must not bootstrap packages, prepare native libraries, start database/telemetry services or
reload an executing endpoint. Do not print environment secrets to explain an environment hash.

[Official OpenAI app-server documentation](https://learn.chatgpt.com/docs/app-server#api-overview)
describes `config/mcpServer/reload`, `mcpServerStatus/list` and startup-status notifications.
These are concrete host-integration leads: determine whether the installed Codex frontend
exposes an appropriate reload/refresh route and test it there. Their existence is not evidence
that this session can call them or that a running effect can be safely restarted. The review
did not reload Codex, change host settings or establish automatic in-session reconnection.

Recommend four explicit outcomes: fresh CLI for stale/disconnected attachment; original launch
diagnosis/direct native commands for broken endpoint startup; explicit prerequisite preparation
for native unavailability; original status/result/reconciliation reads for lost action ACK.
Never replay an effect merely because transport failed. Keep source fences; automatic module
hot reload would weaken their meaning and complicate retained owners.

**Delivery demonstration:** representative stale-source, missing attachment, missing executable
and lost-ACK controls return distinct stages/remedies without duplicate work. Changing a domain
owner does not require preparing unrelated domains. No additional always-on broker is justified
for the current single-operator case. Sources: [coherence](https://github.com/paul-heyse/pse-arrow/blob/e0a7fe7ea21976f5edef43244e4c04983be2d3e8/scripts/capabilities/coherence.py),
[readiness](https://github.com/paul-heyse/pse-arrow/blob/e0a7fe7ea21976f5edef43244e4c04983be2d3e8/scripts/capabilities/readiness.py),
[environment guide](../dev/agent-environment.md) and FastMCP §8.1/§11.3.

<a id="O03"></a>
### O03 — Lifetime-accurate edit workflows and reconnection

CLI recovery must state which references survive the connection. Current
[`Specialists`](https://github.com/paul-heyse/pse-arrow/blob/e0a7fe7ea21976f5edef43244e4c04983be2d3e8/scripts/capabilities/specialists.py) retains immutable edit previews in
an endpoint-owned map; its summaries already say reconnecting creates a new owner, and close
clears that retention. Native invocation references read through
[`rust_references.py`](https://github.com/paul-heyse/pse-arrow/blob/e0a7fe7ea21976f5edef43244e4c04983be2d3e8/scripts/capabilities/rust_references.py) instead resolve original
registry-owned work. A one-process-per-command CLI cannot transparently carry an edit preview
from preview to apply just by reusing the printed handle.

Initially support stateless operations and durable original-owner reads, and explicitly explain
endpoint-bound exclusions. If CLI editing is selected, compare a foreground connection-scoped
session that the actual agent can drive with saved-preview export/adoption at the existing edit
owner. The former preserves current semantics but needs a useful interaction and loss behavior;
it is not automatically a detachable daemon. The latter enables cross-command work but changes
the preview contract and deserves its own design decision.

There is native groundwork: [semantic navigation](https://github.com/paul-heyse/pse-arrow/blob/e0a7fe7ea21976f5edef43244e4c04983be2d3e8/scripts/semantic_navigation.py) and
[structural migrations](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/structural_migrations.py) load saved previews through
[`semantic_edits.py`](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/semantic_edits.py), which validates complete content, basis,
digests and edits, rechecks source currency and writes reconciliation state before mutation.
Reuse it if adopting previews. Bind checkout, immutable contents, source basis, protection rules,
expiry/retention and interruption obligations. A file hash is neither authorization nor proof
that an earlier apply did not happen. Re-adoption must not erase unresolved prior mutation.

**Delivery demonstration:** preview/reconnect/apply is either explicitly unsupported with a
useful route, or succeeds through a qualified retained/adopted owner. Stale source, altered saved
preview and interrupted apply remain distinct. No universal persistent session store is needed
to deliver O01's immediate read/recovery value.

<a id="O04"></a>
### O04 — Explain an intended invocation before effects

Before selecting a native check or model inspection, Codex should be able to ask: “What will
this request do in this checkout, what must already exist, and which facts remain unknown?”
Current descriptions, `Capability.requirements`, readiness observations, preparation routes
and execution-basis contracts supply the parts. Compose them around the **actual typed request**
when a domain can explain more than generic help.

Return operation/effect kind, current source/schema basis, passive prerequisite observations,
explicit preparation choices, selected population and original readers for eventual results.
Differentiate describing, validating input, estimating selection and actually admitting work.
Do not market this as universal dry run: model admission, provider preparation or contextual
scientific checks may require effects. Do not call a missing service a misconfiguration when
the selected read does not require it. Reuse existing descriptions/resources/prompts rather
than another capability catalogue or a speculative universal planner.

**Value and burden:** this reduces repeated manual interpretation of flags, feature sets and
environment identities without inventing a new work scheduler. Begin with one important native
validation or model-development route. Generated typed request variants and Pydantic
discriminators keep the explanation structurally aligned; FastMCP resources/prompts can guide
the sequence without adding another public server. A selected operation's valid input schema
does not prove its runtime requirements are met.

**Delivery demonstration:** the same request can be explained without preparing anything and
then run through its ordinary owner; source changes invalidate the explanation's applicability,
and unexplored admission conditions stay unknown. Sources: capability model/catalogue,
[readiness](https://github.com/paul-heyse/pse-arrow/blob/e0a7fe7ea21976f5edef43244e4c04983be2d3e8/scripts/capabilities/readiness.py),
[native contracts](https://github.com/paul-heyse/pse-arrow/blob/e0a7fe7ea21976f5edef43244e4c04983be2d3e8/scripts/capabilities/native_contracts.py), FastMCP §14.2 and Pydantic §4/§7.

<a id="O05"></a>
### O05 — Attributed affected-validation selection and cost explanation

For a proposed edit, return a useful validation recommendation with reasons, exclusions and
unknowns. The current `workspace_integration_view` test projection supplies nearest-package seeds
and suggested routes. [`scripts/affected.py`](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/affected.py) already computes richer
Cargo reverse dependency closure, workspace-configuration widening, scientific-input consumers
and composed commands. Expose/compose that owner instead of implementing a second selector in MCP.

Combine the original selection with exact source/configuration, validation mode and test
population. Let the agent inspect why a crate, generated contract or cross-language consumer is
included and which dynamic consumers remain unresolved. Where existing comparable evidence is
available, show prior preparation versus execution observations with their source/feature/native
basis and age. A cold build is different from a long test; a historical duration is not a
prediction or a reason to silently omit correctness coverage.

Use existing assessment/test-selection/readers and declared relationships, not telemetry as
test-selection authority. Preserve explicit force-validation and scientific-pool contracts;
ordinary build settings remain the maintainer-selected high-throughput configuration. Report
missing comparable evidence instead of starting a benchmark or campaign to fill a table.

**Delivery demonstration:** an ordinary Rust edit, registry declaration change and Python/native
boundary change produce attributable selections matching the existing owner's meaning; unrelated
same-directory files do not acquire fabricated test coverage. The agent can choose a targeted
check from the answer and read its original result. Sources: affected owner,
[workspace domains](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/workspace_domains.py),
[assessment selection](https://github.com/paul-heyse/pse-arrow/blob/e0a7fe7ea21976f5edef43244e4c04983be2d3e8/scripts/capabilities/qualification.py) and
[qualification guide](../dev/validation-assessment.md).

<a id="O06"></a>
### O06 — Explicit change-to-contract-to-evidence relationships

A change packet should answer “Which exact declaration produces this output, which actual
consumers use it, which contract constrains it, and what retained observations apply?” Current
source owners already provide producer triples, task packets and rationale, but explicitly leave
some governing-contract/evidence links and dynamic consumers unknown. That is useful honesty;
querying harder cannot manufacture the missing relationship.

Extend original producers where a real relationship can be declared or extracted reliably.
Keep `produces`, `consumes`, textual reference, semantic symbol reference, selected test,
executed evidence and accepted decision as distinct edge meanings. Project the attributed facts
through workspace/knowledge owners. Return exact paths, directions, source generations and
original evidence references, with unknown branches and population completeness. Do not create
a manually maintained second ownership ledger or infer `tested_by` from nearby names.

SurrealDB attributed `ENFORCED` relations, selective incoming/outgoing paths and bounded
recursive traversal can serve repeated structured questions over an admitted knowledge cut.
Current native graph selection is one outgoing hop; expanding it requires explicit query and
cursor semantics. Depth bounds alone do not bound fanout or prove closure. For a small source
population, direct reader composition is simpler; for repeated global closure, compare native
queries with an exported compact graph and an established Rust algorithm.

**Delivery demonstration:** one cross-language declaration change returns its specific outputs,
real consumers, governing source and original relevant evidence, excludes unrelated declarations
sharing the generator, and preserves unresolved dynamic consumers. Source rereads verify digest
before editing. Sources: [workspace sources](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/workspace_sources.py), workspace
domains/tasks, [native knowledge](https://github.com/paul-heyse/pse-arrow/blob/1bee18378a20670af31838be0d35e3173917fc58/crates/pse-operations/src/knowledge.rs), and
[3.3.4 graph/recursive parser](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/syn/src/parser/idiom.rs).

<a id="O07"></a>
### O07 — Expose existing indexed knowledge questions

Codex can already read source corpus and library packets through MCP, and use native
`just knowledge query/explain` separately. The existing knowledge store supports exact facet
conjunctions, attributed literal full-text filtering, production triples, graph edges, immutable
generations and query/cut-bound cursors. Adapt those questions to the canonical typed interface
so repeated structured research does not require reconstructing native CLI arguments.

For example: select `generated_artifact:python/pse/_native.pyi`, inspect its declaration–generator–
output assertions and attributed contract excerpts, then reopen the returned source at its digest.
Keep exact identity/facet/relationship constraints first. Existing lexical filtering is useful
when explicitly requested; this does not propose semantic search, relevance ranking or inferred
relationships. A BM25-capable library is not a reason to turn exact discovery into ranking.

Reads must not import, refresh an index, initialize a service or silently switch to another
population. Return source freshness and generation explicitly; import/refresh remains a separate
effect. Keep source-only readers available when the index is absent. Native selective queries
earn their preparation/storage cost for repeated inquiries; `rg` or a direct source read remains
better for a known file or unimported material.

**Delivery demonstration:** an actual attachment follows a known production triple under one
explicit generation; changed cursor predicates, stale source, missing store and unattributed text
produce distinct outcomes. Sources: [knowledge adapter](https://github.com/paul-heyse/pse-arrow/blob/e0a7fe7ea21976f5edef43244e4c04983be2d3e8/scripts/capabilities/knowledge_queries.py),
[knowledge operator](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/knowledge_store.py), native knowledge owner;
[3.3.4 index/analyzer definitions](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/syn/src/parser/stmt/define.rs).

<a id="O08"></a>
### O08 — Native authored-model inspection and comparison

An agent extending the simulator needs to understand what an authored change means before
solving: declarations, selected case/procedure/route, instantiated members, lineage, defaults,
overrides, presets, preparation dependencies and reasons a case cannot proceed. This is a
substantive developer workflow, with direct eventual edit/re-solve product value.

Much already exists in Rust and the Python facade:
`ModelingPackage.declarations`, `project_compare(other, source_only=False)` and `inspect`, backed
by generated `DeclarationInventory`, `ProjectComparison` and `ModelingInspection` documents.
Adapt these through the existing native binding catalogue and owned specialist child. Compose
finite supplied sources there; do not return ephemeral Python model handles or create a second
host implementation of scientific meaning.

Distinguish authored differences, admitted-model differences, preparation availability and
executed outcomes. Source-only comparison can explain physical-context mismatch; it cannot
infer physical equivalence, reusable numerics or valid scientific outcomes from matching names
or text. Expose admission effects accurately—inspection that instantiates/adopts a model is not
universally passive. Selective summaries and original detail readers should preserve applicable
units/conventions and unresolved physical context.

**Alternative and demonstration:** direct Python use already works and remains appropriate for
exploration. Canonical adaptation is valuable when Codex can compare two finite authored sources,
trace a changed binding/default to its declaration, and see an inadmissible case's native reason
without a solver run. Validate changed physical context and preparation source cuts, not only a
happy-path diff. Sources: [Python modeling facade](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/python/pse/_modeling.py),
[native comparison](../../crates/pse-runtime/src/workflow/modeling/project_comparison.rs),
[native modeling](../../crates/pse-runtime/src/workflow/modeling.rs), native binding catalogue;
Serde/Schemars overview § directional schema and error-path capabilities.

<a id="O09"></a>
### O09 — Useful bounded inspection of nested scientific results

The original native result inspector is the right owner, but some meaningful values remain
unreadable through its current context/shape boundary. Prior actual Codex evidence in
[52h's native recovery account](../plans/52h-qualification-and-publication.md#actual-codex-telemetry-and-current-native-recovery-boundary)
records a fifteen-field deeply nested study relation whose mandatory context descriptors exceed
the 64 KiB descriptor bound. Schema paging succeeded; selected values returned `context_extent`
before acquisition. This is retained prior evidence, not a newly reproduced value-read journey.
Reducing row count cannot fix mandatory descriptor extent.

Extend [`result_inspection.rs`](../../crates/pse-runtime/src/workflow/result_inspection.rs) with
typed native child-field selection and manifest-bound metadata/context detail paging. Preserve
exact relation/run/attempt/window identities, widths, null/nonfinite distinctions and the physical
context required to interpret each value. A compact context reference may lead to another exact
page, but units, basis, quality and usability must remain explicit enough to prevent a false
scientific interpretation. Identify incomplete context rather than calling the value complete.

Where selection removes unnecessary work, place it in the native reader rather than slicing
after full hydration. The retained refusal itself establishes an availability/context-shape gap
before acquisition, not excessive value-read work. Compare child selection/paging with compact
shared descriptors and a justified finite-bound increase for the selected population; a simpler
representation may suffice without a new selector or continuation lifecycle. Judge complete
context size, actual acquisition and interpretability together. JSON Pointer is useful for
actual JSON documents; it is not an invented selector for Arrow child arrays. Keep exact Arrow
IPC/native export as an alternative for shapes whose readable projection is not yet supported.

**Delivery demonstration:** one chosen representative nested/context case, preferably the
previously unavailable study case, actually exposes a selected value with sufficient linked
context under finite bounds. A more precise refusal alone does not deliver this extension.
Other unsupported shapes retain precise refusals. Page continuation rejects mismatched
manifest/window/path. Independent expected values
and context check meaning; a shorter response alone proves no speedup. Blueprint §21.1 and
ADR-0224 remain the contract route if selection/context semantics change.

<a id="O10"></a>
### O10 — Constructible examples and directional outcome contracts

The endpoint already has typed validation, native generated codecs, structured errors and
path-aware refusal diagnostics. The opportunity is to make domain requests easier to construct
and their outcomes easier to consume, particularly result/preparation/project requests with
rich unions and state-dependent output. Do not rebuild the existing error framework.

Add representative schema-owned examples and domain descriptions at their authoritative
declarations. Schemars `#[schemars(example = ...)]`, generated schema definitions and Pydantic
`Field`/schema customization can supply selected examples; validation and serialization modes
remain distinct. Fingerprint-qualified contract/example resources can reuse the existing
SchemaSet and resource projections. Cache immutable schema/adapters by their actual basis,
not operational results whose freshness matters.

For still-generic read answers, define meaningful variants such as selected, admitted,
observed, unavailable and refused only where they are the actual owner's states. An accepted
request is not completed work. A syntactically valid example with a placeholder identity is not
runtime-eligible. Native decoding remains primary; JSON Schema explanation should clarify a
refusal rather than become a competing scientific validator.

**Delivery demonstration:** the agent constructs one previously awkward request from the
selected exact schema/example and receives the correct variant/error path for a controlled
invalid case. Example drift is checked against its declared directional contract. O01 must
forward it unchanged. Sources: native bindings, [schema generation](../../crates/pse-schema),
Pydantic §3.4/§4/§5.4/§7, Serde/Schemars overview, and
[Schemars 1.2.2 derive documentation](https://docs.rs/schemars/1.2.2/schemars/derive.JsonSchema.html).

<a id="O11"></a>
### O11 — Selected structural migrations through existing edit owners

Symbol rename and language actions are already available. Some useful changes cross strings,
generated declarations or conceptual boundaries that a language server cannot infer. The native
[`structural_migrations.py`](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/structural_migrations.py) and codemod owners are a
candidate source of selected additional operations; exposing every script is unnecessary.

Start with a demonstrated recurring migration whose authoritative inputs, affected consumers
and generator obligations are explicit. Return a complete owned preview, protection/source-cut
checks and reconciliation route through the same edit machinery. Generated outputs are changed
by their generator, never by relaxing protection because a codemod found a matching string.
Keep repository-language symbol identity separate from semantic-similarity search.

**Conditional value:** direct native invocation is adequate for rare migrations. Canonical
adaptation earns its cost when it gives Codex a materially safer, more intelligible preview than
manual edits. O06's real producer relationships and O03's lifetime decision enable richer flows;
they do not need a universal migration framework. Demonstrate one representative change,
changed-source refusal and interrupted reconciliation before expanding the operation set.

<a id="O12"></a>
### O12 — Comparative and bisect experiments with explicit predicates

An agent diagnosing a regression should be able to compare selected source/configuration variants
or locate a change using an explicit behavioral predicate and original results. Existing
[`regression_experiments.py`](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/regression_experiments.py), archive/read/replay and
isolation owners provide candidates not all adapted into MCP. Do not replace them with an
unrestricted shell tool or another execution ledger.

Expose a bounded comparison/bisect only when the subject, predicate, selected population,
environment differences, checkout isolation and restoration are expressible through those owners.
Retain original work references, terminal/drain and cleanup outcomes. Current pure-Rust archive
support does not establish native replay support; unsupported source/native combinations must
remain unsupported. A failing comparison assessor must not erase successful or failed subjects.

**Conditional value and demonstration:** one deterministic representative regression can be
isolated and compared through actual canonical access, with the predicate independently checked,
and interruption leaves authenticated resources and original evidence recoverable. Do not
require a large feature matrix or benchmark merely to demonstrate orchestration. Speed and
numerical-performance comparisons require applicable measurement conditions; ordinary functional
differences do not. Existing direct experiments remain preferable when adaptation adds no
material clarity or control.

<a id="O13"></a>
### O13 — Instrument-specific metric filters and grouping

Current producers declare useful finite dimensions—stage, status, failure class, suite, mode and
population—but [`QueryRequest`](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/telemetry_investigation.py) selects metrics mainly
by metric/time/service incarnation. Its p95 construction aggregates `sum by(le)`, combining
other dimensions. An agent cannot directly ask several useful questions without overbroad
selection and manual interpretation.

Extend the existing metric declaration and query owner with instrument-specific permitted
predicates and grouping. Examples: preparation versus execution latency for one mode; failure
counts by declared class; acquisition producers versus waiters for the relevant instrument.
Validate attributes against that instrument's finite vocabulary before backend work and retain
the resulting series population and units. Do not expose arbitrary PromQL/SQL or turn all tags
into an unrestricted query language.

Use existing OTel instruments/views and backend adapters; extra instrumentation is unnecessary
where the facts already exist. Attribute/cardinality limits and overflow can affect filter
coverage; an empty response is not proof of no failure. Histogram quantiles/rates remain estimates,
with reset/window/bucket limitations. High-cardinality source hashes belong in appropriate
references/trace context rather than unbounded metric labels.

**Delivery demonstration:** a controlled two-mode/stage population stays separate when filtered
or grouped; unsupported dimensions refuse before querying; retained raw facts support the
interpretation. This is a high-value extension to both development and eventual operations.
Sources: [metric declarations](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/telemetry_metrics.py), telemetry query owner,
OTel Python overview metric views and Rust overview metrics, and
[Python SDK 1.45.1 views](https://github.com/open-telemetry/opentelemetry-python/blob/v1.45.1/opentelemetry-sdk/src/opentelemetry/sdk/metrics/view/__init__.py).

<a id="O14"></a>
### O14 — Explain one invocation across original execution owners

For “Why is this taking so long?” or “What failed?”, provide a bounded guided answer about one
exact invocation: intended/selected/executed/failed counts, preparation/build stage, current or
frozen terminal state, native drain, available traces/logs and the next original reader. Existing
`product_run_read`, `assessment_read`, `workload_watch`, build observation and profile/pressure
captures already own these facts. Extend their useful projection only where actual manual
composition remains awkward; do not infer that the underlying data is absent.

Distinguish compile/preparation time from subject execution, expected workload cardinality from
observed progress, endpoint interface success from native outcome, and terminal record from
physical settlement. Report which owner was unavailable or not observed. A native acquisition
waiter count is not proof of a deadlock; a sampled profile is not an exact timeline; nested span
durations are inclusive; recorded idle is not equivalent to OS off-CPU time.

The current propagation/metrics/health foundation should be reused. FastMCP observation metadata
and OTel span links can connect references, but telemetry must not replace original state or
establish export delivery from successful native work. Prefer a progressive workflow/resource
over a new universal aggregate tool when it answers the same question economically.

**Delivery demonstration:** one cold preparation, one long subject and one failed/draining
invocation yield different truthful explanations with original references; absent telemetry
leaves execution facts readable. No new Collector, dashboard, task process or background monitor
is required. Sources: [workload readers](https://github.com/paul-heyse/pse-arrow/blob/e0a7fe7ea21976f5edef43244e4c04983be2d3e8/scripts/capabilities/workload_queries.py),
[telemetry queries](https://github.com/paul-heyse/pse-arrow/blob/e0a7fe7ea21976f5edef43244e4c04983be2d3e8/scripts/capabilities/telemetry_queries.py), FastMCP §11.4 and the OTel overviews.

<a id="O15"></a>
### O15 — Retained investigation questions and original-owner handoffs

A new Codex session should be able to continue an investigation without replaying the subject
or treating old telemetry as current. [`telemetry_selection.Investigation`](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/telemetry_selection.py)
already retains schema descriptions, finite copied selections and missing context in original
native member-pack custody. Extend that existing document, if useful, with a bounded question,
typed original-owner references and explicit unexecuted next-reader suggestions.

Separate “copied observation then” from “read original owner now,” including source/interval,
native invocation, result/schema manifest and unresolved physical/cleanup obligations. Reuse
existing evidence packs, issuer-bound cursors and original state readers. Do not create an
agent memory store, automatic restart loop or competing mutable run ledger. Suggestions are
neither execution nor authorization; a resumed investigation may discover that its source basis
no longer applies.

If a later execution is deliberately associated with the investigation, OTel `Link` /
`add_link_with_attributes` is a useful relationship; do not pretend it is an uninterrupted
parent span. Keep the native lineage/reference authoritative. Source: OTel Python link model and
[tracing-opentelemetry 0.34.0 extension trait](https://docs.rs/tracing-opentelemetry/0.34.0/tracing_opentelemetry/trait.OpenTelemetrySpanExt.html).

**Delivery demonstration:** after closing the first connection, the agent opens a retained
selection, understands its missing context and follows an original reader without duplicate
work; unavailable/deleted owners stay unavailable rather than reconstructed from telemetry.
This complements O02/O14 and has secondary production incident value.

<a id="O16"></a>
### O16 — Query-specific database and storage investigations

When a structured knowledge or project inquiry is unexpectedly slow, Codex should distinguish
population selection, traversal/index behavior, transport and retained-engine readers using
the exact query and source cut. Existing `knowledge_explain`, telemetry and storage-health
owners already supply much of this. Recommend a small attributed investigation packet only if
their separate typed reads do not answer the concrete question.

Combine a selected native query/generation, nonexecuting plan, relevant index definitions and
explicitly selected engine/telemetry observations. SurrealQL `EXPLAIN`, index information,
version-specific graph/reference scan counters and MX statistics can be complementary. Preserve
raw versioned plan detail rather than inventing a stable optimizer schema. `EXPLAIN ANALYZE`
executes the query and needs an explicit effect-safe subject/fixture; it is not passive diagnosis.
Reader counts/retained versions suggest hypotheses but do not prove causal memory leaks or
query slowness.

**Conditional value and alternatives:** existing separate reads are the preferred first route.
Choose adjacency/index changes only from actual query shape and evidence; do not prescribe
inline graph storage everywhere. No standalone KV diagnostic can inspect the current MX store.
Demonstrate a known query-selection/index distinction and retain absent/unsupported plan detail
without starting another database monitor. Sources: knowledge owner,
[SurrealDB EXPLAIN](https://surrealdb.com/docs/reference/query-language/statements/explain),
[inline adjacency reference](https://surrealdb.com/docs/reference/query-language/statements/define/table#using-inline-edges-and-inline-references-to-cache-adjacency),
[MX 1.0 statistics](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/stats.rs), and tooling overview.

## Coherent adoption and alternatives

The strongest immediate set is O01/O02 for independent access, O04/O10 for constructible and
explainable requests, and O05/O07/O08/O09 for useful development answers. These share foundations
but need not become one large framework. O06 should grow from real producer assertions in a
representative workflow; graph machinery cannot precede trustworthy relationships. O13/O14/O15
make the delivered telemetry/evidence foundations more useful without another deployment.
O03's advanced editing route, O11/O12 and O16 remain conditional on a concrete workflow whose
native route is inconvenient or insufficient. This is a qualitative priority judgment, not an
implementation schedule or an adoption quota.

| Meaning | Retained owner |
|---|---|
| Capability identity, domain/workflow membership, exposure, requirements and guide references | Existing capability declarations/catalogue/inventory |
| Scientific requests, interpretation, source admission and directional schemas | Rust domain owners and existing generated native boundary |
| Source/test/query relationship meaning | Original workspace, affected-selection and knowledge producers |
| Protocol visibility, validation, source coherence, observation and connection lifetime | Canonical FastMCP composition |
| CLI argv/help/rendering, argument presence and explicit connection scope | Derived Cyclopts consumer |
| Work settlement, cancellation/drain, replay eligibility and evidence custody | Original performing/native owners |

This follows AP-01–AP-06's responsibility/representation direction and AP-07's execution fit;
selective native work and proportionate recovery support the efficiency heuristics. PS-10–PS-12
constrain result/model interpretation, reuse and truthful solve outcomes. A capability that merely
returns fewer bytes after performing the same full acquisition has not established efficient
execution. Conversely, structured source browsing and exact native operations are valuable
without a benchmark proving agent productivity.

### Choices to retain

- **One FastMCP endpoint and the existing Python/native bridge.** rmcp's `Json<T>`, request
  context, cancellation and transports are meaningful for a concrete Rust MCP consumer, but
  do not remove the current admission, environment, schema or lifetime obligations. There is
  no identified consumer warranting wholesale rmcp/gRPC migration or a second public server.
- **Typed ordinary operations and original state readers.** R-55's CodeMode, protocol tasks /
  Docket, Apps/UI and F03 batching deferrals remain. None of O01–O16 requires them. The new CLI
  recovery consumer changes the earlier CLI question, not every conditional disposition.
- **Current database authority and patched MX integration.** `surreal_source.py` and its
  `mx-1.0-controls.patch` already select MX 1.0, SSI, typed errors and statistics; `surreal_server.py`
  owns synchronous persistence controls. Upstream 3.3.4's 0.27/SI adapter is not this checkout's
  effective design. No repeated “adopt MX 1.0” package is warranted.
- **No additional KV/evidence/agent-memory store.** SurrealKV's transactions, temporal reads,
  checkpoints and CLI have real standalone uses, but would add encoding/index/query/lifecycle
  obligations without the developer benefit identified here. Keep finite evidence computation
  in its Arrow/DataFusion owner. Git/source generations retain development history; MX AOL/MVCC
  are not a replacement historical-query or rationale authority.
- **Structured discovery and optional use.** No embedding index, semantic ranking, automatic
  database import, background watcher or obligatory MCP wrapper around shell/editor activity.
  Exact lexical filtering, language-server symbol semantics and attributed graphs have distinct
  useful roles. Do not confuse them with semantic similarity.
- **Existing environment ownership.** No Docker/server broker, resource cap, extra preparation
  service or serialization policy is needed for these proposals. Fixed source-only bootstrap
  remains independent of heavy workload admission; effectful operations retain their native
  environment/admission contracts.

### Remaining decisions and evidence

Planning should resolve CLI generation ownership/upstream fixes, complete command discovery,
argument/result fidelity and the initially supported lifetimes. It should test the installed
Codex refresh route rather than infer it from app-server documentation. Native result context
paging and model inspection need selected request/response examples and scientific meaning
checks. Relationship expansion needs actual producer facts; metric filters need per-instrument
vocabularies. Select one representative consumer for each conditional adaptation before paying
its integration and ongoing maintenance cost.

Detailed implementation packages, exact dependency moves and acceptance suites belong in
subsequent planning. Ordinary tooling/dependency additions alone do not require an ADR; altered
scientific contracts, edit lifetime/authorization or architectural decisions follow the existing
decision route. No rule or architecture change is accepted here. Current plans continue to own
their statuses; these proposals are not retroactive Plan52 completion failures.

**Evidence limit:** no measured speedup, context-token saving, agent productivity improvement,
full scientific qualification, production recovery guarantee or cross-platform delivery is
claimed. The actual new evidence establishes fresh-client access, per-connection selection,
generator behavior and endpoint dependence under the named local conditions. The substantive
workflow extensions remain source-grounded proposals with explicit delivery demonstrations.
