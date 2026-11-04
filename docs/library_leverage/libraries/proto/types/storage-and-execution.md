# Native value codecs and streaming execution contracts

Research date: 2026-11-03. These reusable types describe library mechanisms, not an adoption
decision or qualified application integration. Exact installed implementations were read;
no executable probe or performance campaign ran.

## surrealdb-proto/native-value-interchange

**Need.** Encode rich database values without flattening their distinctions into ordinary
JSON or starting a database RPC connection. Recognizable consumers include native-value
fixtures, database-aware gateways, and explicit native-value interchange files.

**Mechanism.** `surrealdb-types` 3.3.4's optional `proto` feature exports `encode_proto` and
`decode_proto`. Encoding performs `Value → protocol::proto::v1::Value → Prost bytes`;
decoding performs Prost parsing followed by fallible native conversion. Encoding clones the
input Value. The companion protocol examined is `surrealdb-protocol` 0.13.1.

**Discharged obligations.** Published representations for native none/null, number kinds,
binary data, record IDs, collections, temporal values and other supported native variants;
fallible validation of conversion into the supported native value universe.

**Residual obligations.** Input admission before owned decode; accepted numeric and temporal
domains; duplicate-key policy; retained codec identity; exact-byte archive policy; domain
validation; record/table selection; transaction and effect semantics. The codec does not
interpret arbitrary application message schemas or create database records.

**Consequential conditions.** Native duration is unsigned while the protocol can represent
negative durations; the conversion may reject them. A protocol alternative can be parseable
without having a native 3.3 representation. Object conversion inserts entries into a map,
normalizing order and allowing later duplicates to replace earlier ones. Do not infer strict
wire uniqueness checks from the protocol schema alone. Native integer/decimal/float choices
and structured record identity must be retained deliberately through all later adapters.

**Evidence.** **Interface-checked** installed 3.3.4 `src/proto/{mod,value,record}.rs` and public
exports. Public sources: [codec](https://docs.rs/crate/surrealdb-types/3.3.4/source/src/proto/mod.rs),
[value conversion](https://docs.rs/crate/surrealdb-types/3.3.4/source/src/proto/value.rs),
[record conversion](https://docs.rs/crate/surrealdb-types/3.3.4/source/src/proto/record.rs) and
[protocol schema](https://github.com/surrealdb/surrealdb-protocol/blob/v0.13.1/surrealdb/protocol/v1/value.proto).
This evidence does not establish canonical encoding, bit-preserving arbitrary archives,
bounded total heap use or an assembled persistent store.

## connectrpc/terminal-stream-and-native-settlement

**Need.** Consume useful partial messages without confusing transport completion, RPC success,
application acknowledgement and physical worker completion. Recognizable consumers include
long-running operation observers, provider workers and selected-result streams.

**Mechanism.** ConnectRPC 0.9.1 `ServerStream::message()` returns a message, successful terminal
`None`, or terminal `ConnectError`. A terminal error remains sticky; `error()` preserves it
and `trailers()` exposes available termination metadata. `BidiStream::into_split()` separates
send and receive ownership. `BidiSendHalf::close_send` ends input; dropping the receive half
cancels the RPC. `ClientRequestStream` asynchronously backs the outgoing HTTP body.

**Discharged obligations.** Protocol-specific framing and terminal-status interpretation,
typed stream consumption, HTTP-body backpressure, half-close mechanics and 0.9.1's bounded
cleanup of abandoned streaming uploads.

**Residual obligations.** Retained-prefix interpretation; cumulative stream quotas and retained
state; lazy/bounded application producers; operation identity, commit acknowledgement and
retry policy; native cancellation/drain; independent cleanup failures. Dropping a response
stream cannot establish that a blocking or detached task exited or that a durable effect
rolled back.

**Deadline composition.** `DeadlinePolicy::with_max` and `with_default_timeout` bound the
configured request/handler lifetime. `with_enforce_on_streams(true)` extends absolute
enforcement to response streams; `with_inter_message_timeout` bounds silence between yielded
items. Defaults do not supply these stream bounds. `RequestContext::time_remaining()` can
derive a downstream `CallOptions::with_timeout` budget, leaving application time for local
completion and response. The wrapper drops a stream at a yield point; native work ownership
must remain elsewhere until actual settlement.

**Limit composition.** `Limits::with_max_request_body_size` bounds unary/server-streaming
input bodies. `with_max_message_size` bounds each message including decompression expansion;
`with_element_memory_limit` bounds binary decode element amplification. Neither latter
setting caps cumulative client/bidi upload or the application's retained message population.
JSON materialization has different pre-allocation behavior. Independent clients configure
response message and element limits using `ClientConfig` defaults or `CallOptions` overrides.

**Evidence.** **Interface-checked** installed 0.9.1 `client/mod.rs`, `service.rs`, `deadline.rs`
and the tagged guide/changelog. Sources: [ServerStream](https://docs.rs/connectrpc/0.9.1/connectrpc/client/struct.ServerStream.html),
[DeadlinePolicy](https://docs.rs/connectrpc/0.9.1/connectrpc/struct.DeadlinePolicy.html),
[Limits](https://docs.rs/connectrpc/0.9.1/connectrpc/struct.Limits.html),
[guide](https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md) and
[release corrections](https://github.com/connectrpc/connect-rust/blob/v0.9.1/CHANGELOG.md).
No application cancellation/settlement or throughput claim follows from these interfaces.
