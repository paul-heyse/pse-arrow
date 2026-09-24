# `datafusion_tracing::exec_instrument_rule::BoxedClosure`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.exec_instrument_rule.BoxedClosure.json).

<a id="op-85cfb4e096cabfeed1b016e8"></a>
## BoxedClosure

`type_alias` · `datafusion_tracing::exec_instrument_rule::BoxedClosure` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
type BoxedClosure = Box<dyn FnOnce() -> Box<dyn Any + Send> + Send>
```

Source: `src/exec_instrument_rule.rs:120`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.
