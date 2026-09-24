# `opentelemetry_sdk::trace::sampler::jaeger_remote::sampler::JaegerRemoteSampler`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.trace.sampler.jaeger_remote.sampler.JaegerRemoteSampler.json).

<a id="op-d91f91c2c3c8c806b9db2a4c"></a>
## JaegerRemoteSampler

`struct` · `opentelemetry_sdk::trace::sampler::jaeger_remote::sampler::JaegerRemoteSampler` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct JaegerRemoteSampler
```

Source: `src/trace/sampler/jaeger_remote/sampler.rs:144`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Sampler that fetches the sampling configuration from remotes.

It offers the following sampling strategies:
- **Probabilistic**, fetch a probability between [0.0, 1.0] from remotes and use it to sample traces. If the probability is 0.0, it will never sample traces. If the probability is 1.0, it will always sample traces.
- **Rate limiting**, ses a leaky bucket rate limiter to ensure that traces are sampled with a certain constant rate.
- **Per Operations**, instead of sampling all traces, it samples traces based on the span name. Only probabilistic sampling is supported at the moment.

User can build a [`JaegerRemoteSampler`](../operations/opentelemetry_sdk.trace.sampler.jaeger_remote.sampler.JaegerRemoteSampler.md#op-d91f91c2c3c8c806b9db2a4c) by getting a [`JaegerRemoteSamplerBuilder`](../operations/opentelemetry_sdk.trace.sampler.jaeger_remote.sampler.JaegerRemoteSamplerBuilder.md#op-d5dafcecbea21ff9be6e083a) from [`Sampler::jaeger_remote`](../operations/opentelemetry_sdk.trace.sampler.Sampler.md#op-d65429cc175ff395f3d1ab43).

Note that the backend doesn't need to be Jaeger so long as it supports jaeger remote sampling
protocol.

<a id="op-1f5f3c34f913ab424bdb5573"></a>
## clone

`function` · `opentelemetry_sdk::trace::sampler::jaeger_remote::sampler::JaegerRemoteSampler::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> JaegerRemoteSampler
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::sampler::jaeger_remote::sampler::JaegerRemoteSampler", "path": "JaegerRemoteSampler"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [143, 10], "end": [143, 15], "filename": "src/trace/sampler/jaeger_remote/sampler.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/trace/sampler/jaeger_remote/sampler.rs:143`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5bc68dcaa221bf7378391388"></a>
## fmt

`function` · `opentelemetry_sdk::trace::sampler::jaeger_remote::sampler::JaegerRemoteSampler::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::sampler::jaeger_remote::sampler::JaegerRemoteSampler", "path": "JaegerRemoteSampler"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [143, 17], "end": [143, 22], "filename": "src/trace/sampler/jaeger_remote/sampler.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/sampler/jaeger_remote/sampler.rs:143`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-03b2ccb2f67ea48e02318377"></a>
## should_sample

`function` · `opentelemetry_sdk::trace::sampler::jaeger_remote::sampler::JaegerRemoteSampler::should_sample` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn should_sample(&self, parent_context: Option<&Context>, trace_id: TraceId, name: &str, span_kind: &SpanKind, attributes: &[KeyValue], links: &[Link]) -> SamplingResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::sampler::jaeger_remote::sampler::JaegerRemoteSampler", "path": "JaegerRemoteSampler"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [255, 1], "end": [278, 2], "filename": "src/trace/sampler/jaeger_remote/sampler.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::trace::sampler::ShouldSample", "path": "ShouldSample"}, "trait_path": "opentelemetry_sdk::trace::sampler::ShouldSample"}`

Source: `src/trace/sampler/jaeger_remote/sampler.rs:256`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
