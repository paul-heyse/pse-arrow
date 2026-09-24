# DataFusion metrics as span fields

`record_metrics(true)` copies every metric a node reports onto its span as `datafusion.metrics.<name>`. The names are not this library's: they are built as `format!("datafusion.metrics.{}", metric.value().name())` from whatever the node reports, so the field set is open and node-dependent.

## Spans it emits

| Span | Target | Level | Fields | Evidence |
|---|---|---|---:|---|
| `InstrumentedExec` | `integration_utils` | INFO | 49 | recorded |

## What this seam cannot tell you

- A complete list of the metric fields. Six were observed in every metrics-enabled scenario; 35 more appeared only where a particular node ran. A field absent from `index/span-fields.tsv` is unobserved, not unavailable.
- What a metric means. `MetricsSet` is DataFusion's surface.
- Metrics for a node that reports none.

## Read next

- [`catalogs/spans.md`](../catalogs/spans.md)
- [`content/corpus/source/datafusion-tracing/metrics.rs`](../corpus/source/datafusion-tracing/metrics.rs) — upstream, verbatim
