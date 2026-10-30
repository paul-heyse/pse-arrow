# Agent tool discovery and access

Functional-extension review · 2026-10-24 · developer-first, repository-wide scope.

## Recommendation and scope

**Proposed:** provide one source-backed discovery route for all agent-facing capabilities,
and a thin, optional FastMCP interface for selected repeated developer operations. Preserve
direct commands, specialized language/database tools and process skills. Unify how an agent
finds and understands a capability; keep execution, admission, state and scientific meaning
with their existing owners. Do not migrate the repository wholesale to Typer or turn MCP
into the sole execution path.

The discovery improvement has value independently of transport selection. FastMCP 4.1.3 is
the preferred additional agent interface: it is already installed and used, has typed tools,
providers and composition machinery, and can expose structured inputs/results without shell
reconstruction. Adopt its useful developer slice deliberately, then qualify actual Codex use.
An in-process client demonstration does not establish attachment or benefit in the real host.

The operator broadened this inquiry beyond Plans47/50/51/52 to **any repository tooling
targeted at agents**, including tools the reviewer initially overlooked. Those plans supply
important delivered capabilities, not the inventory boundary. This includes source discovery
and editing, design/planning/research, builds and validation, diagnostics, environment and
session setup, evidence, documentation, Git coordination and operational assistance. Product
opportunities are secondary. Tool use stays optional and proportionate; no mandatory gateway,
tool quota, new approval ritual or enterprise deployment is proposed.

This review selects recommended directions, not implementation scope. Existing plans, rules
and architectural owners are unchanged. The separately authorized Pydantic upgrade is actual
maintenance, described under [evidence](#evidence); the proposed interface is not implemented.

## Baseline, method and limits

Source baseline: main `0d13be7f9f81a8c0871c1700223479090b8d6f08`. `history.txt` was already
dirty and was not inspected or edited. User-supplied FastMCP, Pydantic and Typer overviews were
untracked and preserved. The subsequent scoped lock change moves Pydantic 2.13.5 → 2.14.1
and pydantic-core 2.46.5 → 2.50.1; FastMCP remains 4.1.3 and MCP SDK 2.3.0.

The FastMCP, Pydantic and Typer overviews were read end to end during preparation. They
broadened capability awareness; consequential claims below use exact-version documentation,
installed source or bounded probes. Context7 supplied versioned FastMCP/Pydantic references;
old FastMCP v3 behavior was not assumed to apply. Typer 0.27.3 was assessed from its supplied
overview and tagged upstream source, not an installed application trial.

The inventory covered the public Just recipe listing and its derived metadata, agent
configuration/skills/roles, documentation task routes, script families, native xtask frontends,
relevant architectural contracts and plan Outcomes. A read-only mapper widened coverage and
an independent design reviewer assessed the alternatives. Their conclusions were reconciled
against decisive source and the probes. This is broad capability exploration, not an assertion
that every helper, parser, external binary or installed tool was functionally exercised.

The selected standard is Core 3.4, Heuristics 1.0, process-simulator profile 1.5 and the
pse-arrow binding, through [standard.toml](../design_review/design_principles/standard.toml).
AP-01–AP-07 favor explicit operation meaning, local ownership, inspectable effects and small
consumer-shaped adapters. H8/H12/H16/H25/H26 favor useful interaction units, reusable schema
preparation, selective detail and machinery suited to one operator/editor. PS-10 matters when
an interface reports scientific results; no new physical formulation or numerical algorithm
is proposed, so the scientific review is about preserving native interpretation rather than
requalifying physics or solvers.

**Interface-checked:** source and dependency contracts below. **Tested:** only the named
bounded controls in [evidence](#evidence). All opportunities are **Proposed**. No productivity,
latency, reliability-rate or whole-product qualification claim is **Measured** here.

## What an agent can already use

The following inventory is organized by activity and native owner. Access routes include
instructions and read-only aids as well as executable commands. Discoverable does not imply
installed, attached, applicable or authorized for a particular task.

| Activity | Existing routes and owners | Consequence for a shared interface |
|---|---|---|
| Initial orientation and task routing | [AGENTS.md](../../AGENTS.md), [docs task routes](../README.md), [architecture reading guide](../authoritative_design/README.md), [current work](../plans/README.md) | Link the task-specific owner; do not repeat architecture or mutable plan status in a catalogue. |
| Review, planning and execution workflows | [process skills](../../.codex/skills/README.md), [shared roles](../../.agents/roles/README.md), native Codex/Claude adapters | These are instruction workflows with phase boundaries. Finding a skill must not invoke its effects. |
| Recipe discovery and conceptual context | [workspace_context](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/workspace_context.py), [workspace_domains](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/workspace_domains.py), `just context`, live `just --dump --dump-format json` | Existing exact concepts, sources and recipe metadata are foundations; an outer `*args` declaration is incomplete leaf grammar. |
| Source ownership, change reach and rationale | [workspace_tasks](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/workspace_tasks.py), [workspace_sources](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/workspace_sources.py), `just affected`, selected Git history | Preserve declared generator relationships and unresolved dynamic/cross-language reach; textual matches are leads. |
| Language navigation and controlled edits | [semantic_navigation](https://github.com/paul-heyse/pse-arrow/blob/e0a7fe7ea21976f5edef43244e4c04983be2d3e8/scripts/semantic_navigation.py), [semantic_edits](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/semantic_edits.py), mcpls/Pyrefly/rust-analyzer, structural ast-grep/LibCST routes | Preserve captured source cuts, protected/generated-path refusal, preview/apply distinctions and partial application reconciliation. |
| Session attachment | [codex_session](https://github.com/paul-heyse/pse-arrow/blob/e0a7fe7ea21976f5edef43244e4c04983be2d3e8/scripts/codex_session.py), `just codex-session`, [.codex/config.toml](../../.codex/config.toml) | Rust/Python/database capabilities are selected at launch; a configuration file alone does not establish a working attachment. |
| Environment and capability preparation | [pse-env](../../scripts/pse-env), [environment guide](../dev/agent-environment.md), `--explain --json`, `doctor-json`, `setup-env`, bootstrap/optional-tool recipes | Explain before effects. Installation, builds, service startup and live observations have distinct owners. |
| Compilation and cache diagnosis | `check-package`, `check-library`, scoped native checks, `build-frontend`, `build-uncached`, `build-cache-probe`, `bench-builds`, [build_environment](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/build_environment.py) | Describe selected graph/mode and prerequisites; avoid accidentally replacing a targeted check with a workspace build or benchmark. |
| Functional and scientific validation | [validation_scope](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/validation_scope.py), [select](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/select.py), `assessment-list`, `unit-package`, Python/native routes, conformance/parity/reference recipes | Expose the check's actual scope and exclusions, force-validation mode and native setup. A selected check is not full qualification. |
| Verification techniques | `coverage`, `mutants-file`, boundary fuzzing/seed routes, explicit reference generators, [regression_experiments](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/regression_experiments.py) | Distinguish available mechanisms, delivered controls and unexecuted campaigns. Plan52c's denied fuzz campaign is not established coverage. |
| Workload observation and outcomes | [activity](https://github.com/paul-heyse/pse-arrow/blob/a67d235b3f3f1f2ee8e532f778ee900524928ae2/scripts/activity.py), [activity_watch](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/activity_watch.py), [run_inspection](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/run_inspection.py), [validation_result](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/validation_result.py), `just result run` | Reuse authentic handles, partial observations, recorded/live distinctions, native continuation and released-detail rules. |
| Scientific diagnostics and offline review | `just diagnostics`, [diagnostic operating guide](../dev/simulation-diagnostics.md), Plan47 CLI/frozen-pack/result/context owners | Keep source context, diagnostic interpretation and protected result selection with their native owners; offline inspection should stay offline. |
| Active native/Python diagnostics | [active_diagnostics](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/active_diagnostics.py), [diagnostic_trace](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/diagnostic_trace.py), `diagnostics active`, `trace-query`, optional GDB/perf/taskdump/heap/memray/rr tools | Discovery must describe attachment effects and exact process identity, not suggest that listing or tracing owns the subject process. |
| Canonical database operations and tooling | `just surreal`, `surreal-module`, `surrealkit`, [surreal_diagnostics](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/surreal_diagnostics.py), selected native DB MCP | DB query access is useful specialist access, not a substitute for project-operation admission or general agent memory. |
| Projects, revisions, scientific data and evidence | `just project`, `just evidence`, explicit build recipes, [pse-project](https://github.com/paul-heyse/pse-arrow/blob/1bee18378a20670af31838be0d35e3173917fc58/xtask/src/bin/pse-project.rs), [pse-evidence](../../xtask/src/bin/pse-evidence.rs), Python/native APIs | Preserve source revisions, temporal meaning, supplied-only offline reads, independent derived indexes and explicit mutation. Inspection never builds the operator implicitly. |
| Knowledge, libraries and attributed recall | `just knowledge`, `context --library`, `context --retrospective/--recall`, [session_retrospective](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/session_retrospective.py), [library routes](../libraries/README.md), `lib-outline` | Exact conceptual references and versioned primary sources already exist. Historical observations are not current instructions or authority. |
| Experiments and delivery/recovery | `just experiment`, [experiments](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/experiments.py), [tooling_experiments](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/tooling_experiments.py), reference-run and canonical lifecycle/portable controls | Describe selected fixtures, isolation and cleanup obligations. An archive, a Git worktree and a database namespace are different resources. |
| Schema and generated consumers | `just codegen` and scoped generate/check routes, registry declarations, compiled Python stubs, [schema ownership](../authoritative_design/sections/schema-and-relations.md) | Discover the declaration/generator owner before editing a derived result. Fix the producer and regenerate through its existing route. |
| Documentation and decision lifecycle | [document_lifecycle](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/document_lifecycle.py), `plan`, ADR creation/index/supersession/register routes, `docs`/search/publication | Scope projections and retirement preflight do not authorize deletion, acceptance or plan completion. |
| Collaboration, Git and closeout | [worktree](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/worktree.py), [coordination contract](../../.agents/roles/README.md), native Git, Plan52g G04 proposal | Creation has explicit effects and checkout isolation. Git cannot infer task ownership of dirty hunks; generic closeout assistance remains proposed. |
| Repository/tool maintenance | `upgrade`, `family-check`, dependency reports, `versions`, `build-storage`, agent/setup guards and `turn-end`/`ready`/`hygiene` | Preserve exact pins, explicit cost/scope and timing rules. Inventory is not cleanup, and a recipe is not an approval gate. |

Plans47,50,51 record delivered scope and their own qualification limits. Plan52 A–E supplies
additional context, verification, execution and isolation controls; F/G/H contain remaining
comparison, workflow and assembled qualification work. The inventory does not promote those
draft capabilities to delivered status or reopen completed plans as backlogs. Newer Plan53
scientific verification work is an additional consumer, not authority for this interface.

## Concrete discovery failures and their causes

These examples support the functional argument without attributing every failure to discovery.

| Observation | Causal account and limits | Useful change |
|---|---|---|
| The initial review pass entered through the four named plans and `workspace_context`; the wider survey later surfaced `assessment-list`, source-protected editing, `lib-outline`, session selection, lifecycle preflight and worktree setup. | The operator's initial pointers and reviewer selection narrowed the first pass. Existing routes document the tools, but there is no comparable conceptual query spanning commands, skills, session aids and maintenance. Late discovery alone does not prove global undiscoverability. | O01/O03 should let an unfamiliar task reach an applicable tool across owner families. |
| `context experiment --json --limit 3` reports the public recipe's `args` star parameter. `python -m scripts.experiments archive --help` returns outer operations rather than archive options. | Just accurately describes the wrapper. The outer argparse `parse_known_args` consumes help before leaf dispatch. Deriving a complete archive schema from `*args` would fabricate information; changing CLI framework alone would not fix ownership. | O02/O03 expose the actual leaf parser/owned signature and forward help correctly. |
| Recent work required reconstructing selected paths, native setup, resource class and service endpoint repeatedly. A mistaken class selection produced an admission wait. | The environment owner already resolves these facts; callers bypassed or incompletely consumed it. More context prose does not establish the effective selection. | O06 composes existing explanation and actual operation facts. It must not create new defaults. |
| Generic database/tool failures sometimes lost the detailed cause; a fresh fixture was needed to reveal a UNIQUE-index refusal. | Losing a native cause at an adapter boundary obstructs diagnosis. It does not imply every DB failure is an input validation error or that retry is safe. | O04 retains typed causes and fixture/operation references, with bounded detail. |
| A standalone validator rejected Surrealism syntax while the exact server library parser supported it; SQL line flattening also changed a schema after `//` comments. | Tool/parser capability and byte-preservation contracts mattered more than a generic “SQL validation” label. The accepted native parser and HTTP owner supplied the correct route. | O01/O02 describe exact tool capabilities, versions and input transport semantics. |
| “Removed its checkout” caused confusion between Git and database resources; parent return, descendant drain and database settlement also differed. | Resource kinds and outcome axes were insufficiently visible in the interaction. Uniform success text would conceal them further. | O04/O05 identify native resources and separate observation, execution and settlement. |
| This review's first retrospective invocation used `--limit 5` but produced a large nested checkpoint/source inventory. | The recipe/result page bound did not bound the separately nested retrospective checkpoint. A small requested page is not a small total response. No secret content was needed to establish this mismatch. | O05 provides a bounded summary and separately retained continuation, including nested byte/record bounds. |
| During the requested upgrade, exact `uv sync --no-install-project` removed the editable distribution. | This sync mode intentionally excludes the project; it is the preparation half of the existing rebuild route, not a tools-only preserve operation. An attempted cached-wheel restoration briefly registered another checkout; that registration was removed before product execution. Restoration then used the supported native correctness build. The existing `setup-env` already uses an inexact tools-only sync. | O06/O09 describe the desired operation and preserve installed state through the existing preparation owner, rather than encouraging command reconstruction or cache guesses. |

Bounded retrospective inspection selected the 2026-10-23 and 2026-10-24 session directories
and record timestamps within approximately 2026-10-23 05:21–2026-10-24 05:21 UTC. It completed
those directory reads, observing 32,783 and 844 associated records respectively. The earlier
whole-root bounded scan was incomplete and cannot establish absence. Sessions created on
earlier dates with activity in this interval are outside the completed narrowed sample.
The projection surfaced path, traceback, compiler-lock, compilation, permission, storage and
memory-pattern observations. These can include quoted assertions, repeated output and historical
discussion: they are **not incident counts, productivity totals or measured user friction**.
No raw prompts, credentials or bulk log text were published. The direct examples above and
source contracts supply the causal argument; aggregate pattern frequency does not.

## Opportunities

<a id="O01"></a>
### O01 — Discover capabilities across owners through exact concepts

An agent should be able to ask “what helps inspect a generated boundary?” or “how can I
observe this invocation?” without already knowing `codegen-contracts-check`, `activity`,
an external profiler or a particular skill. Extend the existing conceptual discovery owner
to include capabilities and access routes alongside documents, generators and evidence.

Use stable owner-scoped capability identities, explicit purposes, workflow/domain tags,
resource/effect kinds, prerequisite references, access routes and schema/help provenance.
A capability may be an instruction workflow, read-only report, command, Python/native API,
MCP/LSP aid or optional installed utility. One capability can have several routes; a single
command may expose several operation-specific capabilities. Keep these relationships explicit
without inventing an ontology for every shell command.

Derive recipe grammar from Just, native CLI grammar from its parser, tool schemas from owned
signatures, skills from their metadata and source relationships from existing declarations.
New purpose/effect metadata belongs at the responsible owner; derived discovery must cite that
source. Missing leaf grammar stays explicitly incomplete. Do not maintain a second editable
YAML catalogue of arguments/defaults. Source-only discovery must work before a tool is installed
or a server is reachable and must not run live readiness probes by default.

**Value:** a shared discovery route helps every transport and makes overlooked tools visible.
**Delivery evidence:** a fresh agent finds a useful unfamiliar tool outside the four starting
plans from a concrete task, follows its exact owner and understands its limits. Listing every
recipe by name is useful coverage, but does not alone establish this capability.

<a id="O02"></a>
### O02 — Reuse executable host-tool contracts with Pydantic 2.14

Use Pydantic where host tools need one executable declaration for validation, schema and
serialization. Prefer `Annotated` constraints and reusable `TypeAdapter`s applied to `TypedDict`,
standard dataclasses or existing suitable host types; use `BaseModel` when its model features
are useful. This avoids requiring every record to adopt another class hierarchy. Pydantic's
versioned [TypeAdapter documentation](https://github.com/pydantic/pydantic/blob/v2.14.1/docs/concepts/type_adapter.md)
supports validation and serialization of types beyond models.

Represent materially different results with discriminated variants: for example, available
description versus unavailable capability, complete page versus partial page with continuation.
Use strict argument validation and explicit extra-field refusal where exact calls require it.
Keep CLI text parsing separate from validation of already structured MCP values. Python strict
validation and JSON strict validation have different accepted representations; test both consumed
routes rather than assuming one flag gives identical semantics.

Pydantic 2.14's stable `MISSING`, field-set tracking and selective serialization are particularly
useful for partial edits: absent, explicit null and explicit empty collection may mean different
actions. Defaults also need a deliberate validation policy. Separate validation and serialization
JSON Schemas where formats differ, such as Decimal. Native handles, paths and revisions need
owned meaning and contextual admission after shape checking. See the versioned
[serialization](https://github.com/pydantic/pydantic/blob/v2.14.1/docs/concepts/serialization.md)
and [JSON Schema](https://github.com/pydantic/pydantic/blob/v2.14.1/docs/concepts/json_schema.md)
guidance.

Rust serde/schemars declarations, generated msgspec documents, attrs rows and compiled PyO3
metadata remain authoritative under blueprint §21.5. Do not hand-copy their fields or validators
into Pydantic. A wrapper can own host-specific selection/page/attachment arguments and delegate
native document decoding; a generated schema projection requires a real consumer and explicit
loss checks. Pydantic custom schema hooks, `ValidateAs` and serializers are integration leads,
not permission for a competing model. Rust/database contextual constraints and scientific
postconditions remain native.

Cache adapters/schema preparation at an appropriate module/process lifetime. Avoid partial
validation for final authoritative results, `model_construct` or unchecked copy updates at
untrusted boundaries, and implicit `fallback=str` loss for unsupported values. Frozen models
are not deep immutable capabilities. These distinctions matter more than adopting every new
feature. `create_model(__namespace__)` and newer type/schema support are available if a concrete
derived contract needs them; dynamic model generation is not the default foundation.

**Delivery evidence:** one host-owned request produces matching direct/MCP acceptance and schema,
rejects malformed input before effects, preserves absent/null/empty, and does not duplicate a
native generated contract. Actual first-party Pydantic usage should add an exact direct
dependency in its owning group; the present upgrade changes an existing transitive lock only.

<a id="O03"></a>
### O03 — Progressive descriptions, leaf help and usable next actions

Offer short discovery results first, then an exact capability description with input/output
schema, literal argv route, examples, effects, prerequisites, cost class, source basis, known
coverage and relevant instruction owner. Return task-appropriate next reads as structured
references, not a guessed shell paragraph. Preserve lexical discovery for exploratory wording;
exact identities, facets and declared relationships supply first-principles selection.

For example, a query tagged `generated-output` and `python-boundary` can return the declaration,
generator, scoped regeneration/check routes and applicable editing rules. The description must
say which relationships were declared and which dynamic consumers remain unresolved. For an
experiment, `describe` should reach the archive leaf grammar without executing or building it.

FastMCP's `RegexSearchTransform` and BM25 search can reduce advertised schema volume, but they
search textual tool metadata; they do not provide the repository's exact conceptual model.
Search transforms hide tools from listings while original tools remain callable, so they are
not effect/access enforcement. Use an exact discovery tool/provider view over O01, with optional
lexical help. See [versioned tool-search behavior](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/docs/servers/transforms/tool-search.mdx).

**Delivery evidence:** help and description expose real leaf options and deliberate defaults,
including a command with forwarded arguments; an unknown capability or incomplete grammar is
truthful and still points to the appropriate owner. Instructions stay concise because detailed
information is available when needed.

<a id="O04"></a>
### O04 — Actionable errors that preserve independent outcome meanings

Standardize the meanings a caller needs, without requiring every native owner to emit one
universal envelope. Distinguish malformed invocation, unavailable prerequisite, stale source,
native domain refusal, interrupted/partial operation, transport/receiver failure and unexpected
implementation error. Retain native error code, cause chain, source/subject reference and
bounded diagnostic detail. A useful recovery points to the exact existing operation; it does
not silently install, restart, widen admission or retry effects.

Pydantic `ValidationError.errors()` offers locations and error types. Map only the stable
information needed by the host contract; raw input and arbitrary context may contain sensitive
values and should not be echoed automatically. Native diagnostic codes retain their namespace.
Do not classify errors by regular expressions over human messages. FastMCP masking and explicit
`ToolError`/structured `ToolResult` paths need deliberate handling: turning masking off globally
is not a substitute for an attributable, useful domain failure. The installed4.1.3 source
distinguishes argument Pydantic errors from errors raised inside the tool body.

The probe also demonstrates a separate receiver path: an invalid output against the declared
schema raises a client `RuntimeError` even with `raise_on_error=False`. It is not an ordinary
domain `is_error` result. An adapter must preserve and test both cases. Output schemas improve
interface checking; they do not themselves establish valid scientific content.

**Delivery evidence:** malformed count, absent executable, changed source cut, native UNIQUE
refusal, partial observation and malformed server output remain distinguishable. Successful
transport/exit, completed test population, resource drain and accepted scientific outcome must
never collapse into one success bit (blueprint §23.2; PS-10).

<a id="O05"></a>
### O05 — Native handles and bounded detail through the whole response

Expose short operation/result summaries and exact native handles, then owner-specific watch,
page, detail or offline-evidence reads. Apply bounds to nested sources, checkpoint state and
error payloads as well as the top-level item list. Keep large Arrow results, trace files,
archives and raw logs in their existing protected/offline formats; return references and
selected views instead of JSON-encoding entire scientific datasets into MCP.

Handles identify their kind and source: Git worktree, database namespace, invocation, process
generation, immutable result or reader. Reuse existing owner authentication and released-detail
rules. Continuation must retain source/revision and completeness; an empty partial page cannot
mean no matching failures. Retain a checkpoint locally when needed and return a compact locator
rather than copying it into every tool response. A framework session must not become the only
place the handle exists.

Stopping or timing out an observation detaches it; transport disconnection cannot cancel
unrelated or merely observed work. Cancellation of an action that owns native work follows
that action's existing contract. Explicit cancellation delegates to the native owner with
the exact target and preserves cancellation-versus-drain-versus-settlement.
Synchronous thread execution cannot be forcibly undone by a FastMCP timeout; async cancellation
also needs operation-owned cleanup. Framework task state does not replace these contracts.

**Delivery evidence:** context→action→handle→watch→partial result→detail works with a detached
observer, restarted client, changed source, unavailable native owner and a bounded nested output.
Cancellation of an authorized native action is tested separately from cancellation of the read.

<a id="O06"></a>
### O06 — Explain and attach the selected environment without reconstruction

Compose `pse-env --explain`, session selection and existing operation facts into a useful
description of the intended checkout/native/store/resource context. Separate configured,
installed, currently available, attached and exercised states; a readiness observation has a
timestamp and can become stale. Keep source-only discovery independent from compiler/native/
database preparation, so a broken workload environment remains diagnosable.

The FastMCP launch path should reuse the selected checkout environment and `codex_session`
wiring. Use explicit cwd, pinned binaries, lifecycle-owned clients and dependency injection for
trusted selection/services. Do not pass credentials, mutable global “current project” defaults
or arbitrary process state through user-selectable tool arguments. Session/store identity must
come from its existing owner and be rechecked where the native contract requires it.

Offer purpose-specific preparation descriptions such as tools-only sync preserving the installed
extension versus rebuild of the linked correctness extension. Reuse `setup-env`'s existing
preservation mechanism, package pins and native correctness selectors. Tools-only maintenance
must not accidentally launch a cold full extension build, and an actual required rebuild must
remain explicit. This is machine-specific convenience grounded in the current workstation,
not a second environment manager or blanket host-privilege interface.

**Delivery evidence:** a fresh Codex session can explain what is attached and use one selected
tool; unavailable MCP leaves direct discovery/commands useful. A tools-only update preserves the
installed editable registration, and an explicit rebuild records its actual features. No secret
values appear in discovery or environment reports.

<a id="O07"></a>
### O07 — A thin FastMCP developer surface composed from native responsibilities

Expose a useful repeated slice: capability query/description, source/task context, selected
targeted action, exact invocation observation, native result and evidence detail. Ordinary typed
functions and modules suffice initially. FastMCP local providers and mounted namespaces can
compose independent owners; a custom provider is justified when it derives capabilities from
live owner declarations. Do not create a framework of provider subclasses for every script.
See [providers](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/docs/servers/providers/overview.mdx)
and [composition](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/docs/servers/composition.mdx).

Illustrative tool names are `capability_query`, `capability_describe`, `source_context`,
`run_observe`, `run_result` and operation-specific actions. These names are proposed interaction
examples, not a new dispatcher specification. Avoid a universal `execute(command: str)` tool:
it discards the typed contract and reproduces shell reconstruction. Keep native literal argv
execution available through existing shell access. A dedicated action delegates to the same
owner used by the direct route and preserves its environment/admission rules.

Use tools for explicit actions/queries, resources for addressable read-only content and prompts
or skills for reusable guidance where the actual Codex client supports them. `ResourcesAsTools`
and `PromptsAsTools` are fallback presentation options, not a reason to duplicate every asset.
Filesystem/Skills/OpenAPI providers and proxies can expose already-owned sources when useful;
do not infer semantic operation safety from an automatically exposed filesystem or REST API.
Keep specialized language-server and native database MCP routes independently accessible rather
than proxying everything through a new transport hop.

Middleware can provide correlation, bounded diagnostics and ordinary validation behavior;
lifespans/DI manage actual resources. Avoid hidden store startup on import or catalogue listing.
Treat MCP annotations as hints, not admission. Do not introduce FastMCP Tasks/Docket as a second
scheduler for already-owned native invocations. Background tasks, modern elicitation and
multi-round reinvocation require separate idempotency/lifetime design if a concrete need later
appears. ClientGroup/federation, experimental CodeMode and server-side code composition are
available leads, not defaults for this single-editor workflow.

**Value:** typed calls and structured results can remove repeated argument/output reconstruction.
This is a credible static proposition, not measured productivity. **Delivery evidence:** actual
fresh-session Codex use of the composed slice, matching native outcomes and unavailable/partial
cases, plus interruption/cleanup. Installed FastMCP and a listed tool catalogue are insufficient.

<a id="O08"></a>
### O08 — Compose editing, validation and integration by meaningful task

Make existing tools easier to combine for tasks agents actually perform: inspect a generated
contract, resolve its producer, preview an edit, select affected checks, observe them, inspect
failures and assemble an owned integration/handoff. Use source/context and operation contracts
from O01–O05. A supplied task/ownership declaration is evidence; Git diff alone cannot infer
who owns a hunk or prove plan completion.

Return useful next-action references with native parameters and scope/exclusion descriptions.
Keep suggested checks separate from execution. Preserve native force-validation, explicit
setup capabilities, Python classification and scope-end static-check timing. A tool that
helps find a profiler, benchmark or fuzz mechanism must distinguish use authorization and
environment support from scientific acceptance or previous campaign execution.

This composes existing `affected`, assessment, edit-preview and result owners rather than
building a second scheduler, automatic plan executor or mandatory pipeline. Integration assistance
can expose regenerated consumers, unresolved conflicts, unrelated edits and proposed commit
selection; staging/merging/deleting stay separate authorized actions. Native APIs may be called
directly when they already provide the right typed boundary; subprocess adapters are appropriate
for genuine CLI/process ownership, without formatting output just to parse it back.

**Delivery evidence:** a generated-boundary task, a Python-only change and an overlapping dirty
tree each yield correct owners and targeted scope. The agent can choose a direct route at any
step, and a planning-only task produces no implementation effects.

<a id="O09"></a>
### O09 — Progressive instructions and research context for unfamiliar tools

Keep one concise starting route in existing instructions. Discovery supplies the relevant
AGENTS/rule/skill, operation guide, source owner, library version and declared limitations when
the task needs them. Do not append the whole inventory to every prompt. Skills remain deliberate
instruction workflows; a planning skill and its action companion retain different effects.

Integrate library maps/overviews, installed versions, exact primary references and selected
research questions through the existing structured context/evidence owners. Use lexical query,
explicit tags and conceptual relations; no embedding/semantic-similarity store is necessary.
Supplied evidence and historical excerpts keep attribution, source cuts and historical-only
status. Finding an old command or recommendation never supplies current authorization.

FastMCP's generated `SKILL.md` can be a derived quick-start for an adopted tool surface, but
must link current owners and use the existing skill packaging route. It should not override
process skills, duplicate mutable policies or claim a generated example was exercised in Codex.

**Delivery evidence:** a fresh task finds an unfamiliar applicable tool and loads only relevant
guidance. A review/planning request stays read-only; an exact-version library question reaches
the appropriate current source without treating an overview or stale search result as proof.

<a id="O10"></a>
### O10 — Improve CLI access selectively; qualify generated projections

Typer 0.27.3 is attractive for a deliberately designed operator command tree: typed flags,
command groups, help, completion, callbacks and CliRunner. It does not itself create the
cross-owner capability model, native domain errors or structured results. Async owners need an
explicit boundary because callback execution is synchronous. Since0.26, Click is vendored;
older external Click plugin assumptions need checking. See the tagged
[Click compatibility guidance](https://github.com/fastapi/typer/blob/0.27.3/docs/tutorial/click.md).

Prefer fixing leaf help and exposing owned schemas at current parsers first. Add a small Typer
view only when Paul has a concrete command-tree/navigation need that it serves better than
existing routes. If selected, make parser conversion and domain admission separate and test
option provenance, booleans, repeatable lists, paths, errors and exit semantics. A wholesale
argparse/Clap migration would spend effort on a distinct concern and require migrating real
callers; it is not justified by discovery fragmentation alone.

FastMCP4.1.3's `generate-cli` produces a Cyclopts client and skill from MCP tool schemas. It
could supply a convenient secondary CLI without manually duplicating every command definition.
The generated CLI still connects to the server on invocation; complex schemas become JSON
arguments rather than fully elaborated flags. Crucially, the installed generator filters out
all `None` and empty-list values. The [probe](probes/agent-tool-discovery-2026-10-24.py) confirms
that explicit null/empty is lost, even though the Pydantic model preserves it. See the exact
[generator source](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/src/fastmcp/cli/generate.py).

Do not adopt that output as a faithful general mutation interface unchanged. An upstream or
generator-level correction and tests for omission/null/empty semantics are needed before those
operations use it; hand-editing generated output is not the repair. Generic `fastmcp list/call`
is useful for inspection/prototyping, but still has transport/setup obligations. Direct Cyclopts
is a credible new-CLI alternative already present transitively; it requires its own exact direct
dependency and real operator benefit if adopted. Pydantic-settings CLI is a narrower fit for
configuration-oriented tools, not a new environment-precedence authority.

**Delivery evidence:** selected CLI projections preserve the same values/effects/results as
their owners, and the direct route remains usable when the MCP service is unavailable. Choose
whether a new CLI is worthwhile from one real operator investigation, not framework preference.

<a id="O11"></a>
### O11 — Correlate diagnostics and interaction paths without another evidence store

Give discovery/description, selected invocation, watch and result reads enough shared references
to explain which operation/source/environment was used and where detail can be inspected.
Reuse native run IDs, trace/result references, diagnostic codes and Plan51 evidence adapters.
If a facade fails, distinguish time in client transport, host adapter and native operation;
keep unobserved intervals unknown rather than attributing them to Rust/database execution.

FastMCP's OpenTelemetry support and middleware can supply transport-level correlation if needed;
an exporter is a separate selected integration. Compose this with existing traces and Plan52f
comparison, rather than adding a new telemetry database, event for every primitive or compulsory
profiling path. SurrealDB structured evidence queries can support exact tool/version/owner/error
analysis using existing imported evidence, but the operational/scientific stores stay authoritative
in their native scopes. The derived tooling index remains disposable.

**Delivery evidence:** one deliberately failing composed interaction leads from bounded error
to exact native evidence; no duplicated large payloads, secret leakage or forced live database
dependency for source-only work. Quantitative overhead/latency claims require actual measurement.

<a id="O12"></a>
### O12 — Qualify discovery, contracts and actual agent use separately

Use small independent scenarios that challenge the interaction, not tests that simply replay
the metadata generator. Compare direct and optional MCP routes for the same meaningful task.
Include unfamiliar tools outside the starting plans, missing prerequisites, a forwarded-help
command, generated source, partial results, stale source, interrupted observation, explicit
null/empty edits and unrelated dirty work. Check where a next action is useful versus misleading.

Static/source checks can establish owner linkage and structural gaps; library probes establish
named framework behavior; actual Codex sessions establish attachment and usable interactions.
None alone establishes future reliability or full scientific qualification. Keep useful fixtures
and observations with the existing owning plan/test infrastructure, not another evaluation
service, score, per-command receipt scheme or incident ledger.

**Delivery evidence:** the agent actually selects a useful capability it did not initially know,
uses the correct owner, checks the intended behavior and reports the real limits. This directly
tests the operator's concern. A passing catalogue snapshot or help screenshot is insufficient.

<a id="O13"></a>
### O13 — Carry developer interface value into selected product operations

The same contract/discovery mechanisms can make project/revision/evidence interactions useful
to future agents: inspect a project's selected revision, describe a study/result, read protected
evidence and request a specific admitted project action. Preserve temporal/source context and
the distinction between authored model authority, derived tooling knowledge and scientific
outcomes. These are plausible secondary consumers of existing Plan51 owners.

Start from one concrete project/evidence journey when product use is selected. Do not turn this
review into a general simulator API, bulk-data JSON gateway, generic database writer or remote
multi-tenant service. FastMCP HTTP authentication, OpenAPI providers and UI/apps are possible
delivery mechanisms when the consumer needs them; none is necessary for local developer access.
The product value is controlled, discoverable native operations, not merely exposing SQL or a
large tool catalogue.

**Delivery evidence:** a selected agent can inspect the correct revision and evidence, perform
one explicitly admitted action and distinguish native refusal/partial outcome. Product adoption
and remote delivery need their own scope, lifetime and qualification decisions.

## Compare the complete alternatives

| Realization | Main benefit | Material cost or limit | Recommendation |
|---|---|---|---|
| Current routes plus better instructions/help only | Least new runtime machinery; native tools remain direct and diagnosable. | Humans/agents still join schemas, effects, handles and cross-family discovery; instructions alone can grow stale. | Useful immediate improvements; not sufficient for the full structured discovery proposition. |
| Shared discovery/contracts, existing direct execution | Exact cross-owner discovery with native CLI/API access; works without an MCP server. | Requires explicit new capability metadata at owners and truthful incomplete adapters. | Foundational target; useful even if MCP is not selected after actual comparison. |
| Shared foundation plus optional thin FastMCP | Typed discoverable repeated calls and structured results; existing dependency and composition machinery. | Launch/attachment, protocol/error/lifetime adaptation and schema volume need real integration. | Preferred additional agent access direction; qualify the useful slice rather than wrap all tools. |
| Unified Typer tree over owners | Good deliberate terminal UX, nested help and completion. | Additional command hierarchy and parser projection; does not solve cross-tool conceptual discovery by itself. | Conditional operator view, not repository-wide migration. |
| MCP-generated Cyclopts CLI | Derived help/flags/skill for an already selected MCP surface. | Server client lifecycle, complex JSON arguments and tested loss of explicit null/empty. | Secondary option after generator semantics are fixed/qualified for selected operations. |
| Proxy all tools or expose generic shell/SQL through MCP | Broad superficial reach with little initial schema design. | Hides operation contracts and repeats transport layers; generic access does not supply trustworthy meaning or effects. | Keep specialized native access; do not use this as the unification foundation. |
| New daemon, universal workflow engine, semantic tool memory or operator UI | Could support broader future deployment/workflows. | Adds state, lifecycle, interpretation and operation machinery not required by this single-operator target. | No present adoption case; revisit for a concrete unmet consumer. |

The recommended composition keeps native ownership distributed while discovery and selected
access are coherent. AP-04 is satisfied by explicit scoped capability/operation meanings, not
by a generic “tool” envelope alone. AP-07 favors preparing schemas once, bounded descriptions,
native pushdown/result selection and avoiding a subprocess/serialization hop where a suitable
typed owner call already exists. Libraries inform the design; neither a framework label nor
fewer handwritten lines establishes a better assembled workflow.

## Priorities, investigations and correspondence to existing work

Start with O01–O06: source-backed inventory, executable host contracts, leaf descriptions,
errors, handles and environment selection. These are jointly useful through existing commands.
Develop O07 alongside one real O08 task, with O09 guidance and O12 actual-agent assessment.
Do not delay the first useful slice until every capability has a wrapper. Apply the shared
discovery model across all agent tooling, while typed execution exposure follows demonstrated
or credible repeated use. O10/O11 can then address a selected operator/diagnostic consumer;
O13 is secondary product scope.

| Inquiry | Concrete decision and evidence needed |
|---|---|
| Q01 — Owner metadata shape | Choose the smallest capability/access-route concepts needed by representative command, skill, native API, optional utility and session tool. Demonstrate source-derived grammar and explicit unknowns without a duplicate catalogue. |
| Q02 — Actual Codex support | Test fresh-session tool attachment and the real client's resources/prompts/progressive discovery behavior. Use tools as a fallback when appropriate; do not assume a listed feature is exposed in this session. |
| Q03 — Direct versus MCP | Compare one unfamiliar-tool→context→targeted-action→watch→result task, including unavailable and partial cases. Retain direct execution if transport adds little practical value; discovery remains selected. |
| Q04 — Host contracts versus native documents | Identify which new fields are genuinely host-owned. Reuse generated native schemas/documents and test any adapter's declared loss; resolve the Python rule discrepancy below. |
| Q05 — CLI projection fidelity | Decide whether a new operator CLI is useful. If generated, fix/qualify missing/null/empty, nested unions, defaults, paths and error/exit behavior at the generator. |
| Q06 — Cancellation and reattachment | Challenge observer timeout, native-action cancellation, process drain, framework disconnect and resumed result reads independently. Preserve native handle/source authority. |
| Q07 — Output size and discovery freshness | Bound nested output as well as item counts; choose source-cut/cursor invalidation and cheap schema reuse. A stale index must not certify current availability or a complete negative. |
| Q08 — Causal usability | Select useful late-discovered examples outside Plans47/50/51/52 and test whether discovery actually changes a task's owner/tool selection. Avoid turning pattern counts into incident or productivity claims. |

[Plan52g](../plans/52g-agent-and-operator-workflows.md) remains the current scheduled workflow
owner. Its G01 already covers shared interaction meanings, G02 conditional MCP, G03 guidance,
G04 owned integration and G05 optional operator views. This review **broadens the discovery
universe** and supplies concrete framework/contract findings; it does not replace that plan or
mark those packages done. O01–O06 refine G01/G03; O07 develops G02; O08 supports G04; O09/O12
develop G03 and assembled52h use; O10/O11 can support G05/52f. A later plan must choose whether
to revise52g or create separately owned scope, preserving one owner for each accepted meaning.

Plan50's earlier no-adopt decision for an extra operations MCP remains historical evidence
under its tested circumstances. The expanded tooling surface and4.1.3 mechanisms justify
reconsideration, not retroactively declaring the earlier choice incorrect. Existing Plans47/51
need no reopening merely to expose their owners. Their errors/handles may need local adapter
changes only if selected new consumers reveal a concrete contract deficiency.

Migration should remove replaced ad hoc output parsing, duplicate command facts and obsolete
instruction lists when their replacements are usable. Do not retain compatibility dispatchers
or parallel editable schemas. Direct native routes survive because they serve independent
operator/diagnostic use, not as legacy shims for a universally mandatory transport.

## Rule and authority impacts

<a id="RC01"></a>
**RC01 — Clarify Python host-tool contract scope.** `.claude/rules/python.md` describes
msgspec/attrs package contracts and broadly bans Pydantic/Click outside parity. The current
pyproject import-policy comment says those historical dependency exclusions were lifted by
ADR-0066; current enforced imports reflect the narrower policy. Proposed resolution: clarify
that host-tool contracts may use Pydantic without replacing `python/pse`'s Rust-owned/generated
contract system. Preserve blueprint §21.5. If implementation changes the actual public Python
boundary/class systems, it requires the existing ADR/design route; this review grants no such
change. Ordinary dependency use is not itself a reason for an ADR.

<a id="RC02"></a>
**RC02 — Keep optional access and native authority explicit.** No rule should require tool use
or MCP for ordinary editing, planning or validation. Proposed instruction updates point briefly
to discovery and document effective launch/attachment. They must preserve phase authorization,
native workload admission, source ownership, scientific outcomes and existing qualification
timing. There is no new generic execution privilege or approval service.

<a id="RC03"></a>
**RC03 — Derive grammar; own only new discovery meaning.** Extend existing declaration owners
for capability purpose/effects/access links, deriving parsers/schemas/defaults rather than
copying them. Generated CLI/help/skill output follows the generator ownership rule; a semantic
generator bug is fixed upstream or in its owned derivation. This aligns with AP-04/DP-01 and
existing generated-path rules rather than waiving them.

<a id="RC04"></a>
**RC04 — Select new functional scope through the plan route.** The favorable FastMCP direction
does not override Plan52g's status or authorize installing a new service/plugin, replacing
parsers or editing architecture. Plan creation should select the actual target and address
any consequential boundary change through existing decisions. Proposed governance clarifications
remain proposals here; no rules were silently repaired during this review.

<a id="evidence"></a>
## Evidence and completed maintenance

**Implemented:** `just upgrade pydantic` moved only Pydantic2.14.1 and its required
pydantic-core2.50.1 in `uv.lock`; no Cargo packages moved. The latest release was checked against
[PyPI's Pydantic release record](https://pypi.org/project/pydantic/). Existing FastMCP/MCP/settings
constraints admit this version. This is an existing transitive dependency update; no new
first-party production Pydantic contract was introduced.

**Tested, baseline zero:** `scripts/pse-env --resource-class build -- just upgrade pydantic`
completed its family check with zero failures. The linked editable correctness extension was
restored using `just py-sync-native-correctness`; the actual compiled Python stub comparison
passed. This rebuild preserves the required lockfile provenance after a lock change; it is not
scientific/product qualification. The initial tools sync removed registration and a wrong-checkout
cached editable wheel was briefly installed and then removed before the supported rebuild; no foreign registration
was retained.

**Tested, baseline zero:**
`scripts/pse-env --resource-class build -- .venv/bin/python -m unittest scripts.tests.test_semantic_navigation scripts.tests.test_codex_session scripts.tests.test_workspace_context`
ran12 controls with zero failures on Python3.14.8. These cover configuration/discovery behavior,
not all FastMCP clients or whole-product workflows.

**Tested, bounded library probe:**
`scripts/pse-env --resource-class build -- .venv/bin/python docs/functional_reviews/probes/agent-tool-discovery-2026-10-24.py`
passed its expected assertions with FastMCP4.1.3/MCP2.3.0/Pydantic2.14.1/core2.50.1. It verified
TypedDict strict-count rejection (`int_type`, location `limit`), exported constraint schema,
an actual in-process MCP call/result, refusal on both input/body error paths, receiver refusal of
invalid declared output, `MISSING` versus explicit null/empty, and generated-CLI forwarding
that loses null/empty. The final assertion documents that library limitation; it is **not**
acceptance of the generated CLI for mutation. The generated client's network/process transport
was stubbed for that forwarding assertion; the MCP server/client call itself was in-process.
The first exploratory run incorrectly expected malformed output to return a normal error;
the actual receiver exception corrected that assumption.

**Tested, baseline zero:** an inline fixture using
`scripts/pse-env --resource-class build -- .venv/bin/python -` called the existing
`scripts.semantic_navigation.request(..., fixture=True)` adapter with `get_document_symbols`
on an isolated two-line Python file. It found the `square` function through actual mcpls0.8.0
and pinned Pyrefly, negotiated MCP2026-07-28, and completed both server shutdowns with zero
failed assertions. This is a real subprocess-client compatibility check after the upgrade,
not attachment of the proposed interface to the current Codex session.

**Interface-checked:** exact installed FastMCP source for `FunctionTool`, server error handling,
schema caching, regex search and CLI generation; tagged primary Typer source; supplied library
capability overviews and versioned documentation. This does not establish installed Typer use,
FastMCP facade attachment in this Codex session, native cancellation under a future facade,
or correctness of arbitrary user-defined validators/serializers.

`just ready` completed with zero failed checks during maintenance, while its initial doctor
reported the then-missing extension and deferred historical fixture cleanup warnings. After
restoration, `just doctor-json` reported the native extension imported and all listed checks
successful. Historical cleanup refusals were not repaired or treated as qualification here.
The root `just turn-end` also completed its ADR-index and formatting checks with zero failures;
its finalizer reported two further historical fixture cleanup refusals. These remain excluded
from the completed maintenance claim rather than being presented as resolved.
No full Rust/Python functional suite, broad hygiene campaign, production performance run or
new simulator qualification was required for this review. Future delivery must supply the
actual consumer and semantic acceptance described above; this publication supplies proposals
and bounded evidence only.
