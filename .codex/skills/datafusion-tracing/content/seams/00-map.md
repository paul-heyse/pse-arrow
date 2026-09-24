# Seam map

Instrumentation seams. Direct task, symbol and evidence lookup are independent entry points.

| Seam | Covers |
|---|---|
| [Instrumenting execution plans](exec-instrumentation.md) | Wrapping every ExecutionPlan node so each one emits a span while it runs |
| [Instrumenting planning phases](rule-instrumentation.md) | Spans around analyzer, logical-optimizer and physical-optimizer passes |
| [Previewing partial results](preview.md) | Putting sample rows of each node's output into its span |
| [DataFusion metrics as span fields](metrics.md) | Recording each node's own MetricsSet onto its span |
| [Instrumenting the object store](object-store.md) | Spans around the GET and RANGE calls underneath a scan |
| [Wiring a subscriber and an exporter](subscriber-wiring.md) | tracing-subscriber layers, level filters, and the OTLP exporter |
| [The emitted span contract](span-contract.md) | Names, targets, levels, fields and nesting of everything emitted |
| [The DataFusion boundary](datafusion-boundary.md) | Where this repository stops and the DataFusion reference begins |

Limits describe the scope of each seam. Task routes and known-symbol lookup are also available.
