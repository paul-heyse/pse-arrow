# Task routes

Reviewed depth is selective. Search the full operation index for unreviewed tasks.

| Task vocabulary | Decision brief | Input → output |
|---|---|---|
| opentelemetry versions trait bound error; uncharacterized version combination; Cargo lock features compatibility | [Match dependency and feature profiles](../capabilities/tracing.compatibility.md) | Cargo manifest, resolved lock and features → scoped compatibility evidence |
| await under span; spawn task parent context; future polling subscriber; async context | [Propagate async span and subscriber context](../capabilities/tracing.context.md) | Future, Span, Dispatch → instrumented Future, parent relationships |
| trace physical operators; query execution spans; optimizer rule registration last; later plan rewrite | [Instrument physical query execution](../capabilities/tracing.execution.md) | InstrumentationOptions, physical plan → PhysicalOptimizerRule, execution spans |
| OTLP exporter wiring; flush buffered spans shutdown; collector receipt; batch versus simple exporter | [Own the bridge, provider and export lifecycle](../capabilities/tracing.export.md) | subscriber layer, tracer provider, finished spans → SDK exporter records, OTLP attempts |
| attach query identifier; custom field missing; undeclared key; span attributes | [Declare and record custom span fields](../capabilities/tracing.fields.md) | macro field declaration, custom_fields map → span fields |
| stdout works collector empty; local layer receives OTEL missing; sampling filtering; multiple layers | [Distinguish local filtering and SDK sampling](../capabilities/tracing.filtering.md) | layer filters, sampler, spans → branch-specific observations |
| trace ends before fields appear; keep plan alive; early drop error partial partitions; repeat concurrent execution | [Follow execution and recorder lifetimes](../capabilities/tracing.lifecycle.md) | execution streams, task context, parent span → recording and close events |
| record query measurements; missing metric field; output rows metrics; metric units aggregation | [Record native plan metrics](../capabilities/tracing.metrics.md) | native MetricsSet, record_metrics option → datafusion.metrics.* fields |
| preview a few rows; positive limit omitted formatter; default preview; multiple batches partitions preview; custom formatter error | [Preview rows and choose formatting](../capabilities/tracing.preview.md) | preview_limit, optional formatter, batch streams → datafusion.preview string |
| identify optimizer rule changes plan; planning is slow but execution is fast; full | [Planning instrumentation: full](../capabilities/tracing.rules.full.md) | SessionState, RuleInstrumentationOptions → SessionState, Phase spans |
| total planning phase duration; planning is slow but execution is fast; phase_only | [Planning instrumentation: phase](../capabilities/tracing.rules.phase.md) | SessionState, RuleInstrumentationOptions → SessionState, Phase spans |
| only physical optimization; planning is slow but execution is fast; builder | [Planning instrumentation: selected](../capabilities/tracing.rules.selected.md) | SessionState, RuleInstrumentationOptions → SessionState, Phase spans |
| see storage requests; wrap object store register; get ranges bytes stream list multipart errors | [Instrument storage requests and returned streams](../capabilities/tracing.storage.md) | Arc<dyn ObjectStore>, store name → Arc<dyn ObjectStore>, storage spans |
| empty trace EnvFilter; default caller target; explicit target; INFO DEBUG missing spans | [Match macro targets and levels](../capabilities/tracing.targets.md) | macro invocation, filter → selected spans |
