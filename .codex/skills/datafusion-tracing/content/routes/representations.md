# Representation routes

| Representation | Reviewed decisions |
|---|---|
| Arc<dyn ObjectStore> | [Instrument storage requests and returned streams](../capabilities/tracing.storage.md) |
| Cargo manifest | [Match dependency and feature profiles](../capabilities/tracing.compatibility.md) |
| Dispatch | [Propagate async span and subscriber context](../capabilities/tracing.context.md) |
| Future | [Propagate async span and subscriber context](../capabilities/tracing.context.md) |
| InstrumentationOptions | [Instrument physical query execution](../capabilities/tracing.execution.md) |
| OTLP attempts | [Own the bridge, provider and export lifecycle](../capabilities/tracing.export.md) |
| Phase spans | [Planning instrumentation: full](../capabilities/tracing.rules.full.md), [Planning instrumentation: phase](../capabilities/tracing.rules.phase.md), [Planning instrumentation: selected](../capabilities/tracing.rules.selected.md) |
| PhysicalOptimizerRule | [Instrument physical query execution](../capabilities/tracing.execution.md) |
| RuleInstrumentationOptions | [Planning instrumentation: full](../capabilities/tracing.rules.full.md), [Planning instrumentation: phase](../capabilities/tracing.rules.phase.md), [Planning instrumentation: selected](../capabilities/tracing.rules.selected.md) |
| SDK exporter records | [Own the bridge, provider and export lifecycle](../capabilities/tracing.export.md) |
| SessionState | [Planning instrumentation: full](../capabilities/tracing.rules.full.md), [Planning instrumentation: phase](../capabilities/tracing.rules.phase.md), [Planning instrumentation: selected](../capabilities/tracing.rules.selected.md) |
| Span | [Propagate async span and subscriber context](../capabilities/tracing.context.md) |
| batch streams | [Preview rows and choose formatting](../capabilities/tracing.preview.md) |
| branch-specific observations | [Distinguish local filtering and SDK sampling](../capabilities/tracing.filtering.md) |
| custom_fields map | [Declare and record custom span fields](../capabilities/tracing.fields.md) |
| datafusion.metrics.* fields | [Record native plan metrics](../capabilities/tracing.metrics.md) |
| datafusion.preview string | [Preview rows and choose formatting](../capabilities/tracing.preview.md) |
| execution spans | [Instrument physical query execution](../capabilities/tracing.execution.md) |
| execution streams | [Follow execution and recorder lifetimes](../capabilities/tracing.lifecycle.md) |
| filter | [Match macro targets and levels](../capabilities/tracing.targets.md) |
| finished spans | [Own the bridge, provider and export lifecycle](../capabilities/tracing.export.md) |
| instrumented Future | [Propagate async span and subscriber context](../capabilities/tracing.context.md) |
| layer filters | [Distinguish local filtering and SDK sampling](../capabilities/tracing.filtering.md) |
| macro field declaration | [Declare and record custom span fields](../capabilities/tracing.fields.md) |
| macro invocation | [Match macro targets and levels](../capabilities/tracing.targets.md) |
| native MetricsSet | [Record native plan metrics](../capabilities/tracing.metrics.md) |
| optional formatter | [Preview rows and choose formatting](../capabilities/tracing.preview.md) |
| parent relationships | [Propagate async span and subscriber context](../capabilities/tracing.context.md) |
| parent span | [Follow execution and recorder lifetimes](../capabilities/tracing.lifecycle.md) |
| physical plan | [Instrument physical query execution](../capabilities/tracing.execution.md) |
| preview_limit | [Preview rows and choose formatting](../capabilities/tracing.preview.md) |
| record_metrics option | [Record native plan metrics](../capabilities/tracing.metrics.md) |
| recording and close events | [Follow execution and recorder lifetimes](../capabilities/tracing.lifecycle.md) |
| resolved lock and features | [Match dependency and feature profiles](../capabilities/tracing.compatibility.md) |
| sampler | [Distinguish local filtering and SDK sampling](../capabilities/tracing.filtering.md) |
| scoped compatibility evidence | [Match dependency and feature profiles](../capabilities/tracing.compatibility.md) |
| selected spans | [Match macro targets and levels](../capabilities/tracing.targets.md) |
| span fields | [Declare and record custom span fields](../capabilities/tracing.fields.md) |
| spans | [Distinguish local filtering and SDK sampling](../capabilities/tracing.filtering.md) |
| storage spans | [Instrument storage requests and returned streams](../capabilities/tracing.storage.md) |
| store name | [Instrument storage requests and returned streams](../capabilities/tracing.storage.md) |
| subscriber layer | [Own the bridge, provider and export lifecycle](../capabilities/tracing.export.md) |
| task context | [Follow execution and recorder lifetimes](../capabilities/tracing.lifecycle.md) |
| tracer provider | [Own the bridge, provider and export lifecycle](../capabilities/tracing.export.md) |
