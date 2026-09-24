# Worked capability references

Prepared 2026-09-18. These examples demonstrate the intended reference depth and style. They
use the current pinned corpus and inherited probe results; no new Rust behavior was executed
for this plan. Proposed assertions are identified separately from existing evidence.

## 1. Observe planning phases or physical execution

**Task:** associate query work with useful timing and structural observations.

| Mechanism | Input → output | Distinction |
|---|---|---|
| Execution instrumentation macro | Level/target, options and declared fields → physical optimizer rule | Wraps physical execution; registration and later rewrites affect the observed plan |
| Planning instrumentation macro | Session state and rule options → instrumented session state | Observes planning activity rather than only operators executing batches |
| `RuleInstrumentationOptions::phase_only()` | Preset → options | Coarser phase detail |
| `RuleInstrumentationOptions::full()` | Preset → options | Includes individual rule activity |
| Options builder | Selected phase settings → options | Allows narrower phase selection through a reachable, undocumented builder |

**Composition.** Planning and execution are separate instrumentation surfaces and can be used
together. Their macros produce different integration objects. The reference needs the exact
invocation arm and state/rule placement, not merely a list of macro names.

**Evidence.** [Macro grammar](../content/catalogs/macros.md),
[option catalog](../content/catalogs/options.md), and
[inherited probes](../content/probes/00-index.md): S001 observes an execution span against an
uninstrumented control; S007 distinguishes per-rule detail; S008 observes differing captures
for a physical-only configuration. S008 does not assert the complete included/excluded phase set.

**Planned verification.** Parse selected phases and per-rule parents, retain the executed physical
plan, and compare instrumented/uninstrumented query results. A controlled later rewrite tests
registration effects without assuming every later rule creates an uninstrumented node.

## 2. Configure a preview and understand when it appears

**Task:** attach a bounded row preview to an execution span.

| Configuration | Source-backed distinction |
|---|---|
| `preview_limit(0)` | Preview recorder is disabled |
| Positive limit, omitted formatter | The retained implementation supplies a default formatter |
| Positive limit, custom formatter | The callback receives the assembled preview batch and produces a string or formatting error |

**Contract.** `InstrumentationOptions` carries the row limit and an optional formatter. In the
retained source, `InstrumentedExec` creates a recorder when the limit is positive and passes
the optional formatter to `PreviewRecorderBuilder::preview_fn`. That builder selects
`default_preview_fn` for `None`.

Partition streams accumulate slices of observed batches. Their drop path contributes the
partition preview to the shared recorder. The recorder's drop path concatenates available
previews, applies the final row cap, then formats and records the field. This is a source
observation of the pinned implementation, not a newly measured lifecycle guarantee.

The displayed cap is distinct from intermediate data retention. Multiple partition previews,
concatenation, and sliced Arrow buffers make a row limit insufficient evidence of a byte bound.
The result is an execution preview, not a random sample or a complete query result. Formatting
failure and stream-assembly failure have separate code paths and should remain separate claims.

**Conflict to resolve.** The current seam and preview rule imply that `preview_fn` is required;
the source path above supplies a default. The seam's “one batch” explanation also omits
partition accumulation. The original descriptions should remain identifiable as superseded
claims when corrected at their authoring sources.

**Evidence.** [Options source](../content/corpus/source/datafusion-tracing/options.rs),
[execution wrapper](../content/corpus/source/datafusion-tracing/instrumented_exec.rs),
[preview source](../content/corpus/source/datafusion-tracing/preview.rs), and
[current seam](../content/seams/preview.md). Inherited S002/S005 cover zero-limit suppression
and one configured formatter's output. They do not establish the omitted-formatter case.

**Planned verification.** Positive limit with no callback; distinguishable custom callback;
multiple batches/partitions; early drop; formatter error. Assert content, cap and recording
time separately. Characterize allocation only with a dedicated measurement.

## 3. Make fields and filters refer to the emitted span

**Task:** attach a query identifier and receive only the intended instrumentation.

The custom-field mechanism has two inputs: a declared key in the macro's field set and a value
in the options. An option value cannot add an arbitrary new key to a span after creation.
Inherited S004 isolates this declaration/value distinction.

The execution macro's default target derives from `module_path!()` at its call site. An explicit
target overrides that default. The object-store wrapper uses its own instrumentation source;
the same filter text does not necessarily select both. Span name, tracing target, `otel.name`,
and exported attribute names are separate contract fields.

| Question | Evidence needed |
|---|---|
| Was a custom value accepted? | Key/value on the intended span, with an undeclared-key control |
| Does a target filter select this macro? | Macro target construction plus a matching/nonmatching filter pair |
| Did a layer suppress the span? | Observation at another layer or earlier boundary |
| Is the exported name different? | Bridge mapping and captured exported span |

**Evidence.** [Execution macro source](../content/corpus/source/datafusion-tracing/exec_instrument_macros.rs),
[object-store source](../content/corpus/source/instrumented-object-store/instrumented_object_store.rs),
and [probes](../content/probes/00-index.md) S004, S006, S009 and S010. S006 is explicitly
`recorded`: it reads the same capture two ways. Level-filter results are not target-filter tests.

**Planned verification.** An explicit-target control; layer-specific filters; a positive span
at an earlier boundary; exported field/name checks only in a declared bridge/SDK profile.

## 4. Relate metrics and preview finalization to execution lifetime

**Task:** explain why some fields are absent or why a span remains open.

Metrics are native DataFusion plan measurements recorded through the instrumentation. A field
can be absent because the option is disabled, the node lacks that metric, the field has not yet
been recorded, execution was partial, or output processing suppressed it. The observed catalog
cannot select among those explanations by itself.

**Source questions.** The retained execution wrapper has recorder-group ownership and upstream
tests concerning stream completion, retained plan clones, unexecuted partitions, execute errors,
repeated execution, different parent contexts, and independent previews. Those tests provide
more precise leads than “recorded at the end of the query.” The relevant end event must be
identified: stream completion, stream drop, recorder release, span close, or exporter shutdown.

**Evidence.** [Execution wrapper and upstream tests](../content/corpus/source/datafusion-tracing/instrumented_exec.rs),
[metric recorder](../content/corpus/source/datafusion-tracing/metrics.rs),
[field index](../content/index/span-fields.tsv), and inherited S003 in the
[probe index](../content/probes/00-index.md). S003 establishes its option contrast on the small
query; it does not certify the lifecycle cases listed above.

**Planned verification.** Capture span IDs and lifecycle callbacks across retained plan handles,
live streams, partial partitions, early errors and repeated/concurrent executions. Assert
which observation belongs to which execution. Preserve metric units and native aggregation
semantics before interpreting numbers across groups.

## 5. Observe object-store work at its actual boundary

**Task:** distinguish query execution from storage requests and returned payload consumption.

The callable entry point is
`instrument_object_store(store: Arc<dyn ObjectStore>, name: &str) -> Arc<dyn ObjectStore>`.
The returned wrapper is a separate integration object. It has to be the store used by the
consumer's storage path for its instrumentation to be observed.

| Boundary | Distinction to characterize |
|---|---|
| Method invocation/result | Request fields, result metadata and errors |
| Returned stream | Work performed when polled rather than when the handle is returned |
| Multipart handle | Work delegated through subsequent handle operations |
| Store registration | Which actual instance the query's URL/location selects |

These are separate characterization questions. A span around a method returning a stream does
not by itself prove coverage of payload transfer, every poll, or cancellation. Likewise, a
local test does not establish cloud retry, authentication or network behavior.

**Evidence.** [Wrapper implementation](../content/corpus/source/instrumented-object-store/instrumented_object_store.rs),
[store seam](../content/seams/object-store.md), and
[upstream scenarios](../content/index/scenarios.tsv). The current router's S008 citation belongs
to planning instrumentation and supplies no store assertion.

**Planned verification.** Wrapped/unwrapped operations over controlled local storage; compare
returned values/errors and captured metadata. Inspect and test lazy streams and multipart
methods individually where a brief makes a claim about them.

## 6. Compose subscriber, context and export lifetimes

**Task:** follow a query's spans into local output or an OpenTelemetry receiver.

| Component | Contract boundary |
|---|---|
| Instrumented future/attribute | Context active while async work is polled |
| Subscriber/registry/layers | Local observation, formatting, filtering and dispatch |
| OpenTelemetry bridge | Span/context translation into the selected SDK |
| SDK provider/processor | Resource, sampling, buffering, export scheduling and shutdown |
| OTLP exporter/receiver | Transport attempt, response, and downstream retention |

These components are alternatives or compositions according to the task. Local formatting
does not require an OTLP deployment. Output in a formatting layer does not prove output from
the bridge layer, and an export attempt does not prove collector retention.

**Compatibility.** The reference records `tracing-opentelemetry 0.32.0` and the OpenTelemetry
family at `0.31.0`. The historical manifest matrix records specific declared combinations.
It does not establish a timeless “minor plus one” rule or compile arbitrary consumer features.
The existing execution fixture does not include OpenTelemetry dependencies.

**Evidence.** [Compatibility catalog](../content/catalogs/compatibility.md),
[upstream OTLP example](../content/corpus/examples/otlp.rs), and
[consumer fixture manifest](../build/fixtures/probe-crate/Cargo.toml).
The upstream example is source evidence, not a local delivery receipt. Current Context7 guidance
on async instrumentation is a discovery lead described in the [evidence guide](evidence/README.md).

**Planned verification.** Recover full provider/processor lifecycle docs. Compile the selected
graph and features; capture async parent identity; observe an in-memory exporter; separately
qualify OTLP transport against a declared receiver when making delivery claims. Record shutdown
results and failed/partial delivery without generalizing to an untested backend.
