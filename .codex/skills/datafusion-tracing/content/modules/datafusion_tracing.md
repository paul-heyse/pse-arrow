# `datafusion_tracing`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.json).

<a id="op-b9a62e328bba1376c68230b3"></a>
## datafusion_tracing

`module` · `datafusion_tracing` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: hosted.

```rust
mod datafusion_tracing
```

Source: `src/lib.rs:20`. [Exact documentation build](https://docs.rs/crate/datafusion-tracing/55.0.0/json).

DataFusion Tracing is an extension for [Apache DataFusion](https://datafusion.apache.org/) that helps you monitor and debug queries. It uses [`tracing`](https://docs.rs/tracing/latest/tracing/) and [OpenTelemetry](https://opentelemetry.io/) to gather DataFusion metrics, trace execution steps, and preview partial query results.

**Note:** This is not an official Apache Software Foundation release.

# Overview

When you run queries with DataFusion Tracing enabled, it automatically adds tracing around execution steps, records all native DataFusion metrics such as execution time and output row count, lets you preview partial results for easier debugging, and integrates with OpenTelemetry for distributed tracing. This makes it simpler to understand and improve query performance.

## See it in action

Here's what DataFusion Tracing can look like in practice:

<details>
<summary>Jaeger UI</summary>

![Jaeger UI screenshot](https://raw.githubusercontent.com/datafusion-contrib/datafusion-tracing/main/datafusion-tracing/docs/screenshots/jaeger.png)
</details>

<details>
<summary>DataDog UI</summary>

![DataDog UI screenshot](https://raw.githubusercontent.com/datafusion-contrib/datafusion-tracing/main/datafusion-tracing/docs/screenshots/datadog.png)
</details>

# Getting Started

## Installation

Include DataFusion Tracing in your project's `Cargo.toml`:

```toml
[dependencies]
datafusion = "55.0.0"
datafusion-tracing = "55.0.0"
```

## Quick Start Example

```rust
use datafusion::{
    arrow::{array::RecordBatch, util::pretty::pretty_format_batches},
    error::Result,
    execution::SessionStateBuilder,
    prelude::*,
};
use datafusion_tracing::{
    instrument_rules_with_info_spans, instrument_with_info_spans,
    pretty_format_compact_batch, InstrumentationOptions, RuleInstrumentationOptions,
};
use std::sync::Arc;
use tracing::field;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber as usual
    // (See examples/otlp.rs for a complete example).

    // Set up execution plan tracing options (you can customize these).
    let exec_options = InstrumentationOptions::builder()
        .record_metrics(true)
        .preview_limit(5)
        .preview_fn(Arc::new(|batch: &RecordBatch| {
            pretty_format_compact_batch(batch, 64, 3, 10).map(|fmt| fmt.to_string())
        }))
        .add_custom_field("env", "production")
        .add_custom_field("region", "us-west")
        .build();

    let instrument_rule = instrument_with_info_spans!(
        options: exec_options,
        env = field::Empty,
        region = field::Empty,
    );

    let session_state = SessionStateBuilder::new()
        .with_default_features()
        .with_physical_optimizer_rule(instrument_rule)
        .build();

    // Instrument all rules (analyzer, logical optimizer, physical optimizer)
    // Physical plan creation tracing is automatically enabled when physical_optimizer is set
    let rule_options = RuleInstrumentationOptions::full().with_plan_diff();
    let session_state = instrument_rules_with_info_spans!(
        options: rule_options,
        state: session_state
    );

    let ctx = SessionContext::new_with_state(session_state);

    // Execute a query - the entire lifecycle is now traced:
    // SQL Parsing -> Logical Plan -> Analyzer Rules -> Optimizer Rules ->
    // Physical Plan Creation -> Physical Optimizer Rules -> Execution
    let results = ctx.sql("SELECT 1").await?.collect().await?;
    println!(
        "Query Results:\n{}",
        pretty_format_batches(results.as_slice())?
    );

    Ok(())
}
```

A more complete example can be found in the [examples directory](https://github.com/datafusion-contrib/datafusion-tracing/tree/main/examples).


<a id="op-56c206fd47a0d28f68526940"></a>
## datafusion_tracing

`module` · `datafusion_tracing` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: private.

```rust
mod datafusion_tracing
```

Source: `src/lib.rs:20`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

DataFusion Tracing is an extension for [Apache DataFusion](https://datafusion.apache.org/) that helps you monitor and debug queries. It uses [`tracing`](https://docs.rs/tracing/latest/tracing/) and [OpenTelemetry](https://opentelemetry.io/) to gather DataFusion metrics, trace execution steps, and preview partial query results.

**Note:** This is not an official Apache Software Foundation release.

# Overview

When you run queries with DataFusion Tracing enabled, it automatically adds tracing around execution steps, records all native DataFusion metrics such as execution time and output row count, lets you preview partial results for easier debugging, and integrates with OpenTelemetry for distributed tracing. This makes it simpler to understand and improve query performance.

## See it in action

Here's what DataFusion Tracing can look like in practice:

<details>
<summary>Jaeger UI</summary>

![Jaeger UI screenshot](https://raw.githubusercontent.com/datafusion-contrib/datafusion-tracing/main/datafusion-tracing/docs/screenshots/jaeger.png)
</details>

<details>
<summary>DataDog UI</summary>

![DataDog UI screenshot](https://raw.githubusercontent.com/datafusion-contrib/datafusion-tracing/main/datafusion-tracing/docs/screenshots/datadog.png)
</details>

# Getting Started

## Installation

Include DataFusion Tracing in your project's `Cargo.toml`:

```toml
[dependencies]
datafusion = "55.0.0"
datafusion-tracing = "55.0.0"
```

## Quick Start Example

```rust
use datafusion::{
    arrow::{array::RecordBatch, util::pretty::pretty_format_batches},
    error::Result,
    execution::SessionStateBuilder,
    prelude::*,
};
use datafusion_tracing::{
    instrument_rules_with_info_spans, instrument_with_info_spans,
    pretty_format_compact_batch, InstrumentationOptions, RuleInstrumentationOptions,
};
use std::sync::Arc;
use tracing::field;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber as usual
    // (See examples/otlp.rs for a complete example).

    // Set up execution plan tracing options (you can customize these).
    let exec_options = InstrumentationOptions::builder()
        .record_metrics(true)
        .preview_limit(5)
        .preview_fn(Arc::new(|batch: &RecordBatch| {
            pretty_format_compact_batch(batch, 64, 3, 10).map(|fmt| fmt.to_string())
        }))
        .add_custom_field("env", "production")
        .add_custom_field("region", "us-west")
        .build();

    let instrument_rule = instrument_with_info_spans!(
        options: exec_options,
        env = field::Empty,
        region = field::Empty,
    );

    let session_state = SessionStateBuilder::new()
        .with_default_features()
        .with_physical_optimizer_rule(instrument_rule)
        .build();

    // Instrument all rules (analyzer, logical optimizer, physical optimizer)
    // Physical plan creation tracing is automatically enabled when physical_optimizer is set
    let rule_options = RuleInstrumentationOptions::full().with_plan_diff();
    let session_state = instrument_rules_with_info_spans!(
        options: rule_options,
        state: session_state
    );

    let ctx = SessionContext::new_with_state(session_state);

    // Execute a query - the entire lifecycle is now traced:
    // SQL Parsing -> Logical Plan -> Analyzer Rules -> Optimizer Rules ->
    // Physical Plan Creation -> Physical Optimizer Rules -> Execution
    let results = ctx.sql("SELECT 1").await?.collect().await?;
    println!(
        "Query Results:\n{}",
        pretty_format_batches(results.as_slice())?
    );

    Ok(())
}
```

A more complete example can be found in the [examples directory](https://github.com/datafusion-contrib/datafusion-tracing/tree/main/examples).

