# Local SurrealDB SDK corrections

`surrealdb` and `surrealdb-engine-api` are the packaged **3.3.4** sources at upstream
revision `1ac2b1dba7cd177621a4cf5d507caebeb2414395`. Their exact archive checksums,
packaged VCS paths and complete byte-preserving deltas are recorded in
[deltas/readme.md](deltas/readme.md). The workspace selects the SDK by exact version
and explicit path; the SDK selects the sibling engine API the same way. Production
resolution remains in the root lockfile, without root dependency override tables.
Original license files are preserved. REUSE names the imported license parameters;
local additions do not relicense the upstream sources.

The current transport and integration record is [surrealdb/PATCH.md](surrealdb/PATCH.md).
Production selects `protocol-grpc`: bounded request contexts, original operation clocks,
separate application/control admission, physical HTTP/2 connection ownership, bounded
response conversion, complete acknowledged setup restoration without application replay,
structured error trailers and W3C trace-context propagation. The engine API supplies
`RequestContext`, response allowances and prepaid ownership across those boundaries.
WebSocket source remains in the upstream SDK; the former 3.3.0 WebSocket qualification
account does not describe the current canonical transport or establish gRPC qualification.

The SDK retains its standalone manifest and lockfile for dependency-built controls;
its ordinary `target/` is independently owned and must not be shared with the root
checkout. The canonical transport test owner exercises the `grpc-lifecycle-test-support`
feature when selected. Current qualification status belongs to the owning plan and
blueprint §24.2. The source reconstruction here establishes byte fidelity only.
