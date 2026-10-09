# Local SurrealDB SDK corrections

`surrealdb` and `surrealdb-engine-api` are the packaged 3.3.0 sources from upstream
revision `238bfeb11f5725bebed370167656748df8067595`. The workspace dependency selects
the SDK by an exact version and explicit path; the SDK selects its sibling engine API
by the same exact version and explicit path. Production resolution remains in the root
lockfile, without root dependency override tables.
Their original license files are preserved. REUSE names the exact imported license
parameters; local additions do not relicense the upstream sources.

The patch supplies bounded request contexts, conservative dispatch classification,
separate application/control admission, retained submitted correlations, bounded
session/replay state, ordered acknowledged selection checkpoints, finite native WebSocket
buffers and physical connection teardown. Diagnostics avoid logging credentials,
tokens, query bindings or returned payloads on abandoned channels. Canonical consumers
own operation clocks, SQL commit fences and complete response/page validation.

The SDK has a separate qualification workspace and lockfile so its private transport
tests can run against the sibling patched engine API without adding a repository crate
or resolving the root dependency graph again. Runtime dependencies used by the focused
controls match the production pins; upstream development and optional-engine dependencies
remain qualification-only. Its ordinary `target` directory is ignored and independently
owned. Do not share it with the root target directory.

Focused private controls live in `src/engine/remote/ws/mod.rs`: uncertain `start_send`,
cancelled stalled flush, retained correlation across the synthetic 89/91-second boundary,
and closed-channel diagnostic redaction. Run them with the locked standalone manifest,
`--no-default-features --features protocol-ws,rustls --lib`. Application mock-peer and
released-server controls live in `pse-operations`. Private unit controls alone do not
qualify server abort, transaction durability or integrated scientific execution.
