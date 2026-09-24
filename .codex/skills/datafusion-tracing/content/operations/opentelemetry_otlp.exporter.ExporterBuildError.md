# `opentelemetry_otlp::exporter::ExporterBuildError`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_otlp.exporter.ExporterBuildError.json).

<a id="op-b060eec7d4af2fb4de600596"></a>
## ExporterBuildError

`enum` · `opentelemetry_otlp::exporter::ExporterBuildError` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
enum ExporterBuildError
```

Source: `src/exporter/mod.rs:107`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Errors that can occur while building an exporter.

<a id="op-b01bb92110a0d10f2c5e0ea5"></a>
## InternalFailure

`variant` · `opentelemetry_otlp::exporter::ExporterBuildError::InternalFailure` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
InternalFailure
```

Source: `src/exporter/mod.rs:136`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Failed due to an internal error.
The error message is intended for logging purposes only and should not
be used to make programmatic decisions. It is implementation-specific
and subject to change without notice. Consumers of this error should not
rely on its content beyond logging.

<a id="op-574cfa5e5d04bfb0f6ba323d"></a>
## InvalidUri

`variant` · `opentelemetry_otlp::exporter::ExporterBuildError::InvalidUri` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
InvalidUri
```

Source: `src/exporter/mod.rs:128`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Invalid URI.

<a id="op-c4e422f2d343213b2cc73343"></a>
## NoHttpClient

`variant` · `opentelemetry_otlp::exporter::ExporterBuildError::NoHttpClient` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
NoHttpClient
```

Source: `src/exporter/mod.rs:119`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No Http client specified.

<a id="op-89c9e372ef6ea566bfddaa7b"></a>
## ThreadSpawnFailed

`variant` · `opentelemetry_otlp::exporter::ExporterBuildError::ThreadSpawnFailed` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
ThreadSpawnFailed
```

Source: `src/exporter/mod.rs:110`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Spawning a new thread failed.

<a id="op-ab2d04667036f21c47165b98"></a>
## UnsupportedCompressionAlgorithm

`variant` · `opentelemetry_otlp::exporter::ExporterBuildError::UnsupportedCompressionAlgorithm` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
UnsupportedCompressionAlgorithm
```

Source: `src/exporter/mod.rs:123`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Unsupported compression algorithm.

<a id="op-0fef280c16edd5b6a3fcd7bd"></a>
## fmt

`function` · `opentelemetry_otlp::exporter::ExporterBuildError::fmt` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::exporter::ExporterBuildError", "path": "ExporterBuildError"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [101, 17], "end": [101, 22], "filename": "src/exporter/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/exporter/mod.rs:101`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-29433435f0f85cc5a644bff2"></a>
## fmt

`function` · `opentelemetry_otlp::exporter::ExporterBuildError::fmt` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, __formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::exporter::ExporterBuildError", "path": "ExporterBuildError"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [101, 10], "end": [101, 15], "filename": "src/exporter/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Display", "path": "Display"}, "trait_path": "core::fmt::Display"}`

Source: `src/exporter/mod.rs:101`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
