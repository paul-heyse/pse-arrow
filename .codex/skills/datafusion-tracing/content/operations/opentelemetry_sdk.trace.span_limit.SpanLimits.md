# `opentelemetry_sdk::trace::span_limit::SpanLimits`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.trace.span_limit.SpanLimits.json).

<a id="op-841edc32e30f45c0cad34f8e"></a>
## SpanLimits

`struct` · `opentelemetry_sdk::trace::span_limit::SpanLimits` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct SpanLimits
```

Source: `src/trace/span_limit.rs:23`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Span limit configuration to keep attributes, events and links to a span in a reasonable number.

<a id="op-feeadcf6d3da30c24fcd3d15"></a>
## clone

`function` · `opentelemetry_sdk::trace::span_limit::SpanLimits::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> SpanLimits
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::span_limit::SpanLimits", "path": "SpanLimits"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [22, 16], "end": [22, 21], "filename": "src/trace/span_limit.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/trace/span_limit.rs:22`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b4d9aa60b99c2692af6476ac"></a>
## default

`function` · `opentelemetry_sdk::trace::span_limit::SpanLimits::default` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::span_limit::SpanLimits", "path": "SpanLimits"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [36, 1], "end": [46, 2], "filename": "src/trace/span_limit.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/trace/span_limit.rs:37`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-6807aad803768a5200f82357"></a>
## fmt

`function` · `opentelemetry_sdk::trace::span_limit::SpanLimits::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::span_limit::SpanLimits", "path": "SpanLimits"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [22, 23], "end": [22, 28], "filename": "src/trace/span_limit.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/span_limit.rs:22`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5611bba0aef32a6e473a36e4"></a>
## max_attributes_per_event

`struct_field` · `opentelemetry_sdk::trace::span_limit::SpanLimits::max_attributes_per_event` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
max_attributes_per_event: u32
```

Source: `src/trace/span_limit.rs:31`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

The max attributes that can be added into an `Event`

<a id="op-bde8e5bfad7787a754484b2e"></a>
## max_attributes_per_link

`struct_field` · `opentelemetry_sdk::trace::span_limit::SpanLimits::max_attributes_per_link` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
max_attributes_per_link: u32
```

Source: `src/trace/span_limit.rs:33`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

The max attributes that can be added into a `Link`

<a id="op-68f7d98790c0423b32b5cfd6"></a>
## max_attributes_per_span

`struct_field` · `opentelemetry_sdk::trace::span_limit::SpanLimits::max_attributes_per_span` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
max_attributes_per_span: u32
```

Source: `src/trace/span_limit.rs:27`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

The max attributes that can be added to a `Span`.

<a id="op-5635ec2182af9678d9311a9a"></a>
## max_events_per_span

`struct_field` · `opentelemetry_sdk::trace::span_limit::SpanLimits::max_events_per_span` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
max_events_per_span: u32
```

Source: `src/trace/span_limit.rs:25`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

The max events that can be added to a `Span`.

<a id="op-cf9e46d351e0df0aaadd5e08"></a>
## max_links_per_span

`struct_field` · `opentelemetry_sdk::trace::span_limit::SpanLimits::max_links_per_span` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
max_links_per_span: u32
```

Source: `src/trace/span_limit.rs:29`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

The max links that can be added to a `Span`.
