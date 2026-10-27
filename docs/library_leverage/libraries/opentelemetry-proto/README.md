# opentelemetry-proto (with protobuf)

Realizations considered:
- Installed `opentelemetry-proto` 1.45.1 and `protobuf` 7.36.2 in `.venv`. Both are
  transitive: `opentelemetry-exporter-otlp-proto-http` 1.45.1 pulls them in, and that
  package is a direct pin in the `quality` dependency group. Neither is a base dependency.
- The proto package requires `protobuf>=5.0,<8.0`.
- Its `*_pb2_grpc.py` modules import `grpc`, which is not installed, and were generated
  against `grpcio>=1.63.2`.

Research date 2026-10-27. Sources:
- Installed files:
  - `opentelemetry/proto/collector/trace/v1/{trace_service_pb2.py,.pyi,_pb2_grpc.py}`
    (`ExportTraceServiceRequest` and `ExportTraceServiceResponse` message classes;
    `GRPC_GENERATED_VERSION = '1.63.2'`);
  - `opentelemetry_proto-1.45.1.dist-info/METADATA`.
- `uv tree --invert` for provenance.
- protobuf Python message API (`Message.FromString` and `ParseFromString`, `HasField`,
  repeated fields) is documentation knowledge.

## Types

### opentelemetry-proto/otlp-message-decoding
- **Need it serves:** Decode OTLP export payloads (traces, metrics and logs requests) into
  typed messages instead of walking protobuf field numbers by hand.
- **Recognizable code shapes:**
  - A small varint and length-delimited reader returning `dict[int, list[bytes | int]]`.
  - Nested `fields(x).get(1)`, `.get(2)` and `.get(5)` lookups with comments naming OTLP
    fields.
  - Length checks for 16-byte trace ids and 8-byte span ids.
- **Mechanisms and APIs:**
  - `from opentelemetry.proto.collector.trace.v1.trace_service_pb2 import
    ExportTraceServiceRequest`.
  - `req = ExportTraceServiceRequest.FromString(body)`, then iterate
    `req.resource_spans[i].scope_spans[j].spans[k]` with `.trace_id`, `.span_id`,
    `.parent_span_id` and `.name`.
  - The matching metrics and logs request types live in sibling packages.
  - `*_pb2_grpc.TraceServiceServicer` and `add_TraceServiceServicer_to_server` exist for a
    gRPC server, but require `grpcio`.
- **Obligations discharged:**
  - Wire decoding: varints, length-delimited fields, packed repeated fields and unknown
    fields.
  - Schema-correct field typing.
  - Forward compatibility with new OTLP fields.
- **Residual caller obligations:**
  - Size bounds before decoding. protobuf has a default recursion limit, but no
    caller-chosen byte budget beyond what the transport enforces.
  - Validation of id lengths.
  - Error mapping. A `DecodeError` replaces the local `RuntimeError`.
- **Semantic conditions:**
  - Unknown fields are retained, not rejected. A strict reader that refuses unknown wire
    kinds is stricter than protobuf.
  - Decoding is whole-message, with no streaming.
  - Malformed input raises `google.protobuf.message.DecodeError`.
- **Version / build conditions:**
  - protobuf 7.x uses the upb C backend by default.
  - Import cost is that of `google.protobuf` plus generated descriptors. It is unsuitable
    for stdlib-only cold-start paths.
  - Availability depends on the `quality` group being installed.
- **Limits and negative results:**
  - Messages only: no HTTP/2 or gRPC transport. The `_pb2_grpc` modules need `grpcio`.
  - Does not cover non-OTLP protocols, for example a database's own gRPC API, without that
    protocol's `.proto` stubs.
- **Evidence:** Interface-checked: installed stub files and dist metadata. Message API is
  documentation knowledge.

## Current applicability

### [CBO-051](../../../library_leverage/codebase/developer-tooling.md#cbo-051) — Hand-written wire and front-matter parsers in tooling (wire part)
- **Types:** `opentelemetry-proto/otlp-message-decoding`.
- **Realization:** Installed transitively at 1.45.1 with protobuf 7.36.2, in the `quality`
  group. A direct pin would be needed if it became relied on. No version move.
- **Conclusion:**
  - Fits with conditions for the OTLP span decode in the telemetry capture sink.
  - Ruled out for the hand gRPC client of the database in a test helper, which needs that
    service's protobuf stubs and is outside OTLP.
- **What it would discharge / what remains local:**
  - Discharged: the four-level `fields()` walk in the capture `observe()` path, which
    becomes typed attribute access.
  - Remains local:
    - gRPC length-prefix framing: the 5-byte prefix and the compression flag check.
    - The bounded capture totals (16 MiB and 4096 spans).
    - The hand-written HTTP/2 server framing: preface check, SETTINGS, PING and WINDOW_UPDATE
      replies, and static HPACK-encoded response headers.
    - The test helper's SurrealDB gRPC client.
    - The shared varint reader. It stays for that client unless SurrealDB protocol stubs
      are generated for Python.
- **Prerequisites, configuration or extension:**
  - A direct pin of `opentelemetry-proto` in the group that runs the capture.
  - The capture must not sit on a stdlib-only cold-start path. It is an explicit diagnostic
    operation, so this appears satisfied (static reading).
- **Semantic gaps against the CBO's preservation constraints:**
  - *Bounded reads:* the transport bounds stay local, and protobuf decodes a bounded body.
  - *Duplicate-key rejection* concerns front matter, not this decoder.
  - Unknown-field tolerance is more permissive than the local "unsupported wire kind"
    refusal. That is acceptable for an observation sink, but it is a semantic change.
- **Unsettled premises and what would settle them:**
  - Whether the server's OTLP exporter can be switched to `http/protobuf`. The
    environment-variable surface used to point the server at the sink is the standard
    `OTEL_EXPORTER_OTLP_*_ENDPOINT`, and the local telemetry config already defaults other
    producers to `http/protobuf`. If it can, a stdlib `http.server` handler plus this
    decoder would replace the hand HTTP/2 server entirely.
  - A probe that sets `OTEL_EXPORTER_OTLP_TRACES_PROTOCOL=http/protobuf` on one diagnostic
    capture and observes whether the sink receives HTTP/1.1 `POST /v1/traces` would settle
    it.
- **Evidence:** Interface-checked (installed files); application Proposed.

**Reviewer correction (2026-10-27).** The OTLP/HTTP switch premise is answered no and its probe is withdrawn: at tag `v3.3.4` the
SurrealDB server builds its span exporter with `with_tonic()` and compiles `opentelemetry-otlp`
0.31.1 without `http-proto`, and no protocol variable selects HTTP (see
`surrealdb-server/telemetry-export-transport` in the [SurrealDB server tooling](../surrealdb/server-tooling.md)
account). The repository already points the server at the collector's `otlp_grpc_endpoint`. The hand HTTP/2 sink is therefore the minimal fixed peer, and the comparative review retains it, adding only a one-line reason to the module docstring; an upstream HTTP/protobuf exporter would reopen the stdlib-server option once released. See the [comparative review](../../reviews/developer-tooling.md#cbo-051).
