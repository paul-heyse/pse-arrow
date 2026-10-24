# Structured agent capability interface

The canonical FastMCP entry point provides exact domain/workflow navigation, selected tools and
owned instructions. It is optional access to existing operations, not a requirement to use tools.
Native source/status owners, scientific contracts and execution admission retain their authority.

## Start and attach

Prepare this checkout's locked development dependencies explicitly with
`uv sync --locked --no-install-project`. This installs host tooling without compiling the native
extension. Then launch `just capabilities`, or configure an MCP attachment to run this checkout's
`.venv/bin/python` with arguments `-m scripts.capabilities` and its working directory set to the
checkout root. Use an absolute interpreter path. Each attachment starts its own local STDIO process;
there is no daemon, HTTP listener or Docker deployment.

For a host accepting environment JSON, the connection shape is:

```json
{
  "mcpServers": {
    "pse-capabilities": {
      "command": "/absolute/checkout/.venv/bin/python",
      "args": ["-m", "scripts.capabilities"],
      "cwd": "/absolute/checkout"
    }
  }
}
```

Substitute the selected checkout; never borrow another worktree's interpreter. Codex's configuration
uses the same command/args/cwd under `mcp_servers.pse-capabilities`; see the
[official MCP configuration guide](https://learn.chatgpt.com/docs/extend/mcp). Configuration alone
is not evidence of attachment or exercised behavior. Language and selected database operations use
owned lazy connections inside this endpoint, retaining their native identity and admission owners.

Ordinary Codex startup in this trusted project now reads the checked-in `.codex/config.toml`
attachment: `just --quiet capabilities`. Just finds the checkout's recipe from nested directories
and that recipe chooses its own absolute interpreter and repository working directory. No fixed
machine path or global server registration is needed. Startup has a 30-second timeout; explicit
tools have a 1,800-second host timeout to accommodate native preparation. Native timeouts,
cancellation and drain remain operation-owned; a host timeout does not prove settlement.
The inherited standalone Rust attachment is disabled for this project with a complete disabled
transport declaration, so hosts without that inherited entry also have valid configuration.
Other global tools and approval policy remain inherited.

After changing attachment configuration, restart the Codex client/extension or its MCP connection
and inspect `/mcp`. `codex mcp get pse-capabilities --json` verifies the resolved configuration;
seeing the eleven bootstrap tools and calling `capabilities_workflows` verifies actual access.
An already-running session may retain its old server inventory. `just codex-session` remains the
explicit route for selected database/language overrides; it is no longer required just to attach
the canonical endpoint during ordinary checkout use.

This fixed read/control launch intentionally does not enter workload admission, synchronize packages,
prepare a compiler, start a store or import the native extension. It does not accept an arbitrary
executable. Explicit actions enter their existing environment/admission owner when invoked.

### Conditional composition, tasks and telemetry

Ordinary serving does not register CodeMode, the task extension or an OpenTelemetry SDK/exporter.
Their packages are available to retained manual acceptance drivers through development dependencies;
installation is not activation. [Plan52o's pause handoff](../plans/52o-conditional-composition-and-protocol-tasks.md#execution-checkpoint--pause-handoff-2026-10-24)
records current feasibility evidence and remaining acceptance. Its drivers under `scripts/tests/`
are `capability_composition_inquiry`, `capability_tasks_inquiry` and `capability_telemetry_inquiry`.
Run one explicitly with the checkout interpreter using `-m scripts.tests.<module> --output <fresh-directory>`
through `scripts/pse-env --`; they are not part of automatic unittest discovery.

The tested task projection uses authentic retained native fixture records and needs no Docket queue,
but it is not production task support or evidence of continued native execution after a crash.
Current native references/readers remain the supported recovery route. Existing interface metadata
preserves active trace/span IDs when supplied while treating export and end-to-end continuity as
unverified. Native result/admission/drain never follows from a successful span or task response.
A separate thorough OTel review will precede deployment; the in-memory SDK probe starts no Collector
and establishes no Rust or database propagation.

### Selected native overrides

`just codex-session` configures this attachment for the selected checkout and disables inherited
standalone Rust, Python and database attachments that actually exist in inherited configuration.
It never creates disabled transportless placeholder entries. Its `--require-*` options check local preparation;
selected calls establish live readiness. `--without-python`, `--without-rust` and
`--without-database` omit those resources. Equivalent attachment environment selects exact
`PSE_CAPABILITIES_LANGUAGES` (`python,rust`), `PSE_CAPABILITIES_DATABASE` (`1` or `0`) and
`PSE_CAPABILITIES_DATABASE_STATE` (an explicit existing deployment state). Selection does not start
a database, change credentials or provision a project.

## Discover, describe and select

Bootstrap exposes `capabilities_domains`, `capabilities_workflows`, `capabilities_describe`,
`capabilities_select`, `capabilities_reset`, `capabilities_invoke_read` and
`capabilities_invoke_action`, plus `list_resources`, `read_resource`, `list_prompts`
and `get_prompt`: eleven bootstrap tools in total.

1. Browse domains or workflows, then pass an exact ID to list their members. Results are ordered
   and paged with `offset`, `limit`, `total` and `next_offset`.
2. Describe an exact capability. `include_schema=true` returns the actual callable contract where
   a binding exists. A native route or deferred adapter has no fabricated tool schema.
3. Select exact `domains`, `workflows` or `capabilities`. A workflow selects its declared default
   steps; use `workflow_steps={"qualify-change":["prepare"]}` to add a named optional step.
   Browse its actual step declarations rather than assuming every family is selected.
   Selecting a family also reveals its
   actual bound operations; describe it to see those operations and retained native routes.
   Their union replaces the prior selected
   set; repeated identical selection is idempotent. Unknown IDs change nothing. Reset restores bootstrap.
4. Read the compact receipt's `instruction_resources` at their `pse://guidance/...` URIs, then call the
   operation. Descriptions retain critical interpretation/effect constraints; reading guidance is
   not an acknowledgment or admission gate.

Selections change presentation only, never checkout, project, store, credentials, permissions or
execution ownership. Select/reset are read-only with respect to the external world. Two attachments
have independent processes and selections; sharing one STDIO process between clients is unsupported.
Visibility is not a permission boundary.

The inspected Codex host kept its initial callable inventory after successful selection. Use
`capabilities_invoke_read` with the exact selected capability and an `arguments` object matching its
real described schema when an ordinary revealed tool is absent. For example:

```json
{
  "capability": "workspace_task_context",
  "arguments": {
    "task": "docs/plans/52-agent-effectiveness-and-engineering-workflows.md#CAP01",
    "paths": ["scripts/capabilities/server.py"]
  }
}
```

This calls the same registered tool through FastMCP's normal visibility, strict validation,
middleware and result path. The read invoker accepts actual read-only bindings. Use
`capabilities_invoke_action` for an explicitly selected action with the same exact
`capability`/`arguments` shape; it advertises effectful, potentially destructive behavior.
Both refuse hidden tools, bootstrap recursion and arbitrary command dispatch. An action's own
schema may accept a literal launch argv where that is the native diagnostic operation's purpose.
Neither invoker creates another operation implementation or bypasses native admission. Hosts
supporting dynamic callable refresh can use ordinary revealed calls directly.

The four content tools project the same native resources/templates and prompts, including their
visibility checks. `pse://capabilities/{capability}` describes an exact loaded capability with its
actual schemas. Selecting a workflow reveals its `workflow_<id>` prompt; reset hides it again.
Prompts contain compact capability and instruction-resource references with source digests.
Read the referenced resources for complete instructions; prompts do not duplicate or truncate guide
bodies. Planning-skill retrieval supplies instructions and never executes that skill's action phase.

Selection receipts include effective IDs, workflow steps, the loaded revision, complete instruction
references and source/package digests. They carry no instruction bodies. Repeated selection does not
resend bodies; deliberate resource reads return complete captured content. Owned local process skills
also expose a `supporting_manifest` URI and exact supporting-file resources from the same package cut.
Read the manifest before choosing a file. Files retain their original bytes and MIME type; package
changes require a fresh endpoint. Capture is bounded to 128 KiB per file, 64 files and 1 MiB per package,
with traversal bounded to 128 entries, 64 descendant directories and depth eight. Symlinks and excess
packages refuse rather than silently truncate. External linked library skill packages keep their
existing native access route. Retrieval never installs, trusts, activates or executes a skill.

FastMCP's Skills providers are not composed alongside this owner: their eager scan/live-read lifecycle
does not preserve this endpoint's lazy captured cut. Revisit if a real consumer needs native Skills
negotiation and that lifecycle can preserve the same contract. Protocol completion is also unselected:
current prompts have no parameters and the actual Codex path uses explicit schemas and exact resource
IDs. Revisit for a concrete completion consumer with a useful prompt/template parameter; ordinary tool
arguments do not acquire completion from `@mcp.completion`.

Ordinary sessions retain the host's approval policy. Action invocation remains explicitly effectful.
The bounded host acceptance driver configures a session-local approval for its one authorized
`capabilities_invoke_action` request; it does not change global configuration or relabel the action
as read-only. Run that explicit journey with
`scripts/pse-env -- .venv/bin/python -m scripts.tests.capability_codex_check --output build/capabilities-52k/NEW-NAME`.
It starts a fresh Codex process, invokes one exact `pse-ids` unit, and independently checks its original
native result. Do not edit execution inputs or regenerate code while it prepares that invocation.

## Authority, coverage and incomplete results

Exposure declarations in `scripts/capabilities/` own domain/workflow membership, guidance and
integration metadata. Tags and views derive from those declarations. Actual FunctionTool schemas
own callable host shapes; Rust registries/catalogues remain the source of native schemas. This
interface introduces no scientific Python field mirrors or semantic/ranked discovery.

The [generated complete reference](../generated/capabilities/reference.md) derives from those
same composed definitions, including hidden operations, defaults, resources, templates and prompts.
`just codegen` publishes it; `just codegen-check` compares without writes. The dedicated generated
directory belongs to this generator, independently of the Rust Markdown generator. No Cyclopts CLI
is adopted: revisit when an actual terminal/non-MCP consumer needs access beyond native commands
and this endpoint. A future CLI must qualify complete selection and absent/null/empty fidelity.

Callable operations read task context, exact native concepts, declared producer relationships,
recipe descriptions, library evidence, coverage correspondence and the current entry-point inventory.
Environment, workload, assessment, storage and native result readers retain their original owners.
Explicit actions adapt scoped Rust tests, archive/replay, diagnostic experiments, compilation,
assessment, contract generation and documentation builds. `workspace_inventory`
explicitly inspects current public recipes and bounded source candidates; startup and ordinary
catalogue browsing do not run that inspection. Recipe outer grammar comes from Just; leaf-owner
excerpts explicitly do not constitute a complete argument schema.
The Just declaration and leaf excerpts are separate source observations; their combined source
coherence is explicitly unverified rather than an atomic snapshot.

The catalogue also describes retained native routes, internal helpers and deferred adapters with
their work owner and reason. Classification completeness is distinct from migration, readiness and
functional qualification. New unknown entry points remain explicitly unclassified until assessed.
Linked shared-library skills outside the checkout retain native access rather than promise a
checkout-contained resource. Owned process skills are served as lazy instruction resources; retrieving
one does not activate or execute its workflow.

To add a capability, keep its actual handler in the module that owns its reason for change, then
return its `Capability` declaration and real `Tool` from that owner's binding factory. Compose the
factory once in `server.py`. Declare relevant guides, structural `requirements` and actual
`worker_calls`; reuse each fixed `WorkerCall` in dispatch so execution and restart fencing agree.
Fixed workers declare any non-Python execution inputs in `Worker.sources`. Add workflow membership
only in the owning `Workflow.steps`, not again in each binding. Tags, reverse membership, navigation
and the reference derive from these declarations. Run the targeted behavior/owner controls and
regenerate the reference. Pure contract DTOs belong outside binding composition or effectful workers.
Source-only `python -m` ownership resolves modules, package initializers and actual `__main__.py`
without importing them; unresolved/dynamic recipe forms remain attributed leads rather than proven
callable owners. No custom provider is needed just to register an ordinary local operation.

Host context, concepts, relationships, recipe, inventory, environment, source corpus, document scope,
run, assessment, watch, storage, library and coverage readers publish named response schemas.
Their FastMCP structured output is `{"result": <typed answer or detail refusal>}`. Bootstrap,
specialist and native operations retain their described shapes; inspect the actual output schema.
`answer.population` distinguishes complete, partial and unavailable source populations;
`answer.detail` distinguishes selected from omitted optional detail. Deliberately excluded work
and unknown knowledge are separate from missing source populations. A successful reader never
upgrades the observed workload's outcome, assurance or drain.

Selected excerpts, subject pages and artifact tails reach their original reader before text/detail
construction. Global counts and original source attribution remain explicit. Oversize answers return
a typed `detail-refused` outcome with no substituted partial payload. Large/raw detail stays with
selected source owners; narrow or page the request or use the original protected export.

`workspace_concepts_query` takes one owner-defined tagged `request`. For example:

```json
{"request":{"concept":"document","identity":"docs/README.md"}}
```

Document/decision discovery also supports conjunctive metadata facets. Research questions require
their identity, origin and question; evidence/diagnostic queries require an explicit
`selected_reference` naming the original adapter and path. Describe the capability to obtain every
alternative and its required fields. Missing, null, empty and inapplicable fields retain their
declared meanings; there is no compatibility argument bag or similarity fallback.

## Loaded contracts and lifetime

The endpoint captures one catalogue/schema/handler revision from known local serving sources,
including explicitly declared fixed child-worker implementations and non-Python execution inputs
such as Just recipes. Lightweight contracts and operation handlers have separate source ownership;
pure schema inspection does not inherit an execution worker just from its name. Relevant source changes refuse dispatch before
handler effects and report `pse.capabilities.restart_required`; bootstrap still describes the loaded
snapshot and changed sources. Restart after changing serving code or exposure declarations.
An unrelated model/document edit does not invalidate every operation. This bounded source cut does
not establish arbitrary third-party dependency or deployment freshness.

Capture precedes composition and is verified afterward. Local imports remain conservatively traced,
including package initializers and absent candidate paths whose later appearance changes resolution.
Restart obtains a coherent endpoint; it never replays a prior action or hot-reloads a running handler.

Guidance bodies load only when selected, with a bounded complete UTF-8 read and an identity check
against startup. A changed captured guide retains its complete compatible snapshot and reports
restart; a changed uncaptured guide refuses new content. The change affects that guide rather than
poisoning unrelated instruction selection. It never mixes newly edited instructions with old
handlers. Source-inspection tools still read current selected sources under their native freshness
contracts. No whole-repository per-read hash or automatic rebuild is involved.

Selection commits one complete FastMCP visibility-rule replacement after guidance preparation.
Notification failure is distinct from a committed state change. Disconnecting a client tears down
its owned STDIO process; presentation reset neither cancels nor replays native work.

## Native schemas, actions and protected reads

`capability_readiness(capability)` is a passive selected observation of owner-declared requirements
and existing preparation receipts. It reports independent facts for declared requirements, recorded
preparation, compatible import, selected service, language provider and exercised operation. An old
receipt records preparation; its currency stays unknown. Missing metadata stays unknown rather than
becoming a universal ready/not-ready answer. Browsing and passive checks do not compile, import the
extension, start a service or publish files.

`capability_readiness_check(request)` explicitly checks selected `import-compatible`,
`selected-service` or `language-provider` prerequisites through their original owners. Language checks
need a language and file path. Import checks use the selected interpreter, binary origin/digest and
required features; they do not build a missing extension. A toolkit service check does not establish
availability of a request-specific canonical deployment. Checking prerequisites never qualifies the
operation itself.

`python_extension_dev_stage` and `python_extension_native_stage` accept a chosen `stage`, `profile`
and `validated` selection. `environment` ensures locked dependencies, `prepare` builds/installs or
reuses the owner's matching artifact, `stubs` publishes, `stubs-check` compares without publication,
and `admit` checks actual imported bytes/features. These are separate explicit actions using
`scripts/python_extension.py`; there is no second builder or implicit all-stage sync. Owner outcomes
distinguish stage status, exit, artifact identity, receipt and publication. Native provider setup
remains in the selected environment owner. Correctness checks retain `validated=true`.

`native_contract_schema` reads the generated catalogue's exact slug and input/output direction.
Its optional original JSON Pointer selects bounded detail; local references retain the original
root through `root_ref`. The special `authoring` selector describes applicable logical source
sections, without pretending that raw TOML, `.pse` or Parquet packages are JSON objects.
Native tools expose complete generated schema resources and receiving functions in their metadata;
the original native codec and receiver still perform admission before effects. A schema-shaped
request may fail contextual admission. A changed loaded/installed schema fingerprint refuses
the call rather than mixing interface generations.

Native scientific reads require an existing compatible extension and its explicitly prepared
linked import environment. Prepare with `just py-sync-native-correctness`, then attach from
`scripts/pse-env --native=solver,klu,isolation,uno,petsc -- <endpoint-command>` when those providers
are needed. Read operations do not build providers. The two explicit source/preparation actions
may prepare their native import prerequisites through the environment owner; their metadata
discloses this effect. Supplied-only source/bundle inspection does not provision a database.

The explicit `rust_test_run` action also prepares the existing test worker through its native
provider environment (`solver,klu,isolation,uno,petsc`). Even a pure selected unit can require this
worker preparation; its cost and prerequisites are distinct from the selected test population.
The original runner owns selection, force-validation, source/configuration fencing and results.

Ephemeral source admission ends with its child lifetime and does not return a reopenable model
handle. Durable reads name the exact canonical deployment/revision/run/attempt and preserve native
source protection. `native_result_read` preserves exact bounded Arrow IPC bytes. Larger windows
must be narrowed using the same protected native identity.

`native_result_inspect` accepts the generated `ResultInspectionRequest`: exact run, attempt,
relation, half-open `start`/`end`, optional `expected_manifest` and a tagged `view`. A
`{"kind":"schema","offset":0,"limit":8}` view returns original field descriptors, metadata,
manifest and recorded-row population without pulling data batches. A
`{"kind":"values","fields":["value"],"maximum_rows":4}` view returns selected scalar cells plus
all original non-value context columns, including identities, units, basis and quality where the
original relation supplies them. Inspect the schema first for exact field names. Both views retain
the original terminal selection; scientific usability stays unknown rather than being inferred.

Readable cells preserve original widths through their field descriptors and exact floating bits,
with explicit null, finite/nonfinite and unavailable distinctions. Unsupported nested/storage or
oversized cells report unavailable with `readable_complete=false`; they are never normalized into
zero or null. Pages retain original coordinates, including coverage holes. Unknown fields, changed
manifests and finite response/context limits refuse explicitly. The native protected reader still
validates original batches before selected-cell transport; this is not a projected-storage decode
or a measured speed claim. Exact Arrow IPC remains the complete scientific data path.

Read cancellation detaches the observer; its protected native operation finishes under its owner.
Action cancellation signals only the owned helper and waits for its native drain. Diagnostic
attachment never makes the observed subject a cancellation target. An interrupted or lost response
does not prove rollback or terminal settlement: inspect the returned native output/invocation and
its independent terminal report before retrying. Borrowed invocations retain their original
outer terminal owner. No background task scheduler or implicit retry is introduced.

Attached fixed workers publish bounded owner-issued stage/reference snapshots. A caller supplying a
progress token can receive foreground `Context.report_progress` notifications with an increasing
sequence and unknown total; this is not a completion percentage. Snapshots coalesce and notification
loss does not change the native outcome. References appear only after their original owner issues
them. `rust_test_reference_read(output)` reads an actual `invocation-reference.json`, validates its
registry identity and leads to `product_run_read`/`workload_watch` before final action return. Absence
means not observed, not unstarted, failed or safe to retry. This original-owner read is the fallback
for hosts that do not render notifications. Attached cancellation still signals and physically drains
owned work; progress introduces no detachable task contract or new outcome store.

Language resources connect only on a selected call, retain checkout/tool/configuration identity,
and close with the endpoint. Source cuts and UTF-16 positions remain explicit. Rename retains
the complete preview behind an endpoint-owned `preview_sha256`; `edit_preview_read` reads exact
before/after text chunks using Unicode codepoint offsets. Apply and reconcile take that exact
digest, preserving complete source bytes independently of response bounds. Handles expire at
endpoint close or successful explicit release. Retention refuses excess at 32 previews or 64 MiB
without evicting earlier handles. `edit_preview_status` reports retained counts/bytes and optionally
one digest's availability, admitted work and reconciliation obligation. `edit_preview_release`
reclaims only a settled exact handle. Unknown, foreign and already released well-formed digests all
return unavailable. Release refuses during admitted work or unresolved reconciliation; an interrupted
apply keeps its original digest usable by `edit_preview_reconcile`. Reconciliation must classify all
edited/basis paths, including diverged or missing files, before the obligation clears. Classification
does not mean repair, rollback or successful apply. Release never changes source files, and presentation
reset neither releases handles nor increases capacity.
Protected apply and reconciliation report partial edits, and generated/PyO3
declarations are not claimed to be migrated automatically. Database selection validates the
existing owned listener, source-built generation and credentials before opening its pinned MCP
connection. Root VIEWER credentials restrict exploratory queries; canonical project operations
retain their separate native admission. Database tooling is not agent memory.

## Interface diagnostics

Input and selection refusals expose stable codes, stage, safe locations and an owner-appropriate
remedy. Public structural paths retain schema-declared keys and indexes; arbitrary map keys and
variant values are redacted. Native decode reports retain header/body/trailing phase, JSON category,
line/column and tracked path precision, separately from authored-source spans. Related/context
counts and authentic native recovery identities survive explicit wrapping. Supplied messages,
credentials and arbitrary exception text are not echoed or parsed to invent classifications.
An issued recovery identity means inspect that owner's unsettled work; it grants no automatic retry.

Tool, resource and prompt responses carry bounded `_meta["pse.observation"]` correlation alongside
their unchanged native content and owner metadata. Nested canonical invocation shares the same
request correlation. Known native invocation/result fields contribute authentic reader references;
missing identities are not inferred. Follow those references through the original result reader to
distinguish selected subjects, workload exit, frozen assurance and owned drain.

Structured stderr records registered component names, loaded revision, hashed request identity,
failure stage and interface elapsed time. It excludes arguments, raw URI parameters, credentials,
native paths and exception text. Interface return, transport delivery, native outcome and native
timing remain separate facts. A detached observer reports detachment without cancelling its subject.
Existing recording OpenTelemetry spans receive correlation attributes; no exporter, trace store or
end-to-end Surrealism continuity is implied. Unavailable tracing and unobserved transport delivery
are explicit. This is interface diagnosis, not performance qualification or a second evidence store.
