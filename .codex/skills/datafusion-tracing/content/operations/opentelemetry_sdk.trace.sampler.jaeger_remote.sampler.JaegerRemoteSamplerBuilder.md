# `opentelemetry_sdk::trace::sampler::jaeger_remote::sampler::JaegerRemoteSamplerBuilder`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.trace.sampler.jaeger_remote.sampler.JaegerRemoteSamplerBuilder.json).

<a id="op-d5dafcecbea21ff9be6e083a"></a>
## JaegerRemoteSamplerBuilder

`struct` · `opentelemetry_sdk::trace::sampler::jaeger_remote::sampler::JaegerRemoteSamplerBuilder` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct JaegerRemoteSamplerBuilder<C, S, R> where R: RuntimeChannel, C: HttpClient + 'static, S: ShouldSample + 'static
```

Source: `src/trace/sampler/jaeger_remote/sampler.rs:20`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Builder for [`JaegerRemoteSampler`](../operations/opentelemetry_sdk.trace.sampler.jaeger_remote.sampler.JaegerRemoteSampler.md#op-d91f91c2c3c8c806b9db2a4c).
See [Sampler::jaeger_remote](../operations/opentelemetry_sdk.trace.sampler.Sampler.md#op-d65429cc175ff395f3d1ab43) for details.

<a id="op-2af6da87f6a3725dcc617889"></a>
## build

`function` · `opentelemetry_sdk::trace::sampler::jaeger_remote::sampler::JaegerRemoteSamplerBuilder::build` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> Result<Sampler, TraceError>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "C"}}, {"type": {"generic": "S"}}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "opentelemetry_sdk::trace::sampler::jaeger_remote::sampler::JaegerRemoteSamplerBuilder", "path": "JaegerRemoteSamplerBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "C"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_http::HttpClient", "path": "HttpClient"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "C"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::trace::sampler::ShouldSample", "path": "ShouldSample"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::runtime::RuntimeChannel", "path": "RuntimeChannel"}}}], "generic_params": [], "type": {"generic": "R"}}}]}, "is_negative": false, "span": {"begin": [35, 1], "end": [130, 2], "filename": "src/trace/sampler/jaeger_remote/sampler.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/sampler/jaeger_remote/sampler.rs:103`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Build a [JaegerRemoteSampler](../operations/opentelemetry_sdk.trace.sampler.jaeger_remote.sampler.JaegerRemoteSampler.md#op-d91f91c2c3c8c806b9db2a4c) using provided configuration.

Return errors if:

- the endpoint provided is empty.
- the service name provided is empty.

<a id="op-e33abb6a3cee436ca00b2ac4"></a>
## fmt

`function` · `opentelemetry_sdk::trace::sampler::jaeger_remote::sampler::JaegerRemoteSamplerBuilder::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "C"}}, {"type": {"generic": "S"}}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "opentelemetry_sdk::trace::sampler::jaeger_remote::sampler::JaegerRemoteSamplerBuilder", "path": "JaegerRemoteSamplerBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "C"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::runtime::RuntimeChannel", "path": "RuntimeChannel"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "generic_params": [], "type": {"generic": "R"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_http::HttpClient", "path": "HttpClient"}}}, {"outlives": "'static"}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "generic_params": [], "type": {"generic": "C"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::trace::sampler::ShouldSample", "path": "ShouldSample"}}}, {"outlives": "'static"}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [19, 10], "end": [19, 15], "filename": "src/trace/sampler/jaeger_remote/sampler.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/sampler/jaeger_remote/sampler.rs:19`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5bf902a743781eaf535c1d17"></a>
## with_endpoint

`function` · `opentelemetry_sdk::trace::sampler::jaeger_remote::sampler::JaegerRemoteSamplerBuilder::with_endpoint` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_endpoint<Str: Into<String>>(self, endpoint: Str) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "C"}}, {"type": {"generic": "S"}}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "opentelemetry_sdk::trace::sampler::jaeger_remote::sampler::JaegerRemoteSamplerBuilder", "path": "JaegerRemoteSamplerBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "C"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_http::HttpClient", "path": "HttpClient"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "C"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::trace::sampler::ShouldSample", "path": "ShouldSample"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::runtime::RuntimeChannel", "path": "RuntimeChannel"}}}], "generic_params": [], "type": {"generic": "R"}}}]}, "is_negative": false, "span": {"begin": [35, 1], "end": [130, 2], "filename": "src/trace/sampler/jaeger_remote/sampler.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/sampler/jaeger_remote/sampler.rs:78`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

The endpoint of remote servers.

By default it's `http://localhost:5778/sampling`.

If service name is provided as part of the endpoint, it will be ignored.

<a id="op-e75ea1258e711325ac6322af"></a>
## with_leaky_bucket_size

`function` · `opentelemetry_sdk::trace::sampler::jaeger_remote::sampler::JaegerRemoteSamplerBuilder::with_leaky_bucket_size` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_leaky_bucket_size(self, size: f64) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "C"}}, {"type": {"generic": "S"}}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "opentelemetry_sdk::trace::sampler::jaeger_remote::sampler::JaegerRemoteSamplerBuilder", "path": "JaegerRemoteSamplerBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "C"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_http::HttpClient", "path": "HttpClient"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "C"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::trace::sampler::ShouldSample", "path": "ShouldSample"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::runtime::RuntimeChannel", "path": "RuntimeChannel"}}}], "generic_params": [], "type": {"generic": "R"}}}]}, "is_negative": false, "span": {"begin": [35, 1], "end": [130, 2], "filename": "src/trace/sampler/jaeger_remote/sampler.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/sampler/jaeger_remote/sampler.rs:90`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

The size of the leaky bucket.

By default the size is 100.

It's used when sampling strategy is rate limiting.

<a id="op-f56f7e3194ffede1bb71ad11"></a>
## with_update_interval

`function` · `opentelemetry_sdk::trace::sampler::jaeger_remote::sampler::JaegerRemoteSamplerBuilder::with_update_interval` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_update_interval(self, interval: Duration) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "C"}}, {"type": {"generic": "S"}}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "opentelemetry_sdk::trace::sampler::jaeger_remote::sampler::JaegerRemoteSamplerBuilder", "path": "JaegerRemoteSamplerBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "C"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_http::HttpClient", "path": "HttpClient"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "C"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::trace::sampler::ShouldSample", "path": "ShouldSample"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::runtime::RuntimeChannel", "path": "RuntimeChannel"}}}], "generic_params": [], "type": {"generic": "R"}}}]}, "is_negative": false, "span": {"begin": [35, 1], "end": [130, 2], "filename": "src/trace/sampler/jaeger_remote/sampler.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/sampler/jaeger_remote/sampler.rs:66`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Change how often the SDK should fetch the sampling strategy from remote servers

By default it fetches every 5 minutes.

A shorter interval have a performance overhead and should be avoid.
