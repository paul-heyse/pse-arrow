# Capabilities for Codex first Linux work

## Evidence boundary

**Interface-checked:** installed command help, repository contracts, installed
bridge source and exact upstream source where listed below. The JSON recipe dump
was actually read and parsed; this is discovery evidence, not product testing.
No capability here is newly **Tested** for scientific correctness or **Measured**
for agent effectiveness. Existing shared-skill probes are historical library
evidence under their own conditions, not new qualification of this checkout.

Available means a binary or source API exists; configured means a registration
exists; exposed means the running session has its tool; exercised means a call
completed. Those states are not interchangeable. See the root's host evidence
for configuration and endpoint observations beyond the tracked files.

## Existing SurrealDB instance

The SDK is exactly `surrealdb = 3.3.0`, from `vendor/surrealdb`, with defaults
off and `protocol-ws,rustls`. It is a local direct source path, not an unmodified
crates.io artifact. `crates/pse-operations/src/canonical.rs` uses
`Surreal::new::<Ws>` with the repository's bounded-request configuration;
`vendor/surrealdb/src/method/query.rs` exposes `check()` and `take_errors()`.
The typed canonical store remains the consumer for scientific reads,
publication, leases and transactions. Generic database tooling does not carry
those application contracts merely because it sees the same records.

The following conclusions use exact [v3.3.0 server source](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/server/src/ntw/mcp.rs),
not an assumption that today's embedded MCP documentation applies:

- The binary has a Streamable HTTP `/mcp` route. Its service receives the existing
  `AppState.datastore`; it does not create another database. HTTP route capability
  selection can refuse MCP. Authentication and MCP session behavior remain a
  deployed endpoint check, not proof from CLI version alone.
- [The v3.3.0 stdio command](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/server/src/cli/mcp.rs)
  initializes its own datastore and an owner session. It is not a demonstrated
  remote attachment command. Do not substitute it for a connection to the
  active instance or open that instance's store directory in another process.
  The shared skill's SB087 records stdout logging interfering with stdio unless
  `--log none` is selected; that run was 2026-10-05, not this investigation.

The pinned skill's MCP catalog records fourteen data tools: `create`, `delete`,
`gql`, `graphql`, `info`, `insert`, `list`, `query`, `relate`, `run`, `select`,
`update`, `upsert`, `use`; SB086 previously exercised `/mcp` and the deny-route
control against the skill's pinned 3.3.0 image. These include writes and generic
queries. An allowlisted query tool can still execute writes; its name is not a
read-only guarantee. A database principal's permissions and operation intent
must match the consumer. Namespace/database context is part of the request or
session contract, not a global default to infer. The skill's SB129 records
VIEWER writes returning `[]` without an error: absence of a tool error is not
proof of scientific write admission.

The pinned MCP service accepts 2026-07-28, 2025-11-25, 2025-06-18 and
2025-03-26 protocol revisions. For a conventional handshake, initialize with a
supported legacy revision, retain its returned session identifier, notify
initialized, then request tools/list with the same credentials. Data tool scope
precedence is `namespace/database` arguments, `surreal-ns/surreal-db` headers,
handshake `use` state, then configured defaults. An explicit tool scope derives
a session without altering later calls' `use` state. HTTP Basic authentication
scope (`Surreal-Auth-NS/Surreal-Auth-DB`) is distinct from selection headers.
The initialized session is bound to its authenticated subject, and later HTTP
requests, including tools/list, must retain matching credentials. Source:
pinned `surrealdb-mcp/src/service.rs` and `auth.rs`, server `ntw/auth.rs`.

| Candidate | Actual agent task | Native/simple alternative | Added machinery and version gap | Benefit evaluation |
|---|---|---|---|---|
| Direct HTTP MCP to existing `/mcp` | Inspect declared tables, fields, indexes and bounded diagnostic records; investigate a publication failure | Existing `surreal sql`/HTTP inspection, plus application-owned inspection commands | Codex HTTP registration and credential/context setup; no new datastore or bridge process. Exact server source supports it; current endpoint initialization, authentication and tools still require exercise | Compare schema and diagnostic questions with shell inspection for correctness, calls, elapsed time and context volume; include a refused write and wrong-context control |
| Official `surrealdb-local` plugin | Same database inspection, with packaged setup guidance | Direct HTTP MCP registration | Marketplace/plugin manifest and skill lifecycle on top of the same endpoint; current main is unpinned and installed-client compatibility is untested here | Adopt only if packaged guidance materially improves the same tasks over direct registration |
| External remote SDK bridge | Expose narrower operations or attach through a supported remote SDK when direct MCP is unsuitable | CLI or application-owned typed inspection | One bridge executable/process, SDK compatibility, tool schemas, auth/session ownership, tests and maintenance. Third-party bridges exist, but none was qualified against 3.3.0 here | Compare an exact pinned candidate against direct MCP; evaluate only capabilities or isolation the native endpoint cannot supply |
| Typed scientific adapter | A workflow needs PSE snapshot identity, codecs, leases or publication semantics rather than raw tables | Existing Rust/Python scientific operations | Application mapping plus an MCP adapter if a real repeated consumer warrants it. Generic SurrealDB MCP does not provide this meaning | Verify results agree with existing typed operations and that invalid identities/context are rejected |

The official [local plugin](https://github.com/surrealdb/ai-codex-plugin) connects
to the instance `/mcp`; its inspected manifest specifies URL and bearer-token
environment variables. Managed Cloud/account/billing tools and Agent Memory
turn-capture hooks are distinct capabilities and are not imported into this
self-hosted proposal. This research recommends no plugin installation or server
upgrade. `surrealdb-mcp` 3.3.0 itself is built around `Arc<Datastore>` and
`surrealdb-core`; embedding it in a separate adapter would bring core machinery,
not reuse the workspace's lightweight remote SDK automatically.

## Rust navigation and diagnostics

Installed `rust-analyzer-mcp 0.4.0` exposes eleven tool definitions in
`src/mcp/tools.rs`: hover, definition, references, completion, symbols, format,
code actions, rename, set workspace, file diagnostics and workspace diagnostics.
Rename returns edits and file operations rather than applying them. The
source's definition/reference tools, rather than text matches alone, are useful
for aliases, trait method resolution, macros and same-spelling symbols.
Neither the analyzer nor its bridge establishes numerical correctness.

The bridge launches `rust-analyzer` via PATH with a `~/.cargo/bin` fallback and
sets the workspace as its working directory. The rustup proxy can therefore
select the workspace's dated toolchain. Its CLI accepts `--features`,
`--no-default-features` and JSON/string `--config KEY=VALUE` overrides. Defaults
enable build scripts, proc macros, check-on-save and diagnostics. Consequently
navigation startup can entail Cargo activity; a diagnostics tool is not a
pure source lookup. The bridge tracks workspace readiness and flycheck
completion; incomplete diagnostics are not a zero-error verdict.

Tracked `.codex/config.toml` explicitly disables this user-level server for the
project. `rust-analyzer.toml` requests `pse-relations/force-validate` and
`cargo.targetDir=true`; current upstream documents TOML configuration as
experimental. Check the effective initialization options, feature graph,
target placement and native environment when evaluating the configured bridge.
Its built-in settings do not explicitly repeat those two repository settings.
This is an integration question, not a proven configuration failure.

**Proposed optional evaluation:** use the existing installed bridge for a
bounded caller/definition task before adopting more semantic machinery. The
consumer is an agent making a refactor or answering an implementation question;
compare it with `rg`, focused source reads and editor LSP. Added machinery is
the existing MCP bridge plus one analyzer child and its workspace preparation,
configuration and lifecycle. Measure time to a correct answer, omissions,
startup overhead and source-reading volume. Include macro/trait/alias and
ordinary direct-call cases. The root owns launch and lifecycle decisions; this
worker did not enable or launch it.

Other useful native layers remain available even without a current automated
consumer:

- Cargo metadata answers packages, targets and dependency/feature graph context;
  it does not answer symbol meaning. Existing `metadata` and offline metadata
  recipes avoid a new index service. `--no-deps` deliberately omits the resolved
  dependency graph.
- Rustdoc HTML/JSON describes API signatures, generics, docs and impls; JSON has
  no function bodies. Private-item and feature selection affect visibility.
  JSON is experimental: inspect the output `format_version` and selected
  features rather than transferring the Rust code model skill's older format
  61/toolchain pin to this nightly. Generating a fresh workspace API index adds
  extraction, version handling and refresh costs; use existing library evidence
  or scoped docs first. [Rustdoc contract](https://doc.rust-lang.org/stable/rustdoc/unstable-features.html).
- The Rust code model skill's pinned `ra_ap_* 0.0.352` library family can provide
  in-process HIR facts; it is not the installed analyzer's version. A new HIR
  service adds project loading, sysroot/proc-macro setup and a weekly unstable
  API family. Consider it for repeated programmatic code-fact consumers if the
  bridge cannot serve their queries, not only when a consumer already exists.
- Cargo JSON compiler messages provide structured spans, error codes and
  diagnostics with package/target context. Preserve exit status and non-JSON
  stderr. `just check-package <pkg> --message-format json` uses existing recipe
  feature selection and environment. Parsing that stream adds much less machinery
  than a separate diagnostic authority.
- Installed Cargo exposes `--timings`; pinned rustc exposes `-Zself-profile`
  and `-Ztime-passes-format=json`. These answer build/compiler cost questions,
  not runtime solver latency. Profiling changes and raw compiler traces can add
  preparation/cache effects; scope a future experiment to the slow package and
  actual warm/cold workflow. Existing selected runtime measurement recipes are
  the alternative for scientific runtime questions. No profile was captured.

## Recipe and test discovery

Installed just 1.58.0 supports `--show`, `--usage`, `--dump --dump-format json`
and `--json`. The parsed dump contains recipes' bodies, dependencies, docs,
parameters and attributes. This already serves an agent seeking the exact
command and argument contract. Consume it transiently or filter it to a few
recipes; a duplicate persistent recipe catalog would add drift and regeneration
without a demonstrated gain. A small shell/tool wrapper is an option only if
repeated calls show direct filtered discovery is awkward.

Installed nextest 0.9.146 supports JSON full and binaries-only listings and
filtersets. `list` builds test binaries; full discovery executes them in listing
mode. It is not free metadata inspection. The repository already has
`PSE_NEXTEST_ACTION=list`, `just affected` and a JSON nextest-list recipe. Use
the owning recipe's features and environment for a scoped preview before
execution; correctness runs retain explicit force-validation. This worker
did not list or build tests. [Nextest listing contract](https://nexte.st/docs/machine-readable/list).

## Codex configuration and exposure

[Official Codex MCP documentation](https://developers.openai.com/codex/mcp)
distinguishes direct stdio (`command`) from HTTP (`url`) registrations. `enabled
= false` retains a disabled registration; tool allowlists are narrowed by
denylists. Stdio `cwd` and env forwarding configure launch context. Trusted
project configuration and session overrides participate in effective config.
Tracked project configuration disables the catalog and analyzer servers; it
does not demonstrate their absence from the user config or their exposure in
this running session. The coordinator reports installed `codex-cli 0.161.0`,
the user registration pointing at `~/.cargo/bin/rust-analyzer-mcp`, and no
analyzer, catalog or SurrealDB tools in the root session's available metadata.
It also reports the owned deployed server is 3.3.0 with recorded readiness;
that status alone is not a fresh WebSocket query. HTTP MCP initialization
and tools remain a separate exercise. The root's host inventory
is the owner of those observations.

Plugin MCP servers derive their transport from the installed plugin manifest;
`plugins.<plugin>.mcp_servers.<server>` controls enablement and policy.
Disabling a direct server, a plugin, a plugin-provided server and an individual
tool are different actions. Current docs/source are interface evidence only:
confirm accepted keys with the installed Codex version before implementation.
Catalog lookup and semantic tools also answer different questions: the first
finds library use evidence; the second resolves code facts. Repeatedly exposing
an unused server is not itself an improvement, and a disabled server is not
automatically an architectural deficiency.

No new resource, concurrency or runtime caps are proposed. Independent design
judgment should select capabilities by useful agent tasks, full added machinery
and measured benefit, rather than counting installed tools.
