---
name: datafusion-tracing
description: Find and compose DataFusion planning, execution and object-store instrumentation using task routes, exact API and macro contracts, and scoped trace evidence. Use for span fields, metrics, previews, subscriber and exporter wiring, or diagnosing missing telemetry.
---

# DataFusion tracing reference

This offline reference supports instrumentation decisions and precise implementation. It pins
`datafusion-tracing` and `instrumented-object-store` **55.0.0**, with nine tracing/OpenTelemetry
wiring crates. [Crate roles](content/routes/crates.md) explain their boundaries; the
[recorded profile](content/PROVENANCE.json) and consumer Cargo.lock/features are separate inputs.

## Find the relevant capability

Commands run from the skill directory. Scripts locate resources relative to their own file,
so absolute script paths also work from another working directory. Reading needs Python 3.11+
standard library and local files; no service, compiler, collector or network is required.

```bash
python3 scripts/reference.py find --task 'planning is slow but execution is fast'
python3 scripts/reference.py find --task 'stdout works collector empty'
python3 scripts/reference.py show tracing.preview
python3 scripts/reference.py compare tracing.rules.phase tracing.rules.full
```

Direct files are equally useful:

- [Tasks](content/routes/tasks.md): planning, execution, fields, targets, previews, metrics,
  lifetimes, async context, filtering, storage, export and compatibility.
- [Lifecycle](content/routes/lifecycle.md) and [symptoms](content/routes/symptoms.md): observation
  boundaries and competing explanations for missing telemetry.
- [Macro grammar](content/catalogs/macros.md), [options](content/catalogs/options.md),
  [observed fields](content/index/span-fields.tsv) and [declared compatibility](content/catalogs/compatibility.md).

A capability brief relates inputs, outputs, lifecycle, alternatives, evidence and unknowns.
Fourteen reviewed briefs provide selective semantic depth; the broader API inventory remains
available without implying every operation has equivalent characterization.

## Retrieve an exact contract

```bash
python3 scripts/reference.py show datafusion_tracing::InstrumentationOptions --member preview_fn --view contract
python3 scripts/reference.py show tracing.export --view evidence
python3 scripts/reference.py show tracing::Span --member enter --view contract
```

Contract records retain full upstream docs, type trees, implementation context, source locations
and artifact-scoped links. Hosted and private captures remain distinct. Display signatures are
projections; their raw type trees are authoritative when a display contains a placeholder.

`reachability` distinguishes supported, doc-hidden, reachable-undocumented and internal surface.
A canonical private path locates evidence; it is not necessarily an import path. The options
builders are reached through public `builder()` calls. Internal wrappers remain source evidence.

Output defaults to 12,000 bytes. Partial results carry a fingerprint, text fragment and
`next_offset`; repeat the same command/view with `--offset` or read the named file. Facets and
lexical matches do not establish semantic type compatibility or capability absence.

## Interpret the observation

Planning, execution, storage, subscriber handling and export have separate contracts. Check the
relevant boundary when targets, levels, field declarations, async context or completion timing
changes the composition. A local span capture and collector receipt are different observations.

[Probe receipts](skill_improvement/evidence/implementation/probe-results.json) name assertions,
controls, source hashes and the [resolved profile](skill_improvement/evidence/implementation/probe-profile.json).
Snapshot aggregates stay `recorded`; they are not promoted wholesale by one successful assertion.
The metric field vocabulary remains open. A preview row cap is not a retained-memory bound.

See [reference.md](reference.md) for schemas and evidence interpretation, and
[maintenance.md](maintenance.md) for regeneration, targeted validation and portable bundles.
