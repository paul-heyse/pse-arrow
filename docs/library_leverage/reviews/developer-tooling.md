# Developer tooling

**Group G9 of the [first comparative review](README.md), 2026-10-27.** Focused comparative advice;
it authorizes nothing.

| Item | Value |
| --- | --- |
| Opportunities | [CBO-049](../codebase/developer-tooling.md#cbo-049), [CBO-050](../codebase/developer-tooling.md#cbo-050), [CBO-051](../codebase/developer-tooling.md#cbo-051), [CBO-052](../codebase/developer-tooling.md#cbo-052) |
| Source inspected | `main` at `3e9a2525b` plus its dirty tree. `git diff --stat dbc4c9436..HEAD -- scripts/ xtask/ justfile .config/nextest.toml` is empty, so the audit baseline still holds for this group. |
| Consumed accounts | Codebase account (published at `c2f8b485a`), including its dated corrections. The library accounts from the first assessment (2026-10-27): cargo-nextest, cargo-hack, sqlite, duckdb, parquet-datafusion-index, opentelemetry-proto, grpcio, h2, python-frontmatter, cyclopts, typer, just and fastmcp. Patterns: base edition 0.2, plus additional patterns revision 2 (LOP-X010 is central here). |
| Realizations | Host tools: cargo-nextest 0.9.146 and just 1.58.0. cargo-hack 0.6.45 as the library account records it. Python pins: msgspec 0.21.1 (base), tomlkit 0.15.1, cyclopts 5.3.0, pyarrow 25.0.1 and opentelemetry-exporter-otlp-proto-http 1.45.1. Server: SurrealDB 3.3.4, the workspace pin; its upstream server source was read at tag `v3.3.4`. |
| Standard | Core 3.4 (DP-01, DP-13, DP-14, DP-16 and DP-17 are the most relevant) and the pse-arrow binding. The process-simulator profile is not engaged, because no scientific semantics are in scope. |
| Maintainer premises applied | Personal, host-local project. Tooling changes stay light. No new alignment lints. Evidence work aims at reuse and staleness, not traceability. Hook cold-start paths are stdlib-only. U3 is unresolved and gets a branch in CBO-050. |

## Group summary

**Joint decision.** Most of the group already uses the right mechanisms. In each case the hand
code that remains is one of three things: the extension point a tool offers, the right owner of a
repository-specific rule, or a deliberate test peer. The pass makes one bounded recommendation,
which spans CBO-050 and touches CBO-052: the Python wrappers for the Rust-owned evidence and
knowledge requests should consume the Python wire types the registry already generates, instead of
re-declaring them. Every other opportunity is retained, or its premise is rejected on evidence.
The store question in CBO-050 depends on U3.

**Corrections that drive the result.**
- **nextest overlay.** A tool config cannot express the scheduling policy against the repository's
  own test groups (CBO-049).
- **OTLP protocol switch.** LOP-X010's cheapest remedy, changing the counterparty's protocol, is not
  available. SurrealDB 3.3.4 is built to export OTLP over gRPC only (CBO-051).
- **Generated types unused.** The registry already generates msgspec types for the evidence snapshot
  and the knowledge query, but nothing outside tests imports them. The Python producer re-declares
  them by hand (CBO-050).
- **Command surfaces.** The "several command surfaces" are layers with distinct owners, and the
  repeated option names are mostly homonyms with different meanings (CBO-052).

**Interactions with other groups.**
- **G8, the registry and Python boundary (CBO-040–043).** The recommended change uses
  registry-generated msgspec Structs on the Python side. G8's msgspec probe found that constructing
  a Struct validates nothing, so validation belongs at the point of publication, through
  `msgspec.convert`. Reconcile with G8's answer to "must Python re-check documents declared by
  Rust?".
- **G3, SurrealDB transport and SurrealQL (CBO-009, 016, 046 and the SurrealDB parts of 048).** The
  tooling evidence and knowledge stores (`xtask/src/development/{evidence,knowledge}.rs`) consume
  the vendored SDK and the `pse_operations::transport` lifetime. Any SDK or transport move in G3 has
  to migrate them too. Under U3 = no, removing this consumer would shrink G3's footprint, but that
  should not decide either question. Under U1, a SurrealDB server change that adds an HTTP/protobuf
  exporter belongs to the same upstream-contribution question as G3's SDK seams.
- **G1, host processes and file ledgers (CBO-047, 048).** nextest test groups limit concurrency
  within one process only. Cross-process admission stays with `host_admission`, and fixture lifetime
  stays with `test_resources`. The `snapshot_digest` helper in `evidence_projection.py` belongs to
  CBO-047's digest consolidation, not here.
- **G2, observation (CBO-013–015).** The SurrealDB trace-capture sink does not depend on whether
  product observation adopts tracing as its backbone.

**Shared prerequisites and order.** None for the recommended change. A store change for CBO-050
would need the U3 decision first, followed by the architecture and decision route.

**Reinforcing and competing choices.** The CBO-050 recommendation also removes the one real
restatement found under CBO-052, so the two reinforce each other. There are no competing choices
inside the group.

**Consequence-based priority.** Low throughout. The recommended change is small and local. The
only consequential open choice is U3, and even that matters only if a consumer turns out to need
indexed evidence or knowledge reads while the canonical service is unavailable.

---

<a id="cbo-049"></a>

## CBO-049 — Test selection, requirements and runner configuration maintained beside nextest

### Responsibility and target intent

The tooling runs each selected test under scheduling that matches its declared demand, and admits
fixtures and native effects only for tests that declared them. It never manufactures a passing
result.

**What is promised:**
- **One declaration.** `scripts/test_requirements.py` declares owners: `RUST_OWNERS`, keyed by
  package and test-name prefix with longest-prefix precedence, plus fixture, peer and binary rules.
  Python tests declare through pytest markers.
- **Selected catalogue.** The catalogue is computed per run from the actual
  `cargo nextest list --message-format json` inventory (`rust_catalog`).
- **Consumers of that one catalogue.**
  - Scheduling: `nextest_configuration` and `scheduled_rust_arguments`, which emit one exact
    override per test that carries a group.
  - Effect guards: `test_requirements.guard` and `fixture_guard`, used by `test_resources`.
  - Fixture preparation.
  - Worker-binary selection.
- **Closed-world check.** Selected tests must equal catalogued tests, and required roles must be
  present.
- **Caller configuration.** A caller-selected `--config-file` may specialize the group capacities,
  but must define every group it needs (`scheduled_rust_arguments`).

The codebase account says the meaning exists in "two or three forms": the catalogue, the rewritten
TOML and the receipts. In fact there is **one maintained declaration**. The other forms are derived
per run, written to `build/`, and never edited. There are two per-run copies of
`.config/nextest.toml`, which `test_run.rust_report_command` chains:
- the scheduling copy, `native_tests.scheduled_rust_arguments`;
- the JUnit-path copy, `test_invocation.report_configuration`.

### Credible complete options

1. **Retain current.** Per-run exact overrides are applied to a tomlkit copy of the repository
   config. Group capacities stay in `.config/nextest.toml`.
2. **Tool-config overlay** (`cargo-nextest/layered-tool-configuration`: `--tool-config-file
   pse:<abs-path>`). Emit only the scheduling overrides into a tool file instead of copying the
   repository config.
3. **Static ordered prefix overrides** in `.config/nextest.toml`
   (`cargo-nextest/declarative-test-scheduling`), either generated from `RUST_OWNERS` or made the
   authority. This would let bare `cargo nextest run` and the preset recipes schedule too.
4. **nextest setup and wrapper scripts** for fixtures (`cargo-nextest/setup-and-wrapper-scripts`).
5. **cargo-hack in place of `cargo_feature_check.py`** (`cargo-hack/feature-powerset-enumeration`).

### Comparative argument

**Option 2 moves policy to the wrong owner and removes nothing.**
- **Groups cannot cross the layer boundary.** nextest loads config sources lowest priority first:
  embedded defaults, then tool files, then the repository. It validates each file's override
  `test-group` references against the groups known *up to that file*. The relevant code is
  `nextest-runner/src/config/core/imp.rs` (`deserialize_individual_config`, `known_groups`, error
  `UnknownTestGroups`) and `config/core/sources.rs` (`sources()` appends the repository file last).
  So a tool override cannot name the repository's `scientific` or `store` group. Tool-defined
  groups must also be named `@tool:pse:…` (`InvalidTestGroupsDefinedByTool`).
- **Capacities would move into Python.** The group capacities would have to move out of the
  commented repository config and into a file emitted by the tool. The caller-config
  specialization check would lose its meaning.
- **The copy stays anyway.** The JUnit path is a repository-profile setting that outranks tool
  files, so `report_configuration` would still copy the config.

**Option 3 splits scheduling from the selected catalogue.**
- **Two authorities, or a generated config.** The catalogue keeps driving guards, fixtures and
  worker selection. Moving scheduling into static overrides creates a second authority for the owner
  facts (LOP-02). Avoiding that would need a codegen hop into a hand-authored config, and nextest
  has no include, so the hop would end in a tool file and meet option 2's restriction.
- **Precedence has to be re-encoded.** Longest-prefix precedence becomes first-match ordering, and
  the exact-name exceptions must come first. A second algorithm would then have to agree with
  `rust()`.
- **Bare runs gain nothing.** A canonical fixture refuses without a recipe-selected
  `PSE_SURREAL_STATE` (`crates/pse-operations/src/testing.rs`), so a bare canonical run fails rather
  than running unthrottled.

**Option 4** is ruled out, as the library account found. Both features are experimental, there is
no teardown hook, and fixture lifetime cannot be tied to the reconciled outcome.

**Option 5's premise is false.** `cargo_feature_check.py` is the `CARGO_HACK_CARGO_SRC` adapter.
Besides qualifying bare `--features`, it composes Arrow force-validation for each unit-graph closure
(`arrow_validation.compose` and `capture`). That composition is an invariant obligation, so no
outcome of the probe the account suggested could delete the adapter.

**Current approach.** Retaining it keeps one authority and lets scheduling, guards and fixtures
share one per-run image. Its closed-world check fails closed. The copy-then-append mechanism leaves
every repository setting in force (DP-14: it uses nextest's override model rather than rebuilding
it).

### Revealing case

To debug a contended store, a caller passes `--config-file custom.toml` with `store.max-threads`
reduced to 4.
- **Today:** the capacity is honoured, and a missing group is refused before execution.
- **Under option 2:** capacities come from `@tool:pse:` groups in a Python-emitted file. The
  caller's specialization is silently ignored, which weakens a guarantee.
- **Under option 3:** the reduced cap works for the scheduled tests. But when an owner prefix
  changes in `RUST_OWNERS`, the static file goes stale until it is regenerated, and the guard and
  scheduling views disagree within one run.

### Disposition

**Retain current.** The premise is rejected for the "several forms" claim and for cargo-hack.

**Residual work.** None required. One observation for the test-route owner: `just test`,
`test-package` and the 17 preset recipes that call `scripts.arrow_validation nextest` go straight to
nextest, bypassing the selected-catalogue route that `unit-package` (`select.py`) and `test_run`
use. That fits "recipes are thin" in `.claude/rules/tooling.md`. Whether canonical tests selected
through those recipes are meant to fail on the missing fixture is a test-route question, not a
leverage one.

**Revisit trigger:**
- nextest gains cross-source group references or a CLI or environment override for the JUnit path;
  or
- a measurement shows that one exact override per test is costly for large canonical selections
  (a timing of large canonical selections would show it; not run in this pass).

### Rule impacts

None.

### Evidence labels

- *Interface-checked:*
  - the nextest source-ordering and group-validation rules (upstream `main` source of
    `config/core/imp.rs` and `sources.rs`, read through the docs mirror and raw GitHub; not
    re-read at the exact 0.9.146 tag);
  - the precedence and JUnit-path limits (from the library account);
  - the local sources `test_requirements.py`, `native_tests.py` (`scheduled_rust_arguments`),
    `test_invocation.py` (`report_configuration`), `test_run.py`, `select.py`, `cargo_feature_check.py`
    and `.config/nextest.toml`.
- *Proposed:* the disposition.

---

<a id="cbo-050"></a>

## CBO-050 — Development evidence and knowledge pipeline split across two languages

### Responsibility and target intent

**Development evidence.** Authentic owner facts about test invocations, assessments, diagnostics,
measurements, host and storage health are answered as bounded questions, either offline or over
retained generations.

**Development knowledge.** Owner-native concepts, typed relations, productions and excerpts sit
above immutable source versions. Exact, graph and lexical queries run over one corpus cut.

**Current contracts.**
- **Architecture.** Blueprint §24.1 (`operations-and-validation.md`, "Development knowledge and
  source-backed changes") and `workspace-and-dependencies.md` §3.2. These place the storage in
  existing xtask composition and the private tooling database, outside the product closure.
- **ADRs.** ADR-0219 and ADR-0222, both *proposed* and implemented and Tested on 2026-10-23.
  - ADR-0222 rejected "a new knowledge service/store" in favour of reusing the tooling lifecycle.
  - Its revisit trigger is "a concrete consumer needs independent writable knowledge authority or a
    different deployment boundary."

**Current mechanism (inspected).**
- **Evidence.** Python owner adapters (`evidence_sources.py`, `evidence_projection.py`) write
  snapshot JSON files. The Rust `pse-evidence` binary (`xtask/src/evidence_tools.rs`) then answers
  questions by one of two routes:
  - *offline:* supplied snapshot files go through `evidence_query.rs` into an in-memory DataFusion
    relation;
  - *indexed:* `import` stores content-addressed generations in the tooling SurrealDB database.
    `analyze` then reads a `source_cut` back and runs **the same** DataFusion questions.
- **Knowledge.** The Python corpus builder (`knowledge_store.corpus`, over `workspace_sources` and
  `workspace_domains`) produces JSON, which `knowledge-import` writes into SurrealDB. That import
  covers staged reconciliation, CAS activation, reader and importer roles, full-text and graph
  queries, and cursors.
- **Source-only path.** A separate source-only query surface (`workspace_domain_contracts.py`, plus
  `just context`) works without the projection, by design (ADR-0222).

**Correction to the account.** The "several translations" are mostly one declaration. The registry
(`xtask/src/development/catalog/*`) generates the SurrealDB DDL and codecs, the JSON schemas and
Python msgspec types: `scripts/generated/evidence_snapshot_input.py` (`EvidenceSnapshot`,
`EvidenceFact`) and `scripts/generated/knowledge_query_input.py` (`KnowledgeQuery`, `KnowledgeMode`
and the kind enums). The real restatements are on the Python side:
- `evidence_projection.py` re-declares `EvidenceSnapshot` and `EvidenceFact` as hand-written
  TypedDicts, field for field.
- `knowledge_store.py` re-declares the query fields and the `--mode` choices.
- Neither generated module has a non-test importer. They are loaded only by
  `check_private_contracts.py` (a `rg` search across the repository, excluding generated, target and
  build directories, found none).
- The pydantic models in `workspace_domain_contracts.py` are the source-only surface, not a twin of
  the indexed path.

### Credible complete options

1. **Retain current.**
2. **Store-neutral consolidation.** The Python wrappers consume the generated wire declarations.
   This is the recommended change, detailed below.
3. **Parquet plus DataFusion** (`arrow-file-interchange/derived-index-as-files`). Evidence
   generations become content-addressed snapshot files consumed by the existing offline route, which
   merges the indexed and offline evidence paths. Knowledge tables become Parquet, with graph queries
   through recursive CTEs.
4. **SQLite** (`sqlite/rebuildable-local-derived-index`: stdlib `sqlite3` or a new `rusqlite`). FTS5
   for lexical search, recursive CTEs for the graph, and atomic file swap for rebuilds.
5. **DuckDB** (`duckdb/in-place-file-query-engine`; a new dependency, 2.0.0 or 1.x).
6. **Collapse onto one language.** Either all Rust (re-implement the readers of Python-owned records)
   or all Python (`datafusion` 55 for Python plus a Python SurrealDB SDK).

### Comparative argument

**Option 6 is ruled out on ownership.**
- **The adapters belong with Python.** They read records that Python owners write: validation
  receipts, run inspection, resource records. Porting them to Rust would duplicate those readers.
- **The executor belongs with Rust.** The registry declarations, the workspace DataFusion type
  universe and the SurrealDB SDK all live there. Python DataFusion bundles a separate engine outside
  that universe.
- **The seam is right.** One registry-declared, versioned request or snapshot document crosses the
  boundary, so the two-language split is deliberate (DP-17). The defect is that Python does not use
  the declaration (DP-01, DP-16: generated output must not be bypassed by a hand twin).

**Option 2 is a clear, light win.**
- **What it does.**
  - Delete the two TypedDicts.
  - Type the adapters against the generated `EvidenceSnapshot` and `EvidenceFact`.
  - Validate once, with `msgspec.convert(value, EvidenceSnapshot)`, at the existing publication
    points before `resource_store.publish` (snapshot, import and offline in `evidence_store.py`).
  - Build the knowledge query request through `KnowledgeQuery`.
  - Take `--mode` and the enumerated choices from the generated `StrEnum`s.
- **What it gains.** A registry change now surfaces statically, with pyrefly or ty over the
  generated types, and refusal happens in Python before the native binary is spawned. Today the
  first refusal is Rust's decode.
- **Risk.** Low. Rust computes the generation identity from the decoded, typed snapshot
  (`EvidenceSnapshot::identity` in `import_snapshot`), so `null`-versus-absent encoding differences
  cannot trigger `Reused`.

**Options 3 to 5 depend on U3.**
- **What they would remove:** the service dependency for indexed reads, the private tooling roles
  and credentials, the SurrealQL staging, activation and cursor code, and the generated tooling DDL.
- **What they would add:** file-level CAS activation and a pointer, staleness metadata, cursor
  semantics over files, and locking for concurrent importers (CBO-047's primitives).
- **The common gap is lexical discovery.**
  - *Option 3:* DataFusion has no full-text index, so lexical discovery degrades to substring or
    regex scans. Under the no-compromise rule that is a regression, unless a text index such as
    tantivy is added, which is heavier still.
  - *Option 4:* SQLite FTS5 fits knowledge well. But the knowledge model is registry-declared and
    lowered to SurrealDB, so SQLite would need a new lowering (generated DDL) or a second authority.
    Unless the SurrealDB path were removed, it would also be a third store (LOP-09).
  - *Option 5:* DuckDB adds a second SQL engine beside DataFusion, with no capability the others
    lack. It is dominated by options 3 and 4.
- **A partial step is possible.** For evidence alone, option 3 is coherent: it collapses two read
  routes into one and uses an engine already in place. Knowledge holds evidence only by original
  identity (`EvidenceIdentity`), not by record link, so the two stores could be separated.

### Revealing case

Two agents import different sources into one knowledge corpus while a third pages through a query.
- **Today:** staged reconciliation plus CAS activation and issuer, query and cut cursor fences mean
  neither import replaces the other, and the reader never mixes pages (ADR-0222 drivers).
- **A file or SQLite store** must reproduce that through locks and pointer CAS. That is achievable
  but bespoke.
- **Option 2** leaves this behaviour untouched.

The case that challenges the retained store runs the other way. The canonical service is
unavailable because it is being replaced or is not started, and an agent wants an indexed
graph or lexical answer. Today that is refused, because indexed reads "never start a store". The
source-only exact queries still answer, but graph and lexical queries do not.

### Disposition

**Recommend: the bounded, store-neutral consolidation (option 2).** The store question is
conditional on U3.

**Bounded target.**
- **Mechanisms:**
  - msgspec 0.21.1 `msgspec.convert`, strict, with the generated `forbid_unknown_fields` Structs;
  - the generated `StrEnum`s for argparse `choices`;
  - no new dependency and no generator change. The generated modules already exist and pass the
    bootstrap import boundary in `check_private_contracts.py`, which imports only msgspec.
- **Responsibility boundaries.** The Rust registry stays the authority. Python adapters remain owner
  readers and become typed producers. Rust decode stays the final enforcement point (DP-03).
- **Behaviour change.** Malformed snapshots and requests are refused in Python with a msgspec error
  before the native binary runs. No output change.
- **Machinery that disappears:** the TypedDicts in `evidence_projection.py` and the hand-written
  `--mode` choices list.
- **Machinery that remains:**
  - the hand-declared field list of the knowledge query in argparse, since a flag surface is still
    wanted;
  - the evidence *question* document (`operation`, `crate_name`, …), which has no generated Python
    binding today. Adding one would be an optional registry declaration and is not needed for this
    change.
- **Transition.** A local edit across `evidence_projection.py`, `evidence_sources.py`,
  `evidence_store.py` and `knowledge_store.py`. Run the targeted adapter tests.

**U3 = yes** (the "SurrealDB stays canonical" premise covers development evidence and knowledge).
Retain the private tooling database for both families. Options 3 to 5 are excluded as authorities.
Parquet would be tactical only, as a payload format, and there is no payload here that needs it.

**U3 = no.** The store question is **unresolved, leaning retain**. ADR-0222 was decided four days
before this review and implemented with Tested controls. No current consumer is known to need a
different deployment boundary, which is its revisit trigger.
- **Settling evidence:**
  - whether any consumer, human or agent, needs indexed generation, graph or lexical reads while the
    canonical service is unavailable;
  - how often indexed routes are used compared with the offline and source-only routes (from session
    or capability-server history);
  - whether lexical ranking semantics are relied on.
- **If service-free indexed reads are needed,** the coherent first step is evidence only:
  - retire the SurrealDB evidence generations;
  - store content-addressed snapshot files with an atomic active pointer;
  - answer `analyze` through the existing offline DataFusion route, giving one evidence read path.
- **Knowledge would follow** only with a registry lowering to SQLite FTS5 (`rusqlite`), never as a
  Python-side second authority.

### Rule impacts

- **Option 2:** none. It is ordinary tooling code, and generated paths are untouched.
- **Any store change under U3 = no:**
  - amend blueprint §24.1 and §3.2 through the design-edit route, with a revision row;
  - amend ADR-0219 and ADR-0222, which can still be revised while *proposed*, or supersede them;
  - follow the decisions table in `.claude/rules/decisions.md` for a changed relation-family
    deployment.

### Evidence labels

- *Interface-checked:*
  - the generated modules and their absence of importers (searched with `rg` across the repository);
  - the field-for-field match between the TypedDicts and the generated Structs;
  - Rust `import_snapshot` computing identity from the typed snapshot;
  - the routing of `evidence_store.py` (`analyze` and `run_offline`) and `knowledge_store.py`;
  - `evidence_tools.rs`;
  - blueprint §24.1 and ADR-0219 and ADR-0222.
- *Proposed:* the recommendation and the U3 branches.
- Library facts for options 3 to 5 are inherited from the library accounts, with their own labels.

---

<a id="cbo-051"></a>

## CBO-051 — Hand-written wire and front-matter parsers in tooling

### Responsibility and target intent

**Wire.**
- *Trace capture:* a finite, on-demand capture of SurrealDB server span identities during one owned
  operation (`scripts/surreal_telemetry.py`). Bounds: 16 MiB total, 4,096 spans, 1 MiB frames,
  8 MiB per stream and 32 streams, all within an absolute deadline. It never retains attributes or
  payloads. It decodes through `scripts/grpc_wire.fields`.
- *Test client:* a raw HTTP/2 gRPC client in the test helper `scripts/tests/surreal_mx_check.py`.
  Its docstring states that it exists so it "can abruptly drop a connection without an SDK session
  destructor concealing server-side cleanup defects".

**Front matter.**
- One strict reader, `document_metadata.read`. It uses line-exact delimiters, `ruamel` safe and
  pure, with duplicate and merge keys rejected, and `adr.py` delegates to it.
- The edit-guard hook makes a stdlib, fail-closed read of `status:` (`scripts/agent-hooks.py`).
- The agent-config checker strips front matter with a regex before scanning references
  (`check_agent_config.body_of`).

### Credible complete options

1. **Retain current.**
2. **Switch the counterparty to OTLP/HTTP.** This is LOP-X010's primary remedy. A stdlib
   `http.server` handler would replace the HTTP/2 framing.
3. **Decode with opentelemetry-proto** (`opentelemetry-proto/otlp-message-decoding`: 1.45.1 and
   protobuf 7.36.2, installed transitively through the `quality` group). This would replace only the
   four-level field walk in `Capture.observe`.
4. **h2 sans-IO plus opentelemetry-proto** (`h2/sans-io-http2-protocol`: 4.4.1, a new dependency
   with `hpack` and `hyperframe`).
5. **A grpcio server with OTLP stubs** (`grpcio/python-grpc-server-for-stub-services`: 1.85.0, a new
   compiled dependency).
6. **A temporary collector.** Run a short-lived `otelcol-contrib` with an OTLP gRPC receiver and a
   file exporter, then read its JSON with the stdlib. The binary is already declared by
   `telemetry_service.py` as an optional installation.
7. **Front matter, shared stripper.** The two stdlib strippers share one stdlib delimiter function
   exposed by `document_metadata`. python-frontmatter itself is ruled out, as its library account
   found.

### Comparative argument

**Option 2's premise is false.**
- **The server cannot export HTTP.** At tag `v3.3.4`, SurrealDB's server builds its span exporter
  with `opentelemetry_otlp::SpanExporter::builder().with_tonic()`
  (`surrealdb/server/src/telemetry/traces/mod.rs`). It compiles `opentelemetry-otlp` 0.31.1 with
  features `grpc-tonic, tls, tls-roots, trace, metrics, logs, internal-logs` and no `http-proto`
  (`surrealdb/server/Cargo.toml`). The standard `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT` variable is
  honoured, but no protocol variable can select HTTP.
- **The repository already reflects this.** `surreal_server.py` points SurrealDB at the collector's
  `otlp_grpc_endpoint`, while the repository's own Python and Rust producers use HTTP/protobuf
  (`telemetry_config.py`; agent environment guide, "Optional native telemetry").
- **The probe is withdrawn.** The library accounts' probe request about the OTLP HTTP exporter is
  answered statically.

**The capture sink is a minimal fixed peer that is stricter than library defaults.**
- **It refuses** compressed messages, oversized frames and streams, and too many concurrent streams.
- **It keeps only** span identities.
- **Option 3** removes about 25 lines that walk OTLP v1 field numbers. Those numbers are frozen by
  OTLP's stability guarantees. In exchange it adds a direct pin, a protobuf import and
  unknown-field tolerance. `grpc_wire` stays anyway for the test helper, so this is not a clear win.
- **Option 4** removes the frame loop and the static HPACK bytes for three new pins, in a
  diagnostic-only sink. It is not a clear win under "tooling changes stay light".
- **Option 5** adds a compiled C-core runtime with threads to manage, for a finite sink. It is
  dominated.
- **Option 6** removes all hand protocol code, but makes capture depend on the optional collector
  installation. It also adds a process lifecycle: readiness, a fixed port instead of an ephemeral
  `port 0`, flush on shutdown, and isolation through the capture nonce. Capture currently needs only
  the stdlib, so this weakens its availability, which the no-compromise rule forbids.

**The test client is deliberately raw** (the countercondition in LOP-X010). A library client would
mask the abrupt-drop behaviour the test exists to provoke. Generating Python stubs for SurrealDB's
protocol would add a codegen hop for a test helper.

**Front matter is already consolidated** where it matters.
- The two stdlib strippers do different jobs. The hook makes a single-key, fail-closed read on the
  cold edit-guard path. The checker strips for reference scanning.
- Routing the hook through `document_metadata` would couple the edit guard to a larger module whose
  breakage blocks or permits every edit.
- **Static check:** for every current `docs/adr/[0-9]*.md`, the hook's `status:` read equals a
  line-exact delimiter read. No divergence was found.

### Revealing case

A future SurrealDB release changes tonic's HTTP/2 behaviour, for example by sending CONTINUATION
frames or a larger SETTINGS frame.
- **The hand peer** ignores unknown frame kinds and still acknowledges SETTINGS and PING, so it
  degrades to "capture error recorded" rather than a false capture. That is acceptable for a
  diagnostic sink, and the failure stays visible in `capture_errors`.
- **Option 4 (h2)** would handle such changes better. The case does not justify three pins now.

### Disposition

**Retain current.** The premise is rejected for the OTLP/HTTP route and for the front-matter
consolidation.

**Residual work.** Add a one-line §F reason to the module docstring of `surreal_telemetry.py`
(DP-13: "with its reason stated briefly where the next reader will look"). It should say that
SurrealDB 3.3.4 exports OTLP over gRPC only through tonic, and name the alternatives considered
(h2, grpcio, a collector, OTLP stubs).

**Revisit trigger.** A SurrealDB server release that offers an HTTP/protobuf exporter. Then a stdlib
`http.server` handler, using either the existing walk or opentelemetry-proto, replaces the HTTP/2
framing.

**U1 = yes.** Contributing exporter-protocol selection upstream to SurrealDB (honouring
`OTEL_EXPORTER_OTLP_TRACES_PROTOCOL` with an `http-proto` feature) becomes a legitimate option. It
is not available until released, and its value is small: about 60 lines of framing.

**U1 = no.** Retain until the trigger fires.

### Rule impacts

None. Adding a docstring is ordinary.

### Evidence labels

- *Interface-checked:*
  - the upstream SurrealDB 3.3.4 server exporter construction and Cargo features (GitHub contents
    API at tag `v3.3.4`);
  - the local sources `surreal_telemetry.py`, `grpc_wire.py`, the head of `surreal_mx_check.py`, the
    telemetry environment composition in `surreal_server.py`, and the collector configuration in
    `telemetry_service.py`;
  - the front-matter readers.
- *Tested:* none. The ADR `status:` comparison is a static script run over the current files, not a
  product test.
- *Proposed:* the disposition.

---

<a id="cbo-052"></a>

## CBO-052 — Several command surfaces without a shared declaration

### Responsibility and target intent

The maintainer and agents in both runtimes discover and run workflows. Each command runs with the
right admission and environment, and arguments are parsed once by the owner of their meaning.

**Current contract.** `.claude/rules/tooling.md` says "Recipes are thin":
- a recipe composes the real tool;
- it passes `*args` through as positional data;
- it leaves selection visible to Cargo and nextest;
- native recipes declare capabilities through `[script(... scripts/pse-env --native=…)]`.

AGENTS.md says recipes are shortcuts, not gates, and that bare tools are first-class.

### Credible complete options

1. **Retain current.**
2. **cyclopts for all in-environment scripts** (`cyclopts/typed-cli-from-signatures`: 5.3.0, already
   pinned in `quality`). Shared option dataclasses and lazy commands.
3. **just `[arg]` flags and `mod` files** (`just/declarative-recipe-interface`: 1.58.0).
4. **Collapse the preset recipes into nextest profiles** (`cargo-nextest`, through profiles).
5. **Typer** (`typer/click-based-typed-cli`, transitive only).
6. **FastMCP `generate-cli`** (`fastmcp/generate-cli-from-tool-schemas`: 4.1.3).

### Comparative argument

**The premise does not hold for the bulk of the surface.**
- **Recipes forward opaquely.**
  - 194 of 231 recipes have one-line bodies.
  - 100 of those are Python forwards (`python -m scripts.X …` with `"$@"` or no parameters), counted
    from `just --dump --dump-format json`.
  - An option change touches only the script that parses it.
- **Recipe metadata is derived, not re-declared.** `scripts/capabilities/entrypoints.py` builds
  recipe descriptions from the `just --dump` JSON.
- **The MCP and command-line pair already has one declaration.** `capabilities/cli_codegen.py`
  generates `scripts/generated/capabilities/cli.py`, a cyclopts App, from the canonical tool schemas.
- **xtask's clap surface** is the Rust tools' own parser. The Python wrappers pass JSON request files
  to it, not flags.

**The repeated vocabulary is mostly homonyms.** Across 64 non-test argparse scripts there are 421
long-option declarations under 281 distinct names. The most repeated:

| Option | Declarations | Meanings found |
| --- | --- | --- |
| `--output` | 18 | — |
| `--state` | 17 | At least three. A SurrealDB state directory defaulted from `PSE_SURREAL_STATE` (`evidence_store`, `knowledge_store`). The same directory but required, for destructive maintenance (`surreal_diagnostics`, `surrealkit_maintenance`). A plan lifecycle state with choices (`document_lifecycle`). |
| `--json` | 7 | — |
| `--native` | 5 | At least two. A capability list in `pse_env` (`nargs="?"`). Feature-composition booleans elsewhere. |

A shared option layer would either erase these distinctions or keep per-script declarations anyway.
argparse is the stdlib CLI library, so DP-13 is satisfied.

**Why each option fails.**
- **Option 2** converts about 63 scripts for a syntax change.
  - Some must stay stdlib-only: bootstrap, hooks and cold paths.
  - Import cost rises; the library account made one unbenchmarked observation of about 28 ms
    against 3 ms.
  - Positional compatibility must be re-checked for every recipe.
  - It is not a clear win under "stay light".
- **Option 3:** `[arg(long)]` conflicts with the positional recipe contract, and `mod` changes only
  how the file is organized.
- **Option 4:** nextest profiles cannot carry cargo package or feature selection, as the library
  account found.
- **Option 5** would be a second framework (LOP-09).
- **Option 6** routes calls through an MCP client and bypasses the shared runtime, as the library
  account found.

**The one genuine restatement** is the Python wrappers re-declaring Rust-owned request fields:
`knowledge query` flags and choices, and the `evidence offline` and `analyze` `--operation`
choices. The CBO-050 recommendation covers the part that has a generated binding.

### Revealing case

A shared `--state` option defaulted from `PSE_SURREAL_STATE` would be adopted by
`surrealkit_maintenance` or `surreal_diagnostics`. Their maintenance and diagnostic commands
deliberately *require* an explicit path. Applying the shared default would let a destructive
maintenance command act on whatever store the environment selects, which weakens an explicit-path
guarantee.

### Disposition

**Premise rejected.** The surfaces are layers with distinct owners: admission and discovery in the
recipes, argument meaning in the scripts, and generated MCP and CLI declarations. The real
restatement is the cross-language one handled under CBO-050.

**Residual work.** None required. A just variable for the repeated
`"{{ py }}" -m scripts.arrow_validation nextest {{ nextest_action }}` prefix would be cosmetic. It
can be done in passing, but does not merit its own change.

### Rule impacts

None.

### Evidence labels

- *Interface-checked:*
  - the local `just --dump` JSON, analysed with a scratch script;
  - the `rg` counts of argparse declarations;
  - the variants of `--state` and `--native` (each declaration read);
  - `cli_codegen.py`, the generated CLI imports and `entrypoints.py`;
  - `.claude/rules/tooling.md`.
- *Proposed:* the disposition.

## Pattern notes

No new pattern. [LOP-X010](../assessment-patterns.md#lop-x010) is refined in revision 3: check the counterparty's
build as well as its configuration surface (SurrealDB 3.3.4 compiles only the gRPC OTLP exporter),
and when the counterparty is a third-party binary, the hand-built peer may be the minimal correct
remedy until an upstream change is released. Base-framework notes for LOP-02, LOP-03 and LOP-09 are
recorded in that output's maintenance notes.

## Limits

- **Not read:**
  - the bodies of `surreal_server.py` (beyond the telemetry environment composition),
    `test_resources.py` and `test_run.py` (beyond the Rust run, capture and report paths);
  - `workspace_domains.py` and `workspace_sources.py` bodies;
  - most dict-building sites in `evidence_sources.py`, so the size of the CBO-050 transition is a
    qualitative judgment;
  - SurrealDB's metrics exporter (only traces matter for the capture sink).
- **Not verified directly:**
  - The nextest group-reference rule was read from upstream `main`, through the docs mirror and
    raw GitHub, not at the 0.9.146 tag.
  - `msgspec.convert` strictness for these Structs relies on msgspec 0.21.1 documentation and G8's
    msgspec probe; no test was run.
- **Not checked:** usage history for the evidence and knowledge commands.
- **Not done:** no builds, probes or tests. The only executions were static: `rg`, `git`, the
  `just --dump` export, and one stdlib Python script comparing ADR front-matter reads.
- **Assumption:** CBO-052's counts exclude `scripts/tests/`, and multi-line `add_argument` calls
  were counted with a multiline `rg`.
