# grpcio

Realizations considered: not installed. Upgrade route `grpcio` 1.85.0 (PyPI, released
2026-10-26; one day old at research time). The installed OTLP gRPC stubs need
`grpcio>=1.63.2`.
Research date 2026-10-27. Sources: PyPI JSON; installed
`opentelemetry/proto/collector/trace/v1/trace_service_pb2_grpc.py` (imports `grpc`, sets
`GRPC_GENERATED_VERSION = '1.63.2'`). The gRPC server API (`grpc.server`,
`ThreadPoolExecutor`, `add_insecure_port`, servicer registration) is documentation
knowledge.

## Types

### grpcio/python-grpc-server-for-stub-services
- **Need it serves:** Receive or issue gRPC calls in Python tooling with generated service
  stubs, instead of implementing HTTP/2 framing, HPACK and gRPC trailers by hand.
- **Recognizable code shapes:**
  - Socket servers that check the `PRI * HTTP/2.0` preface.
  - Hand-emitted SETTINGS, PING and WINDOW_UPDATE frames.
  - Byte-literal HPACK headers such as `\x88` and `grpc-status`.
  - gRPC 5-byte length prefixes.
- **Mechanisms and APIs:**
  - `grpc.server(futures.ThreadPoolExecutor(...), options=[("grpc.max_receive_message_length", N)])`.
  - `add_TraceServiceServicer_to_server(servicer, server)`, then
    `server.add_insecure_port("127.0.0.1:0")` (returns the port), `server.start()` and
    `server.stop(grace)`.
  - Servicer methods receive decoded request messages.
- **Obligations discharged:**
  - HTTP/2, HPACK, flow control, gRPC framing, status and trailers, and message decoding
    through stubs.
  - Concurrent streams.
- **Residual caller obligations:**
  - Capture bounds: span counts and totals.
  - Lifecycle within a finite capture window.
  - Options for receive limits.
- **Semantic conditions:**
  - It runs its own C-core threads. Shutdown must be explicit to avoid lingering threads.
  - The default max receive size is 4 MiB unless configured.
- **Version / build conditions:**
  - A large compiled wheel.
  - Not stdlib. Must not appear on cold-start paths.
- **Limits and negative results:** Adds a heavyweight dependency for a diagnostic sink.
- **Evidence:** Interface-checked: PyPI metadata and installed stub imports. Server API is
  documentation knowledge.

## Current applicability

### [CBO-051](../../../library_leverage/codebase/developer-tooling.md#cbo-051) — Hand-written wire parsers in tooling (transport part, a manifestation not named by the CBO)
- **Types:** `grpcio/python-grpc-server-for-stub-services`.
- **Realization:** Upgrade route, a new dependency.
- **Conclusion:** Fits with conditions, as the complete replacement of the hand HTTP/2 OTLP
  receiver.
  - Weighed against the premises "tooling changes stay light" and the existing finite,
    bounded sink, it is the heavier option.
  - An OTLP/HTTP switch, if the server supports it, avoids gRPC entirely.
  - A sans-IO HTTP/2 library is the middle option.
- **What it would discharge / what remains local:**
  - Discharged: all HTTP/2 and gRPC framing and the protobuf walk.
  - Remains local: capture bounds and lifecycle.
- **Prerequisites, configuration or extension:** A `grpcio` pin compatible with the stub
  version; receive-size options.
- **Semantic gaps against the CBO's preservation constraints:**
  - *Bounded reads:* expressible through options plus servicer counting.
  - *Stdlib-only cold start:* not affected, because the sink is not a hook path.
- **Unsettled premises and what would settle them:** The OTLP/HTTP switch premise (see the
  OTLP message-decoding record and the probe request).
- **Evidence:** Interface-checked; application Proposed.

**Reviewer correction (2026-10-27).** The OTLP/HTTP switch premise is answered no and its probe is withdrawn: at tag `v3.3.4` the
SurrealDB server builds its span exporter with `with_tonic()` and compiles `opentelemetry-otlp`
0.31.1 without `http-proto`, and no protocol variable selects HTTP (see
`surrealdb-server/telemetry-export-transport` in the [SurrealDB server tooling](../surrealdb/server-tooling.md)
account). The repository already points the server at the collector's `otlp_grpc_endpoint`. Of the three options above, the OTLP/HTTP switch is therefore unavailable. The comparative review retains the hand sink as the minimal fixed peer rather than adopting `grpcio`. See the [comparative review](../../reviews/developer-tooling.md#cbo-051).
