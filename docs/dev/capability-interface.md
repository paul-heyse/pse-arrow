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
is not evidence of attachment or exercised behavior. Existing selected language/database connections
have their own native owners; their internal migration remains separate from this source-only delivery.

This fixed read/control launch intentionally does not enter workload admission, synchronize packages,
prepare a compiler, start a store or import the native extension. It does not accept an arbitrary
executable. Future effectful bindings must retain their native environment/admission owner.

## Discover, describe and select

Bootstrap exposes `capabilities_domains`, `capabilities_workflows`, `capabilities_describe`,
`capabilities_select`, `capabilities_reset` and the exact compatibility read invoker.

1. Browse domains or workflows, then pass an exact ID to list their members. Results are ordered
   and paged with `offset`, `limit`, `total` and `next_offset`.
2. Describe an exact capability. `include_schema=true` returns the actual callable contract where
   a binding exists. A native route or deferred adapter has no fabricated tool schema.
3. Select exact `domains`, `workflows` or `capabilities`. Their union replaces the prior selected
   set; repeated identical selection is idempotent. Unknown IDs change nothing. Reset restores bootstrap.
4. Read the returned relevant instructions or their `pse://guidance/...` resources, then call the
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
middleware and result path. It only accepts actual read-only bindings. Hidden tools, bootstrap
recursion, arbitrary shell/SQL/argv and non-read-only bindings are refused. It is not another
operation implementation or a workaround for native admission. Hosts supporting dynamic callable
refresh can use ordinary revealed calls directly.

## Authority, coverage and incomplete results

Exposure declarations in `scripts/capabilities/` own domain/workflow membership, guidance and
integration metadata. Tags and views derive from those declarations. Actual FunctionTool schemas
own callable host shapes; Rust registries/catalogues remain the source of native schemas. This
interface introduces no scientific Python field mirrors or semantic/ranked discovery.

The initial callable operations read task context, exact native concepts, declared producer
relationships, exact recipe descriptions and the current entry-point inventory. `workspace_inventory`
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

Read results retain original payloads and independent completeness/omission meanings. Nested
truncation, source gaps and response limits cannot become a complete result. Large/raw detail stays
with selected source owners; narrow or page the request when detail is omitted. Native source status
is not inferred from a process exit or a tool transport success. Expected owner refusals retain a
structured operation/code/cause; unexpected failures retain the server's masking policy.

## Loaded contracts and lifetime

The endpoint loads one catalogue/schema/handler revision. Restart after changing serving code,
exposure declarations or relevant instructions. Guidance bodies load only when selected, with a
bounded complete UTF-8 read and an identity check against startup. Changed guidance returns a captured
compatible snapshot or an explicit restart requirement; it never silently mixes newly edited guidance
with old handlers. Source-inspection tools still read current selected sources under their native
freshness contracts. No whole-repository per-read hash or automatic rebuild is involved.

Selection commits one complete FastMCP visibility-rule replacement after guidance preparation.
Notification failure is distinct from a committed state change. Disconnecting a client tears down
its owned STDIO process; presentation reset neither cancels nor replays native work. General native
cancellation/drain, specialist connection lifetimes, generated CLI fidelity and assembled agent
qualification remain at their existing implementation-plan owners.
