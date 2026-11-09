# SurrealDB v4 discovery: coverage and evidence

Research date: 2026-11-09. Baseline: `053ac1c93c84056456f60557dca73256ccb5b283`
with concurrent uncommitted Plan66/runtime/math/model changes. Those changes were preserved;
the inspected SurrealDB integration owners were unchanged at entry. Source observations are
representative reads, not a repository-wide semantic reference census. Follow the
[integrated briefing](briefing.md) for priorities and the thematic briefs for exact sources.

During publication another session committed the concurrent implementation and some
in-progress discovery documents in `408cba4df`. That commit does not change the inspected
source conditions or imply completion of this discovery. The final integrated document set
is the deliverable; source permalinks retain the original baseline where those files were
unchanged, and current plan links retain their owning status.

## Selected realizations

| Realization | Baseline and distinction |
| --- | --- |
| Server | V4.0.0 at `284fe93afc851c3baec2c262c01289602021a70d`; MX 1.1.0; source-built `storage-mem,cli,surrealism` with maintained runtime, extension, compiler and DiskANN patches. [Source owner](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/crates/pse-host/src/server.rs), [integration guide](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/scripts/surrealdb-source/source-build.md). |
| Native client and values | Protocol 0.14.1, types/expr/parser-next 4.0.0; production direct gRPC differs from the SDK's lifecycle. [Manifest](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/Cargo.toml). |
| Storage profile | Persistent MX with explicitly selected serializable isolation and synchronous AOL policy; not a deployment of SurrealKV merely because it is the standard disk backend. [Service owner](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/crates/pse-operations/src/native_service.rs). |
| Extensions | Surrealism 0.7.0 and prepared WASI study policy. Selected ABI/runtime patches and capabilities matter independently of server release availability. |
| SurrealKit | 1.0.0-rc.1, `625e264c4e134def842bfe8ac9d2f918a42bb6c3`; prepared with `--no-default-features --features cli`, SDK4 gRPC/rustls and unwind. Offline embedded compilation and MCP require a changed feature selection. [Preparation owner](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/crates/pse-host/src/surrealkit.rs). |

These are inspected declarations/configuration, not a fresh attestation of a running listener.
Official rolling documentation was consulted alongside exact tagged sources. Companion versions
are independently selected; a v4 server does not establish support in every SDK or tool.

## Supplied references: full population

The planning and execution investigation consumed all three documents end to end, including
their addenda: 2,941 lines of [SurrealDB v4](../../../libraries/surrealDB/SurrealDB_v4.md),
2,473 lines of [SurrealKit](../../../libraries/surrealDB/SurrealKit.md), and 2,212 lines of
[Protobuf with v4](../../../libraries/proto/Protobuf_SurrealDB_v4.md). Section numbers below
refer to each original body or its separately numbered addendum. This is thematic coverage;
it does not claim independent verification of every overview assertion.

| Reference sections | Primary home / disposition |
| --- | --- |
| V4 original 1–3, addendum 7–8, 13.1–13.5 | Data/schema and native values: named types, numeric/temporal contracts, identity, flexibility, validation order, sequences, presence and changed comparison/collection behavior. |
| V4 original 4, addendum 1 and 11 | Graph: links, conventional/lightweight edges, references, adjacency, payload predicates, materialized inverses, GQL; GraphQL remains a client-surface alternative. |
| V4 original 2/5, addendum 3–5 | Query/index/search and observability: executor, counts, bitmap fusion, vector/full-text/spatial access, index construction, plan metrics, spilling and scheduling. |
| V4 original 6–7, addendum 2/9/12 | Transactions, derived state, programming and transports: isolation, views, events, live/feed semantics, stored procedures, streaming and durable sessions. |
| V4 original 8–9/12, addendum 6/9–12/14 | Native values and clients, extensions and tooling: SQL/pgvector/GraphQL, CBOR, SDK, APIs, auth/capabilities, MCP, removed SurrealML and independent companion products. |
| V4 original 10–11/13–15, addendum 13–15 | Storage, rollouts and diagnostics: engines, durability/history, backup/import, migration, version conditions and source routes. Licensing is not a technical rejection criterion. |
| Kit original 1–6, addendum 5–8/11–12 | Deployment: synchronization, modules/targets, seeds, rollout history/recovery, compatibility levels and native migration effects. |
| Kit original 7/9, addendum 1–3/10 | Compilation/generation: embedded semantics, schema v2, external emitters and Rust interfaces; compare selected feature availability and semantic coverage. |
| Kit original 8/10–13, addendum 4/9/13–15 plus scope clarification | Tests/developer workflows: actors, independent tests, cleanup, MCP, templates/build tools and version boundaries. Beta.6/SDK3 original claims are historical where rc.1 supersedes them. |
| Proto original 1–5/10–12, addendum 1–4/9/11–12 | Values, clients and generation: native codecs versus domain contracts, reflection, validation, fidelity, shaped/alternative binary results and conformance. |
| Proto original 6–9, addendum 5–8/10 | Storage, graph, extensions and observation: standalone engines, transaction boundaries, descriptor processing, binary APIs, projections, partial updates and durable streams. |

## Existing operations inspected

| Responsibility | Representative sources and question |
| --- | --- |
| Schema and codecs | `pse-schema` declarations, `pse-codegen/src/codegen/surreal*.rs`, `canonical_codec.rs`: remove duplicated structural interpretation while retaining exact identities/values. |
| Publication and operational transitions | `canonical.rs`, execution, studies, retention and result owners: consolidate guarded operations without losing uncertain-outcome recovery. |
| Bounded graph and metadata | `canonical_explanations.rs`, `canonical_metadata.rs`, private development knowledge/evidence: compare actual query intent with native representations and planner access. |
| Protected results and historical interpretation | Canonical results/readers and Plan66 consuming routes: metadata pushdown cannot establish scientific admission or dispose original payloads. |
| Transport | `canonical_client.rs` and its wire/results/connection owners: cumulative preflight, statement accounting, typed failures and physical settlement differ from per-message limits. |
| Native deployment | Host server/module/kit preparation, `native_service.rs`, runtime controls patches: configured capability differs from upstream availability. |
| Tooling and fixtures | SurrealKit tests, rollout preview/frozen rollout, development knowledge/evidence and telemetry: compare complete preparation/use/recovery/disposal. |

Source links in the thematic briefs identify the actual reads. Blueprint §4/§5/§20/§21/§23,
the operating guide and current Plan66/68 packets provide contracts and context. Proposed or
unfinished consumers were not credited as deployed functionality. Historical test evidence
keeps its original scope; it was not rerun for this account.

## Less relevant and conditional territory

| Territory | Current disposition and trigger |
| --- | --- |
| Vector/hybrid and spatial retrieval | Native capabilities remain eligible, but no embedding pipeline or geographical workflow is invented. Prioritize existing lexical/structured discovery; deepen ANN/spatial research only for an actual current query need. |
| Browser/WASM storage, Earth, Cloud and SurrealDS | Alternative realizations, not current deployments. Revisit where local persistence, isolation, recovery or capacity obligations justify the full operational change; no licence-based exclusion. |
| PostgreSQL/MySQL, pgvector, mql.js/Mongoose | Existing-client simplification candidates, not presumed replacements for rich native RPC or new foreign clients. Exact dialect/type/catalog compatibility needs a consumer-specific check. |
| Agent Memory and Kafka/surreal-sync | Independent products/integrations, not implied needs of study history or development knowledge. No current need established by the representative reads. |
| SurrealML | Removed in v4; cannot provide a new native inference route. |
| Sequences | Useful allocation primitive, but gaps and allocation order do not replace committed ordinals or guarded counters. |
| GraphQL, database APIs, cookies/JWT and MCP | Consider for existing inspection/actor/developer access. Do not create a frontend or weaken authority merely to use an interface. |

## Limits and changed premises

The old family account and Proto's v4-excluded briefing remain dated research inputs. Claims
about unmatched SDKs, old module caching/ABI, removed backends and v3 numeric conversion are
not silently inherited. The new briefs carry specific rechecks and residual questions.

No SurrealDB generated reference/API bundle was prepared: `.config/library-leverage.toml`
selects other families. Spelling searches found candidate owners; representative source reads
ground their interpretation. Missing observations do not establish universal absence.

Rolling docs and release-day publications can disagree on SQL windows/joined writes,
experimental capability names and migration readiness. Prefer exact release source/tests for
a consequential choice; unresolved API exposure, lifecycle and fidelity questions remain
explicit in their briefs. Vendor fetch failures do not establish feature absence.

Three consequential availability limits surfaced: the selected build omits GQL, so its
distinct/hash-join spill cannot be credited to deployed SurrealQL; ordinary graph readers
use indexed relation scans while generated inline/reference mechanisms are already selected
for some families; and the patched isolation policy differs from upstream defaults. These
are reasons to compare exact compositions, not label all native facilities first adoption.
Fetched CBOR and in-memory pages also contained older material; v4 codec/backend specifics
need exact tagged evidence rather than treating every rolling page as version-matched.

No runtime probe was necessary to identify these research directions. No throughput, memory,
durability, protocol-conformance or assembled scientific qualification is claimed. Proposed
follow-ups name focused cases; existing Plan68 receiving results remain with their owners.
