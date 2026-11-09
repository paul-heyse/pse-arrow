# Selected v4 runtime capability contracts

Inspected 2026-11-09. These reusable notes separate supplier behavior from contextual
research in the [discovery briefing](briefing.md). **Interface-checked** means source or
documentation inspection; no runtime probe, compatibility campaign or measurement ran.

## Native values: encoding and decoding have different fidelity

`surrealdb-types` 4.0.0 with `proto` exports `encode_proto` and `decode_proto` for its
own database-value schema. The encoder clones a native value, converts it into protocol
`Value`, then invokes Prost. It does not map arbitrary application messages into records.
This is an owned conversion rather than a borrowed, pre-allocation-admitting decoder.
[Codec entry points](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/types/src/proto/mod.rs)

The tagged native encoder uses widened numeric forms, degrades the new temporal families,
and falls back to text for BigDecimal outside its supported decimal representation.
Its decoder accepts explicitly tagged sized numbers with range checks, but rejects an
unset value variant and an unparsed string record ID. Thus decoder support does not imply
encoder round-trip preservation of type identity. These are implementation observations
at 4.0.0, not guarantees about a later codec.
[Conversion source](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/types/src/proto/value.rs)

Native Protobuf values, application messages and database records remain three different
representations. Unknown application fields can be lost during generated decoding or a
subsequent native projection even where an earlier stage preserves them. Keeping original
bytes can serve archival forwarding; those bytes do not automatically provide queryable fields.

The protocol repository separately publishes value schemas, test vectors and behavioral
conformance infrastructure. Its parity configuration records deliberate Protobuf/FlatBuffers
differences, including a `Kind` correspondence gap. Use conversion-specific evidence rather
than assuming every union arm or numeric tag is interchangeable.
[Protocol 0.14.1 parity](https://github.com/surrealdb/surrealdb-protocol/blob/v0.14.1/parity.toml)
and [conformance](https://github.com/surrealdb/surrealdb-protocol/tree/v0.14.1/conformance).

## Extension execution: prepared initialization and call-local state

Current official documentation describes a WASI Preview 2 component archive, build-time
`init` snapshot and a fresh instance for every invocation. Mutable memory from one call is
discarded. A call-scoped `Transaction` participates in its enclosing database transaction;
its database writes follow that transaction's commit or rollback. Immutable initialization
can be amortized; persistent application state needs another owner.
[Extension architecture](https://surrealdb.com/docs/learn/extensions/guides/extension-architecture)

The same documentation specifies concurrency, execution-time and memory controls. Those
controls do not establish physical teardown completion after cancellation. Default waiting
admission and any additional fail-fast or process-wide custody policy need separate evidence.
External network effects remain outside database rollback.

Extension definitions bind archive provenance and execution capabilities. The extension
reference still documents experimental enablement. Check the selected SDK, runtime and ABI
together, especially when using `BINDINGS`; a runtime supporting a feature does not prove
the selected build emits it. Rebuild older extension archives for the v4 boundary.
[DEFINE EXTENSION](https://surrealdb.com/docs/reference/query-language/statements/define/extension)

## API authority differs from invocation permission

`DEFINE API` predates v4. Invocation permission and handler execution authority are separate:
the current documented defaults are `PERMISSIONS FULL` and `SECURITY DEFINER`. v4 adds
`SECURITY INVOKER`; failed handler database mutations roll back. A binary HTTP body still
requires a decoder and application interpretation. Invoker security does not eliminate
endpoint authentication, body limits or external-effect coordination.
[DEFINE API](https://surrealdb.com/docs/reference/query-language/statements/define/api)

## Streams: notification, replay and operation completion

Live queries deliver committed record changes under subscriber permissions and do not promise
a total order across concurrent writers. A filter describing only the new matching state can
miss an update leaving that state; `$before` can matter. These are observation semantics,
not proof that a separately requested mutation was acknowledged or physically settled.
[Live query guide](https://surrealdb.com/docs/learn/querying/real-time/live-queries)

Changefeeds must be enabled and retained. The current guide documents inclusive `SINCE`,
transactional application of destination data plus cursor, and reconstruction when retention
has expired. A versionstamp covers a batch of changes; it is not an application event ID.
An external reader still owns transfer, filtering, duplicate handling and progress.
[Changefeed guide](https://surrealdb.com/docs/learn/querying/real-time/changefeeds)

## Scope and source limitations

The current public CBOR page fetched on the inspection date exposed legacy tags rather than
the supplied v4 reference's complete `cbor.2` account. The in-memory deployment page exposed
3.x/2.x material. Those pages are orientation, not decisive v4 fidelity/durability evidence.
Future transport selection needs the selected tagged encoder/server path and actual client
negotiation checked together. The notes above are deliberately limited to the inspected
interfaces and do not qualify whole transports, storage engines or cancellation paths.
