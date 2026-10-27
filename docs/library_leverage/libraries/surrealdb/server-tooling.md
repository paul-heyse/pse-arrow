# SurrealDB server tooling (`surreal` CLI, HTTP probes, export/import) and SurrealKit

Realizations considered:
- The installed, source-built SurrealDB 3.3.4 server.
- SurrealKit as already selected by the workspace's tooling: a pinned 1.0 beta built against the edited SDK. Upstream releases go up to 1.0.0-beta.6, published to crates.io 2026-10-02.
- Upgrade route: none beyond server 3.3.4.

Research date 2026-10-27. Sources:
- Context7 `/websites/surrealdb`: `reference/rest-api/http-protocol` (`/health`, `/ready`), `build/migrating/from-old-surrealdb-versions/32-to-33` (startup and health), and `reference/cli/db/commands/export` (help output).
- crates.io versions for `surrealkit`.
- Optional orientation `docs/libraries/surrealdb_tooling.md` §5, on SurrealKit sync, rollout, tests and typegen.
- Read-only skim of the consumer scripts' module heads and function lists, plus the readiness and backup function heads.

## Types

### surrealdb-server/readiness-probes
- **Need it serves:** decide whether a server process has finished startup and can serve requests, as distinct from merely answering on its port.
- **Recognizable code shapes:**
  - A supervisor polls `/health`, then runs its own authenticated round trip to decide "ready".
  - Readiness state machines combine unit state, listener ownership and protocol checks.
- **Mechanisms and APIs:**
  - `GET /health` reads one key in a read-only transaction. It returns 200 or 500, or 403 when the route is denied by capabilities. From 3.3.0 it answers during startup.
  - `GET /ready` (since 3.2.0) returns 200 only when startup has completed and the node's cluster heartbeat is recent. Startup covers the version check and migrations, the default namespace and database, root credentials, node registration and `--import-file`. A heartbeat counts as recent within three `--node-membership-refresh-interval` periods, 3 s by default. While starting, when stale or when the membership record is missing, it returns 503; it returns 500 when the heartbeat cannot be read.
  - `surreal db isready` calls `/ready`.
- **Obligations discharged:**
  - The server's own startup completion signal, including a failed import or credential initialisation that leaves a running server that never becomes ready.
  - Heartbeat liveness.
- **Residual caller obligations:**
  - Process and listener ownership: is this the server I launched?
  - Authenticated protocol readiness for the specific credentials, namespace and database.
  - Maintenance modes.
  - Deadlines.
  - It gives no systemd `Type=notify` integration; that was not found in the sources inspected.
- **Semantic conditions:**
  - It is unauthenticated and port-scoped, so another process on the port could answer.
  - 503 versus 500 distinguishes "not yet" from "broken".
  - On 3.2.x a missing membership record returned 500.
- **Version / build conditions:** `/ready` needs server 3.2.0 or later, with the 3.3.0 semantics described above. The HTTP routes must be served on the probed listener and not denied by capabilities.
- **Limits and negative results:** a probe is not an ownership or identity check, and it does not prove gRPC-session readiness.
- **Evidence:** Interface-checked: the Context7 documentation pages listed above.

### surrealdb-server/logical-export-import
- **Need it serves:** move or back up database contents as a portable SurrealQL script.
- **Recognizable code shapes:** custom backup routines that stop the server and copy storage files; hand-written inventory capture and verification.
- **Mechanisms and APIs:**
  - `surreal export --endpoint … --ns … --db … [--only] [--users] [--accesses] [--params] [--functions] [--analyzers] [--tables [list]] [--versions] [--records] FILE`.
  - `surreal import` runs the script back in.
  - The SDK and RPC equivalents are `export`/`import`, with `ExportSurql`/`ImportSurql` streaming RPCs on gRPC.
- **Obligations discharged:**
  - A logical dump of schema and records per namespace and database, with filtering.
  - Transport-neutral restore into a fresh server.
- **Residual caller obligations:**
  - Files and blobs held outside the database.
  - Physical identity (incarnations, credentials).
  - Verifying completeness.
  - Online versus offline consistency: the docs inspected do not state that a single export runs in one snapshot transaction.
  - Restore speed, because restore replays statements.
  - Permissions and authentication for the dump.
- **Semantic conditions:**
  - Logical, not physical. Restore re-executes `DEFINE` and `INSERT` statements, so events and other definition side effects may run or need ordering.
  - Server-generated identities and versionstamps are not preserved byte-for-byte.
- **Version / build conditions:** SurrealDB 3.x CLI. Protocol 0.14 removes the directory-format export RPC that 0.13 declared.
- **Limits and negative results:** it is not a crash-consistent physical backup of an engine's append-only log and snapshot. It does not replace a physical copy when exact storage identity, external payload inventories or fast restore are required.
- **Evidence:** Interface-checked: the Context7 export help page. Export consistency semantics were not inspected.

### surrealdb-server/telemetry-export-transport
- **Need it serves:** receive the server's own traces or metrics in a collector.
- **Mechanisms and APIs:** at tag `v3.3.4`, the server builds `opentelemetry_otlp::SpanExporter::builder().with_tonic()` (`surrealdb/server/src/telemetry/traces/mod.rs`) and compiles `opentelemetry-otlp` 0.31.1 with features `grpc-tonic, tls, tls-roots, trace, metrics, logs, internal-logs` and no `http-proto` (`surrealdb/server/Cargo.toml`).
- **Semantic conditions:** server OTLP export is tonic gRPC only at 3.3.4. The standard endpoint variables are honoured, but no protocol variable can select OTLP/HTTP, so a receiver must accept OTLP over gRPC.
- **Version / build conditions:** an HTTP/protobuf exporter would need an upstream build change; none exists at 3.3.4.
- **Evidence:** Interface-checked: both files read through the GitHub contents API at `v3.3.4` (2026-10-27).

### surrealkit/schema-rollout
- **Need it serves:** reconcile a declared SurrealQL schema with a live database through tracked, staged rollouts.
- **Recognizable code shapes:** hand-written schema install and upgrade sequences; checks that the catalog matches.
- **Mechanisms and APIs:**
  - `surrealkit sync` (desired state).
  - `surrealkit rollout start|complete|status|lint`.
  - Tool-owned metadata tables (`__entity`, `__rollout`, `__seed`).
  - Declarative TOML tests.
  - `typegen`.
  - Library use from inside a process.
- **Obligations discharged:**
  - Diffing and applying definitions.
  - Rollout state tracking.
  - Migration and sync parity tests, in the 1.0 line.
- **Residual caller obligations:**
  - Activation and settlement semantics.
  - Proof that a submitted step actually settled.
  - Exclusive maintenance ownership.
- **Semantic conditions:**
  - A beta.
  - Its metadata tables are tool-owned state.
  - Arbitrary non-`DEFINE` statements weaken entity tracking.
- **Version / build conditions:** 1.0.0-beta.x requires an SDK build matching the server.
- **Limits and negative results:** it does not supervise processes, check readiness or take backups.
- **Evidence:** Interface-checked: orientation doc §5 and the crates.io version list. The SurrealKit source was not inspected in this pass.

## Current applicability

### [CBO-048](../../../library_leverage/codebase/developer-tooling.md#cbo-048) — Host workload admission and service supervision rebuilt above systemd (SurrealDB parts only)
- **Types:** `surrealdb-server/readiness-probes`, `surrealdb-server/logical-export-import` and `surrealkit/schema-rollout`.
- **Realization:** installed server 3.3.4 and the already-selected SurrealKit beta.
- **Conclusion:**
  - Readiness fits with conditions as a narrow substitution: `/ready` or `surreal db isready` can replace the `/health` step.
  - Export/import is ruled out here as a backup replacement.
  - SurrealKit is already adopted; this is no new route.
  - Most of the supervisor (`surreal_server.py`, 274 functions) is ownership, admission, placement, payload inventory and maintenance policy that the SurrealDB tooling does not own.
- **What it would discharge / what remains local:**
  - Readiness:
    - The supervisor's `service_readiness` probes `http://127.0.0.1:{port}/health`. Since 3.3.0 that returns 200 during startup, so a 200 there only means storage answers.
    - The supervisor then requires its own `protocol-readiness.json` proof bound to the unit's `InvocationID`.
    - `/ready` would report the server's own startup completion and fail-closed states, such as a failed import or credential initialisation.
    - Local code still owns: listener ownership (`_owns_listener`), unit phase, the maintenance listener and authenticated protocol readiness for the selected identity.
  - Backup:
    - `_backup` freezes the payload inventory, captures an application inventory, stops the server and copies offline state, which is SurrealMX AOL and snapshot data.
    - `surreal export` would produce a logical script without the external payload files and without the physical identity.
    - It cannot satisfy the existing guarantees: offline state copy, payload inventory verification, restore re-rooting.
    - Export could serve as an additional portable artifact, not a replacement.
  - Migration: SurrealKit `rollout start|complete|status` is already used by `surrealkit_maintenance.py`, which adds settlement and exclusive-owner policy above it.
- **Prerequisites, configuration or extension:**
  - Readiness needs the HTTP `/ready` route reachable on the canonical listener and not denied by the server's capability flags.
  - Heartbeat staleness logic for a single node needs node membership enabled, which is the default.
- **Semantic gaps against the CBO's preservation constraints:**
  - None for `/ready`, provided it is added as a step and does not replace the ownership checks.
  - "Release never inferred from a missing caller" and boot and start identity remain local.
- **Unsettled premises and what would settle them:**
  - Whether the patched server's canonical profile exposes `/ready`. It is unpatched in the series as far as the patch names indicate; settle by curling it on a fixture server.
  - Whether `/ready`'s heartbeat semantics behave well across suspend and resume of the host.
  - The body of `surreal_server.py` was not read beyond the readiness, protocol-readiness and backup heads, so further generic-versus-policy splits are not established.
- **Evidence:** Interface-checked: documentation and a source skim. Proposed.

**Reviewer correction (2026-10-27).** The comparative review retains the current readiness route. For this deployment (single node, no `--import-file`) the authenticated protocol-readiness proof already subsumes what `/ready` adds, and `/ready`'s heartbeat-staleness rule is a false-unready risk under saturating load (not measured). Export remains no backup. SurrealKit is built against the vendored SDK, but it runs the unbounded upstream-transport profile (it never calls `Config::bounded_requests()`), so the dependence is incidental: artifact identity and consistency, not behaviour. See the [comparative review](../../reviews/canonical-store-transport.md#cbo-048).
