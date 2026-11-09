# SurrealDB v4: discovery for existing functionality

Inspected 2026-11-09 at `053ac1c93c84056456f60557dca73256ccb5b283`, including the
concurrent dirty Plan66/runtime implementation. [Coverage and source conditions](coverage.md)
define this account's limits. Source/API observations are **Interface-checked**;
the improvements and benefits below are **Proposed**. No runtime probe or performance
campaign was run for this discovery.

## Direction

The strongest research direction is to expose more of each existing operation's intent to
the database and its tooling, allowing native planning, representation, transactions and
schema interpretation to remove repeated application work. This includes useful 3.x
capabilities and new combinations enabled by v4. A feature's release age does not rank its value.

Breaking changes are acceptable. The current persistent MX engine, direct gRPC client,
registry generation, database policy placement and tool preparation are comparison inputs,
not fixed target choices. Preserve required scientific meaning, truthful completeness and
effect ownership while reconsidering how those guarantees are implemented. Compare complete
operations: a smaller adapter can still introduce more maintenance or recovery work.

Plan68 established the compatibility baseline and some selected mechanisms. It did not exhaust
v4 leverage. Its [remaining investigations](../../../plans/68-surrealdb-v4-cutover.md)
remain with their owning packets; this discovery creates neither new acceptance nor a second
status ledger. Future Plan66 capabilities are not invented consumers for these hypotheses.

## Current deployment and reading route

One canonical database serves authored revisions, portable compilation products, operational
state, result descriptors and analyses. Disposable development evidence/knowledge selections
have separate authority and custody. Native Rust service owners manage launch, authentication,
schema readiness and physical settlement. Production uses the bounded supplier-protocol client;
the SDK also serves standalone tooling. Persistent MX, extension preparation and patches are
explicit source-built selections. See blueprint §20 and the [operating guide](../../../dev/surreal-substrate.md).

| Capability family / existing responsibility | Discovery home |
| --- | --- |
| Generated canonical declarations, identities, values and constraints | [Data model and schema](briefings/data-model-and-schema.md) |
| Study, provenance and explanation relationships | [Graph representation and traversal](briefings/graph-representation-and-traversal.md) |
| Bounded readers, metadata discovery and private knowledge search | [Query planning, indexes and search](briefings/query-planning-indexes-and-search.md) |
| Publication, guarded transitions, protection and acknowledgements | [Transactions and guarded state](briefings/transactions-and-guarded-state.md) |
| Summaries, freshness, observation and reconstruction | [Derived state and change observation](briefings/derived-state-and-change-observation.md) |
| Persistence, retained originals, binary products and service recovery | [Storage, durability and artifacts](briefings/storage-durability-and-artifacts.md) |
| Exact values, bounded requests, response completeness and clients | [Native values, transports and clients](briefings/native-values-transports-and-clients.md) |
| Existing study policy, actor APIs and callable database operations | [Database programming and extensions](briefings/database-programming-and-extensions.md) |
| Registry output, installed interpretation and schema verification | [Schema compilation and generation](briefings/schema-compilation-and-generation.md) |
| Frozen schema selection, rollout preview and data evolution | [Deployment, rollouts and migrations](briefings/deployment-rollouts-and-migrations.md) |
| Native service fixtures, security tests and independent tools | [Contract testing and developer workflows](briefings/contract-testing-and-developer-workflows.md) |
| Query plans, storage signals and development evidence | [Observability and performance](briefings/observability-and-performance.md) |

## First research priorities

These priorities order investigation, not implementation. Each can enter focused applicability
research directly; the detailed briefs keep additional eligible directions and counterconditions.

| Priority | Mechanism and potential improvement | What could defeat it / next question |
| --- | --- | --- |
| First: operation-shaped native queries | V4 parameter planning, direct IDs, compound/index-only access and bitmap fusion could replace candidate hydration, repeated filtering or overly constrained manual index selection in existing readers. Compose authorization, projection and bounded ordering in one query. | Forced access paths may already be appropriate; plan changes can alter work without improving latency. Compare the actual workbench/protected-reader and evidence query builders with authorized expected results, skew, empty selections and limit-plus-one completeness. |
| First: relationship representation | Lightweight edges, tracked references, inline caches/properties and materialized inverses could remove manually maintained adjacency or duplicate lookup structures for actual study/provenance neighborhoods. | Edge occurrence identity, evidence fields and deletion obligations can rule out endpoint-pair identity. Classify real relation families before selecting a representation; compare write amplification and high-degree behavior. |
| First: schema interpretation and verification | SurrealKit compile/introspection/v2 plus native named types could derive more structural checks and projections from existing meaning, removing repeated parsing and shape interpretation. | The prepared CLI omits embedded compilation, v2 has coverage limits, and a digest covers only selected semantics. Which repeated checks can one engine-normalized schema discharge without introducing another editable authority? |
| First: complete transaction procedures | Native SSI/read-only modes, stored functions and statement retry could consolidate client-orchestrated guarded operations and narrow conflicts. | Original-operation acknowledgements, absent/predicate dependencies and uncertain outcomes still require a complete owner. Can one selected publication or retention transition become simpler while retaining those guarantees? |
| Next: native representation through the whole boundary | Sized values, native derives and suitable binary transport could remove Decimal workarounds or hand mapping for selected generated fields. | The native Proto codec can widen/degrade rich values; replacing an encoder alone does not preserve the stored/query/wire contract. Establish full-range integers, IEEE bits, NONE/NULL and historical interpretation end to end. |
| Next: supplier-owned deployment/test composition | SurrealKit semantic verification, frozen rollouts, migration completion and actor/external tests could replace duplicate schema and fixture orchestration. | Current native owners supply listener identity, credentials, deadlines and disposal absent from a simple tool invocation. Compare the complete receiving lane, including interruption and failed cleanup. |
| Conditional: derived state and storage alternatives | Native views/events/counts can replace eligible repeated summaries; KV or native buckets may simplify a concrete persistence/artifact lifecycle. | Incomplete refresh dependencies or a new transaction boundary can cost more than recomputation/current files. Identify the actual summary or protected-artifact consumer and recovery contract first. |

## Compositions to investigate together

**Graph, planning and diagnostics.** Choose identity/representation and the query together,
then inspect native operators. Adding adjacency caches cannot help a query that deliberately
uses a secondary-index relation scan. Graph traces are explanatory metadata, not proof of a
complete semantic dependency set. Compare existing query builders before changing storage.

**Schema, values and rollout.** A named `u64` field, generated codec, Proto wire path,
installed-definition witness and historical reader form one change. SurrealKit can manage
definitions while native migration handles stored values, but neither alone proves the
other's interpretation. Keep a single declaration of meaning; derive the useful projections.

**Transactions, programming and custody.** Move a guarded operation into a stored function
or extension only if it eliminates meaningful transfer or coordination. Transaction access,
caller permissions, response completion and physical invocation settlement are distinct
questions. Database-local execution need not move scientific or mathematical authority.

**Performance and operations.** Executor parallelism, spills, storage caches and maintenance
share resources. Investigate these within the maintainer's ordinary unbounded workstation
selection; this account proposes no host CPU partition, compiler limit or RAM cap. A supplier
query-memory budget is a separate mechanism whose benefit needs a representative workload.

## Evidence and handoff

The existing [family type pages](README.md#historical-applicability-and-types) retain their
2026-10-27 baselines. The [Proto account](../proto/briefing.md) explicitly excluded v4;
its general distinction between database RPC, native-value codecs and domain contracts is
still useful, but its version-specific conclusions require rechecking. New technical notes
linked from the thematic briefs separate reusable contracts from local hypotheses.

Discovery does not choose adoption, create CBOs or revise production scope. Follow the
[framework](../../library-leverage-framework.md) into focused responsibility investigation,
capability applicability and complete comparative options. An important unresolved feature
contract is a research question, not a reason to reject a promising design. Runtime correctness,
assembled qualification and quantitative benefits remain unestablished by this document.
