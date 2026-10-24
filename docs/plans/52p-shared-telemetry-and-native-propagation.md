---
title: "Plan 52p: Shared telemetry and native propagation"
status: in-progress
date: 2026-10-24
adrs: []
review_sources: [docs/functional_reviews/opentelemetry-and-agent-observability-2026-10-24.md]
doc_role: plan
doc_owner: docs/plans/52-agent-effectiveness-and-engineering-workflows.md
doc_topics: [current-work, design, libraries, diagnostics, environment, validation]
---

# Plan 52p: Shared telemetry and native propagation

## Target and authorization boundary

**Proposed:** compose Python and Rust telemetry once at their process hosts, propagate authentic
context across helper/PyO3/native boundaries, and explain real native stages and lifecycle through
selected spans and structured diagnostics. This develops the [review's O01–O04, O08 and O18](../functional_reviews/opentelemetry-and-agent-observability-2026-10-24.md#O01),
with source activation/configuration contributions to O10 and collection-policy contributions to O19/O20.
The [coordinator's opportunity dispositions](52-agent-effectiveness-and-engineering-workflows.md#opentelemetry-opportunity-dispositions)
own selection, package progress and remaining scope; this companion does not duplicate that ledger.

The maintainer accepted review RC01/RC02 for planning on 2026-10-24. The
[coordinator's rule decisions](52-agent-effectiveness-and-engineering-workflows.md#opentelemetry-rule-decisions)
own their full consequences and adoption routes. The selected target is a persistent native
Collector and OpenObserve OSS with one exact structured FastMCP interface. Authoring this plan
neither installs that pipeline nor authorizes production changes. Its deployment belongs to
[52q](52q-local-telemetry-pipeline-and-health.md); metrics belong to
[52r](52r-operational-metrics-and-diagnostic-integration.md); retrieval and investigations belong to
[52s](52s-telemetry-queries-and-agent-investigations.md).

[52f F02](52f-evidence-analysis-and-trace-correlation.md#F02) remains the sole implementation and
qualification owner for context across the bounded SDK's owned request task, transport HTTP header
injection, and actual Rust→database→Surrealism continuity/I11. F02 consumes TEL01/TEL03 rather
than this companion implementing a second SDK seam. Existing native admission, dispatch uncertainty,
cancellation/drain, result truth and scientific validity remain binding throughout.

## Current foundation and intended improvement

The authoring baseline is main `6e93c07c1` plus inspected current source. Concurrent Plan54
build/tooling edits are preserved; only their eventual observation consumers intersect this work.
The focused assessment applies Core3.4, Heuristics1.0, Test Assessment1.2 and relevant Process
Simulator1.5 constraints, especially truthful outcomes and owner lifetimes. It is plan preparation,
not a new diagnostic audit, completed implementation or subsystem qualification.

`pse-diagnostics::observation` already supplies operation identities, bounded copied observations,
coverage/loss, producer relations and checkpoints. `pse-runtime::tooling::capture::CaptureLayer`
is caller-composed and only reads a clock and attempts bounded enqueue from callbacks. Preserve
its finite investigation and native readers; ordinary export is an additional consumer, not a
replacement for its exact complete-record, admission or evidence-publication contracts.

`scripts/capabilities/observation.py` already associates interface requests with authentic native
readers and borrows the current Python span. `server.py::build` constructs FastMCP and owns a
specialist shutdown lifespan; `__main__.py` starts STDIO without SDK provider composition.
`process.py::run_worker` starts fixed child modules and shields spawn/wait through cancellation.
Those are useful composition and propagation seams. They must not become another scheduler or
surrender their existing process settlement behavior to a telemetry context manager.

Python API/SDK 1.45.1 is declared; Rust has `tracing` and subscriber/capture layers but no selected
OTel provider/export bootstrap. The review's Python SDK and disposable backend inquiries establish
only their named isolated behavior. Neither proves actual SDK network export, Collector delivery,
Rust propagation, process shutdown or scientific workflows. The implementation must inspect the
actual exact feature/dependency closure and selected server build before claiming those behaviors.

AP-01/AP-03 favor process-host composition over provider installation in every Runtime/tool.
AP-04/AP-05 require explicit operation/process/trace distinctions and inspectable source coverage.
AP-07 and H4/H17/H26 favor library batching plus coarse owner stages: no callback network writes,
per-scalar language crossings, duplicated provenance hydration or custom durable export queue.
Python and embedded Rust have distinct SDK state within the same OS process. Their resources must
identify that actual relationship without pretending they are independent process incarnations.

## Composition and semantic boundaries

TEL01 owns the shared mapping/configuration contract. Native operation, attempt, run, entity and
revision IDs retain their issuing authority. Trace/span IDs are diagnostic context, not operation
keys, cancellation handles or inputs to scientific hashes. Parentage records propagated causality;
links record explicit asynchronous/fan-in relationships; an associated native ID alone proves neither.
Process identity includes incarnation/start identity, service/version and relevant checkout/build
association. Instrumentation scope identifies the producing library/module and its version.

Keep exact investigation IDs in selected spans/logs/reference envelopes. Metrics consume TEL09's
bounded dimensions, not every request/case/path/trace ID. Do not invent a universal telemetry
ontology, mirror scientific records in Pydantic or make exportability a new admission requirement.
The existing Rust/native contracts govern scientific documents; host configuration and telemetry
metadata can use existing typed host mechanisms.

Select explicit programmatic SDK bootstrap as the baseline. It composes existing environment
selection with standard supported `OTEL_*` settings, SDK builders and a redacted effective-config
read. Per-signal configuration resolves once, before FastMCP construction or native work; process
hosts retain the provider handles needed for shutdown. API-only library instrumentation does not
install globals. Multiple Runtime instances share their host's composition rather than reinitialize it.

Experimental Python `opentelemetry-configuration`/`OTEL_CONFIG_FILE` is an optional alternative,
not an overlay. Reopen only if the explicit bootstrap develops material duplicated composition or
a required declarative feature has a clear consumer. Compare exact pinned behavior, environment
precedence, extension initialization and shutdown before substituting it; migrate the single source
and its effective-config consumer together. No parallel configuration framework is selected.

The initial Rust transport is OTLP HTTP/protobuf with a blocking-client-compatible export path,
using released API/SDK/OTLP 0.33.1 and `tracing-opentelemetry` 0.34.0 as the review's selected
implementation leads. `SdkTracerProvider::builder().with_batch_exporter(...)` uses a dedicated
thread processor. Qualify the resolved features; do not assume the default HTTP client remains
blocking when Cargo feature unification enables async clients. Select compatible features and,
where needed, an explicit blocking client through the exporter builder. Tonic remains an alternative
only with an owned live suitable Tokio runtime and tested shutdown placement. This choice concerns
telemetry export, not the existing application's gRPC database transport.

No new crate is demanded: instrument existing owners and compose SDK state at actual executable
and embedded-host boundaries. Exact dependency pins live in workspace/Python declarations, with
controlled lockfile changes. Ordinary additions need no ADR; genuine boundary/governance changes
follow the coordinator's scheduled decision route before dependent implementation.

## Work packages

<a id="TEL01"></a>
### TEL01 — Shared identity and configuration composition

Consume existing process/incarnation, invocation/operation and selected-service identities. Define
the minimal shared attribute/carrier mappings, signal enablement, filter/sampler selection and
provider lifetime contract used by Python, Rust, F02 and the pipeline/query companions. Mechanical
per-signal backend field normalization belongs to TEL13, not these source mappings.

Provide a cheap local effective-config/source-activation projection: actual SDK versions/scopes,
enabled signals, selected transport, redacted endpoints, resource association, sampling/detail
policy and lifecycle owner. Explain disabled, initialization-refused and unavailable states without
starting a backend, importing the scientific extension or claiming export delivery. TEL07 consumes
this evidence and adds actual pipeline observations. Never expose credentials through that read.

Route selected launch configuration through `scripts/pse_env.py` and existing fixed launch owners.
MCP STDIO's inherited environment allowlist and per-request context are separate concerns: explicitly
pass supported configuration without assuming arbitrary parent variables arrive. Decide environment
versus programmatic precedence once; error/fallback behavior remains visible to the host. Telemetry
failure must not rewrite native outcomes or prevent a cheap local diagnostic read.

Acceptance uses isolated composition tests with explicit conflicting base/signal settings, disabled
signals, repeated library/Runtime construction, redacted secrets and two real process incarnations.
An SDK scope change must not invent a process restart; a restart with the same service name must
change incarnation. Configuration inspection establishes configuration only, not query visibility.
Remove duplicate bootstrap/default interpretation at migrated consumers in the same change.

<a id="TEL02"></a>
### TEL02 — Python/FastMCP providers, helper carrier and logging lifecycle

At the fixed executable entry and fixed helper entry points, initialize owned `TracerProvider`,
`MeterProvider` and selected log-provider/exporter composition before instrumented work. Use
`BatchSpanProcessor`, OTLP exporters and periodic metric reader with actual transport deadlines;
keep API-only calls inside libraries. Keep `FASTMCP_TELEMETRY_MODE=native` for FastMCP4.1.3's
protocol/provider hierarchy. `propagation_only` or scoped suppression is justified only when a
specific alternative actually supplies the same overlapping spans; do not duplicate wrapper spans.

Extend the existing fixed helper launch with a bounded W3C `traceparent`/`tracestate` carrier,
separate from scientific request JSON and selected launch configuration. Extract, attach and detach
at the child owner with the standard Python propagator/context APIs. Validate size/syntax and
allowlisted keys; malformed or absent carriers produce honest missing continuity with safe local
diagnostics. Do not pass arbitrary baggage, environment contents, bindings or credentials.
FastMCP's AnyIO sync dispatch already preserves contextvars; no blanket threading instrumentor is
needed for that path. Assess actual HTTPX/RPC dependencies before selecting any instrumentor;
Python gRPC instrumentation does not cover the Rust tonic client.

Use an exact compatible `opentelemetry-instrumentation-logging` handler when selecting Python log
export, rather than growing the deprecated SDK `LoggingHandler` path. Map existing diagnostic
codes, owner stage, severity and native references explicitly. Choose one normal representation
for each event: span event or exported log. Retain stderr fallback, exclude exporter/transport
internal recursion and keep all diagnostics off MCP stdout.

Shutdown occurs after owned work has settled, with telemetry flush/shutdown separate from native
drain and result publication. Python1.45.1 shared batch `force_flush` ignores its timeout argument;
its stored export timeout is not a hard bound. Select real exporter deadlines and qualify slow/
refusing/unreachable export. A successful flush or SDK `processed` metric is not backend acceptance.
Do not let post-fork hooks imply exporter/native fork safety without the actual path being tested.

Acceptance includes a real fresh STDIO attachment and selected helper invocation, independent parent
ID assertions, repeated calls with no context leakage, request cancellation/disconnect with native
recovery reference, and refusal/outage shutdown. A local exporter fixture tests lifecycle semantics;
52h later qualifies this same consumer against the persistent pipeline. Retire superseded local
bootstrap/logging paths once consumers move, preserving useful bounded response-local references.

<a id="TEL03"></a>
### TEL03 — Rust provider/layer composition and native-boundary propagation

Compose `tracing_opentelemetry::OpenTelemetryLayer` with existing subscriber, selected
`CaptureLayer` and layer-specific filters. `OpenTelemetryTracingBridge` is the log bridge, not a
trace layer. Instrumented libraries accept/use API context and existing tracing facilities; they
do not install a global subscriber/provider per Runtime, tool, solver or thread.

At `crates/pse-py` and native child launch owners, carry the explicit bounded W3C carrier into a
new native operation span and return authentic native references. Preserve one actual OS process
incarnation when Python embeds Rust; separate instrumentation scopes/providers do not create a
second process identity. The PyO3 seam transfers diagnostic context, not scientific ownership.
Retain source-issued native child parent locators and distinguish their association from a span
parent. A child that outlives a host request has its own correct span lifetime and native handle.

Use `OpenTelemetrySpanExt::set_parent` before the span starts and handle its fallible result.
`OpenTelemetrySpanExt::context`, `tracing::Instrument` and, where OTel context owns the operation,
`FutureExt::with_context` supply task propagation. Never hold an attached thread-local guard across
`await`. Native threads receive/extract context explicitly; shared producer and independent waiter
relationships use actual producer ownership and links rather than multiple causal parents.

F02 consumes these context/provider APIs at its existing bounded SDK task and `Transport::call`
HTTP boundary. The review's inspected server `HttpTraceLayerHooks::make_span` already extracts
W3C headers behind common HTTP middleware, including gRPC. Do not add a proxy, encode trace
context in SurrealQL arguments or create a second RPC path here. Streaming-response lifetime and
the real server/module parent chain remain F02 acceptance, not a TEL03 fixture's inferred result.

Own provider shutdown outside a runtime thread needed for export. Test the resolved HTTP feature
closure and selected host/current-thread runtime cases; a builder compiling is insufficient.
Coarse spans/aggregation must keep network IO and blocking SDK shutdown out of native callbacks.
Acceptance covers Python→PyO3→Rust and native-child edges, task/thread hops, fallible parent
attachment, missing/malformed carrier, linked fan-in and operation continuation after caller end.
Use independently chosen parent/link expectations and actual fixed consumer paths, alongside small
in-memory tests. Keep finite capture independently functional with export disabled or failed.

<a id="TEL04"></a>
### TEL04 — Meaningful owner stages, lifecycle and structured errors

Add missing observations only where performing owners distinguish useful behavior: model
specialization; compiler lowering/materialization; mathematical preparation/rebind; evaluation;
solver/native dispatch; result validation/handling; and publication. Inspect existing spans,
`observation` phases/producers, `math.rs` preparation scopes and structured diagnostics first.
Reuse source-issued classifications; do not reinterpret them from a wrapper's elapsed time.

Project queued admission, reserved capacity, actual execution, producer sharing/waiting, response
handling, cancellation request, physical drain and settlement as distinct facts. A completed MCP
request or missing callback releases no native resource. Keep stable diagnostic codes, native
outcome and exact recovery reader references when span status is `Unset`, `Ok` or `Error`.
Avoid eagerly setting `Ok` before later owned work can fail: OTel gives `Ok` precedence.
Nonconvergence, requested cancellation and infrastructure failure retain their native meanings.

Inclusive phase intervals remain inclusive; overlapping waits/native joins cannot be summed as
exclusive CPU or called a hang. Selected CPU/detail readers answer those different questions.
Dense callback records retain the bounded native capture route; do not export one span/log per
scalar or hydrate full model provenance in a hot loop. TEL09/TEL10 consume owner facts for metrics
without deriving scientific populations from sampled spans.

Acceptance follows real edit/re-solve, shared preparation and selected study/solver operations
through their relevant owner stages, including nonconvergence, failure, cancellation and settled
recovery. Deliberately missing source coverage must remain unavailable rather than zero work.
Compare emitted owner stages/codes/references with original native readers. Delete replaced
redundant wrappers/parsers once migrated; retain native checkpoints/capture because their contracts
remain distinct. 52g consumes compact stage explanations; TEL14 exposes selected historical detail.

## Dependencies, verification and handoff

TEL01's implemented mappings/config projection unlock TEL02/TEL03 and 52q/TEL07; its settled
interface alone does not prove export. TEL02 and TEL03 may develop against local SDK exporters,
then compose at actual helper/PyO3 consumers. TEL04 can extend owner stages independently once
identity/context seams exist. F02 integrates TEL01/TEL03 with the SDK/server; TEL09 consumes
metrics-ready composition without requiring all stage coverage. Coordinate shared edits to
`server.py`, `process.py`, `observation.py`, `pse_env.py`, `crates/pse-py`, runtime and dependency
manifests before parallel implementation. Deployment setup belongs to 52q.

**Proposed verification:** compile touched Rust packages with `just check-package <pkg>` or
`just check` for actual cross-crate changes; run `just unit-package <pkg> <filter>` and targeted
script/Python tests through `scripts/pse-env`. Actual Arrow-consuming correctness closures retain
explicit force-validation. Dependency/skill-selection changes use `just ready` at their authorized
time. No static campaign, native compilation or service probe was run for this authoring.

After all selected functional scope is implemented, 52h/H02 owns assembled O20 qualification,
including real FastMCP→helper/PyO3→Rust→SDK→server→Surrealism continuity with F02, deliberate
broken edges, native cancellation/drain and pipeline outage. Scope-end integration/Python/native
journeys, `just hygiene`, governance/docs and relevant manual native/powerset checks follow the
repository rhythm. The coordinator runs `just turn-end` for changed-file turns. Every reported
check names command, mode, scope, zero failure baseline and actual result. Local passes do not
establish full-plan or scientific qualification; overhead/productivity claims require measurements.

## Source leads and remaining decisions

Use [review S01–S06](../functional_reviews/opentelemetry-and-agent-observability-2026-10-24.md#sources-and-concrete-implementation-leads)
for the exact SDK/server releases, propagator/status contracts and inspected boundaries. Concrete
Rust leads are [SDK0.33.1](https://docs.rs/opentelemetry_sdk/0.33.1/opentelemetry_sdk/),
[OTLP0.33.1](https://docs.rs/opentelemetry-otlp/0.33.1/opentelemetry_otlp/) and
[OpenTelemetrySpanExt0.34.0](https://docs.rs/tracing-opentelemetry/0.34.0/tracing_opentelemetry/trait.OpenTelemetrySpanExt.html).
The Python [v1.45.1 batch implementation](https://github.com/open-telemetry/opentelemetry-python/blob/v1.45.1/opentelemetry-sdk/src/opentelemetry/sdk/_shared_internal/__init__.py)
and [FastMCP4.1.3 telemetry owner](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/fastmcp_slim/fastmcp/server/telemetry.py)
are implementation-selection sources. Context7 orientation at authoring corroborated thread-based
Rust export and Python Views; its unversioned/main results do not replace exact-release evidence.

The target boundaries are settled for planning. Implementation must settle the concrete exporter/
logging companion pins, actual resolved HTTP client features, per-host shutdown placement and
bounded configuration/carrier limits before dependent integration. Reopen only the affected choice
if its tested closure cannot honor lifecycle or propagation; no alternative production path is
created by leaving a local detail open. Enduring §23.1/environment/capability meanings move through
the coordinator's authorized design route when adopted, with native owners retained.
