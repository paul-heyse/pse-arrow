# Version and source evidence

Observed 2026-10-09. Checkout baseline supplied by the coordinator:
`d0f2c41818a34539a910654dfea4760771603f45`; initial `git status --short`
was empty. Commands were invoked with `scripts/pse-env --`; no compilation or
test execution was requested.

| Command | Result | Evidence scope |
|---|---|---|
| `just --version` | passed; 1.58.0 | Installed executable |
| `just --help`, `just --usage check-package` | passed; dump JSON, show, usage supported; `pkg [args...]` | CLI interface |
| Python subprocess calling `just --dump --dump-format json`, `json.loads` | passed; recipe fields include attributes, body, dependencies, doc, name, parameters | Actual lightweight discovery; no recipe body executed |
| `cargo --version` | passed; 1.101.0-nightly (3d7cf6e93 2026-09-25) | Workspace-selected Cargo |
| `cargo check --help` | passed; package selection, JSON messages and timings | CLI interface |
| `cargo nextest --version` | passed; 0.9.146, commit 8af696ddcce8fff2962d6a5168b6d138b8616a35 | Installed runner |
| `cargo nextest list --help` | passed; full/binaries-only, json/json-pretty; explicitly builds tests | CLI interface; no tests listed |
| `rust-analyzer --version` | passed; 1.101.0-nightly (c1070d6 2026-09-28) | Selected LSP binary; not an ra_ap crate version |
| `rust-analyzer-mcp --version`, `--help` | passed; 0.4.0 and feature/config/workspace arguments | Bridge executable exists; no MCP connection |
| `rustc -Z help` | passed; self-profile, time-passes and text/json format | Pinned compiler interface; no profile |
| `rustdoc --help` | passed; private items and output-format advertised | Experimental JSON from documentation/source; no extraction |

No product test baseline was executed: **not_run**. No live MCP/database call:
**not_run**. Endpoint availability, deployed binary version and effective
Codex/plugin state are owned by the root's host investigation. The coordinator
shared these observations during this worker's turn: `codex-cli 0.161.0`,
user bridge registration to `~/.cargo/bin/rust-analyzer-mcp`, analyzer and
catalog disabled by project configuration, no analyzer/catalog/SurrealDB tool
metadata in the root session, and owned deployed SurrealDB binary 3.3.0 with a
recorded protocol readiness and an owned listener. The readiness field derives
from the runner's recorded proof/hash, not a fresh WebSocket query. These are
attributed host observations, not
worker-executed probes; live authenticated HTTP MCP exercise remains with root.

## Inspected local source

- `Cargo.toml`: exact local SDK 3.3.0, `protocol-ws,rustls`, defaults off.
- `vendor/surrealdb/src/method/query.rs`: `IndexedResults::check/take_errors`.
- `crates/pse-operations/src/canonical.rs`: remote `Ws` consumer, bounded
  request config, protected query retry consumer.
- `rust-toolchain.toml`: nightly-2026-09-29 and analyzer/source components.
- `rust-analyzer.toml`, `.codex/config.toml`, `docs/dev/agent-environment.md`,
  `docs/dev/validation-assessment.md`, `justfile`: repository integration and
  selection contracts.
- Installed bridge source at
  `/home/paul/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rust-analyzer-mcp-0.4.0/`:
  `src/main.rs`, `cli.rs`, `settings.rs`, `mcp/tools.rs`, `mcp/handlers.rs`,
  `lsp/client.rs`, `config.rs`; upstream repository
  [zeenix/rust-analyzer-mcp](https://github.com/zeenix/rust-analyzer-mcp).
- Shared `neo4j-surrealdb` skill: `surrealdb/content/catalogs/mcp.md`,
  `content/capabilities/server.frontends.md`, pinned corpus
  `content/corpus/server/surrealdb/server/src/{cli,ntw}/mcp.rs`,
  `content/corpus/surrealdb-mcp/{Cargo.toml.orig,src/service.rs}`.
  Historical probes SB086/SB087 are explicitly not rerun.
- Shared `rust-code-model` skill: `content/topics/00-map.md` and SKILL.md;
  layer boundaries and older exact source pins, not current workspace proof.

## Current documentation fetched

Context7 resolve-library-id preceded query-docs for SurrealDB, nextest, Cargo,
just, rust-analyzer, Codex and rustdoc. No version-specific SurrealDB 3.3 ID
was returned; current docs contain mixed-version SDK snippets, so exact SDK
source decides compatibility.

| Context7 selection | Queries |
|---|---|
| `/surrealdb/docs.surrealdb.com` | Existing self-hosted MCP; Rust SDK statement errors and typed bindings |
| `/websites/nexte_st` | JSON listings/filtersets; binaries-only preparation |
| `/websites/doc_rust-lang_cargo` | JSON diagnostics; build/compiler profiling |
| `/casey/just` | JSON dump, recipe usage and parameters |
| `/rust-lang/rust-analyzer` | Initialization/build scripts/proc macros; TOML configuration |
| `/openai/codex` | MCP/plugin enablement and layering |
| `/websites/doc_rust-lang_stable_rustdoc` | Experimental JSON/private-item extraction |

Exact version source fetched to settle embedded/server MCP applicability:
[3.3.0 HTTP route](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/server/src/ntw/mcp.rs)
and [3.3.0 stdio command](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/server/src/cli/mcp.rs).
An initial fetch under the old `src/cli` layout returned 404; corrected to
`surrealdb/server/src` and fetched successfully. No absence claim derives from
that failed path.

Official OpenAI Docs MCP search/fetch retrieved
[Codex MCP](https://developers.openai.com/codex/mcp) and
[config reference](https://learn.chatgpt.com/docs/config-file/config-reference).
Current docs can exceed the installed client's contract; the coordinator owns
the installed-version compatibility check.

External option coverage is deliberately bounded:
[official local plugin](https://github.com/surrealdb/ai-codex-plugin), its
[manifest](https://github.com/surrealdb/ai-codex-plugin/blob/main/plugins/surrealdb-local/.mcp.json),
and third-party remote bridge
[nsxdavid/surrealdb-mcp-server](https://github.com/nsxdavid/surrealdb-mcp-server)
README/package source. Existence is established; no installation, pinned
compatibility test, exhaustive candidate survey or security audit occurred.
