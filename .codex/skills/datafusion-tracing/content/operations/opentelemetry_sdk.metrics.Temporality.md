# `opentelemetry_sdk::metrics::Temporality`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.Temporality.json).

<a id="op-dc2884d798040a5373238d65"></a>
## Temporality

`enum` · `opentelemetry_sdk::metrics::Temporality` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
enum Temporality
```

Source: `src/metrics/mod.rs:90`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Defines the window that an aggregation was calculated over.

<a id="op-743a2804c86a2f20f0e408e3"></a>
## Cumulative

`variant` · `opentelemetry_sdk::metrics::Temporality::Cumulative` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Cumulative
```

Source: `src/metrics/mod.rs:96`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

A measurement interval that continues to expand forward in time from a
starting point.

New measurements are added to all previous measurements since a start time.

<a id="op-57054835b286178fc16b866a"></a>
## Delta

`variant` · `opentelemetry_sdk::metrics::Temporality::Delta` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Delta
```

Source: `src/metrics/mod.rs:102`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

A measurement interval that resets each cycle.

Measurements from one cycle are recorded independently, measurements from
other cycles do not affect them.

<a id="op-e22238f31f957abefff4fae9"></a>
## LowMemory

`variant` · `opentelemetry_sdk::metrics::Temporality::LowMemory` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
LowMemory
```

Source: `src/metrics/mod.rs:107`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Configures Synchronous Counter and Histogram instruments to use
Delta aggregation temporality, which allows them to shed memory
following a cardinality explosion, thus use less memory.

<a id="op-fb1f982e6d7bcc31b2c05a73"></a>
## clone

`function` · `opentelemetry_sdk::metrics::Temporality::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Temporality
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::Temporality", "path": "Temporality"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [88, 23], "end": [88, 28], "filename": "src/metrics/mod.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/metrics/mod.rs:88`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ca7e664b219029518bef00dd"></a>
## default

`function` · `opentelemetry_sdk::metrics::Temporality::default` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Temporality
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::Temporality", "path": "Temporality"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [88, 30], "end": [88, 37], "filename": "src/metrics/mod.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/metrics/mod.rs:88`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-300ec900ce32fbb2175e0f80"></a>
## eq

`function` · `opentelemetry_sdk::metrics::Temporality::eq` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &Temporality) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::Temporality", "path": "Temporality"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [88, 39], "end": [88, 48], "filename": "src/metrics/mod.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/metrics/mod.rs:88`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f6e908ad75baaa68bd3468c2"></a>
## fmt

`function` · `opentelemetry_sdk::metrics::Temporality::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::Temporality", "path": "Temporality"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [88, 10], "end": [88, 15], "filename": "src/metrics/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/mod.rs:88`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-08deae43097caf5171a39021"></a>
## hash

`function` · `opentelemetry_sdk::metrics::Temporality::hash` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn hash<__H: hash::Hasher>(&self, state: &mut __H)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::Temporality", "path": "Temporality"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [88, 54], "end": [88, 58], "filename": "src/metrics/mod.rs"}, "trait": {"args": null, "id": "core::hash::Hash", "path": "Hash"}, "trait_path": "core::hash::Hash"}`

Source: `src/metrics/mod.rs:88`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
