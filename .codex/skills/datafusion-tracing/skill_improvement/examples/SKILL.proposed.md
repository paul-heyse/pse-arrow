---
name: datafusion-tracing
description: Find and compose DataFusion planning, execution and object-store instrumentation using task routes, exact API and macro contracts, and scoped trace evidence. Use for span fields, metrics, previews, subscriber and exporter wiring, or diagnosing missing telemetry.
---

# DataFusion tracing reference

This is a proposed entry point. Its links use the planning bundle's real files; the implemented
version would link to the corresponding generated routes and contracts. The live skill is
unchanged.

The reference covers `datafusion-tracing` and `instrumented-object-store` 55.0.0 plus nine
tracing/OpenTelemetry wiring crates. Its [recorded profile](../../content/catalogs/crate-map.md)
is distinct from the consumer's resolved dependencies and features.

## Find the relevant capability

| What you have | First reference |
|---|---|
| An instrumentation task | [Task routes](../TARGET_DESIGN.md#task-routes) |
| A choice between mechanisms | [Capability comparisons](../CAPABILITY_EXAMPLES.md) |
| A known API or macro | [Symbol index](../../content/index/symbols.tsv), [options](../../content/catalogs/options.md), or [macro grammar](../../content/catalogs/macros.md) |
| Missing spans or fields | [Lifecycle routes](../TARGET_DESIGN.md#lifecycle-routes), then the relevant capability's evidence |
| A dependency or wiring question | [Crate roles](../TARGET_DESIGN.md#crate-roles) and [declared compatibility](../../content/catalogs/compatibility.md) |

A capability brief relates inputs, observable outputs, lifecycle, integration requirements and
alternatives. Exact operation contracts supply full docs and signatures; deeper evidence answers
questions the brief leaves open. Direct lookup is available without following a fixed sequence.

## Interpret the evidence

Visibility distinguishes nameable APIs, reachable undocumented builders, macro support items
and internal implementation. A canonical private path locates evidence; it may not be an import
path. The builder's public access route is part of its contract.

Trace fields and lifecycle observations belong to the recorded construction. An absent field
is not proof of unavailable behavior. Snapshot-derived catalogs, source-backed claims and
executed assertions retain their own scopes; the metric vocabulary is open.

Planning, execution, storage, subscriber handling and export are separate boundaries. A local
span capture establishes a different observation from collector receipt. Check the relevant
contract when target/level, field declarations, async context or completion timing affects the
composition.

See the [evidence guide](../evidence/README.md) for inherited results and current limitations.
The reader works from portable local reference files; deeper research can use exact source,
isolated consumers, and additional documentation when the task calls for them.
