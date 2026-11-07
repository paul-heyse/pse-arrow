# SurrealDB 4 cutover and functional extensions

Second pass against main `4c8b8294d` and the current Plan 66 and 67 companions.
The installed baseline is the repository's patched SurrealDB 3.3.4 with persistent
SurrealMX 1.0.0, protocol 0.13.1, and Surrealism 0.6.4. The candidate is the
`v4.0.0` tag at `284fe93afc851c3baec2c262c01289602021a70d` with
SurrealMX 1.1.0, SurrealKV 1.2.0, `surrealdb-types` 4.0.0, protocol
0.14.1, and Surrealism 0.7.0. Component versions come from the tagged
lockfile. The second pass reviewed both supplied references including their
addenda and mapped their features to current code before selecting the follow-up
investigations. The coverage appendix records the disposition of each family.
The selected standard is core 3.4, efficient architecture 1.0, test assessment
1.2, process simulator 1.5, and the pse-arrow binding.
[Database reference](../libraries/surrealDB/SurrealDB_v4.md) ·
[Protobuf reference](../libraries/proto/Protobuf_SurrealDB_v4.md) ·
[Tagged lockfile](https://github.com/surrealdb/surrealdb/blob/v4.0.0/Cargo.lock) ·
[Selected standard](../design_review/design_principles/standard.toml)

## Where v4 can change the workflow

| Current workflow | Receiving owner and limits |
| --- | --- |
| Authored definitions → generated schema → native preparation → durable publication | 66b owns generated correspondence, 66c owns execution and physical settlement, and 66e owns the canonical effects and protection witnesses. The registry and generated definitions remain the source for catalog shape. The server runs in an explicitly prepared native generation. |
| Scientific result → exact membership and protection → Arrow/Proto readback → independent views | 66a/d and 66g–j retain scientific semantics, computations and original evidence. 66e owns protected hydration and interpretation. 66k receives the facts for assessments and views. 66l owns assembled acceptance. The useful database changes affect selection and delivery of actual results, including aqueous study, trajectories, fitting and adaptive campaigns. |
| Original invocation → independent evidence → bounded history → current rerun → owner release | 66f and 67b own original verdict, projection, selected rerun and history. The current pilot already has database-clock retirement, holds, exact receipts and independent reconciliation. 67a/e own incident and recurring physical release. |
| Source preparation → database contracts and native fixtures → report → build/storage accounting | 67c owns SurrealKit and assertion migration. 67d owns effective build units and cache comparison. 67e owns service, scratch and generation settlement. |

[Plan 66](../plans/66-integrated-simulation-target.md) ·
[Plan 67](../plans/67-storage-lifecycle-and-build-reuse.md) ·
[Generated declarations](../../crates/pse-codegen/src/codegen/surreal.rs) ·
[Scientific inspection](../../crates/pse-runtime/src/workflow/result_inspection.rs) ·
[History retirement](../../xtask/src/development/developer_history/expiry.rs)

## Cutover prerequisites

**Proposed target.** Build one exact v4 generation and regenerate the catalog
in a fresh owned directory. Recreate authored and reproducible products through
their native producers. Regenerated runs receive new identities. Keep named
original evidence with its existing reader when portable custody suffices.
Use bounded one-time conversion only when that reader requires a v4 import.
Validate the installed schema, protected payload closure and independent readback
before selecting v4. Rehearse reverting to the frozen v3 selection before v4
write admission. Recover forward once v4 admits writes. Upstream's in-place
migration is experimental and one-way. Level 4 and fresh index construction
fit the regeneration premise. The proposed change crosses Plan 66's explicit
v4 exclusion and goes through the owning decision route before implementation.
[Migration guide](https://surrealdb.com/docs/build/migrating/from-old-surrealdb-versions/3x-to-4x) ·
[Current scope](../plans/66-integrated-simulation-target.md)

**Interface-checked supplier deltas.** Preserve default canonical write SSI
with actual caller coverage. Upstream offers explicit serializable transactions
on MX, while ordinary upstream statements still default to snapshot isolation.
Retain synchronous AOL and every-append fsync, snapshots, explicit capacity
and lifetime bounds, failure typing and recovery ownership. Tagged v4 gRPC
uses reconnectable session capabilities and bounded text for unary errors.
Keep authentic connection-bound session cleanup, atomic admission and
bounded native error details unless an equivalent candidate proves the
contract. API v4 stage savepoints require a test with an earlier enclosing
write before replacing the whole-transaction rollback control.
[Tagged MX adapter](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/kvs-mem/src/lib.rs) ·
[Isolation](https://surrealdb.com/docs/learn/querying/concepts-and-guides/transactions) ·
[Tagged gRPC implementation](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/server/src/rpc/grpc.rs) ·
[PSE patch purposes](../../scripts/surrealdb-source/source-build.md)

| Patch or interface group | Disposition for the candidate |
| --- | --- |
| MX source, controls and canonical profile | Upstream owns MX 1.1 integration. Compare finite bounds, startup refusal, metrics, SSI, persistence error typing and uncertain commit recovery individually before retaining or deleting local controls. Include write skew, empty predicate phantoms, contention and acknowledged-write reopen. |
| gRPC ownership, admission and status details | Rebase the three behaviors where upstream differs. Test plain/TLS socket loss, cross-connection refusal, saturation until physical cleanup, typed commit conflicts and the 8 KiB error-details boundary. |
| API rollback | Compare enclosing-write failure and intentional API responses as separate cases. Retain the poison control if an error response can otherwise commit earlier writes. |
| DiskANN and compiler | v4 locks its replacement DiskANN supplier. Prefer the tagged lock over the 3.3.4 archive override after a locked build on the pinned nightly. Drop compiler adaptations only where the actual build proves they are obsolete. |
| Study extension build, features, controllers, memory and deadlines | Use the separate `surreal-ext` and exact server receipts. Compare locked build and explicit feature forwarding, fresh-instance slot accounting, one linear memory, fail-fast typed admission, and deadlines through cold initialization and cancellation. See [O09](#O09). |
| Parser and compatibility level | Replace v3 `surrealdb-syn` and `surrealdb-sql` at generation, lint and maintenance boundaries. Use `parser-next` syntax or `surrealdb-expr::parser::expr_for_definition` where complete typed lowering is needed. Installed-definition validation still uses the running target. Test repeated clauses, field evaluation order, effect restrictions, coercion, authorization redaction, index ordering, and value/cardinality behavior on emitted inputs. |

[Tagged parser](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/expr/src/parser/api.rs) ·
[Maintenance witness](../../crates/pse-operations/src/canonical_maintenance.rs) ·
[Tagged server manifest](https://github.com/surrealdb/surrealdb/blob/v4.0.0/Cargo.toml)

<a id="O01"></a>
## O01 — Generate reusable storage constraints

**Recommend a narrow first use, 66b/66e.** Generate `DEFINE TYPE` from an
existing registry-owned contract only where at least two persisted or callable
boundaries need the same constraint. Start by comparing a shared closed
status or exact scalar constraint used by a generated field and a function
argument. The canonical attempt phase is a candidate for storage typing.
Its transition and cross-field predicates belong to the attempt owner, and
the candidate needs a second matching use before extraction. A stronger
storage and callable candidate is the unsigned decimal representation used
by metadata counts and generations. Keep context-specific assertions and
independent range checks in the codec. Check declaration order, type
permissions, coercion, optional values, enum ordering and normalization under
`INFO` before replacing emitted literals. Reject permissive rounding,
enum widening, and accidental change to the interpretation marker.

**Tested, disposable query only.** The official v4.0.0 CLI, run as
`surreal sql --endpoint memory --ns probe --db probe --json`, accepted
`DEFINE TYPE pse_exact_u64 AS decimal CONSTRAINT range ASSERT $value >= 0dec AND $value <= 18446744073709551615dec AND math::floor($value) = $value`.
A schemafull field and `fn::echo_u64` argument and return used the
same type. Both accepted the exact upper bound. The field refused
`0.5dec`, the function refused `18446744073709551616dec`, and the
field coerced integer `1` to decimal. The CLI returned status 1 for
the two expected refusals and read back exactly the two admitted rows.
This demonstrates constraint reuse and coercion. The repository's
typed transport, optional values and actor permissions need their
own controls before any generated field or callable contract moves.

Compare generated native types with the present literal unions and field
assertions. `SurrealValue::kind_of` describes conversion shape and can remove
mechanical copies where its exact contract fits. The source-owned generator
still decides domain constraints and presence. `FLEXIBLE`, named record-ID
components, expression validation, and `SQL TYPE` require an actual consumer
before widening or changing the current shape. Use level 4 evolution and
`MIGRATE EXISTING ROWS` only for a named later schema change with precise
conversion, refusal and rebuild semantics. Indexed, referenced, inline,
computed and view-dependent fields have migration restrictions.
[Named types](https://surrealdb.com/docs/reference/query-language/statements/define/type) ·
[Field semantics](https://surrealdb.com/docs/reference/query-language/statements/define/field) ·
[Schema source](../../crates/pse-schema/src/catalog/substrate.rs) ·
[Native codec](../../crates/pse-operations/src/canonical_codec.rs)

<a id="O02"></a>
## O02 — Improve protected selection and dependency inspection

**Recommend measuring actual v4 plans, 66e/66l.** Reuse
`inspect_graph_explanation` and its existing bounded, permission-aware static
and opt-in executed JSON observations. Compare all six current indexed
relation queries, covering study dependencies/consumers, run sources, artifact
consumers, and analysis source/target. Compare exact IDs, stable pagination,
parallel occurrences and protected inputs under actor and operator roles.
The current relation-table B-tree queries have no adjacency traversal to tune.
Use the existing `KnowledgeMode::Graph` traversal as the first comparison
for `GraphEdgeScan` frontier batching and predicate scope. Try inline
`relation` or `basis` only if an actual selective filter can be evaluated
from payload before record fetch. Keep exact generation and `key>$after`
ordering. Compare hub overflow, folded/delta paths, parallel edges and denied
fields. Confirm that any fewer fetches still preserve selection semantics.

**Tested, disposable query only.** An official v4.0.0 memory fixture with
two parallel edges and an inline relation field returned the expected
edge for the knowledge-shaped compound filter. Its executed plan was
`Values → Sort → Limit`, with graph counters on `Values`. The
relation-only case reported property evaluations. The compound case did
not emit that counter or a predicate-scope marker. The current diagnostic
projection can miss graph counters on `Values` with only an expression
attribute. Inspect the raw plan or extend that projection before using
it to judge the change. The probe establishes neither payload pushdown
nor a performance improvement.

**Conditional snapshot use.** Trial current `BEGIN READ ONLY` for a single
effect-free metadata request that needs a stable multi-statement snapshot.
The private knowledge inspection already opens plain transactions around
such reads and is the first candidate. Account for the additional
read-only snapshot status in the BEGIN slot and require write refusal.
The v4 gRPC request definitions have no dedicated read-only field, so use
SurrealQL through the current bounded client. Keep ticket acquisition and
publication in guarded effects. A snapshot neither proves current authority
across pages nor replaces the existing before/after generation check where
that check detects changed scope. Retain current freshness requirements.
[Graph inspection](../../crates/pse-operations/src/canonical_explanations.rs) ·
[Knowledge query](../../xtask/src/development/knowledge.rs) ·
[Read-only transactions](https://surrealdb.com/docs/reference/query-language/statements/begin) ·
[Graph controls at the v4 tag](https://github.com/surrealdb/surrealdb/tree/v4.0.0/language-tests/tests/language/graph)

<a id="O03"></a>
## O03 — Triage history and bound expiry work

**Recommend, 66f/67b/67e.** Connect independently verified original failure
facts to the existing current-inventory resolver. Give the original verdict,
projected evidence, and new rerun their own identities. Keep publication and
retirement through the current exact issuer, content, hold, database-clock and
receipt checks. Verify original readback after executable release, failed or
changed current selection, ambiguous acknowledgement, competing hold and
retire, tied expiries, and disappearance of the old working artifacts. Keep
raw evidence custody separate from history row custody.

**Tested, disposable query only.** The original 4.0.0 memory probe used four
records with expiries 99, 100, 100 and 101. Without holds the scalar expiry
index pushed `LIMIT 2` with no separate sort for
`ORDER BY expires_unix_ms ASC, id ASC`. The second pass used the official
4.0.0 executable via
`surreal sql --endpoint memory --ns probe --db probe --json`, adding 100
older held records. With `hold = NONE` the scalar index returned the expected
two records but left a residual filter and a separate limit. A nonunique
`DEFINE INDEX held_expiry ON history FIELDS hold, expires_unix_ms` kept the
same ordered result and pushed the limit into the `[NONE]` expiry range,
without a separate sort or filter. This establishes plan shape in the
simplified fixture. Output counters did not establish exact scan work or a
speedup.

Try the compound index before a materialized eligibility flag. Qualify it on
the real bounded history query, with its reader permissions, hold/unhold
transitions, late cutoffs, and deterministic tie-breaking. Remove the scalar
index only after checking all callers. Keep cohort reads on demand first.
Compare v4 B-tree index-only counts with ordinary grouping and fixed
`COUNT WHERE` only for an actual repeated question. Measure allocated bytes,
resident state and repeated ingest/retire/reopen separately from logical expiry.
[History owner](../../xtask/src/development/developer_history.rs) ·
[Exact retirement](../../xtask/src/development/developer_history/expiry.rs) ·
[Rerun owner](../plans/66f-developer-operations-clients-and-evidence.md) ·
[Index reference](https://surrealdb.com/docs/reference/query-language/statements/define/indexes)

<a id="O04"></a>
## O04 — Consolidate database contracts on an owned v4 endpoint

**Recommend with cutover, 67c/67d.** The current standalone pin is beta.6.
Evaluate SurrealKit `1.0.0-rc.1` at
`625e264c4e134def842bfe8ac9d2f918a42bb6c3`, whose lock selects SDK 4.0.0.
Use the existing native-service fixture and stage the generated definitions.
Move schema, actor/permission and SQL assertions to the suite runner only
after equivalent named evidence and cleanup controls pass. Keep original JSON
through the history owner. Distinguish setup failure, assertion failure,
expected refusal, interrupted report and actual disposal. Exercise a foreign
endpoint, suite isolation and sync versus selected rollout. A suite exit alone
cannot settle the owned server. Measure effective compiler units and cache
reuse before claiming any Plan 67 build benefit.
The current native lane already qualified 11 v3 cases. Preserve their named
outcomes and original `--json-out` evidence. RC.1 also adds named migration
tracking, readiness/await steps, interrupted index rebuild state and
compatibility-level handling. Those features fit a later v4 schema rollout
fixture, separate from fresh bootstrap and the frozen v3 rollback point.
[Fixture](../../crates/pse-operations/tests/native_service/surrealkit.rs) ·
[RC lockfile](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/Cargo.lock) ·
[Suite contracts](https://surrealdb.com/docs/manage/schema-migration/testing)

<a id="O05"></a>
## O05 — Qualify the executor on complete catalog operations

**Recommend a receiving matrix, 66c/66e/66l.** The unified bytecode executor,
plan reuse, permission predicate integration, point lookup and index-aware
ordering are the main v4 improvements that can pay off without schema
redesign. Use the actual workbench `select_rows` with branch and optional
filters, readiness candidates, protected result-set/block lookups, and
artifact/analysis input queries. Compare sparse and dense populations,
missing keys, selective branch predicates, early and late pages, read-only
and denied actors, and fresh versus repeated statement shapes. Look for
uncorrelated work computed once, bound range and top-N execution, and
filter/bitmap fallbacks. A changed plan is useful only with equivalent rows,
ordering, permissions, response bounds, and evidence of less work.

Use existing diagnostics for static and owned executed plans. Compare
expression/partial uniqueness and array expansion only where the source
declares the same identity. Candidate examples are history filtering and
scoped catalog lookup. Preserve exact uniqueness, `NONE` handling and
relationship meaning. Keep index readiness separate from schema installation.
Test fresh and repeated index definitions, changed definitions, interrupted
builds and denied use. Favor reuse of identical definitions where v4 proves
it. Keep scientific calculations in their existing native libraries.

**Tested, disposable query only.** The v4.0.0 memory probe reproduced
`SELECT count() AS n FROM evidence_facts WHERE generation=$generation GROUP ALL`
with the existing unique `(generation, member)` prefix. It reported
`IndexCountScan` and zero record fetches for a fixture of three rows
over two generations. An empty generation returned a single count of zero.
Qualify the real activation read with empty, underfilled and extra
membership plus concurrent staging and retirement. This candidate needs
no extra count index. Scientific seals that also sum bytes or batch counts
are separate operations.

[Workbench statements](../../crates/pse-codegen/src/codegen/surreal_study_workbench.rs) ·
[Catalog controls](../../crates/pse-operations/src/canonical_capability_tests.rs) ·
[Evidence activation query](../../xtask/src/development/evidence.rs) ·
[Plans and metrics](https://surrealdb.com/docs/reference/query-language/statements/explain) ·
[Planner and bitmap behavior](https://surrealdb.com/docs/reference/query-language/clauses/explain)

<a id="O06"></a>
## O06 — Bound resource use and keep authoritative effects explicit

**Recommend as part of native qualification, 66c/66e/67e.** v4 can spill
sort, distinct, grouping and join build state. Assess node memory,
per-query nonspillable allocations, temporary-directory validity and
parallel scan scheduling together. Reuse the launcher's existing
`--temporary-directory` and `TMPDIR` beneath the owned generation.
Exercise cancellation, invalid scratch,
full storage, physical drain and cleanup. Check interference between
long analytical reads, index population and small canonical writes before
changing scan/index settings. Node memory is a cooperative budget with
wait and overrun behavior. Keep the existing host resource policy and
per-operation bounds as separate controls.
Compare `SURREAL_MEMORY_LIMIT`, `SURREAL_QUERY_MEMORY_MINIMUM` and
`SURREAL_QUERY_MEMORY_MAX` with the independent MX engine limits.
Exercise `SURREAL_MORSEL_PLACES` and `SURREAL_MORSEL_OLTP_RESERVE`
against mixed scans and short writes before selecting a setting.

Read the documented storage, query and compaction metrics through the
existing diagnostics and footprint owners. Anonymous metrics expose
process gauges, while the richer scoped families require root or OTLP
access. Keep labels and credentials out of shared reports. Correlate
spill/reopen with allocated-byte census and native settlement rather
than treating an allocator gauge as released disk space.

Compare native known-conflict replay with the protected client retry path.
The tagged standalone statement executor allows up to ten attempts
within the same transaction timeout. It excludes explicit transactions,
unknown outcomes and work after stream emission or commit.
Exercise attempts and original effects separately, including a real
serializable conflict, lost reply after an acknowledged write, and an
extension or external effect that must not be replayed. Retain the
publication witness. Narrowing it requires a named measured contention
case and coverage of empty and future membership predicates. A successful
read-only query or an existing-record lock gives no such guarantee.
[Native executor](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/core/src/dbs/executor.rs) ·
[Runtime memory and scheduling](https://surrealdb.com/docs/reference/cli/db/environment-variables) ·
[Tagged observability contract](https://github.com/surrealdb/surrealdb/blob/v4.0.0/doc/OBSERVABILITY.md) ·
[Publication witness](../../crates/pse-codegen/src/codegen/surreal_study_capture_witness.rs) ·
[Plan 66e E09](../plans/66e-protected-data-and-result-delivery.md#e09-bounded-decisions--november6)

<a id="O07"></a>
## O07 — Improve evidence discovery without duplicating evidence

**Recommend a bounded comparison, 66f/67b.** The knowledge catalog already
has an optional analyzer on `knowledge_excerpts.text` after structural
subject selection. Compare v4 full-text execution on that path first. A
separate relevance-sorted view can help an engineer find excerpts inside a
named generation when key-ordered filtering is insufficient. Compare ranked
BM25 top-K against the existing exact text filter with the same authorized
candidate population and provenance. Include scoped rank order, ties,
different token boundaries, deleted/updated terms, and no-match behavior.
Retain deterministic, exact lookup for schema, paths, failed test identities
and native source selection. Ranked retrieval may help discovery of an
original. It carries no authority to admit a rerun or interpret a result.

**Conditional.** An index on multiple fields or a maintained view becomes
valuable only if repeated history/reporting queries warrant its storage,
freshness and update costs. Compare on-demand grouping and index-only
counts first. If a maintained summary is justified, start with a direct
source table, an explicit stable group ID and a definition that covers empty
cohorts. Test group moves, deletion, ignored dependencies outside the source
tables, bulk import bypass and explicit rebuild. Retain original evidence.
Use `DROP` input only for intrinsically disposable aggregate input. Vector
exact prefilters, HNSW, DiskANN and hybrid lexical/vector/graph search need a
named semantic-discovery corpus and a relevance, access, model and retention
contract. Physical coordinates in simulations do not imply a geospatial search.
[Knowledge declaration](../../xtask/src/development/catalog/knowledge.rs) ·
[Full-text search](https://surrealdb.com/docs/learn/data-models/full-text-search/overview) ·
[Vector index semantics](https://surrealdb.com/docs/learn/data-models/vector-search/vector-indexes) ·
[View definitions](https://surrealdb.com/docs/reference/query-language/statements/define/table)

<a id="O08"></a>
## O08 — Qualify typed transport and independent interpretation

**Recommend at cutover, 66b/66e/66f.** Reuse the single bounded gRPC client.
Both releases already stream query frames. Current code reads negotiated
message limits and collects bounded frames before the typed statement result
is usable. Regenerate the borrowed preflight tables from the exact protocol
descriptor. The compared generated RPC request/response code is unchanged
from protocol 0.13.1 to 0.14.1, while the value and kind unions gain sized
variants. Verify frame/cumulative bytes, nodes, repeated fields and depth
before allocation. Cover missing or duplicate terminal frames, cancellation,
substantive error precedence, duplicate keys, unknown union variants and
commit settlement. Compare a second decoder on versioned golden fixtures.
Keep native result-statistics and accounting distinct from delivery and
operation finality.

v4 introduces more sized numbers and temporal types. The native Protobuf
encoder can widen sized numerics, degrade newer temporal values, or represent
out-of-range BigDecimal as a string. Protobuf and FlatBuffers share a domain
but have documented schema differences. CBOR v2 can preserve more native
value distinctions, with stricter decoding and different negotiation.
The native Protobuf conversion inserts object keys into a map, so check
duplicate-key refusal at the preflight boundary rather than assuming the
native decoder supplies it. Test gzip/zstd only with limits on decompressed
and cumulative input. The v4 tagged gRPC server refuses Arrow-only result
negotiation. Compare a positive `max_batch_records` only for a named
read-only metadata consumer and observe memory and time to first usable
result with original protection and terminal settlement intact.
Exercise existing decimal u64, exact IEEE bytes and projection, optional
values and structured IDs against the actual native codec. Keep finite
application Protobuf and Arrow payloads under their existing owners. Consider
CBOR v2, shaped rows or a bounded read-only incremental receiver only for a
named independent query whose memory or representation demands justify it.
A receiver must preserve producer protection until its last consumer settles.
[Client and frame consumer](../../crates/pse-operations/src/canonical_client.rs) ·
[Generated wire preflight](../../xtask/src/codegen/grpc_wire.rs) ·
[Native v4 conversion](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/types/src/proto/value.rs) ·
[Protocol parity](https://github.com/surrealdb/surrealdb-protocol/blob/v0.14.1/parity.toml) ·
[Protocol conformance fixtures](https://github.com/surrealdb/surrealdb-protocol/tree/v0.14.1/conformance) ·
[CBOR protocol](https://surrealdb.com/docs/reference/rest-api/cbor-protocol)

**Independent of v4.** Build on the already selected Buffa/Buf pipeline.
Retain original descriptor closure where an independent result reader needs
it. Compare static generated views with a bounded `DescriptorPool` /
`DynamicMessage` fallback for acquired historical schemas. Verify unknown
binary fields, oneofs, proto presence, names and reserved tags through Buf
and the native semantic comparison. ProtoJSON, `FieldMask` and
Protovalidate each carry separate policies. Do not infer historical meaning
from today's schema or database type. Editions 2026, alternate Rust/Python
generators, custom options and dynamic `Any` only earn adoption for a
concrete compatibility or consumer gap.
[Generation and compatibility](../plans/66b-contracts-generation-and-interpretation.md) ·
[Historical interpretation](../plans/66e-protected-data-and-result-delivery.md) ·
[Buf breaking categories](https://buf.build/docs/breaking/rules/) ·
[Protobuf presence](https://protobuf.dev/programming-guides/field_presence/)

<a id="O09"></a>
## O09 — Rebuild the study extension and verify API authority

**Required, 66c/66e.** Rebuild the existing pure StudyModule using the
separate `surreal-ext` builder, `DEFINE EXTENSION` and `ext::` calls. The
receipt identifies the whole archive. Definition `DIGEST` identifies the
extension manifest. Preserve both and verify the effective ABI and granted
capabilities. v4 lends transaction access only to functions that declare it.
Use the pure ABI for the current policy. Compare ABI 5 typed `BINDINGS` for
a real repeated configuration input. Defer `Transaction<Read>`,
`Transaction<ReadWrite>`, descriptor processing inside WASM, module network
or self-query access until a concrete operation benefits from transaction
locality and preserves authority, timeout and memory semantics. Check wrong
digest, altered grants, unavailable extension, denied calls, cold start,
concurrent admission and physical drain. Existing module-local state and v3
packages do not transfer to fresh instances.

Use current actor and issuer gates on the privileged control API. v4
`SECURITY INVOKER` is a useful future fit for a permission-native projection
route. Current handlers invoke privileged `PERMISSIONS NONE` functions.
Preserve deliberate definer behavior and verify the effective mode on repeat
installation, since an omitted clause on overwrite resets to definer. Test
anonymous/foreign actors, direct function denial, request limits, deliberate
error responses, and earlier writes rolled back when a handler fails.
[Study module preparation](../../crates/pse-host/src/study_module.rs) ·
[Control API](../../crates/pse-codegen/src/codegen/surreal_control.rs) ·
[Extension definition](https://surrealdb.com/docs/reference/query-language/statements/define/extension) ·
[Tagged API security tests](https://github.com/surrealdb/surrealdb/blob/v4.0.0/language-tests/tests/language/statements/define/api/security.surql)

<a id="O10"></a>
## O10 — Keep topology, backup and recovery proportionate

**Recommend a recovery rehearsal, 66e/67e.** Compare engine export/import
and physical snapshot/AOL recovery as different mechanisms. Engine data
export cannot certify the owner's archive, external payload and hold closure.
Use the existing exact offline inventory and recovery checkpoint for selected
originals. Rebuild generated indexes and derived summaries deliberately.
`OPTION IMPORT` skips ordinary derived effects, so install the expected
definitions and validate or rebuild the affected derivatives separately.
Check old generation identity, partial or modified bytes, interruption,
foreign destination, restart and independent restore.

Keep persistent MX for canonical SSI and local history. Consider the
lock-resolved SurrealKV 1.2 only if an independently scoped history workload
outgrows memory or needs storage isolation and can satisfy its rules on SI.
Compare ingestion, compaction/retention, crash/reopen, active holds, read
access and allocated bytes on equal workloads. Direct MX/KV blobs bypass
query, permissions and ownership machinery. SurrealDS, TiKV and hosted
deployments require a named deployment/failure-domain objective and an
independent durability and operations contract.
[Offline owner](../../crates/pse-operations/src/canonical_files/offline.rs) ·
[Retirement](../plans/67e-resource-retirement-and-storage-verification.md) ·
[Backup and recovery](https://surrealdb.com/docs/manage/self-hosted/backups-and-recovery) ·
[Engine lock](https://github.com/surrealdb/surrealdb/blob/v4.0.0/Cargo.lock)

<a id="O11"></a>
## O11 — Bound new graph projections and change delivery

**Conditional, 66e/66f/66k.** Keep stored occurrence edges for canonical
dependency and analysis relationships. `LIGHTWEIGHT`, `ENFORCED` and
inline adjacency already exist in the 3.3 family. Recordless relations
cannot represent parallel occurrence identity or per-edge attributes and
change observations. v4 materialized back-references add a stored destination
list, with visibility controlled by that destination. For a future tiny
authorized reverse projection compare native references and the existing
filtered relation query first. A destination list can leak source IDs and
requires a real cardinality bound. Knowledge's production triplet preserves
declaration, generator and artifact identity, so it cannot be recovered from
pairwise membership alone.

Synchronous events already serve native guarded effects. Consider live
queries as a wake-up hint for a named operator view that can rescan its
source. If a consumer needs durable continuity, compare bounded
`SHOW CHANGES` replay with a native owner cursor and explicit rescan when
retention expires. Keep the database position, message and original effect
identities separate. Test duplicate/reordered delivery, permission change,
disconnect, gap and bounded replay. Async events, broker/Kafka
`surreal-sync`, and durable subscriptions add failure/recovery owners and
need a receiving workflow. Remote file buckets and GraphQL/SQL/browser
interfaces require a named audience, a narrow read projection and their
own access/custody tests.
[Graph options](https://surrealdb.com/docs/reference/query-language/statements/define/table) ·
[Tagged materialization controls](https://github.com/surrealdb/surrealdb/tree/v4.0.0/language-tests/tests/language/statements/define/field/materialize) ·
[Changefeed replay](https://surrealdb.com/docs/learn/querying/real-time/changefeeds) ·
[Knowledge graph source](../../xtask/src/development/catalog/knowledge.rs)

<a id="O12"></a>
## O12 — Evaluate a read-only interoperability surface when a consumer needs one

**Conditional, 66f/66k/67b.** If an operator needs a SQL client for a
bounded history cohort or exported assessment, compare a short-lived
PostgreSQL-protocol VIEWER fixture against today's typed read and Arrow/
DataFusion export. v4 adds a PostgreSQL SQL dialect, where the older
listener defaulted to SurrealQL, and adds a MySQL listener. Prepared
parameters, cancellation, record-ID mapping, metadata catalogs and
expression support need actual client qualification. The tagged server
includes a window-function subset despite broader release-day prose.
Treat ORM discovery, catalog queries, `pg_dump`, SQL views, writes and
type coercion as separate contracts. An adapter earns a place only if
it materially replaces work for the named consumer.

GraphQL is a different candidate for a named UI that needs an allowlisted
read projection, filters, aggregates and paging. Verify generated types,
aliases and deprecations, disabled mutation and functions, introspection
policy, depth/complexity caps and actor permissions. Browser auth needs
a trusted explicit HTTPS origin, scoped paths, and refresh/revocation
tests. The existing local control/engineering surfaces cover current
consumers. Keep Pg/MySQL, GraphQL, browser access, and dedicated
CBOR diagnostics off the canonical listener unless separately qualified.
[Tagged SQL window tests](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/core/src/kvs/ds/sql_window_tests.rs) ·
[PostgreSQL protocol](https://surrealdb.com/docs/reference/rest-api/postgres-protocol) ·
[SQL capability matrix](https://surrealdb.com/docs/reference/rest-api/sql-compatibility) ·
[GraphQL configuration](https://surrealdb.com/docs/reference/query-language/statements/define/config) ·
[RPC protocol](https://surrealdb.com/docs/reference/rest-api/rpc-protocol)

## Recommended delivery order and remaining evidence

| Order | Work and receiving decision |
| --- | --- |
| 1. Prove the private foundation | Qualify exact source preparation, parser and generated schema, native gRPC, SSI, API rollback, and the extension ABI with its custody and resource controls. Deliver SurrealKit fixture parity separately. Capture the existing matched v3 paths and effective build baseline before removing replacements. |
| 2. Regenerate and receive | Run the scientific protected result journey through original interpretation and the developer failure→history→rerun→expiry journey. Inventory and independently verify named evidence before selection. Test durable reopen, cancellation, lost acknowledgement, conflict, physical cleanup and pre-write rollback. |
| 3. Take the proven low-burden benefits | Qualify pure read-only metadata, existing index/count plans, and the compound held-history index. Extend existing diagnostics when missing counters matter. Use the actual lexical and graph fixtures for a fair comparison. Adopt reusable named constraints only after a concrete duplicated contract passes. These opportunities can be staged independently when their prerequisites hold. |
| 4. Reopen broader changes on named demand | Maintained summaries, ranked retrieval, reverse materialization, continuity, alternate query surfaces, binary transport variants and separate storage require the triggers above. Report gains only for measured, equivalent complete workflows. |

The four new probe results are **Tested** only in isolated official
4.0.0 memory sessions. Supplier semantics are **Interface-checked** where
tagged code, tests or lockfiles support the claim. Local patched-v4
receiving, production qualification, migration and performance remain
proposed. This review is the deliverable. Plan 66 and Plan 67 retain their
current accepted scope and the selected deployment remains on 3.3.4.

## Feature coverage appendix

The tables are keyed to the numbered sections of each supplied reference.
Each row accounts for the distinct mechanisms in that section, including
overlap with the other reference. “Existing” means available in the older
family or already integrated where indicated. “Conditional” names a
consumer or trigger. “Exclude” records a mismatch with the reviewed
workflows. Context and reference-map sections close their coverage by
pointing to the substantive treatment.

### SurrealDB v4 reference

| Section | Mechanisms and disposition |
| --- | --- |
| Core §1 | Multi-model, engine, storage, reactive and integration overview. Crosswalk to [O01](#O01)–[O12](#O12). |
| Core §2 | v4 unified bytecode, caching, integrated SELECT predicates, direct key access, index mutation planning and uncorrelated work → [O05](#O05). Parallel selective scans, spill and resource limits → [O06](#O06). Automatic retry requires original effect accounting. |
| Core §3 | Existing records, table modes and structured IDs. v4 named types, optional values and field migration → [O01](#O01). Exact decimals and presence → [O08](#O08). Named ID fields and prefix locality are conditional on regenerated, non-authoritative data with a real range benefit. Issued IDs and cursors retain their meaning. |
| Core §4 | Existing edges, typed references and GQL. v4 materialized back-references and improved traversal → [O02](#O02), [O11](#O11). GQL is an alternative for a named graph pattern beyond the current six relation queries. Compare with native SurrealQL and bounded Rust extraction. |
| Core §5 | B-tree, uniqueness, expressions, exact/approximate vectors, HNSW, DiskANN, filtered KNN tiers, BM25, lexical/semantic/graph fusion and geospatial predicates → [O05](#O05), [O07](#O07). Compare current lexical plan first. ANN requires recall and quality evidence. Geographic indexes have no identified physical-geography workload. |
| Core §6 | Snapshot baseline, engine-dependent serializable isolation, explicit read-only and bounded staleness → [O02](#O02), [O06](#O06). Keep stale reads out of current protection. MVCC snapshots, `VERSION` history and domain revision are distinct. MX has no historical `VERSION` reader. |
| Core §7 | Existing synchronous and asynchronous events, computed fields, live queries and changefeeds → [O07](#O07), [O11](#O11). Use cheap pure computed fields only for a concrete observational projection. Existing same-transaction witnesses retain effects. |
| Core §8 | SurrealQL, Pg/MySQL dialects, documentation discrepancy, pgvector translation, mql.js and remote/embedded SDK → [O08](#O08), [O12](#O12). Exclude Mongo driver translation without a named external client. Embedding would create another engine owner. |
| Core §9 | Scoped access, auth and database APIs → [O09](#O09), [O12](#O12). Browser auth is conditional on an operator UI. Surrealism rebuild is required. Stable buckets justify requalifying the existing allowlisted extension loader, not moving sealed scientific files. |
| Core §10 | MX snapshot/AOL and memory, KV persistence and compaction, DS/object storage and single sequencer, separate TiKV, Earth/Cloud/OPFS → [O10](#O10). Retain native single-node authority. Multi-zone, managed and browser placement require separate deployment objectives. |
| Core §11 | Exact CLI/feature provenance, metrics and logs, schema tools, import/export and maintenance → [O04](#O04), [O06](#O06), [O10](#O10). Verify source generation, cleanup and portable hold restore through owners. |
| Core §12 | MCP and Agent Memory are separate optional discovery surfaces. A named retrieval workflow would need data scope, source provenance, retention and no grant of tool authority. SurrealML removal is a compatibility inventory item. PSE calculations stay with native scientific owners. |
| Core §13 | Experimental migration consent, format conversion and compatibility, SDK/module changes, rebuilds and licensing → cutover prerequisites and [O10](#O10). Fresh generation avoids physical conversion. Evaluate supplier terms only in the actual selected deployment. |
| Core §§14–15 | Synthesis and primary source map. Applied in the cutover prerequisites and the opportunity references. |
| Addendum §1 | Existing lightweight/enforced edges, inline adjacency/references, inline edge properties, folding and overflow. Compare v4 plans on a real traversal and preserve protected edge and occurrence semantics → [O02](#O02), [O11](#O11). |
| Addendum §2 | Existing maintained views, GROUP ALL, DROP feeds and source-only invalidation. v4 projected view ID consistency is a candidate for a justified cohort. Test initial fill/delta/import/rebuild and zero rows → [O07](#O07). |
| Addendum §3 | Existing COUNT and bitmap fusion, v4 B-tree index-only counts and improved KNN prefilters. Structured plan inspection already integrated. Expression and array indexes or conditional uniqueness only for a matching identity/predicate → [O03](#O03), [O05](#O05), [O07](#O07). |
| Addendum §4 | Index definition, deferred/concurrent build visibility and status, conflicts and v4 unchanged overwrite reuse. Confirm `INFO FOR INDEX` when depending on readiness. Identical ready definition can reuse the build. Changed/failed definitions and changed analyzer inputs need revalidation or rebuild → [O04](#O04), [O05](#O05). |
| Addendum §5 | Node and per-query budgets, spill/wait/overrun, morsel scheduling and OLTP interaction → [O06](#O06). Owned scratch exists. No quota or workload-speed claim follows from spilling. |
| Addendum §6 | CBOR v2 negotiation, sized numerics, temporal/decimal/regex tags, stricter duplicate/overflow/NONE/legacy rejection → [O08](#O08). An independent receiver needs explicit semantic and byte bounds. |
| Addendum §7 | Field-scoped FLEXIBLE, SQL TYPE coercion, dependency order, restricted effects, named migration, derived dependencies, compatibility level and undeclared-field pruning → [O01](#O01), [O04](#O04). Check generated cross-field rules, readonly tampering and imported values. No silent cleanup of held data. |
| Addendum §8 | Existing sequence and batched allocation, gaps on rollback and restart/export state. Exclude for guarded attempt/study ordinals, generation or cursor continuity. Revisit for a human label where gaps and cross-node order have no meaning. |
| Addendum §9 | Caller-permission functions, closures, elevated custom HTTP APIs, v4 invoker option and stage atomicity → [O09](#O09). Keep existing bounded native guards. A separate HTTP protobuf interface needs a concrete client. |
| Addendum §10 | SQL record identity, metadata gaps, dialect/view/function/write boundaries, pgvector translation and documented discrepancy → [O12](#O12). Verify actual driver and statement set on an isolated read-only consumer. |
| Addendum §11 | Generated GraphQL schema, inclusion/exclusion, filters/aggregates, aliases, computed/read-only and mutation behavior → [O12](#O12). Enable only for a named scoped UI. |
| Addendum §12 | SDK features, prior `stream_items`, WebSocket `query_stream`, explicit finish/SDK drop behavior, persisted HTTP session TTL and experimental live routing → [O08](#O08), [O11](#O11). New SDK API and shutdown/options are useful for independent fixtures. Native client lifetime remains a separate contract. |
| Addendum §13 | Exact float/decimal equality and literal precision, vector arithmetic, arrays versus sets, NONE/NULL unique tuples, index rebuild categories, storage-dependent format and auth changes → cutover prerequisites and [O01](#O01), [O05](#O05), [O08](#O08). Test actual Decimal u64/IEEE bits and optional fields. Nonunique NONE in the expiry probe says nothing about unique NONE. |
| Addendum §14 | Capability gates across listeners, bulk import bypass, schema comments and experimental flags → [O09](#O09), [O10](#O10), [O12](#O12). Generate comments only from existing owners. Keep current private module bucket path/allowlist, verify stable files separately from extension flags. |
| Addendum §15 | Additional sources used in the relevant opportunity and cutover sections. |

### Protobuf and SurrealDB v4 reference

| Section | Mechanisms and disposition |
| --- | --- |
| Core §1 | Boundary overview and engine terminology. Crosswalk to [O01](#O01), [O08](#O08), [O09](#O09), [O10](#O10), [O11](#O11). |
| Core §2 | Binary wire, tags, Editions/presence, descriptor reflection, well-known types/Any, ProtoJSON and validation governance → [O08](#O08). Retain exact unknown binary inputs for historical read, limit registry/Any to a real admitted set, keep JSON loss and domain validation explicit. |
| Core §3 | Application messages versus native values and gRPC/CBOR representations → [O08](#O08). Keep generated domain ownership and indexed native projections. |
| Core §4 | Existing native gRPC, capabilities/health, encoding and chunk controls, supplier descriptor tooling and application RPC distinction → [O08](#O08). Consider an independent client using the exact descriptor with explicit endpoints. Preserve native physical controls. |
| Core §5 | Native projected fields, opaque bytes and hybrid storage, data types, missing/null/default, graph/event examples → [O01](#O01), [O08](#O08), [O11](#O11). Keep selected originals with provenance and interpretation. Re-encoding does not establish original identity. |
| Core §6 | MX/KV/DS, MVCC versus domain time/schema evolution, isolation and scoped publication invariants → [O06](#O06), [O10](#O10). |
| Core §7 | Occurrence-bearing edges, lightweight and inverse adjacency, GQL and query tracing → [O02](#O02), [O11](#O11), [O12](#O12). v4 native trace options are available on the applicable HTTP/WS RPC surfaces. Tagged gRPC cannot request them and drops trace items. A separate debug reader needs its own redaction and bounds. |
| Core §8 | WASI extension execution, explicit transaction parameter, atomicity and migration limits → [O09](#O09). Keep pure first. No external effect inherits database commit. |
| Core §9 | Live/events, durable cursor/dedupe projection, Kafka Protobuf ingestion via surreal-sync → [O11](#O11). Existing mechanisms and adjacent tool require an actual feed consumer and retention/recovery objective. |
| Core §10 | Existing Buffa/Buf and validation, Prost and official Rust alternatives, Python interoperable tooling, descriptor inspection and remote registry options → [O08](#O08). Apply through native source generation or a named external boundary. Database v4 requires no generator or language change. |
| Core §§11–12 | Native-vs-Protobuf division, performance and security bounds, version/maturity and synthesis → [O01](#O01), [O08](#O08), [O09](#O09), [O10](#O10). |
| Addendum §1 | Native codec exports, component lock, widening/degradation and serialized data → [O08](#O08). Match consumed semantics and fixture bytes before adoption. |
| Addendum §2 | Database-specific Value/Kind, structured IDs, object keys and temporal distinction → [O08](#O08). Keep native value readback separate from domain protobuf. |
| Addendum §3 | SurrealValue, generated versus wrapper versus reflection mappings, `kind_of`, named contracts → [O01](#O01), [O08](#O08). Prefer existing registry-generated projections. Reflection is for an acquired schema and named reader. |
| Addendum §4 | CBOR v2, shaped rows, distinct FlatBuffers/Protobuf parity and upstream conformance vectors → [O08](#O08). Shaped CBOR tag 57 is separate from the gRPC/Arrow encodings. |
| Addendum §5 | WASI component ABI and fresh instance, explicit transaction/capabilities and WASI HTTP, ABI5 bindings, distribution and integrity → [O09](#O09). Test grant and deadline equivalence on the owned package. Build-time state cannot consume per-call bindings. |
| Addendum §6 | Custom HTTP raw bytes, size and decoding, privilege and narrow architectural fit → [O09](#O09). The current control API already bounds raw bodies. New binary routes need an independent nested-allocation guard and a concrete recipient. |
| Addendum §7 | Direct SurrealKV versioned reads, value-log placement, WAL sync modes and durability. Direct MX borrowed callback/reuse, snapshots and AOL → [O10](#O10). No cross-engine atomicity, query or graph semantics from storing opaque messages. |
| Addendum §8 | Lightweight visibility, source-table-limited materialized views, GraphQL typed surface → [O07](#O07), [O11](#O11), [O12](#O12). |
| Addendum §9 | Dynamic descriptor sets, custom options, unknown field handling, Editions 2026 and generator compatibility → [O08](#O08). Require the pinned compiler, generators and validators to establish a concrete need before moving editions. |
| Addendum §10 | FieldMask policies, descriptor validation, Buf compatibility categories, durable replay identity → [O08](#O08), [O11](#O11). Partial updates require a domain-owned assign/null/remove and merge/replace policy. Buf wire, JSON and file results and semantic compatibility remain distinct. |
| Addendum §§11–12 | Unresolved combinations and synthesis. Preserve limits around full codec fidelity, arbitrary Proto-to-graph generation, WASI/runtime compatibility, KV zero-copy, recordless events, view invalidation and portable validation. Opportunities above specify the required receiving evidence. |

The strongest additions discovered by the repository and tagged-source
cross-check are the held-prefix expiry plan, the existing activation
count's B-tree prefix, the existing knowledge traversal's plan-shape
diagnostic gap, the scoped lexical ranking candidate, the materialized
back-reference visibility trap, the API overwrite security default,
and the RC.1 schema lifecycle. None depends on adding a second
canonical database client or migrating the v3 datastore in place.
