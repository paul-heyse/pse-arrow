# `opentelemetry_sdk::trace::sampler::CloneShouldSample`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.trace.sampler.CloneShouldSample.json).

<a id="op-51a17bdf3efb9d99559a2d6f"></a>
## CloneShouldSample

`trait` · `opentelemetry_sdk::trace::sampler::CloneShouldSample` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait CloneShouldSample
```

Source: `src/trace/sampler.rs:86`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

This trait should not be used directly instead users should use [`ShouldSample`].

<a id="op-34e7e5a4042a10d57f232d01"></a>
## box_clone

`function` · `opentelemetry_sdk::trace::sampler::CloneShouldSample::box_clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn box_clone(&self) -> Box<dyn ShouldSample>
```

Source: `src/trace/sampler.rs:87`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
