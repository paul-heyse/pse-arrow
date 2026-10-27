# h2 (python-hyper)

Realizations considered: not installed. Upgrade route `h2` 4.4.1 (PyPI, released
2026-08-03). It is pure Python and depends on `hpack` and `hyperframe`.
Research date 2026-10-27. Sources: PyPI JSON. The `H2Connection` API
(`initiate_connection`, `receive_data` returning events, `send_headers`, `send_data`,
`end_stream`, `acknowledge_received_data`, `data_to_send`) is documentation knowledge and
was not re-read at 4.4.1.

## Types

### h2/sans-io-http2-protocol
- **Need it serves:** Get HTTP/2 framing, HPACK and flow control right over a socket the
  caller owns, without adopting a server framework.
- **Recognizable code shapes:**
  - Hand-built HTTP/2 frames: a 9-byte header and frame-type constants.
  - Byte-literal HPACK headers.
  - Manual WINDOW_UPDATE emission.
- **Mechanisms and APIs:**
  - `h2.connection.H2Connection(config=h2.config.H2Configuration(client_side=False))`.
  - `conn.receive_data(bytes)` returns events: `RequestReceived`, `DataReceived`,
    `StreamEnded` and `ConnectionTerminated`.
  - Responses go through `send_headers`, `send_data` and trailers via
    `send_headers(..., end_stream=True)`.
  - `conn.data_to_send()` returns outbound bytes.
  - `acknowledge_received_data` handles flow control.
- **Obligations discharged:** HTTP/2 state machine, HPACK encode and decode, and
  flow-control accounting.
- **Residual caller obligations:**
  - Socket I/O and timeouts.
  - gRPC length-prefix framing and status trailers.
  - Size bounds.
  - Message decoding.
- **Semantic conditions:** The library validates HTTP/2 strictly. Peers that a
  hand-written server tolerates may be rejected.
- **Version / build conditions:** Pure Python. Not stdlib.
- **Limits and negative results:** No gRPC semantics.
- **Evidence:** Interface-checked for the version (PyPI). API is documentation knowledge.

## Current applicability

### [CBO-051](../../../library_leverage/codebase/developer-tooling.md#cbo-051) — Hand-written wire parsers in tooling (transport part, a manifestation not named by the CBO)
- **Types:** `h2/sans-io-http2-protocol`.
- **Realization:** Upgrade route, a new dependency.
- **Conclusion:** Fits with conditions, as the middle option between keeping the hand
  HTTP/2 receiver and adopting a full gRPC server library. It keeps the existing
  socketserver, deadlines and bounds, and replaces only frame and HPACK handling. Combined
  with the installed OTLP message stubs, it removes both hand protocol layers of the
  capture sink.
- **What it would discharge / what remains local:**
  - Discharged: the frame loop and static header bytes.
  - Remains local: gRPC 5-byte framing, capture bounds and lifecycle.
- **Prerequisites, configuration or extension:** A pin in the group that runs diagnostic
  capture.
- **Semantic gaps against the CBO's preservation constraints:** Bounded reads stay local.
  The stdlib cold-start constraint does not apply to this path.
- **Unsettled premises and what would settle them:** Whether the producer can export
  OTLP/HTTP instead. If it can, neither HTTP/2 library is needed. See the probe request in
  this cluster's README.
- **Evidence:** Interface-checked (PyPI); application Proposed.
