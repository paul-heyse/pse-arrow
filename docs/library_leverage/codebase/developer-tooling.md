# Developer, test and operator tooling

This page covers `scripts/` (about 100k non-test Python lines across about 150 top-level modules,
plus `scripts/capabilities` at about 16k), `xtask` (about 27k lines), the `justfile` (244
recipes), `pse-testkit` and `pse-buildinfo`. Guidance comes from the
[agent environment guide](../../dev/agent-environment.md) and the
[qualification command guide](../../dev/validation-assessment.md). The baseline is on the
[entry page](README.md).

Two maintainer premises shape this page:
- This is a personal project, so host-local choices win over portability.
- Tooling changes should stay light, and new alignment lints are not wanted.

**System picture.** The scripts own workload environment and admission (`pse-env` with
`pse_env.py`, `host_admission.py`, `operation_lifetime.py`, `native_operation.py`). They also
own test preparation and execution, evidence and knowledge stores, the SurrealDB supervisor,
telemetry, and document tooling.

These tools are used well:
- `systemd-run --user --scope` for cgroup caps.
- nextest test-groups, profiles, JUnit and `rdeps()` filtersets.
- `cargo --unit-graph` and `cargo metadata` for Arrow validation closures.
- `markdown-it-py`, `ruamel.yaml`, `tomlkit` and `tomllib` for documents.
- `clap` for xtask.
- FastMCP, pydantic and msgspec for the capability server.
- cargo-mutants, llvm-cov, criterion, cargo-fuzz and REUSE, all used as wrapped tools.

The following are legitimately custom:
- force-validation closure composition;
- producer unit identity (framed hashing over syn-normalized source);
- fail-closed reuse of evidence;
- boot- and start-time-bound process identity.

<a id="cbo-047"></a>

## CBO-047 — File-ledger primitives re-derived per script

- **Operation and consumers.** Durable metadata files need four primitives: exclusive or shared
  locks, atomic publish, owner-only permission checks, and deadline-bounded waits. Consumers
  are admission, resources, compiler cache, telemetry, the SurrealDB supervisor, producer
  deployment and native-cache state.
- **Current mechanism.**
  - Locks: `fcntl.flock` appears in 9 non-test modules. One of them (`host_metadata.py`) is a
    polling `lock_until` with `sleep`.
  - Atomic publish: atomic write with fsync is spelled out in `resource_store.py`, and 13
    separate publish, settle or atomic-write helpers exist across modules.
  - Digests: 18 `digest` helper copies, and about 80 places that digest JSON with
    `sort_keys=True`.
  - The Rust side repeats the same SHA-256 hex helper in `pse-buildinfo/src/identity.rs`
    (`value_digest`), `xtask/src/producer_history.rs` and `xtask/src/development/knowledge.rs`.
- **Why it matters.** A single locking, publish and digest discipline would apply the stricter
  permission and deadline checks everywhere. Today they are applied unevenly.
- **Patterns.** LOP-09 (primary), LOP-01, LOP-02.
- **Footprint.** Three modules were read. The rest of the footprint is about 20 modules, located
  with `rg`.
- **Preservation constraints.**
  - Lock order: invocation, then identity, then resource.
  - Absolute deadlines.
  - `O_NOFOLLOW` and `0600` private-file checks.
  - Directory fsync.
  - Locks that survive phase changes.
  - Fail-closed handling of unknown state.
  - Stdlib-only cold-start paths in hooks.
- **Counterevidence.** Some locks deliberately use non-inherited fds or unlinked-file semantics,
  which `filelock` or `portalocker` may not reproduce. The remedy may be one in-repo module
  rather than a library.
- **Status.** Proposed addition.
- **Research questions.**
  - Do `filelock`, `fasteners`, `flufl.lock` or `atomicwrites` respect the fd-lifetime rules?
  - Is a shared `fsio` module sufficient?
  - The Rust analogue is [CBO-003](cross-cutting-mechanisms.md#cbo-003).
- **Comparative review correction (2026-10-27).** `host_admission.metadata` defaults to `durable=False`; that path goes through `operation_lifetime.write_json`, which uses a PID-suffixed `O_TRUNC` temp, does not fsync and does not clean up. Residue exists: 22 zero-length `allocations.<pid>.tmp` files dated 2026-10-10 in `~/.local/state/pse-arrow/host-admission/`. The intended shared owner already exists in `scripts/host_metadata.py`. Python 3.14.8 exposes no `renameat2`, so stdlib no-clobber publication stays link plus unlink. See the [comparative review](../reviews/host-processes-and-durable-files.md#cbo-047).

<a id="cbo-048"></a>

## CBO-048 — Host workload admission and service supervision rebuilt above systemd

- **Operation and consumers.** The tooling must bound concurrent workloads, track owned scopes,
  fixtures and services, and drain them. It also starts, health-checks, backs up and migrates
  the local SurrealDB. Consumers are every `scripts/pse-env` command, the tests, the SurrealDB
  and telemetry services, and the Rust runtime ([CBO-020](execution-lifecycle-and-observation.md#cbo-020)).
- **Current mechanism.** A hand-built ledger of allocations, units, lanes and drain observations
  sits above `systemd-run --scope`. Its parts:
  - `host_admission.py` (about 1.6k lines): reconcile, drained, conflict and acquire.
  - `operation_lifetime.py`: terminal ownership and drain.
  - `process_observation.py` and `pressure_observation.py`: `/proc` and pressure readers.
  - `test_resources.py` (about 3.2k lines), `resource_store.py` and `service_borrowers.py`:
    resource records.
  - `compiler_cache.py`: hand-built transient unit text.
  - About 25 `systemctl --user` shell-outs.
  - `scripts/surreal_server.py` (about 11.6k lines, 274 functions), together with `surrealkit*.py`
    and `surreal_*.py`. These combine supervision, readiness, budgeting, receiver generation and
    schema maintenance.

  Combined with tests, this is the largest hand-owned lifecycle machinery in the repository.
- **Why it matters.**
  - Several layers make independent lifetime decisions.
  - systemd already reports authoritative unit state, through `systemctl show` and D-Bus.
  - systemd already provides slices with CPU, memory and task limits, `BindsTo`/`PartOf`,
    `RuntimeMaxSec`, `--wait`, and `Type=notify` readiness.
  - The custom layer re-derives that state by polling `/proc` and parsing `systemctl` output.
- **Patterns.** LOP-08 (primary), LOP-03 (the OS manager as adopted mechanism), LOP-10, LOP-09.
- **Footprint.**
  - Module heads and function lists were read. The bodies of `surreal_server.py` were not read.
  - About 15k production lines are suspected in admission and resources, plus the supervisor.
- **Preservation constraints.**
  - Per-lane capacity semantics (16 lanes).
  - Release never inferred from a missing caller.
  - Boot- and start-identity ownership.
  - The explicit `unbounded` mode.
  - Drain observations stay reportable when the manager is unavailable.
  - Durability across processes without a daemon.
  - Privileged clients cannot bypass admission.
- **Counterevidence.**
  - Incidents motivated these rules: two editor-wide OOM kills and orphaned scopes.
  - systemd has no cross-process admission semaphore, so some ledger remains necessary.
  - It is uncertain how much of the supervisor is generic and how much is policy.
- **Status.** Current promise.
- **Research questions.**
  - Which facts does systemd already own authoritatively?
  - Can a Python D-Bus client (`pystemd`, `dbus-fast`, `jeepney`) replace the `systemctl`
    parsing?
  - Can `psutil` replace the `/proc` readers?
  - Does `surreal isready` or `export/import`, or `surrealkit` migrations, cover the
    supervisor's maintenance work?
- **Library assessment correction (2026-10-27).** `surreal_server.py` alone mentions `systemctl` 36 times, so the earlier count was low. The placement scopes set no `Delegate=`. Since SurrealDB 3.3.0, `/health` returns 200 while the server is still starting, and the later authenticated protocol-readiness check compensates for that.
- **Comparative review correction (2026-10-27).** The premise that admission and supervision re-derive state above systemd is only partly true: the sampled paths already combine manager facts (`systemctl show`), kernel cgroup evidence and the ledger the premises require. Bootstrap admission runs under system Python, so a non-stdlib D-Bus client would create a second route. Residue exists: 24 empty allocation slices remain, 18 of 19 under `pse-allocation.slice` and 6 of 6 under `psebuild-allocation.slice`. For this deployment (single node, no `--import-file`) the authenticated protocol-readiness proof subsumes what `/ready` adds, and `/ready`'s heartbeat-staleness rule is an unmeasured false-unready risk under saturating load. SurrealKit's dependence on the edited SDK is incidental, because it runs the unbounded transport profile. See the comparative review: [non-SurrealDB parts](../reviews/host-processes-and-durable-files.md#cbo-048) and [SurrealDB parts](../reviews/canonical-store-transport.md#cbo-048).

<a id="cbo-049"></a>

## CBO-049 — Test selection, requirements and runner configuration maintained beside nextest

- **Operation and consumers.** Select tests, project their resource requirements, prepare
  invocations, admit fixtures and archive results. Consumers are `just unit-package`, `affected`,
  `py-test`, archives and agents.
- **Current mechanism.**
  - `test_requirements.py` projects per-test capacity, effects, fixtures and roles into a JSON
    catalogue.
  - Nextest config groups are then rewritten to match that catalogue (`native_tests.py`,
    `test_invocation.py` via `tomlkit`). `.config/nextest.toml` says its groups are projected
    from the catalogue.
  - `affected.py` maps files to packages from `cargo metadata` and builds `rdeps()` filtersets.
  - `test_archives.py` builds historical archives.
  - About 20 recipes repeat `scripts.arrow_validation nextest …`.
- **Why it matters.** Selection intent, resource needs and runner configuration exist in two or
  three forms: the catalogue, the rewritten TOML and the receipts.
- **Patterns.** LOP-02 (primary), LOP-03, LOP-06.
- **Footprint.**
  - Read: `affected.py`, the heads of the catalogue and archive modules, and `nextest.toml`.
  - Suspected: 8–10k lines.
- **Preservation constraints.**
  - Force-validation per unit-graph closure.
  - The managed-primary partitions.
  - The tooling never manufactures a passing result.
  - Fixture lifetime is tied to the reconciled outcome.
- **Counterevidence.** Tying fixture lifetime to the outcome is not a nextest concept.
- **Status.** Current promise.
- **Research questions.**
  - Can nextest setup and wrapper scripts, per-test overrides with `threads-required`,
    `archive`/`--archive-file` and `list --message-format json` replace parts of the
    projection?
  - Does `cargo-hack` cover `cargo_feature_check.py`?
- **Library assessment correction (2026-10-27).** nextest list JSON and `--archive-file` are already used. `cargo_feature_check.py` is the adapter that cargo-hack invokes, not a replacement for cargo-hack.
- **Comparative review correction (2026-10-27).** "Two or three forms" should read: one maintained declaration (`RUST_OWNERS` plus the pytest markers in `test_requirements.py`) with per-run derived outputs. There are two chained per-run config copies: scheduling (`native_tests.scheduled_rust_arguments`) and the JUnit path (`test_invocation.report_configuration`, through `test_run.rust_report_command`). `just test`, `test-package` and the 17 preset recipes call `scripts.arrow_validation nextest` directly and bypass the selected-catalogue route, which `unit-package` (`select.py`) and `test_run` use. A nextest tool-config overlay cannot reference repository-defined test groups, and no override for `junit.path` was found. `cargo_feature_check.py` also composes Arrow force-validation for each unit-graph closure (`arrow_validation.compose` and `capture`), so feature unification cannot remove it. See the [comparative review](../reviews/developer-tooling.md#cbo-049).

<a id="cbo-050"></a>

## CBO-050 — Development evidence and knowledge pipeline split across two languages

- **Operation and consumers.** Index and query development evidence, assessments, measurements,
  knowledge concepts and document scope. Consumers are agents through `just evidence`, the MCP
  capabilities and `xtask evidence`.
- **Current mechanism.**
  - Rust side: `xtask/src/development/{evidence,evidence_query,knowledge}*.rs`, backed by
    SurrealDB and DataFusion.
  - Python side, about 12k lines combined:
    - `evidence_store.py`, `evidence_sources.py`, `evidence_projection.py`, `evidence_detail.py`
    - `knowledge_store.py`, `workspace_domains.py`, `workspace_sources.py`
    - `document_lifecycle.py`, `assessment_store.py`
    - `capabilities/*_queries.py`
  - A knowledge concept moves Markdown → Python adapters → JSON → Rust → SurrealDB. Along the
    way its meaning is restated in TypedDicts, pydantic models, Rust structs and rows, and the
    MCP schemas are generated from those.
- **Why it matters.**
  - Two languages and several translations for one pipeline raise the cost of every change.
  - The maintainer frames evidence work around reuse and staleness, not traceability. That
    suggests a smaller query surface could do.
- **Patterns.** LOP-09 (primary), LOP-06, LOP-02.
- **Footprint.** Headers and module boundaries.
- **Preservation constraints.**
  - Source cut and identity checks.
  - A disposable, rebuildable index.
  - Explicit paths only.
  - Offline inspection.
- **Counterevidence.** The split may be deliberate: Python as the service-free source adapter,
  Rust as the executor.
- **Status.** Current promise.
- **Research questions.**
  - Could Python emit Arrow or Parquet straight to DataFusion, or use SQLite or DuckDB, for a
    rebuildable derived index?
  - Is SurrealDB justified for development evidence? The canonical premise covers runtime data.
- **Library assessment correction (2026-10-27).** The Rust evidence-question path already runs on DataFusion without the SurrealDB service.
- **Comparative review correction (2026-10-27).** The registry already generates Python msgspec wire types for this pipeline (`scripts/generated/evidence_snapshot_input.py` with `EvidenceSnapshot` and `EvidenceFact`; `knowledge_query_input.py` with `KnowledgeQuery`, `KnowledgeMode` and the enums), but they have no non-test importers. `evidence_projection.py` re-declares both types as TypedDicts field for field, and `knowledge_store.py` re-declares the query fields and the `--mode` choices. The pydantic models in `workspace_domain_contracts.py` belong to the source-only query surface (ADR-0222 design), not the indexed path. Evidence has two Rust read routes, offline and indexed through `source_cut`, both feeding the same DataFusion questions. The store is an architecture contract: blueprint §24.1 "Development knowledge and source-backed changes", §3.2, and the proposed ADR-0219 and ADR-0222 (both Tested 2026-10-23). See the [comparative review](../reviews/developer-tooling.md#cbo-050).

<a id="cbo-051"></a>

## CBO-051 — Hand-written wire and front-matter parsers in tooling

- **Operation and consumers.** Decode OTLP and gRPC control messages, and read ADR and plan front
  matter. Consumers are telemetry control, document lints, agent hooks and activity.
- **Current mechanism.**
  - A hand-written protobuf varint reader (`scripts/grpc_wire.py`).
  - Front matter is split independently in `adr.py`, `agent-hooks.py`, `check_agent_config.py`
    and `document_metadata.py`.
  - Telemetry queries are built over `urllib` and `socket`.
- **Why it matters.** The code is small, but each copy has its own edge-case handling.
- **Patterns.** LOP-01, LOP-02.
- **Footprint.** At least four front-matter forms and one wire reader.
- **Preservation constraints.**
  - Duplicate-key rejection.
  - Bounded reads.
  - Stdlib-only imports on cold-start hook paths.
- **Counterevidence.** The cold-bootstrap constraint is real, and `protobuf` or
  `python-frontmatter` may be too heavy for hooks.
- **Status.** Proposed addition.
- **Research questions.**
  - Does `opentelemetry-proto` already sit in the environment?
  - Would routing every consumer through `document_metadata` suffice?
- **Library assessment correction (2026-10-27).** Front matter has one strict reader, which `adr.py` delegates to, plus two stdlib strippers, not four parsers. The earlier account missed two hand-written pieces: an HTTP/2/gRPC server in `scripts/surreal_telemetry.py` and a gRPC client in `scripts/tests/surreal_mx_check.py` ([LOP-X010](../assessment-patterns.md#lop-x010)).
- **Comparative review correction (2026-10-27).** Switching SurrealDB's exporter to OTLP/HTTP is not available. At `v3.3.4` the server builds `opentelemetry_otlp::SpanExporter::builder().with_tonic()` and compiles `opentelemetry-otlp` 0.31.1 without `http-proto`, and no protocol variable selects HTTP (read through the GitHub contents API). The hand HTTP/2 sink is therefore the minimal fixed peer, and `surreal_server.py` already points SurrealDB at the collector's `otlp_grpc_endpoint`. See the [comparative review](../reviews/developer-tooling.md#cbo-051).

<a id="cbo-052"></a>

## CBO-052 — Several command surfaces without a shared declaration

- **Operation and consumers.** Discover and run workflows. Consumers are the maintainer and
  agents in both runtimes.
- **Current mechanism.** There are five command surfaces, and option vocabulary such as the
  `--native` variants is repeated across them:
  - **`justfile`.** 244 recipes. Many are one-line `python -m scripts.X` calls, and some inline
    shell.
  - **argparse.** 64 non-test scripts build their own `argparse` parser.
  - **xtask.** It uses `clap`.
  - **FastMCP capability server.**
  - **`capabilities/cli_codegen.py`.** It generates cyclopts-style argv.
- **Why it matters.** Changing an option or adding a workflow touches several surfaces.
- **Patterns.** LOP-02, LOP-09.
- **Footprint.** Counts only.
- **Preservation constraints.**
  - Admission through `pse-env`.
  - Recipes stay positional.
  - Codex and Claude parity.
- **Counterevidence.** "Tooling changes stay light" argues against a broad consolidation. This is
  weak unless it follows from [CBO-049](#cbo-049).
- **Status.** Proposed addition.
- **Research questions.**
  - Could one typed CLI framework, already the target of `cli_codegen.py`, own the script entry
    points?
  - Could just modules collapse the repeated `arrow_validation nextest` compositions?
- **Library assessment correction (2026-10-27).** `just --dump` reports 231 recipes, not 244.
- **Comparative review correction (2026-10-27).** The premise is rejected: the surfaces are layers with distinct owners. 194 of 231 recipes have one-line bodies, 100 of them Python forwards. There are 421 long-option declarations across 64 argparse scripts under 281 distinct names, and repeated names are mostly homonyms: `--state` has three or more meanings, one of them a plan lifecycle state in `document_lifecycle`, and `--native` has two or more. Recipe metadata is already derived from `just --dump` (`capabilities/entrypoints.py`). The one real restatement is covered by [CBO-050](#cbo-050). See the [comparative review](../reviews/developer-tooling.md#cbo-052).

