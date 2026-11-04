# Protocol Buffers and the SurrealDB 4.0 ecosystem

Technical capability reference · Research date: November 3, 2026 · Scope: Protobuf, Rust integrations, SurrealDB 4.0, storage engines, graph querying, and extensibility

## 1. Executive orientation

Protocol Buffers (Protobuf) is most valuable in the SurrealDB ecosystem as a typed data-contract, serialization, and interoperability layer—not as a replacement for SurrealDB's native database representation or query mechanisms.

SurrealDB 4.0 was announced as generally available on November 3, 2026. It introduces a rebuilt query engine, expanded PostgreSQL/MySQL compatibility, additional graph functionality, and significant changes to storage and extensibility. Its integration boundaries create several distinct opportunities for Protobuf.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB+2



The most important distinctions are:

- Protobuf describes structured messages and service interfaces. It supports generated code, runtime reflection, compatibility evolution, and efficient binary interchange.
- SurrealDB provides the stateful database semantics. These include records, relationships, indexes, graph traversal, query execution, transactions, and permissions.
- SurrealKV and SurrealMX implement storage semantics. They are responsible for the underlying key-value storage and transactional behavior, not Protobuf message handling.
- Surrealism provides an in-database execution boundary. Rust functions can execute in sandboxed WebAssembly modules, creating opportunities for Protobuf-aware extensions.
- gRPC provides a direct Protobuf integration point. SurrealDB exposes its RPC service over gRPC, but its database values and query results retain their own native representation and semantics.

### Capability and integration map

| Area                          | Protobuf contribution                                      | SurrealDB-native responsibility                       |
| ----------------------------- | ---------------------------------------------------------- | ----------------------------------------------------- |
| Application data contracts    | Typed schemas, field presence, enums, message evolution    | Record schemas, validation, permissions               |
| gRPC communication            | Service definitions, generated clients, binary framing     | RPC methods, authentication, query execution          |
| Rust/Python interoperability  | Generated types and common wire contracts                  | SDK values and database-specific types                |
| SurrealKV / SurrealMX         | Optional serialized binary payloads                        | Persistence, transactions, versioning                 |
| Graph querying                | Typed nodes, edges, paths, traversal results               | Graph relationships, traversal, indexing              |
| Surrealism                    | Decoding, encoding, validation and transformation          | WASM execution, transaction access, resource controls |
| Change feeds and live queries | Typed event envelopes and schema evolution                 | Change detection, subscription and delivery           |
| Schema governance             | Descriptor sets, compatibility checking, schema registries | Database schema definitions and migrations            |
| Data interchange              | Message-oriented transport contracts                       | Native CBOR, shaped results, SQL and GQL              |

This division is fundamental: a Protobuf schema can describe an external representation of a database entity, but it does not automatically become the database schema, transaction model, or graph ontology.

### Terminology clarification

The documented in-memory storage engine is SurrealMX, rather than SurrealMV. I will cover SurrealMX and the separate concepts of multiversion concurrency control (MVCC) and temporal versioning. SurrealDB 4.0 also introduces SurrealDS as a distributed-storage option.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



The reference below concentrates on how these components compose, including less obvious opportunities and semantic limitations.

## 2. Protocol Buffers: capabilities and architectural mechanisms

### 2.1. Schema language, wire format, and generated types

Protobuf comprises three separable mechanisms:

1. An interface definition language (IDL): `.proto` files define messages, fields, enums, services, and metadata.
2. A binary serialization format: field-numbered, type-tagged values are encoded without repeatedly transmitting field names.
3. A compiler and runtime ecosystem: code generators produce language-native representations, serializers, parsers, and service bindings.

The wire format uses field numbers and wire types, with variable-length integers, fixed-width numerical encodings, and length-delimited strings, bytes, nested messages, and packed fields. The schema is not embedded automatically in each message. Consequently, receiving a Protobuf payload does not establish what message type it represents; the decoder must already know or retrieve the appropriate descriptor.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://protobuf.dev\&sz=32)

Protocol Buffers Documentation



This separation makes Protobuf useful when a stable semantic contract needs multiple implementations, but it also explains why Protobuf alone cannot interpret an arbitrary SurrealDB record or dynamically discovered graph.

### 2.2. Editions, field presence, and schema evolution

The latest released Protobuf language edition is Edition 2026, released August 20, 2026, with `protoc` 36.0 as its minimum compiler version. Editions 2023 and 2024, plus `proto2` and `proto3` syntax, remain relevant for library compatibility. An edition number is not a Protobuf runtime version, and generator support must be verified independently.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://protobuf.dev\&sz=32)

Protocol Buffers Documentation



Editions provide configurable language features rather than requiring a wholesale change to the binary format. Important controls include field presence, open versus closed enums, repeated-field encoding, UTF-8 validation, symbol visibility, and language-specific generation options. Edition 2026 additionally changes naming and schema-size enforcement defaults.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://protobuf.dev\&sz=32)

Protocol Buffers Documentation



Field presence is particularly consequential for database integration.

| Protobuf declaration                                 | Semantic distinction                                              |
| ---------------------------------------------------- | ----------------------------------------------------------------- |
| `string name = 1` in conventional `proto3`           | Default empty string generally cannot be distinguished from unset |
| `optional string name = 1`                           | Explicitly tracks whether the scalar was supplied                 |
| Editions scalar field with default explicit presence | Tracks unset versus set-to-default                                |
| `oneof`                                              | Tracks which alternative is selected                              |
| `repeated` / `map`                                   | Does not track presence independently of an empty collection      |
| Nested message                                       | Has explicit presence                                             |

This matters because a database update can distinguish leave unchanged, set to zero, clear a field, and set a nullable value. A Protobuf scalar without explicit presence can collapse two of those states.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://protobuf.dev\&sz=32)

Protocol Buffers Documentation



For schema evolution, Protobuf's principal mechanism is stable field identity, not database migration. Fields can often be added compatibly; removed field numbers and names should be reserved; changes to types, cardinality, oneofs, and enum semantics require compatibility review. Binary compatibility, source-code compatibility, application semantics, and ProtoJSON compatibility are separate questions.

### 2.3. Reflection and descriptor-driven operation

An important capability beyond generated structs is Protobuf's descriptor system.

A compiled `FileDescriptorSet` describes messages, fields, enums, dependencies, services, and custom options. Reflection-capable runtimes can use these descriptors to:

- Decode messages whose concrete types are unknown at application build time.
- Inspect field names, numbers, types, and nested structure.
- Construct dynamic messages and convert between representations.
- Implement generic validation, inspection, routing, and schema transformation.
- Attach schema identities to stored messages or data-processing envelopes.

For a heterogeneous SurrealDB database, this makes a descriptor registry potentially more useful than generating a separate Rust type for every database table. It can provide a common external contract catalog while allowing runtime-discovered message types.

However, descriptors express message structure, not arbitrary database constraints. They do not automatically express record existence, graph endpoint validity, transaction isolation, row-level permissions, uniqueness indexes, or cross-record invariants.

### 2.4. Well-known types and controlled polymorphism

Several Protobuf well-known types are especially relevant to multimodel databases.

| Type or mechanism              | Capability                                          | Integration consideration                                        |
| ------------------------------ | --------------------------------------------------- | ---------------------------------------------------------------- |
| `google.protobuf.Timestamp`    | Timestamp with seconds/nanoseconds                  | Preserve timezone/precision semantics across conversions         |
| `Duration`                     | Signed duration                                     | Not interchangeable with a calendar-relative duration            |
| `Any`                          | Carries a typed serialized message                  | Requires a type-resolution policy and trusted descriptors        |
| `Struct`, `Value`, `ListValue` | Dynamically shaped JSON-like data                   | Weaker typing; number and null semantics require care            |
| `FieldMask`                    | Identifies fields for projection or partial updates | Requires explicit database update semantics                      |
| `oneof`                        | Tagged alternatives                                 | Useful for command/event unions and heterogeneous result types   |
| Custom options                 | Descriptor-attached application metadata            | Can describe mapping policy without placing it in payload fields |

The `Any` and `Struct` families are not substitutes for a schema registry or SurrealDB's native record model. They are mechanisms for intentionally relaxing static typing at an interchange boundary.

### 2.5. ProtoJSON, binary compatibility, and loss of information

Protobuf JSON is not ordinary JSON serialization of generated structs. ProtoJSON defines a specific mapping: 64-bit integers commonly appear as decimal strings, `bytes` as base64, enums by name, and timestamps in prescribed textual forms. A generic Rust Serde representation is not automatically ProtoJSON-compatible.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://protobuf.dev\&sz=32)

Protocol Buffers Documentation



Three consequences matter when combining Protobuf with SurrealDB's JSON/CBOR interfaces:

- Unknown fields: binary Protobuf can preserve unknown fields in runtimes that implement that behavior, but ProtoJSON does not preserve unknown fields through a round-trip.
- Absent versus default: a database projection or JSON transformation can erase field-presence information that an original Protobuf message carried.
- Numeric and binary fidelity: generic JSON conversion can alter integer representation, typed numerics, and byte handling unless the mapping is deliberate.

Protobuf also does not guarantee canonical binary serialization. Even deterministic serialization should not be assumed identical across implementations, schema versions, or processing histories. Bytewise hashes, signatures, deduplication, and content-addressed storage need an explicit canonicalization or immutable-byte policy.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://protobuf.dev\&sz=32)

Protocol Buffers Documentation



### 2.6. Validation and schema governance

Protobuf syntax establishes structural validity; it does not generally enforce business invariants. Additional mechanisms include custom field/message options, descriptor-driven validators, Buf's compatibility/linting facilities, and validation specifications such as CEL-based Protovalidate.

These capabilities can complement SurrealDB's `DEFINE FIELD`, `ASSERT`, type definitions, and permissions, but they operate at different boundaries:

- Protobuf validation checks an interchange object.
- SurrealDB schema validation checks the database representation.
- Transactional validation may depend on other records and concurrent writes.

None of these automatically replaces the others.

Primary references:

`https://protobuf.dev/programming-guides/encoding/`

`https://protobuf.dev/programming-guides/field_presence/`

`https://protobuf.dev/editions/features/`

`https://protobuf.dev/programming-guides/json/`

`https://protobuf.dev/programming-guides/serialization-not-canonical/`

`https://protobuf.dev/support/version-support/`

## 3. SurrealDB 4.0: where Protobuf fits within the database architecture

### 3.1. The database is not a Protobuf object store

SurrealDB's conceptual model is built around records, record identifiers, nested fields, typed values, graph relationships, indexes, and declarative queries. Its query engine operates over transactional key/value storage, with the same broad query model available across different supported storage engines. Protobuf is not the internal record schema or the defining storage format in this architecture.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



A Protobuf message could be:

- Decoded into fields and relationships that SurrealDB can query natively.
- Stored as opaque binary data for later exact-message retrieval.
- Stored alongside selectively extracted fields and graph edges.
- Produced from a query projection or change event for an external consumer.

These are distinct representation strategies, not interchangeable encoding optimizations.

### 3.2. SurrealDB's native network and query protocols

SurrealDB 4.0 has a broader interoperability surface than gRPC alone.

| Interface                 | Primary purpose                                    | Relationship to Protobuf                                                             |
| ------------------------- | -------------------------------------------------- | ------------------------------------------------------------------------------------ |
| Native RPC over WebSocket | Sessions, queries, live notifications, streaming   | Uses SurrealDB RPC/CBOR representations rather than a user-defined `.proto` contract |
| Native RPC over HTTP      | Request/response database operations               | JSON/CBOR and supported native result formats                                        |
| Native RPC over gRPC      | Typed RPC transport and remote incremental results | Direct integration: service defined with Protobuf                                    |
| Shaped results            | Compact delivery of repeated row structures        | Separate SurrealDB encoding, not Protobuf                                            |
| PostgreSQL wire/SQL       | SQL-oriented interoperability                      | PostgreSQL types and protocol, not Protobuf                                          |
| MySQL wire/SQL            | MySQL-oriented interoperability                    | MySQL protocol, not Protobuf                                                         |
| SurrealQL                 | Native multimodel querying                         | Expresses database operations, independent of message encoding                       |
| ISO GQL support           | Declarative graph pattern matching                 | Can produce results later mapped into Protobuf messages                              |

The most important distinction is that gRPC and Protobuf are not synonymous with SurrealDB's data model. gRPC defines how a client communicates with the service; the service's protocol definitions specify which SurrealDB operations and values can be exchanged.

SurrealDB 4.0 also introduces shaped results, in which homogeneous result rows share one structural description rather than repeating field names and type tags. The surrounding representation is CBOR, and the shaped row block has its own encoding. This is a native alternative worth considering before introducing an extra database-result-to-Protobuf conversion solely to reduce repeated column metadata.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Primary reference: `https://surrealdb.com/docs/reference/rest-api/rpc-protocol`

## 4. Direct gRPC integration with SurrealDB

### 4.1. A native Protobuf service already exists

Since SurrealDB 3.3, its RPC service has also been exposed over gRPC. The service is named:

`surrealdb.protocol.rpc.v1.SurrealDBService`

Its definitions are maintained in the upstream `surrealdb-protocol` repository. This is an existing database protocol, not a service that must be designed from scratch. The Rust SDK provides the `protocol-grpc` feature and accepts `grpc://` and `grpcs://` connections. gRPC uses the same listener, port, TLS configuration, authentication middleware, and RPC capability restrictions as the other RPC transports.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: `https://github.com/surrealdb/surrealdb-protocol`

### 4.2. Less obvious capabilities and operational constraints

| Mechanism            | Documented behavior                                                | Consequence                                                                 |
| -------------------- | ------------------------------------------------------------------ | --------------------------------------------------------------------------- |
| HTTP/2 transport     | Cleartext HTTP/2 preface or TLS ALPN negotiation                   | No separate gRPC listener required                                          |
| Capability handshake | `GetCapabilities` advertises server limits                         | Client can discover constraints before submitting large messages            |
| Incremental results  | Query responses can be split across gRPC messages                  | Large result sets need not be buffered as one response                      |
| Compression          | gzip and zstd negotiation                                          | Useful for repetitive or structured payloads; effectiveness depends on data |
| Attached sessions    | Session and notification buffers have separate limits              | Subscription lifecycle and backpressure matter                              |
| Message size         | Default server limit is 4 MiB per message                          | A single oversized record or request can still fail                         |
| Reflection           | No gRPC server reflection                                          | Inspection tools require local `.proto` definitions                         |
| Health service       | Proprietary service `Health`, not standard `grpc.health.v1.Health` | Standard gRPC health probes are not directly compatible                     |
| Security             | Same middleware and capability gates as native RPC                 | gRPC does not bypass database permissions                                   |

The server's 4 MiB default is governed by `SURREAL_GRPC_MAX_MESSAGE_SIZE`. Large query results can be split into messages, but individual records and notifications still need to fit. That is relevant if Protobuf is also used to carry large opaque blobs within a record.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



The Rust SDK's `.stream_items()` can exploit remote incremental results over gRPC. This is an important distinction from simply serializing a complete query response into one large Protobuf message.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 4.3. Database gRPC versus application gRPC

There are two different integration models.

Direct database RPC

A client uses SurrealDB's published gRPC protocol to issue queries and operations. SurrealDB owns the wire contract.

Application-specific gRPC

A service defines business messages—for example, `RecordChanged`, `GraphPath`, or `EntitySnapshot`—and uses SurrealDB internally. The application owns the external contract and transforms database values into those messages.

The second model permits stronger domain-level contracts and hides database-specific types, but introduces a transformation boundary. It should not be described as SurrealDB natively storing those Protobuf messages.

An additional limitation is that SurrealDB's 4.0 graph query trace option is not available over gRPC. Applications needing those particular diagnostics must use a supported RPC path; gRPC streaming is not a superset of every WebSocket RPC capability.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Primary references:

`https://surrealdb.com/docs/reference/rest-api/rpc-protocol`

`https://surrealdb.com/docs/reference/rust/methods/connect`

`https://github.com/surrealdb/surrealdb-protocol`

## 5. Representing Protobuf data in SurrealDB records

### 5.1. Three representation strategies

Native

Decoded, schema-aware records

Protobuf messages are decoded into SurrealDB fields, typed references, and relation records. This exposes the contents to SurrealQL, GQL, secondary indexes, permissions, and graph traversal. The conversion must reconcile type and presence semantics.

Opaque

Serialized Protobuf payloads

A record holds the original encoded message as `bytes`, accompanied by a message type and schema identifier. This preserves the original payload when it is stored without rewriting it, but its nested fields are not independently queryable through ordinary database indexes.

Hybrid

Binary payload plus queryable projections

A record contains the serialized message alongside selected native fields, references, and graph edges. This combines exact-byte retention with native indexing and traversal, at the cost of synchronizing the projections with the payload.

SurrealDB has a native `bytes` type, so storing an opaque message does not require converting the binary payload to a base64 string first. Conversely, native records expose functionality that remains unavailable to the query engine while the same information is hidden inside an opaque payload.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 5.2. Type mapping is not a trivial field copy

| Protobuf concept                     | SurrealDB mapping consideration                                                                           |
| ------------------------------------ | --------------------------------------------------------------------------------------------------------- |
| `int32`, `int64`, `uint32`, `uint64` | Check exact SurrealDB integer type, signedness, width, and bounds                                         |
| `float`, `double`                    | Consider numeric precision and nonfinite values                                                           |
| `string`                             | Native string, subject to validation and normalization policies                                           |
| `bytes`                              | Native `bytes` field                                                                                      |
| `enum`                               | SurrealDB named types or constrained string/integer; preserve unknown-value behavior                      |
| Nested message                       | Nested object or separate record                                                                          |
| `repeated`                           | Array, or related records when independent identity/querying is needed                                    |
| `map<K,V>`                           | Native object only when key representation is compatible; otherwise entries as records                    |
| `Timestamp`                          | Native datetime with an explicit precision and normalization policy                                       |
| Decimal domain value                 | Explicit representation, such as scaled integer or decimal string, not a built-in Protobuf decimal scalar |
| `Any`                                | Type-discriminated payload, often with descriptor metadata                                                |
| `FieldMask`                          | Explicit field-selection or update operation; not automatically a SurrealQL patch                         |
| Message identity                     | Protobuf has no universal equivalent of a SurrealDB record ID                                             |

SurrealDB 4.0 extends its typed numeric and schema capabilities, making faithful numeric projection more feasible, but the complete Protobuf-to-SurrealDB mapping remains an application or integration concern.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 5.3. The critical absent/null/default distinction

SurrealDB distinguishes:

- `NONE`: a value or field is absent; assigning it to a record field removes that field.
- `NULL`: the field exists and explicitly holds null.
- A concrete value: including zero, `false`, or an empty string.

This is a more expressive state space than a conventional non-optional proto3 scalar.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



For example, a reliable partial-update contract may need four operations: retain the existing value, set a concrete value, set it to null, or remove the field. Neither `optional string` nor an ordinary Protobuf `FieldMask` establishes all four meanings automatically. A discriminated update message or explicit operation semantics can express the distinction.

That mapping is important for SurrealDB 4.0's schema rules. `option<T>` allows omission; nullable types permit actual `NULL`; and narrowing an existing `SCHEMAFULL` field's type can require `MIGRATE EXISTING ROWS`. A Protobuf-compatible schema change does not therefore imply a compatible SurrealDB schema change.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 5.4. Illustrative typed graph/event contract

The following is an application-level example, not an upstream SurrealDB protocol definition. It shows how a Protobuf contract can retain record identities and distinguish graph mutation types without attempting to replace SurrealQL.

```

syntax = "proto3";
package example.graph.v1;

import "google/protobuf/timestamp.proto";

message RecordRef {
  string table = 1;
  string id = 2;
}

message EdgeChanged {
  string edge_table = 1;
  RecordRef source = 2;
  RecordRef target = 3;
  optional string edge_id = 4;
}

message RecordChanged {
  RecordRef record = 1;
  bytes encoded_payload = 2;
  string payload_message_type = 3;
}

message ChangeEnvelope {
  string schema_revision = 1;
  bytes source_cursor = 2;
  google.protobuf.Timestamp emitted_at = 3;

  oneof change {
    RecordChanged upsert = 10;
    RecordRef deleted = 11;
    EdgeChanged relation_changed = 12;
  }
}

```

The useful distinction is between a Protobuf message describing a graph mutation and the database operation that actually creates or changes a graph relation. This contract could be used by an external service or connector, while SurrealDB continues to enforce its own relationship definitions and transactional rules.

Primary references:

`https://surrealdb.com/docs/reference/query-language/language-primitives/data-types/bytes`

`https://surrealdb.com/docs/reference/query-language/language-primitives/data-types/none-and-null`

`https://surrealdb.com/docs/reference/query-language/statements/define/field`

## 6. SurrealKV, SurrealMX, SurrealDS, and multiversion state

### 6.1. Storage engines provide guarantees independently of Protobuf

| Engine        | SurrealDB 4.0 role                                            | Relevant properties                                                                                                    |
| ------------- | ------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| SurrealKV 1.0 | Primary durable single-node/embedded on-disk storage          | LSM-based key-value engine, stable 1.0 disk format, optional temporal/versioned reads                                  |
| SurrealMX     | In-memory storage                                             | Transactional in-memory engine; supports serializable isolation, but not versioned historical queries                  |
| SurrealDS     | Distributed storage in supported Enterprise/Cloud deployments | Object-backed distributed storage, separate compute/storage scaling, serializable transactions and published snapshots |
| TiKV          | Separately built backend where supported                      | Distributed transactional key/value backend with its own deployment requirements                                       |

The storage engine implements transactional reads and writes of keys and key ranges. It need not know that an application's `bytes` field contains Protobuf. Nor does using Protobuf provide a shortcut around SurrealDB's transactional storage layer.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



### 6.2. MVCC, temporal versioning, and message versioning are different

There are three independent dimensions:

Database transaction versions govern which committed state a query observes and which concurrent operations conflict.

Temporal history governs whether older database states can be explicitly queried. SurrealKV supports this with versioned operation configured; SurrealMX does not provide the same historical query capability.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Protobuf schema versions govern how payload fields are interpreted and evolved over time. They do not determine which database state is visible or whether a historical record can be retrieved.

A system preserving historical Protobuf messages must distinguish all three. A historical SurrealDB read could return a record with a payload produced under an older `.proto` descriptor; a new decoder may be binary-compatible but not semantically equivalent to the original application interpretation.

### 6.3. Transaction isolation and invariants

SurrealDB 4.0 documents snapshot isolation with write-conflict detection across its supported engines. `SERIALIZABLE` is available on SurrealMX and SurrealDS, while SurrealKV and TiKV do not support that isolation level. `SELECT ... FOR UPDATE` can add conflict checks for specific records read during a write transaction.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



For a Protobuf-to-record transformation, this means:

- Decoding a well-formed message does not ensure the eventual write is transactionally valid.
- Graph endpoint existence, uniqueness, and multi-record invariants still need database enforcement or transactional coordination.
- An external event consumer's retry and ordering semantics remain distinct from SurrealDB's transaction isolation.
- A transaction committed successfully says nothing about whether its original Protobuf input has been durably acknowledged by an external message broker.

In particular, changing from SurrealKV to SurrealMX does not alter the meaning of a Protobuf descriptor, but can alter isolation capabilities, durability, and historical-query availability.

Primary references:

`https://surrealdb.com/docs/learn/data-models/architecture`

`https://surrealdb.com/docs/manage/self-hosted/deployment-models`

`https://surrealdb.com/docs/learn/querying/concepts-and-guides/transactions`

## 7. Graph querying: Protobuf contracts versus native graph machinery

### 7.1. Graph identity and relationship semantics

SurrealDB offers three importantly different ways to represent connections:

| Mechanism                | Native semantics                                                            | Protobuf interaction                                                  |
| ------------------------ | --------------------------------------------------------------------------- | --------------------------------------------------------------------- |
| Record link              | A field contains another record's ID                                        | Map an identifier into a typed reference or message field             |
| Tracked record reference | A link with inverse-reference support and deletion policies                 | Model both sides externally without manually maintaining inverse data |
| Graph relation           | An edge associates `in` and `out` endpoints, optionally carrying properties | Map an edge message to a relation record through database operations  |

A Protobuf message containing `source_id` and `target_id` is not itself a SurrealDB graph edge. The edge becomes traversable after the corresponding relationship is represented in the database's native graph structure.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



SurrealDB's `TYPE RELATION ... ENFORCED` can require endpoints to exist, while record reference fields can use existence assertions and deletion policies. These checks address data integrity that Protobuf schema validation cannot enforce independently.

### 7.2. Lightweight relations eliminate unnecessary edge payloads

An easily overlooked native feature is `LIGHTWEIGHT` relation tables, introduced in SurrealDB 3.3 and available in 4.0.

A lightweight edge stores the connection without an independent property-bearing edge record. Its identity is derived from the endpoint pair, yielding naturally idempotent `RELATE` behavior for the same directed connection. The database synthesizes an edge-shaped result when queried.

This changes the role of Protobuf edge messages:

- A Protobuf message may still communicate that a relation was created or removed.
- The database need not store the entire message as an edge record.
- Edge properties, per-edge events, indexes, changefeeds, and `LIVE SELECT` on the lightweight relation table are not supported, because there is no ordinary stored edge record to carry them.

Lightweight relations are therefore more than a storage optimization; they define a different semantic contract. They are unsuitable where each relationship occurrence needs its own identity, attributes, or independent lifecycle.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 7.3. Native adjacency and inverse-reference optimization

SurrealDB 4.0 includes several mechanisms that can eliminate bespoke graph materialization code.

`INLINE EDGES` and `INLINE REFERENCES` cache bounded adjacency or inverse-reference lists alongside records. This can reduce separate key-range scans during traversal, particularly for low-degree vertices. Limits are configured per table, with a cap of 256; high-degree vertices fall back to normal scanning. Updates to these caches participate in the same transaction as the graph changes. The engine also performs background adjacency-key folding.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



`MATERIALIZE INTO`, available since 4.0, maintains a field containing records that reference a target, updating it transactionally as the source reference changes. This turns some reverse-reference queries into ordinary field reads.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



`DEFINE TABLE ... AS SELECT` supports maintained, precomputed table views. These can materialize useful derived projections, although their dependency tracking is not equivalent to recomputing an arbitrary query whenever any related record changes.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



These facilities matter because Protobuf's strong typing can tempt implementations to maintain duplicate adjacency lists, inverse links, and denormalized graph summaries outside the database. SurrealDB has native mechanisms for several of those operations. Protobuf remains useful for the external representation of the results, not necessarily their internal maintenance.

### 7.4. SurrealQL and ISO GQL

SurrealQL provides arrow-based graph traversal, recursive paths, and the ability to combine record fields and relations in a query.

SurrealDB additionally supports a subset of ISO/IEC 39075 GQL through `POST /gql`, the `gql` RPC method, and supported SQL-wire dialect selection. This is distinct from GraphQL. Supported features include multiple graph patterns, optional matching, variable-length edges, path variables, `ANY SHORTEST` and `ALL SHORTEST` selectors, aggregation, and graph mutations. The implementation does not support the entire ISO standard or the full openCypher language.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



A Protobuf representation of a graph query result can reasonably take several forms:

- Record-oriented: nodes and edges as typed messages, with identifiers and properties.
- Path-oriented: ordered sequences of node references and traversed edge references.
- Projection-oriented: a specific query's output as a strongly typed response message.
- Subgraph-oriented: node and edge sets plus optional path membership, preserving shared identities.

A repeated list of nested node messages can duplicate the same node many times. An identity-oriented representation can instead send node records once and refer to them from edges and paths. This is a Protobuf modeling choice; the native database already maintains record identity.

### 7.5. Graph query tracing in SurrealDB 4.0

SurrealDB 4.0 also documents opt-in graph tracing for applicable `query` and `query_stream` RPC calls. Trace kinds include:

- `relations`: graph edges traversed.
- `references`: inverse references used.
- `links`: record links dereferenced.
- `variables`: top-level variable bindings.
- `subscriptions`: live queries initiated.

This can support graph visualization, explanation of retrieved relationships, and external inspection. However, it is a partial account of result-producing traversals, not a complete dependency graph or execution provenance system. Filtered paths, some conditions, aggregates, and other operations can be omitted; traces respect permissions and have size limits. The feature is not available through the native gRPC transport.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



A Protobuf trace message could normalize these results for another system, but Protobuf does not supply the missing execution information. The completeness of the underlying SurrealDB trace remains the limiting factor.

Primary references:

`https://surrealdb.com/docs/learn/data-models/graph/overview`

`https://surrealdb.com/docs/learn/data-models/graph/record-links-vs-graph-relations`

`https://surrealdb.com/docs/reference/query-language/statements/define/table`

`https://surrealdb.com/docs/reference/query-language/statements/define/field`

`https://surrealdb.com/docs/learn/querying/gql/overview`

## 8. Surrealism: Protobuf inside sandboxed Rust extensions

### 8.1. Native extension boundary

Surrealism is SurrealDB's extension framework for running Rust functionality inside the database through WebAssembly components.

Its documented model uses Rust crates packaged as `.surli` archives, targeting `wasm32-wasip2` and WASI Preview 2. Exported functions are declared through Surrealism's own macros and metadata. The database calls them through an explicit ABI rather than loading arbitrary Rust code into its process.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Potential Protobuf-oriented uses include:

- Decode a serialized payload supplied to a Surrealism function.
- Inspect message fields using generated types or an embedded descriptor set.
- Validate or normalize data before producing SurrealDB values.
- Encode native values into an externally defined Protobuf message.
- Translate typed graph or domain concepts into database fields.

These are architectural possibilities using Protobuf libraries inside an extension, not evidence of a built-in Protobuf-to-Surrealism adapter or an automatically available gRPC client within WASM.

### 8.2. The important SurrealDB 4.0 execution change

Surrealism's runtime behavior changed substantially in 4.0:

- Each invocation receives a fresh WASM instance.
- Instance memory is discarded after the invocation.
- Build-time `init` state can be snapshotted into the extension archive and reused as the starting image for each invocation.
- Database access occurs through a transaction handle valid only during the call.
- Extension writes can participate in the calling query's transaction.
- Concurrency, memory, and execution time are bounded by configurable controls.&#x20;

  [image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

  Database



For Protobuf, a useful implication is that static descriptor data may be a good candidate for the build-time initialization snapshot rather than being reconstructed from raw schema files for every call. Whether a particular `prost-reflect` configuration can efficiently be preinitialized and compiled for the required WASM component target must be validated against that crate's dependency and target support.

Runtime reflection and descriptor resolution have a cost. A small generated message decoder may be simpler and leaner than carrying a general descriptor registry into every extension instance.

### 8.3. Why transaction locality matters

When a conversion function runs within the database's transaction, it can potentially transform a payload and read or write relevant records without a separate network round-trip or another transaction boundary.

This can simplify certain forms of type conversion and validation. It does not imply that arbitrary external service calls, global caches, background workers, or persistent process state are available inside the sandbox.

The distinction is between:

- Pure decoding/transformation, which is a natural fit for generated Protobuf logic.
- Database-aware validation, which may need the Surrealism transaction handle.
- External transport and message delivery, which generally belongs in an appropriate external component unless the extension environment specifically exposes the needed capabilities.

### 8.4. Important migration limitation

SurrealDB 4.0's `MIGRATE EXISTING ROWS` supports built-in expressions, but its documented migration expressions cannot call a custom function, model, script, or extension. Consequently, a Protobuf decoder implemented in Surrealism cannot simply be inserted into that schema-migration clause. A separate migration process would be needed for transformations requiring extension execution.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



This is an important difference between an extension callable during database operations and an extension usable in every database schema-management operation.

Primary references:

`https://surrealdb.com/docs/learn/extensions/guides/extension-architecture`

`https://surrealdb.com/docs/reference/query-language/statements/define/field`

## 9. Protobuf, event processing, Kafka, and changefeeds

### 9.1. SurrealDB provides several different change mechanisms

SurrealDB has native mechanisms for reacting to data changes. Their purposes differ from Protobuf's role as an external event representation.

| Mechanism                                | Primary behavior                                                  | Protobuf applicability                                                      |
| ---------------------------------------- | ----------------------------------------------------------------- | --------------------------------------------------------------------------- |
| `LIVE SELECT`                            | Pushes notifications about matching record changes                | Transform notifications into typed external messages                        |
| `CHANGEFEED` / `SHOW CHANGES`            | Retains and replays database mutations within a configured window | Export changes with explicit cursor and schema metadata                     |
| `DEFINE EVENT`                           | Executes logic in response to database mutations                  | Prepare or maintain data that an external publisher subsequently serializes |
| Materialized table views                 | Maintain selected derived database data                           | Provide queryable projections from which typed messages can be constructed  |
| Kafka integration through `surreal-sync` | Consumes external Protobuf messages into records                  | Direct, documented Protobuf ingestion facility                              |

Live queries and changefeeds should not be treated as interchangeable event buses.

Live queries are subscription-oriented: they push committed updates and apply visibility rules, but their notifications are not guaranteed to have a strict global order across concurrent clients. They are also subject to connection and subscription lifecycle constraints.

Changefeeds are replay-oriented. They require explicit retention configuration and provide versionstamps for incremental processing. A consumer that falls behind the retention period must recover through another mechanism, potentially a full resynchronization.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



### 9.2. Changefeed-to-Protobuf projection

A useful general architecture is:

SurrealDB transaction

Record/graph mutations commit

SurrealDB CHANGEFEED

Mutation history + versionstamp

Projection / serialization boundary

Descriptor selection, field mapping, validation

Protobuf event envelope

External transport, consumers, archives

A changefeed reports database mutations, not automatically domain-level events. An application may need to collapse multiple low-level writes into a single business event or include transaction identifiers, schema revisions, and provenance metadata.

For replay, SurrealDB documents using a stored versionstamp to determine the next changefeed batch. Notably, its `SINCE` handling can include the supplied starting versionstamp, so cursor advancement must follow the documented rules. Versionstamps are monotonically increasing but should not be confused with an application-defined event sequence.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



A Protobuf envelope can preserve cursor information and message identity, but it does not automatically provide exactly-once delivery. End-to-end correctness still depends on coordinating database progress, external publication, consumer acknowledgements, and deduplication.

### 9.3. The direct integration: `surreal-sync` Kafka ingestion

A particularly relevant existing integration is `surreal-sync from kafka`.

The upstream Kafka-source documentation specifies that message payloads are Protobuf-encoded. The tool accepts a `.proto` schema and message type, decodes Kafka records, and converts them into SurrealDB upsert operations. The Kafka topic becomes the target table, and record identity can be derived from configured message keys or fields.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



The relevant interface includes:

```
surreal-sync from kafka \
  --proto-path ./schema.proto \
  --message-type Event \
  --brokers localhost:9092 \
  --group-id example-consumer \
  --topic events \
  --to-namespace example \
  --to-database main
```

Illustrative command shape based on the upstream Kafka-source reference; endpoint and authentication options depend on the deployment.

This integration can eliminate a significant amount of bespoke Kafka consumption, Protobuf decoding, and SurrealDB upsert orchestration. It is a separate integration tool, not a claim that SurrealDB itself natively accepts arbitrary Kafka topics.

There are important qualifications:

- Kafka consumption and SurrealDB write batching have separately configurable sizes.
- Consumer offsets are committed after sink batches, so retries and idempotent record identity matter.
- The documented default consumption timeout is one hour; permanent operation needs an appropriate supervisor.
- The optional `--schema-file` mechanism is described as limited: the currently documented schema-aware conversion supports a Protobuf string containing JSON being converted to a SurrealDB object. It is not a complete Protobuf-to-SurrealDB semantic type-mapping framework.
- These capabilities are documented in the upstream repository's `main` branch; the exact functionality available in a particular packaged `surreal-sync` release should be checked separately.

This is one of the strongest concrete Protobuf integrations in the SurrealDB ecosystem, distinct from merely storing Protobuf binary data.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Primary references:

`https://surrealdb.com/docs/learn/querying/real-time/live-queries`

`https://surrealdb.com/docs/learn/querying/real-time/changefeeds`

`https://surrealdb.com/docs/reference/query-language/statements/show`

`https://github.com/surrealdb/surreal-sync/blob/main/docs/kafka.md`

## 10. Rust and Python Protobuf tooling relevant to SurrealDB

The Rust ecosystem has several different Protobuf implementation families. They share the underlying protocol but differ materially in generated representations, reflection, JSON handling, and performance mechanisms.

### 10.1. Rust libraries and generators

| Package                       | Verified role                            | Distinctive capability or consideration                                                     |
| ----------------------------- | ---------------------------------------- | ------------------------------------------------------------------------------------------- |
| `prost`                       | Rust Protobuf runtime                    | Idiomatic generated structs; `bytes::Buf`/`BufMut`; supports proto2/proto3                  |
| `prost-build`                 | Code generation                          | Build-script integration, configurable field representations, descriptor-set output         |
| `prost-types`                 | Well-known messages                      | Generated representations for common Protobuf types                                         |
| `prost-reflect`               | Runtime reflection                       | `DescriptorPool`, `DynamicMessage`, descriptor-based decoding                               |
| `tonic` / `tonic-prost-build` | gRPC and service generation              | Generated Rust clients and servers for Protobuf RPC services                                |
| `pbjson` / `pbjson-build`     | ProtoJSON support                        | Generates Serde implementations following the Protobuf JSON specification                   |
| `buffa` family                | Alternative Rust Protobuf implementation | Editions support, owned/borrowed message representations, explicit unknown-field facilities |
| Buf CLI / BSR                 | Schema tooling and distribution          | Linting, breaking checks, descriptor generation, dependencies, published schemas            |

`prost` 0.14.4 is an established option for generated Rust messages. It preserves unknown enum values, but its own documentation explicitly states that it does not provide runtime reflection or message descriptors. `prost-reflect` supplies the separate dynamic-message functionality.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Useful `prost-build` controls include choosing `bytes::Bytes` instead of `Vec<u8>` for selected fields, choosing ordered maps, mapping external Protobuf types to existing Rust types, adding generated attributes, and outputting descriptor sets. These are particularly relevant where a Protobuf boundary would otherwise require unnecessary copying or duplicate Rust model definitions.

The current `tonic-prost-build` family generates Protobuf-based gRPC services; it is distinct from a plain message-generation step. `pbjson-build` consumes descriptors and generates ProtoJSON-compatible Serde implementations, addressing a semantic gap that ordinary derived Serde implementations do not necessarily close.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



### 10.2. Buffa: a consequential alternative to Prost

The independently maintained `buffa` family, verified at 0.9.2, deserves attention for this particular combination of typed messages, Rust execution, and heterogeneous storage.

Its documented capabilities include:

- Generated owned messages implementing its `Message` trait.
- Zero-copy borrowed views such as `FooView<'a>` for reading string and byte fields from an input buffer.
- Explicit unknown-field preservation and extension handling.
- Open-enum representations preserving unknown numeric values.
- First-class Protobuf Editions support.
- Configurable decoding limits, including recursion depth, message size, and unknown-field counts.
- Optional ProtoJSON, text-format, and `no_std` support.&#x20;

  [image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

  Rust



These distinctions matter in several workloads.

For decoding large numbers of messages: borrowed views can reduce copying of string and byte data, although their benefit depends on how long the source buffer remains alive and whether the destination database representation requires owned values.

For retaining future-schema data: explicit unknown-field support can help avoid information loss when a message is decoded and re-encoded by an older service.

For WASM extensions: `no_std` and pure Rust implementations may ease portability, but actual Surrealism WASI/component compatibility and the relevant generated code must still be verified.

For heterogeneous field types: the generated representation and owned-type customization can influence whether conversions require extra allocations or bespoke wrapper types.

The family comprises `buffa`, `buffa-build`, `buffa-codegen`, `buffa-types`, `buffa-descriptor`, and related packages. The generator accepts descriptors obtained through `protoc`, Buf, or an existing descriptor set. The runtime was still pre-1.0 at the researched version, so APIs and integration compatibility deserve separate scrutiny.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



References:

`https://docs.rs/prost/latest/prost/`

`https://docs.rs/prost-reflect/latest/prost_reflect/`

`https://docs.rs/tonic-prost-build/latest/tonic_prost_build/`

`https://docs.rs/pbjson-build/latest/pbjson_build/`

`https://docs.rs/buffa/latest/buffa/`

`https://github.com/anthropics/buffa`

### 10.3. Official Rust Protobuf versus ecosystem implementations

Google also documents an official Rust Protobuf implementation with its own generated API conventions, including message views and mutable accessors. It should not be confused with Prost or Buffa: the same `.proto` schema does not imply identical generated Rust types or interchangeable traits.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://protobuf.dev\&sz=32)

Protocol Buffers Documentation



For SurrealDB's native gRPC integration, the database's upstream protocol definitions and its SDK should determine compatibility. Choosing an application-level Protobuf runtime does not automatically change the runtime used internally by the SurrealDB SDK.

The published `surrealdb-protocol` Rust crate includes both generated Protobuf and FlatBuffers modules. These are separate representation families under the protocol project, not a guarantee that every service method or transport accepts both. Its generated API also exposes conversions involving `v1::Value`, which is useful evidence that native SurrealDB values are modeled explicitly rather than automatically becoming arbitrary user-defined messages.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Reference: `https://docs.rs/surrealdb-protocol/latest/surrealdb_protocol/`

### 10.4. Python interoperability

Python's official `google.protobuf` runtime supports generated messages, descriptors, dynamic message operations, and ProtoJSON conversion through `json_format`.

A Python-to-Rust interchange contract can therefore use the same `.proto` definitions without requiring the Rust and Python runtimes to share object layouts. This is useful for passing structured payloads through SurrealDB-backed workflows.

The main interoperability risks are semantic rather than syntactic:

- Python integer values can exceed the range of a declared Protobuf integer field.
- ProtoJSON's 64-bit integer representation differs from naive JSON number handling.
- A generated Python message's field-presence behavior must match the schema's syntax or edition.
- Record IDs, decimal-like values, datetimes, and binary payloads require explicit mapping to SurrealDB's native types.
- Runtime schema support may lag the newest Protobuf edition.

The official version-support table identifies the current Python Protobuf runtime major family as 7.x, while `protoc` versioning and Editions versions remain separate.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://protobuf.dev\&sz=32)

Protocol Buffers Documentation



### 10.5. Schema governance and inspection with Buf

Buf provides mechanisms that become especially valuable once Protobuf spans multiple SurrealDB clients or ingestion processes:

- `buf build` produces compiled schema images or descriptor sets.
- `buf lint` and `buf breaking` check schema structure and compatibility.
- `buf generate` manages code generation across languages.
- `buf convert` converts between binary Protobuf and JSON.
- `buf curl` invokes gRPC/Connect endpoints using known schemas.
- The Buf Schema Registry distributes versioned modules, dependencies, documentation, and generated SDKs.&#x20;

  [image](https://www.google.com/s2/favicons?domain=https://buf.build\&sz=32)

  Buf Docs+1



These are Protobuf lifecycle facilities, not SurrealDB database schema migrations.

There is a particularly practical inspection benefit: because SurrealDB's gRPC server does not expose server reflection, tools such as `grpcurl` or `buf curl` need access to the relevant published protocol definitions rather than discovering them automatically from the running server.

For an application-defined Protobuf API, service reflection can be provided separately through the chosen gRPC framework. Libraries such as `tonic` and `connectrpc` offer different mechanisms, but this does not enable reflection on the existing SurrealDB server.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



References:

`https://buf.build/docs/cli/`

`https://buf.build/docs/`

`https://github.com/connectrpc/connect-rust`

## 11. Interactions, tradeoffs, and opportunities to avoid custom machinery

### 11.1. Where native SurrealDB features are preferable to building around Protobuf

The most consequential architectural opportunities are not necessarily additional Protobuf components. They are cases where SurrealDB already handles the stateful operation and Protobuf only needs to describe its boundary.

| General problem                            | Native mechanism to investigate first                  | What Protobuf still contributes                        |
| ------------------------------------------ | ------------------------------------------------------ | ------------------------------------------------------ |
| Traverse connected records                 | `RELATE`, links, references, recursive paths, GQL      | Typed request/result models                            |
| Maintain reverse links                     | `REFERENCE`, `MATERIALIZE INTO`                        | Interchange representation of relationships            |
| Optimize bounded adjacency                 | `INLINE EDGES`, `INLINE REFERENCES`, adjacency folding | No special requirement                                 |
| Store propertyless connections             | `LIGHTWEIGHT` relations                                | Edge creation/deletion event contracts                 |
| Maintain derived tables                    | `DEFINE TABLE ... AS SELECT`                           | External typed projections                             |
| Index fields and search                    | Native indexes, full-text, HNSW, DiskANN               | Typed payloads entering/leaving the search workflow    |
| Execute transformations in a transaction   | Surrealism with transaction handles                    | Decode/encode or validate Protobuf messages            |
| Subscribe to recent changes                | `LIVE SELECT`                                          | Typed notification envelopes                           |
| Replay recent mutations                    | `CHANGEFEED`                                           | Versioned external change contracts                    |
| Ingest Protobuf from Kafka                 | `surreal-sync from kafka`                              | Existing wire schema, decoder, compatibility policy    |
| Stream query results                       | Native gRPC incremental results                        | Existing database Protobuf RPC protocol                |
| Minimize homogeneous row encoding overhead | Shaped results                                         | Only necessary when another consumer requires Protobuf |

These alternatives do not establish that Protobuf is unnecessary. They establish that serialization, indexing, transaction coordination, graph traversal, and data-contract governance are different jobs.

### 11.2. Performance boundaries

Protobuf can improve the efficiency of structured interchange when compared with more verbose encodings, but the actual cost of a SurrealDB operation can lie elsewhere:

- Serialization work: encoding/decoding and intermediate object construction.
- Network movement: message size, compression, framing, and streaming.
- Database execution: index selection, graph expansion, filters, and query planning.
- Storage behavior: key-range access, compaction, caches, and transaction conflicts.
- Data-model duplication: maintaining opaque payloads alongside queryable projections.

Using a zero-copy Protobuf view may avoid allocations during decoding. It cannot eliminate allocations needed to materialize an independent SurrealDB value, nor can it avoid database indexing work.

Similarly, encoding an entire query result into Protobuf can reduce repeated names or textual overhead, but SurrealDB's shaped results and incremental gRPC delivery address related concerns natively. Their relative efficiency needs workload-specific measurement rather than a presumed hierarchy.

### 11.3. Security and correctness boundaries

Protobuf schemas are not authorization policies. A correctly encoded message may still request an unauthorized database operation.

For external or untrusted messages, decoder limits on total size, recursion depth, nesting, and unknown fields are important. Protobuf's binary format does not prevent intentionally hostile payloads or excessive memory consumption.

Several additional hazards merit explicit treatment:

- Schema resolution: dynamically loading descriptors creates a trust boundary. A type URL or schema name should not be treated as authority to load arbitrary code or schemas.
- Data exposure: encoding query results into Protobuf must preserve SurrealDB's permissions and field redaction.
- Unknown fields: preservation differs by runtime and conversion path.
- Canonicalization: deterministic serialization is insufficient by itself for portable cryptographic identity.
- Replay: message delivery, database commit, and external offset acknowledgement are independent steps.
- Graph correctness: a well-formed `EdgeChanged` message does not prove its endpoints exist or that the edge write committed.

### 11.4. Version and maturity distinctions

| Capability                           | Verified status as of November 3, 2026                                                               |
| ------------------------------------ | ---------------------------------------------------------------------------------------------------- |
| SurrealDB 4.0                        | Announced generally available November 3, 2026                                                       |
| Migration from SurrealDB 3.x to 4.0  | Experimental, opt-in, one-way; upstream recommends waiting for 4.1 for existing production migration |
| SurrealKV 1.0                        | Released storage-engine line used by SurrealDB 4.0                                                   |
| SurrealDB gRPC RPC                   | Available since SurrealDB 3.3                                                                        |
| Shaped results                       | Available since SurrealDB 4.0                                                                        |
| Graph query tracing                  | Available since 4.0, with transport and completeness limitations                                     |
| ISO GQL                              | Available since 3.2; enabled by default from 3.3; subset of the standard                             |
| Surrealism 4.0 invocation model      | Fresh WASM instance per function call                                                                |
| Protobuf Edition 2026                | Released August 20, 2026; minimum `protoc` 36.0                                                      |
| Buffa                                | Verified 0.9.2, pre-1.0                                                                              |
| `surreal-sync` Kafka Protobuf source | Documented upstream; packaged-release feature parity requires verification                           |

SurrealDB's current release announcement explicitly recommends 4.0 for greenfield deployments and additional validation before migrating existing production 3.x datastores. The new storage formats and migration behaviors should not be conflated with Protobuf schema compatibility.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB



The separate SurrealDB Agent Memory managed service, SurrealDB Earth, and SurrealDS are likewise distinct products or deployment capabilities. Their existence does not imply a common native Protobuf storage representation or a standard Protobuf API for agent memory.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB



Primary release and migration references:

`https://surrealdb.com/blog/surrealdb-4-the-database-and-memory-for-every-agent`

`https://surrealdb.com/docs/build/migrating/from-old-surrealdb-versions/3x-to-4x`

## 12. Final synthesis

The strongest opportunities for Protobuf within the SurrealDB 4.0 ecosystem fall into three categories.

1\. Typed interoperability. Protobuf can establish durable, language-neutral contracts for records, graph mutations, query results, agent state, and external events. The published SurrealDB gRPC protocol is a concrete example; custom application contracts represent a separate layer.

2\. Schema-aware transformation. Generated Rust/Python types, descriptors, dynamic decoding, ProtoJSON conversion, and Surrealism extensions can support systematic translation between structured external messages and SurrealDB's richer native values. This is where attention to field presence, identity, numeric precision, and schema revisions matters most.

3\. Integration without duplicating database machinery. SurrealDB already supplies graph relationships, reference tracking, materialization, transaction isolation, query planning, live updates, changefeeds, and multiple storage engines. Protobuf complements these mechanisms most effectively when it defines what crosses a boundary, while SurrealDB retains responsibility for state and querying.

The most easily overlooked capabilities are the existing gRPC protocol, descriptor-driven Rust processing, `surreal-sync`'s Protobuf Kafka ingestion, Surrealism's transactional WASM execution, and SurrealDB's native lightweight edges, adjacency caches, and inverse-reference materialization. Each can substantially reduce the amount of custom infrastructure needed—but they solve different problems and carry different guarantees.

The governing distinction is straightforward: Protobuf standardizes the shape and evolution of information in motion; SurrealDB governs the meaning, relationships, and consistency of information at rest and under query.

# Addendum: Advanced Protocol Buffers integration with SurrealDB 4.0

Supplement to the original technical overview · November 3, 2026 · Primary version scope: SurrealDB 4.0.0 and its associated protocol, storage-engine, and extension libraries

## 1. Native Protobuf support in `surrealdb-types`

The most significant addition to the original reference is that SurrealDB 4.0 already contains a native Protobuf value codec, separate from its gRPC transport integration.

This provides a direct mechanism for encoding and decoding SurrealDB's public Rust `Value` representation using the database's own Protobuf schema, without independently implementing the entire database-value mapping.

### 1.1. Architecture and version boundaries

Inspection of the tagged SurrealDB 4.0.0 source establishes the following component versions:

| Component            | Version used by SurrealDB 4.0.0 | Role                                                  |
| -------------------- | ------------------------------- | ----------------------------------------------------- |
| `surrealdb-types`    | 4.0.0                           | Public Rust database values, types, conversion traits |
| `surrealdb-protocol` | 0.14.1                          | Published network value and RPC representations       |
| `prost`              | 0.14.4                          | Protobuf runtime used for native value encoding       |
| `surrealkv`          | 1.2.0                           | Durable embedded key-value storage                    |
| `surrealmx`          | 1.1.0                           | Transactional in-memory storage                       |

These are the workspace dependency declarations, not a claim about separately installed packages.

The central conversions are:

`surrealdb_types::Value`

Native Rust database-value representation

Fallible native/protocol conversion

`surrealdb_protocol::proto::v1::Value`

Database-specific Protobuf message

Prost binary encoding/decoding

Protobuf binary bytes

The source exports two functions when the `proto` Cargo feature is enabled:

```
pub fn encode(value: &Value) -> anyhow::Result<Vec<u8>>;
pub fn decode(bytes: &[u8]) -> anyhow::Result<Value>;
```

They are re-exported publicly as `encode_proto` and `decode_proto`. The optional feature enables the Prost dependency and Protobuf support in `surrealdb-protocol`.

The same feature also re-exports `prost`, allowing consumers to work with the runtime version used by the conversion implementation.

A minimal illustration of the public API:

```

use surrealdb_types::{decode_proto, encode_proto, Value};

fn roundtrip(value: &Value) -> anyhow::Result<Value> {
    let encoded = encode_proto(value)?;
    let decoded = decode_proto(&encoded)?;
    Ok(decoded)
}

```

Illustrative use of the exported 4.0.0 functions; not executed or benchmarked here.

This capability is specifically for SurrealDB's own Protobuf `Value` schema. It does not automatically transform arbitrary Prost-generated application messages into SurrealDB records.

Primary tagged sources:

`https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/types/src/proto/mod.rs`

`https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/types/Cargo.toml`

`https://github.com/surrealdb/surrealdb/blob/v4.0.0/Cargo.toml`

### 1.2. Conversion fidelity and asymmetric type support

The native implementation is useful, but it is not a universally lossless representation of every SurrealDB 4.0 value.

Several behaviors are explicit in the tagged source:

| Type or situation                          | Native codec behavior                                           | Consequence                                                            |
| ------------------------------------------ | --------------------------------------------------------------- | ---------------------------------------------------------------------- |
| Signed and other sized integers            | Encoding uses the native number's widened representation        | Original width can be lost                                             |
| `f16` and `f32`                            | Native-to-wire encoding can widen floating-point representation | Numeric value may survive while type identity changes                  |
| Exact decimals                             | Encoded through the supported decimal representation            | Fidelity depends on representable precision and scale                  |
| BigDecimal outside supported decimal range | Falls back to a string representation                           | Numeric type information is lost                                       |
| `Date`, `Time`, `Timestamp`, `Interval`    | Converted through `degrade_temporal()`                          | New temporal type identity is not preserved                            |
| Large durations                            | Out-of-range conversion fails                                   | The codec rejects rather than silently wrapping                        |
| Unrecognized `Value` variant               | Decoding fails                                                  | Unknown future types are not silently treated as valid database values |
| Unset Protobuf value `oneof`               | Decoding fails                                                  | Cannot be interpreted as native `NONE`                                 |
| Unparsed string record ID variant          | Not representable by that native decoder                        | Explicit conversion or parsing boundary required                       |

An important asymmetry is that the Protobuf decoder understands several narrow-number variants even though the native encoder can widen values when writing them.

Consequently:

\\[ \operatorname{decode}(\operatorname{encode}(v))=v \\]

is not a universal invariant for every 4.0 native value when exact type identity matters.

The source includes round-trip tests for a substantial collection of conventional values—scalars, arrays, objects, record IDs, ranges, sets, files and geometries—but the existence of those tests should not be generalized to a guarantee covering all newer value variants.

The conversion methods are fallible for substantive semantic reasons, not merely because the byte stream might be malformed.

Source:

`https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/types/src/proto/value.rs`

### 1.3. Implications for serialized data

The public codec enables direct Protobuf interchange of native database values with significantly less custom code. Its most important limitation is the distinction between transportable values and round-trip-preserving values.

That difference becomes material for typed graph identifiers, schema-aware backups, historical snapshots, exact numeric data, and cross-version clients.

Where exact representation must survive, the encoding version and supported type subset are part of the contract. The presence of a Protobuf encoder does not by itself establish that every native database type can be archived and restored without semantic changes.

## 2. The SurrealDB Protobuf schema is richer than ordinary messages

The associated `surrealdb-protocol` package defines database-specific types rather than relying exclusively on standard Protobuf scalar and well-known types.

### 2.1. Database-specific semantics

The tagged protocol includes several structures worth understanding directly.

| Protocol type                | Distinctive capability                                            |
| ---------------------------- | ----------------------------------------------------------------- |
| `Value`                      | Tagged union for native database values                           |
| `NoneValue` and `NullValue`  | Separate representations for absence and explicit null            |
| `Object` and `KeyValue`      | Object properties with prescribed key representation and ordering |
| `RecordId` and `RecordIdKey` | Table-qualified IDs with structured keys                          |
| `Range` and bound types      | Inclusive, exclusive and unbounded limits                         |
| `Datetime` and `Duration`    | Database-specific temporal range and normalization                |
| `Decimal`                    | Explicit decimal representation                                   |
| `Geometry`                   | Spatial value variants                                            |
| `Kind`                       | Protobuf representation of database type descriptions             |

This means external Protobuf applications need not necessarily invent a string-only record ID or reduce every value to JSON-compatible primitives.

### 2.2. Structured record identity

SurrealDB record IDs may use several key representations, including strings, integers, UUIDs, arrays, objects and ranges.

The published protocol models these variants explicitly. Its native conversion implementation also rejects floating-point and decimal record-key variants that cannot be represented by the public Rust `RecordIdKey` type.

This is consequential when encoding graph topology: two identifiers with identical textual renderings under an application-specific convention are not necessarily equivalent to the corresponding typed database identities.

A Protobuf graph representation that preserves the native typed `RecordId` avoids imposing unnecessary string parsing and serialization conventions on every consumer.

Source:

`https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/types/src/proto/record.rs`

### 2.3. Objects and missing values

The protocol deliberately represents object entries using repeated `KeyValue` messages rather than treating an ordinary Protobuf map as a sufficient semantic representation.

The tagged schema describes unique object keys and ascending UTF-8 byte ordering. This avoids relying on implementation-specific map ordering and addresses historical issues involving default-valued entries.

The distinction between an absent object field, a field holding `NONE`, and a field holding `NULL` must remain explicit.

Likewise, an unknown or unset value variant cannot safely be reinterpreted as `NONE`: that could turn an unsupported future value into a database operation with deletion-like semantics.

### 2.4. Temporal representations

SurrealDB's custom `Datetime` message accommodates a broader range than `google.protobuf.Timestamp`, whose standardized range is limited to years 0001–9999.

Its protocol also defines explicit seconds and nanoseconds rules rather than assuming every native duration maps directly to `google.protobuf.Duration`.

This creates an important distinction between the protocol's ability to represent certain temporal values and the current native codec's treatment of newer temporal types.

The schema and the conversion implementation must be evaluated together.

Primary protocol schema for SurrealDB 4.0.0:

`https://github.com/surrealdb/surrealdb-protocol/blob/v0.14.1/surrealdb/protocol/v1/value.proto`

## 3. Connecting generated Protobuf types to SurrealDB's Rust type system

### 3.1. `SurrealValue`: the native conversion contract

SurrealDB 4.0 exposes its public value system through the separate `surrealdb-types` crate, also accessible as `surrealdb::types`.

The central abstraction is the `SurrealValue` trait, which supports three operations:

- `kind_of()`: describes a Rust type as a SurrealQL `Kind`.
- `into_value()`: converts a Rust value to a native SurrealDB `Value`.
- `from_value()`: converts a native database value back into a Rust type, returning an error for incompatible representations.

The derive macro additionally supports Rust struct and enum conversions, while the `kind!` macro constructs database type descriptions using SurrealQL-like syntax.

The SDK documentation explicitly distinguishes `#[surreal(...)]` attributes from Serde attributes. A Rust field configured with `#[serde(flatten)]` does not automatically inherit equivalent `SurrealValue` behavior.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



This matters when working with generated Protobuf structs: their generated types and traits do not automatically make them valid SurrealDB record models.

### 3.2. Three alternatives to duplicate model implementations

The existing mechanisms support several different levels of integration.

Generated Protobuf type → native SurrealDB type. A conversion can be implemented between a generated message and a Rust type deriving `SurrealValue`. This gives each format an independently defined representation while delegating the database-side conversion.

Generated Protobuf type → `Value` directly. For dynamic or highly heterogeneous messages, reflection-driven traversal can construct a `Value::Object`, `Value::Array`, or other native variant. This avoids defining an intermediate Rust struct for every message shape.

Native SurrealDB `Value` → native Protobuf `Value`. When the required contract is SurrealDB's own protocol rather than a domain-specific message, the `encode_proto`/`decode_proto` facility removes the need to implement the conversion again.

The third route involves the least transformation machinery, but it exposes the consumer to SurrealDB-specific protocol types. The first two support independently governed application contracts.

### 3.3. Generating database type descriptions

`kind_of()` returns a database `Kind`; its `to_sql()` representation can be incorporated into SurrealQL schema definitions.

For example, the documented mapping of Rust types includes:

| Rust type                               | Generated SurrealQL type                   |
| --------------------------------------- | ------------------------------------------ |
| `Option<String>`                        | `none \\| string`                          |
| `Vec<String>`                           | `array<string>`                            |
| `RecordId`                              | `record`                                   |
| Rust struct deriving `SurrealValue`     | Object type containing its fields          |
| Custom enum implementing `SurrealValue` | Explicitly specified union or object shape |

This makes a Rust-defined model capable of providing part of its corresponding database type description.

It does not establish native generation of database schemas from arbitrary `.proto` descriptors. Such generation would still need to decide how Protobuf field presence, enums, maps, nested messages and identifiers correspond to SurrealQL types and constraints.

### 3.4. Named types and reusable database contracts

SurrealDB 4.0 introduces `DEFINE TYPE`, which permits reusable named types in database fields, function signatures, casts, and nested type expressions.

Three especially relevant mechanisms are:

Literal unions. A declaration such as `DEFINE TYPE priority AS 'low' | 'medium' | 'high'` provides a constrained string domain. Its declaration order also defines value ordering, unlike ordinary Protobuf numeric enum semantics.

Structured object types. A named object type can specify required and optional fields. SurrealDB checks its declared object shape and rejects unexpected fields where the type requires an exact structure.

Named constraints and defaults. `CONSTRAINT ... ASSERT` attaches validation rules, while `DEFAULT` supplies database defaults.

`ALTER TYPE` and `DEFINE TYPE OVERWRITE` support evolution, but the database does not automatically rewrite records when the type's definition changes. Named types also prohibit recursive self-reference.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



One important interaction with Protobuf concerns enum extension.

A Protobuf enum value is identified by its integer number. A SurrealDB literal-string union is identified through its literal values, with order affecting comparisons. A generated mapping must preserve the intended identity and ordering semantics rather than assuming the two enum models are equivalent.

Protobuf message compatibility and SurrealDB named-type compatibility therefore form two distinct validation boundaries.

Primary references:

`https://surrealdb.com/docs/reference/rust/concepts/working-with-types`

`https://surrealdb.com/docs/reference/rust/concepts/surrealvalue-attributes`

`https://surrealdb.com/docs/reference/query-language/statements/define/type`

`https://surrealdb.com/docs/reference/query-language/language-primitives/data-types/named-types`

## 4. CBOR v2, FlatBuffers, shaped results, and protocol conformance

### 4.1. CBOR v2 changes the representation comparison

SurrealDB 4.0 introduces `cbor.2` for additional native value representations.

The original CBOR protocol widens sized numeric values such as `u8`, `i32`, and `f32`; a `u64` exceeding the signed 64-bit range is represented as a decimal.

The new `date`, `time`, `timestamp`, `interval`, and `bigdecimal` value families also have representations available only in CBOR v2. The original CBOR protocol degrades these to older compatible types, and it does not provide the same regex representation.

CBOR v2 is consequently more than a change of media type: it offers stronger fidelity for new SurrealDB 4.0 values.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



The migration documentation also identifies stricter CBOR decoding in 4.0:

- Legacy textual datetime, UUID and duration tags are rejected in favor of their compact forms.
- An integer outside the supported signed range is rejected rather than silently wrapped.
- Duplicate keys in a CBOR object are rejected.
- The `NONE` tag must carry the expected null representation.

These changes are relevant to custom Protobuf/CBOR bridges because an older codec can appear to work with conventional values while failing on a new numeric or temporal type.

### 4.2. Choosing between native binary representations

| Representation                | What it offers                                               | Important qualification                                       |
| ----------------------------- | ------------------------------------------------------------ | ------------------------------------------------------------- |
| `cbor.2`                      | Rich native database-value fidelity, including new 4.0 types | Requires a CBOR v2-aware client                               |
| Original CBOR                 | Compatibility with older SurrealDB clients                   | Degrades or refuses several newer value types                 |
| Native Protobuf `Value`       | Generated Protobuf representation and a Rust codec           | The tagged native encoder has type-widening/degradation cases |
| Native FlatBuffers            | A separately implemented database-value representation       | Not automatically interchangeable with Protobuf               |
| Shaped results                | Shared row metadata and compact repeated row values          | Separate encoding carried within CBOR                         |
| Application-specific Protobuf | Independently defined, language-neutral contracts            | Requires mapping from native SurrealDB values                 |

SurrealDB's shaped results are particularly relevant to row-oriented workloads. A shaped block describes column names and types once and then carries individual row values without repeatedly transmitting full field metadata.

The documented format uses CBOR tag `57` to carry the specialized row block. Negotiation is available through the applicable WebSocket and HTTP interfaces, whereas the native gRPC transport has its own incremental-result mechanism.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Neither feature should be portrayed as a universal replacement for Protobuf: they solve different representation problems.

### 4.3. Protobuf and FlatBuffers parity is not complete

The tagged `surrealdb-protocol` repository contains both Protobuf and FlatBuffers definitions, alongside explicit parity checks.

The `parity.toml` file documents accepted schema differences, including deliberately different value-union tag numbers. A numerical correspondence between every Protobuf variant and FlatBuffers union arm cannot be presumed.

More importantly, it records an unresolved divergence between the Protobuf `Kind` representation and its FlatBuffers counterpart. Certain variant identifiers and names do not align, and the repository states that general `Kind` conversion between the representations is not implemented.

The fact that both schemas describe SurrealDB values therefore establishes neither binary interchangeability nor complete semantic compatibility.

For application-level type introspection, this matters more than it initially appears: a generic schema gateway could successfully convert ordinary values but still fail to preserve type descriptions.

Source:

`https://github.com/surrealdb/surrealdb-protocol/blob/v0.14.1/parity.toml`

### 4.4. Existing protocol conformance facilities

The upstream protocol project also includes facilities for validating independent client implementations:

- `spec/` contains language-neutral protocol information, CBOR tags, value test vectors, RPC method descriptions, headers and error kinds.
- `conformance/` contains behavioral scenarios and a runner for SDK implementations.
- Protobuf/FlatBuffers schema parity is checked separately.
- The SurrealDB 4.0 native codec has its own round-trip tests.

These are distinct forms of validation.

Passing Protobuf decoding tests demonstrates that a particular message can be decoded. Passing SurrealDB value-conversion tests demonstrates compatibility with the database's expected value semantics. Passing RPC conformance scenarios additionally covers observable client/server behavior.

None automatically proves that a custom Protobuf schema preserves every database type or transaction-level invariant.

Primary references:

`https://github.com/surrealdb/surrealdb-protocol/tree/v0.14.1/spec`

`https://github.com/surrealdb/surrealdb-protocol/tree/v0.14.1/conformance`

`https://github.com/surrealdb/surrealdb-protocol/blob/v0.14.1/parity.toml`

`https://surrealdb.com/docs/reference/rest-api/cbor-protocol`

`https://surrealdb.com/docs/reference/rest-api/rpc-protocol`

## 5. Surrealism 4.0: Protobuf processing inside the database

Surrealism introduces an additional integration possibility beyond decoding Protobuf in an external application: executing Rust serialization, decoding, validation, or transformation functions within SurrealDB's transaction environment.

The important new details concern the extension ABI, lifecycle, security, resources, and data access.

### 5.1. WASM component ABI and execution lifecycle

SurrealDB 4.0's extension system uses `wasm32-wasip2`, WASI Preview 2, and the WebAssembly component model.

The `v4.0.0` runtime supports ABI 4 and ABI 5:

- ABI 4 provides the SurrealDB 4.0 transaction-capable extension interface.
- ABI 5 adds typed extension bindings for supplying values through a definition's `BINDINGS` clause.

The tagged runtime selects the oldest supported ABI containing all features required by an extension. This is preferable to assuming that the version of the SDK used to build an extension determines its ABI automatically.

The compiled `.surli` package contains its WASM component, exported function metadata, and other package resources.

Each invocation starts a fresh instance, using the memory state prepared by its build-time initialization. State mutated by one invocation is discarded afterward.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



This creates two distinct optimization opportunities for Protobuf:

Static schema initialization. Descriptor data, lookup tables and compatible runtime objects can potentially be initialized during the extension build and included in the initial memory snapshot.

Call-local processing. Protobuf decoding, validation, reflection and conversion operate within the isolated invocation. Temporary decoded objects do not require a separately managed long-lived cache.

The first opportunity is conditional: preinitializing a descriptor runtime requires that its representation and dependencies can survive the build-time snapshot correctly. The documented snapshot mechanism does not establish compatibility with every Rust Protobuf reflection implementation.

### 5.2. Transaction integration

Surrealism 4.0 replaces older implicit database-access functions with an explicit `Transaction` parameter.

The provided operations include:

| API                                         | Purpose                                          |
| ------------------------------------------- | ------------------------------------------------ |
| `tx.sql(query)`                             | Execute SurrealQL within the current transaction |
| `tx.sql_with_vars(query, vars)`             | Execute a parameterized query                    |
| `tx.run(function, args)`                    | Call a defined function                          |
| `tx.run_versioned(function, version, args)` | Call a specified function version                |
| `Transaction<ReadWrite>`                    | Declare that the extension can perform writes    |

The transaction handle is scoped to the invocation and cannot be retained for later use.

An extension's writes participate in the transaction that invoked it: they can observe uncommitted state from that transaction, and they commit or roll back with the surrounding operation.

This differs materially from an external Protobuf microservice that must establish a separate database connection and transaction.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



A Protobuf decoder embedded in an extension could therefore transform an external message and construct native records or graph relations within one transactional operation.

However, extensions remain subject to Surrealism's execution restrictions. They cannot be called from every type of schema clause. In particular, extension functions are excluded from `COMPUTED`, `VALUE`, `DEFAULT`, `ASSERT`, and `PERMISSIONS` expressions. The presence of a Protobuf validation function does not make it a direct substitute for all native database constraints.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 5.3. Capability controls and constrained network access

A Surrealism extension does not receive unrestricted process-level authority.

`DEFINE EXTENSION` specifies capabilities such as:

| Capability       | Controlled operation                   |
| ---------------- | -------------------------------------- |
| `net`            | Outbound network destinations          |
| `functions`      | Callable SurrealQL functions           |
| `queries`        | Arbitrary SurrealQL queries            |
| `scripting`      | Embedded JavaScript                    |
| `write`          | Database-modifying operations          |
| `memory`         | Per-invocation memory ceiling          |
| `timeout`        | Per-invocation execution-time ceiling  |
| `strict_timeout` | Behavior of execution-time enforcement |

The extension's permissions cannot exceed restrictions imposed at the server level.

SurrealDB 4.0 specifically supports outbound HTTP requests through `wasi:http`. Crates using that interface, such as `waki`, can make requests subject to the declared network allowlist and host-side address checks.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



This enables some useful Protobuf-related possibilities, including fetching descriptor sets or submitting HTTP-compatible binary messages to another service.

But the transport distinction is important:

- The existence of `wasi:http` does not prove that a conventional Tonic gRPC client works in the extension.
- A library requiring unrestricted sockets, a multithreaded Tokio runtime, or unsupported WASI facilities may not work.
- Network access is permissioned and must respect the extension's resource limits.
- External calls can have side effects that are not rolled back if the surrounding database transaction fails.

The last point is especially consequential. In-database transactional execution does not automatically make an external Protobuf message publication transactional.

### 5.4. Typed bindings and descriptor configuration

ABI 5 introduces `BINDINGS`, which allows an extension definition to supply typed values to a WASM component.

An extension declares bindings in `package.toml` and accesses them through `surrealism::binding()`.

The database evaluates the binding expression on first access during an invocation and reuses the result for the remainder of that invocation.

For Protobuf integration, this can support configuration such as schema identifiers, endpoint names, validation policy, or database-specific transformation settings without rebuilding the same extension for every configuration.

Binding expressions run under a restricted authority model: they can access protected values under the documented rules, but cannot simply elevate the caller above the authority associated with the definition. An `init` function cannot access these bindings because initialization occurs before a database-specific extension definition exists.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



An additional distinction is that `BINDINGS` is a typed configuration mechanism, not a replacement for the removed Surrealism module key-value store.

SurrealDB 4.0 removes that former KV facility. Extension state that must survive calls belongs in database records or another appropriately managed persistent system.

### 5.5. Distribution, integrity and deployment boundaries

A Surrealism package can be referenced through a file bucket or the supported Silo package registry.

The extension definition supports:

- `DIGEST` to pin the exact archive content.
- `SIGNER` to verify signed packages.
- `UNSIGNED` for intentionally unsigned packages.
- `CAPABILITIES` to grant runtime authority.
- `BINDINGS` for ABI 5 configuration.
- `PERMISSIONS` to control who may invoke the extension.

These mechanisms make it possible to distribute Protobuf-aware conversion logic without changing SurrealDB's core executable. They also create a clear distinction between schema compatibility and executable-package integrity: a trusted Protobuf descriptor does not establish that a particular WASM binary is trusted, and vice versa.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Primary references:

`https://surrealdb.com/docs/learn/extensions/guides/extension-architecture`

`https://surrealdb.com/docs/learn/extensions/guides/creating-custom-extensions`

`https://surrealdb.com/docs/learn/extensions/guides/surrealism-attribute-reference`

`https://surrealdb.com/docs/reference/query-language/statements/define/extension`

`https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealism/runtime/src/abi.rs`

## 6. Custom HTTP APIs with binary Protobuf payloads

### 6.1. `DEFINE API` can receive raw bytes

SurrealDB's custom API framework provides an HTTP integration surface independent of the database RPC protocol.

Its body middleware includes `api::req::body("bytes")`, which makes raw request bytes available without first interpreting them as JSON or CBOR. The API also provides response-body serialization controls, headers, status management and request limits.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



The resulting composition is:

HTTP binary body → raw SurrealDB bytes → Protobuf decoder → native values/relations → optionally encoded binary response.

This may eliminate a separate application server for suitable Protobuf-over-HTTP interfaces.

It is not an automatically generated Protobuf service. The API framework does not establish built-in support for arbitrary `.proto` schemas, Protobuf service dispatch, or gRPC framing. Decoding and encoding need an actual supported implementation, such as an extension.

### 6.2. Body limits and decoding responsibility

`api::req::max_body()` imposes a limit on an incoming raw body before its parsing middleware processes it.

It returns HTTP `413` when the raw payload exceeds the limit, avoiding unnecessary decoding work.

This is complementary to Protobuf-specific decoder limits. A relatively small payload can still be deeply nested or computationally expensive to process, and a Protobuf decoder must separately enforce its own relevant constraints.

The documented middleware supports request body strategies including JSON, CBOR, FlatBuffers, plain text, bytes and native values. These built-in strategies should not be confused with an application-defined Protobuf decoder.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 6.3. API execution authority

A particularly important SurrealDB 4.0 capability is the distinction between `SECURITY DEFINER` and `SECURITY INVOKER` on custom API definitions.

| Mode               | Execution behavior                                                        |
| ------------------ | ------------------------------------------------------------------------- |
| `SECURITY DEFINER` | Default; API body executes without ordinary table/field permission checks |
| `SECURITY INVOKER` | API body runs under the caller's database permissions                     |

This is separate from the API endpoint's own `PERMISSIONS` clause, which controls who can invoke it.

The default is consequential: omitting endpoint permissions can expose a custom API to callers who are not otherwise allowed to access the underlying records.

A Protobuf contract does not limit database authority. A message can be structurally valid and still request data that the caller must not receive.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



SurrealDB also documents transaction rollback behavior for failed API handlers, including database writes and resulting index/event effects. That does not extend atomicity to externally delivered HTTP requests or messages.

### 6.4. Conditional architectural value

An API plus Surrealism can be valuable where the operation is naturally a small synchronous request, the Protobuf message is of bounded size, and the transformation benefits from executing inside the database's transaction.

A separate service can remain more appropriate where processing requires long-running jobs, broad filesystem or network capabilities, complex streaming, standard gRPC interoperability, or independent scaling of decoding workloads.

The distinction is not merely deployment convenience: it affects permission scope, extension execution limits, transaction ownership and external side effects.

Primary references:

`https://surrealdb.com/docs/reference/query-language/statements/define/api`

`https://surrealdb.com/docs/reference/query-language/functions/database-functions/api`

## 7. Direct Protobuf persistence in SurrealKV and SurrealMX

The original overview considered SurrealKV and SurrealMX mainly as storage engines beneath SurrealDB. Both also expose standalone Rust interfaces, allowing applications to store Protobuf-encoded messages as transactional key/value data without invoking SurrealQL.

This is a different integration boundary from storing Protobuf bytes in a SurrealDB record. It delegates persistence to the storage engine but not schema validation, graph traversal, or secondary indexing.

### 7.1. SurrealKV 1.2.0: durable, versioned binary-message storage

The standalone SurrealKV engine provides an LSM-tree storage architecture, transactional key/value access, and a configurable asynchronous commit pipeline. Its versioned-key machinery is particularly relevant to serialized messages.

The tagged 1.2.0 implementation documents:

| Facility                       | Native mechanism                                  | Protobuf interaction                                                |
| ------------------------------ | ------------------------------------------------- | ------------------------------------------------------------------- |
| Versioned values               | `with_versioning(true, retention_ns)`             | Retain successive serialized message versions under a key           |
| Historical lookup              | `get_at(key, ts)`                                 | Retrieve a previously committed or timestamped payload              |
| Version history                | `history`                                         | Enumerate retained historical values                                |
| Explicit historical timestamps | `set_at`                                          | Associate writes with an application-specified historical timestamp |
| Value-log separation           | `with_enable_vlog(true)`                          | Keep large values outside the main LSM tree                         |
| Transaction durability         | `Durability::Eventual` or `Durability::Immediate` | Choose when a committed payload is acknowledged as durable          |
| Encryption at rest             | AES-256-GCM or XChaCha20-Poly1305                 | Protect stored message bytes without changing the Protobuf schema   |
| Storage diagnostics            | `skv` inspection and integrity facilities         | Diagnose persistence independently of message decoding              |

The versioning and Protobuf mechanisms remain independent. SurrealKV retains byte values according to its configured temporal policy; it does not interpret a Protobuf schema revision or automatically migrate older messages.

A particularly useful interaction arises when historical Protobuf payloads are retained unchanged. A versioned key can preserve the original bytes of an earlier message, while a separately identified descriptor set allows those bytes to be interpreted using the schema appropriate to that historical version.

That is stronger than simply decoding each older message through the newest generated struct. A new decoder may be binary-compatible yet still lose older semantic distinctions, unknown fields, or information during conversion.

#### Value-log separation

SurrealKV 1.2.0 documents an optional value log for values exceeding a configurable threshold, defaulting to 1 KiB when enabled.

This separates large payload storage from the LSM tree's key/index structure, potentially reducing compaction-related write amplification for large serialized messages. The benefit depends on message-size distribution, update frequency, read locality, and garbage-collection behavior.

It is not evidence that storing Protobuf is intrinsically faster than storing another binary format of comparable size.

#### Durability semantics

In the standalone crate, `Durability::Eventual` is the documented default. `Durability::Immediate` requests a write-ahead-log synchronization before commit returns.

The browser OPFS implementation has different persistence guarantees: requesting a file flush does not establish the same power-loss durability guarantee as a supported native filesystem.

These options also must not be automatically transferred to the configuration used by SurrealDB's integrated SurrealKV backend.

The standalone engine additionally supports OPFS in dedicated browser Web Workers, enabling a local binary-message store in an environment where maintaining a traditional database-server process is inappropriate. Browser support and its associated feature limitations are documented separately from native-platform behavior.

Primary source, tagged release:

`https://github.com/surrealdb/surrealkv/blob/v1.2.0/README.md`

### 7.2. SurrealMX 1.1.0: transactional in-memory message access

SurrealMX exposes a lower-level transactional store with MVCC, concurrent operations, savepoints, and ordered key scans. Its standalone documentation describes snapshot isolation for read-only transactions and strict serializability for update transactions.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Several APIs are especially interesting for Protobuf processing:

- `with_value` permits inspection of borrowed value bytes.
- `scan_with` and `keys_for_each` support callback-based traversal without materializing an allocated collection of key/value pairs.
- `scan_into` and `keys_into` reuse caller-provided buffers.
- Direct operations such as `get`, `set`, `put`, `del`, and `exists` avoid a query-language interface when only key-based state is needed.

The relevant optimization opportunity is buffer ownership, not just serialization speed.

Suppose a message consumer only needs to inspect a few fields before deciding what to do. A Protobuf implementation offering borrowed message views—such as Buffa—may be able to inspect string and byte fields directly from a SurrealMX value buffer.

That can avoid constructing a complete owned message during every lookup. However, the borrowed view cannot outlive the underlying buffer or callback's validity. Storing a view beyond its lifetime or using it after the store releases the borrowed bytes would violate Rust's ownership requirements.

Other combinations may still require copying: a decoded message inserted into a SurrealDB `Value::Object`, for example, generally needs independently owned values rather than references into transient KV storage.

#### Optional persistence

SurrealMX 1.1.0 defaults to in-memory operation but supports optional persistence using `Database::new_with_persistence` and `PersistenceOptions`.

Its facilities include append-only logging and snapshots, with modes such as:

- `AolMode::Never`
- `AolMode::SynchronousOnCommit`
- `AolMode::AsynchronousAfterCommit`

Snapshot and fsync behavior are configurable separately.

This permits different recovery and commit-latency characteristics for standalone use. It does not establish that SurrealDB's ordinary in-memory datastore exposes all of these standalone persistence options.

The engine also includes background cleanup and garbage collection, which can be configured and invoked explicitly. These are relevant to long-lived stores with frequently replaced serialized message values.

Primary source, tagged release:

`https://github.com/surrealdb/surrealmx/blob/v1.1.0/README.md`

### 7.3. Storage representation does not confer graph semantics

A Protobuf message with fields describing a node and several edges can be stored efficiently as one KV value. That does not make its relationships traversable by SurrealDB's native graph planner.

The distinction is structural:

| Representation                             | What the engine can interpret directly                                       |
| ------------------------------------------ | ---------------------------------------------------------------------------- |
| Protobuf bytes in standalone SurrealKV     | Key, value, version, transaction and range operations                        |
| Protobuf bytes in standalone SurrealMX     | Key/value operations, transactional versions and scans                       |
| Opaque Protobuf `bytes` field in SurrealDB | The record and any separately indexed native fields                          |
| Decoded SurrealDB nodes and relations      | Record identity, graph topology, indexes, traversal and database constraints |
| Hybrid native graph plus original Protobuf | Native graph over extracted relationships, with a retained original payload  |

A hybrid representation can expose native graph relationships while preserving an original binary message. It introduces a projection-consistency obligation: the indexed representation and original message must be updated coherently.

Where this work occurs across independently managed storage engines, their individual transactions do not automatically form one atomic transaction.

## 8. Graph projections, change visibility, and typed query alternatives

### 8.1. Lightweight relationships change the event model

SurrealDB's `LIGHTWEIGHT` relations do not store ordinary independent edge records. Instead, an edge has canonical identity determined by its endpoints, and the database synthesizes an edge-shaped result during reads.

This improves the correspondence between relation identity and topology, but deliberately removes several record-oriented capabilities.

A lightweight relation table cannot define fields, indexes, or events. It also cannot have a changefeed or receive `LIVE SELECT` subscriptions directly.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



This matters when designing Protobuf graph events.

A Protobuf message describing an edge addition or deletion may be perfectly valid, yet there is no corresponding conventional edge-record event that an integration can always observe through a lightweight relation table.

A graph event stream must therefore distinguish:

- Changes to ordinary property-bearing relation records.
- Changes to record references.
- Changes to lightweight, recordless topology.
- Changes inferred from a higher-level operation that modifies several relations.

The database can query all these relationships, but query visibility is not equivalent to uniform change-event availability.

### 8.2. Incremental materialized views have a dependency boundary

`DEFINE TABLE ... AS SELECT` offers maintained materialized projections and can reduce the need to implement and persist external denormalized views.

However, the maintenance model is not an unrestricted dependency-tracking system.

The official documentation distinguishes writes to the view's principal `FROM` table from updates to other records merely read by the query. A view may reference data from another table without automatically being refreshed when that other table changes.

Similarly, writes performed under `OPTION IMPORT` bypass view maintenance. A view can be repopulated using `DEFINE TABLE OVERWRITE`, which reruns its source query.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



For Protobuf-backed graph processing, this creates a consequential distinction:

A materialized Protobuf projection is not guaranteed current merely because its source query can reach all the records it depends on.

Its freshness depends on which changes trigger native maintenance.

When a typed external message is generated from a materialized view, its schema can be fully correct while its contents are stale relative to another related table.

SurrealDB 4.0 also allows a projected view `id` to determine the view-record key, including structured key forms, rather than necessarily inheriting the source-record key. This is relevant when an external Protobuf contract needs stable identity for an aggregate or derived entity.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 8.3. GraphQL is a third typed-interface boundary

Alongside native RPC and ISO GQL, SurrealDB offers a GraphQL endpoint with schema exposure configured through `DEFINE CONFIG GRAPHQL`.

This provides a typed, introspectable query API that can expose selected tables and functions. Its supported surface includes filters, ordering, aggregation, pagination, and generated mutation operations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



These three interfaces have different ownership models:

| Interface     | Contract owner                                        | Primary capability                            |
| ------------- | ----------------------------------------------------- | --------------------------------------------- |
| Protobuf/gRPC | Published RPC service or application-defined `.proto` | Typed message exchange and streaming          |
| GraphQL       | Schema generated from selected database definitions   | Typed, client-selected field projections      |
| ISO GQL       | Graph query language and database graph schema        | Graph patterns, paths, traversal and mutation |

GraphQL does not provide Protobuf binary interchange; Protobuf does not inherently provide GraphQL's client-controlled field selection; and neither replaces ISO GQL's graph pattern semantics.

A separate Protobuf projection can be unnecessary if the required integration is simply an introspectable typed query interface, while remaining useful when clients need fixed binary messages or strongly versioned service contracts.

Primary references:

`https://surrealdb.com/docs/reference/query-language/statements/define/table`

`https://surrealdb.com/docs/learn/querying/graphql/overview`

`https://surrealdb.com/docs/reference/query-language/statements/define/config`

`https://surrealdb.com/docs/learn/querying/gql/overview`

## 9. Descriptor-driven integration, schema governance, and validation

### 9.1. Dynamic schemas can reduce generated-model duplication

A compiled Protobuf `FileDescriptorSet` provides a machine-readable description of messages, fields, enums, services, dependencies, and custom options.

In Rust, `prost-reflect` exposes two particularly relevant abstractions:

- `DescriptorPool`: loads and resolves compiled schema definitions.
- `DynamicMessage`: encodes, decodes, inspects, and modifies messages using a runtime descriptor rather than a statically generated Rust type.

`DynamicMessage` also provides `transcode_from` and `transcode_to` for interoperability with compatible generated Prost messages. Unknown fields are preserved when a dynamic message is decoded and re-encoded.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



This opens a useful general integration pattern for heterogeneous SurrealDB data:

Compiled Protobuf descriptor set

Types, field numbers, options, constraints

`DescriptorPool` + `DynamicMessage`

Runtime type resolution and field inspection

Descriptor-aware conversion

Presence, enums, references, native type mapping

SurrealDB `Value`, records, and relations

The principal advantage is not that reflection makes conversion automatic. It is that a generic converter can use one runtime mechanism for many message types, while retaining schema-specific rules in descriptors or associated metadata.

That can avoid generating and maintaining a separate conversion implementation for every pair of application message and database record types.

The remaining work is semantic: Protobuf descriptors do not inherently say whether a field is a SurrealDB record reference, graph relation, decimal quantity, timestamp convention, or ordinary string.

### 9.2. Custom options and schema metadata

Protobuf custom options can attach metadata to schema elements such as messages, fields, enums, and services.

For example, an integration-specific descriptor extension could describe:

| Metadata            | Possible meaning                                                      |
| ------------------- | --------------------------------------------------------------------- |
| Record table        | Which SurrealDB table represents a message                            |
| Record identity     | Which fields contribute to a database record key                      |
| Reference target    | Whether a field represents another record                             |
| Edge relation       | Which relation table and direction correspond to a relationship       |
| Native type mapping | How to represent decimals, structured identifiers, or temporal values |
| Projection policy   | Whether a field is native, opaque, indexed, or excluded               |
| Validation rule     | Additional constraints enforced before persistence                    |
| Schema revision     | Which mapping contract applies to a particular message version        |

These are possible uses of standard Protobuf extensibility, not built-in SurrealDB schema annotations.

The metadata becomes operational only when a generator, reflection-based converter, or extension interprets it. Furthermore, permissions, uniqueness, and transactional integrity must still be enforced by the database.

For immutable archives or heterogeneous event systems, a useful accompanying mechanism is to retain the message's fully qualified type name and a descriptor-set revision or digest alongside its original encoded bytes.

The descriptor identifies the schema used to interpret the message. The digest is useful for identifying schema content, but it should not be confused with a digest of a canonically serialized Protobuf message: Protobuf binary serialization itself is not globally canonical.

### 9.3. Runtime behavior matters for unknown fields

The difference between Protobuf implementations is especially significant when messages persist longer than the software version that created them.

| Rust implementation | Relevant behavior                                                                                                     |
| ------------------- | --------------------------------------------------------------------------------------------------------------------- |
| `prost`             | Generated message types, efficient binary handling, preservation of unknown enum numeric values                       |
| `prost-reflect`     | Dynamic messages and descriptor-driven inspection; explicit unknown-field preservation through binary round-trips     |
| `buffa`             | Generated owned messages and borrowed views; explicit `UnknownFields` support and configurable preservation           |
| `pbjson`            | ProtoJSON-compatible mappings for supported generated types; not an unknown-field-preserving binary archive mechanism |

Buffa 0.9.2 exposes `UnknownFields`, `ExtensionSet`, borrowed `MessageView` types, and `OwnedView` backed by `Bytes`. Its generator enables unknown-field preservation by default, with an option to omit that storage when the application does not need it.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



These distinctions affect Protobuf data stored in SurrealKV or SurrealMX.

If an older consumer parses a future-schema message, changes one recognized field, and re-encodes it, preserving unknown fields can be essential to retaining information that the older consumer does not understand.

By contrast, converting the message into a new SurrealDB object containing only recognized fields generally discards those unrecognized fields unless the conversion explicitly preserves them separately.

A conversion through ProtoJSON likewise does not offer the same unknown-binary-field preservation guarantee.

For exact-message archival, retaining the original bytes is different from retaining a message that can be reconstructed approximately from known fields.

### 9.4. Protobuf Editions 2026 and generator compatibility

Protobuf Edition 2026 is available in the released Protobuf 36.x compiler family. Its relevant changes include stricter naming conventions, default symbol visibility, and schema-size limits. The official feature reference also describes the edition-specific defaults for field presence, enum behavior, and repeated-field encoding.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://protobuf.dev\&sz=32)

Protocol Buffers Documentation+1



However, compiler acceptance of an edition and complete support by a particular Rust generator are separate capabilities.

For example, a `protoc` release may understand Edition 2026 while a third-party generator does not yet implement all edition-specific options or generated API semantics.

This is particularly relevant for:

- `prost-build` and custom code generators.
- Buffa's editions-aware generation.
- Descriptor-driven validators.
- Generated code compiled for Surrealism's WASI target.
- Python consumers using a runtime with different edition support.

A descriptor set can sometimes serve as an interchange between compiler and generator, but it does not automatically resolve unsupported edition features.

The edition, `protoc` version, generator version, and runtime version should therefore be treated as independently versioned components.

Primary references:

`https://protobuf.dev/editions/features/`

`https://github.com/protocolbuffers/protobuf/releases/tag/v36.0`

`https://docs.rs/prost-reflect/latest/prost_reflect/`

`https://docs.rs/buffa/0.9.2/buffa/`

## 10. Partial updates, validation, and durable change streams

### 10.1. `FieldMask` is more nuanced than a list of fields

The original overview identified field presence as important but did not fully explain Protobuf's `FieldMask` update semantics.

The published `FieldMask` contract distinguishes fields included in an update from those left untouched. Its documented default behavior includes:

- Repeated fields may be appended rather than replaced.
- A terminal submessage may be merged with the existing submessage.
- Resetting a field to its default requires explicitly including that field in the mask.
- Omitting a mask can imply updating every field, potentially resetting fields unknown to an older client.
- An application may define different replacement or merge semantics where documented.&#x20;

  [image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

  GitHub



This matters because SurrealDB supports several update operations with their own semantics, including assignments, `MERGE`, and `NONE`-based field removal.

A Protobuf `FieldMask` is not automatically equivalent to any one SurrealQL update statement.

A faithful partial-update contract may need to distinguish:

| Intended operation     | Database meaning                               |
| ---------------------- | ---------------------------------------------- |
| Leave unchanged        | Do not modify the field                        |
| Assign a value         | Replace or update with a concrete value        |
| Assign `NULL`          | Preserve field existence with explicit null    |
| Remove                 | Assign `NONE` or otherwise delete the field    |
| Merge nested fields    | Update selected members while retaining others |
| Replace a collection   | Replace its entire array, map, or set          |
| Append to a collection | Extend the existing collection                 |

A discriminated Protobuf operation message can express these cases explicitly. A simple `optional` field cannot encode all of them because it only distinguishes absent from present.

This is a general semantic distinction between message field presence and database mutation intent.

### 10.2. Descriptor-level validation does not replace database constraints

Protovalidate provides validation rules attached to Protobuf schemas, using standard annotations and custom Common Expression Language (CEL) expressions.

Its documented capabilities cover numeric and string constraints, timestamps, durations, collection sizes, unique items, nested validation, enums, oneofs, field relationships, and allowed type URLs for `Any`. The annotations are interpreted by a compatible validator rather than being automatically enforced by the binary Protobuf parser.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://protovalidate.com\&sz=32)

Protovalidate



An important integration boundary is:

- Protobuf validation establishes properties of the incoming message.
- SurrealDB schema validation establishes properties of the stored database representation.
- Transaction-level validation establishes invariants involving other records and concurrent writes.

For example, a message validator may establish that a relationship references two nonempty record identifiers. It cannot establish that the referenced records still exist at the time the relation is committed unless it participates in a database-aware operation.

The official Protovalidate documentation identifies implementations for Go, JavaScript/TypeScript, Java, Python and C++. A supported Rust/WASI implementation was not established from those primary sources. It would therefore be incorrect to describe Protovalidate annotations as automatically executable inside a Surrealism Rust extension.

A Rust implementation would need an appropriate validation runtime or separately implemented rules.

### 10.3. Buf compatibility checks cover distinct failure classes

Buf's breaking-change checker defines four built-in categories, ordered from stricter generated-source compatibility toward binary wire compatibility.

| Category    | Protection                                          |
| ----------- | --------------------------------------------------- |
| `FILE`      | Generated source compatibility at the file level    |
| `PACKAGE`   | Generated source compatibility at the package level |
| `WIRE_JSON` | Binary Protobuf and ProtoJSON compatibility         |
| `WIRE`      | Binary Protobuf compatibility                       |

The categories do not provide equal guarantees. Renaming an existing Protobuf field can preserve its binary tag identity while breaking a JSON client or generated source code. Moving a message between files may be harmless to some consumers but break generated imports in others.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://buf.build\&sz=32)

Buf Docs



For SurrealDB, there is also a separate database-compatibility dimension.

A Protobuf message may pass Buf's binary compatibility checks while a corresponding `DEFINE TYPE`, field assertion, relation constraint, or index definition changes incompatibly.

Likewise, a database schema may remain valid while a Protobuf event changes its interpretation of a record's identity or a graph relationship.

Schema-governance tooling can detect many message-level incompatibilities, but it does not automatically establish database-level semantic compatibility.

Primary references:

`https://buf.build/docs/breaking/rules/`

`https://protovalidate.com/schemas/standard-rules/`

`https://protobuf.dev/programming-guides/field_presence/`

`https://protobuf.dev/reference/protobuf/google.protobuf/`

### 10.4. Changefeed replay requires separate database and message identities

SurrealDB changefeeds provide retained mutation history and versionstamps. `SHOW CHANGES` can replay changes from a specified point, subject to the changefeed's configured retention period.

The documented `SINCE` behavior includes the supplied starting versionstamp, making cursor advancement consequential. The upstream example stores the next cursor with the target-side database changes in one transaction.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



For an external Protobuf event stream, several different identities need to remain distinguishable:

| Identity                   | What it identifies                                        |
| -------------------------- | --------------------------------------------------------- |
| Database record ID         | The entity being changed                                  |
| Changefeed versionstamp    | Position in a source database's retained mutation history |
| Protobuf message type      | The schema used for a serialized message                  |
| Descriptor/schema revision | The precise interpretation of that message                |
| Domain event ID            | An externally meaningful operation or event occurrence    |
| Consumer acknowledgement   | Which externally delivered events have been processed     |

A versionstamp is not inherently a domain event ID, and an external message broker's offset is not a database transaction identifier.

Consequently, replay correctness includes more than decoding the Protobuf envelope successfully.

One subtle limitation is that SurrealDB's documented changefeed replication example applies record updates and deletions while handling schema definitions separately. A replicated event stream therefore should not be assumed to carry and apply every schema evolution automatically.

If a consumer falls behind the configured changefeed retention period, replay alone cannot recover the missing changes. A snapshot or equivalent state reconstruction is required before normal incremental processing resumes.

The native changefeed mechanism can reduce custom change detection, but does not alone guarantee exactly-once delivery across an external Protobuf transport.

Source:

`https://surrealdb.com/docs/learn/querying/real-time/changefeeds`

## 11. Remaining boundaries and unresolved questions

The mechanisms in this addendum provide a substantially richer capability picture, but several combinations still require explicit verification.

| Combination                                     | Established capability                                               | What remains unestablished                                         |
| ----------------------------------------------- | -------------------------------------------------------------------- | ------------------------------------------------------------------ |
| `surrealdb-types` + Protobuf                    | Native feature-gated encoding and decoding                           | Lossless conversion of every 4.0 native value                      |
| Generated application messages + `SurrealValue` | Existing Rust conversion traits                                      | Automatic mapping of arbitrary generated messages                  |
| Descriptors + `DEFINE TYPE`                     | Machine-readable schema metadata and named database types            | First-party, bidirectional schema generation                       |
| Surrealism + Protobuf reflection                | WASM extensions and Rust Protobuf runtimes exist                     | Specific dependency/target compatibility and snapshot behavior     |
| Surrealism + external transport                 | Controlled outbound WASI HTTP is available                           | General Tonic/gRPC client compatibility inside the sandbox         |
| Standalone SurrealMX + borrowed message views   | Borrowed value inspection and borrowed Protobuf views exist          | Zero-copy behavior through every subsequent database conversion    |
| Lightweight graph relations + event streaming   | Native relation traversal and deletion are supported                 | A conventional per-edge changefeed on a recordless relation        |
| Materialized views + Protobuf projections       | Incremental view maintenance and typed serialization can be composed | Refresh triggered by every dependency referenced in the view query |
| Protobuf validation + Surrealism                | Schema annotations and database extension execution exist            | A verified general-purpose Protovalidate runtime for Rust/WASI     |

These qualifications distinguish an implemented feature from an architectural combination that the available primitives make possible.

In particular, the existence of both a native Protobuf codec and an extension framework should not be interpreted as an end-to-end, first-party Protobuf schema-to-graph integration. The conversion between arbitrary messages and the database's record/relationship semantics remains a separate concern.

## 12. Addendum synthesis

Three distinctions materially extend the original overview.

Native Protobuf interchange is already implemented. SurrealDB 4.0 provides a specific Protobuf representation of its own database values and public Rust encode/decode functions. The most important technical question is not whether such a codec exists, but which values it preserves exactly.

Protobuf can operate at multiple execution and storage boundaries. Application services, custom HTTP APIs, Surrealism WASM extensions, the standalone SurrealKV/SurrealMX engines, and the native database protocol are different integration points. Each offers different ownership, transaction, durability, and performance characteristics.

Schema interoperability is more demanding than wire compatibility. Descriptors, generated types, validation rules, Buf compatibility checks, SurrealDB named types, graph relationships, and transactional constraints govern different aspects of correctness. They can reduce duplicated machinery when composed deliberately, but none universally substitutes for the others.

The central additional capability is therefore schema-aware transformation between independently governed Protobuf contracts and SurrealDB's native value, graph, and transactional models, while using already available codecs, storage facilities, and extension points wherever their guarantees are sufficient.