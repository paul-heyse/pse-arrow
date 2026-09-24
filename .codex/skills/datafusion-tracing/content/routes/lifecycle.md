# Lifecycle routes

Each route exposes alternatives and evidence limits.

## planning

- [Planning instrumentation: full](../capabilities/tracing.rules.full.md): Per-rule detail increases observable activity; volume depends on the plan.
- [Planning instrumentation: phase](../capabilities/tracing.rules.phase.md): Phase-level observation omits individual Rule spans.
- [Planning instrumentation: selected](../capabilities/tracing.rules.selected.md): Builder selectors choose particular phase and rule detail.

## execution

- [Instrument physical query execution](../capabilities/tracing.execution.md): Execution macros construct an optimizer rule that wraps the physical plan it receives. Later plan rewrites can change which work is represented.

## completion

- [Record native plan metrics](../capabilities/tracing.metrics.md): The recorder reads native plan metrics, aggregates by name, and records dynamic datafusion.metrics.* fields when its recorder is dropped.
- [Preview rows and choose formatting](../capabilities/tracing.preview.md): A positive preview limit enables the recorder. An omitted formatter selects the default pretty formatter; a custom callback changes presentation.

## async

- [Propagate async span and subscriber context](../capabilities/tracing.context.md): Async instrumentation scopes context around polling. Span context and subscriber dispatch are separate parts of a composition, especially across spawned work.

## storage

- [Instrument storage requests and returned streams](../capabilities/tracing.storage.md): The returned wrapper observes calls made through it. Registration and use of the wrapped instance determine whether query I/O reaches that boundary.

## export

- [Own the bridge, provider and export lifecycle](../capabilities/tracing.export.md): The bridge connects tracing spans to an SDK tracer. Provider/processor lifetime and span completion govern export; transport success and collector retention are further observations.
- [Distinguish local filtering and SDK sampling](../capabilities/tracing.filtering.md): Different layers can accept different telemetry. A local formatted span is evidence for that output branch, not proof that an SDK exported it.
