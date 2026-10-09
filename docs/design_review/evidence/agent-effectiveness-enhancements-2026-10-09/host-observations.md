# Host and interface observations

Observed on 2026-10-09 in the personal Linux checkout, initially clean `main`
at `d0f2c41818a34539a910654dfea4760771603f45`. These are point-in-time
observations, not portable defaults or product qualification. Credentials,
authorization headers and MCP session identifiers are not retained.

## Runtime configuration and exposure

**Interface-checked:** installed `codex --version` reports `codex-cli 0.161.0`.
Selective TOML inspection of the tracked `.codex/config.toml` and personal
`~/.codex/config.toml` shows:

- Project coordinator selection is `gpt-6-astra`, high effort; default
  subagent selection is `gpt-6.1-sol`, high effort. This records configuration;
  it does not establish which model a host-provided session actually selected.
- The project document limit is 65,536 bytes; `AGENTS.md` is 21,955 bytes.
  Size alone establishes neither instruction load nor effectiveness.
- User registrations include the installed `rust-analyzer-mcp` executable,
  Context7, OpenAI Developer Docs and Node REPL. Project configuration disables
  rust-analyzer and the library catalog. The effective CLI MCP inventory in
  [interface-observations.json](interface-observations.json) confirms those two
  disabled registrations. No SurrealDB registration appears there.
- Twelve user plugin entries cover GitHub, app tools, browser, visualization,
  documents, PDF, spreadsheets, presentations, template creation, Chrome,
  computer use and code review. Registration does not establish use.

The root's exposed tool metadata contained 360 entries, including 328
`codex_apps` entries, nine `codex_tui` entries, two Context7 entries and five
OpenAI Developer Docs entries. No rust-analyzer, library-catalog or SurrealDB
tool was exposed to this running root session. This is a session inventory;
it does not measure token overhead, startup delay, tool-selection accuracy
or whether an optional capability would help. A narrower coding session is
an evaluable option, not a demonstrated performance correction.

`scripts/pse-env --explain` identified the checkout `.venv`, checkout `target/`,
managed sccache wrapper and shared cache, native-extension installation and
finite existing placement. Its extension observation explicitly was not an
admission claim. Shell values included `UV_PROJECT_ENVIRONMENT=.venv`; the
uncomposed root shell lacked `PSE_SURREAL_STATE`. The environment boundary
supplies the canonical default. See the owning
[environment guide](../../../dev/agent-environment.md) rather than copying
host placement values into new agent instructions.

## Persistent SurrealDB and live MCP

**Interface-checked:** the following status command passed with native exit 0,
including a final read during evidence integration:

```bash
scripts/pse-env --resource-class light -- .venv/bin/python scripts/surreal_server.py status
```

It reported active, owned listener, open admission and recorded authenticated
WebSocket readiness for `~/.local/state/pse-arrow/surreal-functional-v2`,
`127.0.0.1:18240`, interpretation `pse.substrate.v2`, configured context
`pse/canonical`, service generation `82507491-2bbc-4c4d-9627-c3c8ffd5210d`.
Installed server version is **3.3.0**, binary SHA256
`58ad479cbdd8b1926a636524d259ef309f029d8d9968fb88f350590f7f2b7bc5`.
The readiness field checks recorded protocol proof against the current
invocation, binary and credentials; it is not a newly executed WebSocket query.
The resource report describes existing ownership and limits, not a proposal
for new caps or proof of physical memory reservation.

**Tested, focused interface only:** [probe-interfaces.py](probe-interfaces.py)
initialized the existing HTTP `/mcp` endpoint using the existing root VIEWER
selection account, negotiated protocol `2025-11-25`, sent the initialized
notification and listed fourteen tools. Initialization/listing returned HTTP
200; notification returned 202. No new process, server, database, credentials
or persistent record was created. The server reported 3.3.0.

An explicitly scoped `query` call with `RETURN 1;` returned scalar 1.
An explicitly scoped table listing returned `isError: true`, kind `NotFound`,
because the configured `canonical` database did not exist at observation time.
Both had HTTP 200 and no outer JSON-RPC error. This demonstrates why consumers
must inspect tool and statement results and cannot infer initialized scientific
schema from transport success or a constant query. The reason that context is
absent was not investigated; no initialization or migration was attempted.
This does not overturn the server's serving/admission status or qualify
scientific publication, data access or failure recovery.

The live endpoint can therefore support an optional inspection integration
without another datastore. Correct context and application semantics remain
necessary. Exact source and alternatives are in
[capability research](capabilities/capabilities.md).

## Focused command and failure controls

**Tested:** the corrected probe command passed, native exit 0, zero
probe-execution-failure baseline:

```bash
scripts/pse-env --resource-class light -- .venv/bin/python docs/design_review/evidence/agent-effectiveness-enhancements-2026-10-09/probe-interfaces.py
```

All four negative-control assertions matched their expected observations:

| Observation | Actual result and scope |
|---|---|
| Direct selector entry | `.venv/bin/python scripts/select.py --help` exited 1, `ModuleNotFoundError: scripts`; module invocation reached its own usage parser, exit 2. No test binaries built. |
| Quoted argument transport | `just surreal status --state "/tmp/pse review absent-state"` split the one path into separate parser arguments, exit 2. No state directory created. |
| Codegen check composition | `just --dry-run codegen --check` displayed unconditional generator and Hakari mutation stages around the checked inner command, exit 0. Only dry-run/source evidence; codegen was not executed. |
| Observation under refused admission | Isolated mocks forced admission refusal: acquisition reached once, observer executed zero times, exit 125. No live pressure, allocation or service changed. |
| Failed activity transport | Isolated failed `systemctl` result became exit 0 with `units: []`. Build-process discovery was separately mocked empty. This tests failure representation, not actual user-manager availability. |
| Assessment argument ergonomics | `just assessment --group turn-end` exited 2 because the optional output position consumed `--group`. `just --usage assessment` documents that position; `assessment-list --group turn-end` passed. No assessment checks ran. |

The first development admission control cleared unrelated environment
prerequisites and stopped before admission. It establishes no admission claim;
the retained result is the corrected control. CLI parser refusals deliberately
reproduced by the probe are observed defects/ergonomics, not failed product
tests. Product tests, builds, profiling, native qualification, server lifecycle
operations, configuration changes and dependency upgrades were **not run**.
