# `opentelemetry_otlp::exporter::ExportConfig`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_otlp.exporter.ExportConfig.json).

<a id="op-cdfcaa8d369d011331bae7f7"></a>
## ExportConfig

`struct` · `opentelemetry_otlp::exporter::ExportConfig` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct ExportConfig
```

Source: `src/exporter/mod.rs:70`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Configuration for the OTLP exporter.

<a id="op-e532aad1fafb772593343324"></a>
## default

`function` · `opentelemetry_otlp::exporter::ExportConfig::default` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::exporter::ExportConfig", "path": "ExportConfig"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [87, 1], "end": [99, 2], "filename": "src/exporter/mod.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/exporter/mod.rs:88`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2f0d48924bf485946c436c48"></a>
## endpoint

`struct_field` · `opentelemetry_otlp::exporter::ExportConfig::endpoint` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
endpoint: Option<String>
```

Source: `src/exporter/mod.rs:75`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

The address of the OTLP collector.
Default address will be used based on the protocol.

Note: Programmatically setting this will override any value set via the environment variable.

<a id="op-62645f68165a65650396d560"></a>
## fmt

`function` · `opentelemetry_otlp::exporter::ExportConfig::fmt` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::exporter::ExportConfig", "path": "ExportConfig"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [69, 10], "end": [69, 15], "filename": "src/exporter/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/exporter/mod.rs:69`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-94164e0cdfb211524b9ae30c"></a>
## protocol

`struct_field` · `opentelemetry_otlp::exporter::ExportConfig::protocol` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
protocol: Protocol
```

Source: `src/exporter/mod.rs:78`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

The protocol to use when communicating with the collector.

<a id="op-3c9ce635c145057f395c35e9"></a>
## timeout

`struct_field` · `opentelemetry_otlp::exporter::ExportConfig::timeout` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
timeout: Option<std::time::Duration>
```

Source: `src/exporter/mod.rs:84`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

The timeout to the collector.
The default value is 10 seconds.

Note: Programmatically setting this will override any value set via the environment variable.
