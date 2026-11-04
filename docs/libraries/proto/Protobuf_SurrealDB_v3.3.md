# Protocol Buffers and the SurrealDB 3.3.x ecosystem

Technical capability reference · November 3, 2026 · Detailed baseline: SurrealDB 3.3.5 · Scope: Protobuf, native protocols, Rust and Python integrations, storage, graph querying, extensions, and change processing

## 1. Executive orientation

**SurrealDB 3.3.x already has native Protobuf integration.** It exposes a database gRPC service and a public Rust codec for its own Protobuf `Value` schema. Application-defined Protobuf messages can also be projected into native records and relations, retained as binary payloads, or processed by application services and experimental Surrealism modules. These are different integration boundaries. A `.proto` definition describes interchange data; SurrealDB still owns record identity, schema checks, indexes, graph traversal, permissions, and transaction execution.

The detailed implementation baseline here is **3.3.5**, published October 29, 2026, with 3.3.0 used to establish the initial availability of important features. The GitHub release is newer than the website's 3.3 release summary. Both the public native Protobuf codec and the server gRPC route are present in 3.3.0; the examined files are unchanged in 3.3.5. This is a source-grounded capability guide, not a deployment qualification or benchmark. Code and command examples are illustrative and were not executed. ([3.3.5 release](https://github.com/surrealdb/surrealdb/releases/tag/v3.3.5), [3.3.0 codec](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/types/src/proto/value.rs), [3.3.0 gRPC route](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/server/src/ntw/grpc.rs))

### 1.1. Capability and integration map

| Area | Available 3.3.x mechanism | Protobuf's contribution |
| --- | --- | --- |
| Database RPC | Native gRPC service, alongside HTTP and WebSocket RPC | Published database service and value messages |
| Native Rust values | Feature-gated `encode_proto` and `decode_proto` | Encode SurrealDB's own rich value representation |
| Application contracts | Explicit adapters into records, fields, and relationships | Independently governed messages, commands, and events |
| Binary retention | Native `bytes` fields or a separate standalone KV store | Retain original encoded payloads and schema identity |
| Graph querying | Links, tracked references, relation records, lightweight relations, SurrealQL and ISO GQL | Typed node, edge, path, and projection results |
| Graph optimization | Inline adjacency/reference caches and inline edge fields | Usually no additional serialization machinery needed |
| In-database transformation | Experimental Surrealism modules using ABI 2 | Potential generated or descriptor-driven codecs |
| HTTP integration | Custom APIs with raw-body middleware | Possible binary request/response contract with an explicit decoder |
| Change processing | Live queries, retained changefeeds, events, maintained views | Versioned external envelopes and replay metadata |
| Kafka ingestion | Independently versioned `surreal-sync` tool | Existing Protobuf-to-record ingestion path with decoder limits |
| Schema governance | Protobuf descriptors, generators, Buf and validation runtimes | Message evolution and consistency checks outside database DDL |

The table combines implemented primitives with compositions explained in later sections. In particular, a generic descriptor-to-SurrealDB schema mapper, a ready-made Protobuf Surrealism module, and an exactly-once broker publisher are not implied by the underlying primitives.

### 1.2. The important differences from the 4.0 guide

Several graph and transport features described in the supplied 4.0 reference already belong to 3.3. Others require a different implementation or are unavailable in this version line.

| Topic | Correct 3.3.x treatment |
| --- | --- |
| gRPC and native Protobuf `Value` | Already available; use protocol 0.13.1 and the 3.3 codec |
| Lightweight relations and inline graph caches | Already present in 3.3.0 |
| ISO GQL | Introduced in 3.2; enabled by default in 3.3; a supported subset |
| FlatBuffers | Already an available native binary representation |
| CBOR v2 and shaped results | 4.0 features; absent from the 3.3 RPC format set |
| Named `DEFINE TYPE` contracts | 4.0 feature; use 3.3 field kinds, literals, assertions, and adapter types |
| Reverse-reference `MATERIALIZE INTO` | 4.0 feature; use inverse traversal and applicable 3.3 caches |
| Graph RPC tracing | 4.0 feature; do not promise this diagnostic surface in 3.3 |
| Surrealism | `MODULE` / `mod::`, ABI 2, reused pooled instances, implicit host callbacks |
| Serializable transaction syntax | The documented `BEGIN ... SERIALIZABLE` form is 4.0, not a 3.3 option |
| Custom API execution | No `SECURITY INVOKER` selector; important pre-4.0 failure/rollback limitations |
| Storage dependencies | SurrealKV 0.21.4 and SurrealMX 0.27.0, rather than the later 1.x crates |
| SurrealDS | Already an Enterprise option in the 3.3 line; not a universal community backend |

These boundaries follow the tagged format, type, runtime, and storage sources and the official documentation's explicit version markers. ([3.3 RPC formats](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/core/src/rpc/format/mod.rs), [3.3 type kinds](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/types/src/kind/mod.rs), [3.3 release notes](https://surrealdb.com/releases/3.3), [3.x to 4.x migration](https://surrealdb.com/docs/build/migrating/from-old-surrealdb-versions/3x-to-4x))

### 1.3. Component versions are independent

The 3.3.5 lockfile resolves `surrealdb-protocol` 0.13.1, SurrealKV 0.21.4, SurrealMX 0.27.0, and Surrealism 0.6.5. The relevant native Protobuf runtime resolves to Prost 0.14.3, although the workspace declares a compatible 0.14.1 range and other Prost versions occur elsewhere in the dependency graph. Surrealism was 0.6.0 at the 3.3.0 baseline; its ABI remains 2 at 3.3.5. These are upstream build facts, separate from independently selected application dependencies. ([3.3.5 manifest](https://github.com/surrealdb/surrealdb/blob/v3.3.5/Cargo.toml), [3.3.5 lockfile](https://github.com/surrealdb/surrealdb/blob/v3.3.5/Cargo.lock), [3.3.0 lockfile](https://github.com/surrealdb/surrealdb/blob/v3.3.0/Cargo.lock))

The engine name is **SurrealMX**, not SurrealMV. MVCC is a concurrency-control concept, not another engine or a Protobuf versioning mechanism.

## 2. Protocol Buffers capabilities and architectural mechanisms

### 2.1. Schemas, generated APIs, and wire data

Protobuf combines a schema language, a binary encoding, and language-specific compilers and runtimes. A `.proto` file defines numbered fields, messages, enums, services, and options. The encoded message carries field numbers and wire types rather than field names or the schema itself. A receiving system must already know which type and descriptor interpret those bytes. A stored blob therefore needs a trustworthy type/schema association if it must remain intelligible independently of one application binary. ([Wire encoding](https://protobuf.dev/programming-guides/encoding/))

Generated code provides convenient language-native access to a known message shape. It does not make the message a database record or decide its table, key, relationship direction, index policy, or authorization. Those choices belong in an explicit mapping. Treating them as implicit conventions is especially risky when a message contains several identifiers, nested repeated entities, or a field whose textual appearance resembles a record ID.

A useful design starts with the consumer's contract. A database-native client can use SurrealDB's published `Value` messages. A business service can expose a smaller application schema. A raw archive can preserve an uninterpreted payload. One boundary need not serve all three purposes.

### 2.2. Presence and schema evolution

Presence determines whether a decoder can distinguish an omitted field from a field explicitly assigned its default. Ordinary proto3 scalars generally have implicit presence; `optional` scalars, message fields, and `oneof` alternatives have explicit presence. Repeated and map fields do not independently record whether an empty collection was transmitted. Editions use configurable features, with explicit scalar presence as the normal default. ([Field presence](https://protobuf.dev/programming-guides/field_presence/), [Edition features](https://protobuf.dev/editions/features/))

| Declaration or mechanism | Information available to an adapter |
| --- | --- |
| Ordinary proto3 `string` | Value, but normally no unset-versus-empty distinction |
| Proto3 `optional string` | Absent versus present, including present empty string |
| Message field | Presence of the nested message |
| `oneof` | Which alternative is selected |
| `repeated` or `map` | Contents, but no separate omitted-versus-empty presence |
| Explicit mutation operation | Whatever assign/remove/merge semantics the application defines |

Stable field numbers support message evolution, but compatibility has several dimensions. An added field can be binary-compatible while changing a database projection. A field rename can preserve binary identity while breaking JSON or generated clients. Removed names and numbers should be reserved. Changes involving enums, cardinality, `oneof` membership, or business meaning deserve separate review rather than a blanket assertion that the wire format is compatible.

Edition 2026 is present in the released Protobuf v36.0 compiler family, published August 20, 2026. Its availability does not establish support in every Rust generator, validator, or WASI build. The edition year, `protoc` version, generated-code version, and runtime package version are independent coordinates. The released v36.0 source is more specific than the overview page's older latest-edition sentence. ([v36.0 release](https://github.com/protocolbuffers/protobuf/releases/tag/v36.0), [Edition implementation](https://protobuf.dev/editions/implementation/))

### 2.3. Descriptors and dynamic messages

A descriptor bundle describes files, messages, fields, enums, services, dependencies, and options. A descriptor pool resolves names and relationships; a dynamic message runtime uses a selected descriptor to decode and manipulate messages without generating a concrete Rust struct for every type. `prost-reflect` supplies these facilities through `DescriptorPool` and `DynamicMessage`. ([prost-reflect 0.16.5](https://docs.rs/prost-reflect/0.16.5/prost_reflect/), [DescriptorPool](https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.DescriptorPool.html))

For a heterogeneous database, reflection can centralize conversion mechanics while keeping message-specific mapping metadata alongside the schema. This reduces repeated adapter code; it does not eliminate mapping decisions. A string descriptor does not establish that the field is a record reference, that another record exists, or that a caller can link to it. A registry must distinguish a schema's structural identity from the authority to interpret it as database operations.

### 2.4. Well-known types and controlled polymorphism

| Protobuf mechanism | Useful role | Mapping obligation |
| --- | --- | --- |
| `Timestamp` | Instant represented by seconds and nanoseconds | Check range, normalization, and native datetime conversion |
| `Duration` | Signed elapsed duration | Reconcile with 3.3's unsigned native duration |
| `Any` | Typed binary message envelope | Resolve approved types and dependencies |
| `Struct` / `Value` / `ListValue` | JSON-shaped dynamic data | Double-valued numbers are insufficient for arbitrary exact integers and decimals |
| `FieldMask` | Selected field paths | Define update, merge, reset, and collection semantics |
| `oneof` | A chosen variant | Map explicitly into native unions or operations |

These mechanisms relax or organize application typing. They do not create native graph semantics or replace schema governance. In particular, the well-known temporal types are not identical to the database protocol's custom temporal messages. ([Well-known types](https://protobuf.dev/reference/protobuf/google.protobuf/))

### 2.5. ProtoJSON and binary preservation

ProtoJSON defines its own field names and value mappings: 64-bit integers are normally printed as strings, bytes use base64, and enums and temporal messages follow prescribed forms. JSON `null` generally leaves a field unset except for special sentinel types. Generic Serde output is not automatically ProtoJSON. A JSON round-trip also cannot retain arbitrary unknown binary fields. ([ProtoJSON mapping](https://protobuf.dev/programming-guides/json/))

Retaining the original bytes, decoding and re-encoding a message, and reconstructing a message from projected fields are three different preservation policies. The first preserves exact input if storage leaves it untouched. The others depend on runtime behavior and the information retained by the transformation.

Protobuf does not define a globally canonical binary serialization. Deterministic output from one implementation is insufficient as a portable semantic identity rule. For signatures or content-addressed archives, either hash immutable original bytes or define a separate canonical semantic representation and its treatment of unknown fields, ordering, defaults, and schema changes. ([Serialization is not canonical](https://protobuf.dev/programming-guides/serialization-not-canonical/))

### 2.6. Validation and database integrity

Parsing establishes that a message is structurally acceptable to the decoder. A validation runtime can additionally enforce schema annotations, numeric bounds, collection limits, or CEL expressions. Database fields, assertions, indexes, permissions, and transactions address different conditions. A valid edge message does not establish that its endpoints exist when a write commits.

Protovalidate annotations require a compatible implementation. The official documentation establishes implementations including Python, Go, Java, JavaScript/TypeScript, and C++; it does not establish a general first-party Rust/WASI validator that can simply be inserted into a Surrealism module. Rust decoding support alone is therefore insufficient to claim execution of those annotations. ([Standard validation rules](https://protovalidate.com/schemas/standard-rules/), [Custom CEL rules](https://protovalidate.com/schemas/custom-rules/))

## 3. Where Protobuf fits in the 3.3 database architecture

### 3.1. Native values remain the queryable representation

The public value system includes records, structured identifiers, nested objects, arrays, sets, exact decimals, bytes, temporal values, ranges, files, and geometry. Protobuf can carry a representation of these values or an independent application contract. The database queries its native representation; it does not discover arbitrary domain `.proto` fields inside an opaque `bytes` payload. ([3.3 native value conversion](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/types/src/proto/value.rs))

An application can decode a payload into indexed native fields, retain it unchanged for later retrieval, or combine both approaches. Producing a Protobuf response from a query is a fourth operation: its output schema need not resemble the stored record. A query projection is often a better service contract than an exact copy of the storage model because it can remain stable as internal tables and relations evolve.

### 3.2. Protocol and query surfaces

| Surface | 3.3.x role | Relationship to Protobuf |
| --- | --- | --- |
| HTTP and WebSocket RPC | Database requests, sessions, and supported notifications | Native JSON, tagged CBOR, and FlatBuffers representations |
| Native gRPC | Database RPC over HTTP/2 | Upstream Protobuf service and rich database values |
| SurrealQL | Native record and multimodel operations | Independent of message encoding |
| ISO GQL | Graph patterns, paths, and graph mutations | Results can be mapped to application messages |
| PostgreSQL dialect surface | Supported SQL-oriented interoperability | Separate SQL/type/protocol boundary |
| GraphQL | Typed, introspectable field selection | Separate generated query schema; not binary Protobuf |
| Custom APIs | User-defined HTTP handlers | Can receive raw bytes; require explicit Protobuf logic |

The 3.3 RPC format implementation enumerates JSON, CBOR, and FlatBuffers. A native Protobuf codec and a gRPC service do not imply a `protobuf` WebSocket subprotocol. Likewise, CBOR v2, shaped-result negotiation, and MySQL-wire functionality described for 4.0 should not be projected back onto this format set. ([Tagged RPC formats](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/core/src/rpc/format/mod.rs), [GQL overview](https://surrealdb.com/docs/learn/querying/gql/overview))

## 4. Direct gRPC integration

### 4.1. The existing database service

The service is `surrealdb.protocol.rpc.v1.SurrealDBService`. Its 0.13.1 definition covers capability discovery, health, session lifecycle, authentication, variables, explicit transactions, queries, function execution, subscriptions, and import/export operations. CRUD helpers are implemented through query operations rather than separate Create/Select/Update/Delete RPCs. A method appearing in the schema is not sufficient evidence that a particular server advertises or implements it. ([RPC schema 0.13.1](https://github.com/surrealdb/surrealdb-protocol/blob/v0.13.1/surrealdb/protocol/rpc/v1/rpc.proto))

The Rust SDK's `protocol-grpc` feature is opt-in. It accepts `grpc://` and `grpcs://` endpoints. The server shares its HTTP listener, port, TLS setup, middleware, and RPC capability gate with the other routes; HTTP/2 is selected through cleartext negotiation or TLS ALPN. ([SDK feature manifest](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/Cargo.toml), [SDK connection documentation](https://surrealdb.com/docs/reference/rust/methods/connect), [Tagged gRPC route](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/server/src/ntw/grpc.rs))

An illustrative pinned dependency declaration is:

```toml
[dependencies]
surrealdb = { version = "=3.3.5", features = ["protocol-grpc"] }
```

This enables an existing transport; it does not establish credentials, server grants, session ownership, or a fully configured application.

### 4.2. Capabilities, limits, and streaming

`GetCapabilities` reports protocol support, method restrictions, limits, compression, and live-query guarantees. A client should negotiate rather than infer every detail from a version string. Request context is carried in Protobuf request bodies; streaming imports introduce a Begin envelope, so a generic client must follow the actual message nesting rather than assume every first field is a direct context message. ([Common RPC envelopes](https://github.com/surrealdb/surrealdb-protocol/blob/v0.13.1/surrealdb/protocol/rpc/v1/common.proto))

| Mechanism | 3.3.5 behavior | Practical consequence |
| --- | --- | --- |
| Message size | Default 4 MiB, through `SURREAL_GRPC_MAX_MESSAGE_SIZE` | Both incoming and outgoing individual messages are bounded |
| Query batches | Record-count and encoded-byte limits | Many rows can stream; one oversized value still cannot fit |
| Compression | gzip and zstd | Transport compression is separate from result encoding |
| Attached sessions | Default process-wide maximum 16,384 | Session cleanup and resource budgeting matter |
| Notification buffering | Default 1,024 per subscription | Exhaustion terminates the stream rather than retaining indefinitely |
| Reflection | Not exposed by the server | Inspection tools need matching local protocol definitions |
| Health | Service-specific RPC | Standard `grpc.health.v1.Health` is a different service |

The size and buffer defaults come from the tagged server configuration. Client configuration can impose another ceiling; increasing its receive allowance alone does not increase the server's send or receive ceiling. ([Server configuration](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/server/src/cnf/mod.rs), [SDK gRPC configuration](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/src/opt/grpc.rs), [RPC documentation](https://surrealdb.com/docs/reference/rest-api/rpc-protocol))

The server streams query results with bounded frames. An Arrow result encoding exists in the protocol vocabulary but is not served by the examined implementation; requesting only that encoding fails rather than yielding columnar results. Live subscriptions advertise at-most-once, nonresumable delivery, one subscriber per live query, and no retention. These implementation guarantees are narrower than everything a future schema could express. ([3.3.5 gRPC implementation](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/server/src/rpc/grpc.rs))

Incremental rows also have a completion boundary. SDK `.stream_items()` exposes rows and statement completion separately; rows can precede a later statement error, and an enclosing transaction can still roll back. A consumer must not publish a provisional row as a committed domain event merely because it arrived successfully. Awaiting a normal query and processing an item stream have different buffering and completion responsibilities. ([Query streaming implementation](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/src/method/query.rs))

### 4.3. Sessions and domain services

Attached sessions survive HTTP/2 reconnects through process-owned state. Requests without them use ephemeral state; explicit transactions require attachment. Session identifiers are capabilities that must be kept secret, and detach tears down owned transactions and subscriptions. Generating a client from `.proto` files does not implement these lifecycle semantics automatically. ([Session implementation](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/server/src/rpc/grpc.rs))

For a database client, the native service avoids inventing another database transport. For a domain API, an application-owned service can expose messages such as `EquipmentSnapshot` or `RouteResult` while using SurrealDB internally. That isolates clients from the database value schema and query language, at the cost of maintaining an adapter and authorization boundary. Generating domain messages does not replace the server's native protocol runtime or make those messages interchangeable with its `Value` union.

## 5. Representing application messages in records

### 5.1. Native, opaque, and hybrid storage

| Strategy | What is retained | What can be queried natively | Main obligation |
| --- | --- | --- | --- |
| Native projection | Decoded fields, IDs, and relationships | Projected fields, indexes, graph topology | Define a faithful mapping |
| Opaque payload | Original bytes and schema/type identity | Envelope and separately stored metadata | Preserve bytes and schema availability |
| Hybrid | Original bytes plus selected native projections | Projected fields and relations | Keep payload and projection coherent |

A hybrid model is often useful when long-lived interchange contracts coexist with selective database queries. The payload is the archival representation; native fields are a query projection. If both are mutable, designate which is authoritative and update them coherently. Otherwise a repair process cannot know whether to regenerate the bytes or the fields.

Projection can also be deliberately one-way. An event may contain fields that are irrelevant to current queries, or a native graph may contain derived relationships that never appeared literally in the message. A versioned mapping should record those choices rather than promise a reversible field copy.

### 5.2. Type mapping in 3.3

| Application value | Suitable mapping choices | Particular 3.3 concern |
| --- | --- | --- |
| Signed integer | Native integer after range validation | Native integer representation is signed 64-bit |
| `uint64` | Checked signed integer, explicit decimal, or decimal string | Do not use unchecked `u64.into_value()` |
| Float/double | Native float | Define rounding and nonfinite-value policy |
| Bytes | Native `bytes` | Keep binary distinct from base64 text |
| Enum | Integer identity, constrained string, or tagged object | Preserve unknown numeric values deliberately |
| Nested message | Object or separate record | Decide ownership, identity, and lifecycle |
| Repeated message | Array or related records | Define ordering and duplicate semantics |
| Map | Object or entry array/records | Object keys and Protobuf key types differ |
| Timestamp | Native datetime after checked conversion | Protobuf and native representable ranges differ |
| Signed duration | Explicit signed representation or restricted native duration | Native 3.3 duration is unsigned |
| Exact decimal | Decimal/scaled representation with a documented range | Protobuf has no universal decimal scalar |
| Record reference | Typed native record ID or a restricted domain ID | Arbitrary native keys are not all strings |

The most serious convenience-conversion hazard is **3.3.5's Rust `SurrealValue` implementation for `u64` and `usize`**: it uses an `as i64` cast on the write path. Values above `i64::MAX` wrap; `u64::MAX` becomes `-1`. The corresponding read conversion uses checked conversion, so the path is asymmetric. Use an explicit adapter with range checks or an intentional decimal/string representation. Newer rolling documentation describing decimal fallback must not be used as evidence for this 3.3 implementation. ([Tagged numeric conversion](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/types/src/traits/surreal_value.rs#L1212))

### 5.3. Absence, null, defaults, and mutation intent

SurrealDB distinguishes `NONE`, `NULL`, and concrete values. Assigning `NONE` to a record field removes it; assigning `NULL` retains explicit null. A concrete zero, false, empty string, or empty collection is another state. Message field presence covers only part of this state space. ([NONE and NULL](https://surrealdb.com/docs/reference/query-language/language-primitives/data-types/none-and-null))

For partial updates, define at least leave unchanged, assign a value, assign null, and remove. Collection replacement and append, or nested-message replacement and merge, require further choices. Missing input should never silently become a removal operation unless that is the published contract. The advanced addendum gives an explicit operation example and explains why `FieldMask` needs more policy than a list of paths.

In 3.3, optional or nullable field kinds and assertions can express stored-state constraints. They do not automatically interpret Protobuf presence. Named `DEFINE TYPE` and `MIGRATE EXISTING ROWS` examples from 4.0 should not be substituted for a 3.3 schema/backfill procedure.

### 5.4. An illustrative graph event envelope

The following original example is an application contract, not the native SurrealDB protocol. It deliberately restricts record keys to strings and signed integers; an application needing all native key kinds should use the upstream typed ID representation or a richer equivalent.

```proto
syntax = "proto3";
package example.topology.v1;

message RecordRef {
  string table = 1;
  oneof key {
    string text_key = 2;
    int64 integer_key = 3;
  }
}

message ConnectionChange {
  RecordRef source = 1;
  RecordRef target = 2;
  string relation_table = 3;
  bool removed = 4;
  optional RecordRef occurrence = 5;
}

message StoredMessage {
  string full_type_name = 1;
  bytes descriptor_digest = 2;
  string mapping_revision = 3;
  bytes original_payload = 4;
}

message GraphEvent {
  string event_id = 1;
  bytes source_cursor = 2;
  uint32 mutation_ordinal = 3;
  oneof change {
    ConnectionChange connection = 10;
    StoredMessage entity_message = 11;
    RecordRef deleted_record = 12;
  }
}
```

The optional occurrence distinguishes an independently identified relation record from a propertyless connection. The cursor identifies source progress, while `event_id` identifies an external event. Neither creates a native graph edge. The receiving adapter still selects and validates the corresponding database operation.

## 6. Storage engines and multiversion state

### 6.1. The actual 3.3 storage choices

SurrealDB 3.3 retains build surfaces for RocksDB, SurrealKV, SurrealMX-backed memory, TiKV, and IndexedDB. RocksDB remained the recommended disk engine before 4.0. SurrealDS was already a separate Enterprise distributed option, including fixes in the 3.3 release line. The existence of an engine feature does not mean every binary includes it or every deployment has the same guarantees. ([Deployment models](https://surrealdb.com/docs/manage/self-hosted/deployment-models), [3.3.5 build features](https://github.com/surrealdb/surrealdb/blob/v3.3.5/Cargo.toml))

| Boundary | Relevant 3.3 fact | Protobuf implication |
| --- | --- | --- |
| Integrated SurrealKV | Version 0.21.4 with wrapper-selected settings | Persistence and optional history for database state |
| Integrated memory engine | SurrealMX 0.27.0; snapshot isolation selected | MVCC does not imply retained historical queries |
| Memory persistence | Optional path/configuration is wired through the wrapper | Ordinary pathless memory remains volatile |
| Enterprise SurrealDS | Separate availability and operational contract | Do not infer community-tag internals or 4.0 syntax |
| Standalone KV crates | Independent Rust APIs | Direct byte storage without record/graph semantics |

SurrealDB's memory backend rejects versioned historical-query configuration and version arguments in reads/scans. Its transaction constructor explicitly selects snapshot isolation. Standalone SurrealMX's stronger transaction options do not automatically become integrated 3.3 query options. ([Memory wrapper](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/kvs-mem/src/lib.rs))

### 6.2. Three independent kinds of version

A transaction snapshot determines which committed database state an operation observes. Historical retention determines whether an earlier state remains explicitly retrievable. A Protobuf schema revision determines how a payload is interpreted. These can move independently.

For example, a retained database version may contain an original payload written under descriptor A. Current code may decode it using compatible descriptor B, then produce a native projection using mapping revision C. Successful parsing does not prove that this is the historical interpretation. An archive that needs that interpretation must preserve the appropriate descriptor and mapping identity as well as the bytes.

The SurrealKV wrapper enables versioning and retention through its tree configuration. It does not derive either policy from a message's schema revision or timestamp field. A payload timestamp is application data unless explicitly mapped into a supported storage-history operation. ([SurrealKV integration](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/kvs-surrealkv/src/lib.rs))

### 6.3. Isolation and caller-owned retries

`SELECT ... FOR UPDATE`, added in 3.3, registers concrete record reads for optimistic commit-time conflict checking. It is not a blocking row lock: another writer can commit, causing the registered reader's later transaction commit to fail. Concrete absent IDs can be registered; arbitrary table predicates and ranges are not covered. The documented serializable transaction syntax is a 4.0 addition. ([Transaction semantics](https://surrealdb.com/docs/learn/querying/concepts-and-guides/transactions))

A decoder can validate one message while concurrent writes invalidate a cross-record assumption. Use native endpoint enforcement, uniqueness constraints, and an appropriate coordination pattern. For a predicate invariant, a shared guard record can coordinate cooperating writers if every writer follows the convention; it does not create a general predicate lock.

Retry the whole database transaction after a conflict, using a bounded policy appropriate to the application. Any external publication inside that operation must tolerate repetition or be deferred through durable state. Retrying only the final write can reuse decisions made from an obsolete snapshot, and retrying a decoder does not repair a missing transaction-level invariant.

## 7. Graph querying and typed graph contracts

### 7.1. Links, tracked references, and relations

A record link stores another record's ID. A tracked `REFERENCE` adds inverse-reference support and deletion behavior. A property-bearing relation record associates `in` and `out` endpoints and can carry its own attributes and identity. These represent different domain choices, not interchangeable encodings of two strings. ([Links and graph relations](https://surrealdb.com/docs/learn/data-models/graph/record-links-vs-graph-relations), [Record references](https://surrealdb.com/docs/reference/query-language/language-primitives/record-references))

An application message should specify whether a relationship is a reference, a unique endpoint pair, or an occurrence with its own lifecycle. A repeated list of target IDs may express an ordered application collection, while graph topology has different duplicate and ordering semantics. The adapter must choose deliberately rather than turning every repeated identifier into the same edge pattern.

### 7.2. Lightweight relations already exist in 3.3

The following SurrealQL example is illustrative:

```sql
DEFINE TABLE follows TYPE RELATION IN person OUT person LIGHTWEIGHT;
RELATE person:reader->follows->person:author;
```

A lightweight edge has canonical pair identity, such as `follows:[person:reader, person:author]`; repeating the directed pair is idempotent. Required endpoint-table definitions and endpoint enforcement accompany this model. A synthetic result exposes `id`, `in`, and `out`, but there is no independent payload-bearing edge record. ([Lightweight relation documentation](https://surrealdb.com/docs/reference/query-language/statements/define/table#using-lightweight-for-relations-without-properties))

Consequently, lightweight tables reject fields, indexes, events, changefeeds, direct live subscriptions, and data-bearing relation updates. They remain traversable and support applicable reads, counts, scans, deletion, cascades, and table permissions. Existing edges also constrain changes to their definition. ([Tagged DDL validation](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/core/src/legacy/expr/statements/define/table.rs), [Lightweight scanning](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/core/src/kvs/lightweight.rs))

Use ordinary relation records when parallel occurrences, attributes, or per-edge change visibility matter. A Protobuf edge event can describe either model, but an exporter cannot assume every traversable relationship has a conventional record changefeed.

### 7.3. Native adjacency caches and inline edge fields

`INLINE EDGES` and `INLINE REFERENCES` belong to the vertex table and cache bounded adjacency or incoming-reference information. Caps range from 0 to 256; high-degree cases fall back to normal scans. Updates are transactional, but cache construction is lazy. Exceeding a cap disables its use for that vertex until a relevant cap change, so reducing degree does not necessarily restore the previous fast path. ([Inline adjacency documentation](https://surrealdb.com/docs/reference/query-language/statements/define/table#using-inline-edges-and-inline-references-to-cache-adjacency), [Cache tests](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/core/src/kvs/tests/inline_cache_test.rs))

```sql
DEFINE TABLE person INLINE EDGES 64 INLINE REFERENCES 32;
```

A separate facility marks a small stored top-level relation field `INLINE`, allowing an in-path predicate to filter during adjacency scanning. The parenthesized traversal form can use it; a later array filter has a different execution position. The default inline-property cap is 64 bytes, with fallback for oversized payloads. This does not apply to lightweight edge payloads, which do not exist. ([Inline field documentation](https://surrealdb.com/docs/reference/query-language/statements/define/field#using-inline-to-filter-a-traversal-during-the-scan), [Inline property write path](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/core/src/doc/inline.rs))

```sql
DEFINE FIELD score ON likes TYPE number INLINE;
SELECT VALUE ->(likes WHERE score > 8)->post FROM person:reader;
```

These examples illustrate native mechanisms to investigate before maintaining serialized adjacency lists in application blobs. Their benefit depends on degree distribution, predicate selectivity, payload size, and write amplification. Reverse-reference `MATERIALIZE INTO` is a separate 4.0 feature; 3.3 inverse traversal and caching should not be described as that clause. ([Field definition version markers](https://surrealdb.com/docs/reference/query-language/statements/define/field#keeping-a-list-of-references-with-materialize-into))

### 7.4. SurrealQL, ISO GQL, and result design

ISO GQL arrived in 3.2 and is enabled by default in 3.3 through `POST /gql`, the `gql` RPC surface, and supported integrations. Table names supply node labels and relation tables supply edge types. The supported subset includes graph patterns, path operations, aggregation, and mutations; it is not a claim of complete ISO or openCypher conformance. ([GQL overview](https://surrealdb.com/docs/learn/querying/gql/overview), [Tagged GQL implementation](https://github.com/surrealdb/surrealdb/tree/v3.3.5/surrealdb/gql))

Choose the output contract for its actual use. A path response needs ordered identities. A subgraph response can send node and edge sets once and refer to them from paths. A business projection can expose only selected properties. Recursively nesting every neighbor can duplicate shared nodes or expand cycles into an unsuitable message structure. Protobuf offers message composition; it does not choose graph identity or a finite traversal boundary for the application.

### 7.5. Diagnostics have a 4.0 boundary

The opt-in RPC graph trace categories described in the 4.0 reference are explicitly marked since 4.0. They should not appear as an available 3.3 query option. In 3.3, returned path identities, query plans, and application instrumentation can answer narrower diagnostic questions, but none should be relabeled as the later trace or as complete execution provenance. ([RPC trace version marker](https://surrealdb.com/docs/reference/rest-api/rpc-protocol#tracing-the-graph-a-query-walked))

## 8. Surrealism modules as a Protobuf boundary

### 8.1. The experimental 3.3 module model

Surrealism in 3.3 uses Rust WebAssembly components targeting `wasm32-wasip2`, packaged as `.surli` archives, a `surrealism.toml` project, and `DEFINE MODULE mod::... FROM ... UNSIGNED`. The framework is experimental and capability-gated. A compatible archive includes export metadata and can attach a read-only filesystem containing descriptor resources. ([3.3 demo configuration](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealism/demo/surrealism.toml), [3.3 module statement](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/sql/src/statements/define/module.rs), [ABI configuration](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealism/runtime/src/config.rs))

A generated decoder could accept native bytes and return native values; a reflection-based module could load an approved descriptor set and handle several message types. An encoder could produce an application response. These are possible compositions, not evidence that an upstream generic Protobuf module has been shipped or that a chosen decoder has been built successfully for the component target.

### 8.2. Pooled instances and retained state

Unlike 4.0's fresh-invocation model, 3.3 reuses pooled controllers. Linear memory and Rust statics persist between calls to the same controller, while invocation context is replaced and cleared. Initialization can prepare state once per controller. Multiple controllers mean multiple independent statics, not one global descriptor cache. ([Controller lifecycle](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealism/runtime/src/controller.rs), [Initialization demo](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealism/demo/src/lib.rs))

For schema resources, a per-controller immutable descriptor pool can be useful. It must tolerate controller replacement, package changes, and server restart. Avoid caching caller-sensitive values in a static unless their identity and lifetime are controlled. A successful earlier call must not determine a later caller's schema authorization or tenant context.

### 8.3. Database access and writes

3.3 modules expose host functions such as `sql`, `sql_with_vars`, and `run`. Functions marked `#[surrealism(writeable)]` drive read-write planning, and host callbacks execute using the caller's child context and options. Thus 3.3 modules are not universally pure or read-only. The explicit Rust `Transaction<ReadWrite>` parameter shown for 4.0 is nevertheless a different API and must not be copied into a 3.3 example. ([Guest imports](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealism/src/imports.rs), [Host execution](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/core/src/surrealism/host.rs), [Module access planning](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/core/src/exec/physical_expr/function/module.rs))

Transaction-local conversion can avoid a separate network round-trip, but a database-aware decoder still requires correct grants, query parameterization, and failure handling. An outbound network side effect cannot be undone by rolling back native records. Durable publication therefore remains better modeled through explicit persisted work and acknowledgement than through an assumption that serialization inside the database makes external delivery atomic.

### 8.4. Migration and packaging boundaries

Keep a compatible 0.6.x SDK/runtime/ABI combination with the selected 3.3 patch. Do not use current `EXTENSION`, `ext::`, ABI 4/5, `BINDINGS`, digest/signer clauses, or build-time initialization snapshot examples as 3.3 APIs. Descriptor-driven backfills require a separate explicit transformation process; `MIGRATE EXISTING ROWS` is not a 3.3 facility. Details of resource controls and distribution appear in the addendum. ([Extension architecture with pre-4.0 contrast](https://surrealdb.com/docs/learn/extensions/guides/extension-architecture), [3.x migration guide](https://surrealdb.com/docs/build/migrating/from-old-surrealdb-versions/3x-to-4x))

## 9. Events, changefeeds, and Kafka

### 9.1. Change mechanisms have different contracts

| Mechanism | Native behavior | Appropriate Protobuf role |
| --- | --- | --- |
| Live query | Push committed matching changes through a subscription | Typed transient notifications |
| Changefeed | Retained mutations queried by cursor | Replay envelopes and export progress |
| Synchronous event | Logic inside the triggering transaction | Maintain native projection or outbox state |
| Asynchronous event | Atomically queued work, later execution | Retryable processing with explicit side-effect policy |
| Maintained view | Derived table updated from source-table writes | Queryable projection for response generation |
| Kafka source tool | External consumption and database upserts | Existing ingestion path for supported Protobuf schemas |

A live subscription is not retained replay. Concurrent clients do not establish a strict total notification order, and the 4.0 registration/concurrent-write visibility improvement should not be promised for 3.3. Native gRPC's advertised live guarantees make the reconnect boundary particularly explicit. ([Live query documentation](https://surrealdb.com/docs/learn/querying/real-time/live-queries))

A synchronous event's failure rolls back the triggering transaction. Asynchronous events execute later in their own transaction, can observe newer state, and have at-least-once behavior with retry controls. Conflict-retry treatment has a 3.3.1 version qualification; the 3.3.5 baseline includes it. A repeated event must not blindly repeat an irreversible external effect. ([Event documentation](https://surrealdb.com/docs/reference/query-language/statements/define/event), [Tagged event execution](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/core/src/doc/event.rs))

### 9.2. Changefeed projection and replay

An illustrative configuration is:

```sql
DEFINE TABLE reading CHANGEFEED 3d INCLUDE ORIGINAL;
SHOW CHANGES FOR TABLE reading SINCE 0 LIMIT 100;
```

Retention is finite. `INCLUDE ORIGINAL` adds information needed to reconstruct earlier images, and deletes can carry original data in the supported form. Access to `SHOW CHANGES` requires database-level-or-higher system authority; it is not equivalent to a record user's permitted SELECT. ([SHOW CHANGES](https://surrealdb.com/docs/reference/query-language/statements/show), [Changefeed overview](https://surrealdb.com/docs/learn/querying/real-time/changefeeds))

The 3.3.5 reader treats the starting version inclusively, defaults to 100 scanned entries, and caps the limit at 1,000. It scans the database feed before filtering by table. Therefore, an empty table-specific result can occur even when later matching changes remain beyond entries consumed by other tables. Cursor advancement and completion detection must account for this; an empty filtered batch alone is not a reliable caught-up signal. ([Tagged changefeed reader](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/core/src/cf/reader.rs))

A durable exporter can persist original-message metadata and an explicit event/outbox record with the native mutation, then publish outside the transaction and acknowledge progress after successful delivery. This is an architectural pattern, not built-in exactly-once delivery. Stable event identity and consumer deduplication cover retry windows; retention monitoring and a snapshot/resynchronization procedure cover replay gaps.

### 9.3. Existing Kafka ingestion

`surreal-sync from kafka` accepts a `.proto` schema, decodes message payloads, and upserts into a table associated with the topic. Record identity can come from a configured key or payload field. This is a separate tool; the pinned source examined here is commit `30c66b1ba90b2b27706a810ebc7a3c2fc1580827`, reporting package 0.6.1. Its lockfile uses SurrealDB v3 SDK 3.2.3. That establishes a v3 integration surface, not a verified deployment against 3.3.5. ([Pinned Kafka guide](https://github.com/surrealdb/surreal-sync/blob/30c66b1ba90b2b27706a810ebc7a3c2fc1580827/docs/kafka.md), [Pinned lockfile](https://github.com/surrealdb/surreal-sync/blob/30c66b1ba90b2b27706a810ebc7a3c2fc1580827/Cargo.lock))

```sh
surreal-sync from kafka \
  --proto-path ./events.proto \
  --message-type Event \
  --brokers localhost:9092 \
  --group-id topology-import \
  --topic events \
  --to-namespace example \
  --to-database main
```

This is an illustrative command shape; endpoint, credentials, exact packaged version, and operational supervision need deployment-specific configuration.

The importer has a material compatibility limit: **its custom decoder errors on an unknown field number** rather than skipping or preserving the field. An additive producer change can therefore break ingestion until the supplied schema is updated, despite ordinary Protobuf wire compatibility. It also consumes raw payload bytes and rejects absent payloads; do not presume Confluent schema-registry framing or Kafka tombstones are handled by this path. ([Pinned decoder](https://github.com/surrealdb/surreal-sync/blob/30c66b1ba90b2b27706a810ebc7a3c2fc1580827/crates/kafka/src/from_kafka/proto/decoder.rs#L66), [Pinned consumer](https://github.com/surrealdb/surreal-sync/blob/30c66b1ba90b2b27706a810ebc7a3c2fc1580827/crates/kafka/src/from_kafka/consumer.rs#L378))

The implementation constructs a simplified schema from `.proto`, not an arbitrary external descriptor-set-driven mapper. Kafka polling and database write batching are separate; sink application precedes offset commit, leaving a replay window. Its limited JSON-string-to-native-object schema conversion is not general semantic mapping. Evaluate the existing integration before building a replacement, but test the precise producer schema and target server rather than extrapolate from the tool's name. ([Pinned parser](https://github.com/surrealdb/surreal-sync/blob/30c66b1ba90b2b27706a810ebc7a3c2fc1580827/crates/kafka/src/from_kafka/proto/parser.rs), [Incremental sink sequence](https://github.com/surrealdb/surreal-sync/blob/30c66b1ba90b2b27706a810ebc7a3c2fc1580827/crates/kafka/src/from_kafka/sync/incremental.rs))

## 10. Rust and Python tooling

### 10.1. Rust implementation families

| Tool family | Role | Integration consideration |
| --- | --- | --- |
| Prost / prost-build / prost-types | Generated messages, compiler integration, well-known types | Close alignment with the native Rust protocol ecosystem |
| prost-reflect | Descriptor pools and dynamic messages | Runtime schema handling and explicit unknown-field retention |
| Tonic / tonic-prost-build | gRPC transport and service generation | Separate message generation from service/client generation |
| pbjson / pbjson-build | Descriptor-based ProtoJSON implementations | Avoid equating generic Serde with ProtoJSON |
| Buffa | Owned messages, borrowed views, reflection, unknown fields | Different traits and representation; pre-1.0 at 0.9.2 |
| Google's Rust Protobuf | Official generated API and runtime family | Distinct kernel/API and compatibility requirements |
| rust-protobuf | Independent implementation with reflection | Used by the examined Kafka tool, not Google's Rust runtime |
| Buf | Build, lint, compatibility, generation, inspection, registry | Message lifecycle governance rather than database migrations |

The versions discussed for application tooling are independent of SurrealDB's lockfile. Relevant examined releases include Prost/prost-build/prost-types 0.14.4, prost-reflect 0.16.5, tonic-prost-build 0.14.9, pbjson-build 0.9.0, Buffa 0.9.2, and rust-protobuf 3.7.2. They are examples of available capabilities, not a tested combined dependency set. ([Prost](https://docs.rs/prost/0.14.4/prost/), [tonic-prost-build](https://docs.rs/tonic-prost-build/0.14.9/tonic_prost_build/), [pbjson-build](https://docs.rs/pbjson-build/0.9.0/pbjson_build/), [rust-protobuf](https://docs.rs/protobuf/3.7.2/protobuf/))

`prost-build` can select Bytes-backed fields, ordered maps, external type paths, generated attributes, and descriptor output. These controls can reduce duplication and copying, but generated attributes must satisfy the destination trait's actual requirements. They do not automatically infer table identity, null semantics, or relationship lifecycle. ([prost-build configuration](https://docs.rs/prost-build/0.14.4/prost_build/struct.Config.html))

### 10.2. Buffa as a technical alternative

Buffa 0.9.2 provides owned messages, borrowed views, Bytes-backed owned views, unknown-field retention, open-enum representations, optional JSON/text support, and `no_std + alloc` operation. It also documents runtime reflection with `DescriptorPool` and `DynamicMessage`, plus generated `ReflectMessage` support. It is therefore more than a collection of descriptor structs. ([Buffa 0.9.2 README](https://github.com/anthropics/buffa/blob/v0.9.2/README.md))

Borrowed decoding is attractive when a consumer inspects a few fields and can keep the backing bytes alive. It is less decisive when all fields must become independently owned native values, graph records, or index entries. Lazy views can defer a malformed-field error until the field is accessed, so validation and error timing should be deliberate. ([Buffa guide](https://github.com/anthropics/buffa/blob/v0.9.2/docs/guide.md))

Buffa's tagged conformance/compiler claims extend through Edition 2024 and compiler v33.5. They do not establish Edition 2026 support. Its generated message traits are not drop-in replacements for the Prost traits expected by existing Tonic/SDK code. Select it for a demonstrated representation or runtime advantage and qualify the complete integration rather than infer compatibility from a shared wire format.

### 10.3. Official Rust and independent Rust APIs

Google's official Rust implementation uses its own owner, view, and mutable-accessor conventions and kernel abstraction. It is separate from Prost, Buffa, and the independent `protobuf` crate. Generated structures from the same schema need not share Rust types or traits. ([Official Rust generated-code guide](https://protobuf.dev/reference/rust/rust-generated/))

Google's runtime compatibility guarantees also do not govern independent implementations. Newer generated code against older runtimes is unsupported, and Rust/C++ coupling differs from Python's. Pin the generator/runtime family deliberately; a descriptor's compatibility alone does not prove generated-code linkage compatibility. ([Cross-version runtime guarantees](https://protobuf.dev/support/cross-version-runtime-guarantee/))

### 10.4. Python interoperability

Python and Rust can share the wire schema without sharing object layouts. Python generated messages expose presence and serialization APIs, while dynamic factories and `json_format` serve runtime schemas and ProtoJSON. Prefer version-matched APIs: an older rendered documentation page branded as latest can show obsolete signatures. ([Python generated-code guide](https://protobuf.dev/reference/python/python-generated/), [v36.0 dynamic factory](https://github.com/protocolbuffers/protobuf/blob/v36.0/python/google/protobuf/message_factory.py), [v36.0 JSON conversion](https://github.com/protocolbuffers/protobuf/blob/v36.0/python/google/protobuf/json_format.py))

Python's arbitrary-precision integer implementation does not enlarge a declared Protobuf field or the native database integer range. Cross-language fixtures should cover boundary integers, bytes, decimals, temporal normalization, enum numbers, and explicitly absent/default values. A plain dictionary produced by a JSON helper also needs an adapter; it is not automatically a lossless native database value.

### 10.5. Buf and inspection

Buf provides schema compilation, linting, breaking checks, multi-language generation, conversion, RPC invocation, and registry/module workflows. Its registry can distribute schemas and generated SDKs, but does not apply database DDL or mapping revisions. ([Buf CLI](https://buf.build/docs/cli/), [Buf Schema Registry](https://buf.build/docs/bsr/))

Because SurrealDB's native gRPC service lacks reflection, inspection tools need its matching published definitions. Providing reflection in an application-owned service is a separate choice and does not enable it on the database listener. The practical integration artifact is a pinned descriptor/source bundle that matches the selected native protocol, not an assumption that an inspection client can discover all definitions from the server.

## 11. Tradeoffs and practical selection

### 11.1. Use native mechanisms where their semantics fit

| Requirement | Mechanism to evaluate first | Additional Protobuf work |
| --- | --- | --- |
| Database-value exchange | Native gRPC or public native Value codec | Matching published protocol and semantic checks |
| Stable business API | Application-owned Protobuf service | Explicit query/projection and authorization adapter |
| Native graph traversal | Relations, references, SurrealQL/GQL | Typed output contracts |
| Propertyless unique connection | Lightweight relation | Explicit domain events if required |
| Bounded adjacency optimization | Inline edges/references or inline edge predicates | Usually none |
| Exact incoming-message archive | Native bytes plus type/schema identity | Immutable-byte and descriptor-retention policy |
| Heterogeneous schemas | Descriptor pool and dynamic messages | Mapping metadata and validation |
| Recent-change replay | Changefeed or persisted outbox | Cursor, acknowledgement, deduplication, and recovery policy |
| Protobuf Kafka ingestion | Existing surreal-sync source path | Exact schema/server compatibility tests |
| Small in-database conversion | Compatible experimental module | WASI, ABI, resource, and authority qualification |

### 11.2. Performance follows the whole operation

Measure decoding, conversion, network framing/compression, query execution, graph expansion, indexing, storage, and conflicts separately. A small binary message does not compensate for an unsuitable query, a duplicated graph projection, or an excessively broad transaction. Streaming lowers one buffering requirement but still leaves individual-record limits and statement-completion semantics.

For an original-bytes archive, decoding every payload before storage can be unnecessary. For indexed retrieval, hiding all fields in bytes can create application-side scan costs. A hybrid model trades projection maintenance for native query access. Compare these actual workflows before choosing a codec based on a microbenchmark. No whole-database speed ranking among Protobuf, CBOR, FlatBuffers, Prost, or Buffa is established here.

### 11.3. Authority and resource controls remain explicit

An accepted descriptor, a structurally valid message, an authorized operation, and a committed write are different conditions. Resolve schemas from approved sources, bound decoding resources, parameterize database values, and restrict table/path mappings. Do not interpret an arbitrary message type name as permission to access the similarly named table.

For output, apply the intended permissions before constructing the response. Retaining original bytes can preserve fields that are intentionally omitted from a native projection, so the archive needs its own access policy. A consumer authorized to query a few projected fields should not automatically gain access to the original full payload.

### 11.4. Implemented primitives and conditional compositions

Native gRPC, the public native Protobuf codec, graph features, and tagged engine/module APIs are implemented facilities. A schema-to-graph generator, a general Protobuf validation module, a registry-aware Kafka adapter, and an exactly-once exporter are separate integrations. Their required pieces can exist without the assembled behavior being established.

For a 3.3 deployment, pin the server patch, native protocol, decoder/generator family, mapping revision, and any module ABI or connector version. This gives a concrete compatibility boundary that can be exercised as a whole rather than a loose assertion that all components support Protobuf.

## 12. Integration guidance

The lowest-complexity native path is the published database protocol and its own `Value` codec. The strongest independent application contract is a domain schema with a deliberate adapter. The strongest exact-input archive is the original bytes with durable schema identity. Choosing among them first makes later decisions about reflection, modules, storage history, and change processing substantially clearer.

For SurrealDB 3.3.x, the most consequential implementation details are the `uint64` conversion hazard, the fallible native codec, pooled ABI 2 module state, native graph event differences, and replay/cursor semantics. These determine whether an integration preserves meaning under large values, concurrent writes, schema evolution, reconnects, and retries. They deserve more attention than adding another serialization layer where a native mechanism already meets the requirement.

# Addendum: Advanced Protocol Buffers integration with SurrealDB 3.3.x

The following sections expand the implementation boundaries behind the overview. They use the same 3.3.5 baseline and distinguish native database schemas from independently governed domain messages.

## A1. Native Protobuf support in surrealdb-types

### A1.1. The public codec is independent of gRPC

The optional `proto` feature on `surrealdb-types` enables the native Protobuf codec and matching runtime exports. Its public aliases are `encode_proto` and `decode_proto`. An application using public values can therefore serialize them without establishing a gRPC connection or depending on the full database execution core. ([Types feature declaration](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/types/Cargo.toml), [Public exports](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/types/src/lib.rs))

The conversion has two stages: native `surrealdb_types::Value` to the protocol's `proto::v1::Value`, then Protobuf encoding through Prost. Decoding reverses those stages, with native conversion allowed to fail even after valid Protobuf parsing. The encoder clones its input during conversion; its existence is not a zero-allocation claim. ([Codec implementation](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/types/src/proto/mod.rs))

```toml
[dependencies]
surrealdb-types = { version = "=3.3.5", features = ["proto"] }
```

An illustrative call sequence, assuming `value` is an existing native value, is:

```rust
let encoded = surrealdb_types::encode_proto(&value)?;
let restored = surrealdb_types::decode_proto(&encoded)?;
```

These functions consume the database's specific schema. Passing an application's `GraphEvent` encoding does not cause them to discover that application type or create graph records.

### A1.2. The 3.3 conversion subset

3.3's native numeric model has `Int(i64)`, `Float(f64)`, and `Decimal`. It does not have the newer sized numeric and date/time/timestamp/interval value families described in the 4.0 addendum. Consequently, the 4.0 widening and `degrade_temporal` discussion is not the correct failure table for 3.3. ([3.3 Number](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/types/src/value/number.rs))

| Value or wire situation | 3.3 native conversion behavior | Integration implication |
| --- | --- | --- |
| Int, float, decimal | Explicit native variants | Preserve intended type rather than passing through generic JSON |
| Native none versus null | Separate wire alternatives | Missing input is not automatically native NONE |
| Bytes, arrays, sets, objects | Native structures have corresponding messages | Structure alone does not establish canonical bytes |
| UUID | Requires valid 16-byte representation | Reject malformed identity data |
| Decimal text | Parsed into supported native decimal | Invalid or unrepresentable data can fail |
| Datetime | Checked native conversion | A valid message can exceed native range |
| Duration encoding | Seconds must fit signed wire range | Large unsigned native duration can fail |
| Negative duration decoding | Rejected by unsigned native duration | Wire schema can express more than native 3.3 |
| Unset or unsupported Value alternative | Rejected | Never reinterpret it as removal-like NONE |
| Raw `string_record_id` | Not accepted by the native Value decoder | Parsing is a separate explicit step |
| Floating/decimal record key | Rejected by native record-key conversion | Wire representability is broader than native key support |
| Invalid regex or missing required nested value | Conversion can fail | Parsing success is insufficient validation |

The supported variants and failures are explicit in the tagged value and record converters; temporal normalization lives in the protocol helper implementation. ([Value conversion](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/types/src/proto/value.rs), [Record conversion](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/types/src/proto/record.rs), [Protocol temporal helpers](https://github.com/surrealdb/surrealdb-protocol/blob/v0.13.1/rust/methods.rs))

### A1.3. Round-trip tests and archival meaning

The native codec's source tests cover a broad collection of conventional native values and compare reconstructed values with FlatBuffers paths. They establish intended test coverage, not a test run performed for this guide, canonical wire equality, or arbitrary future-value support. ([Codec tests](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/types/src/proto/mod.rs))

For a native-value archive, record the protocol/codec version and supported value subset. For an application-message archive, retain the application's original payload and descriptor identity. Converting the payload into native values and then invoking the native codec creates a new database-specific encoding; it does not preserve the original application message or fields that the projection omitted.

## A2. Rich protocol values and semantic validation

### A2.1. Database-specific structures

The native schema provides a `Value` union plus explicit records, keys, bounds/ranges, geometry, files, objects, sets, and custom temporal values. It is a database protocol designed to carry distinctions that ordinary JSON-shaped messages can lose. Its UUID bytes, decimal text, record table, and key alternative are part of the contract, not decorative metadata. ([Value schema 0.13.1](https://github.com/surrealdb/surrealdb-protocol/blob/v0.13.1/surrealdb/protocol/v1/value.proto))

This can save substantial custom representation work when every consumer is intentionally a SurrealDB-aware client. It also makes the external contract database-specific. An application seeking a portable domain API can instead expose only the key and value types its domain actually uses, with an adapter at the boundary.

### A2.2. Structured record IDs

Native 3.3 record keys include signed integers, strings, UUIDs, arrays, objects, and ranges. The protocol also contains floating-point and decimal key alternatives that the native 3.3 converter rejects. A raw string-record-ID alternative is not a native parsed ID. ([Record conversion](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/types/src/proto/record.rs))

A string-only record-reference message is valid for an application that deliberately restricts keys to strings. It is insufficient as a universal database-ID representation. Likewise, concatenating a table and key with a colon introduces a parsing/escaping convention; it should not be treated as equivalent to carrying a typed native key. This matters for graph joins, deduplication, and signatures because identity depends on the structured value, not merely a convenient rendering.

### A2.3. Objects and an implementation/specification discrepancy

Protocol objects use repeated `KeyValue` entries rather than ordinary Protobuf maps. Their contract requires unique keys, ascending UTF-8-byte ordering, and present nested values; absence of a key, a key holding NONE, and a key holding NULL are distinct representations. ([Object schema](https://github.com/surrealdb/surrealdb-protocol/blob/v0.13.1/surrealdb/protocol/v1/value.proto))

The 3.3 native encoder naturally produces sorted unique entries from a BTreeMap. The examined decoder, however, inserts entries into a map without rejecting duplicates or unsorted input. **Later duplicates overwrite earlier entries**, while ordering is normalized. This is a concrete implementation/schema discrepancy, not permission to assume all decoders behave that way. Strict gateways should validate these conditions before native conversion if they rely on unique wire keys. ([Native object decoder](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/types/src/proto/value.rs))

### A2.4. Temporal messages are not the well-known types

The database's custom datetime can represent a wider range than the standard `Timestamp` year range. Its duration wire message permits signed seconds, while native 3.3 uses `std::time::Duration`. The native converter rejects negative wire durations and seconds outside its supported conversion range; the custom message names do not confer standard well-known-type ProtoJSON behavior. ([Temporal schema](https://github.com/surrealdb/surrealdb-protocol/blob/v0.13.1/surrealdb/protocol/v1/value.proto), [Temporal conversion helpers](https://github.com/surrealdb/surrealdb-protocol/blob/v0.13.1/rust/methods.rs))

When bridging to `google.protobuf.Timestamp` or `Duration`, define the accepted intersection explicitly. A signed application duration can be represented by a signed amount plus unit or a dedicated signed message rather than silently forced into the database's unsigned duration. Retaining the original signed message alongside a restricted query projection is another deliberate hybrid choice.

## A3. Generated messages and the Rust type system

### A3.1. SurrealValue and structural kinds

`SurrealValue` supplies `kind_of`, `into_value`, and fallible `from_value`, with a derive macro and public `kind!` support. `Kind` can express primitives, arrays, sets, records, unions, and object/literal structure; its SQL rendering can help construct the type part of field definitions. This describes structure, not indexes, events, permissions, or a complete migration. ([Trait implementation](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/types/src/traits/surreal_value.rs), [Kind implementation](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/types/src/kind/mod.rs), [Literal kinds](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/types/src/kind/literal.rs))

Serde configuration and `#[surreal(...)]` configuration are different. Adding a Serde flattening attribute to a generated message does not automatically establish the equivalent native database shape. Generated code must actually implement the required destination traits or pass through a suitable adapter. ([Derive implementation](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/types/derive/src/lib.rs))

### A3.2. Three adapter designs

| Design | Strength | Cost or limitation |
| --- | --- | --- |
| Generated message to explicit Rust database model | Clear domain/native separation and focused range checks | A model and conversion must be maintained |
| Dynamic/generated message directly to native Value | One mapping engine can support heterogeneous shapes | Runtime mapping and error paths need explicit semantics |
| Native Value through the upstream Protobuf codec | Reuses database-specific value handling | External consumers depend on the native schema |

An explicit model is often the most reviewable choice for a small stable domain. Reflection is more attractive when many schemas share a mapping policy. The native codec fits a database client or gateway whose contract is already the native value system. None should be selected only to avoid writing a few conversion functions if doing so obscures identity or mutation semantics.

For unsigned values, place the range check at the application boundary. A checked `i64::try_from` is appropriate when the domain permits only signed-range values; otherwise use a deliberate exact decimal/string path. Do not first invoke the convenience `u64` conversion and then validate the already wrapped result. Reading from storage also needs the inverse policy, including a clear response when older records use a different representation.

### A3.3. Reusable database contracts without DEFINE TYPE

3.3 has structural kinds, field definitions, literal unions, and assertions, but not 4.0's named `DEFINE TYPE` facility. A generator can still reuse its own intermediate schema model to emit multiple field definitions. That reuse belongs in the generator/application, not in a fabricated 3.3 named-type syntax. ([3.3 Kind source](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/types/src/kind/mod.rs), [DEFINE TYPE version boundary](https://surrealdb.com/docs/reference/query-language/statements/define/type))

A generated enum mapping should specify whether numeric identity or human-readable names are authoritative. A numeric enum can receive an unknown future number; a closed string union can reject the corresponding new label. An adapter can retain both the number and a best-effort label, or deliberately enforce a closed domain. Protobuf compatibility checks and database constraint checks must then follow that declared policy.

## A4. CBOR, FlatBuffers, and conformance

### A4.1. Representation selection in 3.3

Tagged CBOR carries richer database types than ordinary JSON, including record IDs, bytes, decimals, temporal values, geometry, ranges, and NONE. FlatBuffers is another native representation already present in 3.3. CBOR v2 and shaped results are 4.0 additions; the later removal of legacy textual temporal/UUID CBOR tags also must not be projected backward. ([CBOR protocol with version qualifications](https://surrealdb.com/docs/reference/rest-api/cbor-protocol), [3.3 formats](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/core/src/rpc/format/mod.rs))

| Goal | Candidate | Decision criterion |
| --- | --- | --- |
| Native database fidelity with existing client tooling | Tagged CBOR | Client's actual native-tag support |
| Generated native value messages or gRPC | Native Protobuf | Supported native conversion subset and service semantics |
| Existing FlatBuffers client path | Native FlatBuffers | Implemented value and type-description parity |
| Independent service or event schema | Domain Protobuf | Mapping and schema lifecycle ownership |
| Exact original payload retention | Opaque bytes | Descriptor availability and byte preservation |

For homogeneous result sets, 3.3 gRPC streaming can reduce whole-response buffering. It is not shaped encoding and does not remove repeated per-value metadata by the same mechanism. Choosing a compact domain projection can reduce output data for an independent reason: the application simply sends fewer fields.

### A4.2. Schema parity is not binary interchangeability

Protocol 0.13.1 has separate Protobuf and FlatBuffers definitions and an explicit parity configuration. Union identifiers do not map numerically one-to-one; a legacy positional difference creates a deliberate offset. `Kind` has unresolved representation divergence and no general cross-format conversion. A gateway can therefore handle ordinary values correctly while failing on type descriptions. ([0.13.1 parity configuration](https://github.com/surrealdb/surrealdb-protocol/blob/v0.13.1/parity.toml))

Convert through defined semantic structures, not casts between enum/tag numbers. Test values and kinds separately. A successful generated-code build only demonstrates that the schema can generate code, not that all cross-format conversions preserve meaning.

### A4.3. The applicable validation facilities

The 0.13.1 protocol tree contains schema parity machinery and Rust conversion tests. It does not contain the later `spec/` and `conformance/` directories cited for protocol 0.14.1 in the 4.0 addendum. Those later facilities may be useful development context, but should not be described as version-matched 3.3 conformance artifacts. ([0.13.1 tree](https://github.com/surrealdb/surrealdb-protocol/tree/v0.13.1), [Protocol Rust tests](https://github.com/surrealdb/surrealdb-protocol/blob/v0.13.1/rust/lib.rs))

Keep validation levels distinct: Protobuf parsing, native-value conversion, RPC/session behavior, application mapping, and transaction/delivery behavior. A fixture that covers one cannot establish the others. Boundary cases should come from the application's actual contract, including absent/null/default values, unknown enum numbers, large unsigned values, typed IDs, and publication retries.

## A5. Surrealism runtime, capabilities, and distribution

### A5.1. ABI 2 and state lifecycle

The 3.3 package uses `surrealism.toml`, WASI Preview 2 components, and ABI 2. Its archive can include descriptor resources through attached files. A controller is initialized and reused in a bounded pool, so statics can retain parsed descriptors across invocations. This is runtime initialization, not the 4.0 build-time memory snapshot lifecycle. ([3.3 WIT](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealism/wit/surrealism.wit), [Runtime configuration](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealism/runtime/src/config.rs), [Demo init](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealism/demo/src/lib.rs))

For a decoder module, an immutable descriptor bundle attached to the package gives a clear build/schema association. Initialize it per controller, cap its size, and expect reinitialization after eviction or restart. If descriptors vary per call, cache keys must include the complete schema identity and authorization domain. A type name alone is insufficient because distinct bundles can define different versions of that name.

### A5.2. Database authority and resource limits

Package capabilities include `allow_scripting`, `allow_arbitrary_queries`, `allow_functions`, `allow_net`, memory and execution limits, pool size, and KV limits. Empty function/network lists deny those operations. Requested capabilities cannot exceed server grants, and arbitrary SQL has its own check. The `writeable` metadata used by planning is separate from permission to run arbitrary host operations. ([Capabilities](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealism/runtime/src/capabilities.rs), [Capability validation](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/core/src/surrealism/mod.rs))

`strict_timeout=false` disables the compiled epoch checks and requires operator trust for unbounded execution. A statement that every module always has enforced timeouts would therefore be too broad. Bound decoded message size, descriptor size, recursion, collections, and unknown-field storage independently of total WASM memory. ([Server Surrealism configuration](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/core/src/surrealism/config.rs))

Network capability does not establish compatibility of a conventional Tonic client with the component target. A runtime may depend on sockets, threading, or asynchronous facilities absent from the guest environment. A pure Rust or `no_std` codec can simplify one part of portability while leaving ABI, host imports, panic behavior, and resource limits unresolved.

### A5.3. The module KV store is volatile

3.3 exposes a per-module KV facility backed by a shared locked map in the runtime. It is shared across that runtime's invocations but is lost on runtime drop, cache eviction, or restart. It is neither a database transaction nor a durable outbox/checkpoint store. ([Runtime KV implementation](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealism/runtime/src/kv.rs))

Use it only for reconstructible state whose loss is acceptable. Authoritative descriptor revisions, delivery progress, and deduplication decisions belong in persistent records or another explicitly durable system. If a cache entry affects tenant-sensitive behavior, include tenant identity and validate access rather than assuming the host's cleared invocation context clears retained guest data.

### A5.4. Package integrity and deployment

A 3.3 definition can load an unsigned archive from a file bucket or a versioned Silo package. The Silo loader defaults to the official endpoint and supports a 3.3 endpoint override, package-size checks, and path validation. Registry retrieval is a distinct host operation from a module's runtime network allowlist. ([3.3 module definition](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/sql/src/statements/define/module.rs), [Silo loader](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/core/src/surrealism/silo.rs))

Do not assume the 4.0 `DIGEST`, `SIGNER`, and typed `BINDINGS` clauses exist in 3.3. Pin and verify package content through the available deployment process, and keep executable trust separate from descriptor trust. A legitimate schema does not make an arbitrary WASM archive trustworthy; a trusted archive can still reject or misinterpret an unsupported schema revision.

## A6. Custom HTTP APIs carrying Protobuf

### A6.1. Bytes are available, but not a generic decoder

3.3 custom APIs can parse bodies using strategies including auto, JSON, CBOR, FlatBuffers, plain, bytes, and native. For explicit `api::req::body("bytes")`, the examined middleware validates `application/octet-stream`. Sending `application/x-protobuf` to that strategy is not automatically equivalent. Automatic parsing has no generic arbitrary-Protobuf interpretation. ([Request parser](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/core/src/api/middleware/req.rs), [Body strategies](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/core/src/api/middleware/common.rs))

The possible composition is bounded octet-stream input, an explicit compatible decoder, validated native values, a consciously managed database operation, and optional binary output. Response middleware provides supported body serialization, but does not infer an application message schema. This is Protobuf-over-HTTP composition, not generated gRPC dispatch. ([Response serializer](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/core/src/api/middleware/res.rs))

### A6.2. Bound the raw body before decoding

`api::req::max_body` checks raw bytes or string byte length and returns 413 when the configured limit is exceeded. Place it before parsing; already parsed nonraw values do not receive the same check. Decoder-specific nesting, collection, recursion, and computational limits remain necessary. ([Request limit implementation](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/core/src/fnc/api/req.rs))

It is useful to reject obviously excessive input before allocating a descriptor-based message graph. A small request can still create expensive nested work, however. Treat raw size, decoded structure, database fan-out, and execution time as separate budgets rather than one interchangeable maximum.

### A6.3. Endpoint permission and execution authority differ

The 3.3 API implementation checks endpoint/method permissions and then executes with ordinary table/field permission checks disabled in an auth-limited context. Tenant checks remain explicit. It has no 4.0 `SECURITY INVOKER` selector. Consequently, controlling who invokes the endpoint does not automatically make the handler's reads and writes equivalent to an ordinary query by that caller. ([API invocation](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/core/src/api/invocation.rs), [3.3 API statement](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/sql/src/statements/define/api.rs), [API permissions version marker](https://surrealdb.com/docs/reference/query-language/statements/define/api#permissions))

A public binary endpoint therefore needs an explicit authority policy for every decoded operation and returned field. A message that includes a record ID is not authorization to retrieve that record. If handler code acts with broader access than the caller, restrict selectable tables, fields, and operation types directly.

### A6.4. Pre-4.0 failure behavior matters

The official documentation identifies improved handler rollback as a 4.0 change and describes pre-4.0 cases where a failing handler could retain earlier writes, including inconsistent index-related effects. A 3.3 API that decodes and writes must not be presented as automatically all-or-nothing merely because its code executes in the database. ([Handler-failure version boundary](https://surrealdb.com/docs/reference/query-language/statements/define/api#when-a-handler-fails))

For 3.3, qualify the precise endpoint failure path or keep orchestration in a service using an explicit, tested transaction boundary. External HTTP calls and broker publication remain outside database rollback in either design. A separate service can also be a clearer fit for standard gRPC, large streams, long-running work, or independently scaled decoding.

## A7. Standalone SurrealKV and SurrealMX persistence

### A7.1. SurrealKV 0.21.4

The standalone crate provides an embedded LSM store with transactions, optional retained history, `get_at`/`set_at`, history iteration, configurable value logs, and transaction modes. A Protobuf payload can be stored directly as the value; its key and descriptor metadata remain application choices. These APIs do not create SurrealDB tables, indexes, permissions, or graph relationships. ([SurrealKV 0.21.4 README](https://github.com/surrealdb/surrealkv/blob/v0.21.4/README.md), [Transaction implementation](https://github.com/surrealdb/surrealkv/blob/v0.21.4/src/transaction.rs))

Versioning enables the value log; zero retention means unlimited retention. Out-of-order historical timestamps need the versioned B+tree index for correct reads. The default enabled value-log threshold is 1 KiB. `Eventual` durability returns after writing through OS buffers; `Immediate` requests synchronization before successful return. Database wrapper settings must be checked independently of the standalone default. ([Versioning](https://github.com/surrealdb/surrealkv/blob/v0.21.4/README.md#versioning-configuration), [Durability](https://github.com/surrealdb/surrealkv/blob/v0.21.4/README.md#durability-levels), [Integrated wrapper](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/kvs-surrealkv/src/lib.rs))

For historical message archives, retain both payload bytes and schema/mapping identity at the appropriate history point. Updating only the current descriptor pointer can make old byte versions ambiguous. Value-log separation can reduce large-value compaction work in suitable workloads, but is not proof of a Protobuf-specific advantage.

The later crate's encryption, diagnostic CLI, stable 1.x format, and browser OPFS discussion should not be imported as 0.21.4 capabilities without separate version-specific evidence.

### A7.2. SurrealMX 0.27.0

Standalone SurrealMX provides in-memory transactions, savepoints, snapshot and serializable options, and buffer-reusing `scan_into`/`keys_into`. The later `with_value` and `scan_with` methods cited in the 4.0 addendum are not established by the examined 0.27.0 source. ([SurrealMX 0.27.0 README](https://github.com/surrealdb/surrealmx/blob/v0.27.0/README.md), [Transaction APIs](https://github.com/surrealdb/surrealmx/blob/v0.27.0/src/tx.rs))

Reusing scan buffers can reduce allocation churn, but a decoder's view lifetime must still be limited by the backing bytes. An owned native database projection can require copying even when a message parser offers borrowed views. Avoid describing an unassembled combination of scan APIs and borrowed decoding as an established end-to-end zero-copy path.

Optional persistence uses append-only logging and snapshots, with separate logging, fsync, and snapshot policies. Synchronous logging alone is not a power-loss durability guarantee. The integrated 3.3 memory wrapper also wires optional persistence when a path is configured; ordinary pathless memory is volatile. ([Persistence modes](https://github.com/surrealdb/surrealmx/blob/v0.27.0/README.md#persistence-modes), [Memory integration](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/kvs-mem/src/lib.rs))

### A7.3. Keep engine stores and database internals separate

Use a separate standalone store or the supported database API. Writing application payloads into an existing SurrealDB engine's internal files/keyspace through lower-level crate APIs is not a supported record-integration shortcut. Sharing the same engine implementation does not make the internal database layout an application contract.

A second standalone store also introduces a second atomicity boundary. A successful payload commit there and a successful graph projection commit in SurrealDB do not automatically form one transaction. If both must stay coherent, use an explicit recovery protocol, immutable identity, or a design that keeps the authoritative mutation in one transactional system.

## A8. Graph projection freshness and typed alternatives

### A8.1. Lightweight topology needs an explicit event policy

Lightweight pair identity is ideal for a set of connections but unsuitable for distinct repeated occurrences. It also removes the ordinary edge-record event surface. An operation that needs durable domain events can write an explicit outbox/event record in the same intended database operation, rather than expecting direct live/changefeed support on the lightweight table. ([Lightweight tests](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/core/src/kvs/tests/lightweight_test.rs))

The event should specify whether it records an attempted operation, a newly created pair, or the final topology state. Repeated `RELATE` can leave topology unchanged while an application still receives a repeated input message. Stable event and operation identities prevent that distinction from becoming an accidental duplicate notification policy.

### A8.2. Maintained views have dependency and identity limits

`DEFINE TABLE ... AS SELECT` maintains a projection on writes to its `FROM` tables, not every related record read by the query. `OPTION IMPORT` bypasses maintenance, while redefinition with `OVERWRITE` can refill it. Views are read-only in this version line except supported import restoration. ([Precomputed views](https://surrealdb.com/docs/reference/query-language/statements/define/table#pre-computed-table-views))

There is also a 3.3/4.0 identity difference: 3.3 initial population could use a projected ID while subsequent maintenance used the source identity. The 4.0 projected-ID behavior should not be relied on for stable derived-row identity in 3.3. ([View ID behavior](https://surrealdb.com/docs/reference/query-language/statements/define/table#choosing-the-id-of-a-view-row))

A typed response generated from a maintained view can therefore have the correct schema and stale contents. Document the refresh dependency and repair path. If an independently changing related record influences a message, either make its updates trigger the projection explicitly or compute the response from current native records instead of assuming arbitrary dependency tracking.

### A8.3. GraphQL serves a different typed boundary

3.3 includes generated GraphQL schemas and configuration such as `DEFINE CONFIG GRAPHQL AUTO`, with selected table/function exposure and supported query, mutation, aggregation, and pagination behavior. This can meet an introspectable typed-query requirement without defining a second fixed message API. Limit exposure deliberately. ([GraphQL overview](https://surrealdb.com/docs/learn/querying/graphql/overview), [Tagged GraphQL implementation](https://github.com/surrealdb/surrealdb/tree/v3.3.5/surrealdb/core/src/graphql))

| Interface | Contract shape | Appropriate use |
| --- | --- | --- |
| Native gRPC | Database methods and native Value messages | Database-aware clients |
| Domain Protobuf | Application-owned fixed messages/services | Stable business operations and binary events |
| GraphQL | Generated typed field-selection schema | Client-selected projections |
| ISO GQL | Graph pattern language | Traversal, paths, and graph mutations |

These can coexist. An application can query with GQL, project selected results into a domain message, and serve them through gRPC. That composition should remain explicit rather than imply GraphQL or GQL itself provides a Protobuf wire representation.

## A9. Descriptor-driven mapping and evolution

### A9.1. A generic mapping pipeline

An effective descriptor-driven adapter resolves an approved bundle and fully qualified message name, decodes with presence information, validates the message, applies the mapping revision, and constructs native values/relationships. Generated and dynamic messages can be bridged through `prost-reflect`, but converting into a generated type can discard data that the dynamic representation retained. ([DynamicMessage](https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.DynamicMessage.html))

The reusable part is the traversal and conversion machinery. The schema-specific part decides identity, nesting, decimal and temporal policy, references, relation direction, and projection inclusion. Keeping those rules declarative can make many schemas manageable, but it creates an integration specification that needs its own versioning and validation.

### A9.2. Custom options and schema identity

| Mapping metadata | Example decision it can express |
| --- | --- |
| Table and key source | Message identity maps to a chosen table/key policy |
| Reference target | A field is a native record reference rather than plain text |
| Edge direction and type | A pair of fields creates a specific relationship |
| Numeric policy | A high-range integer becomes checked int, decimal, or string |
| Projection policy | A field is indexed natively, archived only, or excluded |
| Presence/update policy | Missing means retain, omit, remove, or reject |
| Mapping revision | Which interpretation governs stored and emitted data |

Custom descriptor options or a companion mapping manifest can carry such metadata. They become operational only when a generator or runtime interprets them. No first-party arbitrary-descriptor-to-graph generator is established here. In 3.3, database schema output must use the actual field/kind facilities rather than 4.0 named types.

Store a descriptor bundle identity including dependencies, the full message name, and the mapping revision with long-lived payloads. A digest identifies the selected schema bytes, not a canonical business entity. A registry lookup should not fetch executable code or unrelated schemas merely because a payload supplies a URL-like name.

### A9.3. Unknown fields depend on the entire path

| Path | Relevant behavior |
| --- | --- |
| Prost generated message | Unknown enum numbers survive; unknown message fields are skipped |
| prost-reflect dynamic message | Unknown fields can survive binary decode/re-encode |
| Buffa generated message | Preservation enabled by default in the examined generator |
| ProtoJSON conversion | Cannot preserve arbitrary unknown binary fields |
| Known-field native projection | Preserves only what its mapper chooses to retain |
| Examined surreal-sync decoder | Unknown field number is an error |
| Original immutable bytes | Retains input encoding independently of decoding |

The Prost distinction is explicit in generated merge logic, while dynamic-message and Buffa APIs document their preservation behavior. These are not interchangeable guarantees. ([Prost generated merge](https://github.com/tokio-rs/prost/blob/v0.14.4/prost-derive/src/lib.rs#L189), [Dynamic unknown fields](https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.DynamicMessage.html#method.unknown_fields), [Buffa generator configuration](https://github.com/anthropics/buffa/blob/v0.9.2/buffa-codegen/src/lib.rs#L1101))

Preservation at the first decoder does not protect a later projection that reconstructs only recognized fields. Conversely, byte retention does not mean an older consumer understands a new field's meaning. Decide whether a component forwards unknown data, rejects unsupported schema revisions, or deliberately drops data as a documented projection.

### A9.4. Edition support is a compatibility matrix

Edition 2026 support in `protoc` does not establish its support in a third-party generator, generated runtime, validator, Python consumer, or WASI target. Buffa's examined v0.9.2 conformance remains through Edition 2024; other generators require their own version-specific evidence. Descriptor interchange does not resolve unsupported edition features by itself. ([Protobuf v36.0](https://github.com/protocolbuffers/protobuf/releases/tag/v36.0), [Buffa conformance](https://github.com/anthropics/buffa/blob/v0.9.2/README.md#conformance))

Record these coordinates with build artifacts. A useful compatibility fixture uses the same schema to generate each supported language, exercises presence and unknown values, and checks the actual database adapter. This is more informative than a single declaration that the system uses the latest Protobuf edition.

## A10. Partial updates, validation, and durable streams

### A10.1. FieldMask requires an update policy

The standard `FieldMask` comments describe selected fields, reset-to-default behavior, terminal message merge, and repeated-field append defaults, while allowing applications to define other semantics. Omitting a mask can mean updating all fields, which can reset values unknown to an older client. It is not automatically equivalent to SurrealQL `MERGE`, `CONTENT`, `PATCH`, or an assignment list. ([FieldMask definition](https://github.com/protocolbuffers/protobuf/blob/v36.0/src/google/protobuf/field_mask.proto))

| Intent | Suggested adapter rule |
| --- | --- |
| Leave unchanged | No operation for that path |
| Set concrete value | Checked type conversion followed by assignment |
| Set NULL | Explicit null operation/value |
| Remove field | Explicit removal mapped to NONE |
| Merge nested object | Named merge operation with defined child semantics |
| Replace collection | Explicit replacement, including empty collection |
| Append collection | Explicit append with order/duplicate policy |

The following original illustrative contract makes scalar assignment and removal explicit. It deliberately uses typed numeric alternatives rather than JSON-shaped `Value` doubles for exact integer/decimal data:

```proto
syntax = "proto3";
package example.patch.v1;

import "google/protobuf/empty.proto";
import "google/protobuf/struct.proto";

message ScalarAssignment {
  oneof value {
    string text = 1;
    int64 signed_integer = 2;
    string exact_decimal = 3;
    bool flag = 4;
    bytes binary = 5;
    google.protobuf.NullValue explicit_null = 6;
  }
}

message FieldOperation {
  string path = 1;
  oneof operation {
    ScalarAssignment assign = 2;
    google.protobuf.Empty remove = 3;
  }
}

message RecordPatch {
  string record_key = 1;
  repeated FieldOperation operations = 2;
}
```

An adapter must reject an empty assignment alternative, validate decimal syntax/range, authorize each path, and define duplicate-path handling and operation order. Paths must resolve against approved schema metadata; do not interpolate an arbitrary path string into query text. Arrays and object merge can be added as separately specified operations rather than hidden behind an ambiguous assign.

### A10.2. Validation has three owners

Message validation covers the received object. Native field/assertion/index validation covers the stored representation. Transaction coordination covers relationships and invariants involving concurrent state. For example, a message validator can require two nonempty identifiers, while native relation enforcement and transaction logic determine whether the referenced endpoints exist and remain valid for the committed operation.

If a descriptor validator is unavailable in the selected Rust/WASI environment, enforce equivalent required rules in a supported component or use an external validation service boundary. Do not silently accept annotations that no runtime executes. Validation error reporting should identify the schema and mapping revision so consumers can distinguish malformed data from an unsupported integration contract.

### A10.3. Buf checks different failure classes

| Category | Compatibility focus |
| --- | --- |
| `FILE` | Generated source compatibility at file level |
| `PACKAGE` | Generated source compatibility at package level |
| `WIRE_JSON` | Binary wire plus ProtoJSON compatibility |
| `WIRE` | Binary wire compatibility |

Choose categories for actual consumers. A rename can preserve binary tags and still break JSON clients or field-name-based mapping. A wire-compatible enum addition can violate a closed database assertion. A changed mapping revision can redirect identity without changing the `.proto` at all. ([Buf breaking rules](https://buf.build/docs/breaking/rules/))

Thus schema CI should pair message compatibility checks with mapping review and representative adapter fixtures. Database DDL evolution, backfills, retained historical descriptors, and consumer rollout order remain independent work. Passing a wire check cannot prove that an old importer tolerates unknown fields when its implementation explicitly rejects them.

### A10.4. Replay identities and acknowledgement

| Identity | What it identifies |
| --- | --- |
| Native record ID | The entity or relation record |
| Database changefeed cursor | Position in retained source mutations |
| Mutation ordinal | A stable member within a grouped source change |
| Domain event ID | An externally meaningful occurrence |
| Message full type name | The selected message definition |
| Descriptor digest/revision | Exact schema bundle |
| Mapping revision | Interpretation into native/domain state |
| Kafka topic/partition/offset | Broker source position |
| Consumer acknowledgement | Processed external delivery progress |

Keep the cursor opaque where possible; implement the documented inclusive-start advancement once in the source adapter rather than ask every consumer to invent arithmetic. The 3.3 mixed-table scan/filter issue also makes a database-wide cursor/export path worth considering when it fits the authority and data-exposure model. A filtered empty result cannot alone justify skipping ahead.

Publication has a crash window: a broker can acknowledge an event before the exporter records progress. Retrying then republishes the event. Recording progress first reverses the risk and can lose delivery. Stable IDs, deduplication, or a stronger explicitly implemented coordination mechanism are required. One database versionstamp can cover several mutations, so using the cursor alone as every event's ID can incorrectly collapse distinct changes.

Retention expiry needs a recovery procedure: obtain a suitable snapshot/state reconstruction, establish a coherent new incremental boundary, and resume. Protobuf descriptors help interpret payloads but do not reconstruct discarded source history or carry every database schema change automatically. ([Changefeed documentation](https://surrealdb.com/docs/learn/querying/real-time/changefeeds))

## A11. Qualification boundaries for assembled integrations

| Combination | Established primitive | What the assembled integration must establish |
| --- | --- | --- |
| Native Value plus Protobuf | Public fallible codec | Supported types and strict input validation policy |
| Generated domain messages plus SurrealValue | Conversion trait and derive | Presence, range, identity, and inverse mapping |
| Descriptors plus field DDL | Machine-readable schema and structural kinds | Mapping rules, assertions, migration/backfill policy |
| Surrealism plus decoder/reflection | ABI 2 module execution | Actual WASI build, resource bounds, retained-state behavior |
| Module plus outbound transport | Capability-controlled host boundary | Specific guest transport compatibility and retry effects |
| Custom API plus writes | Raw bytes and handler machinery | 3.3 authority and failure/rollback behavior |
| KV history plus old messages | Retained historical byte values | Descriptor retention and durability settings |
| Borrowed decoder plus storage scans | Separate borrowing/buffer APIs | Actual ownership lifetime and measured allocation benefit |
| Lightweight topology plus events | Native traversable pair connections | Explicit durable event representation |
| Maintained view plus typed response | Native incremental projection | Dependency freshness and stable identity |
| Kafka tool plus 3.3 server | Source-level v3 SDK integration | Exact server/schema/framing compatibility |
| Changefeed plus broker | Retained source mutations | Cursor correctness, acknowledgement, deduplication, recovery |

No server attachment, module compilation, Kafka exercise, custom endpoint execution, or performance benchmark was performed for this guide. The table identifies what is implemented and what an actual deployment must demonstrate, without promoting source compatibility into end-to-end qualification.

## A12. Architectural implication

SurrealDB 3.3.x provides more native Protobuf and graph machinery than a simple comparison with 4.0 might suggest. The useful integration boundary is therefore chosen by contract ownership: native database clients can reuse the published protocol, independent services can own domain messages, and archives can retain original payloads alongside queryable projections.

The remaining semantic work is specific and reviewable. Define identity and mutation intent, constrain numeric/temporal conversion, retain schema and mapping revisions, select the actual 3.3 execution/authority boundary, and coordinate delivery around durable progress. Those decisions determine interoperability and recovery; adding another codec or registry does not make them automatic.

## Appendix B. Source ledger and reproducibility

The server source baseline is tag **v3.3.5**, commit [5c3383fe1841e10f7fc67fdf41b6d0671cbc1b13](https://github.com/surrealdb/surrealdb/tree/5c3383fe1841e10f7fc67fdf41b6d0671cbc1b13). The feature-introduction comparison is **v3.3.0**, commit [238bfeb11f5725bebed370167656748df8067595](https://github.com/surrealdb/surrealdb/tree/238bfeb11f5725bebed370167656748df8067595). Tag links below are version-scoped source references; the commit hashes identify the exact observations. Rolling documentation is used with explicit version markers rather than as a substitute for the tagged implementation.

| Source group | Version or identity | Primary references |
| --- | --- | --- |
| Server releases | 3.3.0 and 3.3.5 | [3.3.0](https://github.com/surrealdb/surrealdb/releases/tag/v3.3.0), [3.3.5](https://github.com/surrealdb/surrealdb/releases/tag/v3.3.5) |
| Server dependency graph | v3.3.5 | [Manifest](https://github.com/surrealdb/surrealdb/blob/v3.3.5/Cargo.toml), [Lockfile](https://github.com/surrealdb/surrealdb/blob/v3.3.5/Cargo.lock) |
| Native values and codec | v3.3.5 | [Codec](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/types/src/proto/mod.rs), [Value conversion](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/types/src/proto/value.rs), [Record conversion](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/types/src/proto/record.rs) |
| Rust mapping | v3.3.5 | [SurrealValue](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/types/src/traits/surreal_value.rs), [Kind](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/types/src/kind/mod.rs) |
| Native protocol | v0.13.1 | [Value schema](https://github.com/surrealdb/surrealdb-protocol/blob/v0.13.1/surrealdb/protocol/v1/value.proto), [RPC schema](https://github.com/surrealdb/surrealdb-protocol/blob/v0.13.1/surrealdb/protocol/rpc/v1/rpc.proto), [Parity](https://github.com/surrealdb/surrealdb-protocol/blob/v0.13.1/parity.toml) |
| gRPC server and SDK | v3.3.5 | [Server](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/server/src/rpc/grpc.rs), [Route](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/server/src/ntw/grpc.rs), [Client configuration](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/src/opt/grpc.rs) |
| Integrated engine wrappers | v3.3.5 | [SurrealKV](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/kvs-surrealkv/src/lib.rs), [Memory/SurrealMX](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/kvs-mem/src/lib.rs) |
| Standalone engines | SurrealKV 0.21.4; SurrealMX 0.27.0 | [SurrealKV](https://github.com/surrealdb/surrealkv/blob/v0.21.4/README.md), [SurrealMX](https://github.com/surrealdb/surrealmx/blob/v0.27.0/README.md) |
| Graph behavior | v3.3.5 | [Lightweight tests](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/core/src/kvs/tests/lightweight_test.rs), [Inline cache tests](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/core/src/kvs/tests/inline_cache_test.rs), [Inline property tests](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/core/src/kvs/tests/inline_props_test.rs) |
| Surrealism | 0.6.5 in v3.3.5; ABI 2 | [Runtime controller](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealism/runtime/src/controller.rs), [Capabilities](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealism/runtime/src/capabilities.rs), [Host](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/core/src/surrealism/host.rs) |
| Custom APIs | v3.3.5 | [Request body](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/core/src/api/middleware/req.rs), [Invocation](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/core/src/api/invocation.rs) |
| Change processing | v3.3.5 | [Feed reader](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/core/src/cf/reader.rs), [Events](https://github.com/surrealdb/surrealdb/blob/v3.3.5/surrealdb/core/src/doc/event.rs) |
| Kafka connector | surreal-sync commit 30c66b1ba90b2b27706a810ebc7a3c2fc1580827 | [Guide](https://github.com/surrealdb/surreal-sync/blob/30c66b1ba90b2b27706a810ebc7a3c2fc1580827/docs/kafka.md), [Decoder](https://github.com/surrealdb/surreal-sync/blob/30c66b1ba90b2b27706a810ebc7a3c2fc1580827/crates/kafka/src/from_kafka/proto/decoder.rs) |
| Protobuf edition/compiler | v36.0 | [Release](https://github.com/protocolbuffers/protobuf/releases/tag/v36.0), [Version families](https://github.com/protocolbuffers/protobuf/blob/v36.0/src/google/protobuf/compiler/versions.h) |
| Prost and reflection | 0.14.4 and 0.16.5 application-tooling references | [Prost](https://docs.rs/prost/0.14.4/prost/), [prost-reflect](https://docs.rs/prost-reflect/0.16.5/prost_reflect/) |
| Buffa | v0.9.2, commit 1f5d80e2c8d31c147742d8ec826a3219c4fcd4a7 | [Tagged README](https://github.com/anthropics/buffa/blob/v0.9.2/README.md), [Guide](https://github.com/anthropics/buffa/blob/v0.9.2/docs/guide.md) |
| Governance | Version-qualified Protobuf sources plus rolling official guides | [Buf rules](https://buf.build/docs/breaking/rules/), [Validation rules](https://protovalidate.com/schemas/standard-rules/), [FieldMask v36.0](https://github.com/protocolbuffers/protobuf/blob/v36.0/src/google/protobuf/field_mask.proto) |

## Appendix C. Coverage against the supplied 4.0 guide

The mapping below identifies the equivalent subject matter, including subjects whose correct 3.3 treatment is an explicit version boundary. Main-guide numbers refer to this guide's sections; A-numbers refer to the advanced addendum.

### C1. Main guide coverage

| Supplied section | Equivalent coverage here |
| --- | --- |
| 1 Executive orientation and capability map | 1.1–1.3; corrected 3.3 feature and component baselines |
| 2.1 Schema, wire format, generated types | 2.1 |
| 2.2 Editions, presence, evolution | 2.2 and A9.4 |
| 2.3 Reflection/descriptors | 2.3 and A9.1–A9.2 |
| 2.4 Well-known types and polymorphism | 2.4 and A2.4 |
| 2.5 ProtoJSON and information loss | 2.5 and A9.3 |
| 2.6 Validation/governance | 2.6 and A10.2–A10.3 |
| 3.1 Database architecture | 3.1 and 5.1 |
| 3.2 Network/query protocols | 3.2 and A4.1 |
| 4.1 Native gRPC | 4.1 |
| 4.2 Operational constraints | 4.2–4.3 |
| 4.3 Database versus application gRPC | 4.3 and 11.1 |
| 5.1 Representation strategies | 5.1 |
| 5.2 Type mapping | 5.2 and A1.2–A3.3 |
| 5.3 Absent/null/default | 5.3 and A10.1 |
| 5.4 Typed graph/event example | 5.4 |
| 6.1 Storage engine guarantees | 6.1 and A7 |
| 6.2 MVCC/history/message versions | 6.2 |
| 6.3 Isolation and invariants | 6.3 |
| 7.1 Graph identity | 7.1 and A2.2 |
| 7.2 Lightweight relations | 7.2 and A8.1 |
| 7.3 Adjacency/reverse references | 7.3; MATERIALIZE INTO boundary retained |
| 7.4 SurrealQL and ISO GQL | 7.4 |
| 7.5 Graph tracing | 7.5; explicitly 4.0 |
| 8.1 Extension boundary | 8.1 and A5 |
| 8.2 Invocation execution model | 8.2 and A5.1; actual 3.3 pool semantics |
| 8.3 Transaction locality | 8.3 and A5.2 |
| 8.4 Migration limitation | 8.4; no 3.3 MIGRATE EXISTING ROWS |
| 9.1 Change mechanisms | 9.1 |
| 9.2 Changefeed projection | 9.2 and A10.4 |
| 9.3 Kafka integration | 9.3; pinned connector and extra decoder limits |
| 10.1 Rust tools | 10.1 and A3 |
| 10.2 Buffa | 10.2 and A9.3–A9.4 |
| 10.3 Official Rust versus independent runtimes | 10.3 |
| 10.4 Python | 10.4 |
| 10.5 Buf and inspection | 10.5 and A10.3 |
| 11.1 Native mechanisms | 11.1 |
| 11.2 Performance | 11.2 and A7 |
| 11.3 Security/correctness | 11.3, A5.2, and A6.3 |
| 11.4 Version/maturity | 1.2–1.3, 11.4, and A11 |
| 12 Synthesis | 12 and A12 |

### C2. Advanced addendum coverage

| Supplied addendum section | Equivalent coverage here |
| --- | --- |
| 1.1 Native codec architecture/versions | A1.1 and 1.3 |
| 1.2 Conversion fidelity/asymmetry | A1.2 and A2; actual 3.3 failures rather than 4.0 degradation |
| 1.3 Serialized-data implications | A1.3 |
| 2.1 Rich schema | A2.1 |
| 2.2 Structured identity | A2.2 |
| 2.3 Object/missing-value rules | A2.3; duplicate-key implementation discrepancy added |
| 2.4 Temporal representation | A2.4 |
| 3.1 SurrealValue | A3.1 |
| 3.2 Adapter alternatives | A3.2 |
| 3.3 Database type descriptions | A3.1 and A3.3 |
| 3.4 Named reusable types | A3.3; explicitly 4.0, with 3.3 alternatives |
| 4.1 CBOR v2 | A4.1; version boundary |
| 4.2 Binary representation selection | A4.1 |
| 4.3 Protobuf/FlatBuffers parity | A4.2 |
| 4.4 Conformance facilities | A4.3; version-matched 0.13.1 facilities |
| 5.1 ABI/lifecycle | A5.1 and 8.2 |
| 5.2 Transaction integration | 8.3; 3.3 host callbacks and writeable metadata |
| 5.3 Capabilities/network | A5.2 |
| 5.4 Bindings/configuration | A5.1–A5.3; no ABI 5 bindings, descriptors/cache alternatives |
| 5.5 Distribution/integrity | A5.4 |
| 6.1 Raw binary HTTP bodies | A6.1; octet-stream requirement |
| 6.2 Limits/decoding | A6.2 |
| 6.3 Execution authority | A6.3–A6.4; pre-4.0 permissions and failure behavior |
| 6.4 Architectural value | A6.4 and 11.1 |
| 7.1 SurrealKV, value log, durability | A7.1; actual 0.21.4 APIs and guarantees |
| 7.2 SurrealMX, access, persistence | A7.2; actual 0.27.0 APIs and integration |
| 7.3 Storage versus graph semantics | A7.3 and 5.1 |
| 8.1 Lightweight change visibility | A8.1 |
| 8.2 View dependencies and IDs | A8.2; important 3.3 projected-ID difference |
| 8.3 GraphQL | A8.3 |
| 9.1 Dynamic schemas | A9.1 |
| 9.2 Options/metadata | A9.2 |
| 9.3 Unknown fields | A9.3; generated Prost versus reflection/importer distinctions |
| 9.4 Edition 2026 compatibility | A9.4 and 2.2 |
| 10.1 FieldMask/update semantics | A10.1 |
| 10.2 Validation ownership | A10.2 and 2.6 |
| 10.3 Buf compatibility categories | A10.3 |
| 10.4 Replay identities | A10.4 and 9.2; scan-before-filter limitation added |
| 11 Unresolved combinations | A11 |
| 12 Synthesis | A12 |
