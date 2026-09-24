# `opentelemetry_sdk::trace::sampler::Sampler`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.trace.sampler.Sampler.json).

<a id="op-18207afc94b6ea2d542774ef"></a>
## Sampler

`enum` · `opentelemetry_sdk::trace::sampler::Sampler` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
enum Sampler
```

Source: `src/trace/sampler.rs:113`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Default Sampling options

The [built-in samplers] allow for simple decisions. For more complex scenarios consider
implementing your own sampler using [`ShouldSample`](../operations/opentelemetry_sdk.trace.sampler.ShouldSample.md#op-e261fe03f8ec2adecc6ef339) trait.

[built-in samplers]: https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/trace/sdk.md#built-in-samplers

<a id="op-099305d5a2529818e642044d"></a>
## AlwaysOff

`variant` · `opentelemetry_sdk::trace::sampler::Sampler::AlwaysOff` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
AlwaysOff
```

Source: `src/trace/sampler.rs:117`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Never sample the trace

<a id="op-8dd982b7a8af9c99bd4f4e15"></a>
## AlwaysOn

`variant` · `opentelemetry_sdk::trace::sampler::Sampler::AlwaysOn` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
AlwaysOn
```

Source: `src/trace/sampler.rs:115`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Always sample the trace

<a id="op-00ae4e1c81a6cd03db04eafd"></a>
## JaegerRemote

`variant` · `opentelemetry_sdk::trace::sampler::Sampler::JaegerRemote` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
JaegerRemote
```

Source: `src/trace/sampler.rs:135`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Jaeger remote sampler supports any remote service that implemented the jaeger remote sampler protocol.
The proto definition can be found [here](https://github.com/jaegertracing/jaeger-idl/blob/main/proto/api_v2/sampling.proto)

Jaeger remote sampler allows remotely controlling the sampling configuration for the SDKs.
The sampling is typically configured at the collector and the SDKs actively poll for changes.
The sampler uses TraceIdRatioBased or rate-limited sampler under the hood.
These samplers can be configured per whole service (a.k.a default), or per span name in a
given service (a.k.a per operation).

<a id="op-8a6496c3581e23ccdf800b6b"></a>
## ParentBased

`variant` · `opentelemetry_sdk::trace::sampler::Sampler::ParentBased` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
ParentBased
```

Source: `src/trace/sampler.rs:119`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Respects the parent span's sampling decision or delegates a delegate sampler for root spans.

<a id="op-688ce76fdf8cce4d2992018b"></a>
## TraceIdRatioBased

`variant` · `opentelemetry_sdk::trace::sampler::Sampler::TraceIdRatioBased` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
TraceIdRatioBased
```

Source: `src/trace/sampler.rs:125`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Sample a given fraction of traces. Fractions >= 1 will always sample. If the parent span is
sampled, then it's child spans will automatically be sampled. Fractions < 0 are treated as
zero, but spans may still be sampled if their parent is.
*Note:* If this is used then all Spans in a trace will become sampled assuming that the
first span is sampled as it is based on the `trace_id` not the `span_id`

<a id="op-84ede8dcd29f25572d64eab8"></a>
## clone

`function` · `opentelemetry_sdk::trace::sampler::Sampler::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Sampler
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::sampler::Sampler", "path": "Sampler"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [111, 10], "end": [111, 15], "filename": "src/trace/sampler.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/trace/sampler.rs:111`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-83aa7e31c13e93ec0c51c142"></a>
## fmt

`function` · `opentelemetry_sdk::trace::sampler::Sampler::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::sampler::Sampler", "path": "Sampler"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [111, 17], "end": [111, 22], "filename": "src/trace/sampler.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/sampler.rs:111`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d65429cc175ff395f3d1ab43"></a>
## jaeger_remote

`function` · `opentelemetry_sdk::trace::sampler::Sampler::jaeger_remote` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn jaeger_remote<C, Sampler, R, Svc>(runtime: R, http_client: C, default_sampler: Sampler, service_name: Svc) -> JaegerRemoteSamplerBuilder<C, Sampler, R> where C: HttpClient + 'static, Sampler: ShouldSample, R: runtime::RuntimeChannel, Svc: Into<String>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::sampler::Sampler", "path": "Sampler"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [138, 1], "end": [163, 2], "filename": "src/trace/sampler.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/sampler.rs:149`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Create a jaeger remote sampler builder.

### Arguments
* `runtime` - A runtime to run the HTTP client.
* `http_client` - An HTTP client to query the sampling endpoint.
* `default_sampler` - A default sampler to make a sampling decision when the remote is unavailable or before the SDK receives the first response from remote.
* `service_name` - The name of the service. This is a required parameter to query the sampling endpoint.

See [here](https://github.com/open-telemetry/opentelemetry-rust/blob/main/examples/jaeger-remote-sampler/src/main.rs) for an example.

<a id="op-4de600f9e044e688e182fcfc"></a>
## should_sample

`function` · `opentelemetry_sdk::trace::sampler::Sampler::should_sample` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn should_sample(&self, parent_context: Option<&Context>, trace_id: TraceId, name: &str, span_kind: &SpanKind, attributes: &[KeyValue], links: &[Link]) -> SamplingResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::sampler::Sampler", "path": "Sampler"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [165, 1], "end": [226, 2], "filename": "src/trace/sampler.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::trace::sampler::ShouldSample", "path": "ShouldSample"}, "trait_path": "opentelemetry_sdk::trace::sampler::ShouldSample"}`

Source: `src/trace/sampler.rs:166`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
