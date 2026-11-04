# SurrealDB storage, projections and retained Proto interpretation

Inspected 2026-11-03 at source `f55951cfe995a689f9b75185b8e89cb362021616`, with the
uncommitted Proto reference additions included as research inputs. Source mechanisms below
are **Interface-checked**; local applicability and the target are **Proposed**. No database
attachment, module execution, replay campaign or performance measurement was performed.
The investigation excludes the SurrealDB v4 document and all v4 adoption assumptions.

## Proposed target and its current basis

Use one source-owned semantic model to generate the native database representation and the
portable domain contract, with an explicit mapping between them. Use approved descriptors to
inspect and exchange that contract, rather than infer database behavior from arbitrary message
shapes. Keep native records and relationships queryable. Retain original message bytes only
where a concrete archive, forwarding or independent-reference consumer needs information that
the native projection does not preserve. This is a selective hybrid design, not a universal
second copy of every database row.

The target keeps SurrealDB for the operations whose graph, transaction, permission and durable
ownership semantics already fit. That is a comparative recommendation, not a premise that
prohibits replacement. A Proto-only standalone store would also need to supply or replace those
capabilities; a codec alone does not make it the simpler complete operation. Existing
SurrealDB protocol/client integration and scientific Arrow payloads remain distinct from the
application's generated domain contracts.

Current source already goes further than an ordinary Serde-to-database adapter:

| Current owner | Observed responsibility and consequence |
| --- | --- |
| [Canonical codec](https://github.com/paul-heyse/pse-arrow/blob/f55951cfe995a689f9b75185b8e89cb362021616/crates/pse-operations/src/canonical_codec.rs) | `encode_uint` uses exact native Decimal; finite values retain authoritative IEEE bits plus a query projection; record links, text, bytes and identities have separate checked paths. A generated replacement must preserve these policies rather than use convenience casts. |
| [Borrowed wire preflight](https://github.com/paul-heyse/pse-arrow/blob/f55951cfe995a689f9b75185b8e89cb362021616/crates/pse-operations/src/canonical_client/wire.rs) | Cumulative wire, nodes, depth, strings, blobs and container occurrences are checked before owned Prost values are allocated. A per-message byte limit is not an equivalent replacement. |
| [Native query results](https://github.com/paul-heyse/pse-arrow/blob/f55951cfe995a689f9b75185b8e89cb362021616/crates/pse-operations/src/canonical_client/results.rs) | Ordered statement results become available after `QueryEnd`, complete statement accounting and trailers. Transport EOF does not establish query completion. |
| [Protected result ownership](https://github.com/paul-heyse/pse-arrow/blob/f55951cfe995a689f9b75185b8e89cb362021616/crates/pse-operations/src/canonical_results.rs) | Exact protections, release tasks and failed-release recovery are retained independently of the reader. Proto messages can report these facts; message lifetime does not supply their custody. |
| [Surrealism study module](https://github.com/paul-heyse/pse-arrow/blob/f55951cfe995a689f9b75185b8e89cb362021616/crates/pse-study-module/src/lib.rs) | Existing typed `SurrealValue` carriers and admitted readiness decisions already execute inside a module. In-database policy is an existing use, not an unimplemented first adoption. |

Blueprint §4/§6, §20 and §23 own the existing schema, publication and failure contracts.
The [persistence opportunity account](../../../codebase/persistence-and-transport.md) contains
older comparison premises and current opportunities. Its assumption that SurrealDB remains
canonical does not restrict this proposed target comparison.

## Three different Proto boundaries

The newly supplied [3.3 capability reference](../../../../libraries/proto/Protobuf_SurrealDB_v3.3.md)
distinguishes database RPC, the native Value codec, and application-defined contracts. That
distinction should become explicit in the target's API and documentation.

1. **Database-aware traffic:** retain the published `surrealdb-protocol` service and rich Value
   types where clients intentionally speak database operations. The workspace uses protocol
   0.13.1, `surrealdb-types` 3.3.4 with `proto`, and Tonic 0.14.6. Its exact declarations remain
   in Cargo.toml. A Buffa application service need not replace this externally defined stack.
2. **Native-value interchange:** `surrealdb_types::encode_proto(&Value)` and
   `decode_proto(&[u8])` serialize the database's own Value schema without a gRPC connection.
   They are useful for a native-value fixture, explicitly database-aware export or gateway.
   They do not decode an arbitrary domain message or create database records.
3. **Portable domain exchange:** generated PSE messages carry scientific identities, physical
   meaning, edits, outcomes and result descriptions. Their storage projection uses the model's
   declared policies. Internal table names and arbitrary native record-key forms should enter
   such contracts only where the domain actually requires them.

Exact installed 3.3.4 `src/proto/mod.rs` constructs a protocol Value from `value.clone()` and
calls Prost `encode_to_vec`; decoding parses a protocol Value then performs fallible native
conversion. This confirms the same public codec route described for 3.3.5 in the supplied
reference, but establishes neither zero-copy encoding nor pre-allocation admission. See the
[3.3.4 codec](https://docs.rs/crate/surrealdb-types/3.3.4/source/src/proto/mod.rs) and
[protocol Value schema](https://github.com/surrealdb/surrealdb-protocol/blob/v0.13.1/surrealdb/protocol/v1/value.proto).

## Generate projections from meaning, not from field spelling

The proposed generator consumes the existing source-owned contract graph and emits both
approved domain descriptors and native field/codecs. A descriptor-aware reader can then
inspect quantities and relationships without becoming another authority over them. Schema
metadata must express the meaning that cannot be inferred: key/identity source, relationship
direction and cardinality, numeric and temporal policy, projection inclusion, presence/update
policy and mapping interpretation revision.

`SurrealValue::{kind_of, into_value, from_value}`, the derive macro, and native `Kind` provide
useful conversion and structural-type primitives. They do not derive indexes, permissions,
transactional predicates, retention, source-parent rules or schema migration. Use generated
conversions where all field policies are declared and explicit adapters where the mapping is
domain-specific. A dynamic mapper becomes worthwhile for many externally supplied approved
schemas sharing one policy; it is not the default replacement for compiled, locally owned
scientific records. See [3.3.4 SurrealValue](https://docs.rs/crate/surrealdb-types/3.3.4/source/src/traits/surreal_value.rs)
and [Kind](https://docs.rs/crate/surrealdb-types/3.3.4/source/src/kind/mod.rs).

The important mapping decisions are:

| Distinction | Proposed handling |
| --- | --- |
| Full `u64` versus native `Int(i64)` | Preserve the current exact Decimal path, or an explicitly versioned exact representation. The 3.3 convenience unsigned conversion casts to signed integer and is unsuitable for full-range scientific identity/count data. |
| Finite scientific value versus diagnostic bits | Retain explicit domain admission and the original bits when required. JSON numbers and a query float alone cannot replace signed-zero or raw-diagnostic semantics. |
| Absent, default, native NONE and NULL | Preserve each declared meaning. A missing message field is not an instruction to remove a stored field. |
| Enum identity versus label | Retain numeric identity where unknown future values are forwardable; admit known domain meanings separately. A closed database string assertion needs its own evolution disposition. |
| Domain ID versus database record key | Keep typed role identity and a checked projection. Native keys include forms beyond strings; colon-concatenated text is not a universal typed ID. |
| Signed application duration versus native unsigned duration | Preserve signed meaning explicitly or restrict the supported intersection; do not reinterpret a negative duration. |
| Nested/repeated messages versus records/edges | Declare ownership, ordering, duplicates and independent occurrence identity; do not turn every list of IDs into identical graph edges. |

Prost generated messages discard unknown message fields, while dynamic `prost-reflect` can
preserve them through binary handling. Buffa's examined generator preserves unknown fields by
default. Any subsequent native projection or ProtoJSON conversion can still lose them. For an
exact archive, retaining original bytes is the reliable preservation target; semantic admission
and forwarding remain different operations. See [Prost merge generation](https://github.com/tokio-rs/prost/blob/v0.14.4/prost-derive/src/lib.rs),
[dynamic unknown fields](https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.DynamicMessage.html#method.unknown_fields)
and [Buffa configuration](https://github.com/anthropics/buffa/blob/v0.9.2/buffa-codegen/src/lib.rs).

## Retained payloads, graph state and recovery

For a retained external case or event, the proposed envelope associates immutable original
bytes with its fully qualified message name, approved dependency-complete descriptor bundle,
mapping interpretation and scientific provenance. Descriptor identity answers which schema
bytes were selected; it does not replace the semantic identity of a model, case or result.
Retaining a second native projection is justified by a real query/index/traversal consumer and
must have a reconstructible relationship to its authoritative input.

Coherent publication uses the existing native transaction owner. A descriptor-driven mapper
cannot coordinate concurrent facts merely by validating one decoded message. Database
assertions and enforced relations protect stored shape; transaction logic protects multi-record
invariants. Retry the complete guarded decision after a definite conflict. After an uncertain
acknowledgement, settle the original operation identity before deciding whether another effect
is permitted. Sharing a `.proto` does not make application and database transactions identical.

Graph choices remain domain choices. Lightweight relations represent an idempotent endpoint
pair and lack the ordinary field/event/changefeed surface. Use them for propertyless topology;
use ordinary relation records for independent occurrences, attributes or per-edge change
visibility. Native inline edge/reference caches already accelerate applicable traversal; a
Proto adjacency cache adds no value merely by being typed. Maintained views refresh from
their declared source-table writes, not arbitrary related-record dependencies. A correctly
typed response can therefore be stale. Prefer a current selected query when complete refresh
dependencies are not available. See the tagged [lightweight implementation](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/core/src/kvs/lightweight.rs)
and [view semantics](https://surrealdb.com/docs/reference/query-language/statements/define/table#pre-computed-table-views).

Live notification, retained changefeed and an application outbox solve different problems.
Use a live subscription for current observation, with reconnect explicitly losing continuity
unless replay is supplied. If a future local consumer requires durable incremental delivery,
give that operation a stable event identity and source cursor/ordinal, persist acknowledgement,
and define snapshot/resynchronization on retention expiry. The 3.3.5 reader starts inclusively,
scans before table filtering and can return an empty filtered batch before later relevant
mutations. Empty does not prove caught-up. A cursor may cover several mutations; it cannot
alone identify each event. These are release-specific upstream observations, not an executed
local replay implementation. See [changefeed reader](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/core/src/cf/reader.rs)
and [changefeeds](https://surrealdb.com/docs/learn/querying/real-time/changefeeds).

## Other newly documented routes and comparative disposition

| Capability | Applicability and target disposition |
| --- | --- |
| Surrealism ABI 2 codecs and attached descriptors | **Fits with conditions** for database-local generic decode/validation only when an operation benefits from avoiding external transfer. Reuse existing module preparation/capability owners. A generic Proto decoder module is additional work, not a provided first-party mapper. Do not move solver/native libraries into WASI merely for contract reuse. |
| Module descriptor caching | **Fits with conditions** for immutable reconstructible per-controller metadata. Module KV is volatile; it must not own acknowledgement, authoritative schema history or deduplication. Controller reuse is not a transaction boundary. |
| Custom HTTP APIs carrying bytes | **Unsettled** as a replacement for the existing bounded canonical client. Raw body access still needs a decoder, aggregate admission, request identity, transaction semantics and settlement. The repository already patches API rollback, so the unpatched guide's failure behavior cannot be asserted unchanged locally. |
| Standalone SurrealKV/SurrealMX | **Ruled out here as a general store replacement** on current evidence: raw byte/history primitives do not discharge the existing graph, query, permission and recovery responsibilities. Remain eligible for a specific independently valuable byte-store responsibility; a second store adds a second atomicity boundary. Never write into SurrealDB's internal keyspace. |
| GraphQL and ISO GQL | **Fits with conditions** for database-aware selected queries/traversal. They are alternatives to fixed domain reads, not Proto codecs or automatic scientific admission. A public client must not obtain arbitrary store authority merely because it can select fields. |
| Kafka/`surreal-sync` | **Ruled out here as a prerequisite** for the local target. The documented 0.6.1 decoder rejects unknown field numbers and has framing/tombstone limits; this is not a transparent Proto archive/importer. Revisit for a concrete broker consumer. |
| Alternative FlatBuffers/CBOR routes | **Retain as protocol alternatives**, not parallel first-party paths without a consumer. Protocol schema parity does not establish binary interchange or equivalent limits. |

The library primitives and version boundaries are documented in the supplied 3.3 guide
§§6–10/A3–A11. Sources include [Surrealism capabilities](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealism/runtime/src/capabilities.rs),
[volatile KV](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealism/runtime/src/kv.rs),
[SurrealKV 0.21.4](https://docs.rs/surrealkv/0.21.4/surrealkv/),
[SurrealMX 0.27.0](https://docs.rs/surrealmx/0.27.0/surrealmx/) and the pinned
[Kafka decoder](https://github.com/surrealdb/surreal-sync/blob/30c66b1ba90b2b27706a810ebc7a3c2fc1580827/crates/kafka/src/from_kafka/proto/decoder.rs).
Local source inspection included [API rollback patch](https://github.com/paul-heyse/pse-arrow/blob/f55951cfe995a689f9b75185b8e89cb362021616/scripts/surrealdb-source/api-failure-rollback.patch)
and module deadline/controller/build patches; this is not a fresh qualification of the patched server.

## Revealing changes, deletions and remaining evidence

Adding a field with a new physical interpretation should update its source owner and generated
projections, then the consumers that actually use its meaning. It should not require a separate
handwritten descriptor-to-database interpretation. Adding a preserved but unqueried field to an
external event should not demand a new native column merely to retain it. Adding a queryable
projection requires a mapping revision and a deliberate rebuild/backfill, not mutation of the
original archived input.

When implemented, delete replaced per-consumer mechanical mappings and duplicate message
definitions after the generated route serves their complete consumers. Retain the current
numeric/domain checks, borrowed preflight, transaction and protection owners until their full
replacement is demonstrated. This design does not claim those mechanisms disappear simply
because more declarations become generated.

The decisive controls for implementation are full-range unsigned values; negative zero/raw
diagnostic bits; absent/null/remove; unknown-field forwarding versus domain rejection; graph
pair versus occurrence identity; old descriptor/mapping interpretation; stale projection repair;
and interruption after commit before reply. A mapper or storage migration is not qualified by
wire compatibility alone. Buf `WIRE`/`WIRE_JSON` checks must be combined with mapping and
semantic review. No performance benefit or assembled database integration is claimed here.

Research used Context7 first; its SurrealDB result was unversioned and insufficient for 3.3
codec semantics. Installed 3.3.4 source and tagged 3.3.5 primary sources supplied the distinctions
above. The newer guide is evidence for eligible capability, not an implicit dependency upgrade.
